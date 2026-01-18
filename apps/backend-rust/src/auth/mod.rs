use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use rand::distributions::{Alphanumeric, DistString};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::db::NewUser;
use crate::AppState;

/// Cookie name for auth token
pub const AUTH_COOKIE_NAME: &str = "auth_token";

pub mod oauth;
pub mod passkey;

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

#[derive(Debug, Serialize, Clone)]
pub struct UserInfo {
    pub id: i64,
    pub email: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
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
        // Passkey endpoints
        .nest("/api/passkey", passkey::routes())
        // OAuth endpoints
        .nest("/api/oauth", oauth::routes())
        // Admin endpoints
        .route("/api/admin/users", get(admin_list_users))
        .route("/api/admin/users/{id}/admin", post(admin_set_user_admin))
        .route("/api/admin/users/{id}/verify", post(admin_verify_user))
        .route("/api/admin/users/{id}", axum::routing::delete(admin_delete_user))
        .route("/api/admin/packs", get(admin_list_packs))
        .route("/api/admin/packs", post(admin_create_pack))
        .route("/api/admin/packs/{id}", axum::routing::delete(admin_delete_pack))
        .route("/api/admin/puzzles", get(admin_list_puzzles))
        .route("/api/admin/puzzles", post(admin_add_puzzle))
        .route("/api/admin/puzzles/import", post(admin_import_puzzles))
        .route("/api/admin/puzzles/{id}", axum::routing::delete(admin_delete_puzzle))
        .route("/api/admin/rooms", get(admin_list_rooms))
        .route("/api/admin/rooms/{name}", axum::routing::delete(admin_delete_room))
        .route("/api/admin/settings/{room}", get(admin_get_settings))
        .route("/api/admin/settings/{room}", post(admin_save_settings))
}

// ========== LOGIN ENDPOINTS ==========

async fn api_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Response {
    // Get user by email
    let user = match state.db.get_user_by_email(&req.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Json(LoginResponse {
                ok: false,
                token: None,
                user: None,
                error: Some("Invalid email or password".to_string()),
            }).into_response();
        }
        Err(_) => {
            return Json(LoginResponse {
                ok: false,
                token: None,
                user: None,
                error: Some("Database error".to_string()),
            }).into_response();
        }
    };

    // Verify password
    let password_valid = match &user.password_hash {
        Some(hash) => verify_password(&req.password, hash),
        None => false, // OAuth-only user has no password
    };

    if !password_valid {
        return Json(LoginResponse {
            ok: false,
            token: None,
            user: None,
            error: Some("Invalid email or password".to_string()),
        }).into_response();
    }

    // Check if verified
    if !user.verified {
        return Json(LoginResponse {
            ok: false,
            token: None,
            user: None,
            error: Some("Please verify your email first".to_string()),
        }).into_response();
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
        }).into_response();
    }
    let _ = state.db.update_last_login(user.id).await;

    // Build response with Set-Cookie header
    let response = LoginResponse {
        ok: true,
        token: Some(token.clone()),
        user: Some(UserInfo {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            is_admin: if user.is_admin { Some(true) } else { None },
        }),
        error: None,
    };

    let mut res = Json(response).into_response();
    res.headers_mut().insert(
        header::SET_COOKIE,
        set_cookie_header(&token),
    );
    res
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Response {
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
                    is_admin: if user.is_admin { Some(true) } else { None },
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
) -> Response {
    // Clear token from database
    if let Some(token) = extract_auth_token(&headers) {
        if let Ok(Some(user)) = state.db.get_user_by_token(&token).await {
            let _ = state.db.clear_remember_token(user.id).await;
        }
    }

    // Build response that clears the cookie
    let response = SimpleResponse {
        ok: true,
        message: Some("Logged out".to_string()),
        error: None,
    };

    let mut res = Json(response).into_response();
    res.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&build_logout_cookie().to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("")),
    );
    res
}

async fn me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Json<VerifyResponse> {
    let token = match extract_auth_token(&headers) {
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
                is_admin: if user.is_admin { Some(true) } else { None },
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
    // Verify auth (supports both Bearer token and cookie)
    let token = match extract_auth_token(&headers) {
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

/// Extract auth token from Bearer header OR HttpOnly cookie
pub fn extract_auth_token(headers: &HeaderMap) -> Option<String> {
    // First try Bearer token (for API clients)
    if let Some(token) = extract_bearer_token(headers) {
        return Some(token);
    }
    // Fall back to cookie (for web client)
    extract_cookie_token(headers)
}

/// Extract token from Authorization: Bearer header
pub fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Extract token from auth cookie
pub fn extract_cookie_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let cookie = cookie.trim();
                cookie.strip_prefix(&format!("{}=", AUTH_COOKIE_NAME))
                    .map(|value| value.to_string())
            })
        })
}

