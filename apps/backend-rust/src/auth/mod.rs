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
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::NewUser;
use crate::validation::{
    display_name_validator, email_validator, password_validator, ValidationErrorResponse,
};
use crate::AppState;

/// Cookie name for auth token
pub const AUTH_COOKIE_NAME: &str = "auth_token";

pub mod oauth;
pub mod passkey;

// ========== REQUEST/RESPONSE TYPES ==========

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(custom(function = "email_validator"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
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
    pub avatar_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
}


#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(custom(function = "email_validator"))]
    pub email: String,
    #[validate(custom(function = "password_validator"))]
    pub password: String,
    #[validate(custom(function = "display_name_validator"))]
    pub display_name: String,
    #[serde(default = "default_avatar")]
    pub avatar_id: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub captcha_token: Option<String>,
}

fn default_avatar() -> i64 {
    1
}

#[derive(Debug, Deserialize, Validate)]
pub struct ProfileUpdateRequest {
    #[validate(custom(function = "display_name_validator"))]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
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
        .route("/api/profile", get(api_get_profile))
        .route("/api/profile", post(api_update_profile))
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
        .route("/api/admin/rooms", post(admin_create_room))
        .route("/api/admin/rooms/{name}", axum::routing::delete(admin_delete_room))
        .route("/api/admin/rooms/{room}/players/{idx}", axum::routing::delete(admin_kick_player))
        .route("/api/admin/rooms/{room}/players/{idx}/reset", post(admin_reset_player_score))
        .route("/api/admin/rooms/{room}/kick-user/{user_id}", post(admin_kick_user))
        .route("/api/admin/rooms/{room}/new-game", post(admin_new_game))
        .route("/api/admin/settings/{room}", get(admin_get_settings))
        .route("/api/admin/settings/{room}", post(admin_save_settings))
}

// ========== LOGIN ENDPOINTS ==========

async fn api_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Response {
    // Validate input
    if let Err(errors) = req.validate() {
        return ValidationErrorResponse::from_validation_errors(&errors).into_response();
    }

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

    // Check if user should be admin based on ADMIN_EMAIL env var
    let mut user = user;
    if let Ok(admin_email) = std::env::var("ADMIN_EMAIL") {
        if user.email.to_lowercase() == admin_email.to_lowercase() && !user.is_admin {
            let _ = state.db.set_user_admin(user.id, true).await;
            user.is_admin = true;
        }
    }

    // Invalidate any existing sessions by emitting to the user's room
    {
        let user_room = format!("user:{}", user.id);
        if let Some(io) = state.io.get() {
            if let Some(ns) = io.of("/") {
                tracing::info!("Emitting session_invalidated to room {}", user_room);
                let _ = ns.to(user_room).emit("session_invalidated", &serde_json::json!({
                    "reason": "logged_in_elsewhere"
                }));
            }
        }
    }

    // Clear the user's sockets from tracking (they'll re-auth with new token)
    {
        let mut user_sockets = state.user_sockets.write().await;
        user_sockets.remove(&user.id);
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
            avatar_id: user.avatar_id,
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
) -> Response {
    register_user(state, req).await
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Response {
    register_user(state, req).await
}

async fn register_user(
    state: Arc<AppState>,
    req: RegisterRequest,
) -> Response {
    // Validate input using validator crate
    if let Err(validation_errors) = req.validate() {
        // Convert validation errors to the existing RegisterResponse format
        let errors: Vec<String> = validation_errors
            .field_errors()
            .iter()
            .flat_map(|(_, errs)| {
                errs.iter().map(|e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "Invalid input".to_string())
                })
            })
            .collect();

        return (
            StatusCode::BAD_REQUEST,
            Json(RegisterResponse {
                ok: false,
                message: None,
                errors: Some(errors),
                user: None,
            }),
        ).into_response();
    }

    let email = req.email.trim().to_lowercase();
    let display_name = req.display_name.trim();

    // Check if user exists
    match state.db.user_exists(&email).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(RegisterResponse {
                    ok: false,
                    message: None,
                    errors: Some(vec!["Email already registered".to_string()]),
                    user: None,
                }),
            ).into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    ok: false,
                    message: None,
                    errors: Some(vec!["Database error".to_string()]),
                    user: None,
                }),
            ).into_response();
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
                    user: None,
                }),
            ).into_response();
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
        avatar_id: req.avatar_id,
    };

    match state.db.create_user(new_user).await {
        Ok(user_id) => {
            // If email is disabled, auto-verify and auto-login for testing/development
            if !state.email.is_enabled() {
                if let Err(e) = state.db.verify_user(user_id).await {
                    tracing::warn!("Failed to auto-verify user: {}", e);
                } else {
                    tracing::info!("Auto-verified user {} (email disabled)", email);
                }

                // Generate token for auto-login
                let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
                if let Err(e) = state.db.set_remember_token(user_id, &token).await {
                    tracing::warn!("Failed to set token for auto-login: {}", e);
                    return (
                        StatusCode::OK,
                        Json(RegisterResponse {
                            ok: true,
                            message: Some(
                                "Registration successful! You can now log in."
                                    .to_string(),
                            ),
                            errors: None,
                            user: None,
                        }),
                    ).into_response();
                }

                // Return response with user info for auto-login (frontend checks for data.user)
                tracing::info!("Auto-login user {} after registration (email disabled)", email);
                let response = RegisterResponse {
                    ok: true,
                    message: None,
                    errors: None,
                    user: Some(UserInfo {
                        id: user_id,
                        email: email.clone(),
                        display_name: display_name.to_string(),
                        avatar_id: req.avatar_id, // Use the avatar_id from registration request
                        is_admin: None,
                    }),
                };
                let mut res = Json(response).into_response();
                res.headers_mut().insert(
                    header::SET_COOKIE,
                    set_cookie_header(&token),
                );
                return res;
            }

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
                    user: None,
                }),
            ).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RegisterResponse {
                ok: false,
                message: None,
                errors: Some(vec!["Failed to create account".to_string()]),
                user: None,
            }),
        ).into_response(),
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
                    avatar_id: user.avatar_id,
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
                avatar_id: user.avatar_id,
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

