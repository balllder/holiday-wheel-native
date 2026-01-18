use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};

use crate::AppState;

use super::{set_cookie_header, UserInfo};

// ========== REQUEST/RESPONSE TYPES ==========

#[derive(Debug, Deserialize)]
pub struct GoogleAuthRequest {
    pub id_token: Option<String>,      // For mobile apps (JWT)
    pub access_token: Option<String>,  // For web (OAuth access token)
}

#[derive(Debug, Deserialize)]
pub struct AppleAuthRequest {
    pub identity_token: String,
    #[allow(dead_code)]
    pub user_identifier: Option<String>,
    pub email: Option<String>,
    pub full_name: Option<AppleFullName>,
}

#[derive(Debug, Deserialize)]
pub struct AppleFullName {
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_new_user: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ========== JWT CLAIMS ==========

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GoogleClaims {
    sub: String,           // Google user ID
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
    aud: String,           // Client ID (validated by jsonwebtoken)
    iss: String,           // Issuer (validated by jsonwebtoken)
    exp: i64,              // Expiration (validated by jsonwebtoken)
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AppleClaims {
    sub: String,           // Apple user ID
    email: Option<String>,
    email_verified: Option<String>, // Apple returns this as a string "true"/"false"
    aud: String,           // Client ID (validated by jsonwebtoken)
    iss: String,           // Issuer (validated by jsonwebtoken)
    exp: i64,              // Expiration (validated by jsonwebtoken)
}

// ========== JWKS TYPES ==========

#[derive(Debug, Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Jwk {
    kty: String,           // Key type (required by JWKS spec)
    kid: String,
    #[serde(rename = "use")]
    use_: Option<String>,  // Key usage
    alg: Option<String>,   // Algorithm
    n: Option<String>,     // RSA modulus
    e: Option<String>,     // RSA exponent
}

// ========== ROUTES ==========

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/google", post(google_auth))
        .route("/apple", post(apple_auth))
}

// ========== GOOGLE AUTH ==========

// Response from Google's userinfo endpoint (for access token flow)
#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
}

async fn google_auth(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GoogleAuthRequest>,
) -> Response {
    // Get configured client IDs
    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    let client_id_ios = std::env::var("GOOGLE_CLIENT_ID_IOS").unwrap_or_default();
    let client_id_android = std::env::var("GOOGLE_CLIENT_ID_ANDROID").unwrap_or_default();

    if client_id.is_empty() && client_id_ios.is_empty() && client_id_android.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(OAuthResponse {
                ok: false,
                token: None,
                user: None,
                is_new_user: None,
                error: Some("Google Sign-In not configured".to_string()),
            }),
        ).into_response();
    }

    // Determine which token type we have and verify accordingly
    let (sub, email, name) = if let Some(id_token) = &req.id_token {
        // Mobile app flow: verify JWT ID token
        match verify_google_token(id_token, &[&client_id, &client_id_ios, &client_id_android]).await {
            Ok(claims) => {
                let email = match claims.email {
                    Some(e) => e.to_lowercase(),
                    None => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(OAuthResponse {
                                ok: false,
                                token: None,
                                user: None,
                                is_new_user: None,
                                error: Some("Email not provided by Google".to_string()),
                            }),
                        ).into_response();
                    }
                };
                (claims.sub, email, claims.name)
            }
            Err(e) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(OAuthResponse {
                        ok: false,
                        token: None,
                        user: None,
                        is_new_user: None,
                        error: Some(format!("Invalid ID token: {}", e)),
                    }),
                ).into_response();
            }
        }
    } else if let Some(access_token) = &req.access_token {
        // Web flow: verify access token via Google's userinfo endpoint
        match verify_google_access_token(access_token).await {
            Ok(user_info) => {
                let email = match user_info.email {
                    Some(e) => e.to_lowercase(),
                    None => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(OAuthResponse {
                                ok: false,
                                token: None,
                                user: None,
                                is_new_user: None,
                                error: Some("Email not provided by Google".to_string()),
                            }),
                        ).into_response();
                    }
                };
                (user_info.sub, email, user_info.name)
            }
            Err(e) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(OAuthResponse {
                        ok: false,
                        token: None,
                        user: None,
                        is_new_user: None,
                        error: Some(format!("Invalid access token: {}", e)),
                    }),
                ).into_response();
            }
        }
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(OAuthResponse {
                ok: false,
                token: None,
                user: None,
                is_new_user: None,
                error: Some("No token provided. Send id_token or access_token".to_string()),
            }),
        ).into_response();
    };

    let display_name = name.unwrap_or_else(|| email.split('@').next().unwrap_or(&email).to_string());

    // Handle login/registration
    handle_oauth_user(&state, "google", &sub, &email, &display_name).await
}

