use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::AppState;

use super::{extract_auth_token, set_cookie_header, UserInfo};

// ========== REQUEST/RESPONSE TYPES ==========

#[derive(Debug, Deserialize)]
pub struct RegisterStartRequest {
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterStartResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterFinishRequest {
    pub email: String,
    pub credential: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct RegisterFinishResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginStartRequest {
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginStartResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginFinishRequest {
    pub credential: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct LoginFinishResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PasskeyListResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passkeys: Option<Vec<PasskeyInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PasskeyInfo {
    pub id: String,
    pub device_name: Option<String>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AddPasskeyStartRequest {
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeletePasskeyRequest {
    pub credential_id: String,
}

#[derive(Debug, Serialize)]
pub struct SimpleResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ========== ROUTES ==========

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register/start", post(register_start))
        .route("/register/finish", post(register_finish))
        .route("/login/start", post(login_start))
        .route("/login/finish", post(login_finish))
        .route("/list", post(list_passkeys))
        .route("/add/start", post(add_passkey_start))
        .route("/add/finish", post(add_passkey_finish))
        .route("/delete", post(delete_passkey))
}

// ========== HELPER FUNCTIONS ==========

fn get_webauthn(_state: &AppState) -> Result<Webauthn, String> {
    let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_name = std::env::var("WEBAUTHN_RP_NAME").unwrap_or_else(|_| "Holiday Wheel".to_string());
    let rp_origin = std::env::var("WEBAUTHN_RP_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5000".to_string());

    let rp_origin_url = Url::parse(&rp_origin).map_err(|e| format!("Invalid RP origin: {}", e))?;

    let builder = WebauthnBuilder::new(&rp_id, &rp_origin_url)
        .map_err(|e| format!("WebAuthn builder error: {}", e))?
        .rp_name(&rp_name);

    builder
        .build()
        .map_err(|e| format!("WebAuthn build error: {}", e))
}

// ========== REGISTRATION ENDPOINTS ==========

async fn register_start(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterStartRequest>,
) -> (StatusCode, Json<RegisterStartResponse>) {
    let email = req.email.trim().to_lowercase();

    // Check if user already exists
    match state.db.user_exists(&email).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(RegisterStartResponse {
                    ok: false,
                    options: None,
                    error: Some("Email already registered. Try logging in instead.".to_string()),
                }),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterStartResponse {
                    ok: false,
                    options: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            );
        }
        Ok(false) => {}
    }

    let webauthn = match get_webauthn(&state) {
        Ok(w) => w,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterStartResponse {
                    ok: false,
                    options: None,
                    error: Some(e),
                }),
            );
        }
    };

    // Create a user ID based on email hash
    let user_unique_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, email.as_bytes());

    // Start registration - no existing credentials for new user
    let exclude_credentials: Vec<CredentialID> = vec![];

    let (ccr, reg_state) = match webauthn.start_passkey_registration(
        user_unique_id,
        &email,
        &req.display_name,
        Some(exclude_credentials),
    ) {
        Ok(result) => result,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterStartResponse {
                    ok: false,
                    options: None,
                    error: Some(format!("WebAuthn error: {}", e)),
                }),
            );
        }
    };

    // Serialize and store the registration state
    let state_json = match serde_json::to_string(&reg_state) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterStartResponse {
                    ok: false,
                    options: None,
                    error: Some(format!("Serialization error: {}", e)),
                }),
            );
        }
    };

    // Store challenge with email for later lookup
    let challenge_id = URL_SAFE_NO_PAD.encode(ccr.public_key.challenge.as_ref());
    if let Err(e) = state
        .db
        .store_challenge(&challenge_id, None, Some(&email), "registration", 300)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RegisterStartResponse {
                ok: false,
                options: None,
                error: Some(format!("Failed to store challenge: {}", e)),
            }),
        );
    }

    // Store state in challenge (we'll need it for finish)
    // For simplicity, store it alongside the challenge data
    if let Err(e) = state
        .db
        .store_challenge(&format!("state:{}", challenge_id), None, Some(&state_json), "state", 300)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RegisterStartResponse {
                ok: false,
                options: None,
                error: Some(format!("Failed to store state: {}", e)),
            }),
        );
    }

    // Convert to JSON value for response
    let options = match serde_json::to_value(&ccr) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterStartResponse {
                    ok: false,
                    options: None,
                    error: Some(format!("JSON error: {}", e)),
                }),
            );
        }
    };

    (
        StatusCode::OK,
        Json(RegisterStartResponse {
            ok: true,
            options: Some(options),
            error: None,
        }),
    )
}

