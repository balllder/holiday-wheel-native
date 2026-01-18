use std::sync::Arc;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Json, Router,
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

use crate::AppState;

use super::{build_auth_cookie, set_cookie_header, UserInfo};

// In-memory store for OAuth state tokens (expires after 10 minutes)
static OAUTH_STATES: Lazy<RwLock<HashMap<String, OAuthState>>> = Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Clone)]
struct OAuthState {
    created_at: u64,
    redirect_uri: String,
}

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
    #[serde(default)]
    email_verified: Option<bool>, // Apple returns this as boolean (true/false)
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
        // Apple web flow (redirect-based)
        .route("/apple/authorize", get(apple_authorize))
        .route("/apple/callback", post(apple_callback))
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

// ========== APPLE WEB FLOW ==========

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Clean up expired OAuth states (older than 10 minutes)
async fn cleanup_expired_states() {
    let now = now_secs();
    let mut states = OAUTH_STATES.write().await;
    states.retain(|_, state| now - state.created_at < 600);
}

#[derive(Debug, Deserialize)]
pub struct AppleAuthorizeQuery {
    /// Where to redirect after successful auth (default: /lobby)
    redirect: Option<String>,
}

/// Initiates Apple Sign-In by redirecting to Apple's authorization page
async fn apple_authorize(
    Query(query): Query<AppleAuthorizeQuery>,
) -> Response {
    // Get client ID for web (Services ID, not bundle ID)
    let client_id = std::env::var("APPLE_CLIENT_ID_WEB")
        .or_else(|_| std::env::var("APPLE_CLIENT_ID"))
        .unwrap_or_default();

    let redirect_uri = std::env::var("APPLE_REDIRECT_URI").unwrap_or_default();

    if client_id.is_empty() || redirect_uri.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Apple Sign-In not configured. Set APPLE_CLIENT_ID_WEB and APPLE_REDIRECT_URI environment variables."
        ).into_response();
    }

    // Generate state token for CSRF protection
    let state_token = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);

    // Store state with redirect destination
    let final_redirect = query.redirect.unwrap_or_else(|| "/lobby".to_string());
    {
        // Clean up old states first
        cleanup_expired_states().await;

        let mut states = OAUTH_STATES.write().await;
        states.insert(state_token.clone(), OAuthState {
            created_at: now_secs(),
            redirect_uri: final_redirect,
        });
    }

    // Build Apple authorization URL
    // response_mode=form_post means Apple will POST the response to our callback
    let auth_url = format!(
        "https://appleid.apple.com/auth/authorize?\
         client_id={}&\
         redirect_uri={}&\
         response_type=code%20id_token&\
         response_mode=form_post&\
         scope=name%20email&\
         state={}",
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&state_token)
    );

    Redirect::to(&auth_url).into_response()
}

/// Apple callback form data (Apple POSTs to this endpoint)
#[derive(Debug, Deserialize)]
pub struct AppleCallbackForm {
    /// Authorization code (not used directly, we use id_token)
    #[allow(dead_code)]
    code: Option<String>,
    /// The ID token (JWT) containing user info
    id_token: Option<String>,
    /// State parameter for CSRF verification
    state: Option<String>,
    /// User info (only provided on first authorization, as JSON string)
    user: Option<String>,
    /// Error from Apple
    error: Option<String>,
}