async fn verify_google_access_token(access_token: &str) -> Result<GoogleUserInfo, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to verify token: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Google API error: {}", response.status()));
    }

    response
        .json::<GoogleUserInfo>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

async fn verify_google_token(token: &str, valid_client_ids: &[&str]) -> Result<GoogleClaims, String> {
    // Decode header to get kid
    let header = decode_header(token).map_err(|e| format!("Invalid token header: {}", e))?;
    let kid = header.kid.ok_or("Token missing kid")?;

    // Fetch Google's public keys
    let jwks = fetch_google_jwks().await?;

    // Find matching key
    let jwk = jwks.keys.iter().find(|k| k.kid == kid).ok_or("Key not found")?;

    // Build decoding key
    let n = jwk.n.as_ref().ok_or("Missing modulus")?;
    let e = jwk.e.as_ref().ok_or("Missing exponent")?;
    let decoding_key = DecodingKey::from_rsa_components(n, e)
        .map_err(|e| format!("Invalid key: {}", e))?;

    // Configure validation
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["accounts.google.com", "https://accounts.google.com"]);
    validation.set_audience(valid_client_ids);

    // Decode and verify
    let token_data = decode::<GoogleClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Token verification failed: {}", e))?;

    Ok(token_data.claims)
}

async fn fetch_google_jwks() -> Result<JwkSet, String> {
    let url = "https://www.googleapis.com/oauth2/v3/certs";
    let client = reqwest::Client::new();

    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;

    response.json::<JwkSet>()
        .await
        .map_err(|e| format!("Failed to parse JWKS: {}", e))
}

// ========== APPLE AUTH ==========

async fn apple_auth(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AppleAuthRequest>,
) -> Response {
    let client_id = std::env::var("APPLE_CLIENT_ID").unwrap_or_default();

    if client_id.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(OAuthResponse {
                ok: false,
                token: None,
                user: None,
                is_new_user: None,
                error: Some("Apple Sign-In not configured".to_string()),
            }),
        ).into_response();
    }

    // Verify token
    let claims = match verify_apple_token(&req.identity_token, &client_id).await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(OAuthResponse {
                    ok: false,
                    token: None,
                    user: None,
                    is_new_user: None,
                    error: Some(format!("Invalid token: {}", e)),
                }),
            ).into_response();
        }
    };

    // Apple only provides email and name on first auth
    // Use email from claims if available, otherwise from request
    let email = claims.email
        .or(req.email)
        .map(|e| e.to_lowercase())
        .unwrap_or_else(|| format!("{}@privaterelay.appleid.com", claims.sub));

    // Build display name from Apple's full name data (only on first auth)
    let display_name = req.full_name
        .map(|n| {
            let parts: Vec<&str> = [n.given_name.as_deref(), n.family_name.as_deref()]
                .into_iter()
                .flatten()
                .collect();
            if parts.is_empty() {
                email.split('@').next().unwrap_or(&email).to_string()
            } else {
                parts.join(" ")
            }
        })
        .unwrap_or_else(|| email.split('@').next().unwrap_or(&email).to_string());

    // Handle login/registration
    handle_oauth_user(&state, "apple", &claims.sub, &email, &display_name).await
}