async fn register_finish(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterFinishRequest>,
) -> Response {
    let email = req.email.trim().to_lowercase();

    let webauthn = match get_webauthn(&state) {
        Ok(w) => w,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(e),
                }),
            ).into_response();
        }
    };

    // Parse the credential response
    let reg_response: RegisterPublicKeyCredential = match serde_json::from_value(req.credential) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RegisterFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Invalid credential: {}", e)),
                }),
            ).into_response();
        }
    };

    // Get the challenge from the response to look up stored state
    let _challenge_b64 = URL_SAFE_NO_PAD.encode(&reg_response.response.client_data_json);

    // Parse client data to get challenge
    let client_data: serde_json::Value = match serde_json::from_slice(&reg_response.response.client_data_json) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RegisterFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Invalid client data: {}", e)),
                }),
            ).into_response();
        }
    };

    let challenge_str = client_data["challenge"].as_str().unwrap_or("");

    // Look up stored state
    let state_challenge = match state.db.consume_challenge(&format!("state:{}", challenge_str)).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RegisterFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some("Challenge expired or not found".to_string()),
                }),
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            ).into_response();
        }
    };

    // Also consume the main challenge
    let _ = state.db.consume_challenge(challenge_str).await;

    // Deserialize the registration state
    let reg_state: PasskeyRegistration = match state_challenge.email.as_ref().and_then(|s| serde_json::from_str(s).ok()) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RegisterFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some("Invalid registration state".to_string()),
                }),
            ).into_response();
        }
    };

    // Complete registration
    let passkey = match webauthn.finish_passkey_registration(&reg_response, &reg_state) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RegisterFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Registration failed: {}", e)),
                }),
            ).into_response();
        }
    };

    // Create user (verified since passkey proves device ownership)
    let user_id = match state.db.create_oauth_user(&email, email.split('@').next().unwrap_or(&email), true).await {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Failed to create user: {}", e)),
                }),
            ).into_response();
        }
    };

    // Store passkey credential
    let cred_id = URL_SAFE_NO_PAD.encode(passkey.cred_id().as_ref());
    let public_key = match serde_json::to_vec(&passkey) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Failed to serialize passkey: {}", e)),
                }),
            ).into_response();
        }
    };

    if let Err(e) = state
        .db
        .create_passkey(&cred_id, user_id, &public_key, 0, None, None)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RegisterFinishResponse {
                ok: false,
                token: None,
                user: None,
                error: Some(format!("Failed to store passkey: {}", e)),
            }),
        ).into_response();
    }

    // Generate auth token
    let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    if let Err(e) = state.db.set_remember_token(user_id, &token).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RegisterFinishResponse {
                ok: false,
                token: None,
                user: None,
                error: Some(format!("Failed to create session: {}", e)),
            }),
        ).into_response();
    }
    let _ = state.db.update_last_login(user_id).await;

    // Build response with Set-Cookie header
    let response = RegisterFinishResponse {
        ok: true,
        token: Some(token.clone()),
        user: Some(UserInfo {
            id: user_id,
            email: email.clone(),
            display_name: email.split('@').next().unwrap_or(&email).to_string(),
        }),
        error: None,
    };

    let mut res = (StatusCode::OK, Json(response)).into_response();
    res.headers_mut().insert(header::SET_COOKIE, set_cookie_header(&token));
    res
}

// ========== LOGIN ENDPOINTS ==========

