use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use rand::distributions::{Alphanumeric, DistString};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::db::NewUser;
use crate::AppState;

// ========== REQUEST/RESPONSE TYPES ==========

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub captcha_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SimpleResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RoomsResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rooms: Option<Vec<RoomInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RoomInfo {
    pub name: String,
    pub player_count: usize,
    pub total_slots: usize,
}

// ========== ROUTES ==========

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // Auth endpoints
        .route("/api/login", post(api_login))
        .route("/login", post(login))
        .route("/api/register", post(api_register))
        .route("/register", post(register))
        .route("/api/verify", get(api_verify_token))
        .route("/verify/{token}", get(verify_email))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/api/rooms", get(api_rooms))
        .route("/rooms", get(list_rooms))
}

// ========== LOGIN ENDPOINTS ==========

async fn api_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Json<LoginResponse> {
    // Get user by email
    let user = match state.db.get_user_by_email(&req.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Json(LoginResponse {
                ok: false,
                token: None,
                user: None,
                error: Some("Invalid email or password".to_string()),
            });
        }
        Err(_) => {
            return Json(LoginResponse {
                ok: false,
                token: None,
                user: None,
                error: Some("Database error".to_string()),
            });
        }
    };

    // Verify password
    if !verify_password(&req.password, &user.password_hash) {
        return Json(LoginResponse {
            ok: false,
            token: None,
            user: None,
            error: Some("Invalid email or password".to_string()),
        });
    }

    // Check if verified
    if !user.verified {
        return Json(LoginResponse {
            ok: false,
            token: None,
            user: None,
            error: Some("Please verify your email first".to_string()),
        });
    }

    // Generate new token
    let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);

    // Save token and update last login
    if state.db.set_remember_token(user.id, &token).await.is_err() {
        return Json(LoginResponse {
            ok: false,
            token: None,
            user: None,
            error: Some("Failed to generate token".to_string()),
        });
    }
    let _ = state.db.update_last_login(user.id).await;

    Json(LoginResponse {
        ok: true,
        token: Some(token),
        user: Some(UserInfo {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
        }),
        error: None,
    })
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Json<LoginResponse> {
    api_login(State(state), Json(req)).await
}

// ========== REGISTRATION ENDPOINTS ==========

async fn api_register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> (StatusCode, Json<RegisterResponse>) {
    register_user(state, req).await
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> (StatusCode, Json<RegisterResponse>) {
    register_user(state, req).await
}

async fn register_user(
    state: Arc<AppState>,
    req: RegisterRequest,
) -> (StatusCode, Json<RegisterResponse>) {
    let mut errors = Vec::new();

    // Validate email
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    let email = req.email.trim().to_lowercase();
    if email.is_empty() || !email_regex.is_match(&email) {
        errors.push("Valid email is required".to_string());
    }

    // Validate password
    if req.password.len() < 8 {
        errors.push("Password must be at least 8 characters".to_string());
    }

    // Validate display name
    let display_name = req.display_name.trim();
    if display_name.len() < 2 {
        errors.push("Display name must be at least 2 characters".to_string());
    }
    if display_name.len() > 24 {
        errors.push("Display name must be 24 characters or less".to_string());
    }

    if !errors.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(RegisterResponse {
                ok: false,
                message: None,
                errors: Some(errors),
            }),
        );
    }

    // Check if user exists
    match state.db.user_exists(&email).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(RegisterResponse {
                    ok: false,
                    message: None,
                    errors: Some(vec!["Email already registered".to_string()]),
                }),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    ok: false,
                    message: None,
                    errors: Some(vec!["Database error".to_string()]),
                }),
            );
        }
        Ok(false) => {}
    }

    // Hash password
    let password_hash = match hash_password(&req.password) {
        Some(h) => h,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    ok: false,
                    message: None,
                    errors: Some(vec!["Failed to hash password".to_string()]),
                }),
            );
        }
    };

    // Generate verification token
    let verification_token = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    let token_expires = now_secs() + 86400; // 24 hours

    // Create user
    let new_user = NewUser {
        email: email.clone(),
        password_hash,
        display_name: display_name.to_string(),
        verification_token: verification_token.clone(),
        verification_token_expires: token_expires,
    };

    match state.db.create_user(new_user).await {
        Ok(_) => {
            // Send verification email
            if let Err(e) = state
                .email
                .send_verification_email(&email, &verification_token)
                .await
            {
                tracing::warn!("Failed to send verification email: {}", e);
                // Don't fail registration if email fails - user can request resend
            }

            (
                StatusCode::OK,
                Json(RegisterResponse {
                    ok: true,
                    message: Some(
                        "Registration successful! Please check your email to verify your account."
                            .to_string(),
                    ),
                    errors: None,
                }),
            )
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RegisterResponse {
                ok: false,
                message: None,
                errors: Some(vec!["Failed to create account".to_string()]),
            }),
        ),
    }
}

// ========== VERIFICATION ENDPOINTS ==========