/// Build an HttpOnly auth cookie
pub fn build_auth_cookie(token: &str, secure: bool) -> Cookie<'static> {
    let mut cookie = Cookie::build((AUTH_COOKIE_NAME, token.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(30));

    if secure {
        cookie = cookie.secure(true);
    }

    cookie.build()
}

/// Build a cookie that clears the auth token
pub fn build_logout_cookie() -> Cookie<'static> {
    Cookie::build((AUTH_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::seconds(0))
        .build()
}

/// Create Set-Cookie header value
pub fn set_cookie_header(token: &str) -> HeaderValue {
    let secure = std::env::var("SSL_ENABLED")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);
    let cookie = build_auth_cookie(token, secure);
    HeaderValue::from_str(&cookie.to_string()).unwrap_or_else(|_| HeaderValue::from_static(""))
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

// ========== ADMIN HELPER ==========

async fn get_admin_user(state: &Arc<AppState>, headers: &HeaderMap) -> Option<crate::db::User> {
    let token = extract_auth_token(headers)?;
    let user = state.db.get_user_by_token(&token).await.ok()??;
    if user.is_admin {
        Some(user)
    } else {
        None
    }
}

// ========== ADMIN ENDPOINTS ==========

#[derive(Serialize)]
struct AdminUsersResponse {
    ok: bool,
    users: Option<Vec<AdminUserInfo>>,
    error: Option<String>,
}

#[derive(Serialize)]
struct AdminUserInfo {
    id: i64,
    email: String,
    display_name: String,
    verified: bool,
    is_admin: bool,
    created_at: i64,
    last_login_at: Option<i64>,
}

async fn admin_list_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<AdminUsersResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(AdminUsersResponse {
            ok: false, users: None, error: Some("Admin access required".to_string())
        }));
    }

    match state.db.list_all_users().await {
        Ok(users) => (StatusCode::OK, Json(AdminUsersResponse {
            ok: true,
            users: Some(users.into_iter().map(|u| AdminUserInfo {
                id: u.id,
                email: u.email,
                display_name: u.display_name,
                verified: u.verified,
                is_admin: u.is_admin,
                created_at: u.created_at,
                last_login_at: u.last_login_at,
            }).collect()),
            error: None,
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminUsersResponse {
            ok: false, users: None, error: Some("Database error".to_string())
        })),
    }
}

#[derive(Deserialize)]
struct SetAdminRequest {
    is_admin: bool,
}