async fn verify_apple_token(token: &str, client_id: &str) -> Result<AppleClaims, String> {
    // Decode header to get kid
    let header = decode_header(token).map_err(|e| format!("Invalid token header: {}", e))?;
    let kid = header.kid.ok_or("Token missing kid")?;

    // Fetch Apple's public keys
    let jwks = fetch_apple_jwks().await?;

    // Find matching key
    let jwk = jwks.keys.iter().find(|k| k.kid == kid).ok_or("Key not found")?;

    // Build decoding key
    let n = jwk.n.as_ref().ok_or("Missing modulus")?;
    let e = jwk.e.as_ref().ok_or("Missing exponent")?;
    let decoding_key = DecodingKey::from_rsa_components(n, e)
        .map_err(|e| format!("Invalid key: {}", e))?;

    // Configure validation
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://appleid.apple.com"]);
    validation.set_audience(&[client_id]);

    // Decode and verify
    let token_data = decode::<AppleClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Token verification failed: {}", e))?;

    Ok(token_data.claims)
}

async fn fetch_apple_jwks() -> Result<JwkSet, String> {
    let url = "https://appleid.apple.com/auth/keys";
    let client = reqwest::Client::new();

    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;

    response.json::<JwkSet>()
        .await
        .map_err(|e| format!("Failed to parse JWKS: {}", e))
}

// ========== COMMON OAUTH HANDLER ==========

async fn handle_oauth_user(
    state: &Arc<AppState>,
    provider: &str,
    provider_user_id: &str,
    email: &str,
    display_name: &str,
) -> Response {
    // Check if OAuth account already exists
    let existing_oauth = state.db.get_oauth_account(provider, provider_user_id).await.ok().flatten();

    let (user_id, is_new_user) = if let Some(oauth) = existing_oauth {
        // Existing OAuth link - get the user
        (oauth.user_id, false)
    } else {
        // Check if user exists with this email
        match state.db.get_user_by_email(email).await {
            Ok(Some(existing_user)) => {
                // Link OAuth to existing account
                if let Err(e) = state.db.create_oauth_account(existing_user.id, provider, provider_user_id, Some(email)).await {
                    tracing::warn!("Failed to link OAuth account: {}", e);
                }
                (existing_user.id, false)
            }
            Ok(None) => {
                // Create new user (auto-verified for OAuth)
                match state.db.create_oauth_user(email, display_name, true).await {
                    Ok(new_user_id) => {
                        // Link OAuth account
                        if let Err(e) = state.db.create_oauth_account(new_user_id, provider, provider_user_id, Some(email)).await {
                            tracing::warn!("Failed to create OAuth link: {}", e);
                        }
                        (new_user_id, true)
                    }
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(OAuthResponse {
                                ok: false,
                                token: None,
                                user: None,
                                is_new_user: None,
                                error: Some(format!("Failed to create user: {}", e)),
                            }),
                        ).into_response();
                    }
                }
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(OAuthResponse {
                        ok: false,
                        token: None,
                        user: None,
                        is_new_user: None,
                        error: Some(format!("Database error: {}", e)),
                    }),
                ).into_response();
            }
        }
    };

    // Get user info
    let user = match state.db.get_user_by_id(user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OAuthResponse {
                    ok: false,
                    token: None,
                    user: None,
                    is_new_user: None,
                    error: Some("User not found after creation".to_string()),
                }),
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OAuthResponse {
                    ok: false,
                    token: None,
                    user: None,
                    is_new_user: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            ).into_response();
        }
    };

    // Check if user should be admin based on ADMIN_EMAIL env var
    let mut user = user;
    if let Ok(admin_email) = std::env::var("ADMIN_EMAIL") {
        if user.email.to_lowercase() == admin_email.to_lowercase() && !user.is_admin {
            let _ = state.db.set_user_admin(user.id, true).await;
            user.is_admin = true;
        }
    }

    // Generate auth token
    let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    if let Err(e) = state.db.set_remember_token(user_id, &token).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OAuthResponse {
                ok: false,
                token: None,
                user: None,
                is_new_user: None,
                error: Some(format!("Failed to create session: {}", e)),
            }),
        ).into_response();
    }
    let _ = state.db.update_last_login(user_id).await;

    // Build response with Set-Cookie header
    let response = OAuthResponse {
        ok: true,
        token: Some(token.clone()),
        user: Some(UserInfo {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            is_admin: if user.is_admin { Some(true) } else { None },
        }),
        is_new_user: Some(is_new_user),
        error: None,
    };

    let mut res = (StatusCode::OK, Json(response)).into_response();
    res.headers_mut().insert(header::SET_COOKIE, set_cookie_header(&token));
    res
}