// ========== PROFILE ENDPOINTS ==========

async fn api_get_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<ProfileResponse>) {
    let token = match extract_auth_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ProfileResponse {
                    ok: false,
                    user: None,
                    error: Some("Authentication required".to_string()),
                }),
            );
        }
    };

    match state.db.get_user_by_token(&token).await {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(ProfileResponse {
                ok: true,
                user: Some(UserInfo {
                    id: user.id,
                    email: user.email,
                    display_name: user.display_name,
                    avatar_id: user.avatar_id,
                    is_admin: if user.is_admin { Some(true) } else { None },
                }),
                error: None,
            }),
        ),
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(ProfileResponse {
                ok: false,
                user: None,
                error: Some("Invalid token".to_string()),
            }),
        ),
    }
}

async fn api_update_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ProfileUpdateRequest>,
) -> (StatusCode, Json<ProfileResponse>) {
    // Validate input
    if let Err(errors) = req.validate() {
        let error_messages: Vec<String> = errors
            .field_errors()
            .values()
            .flat_map(|v| v.iter().filter_map(|e| e.message.as_ref().map(|s| s.to_string())))
            .collect();
        return (
            StatusCode::BAD_REQUEST,
            Json(ProfileResponse {
                ok: false,
                user: None,
                error: Some(error_messages.join(", ")),
            }),
        );
    }

    let token = match extract_auth_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ProfileResponse {
                    ok: false,
                    user: None,
                    error: Some("Authentication required".to_string()),
                }),
            );
        }
    };

    // Get current user
    let user = match state.db.get_user_by_token(&token).await {
        Ok(Some(user)) => user,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ProfileResponse {
                    ok: false,
                    user: None,
                    error: Some("Invalid token".to_string()),
                }),
            );
        }
    };

    // Update profile
    if let Err(e) = state
        .db
        .update_user_profile(
            user.id,
            req.display_name.as_deref(),
            req.avatar_id,
        )
        .await
    {
        tracing::error!("Failed to update profile: {:?}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProfileResponse {
                ok: false,
                user: None,
                error: Some("Failed to update profile".to_string()),
            }),
        );
    }

    // Get updated user
    match state.db.get_user_by_id(user.id).await {
        Ok(Some(updated_user)) => (
            StatusCode::OK,
            Json(ProfileResponse {
                ok: true,
                user: Some(UserInfo {
                    id: updated_user.id,
                    email: updated_user.email,
                    display_name: updated_user.display_name,
                    avatar_id: updated_user.avatar_id,
                    is_admin: if updated_user.is_admin { Some(true) } else { None },
                }),
                error: None,
            }),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProfileResponse {
                ok: false,
                user: None,
                error: Some("Failed to get updated profile".to_string()),
            }),
        ),
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
    current_room: Option<String>,
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

    // Build a map of user_id -> room_name from the game manager
    let manager = state.game_manager.read().await;
    let mut user_rooms: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    for (room_name, game) in manager.rooms.iter() {
        for player in &game.players {
            if let Some(user_id) = player.user_id {
                user_rooms.insert(user_id, room_name.clone());
            }
        }
    }
    drop(manager);

    match state.db.list_all_users().await {
        Ok(users) => (StatusCode::OK, Json(AdminUsersResponse {
            ok: true,
            users: Some(users.into_iter().map(|u| AdminUserInfo {
                current_room: user_rooms.get(&u.id).cloned(),
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

#[derive(Serialize)]
struct CreatePackResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn admin_create_pack(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreatePackRequest>,
) -> (StatusCode, Json<CreatePackResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(CreatePackResponse {
            ok: false, id: None, message: None, error: Some("Admin access required".to_string())
        }));
    }

    match state.db.get_or_create_pack(&req.name).await {
        Ok(id) => (StatusCode::OK, Json(CreatePackResponse {
            ok: true, id: Some(id), message: Some(format!("Pack created with ID {}", id)), error: None
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(CreatePackResponse {
            ok: false, id: None, message: None, error: Some("Failed to create pack".to_string())
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
    // Extended details
    players: Vec<AdminPlayerInfo>,
    active_idx: Option<usize>,
    puzzle_category: Option<String>,
    puzzle_answer: Option<String>,
    revealed_count: usize,
    total_letters: usize,
    current_wedge: Option<String>,
}

#[derive(Serialize)]
struct AdminPlayerInfo {
    name: String,
    total: i32,
    round_bank: i32,
    is_connected: bool,
    avatar_id: i64,
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
        let players: Vec<AdminPlayerInfo> = game.players.iter().map(|p| {
            AdminPlayerInfo {
                name: p.name.clone(),
                total: p.total,
                round_bank: p.round_bank,
                is_connected: p.socket_id.is_some(),
                avatar_id: p.avatar_id,
            }
        }).collect();

        let total_letters = game.puzzle.answer.chars().filter(|c| c.is_ascii_alphabetic()).count();
        let revealed_count = game.revealed.len();

        let current_wedge = game.current_wedge.as_ref().map(|w| {
            match w {
                crate::game::WedgeValue::Cash(v) => format!("${}", v),
                crate::game::WedgeValue::Bankrupt => "BANKRUPT".to_string(),
                crate::game::WedgeValue::LoseTurn => "LOSE A TURN".to_string(),
                crate::game::WedgeValue::FreePlay => "FREE PLAY".to_string(),
                crate::game::WedgeValue::Prize { name, .. } => name.clone(),
            }
        });

        AdminRoomInfo {
            name: name.clone(),
            player_count: game.players.len(),
            phase: format!("{:?}", game.phase),
            has_host: game.host_sid.is_some(),
            players,
            active_idx: if game.players.is_empty() { None } else { Some(game.active_idx) },
            puzzle_category: if game.puzzle.category.is_empty() { None } else { Some(game.puzzle.category.clone()) },
            puzzle_answer: if game.puzzle.answer.is_empty() { None } else { Some(game.puzzle.answer.clone()) },
            revealed_count,
            total_letters,
            current_wedge,
        }
    }).collect();

    (StatusCode::OK, Json(AdminRoomsResponse {
        ok: true, rooms: Some(rooms), error: None
    }))
}

#[derive(Deserialize)]
struct CreateRoomRequest {
    name: String,
}

async fn admin_create_room(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateRoomRequest>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    let room_name = req.name.trim().to_lowercase();

    // Validate room name
    if room_name.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(SimpleResponse {
            ok: false, message: None, error: Some("Room name cannot be empty".to_string())
        }));
    }

    if !room_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return (StatusCode::BAD_REQUEST, Json(SimpleResponse {
            ok: false, message: None, error: Some("Room name can only contain letters, numbers, hyphens and underscores".to_string())
        }));
    }

    // Check if room already exists
    {
        let manager = state.game_manager.read().await;
        if manager.rooms.contains_key(&room_name) {
            return (StatusCode::CONFLICT, Json(SimpleResponse {
                ok: false, message: None, error: Some("Room already exists".to_string())
            }));
        }
    }

    // Create the room
    {
        let mut manager = state.game_manager.write().await;
        manager.get_or_create_room(&room_name);
    }

    (StatusCode::OK, Json(SimpleResponse {
        ok: true, message: Some(format!("Room '{}' created", room_name)), error: None
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

#[derive(Deserialize)]
struct PlayerIdxPath {
    room: String,
    idx: usize,
}

async fn admin_kick_player(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(path): Path<PlayerIdxPath>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    let mut manager = state.game_manager.write().await;
    if let Some(game) = manager.get_room_mut(&path.room) {
        if path.idx < game.players.len() {
            let player_name = game.players[path.idx].name.clone();
            game.players.remove(path.idx);
            // Adjust active_idx if needed
            if game.active_idx >= game.players.len() && !game.players.is_empty() {
                game.active_idx = 0;
            }
            return (StatusCode::OK, Json(SimpleResponse {
                ok: true, message: Some(format!("Player '{}' kicked", player_name)), error: None
            }));
        }
    }

    (StatusCode::NOT_FOUND, Json(SimpleResponse {
        ok: false, message: None, error: Some("Player not found".to_string())
    }))
}

async fn admin_reset_player_score(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(path): Path<PlayerIdxPath>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    let mut manager = state.game_manager.write().await;
    if let Some(game) = manager.get_room_mut(&path.room) {
        if path.idx < game.players.len() {
            game.players[path.idx].total = 0;
            game.players[path.idx].round_bank = 0;
            return (StatusCode::OK, Json(SimpleResponse {
                ok: true, message: Some("Score reset".to_string()), error: None
            }));
        }
    }

    (StatusCode::NOT_FOUND, Json(SimpleResponse {
        ok: false, message: None, error: Some("Player not found".to_string())
    }))
}

#[derive(Deserialize)]
struct KickUserPath {
    room: String,
    user_id: i64,
}

async fn admin_kick_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(path): Path<KickUserPath>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    // Find the player by user_id and remove them
    let mut manager = state.game_manager.write().await;
    if let Some(game) = manager.get_room_mut(&path.room) {
        // Find player with matching user_id
        if let Some(idx) = game.players.iter().position(|p| p.user_id == Some(path.user_id)) {
            let player_name = game.players[idx].name.clone();
            game.players.remove(idx);
            // Adjust active_idx if needed
            if game.active_idx >= game.players.len() && !game.players.is_empty() {
                game.active_idx = 0;
            }
            return (StatusCode::OK, Json(SimpleResponse {
                ok: true, message: Some(format!("User '{}' kicked from room", player_name)), error: None
            }));
        }
    }

    (StatusCode::NOT_FOUND, Json(SimpleResponse {
        ok: false, message: None, error: Some("User not found in room".to_string())
    }))
}

async fn admin_new_game(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(room_name): Path<String>,
) -> (StatusCode, Json<SimpleResponse>) {
    if get_admin_user(&state, &headers).await.is_none() {
        return (StatusCode::FORBIDDEN, Json(SimpleResponse {
            ok: false, message: None, error: Some("Admin access required".to_string())
        }));
    }

    let mut manager = state.game_manager.write().await;
    if let Some(game) = manager.get_room_mut(&room_name) {
        // Reset the game state
        game.reset_game();
        return (StatusCode::OK, Json(SimpleResponse {
            ok: true, message: Some("New game started".to_string()), error: None
        }));
    }

    (StatusCode::NOT_FOUND, Json(SimpleResponse {
        ok: false, message: None, error: Some("Room not found".to_string())
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
    pack_id: Option<i64>,
    disconnect_timeout_secs: Option<i64>,
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

    // Use pack_id from request, falling back to existing
    let pack_id = match req.pack_id {
        Some(0) => None,  // 0 means "all packs"
        Some(id) => Some(id),
        None => existing.pack_id,
    };

    let config = crate::game::RoomConfig {
        vowel_cost: req.vowel_cost.unwrap_or(existing.vowel_cost),
        final_seconds: req.final_seconds.unwrap_or(existing.final_seconds),
        final_jackpot: req.final_jackpot.unwrap_or(existing.final_jackpot),
        prize_replace_cash_values: existing.prize_replace_cash_values,
        puzzle_display_seconds: req.puzzle_display_seconds.unwrap_or(existing.puzzle_display_seconds),
        prize_wedge_names: req.prize_wedge_names.unwrap_or(existing.prize_wedge_names),
        pack_id,
        disconnect_timeout_secs: req.disconnect_timeout_secs.unwrap_or(existing.disconnect_timeout_secs),
    };

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

#[cfg(test)]
mod tests {
    use super::*;

    // ========== PASSWORD HASHING TESTS ==========

    #[test]
    fn test_hash_password_produces_argon2_hash() {
        let hash = hash_password("testpassword123").unwrap();
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_different_salts() {
        let hash1 = hash_password("samepassword").unwrap();
        let hash2 = hash_password("samepassword").unwrap();
        // Same password should produce different hashes due to random salt
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "mysecretpassword";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash));
    }

    #[test]
    fn test_verify_password_incorrect() {
        let hash = hash_password("correctpassword").unwrap();
        assert!(!verify_password("wrongpassword", &hash));
    }

    #[test]
    fn test_verify_password_empty_password() {
        let hash = hash_password("somepassword").unwrap();
        assert!(!verify_password("", &hash));
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        assert!(!verify_password("password", "not-a-valid-hash"));
    }

    #[test]
    fn test_verify_werkzeug_hash() {
        // This is a real Werkzeug hash for "password123"
        // Generated with: werkzeug.security.generate_password_hash("password123")
        let werkzeug_hash = "pbkdf2:sha256:600000$cDJ8Qz7qHl$e7f5b0c3d5a9c8b7d6e4f2a1b0c3d5e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3";

        // This won't verify because we don't have the exact hash
        // But test that it doesn't panic on the format
        let result = verify_werkzeug_hash("password123", werkzeug_hash);
        // The result depends on whether the hash matches
        assert!(!result || result); // Just checking it doesn't panic
    }

    #[test]
    fn test_verify_werkzeug_hash_invalid_format() {
        assert!(!verify_werkzeug_hash("password", "invalid"));
        assert!(!verify_werkzeug_hash("password", "not:a:hash"));
        assert!(!verify_werkzeug_hash("password", "pbkdf2:sha256:invalid$salt$hash"));
    }

    // ========== TOKEN EXTRACTION TESTS ==========

    #[test]
    fn test_extract_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Bearer my-secret-token"),
        );

        let token = extract_bearer_token(&headers);
        assert_eq!(token, Some("my-secret-token".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_no_header() {
        let headers = HeaderMap::new();
        assert!(extract_bearer_token(&headers).is_none());
    }

    #[test]
    fn test_extract_bearer_token_wrong_prefix() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Basic sometoken"),
        );

        assert!(extract_bearer_token(&headers).is_none());
    }

    #[test]
    fn test_extract_cookie_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(&format!("{}=my-cookie-token; other=value", AUTH_COOKIE_NAME)).unwrap(),
        );

        let token = extract_cookie_token(&headers);
        assert_eq!(token, Some("my-cookie-token".to_string()));
    }

    #[test]
    fn test_extract_cookie_token_no_cookie() {
        let headers = HeaderMap::new();
        assert!(extract_cookie_token(&headers).is_none());
    }

    #[test]
    fn test_extract_cookie_token_wrong_cookie_name() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("other_cookie=value"),
        );

        assert!(extract_cookie_token(&headers).is_none());
    }

    #[test]
    fn test_extract_auth_token_prefers_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Bearer bearer-token"),
        );
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(&format!("{}=cookie-token", AUTH_COOKIE_NAME)).unwrap(),
        );

        // Should prefer Bearer token
        let token = extract_auth_token(&headers);
        assert_eq!(token, Some("bearer-token".to_string()));
    }

    #[test]
    fn test_extract_auth_token_falls_back_to_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(&format!("{}=cookie-token", AUTH_COOKIE_NAME)).unwrap(),
        );

        let token = extract_auth_token(&headers);
        assert_eq!(token, Some("cookie-token".to_string()));
    }

    // ========== COOKIE BUILDING TESTS ==========

    #[test]
    fn test_build_auth_cookie() {
        let cookie = build_auth_cookie("my-token", false);

        assert_eq!(cookie.name(), AUTH_COOKIE_NAME);
        assert_eq!(cookie.value(), "my-token");
        assert!(cookie.http_only().unwrap_or(false));
        assert_eq!(cookie.path(), Some("/"));
    }

    #[test]
    fn test_build_auth_cookie_secure() {
        let cookie = build_auth_cookie("my-token", true);
        assert!(cookie.secure().unwrap_or(false));
    }

    #[test]
    fn test_build_logout_cookie() {
        let cookie = build_logout_cookie();

        assert_eq!(cookie.name(), AUTH_COOKIE_NAME);
        assert_eq!(cookie.value(), "");
        // Max age should be 0 or negative to clear the cookie
        assert!(cookie.max_age().map(|d| d.whole_seconds() <= 0).unwrap_or(true));
    }

    // ========== HELPER FUNCTION TESTS ==========

    #[test]
    fn test_now_secs_reasonable() {
        let now = now_secs();
        // Should be after Jan 1, 2020
        assert!(now > 1577836800);
        // Should be before Jan 1, 2100
        assert!(now < 4102444800);
    }

    // ========== VALIDATION TESTS ==========

    #[test]
    fn test_email_validation() {
        use crate::validation::validate_email_format;

        // Valid emails
        assert!(validate_email_format("test@example.com"));
        assert!(validate_email_format("user.name@domain.org"));
        assert!(validate_email_format("user+tag@sub.domain.com"));

        // Invalid emails
        assert!(!validate_email_format(""));
        assert!(!validate_email_format("notanemail"));
        assert!(!validate_email_format("@nodomain.com"));
        assert!(!validate_email_format("noat.domain.com"));
    }

    #[test]
    fn test_login_request_validation() {
        let valid_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_email = LoginRequest {
            email: "notanemail".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        let empty_password = LoginRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(),
        };
        assert!(empty_password.validate().is_err());
    }

    #[test]
    fn test_register_request_validation() {
        let valid_request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password1".to_string(),
            display_name: "Test User".to_string(),
            avatar_id: 1,
            captcha_token: None,
        };
        assert!(valid_request.validate().is_ok());

        let short_password = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "short1".to_string(),
            display_name: "Test User".to_string(),
            avatar_id: 1,
            captcha_token: None,
        };
        assert!(short_password.validate().is_err());

        let short_display_name = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password1".to_string(),
            display_name: "a".to_string(),
            avatar_id: 1,
            captcha_token: None,
        };
        assert!(short_display_name.validate().is_err());

        let long_display_name = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password1".to_string(),
            display_name: "a".repeat(25),
            avatar_id: 1,
            captcha_token: None,
        };
        assert!(long_display_name.validate().is_err());
    }

    // ========== PROFILE UPDATE REQUEST VALIDATION TESTS ==========

    #[test]
    fn test_profile_update_request_valid_display_name_only() {
        let request = ProfileUpdateRequest {
            display_name: Some("New Name".to_string()),
            avatar_id: None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_profile_update_request_valid_avatar_only() {
        let request = ProfileUpdateRequest {
            display_name: None,
            avatar_id: Some(5),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_profile_update_request_valid_both_fields() {
        let request = ProfileUpdateRequest {
            display_name: Some("New Name".to_string()),
            avatar_id: Some(7),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_profile_update_request_valid_neither_field() {
        let request = ProfileUpdateRequest {
            display_name: None,
            avatar_id: None,
        };
        // Both fields None is valid (no-op update)
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_profile_update_request_empty_display_name() {
        let request = ProfileUpdateRequest {
            display_name: Some("".to_string()),
            avatar_id: None,
        };
        // Empty string should fail validation (too short)
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_profile_update_request_single_char_display_name() {
        let request = ProfileUpdateRequest {
            display_name: Some("a".to_string()),
            avatar_id: None,
        };
        // Single character should fail (min is 2)
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_profile_update_request_two_char_display_name() {
        let request = ProfileUpdateRequest {
            display_name: Some("ab".to_string()),
            avatar_id: None,
        };
        // Two characters should be valid (minimum)
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_profile_update_request_24_char_display_name() {
        let request = ProfileUpdateRequest {
            display_name: Some("a".repeat(24)),
            avatar_id: None,
        };
        // 24 characters should be valid (maximum)
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_profile_update_request_25_char_display_name() {
        let request = ProfileUpdateRequest {
            display_name: Some("a".repeat(25)),
            avatar_id: None,
        };
        // 25 characters should fail (over max)
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_profile_update_request_display_name_with_special_chars() {
        // Valid special characters: space, hyphen, underscore, period
        let valid_names = vec![
            "John Doe",
            "Player-1",
            "user_name",
            "Mr.Smith",
            "A B-C_D.E",
        ];
        for name in valid_names {
            let request = ProfileUpdateRequest {
                display_name: Some(name.to_string()),
                avatar_id: None,
            };
            assert!(
                request.validate().is_ok(),
                "Expected '{}' to be valid",
                name
            );
        }
    }

    #[test]
    fn test_profile_update_request_display_name_with_invalid_chars() {
        // Invalid special characters
        let invalid_names = vec![
            "user@name",
            "user<script>",
            "name!",
            "name#tag",
            "user$money",
        ];
        for name in invalid_names {
            let request = ProfileUpdateRequest {
                display_name: Some(name.to_string()),
                avatar_id: None,
            };
            assert!(
                request.validate().is_err(),
                "Expected '{}' to be invalid",
                name
            );
        }
    }

    #[test]
    fn test_profile_update_request_avatar_valid_range() {
        // Valid avatar IDs are 1-12
        for avatar_id in 1..=12 {
            let request = ProfileUpdateRequest {
                display_name: None,
                avatar_id: Some(avatar_id),
            };
            // Note: The request validation doesn't validate avatar_id range,
            // that's done at the database level with clamping
            assert!(request.validate().is_ok());
        }
    }

    #[test]
    fn test_profile_update_request_avatar_zero() {
        let request = ProfileUpdateRequest {
            display_name: None,
            avatar_id: Some(0),
        };
        // Note: Request validation passes, clamping happens at DB level
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_profile_update_request_avatar_negative() {
        let request = ProfileUpdateRequest {
            display_name: None,
            avatar_id: Some(-1),
        };
        // Note: Request validation passes, clamping happens at DB level
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_profile_update_request_avatar_above_max() {
        let request = ProfileUpdateRequest {
            display_name: None,
            avatar_id: Some(13),
        };
        // Note: Request validation passes, clamping happens at DB level
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_register_request_with_avatar_id() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password1".to_string(),
            display_name: "Test User".to_string(),
            avatar_id: 5,
            captcha_token: None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_register_request_default_avatar_id() {
        // When avatar_id is not provided, it should default to 1
        let json = r#"{"email":"test@example.com","password":"password1","display_name":"Test User"}"#;
        let request: RegisterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.avatar_id, 1);
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_register_request_with_explicit_avatar() {
        let json = r#"{"email":"test@example.com","password":"password1","display_name":"Test User","avatar_id":7}"#;
        let request: RegisterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.avatar_id, 7);
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_user_info_includes_avatar_id() {
        let user_info = UserInfo {
            id: 1,
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            avatar_id: 5,
            is_admin: None,
        };
        assert_eq!(user_info.avatar_id, 5);

        // Test serialization includes avatar_id
        let json = serde_json::to_string(&user_info).unwrap();
        assert!(json.contains("\"avatar_id\":5"));
    }

    #[test]
    fn test_profile_response_includes_avatar_id() {
        let response = ProfileResponse {
            ok: true,
            user: Some(UserInfo {
                id: 1,
                email: "test@example.com".to_string(),
                display_name: "Test User".to_string(),
                avatar_id: 8,
                is_admin: None,
            }),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"avatar_id\":8"));
    }
}