async fn api_verify_token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<VerifyResponse>) {
    let token = match extract_bearer_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(VerifyResponse {
                    ok: false,
                    user: None,
                    error: Some("No token provided".to_string()),
                }),
            );
        }
    };

    match state.db.get_user_by_token(&token).await {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(VerifyResponse {
                ok: true,
                user: Some(UserInfo {
                    id: user.id,
                    email: user.email,
                    display_name: user.display_name,
                }),
                error: None,
            }),
        ),
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            Json(VerifyResponse {
                ok: false,
                user: None,
                error: Some("Invalid token".to_string()),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(VerifyResponse {
                ok: false,
                user: None,
                error: Some("Database error".to_string()),
            }),
        ),
    }
}

async fn verify_email(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> (StatusCode, Json<SimpleResponse>) {
    let user = match state.db.get_user_by_verification_token(&token).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(SimpleResponse {
                    ok: false,
                    message: None,
                    error: Some("Invalid or expired verification link".to_string()),
                }),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SimpleResponse {
                    ok: false,
                    message: None,
                    error: Some("Database error".to_string()),
                }),
            );
        }
    };

    // Check if token expired
    if let Some(expires) = user.verification_token_expires {
        if expires < now_secs() {
            return (
                StatusCode::BAD_REQUEST,
                Json(SimpleResponse {
                    ok: false,
                    message: None,
                    error: Some("Verification link has expired. Please register again.".to_string()),
                }),
            );
        }
    }

    // Verify the user
    if state.db.verify_user(user.id).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SimpleResponse {
                ok: false,
                message: None,
                error: Some("Failed to verify user".to_string()),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(SimpleResponse {
            ok: true,
            message: Some("Email verified successfully! You can now log in.".to_string()),
            error: None,
        }),
    )
}

// ========== OTHER AUTH ENDPOINTS ==========

async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Json<SimpleResponse> {
    if let Some(token) = extract_bearer_token(&headers) {
        if let Ok(Some(user)) = state.db.get_user_by_token(&token).await {
            let _ = state.db.clear_remember_token(user.id).await;
        }
    }

    Json(SimpleResponse {
        ok: true,
        message: Some("Logged out".to_string()),
        error: None,
    })
}

async fn me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Json<VerifyResponse> {
    let token = match extract_bearer_token(&headers) {
        Some(t) => t,
        None => {
            return Json(VerifyResponse {
                ok: false,
                user: None,
                error: None,
            });
        }
    };

    match state.db.get_user_by_token(&token).await {
        Ok(Some(user)) => Json(VerifyResponse {
            ok: true,
            user: Some(UserInfo {
                id: user.id,
                email: user.email,
                display_name: user.display_name,
            }),
            error: None,
        }),
        _ => Json(VerifyResponse {
            ok: false,
            user: None,
            error: None,
        }),
    }
}

async fn api_rooms(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<RoomsResponse>) {
    // Verify auth
    let token = match extract_bearer_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(RoomsResponse {
                    ok: false,
                    rooms: None,
                    error: Some("No token provided".to_string()),
                }),
            );
        }
    };

    if state.db.get_user_by_token(&token).await.ok().flatten().is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(RoomsResponse {
                ok: false,
                rooms: None,
                error: Some("Invalid token".to_string()),
            }),
        );
    }

    // Get rooms from game manager
    let manager = state.game_manager.read().await;
    let rooms: Vec<RoomInfo> = manager
        .list_rooms()
        .into_iter()
        .map(|r| RoomInfo {
            name: r.name,
            player_count: r.player_count,
            total_slots: 8,
        })
        .collect();

    (
        StatusCode::OK,
        Json(RoomsResponse {
            ok: true,
            rooms: Some(rooms),
            error: None,
        }),
    )
}

async fn list_rooms(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<RoomsResponse>) {
    api_rooms(State(state), headers).await
}

// ========== HELPER FUNCTIONS ==========

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn hash_password(password: &str) -> Option<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .ok()
        .map(|h| h.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};

    // Handle different hash formats
    if hash.starts_with("$argon2") {
        // Argon2 hash
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    } else if hash.starts_with("pbkdf2:") {
        // Werkzeug PBKDF2 hash (from Python) - for compatibility
        verify_werkzeug_hash(password, hash)
    } else {
        false
    }
}

fn verify_werkzeug_hash(password: &str, hash: &str) -> bool {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;

    // Parse hash: pbkdf2:sha256:iterations$salt$hash
    let parts: Vec<&str> = hash.split('$').collect();
    if parts.len() != 3 {
        return false;
    }

    let method_parts: Vec<&str> = parts[0].split(':').collect();
    if method_parts.len() < 3 || method_parts[0] != "pbkdf2" {
        return false;
    }

    let iterations: u32 = match method_parts[2].parse() {
        Ok(i) => i,
        Err(_) => return false,
    };

    let salt = parts[1];
    let expected_hash = parts[2];

    // Compute hash
    let mut result = vec![0u8; expected_hash.len() / 2];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt.as_bytes(), iterations, &mut result);

    // Compare
    let computed = hex::encode(&result);
    computed == expected_hash
}