async fn login_start(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginStartRequest>,
) -> (StatusCode, Json<LoginStartResponse>) {
    let webauthn = match get_webauthn(&state) {
        Ok(w) => w,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginStartResponse {
                    ok: false,
                    options: None,
                    error: Some(e),
                }),
            );
        }
    };

    // If email provided, get user's passkeys
    let (user_id, passkeys) = if let Some(ref email) = req.email {
        let email = email.trim().to_lowercase();
        match state.db.get_user_by_email(&email).await {
            Ok(Some(user)) => {
                let creds = state.db.get_user_passkeys(user.id).await.unwrap_or_default();
                if creds.is_empty() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(LoginStartResponse {
                            ok: false,
                            options: None,
                            error: Some("No passkeys registered for this account".to_string()),
                        }),
                    );
                }
                (Some(user.id), creds)
            }
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(LoginStartResponse {
                        ok: false,
                        options: None,
                        error: Some("Account not found".to_string()),
                    }),
                );
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(LoginStartResponse {
                        ok: false,
                        options: None,
                        error: Some(format!("Database error: {}", e)),
                    }),
                );
            }
        }
    } else {
        // Discoverable credential flow - allow any passkey
        (None, vec![])
    };

    // Convert stored passkeys to webauthn format
    let allow_credentials: Vec<Passkey> = passkeys
        .iter()
        .filter_map(|p| serde_json::from_slice(&p.public_key).ok())
        .collect();

    // webauthn-rs 0.5 doesn't support discoverable credentials without email
    // Require email for passkey authentication
    if allow_credentials.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(LoginStartResponse {
                ok: false,
                options: None,
                error: Some("Email is required for passkey login".to_string()),
            }),
        );
    }

    let (rcr, auth_state) = match webauthn.start_passkey_authentication(&allow_credentials) {
        Ok(result) => result,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginStartResponse {
                    ok: false,
                    options: None,
                    error: Some(format!("WebAuthn error: {}", e)),
                }),
            );
        }
    };

    // Store authentication state
    let state_json = match serde_json::to_string(&auth_state) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginStartResponse {
                    ok: false,
                    options: None,
                    error: Some(format!("Serialization error: {}", e)),
                }),
            );
        }
    };

    let challenge_id = URL_SAFE_NO_PAD.encode(rcr.public_key.challenge.as_ref());
    if let Err(e) = state
        .db
        .store_challenge(&challenge_id, user_id, None, "authentication", 300)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LoginStartResponse {
                ok: false,
                options: None,
                error: Some(format!("Failed to store challenge: {}", e)),
            }),
        );
    }

    if let Err(e) = state
        .db
        .store_challenge(&format!("state:{}", challenge_id), user_id, Some(&state_json), "state", 300)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LoginStartResponse {
                ok: false,
                options: None,
                error: Some(format!("Failed to store state: {}", e)),
            }),
        );
    }

    let options = match serde_json::to_value(&rcr) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginStartResponse {
                    ok: false,
                    options: None,
                    error: Some(format!("JSON error: {}", e)),
                }),
            );
        }
    };

    (
        StatusCode::OK,
        Json(LoginStartResponse {
            ok: true,
            options: Some(options),
            error: None,
        }),
    )
}

async fn login_finish(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginFinishRequest>,
) -> Response {
    let webauthn = match get_webauthn(&state) {
        Ok(w) => w,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(e),
                }),
            ).into_response();
        }
    };

    // Parse the authentication response
    let auth_response: PublicKeyCredential = match serde_json::from_value(req.credential) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Invalid credential: {}", e)),
                }),
            ).into_response();
        }
    };

    // Parse client data to get challenge
    let client_data: serde_json::Value = match serde_json::from_slice(&auth_response.response.client_data_json) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Invalid client data: {}", e)),
                }),
            ).into_response();
        }
    };

    let challenge_str = client_data["challenge"].as_str().unwrap_or("");

    // Look up stored state
    let state_challenge = match state.db.consume_challenge(&format!("state:{}", challenge_str)).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some("Challenge expired or not found".to_string()),
                }),
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            ).into_response();
        }
    };

    // Also consume the main challenge
    let _main_challenge = state.db.consume_challenge(challenge_str).await.ok().flatten();

    // Look up the credential to find the user
    // Use raw_id which is the raw bytes of the credential ID
    let cred_id = URL_SAFE_NO_PAD.encode(auth_response.raw_id.as_ref() as &[u8]);
    let stored_cred = match state.db.get_passkey(&cred_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some("Passkey not found".to_string()),
                }),
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            ).into_response();
        }
    };

    // Deserialize the passkey (validates the stored format)
    let _passkey: Passkey = match serde_json::from_slice(&stored_cred.public_key) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Invalid stored passkey: {}", e)),
                }),
            ).into_response();
        }
    };

    // Deserialize auth state
    let auth_state: PasskeyAuthentication = match state_challenge.email.as_ref().and_then(|s| serde_json::from_str(s).ok()) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some("Invalid authentication state".to_string()),
                }),
            ).into_response();
        }
    };

    // Complete authentication
    let auth_result = match webauthn.finish_passkey_authentication(&auth_response, &auth_state) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Authentication failed: {}", e)),
                }),
            ).into_response();
        }
    };

    // Update counter
    let _ = state
        .db
        .update_passkey_counter(&cred_id, auth_result.counter() as i64)
        .await;

    // Get user
    let user = match state.db.get_user_by_id(stored_cred.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some("User not found".to_string()),
                }),
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginFinishResponse {
                    ok: false,
                    token: None,
                    user: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            ).into_response();
        }
    };

    // Generate auth token
    let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    if let Err(e) = state.db.set_remember_token(user.id, &token).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LoginFinishResponse {
                ok: false,
                token: None,
                user: None,
                error: Some(format!("Failed to create session: {}", e)),
            }),
        ).into_response();
    }
    let _ = state.db.update_last_login(user.id).await;

    // Build response with Set-Cookie header
    let response = LoginFinishResponse {
        ok: true,
        token: Some(token.clone()),
        user: Some(UserInfo {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
        }),
        error: None,
    };

    let mut res = (StatusCode::OK, Json(response)).into_response();
    res.headers_mut().insert(header::SET_COOKIE, set_cookie_header(&token));
    res
}