/// User info from Apple (only on first auth)
#[derive(Debug, Deserialize)]
struct AppleUserData {
    name: Option<AppleNameData>,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppleNameData {
    #[serde(rename = "firstName")]
    first_name: Option<String>,
    #[serde(rename = "lastName")]
    last_name: Option<String>,
}

/// Handles Apple's callback (form POST with id_token)
async fn apple_callback(
    State(state): State<Arc<AppState>>,
    Form(form): Form<AppleCallbackForm>,
) -> Response {
    // Check for errors from Apple
    if let Some(error) = form.error {
        tracing::warn!("Apple auth error: {}", error);
        return Redirect::to(&format!("/?error={}", urlencoding::encode(&error))).into_response();
    }

    // Verify state parameter
    let state_token = match form.state {
        Some(s) => s,
        None => {
            return Redirect::to("/?error=missing_state").into_response();
        }
    };

    let final_redirect = {
        let mut states = OAUTH_STATES.write().await;
        match states.remove(&state_token) {
            Some(oauth_state) => {
                // Check if state is expired (10 minutes)
                if now_secs() - oauth_state.created_at > 600 {
                    return Redirect::to("/?error=state_expired").into_response();
                }
                oauth_state.redirect_uri
            }
            None => {
                return Redirect::to("/?error=invalid_state").into_response();
            }
        }
    };

    // Get and verify the id_token
    let id_token = match form.id_token {
        Some(t) => t,
        None => {
            return Redirect::to("/?error=missing_token").into_response();
        }
    };

    // Get client ID for verification
    let client_id = std::env::var("APPLE_CLIENT_ID_WEB")
        .or_else(|_| std::env::var("APPLE_CLIENT_ID"))
        .unwrap_or_default();

    // Verify the token
    let claims = match verify_apple_token(&id_token, &client_id).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Apple token verification failed: {}", e);
            return Redirect::to(&format!("/?error={}", urlencoding::encode(&e))).into_response();
        }
    };

    // Parse user data if provided (only on first authorization)
    let user_data: Option<AppleUserData> = form.user.and_then(|u| {
        serde_json::from_str(&u).ok()
    });

    // Build email and display name
    let email = claims.email
        .or_else(|| user_data.as_ref().and_then(|u| u.email.clone()))
        .map(|e| e.to_lowercase())
        .unwrap_or_else(|| format!("{}@privaterelay.appleid.com", claims.sub));

    let display_name = user_data
        .and_then(|u| u.name)
        .map(|n| {
            let parts: Vec<&str> = [n.first_name.as_deref(), n.last_name.as_deref()]
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

    // Handle user creation/login using the common handler
    // But we need to handle it differently since we want to redirect, not return JSON
    let (user_id, _is_new_user) = match handle_apple_web_user(&state, &claims.sub, &email, &display_name).await {
        Ok(result) => result,
        Err(e) => {
            return Redirect::to(&format!("/?error={}", urlencoding::encode(&e))).into_response();
        }
    };

    // Get user info for the response
    let user = match state.db.get_user_by_id(user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Redirect::to("/?error=user_not_found").into_response();
        }
        Err(_) => {
            return Redirect::to("/?error=database_error").into_response();
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
    if let Err(_) = state.db.set_remember_token(user_id, &token).await {
        return Redirect::to("/?error=session_error").into_response();
    }
    let _ = state.db.update_last_login(user_id).await;

    // Build redirect response with auth cookie
    // Also include user info in URL fragment for client-side storage
    let user_json = serde_json::json!({
        "id": user.id,
        "email": user.email,
        "display_name": user.display_name,
        "is_admin": user.is_admin
    });
    let user_json_str = user_json.to_string();
    let user_encoded = urlencoding::encode(&user_json_str);

    let redirect_url = format!("{}#auth_token={}&user={}", final_redirect, token, user_encoded);

    let secure = std::env::var("SSL_ENABLED")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);
    let cookie = build_auth_cookie(&token, secure);

    let mut response = Redirect::to(&redirect_url).into_response();
    if let Ok(cookie_value) = cookie.to_string().parse() {
        response.headers_mut().insert(header::SET_COOKIE, cookie_value);
    }

    response
}

/// Handle Apple web user - similar to handle_oauth_user but returns Result
async fn handle_apple_web_user(
    state: &Arc<AppState>,
    provider_user_id: &str,
    email: &str,
    display_name: &str,
) -> Result<(i64, bool), String> {
    let provider = "apple";

    // Check if OAuth account already exists
    let existing_oauth = state.db.get_oauth_account(provider, provider_user_id).await.ok().flatten();

    if let Some(oauth) = existing_oauth {
        // Existing OAuth link - get the user
        return Ok((oauth.user_id, false));
    }

    // Check if user exists with this email
    match state.db.get_user_by_email(email).await {
        Ok(Some(existing_user)) => {
            // Link OAuth to existing account
            if let Err(e) = state.db.create_oauth_account(existing_user.id, provider, provider_user_id, Some(email)).await {
                tracing::warn!("Failed to link OAuth account: {}", e);
            }
            Ok((existing_user.id, false))
        }
        Ok(None) => {
            // Create new user (auto-verified for OAuth)
            match state.db.create_oauth_user(email, display_name, true).await {
                Ok(new_user_id) => {
                    // Link OAuth account
                    if let Err(e) = state.db.create_oauth_account(new_user_id, provider, provider_user_id, Some(email)).await {
                        tracing::warn!("Failed to create OAuth link: {}", e);
                    }
                    Ok((new_user_id, true))
                }
                Err(e) => Err(format!("Failed to create user: {}", e)),
            }
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}