async fn admin_set_user_admin(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
    Json(req): Json<SetAdminRequest>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    match state.db.set_user_admin(user_id, req.is_admin).await {
        Ok(()) => (StatusCode::OK, Json(SimpleResponse {
            ok: true, message: Some("User admin status updated".to_string()), error: None
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(SimpleResponse {
            ok: false, message: None, error: Some("Failed to update user".to_string())
        })),
    }
}

async fn admin_verify_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    match state.db.verify_user(user_id).await {
        Ok(()) => (StatusCode::OK, Json(SimpleResponse {
            ok: true, message: Some("User verified".to_string()), error: None
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(SimpleResponse {
            ok: false, message: None, error: Some("Failed to verify user".to_string())
        })),
    }
}

async fn admin_delete_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    match state.db.delete_user(user_id).await {
        Ok(true) => (StatusCode::OK, Json(SimpleResponse {
            ok: true, message: Some("User deleted".to_string()), error: None
        })),
        Ok(false) => (StatusCode::NOT_FOUND, Json(SimpleResponse {
            ok: false, message: None, error: Some("User not found".to_string())
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(SimpleResponse {
            ok: false, message: None, error: Some("Failed to delete user".to_string())
        })),
    }
}

#[derive(Serialize)]
struct AdminPacksResponse {
    ok: bool,
    packs: Option<Vec<PackInfo>>,
    error: Option<String>,
}

#[derive(Serialize)]
struct PackInfo {
    id: i64,
    name: String,
    puzzle_count: i64,
}

async fn admin_list_packs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<AdminPacksResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(AdminPacksResponse {
            ok: false, packs: None, error: Some("Admin access required".to_string())
        }));
    }

    match state.db.get_puzzle_counts().await {
        Ok(counts) => (StatusCode::OK, Json(AdminPacksResponse {
            ok: true,
            packs: Some(counts.into_iter().map(|(id, name, count)| PackInfo {
                id, name, puzzle_count: count
            }).collect()),
            error: None,
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminPacksResponse {
            ok: false, packs: None, error: Some("Database error".to_string())
        })),
    }
}

#[derive(Deserialize)]
struct CreatePackRequest {
    name: String,
}

async fn admin_create_pack(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreatePackRequest>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    match state.db.get_or_create_pack(&req.name).await {
        Ok(id) => (StatusCode::OK, Json(SimpleResponse {
            ok: true, message: Some(format!("Pack created with ID {}", id)), error: None
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(SimpleResponse {
            ok: false, message: None, error: Some("Failed to create pack".to_string())
        })),
    }
}

async fn admin_delete_pack(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(pack_id): Path<i64>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    if pack_id == 1 {
        return (StatusCode::BAD_REQUEST, Json(SimpleResponse {
            ok: false, message: None, error: Some("Cannot delete default pack".to_string())
        }));
    }

    match state.db.delete_pack(pack_id).await {
        Ok(true) => (StatusCode::OK, Json(SimpleResponse {
            ok: true, message: Some("Pack deleted".to_string()), error: None
        })),
        Ok(false) => (StatusCode::NOT_FOUND, Json(SimpleResponse {
            ok: false, message: None, error: Some("Pack not found".to_string())
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(SimpleResponse {
            ok: false, message: None, error: Some("Failed to delete pack".to_string())
        })),
    }
}

#[derive(Serialize)]
struct AdminPuzzlesResponse {
    ok: bool,
    puzzles: Option<Vec<PuzzleInfo>>,
    error: Option<String>,
}

#[derive(Serialize)]
struct PuzzleInfo {
    id: i64,
    category: String,
    answer: String,
    pack_id: i64,
    enabled: bool,
}

#[derive(Deserialize)]
struct ListPuzzlesQuery {
    pack_id: Option<i64>,
}

async fn admin_list_puzzles(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<ListPuzzlesQuery>,
) -> (StatusCode, Json<AdminPuzzlesResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(AdminPuzzlesResponse {
            ok: false, puzzles: None, error: Some("Admin access required".to_string())
        }));
    }

    match state.db.list_all_puzzles(query.pack_id).await {
        Ok(puzzles) => (StatusCode::OK, Json(AdminPuzzlesResponse {
            ok: true,
            puzzles: Some(puzzles.into_iter().map(|p| PuzzleInfo {
                id: p.id,
                category: p.category,
                answer: p.answer,
                pack_id: p.pack_id,
                enabled: p.enabled,
            }).collect()),
            error: None,
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminPuzzlesResponse {
            ok: false, puzzles: None, error: Some("Database error".to_string())
        })),
    }
}

#[derive(Deserialize)]
struct AddPuzzleRequest {
    category: String,
    answer: String,
    pack_id: i64,
}

async fn admin_add_puzzle(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddPuzzleRequest>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    match state.db.add_puzzle(&req.category, &req.answer, req.pack_id).await {
        Ok(id) => (StatusCode::OK, Json(SimpleResponse {
            ok: true, message: Some(format!("Puzzle added with ID {}", id)), error: None
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(SimpleResponse {
            ok: false, message: None, error: Some("Failed to add puzzle".to_string())
        })),
    }
}

#[derive(Deserialize)]
struct ImportPuzzleItem {
    category: String,
    answer: String,
}

#[derive(Deserialize)]
struct ImportPuzzlesRequest {
    puzzles: Vec<ImportPuzzleItem>,
    pack_id: Option<i64>,
    pack_name: Option<String>,
}

async fn admin_import_puzzles(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ImportPuzzlesRequest>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    // Determine pack_id: use provided pack_id, or create/get pack by name, or default to 1
    let pack_id = if let Some(id) = req.pack_id {
        id
    } else if let Some(name) = req.pack_name {
        match state.db.get_or_create_pack(&name).await {
            Ok(id) => id,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(SimpleResponse {
                ok: false, message: None, error: Some("Failed to create pack".to_string())
            })),
        }
    } else {
        1 // Default pack
    };

    // Convert to tuple format for import
    let puzzles: Vec<(String, String)> = req.puzzles
        .into_iter()
        .map(|p| (p.category, p.answer))
        .collect();

    let count = puzzles.len();
    match state.db.import_puzzles(puzzles, pack_id).await {
        Ok(imported) => (StatusCode::OK, Json(SimpleResponse {
            ok: true,
            message: Some(format!("Imported {} puzzles into pack {}", imported, pack_id)),
            error: None
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(SimpleResponse {
            ok: false,
            message: Some(format!("Failed after importing some of {} puzzles", count)),
            error: Some("Import failed".to_string())
        })),
    }
}

async fn admin_delete_puzzle(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(puzzle_id): Path<i64>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    match state.db.delete_puzzle(puzzle_id).await {
        Ok(true) => (StatusCode::OK, Json(SimpleResponse {
            ok: true, message: Some("Puzzle deleted".to_string()), error: None
        })),
        Ok(false) => (StatusCode::NOT_FOUND, Json(SimpleResponse {
            ok: false, message: None, error: Some("Puzzle not found".to_string())
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(SimpleResponse {
            ok: false, message: None, error: Some("Failed to delete puzzle".to_string())
        })),
    }
}

#[derive(Serialize)]
struct AdminRoomsResponse {
    ok: bool,
    rooms: Option<Vec<AdminRoomInfo>>,
    error: Option<String>,
}

#[derive(Serialize)]
struct AdminRoomInfo {
    name: String,
    player_count: usize,
    phase: String,
    has_host: bool,
}

async fn admin_list_rooms(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<AdminRoomsResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(AdminRoomsResponse {
            ok: false, rooms: None, error: Some("Admin access required".to_string())
        }));
    }

    let manager = state.game_manager.read().await;
    let rooms: Vec<AdminRoomInfo> = manager.rooms.iter().map(|(name, game)| {
        AdminRoomInfo {
            name: name.clone(),
            player_count: game.players.len(),
            phase: format!("{:?}", game.phase),
            has_host: game.host_sid.is_some(),
        }
    }).collect();

    (StatusCode::OK, Json(AdminRoomsResponse {
        ok: true, rooms: Some(rooms), error: None
    }))
}

async fn admin_delete_room(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(room_name): Path<String>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    // Remove from game manager
    {
        let mut manager = state.game_manager.write().await;
        manager.rooms.remove(&room_name);
    }

    // Remove from database
    let _ = state.db.delete_room(&room_name).await;

    (StatusCode::OK, Json(SimpleResponse {
        ok: true, message: Some("Room deleted".to_string()), error: None
    }))
}

// ========== SETTINGS ENDPOINTS ==========

#[derive(Serialize)]
struct SettingsResponse {
    ok: bool,
    config: Option<crate::game::RoomConfig>,
    error: Option<String>,
}

async fn admin_get_settings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(room_name): Path<String>,
) -> (StatusCode, Json<SettingsResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SettingsResponse {
            ok: false, config: None, error: Some("Admin access required".to_string())
        }));
    }

    match state.db.get_room_config(&room_name).await {
        Ok(config) => (StatusCode::OK, Json(SettingsResponse {
            ok: true, config: Some(config), error: None
        })),
        Err(_) => (StatusCode::OK, Json(SettingsResponse {
            ok: true, config: Some(crate::game::RoomConfig::default()), error: None
        })),
    }
}

#[derive(Deserialize)]
struct SaveSettingsRequest {
    puzzle_display_seconds: Option<i32>,
    vowel_cost: Option<i32>,
    final_seconds: Option<i32>,
    final_jackpot: Option<i32>,
    prize_wedge_names: Option<Vec<String>>,
}

async fn admin_save_settings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(room_name): Path<String>,
    Json(req): Json<SaveSettingsRequest>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    // Get existing config and merge with new values
    let existing = state.db.get_room_config(&room_name).await.unwrap_or_default();
    let config = crate::game::RoomConfig {
        vowel_cost: req.vowel_cost.unwrap_or(existing.vowel_cost),
        final_seconds: req.final_seconds.unwrap_or(existing.final_seconds),
        final_jackpot: req.final_jackpot.unwrap_or(existing.final_jackpot),
        prize_replace_cash_values: existing.prize_replace_cash_values,
        puzzle_display_seconds: req.puzzle_display_seconds.unwrap_or(existing.puzzle_display_seconds),
        prize_wedge_names: req.prize_wedge_names.unwrap_or(existing.prize_wedge_names),
    };

    // Get existing pack ID
    let pack_id = state.db.get_active_pack_id(&room_name).await.ok().flatten();

    match state.db.set_room_config(&room_name, &config, pack_id).await {
        Ok(_) => {
            // Also update in-memory game state if room exists
            {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.rooms.get_mut(&room_name) {
                    game.config = config;
                }
            }
            (StatusCode::OK, Json(SimpleResponse {
                ok: true, message: Some("Settings saved".to_string()), error: None
            }))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(SimpleResponse {
            ok: false, message: None, error: Some(format!("Failed to save settings: {}", e))
        })),
    }
}