// ========== PASSKEY MANAGEMENT ENDPOINTS ==========

async fn list_passkeys(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<PasskeyListResponse>) {
    let token = match extract_auth_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(PasskeyListResponse {
                    ok: false,
                    passkeys: None,
                    error: Some("Authentication required".to_string()),
                }),
            );
        }
    };

    let user = match state.db.get_user_by_token(&token).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(PasskeyListResponse {
                    ok: false,
                    passkeys: None,
                    error: Some("Invalid token".to_string()),
                }),
            );
        }
    };

    let passkeys = match state.db.get_user_passkeys(user.id).await {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PasskeyListResponse {
                    ok: false,
                    passkeys: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            );
        }
    };

    (
        StatusCode::OK,
        Json(PasskeyListResponse {
            ok: true,
            passkeys: Some(
                passkeys
                    .into_iter()
                    .map(|p| PasskeyInfo {
                        id: p.id,
                        device_name: p.device_name,
                        created_at: p.created_at,
                        last_used_at: p.last_used_at,
                    })
                    .collect(),
            ),
            error: None,
        }),
    )
}

async fn add_passkey_start(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddPasskeyStartRequest>,
) -> (StatusCode, Json<RegisterStartResponse>) {
    let token = match extract_auth_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(RegisterStartResponse {
                    ok: false,
                    options: None,
                    error: Some("Authentication required".to_string()),
                }),
            );
        }
    };

    let user = match state.db.get_user_by_token(&token).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(RegisterStartResponse {
                    ok: false,
                    options: None,
                    error: Some("Invalid token".to_string()),
                }),
            );
        }
    };

    let webauthn = match get_webauthn(&state) {
        Ok(w) => w,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterStartResponse {
                    ok: false,
                    options: None,
                    error: Some(e),
                }),
            );
        }
    };

    // Get existing passkeys to exclude
    let existing = state.db.get_user_passkeys(user.id).await.unwrap_or_default();
    let exclude_credentials: Vec<CredentialID> = existing
        .iter()
        .filter_map(|p| URL_SAFE_NO_PAD.decode(&p.id).ok())
        .map(CredentialID::from)
        .collect();

    let user_unique_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, user.email.as_bytes());

    let (ccr, reg_state) = match webauthn.start_passkey_registration(
        user_unique_id,
        &user.email,
        &user.display_name,
        Some(exclude_credentials),
    ) {
        Ok(result) => result,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterStartResponse {
                    ok: false,
                    options: None,
                    error: Some(format!("WebAuthn error: {}", e)),
                }),
            );
        }
    };

    let state_json = serde_json::to_string(&reg_state).unwrap_or_default();
    let challenge_id = URL_SAFE_NO_PAD.encode(ccr.public_key.challenge.as_ref());

    // Store device name in email field (hacky but works)
    let device_info = req.device_name.unwrap_or_else(|| "Unknown Device".to_string());

    let _ = state
        .db
        .store_challenge(&challenge_id, Some(user.id), Some(&device_info), "add_passkey", 300)
        .await;

    let _ = state
        .db
        .store_challenge(&format!("state:{}", challenge_id), Some(user.id), Some(&state_json), "state", 300)
        .await;

    let options = serde_json::to_value(&ccr).unwrap_or_default();

    (
        StatusCode::OK,
        Json(RegisterStartResponse {
            ok: true,
            options: Some(options),
            error: None,
        }),
    )
}

