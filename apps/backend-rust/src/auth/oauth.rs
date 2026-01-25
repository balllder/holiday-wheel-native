use std::sync::Arc;

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

use crate::AppState;

use super::{build_auth_cookie, set_cookie_header, UserInfo};

/// JSON structure for OAuth state user data
#[derive(Debug, Serialize, Deserialize)]
struct OAuthStateUserData {
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
#[allow(dead_code)]
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
                tracing::warn!("Google ID token verification failed: {}", e);
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(OAuthResponse {
                        ok: false,
                        token: None,
                        user: None,
                        is_new_user: None,
                        error: Some("Invalid or expired token".to_string()),
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
                tracing::warn!("Google access token verification failed: {}", e);
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(OAuthResponse {
                        ok: false,
                        token: None,
                        user: None,
                        is_new_user: None,
                        error: Some("Invalid or expired token".to_string()),
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
            tracing::warn!("Apple token verification failed: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(OAuthResponse {
                    ok: false,
                    token: None,
                    user: None,
                    is_new_user: None,
                    error: Some("Invalid or expired token".to_string()),
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
                        tracing::error!("Failed to create OAuth user for {}: {}", email, e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(OAuthResponse {
                                ok: false,
                                token: None,
                                user: None,
                                is_new_user: None,
                                error: Some("Failed to create account".to_string()),
                            }),
                        ).into_response();
                    }
                }
            }
            Err(e) => {
                tracing::error!("Database error looking up user by email {}: {}", email, e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(OAuthResponse {
                        ok: false,
                        token: None,
                        user: None,
                        is_new_user: None,
                        error: Some("An unexpected error occurred".to_string()),
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
            tracing::error!("Database error fetching user by id {}: {}", user_id, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OAuthResponse {
                    ok: false,
                    token: None,
                    user: None,
                    is_new_user: None,
                    error: Some("An unexpected error occurred".to_string()),
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
        tracing::error!("Failed to set remember token for user {}: {}", user_id, e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OAuthResponse {
                ok: false,
                token: None,
                user: None,
                is_new_user: None,
                error: Some("Failed to create session".to_string()),
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
            avatar_id: user.avatar_id,
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

#[derive(Debug, Deserialize)]
pub struct AppleAuthorizeQuery {
    /// Where to redirect after successful auth (default: /lobby)
    redirect: Option<String>,
}

/// Initiates Apple Sign-In by redirecting to Apple's authorization page
async fn apple_authorize(
    State(state): State<Arc<AppState>>,
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

    // Store state with redirect destination in database
    let final_redirect = query.redirect.unwrap_or_else(|| "/lobby".to_string());
    let user_data = serde_json::to_string(&OAuthStateUserData {
        redirect_uri: final_redirect,
    }).unwrap_or_else(|_| r#"{"redirect_uri":"/lobby"}"#.to_string());

    // Clean up expired states and store new state
    let _ = state.db.cleanup_expired_oauth_states().await;
    if let Err(e) = state.db.store_oauth_state(&state_token, &user_data, "apple").await {
        tracing::error!("Failed to store OAuth state: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to initialize OAuth flow"
        ).into_response();
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

    // Get and delete state from database (one-time use)
    let final_redirect = match state.db.get_and_delete_oauth_state(&state_token).await {
        Ok(Some(oauth_state)) => {
            // Parse user_data JSON to get redirect_uri
            match serde_json::from_str::<OAuthStateUserData>(&oauth_state.user_data) {
                Ok(data) => data.redirect_uri,
                Err(_) => "/lobby".to_string(),
            }
        }
        Ok(None) => {
            // State not found or expired
            return Redirect::to("/?error=invalid_state").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to retrieve OAuth state: {}", e);
            return Redirect::to("/?error=state_error").into_response();
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
    if state.db.set_remember_token(user_id, &token).await.is_err() {
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
                Err(e) => {
                    tracing::error!("Failed to create Apple web user for {}: {}", email, e);
                    Err("Failed to create account".to_string())
                }
            }
        }
        Err(e) => {
            tracing::error!("Database error in Apple web auth for {}: {}", email, e);
            Err("An unexpected error occurred".to_string())
        }
    }
}

// ========== TESTS ==========

#[cfg(test)]
mod tests {
    use super::*;

    // ========== REQUEST/RESPONSE SERIALIZATION ==========

    #[test]
    fn test_google_auth_request_deserialize_id_token() {
        let json = r#"{"id_token": "my-jwt-token"}"#;
        let req: GoogleAuthRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.id_token, Some("my-jwt-token".to_string()));
        assert_eq!(req.access_token, None);
    }

    #[test]
    fn test_google_auth_request_deserialize_access_token() {
        let json = r#"{"access_token": "my-access-token"}"#;
        let req: GoogleAuthRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.id_token, None);
        assert_eq!(req.access_token, Some("my-access-token".to_string()));
    }

    #[test]
    fn test_google_auth_request_deserialize_both_tokens() {
        let json = r#"{"id_token": "jwt", "access_token": "access"}"#;
        let req: GoogleAuthRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.id_token, Some("jwt".to_string()));
        assert_eq!(req.access_token, Some("access".to_string()));
    }

    #[test]
    fn test_apple_auth_request_deserialize() {
        let json = r#"{
            "identity_token": "apple-jwt",
            "user_identifier": "user-123",
            "email": "test@example.com",
            "full_name": {"given_name": "John", "family_name": "Doe"}
        }"#;
        let req: AppleAuthRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.identity_token, "apple-jwt");
        assert_eq!(req.email, Some("test@example.com".to_string()));
        let name = req.full_name.unwrap();
        assert_eq!(name.given_name, Some("John".to_string()));
        assert_eq!(name.family_name, Some("Doe".to_string()));
    }

    #[test]
    fn test_apple_auth_request_minimal() {
        let json = r#"{"identity_token": "apple-jwt"}"#;
        let req: AppleAuthRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.identity_token, "apple-jwt");
        assert_eq!(req.email, None);
        assert!(req.full_name.is_none());
    }

    #[test]
    fn test_oauth_response_serialize_success() {
        let response = OAuthResponse {
            ok: true,
            token: Some("auth-token".to_string()),
            user: Some(UserInfo {
                id: 1,
                email: "test@example.com".to_string(),
                display_name: "Test User".to_string(),
                avatar_id: 1,
                is_admin: None,
            }),
            is_new_user: Some(true),
            error: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ok\":true"));
        assert!(json.contains("\"token\":\"auth-token\""));
        assert!(json.contains("\"is_new_user\":true"));
        assert!(!json.contains("\"error\"")); // None values should be skipped
    }

    #[test]
    fn test_oauth_response_serialize_error() {
        let response = OAuthResponse {
            ok: false,
            token: None,
            user: None,
            is_new_user: None,
            error: Some("Invalid token".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ok\":false"));
        assert!(json.contains("\"error\":\"Invalid token\""));
        assert!(!json.contains("\"token\"")); // None values should be skipped
    }

    // ========== JWKS/JWK PARSING ==========

    #[test]
    fn test_jwk_set_deserialize() {
        let json = r#"{
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "key-id-1",
                    "use": "sig",
                    "alg": "RS256",
                    "n": "modulus-value",
                    "e": "AQAB"
                }
            ]
        }"#;
        let jwks: JwkSet = serde_json::from_str(json).unwrap();
        assert_eq!(jwks.keys.len(), 1);
        let key = &jwks.keys[0];
        assert_eq!(key.kty, "RSA");
        assert_eq!(key.kid, "key-id-1");
        assert_eq!(key.use_, Some("sig".to_string()));
        assert_eq!(key.alg, Some("RS256".to_string()));
        assert_eq!(key.n, Some("modulus-value".to_string()));
        assert_eq!(key.e, Some("AQAB".to_string()));
    }

    #[test]
    fn test_jwk_set_multiple_keys() {
        let json = r#"{
            "keys": [
                {"kty": "RSA", "kid": "key1", "n": "n1", "e": "e1"},
                {"kty": "RSA", "kid": "key2", "n": "n2", "e": "e2"}
            ]
        }"#;
        let jwks: JwkSet = serde_json::from_str(json).unwrap();
        assert_eq!(jwks.keys.len(), 2);
        assert_eq!(jwks.keys[0].kid, "key1");
        assert_eq!(jwks.keys[1].kid, "key2");
    }

    // ========== CLAIMS PARSING ==========

    #[test]
    fn test_google_claims_deserialize() {
        let json = r#"{
            "sub": "google-user-id",
            "email": "user@gmail.com",
            "email_verified": true,
            "name": "John Doe",
            "picture": "https://example.com/photo.jpg",
            "aud": "client-id",
            "iss": "https://accounts.google.com",
            "exp": 1234567890
        }"#;
        let claims: GoogleClaims = serde_json::from_str(json).unwrap();
        assert_eq!(claims.sub, "google-user-id");
        assert_eq!(claims.email, Some("user@gmail.com".to_string()));
        assert_eq!(claims.name, Some("John Doe".to_string()));
        assert_eq!(claims.aud, "client-id");
        assert_eq!(claims.iss, "https://accounts.google.com");
        assert_eq!(claims.exp, 1234567890);
    }

    #[test]
    fn test_google_claims_minimal() {
        let json = r#"{
            "sub": "user-id",
            "aud": "client-id",
            "iss": "accounts.google.com",
            "exp": 123
        }"#;
        let claims: GoogleClaims = serde_json::from_str(json).unwrap();
        assert_eq!(claims.sub, "user-id");
        assert_eq!(claims.email, None);
        assert_eq!(claims.name, None);
    }