async fn add_passkey_finish(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RegisterFinishRequest>,
) -> (StatusCode, Json<SimpleResponse>) {
    let token = match extract_auth_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(SimpleResponse {
                    ok: false,
                    message: None,
                    error: Some("Authentication required".to_string()),
                }),
            );
        }
    };

    let user = match state.db.get_user_by_token(&token).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(SimpleResponse {
                    ok: false,
                    message: None,
                    error: Some("Invalid token".to_string()),
                }),
            );
        }
    };

    let webauthn = match get_webauthn(&state) {
        Ok(w) => w,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SimpleResponse {
                    ok: false,
                    message: None,
                    error: Some(e),
                }),
            );
        }
    };

    let reg_response: RegisterPublicKeyCredential = match serde_json::from_value(req.credential) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(SimpleResponse {
                    ok: false,
                    message: None,
                    error: Some(format!("Invalid credential: {}", e)),
                }),
            );
        }
    };

    let client_data: serde_json::Value =
        serde_json::from_slice(&reg_response.response.client_data_json).unwrap_or_default();
    let challenge_str = client_data["challenge"].as_str().unwrap_or("");

    let main_challenge = state.db.consume_challenge(challenge_str).await.ok().flatten();
    let state_challenge = state
        .db
        .consume_challenge(&format!("state:{}", challenge_str))
        .await
        .ok()
        .flatten();

    let device_name = main_challenge.and_then(|c| c.email);

    let reg_state: PasskeyRegistration = match state_challenge.and_then(|c| c.email).and_then(|s| serde_json::from_str(&s).ok()) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(SimpleResponse {
                    ok: false,
                    message: None,
                    error: Some("Invalid registration state".to_string()),
                }),
            );
        }
    };

    let passkey = match webauthn.finish_passkey_registration(&reg_response, &reg_state) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(SimpleResponse {
                    ok: false,
                    message: None,
                    error: Some(format!("Registration failed: {}", e)),
                }),
            );
        }
    };

    let cred_id = URL_SAFE_NO_PAD.encode(passkey.cred_id().as_ref());
    let public_key = serde_json::to_vec(&passkey).unwrap_or_default();

    if let Err(e) = state
        .db
        .create_passkey(&cred_id, user.id, &public_key, 0, None, device_name.as_deref())
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SimpleResponse {
                ok: false,
                message: None,
                error: Some(format!("Failed to store passkey: {}", e)),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(SimpleResponse {
            ok: true,
            message: Some("Passkey added successfully".to_string()),
            error: None,
        }),
    )
}

async fn delete_passkey(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DeletePasskeyRequest>,
) -> (StatusCode, Json<SimpleResponse>) {
    let token = match extract_auth_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(SimpleResponse {
                    ok: false,
                    message: None,
                    error: Some("Authentication required".to_string()),
                }),
            );
        }
    };

    let user = match state.db.get_user_by_token(&token).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(SimpleResponse {
                    ok: false,
                    message: None,
                    error: Some("Invalid token".to_string()),
                }),
            );
        }
    };

    // Check if user has other login methods before deleting last passkey
    let passkeys = state.db.get_user_passkeys(user.id).await.unwrap_or_default();
    let has_password = state.db.user_has_password(user.id).await.unwrap_or(false);
    let oauth_accounts = state.db.get_user_oauth_accounts(user.id).await.unwrap_or_default();

    if passkeys.len() <= 1 && !has_password && oauth_accounts.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(SimpleResponse {
                ok: false,
                message: None,
                error: Some("Cannot delete last authentication method".to_string()),
            }),
        );
    }

    match state.db.delete_passkey(&req.credential_id, user.id).await {
        Ok(true) => (
            StatusCode::OK,
            Json(SimpleResponse {
                ok: true,
                message: Some("Passkey deleted".to_string()),
                error: None,
            }),
        ),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(SimpleResponse {
                ok: false,
                message: None,
                error: Some("Passkey not found".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SimpleResponse {
                ok: false,
                message: None,
                error: Some(format!("Database error: {}", e)),
            }),
        ),
    }
}