    #[test]
    fn test_apple_claims_deserialize() {
        let json = r#"{
            "sub": "apple-user-id",
            "email": "user@privaterelay.appleid.com",
            "email_verified": true,
            "aud": "com.example.app",
            "iss": "https://appleid.apple.com",
            "exp": 1234567890
        }"#;
        let claims: AppleClaims = serde_json::from_str(json).unwrap();
        assert_eq!(claims.sub, "apple-user-id");
        assert_eq!(claims.email, Some("user@privaterelay.appleid.com".to_string()));
        assert_eq!(claims.aud, "com.example.app");
        assert_eq!(claims.iss, "https://appleid.apple.com");
    }

    #[test]
    fn test_apple_claims_boolean_email_verified() {
        // Apple sends email_verified as boolean, not string
        let json = r#"{
            "sub": "user-id",
            "email_verified": false,
            "aud": "client",
            "iss": "https://appleid.apple.com",
            "exp": 123
        }"#;
        let claims: AppleClaims = serde_json::from_str(json).unwrap();
        assert_eq!(claims.email_verified, Some(false));
    }

    // ========== GOOGLE USER INFO ==========

    #[test]
    fn test_google_user_info_deserialize() {
        let json = r#"{
            "sub": "123456789",
            "email": "user@gmail.com",
            "email_verified": true,
            "name": "Test User",
            "picture": "https://example.com/photo.jpg"
        }"#;
        let info: GoogleUserInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.sub, "123456789");
        assert_eq!(info.email, Some("user@gmail.com".to_string()));
        assert_eq!(info.name, Some("Test User".to_string()));
    }

    // ========== APPLE CALLBACK FORM ==========

    #[test]
    fn test_apple_callback_form_deserialize() {
        let json = r#"{
            "code": "auth-code",
            "id_token": "jwt-token",
            "state": "csrf-token",
            "user": "{\"name\":{\"firstName\":\"John\"}}"
        }"#;
        let form: AppleCallbackForm = serde_json::from_str(json).unwrap();
        assert_eq!(form.id_token, Some("jwt-token".to_string()));
        assert_eq!(form.state, Some("csrf-token".to_string()));
        assert!(form.user.is_some());
    }

    #[test]
    fn test_apple_callback_form_with_error() {
        let json = r#"{"error": "user_cancelled_authorize"}"#;
        let form: AppleCallbackForm = serde_json::from_str(json).unwrap();
        assert_eq!(form.error, Some("user_cancelled_authorize".to_string()));
        assert_eq!(form.id_token, None);
    }

    #[test]
    fn test_apple_user_data_deserialize() {
        let json = r#"{
            "name": {"firstName": "John", "lastName": "Doe"},
            "email": "john@example.com"
        }"#;
        let data: AppleUserData = serde_json::from_str(json).unwrap();
        assert_eq!(data.email, Some("john@example.com".to_string()));
        let name = data.name.unwrap();
        assert_eq!(name.first_name, Some("John".to_string()));
        assert_eq!(name.last_name, Some("Doe".to_string()));
    }

    // ========== OAUTH STATE USER DATA ==========

    #[test]
    fn test_oauth_state_user_data_serialize() {
        let data = OAuthStateUserData {
            redirect_uri: "/lobby".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("redirect_uri"));
        assert!(json.contains("/lobby"));
    }

    #[test]
    fn test_oauth_state_user_data_deserialize() {
        let json = r#"{"redirect_uri": "/custom-page"}"#;
        let data: OAuthStateUserData = serde_json::from_str(json).unwrap();
        assert_eq!(data.redirect_uri, "/custom-page");
    }

    #[test]
    fn test_oauth_state_user_data_roundtrip() {
        let original = OAuthStateUserData {
            redirect_uri: "/games/wheel".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: OAuthStateUserData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.redirect_uri, original.redirect_uri);
    }

    // ========== APPLE AUTHORIZE QUERY ==========

    #[test]
    fn test_apple_authorize_query_deserialize() {
        let json = r#"{"redirect": "/custom-page"}"#;
        let query: AppleAuthorizeQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.redirect, Some("/custom-page".to_string()));
    }

    #[test]
    fn test_apple_authorize_query_default() {
        let json = r#"{}"#;
        let query: AppleAuthorizeQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.redirect, None);
    }
}
