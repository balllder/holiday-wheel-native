use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::AppState;

/// Server start time for uptime calculation
static START_TIME: Lazy<Instant> = Lazy::new(Instant::now);

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub checks: HealthChecks,
}

/// Individual health checks
#[derive(Serialize)]
pub struct HealthChecks {
    pub database: &'static str,
    pub uptime_seconds: u64,
}

/// Health check endpoint
///
/// Returns 200 if all critical checks pass, 503 if any fail.
pub async fn health(State(state): State<Arc<AppState>>) -> Response {
    // Initialize start time on first call
    let uptime_seconds = START_TIME.elapsed().as_secs();

    // Check database connectivity
    let db_status = match state.db.ping().await {
        Ok(()) => "ok",
        Err(_) => "error",
    };

    let is_healthy = db_status == "ok";
    let overall_status = if is_healthy { "healthy" } else { "unhealthy" };

    let response = HealthResponse {
        status: overall_status,
        version: env!("CARGO_PKG_VERSION"),
        checks: HealthChecks {
            database: db_status,
            uptime_seconds,
        },
    };

    if is_healthy {
        (StatusCode::OK, Json(response)).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response()
    }
}

/// Favicon - Holiday Wheel themed SVG
pub async fn favicon() -> impl IntoResponse {
    // Simple wheel-themed SVG favicon
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">
  <circle cx="16" cy="16" r="14" fill="#d4af37" stroke="#0d0628" stroke-width="2"/>
  <circle cx="16" cy="16" r="10" fill="#1a5cb8"/>
  <circle cx="16" cy="16" r="3" fill="#d4af37"/>
  <line x1="16" y1="6" x2="16" y2="12" stroke="#d4af37" stroke-width="2"/>
  <line x1="16" y1="20" x2="16" y2="26" stroke="#d4af37" stroke-width="2"/>
  <line x1="6" y1="16" x2="12" y2="16" stroke="#d4af37" stroke-width="2"/>
  <line x1="20" y1="16" x2="26" y2="16" stroke="#d4af37" stroke-width="2"/>
</svg>"##;

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
        svg,
    )
}

/// Join query parameters
#[derive(Deserialize)]
pub struct JoinQuery {
    room: Option<String>,
}

/// Universal link join page - tries app first, falls back to web
pub async fn join(Query(query): Query<JoinQuery>) -> Html<String> {
    let room = query.room.unwrap_or_else(|| "main".to_string());
    let room_escaped = room
        .replace('\"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Join Holiday Wheel</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Mountains+of+Christmas:wght@700&display=swap" rel="stylesheet">
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #0d0628 0%, #1a0a3e 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            color: #fff;
        }}
        .container {{
            background: rgba(26, 10, 62, 0.8);
            padding: 40px;
            border-radius: 16px;
            border: 2px solid #333;
            text-align: center;
            max-width: 400px;
        }}
        h1 {{
            margin-bottom: 16px;
            font-size: 32px;
            font-weight: 800;
            background: linear-gradient(
                135deg,
                #ff4444 0%,
                #d4af37 25%,
                #22c55e 50%,
                #d4af37 75%,
                #ff4444 100%
            );
            background-size: 200% auto;
            -webkit-background-clip: text;
            background-clip: text;
            -webkit-text-fill-color: transparent;
            animation: festiveShimmer 4s linear infinite;
            letter-spacing: 1px;
        }}
        @keyframes festiveShimmer {{
            0% {{ background-position: 0% center; }}
            100% {{ background-position: 200% center; }}
        }}
        p {{
            color: #ccc;
            margin-bottom: 24px;
            line-height: 1.5;
        }}
        .room-name {{
            color: #d4af37;
            font-weight: bold;
        }}
        .btn {{
            display: inline-block;
            background: linear-gradient(135deg, #d4af37 0%, #b8962e 100%);
            color: #000;
            padding: 14px 32px;
            border-radius: 8px;
            text-decoration: none;
            font-weight: bold;
            font-size: 16px;
            margin: 8px;
            border: none;
            cursor: pointer;
        }}
        .btn:hover {{
            opacity: 0.9;
        }}
        .btn-secondary {{
            background: #333;
            color: #fff;
        }}
        .spinner {{
            display: inline-block;
            width: 20px;
            height: 20px;
            border: 2px solid #d4af37;
            border-top-color: transparent;
            border-radius: 50%;
            animation: spin 1s linear infinite;
            margin-right: 8px;
        }}
        @keyframes spin {{
            to {{ transform: rotate(360deg); }}
        }}
        #status {{
            margin-top: 16px;
            color: #888;
            font-size: 14px;
        }}
        .hidden {{ display: none; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🎡 Holiday Wheel</h1>
        <p>Joining room: <span class="room-name">{room_escaped}</span></p>

        <div id="loading">
            <span class="spinner"></span>
            <span>Opening app...</span>
        </div>

        <div id="fallback" class="hidden">
            <p>App not installed? Play in your browser instead!</p>
            <a href="/game?room={room_escaped}" class="btn">Play in Browser</a>
            <br>
            <a href="holidaywheel://join?room={room_escaped}&server={server}" class="btn btn-secondary">Try App Again</a>
        </div>

        <p id="status"></p>
    </div>

    <script>
        const room = "{room_escaped}";
        const server = window.location.origin;
        const deepLink = `holidaywheel://join?room=${{encodeURIComponent(room)}}&server=${{encodeURIComponent(server)}}`;

        let appOpened = false;

        // Try to open the app
        function tryOpenApp() {{
            const start = Date.now();

            // Create hidden iframe to try deep link (works on some platforms)
            const iframe = document.createElement('iframe');
            iframe.style.display = 'none';
            iframe.src = deepLink;
            document.body.appendChild(iframe);

            // Also try direct location change
            window.location.href = deepLink;

            // Check if we're still here after a delay
            setTimeout(() => {{
                // If more than 2.5 seconds passed and page is still visible, app didn't open
                if (Date.now() - start > 2000 && document.visibilityState !== 'hidden') {{
                    showFallback();
                }}
            }}, 2500);
        }}

        function showFallback() {{
            document.getElementById('loading').classList.add('hidden');
            document.getElementById('fallback').classList.remove('hidden');
            document.getElementById('status').textContent = 'App not detected - use browser instead';
        }}

        // Listen for visibility change (app opened successfully)
        document.addEventListener('visibilitychange', () => {{
            if (document.visibilityState === 'hidden') {{
                appOpened = true;
            }}
        }});

        // Start the process
        tryOpenApp();
    </script>
</body>
</html>"#,
        room_escaped = room_escaped,
        server = "{server}"
    ))
}

/// Common styles for all pages
const COMMON_STYLES: &str = r#"
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        background: linear-gradient(135deg, #0d0628 0%, #1a0a3e 100%);
        min-height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #fff;
    }
    .container {
        background: rgba(26, 10, 62, 0.8);
        padding: 40px;
        border-radius: 16px;
        border: 2px solid #333;
        width: 100%;
        max-width: 400px;
    }
    .container.wide { max-width: 900px; }
    /* Seasonal Theme System */
    :root {
        /* Default/Christmas theme */
        --theme-color-1: #ff4444;
        --theme-color-2: #d4af37;
        --theme-color-3: #22c55e;
        --theme-decorator: "❄";
        --theme-decorator-color: #87ceeb;
    }
    /* Christmas (Dec 1-25) */
    body.theme-christmas { --theme-color-1: #ff4444; --theme-color-2: #d4af37; --theme-color-3: #22c55e; --theme-decorator: "❄"; --theme-decorator-color: #87ceeb; }
    /* New Year (Dec 26 - Jan 7) */
    body.theme-newyear { --theme-color-1: #ffd700; --theme-color-2: #c0c0c0; --theme-color-3: #ffffff; --theme-decorator: "✨"; --theme-decorator-color: #ffd700; }
    /* Valentine's (Feb 1-14) */
    body.theme-valentines { --theme-color-1: #ff69b4; --theme-color-2: #ff1493; --theme-color-3: #ffffff; --theme-decorator: "❤"; --theme-decorator-color: #ff69b4; }
    /* St. Patrick's (Mar 10-17) */
    body.theme-stpatricks { --theme-color-1: #22c55e; --theme-color-2: #ffd700; --theme-color-3: #16a34a; --theme-decorator: "☘"; --theme-decorator-color: #22c55e; }
    /* Easter (variable, approx Mar 20 - Apr 20) */
    body.theme-easter { --theme-color-1: #ffb6c1; --theme-color-2: #98fb98; --theme-color-3: #dda0dd; --theme-decorator: "🐣"; --theme-decorator-color: #ffeb3b; }
    /* Independence Day (Jun 25 - Jul 10) */
    body.theme-july4th { --theme-color-1: #ff4444; --theme-color-2: #ffffff; --theme-color-3: #3b82f6; --theme-decorator: "🎆"; --theme-decorator-color: #ff4444; }
    /* Halloween (Oct 1-31) */
    body.theme-halloween { --theme-color-1: #ff6b00; --theme-color-2: #9333ea; --theme-color-3: #1a1a1a; --theme-decorator: "🎃"; --theme-decorator-color: #ff6b00; }
    /* Thanksgiving (Nov 15-30) */
    body.theme-thanksgiving { --theme-color-1: #d97706; --theme-color-2: #92400e; --theme-color-3: #fbbf24; --theme-decorator: "🦃"; --theme-decorator-color: #d97706; }
    /* Summer default (May-Sep when no holiday) */
    body.theme-summer { --theme-color-1: #f59e0b; --theme-color-2: #ef4444; --theme-color-3: #eab308; --theme-decorator: "☀"; --theme-decorator-color: #fbbf24; }

    h1 {
        text-align: center;
        margin-bottom: 8px;
        font-family: 'Mountains of Christmas', cursive;
        font-size: 42px;
        font-weight: 700;
        background: linear-gradient(
            135deg,
            var(--theme-color-1) 0%,
            var(--theme-color-2) 25%,
            var(--theme-color-3) 50%,
            var(--theme-color-2) 75%,
            var(--theme-color-1) 100%
        );
        background-size: 200% auto;
        -webkit-background-clip: text;
        background-clip: text;
        -webkit-text-fill-color: transparent;
        animation: festiveShimmer 4s linear infinite;
        text-shadow: none;
        position: relative;
        display: inline-block;
        width: 100%;
        letter-spacing: 2px;
    }
    h1::before {
        content: var(--theme-decorator);
        position: absolute;
        left: 0;
        top: 50%;
        transform: translateY(-50%);
        font-size: 20px;
        -webkit-text-fill-color: var(--theme-decorator-color);
        animation: decoratorSpin 6s linear infinite;
        opacity: 0.8;
    }
    h1::after {
        content: var(--theme-decorator);
        position: absolute;
        right: 0;
        top: 50%;
        transform: translateY(-50%);
        font-size: 20px;
        -webkit-text-fill-color: var(--theme-decorator-color);
        animation: decoratorSpin 6s linear infinite reverse;
        opacity: 0.8;
    }
    @keyframes festiveShimmer {
        0% { background-position: 0% center; }
        100% { background-position: 200% center; }
    }
    @keyframes decoratorSpin {
        0% { transform: translateY(-50%) rotate(0deg); }
        100% { transform: translateY(-50%) rotate(360deg); }
    }
    /* Fallback for browsers that don't support background-clip: text */
    @supports not (-webkit-background-clip: text) {
        h1 {
            color: #d4af37;
            background: none;
            text-shadow: 0 0 20px rgba(212, 175, 55, 0.5), 0 2px 4px rgba(0, 0, 0, 0.3);
        }
    }
    .subtitle {
        color: #888;
        text-align: center;
        margin-bottom: 32px;
    }
    .form-group { margin-bottom: 20px; }
    label {
        display: block;
        margin-bottom: 8px;
        color: #aaa;
    }
    input {
        width: 100%;
        padding: 12px 16px;
        border: 2px solid #333;
        border-radius: 8px;
        background: #0d0628;
        color: #fff;
        font-size: 16px;
    }
    input:focus {
        outline: none;
        border-color: #d4af37;
    }
    button, .btn {
        display: inline-block;
        padding: 14px 24px;
        background: #d4af37;
        color: #0d0628;
        border: none;
        border-radius: 8px;
        font-size: 18px;
        font-weight: bold;
        cursor: pointer;
        text-decoration: none;
        text-align: center;
    }
    button:hover, .btn:hover { background: #e5c048; }
    button.full { width: 100%; margin-top: 10px; }
    .btn-secondary {
        background: #333;
        color: #d4af37;
    }
    .btn-secondary:hover { background: #444; }
    .error {
        background: #ff4444;
        color: #fff;
        padding: 12px;
        border-radius: 8px;
        margin-bottom: 20px;
        display: none;
    }
    .links {
        text-align: center;
        margin-top: 20px;
    }
    .links a {
        color: #d4af37;
        text-decoration: none;
    }
    .links a:hover { text-decoration: underline; }
    .user-info {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 24px;
        padding-bottom: 16px;
        border-bottom: 1px solid #333;
    }
    .user-name { color: #d4af37; font-weight: bold; }
    .rooms-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        gap: 16px;
        margin: 24px 0;
    }
    .room-card {
        background: #0d0628;
        border: 2px solid #333;
        border-radius: 12px;
        padding: 20px;
        cursor: pointer;
        transition: all 0.2s;
    }
    .room-card:hover {
        border-color: #d4af37;
        transform: translateY(-2px);
    }
    .room-name { font-size: 18px; font-weight: bold; color: #fff; }
    .room-players { color: #888; margin-top: 8px; }
    .join-form {
        display: flex;
        gap: 12px;
        margin-top: 24px;
    }
    .join-form input { flex: 1; }
"#;

/// Root route - serves login page
pub async fn index() -> Html<String> {
    let google_client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    Html(format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Login</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Mountains+of+Christmas:wght@700&display=swap" rel="stylesheet">
    <style>
        {common_styles}
        .social-buttons {{
            display: flex;
            flex-direction: column;
            gap: 12px;
            margin-bottom: 24px;
        }}
        .social-btn {{
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 12px;
            width: 100%;
            padding: 14px 24px;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
        }}
        .social-btn svg {{
            width: 20px;
            height: 20px;
        }}
        .btn-passkey {{
            background: #5856d6;
            color: #fff;
        }}
        .btn-passkey:hover {{ background: #6b69e0; }}
        .btn-google {{
            background: #fff;
            color: #444;
            border: 1px solid #ddd;
        }}
        .btn-google:hover {{ background: #f5f5f5; }}
        .btn-apple {{
            background: #000;
            color: #fff;
        }}
        .btn-apple:hover {{ background: #222; }}
        .divider {{
            display: flex;
            align-items: center;
            margin: 24px 0;
            color: #888;
        }}
        .divider::before, .divider::after {{
            content: '';
            flex: 1;
            height: 1px;
            background: #333;
        }}
        .divider span {{
            padding: 0 16px;
            font-size: 14px;
        }}
        .hidden {{ display: none !important; }}
    </style>
</head>
<body>
    <script>
        // Seasonal theme detection
        (function() {{
            const now = new Date();
            const month = now.getMonth() + 1; // 1-12
            const day = now.getDate();
            let theme = 'theme-summer'; // default

            // Christmas: Dec 1-25
            if (month === 12 && day <= 25) theme = 'theme-christmas';
            // New Year: Dec 26 - Jan 7
            else if ((month === 12 && day >= 26) || (month === 1 && day <= 7)) theme = 'theme-newyear';
            // Valentine's: Feb 1-14
            else if (month === 2 && day <= 14) theme = 'theme-valentines';
            // St. Patrick's: Mar 10-17
            else if (month === 3 && day >= 10 && day <= 17) theme = 'theme-stpatricks';
            // Easter: approx Mar 20 - Apr 20
            else if ((month === 3 && day >= 20) || (month === 4 && day <= 20)) theme = 'theme-easter';
            // Independence Day: Jun 25 - Jul 10
            else if ((month === 6 && day >= 25) || (month === 7 && day <= 10)) theme = 'theme-july4th';
            // Halloween: Oct 1-31
            else if (month === 10) theme = 'theme-halloween';
            // Thanksgiving: Nov 15-30
            else if (month === 11 && day >= 15) theme = 'theme-thanksgiving';

            document.body.classList.add(theme);
        }})();
    </script>
    <div class="container">
        <h1>🎡 Holiday Wheel</h1>
        <p class="subtitle">Sign in to play</p>
        <div class="error" id="error"></div>

        <div class="social-buttons">
            <button id="passkeyBtn" class="social-btn btn-passkey hidden" onclick="loginWithPasskey()">
                <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 1C8.14 1 5 4.14 5 8c0 2.38 1.19 4.47 3 5.74V17a1 1 0 0 0 1 1h1v2a1 1 0 0 0 1 1h2a1 1 0 0 0 1-1v-2h1a1 1 0 0 0 1-1v-3.26c1.81-1.27 3-3.36 3-5.74 0-3.86-3.14-7-7-7zm0 2c2.76 0 5 2.24 5 5s-2.24 5-5 5-5-2.24-5-5 2.24-5 5-5zm0 2a3 3 0 0 0-3 3 3 3 0 0 0 3 3 3 3 0 0 0 3-3 3 3 0 0 0-3-3z"/></svg>
                Sign in with Passkey
            </button>
            <button id="googleBtn" class="social-btn btn-google" onclick="loginWithGoogle()">
                <svg viewBox="0 0 24 24"><path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/><path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/><path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/><path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/></svg>
                Sign in with Google
            </button>
            <button id="appleBtn" class="social-btn btn-apple" onclick="loginWithApple()">
                <svg viewBox="0 0 24 24" fill="currentColor"><path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/></svg>
                Sign in with Apple
            </button>
        </div>

        <div class="divider"><span>or sign in with email</span></div>

        <form id="loginForm">
            <div class="form-group">
                <label for="email">Email</label>
                <input type="email" id="email" name="email" required>
            </div>
            <div class="form-group">
                <label for="password">Password</label>
                <input type="password" id="password" name="password" required>
            </div>
            <button type="submit" class="full">Sign In</button>
        </form>
        <div class="links">
            <p>Don't have an account? <a href="/register">Register</a></p>
        </div>
    </div>
    <script src="https://accounts.google.com/gsi/client" async defer></script>
    <script>
        // Inject Google Client ID from server
        window.GOOGLE_CLIENT_ID = '{google_client_id}';

        // Check if already logged in
        if (localStorage.getItem('user')) {{
            window.location.href = '/lobby';
        }}

        // Handle OAuth callback (Apple Sign-In returns with hash fragment)
        if (window.location.hash) {{
            const params = new URLSearchParams(window.location.hash.substring(1));
            const authToken = params.get('auth_token');
            const userEncoded = params.get('user');
            if (authToken && userEncoded) {{
                try {{
                    const user = JSON.parse(decodeURIComponent(userEncoded));
                    localStorage.setItem('user', JSON.stringify(user));
                    localStorage.setItem('auth_token', authToken);
                    // Clear the hash and redirect to lobby
                    window.location.href = '/lobby';
                }} catch (e) {{
                    console.error('Failed to parse user data:', e);
                }}
            }}
            // Check for error in hash
            const error = params.get('error');
            if (error) {{
                const errorDiv = document.getElementById('error');
                errorDiv.textContent = decodeURIComponent(error).replace(/_/g, ' ');
                errorDiv.style.display = 'block';
                // Clear the hash
                history.replaceState(null, '', window.location.pathname);
            }}
        }}

        // Check for error in query string (from Apple callback redirect)
        const urlParams = new URLSearchParams(window.location.search);
        if (urlParams.get('error')) {{
            const errorDiv = document.getElementById('error');
            errorDiv.textContent = urlParams.get('error').replace(/_/g, ' ');
            errorDiv.style.display = 'block';
            // Clear the query string
            history.replaceState(null, '', window.location.pathname);
        }}

        // Check passkey support
        if (window.PublicKeyCredential && PublicKeyCredential.isConditionalMediationAvailable) {{
            PublicKeyCredential.isConditionalMediationAvailable().then(available => {{
                if (available || window.PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable) {{
                    document.getElementById('passkeyBtn').classList.remove('hidden');
                }}
            }});
        }}

        async function loginWithPasskey() {{
            const errorDiv = document.getElementById('error');
            errorDiv.style.display = 'none';
            try {{
                // Start passkey login - need email first for discoverable credentials
                const email = document.getElementById('email').value;
                const startRes = await fetch('/auth/api/passkey/login/start', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ email: email || undefined }}),
                    credentials: 'include'
                }});
                const startData = await startRes.json();
                if (!startRes.ok) {{
                    errorDiv.textContent = startData.error || 'Failed to start passkey login';
                    errorDiv.style.display = 'block';
                    return;
                }}

                // Convert base64url to ArrayBuffer
                const pubKey = startData.options.publicKey;
                const challenge = base64urlToBuffer(pubKey.challenge);
                const allowCredentials = (pubKey.allowCredentials || []).map(c => ({{
                    id: base64urlToBuffer(c.id),
                    type: c.type,
                    transports: c.transports
                }}));

                const credential = await navigator.credentials.get({{
                    publicKey: {{
                        challenge,
                        allowCredentials,
                        userVerification: pubKey.userVerification || 'preferred',
                        timeout: pubKey.timeout || 60000,
                        rpId: pubKey.rpId
                    }}
                }});

                // Complete login
                const finishRes = await fetch('/auth/api/passkey/login/finish', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        credential: {{
                            id: credential.id,
                            rawId: bufferToBase64url(credential.rawId),
                            response: {{
                                clientDataJSON: bufferToBase64url(credential.response.clientDataJSON),
                                authenticatorData: bufferToBase64url(credential.response.authenticatorData),
                                signature: bufferToBase64url(credential.response.signature),
                                userHandle: credential.response.userHandle ? bufferToBase64url(credential.response.userHandle) : null
                            }},
                            type: credential.type
                        }}
                    }}),
                    credentials: 'include'
                }});

                const finishData = await finishRes.json();
                if (finishRes.ok && finishData.user) {{
                    localStorage.setItem('user', JSON.stringify(finishData.user));
                    if (finishData.token) {{
                        localStorage.setItem('auth_token', finishData.token);
                    }}
                    window.location.href = '/lobby';
                }} else {{
                    errorDiv.textContent = finishData.error || 'Passkey login failed';
                    errorDiv.style.display = 'block';
                }}
            }} catch (err) {{
                console.error('Passkey error:', err);
                errorDiv.textContent = err.name === 'NotAllowedError' ? 'Passkey authentication was cancelled' : 'Passkey login failed';
                errorDiv.style.display = 'block';
            }}
        }}

        async function loginWithGoogle() {{
            const errorDiv = document.getElementById('error');
            errorDiv.style.display = 'none';
            try {{
                // Use Google Identity Services popup
                const client = google.accounts.oauth2.initTokenClient({{
                    client_id: window.GOOGLE_CLIENT_ID || '',
                    scope: 'email profile',
                    callback: async (response) => {{
                        if (response.error) {{
                            errorDiv.textContent = 'Google sign-in failed';
                            errorDiv.style.display = 'block';
                            return;
                        }}
                        // Exchange access token for ID token info
                        const userInfoRes = await fetch('https://www.googleapis.com/oauth2/v3/userinfo', {{
                            headers: {{ 'Authorization': 'Bearer ' + response.access_token }}
                        }});
                        const userInfo = await userInfoRes.json();

                        // Send to backend - use access_token since we can't get id_token directly
                        const res = await fetch('/auth/api/oauth/google', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{ access_token: response.access_token, user_info: userInfo }}),
                            credentials: 'include'
                        }});
                        const data = await res.json();
                        if (res.ok && data.user) {{
                            localStorage.setItem('user', JSON.stringify(data.user));
                            if (data.token) {{
                                localStorage.setItem('auth_token', data.token);
                            }}
                            window.location.href = '/lobby';
                        }} else {{
                            errorDiv.textContent = data.error || 'Google login failed';
                            errorDiv.style.display = 'block';
                        }}
                    }}
                }});
                client.requestAccessToken();
            }} catch (err) {{
                console.error('Google error:', err);
                errorDiv.textContent = 'Google sign-in failed';
                errorDiv.style.display = 'block';
            }}
        }}

        async function loginWithApple() {{
            // Redirect to Apple Sign In authorization endpoint
            window.location.href = '/auth/api/oauth/apple/authorize';
        }}

        // Base64URL utilities
        function base64urlToBuffer(base64url) {{
            const base64 = base64url.replace(/-/g, '+').replace(/_/g, '/');
            const pad = base64.length % 4;
            const padded = pad ? base64 + '='.repeat(4 - pad) : base64;
            const binary = atob(padded);
            const buffer = new Uint8Array(binary.length);
            for (let i = 0; i < binary.length; i++) {{
                buffer[i] = binary.charCodeAt(i);
            }}
            return buffer.buffer;
        }}

        function bufferToBase64url(buffer) {{
            const bytes = new Uint8Array(buffer);
            let binary = '';
            for (let i = 0; i < bytes.byteLength; i++) {{
                binary += String.fromCharCode(bytes[i]);
            }}
            return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
        }}

        document.getElementById('loginForm').addEventListener('submit', async (e) => {{
            e.preventDefault();
            const errorDiv = document.getElementById('error');
            errorDiv.style.display = 'none';

            const email = document.getElementById('email').value;
            const password = document.getElementById('password').value;

            try {{
                const res = await fetch('/auth/api/login', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ email, password }}),
                    credentials: 'include'
                }});
                const data = await res.json();

                if (res.ok && data.user) {{
                    localStorage.setItem('user', JSON.stringify(data.user));
                    if (data.token) {{
                        localStorage.setItem('auth_token', data.token);
                    }}
                    window.location.href = '/lobby';
                }} else {{
                    errorDiv.textContent = data.error || 'Login failed';
                    errorDiv.style.display = 'block';
                }}
            }} catch (err) {{
                errorDiv.textContent = 'Connection error';
                errorDiv.style.display = 'block';
            }}
        }});
    </script>
</body>
</html>"##,
        common_styles = COMMON_STYLES,
        google_client_id = google_client_id
    ))
}

/// Register page
pub async fn register() -> Html<String> {
    Html(format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Register</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Mountains+of+Christmas:wght@700&display=swap" rel="stylesheet">
    <style>
        {common_styles}
        .passkey-section {{
            margin-bottom: 24px;
            padding-bottom: 24px;
            border-bottom: 1px solid #333;
        }}
        .passkey-btn {{
            width: 100%;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 12px;
            padding: 14px 24px;
            background: #333;
            color: #fff;
            border: 2px solid #d4af37;
            border-radius: 8px;
            font-size: 16px;
            font-weight: bold;
            cursor: pointer;
        }}
        .passkey-btn:hover {{
            background: #444;
        }}
        .passkey-btn svg {{
            width: 24px;
            height: 24px;
            fill: #d4af37;
        }}
        .or-divider {{
            display: flex;
            align-items: center;
            margin: 16px 0;
            color: #666;
        }}
        .or-divider::before,
        .or-divider::after {{
            content: '';
            flex: 1;
            border-bottom: 1px solid #333;
        }}
        .or-divider span {{
            padding: 0 16px;
        }}
        .hidden {{ display: none !important; }}
        .success {{
            background: #4CAF50;
            color: #fff;
            padding: 12px;
            border-radius: 8px;
            margin-bottom: 20px;
            display: none;
        }}
        .avatar-grid {{
            display: grid;
            grid-template-columns: repeat(6, 1fr);
            gap: 10px;
            margin-top: 8px;
        }}
        .avatar-item {{
            display: flex;
            align-items: center;
            justify-content: center;
            width: 48px;
            height: 48px;
            font-size: 28px;
            background: #1a1040;
            border: 2px solid #333;
            border-radius: 12px;
            cursor: pointer;
            transition: all 0.2s ease;
        }}
        .avatar-item:hover {{
            background: #2a1850;
            border-color: #d4af37;
            transform: scale(1.1);
        }}
        .avatar-item.selected {{
            background: linear-gradient(135deg, #2a1850, #3a2060);
            border-color: #d4af37;
            box-shadow: 0 0 12px rgba(212, 175, 55, 0.5);
        }}
    </style>
</head>
<body>
    <script>
        // Seasonal theme detection
        (function() {{
            const now = new Date();
            const month = now.getMonth() + 1;
            const day = now.getDate();
            let theme = 'theme-summer';
            if (month === 12 && day <= 25) theme = 'theme-christmas';
            else if ((month === 12 && day >= 26) || (month === 1 && day <= 7)) theme = 'theme-newyear';
            else if (month === 2 && day <= 14) theme = 'theme-valentines';
            else if (month === 3 && day >= 10 && day <= 17) theme = 'theme-stpatricks';
            else if ((month === 3 && day >= 20) || (month === 4 && day <= 20)) theme = 'theme-easter';
            else if ((month === 6 && day >= 25) || (month === 7 && day <= 10)) theme = 'theme-july4th';
            else if (month === 10) theme = 'theme-halloween';
            else if (month === 11 && day >= 15) theme = 'theme-thanksgiving';
            document.body.classList.add(theme);
        }})();
    </script>
    <div class="container">
        <h1>🎡 Holiday Wheel</h1>
        <p class="subtitle">Create your account</p>
        <div class="error" id="error"></div>
        <div class="success" id="success"></div>

        <!-- Passkey Registration -->
        <div class="passkey-section" id="passkeySection">
            <div class="form-group">
                <label for="passkeyDisplayName">Display Name</label>
                <input type="text" id="passkeyDisplayName" name="passkeyDisplayName" placeholder="Your name">
            </div>
            <div class="form-group">
                <label for="passkeyEmail">Email</label>
                <input type="email" id="passkeyEmail" name="passkeyEmail" placeholder="your@email.com">
            </div>
            <button type="button" class="passkey-btn" id="passkeyBtn" onclick="registerWithPasskey()">
                <svg viewBox="0 0 24 24"><path d="M12 1C8.14 1 5 4.14 5 8c0 2.38 1.19 4.47 3 5.74V17a1 1 0 0 0 1 1h1v2a1 1 0 0 0 1 1h2a1 1 0 0 0 1-1v-2h1a1 1 0 0 0 1-1v-3.26c1.81-1.27 3-3.36 3-5.74 0-3.86-3.14-7-7-7zm0 2c2.76 0 5 2.24 5 5s-2.24 5-5 5-5-2.24-5-5 2.24-5 5-5zm0 2a3 3 0 0 0-3 3 3 3 0 0 0 3 3 3 3 0 0 0 3-3 3 3 0 0 0-3-3z"/></svg>
                Create Account with Passkey
            </button>
            <p style="color: #888; font-size: 12px; margin-top: 8px; text-align: center;">
                Secure, passwordless authentication using your device
            </p>
        </div>

        <div class="or-divider"><span>or use email &amp; password</span></div>

        <form id="registerForm">
            <div class="form-group">
                <label for="displayName">Display Name</label>
                <input type="text" id="displayName" name="displayName" required>
            </div>
            <div class="form-group">
                <label for="email">Email</label>
                <input type="email" id="email" name="email" required>
            </div>
            <div class="form-group">
                <label for="password">Password</label>
                <input type="password" id="password" name="password" required minlength="6">
            </div>
            <div class="form-group">
                <label for="confirmPassword">Confirm Password</label>
                <input type="password" id="confirmPassword" name="confirmPassword" required>
            </div>
            <div class="form-group">
                <label>Choose Your Avatar</label>
                <div class="avatar-grid">
                    <div class="avatar-item selected" data-avatar="1" onclick="selectAvatar(1)">🎅</div>
                    <div class="avatar-item" data-avatar="2" onclick="selectAvatar(2)">🤶</div>
                    <div class="avatar-item" data-avatar="3" onclick="selectAvatar(3)">🦌</div>
                    <div class="avatar-item" data-avatar="4" onclick="selectAvatar(4)">⛄</div>
                    <div class="avatar-item" data-avatar="5" onclick="selectAvatar(5)">🎄</div>
                    <div class="avatar-item" data-avatar="6" onclick="selectAvatar(6)">🎁</div>
                    <div class="avatar-item" data-avatar="7" onclick="selectAvatar(7)">🔔</div>
                    <div class="avatar-item" data-avatar="8" onclick="selectAvatar(8)">❄️</div>
                    <div class="avatar-item" data-avatar="9" onclick="selectAvatar(9)">🌟</div>
                    <div class="avatar-item" data-avatar="10" onclick="selectAvatar(10)">🕯️</div>
                    <div class="avatar-item" data-avatar="11" onclick="selectAvatar(11)">🧝</div>
                    <div class="avatar-item" data-avatar="12" onclick="selectAvatar(12)">🤴</div>
                </div>
                <input type="hidden" id="avatarId" name="avatarId" value="1">
            </div>
            <button type="submit" class="full">Create Account</button>
        </form>
        <div class="links">
            <p>Already have an account? <a href="/">Sign In</a></p>
        </div>
    </div>
    <script>
        // Check for WebAuthn support
        if (window.PublicKeyCredential) {{
            PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable().then(available => {{
                if (!available) {{
                    document.getElementById('passkeySection').classList.add('hidden');
                }}
            }}).catch(() => {{
                document.getElementById('passkeySection').classList.add('hidden');
            }});
        }} else {{
            document.getElementById('passkeySection').classList.add('hidden');
        }}

        // Base64URL utilities
        function base64urlToBuffer(base64url) {{
            const base64 = base64url.replace(/-/g, '+').replace(/_/g, '/');
            const pad = base64.length % 4;
            const padded = pad ? base64 + '='.repeat(4 - pad) : base64;
            const binary = atob(padded);
            const buffer = new Uint8Array(binary.length);
            for (let i = 0; i < binary.length; i++) {{
                buffer[i] = binary.charCodeAt(i);
            }}
            return buffer.buffer;
        }}

        function bufferToBase64url(buffer) {{
            const bytes = new Uint8Array(buffer);
            let binary = '';
            for (let i = 0; i < bytes.byteLength; i++) {{
                binary += String.fromCharCode(bytes[i]);
            }}
            return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
        }}

        async function registerWithPasskey() {{
            const errorDiv = document.getElementById('error');
            const successDiv = document.getElementById('success');
            errorDiv.style.display = 'none';
            successDiv.style.display = 'none';

            const email = document.getElementById('passkeyEmail').value.trim();
            const display_name = document.getElementById('passkeyDisplayName').value.trim();

            if (!email) {{
                errorDiv.textContent = 'Please enter your email';
                errorDiv.style.display = 'block';
                return;
            }}
            if (!display_name) {{
                errorDiv.textContent = 'Please enter your display name';
                errorDiv.style.display = 'block';
                return;
            }}

            try {{
                // Start passkey registration
                const startRes = await fetch('/auth/api/passkey/register/start', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ email, display_name }}),
                    credentials: 'include'
                }});
                const startData = await startRes.json();

                if (!startRes.ok || !startData.ok) {{
                    errorDiv.textContent = startData.error || 'Failed to start passkey registration';
                    errorDiv.style.display = 'block';
                    return;
                }}

                const options = startData.options;

                // Convert base64url strings to ArrayBuffers
                const publicKeyOptions = {{
                    challenge: base64urlToBuffer(options.publicKey.challenge),
                    rp: options.publicKey.rp,
                    user: {{
                        id: base64urlToBuffer(options.publicKey.user.id),
                        name: options.publicKey.user.name,
                        displayName: options.publicKey.user.displayName
                    }},
                    pubKeyCredParams: options.publicKey.pubKeyCredParams,
                    timeout: options.publicKey.timeout || 60000,
                    authenticatorSelection: options.publicKey.authenticatorSelection,
                    attestation: options.publicKey.attestation || 'none'
                }};

                if (options.publicKey.excludeCredentials) {{
                    publicKeyOptions.excludeCredentials = options.publicKey.excludeCredentials.map(c => ({{
                        id: base64urlToBuffer(c.id),
                        type: c.type,
                        transports: c.transports
                    }}));
                }}

                // Create credential
                const credential = await navigator.credentials.create({{
                    publicKey: publicKeyOptions
                }});

                // Prepare response for server
                const credentialResponse = {{
                    id: credential.id,
                    rawId: bufferToBase64url(credential.rawId),
                    response: {{
                        clientDataJSON: bufferToBase64url(credential.response.clientDataJSON),
                        attestationObject: bufferToBase64url(credential.response.attestationObject)
                    }},
                    type: credential.type
                }};

                // Finish registration
                const finishRes = await fetch('/auth/api/passkey/register/finish', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ email, credential: credentialResponse }}),
                    credentials: 'include'
                }});
                const finishData = await finishRes.json();

                if (finishRes.ok && finishData.user) {{
                    localStorage.setItem('user', JSON.stringify(finishData.user));
                    if (finishData.token) {{
                        localStorage.setItem('auth_token', finishData.token);
                    }}
                    window.location.href = '/lobby';
                }} else {{
                    errorDiv.textContent = finishData.error || 'Passkey registration failed';
                    errorDiv.style.display = 'block';
                }}
            }} catch (err) {{
                console.error('Passkey error:', err);
                if (err.name === 'NotAllowedError') {{
                    errorDiv.textContent = 'Passkey registration was cancelled';
                }} else if (err.name === 'InvalidStateError') {{
                    errorDiv.textContent = 'A passkey already exists for this device';
                }} else {{
                    errorDiv.textContent = 'Passkey registration failed: ' + err.message;
                }}
                errorDiv.style.display = 'block';
            }}
        }}

        function selectAvatar(avatarId) {{
            // Remove selected class from all avatars
            document.querySelectorAll('.avatar-item').forEach(item => {{
                item.classList.remove('selected');
            }});
            // Add selected class to clicked avatar
            const selectedItem = document.querySelector('.avatar-item[data-avatar="' + avatarId + '"]');
            if (selectedItem) {{
                selectedItem.classList.add('selected');
            }}
            // Update hidden input
            document.getElementById('avatarId').value = avatarId;
        }}

        document.getElementById('registerForm').addEventListener('submit', async (e) => {{
            e.preventDefault();
            const errorDiv = document.getElementById('error');
            errorDiv.style.display = 'none';

            const display_name = document.getElementById('displayName').value;
            const email = document.getElementById('email').value;
            const password = document.getElementById('password').value;
            const avatar_id = parseInt(document.getElementById('avatarId').value, 10);
            const confirmPassword = document.getElementById('confirmPassword').value;

            if (password !== confirmPassword) {{
                errorDiv.textContent = 'Passwords do not match';
                errorDiv.style.display = 'block';
                return;
            }}

            try {{
                const res = await fetch('/auth/api/register', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ email, password, display_name, avatar_id }}),
                    credentials: 'include'
                }});
                const data = await res.json();

                if (res.ok && data.user) {{
                    localStorage.setItem('user', JSON.stringify(data.user));
                    if (data.token) {{
                        localStorage.setItem('auth_token', data.token);
                    }}
                    window.location.href = '/lobby';
                }} else if (res.ok || data.ok) {{
                    // Registration succeeded but needs email verification
                    const successDiv = document.getElementById('success');
                    successDiv.textContent = data.message || 'Registration successful! Check your email to verify.';
                    successDiv.style.display = 'block';
                }} else {{
                    errorDiv.textContent = data.error || 'Registration failed';
                    errorDiv.style.display = 'block';
                }}
            }} catch (err) {{
                errorDiv.textContent = 'Connection error';
                errorDiv.style.display = 'block';
            }}
        }});
    </script>
</body>
</html>"##,
        common_styles = COMMON_STYLES
    ))
}

/// Lobby page
pub async fn lobby() -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Lobby</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Mountains+of+Christmas:wght@700&display=swap" rel="stylesheet">
    <style>
        {common_styles}
        .lobby-header {{
            text-align: center;
            margin-bottom: 24px;
            padding-bottom: 16px;
            border-bottom: 1px solid #333;
        }}
        .lobby-header h1 {{
            font-size: 40px;
            margin-bottom: 8px;
            filter: drop-shadow(0 2px 4px rgba(212, 175, 55, 0.3));
        }}
        .user-row {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-top: 16px;
        }}
        .header-buttons {{
            display: flex;
            gap: 12px;
            align-items: center;
        }}
        .profile-btn {{
            background: none;
            border: 2px solid #d4af37;
            border-radius: 50%;
            width: 44px;
            height: 44px;
            font-size: 24px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.2s;
            padding: 0;
        }}
        .profile-btn:hover {{
            background: rgba(212, 175, 55, 0.2);
            transform: scale(1.05);
        }}
        .hidden {{ display: none !important; }}
        /* Profile Modal Styles */
        .profile-modal-overlay {{
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.8);
            z-index: 1000;
            align-items: center;
            justify-content: center;
        }}
        .profile-modal-overlay.active {{
            display: flex;
        }}
        .profile-modal {{
            background: linear-gradient(180deg, #1a0a3e 0%, #0d0628 100%);
            border: 2px solid #d4af37;
            border-radius: 16px;
            padding: 32px;
            max-width: 420px;
            width: 90%;
            max-height: 90vh;
            overflow-y: auto;
        }}
        .profile-modal h2 {{
            color: #d4af37;
            margin: 0 0 24px 0;
            text-align: center;
            font-size: 28px;
        }}
        .profile-modal .form-group {{
            margin-bottom: 20px;
        }}
        .profile-modal label {{
            display: block;
            color: #aaa;
            margin-bottom: 8px;
            font-size: 14px;
        }}
        .profile-modal input {{
            width: 100%;
            padding: 12px 16px;
            border: 2px solid #333;
            border-radius: 8px;
            background: #0d0628;
            color: #fff;
            font-size: 16px;
        }}
        .profile-modal input:focus {{
            outline: none;
            border-color: #d4af37;
        }}
        .profile-modal input:disabled {{
            background: #1a0a3e;
            color: #888;
            cursor: not-allowed;
        }}
        .avatar-grid {{
            display: grid;
            grid-template-columns: repeat(6, 1fr);
            gap: 8px;
            margin-top: 8px;
        }}
        .avatar-option {{
            width: 100%;
            aspect-ratio: 1;
            border: 2px solid #333;
            border-radius: 8px;
            background: #0d0628;
            font-size: 28px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.2s;
        }}
        .avatar-option:hover {{
            border-color: #d4af37;
            background: rgba(212, 175, 55, 0.1);
            transform: scale(1.05);
        }}
        .avatar-option.selected {{
            border-color: #d4af37;
            background: rgba(212, 175, 55, 0.2);
            box-shadow: 0 0 12px rgba(212, 175, 55, 0.4);
        }}
        .profile-modal-buttons {{
            display: flex;
            gap: 12px;
            margin-top: 24px;
        }}
        .profile-modal-buttons button {{
            flex: 1;
            padding: 14px 24px;
            font-size: 16px;
            font-weight: bold;
            border-radius: 8px;
            cursor: pointer;
            border: none;
        }}
        .profile-save-btn {{
            background: #d4af37;
            color: #0d0628;
        }}
        .profile-save-btn:hover {{
            background: #e5c048;
        }}
        .profile-save-btn:disabled {{
            background: #666;
            cursor: not-allowed;
        }}
        .profile-cancel-btn {{
            background: #333;
            color: #d4af37;
        }}
        .profile-cancel-btn:hover {{
            background: #444;
        }}
        .profile-toast {{
            position: fixed;
            bottom: 20px;
            left: 50%;
            transform: translateX(-50%);
            padding: 12px 24px;
            border-radius: 8px;
            font-weight: 500;
            z-index: 2000;
            animation: slideUp 0.3s ease;
        }}
        .profile-toast.success {{
            background: #27ae60;
            color: #fff;
        }}
        .profile-toast.error {{
            background: #e74c3c;
            color: #fff;
        }}
        @keyframes slideUp {{
            from {{ opacity: 0; transform: translateX(-50%) translateY(20px); }}
            to {{ opacity: 1; transform: translateX(-50%) translateY(0); }}
        }}
        .current-avatar {{
            font-size: 48px;
            text-align: center;
            margin-bottom: 16px;
        }}
        .lobby-content {{
            display: grid;
            grid-template-columns: 1fr 280px;
            gap: 24px;
        }}
        @media (max-width: 768px) {{
            .lobby-content {{
                grid-template-columns: 1fr;
            }}
        }}
        .qr-section {{
            background: #1a0a3e;
            border: 2px solid #333;
            border-radius: 16px;
            padding: 24px;
            text-align: center;
        }}
        .qr-section h3 {{
            color: #d4af37;
            margin-bottom: 16px;
            font-size: 18px;
        }}
        .qr-container {{
            background: #fff;
            padding: 16px;
            border-radius: 8px;
            display: inline-block;
            margin-bottom: 16px;
        }}
        .qr-room-name {{
            color: #d4af37;
            font-size: 16px;
            font-weight: bold;
            margin-bottom: 8px;
        }}
        .qr-hint {{
            color: #888;
            font-size: 12px;
        }}
        .qr-input-row {{
            display: flex;
            gap: 8px;
            margin-bottom: 16px;
        }}
        .qr-input-row input {{
            flex: 1;
            padding: 8px 12px;
            font-size: 14px;
        }}
        .qr-input-row button {{
            padding: 8px 16px;
            font-size: 14px;
        }}
        .sidebar-sections {{
            display: flex;
            flex-direction: column;
            gap: 20px;
        }}
        .passkey-section {{
            background: #1a0a3e;
            border: 2px solid #333;
            border-radius: 16px;
            padding: 24px;
        }}
        .passkey-section h3 {{
            color: #d4af37;
            margin-bottom: 16px;
            font-size: 18px;
        }}
        .passkey-list {{
            max-height: 200px;
            overflow-y: auto;
        }}
        .passkey-item {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px 12px;
            background: rgba(0,0,0,0.3);
            border-radius: 8px;
            margin-bottom: 8px;
        }}
        .passkey-item:last-child {{ margin-bottom: 0; }}
        .passkey-info {{
            flex: 1;
        }}
        .passkey-name {{
            color: #fff;
            font-size: 14px;
            font-weight: 500;
        }}
        .passkey-date {{
            color: #888;
            font-size: 12px;
            margin-top: 2px;
        }}
        .passkey-delete {{
            background: transparent;
            border: none;
            color: #ff6b6b;
            cursor: pointer;
            padding: 4px 8px;
            font-size: 12px;
        }}
        .passkey-delete:hover {{
            color: #ff4444;
            text-decoration: underline;
        }}
        .modal-overlay {{
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0,0,0,0.8);
            align-items: center;
            justify-content: center;
            z-index: 1000;
        }}
        .modal-overlay.active {{
            display: flex;
        }}
        .modal {{
            background: #1a0a3e;
            border: 2px solid #d4af37;
            border-radius: 16px;
            padding: 32px;
            max-width: 400px;
            width: 90%;
        }}
        .modal h2 {{
            color: #d4af37;
            margin-bottom: 16px;
        }}
        .modal-buttons {{
            display: flex;
            gap: 12px;
            margin-top: 20px;
        }}
        .modal-buttons button {{
            flex: 1;
        }}
        .hidden {{ display: none !important; }}
    </style>
</head>
<body>
    <script>
        // Seasonal theme detection
        (function() {{
            const now = new Date();
            const month = now.getMonth() + 1;
            const day = now.getDate();
            let theme = 'theme-summer';
            if (month === 12 && day <= 25) theme = 'theme-christmas';
            else if ((month === 12 && day >= 26) || (month === 1 && day <= 7)) theme = 'theme-newyear';
            else if (month === 2 && day <= 14) theme = 'theme-valentines';
            else if (month === 3 && day >= 10 && day <= 17) theme = 'theme-stpatricks';
            else if ((month === 3 && day >= 20) || (month === 4 && day <= 20)) theme = 'theme-easter';
            else if ((month === 6 && day >= 25) || (month === 7 && day <= 10)) theme = 'theme-july4th';
            else if (month === 10) theme = 'theme-halloween';
            else if (month === 11 && day >= 15) theme = 'theme-thanksgiving';
            document.body.classList.add(theme);
        }})();
    </script>
    <div class="container wide">
        <div class="lobby-header">
            <h1>🎡 Holiday Wheel</h1>
            <div class="user-row">
                <span>Welcome, <span class="user-name" id="userName">Player</span>!</span>
                <div class="header-buttons">
                    <button class="profile-btn" id="profileBtn" onclick="openProfileModal()" title="Edit Profile"></button>
                    <a href="/admin" id="adminBtn" class="btn hidden" style="background:#5856d6;">Admin</a>
                    <button class="btn btn-secondary" onclick="logout()">Logout</button>
                </div>
            </div>
        </div>

        <div class="lobby-content">
            <div class="main-section">
                <h2 style="color: #fff; margin-bottom: 16px;">Active Rooms</h2>
                <div class="rooms-grid" id="roomsGrid">
                    <p style="color: #888;">Loading rooms...</p>
                </div>

                <div class="join-form">
                    <input type="text" id="roomName" placeholder="Enter room name" value="main" oninput="updateQRCode()">
                    <button class="btn" onclick="joinRoom()">Join Room</button>
                </div>
            </div>

            <div class="sidebar-sections">
                <div class="qr-section">
                    <h3>📱 Phone Connection</h3>
                    <div class="qr-input-row">
                        <input type="text" id="qrRoomName" value="main" placeholder="Room name" oninput="updateQRCode()">
                        <button class="btn btn-secondary" onclick="updateQRCode()">Update</button>
                    </div>
                    <div class="qr-container">
                        <div id="qrCode"></div>
                    </div>
                    <div class="qr-room-name">Room: <span id="qrRoomDisplay">main</span></div>
                    <div class="qr-hint">Scan with phone app to join as controller</div>
                </div>

                <div class="passkey-section" id="passkeySection">
                    <h3>🔐 Passkeys</h3>
                    <div id="passkeyList" class="passkey-list">
                        <p style="color: #888; font-size: 14px;">Loading...</p>
                    </div>
                    <button class="btn btn-secondary" id="addPasskeyBtn" onclick="addPasskey()" style="width: 100%; margin-top: 12px; font-size: 14px;">
                        + Add Passkey
                    </button>
                </div>
            </div>
        </div>
    </div>

    <!-- Add Passkey Modal -->
    <div class="modal-overlay" id="addPasskeyModal">
        <div class="modal">
            <h2>Add Passkey</h2>
            <p style="color: #aaa; margin-bottom: 16px;">Register a new passkey for passwordless login.</p>
            <div class="form-group">
                <label for="deviceName">Device Name (optional)</label>
                <input type="text" id="deviceName" placeholder="e.g., MacBook Pro, iPhone">
            </div>
            <div class="modal-buttons">
                <button class="btn" onclick="confirmAddPasskey()">Add Passkey</button>
                <button class="btn btn-secondary" onclick="closeModal('addPasskeyModal')">Cancel</button>
            </div>
        </div>
    </div>

    <!-- Profile Modal -->
    <div class="profile-modal-overlay" id="profileModal">
        <div class="profile-modal">
            <h2>Edit Profile</h2>
            <div class="current-avatar" id="currentAvatarDisplay"></div>
            <div class="form-group">
                <label for="profileDisplayName">Display Name</label>
                <input type="text" id="profileDisplayName" placeholder="Enter your display name" maxlength="30">
            </div>
            <div class="form-group">
                <label for="profileEmail">Email</label>
                <input type="email" id="profileEmail" disabled>
            </div>
            <div class="form-group">
                <label>Choose Avatar</label>
                <div class="avatar-grid" id="avatarGrid"></div>
            </div>
            <div class="profile-modal-buttons">
                <button class="profile-save-btn" id="profileSaveBtn" onclick="saveProfile()">Save Changes</button>
                <button class="profile-cancel-btn" onclick="closeProfileModal()">Cancel</button>
            </div>
        </div>
    </div>
    <script src="https://cdn.socket.io/4.7.5/socket.io.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/qrcodejs@1.0.0/qrcode.min.js"></script>
    <script>
        // Handle OAuth callback (Apple Sign-In returns with hash fragment)
        if (window.location.hash) {{
            const params = new URLSearchParams(window.location.hash.substring(1));
            const authToken = params.get('auth_token');
            const userEncoded = params.get('user');
            if (authToken && userEncoded) {{
                try {{
                    const user = JSON.parse(decodeURIComponent(userEncoded));
                    localStorage.setItem('user', JSON.stringify(user));
                    // Clear the hash
                    history.replaceState(null, '', window.location.pathname);
                }} catch (e) {{
                    console.error('Failed to parse user data:', e);
                }}
            }}
        }}

        // Check auth - user info stored in localStorage
        const user = JSON.parse(localStorage.getItem('user') || 'null');

        if (!user) {{
            window.location.href = '/';
        }} else {{
            document.getElementById('userName').textContent = user.display_name || user.email;
        }}

        // ========== PROFILE MANAGEMENT ==========
        const AVATARS = ['🎅', '🤶', '🦌', '⛄', '🎄', '🎁', '🔔', '❄️', '🌟', '🕯️', '🧝', '🤴'];
        let selectedAvatarId = user?.avatar_id || 1;

        // Initialize profile button with current avatar
        function initProfileButton() {{
            const avatarId = user?.avatar_id || 1;
            const avatarEmoji = AVATARS[(avatarId - 1) % AVATARS.length] || '🎅';
            document.getElementById('profileBtn').textContent = avatarEmoji;
        }}
        initProfileButton();

        // Initialize avatar grid
        function initAvatarGrid() {{
            const grid = document.getElementById('avatarGrid');
            grid.innerHTML = AVATARS.map((emoji, index) => `
                <button class="avatar-option ${{(index + 1) === selectedAvatarId ? 'selected' : ''}}"
                        data-avatar-id="${{index + 1}}"
                        onclick="selectAvatar(${{index + 1}})">
                    ${{emoji}}
                </button>
            `).join('');
        }}

        function selectAvatar(avatarId) {{
            selectedAvatarId = avatarId;
            // Update selected state in grid
            document.querySelectorAll('.avatar-option').forEach(btn => {{
                btn.classList.toggle('selected', parseInt(btn.dataset.avatarId) === avatarId);
            }});
            // Update current avatar display
            document.getElementById('currentAvatarDisplay').textContent = AVATARS[(avatarId - 1) % AVATARS.length];
        }}

        function openProfileModal() {{
            // Reset to current user values
            selectedAvatarId = user?.avatar_id || 1;
            document.getElementById('profileDisplayName').value = user?.display_name || '';
            document.getElementById('profileEmail').value = user?.email || '';
            document.getElementById('currentAvatarDisplay').textContent = AVATARS[(selectedAvatarId - 1) % AVATARS.length];
            initAvatarGrid();
            document.getElementById('profileModal').classList.add('active');
        }}

        function closeProfileModal() {{
            document.getElementById('profileModal').classList.remove('active');
        }}

        function showProfileToast(message, type) {{
            // Remove any existing toast
            const existing = document.querySelector('.profile-toast');
            if (existing) existing.remove();

            const toast = document.createElement('div');
            toast.className = `profile-toast ${{type}}`;
            toast.textContent = message;
            document.body.appendChild(toast);

            setTimeout(() => toast.remove(), 3000);
        }}

        async function saveProfile() {{
            const saveBtn = document.getElementById('profileSaveBtn');
            const displayName = document.getElementById('profileDisplayName').value.trim();

            if (!displayName) {{
                showProfileToast('Please enter a display name', 'error');
                return;
            }}

            // Disable save button and show loading
            saveBtn.disabled = true;
            saveBtn.textContent = 'Saving...';

            try {{
                const res = await fetch('/auth/api/profile', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        display_name: displayName,
                        avatar_id: selectedAvatarId
                    }}),
                    credentials: 'include'
                }});

                const data = await res.json();

                if (res.ok && data.ok && data.user) {{
                    // Update local storage with new user data
                    const updatedUser = {{
                        ...user,
                        display_name: data.user.display_name,
                        avatar_id: data.user.avatar_id
                    }};
                    localStorage.setItem('user', JSON.stringify(updatedUser));

                    // Update the global user object
                    Object.assign(user, updatedUser);

                    // Update UI
                    document.getElementById('userName').textContent = data.user.display_name || data.user.email;
                    initProfileButton();

                    closeProfileModal();
                    showProfileToast('Profile updated successfully!', 'success');
                }} else {{
                    showProfileToast(data.error || 'Failed to update profile', 'error');
                }}
            }} catch (err) {{
                console.error('Profile update error:', err);
                showProfileToast('Failed to update profile. Please try again.', 'error');
            }} finally {{
                saveBtn.disabled = false;
                saveBtn.textContent = 'Save Changes';
            }}
        }}

        // Check admin status
        async function checkAdmin() {{
            try {{
                const res = await fetch('/auth/api/admin/users', {{
                    credentials: 'include'
                }});
                if (res.ok) {{
                    // User is admin - show admin button
                    document.getElementById('adminBtn').classList.remove('hidden');
                }}
            }} catch (e) {{
                // Not admin or error - keep button hidden
            }}
        }}
        checkAdmin();

        // QR Code generation
        let qrCodeInstance = null;
        function updateQRCode() {{
            const roomName = document.getElementById('qrRoomName').value || 'main';
            const serverUrl = window.location.origin;
            // Use web URL with fallback to app - works whether app is installed or not
            const joinUrl = `${{serverUrl}}/join?room=${{encodeURIComponent(roomName)}}`;

            document.getElementById('qrRoomDisplay').textContent = roomName;
            document.getElementById('roomName').value = roomName;

            const qrContainer = document.getElementById('qrCode');
            qrContainer.innerHTML = '';

            try {{
                qrCodeInstance = new QRCode(qrContainer, {{
                    text: joinUrl,
                    width: 160,
                    height: 160,
                    colorDark: '#1a0a3e',
                    colorLight: '#ffffff',
                    correctLevel: QRCode.CorrectLevel.M
                }});
            }} catch (e) {{
                console.error('QR Code error:', e);
                qrContainer.innerHTML = '<p style="color:#666;font-size:12px;">QR generation failed</p>';
            }}
        }}

        // Sync room name inputs
        document.getElementById('roomName').addEventListener('input', function() {{
            document.getElementById('qrRoomName').value = this.value;
            updateQRCode();
        }});

        // Load rooms
        async function loadRooms() {{
            try {{
                const res = await fetch('/auth/api/rooms', {{
                    credentials: 'include'
                }});
                const data = await res.json();

                const grid = document.getElementById('roomsGrid');
                if (data.rooms && data.rooms.length > 0) {{
                    grid.innerHTML = data.rooms.map(room => `
                        <div class="room-card" onclick="joinRoom('${{room.name}}')">
                            <div class="room-name">${{room.name}}</div>
                            <div class="room-players">${{room.player_count}} players</div>
                        </div>
                    `).join('');
                }} else {{
                    grid.innerHTML = '<p style="color: #888;">No active rooms. Create one below!</p>';
                }}
            }} catch (err) {{
                console.error('Failed to load rooms:', err);
            }}
        }}

        function joinRoom(name) {{
            const roomName = name || document.getElementById('roomName').value || 'main';
            window.location.href = '/game?room=' + encodeURIComponent(roomName);
        }}

        async function logout() {{
            // Clear server cookie
            try {{
                await fetch('/auth/logout', {{
                    method: 'POST',
                    credentials: 'include'
                }});
            }} catch (e) {{
                // Continue with local logout even if server call fails
            }}
            localStorage.removeItem('user');
            localStorage.removeItem('auth_token');
            window.location.href = '/';
        }}

        // Initial load and refresh every 5 seconds
        loadRooms();
        setInterval(loadRooms, 5000);

        // Generate initial QR code
        updateQRCode();

        // ========== PASSKEY MANAGEMENT ==========

        // Check for WebAuthn support
        if (!window.PublicKeyCredential) {{
            document.getElementById('passkeySection').classList.add('hidden');
        }}

        // Base64URL utilities
        function base64urlToBuffer(base64url) {{
            const base64 = base64url.replace(/-/g, '+').replace(/_/g, '/');
            const pad = base64.length % 4;
            const padded = pad ? base64 + '='.repeat(4 - pad) : base64;
            const binary = atob(padded);
            const buffer = new Uint8Array(binary.length);
            for (let i = 0; i < binary.length; i++) {{
                buffer[i] = binary.charCodeAt(i);
            }}
            return buffer.buffer;
        }}

        function bufferToBase64url(buffer) {{
            const bytes = new Uint8Array(buffer);
            let binary = '';
            for (let i = 0; i < bytes.byteLength; i++) {{
                binary += String.fromCharCode(bytes[i]);
            }}
            return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
        }}

        // Load user's passkeys
        async function loadPasskeys() {{
            try {{
                const res = await fetch('/auth/api/passkey/list', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json'
                    }},
                    credentials: 'include'
                }});
                const data = await res.json();

                const list = document.getElementById('passkeyList');
                if (data.ok && data.passkeys && data.passkeys.length > 0) {{
                    list.innerHTML = data.passkeys.map(pk => `
                        <div class="passkey-item">
                            <div class="passkey-info">
                                <div class="passkey-name">${{pk.device_name || 'Passkey'}}</div>
                                <div class="passkey-date">Added ${{formatDate(pk.created_at)}}</div>
                            </div>
                            <button class="passkey-delete" onclick="deletePasskey('${{pk.id}}')">Delete</button>
                        </div>
                    `).join('');
                }} else {{
                    list.innerHTML = '<p style="color: #888; font-size: 14px;">No passkeys registered</p>';
                }}
            }} catch (err) {{
                console.error('Failed to load passkeys:', err);
                document.getElementById('passkeyList').innerHTML = '<p style="color: #888; font-size: 14px;">Failed to load</p>';
            }}
        }}

        function formatDate(timestamp) {{
            const date = new Date(timestamp * 1000);
            return date.toLocaleDateString();
        }}

        function closeModal(modalId) {{
            document.getElementById(modalId).classList.remove('active');
        }}

        function addPasskey() {{
            document.getElementById('deviceName').value = '';
            document.getElementById('addPasskeyModal').classList.add('active');
        }}

        async function confirmAddPasskey() {{
            const deviceName = document.getElementById('deviceName').value.trim() || null;
            closeModal('addPasskeyModal');

            try {{
                // Start passkey registration
                const startRes = await fetch('/auth/api/passkey/add/start', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json'
                    }},
                    body: JSON.stringify({{ device_name: deviceName }}),
                    credentials: 'include'
                }});
                const startData = await startRes.json();

                if (!startRes.ok || !startData.ok) {{
                    alert(startData.error || 'Failed to start passkey registration');
                    return;
                }}

                const options = startData.options;

                // Convert base64url strings to ArrayBuffers
                const publicKeyOptions = {{
                    challenge: base64urlToBuffer(options.publicKey.challenge),
                    rp: options.publicKey.rp,
                    user: {{
                        id: base64urlToBuffer(options.publicKey.user.id),
                        name: options.publicKey.user.name,
                        displayName: options.publicKey.user.displayName
                    }},
                    pubKeyCredParams: options.publicKey.pubKeyCredParams,
                    timeout: options.publicKey.timeout || 60000,
                    authenticatorSelection: options.publicKey.authenticatorSelection,
                    attestation: options.publicKey.attestation || 'none'
                }};

                if (options.publicKey.excludeCredentials) {{
                    publicKeyOptions.excludeCredentials = options.publicKey.excludeCredentials.map(c => ({{
                        id: base64urlToBuffer(c.id),
                        type: c.type,
                        transports: c.transports
                    }}));
                }}

                // Create credential
                const credential = await navigator.credentials.create({{
                    publicKey: publicKeyOptions
                }});

                // Prepare response for server
                const credentialResponse = {{
                    id: credential.id,
                    rawId: bufferToBase64url(credential.rawId),
                    response: {{
                        clientDataJSON: bufferToBase64url(credential.response.clientDataJSON),
                        attestationObject: bufferToBase64url(credential.response.attestationObject)
                    }},
                    type: credential.type
                }};

                // Finish registration
                const finishRes = await fetch('/auth/api/passkey/add/finish', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json'
                    }},
                    body: JSON.stringify({{ email: user.email, credential: credentialResponse }}),
                    credentials: 'include'
                }});
                const finishData = await finishRes.json();

                if (finishRes.ok && finishData.ok) {{
                    loadPasskeys();
                }} else {{
                    alert(finishData.error || 'Failed to add passkey');
                }}
            }} catch (err) {{
                console.error('Passkey error:', err);
                if (err.name === 'NotAllowedError') {{
                    alert('Passkey registration was cancelled');
                }} else if (err.name === 'InvalidStateError') {{
                    alert('A passkey already exists for this device');
                }} else {{
                    alert('Passkey registration failed: ' + err.message);
                }}
            }}
        }}

        async function deletePasskey(credentialId) {{
            if (!confirm('Are you sure you want to delete this passkey?')) {{
                return;
            }}

            try {{
                const res = await fetch('/auth/api/passkey/delete', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json'
                    }},
                    body: JSON.stringify({{ credential_id: credentialId }}),
                    credentials: 'include'
                }});
                const data = await res.json();

                if (res.ok && data.ok) {{
                    loadPasskeys();
                }} else {{
                    alert(data.error || 'Failed to delete passkey');
                }}
            }} catch (err) {{
                console.error('Delete passkey error:', err);
                alert('Failed to delete passkey');
            }}
        }}

        // Load passkeys on page load
        loadPasskeys();

        // Close modal on escape
        document.addEventListener('keydown', (e) => {{
            if (e.key === 'Escape') {{
                closeModal('addPasskeyModal');
                closeProfileModal();
            }}
        }});

        // Close profile modal on backdrop click
        document.getElementById('profileModal').addEventListener('click', (e) => {{
            if (e.target.id === 'profileModal') {{
                closeProfileModal();
            }}
        }});
    </script>
</body>
</html>"#,
        common_styles = COMMON_STYLES
    ))
}

/// Game page
pub async fn game() -> Html<String> {
    Html(format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Game</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Mountains+of+Christmas:wght@700&display=swap" rel="stylesheet">
    <style>
        {common_styles}
        body {{ align-items: flex-start; padding: 20px; }}

        /* ========== MODERN THEME COLORS ========== */
        :root {{
            /* Seasonal theme defaults (overridden by body class) */
            --theme-color-1: #ff4444;
            --theme-color-2: #d4af37;
            --theme-color-3: #22c55e;
            --theme-decorator: "❄";
            --theme-decorator-color: #87ceeb;
            /* App colors */
            --color-primary: #d4af37;
            --color-primary-light: #ffd700;
            --color-primary-dark: #b8860b;
            --color-primary-glow: rgba(212, 175, 55, 0.4);
            --color-accent: #6366f1;
            --color-accent-glow: rgba(99, 102, 241, 0.4);
            --color-success: #22c55e;
            --color-success-glow: rgba(34, 197, 94, 0.4);
            --color-danger: #ef4444;
            --color-danger-glow: rgba(239, 68, 68, 0.4);
            --color-warning: #f59e0b;
            --color-background: #0d0628;
            --color-surface: #1a0a3e;
            --color-surface-light: #2a1a4e;
            --color-border: #333;
            --color-text: #ffffff;
            --color-text-muted: rgba(255, 255, 255, 0.5);
            --color-board-bg: #1a5cb8;
            --color-empty-cell: #228b22;
        }}
        /* Seasonal Theme Classes */
        body.theme-christmas {{ --theme-color-1: #ff4444; --theme-color-2: #d4af37; --theme-color-3: #22c55e; --theme-decorator: "❄"; --theme-decorator-color: #87ceeb; }}
        body.theme-newyear {{ --theme-color-1: #ffd700; --theme-color-2: #c0c0c0; --theme-color-3: #ffffff; --theme-decorator: "✨"; --theme-decorator-color: #ffd700; }}
        body.theme-valentines {{ --theme-color-1: #ff69b4; --theme-color-2: #ff1493; --theme-color-3: #ffffff; --theme-decorator: "❤"; --theme-decorator-color: #ff69b4; }}
        body.theme-stpatricks {{ --theme-color-1: #22c55e; --theme-color-2: #ffd700; --theme-color-3: #16a34a; --theme-decorator: "☘"; --theme-decorator-color: #22c55e; }}
        body.theme-easter {{ --theme-color-1: #ffb6c1; --theme-color-2: #98fb98; --theme-color-3: #dda0dd; --theme-decorator: "🐣"; --theme-decorator-color: #ffeb3b; }}
        body.theme-july4th {{ --theme-color-1: #ff4444; --theme-color-2: #ffffff; --theme-color-3: #3b82f6; --theme-decorator: "🎆"; --theme-decorator-color: #ff4444; }}
        body.theme-halloween {{ --theme-color-1: #ff6b00; --theme-color-2: #9333ea; --theme-color-3: #1a1a1a; --theme-decorator: "🎃"; --theme-decorator-color: #ff6b00; }}
        body.theme-thanksgiving {{ --theme-color-1: #d97706; --theme-color-2: #92400e; --theme-color-3: #fbbf24; --theme-decorator: "🦃"; --theme-decorator-color: #d97706; }}
        body.theme-summer {{ --theme-color-1: #f59e0b; --theme-color-2: #ef4444; --theme-color-3: #eab308; --theme-decorator: "☀"; --theme-decorator-color: #fbbf24; }}

        .game-container {{
            width: 100%;
            max-width: 1200px;
            display: grid;
            grid-template-columns: 1fr 300px;
            gap: 20px;
        }}
        .main-area {{
            background: rgba(26, 10, 62, 0.8);
            backdrop-filter: blur(10px);
            -webkit-backdrop-filter: blur(10px);
            border-radius: 16px;
            border: 1px solid rgba(255, 255, 255, 0.1);
            padding: 24px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        }}
        .sidebar {{
            background: rgba(26, 10, 62, 0.8);
            backdrop-filter: blur(10px);
            -webkit-backdrop-filter: blur(10px);
            border-radius: 16px;
            border: 1px solid rgba(255, 255, 255, 0.1);
            padding: 24px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        }}

        /* ========== PUZZLE BOARD ========== */
        .puzzle-board {{
            background: linear-gradient(180deg, #2070d0 0%, #1a5cb8 50%, #1450a0 100%);
            border-radius: 12px;
            padding: 16px 8px;
            margin: 20px 0;
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 4px;
            border: 4px solid var(--color-primary);
            box-shadow:
                0 0 30px var(--color-primary-glow),
                0 0 60px rgba(212, 175, 55, 0.15),
                inset 0 2px 4px rgba(255, 255, 255, 0.15),
                inset 0 -4px 8px rgba(0, 0, 0, 0.2);
            position: relative;
            overflow: visible;
        }}
        /* TV-style inner light reflection */
        .puzzle-board::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 50%;
            background: linear-gradient(180deg, rgba(255,255,255,0.08) 0%, transparent 100%);
            border-radius: 8px 8px 0 0;
            pointer-events: none;
        }}
        .puzzle-row {{
            display: flex;
            justify-content: center;
            gap: 3px;
        }}
        .letter-tile {{
            width: 42px;
            height: 52px;
            background: linear-gradient(180deg, #ffffff 0%, #e8e8e8 100%);
            border-radius: 4px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 28px;
            font-weight: bold;
            color: #000;
            border: 2px solid #0e4a8f;
            box-shadow:
                inset 0 1px 0 rgba(255,255,255,0.8),
                inset 0 -2px 3px rgba(0,0,0,0.1),
                0 3px 6px rgba(0,0,0,0.3),
                0 1px 2px rgba(0,0,0,0.2);
            text-shadow: 0 1px 0 rgba(255,255,255,0.5);
            perspective: 1000px;
            transform-style: preserve-3d;
            transition: transform 0.6s, box-shadow 0.3s;
            position: relative;
        }}
        /* Hidden letter tile - white for unrevealed letters */
        .letter-tile.hidden {{
            background: linear-gradient(180deg, #ffffff 0%, #f5f5f5 50%, #e8e8e8 100%);
            color: transparent;
            border-color: #c0c0c0;
            box-shadow:
                inset 0 2px 4px rgba(255,255,255,0.5),
                inset 0 -3px 6px rgba(0,0,0,0.15),
                0 4px 8px rgba(0,0,0,0.3),
                0 0 0 1px rgba(255,255,255,0.3);
        }}
        .letter-tile.hidden::before {{
            content: '';
            position: absolute;
            top: 3px;
            left: 3px;
            right: 3px;
            bottom: 50%;
            background: linear-gradient(180deg, rgba(255,255,255,0.2) 0%, transparent 100%);
            border-radius: 2px 2px 0 0;
            pointer-events: none;
        }}
        .letter-tile.blank {{
            background: linear-gradient(180deg, #3da85e 0%, #2d8f4e 50%, #1e7a3d 100%);
            border-color: #1a5c32;
            box-shadow:
                inset 0 1px 0 rgba(255,255,255,0.3),
                inset 0 -2px 3px rgba(0,0,0,0.2),
                0 3px 6px rgba(0,0,0,0.3),
                0 1px 2px rgba(0,0,0,0.2);
        }}
        .letter-tile.revealed {{
            color: #1a1a1a;
            text-shadow: 0 1px 0 rgba(255,255,255,0.8);
            background: linear-gradient(180deg, #ffffff 0%, #f5f5f5 50%, #e8e8e8 100%);
        }}

        /* ========== VANNA-STYLE LETTER REVEAL ANIMATIONS ========== */

        /* Main reveal animation with dramatic flip */
        .letter-tile.revealing {{
            animation: letterReveal 0.7s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
        }}
        @keyframes letterReveal {{
            0% {{
                transform: rotateY(180deg) scale(0.8);
                opacity: 0;
                filter: brightness(0.5);
            }}
            40% {{
                transform: rotateY(-20deg) scale(1.1);
                opacity: 1;
                filter: brightness(2);
            }}
            60% {{
                transform: rotateY(10deg) scale(1.05);
                filter: brightness(1.5);
            }}
            80% {{
                transform: rotateY(-5deg) scale(1.02);
            }}
            100% {{
                transform: rotateY(0deg) scale(1);
                opacity: 1;
                filter: brightness(1);
            }}
        }}

        /* Glow effect for revealing letters */
        .letter-tile.revealing::after {{
            content: '';
            position: absolute;
            inset: -8px;
            background: radial-gradient(ellipse at center, rgba(212, 175, 55, 0.8) 0%, rgba(212, 175, 55, 0) 70%);
            border-radius: 8px;
            animation: revealGlow 0.7s ease-out forwards;
            pointer-events: none;
            z-index: -1;
        }}
        @keyframes revealGlow {{
            0% {{ opacity: 0; transform: scale(0.5); }}
            40% {{ opacity: 1; transform: scale(1.3); }}
            100% {{ opacity: 0; transform: scale(1); }}
        }}

        /* Just revealed - subtle glow persists */
        .letter-tile.just-revealed {{
            box-shadow:
                0 0 20px rgba(212, 175, 55, 0.4),
                0 0 10px rgba(255, 255, 255, 0.3),
                inset 0 1px 0 rgba(255,255,255,0.9),
                0 3px 6px rgba(0,0,0,0.3);
            background: linear-gradient(180deg, #fffef5 0%, #fff8e6 50%, #f0e6c8 100%);
            animation: gentlePulse 2s ease-in-out infinite;
        }}
        @keyframes gentlePulse {{
            0%, 100% {{ box-shadow: 0 0 15px rgba(212, 175, 55, 0.3), 0 3px 6px rgba(0,0,0,0.3); }}
            50% {{ box-shadow: 0 0 25px rgba(212, 175, 55, 0.5), 0 3px 6px rgba(0,0,0,0.3); }}
        }}

        /* Vowel purchase reveal - blue/purple flash */
        .letter-tile.revealing.vowel-reveal::after {{
            background: radial-gradient(ellipse at center, rgba(100, 149, 237, 0.8) 0%, rgba(138, 43, 226, 0.4) 50%, transparent 70%);
        }}

        /* Final letter reveal - extra celebration */
        .letter-tile.revealing.final-letter {{
            animation: finalLetterReveal 1s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
        }}
        @keyframes finalLetterReveal {{
            0% {{
                transform: rotateY(180deg) scale(0.5);
                opacity: 0;
            }}
            30% {{
                transform: rotateY(-30deg) scale(1.3);
                opacity: 1;
                filter: brightness(2.5);
            }}
            50% {{
                transform: rotateY(15deg) scale(1.15);
                filter: brightness(1.8);
            }}
            70% {{
                transform: rotateY(-8deg) scale(1.08);
            }}
            85% {{
                transform: rotateY(4deg) scale(1.03);
            }}
            100% {{
                transform: rotateY(0deg) scale(1);
                filter: brightness(1);
            }}
        }}
        .letter-tile.revealing.final-letter::after {{
            background: radial-gradient(ellipse at center, rgba(255, 215, 0, 1) 0%, rgba(255, 140, 0, 0.6) 40%, transparent 70%);
            animation: finalGlow 1s ease-out forwards;
        }}
        @keyframes finalGlow {{
            0% {{ opacity: 0; transform: scale(0.3); }}
            30% {{ opacity: 1; transform: scale(1.8); }}
            100% {{ opacity: 0; transform: scale(1); }}
        }}

        /* Multi-letter cascade effect */
        .letter-tile.revealing.cascade {{
            animation: cascadeReveal 0.6s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
        }}
        @keyframes cascadeReveal {{
            0% {{
                transform: rotateY(180deg) translateY(-10px) scale(0.9);
                opacity: 0;
            }}
            50% {{
                transform: rotateY(-15deg) translateY(5px) scale(1.08);
                opacity: 1;
                filter: brightness(1.8);
            }}
            100% {{
                transform: rotateY(0deg) translateY(0) scale(1);
                filter: brightness(1);
            }}
        }}

        /* ========== LETTER VALUE POPUP ========== */
        .letter-value-popup {{
            position: fixed;
            pointer-events: none;
            z-index: 1000;
            font-family: 'Segoe UI', sans-serif;
            font-weight: bold;
            font-size: 28px;
            color: #ffd700;
            text-shadow:
                0 0 10px rgba(255, 215, 0, 0.8),
                0 2px 4px rgba(0,0,0,0.5),
                0 0 20px rgba(255, 215, 0, 0.4);
            animation: valuePopup 1.5s ease-out forwards;
            white-space: nowrap;
        }}
        .letter-value-popup.vowel {{
            color: #87ceeb;
            text-shadow:
                0 0 10px rgba(135, 206, 235, 0.8),
                0 2px 4px rgba(0,0,0,0.5),
                0 0 20px rgba(135, 206, 235, 0.4);
        }}
        @keyframes valuePopup {{
            0% {{
                opacity: 0;
                transform: translateY(0) scale(0.5);
            }}
            20% {{
                opacity: 1;
                transform: translateY(-20px) scale(1.2);
            }}
            40% {{
                transform: translateY(-40px) scale(1);
            }}
            100% {{
                opacity: 0;
                transform: translateY(-80px) scale(0.8);
            }}
        }}

        /* ========== CATEGORY BANNER ========== */
        .category {{
            background: linear-gradient(180deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
            color: #ffd700;
            text-align: center;
            font-size: 20px;
            font-weight: bold;
            padding: 12px 24px;
            margin-bottom: 16px;
            border-radius: 8px;
            border: 2px solid #d4af37;
            box-shadow:
                0 4px 15px rgba(0,0,0,0.4),
                inset 0 1px 0 rgba(255,255,255,0.1),
                0 0 20px rgba(212, 175, 55, 0.2);
            text-shadow: 0 2px 4px rgba(0,0,0,0.5);
            letter-spacing: 2px;
            text-transform: uppercase;
            position: relative;
            overflow: hidden;
        }}
        .category::before {{
            content: '';
            position: absolute;
            top: 0;
            left: -100%;
            width: 50%;
            height: 100%;
            background: linear-gradient(90deg, transparent, rgba(255,255,255,0.1), transparent);
            animation: categoryShine 4s ease-in-out infinite;
        }}
        @keyframes categoryShine {{
            0%, 100% {{ left: -100%; }}
            50% {{ left: 150%; }}
        }}
        .category span {{
            color: #fff;
            font-weight: normal;
            letter-spacing: 1px;
        }}
        .game-layout {{
            display: flex;
            gap: 24px;
            align-items: flex-start;
        }}
        .wheel-area {{
            display: flex;
            flex-direction: column;
            align-items: center;
            flex-shrink: 0;
            perspective: 800px;
        }}
        .wheel-container {{
            position: relative;
            display: flex;
            flex-direction: column;
            align-items: center;
            width: min(380px, 32vw);
            height: min(380px, 32vw);
            min-width: 280px;
            min-height: 280px;
            transform-style: preserve-3d;
            transform: rotateX(35deg);
        }}
        .wheel-outer-rim {{
            position: absolute;
            top: 50%;
            left: 50%;
            width: 102%;
            height: 102%;
            transform: translate(-50%, -50%);
            border-radius: 50%;
            background: linear-gradient(135deg, #c0c0c0 0%, #808080 25%, #c0c0c0 50%, #606060 75%, #909090 100%);
            box-shadow:
                inset 0 2px 4px rgba(255,255,255,0.3),
                inset 0 -2px 4px rgba(0,0,0,0.3),
                0 8px 20px rgba(0,0,0,0.5),
                0 4px 8px rgba(0,0,0,0.3);
            z-index: 0;
        }}
        .wheel-inner-rim {{
            position: absolute;
            top: 50%;
            left: 50%;
            width: 96%;
            height: 96%;
            transform: translate(-50%, -50%);
            border-radius: 50%;
            border: 3px solid #404040;
            box-shadow: inset 0 0 10px rgba(0,0,0,0.5);
            z-index: 1;
            pointer-events: none;
        }}
        .wheel-center-hub {{
            position: absolute;
            top: 50%;
            left: 50%;
            width: 60px;
            height: 60px;
            transform: translate(-50%, -50%);
            border-radius: 50%;
            background: linear-gradient(145deg, #e8e8e8 0%, #b0b0b0 30%, #888888 70%, #606060 100%);
            box-shadow:
                inset 0 2px 4px rgba(255,255,255,0.5),
                inset 0 -2px 4px rgba(0,0,0,0.3),
                0 4px 8px rgba(0,0,0,0.4);
            z-index: 20;
            border: 2px solid #505050;
        }}
        .wheel-center-hub::after {{
            content: '';
            position: absolute;
            top: 50%;
            left: 50%;
            width: 20px;
            height: 20px;
            transform: translate(-50%, -50%);
            border-radius: 50%;
            background: linear-gradient(145deg, #ffd700 0%, #d4af37 50%, #b8960c 100%);
            box-shadow: inset 0 1px 2px rgba(255,255,255,0.5);
        }}
        .wheel-pointer {{
            position: absolute;
            top: -12px;
            z-index: 30;
            width: 0;
            height: 0;
            border-left: 18px solid transparent;
            border-right: 18px solid transparent;
            border-top: 32px solid #d4af37;
            filter: drop-shadow(0 3px 4px rgba(0,0,0,0.5));
        }}
        .wheel-pointer::before {{
            content: '';
            position: absolute;
            top: -32px;
            left: -14px;
            border-left: 14px solid transparent;
            border-right: 14px solid transparent;
            border-top: 26px solid #ffd700;
        }}
        .wheel-svg {{
            width: 94%;
            height: 94%;
            position: relative;
            z-index: 2;
            filter: drop-shadow(0 4px 8px rgba(0,0,0,0.4));
        }}
        .wheel-result {{
            font-family: 'Mountains of Christmas', cursive;
            font-size: clamp(28px, 3vw, 42px);
            font-weight: 700;
            color: #d4af37;
            margin-top: 12px;
            text-shadow: 0 2px 4px rgba(0,0,0,0.5), 0 0 20px rgba(212, 175, 55, 0.3);
            text-align: center;
            letter-spacing: 2px;
            transition: all 0.3s ease;
        }}
        .wheel-result.big-win {{
            font-size: clamp(36px, 4vw, 56px);
            color: #ffd700;
            text-shadow: 0 0 20px rgba(255, 215, 0, 0.8), 0 0 40px rgba(255, 215, 0, 0.5), 0 4px 0 #b8860b;
            animation: bigWinPulse 0.8s ease-out;
        }}
        @keyframes bigWinPulse {{
            0% {{ transform: scale(0.5); opacity: 0; }}
            50% {{ transform: scale(1.3); }}
            70% {{ transform: scale(0.9); }}
            100% {{ transform: scale(1); opacity: 1; }}
        }}
        .wheel-result.bankrupt {{
            color: #ef4444;
            text-shadow: 0 0 20px rgba(239, 68, 68, 0.5), 0 4px 0 #b91c1c;
            animation: bankruptShake 0.5s ease-out;
        }}
        @keyframes bankruptShake {{
            0%, 100% {{ transform: translateX(0) rotate(0deg); }}
            20% {{ transform: translateX(-10px) rotate(-2deg); }}
            40% {{ transform: translateX(10px) rotate(2deg); }}
            60% {{ transform: translateX(-5px) rotate(-1deg); }}
            80% {{ transform: translateX(5px) rotate(1deg); }}
        }}
        .wheel-result.free-play {{
            color: #22c55e;
            text-shadow: 0 0 20px rgba(34, 197, 94, 0.5), 0 4px 0 #15803d;
            animation: freePlayBounce 0.6s ease-out;
        }}
        @keyframes freePlayBounce {{
            0% {{ transform: scale(0) rotate(-10deg); }}
            50% {{ transform: scale(1.2) rotate(5deg); }}
            70% {{ transform: scale(0.9) rotate(-2deg); }}
            100% {{ transform: scale(1) rotate(0deg); }}
        }}
        .puzzle-section {{
            flex: 1;
            min-width: 0;
        }}
        @media (max-width: 900px) {{
            .game-layout {{
                flex-direction: column;
                align-items: center;
            }}
            .wheel-container {{
                width: min(340px, 70vw);
                height: min(340px, 70vw);
                transform: rotateX(28deg);
            }}
            .wheel-center-hub {{
                width: 50px;
                height: 50px;
            }}
            .wheel-center-hub::after {{
                width: 16px;
                height: 16px;
            }}
        }}
        .controls {{ display: flex; gap: 10px; flex-wrap: wrap; justify-content: center; margin-top: 20px; }}

        /* ========== ANIMATED BUTTONS ========== */
        .btn {{
            background: linear-gradient(180deg, #ffd700 0%, #d4af37 100%);
            color: #1a0a3e;
            padding: 14px 28px;
            border-radius: 12px;
            font-weight: bold;
            font-size: 16px;
            border: none;
            cursor: pointer;
            transition: all 0.15s ease;
            box-shadow: 0 4px 15px rgba(212, 175, 55, 0.3);
            position: relative;
            overflow: hidden;
        }}
        .btn:hover {{
            transform: translateY(-2px);
            box-shadow: 0 6px 20px rgba(212, 175, 55, 0.4);
        }}
        .btn:active {{
            transform: scale(0.95);
            box-shadow: 0 2px 10px rgba(212, 175, 55, 0.3);
        }}
        .btn:disabled {{
            opacity: 0.5;
            cursor: not-allowed;
            transform: none;
        }}
        .btn.waiting {{
            animation: btnPulse 1.5s ease-in-out infinite;
        }}
        @keyframes btnPulse {{
            0%, 100% {{ box-shadow: 0 4px 15px rgba(212, 175, 55, 0.3); }}
            50% {{ box-shadow: 0 0 30px rgba(255, 215, 0, 0.8), 0 0 50px rgba(255, 215, 0, 0.4); }}
        }}
        .btn-secondary {{
            background: linear-gradient(180deg, #444 0%, #333 100%);
            color: #fff;
            box-shadow: 0 4px 15px rgba(0, 0, 0, 0.3);
        }}
        .btn-secondary:hover {{
            box-shadow: 0 6px 20px rgba(0, 0, 0, 0.4);
        }}
        .btn-danger {{
            background: linear-gradient(180deg, #f87171 0%, #ef4444 100%);
            color: #fff;
            box-shadow: 0 4px 15px rgba(239, 68, 68, 0.3);
        }}
        .btn-success {{
            background: linear-gradient(180deg, #4ade80 0%, #22c55e 100%);
            color: #1a0a3e;
            box-shadow: 0 4px 15px rgba(34, 197, 94, 0.3);
        }}

        /* ========== PLAYER LIST ========== */
        .player-list {{ margin-top: 20px; }}
        .player {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 12px 16px;
            background: linear-gradient(180deg, #1a0a3e 0%, #0d0628 100%);
            border-radius: 12px;
            margin-bottom: 8px;
            border: 2px solid #333;
            transition: all 0.3s ease;
            position: relative;
        }}
        .player.active {{
            border-color: var(--color-primary);
            box-shadow: 0 0 15px var(--color-primary-glow);
        }}
        .player-info {{
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        .player-avatar {{
            font-size: 24px;
            line-height: 1;
            transition: transform 0.3s ease;
        }}
        .player-avatar.active {{
            transform: scale(1.2);
            animation: avatarPulse 1.5s ease-in-out infinite;
        }}
        @keyframes avatarPulse {{
            0%, 100% {{ transform: scale(1.2); }}
            50% {{ transform: scale(1.35); }}
        }}
        .player-name {{ color: #fff; font-weight: 500; }}
        .player-score {{
            color: var(--color-primary);
            font-family: 'Courier New', monospace;
            font-weight: 700;
            font-size: 14px;
            position: relative;
            text-align: right;
        }}
        .player-score-details {{
            display: flex;
            flex-direction: column;
            align-items: flex-end;
            gap: 2px;
        }}
        .player-score-total {{
            color: var(--color-primary);
            font-size: 16px;
            font-weight: bold;
        }}
        .player-score-round {{
            color: #888;
            font-size: 12px;
        }}
        .player-prizes {{
            display: flex;
            gap: 4px;
            flex-wrap: wrap;
            justify-content: flex-end;
            margin-top: 4px;
        }}
        .player-prize {{
            background: linear-gradient(135deg, #8b5cf6, #6366f1);
            color: #fff;
            font-size: 10px;
            padding: 2px 6px;
            border-radius: 4px;
            white-space: nowrap;
        }}

        /* Score change animation */
        .score-change {{
            position: absolute;
            right: 0;
            top: -20px;
            font-size: 16px;
            font-weight: bold;
            animation: scoreFloat 1.5s ease-out forwards;
            pointer-events: none;
            z-index: 10;
        }}
        .score-change.positive {{ color: var(--color-success); }}
        .score-change.negative {{ color: var(--color-danger); }}
        @keyframes scoreFloat {{
            0% {{ opacity: 1; transform: translateY(0); }}
            100% {{ opacity: 0; transform: translateY(-30px); }}
        }}

        /* Wild card indicator */
        .player-wildcards {{
            display: flex;
            gap: 4px;
            margin-left: 8px;
        }}
        .wildcard-icon {{
            width: 20px;
            height: 20px;
            background: linear-gradient(135deg, #ffd700 0%, #ff8c00 100%);
            border-radius: 4px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 12px;
            font-weight: bold;
            color: #000;
        }}

        /* Input row with timer */
        .input-timer-row {{
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 16px;
            margin-top: 20px;
        }}

        /* Turn timer countdown */
        .turn-timer {{
            display: none;
            align-items: center;
            justify-content: center;
            gap: 8px;
            padding: 12px 20px;
            background: linear-gradient(135deg, rgba(255, 68, 68, 0.3), rgba(255, 136, 0, 0.3));
            border: 2px solid #ff4444;
            border-radius: 10px;
            font-size: 18px;
            font-weight: bold;
            box-shadow: 0 4px 15px rgba(255, 68, 68, 0.4);
            animation: timerPulse 1s ease-in-out infinite;
            min-width: 90px;
        }}
        .turn-timer.active {{
            display: flex;
        }}
        .turn-timer.urgent {{
            border-color: #ff0000;
            background: linear-gradient(135deg, rgba(255, 0, 0, 0.3), rgba(255, 68, 68, 0.3));
            animation: timerUrgent 0.5s ease-in-out infinite;
        }}
        @keyframes timerPulse {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.8; }}
        }}
        @keyframes timerUrgent {{
            0%, 100% {{ transform: scale(1); }}
            50% {{ transform: scale(1.02); }}
        }}
        .turn-timer-icon {{
            font-size: 18px;
        }}
        .turn-timer-text {{
            color: #ff4444;
            font-weight: bold;
            font-size: 16px;
            font-family: 'Courier New', monospace;
        }}
        .turn-timer.urgent .turn-timer-text {{
            color: #ff0000;
            animation: timerTextBlink 0.3s ease-in-out infinite;
        }}
        @keyframes timerTextBlink {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.5; }}
        }}

        .guess-input {{
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 8px;
        }}
        .guess-label {{
            color: #d4af37;
            font-weight: bold;
            font-size: 14px;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        .guess-input input {{
            width: 80px;
            height: 60px;
            text-align: center;
            font-size: 32px;
            font-weight: bold;
            text-transform: uppercase;
            border: 3px solid #444;
            border-radius: 12px;
            background: #1a1a2e;
            color: #fff;
            transition: border-color 0.3s, box-shadow 0.3s;
        }}
        .guess-input input:focus {{
            outline: none;
            border-color: #d4af37;
            box-shadow: 0 0 15px rgba(212, 175, 55, 0.4);
        }}
        .guess-input.waiting input {{
            animation: inputPulse 1s ease-in-out infinite;
        }}
        @keyframes inputPulse {{
            0%, 100% {{ border-color: #d4af37; }}
            50% {{ border-color: #ffd700; box-shadow: 0 0 15px rgba(255, 215, 0, 0.5); }}
        }}
        @keyframes labelPulse {{
            0%, 100% {{ color: #d4af37; }}
            50% {{ color: #ffd700; }}
        }}
        .notification {{
            background: rgba(26, 10, 62, 0.95);
            color: #ffffff;
            padding: 12px 20px;
            border-radius: 10px;
            text-align: center;
            font-family: 'Mountains of Christmas', cursive;
            font-size: 18px;
            font-weight: 700;
            letter-spacing: 0.5px;
            border: 2px solid rgba(212, 175, 55, 0.5);
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
            display: none;
            opacity: 0;
            transition: opacity 0.3s ease;
            margin: 12px auto;
            max-width: 500px;
            min-height: 20px;
        }}
        .notification.show {{
            display: block;
            opacity: 1;
            animation: fadeIn 0.3s ease forwards;
        }}
        .notification.hide {{
            animation: fadeOut 0.2s ease forwards;
        }}
        .notification.success {{
            border-color: rgba(34, 197, 94, 0.7);
            background: linear-gradient(135deg, rgba(26, 10, 62, 0.95), rgba(34, 197, 94, 0.15));
        }}
        .notification.error {{
            border-color: rgba(239, 68, 68, 0.7);
            background: linear-gradient(135deg, rgba(26, 10, 62, 0.95), rgba(239, 68, 68, 0.15));
        }}
        .notification.warning {{
            border-color: rgba(245, 158, 11, 0.7);
            background: linear-gradient(135deg, rgba(26, 10, 62, 0.95), rgba(245, 158, 11, 0.15));
        }}
        .notification .icon {{
            margin-right: 8px;
        }}
        .notification.letter-result {{
            padding: 16px 32px;
            min-width: 200px;
        }}
        .letter-result-content {{
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 4px;
        }}
        .letter-big {{
            font-size: 48px;
            font-weight: bold;
            line-height: 1;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }}
        .notification.success .letter-big {{
            color: #4ade80;
        }}
        .notification.error .letter-big,
        .letter-big.letter-miss {{
            color: #f87171;
        }}
        .letter-count {{
            font-size: 16px;
            font-weight: 600;
            opacity: 0.9;
        }}
        @keyframes fadeIn {{
            from {{ opacity: 0; transform: translateY(-10px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}
        @keyframes fadeOut {{
            from {{ opacity: 1; transform: translateY(0); }}
            to {{ opacity: 0; transform: translateY(-10px); }}
        }}
        .modal-overlay {{
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0, 0, 0, 0.8);
            z-index: 1000;
            align-items: center;
            justify-content: center;
        }}
        .modal-overlay.active {{
            display: flex;
        }}
        /* Bottom-positioned modals that don't obscure the puzzle */
        .modal-overlay.bottom-modal {{
            align-items: flex-end;
            background: rgba(0, 0, 0, 0.5);
        }}
        .modal-overlay.bottom-modal .modal {{
            margin-bottom: 20px;
            border-radius: 16px 16px 16px 16px;
            animation: slideUp 0.2s ease-out;
        }}
        @keyframes slideUp {{
            from {{ transform: translateY(100%); opacity: 0; }}
            to {{ transform: translateY(0); opacity: 1; }}
        }}
        .modal {{
            background: #1a0a3e;
            border: 2px solid #d4af37;
            border-radius: 16px;
            padding: 32px;
            min-width: 320px;
            max-width: 90%;
            text-align: center;
        }}
        .modal h2 {{
            color: #d4af37;
            margin-bottom: 20px;
            font-size: 24px;
        }}
        .modal input {{
            width: 100%;
            padding: 14px 16px;
            font-size: 18px;
            text-transform: uppercase;
            margin-bottom: 20px;
        }}
        .modal-buttons {{
            display: flex;
            gap: 12px;
            justify-content: center;
        }}
        .modal-buttons button {{
            min-width: 100px;
        }}
        .vowel-buttons {{
            display: flex;
            gap: 8px;
            justify-content: center;
            margin-bottom: 20px;
        }}
        .vowel-btn {{
            width: 50px;
            height: 50px;
            font-size: 24px;
            font-weight: bold;
            border-radius: 8px;
            cursor: pointer;
        }}
        .game-header {{
            text-align: center;
            margin-bottom: 16px;
            padding-bottom: 16px;
            border-bottom: 1px solid #333;
            position: relative;
        }}
        .game-header h1 {{
            margin: 0;
            font-family: 'Mountains of Christmas', cursive;
            font-size: 44px;
            font-weight: 700;
            background: linear-gradient(
                135deg,
                var(--theme-color-1) 0%,
                var(--theme-color-2) 25%,
                var(--theme-color-3) 50%,
                var(--theme-color-2) 75%,
                var(--theme-color-1) 100%
            );
            background-size: 200% auto;
            -webkit-background-clip: text;
            background-clip: text;
            -webkit-text-fill-color: transparent;
            animation: festiveShimmer 4s linear infinite;
            letter-spacing: 3px;
            filter: drop-shadow(0 2px 4px rgba(212, 175, 55, 0.3));
        }}
        .header-controls {{
            position: absolute;
            right: 0;
            top: 50%;
            transform: translateY(-50%);
            display: flex;
            gap: 8px;
            align-items: center;
        }}
        .btn-icon {{
            width: 40px;
            height: 40px;
            padding: 0;
            font-size: 20px;
            display: flex;
            align-items: center;
            justify-content: center;
            background: rgba(255,255,255,0.1);
            border: 1px solid #444;
            border-radius: 8px;
            cursor: pointer;
            transition: all 0.2s ease;
        }}
        .btn-icon:hover {{
            background: rgba(255,255,255,0.2);
        }}
        .btn-icon.muted {{
            opacity: 0.5;
        }}
        .spectator-banner {{
            background: linear-gradient(90deg, #3498db, #2980b9);
            color: white;
            padding: 8px 16px;
            border-radius: 4px;
            font-size: 14px;
            display: inline-block;
            margin-top: 8px;
        }}

        /* ========== CONFETTI ========== */
        #confetti-container {{
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            pointer-events: none;
            z-index: 9999;
            overflow: hidden;
        }}
        .confetti {{
            position: absolute;
            width: 10px;
            height: 10px;
            animation: confettiFall 3s ease-out forwards;
        }}
        @keyframes confettiFall {{
            0% {{ transform: translateY(-10px) rotate(0deg); opacity: 1; }}
            100% {{ transform: translateY(100vh) rotate(720deg); opacity: 0; }}
        }}
        /* Enhanced gold confetti for winners */
        .confetti.gold {{
            width: 12px;
            height: 12px;
            background: linear-gradient(135deg, #ffd700 0%, #ffec8b 50%, #d4af37 100%) !important;
            box-shadow: 0 0 6px rgba(255, 215, 0, 0.8);
        }}
        /* Sparkle particles */
        .sparkle {{
            position: absolute;
            width: 8px;
            height: 8px;
            background: radial-gradient(circle, #fff 0%, #ffd700 40%, transparent 70%);
            border-radius: 50%;
            animation: sparkleFloat 2s ease-out forwards;
            pointer-events: none;
        }}
        @keyframes sparkleFloat {{
            0% {{ transform: scale(0) rotate(0deg); opacity: 1; }}
            50% {{ transform: scale(1.5) rotate(180deg); opacity: 0.8; }}
            100% {{ transform: scale(0) rotate(360deg) translateY(-100px); opacity: 0; }}
        }}
        /* Firework burst effect */
        .firework {{
            position: absolute;
            width: 6px;
            height: 6px;
            border-radius: 50%;
            animation: fireworkBurst 1.5s ease-out forwards;
        }}
        @keyframes fireworkBurst {{
            0% {{ transform: scale(0); opacity: 1; }}
            20% {{ transform: scale(1); opacity: 1; }}
            100% {{ transform: scale(0); opacity: 0; }}
        }}

        /* ========== PHASE TRANSITION OVERLAY (Wheel of Fortune Style) ========== */
        .phase-overlay {{
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background:
                radial-gradient(ellipse at center, rgba(75, 0, 130, 0.95) 0%, rgba(26, 10, 62, 0.98) 50%, rgba(0, 0, 0, 0.99) 100%);
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            z-index: 5000;
            opacity: 0;
            pointer-events: none;
            transition: opacity 0.6s ease;
            overflow: hidden;
        }}
        .phase-overlay::before {{
            content: '';
            position: absolute;
            top: 0;
            left: -100%;
            width: 300%;
            height: 100%;
            background: linear-gradient(90deg,
                transparent 0%,
                rgba(255, 215, 0, 0.03) 25%,
                rgba(255, 215, 0, 0.08) 50%,
                rgba(255, 215, 0, 0.03) 75%,
                transparent 100%);
            animation: overlayShine 3s linear infinite;
        }}
        @keyframes overlayShine {{
            0% {{ transform: translateX(-33%); }}
            100% {{ transform: translateX(33%); }}
        }}
        .phase-overlay.active {{
            opacity: 1;
            pointer-events: auto;
        }}
        /* Animated border frame */
        .phase-frame {{
            position: absolute;
            top: 10%;
            left: 10%;
            right: 10%;
            bottom: 10%;
            border: 4px solid transparent;
            border-image: linear-gradient(45deg, #d4af37, #ffd700, #d4af37, #b8860b, #d4af37) 1;
            animation: framePulse 2s ease-in-out infinite;
            pointer-events: none;
        }}
        @keyframes framePulse {{
            0%, 100% {{ opacity: 0.4; }}
            50% {{ opacity: 0.8; }}
        }}
        /* Corner decorations */
        .phase-corner {{
            position: absolute;
            width: 60px;
            height: 60px;
            border: 3px solid #ffd700;
            opacity: 0.6;
        }}
        .phase-corner.tl {{ top: 8%; left: 8%; border-right: none; border-bottom: none; }}
        .phase-corner.tr {{ top: 8%; right: 8%; border-left: none; border-bottom: none; }}
        .phase-corner.bl {{ bottom: 8%; left: 8%; border-right: none; border-top: none; }}
        .phase-corner.br {{ bottom: 8%; right: 8%; border-left: none; border-top: none; }}
        /* Floating sparkles container */
        .phase-sparkles {{
            position: absolute;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            pointer-events: none;
            overflow: hidden;
        }}
        .phase-sparkle {{
            position: absolute;
            width: 4px;
            height: 4px;
            background: #ffd700;
            border-radius: 50%;
            box-shadow: 0 0 10px #ffd700, 0 0 20px #ffd700;
            animation: phaseSparkleFloat 3s ease-in-out infinite;
        }}
        @keyframes phaseSparkleFloat {{
            0%, 100% {{ opacity: 0.3; transform: translateY(0) scale(1); }}
            50% {{ opacity: 1; transform: translateY(-20px) scale(1.5); }}
        }}
        .phase-content {{
            text-align: center;
            animation: phaseZoom 0.8s cubic-bezier(0.175, 0.885, 0.32, 1.275);
            position: relative;
            z-index: 1;
        }}
        .phase-title {{
            font-family: 'Mountains of Christmas', cursive;
            font-size: 84px;
            font-weight: 700;
            color: #ffd700;
            text-shadow:
                0 0 20px rgba(255, 215, 0, 0.8),
                0 0 40px rgba(255, 215, 0, 0.6),
                0 0 80px rgba(255, 215, 0, 0.4),
                0 4px 0 #b8860b,
                0 6px 0 #a67c00,
                0 8px 20px rgba(0, 0, 0, 0.5);
            margin-bottom: 20px;
            letter-spacing: 6px;
            animation: phasePulse 1.5s ease-in-out infinite, phaseShimmer 3s linear infinite;
            background: linear-gradient(90deg, #ffd700 0%, #ffec8b 25%, #ffd700 50%, #d4af37 75%, #ffd700 100%);
            background-size: 200% 100%;
            -webkit-background-clip: text;
            background-clip: text;
            -webkit-text-fill-color: transparent;
            filter: drop-shadow(0 4px 0 #b8860b) drop-shadow(0 8px 15px rgba(0,0,0,0.5));
        }}
        @keyframes phaseShimmer {{
            0% {{ background-position: 200% center; }}
            100% {{ background-position: -200% center; }}
        }}
        .phase-subtitle {{
            font-size: 32px;
            color: #fff;
            letter-spacing: 3px;
            text-transform: uppercase;
            text-shadow: 0 0 20px rgba(255, 255, 255, 0.5), 0 2px 4px rgba(0, 0, 0, 0.5);
            animation: subtitleFadeIn 0.8s ease-out 0.3s backwards;
        }}
        @keyframes subtitleFadeIn {{
            0% {{ opacity: 0; transform: translateY(20px); }}
            100% {{ opacity: 1; transform: translateY(0); }}
        }}
        .phase-countdown {{
            font-size: 22px;
            color: #d4af37;
            margin-top: 50px;
            letter-spacing: 2px;
            text-shadow: 0 0 15px rgba(212, 175, 55, 0.5);
            animation: countdownPulse 1s ease-in-out infinite;
        }}
        @keyframes countdownPulse {{
            0%, 100% {{ opacity: 0.7; transform: scale(1); }}
            50% {{ opacity: 1; transform: scale(1.05); }}
        }}
        @keyframes phaseZoom {{
            0% {{ transform: scale(0.1) rotate(-10deg); opacity: 0; }}
            50% {{ transform: scale(1.15) rotate(3deg); }}
            70% {{ transform: scale(0.95) rotate(-1deg); }}
            100% {{ transform: scale(1) rotate(0deg); opacity: 1; }}
        }}
        @keyframes phasePulse {{
            0%, 100% {{ transform: scale(1); }}
            50% {{ transform: scale(1.03); }}
        }}
        /* Special phase-specific styles */
        .phase-overlay.winner-mode {{
            background:
                radial-gradient(ellipse at center, rgba(34, 197, 94, 0.3) 0%, transparent 50%),
                radial-gradient(ellipse at center, rgba(75, 0, 130, 0.95) 0%, rgba(26, 10, 62, 0.98) 50%, rgba(0, 0, 0, 0.99) 100%);
        }}
        .phase-overlay.winner-mode .phase-title {{
            font-size: 100px;
            animation: winnerBounce 0.6s ease-out, phasePulse 1.5s ease-in-out infinite 0.6s, phaseShimmer 3s linear infinite;
        }}
        @keyframes winnerBounce {{
            0% {{ transform: scale(0) rotate(-15deg); }}
            50% {{ transform: scale(1.3) rotate(5deg); }}
            70% {{ transform: scale(0.9) rotate(-2deg); }}
            85% {{ transform: scale(1.1) rotate(1deg); }}
            100% {{ transform: scale(1) rotate(0deg); }}
        }}
        /* Prize amount display */
        .phase-prize {{
            font-family: 'Mountains of Christmas', cursive;
            font-size: 64px;
            font-weight: 700;
            color: #ffd700;
            margin-top: 20px;
            text-shadow:
                0 0 30px rgba(255, 215, 0, 0.8),
                0 4px 0 #b8860b;
            animation: prizeCountUp 0.5s ease-out;
        }}
        @keyframes prizeCountUp {{
            0% {{ transform: scale(0.5); opacity: 0; }}
            60% {{ transform: scale(1.2); }}
            100% {{ transform: scale(1); opacity: 1; }}
        }}
        /* Mini wheel animation in background */
        .phase-wheel-bg {{
            position: absolute;
            width: 400px;
            height: 400px;
            opacity: 0.1;
            animation: wheelBgSpin 20s linear infinite;
            pointer-events: none;
        }}
        @keyframes wheelBgSpin {{
            0% {{ transform: rotate(0deg); }}
            100% {{ transform: rotate(360deg); }}
        }}

        /* ========== MYSTERY WEDGE MODAL ========== */
        .mystery-modal {{
            background: linear-gradient(180deg, #2a1a4e 0%, #1a0a3e 100%);
            border: 3px solid var(--color-primary);
            box-shadow: 0 0 40px var(--color-primary-glow);
        }}
        .mystery-options {{
            display: flex;
            gap: 20px;
            justify-content: center;
            margin: 24px 0;
        }}
        .mystery-option {{
            padding: 24px 32px;
            border-radius: 12px;
            cursor: pointer;
            transition: all 0.2s ease;
            min-width: 140px;
        }}
        .mystery-option.keep {{
            background: linear-gradient(180deg, #22c55e 0%, #16a34a 100%);
            color: #fff;
        }}
        .mystery-option.flip {{
            background: linear-gradient(180deg, #ef4444 0%, #dc2626 100%);
            color: #fff;
        }}
        .mystery-option:hover {{
            transform: scale(1.05);
        }}
        .mystery-option .amount {{
            font-size: 28px;
            font-weight: bold;
            display: block;
        }}
        .mystery-option .label {{
            font-size: 14px;
            opacity: 0.9;
        }}
        .mystery-result {{
            font-size: 36px;
            font-weight: bold;
            margin: 24px 0;
            animation: mysteryReveal 0.5s ease-out;
        }}
        .mystery-result.win {{ color: var(--color-success); }}
        .mystery-result.lose {{ color: var(--color-danger); }}
        @keyframes mysteryReveal {{
            0% {{ transform: scale(0) rotate(-180deg); }}
            100% {{ transform: scale(1) rotate(0deg); }}
        }}

        /* ========== EXPRESS MODE INDICATOR ========== */
        .express-indicator {{
            position: fixed;
            top: 20px;
            left: 50%;
            transform: translateX(-50%);
            background: linear-gradient(180deg, #6366f1 0%, #4f46e5 100%);
            padding: 12px 24px;
            border-radius: 12px;
            display: none;
            align-items: center;
            gap: 12px;
            z-index: 100;
            box-shadow: 0 4px 20px rgba(99, 102, 241, 0.4);
            animation: expressPulse 2s ease-in-out infinite;
        }}
        .express-indicator.active {{
            display: flex;
        }}
        .express-label {{
            font-weight: bold;
            color: #fff;
            font-size: 18px;
        }}
        .express-streak {{
            background: rgba(255, 255, 255, 0.2);
            padding: 4px 12px;
            border-radius: 20px;
            color: #fff;
            font-weight: bold;
        }}
        @keyframes expressPulse {{
            0%, 100% {{ box-shadow: 0 4px 20px rgba(99, 102, 241, 0.4); }}
            50% {{ box-shadow: 0 4px 30px rgba(99, 102, 241, 0.7); }}
        }}

        /* ========== ROUND PROGRESS INDICATOR ========== */
        .round-indicator {{
            display: none;
            background: rgba(26, 10, 62, 0.9);
            padding: 12px 20px;
            border-radius: 12px;
            margin-bottom: 16px;
            border: 1px solid #333;
        }}
        .round-indicator.active {{
            display: block;
        }}
        .round-header {{
            display: flex;
            align-items: baseline;
            justify-content: center;
            gap: 8px;
            margin-bottom: 8px;
        }}
        .round-number {{
            font-size: 18px;
            font-weight: bold;
            color: #fff;
        }}
        .round-total {{
            font-size: 14px;
            color: var(--color-text-muted);
        }}
        .round-dots {{
            display: flex;
            justify-content: center;
            gap: 6px;
            margin-bottom: 8px;
        }}
        .round-dot {{
            width: 10px;
            height: 10px;
            border-radius: 50%;
            background: #333;
            transition: all 0.3s ease;
        }}
        .round-dot.completed {{
            background: var(--color-success);
        }}
        .round-dot.current {{
            background: var(--color-primary);
            width: 14px;
            height: 14px;
            box-shadow: 0 0 10px var(--color-primary-glow);
        }}
        .round-badges {{
            display: flex;
            justify-content: center;
            gap: 8px;
        }}
        .round-badge {{
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 12px;
            font-weight: bold;
        }}
        .round-badge.type {{
            background: var(--color-accent);
            color: #fff;
        }}
        .round-badge.multiplier {{
            background: var(--color-primary);
            color: #1a0a3e;
        }}

        /* ========== TOSS-UP VALUE DISPLAY ========== */
        .tossup-display {{
            display: none;
            background: linear-gradient(180deg, #ef4444 0%, #dc2626 100%);
            border: 4px solid var(--color-primary);
            border-radius: 16px;
            padding: 20px 28px;
            text-align: center;
            margin-bottom: 16px;
            box-shadow:
                0 0 30px rgba(239, 68, 68, 0.5),
                0 0 60px rgba(239, 68, 68, 0.2),
                inset 0 2px 4px rgba(255, 255, 255, 0.2);
            position: relative;
            overflow: hidden;
        }}
        .tossup-display::before {{
            content: '';
            position: absolute;
            top: -50%;
            left: -50%;
            width: 200%;
            height: 200%;
            background: linear-gradient(45deg, transparent 40%, rgba(255, 255, 255, 0.1) 50%, transparent 60%);
            animation: tossupShine 3s linear infinite;
            pointer-events: none;
        }}
        @keyframes tossupShine {{
            0% {{ transform: translateX(-100%) rotate(45deg); }}
            100% {{ transform: translateX(100%) rotate(45deg); }}
        }}
        .tossup-display.active {{
            display: block;
            animation: tossupPulse 0.5s ease-out, tossupGlow 2s ease-in-out infinite;
        }}
        @keyframes tossupGlow {{
            0%, 100% {{ box-shadow: 0 0 30px rgba(239, 68, 68, 0.5), 0 0 60px rgba(239, 68, 68, 0.2); }}
            50% {{ box-shadow: 0 0 40px rgba(239, 68, 68, 0.7), 0 0 80px rgba(239, 68, 68, 0.3); }}
        }}
        .triple-header {{
            margin-bottom: 12px;
        }}
        .triple-label {{
            font-size: 14px;
            font-weight: bold;
            color: #fff;
            letter-spacing: 1px;
        }}
        .triple-indicators {{
            display: flex;
            justify-content: center;
            gap: 8px;
            margin-top: 8px;
        }}
        .triple-dot {{
            width: 16px;
            height: 16px;
            border-radius: 50%;
            background: rgba(255, 255, 255, 0.3);
            border: 2px solid #fff;
        }}
        .triple-dot.completed {{
            background: var(--color-success);
            border-color: var(--color-success);
        }}
        .triple-dot.current {{
            background: var(--color-primary);
            border-color: var(--color-primary);
        }}
        .tossup-value {{
            font-size: 36px;
            font-weight: bold;
            color: #fff;
            text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.5);
        }}
        .tossup-for {{
            font-size: 14px;
            color: rgba(255, 255, 255, 0.8);
            margin-bottom: 4px;
        }}
        @keyframes tossupPulse {{
            0% {{ transform: scale(1.1); }}
            100% {{ transform: scale(1); }}
        }}

        /* ========== WHEEL WEDGE HIGHLIGHT ========== */
        .wheel-svg .wedge-highlight {{
            animation: wedgeFlash 0.3s ease-out 3;
        }}
        @keyframes wedgeFlash {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.5; filter: brightness(1.5); }}
        }}

        /* ========== ENHANCED WHEEL SPIN EFFECTS ========== */
        .wheel-container.spinning .wheel-svg {{
            filter: drop-shadow(0 4px 8px rgba(0,0,0,0.4)) drop-shadow(0 0 20px rgba(212, 175, 55, 0.6));
        }}
        .wheel-container.spinning .wheel-pointer {{
            animation: pointerBounce 0.15s ease-in-out infinite;
        }}
        @keyframes pointerBounce {{
            0%, 100% {{ transform: translateY(0); }}
            50% {{ transform: translateY(3px); }}
        }}
        .wheel-glow {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            width: 110%;
            height: 110%;
            border-radius: 50%;
            pointer-events: none;
            opacity: 0;
            transition: opacity 0.3s ease;
            background: radial-gradient(circle, rgba(212, 175, 55, 0.3) 0%, transparent 70%);
        }}
        .wheel-container.spinning .wheel-glow {{
            opacity: 1;
            animation: wheelGlowPulse 0.5s ease-in-out infinite;
        }}
        @keyframes wheelGlowPulse {{
            0%, 100% {{ transform: translate(-50%, -50%) scale(1); opacity: 0.8; }}
            50% {{ transform: translate(-50%, -50%) scale(1.05); opacity: 1; }}
        }}
        .wheel-tick-flash {{
            position: absolute;
            top: 0;
            left: 50%;
            transform: translateX(-50%);
            width: 40px;
            height: 40px;
            background: radial-gradient(circle, rgba(255, 215, 0, 0.9) 0%, transparent 70%);
            border-radius: 50%;
            pointer-events: none;
            opacity: 0;
            z-index: 15;
        }}
        .wheel-tick-flash.flash {{
            animation: tickFlash 0.08s ease-out;
        }}
        @keyframes tickFlash {{
            0% {{ opacity: 1; transform: translateX(-50%) scale(1.2); }}
            100% {{ opacity: 0; transform: translateX(-50%) scale(0.8); }}
        }}
        .winning-wedge {{
            animation: winningPulse 0.5s ease-in-out 3;
        }}
        @keyframes winningPulse {{
            0%, 100% {{ filter: brightness(1); }}
            50% {{ filter: brightness(1.4) saturate(1.3); }}
        }}
        .wheel-result.winner {{
            animation: resultPop 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
        }}
        @keyframes resultPop {{
            0% {{ transform: scale(0.5); opacity: 0; }}
            70% {{ transform: scale(1.1); }}
            100% {{ transform: scale(1); opacity: 1; }}
        }}

        /* ========== WILD CARD BUTTON ========== */
        .wildcard-btn {{
            background: linear-gradient(135deg, #ffd700 0%, #ff8c00 100%);
            color: #1a0a3e;
            display: none;
            align-items: center;
            gap: 8px;
            padding: 10px 20px;
        }}
        .wildcard-btn.available {{
            display: flex;
        }}
        .wildcard-btn .icon {{
            font-size: 20px;
        }}

        /* ========== BONUS ROUND STYLES ========== */
        .bonus-round-overlay {{ display: none; position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: linear-gradient(135deg, #0a0520 0%, #1a0a3e 50%, #2a1050 100%); z-index: 6000; overflow: hidden; }}
        .bonus-round-overlay.active {{ display: flex; flex-direction: column; animation: bonusEnter 0.8s ease-out; }}
        @keyframes bonusEnter {{ 0% {{ opacity: 0; transform: scale(1.1); }} 100% {{ opacity: 1; transform: scale(1); }} }}
        .bonus-stars {{ position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; overflow: hidden; }}
        .bonus-star {{ position: absolute; width: 4px; height: 4px; background: #ffd700; border-radius: 50%; animation: twinkle 2s infinite ease-in-out; }}
        @keyframes twinkle {{ 0%, 100% {{ opacity: 0.3; transform: scale(1); }} 50% {{ opacity: 1; transform: scale(1.5); }} }}
        .bonus-header {{ text-align: center; padding: 16px; position: relative; z-index: 1; }}
        .bonus-title {{ font-family: 'Mountains of Christmas', cursive; font-size: 42px; font-weight: 700; color: #ffd700; text-shadow: 0 0 30px rgba(255, 215, 0, 0.5), 0 4px 8px rgba(0, 0, 0, 0.5); margin-bottom: 6px; animation: bonusTitlePulse 2s ease-in-out infinite; }}
        @keyframes bonusTitlePulse {{ 0%, 100% {{ text-shadow: 0 0 30px rgba(255, 215, 0, 0.5), 0 4px 8px rgba(0, 0, 0, 0.5); }} 50% {{ text-shadow: 0 0 50px rgba(255, 215, 0, 0.8), 0 4px 8px rgba(0, 0, 0, 0.5); }} }}
        .bonus-player-name {{ font-size: 20px; color: #fff; opacity: 0.9; display: flex; align-items: center; justify-content: center; gap: 8px; }}
        .bonus-player-avatar {{ font-size: 32px; }}
        .bonus-content {{ flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: flex-start; gap: 12px; padding: 16px; position: relative; z-index: 1; overflow-y: auto; }}
        .prize-wheel-container {{ position: relative; width: 240px; height: 240px; margin: 0 auto; }}
        .prize-wheel-svg {{ width: 100%; height: 100%; filter: drop-shadow(0 8px 24px rgba(0, 0, 0, 0.5)); transition: transform 4s cubic-bezier(0.17, 0.67, 0.12, 0.99); }}
        .prize-wheel-pointer {{ position: absolute; top: -8px; left: 50%; transform: translateX(-50%); border-left: 14px solid transparent; border-right: 14px solid transparent; border-top: 24px solid #ffd700; filter: drop-shadow(0 3px 4px rgba(0, 0, 0, 0.5)); z-index: 10; }}
        .prize-wheel-result {{ text-align: center; margin-top: 10px; font-family: 'Mountains of Christmas', cursive; font-size: 28px; font-weight: 700; color: #ffd700; text-shadow: 0 2px 8px rgba(0, 0, 0, 0.5); }}
        .given-letters {{ display: flex; flex-direction: column; align-items: center; gap: 8px; margin: 8px 0; }}
        .given-letters-label {{ font-size: 12px; color: #888; text-transform: uppercase; letter-spacing: 2px; }}
        .given-letters-row {{ display: flex; gap: 5px; }}
        .given-letter {{ width: 36px; height: 44px; background: linear-gradient(180deg, #22c55e 0%, #16a34a 100%); border-radius: 5px; display: flex; align-items: center; justify-content: center; font-size: 22px; font-weight: bold; color: #fff; border: 2px solid #15803d; box-shadow: 0 3px 8px rgba(0, 0, 0, 0.3); animation: givenLetterAppear 0.3s ease-out backwards; }}
        @keyframes givenLetterAppear {{ 0% {{ transform: scale(0) rotateY(90deg); opacity: 0; }} 100% {{ transform: scale(1) rotateY(0); opacity: 1; }} }}
        .picked-letters {{ display: flex; flex-direction: column; align-items: center; gap: 8px; margin: 8px 0; }}
        .picked-letters-label {{ font-size: 12px; color: #888; text-transform: uppercase; letter-spacing: 2px; }}
        .picked-letters-row {{ display: flex; gap: 5px; }}
        .picked-letter {{ width: 36px; height: 44px; background: linear-gradient(180deg, #3b82f6 0%, #2563eb 100%); border-radius: 5px; display: flex; align-items: center; justify-content: center; font-size: 22px; font-weight: bold; color: #fff; border: 2px solid #1d4ed8; box-shadow: 0 3px 8px rgba(0, 0, 0, 0.3); animation: pickedLetterReveal 0.5s ease-out backwards; }}
        .picked-letter.empty {{ background: linear-gradient(180deg, #333 0%, #222 100%); border-color: #444; color: #666; }}
        @keyframes pickedLetterReveal {{ 0% {{ transform: scale(0) rotateX(-90deg); opacity: 0; }} 50% {{ transform: scale(1.2) rotateX(10deg); }} 100% {{ transform: scale(1) rotateX(0); opacity: 1; }} }}
        .letter-pick-section {{ background: rgba(26, 10, 62, 0.9); border: 2px solid #d4af37; border-radius: 14px; padding: 16px; margin: 8px 0; max-width: 480px; width: 100%; }}
        .letter-pick-title {{ text-align: center; font-size: 16px; color: #d4af37; margin-bottom: 10px; }}
        .letter-pick-instruction {{ text-align: center; font-size: 13px; color: #aaa; margin-bottom: 12px; }}
        .letter-grid {{ display: flex; flex-wrap: wrap; gap: 5px; justify-content: center; }}
        .letter-pick-btn {{ width: 36px; height: 36px; background: linear-gradient(180deg, #444 0%, #333 100%); border: 2px solid #555; border-radius: 6px; font-size: 16px; font-weight: bold; color: #fff; cursor: pointer; transition: all 0.2s ease; }}
        .letter-pick-btn:hover:not(:disabled) {{ background: linear-gradient(180deg, #d4af37 0%, #b8962e 100%); border-color: #ffd700; color: #1a0a3e; transform: scale(1.1); }}
        .letter-pick-btn:disabled {{ opacity: 0.3; cursor: not-allowed; }}
        .letter-pick-btn.selected {{ background: linear-gradient(180deg, #22c55e 0%, #16a34a 100%); border-color: #15803d; }}
        .bonus-timer {{ position: relative; width: 140px; height: 140px; margin: 8px auto; }}
        .bonus-timer-ring {{ position: absolute; top: 0; left: 0; width: 100%; height: 100%; }}
        .bonus-timer-ring circle {{ fill: none; stroke-width: 8; transform: rotate(-90deg); transform-origin: center; }}
        .bonus-timer-ring .track {{ stroke: #333; }}
        .bonus-timer-ring .progress {{ stroke: #22c55e; stroke-linecap: round; transition: stroke-dashoffset 1s linear, stroke 0.3s ease; }}
        .bonus-timer-ring .progress.warning {{ stroke: #f59e0b; }}
        .bonus-timer-ring .progress.danger {{ stroke: #ef4444; animation: timerPulse 0.5s ease-in-out infinite; }}
        @keyframes timerPulse {{ 0%, 100% {{ opacity: 1; }} 50% {{ opacity: 0.5; }} }}
        .bonus-timer-text {{ position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); text-align: center; }}
        .bonus-timer-seconds {{ font-family: 'Mountains of Christmas', cursive; font-size: 48px; font-weight: 700; color: #fff; line-height: 1; }}
        .bonus-timer-seconds.warning {{ color: #f59e0b; }}
        .bonus-timer-seconds.danger {{ color: #ef4444; animation: timerTextPulse 0.5s ease-in-out infinite; }}
        @keyframes timerTextPulse {{ 0%, 100% {{ transform: scale(1); }} 50% {{ transform: scale(1.1); }} }}
        .bonus-timer-label {{ font-size: 10px; color: #888; text-transform: uppercase; letter-spacing: 2px; }}
        .bonus-result {{ text-align: center; padding: 30px; position: relative; }}
        .bonus-result::before {{ content: ''; position: absolute; top: 50%; left: 50%; width: 300px; height: 300px; background: radial-gradient(circle, rgba(255, 215, 0, 0.15) 0%, transparent 70%); transform: translate(-50%, -50%); animation: resultGlow 2s ease-in-out infinite; pointer-events: none; }}
        @keyframes resultGlow {{ 0%, 100% {{ opacity: 0.5; transform: translate(-50%, -50%) scale(1); }} 50% {{ opacity: 1; transform: translate(-50%, -50%) scale(1.2); }} }}
        .bonus-result-icon {{ font-size: 100px; margin-bottom: 16px; animation: resultIconBounce 0.8s cubic-bezier(0.175, 0.885, 0.32, 1.275); filter: drop-shadow(0 0 30px rgba(255, 215, 0, 0.5)); }}
        @keyframes resultIconBounce {{ 0% {{ transform: scale(0) rotate(-30deg); opacity: 0; }} 40% {{ transform: scale(1.4) rotate(10deg); }} 60% {{ transform: scale(0.8) rotate(-5deg); }} 80% {{ transform: scale(1.1) rotate(2deg); }} 100% {{ transform: scale(1) rotate(0); opacity: 1; }} }}
        .bonus-result-title {{ font-family: 'Mountains of Christmas', cursive; font-size: 56px; font-weight: 700; margin-bottom: 16px; letter-spacing: 4px; }}
        .bonus-result-title.win {{ color: #ffd700; text-shadow: 0 0 20px rgba(255, 215, 0, 0.8), 0 0 40px rgba(255, 215, 0, 0.6), 0 0 60px rgba(255, 215, 0, 0.4), 0 4px 0 #b8860b; animation: winnerTitlePulse 1s ease-in-out infinite, winnerTitleShimmer 2s linear infinite; background: linear-gradient(90deg, #ffd700 0%, #fff 25%, #ffd700 50%, #ffec8b 75%, #ffd700 100%); background-size: 200% 100%; -webkit-background-clip: text; background-clip: text; -webkit-text-fill-color: transparent; filter: drop-shadow(0 4px 0 #b8860b) drop-shadow(0 0 30px rgba(255, 215, 0, 0.5)); }}
        @keyframes winnerTitlePulse {{ 0%, 100% {{ transform: scale(1); }} 50% {{ transform: scale(1.05); }} }}
        @keyframes winnerTitleShimmer {{ 0% {{ background-position: 200% center; }} 100% {{ background-position: -200% center; }} }}
        .bonus-result-title.lose {{ color: #ef4444; text-shadow: 0 0 30px rgba(239, 68, 68, 0.5), 0 4px 0 #b91c1c; animation: loseTitleShake 0.5s ease-out; }}
        @keyframes loseTitleShake {{ 0%, 100% {{ transform: translateX(0); }} 20% {{ transform: translateX(-10px); }} 40% {{ transform: translateX(10px); }} 60% {{ transform: translateX(-5px); }} 80% {{ transform: translateX(5px); }} }}
        .bonus-result-amount {{ font-family: 'Mountains of Christmas', cursive; font-size: 72px; font-weight: 700; color: #ffd700; text-shadow: 0 0 30px rgba(255, 215, 0, 0.8), 0 0 60px rgba(255, 215, 0, 0.5), 0 6px 0 #b8860b; margin-bottom: 12px; animation: amountPop 0.6s cubic-bezier(0.175, 0.885, 0.32, 1.275) 0.3s backwards; }}
        @keyframes amountPop {{ 0% {{ transform: scale(0) rotate(-10deg); opacity: 0; }} 60% {{ transform: scale(1.2) rotate(3deg); }} 100% {{ transform: scale(1) rotate(0); opacity: 1; }} }}
        .bonus-result-answer {{ font-size: 22px; color: #fff; margin-top: 20px; padding: 14px 28px; background: linear-gradient(180deg, rgba(75, 0, 130, 0.6) 0%, rgba(45, 0, 80, 0.8) 100%); border: 2px solid rgba(255, 215, 0, 0.3); border-radius: 12px; display: inline-block; box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3); animation: answerSlideUp 0.5s ease-out 0.5s backwards; }}
        @keyframes answerSlideUp {{ 0% {{ transform: translateY(30px); opacity: 0; }} 100% {{ transform: translateY(0); opacity: 1; }} }}
        /* Winner celebration stars */
        .winner-stars {{ position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; overflow: hidden; }}
        .winner-star {{ position: absolute; font-size: 24px; animation: starBurst 1.5s ease-out forwards; opacity: 0; }}
        @keyframes starBurst {{ 0% {{ opacity: 0; transform: scale(0) rotate(0deg); }} 20% {{ opacity: 1; transform: scale(1.2) rotate(90deg); }} 100% {{ opacity: 0; transform: scale(0.5) rotate(180deg) translateY(-100px); }} }}
        .bonus-stage-indicator {{ display: flex; justify-content: center; align-items: center; gap: 8px; margin: 8px 0; }}
        .bonus-stage {{ display: flex; flex-direction: column; align-items: center; gap: 3px; opacity: 0.4; transition: all 0.3s ease; }}
        .bonus-stage.active {{ opacity: 1; }}
        .bonus-stage.completed {{ opacity: 0.7; }}
        .bonus-stage-icon {{ width: 32px; height: 32px; border-radius: 50%; background: #333; display: flex; align-items: center; justify-content: center; font-size: 16px; border: 2px solid #555; transition: all 0.3s ease; }}
        .bonus-stage.active .bonus-stage-icon {{ background: linear-gradient(180deg, #d4af37 0%, #b8962e 100%); border-color: #ffd700; box-shadow: 0 0 16px rgba(212, 175, 55, 0.5); }}
        .bonus-stage.completed .bonus-stage-icon {{ background: linear-gradient(180deg, #22c55e 0%, #16a34a 100%); border-color: #15803d; }}
        .bonus-stage-label {{ font-size: 9px; color: #888; text-transform: uppercase; letter-spacing: 1px; }}
        .bonus-stage.active .bonus-stage-label {{ color: #d4af37; }}
        .bonus-stage-connector {{ width: 24px; height: 2px; background: #333; margin-bottom: 16px; }}
        .bonus-stage-connector.completed {{ background: #22c55e; }}
        .bonus-puzzle-board {{ background: linear-gradient(180deg, #2070d0 0%, #1a5cb8 50%, #1450a0 100%); border-radius: 8px; padding: 8px 5px; display: flex; flex-direction: column; align-items: center; gap: 2px; border: 3px solid var(--color-primary); box-shadow: 0 0 25px var(--color-primary-glow); max-width: 100%; overflow-x: auto; }}
        .bonus-puzzle-board .puzzle-row {{ gap: 2px; }}
        .bonus-puzzle-board .letter-tile {{ width: 28px; height: 36px; font-size: 18px; }}
        .bonus-category {{ color: #d4af37; font-size: 14px; margin-bottom: 5px; text-align: center; }}
        .confirm-picks-btn {{ margin-top: 12px; padding: 12px 36px; font-size: 16px; }}
        .confirm-picks-btn:disabled {{ opacity: 0.5; }}
        .bonus-solve-section {{ background: rgba(26, 10, 62, 0.9); border: 2px solid #d4af37; border-radius: 14px; padding: 14px; text-align: center; max-width: 420px; width: 100%; }}
        .bonus-solve-input {{ width: 100%; padding: 10px; font-size: 18px; text-transform: uppercase; background: #0d0628; border: 2px solid #333; border-radius: 6px; color: #fff; margin-bottom: 10px; }}
        .bonus-solve-input:focus {{ outline: none; border-color: #d4af37; box-shadow: 0 0 16px rgba(212, 175, 55, 0.3); }}

        /* ========== GAME OVER OVERLAY ========== */
        .game-over-overlay {{
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: linear-gradient(180deg, rgba(13, 6, 40, 0.98) 0%, rgba(75, 0, 130, 0.95) 50%, rgba(13, 6, 40, 0.98) 100%);
            z-index: 2000;
            display: none;
            justify-content: center;
            align-items: center;
            flex-direction: column;
            padding: 20px;
            overflow: hidden;
        }}
        .game-over-overlay.active {{
            display: flex;
            animation: fadeInOverlay 0.5s ease-out;
        }}
        @keyframes fadeInOverlay {{
            from {{ opacity: 0; }}
            to {{ opacity: 1; }}
        }}
        .game-over-stars {{
            position: absolute;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            pointer-events: none;
            overflow: hidden;
        }}
        .game-over-content {{
            text-align: center;
            z-index: 1;
            max-width: 600px;
            width: 100%;
        }}
        .game-over-title {{
            font-family: 'Mountains of Christmas', cursive;
            font-size: 72px;
            font-weight: 700;
            color: #ffd700;
            text-shadow: 0 0 30px rgba(255, 215, 0, 0.8), 0 0 60px rgba(255, 215, 0, 0.5), 0 6px 0 #b8860b;
            margin-bottom: 10px;
            animation: titleBounce 0.8s ease-out;
        }}
        @keyframes titleBounce {{
            0% {{ transform: scale(0.3); opacity: 0; }}
            50% {{ transform: scale(1.1); }}
            100% {{ transform: scale(1); opacity: 1; }}
        }}
        .game-over-winner {{
            font-size: 24px;
            color: #fff;
            margin-bottom: 5px;
            animation: slideUp 0.5s ease-out 0.3s backwards;
        }}
        .game-over-winner-name {{
            font-family: 'Mountains of Christmas', cursive;
            font-size: 48px;
            font-weight: 700;
            color: #22c55e;
            text-shadow: 0 0 20px rgba(34, 197, 94, 0.5), 0 4px 0 #16a34a;
            margin-bottom: 10px;
            animation: slideUp 0.5s ease-out 0.4s backwards;
        }}
        .game-over-score {{
            font-family: 'Mountains of Christmas', cursive;
            font-size: 64px;
            font-weight: 700;
            color: #ffd700;
            text-shadow: 0 0 30px rgba(255, 215, 0, 0.8), 0 4px 0 #b8860b;
            margin-bottom: 30px;
            animation: scorePopIn 0.6s ease-out 0.5s backwards;
        }}
        @keyframes scorePopIn {{
            0% {{ transform: scale(0); opacity: 0; }}
            60% {{ transform: scale(1.2); }}
            100% {{ transform: scale(1); opacity: 1; }}
        }}
        @keyframes slideUp {{
            from {{ transform: translateY(30px); opacity: 0; }}
            to {{ transform: translateY(0); opacity: 1; }}
        }}
        .game-over-final-standings {{
            background: rgba(26, 10, 62, 0.9);
            border: 2px solid #d4af37;
            border-radius: 16px;
            padding: 20px;
            margin-bottom: 30px;
            animation: fadeIn 0.5s ease-out 0.6s backwards;
        }}
        @keyframes fadeIn {{
            from {{ opacity: 0; }}
            to {{ opacity: 1; }}
        }}
        .game-over-standings-title {{
            font-size: 20px;
            color: #d4af37;
            margin-bottom: 15px;
            text-transform: uppercase;
            letter-spacing: 2px;
        }}
        .game-over-standing {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px 15px;
            margin: 5px 0;
            background: rgba(255, 255, 255, 0.05);
            border-radius: 8px;
            transition: all 0.3s ease;
        }}
        .game-over-standing.winner {{
            background: linear-gradient(90deg, rgba(34, 197, 94, 0.3) 0%, rgba(34, 197, 94, 0.1) 100%);
            border: 1px solid rgba(34, 197, 94, 0.5);
        }}
        .game-over-standing-rank {{
            width: 30px;
            height: 30px;
            border-radius: 50%;
            background: linear-gradient(180deg, #d4af37 0%, #b8860b 100%);
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: bold;
            color: #1a1a2e;
            font-size: 14px;
        }}
        .game-over-standing-rank.gold {{ background: linear-gradient(180deg, #ffd700 0%, #d4af37 100%); }}
        .game-over-standing-rank.silver {{ background: linear-gradient(180deg, #c0c0c0 0%, #a0a0a0 100%); }}
        .game-over-standing-rank.bronze {{ background: linear-gradient(180deg, #cd7f32 0%, #a05a2c 100%); }}
        .game-over-standing-name {{
            flex: 1;
            text-align: left;
            margin-left: 15px;
            font-size: 18px;
            color: #fff;
        }}
        .game-over-standing-score {{
            font-size: 20px;
            font-weight: bold;
            color: #ffd700;
        }}
        .game-over-timer {{
            font-size: 16px;
            color: #888;
            margin-bottom: 20px;
            animation: fadeIn 0.5s ease-out 0.7s backwards;
        }}
        .game-over-countdown {{
            font-size: 24px;
            font-weight: bold;
            color: #d4af37;
        }}
        .game-over-btn {{
            font-size: 20px;
            padding: 15px 40px;
            animation: fadeIn 0.5s ease-out 0.8s backwards;
        }}
    </style>
</head>
<body>
    <script>
        // Seasonal theme detection
        (function() {{
            const now = new Date();
            const month = now.getMonth() + 1;
            const day = now.getDate();
            let theme = 'theme-summer';
            if (month === 12 && day <= 25) theme = 'theme-christmas';
            else if ((month === 12 && day >= 26) || (month === 1 && day <= 7)) theme = 'theme-newyear';
            else if (month === 2 && day <= 14) theme = 'theme-valentines';
            else if (month === 3 && day >= 10 && day <= 17) theme = 'theme-stpatricks';
            else if ((month === 3 && day >= 20) || (month === 4 && day <= 20)) theme = 'theme-easter';
            else if ((month === 6 && day >= 25) || (month === 7 && day <= 10)) theme = 'theme-july4th';
            else if (month === 10) theme = 'theme-halloween';
            else if (month === 11 && day >= 15) theme = 'theme-thanksgiving';
            document.body.classList.add(theme);
        }})();
    </script>
    <!-- Confetti Container -->
    <div id="confetti-container"></div>

    <!-- Phase Transition Overlay -->
    <div class="phase-overlay" id="phaseOverlay">
        <!-- Decorative frame corners -->
        <div class="phase-corner tl"></div>
        <div class="phase-corner tr"></div>
        <div class="phase-corner bl"></div>
        <div class="phase-corner br"></div>
        <!-- Floating sparkles -->
        <div class="phase-sparkles" id="phaseSparkles"></div>
        <!-- Background wheel silhouette -->
        <svg class="phase-wheel-bg" id="phaseWheelBg" viewBox="0 0 200 200">
            <circle cx="100" cy="100" r="90" fill="none" stroke="#ffd700" stroke-width="4" opacity="0.3"/>
            <circle cx="100" cy="100" r="70" fill="none" stroke="#ffd700" stroke-width="2" opacity="0.2"/>
            <circle cx="100" cy="100" r="50" fill="none" stroke="#ffd700" stroke-width="1" opacity="0.1"/>
            <g id="wheelSpokes">
                <line x1="100" y1="10" x2="100" y2="190" stroke="#ffd700" stroke-width="1" opacity="0.2"/>
                <line x1="10" y1="100" x2="190" y2="100" stroke="#ffd700" stroke-width="1" opacity="0.2"/>
                <line x1="27" y1="27" x2="173" y2="173" stroke="#ffd700" stroke-width="1" opacity="0.2"/>
                <line x1="173" y1="27" x2="27" y2="173" stroke="#ffd700" stroke-width="1" opacity="0.2"/>
            </g>
        </svg>
        <div class="phase-content">
            <div class="phase-title" id="phaseTitle">TOSS-UP!</div>
            <div class="phase-subtitle" id="phaseSubtitle">Buzz in to answer!</div>
            <div class="phase-prize" id="phasePrize" style="display: none;"></div>
        </div>
        <div class="phase-countdown" id="phaseCountdown"></div>
    </div>

    <!-- Bonus Round Overlay -->
    <div class="bonus-round-overlay" id="bonusRoundOverlay">
        <div class="bonus-stars" id="bonusStars"></div>
        <div class="bonus-header">
            <div class="bonus-title">BONUS ROUND</div>
            <div class="bonus-player-name" id="bonusPlayerName">Player Name</div>
        </div>
        <div class="bonus-content">
            <!-- Stage Indicator -->
            <div class="bonus-stage-indicator">
                <div class="bonus-stage" id="stagePrize">
                    <div class="bonus-stage-icon">🎰</div>
                    <div class="bonus-stage-label">Prize</div>
                </div>
                <div class="bonus-stage-connector" id="connectorPrizePick"></div>
                <div class="bonus-stage" id="stagePick">
                    <div class="bonus-stage-icon">✍️</div>
                    <div class="bonus-stage-label">Pick</div>
                </div>
                <div class="bonus-stage-connector" id="connectorPickSolve"></div>
                <div class="bonus-stage" id="stageSolve">
                    <div class="bonus-stage-icon">🎯</div>
                    <div class="bonus-stage-label">Solve</div>
                </div>
            </div>

            <!-- Prize Wheel Section -->
            <div id="bonusPrizeSection" style="display: none;">
                <div class="prize-wheel-container">
                    <div class="prize-wheel-pointer"></div>
                    <svg id="prizeWheelSvg" class="prize-wheel-svg" width="240" height="240" viewBox="0 0 240 240"></svg>
                </div>
                <div class="prize-wheel-result" id="prizeWheelResult">Spin to reveal your prize!</div>
                <button class="btn confirm-picks-btn" id="spinPrizeBtn" onclick="spinPrizeWheel()">Spin Prize Wheel</button>
            </div>

            <!-- RSTLNE Given Letters -->
            <div class="given-letters" id="givenLettersSection" style="display: none;">
                <div class="given-letters-label">Given Letters (R S T L N E)</div>
                <div class="given-letters-row">
                    <div class="given-letter" style="animation-delay: 0s">R</div>
                    <div class="given-letter" style="animation-delay: 0.1s">S</div>
                    <div class="given-letter" style="animation-delay: 0.2s">T</div>
                    <div class="given-letter" style="animation-delay: 0.3s">L</div>
                    <div class="given-letter" style="animation-delay: 0.4s">N</div>
                    <div class="given-letter" style="animation-delay: 0.5s">E</div>
                </div>
            </div>

            <!-- Picked Letters Display -->
            <div class="picked-letters" id="pickedLettersSection" style="display: none;">
                <div class="picked-letters-label">Your Picks (3 Consonants + 1 Vowel)</div>
                <div class="picked-letters-row" id="pickedLettersRow">
                    <div class="picked-letter empty" id="pick0">?</div>
                    <div class="picked-letter empty" id="pick1">?</div>
                    <div class="picked-letter empty" id="pick2">?</div>
                    <div class="picked-letter empty" id="pick3">?</div>
                </div>
            </div>

            <!-- Letter Pick Section -->
            <div class="letter-pick-section" id="letterPickSection" style="display: none;">
                <div class="letter-pick-title" id="letterPickTitle">Pick 3 Consonants</div>
                <div class="letter-pick-instruction" id="letterPickInstruction">Choose letters not in R S T L N E</div>
                <div class="letter-grid" id="consonantGrid"></div>
                <div class="letter-grid" id="vowelGrid" style="display: none; margin-top: 12px;"></div>
                <button class="btn confirm-picks-btn" id="confirmPicksBtn" onclick="confirmBonusPicks()" disabled>Confirm Picks</button>
            </div>

            <!-- Puzzle Board for Bonus Round -->
            <div id="bonusPuzzleSection" style="display: none;">
                <div class="bonus-category" id="bonusCategory">Category: -</div>
                <div class="bonus-puzzle-board" id="bonusPuzzleBoard"></div>
            </div>

            <!-- Timer -->
            <div class="bonus-timer" id="bonusTimerSection" style="display: none;">
                <svg class="bonus-timer-ring" viewBox="0 0 140 140">
                    <circle class="track" cx="70" cy="70" r="62"></circle>
                    <circle class="progress" id="timerProgress" cx="70" cy="70" r="62" stroke-dasharray="389.56" stroke-dashoffset="0"></circle>
                </svg>
                <div class="bonus-timer-text">
                    <div class="bonus-timer-seconds" id="bonusTimerSeconds">10</div>
                    <div class="bonus-timer-label">seconds</div>
                </div>
            </div>

            <!-- Solve Section -->
            <div class="bonus-solve-section" id="bonusSolveSection" style="display: none;">
                <input type="text" class="bonus-solve-input" id="bonusSolveInput" placeholder="Enter your solution..." autocomplete="off">
                <div style="display: flex; gap: 10px; justify-content: center;">
                    <button class="btn" onclick="submitBonusSolve()">Solve!</button>
                </div>
            </div>

            <!-- Result Section -->
            <div class="bonus-result" id="bonusResultSection" style="display: none;">
                <div class="bonus-result-icon" id="bonusResultIcon">🎉</div>
                <div class="bonus-result-title" id="bonusResultTitle">WINNER!</div>
                <div class="bonus-result-amount" id="bonusResultAmount">$50,000</div>
                <div class="bonus-result-answer" id="bonusResultAnswer">THE ANSWER WAS: EXAMPLE</div>
                <button class="btn" onclick="startNewGameFromBonus()" style="margin-top: 16px;">New Game</button>
            </div>
        </div>
    </div>

    <!-- Game Over Overlay -->
    <div class="game-over-overlay" id="gameOverOverlay">
        <div class="game-over-stars" id="gameOverStars"></div>
        <div class="game-over-content">
            <div class="game-over-title" id="gameOverTitle">GAME OVER</div>
            <div class="game-over-winner" id="gameOverWinner">Congratulations!</div>
            <div class="game-over-winner-name" id="gameOverWinnerName">Player Name</div>
            <div class="game-over-score" id="gameOverScore">$0</div>
            <div class="game-over-final-standings" id="gameOverStandings">
                <!-- Final standings will be populated by JS -->
            </div>
            <div class="game-over-timer">
                <span>New game starting in </span>
                <span class="game-over-countdown" id="gameOverCountdown">15</span>
                <span> seconds...</span>
            </div>
            <button class="btn btn-primary game-over-btn" onclick="startNewGameNow()">Play Again Now</button>
        </div>
    </div>

    <!-- Express Mode Indicator -->
    <div class="express-indicator" id="expressIndicator">
        <span class="express-label">⚡ EXPRESS MODE</span>
        <span class="express-streak" id="expressStreak">$0</span>
    </div>

    <div class="game-container">
        <div class="main-area">
            <div class="game-header">
                <h1>🎡 Holiday Wheel</h1>
                <div class="spectator-banner" id="spectatorBanner" style="display: none;">👁️ Spectator Mode - View Only</div>
                <div class="header-controls">
                    <button id="muteBtn" class="btn btn-icon" onclick="toggleMute()" title="Toggle Sound">🔊</button>
                    <a href="/lobby" class="btn btn-secondary leave-btn">Leave Room</a>
                </div>
            </div>

            <!-- Round Progress Indicator -->
            <div class="round-indicator" id="roundIndicator">
                <div class="round-header">
                    <span class="round-number" id="roundNumber">ROUND 1</span>
                    <span class="round-total" id="roundTotal">of 4</span>
                </div>
                <div class="round-dots" id="roundDots"></div>
                <div class="round-badges" id="roundBadges"></div>
            </div>

            <!-- Toss-Up Value Display -->
            <div class="tossup-display" id="tossupDisplay">
                <div class="triple-header" id="tripleHeader" style="display: none;">
                    <span class="triple-label">TRIPLE TOSS-UP</span>
                    <div class="triple-indicators" id="tripleIndicators">
                        <div class="triple-dot" id="triple0"></div>
                        <div class="triple-dot" id="triple1"></div>
                        <div class="triple-dot" id="triple2"></div>
                    </div>
                </div>
                <div class="tossup-for">FOR</div>
                <div class="tossup-value" id="tossupValue">$1,000</div>
            </div>

            <div class="game-layout">
                <div class="wheel-area">
                    <div class="wheel-container" id="wheelContainer">
                        <div class="wheel-glow"></div>
                        <div class="wheel-outer-rim"></div>
                        <div class="wheel-tick-flash" id="wheelTickFlash"></div>
                        <div class="wheel-pointer"></div>
                        <svg id="wheelSvg" class="wheel-svg" width="340" height="340" viewBox="0 0 340 340"></svg>
                        <div class="wheel-inner-rim"></div>
                        <div class="wheel-center-hub"></div>
                    </div>
                    <div class="wheel-result" id="wheelResult">Spin!</div>
                </div>

                <div class="puzzle-section">
                    <div class="theme" id="theme" style="color: #888; font-size: 12px; text-align: center; margin-bottom: 4px;"></div>
                    <div class="category">Category: <span id="category">-</span></div>
                    <div class="puzzle-board" id="puzzleBoard">
                        <p style="color: #fff;">Connecting to game...</p>
                    </div>
                </div>
            </div>

            <div class="notification" id="notification"></div>

            <div class="controls" id="controls">
                <button class="btn btn-success" id="startGameBtn" onclick="startGame()" style="display: none; font-size: 24px; padding: 16px 32px;">Start Game</button>
                <button class="btn" id="spinBtn" onclick="spin()">Spin</button>
                <button class="btn" id="finalSpinSpinBtn" onclick="doFinalSpin()" style="display: none;">Spin (Final)</button>
                <button class="btn btn-secondary" id="buyVowelBtn" onclick="buyVowel()">Buy Vowel ($250)</button>
                <button class="btn btn-secondary" id="solveBtn" onclick="promptSolve()">Solve</button>
                <button class="btn wildcard-btn" id="wildcardBtn" onclick="useWildCard()">
                    <span class="icon">🃏</span> Wild Card
                </button>
                <button class="btn btn-danger" id="buzzBtn" onclick="buzz()" style="display: none;">🔔 BUZZ IN!</button>
            </div>

            <div class="final-spin-value" id="finalSpinValue" style="display: none; text-align: center; margin: 16px 0; padding: 12px; background: linear-gradient(135deg, #d4af37 0%, #f0d060 50%, #d4af37 100%); border-radius: 12px; box-shadow: 0 4px 15px rgba(212, 175, 55, 0.4);">
                <span style="color: #1a1a2e; font-size: 14px; font-weight: bold; display: block;">FINAL SPIN VALUE</span>
                <span id="finalSpinAmount" style="color: #1a1a2e; font-size: 32px; font-weight: bold;">$0</span>
                <span style="color: #1a1a2e; font-size: 12px; display: block;">+ $1,000 per letter</span>
            </div>

            <div class="input-timer-row">
                <div class="turn-timer" id="turnTimer">
                    <span class="turn-timer-icon">⏱️</span>
                    <span class="turn-timer-text" id="turnTimerText">10s</span>
                </div>

                <div class="turn-timer" id="buzzTimer" style="display: none;">
                    <span class="turn-timer-icon">🔔</span>
                    <span class="turn-timer-text" id="buzzTimerText">5s</span>
                </div>

                <div class="guess-input" id="guessArea">
                    <label class="guess-label" for="letterInput">Select a Consonant</label>
                    <input type="text" id="letterInput" maxlength="1" placeholder="A-Z" style="font-size: 24px; padding: 0;">
                </div>
            </div>

            <div class="host-controls" id="hostControls" style="display: none; margin-top: 20px; padding-top: 16px; border-top: 1px solid #333;">
                <p style="color: #d4af37; margin-bottom: 12px; font-weight: bold;">Host Controls</p>
                <div style="display: flex; gap: 8px; flex-wrap: wrap; align-items: center; margin-bottom: 10px;">
                    <button class="btn" onclick="newGame()">New Game</button>
                    <button class="btn btn-secondary" onclick="newPuzzle()">New Puzzle</button>
                    <button class="btn btn-secondary" onclick="hostSpin()">Spin</button>
                    <button class="btn btn-secondary" onclick="revealAll()">Reveal All</button>
                </div>
                <div style="display: flex; gap: 8px; flex-wrap: wrap; align-items: center; margin-bottom: 10px;">
                    <button class="btn" id="tossupBtn" onclick="toggleTossup()">Start Toss-up</button>
                    <button class="btn" id="finalSpinBtn" onclick="toggleFinalSpin()">Start Final Spin</button>
                    <button class="btn" id="bonusBtn" onclick="toggleBonus()">Start Bonus</button>
                </div>
                <div style="display: flex; gap: 8px; flex-wrap: wrap; align-items: center; margin-bottom: 10px;">
                    <span style="color: #888; font-size: 14px;">Round:</span>
                    <select id="roundSelect" onchange="setRound()" style="padding: 8px 12px; border-radius: 8px; background: #1a1a2e; color: #fff; border: 2px solid #333; font-size: 14px; cursor: pointer; min-width: 80px;">
                        <option value="1">1</option>
                        <option value="2">2</option>
                        <option value="3">3</option>
                        <option value="4">4</option>
                    </select>
                    <button class="btn btn-secondary" onclick="advanceRound()">Next Round</button>
                </div>
                <div style="display: flex; gap: 8px; flex-wrap: wrap; align-items: center;">
                    <select id="packSelect" onchange="changePack()" style="padding: 8px 12px; border-radius: 8px; background: #1a1a2e; color: #fff; border: 2px solid #333; font-size: 14px; cursor: pointer;">
                        <option value="">All Packs</option>
                    </select>
                    <select id="activePlayerSelect" onchange="setActivePlayer()" style="padding: 8px 12px; border-radius: 8px; background: #1a1a2e; color: #fff; border: 2px solid #333; font-size: 14px; cursor: pointer;">
                        <option value="">Set Active Player</option>
                    </select>
                </div>
            </div>
        </div>

        <div class="sidebar">
            <h2 style="color: #fff; margin-bottom: 16px;">Players</h2>
            <div class="player-list" id="playerList">
                <p style="color: #888;">No players yet</p>
            </div>

            <div id="claimHostSection" style="margin-top: 24px; padding-top: 16px; border-top: 1px solid #333;">
                <button class="btn btn-secondary" onclick="promptClaimHost()" style="width: 100%; font-size: 14px;">Claim Host</button>
            </div>

            <div style="margin-top: 16px; padding-top: 16px; border-top: 1px solid #333;">
                <p style="color: #888; font-size: 14px;">Room: <span id="roomName">-</span></p>
                <p style="color: #888; font-size: 14px; margin-top: 4px;">Round: <span id="roundDisplay" style="color: #d4af37; font-weight: bold;">1</span> / 4</p>
                <p style="color: #888; font-size: 14px; margin-top: 4px;">Phase: <span id="phase">Connecting...</span></p>
            </div>
        </div>
    </div>

    <!-- Solve Modal (bottom positioned) -->
    <div class="modal-overlay bottom-modal" id="solveModal">
        <div class="modal">
            <h2>Solve the Puzzle</h2>
            <input type="text" id="solveInput" placeholder="Enter your solution" autocomplete="off">
            <div class="modal-buttons">
                <button class="btn" onclick="submitSolve()">Submit</button>
                <button class="btn btn-secondary" onclick="closeModal('solveModal')">Cancel</button>
            </div>
        </div>
    </div>

    <!-- Buy Vowel Modal (bottom positioned) -->
    <div class="modal-overlay bottom-modal" id="vowelModal">
        <div class="modal">
            <h2>Buy a Vowel ($250)</h2>
            <div class="vowel-buttons">
                <button class="btn vowel-btn" onclick="selectVowel('A')">A</button>
                <button class="btn vowel-btn" onclick="selectVowel('E')">E</button>
                <button class="btn vowel-btn" onclick="selectVowel('I')">I</button>
                <button class="btn vowel-btn" onclick="selectVowel('O')">O</button>
                <button class="btn vowel-btn" onclick="selectVowel('U')">U</button>
            </div>
            <div class="modal-buttons">
                <button class="btn btn-secondary" onclick="closeModal('vowelModal')">Cancel</button>
            </div>
        </div>
    </div>

    <!-- Claim Host Modal -->
    <div class="modal-overlay" id="claimHostModal">
        <div class="modal">
            <h2>Claim Host</h2>
            <input type="password" id="hostCodeInput" placeholder="Enter host code" autocomplete="off">
            <div class="modal-buttons">
                <button class="btn" onclick="submitClaimHost()">Submit</button>
                <button class="btn btn-secondary" onclick="closeModal('claimHostModal')">Cancel</button>
            </div>
        </div>
    </div>

    <!-- Mystery Wedge Modal -->
    <div class="modal-overlay" id="mysteryModal">
        <div class="modal mystery-modal">
            <h2>🎭 Mystery Wedge!</h2>
            <p style="color: #ccc; margin-bottom: 16px;">Choose your fate...</p>
            <div class="mystery-options" id="mysteryOptions">
                <div class="mystery-option keep" onclick="mysteryChoice('keep')">
                    <span class="amount">$1,000</span>
                    <span class="label">Keep it safe</span>
                </div>
                <div class="mystery-option flip" onclick="mysteryChoice('flip')">
                    <span class="amount">$10,000?</span>
                    <span class="label">Risk it all!</span>
                </div>
            </div>
            <div class="mystery-result" id="mysteryResult" style="display: none;"></div>
            <div class="modal-buttons" id="mysteryClose" style="display: none;">
                <button class="btn" onclick="closeModal('mysteryModal')">Continue</button>
            </div>
        </div>
    </div>

    <!-- Wild Card Modal -->
    <div class="modal-overlay" id="wildcardModal">
        <div class="modal">
            <h2>🃏 Use Wild Card</h2>
            <p style="color: #ccc; margin-bottom: 16px;">Pick any consonant - even one already called!</p>
            <input type="text" id="wildcardInput" maxlength="1" placeholder="Enter a consonant" autocomplete="off">
            <div class="modal-buttons">
                <button class="btn" onclick="submitWildCard()">Use Card</button>
                <button class="btn btn-secondary" onclick="closeModal('wildcardModal')">Cancel</button>
            </div>
        </div>
    </div>

    <script src="https://cdn.socket.io/4.7.5/socket.io.min.js"></script>
    <script>
        const user = JSON.parse(localStorage.getItem('user') || 'null');
        if (!user) {{ window.location.href = '/'; }}

        // Get auth token from localStorage
        const token = localStorage.getItem('auth_token') || '';
        console.log('Auth token from localStorage:', token ? 'present (' + token.substring(0, 20) + '...)' : 'MISSING - please log out and log in again');

        const urlParams = new URLSearchParams(window.location.search);
        const room = urlParams.get('room') || 'main';
        const isSpectating = urlParams.get('spectate') === 'true';
        document.getElementById('roomName').textContent = room + (isSpectating ? ' (Spectating)' : '');

        let socket;
        let gameState = null;
        let myPlayerIdx = null;

        // Wheel animation state - 28 unique colors for wheel wedges
        const WHEEL_COLORS = [
            '#c41e3a', '#0047ab', '#ff8c00', '#ffcc00', '#9932cc', '#ff1493',
            '#008b8b', '#dc143c', '#4169e1', '#ff4500', '#32cd32', '#9400d3',
            '#ff69b4', '#1e90ff', '#ffd700', '#00ced1', '#ff6347', '#8a2be2',
            '#20b2aa', '#ff7f50', '#6495ed', '#daa520', '#b22222', '#228b22',
            '#4682b4', '#cd853f', '#8b0000', '#2e8b57',
        ];
        let wheelRotation = 0;
        let prevSpinIdx = null;
        let wheelAnimationId = null;
        let isWheelSpinning = false;
        let pendingWheelResult = null;
        let pendingToasts = [];
        let prevPuzzleSolvedBy = null;
        let prevRevealed = new Set();
        let prevPhase = null;
        let prevRound = null;
        let prevScores = {{}};

        // ========== SOUND EFFECTS ==========
        const SoundService = {{
            enabled: localStorage.getItem('soundEnabled') !== 'false',
            sounds: {{}},
            audioContext: null,

            init() {{
                try {{
                    this.audioContext = new (window.AudioContext || window.webkitAudioContext)();
                    // Update mute button on init
                    this.updateMuteButton();
                }} catch (e) {{
                    console.log('Web Audio API not supported');
                }}
            }},

            toggle() {{
                this.enabled = !this.enabled;
                localStorage.setItem('soundEnabled', this.enabled);
                this.updateMuteButton();
            }},

            updateMuteButton() {{
                const btn = document.getElementById('muteBtn');
                if (btn) {{
                    btn.textContent = this.enabled ? '🔊' : '🔇';
                    btn.classList.toggle('muted', !this.enabled);
                    btn.title = this.enabled ? 'Mute Sound' : 'Unmute Sound';
                }}
            }},

            playTone(frequency, duration, type = 'sine') {{
                if (!this.enabled || !this.audioContext) return;
                try {{
                    const osc = this.audioContext.createOscillator();
                    const gain = this.audioContext.createGain();
                    osc.connect(gain);
                    gain.connect(this.audioContext.destination);
                    osc.type = type;
                    osc.frequency.value = frequency;
                    gain.gain.setValueAtTime(0.3, this.audioContext.currentTime);
                    gain.gain.exponentialRampToValueAtTime(0.01, this.audioContext.currentTime + duration);
                    osc.start();
                    osc.stop(this.audioContext.currentTime + duration);
                }} catch (e) {{ }}
            }},

            wheelTick() {{ this.playTone(800, 0.05, 'square'); }},
            wheelStop() {{ this.playTone(600, 0.3, 'sine'); }},
            letterCorrect() {{ this.playTone(880, 0.15, 'sine'); this.playTone(1100, 0.15, 'sine'); }},
            letterWrong() {{ this.playTone(200, 0.3, 'sawtooth'); }},
            bankrupt() {{ this.playTone(100, 0.5, 'sawtooth'); }},
            loseTurn() {{
                // Sad trombone - descending notes
                [300, 280, 250, 200].forEach((f, i) => {{
                    setTimeout(() => this.playTone(f, 0.2, 'triangle'), i * 180);
                }});
            }},
            solve() {{
                [523, 659, 784, 1047].forEach((f, i) => {{
                    setTimeout(() => this.playTone(f, 0.2, 'sine'), i * 100);
                }});
            }},
            buzz() {{ this.playTone(440, 0.1, 'square'); }},
            // Multi-letter reveal ding - cascading tones
            multiLetterDing(count) {{
                const baseFreq = 880;
                for (let i = 0; i < Math.min(count, 5); i++) {{
                    setTimeout(() => {{
                        this.playTone(baseFreq + (i * 110), 0.12, 'sine');
                    }}, i * 120);
                }}
            }},
            // Vowel purchase sound - softer tone
            vowelPurchase() {{
                this.playTone(660, 0.1, 'sine');
                setTimeout(() => this.playTone(880, 0.1, 'sine'), 80);
            }},
            bonusDrumroll() {{
                for (let i = 0; i < 10; i++) {{
                    setTimeout(() => this.playTone(200 + i * 30, 0.08, 'square'), i * 50);
                }}
            }},
            bonusWin() {{
                [523, 659, 784, 1047, 1319].forEach((f, i) => {{
                    setTimeout(() => this.playTone(f, 0.3, 'sine'), i * 120);
                }});
            }},
            bonusLose() {{
                [400, 300, 200].forEach((f, i) => {{
                    setTimeout(() => this.playTone(f, 0.4, 'sawtooth'), i * 200);
                }});
            }},
            timerTick() {{
                this.playTone(600, 0.05, 'sine');
            }},
            timerUrgent() {{
                this.playTone(800, 0.1, 'square');
            }},
            timesUp() {{
                // Buzzer sound - descending harsh tones
                [500, 400, 300].forEach((f, i) => {{
                    setTimeout(() => this.playTone(f, 0.25, 'sawtooth'), i * 150);
                }});
            }},
        }};

        // ========== BONUS ROUND STATE ==========
        const BONUS_PRIZES = [25000, 30000, 35000, 50000, 75000, 100000];
        const RSTLNE = ['R', 'S', 'T', 'L', 'N', 'E'];
        const CONSONANTS = ['B', 'C', 'D', 'F', 'G', 'H', 'J', 'K', 'M', 'P', 'Q', 'V', 'W', 'X', 'Y', 'Z'];
        const VOWELS = ['A', 'I', 'O', 'U'];

        let bonusState = {{
            active: false,
            stage: 'prize', // prize, pick, solve, result
            prizeAmount: 0,
            prizeWheelRotation: 0,
            pickedConsonants: [],
            pickedVowel: null,
            timerInterval: null,
            totalSeconds: 10,
            remainingSeconds: 10,
        }};

        // Initialize bonus round stars
        function initBonusStars() {{
            const container = document.getElementById('bonusStars');
            container.innerHTML = '';
            for (let i = 0; i < 50; i++) {{
                const star = document.createElement('div');
                star.className = 'bonus-star';
                star.style.left = Math.random() * 100 + '%';
                star.style.top = Math.random() * 100 + '%';
                star.style.animationDelay = Math.random() * 2 + 's';
                star.style.width = (2 + Math.random() * 4) + 'px';
                star.style.height = star.style.width;
                container.appendChild(star);
            }}
        }}

        // Render the prize wheel
        function renderPrizeWheel() {{
            const svg = document.getElementById('prizeWheelSvg');
            const size = 240;
            const radius = size / 2 - 8;
            const cx = size / 2;
            const cy = size / 2;
            const numSlots = BONUS_PRIZES.length;
            const anglePerSlot = 360 / numSlots;
            const colors = ['#c41e3a', '#0047ab', '#ff8c00', '#22c55e', '#9932cc', '#ffd700'];

            let html = '';
            BONUS_PRIZES.forEach((prize, idx) => {{
                const startAngle = idx * anglePerSlot - 90;
                const endAngle = startAngle + anglePerSlot;
                const startRad = startAngle * Math.PI / 180;
                const endRad = endAngle * Math.PI / 180;

                const x1 = cx + radius * Math.cos(startRad);
                const y1 = cy + radius * Math.sin(startRad);
                const x2 = cx + radius * Math.cos(endRad);
                const y2 = cy + radius * Math.sin(endRad);

                const largeArc = anglePerSlot > 180 ? 1 : 0;
                const pathD = `M ${{cx}} ${{cy}} L ${{x1}} ${{y1}} A ${{radius}} ${{radius}} 0 ${{largeArc}} 1 ${{x2}} ${{y2}} Z`;

                html += `<path d="${{pathD}}" fill="${{colors[idx]}}" stroke="#111" stroke-width="2"/>`;

                const midAngle = (startAngle + endAngle) / 2;
                const midRad = midAngle * Math.PI / 180;
                const textRadius = radius * 0.65;
                const textX = cx + textRadius * Math.cos(midRad);
                const textY = cy + textRadius * Math.sin(midRad);

                let rotation = midAngle;
                const normAngle = ((midAngle % 360) + 360) % 360;
                if (normAngle > 90 && normAngle < 270) rotation = midAngle + 180;

                const prizeText = '$' + (prize / 1000) + 'K';
                html += `<text x="${{textX}}" y="${{textY}}" fill="#fff" stroke="#000" stroke-width="0.5" font-size="18" font-weight="bold" text-anchor="middle" dominant-baseline="middle" transform="rotate(${{rotation}}, ${{textX}}, ${{textY}})" style="paint-order: stroke fill">${{prizeText}}</text>`;
            }});

            html += `<circle cx="${{cx}}" cy="${{cy}}" r="20" fill="#2a2a2a" stroke="#d4af37" stroke-width="3"/>`;
            html += `<circle cx="${{cx}}" cy="${{cy}}" r="10" fill="#d4af37"/>`;

            svg.innerHTML = html;
        }}

        // Spin the prize wheel
        function spinPrizeWheel() {{
            const btn = document.getElementById('spinPrizeBtn');
            btn.disabled = true;
            btn.textContent = 'Spinning...';

            SoundService.bonusDrumroll();

            const targetIdx = Math.floor(Math.random() * BONUS_PRIZES.length);
            const prize = BONUS_PRIZES[targetIdx];
            bonusState.prizeAmount = prize;

            const anglePerSlot = 360 / BONUS_PRIZES.length;
            const wedgeCenterAngle = targetIdx * anglePerSlot + anglePerSlot / 2;
            const finalAngle = (360 - wedgeCenterAngle) % 360;

            const spins = 4;
            const targetRotation = bonusState.prizeWheelRotation + spins * 360 + finalAngle;
            const startRotation = bonusState.prizeWheelRotation;
            const duration = 4000;
            const startTime = performance.now();

            function animate(currentTime) {{
                const elapsed = currentTime - startTime;
                const progress = Math.min(elapsed / duration, 1);
                const eased = 1 - Math.pow(1 - progress, 3);
                bonusState.prizeWheelRotation = startRotation + (targetRotation - startRotation) * eased;

                document.getElementById('prizeWheelSvg').style.transform = `rotate(${{bonusState.prizeWheelRotation}}deg)`;

                if (progress < 1) {{
                    requestAnimationFrame(animate);
                }} else {{
                    document.getElementById('prizeWheelResult').textContent = 'You could win $' + prize.toLocaleString() + '!';
                    btn.textContent = 'Continue to Letter Pick';
                    btn.disabled = false;
                    btn.onclick = () => showBonusStage('pick');
                }}
            }}
            requestAnimationFrame(animate);
        }}

        // Show a specific bonus round stage
        function showBonusStage(stage) {{
            bonusState.stage = stage;

            // Update stage indicators
            const stages = ['prize', 'pick', 'solve'];
            stages.forEach((s, i) => {{
                const el = document.getElementById('stage' + s.charAt(0).toUpperCase() + s.slice(1));
                const connEl = i < stages.length - 1 ? document.getElementById('connector' + s.charAt(0).toUpperCase() + s.slice(1) + stages[i + 1].charAt(0).toUpperCase() + stages[i + 1].slice(1)) : null;

                el.classList.remove('active', 'completed');
                if (connEl) connEl.classList.remove('completed');

                if (s === stage) {{
                    el.classList.add('active');
                }} else if (stages.indexOf(s) < stages.indexOf(stage)) {{
                    el.classList.add('completed');
                    if (connEl) connEl.classList.add('completed');
                }}
            }});

            // Hide all sections
            document.getElementById('bonusPrizeSection').style.display = 'none';
            document.getElementById('letterPickSection').style.display = 'none';
            document.getElementById('bonusPuzzleSection').style.display = 'none';
            document.getElementById('bonusTimerSection').style.display = 'none';
            document.getElementById('bonusSolveSection').style.display = 'none';
            document.getElementById('bonusResultSection').style.display = 'none';
            document.getElementById('givenLettersSection').style.display = 'none';
            document.getElementById('pickedLettersSection').style.display = 'none';

            switch (stage) {{
                case 'prize':
                    // Prize wheel stage is now skipped - go directly to pick
                    // But keep for backwards compatibility
                    document.getElementById('bonusPrizeSection').style.display = 'block';
                    renderPrizeWheel();
                    break;
                case 'pick':
                    document.getElementById('letterPickSection').style.display = 'block';
                    document.getElementById('givenLettersSection').style.display = 'flex';
                    document.getElementById('pickedLettersSection').style.display = 'flex';
                    // Also show puzzle board during pick so player can see what they're solving
                    document.getElementById('bonusPuzzleSection').style.display = 'block';
                    initLetterPick();
                    break;
                case 'solve':
                    document.getElementById('bonusPuzzleSection').style.display = 'block';
                    document.getElementById('bonusTimerSection').style.display = 'block';
                    document.getElementById('bonusSolveSection').style.display = 'block';
                    document.getElementById('givenLettersSection').style.display = 'flex';
                    document.getElementById('pickedLettersSection').style.display = 'flex';
                    // Only start timer if not already running (server controls timing via updateBonusTimer)
                    if (!bonusState.timerInterval) {{
                        startBonusTimer();
                    }}
                    document.getElementById('bonusSolveInput').focus();
                    break;
                case 'result':
                    document.getElementById('bonusResultSection').style.display = 'block';
                    break;
            }}
        }}

        // Initialize letter pick UI - syncs with server state
        function initLetterPick() {{
            // Sync from server state if available
            const bonusData = gameState?.bonus;
            if (bonusData?.picks) {{
                bonusState.pickedConsonants = bonusData.picks.consonants || [];
                bonusState.pickedVowel = bonusData.picks.vowel || null;
            }} else {{
                bonusState.pickedConsonants = [];
                bonusState.pickedVowel = null;
            }}

            // Already used letters (from server state)
            const usedLetters = new Set(gameState?.used || []);

            const consonantGrid = document.getElementById('consonantGrid');
            consonantGrid.innerHTML = CONSONANTS.map(c => {{
                // Disable if in RSTLNE, already used, or already picked
                const isRstlne = RSTLNE.includes(c);
                const isUsed = usedLetters.has(c);
                const isPicked = bonusState.pickedConsonants.includes(c);
                const disabled = isRstlne || isUsed ? 'disabled' : '';
                const selected = isPicked ? 'selected' : '';
                return `<button class="letter-pick-btn ${{selected}}" ${{disabled}} onclick="pickConsonant('${{c}}')">${{c}}</button>`;
            }}).join('');

            const vowelGrid = document.getElementById('vowelGrid');
            vowelGrid.innerHTML = VOWELS.map(v => {{
                const isRstlne = RSTLNE.includes(v);
                const isUsed = usedLetters.has(v);
                const isPicked = v === bonusState.pickedVowel;
                const disabled = isRstlne || isUsed ? 'disabled' : '';
                const selected = isPicked ? 'selected' : '';
                return `<button class="letter-pick-btn ${{selected}}" ${{disabled}} onclick="pickVowel('${{v}}')">${{v}}</button>`;
            }}).join('');

            // Show vowel grid if 3 consonants picked, hide otherwise
            vowelGrid.style.display = bonusState.pickedConsonants.length >= 3 ? 'flex' : 'none';

            // Update title based on current state
            if (bonusState.pickedConsonants.length < 3) {{
                const remaining = 3 - bonusState.pickedConsonants.length;
                document.getElementById('letterPickTitle').textContent = `Pick ${{remaining}} Consonant${{remaining > 1 ? 's' : ''}}`;
                document.getElementById('letterPickInstruction').textContent = 'Choose letters not in R S T L N E';
            }} else if (!bonusState.pickedVowel) {{
                document.getElementById('letterPickTitle').textContent = 'Pick 1 Vowel';
                document.getElementById('letterPickInstruction').textContent = 'Choose A, I, O, or U';
            }} else {{
                document.getElementById('letterPickTitle').textContent = 'All letters picked!';
                document.getElementById('letterPickInstruction').textContent = 'Revealing letters...';
            }}

            updatePickedLettersDisplay();

            // Hide confirm button - server auto-starts when all picks complete
            document.getElementById('confirmPicksBtn').style.display = 'none';
        }}

        function pickConsonant(c) {{
            if (bonusState.pickedConsonants.length >= 3) return;
            if (bonusState.pickedConsonants.includes(c)) return;

            bonusState.pickedConsonants.push(c);
            SoundService.letterCorrect();

            // Mark button as selected
            const btns = document.querySelectorAll('#consonantGrid .letter-pick-btn');
            btns.forEach(btn => {{
                if (btn.textContent === c) btn.classList.add('selected');
            }});

            updatePickedLettersDisplay();

            // Send to server immediately
            socket.emit('bonus_pick', {{ room, kind: 'consonant', letter: c }});

            if (bonusState.pickedConsonants.length === 3) {{
                document.getElementById('letterPickTitle').textContent = 'Pick 1 Vowel';
                document.getElementById('letterPickInstruction').textContent = 'Choose A, I, O, or U';
                document.getElementById('vowelGrid').style.display = 'flex';
            }}
        }}

        function pickVowel(v) {{
            if (bonusState.pickedVowel) return;

            bonusState.pickedVowel = v;
            SoundService.letterCorrect();

            // Mark button as selected
            const btns = document.querySelectorAll('#vowelGrid .letter-pick-btn');
            btns.forEach(btn => {{
                if (btn.textContent === v) btn.classList.add('selected');
            }});

            updatePickedLettersDisplay();

            // Send to server immediately - this completes the picks
            // Server auto-starts timer when all picks are complete
            socket.emit('bonus_pick', {{ room, kind: 'vowel', letter: v }});

            // Hide confirm button since server auto-advances
            document.getElementById('confirmPicksBtn').style.display = 'none';
        }}

        function updatePickedLettersDisplay() {{
            const picks = [...bonusState.pickedConsonants];
            if (bonusState.pickedVowel) picks.push(bonusState.pickedVowel);

            for (let i = 0; i < 4; i++) {{
                const el = document.getElementById('pick' + i);
                if (picks[i]) {{
                    el.textContent = picks[i];
                    el.classList.remove('empty');
                    el.style.animationDelay = (i * 0.1) + 's';
                }} else {{
                    el.textContent = '?';
                    el.classList.add('empty');
                }}
            }}
        }}

        function confirmBonusPicks() {{
            // This function is now deprecated - picks are sent immediately
            // Server auto-starts timer when all picks are complete
            // Kept for backwards compatibility
            if (bonusState.pickedConsonants.length < 3 || !bonusState.pickedVowel) return;

            // If for some reason the picks weren't sent, send them now
            bonusState.pickedConsonants.forEach(c => {{
                socket.emit('bonus_pick', {{ room, kind: 'consonant', letter: c }});
            }});
            if (bonusState.pickedVowel) {{
                socket.emit('bonus_pick', {{ room, kind: 'vowel', letter: bonusState.pickedVowel }});
            }}
        }}

        function startBonusTimer() {{
            const totalTime = bonusState.totalSeconds;
            let remaining = totalTime;

            const circumference = 2 * Math.PI * 62; // r=62
            const progressEl = document.getElementById('timerProgress');
            const secondsEl = document.getElementById('bonusTimerSeconds');

            progressEl.style.strokeDasharray = circumference;
            progressEl.style.strokeDashoffset = 0;

            if (bonusState.timerInterval) clearInterval(bonusState.timerInterval);

            bonusState.timerInterval = setInterval(() => {{
                remaining--;
                bonusState.remainingSeconds = remaining;

                const offset = circumference * (1 - remaining / totalTime);
                progressEl.style.strokeDashoffset = offset;
                secondsEl.textContent = remaining;

                // Change colors based on time remaining
                progressEl.classList.remove('warning', 'danger');
                secondsEl.classList.remove('warning', 'danger');

                if (remaining <= 3) {{
                    progressEl.classList.add('danger');
                    secondsEl.classList.add('danger');
                    SoundService.timerUrgent();
                }} else if (remaining <= 5) {{
                    progressEl.classList.add('warning');
                    secondsEl.classList.add('warning');
                    SoundService.timerTick();
                }} else {{
                    SoundService.timerTick();
                }}

                if (remaining <= 0) {{
                    clearInterval(bonusState.timerInterval);
                    showBonusResult(false);
                }}
            }}, 1000);
        }}

        // Update timer display from server state (without starting interval)
        function updateBonusTimer(remaining) {{
            const totalTime = bonusState.totalSeconds;
            const circumference = 2 * Math.PI * 62;
            const progressEl = document.getElementById('timerProgress');
            const secondsEl = document.getElementById('bonusTimerSeconds');

            if (!progressEl || !secondsEl) return;

            progressEl.style.strokeDasharray = circumference;
            const offset = circumference * (1 - remaining / totalTime);
            progressEl.style.strokeDashoffset = offset;
            secondsEl.textContent = remaining;

            // Update colors based on time remaining
            progressEl.classList.remove('warning', 'danger');
            secondsEl.classList.remove('warning', 'danger');

            if (remaining <= 3) {{
                progressEl.classList.add('danger');
                secondsEl.classList.add('danger');
            }} else if (remaining <= 5) {{
                progressEl.classList.add('warning');
                secondsEl.classList.add('warning');
            }}
        }}

        function submitBonusSolve() {{
            const input = document.getElementById('bonusSolveInput');
            const solution = input.value.trim();
            if (!solution) return;

            socket.emit('solve', {{ room, attempt: solution }});
            input.value = '';
        }}

        function showBonusResult(won, prizeAmount = null, answer = '') {{
            if (bonusState.timerInterval) {{
                clearInterval(bonusState.timerInterval);
                bonusState.timerInterval = null;
            }}

            showBonusStage('result');

            const iconEl = document.getElementById('bonusResultIcon');
            const titleEl = document.getElementById('bonusResultTitle');
            const amountEl = document.getElementById('bonusResultAmount');
            const answerEl = document.getElementById('bonusResultAnswer');
            const resultSection = document.getElementById('bonusResultSection');

            const prize = prizeAmount || bonusState.prizeAmount || 0;

            // Add winner stars container for celebration
            let starsContainer = resultSection.querySelector('.winner-stars');
            if (!starsContainer) {{
                starsContainer = document.createElement('div');
                starsContainer.className = 'winner-stars';
                resultSection.insertBefore(starsContainer, resultSection.firstChild);
            }}
            starsContainer.innerHTML = '';

            if (won) {{
                iconEl.textContent = '🎉';
                titleEl.textContent = 'WINNER!';
                titleEl.className = 'bonus-result-title win';

                // Animate prize amount counting up
                amountEl.textContent = '$0';
                setTimeout(() => {{
                    animateCountUp(amountEl, 0, prize, 2500);
                }}, 400);

                // Full celebration effects
                celebrateWinner(null, prize);

                // Add floating stars around the result
                const starEmojis = ['⭐', '✨', '🌟', '💫'];
                for (let i = 0; i < 12; i++) {{
                    setTimeout(() => {{
                        const star = document.createElement('div');
                        star.className = 'winner-star';
                        star.textContent = starEmojis[Math.floor(Math.random() * starEmojis.length)];
                        star.style.left = (10 + Math.random() * 80) + '%';
                        star.style.top = (10 + Math.random() * 80) + '%';
                        star.style.animationDelay = (Math.random() * 0.5) + 's';
                        starsContainer.appendChild(star);
                        setTimeout(() => star.remove(), 1800);
                    }}, i * 200);
                }}

                // Additional confetti waves
                setTimeout(() => launchConfetti(100, {{ gold: true }}), 1500);
                setTimeout(() => launchSparkles(20), 2000);
            }} else {{
                iconEl.textContent = '😢';
                titleEl.textContent = 'TIME\'S UP!';
                titleEl.className = 'bonus-result-title lose';
                amountEl.textContent = '$0';
                SoundService.bonusLose();
            }}

            answerEl.textContent = 'The answer was: ' + (answer || gameState?.puzzle?.answer || '???');
        }}

        function closeBonusRound() {{
            bonusState.active = false;
            document.getElementById('bonusRoundOverlay').classList.remove('active');
            // Server controls game state - don't automatically emit new_game
        }}

        function startNewGameFromBonus() {{
            closeBonusRound();
            socket.emit('new_game', {{ room }});
        }}

        // ========== GAME OVER FUNCTIONS ==========
        let gameOverCountdownInterval = null;

        function showGameOver(gameOverData, players) {{
            // Close bonus overlay if open
            closeBonusRound();

            const overlay = document.getElementById('gameOverOverlay');
            const winnerNameEl = document.getElementById('gameOverWinnerName');
            const scoreEl = document.getElementById('gameOverScore');
            const standingsEl = document.getElementById('gameOverStandings');
            const countdownEl = document.getElementById('gameOverCountdown');

            // Set winner info
            if (gameOverData.winner_name) {{
                winnerNameEl.textContent = gameOverData.winner_name;
            }} else {{
                winnerNameEl.textContent = 'No Winner';
            }}

            // Set winning score
            scoreEl.textContent = '$' + (gameOverData.winner_score || 0).toLocaleString();

            // Build final standings
            let standingsHtml = '<div class="game-over-standings-title">Final Standings</div>';
            const sortedPlayers = [...players].sort((a, b) => b.total - a.total);

            sortedPlayers.forEach((player, index) => {{
                const rank = index + 1;
                const rankClass = rank === 1 ? 'gold' : rank === 2 ? 'silver' : rank === 3 ? 'bronze' : '';
                const winnerClass = index === 0 ? 'winner' : '';
                standingsHtml += `
                    <div class="game-over-standing ${{winnerClass}}">
                        <div class="game-over-standing-rank ${{rankClass}}">${{rank}}</div>
                        <div class="game-over-standing-name">${{player.name}}</div>
                        <div class="game-over-standing-score">$${{player.total.toLocaleString()}}</div>
                    </div>
                `;
            }});
            standingsEl.innerHTML = standingsHtml;

            // Show overlay
            overlay.classList.add('active');
            initGameOverStars();

            // Play celebration sound and confetti
            SoundService.bonusWin();
            launchConfetti(300);

            // Start countdown timer
            let remaining = gameOverData.remaining_seconds || 15;
            countdownEl.textContent = remaining;

            if (gameOverCountdownInterval) {{
                clearInterval(gameOverCountdownInterval);
            }}

            gameOverCountdownInterval = setInterval(() => {{
                remaining--;
                if (remaining >= 0) {{
                    countdownEl.textContent = remaining;
                }}
                if (remaining <= 0) {{
                    clearInterval(gameOverCountdownInterval);
                    gameOverCountdownInterval = null;
                }}
            }}, 1000);
        }}

        function closeGameOver() {{
            document.getElementById('gameOverOverlay').classList.remove('active');
            if (gameOverCountdownInterval) {{
                clearInterval(gameOverCountdownInterval);
                gameOverCountdownInterval = null;
            }}
        }}

        function startNewGameNow() {{
            closeGameOver();
            socket.emit('new_game', {{ room }});
        }}

        function initGameOverStars() {{
            const container = document.getElementById('gameOverStars');
            container.innerHTML = '';
            for (let i = 0; i < 50; i++) {{
                const star = document.createElement('div');
                star.className = 'winner-star';
                star.textContent = ['⭐', '✨', '🌟'][Math.floor(Math.random() * 3)];
                star.style.left = Math.random() * 100 + '%';
                star.style.top = Math.random() * 100 + '%';
                star.style.animationDelay = Math.random() * 3 + 's';
                star.style.animationDuration = (2 + Math.random() * 2) + 's';
                container.appendChild(star);
            }}
        }}

        function openBonusRound(playerName, avatarId) {{
            bonusState.active = true;
            bonusState.stage = 'pick';  // Skip prize wheel, go directly to pick
            bonusState.prizeAmount = gameState?.config?.bonus_jackpot || gameState?.bonus?.jackpot || 10000;
            bonusState.pickedConsonants = [];
            bonusState.pickedVowel = null;
            bonusState.totalSeconds = gameState?.config?.bonus_seconds || gameState?.config?.final_seconds || 10;
            bonusState.remainingSeconds = bonusState.totalSeconds;

            const avatar = getAvatarEmoji(avatarId);
            document.getElementById('bonusPlayerName').innerHTML = `<span class="bonus-player-avatar">${{avatar}}</span> ${{playerName}}`;
            document.getElementById('bonusRoundOverlay').classList.add('active');
            initBonusStars();

            // Update jackpot display in result area
            document.getElementById('bonusResultAmount').textContent = '$' + bonusState.prizeAmount.toLocaleString();

            // Show puzzle board during pick phase so player can see what they're solving
            const bonusPuzzleSection = document.getElementById('bonusPuzzleSection');
            bonusPuzzleSection.style.display = 'block';
            document.getElementById('bonusCategory').textContent = 'Category: ' + (gameState?.puzzle?.category || '-');

            // Render puzzle with RSTLNE revealed
            const answer = gameState?.puzzle?.answer || '';
            const givenLetters = new Set(['R', 'S', 'T', 'L', 'N', 'E']);
            renderBonusPuzzleBoard(answer, givenLetters);

            showBonusStage('pick');  // Go directly to letter picking
        }}

        // Render bonus puzzle board
        function renderBonusPuzzleBoard(answer, revealed) {{
            const board = document.getElementById('bonusPuzzleBoard');
            const ROW_SIZES = [12, 14, 14, 12];
            const words = answer.toUpperCase().split(' ');

            function layoutWords(startRow) {{
                const rows = [[], [], [], []];
                let currentRow = startRow;
                for (const word of words) {{
                    if (currentRow >= 4) return null;
                    const currentLen = rows[currentRow].reduce((sum, w) => sum + w.length + 1, 0) - 1;
                    const spaceNeeded = currentLen > 0 ? word.length + 1 : word.length;
                    if (currentLen + spaceNeeded <= ROW_SIZES[currentRow]) {{
                        rows[currentRow].push(word);
                    }} else {{
                        currentRow++;
                        if (currentRow >= 4) return null;
                        rows[currentRow].push(word);
                    }}
                }}
                return rows;
            }}

            let rows = layoutWords(1);
            if (!rows) rows = layoutWords(0);
            if (!rows) rows = [[], [], [], []];

            let html = '';
            for (let r = 0; r < 4; r++) {{
                const rowSize = ROW_SIZES[r];
                const rowWords = rows[r];
                const rowText = rowWords.join(' ');
                const padding = Math.floor((rowSize - rowText.length) / 2);

                html += '<div class="puzzle-row">';
                for (let i = 0; i < rowSize; i++) {{
                    const charIdx = i - padding;
                    if (charIdx >= 0 && charIdx < rowText.length) {{
                        const char = rowText[charIdx];
                        if (char === ' ') {{
                            html += '<div class="letter-tile blank"></div>';
                        }} else if (revealed.has(char)) {{
                            html += `<div class="letter-tile revealed">${{char}}</div>`;
                        }} else {{
                            html += '<div class="letter-tile hidden"></div>';
                        }}
                    }} else {{
                        html += '<div class="letter-tile blank"></div>';
                    }}
                }}
                html += '</div>';
            }}
            board.innerHTML = html;
        }}

        // ========== CONFETTI & CELEBRATION SYSTEM ==========
        function launchConfetti(count = 100, options = {{}}) {{
            const container = document.getElementById('confetti-container');
            const isGold = options.gold || false;
            const colors = isGold
                ? ['#ffd700', '#ffec8b', '#d4af37', '#f0e68c', '#fff8dc', '#daa520']
                : ['#d4af37', '#ffd700', '#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4', '#ff8c00', '#9b59b6', '#3498db'];
            const shapes = ['square', 'circle', 'triangle'];

            for (let i = 0; i < count; i++) {{
                setTimeout(() => {{
                    const confetti = document.createElement('div');
                    confetti.className = 'confetti' + (isGold ? ' gold' : '');
                    confetti.style.left = Math.random() * 100 + 'vw';
                    confetti.style.background = colors[Math.floor(Math.random() * colors.length)];
                    confetti.style.animationDuration = (2 + Math.random() * 2) + 's';
                    confetti.style.animationDelay = Math.random() * 0.5 + 's';
                    // Random shapes
                    const shape = shapes[Math.floor(Math.random() * shapes.length)];
                    if (shape === 'circle') confetti.style.borderRadius = '50%';
                    if (shape === 'triangle') {{
                        confetti.style.width = '0';
                        confetti.style.height = '0';
                        confetti.style.borderLeft = '6px solid transparent';
                        confetti.style.borderRight = '6px solid transparent';
                        confetti.style.borderBottom = '12px solid ' + colors[Math.floor(Math.random() * colors.length)];
                        confetti.style.background = 'transparent';
                    }}
                    // Random size variation
                    const size = 8 + Math.random() * 8;
                    if (shape !== 'triangle') {{
                        confetti.style.width = size + 'px';
                        confetti.style.height = size + 'px';
                    }}
                    container.appendChild(confetti);
                    setTimeout(() => confetti.remove(), 4500);
                }}, i * 15);
            }}
        }}

        // Launch sparkles for winner celebrations
        function launchSparkles(count = 30) {{
            const container = document.getElementById('confetti-container');
            for (let i = 0; i < count; i++) {{
                setTimeout(() => {{
                    const sparkle = document.createElement('div');
                    sparkle.className = 'sparkle';
                    sparkle.style.left = (20 + Math.random() * 60) + 'vw';
                    sparkle.style.top = (30 + Math.random() * 40) + 'vh';
                    sparkle.style.animationDuration = (1.5 + Math.random()) + 's';
                    container.appendChild(sparkle);
                    setTimeout(() => sparkle.remove(), 2500);
                }}, i * 100);
            }}
        }}

        // Launch firework bursts
        function launchFireworks(count = 5) {{
            const container = document.getElementById('confetti-container');
            const colors = ['#ffd700', '#ff6b6b', '#4ecdc4', '#9b59b6', '#3498db', '#22c55e'];

            for (let i = 0; i < count; i++) {{
                setTimeout(() => {{
                    const x = 15 + Math.random() * 70;
                    const y = 20 + Math.random() * 40;
                    const color = colors[Math.floor(Math.random() * colors.length)];
                    // Create burst of particles
                    for (let j = 0; j < 20; j++) {{
                        const particle = document.createElement('div');
                        particle.className = 'firework';
                        particle.style.background = color;
                        particle.style.boxShadow = `0 0 6px ${{color}}, 0 0 12px ${{color}}`;
                        particle.style.left = x + 'vw';
                        particle.style.top = y + 'vh';
                        const angle = (j / 20) * Math.PI * 2;
                        const distance = 50 + Math.random() * 50;
                        particle.style.setProperty('--dx', Math.cos(angle) * distance + 'px');
                        particle.style.setProperty('--dy', Math.sin(angle) * distance + 'px');
                        particle.style.animation = `fireworkParticle 1.2s ease-out forwards`;
                        container.appendChild(particle);
                        setTimeout(() => particle.remove(), 1500);
                    }}
                }}, i * 400);
            }}
        }}

        // Add dynamic keyframes for firework particles
        if (!document.getElementById('dynamicFireworkStyles')) {{
            const style = document.createElement('style');
            style.id = 'dynamicFireworkStyles';
            style.textContent = `
                @keyframes fireworkParticle {{
                    0% {{ transform: translate(0, 0) scale(1); opacity: 1; }}
                    100% {{ transform: translate(var(--dx), var(--dy)) scale(0); opacity: 0; }}
                }}
            `;
            document.head.appendChild(style);
        }}

        // Generate phase sparkles
        function initPhaseSparkles() {{
            const container = document.getElementById('phaseSparkles');
            if (!container) return;
            container.innerHTML = '';
            for (let i = 0; i < 20; i++) {{
                const sparkle = document.createElement('div');
                sparkle.className = 'phase-sparkle';
                sparkle.style.left = Math.random() * 100 + '%';
                sparkle.style.top = Math.random() * 100 + '%';
                sparkle.style.animationDelay = Math.random() * 3 + 's';
                sparkle.style.animationDuration = (2 + Math.random() * 2) + 's';
                container.appendChild(sparkle);
            }}
        }}

        // Winner celebration - full effect
        function celebrateWinner(playerName, amount) {{
            launchConfetti(200, {{ gold: true }});
            setTimeout(() => launchSparkles(40), 300);
            setTimeout(() => launchFireworks(6), 500);
            setTimeout(() => launchConfetti(100, {{ gold: true }}), 1500);
            SoundService.bonusWin();
        }}

        // ========== PHASE TRANSITIONS ==========
        // Display duration for transition overlays (5 seconds)
        const TRANSITION_DISPLAY_MS = 5000;
        let transitionCountdownInterval = null;

        function startTransitionCountdown(durationMs) {{
            const countdownEl = document.getElementById('phaseCountdown');
            if (!countdownEl) return;

            // Clear any existing countdown
            if (transitionCountdownInterval) {{
                clearInterval(transitionCountdownInterval);
            }}

            let remaining = Math.ceil(durationMs / 1000);
            countdownEl.textContent = `Starting in ${{remaining}}...`;

            transitionCountdownInterval = setInterval(() => {{
                remaining--;
                if (remaining > 0) {{
                    countdownEl.textContent = `Starting in ${{remaining}}...`;
                }} else {{
                    countdownEl.textContent = '';
                    clearInterval(transitionCountdownInterval);
                    transitionCountdownInterval = null;
                }}
            }}, 1000);
        }}

        function showPhaseTransition(phase, round) {{
            const overlay = document.getElementById('phaseOverlay');
            const title = document.getElementById('phaseTitle');
            const subtitle = document.getElementById('phaseSubtitle');
            const prizeEl = document.getElementById('phasePrize');

            // Initialize sparkles for visual effect
            initPhaseSparkles();

            const phases = {{
                'tossup': {{ title: 'TOSS-UP!', subtitle: 'Buzz in to answer!', icon: null }},
                'final': {{ title: 'FINAL SPIN!', subtitle: 'Call letters - vowels are FREE!', icon: null }},
                'bonus': {{ title: 'BONUS ROUND!', subtitle: 'Pick your letters wisely...', icon: null }},
                'normal': {{ title: 'SPIN THE WHEEL!', subtitle: 'Good luck!', icon: null }},
            }};

            const config = phases[phase] || {{ title: phase.toUpperCase(), subtitle: '', icon: null }};
            title.textContent = config.title;
            subtitle.textContent = config.subtitle;
            if (prizeEl) prizeEl.style.display = 'none';

            // Remove any special modes
            overlay.classList.remove('winner-mode');

            // Play appropriate sound
            if (phase === 'bonus') {{
                SoundService.bonusStart();
            }} else if (phase === 'final') {{
                SoundService.finalSpin();
            }}

            overlay.classList.add('active');
            startTransitionCountdown(TRANSITION_DISPLAY_MS);

            // Add some sparkles for excitement
            if (phase === 'bonus' || phase === 'final') {{
                setTimeout(() => launchSparkles(15), 500);
            }}

            setTimeout(() => {{
                overlay.classList.remove('active');
            }}, TRANSITION_DISPLAY_MS);
        }}

        function showRoundTransition(round, fromTossup) {{
            const overlay = document.getElementById('phaseOverlay');
            const title = document.getElementById('phaseTitle');
            const subtitle = document.getElementById('phaseSubtitle');
            const prizeEl = document.getElementById('phasePrize');

            // Initialize sparkles
            initPhaseSparkles();

            // Round-specific titles and subtitles with more flair
            const roundConfigs = {{
                1: {{ title: 'ROUND 1', subtitle: fromTossup ? 'Toss-up winner plays first!' : 'Let the game begin!' }},
                2: {{ title: 'ROUND 2', subtitle: 'The stakes are rising!' }},
                3: {{ title: 'ROUND 3', subtitle: 'Keep that momentum!' }},
                4: {{ title: 'FINAL ROUND', subtitle: 'This is it - give it your all!' }},
            }};

            const config = roundConfigs[round] || {{ title: 'ROUND ' + round, subtitle: '' }};
            title.textContent = config.title;
            subtitle.textContent = config.subtitle;
            if (prizeEl) prizeEl.style.display = 'none';

            // Remove special modes
            overlay.classList.remove('winner-mode');

            overlay.classList.add('active');
            startTransitionCountdown(TRANSITION_DISPLAY_MS);

            // Light confetti for round transitions
            setTimeout(() => launchConfetti(30), 300);

            setTimeout(() => {{
                overlay.classList.remove('active');
            }}, TRANSITION_DISPLAY_MS);
        }}

        // Winner transition - extra special celebration
        function showWinnerTransition(playerName, totalWinnings) {{
            const overlay = document.getElementById('phaseOverlay');
            const title = document.getElementById('phaseTitle');
            const subtitle = document.getElementById('phaseSubtitle');
            const prizeEl = document.getElementById('phasePrize');

            initPhaseSparkles();

            title.textContent = 'CONGRATULATIONS!';
            subtitle.textContent = playerName + ' WINS!';

            // Show prize amount with counting animation
            if (prizeEl) {{
                prizeEl.style.display = 'block';
                animateCountUp(prizeEl, 0, totalWinnings, 2000);
            }}

            overlay.classList.add('active', 'winner-mode');

            // Full celebration
            celebrateWinner(playerName, totalWinnings);

            // Extended display for winner
            const winnerDisplayMs = 8000;
            setTimeout(() => {{
                overlay.classList.remove('active', 'winner-mode');
            }}, winnerDisplayMs);
        }}

        // Animate counting up for prize amounts
        function animateCountUp(element, start, end, duration) {{
            const startTime = performance.now();
            const diff = end - start;

            function update(currentTime) {{
                const elapsed = currentTime - startTime;
                const progress = Math.min(elapsed / duration, 1);
                // Ease out cubic
                const easeProgress = 1 - Math.pow(1 - progress, 3);
                const current = Math.round(start + diff * easeProgress);
                element.textContent = '$' + current.toLocaleString();

                if (progress < 1) {{
                    requestAnimationFrame(update);
                }}
            }}
            requestAnimationFrame(update);
        }}

        // ========== AVATAR HELPER ==========
        const AVATAR_EMOJIS = ['', '\ud83c\udf85', '\ud83e\uddd1\u200d\ud83c\udf84', '\ud83e\udd8c', '\u26c4', '\ud83c\udf84', '\ud83c\udf81', '\ud83d\udd14', '\u2744\ufe0f', '\u2b50', '\ud83d\udd6f\ufe0f', '\ud83e\udddd', '\ud83e\udd34'];
        // Avatar IDs map to: 1=Santa, 2=Mrs Claus, 3=Reindeer, 4=Snowman, 5=Tree, 6=Gift, 7=Bell, 8=Snowflake, 9=Star, 10=Candle, 11=Elf, 12=King
        function getAvatarEmoji(avatarId) {{
            const id = parseInt(avatarId) || 1;
            // Clamp to valid range 1-12
            const validId = Math.min(12, Math.max(1, id));
            return AVATAR_EMOJIS[validId] || AVATAR_EMOJIS[1];
        }}

        // ========== SCORE CHANGE ANIMATION ==========
        function formatCash(amount) {{
            return '$' + Math.abs(amount).toLocaleString();
        }}

        function showScoreChange(playerIdx, amount) {{
            const playerEl = document.querySelectorAll('.player')[playerIdx];
            if (!playerEl) return;

            const change = document.createElement('span');
            change.className = 'score-change ' + (amount >= 0 ? 'positive' : 'negative');
            change.textContent = (amount >= 0 ? '+' : '-') + formatCash(amount);

            const scoreEl = playerEl.querySelector('.player-score-total');
            if (scoreEl) {{
                scoreEl.style.position = 'relative';
                scoreEl.appendChild(change);
                setTimeout(() => change.remove(), 1500);
            }}
        }}

        // ========== LETTER VALUE POPUP ==========
        function showLetterValuePopup(cashValue, letterCount, isVowel) {{
            const board = document.getElementById('puzzleBoard');
            if (!board) return;

            const rect = board.getBoundingClientRect();
            const popup = document.createElement('div');
            popup.className = 'letter-value-popup' + (isVowel ? ' vowel' : '');

            if (isVowel) {{
                popup.textContent = `-$250`;
            }} else {{
                const total = cashValue * letterCount;
                if (letterCount > 1) {{
                    popup.textContent = `${{letterCount}} x $${{cashValue.toLocaleString()}} = $${{total.toLocaleString()}}`;
                }} else {{
                    popup.textContent = `+$${{total.toLocaleString()}}`;
                }}
            }}

            // Position at center-top of puzzle board
            popup.style.left = (rect.left + rect.width / 2) + 'px';
            popup.style.top = (rect.top + 40) + 'px';
            popup.style.transform = 'translateX(-50%)';

            document.body.appendChild(popup);
            setTimeout(() => popup.remove(), 1500);
        }}

        function getWedgeLabel(slot) {{
            // Handle null/undefined
            if (slot === null || slot === undefined) return '?';

            // Direct number (Cash value)
            if (typeof slot === 'number') return '$' + slot;

            // String values (BANKRUPT, LOSE A TURN, FREE PLAY)
            if (typeof slot === 'string') return slot;

            // Object with Cash property {{ Cash: 500 }}
            if (slot.Cash !== undefined) return '$' + slot.Cash;

            // Prize object {{ type: "PRIZE", name: "..." }} or {{ Prize: {{ name: "..." }} }}
            if (slot.Prize) return slot.Prize.name || slot.Prize || 'PRIZE';
            if (slot.type === 'PRIZE' || slot.wedge_type === 'PRIZE') return slot.name || 'PRIZE';

            // Unit variants that serialize as objects {{ Bankrupt: null }}
            if ('Bankrupt' in slot) return 'BANKRUPT';
            if ('LoseTurn' in slot) return 'LOSE A TURN';
            if ('FreePlay' in slot) return 'FREE PLAY';

            // Fallback: get first key
            const keys = Object.keys(slot);
            if (keys.length > 0) {{
                const key = keys[0];
                const val = slot[key];
                if (typeof val === 'number') return '$' + val;
                if (typeof val === 'string') return val;
                if (val && val.name) return val.name;
                return key.replace(/([A-Z])/g, ' $1').trim();
            }}

            console.log('Unknown wedge format:', slot);
            return '???';
        }}

        function renderWheel(slots) {{
            const svg = document.getElementById('wheelSvg');
            if (!slots || slots.length === 0) {{
                svg.innerHTML = "<text x='170' y='170' text-anchor='middle' fill='#888'>No wheel data</text>";
                return;
            }}

            const size = 340;
            const radius = size / 2 - 10;
            const centerX = size / 2;
            const centerY = size / 2;
            const numSlots = slots.length;

            let html = '';

            // Draw wedges
            slots.forEach((slot, idx) => {{
                const anglePerSlot = 360 / numSlots;
                const startAngle = idx * anglePerSlot - 90;
                const endAngle = startAngle + anglePerSlot;
                const startRad = startAngle * Math.PI / 180;
                const endRad = endAngle * Math.PI / 180;

                const x1 = centerX + radius * Math.cos(startRad);
                const y1 = centerY + radius * Math.sin(startRad);
                const x2 = centerX + radius * Math.cos(endRad);
                const y2 = centerY + radius * Math.sin(endRad);

                const largeArc = anglePerSlot > 180 ? 1 : 0;
                const pathD = "M " + centerX + " " + centerY + " L " + x1 + " " + y1 + " A " + radius + " " + radius + " 0 " + largeArc + " 1 " + x2 + " " + y2 + " Z";

                const label = getWedgeLabel(slot);
                const isBankrupt = label.includes('BANKRUPT');
                const isLoseTurn = label.includes('LOSE');
                const isFreePlay = label.includes('FREE');
                const color = isBankrupt ? '#1a1a1a' : isLoseTurn ? '#f5f5f5' : isFreePlay ? '#228b22' : WHEEL_COLORS[idx % WHEEL_COLORS.length];

                html += "<path d='" + pathD + "' fill='" + color + "' stroke='#111' stroke-width='2'/>";

                // Text label - positioned toward outer edge
                const midAngle = (startAngle + endAngle) / 2;
                const midRad = midAngle * Math.PI / 180;
                const textRadius = radius * 0.68;
                const textX = centerX + textRadius * Math.cos(midRad);
                const textY = centerY + textRadius * Math.sin(midRad);

                const normalizedAngle = ((midAngle % 360) + 360) % 360;
                let rotation = midAngle;
                if (normalizedAngle > 90 && normalizedAngle < 270) rotation = midAngle + 180;

                // Dynamic font size based on label length and number of slots (slightly larger for bigger wheel)
                const baseSize = numSlots > 20 ? 15 : numSlots > 16 ? 17 : 19;
                let fontSize = baseSize;
                if (label.length > 10) fontSize = baseSize - 3;
                else if (label.length > 7) fontSize = baseSize - 2;
                else if (label.length > 5) fontSize = baseSize - 1;

                // Crisp white text with thin black outline and drop shadow
                html += "<text x='" + textX + "' y='" + textY + "' fill='#fff' stroke='#000' stroke-width='1' font-size='" + fontSize + "' font-weight='bold' font-family='Arial, sans-serif' text-anchor='middle' dominant-baseline='middle' transform='rotate(" + rotation + ", " + textX + ", " + textY + ")' style='paint-order: stroke fill; filter: drop-shadow(1px 1px 1px rgba(0,0,0,0.8))'>" + label + "</text>";
            }});

            // Center hub is rendered by HTML element (wheel-center-hub), not SVG

            svg.innerHTML = html;
        }}

        function animateWheelTo(targetIdx, slots) {{
            if (wheelAnimationId) {{
                cancelAnimationFrame(wheelAnimationId);
            }}

            isWheelSpinning = true;
            const wheelContainer = document.getElementById('wheelContainer');
            const wheelSvg = document.getElementById('wheelSvg');
            const wheelResult = document.getElementById('wheelResult');
            const tickFlash = document.getElementById('wheelTickFlash');

            wheelContainer.classList.add('spinning');
            wheelResult.textContent = 'Spinning...';
            wheelResult.classList.remove('winner', 'big-win', 'bankrupt', 'free-play');

            const numSlots = slots.length;
            const anglePerSlot = 360 / numSlots;

            // Calculate the angle where the target wedge center should be at top (pointer position)
            const wedgeCenterAngle = targetIdx * anglePerSlot + anglePerSlot / 2;
            const finalAngle = (360 - wedgeCenterAngle) % 360;

            const currentAngle = ((wheelRotation % 360) + 360) % 360;
            let delta = (finalAngle - currentAngle + 360) % 360;
            if (delta < 30) delta += 360;

            const spins = 4;  // More spins for dramatic effect
            const targetRotation = wheelRotation + spins * 360 + delta;

            const startRotation = wheelRotation;
            const totalDelta = targetRotation - startRotation;
            const duration = 5000;  // Longer duration for more suspense
            const startTime = performance.now();

            // Track last wedge for tick effect
            let lastWedgeIdx = -1;
            let tickCooldown = 0;

            // Smooth ease-out function - starts fast, gradually slows to stop
            function customEase(t) {{
                // Ease out quint - smooth deceleration, no overshoot
                return 1 - Math.pow(1 - t, 5);
            }}

            // Function to calculate current wedge index from rotation
            function getCurrentWedgeIdx(rotation) {{
                const normalizedRotation = ((rotation % 360) + 360) % 360;
                // The pointer is at top (0 degrees in CSS terms)
                // Wedges are drawn starting at -90 degrees
                // So we need to figure out which wedge is at the pointer
                const pointerAngle = (360 - normalizedRotation + 90) % 360;
                return Math.floor(pointerAngle / anglePerSlot) % numSlots;
            }}

            // Trigger tick flash effect
            function triggerTick() {{
                tickFlash.classList.remove('flash');
                void tickFlash.offsetWidth; // Force reflow
                tickFlash.classList.add('flash');
                SoundService.wheelTick();
            }}

            function animate(currentTime) {{
                const elapsed = currentTime - startTime;
                const progress = Math.min(elapsed / duration, 1);

                // Apply custom easing
                const eased = customEase(progress);
                wheelRotation = startRotation + totalDelta * eased;

                wheelSvg.style.transform = `rotate(${{wheelRotation}}deg)`;

                // Tick effect when passing wedge boundaries
                const currentWedge = getCurrentWedgeIdx(wheelRotation);
                tickCooldown--;
                if (currentWedge !== lastWedgeIdx && tickCooldown <= 0) {{
                    lastWedgeIdx = currentWedge;
                    // Only tick if spinning fast enough
                    if (progress < 0.85) {{
                        triggerTick();
                        tickCooldown = 2; // Prevent too rapid ticking
                    }}
                }}

                if (progress < 1) {{
                    wheelAnimationId = requestAnimationFrame(animate);
                }} else {{
                    // Animation complete
                    wheelAnimationId = null;
                    isWheelSpinning = false;

                    // Remove spinning state
                    wheelContainer.classList.remove('spinning');
                    wheelSvg.style.filter = 'drop-shadow(0 4px 8px rgba(0,0,0,0.4))';

                    // Highlight winning wedge
                    highlightWinningWedge(targetIdx);

                    onWheelStopped();
                }}
            }}

            wheelAnimationId = requestAnimationFrame(animate);
        }}

        function highlightWinningWedge(wedgeIdx) {{
            const wheelSvg = document.getElementById('wheelSvg');
            const wheelResult = document.getElementById('wheelResult');

            // Find the winning wedge path and add highlight class
            const wedges = wheelSvg.querySelectorAll('path');
            if (wedges[wedgeIdx]) {{
                wedges[wedgeIdx].classList.add('winning-wedge');
                // Remove class after animation
                setTimeout(() => {{
                    wedges[wedgeIdx].classList.remove('winning-wedge');
                }}, 1500);
            }}

            // Add winner animation to result text
            wheelResult.classList.add('winner');
        }}

        function onWheelStopped() {{
            // Play wheel stop sound
            SoundService.wheelStop();

            // Show the wheel result with appropriate styling
            if (pendingWheelResult !== null) {{
                const wheelResult = document.getElementById('wheelResult');
                wheelResult.textContent = pendingWheelResult;

                // Apply special styling based on result
                wheelResult.classList.remove('big-win', 'bankrupt', 'free-play');

                if (pendingWheelResult === 'BANKRUPT') {{
                    SoundService.bankrupt();
                    wheelResult.classList.add('bankrupt');
                }} else if (pendingWheelResult === 'LOSE A TURN') {{
                    SoundService.loseTurn();
                    wheelResult.classList.add('bankrupt');
                }} else if (pendingWheelResult === 'FREE PLAY') {{
                    wheelResult.classList.add('free-play');
                    launchSparkles(10);
                }} else if (pendingWheelResult.includes('$')) {{
                    // Check for big wins ($1000+)
                    const amount = parseInt(pendingWheelResult.replace(/[^0-9]/g, ''));
                    if (amount >= 1000) {{
                        wheelResult.classList.add('big-win');
                        launchSparkles(15);
                    }}
                }}

                pendingWheelResult = null;
            }}
            // Show any pending toasts
            while (pendingToasts.length > 0) {{
                const msg = pendingToasts.shift();
                showNotification(msg);
            }}
            // Focus the letter input so user is ready to guess
            document.getElementById('letterInput').focus();

            // Notify server that spin animation is complete to start turn timer
            if (socket && room) {{
                socket.emit('spin_complete', {{ room }});
            }}
        }}

        function connect() {{
            socket = io(window.location.origin, {{ transports: ['websocket'] }});

            // Helper to join the game (called on connect and on page load if already connected)
            function joinGame() {{
                console.log('Joining game with socket:', socket.id);
                // Authenticate the socket for session management
                socket.emit('auth', {{ token }});
                if (isSpectating) {{
                    // Spectator mode - just watch, don't join as player
                    socket.emit('join', {{ room }});
                    myPlayerIdx = null;
                }} else {{
                    socket.emit('join_game', {{ room, name: user.display_name || user.email }});
                }}
            }}

            socket.on('connect', () => {{
                console.log('Connected:', socket.id);
                joinGame();
            }});

            // If socket is already connected (e.g., page refresh), join immediately
            if (socket.connected) {{
                joinGame();
            }}

            // Handle session invalidation (logged in from another device)
            socket.on('session_invalidated', (data) => {{
                console.log('Session invalidated:', data);
                // Clear stored credentials
                localStorage.removeItem('auth_token');
                document.cookie = 'auth_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT';
                // Show message and redirect to login
                alert('You have been logged out because your account was accessed from another device.');
                window.location.href = '/';
            }});

            socket.on('you', (data) => {{
                myPlayerIdx = data.player_idx;
                console.log('I am player', myPlayerIdx);
            }});

            socket.on('state', (state) => {{
                console.log('Received state:', state);
                gameState = state;
                renderGame();
            }});

            socket.on('host_granted', (data) => {{
                console.log('Host granted:', data);
                isHost = data.granted === true;
                updateHostUI();
                if (isHost) {{
                    loadPacks();
                }}
            }});

            socket.on('toast', (data) => {{
                console.log('Toast:', data);
                const msg = data.msg || data;
                if (isWheelSpinning) {{
                    pendingToasts.push(msg);
                }} else {{
                    showNotification(msg);
                }}
            }});

            socket.on('notification', (msg) => {{
                if (isWheelSpinning) {{
                    pendingToasts.push(msg);
                }} else {{
                    showNotification(msg);
                }}
            }});

            socket.on('error', (err) => {{
                console.error('Socket error:', err);
                showNotification('Error: ' + (err.message || err));
            }});

            socket.on('disconnect', () => {{
                console.log('Disconnected');
                document.getElementById('phase').textContent = 'Disconnected';
            }});
        }}

        function renderGame() {{
            if (!gameState) return;

            const phase = gameState.phase || 'normal';

            // ========== PHASE & ROUND TRANSITIONS ==========
            const currentRound = gameState.round || 1;
            const fromTossup = prevPhase === 'tossup' && phase === 'normal';
            const fromFinalToBonus = prevPhase === 'final' && phase === 'bonus';

            // Detect phase transitions (tossup, final, bonus)
            if (prevPhase && prevPhase !== phase) {{
                // Special case: tossup -> normal triggers Round 1 transition
                if (fromTossup) {{
                    showRoundTransition(1, true);
                }}
                // Special case: final -> bonus triggers Bonus Round transition
                else if (fromFinalToBonus) {{
                    showPhaseTransition('bonus', currentRound);
                }}
                // Other phase transitions (e.g., normal -> tossup, normal -> final, normal -> bonus)
                else {{
                    showPhaseTransition(phase, currentRound);
                }}
            }}
            // Detect round transitions (when phase stays normal but round changes)
            else if (phase === 'normal' && prevRound !== null && prevRound !== currentRound && currentRound > prevRound) {{
                showRoundTransition(currentRound, false);
            }}

            prevPhase = phase;
            prevRound = currentRound;

            // Check for puzzle solved
            const solvedBy = gameState.puzzle_solved_by;
            if (solvedBy && solvedBy !== prevPuzzleSolvedBy) {{
                prevPuzzleSolvedBy = solvedBy;
                showNotification(`🎉 ${{solvedBy}} solved it! Answer: ${{gameState.puzzle?.answer}}`);
                SoundService.solve();
                launchConfetti(150);
            }} else if (!solvedBy && prevPuzzleSolvedBy) {{
                prevPuzzleSolvedBy = null;
                hideNotification();
            }}

            // Phase display
            const isPregame = phase === 'pregame';
            const isGameOver = phase === 'gameover';
            let phaseText = phase;
            if (isPregame) {{
                phaseText = 'Waiting to start...';
            }} else if (phase === 'final') {{
                phaseText = 'Final Spin';
            }} else if (phase === 'tossup') {{
                phaseText = 'Toss-up';
            }} else if (phase === 'bonus') {{
                phaseText = 'Bonus Round';
            }} else if (phase === 'normal') {{
                phaseText = 'Normal';
            }} else if (isGameOver) {{
                phaseText = 'Game Over';
            }}
            document.getElementById('phase').textContent = phaseText;

            // Theme (pack name) and Category
            const packName = gameState.active_pack_name;
            const themeEl = document.getElementById('theme');
            if (packName && packName !== 'All Packs') {{
                themeEl.textContent = packName;
                themeEl.style.display = 'block';
            }} else {{
                themeEl.style.display = 'none';
            }}
            document.getElementById('category').textContent = gameState.puzzle?.category || '-';

            // ========== ROUND PROGRESS ==========
            // currentRound already declared above for transition detection
            const totalRounds = 4;
            const roundIndicator = document.getElementById('roundIndicator');

            // Update sidebar round display
            document.getElementById('roundDisplay').textContent = currentRound;

            // Update host controls round select
            const roundSelect = document.getElementById('roundSelect');
            if (roundSelect && roundSelect.value !== String(currentRound)) {{
                roundSelect.value = String(currentRound);
            }}

            // Update round indicator UI
            if (roundIndicator) {{
                roundIndicator.classList.add('active');
                document.getElementById('roundNumber').textContent = 'ROUND ' + currentRound;
                document.getElementById('roundTotal').textContent = 'of ' + totalRounds;

                // Render round dots
                let dotsHtml = '';
                for (let i = 1; i <= totalRounds; i++) {{
                    const dotClass = i < currentRound ? 'completed' :
                                    i === currentRound ? 'current' : '';
                    dotsHtml += `<div class="round-dot ${{dotClass}}"></div>`;
                }}
                document.getElementById('roundDots').innerHTML = dotsHtml;

                // Round badges based on current phase
                let badgesHtml = '';
                const phaseLower = phase.toLowerCase();
                if (isPregame) {{
                    badgesHtml += `<span class="round-badge type">PREGAME</span>`;
                }} else if (isGameOver) {{
                    badgesHtml += `<span class="round-badge type" style="background: linear-gradient(180deg, #ffd700 0%, #d4af37 100%); color: #1a1a2e;">GAME OVER</span>`;
                }} else if (phaseLower === 'tossup') {{
                    badgesHtml += `<span class="round-badge type">TOSS-UP</span>`;
                }} else if (phaseLower === 'final') {{
                    badgesHtml += `<span class="round-badge type">FINAL SPIN</span>`;
                }} else if (phaseLower === 'bonus') {{
                    badgesHtml += `<span class="round-badge type">BONUS</span>`;
                }}
                if (currentRound === 4 && phaseLower === 'normal') {{
                    badgesHtml += `<span class="round-badge multiplier">SPEED</span>`;
                    badgesHtml += `<span class="round-badge warning" style="background: #f59e0b; color: #1a1a2e;">Final Spin may trigger!</span>`;
                }}
                document.getElementById('roundBadges').innerHTML = badgesHtml;
            }}

            // ========== TOSS-UP DISPLAY ==========
            const tossupDisplay = document.getElementById('tossupDisplay');
            const tossupConfig = gameState.tossup_config;
            if (phase === 'tossup' && tossupConfig) {{
                tossupDisplay.classList.add('active');
                const value = tossupConfig.is_triple && tossupConfig.values ?
                    tossupConfig.values[tossupConfig.triple_index] || 1000 : 1000;
                document.getElementById('tossupValue').textContent = '$' + value.toLocaleString();

                // Triple toss-up indicators
                if (tossupConfig.is_triple) {{
                    document.getElementById('tripleHeader').style.display = 'block';
                    for (let i = 0; i < 3; i++) {{
                        const dot = document.getElementById('triple' + i);
                        dot.className = 'triple-dot' +
                            (i < tossupConfig.triple_index ? ' completed' :
                             i === tossupConfig.triple_index ? ' current' : '');
                    }}
                }} else {{
                    document.getElementById('tripleHeader').style.display = 'none';
                }}
            }} else {{
                tossupDisplay.classList.remove('active');
            }}

            // ========== EXPRESS MODE ==========
            const express = gameState.express;
            const expressIndicator = document.getElementById('expressIndicator');
            if (express && express.active && express.player_idx === myPlayerIdx) {{
                expressIndicator.classList.add('active');
                document.getElementById('expressStreak').textContent =
                    '$' + (express.correct_count * (express.value_per_consonant || 1000)).toLocaleString();
            }} else {{
                expressIndicator.classList.remove('active');
            }}

            // ========== MYSTERY WEDGE ==========
            const mystery = gameState.mystery;
            if (mystery && mystery.stage === 'awaiting_choice' && mystery.player_idx === myPlayerIdx) {{
                document.getElementById('mysteryOptions').style.display = 'flex';
                document.getElementById('mysteryResult').style.display = 'none';
                document.getElementById('mysteryClose').style.display = 'none';
                document.getElementById('mysteryModal').classList.add('active');
            }} else if (mystery && mystery.stage === 'revealing') {{
                document.getElementById('mysteryOptions').style.display = 'none';
                const result = document.getElementById('mysteryResult');
                if (mystery.flip_result === 'win') {{
                    result.className = 'mystery-result win';
                    result.textContent = '🎉 $10,000!';
                }} else if (mystery.flip_result === 'bankrupt') {{
                    result.className = 'mystery-result lose';
                    result.textContent = '💀 BANKRUPT!';
                    SoundService.bankrupt();
                }}
                result.style.display = 'block';
                document.getElementById('mysteryClose').style.display = 'flex';
            }}

            // Render wheel
            if (gameState.wheel_slots && gameState.wheel_slots.length > 0) {{
                renderWheel(gameState.wheel_slots);

                // Animate wheel if spin index changed
                const spinIdx = gameState.last_spin_index;
                if (spinIdx !== null && spinIdx !== undefined && spinIdx !== prevSpinIdx) {{
                    prevSpinIdx = spinIdx;
                    animateWheelTo(spinIdx, gameState.wheel_slots);
                }}
            }}

            // ========== PUZZLE BOARD WITH VANNA-STYLE LETTER ANIMATIONS ==========
            const board = document.getElementById('puzzleBoard');
            const ROW_SIZES = [12, 14, 14, 12];
            const currentRevealed = new Set(gameState.revealed || []);
            const newlyRevealed = [...currentRevealed].filter(c => !prevRevealed.has(c));
            const VOWELS = new Set(['A', 'E', 'I', 'O', 'U']);

            if (gameState.puzzle?.answer) {{
                const answer = gameState.puzzle.answer.toUpperCase();
                const words = answer.split(' ');

                // Count how many letters are left to reveal
                const allLetters = answer.replace(/[^A-Z]/g, '').split('');
                const unrevealedCount = allLetters.filter(c => !currentRevealed.has(c)).length;
                const isFinalLetter = newlyRevealed.length > 0 && unrevealedCount === 0;

                // Check if this was a vowel purchase (exactly 1 unique vowel revealed, and puzzle not solved)
                // A vowel purchase reveals only 1 vowel letter type. If puzzle is solved (isFinalLetter), it's not a purchase.
                const isVowelPurchase = newlyRevealed.length === 1 && VOWELS.has(newlyRevealed[0]) && !isFinalLetter;

                // Count instances of newly revealed letters in the puzzle for value display
                let letterInstanceCount = 0;
                if (newlyRevealed.length > 0) {{
                    for (const char of allLetters) {{
                        if (newlyRevealed.includes(char)) letterInstanceCount++;
                    }}
                }}

                function layoutWords(startRow) {{
                    const rows = [[], [], [], []];
                    let currentRow = startRow;
                    for (const word of words) {{
                        if (currentRow >= 4) return null;
                        const currentLen = rows[currentRow].reduce((sum, w) => sum + w.length + 1, 0) - 1;
                        const spaceNeeded = currentLen > 0 ? word.length + 1 : word.length;
                        if (currentLen + spaceNeeded <= ROW_SIZES[currentRow]) {{
                            rows[currentRow].push(word);
                        }} else {{
                            currentRow++;
                            if (currentRow >= 4) return null;
                            rows[currentRow].push(word);
                        }}
                    }}
                    return rows;
                }}

                let rows = layoutWords(1);
                if (!rows) rows = layoutWords(0);
                if (!rows) rows = [[], [], [], []];

                let html = '';
                let revealDelay = 0;
                const isMultiReveal = newlyRevealed.length > 0 && letterInstanceCount > 1;

                for (let r = 0; r < 4; r++) {{
                    const rowSize = ROW_SIZES[r];
                    const rowWords = rows[r];
                    const rowText = rowWords.join(' ');
                    const padding = Math.floor((rowSize - rowText.length) / 2);

                    html += '<div class="puzzle-row">';
                    for (let i = 0; i < rowSize; i++) {{
                        const charIdx = i - padding;
                        if (charIdx >= 0 && charIdx < rowText.length) {{
                            const char = rowText[charIdx];
                            if (char === ' ') {{
                                html += '<div class="letter-tile blank"></div>';
                            }} else if (currentRevealed.has(char)) {{
                                const isNew = newlyRevealed.includes(char);
                                let animClasses = isNew ? 'revealing' : 'just-revealed';

                                // Add special effect classes
                                if (isNew) {{
                                    if (isFinalLetter) {{
                                        animClasses += ' final-letter';
                                    }} else if (isMultiReveal) {{
                                        animClasses += ' cascade';
                                    }}
                                    if (isVowelPurchase) {{
                                        animClasses += ' vowel-reveal';
                                    }}
                                }}

                                const delay = isNew ? `animation-delay: ${{revealDelay * 0.12}}s` : '';
                                if (isNew) revealDelay++;
                                html += `<div class="letter-tile revealed ${{animClasses}}" style="${{delay}}" data-char="${{char}}">${{char}}</div>`;
                            }} else {{
                                html += '<div class="letter-tile hidden"></div>';
                            }}
                        }} else {{
                            html += '<div class="letter-tile blank"></div>';
                        }}
                    }}
                    html += '</div>';
                }}
                board.innerHTML = html;

                // Play sound for new letters
                if (newlyRevealed.length > 0) {{
                    // Use appropriate sound based on reveal type
                    if (isVowelPurchase) {{
                        SoundService.vowelPurchase();
                    }} else if (letterInstanceCount > 1) {{
                        SoundService.multiLetterDing(letterInstanceCount);
                    }} else {{
                        SoundService.letterCorrect();
                    }}

                    // Show letter value popup if we have cash value from wheel
                    const wedge = gameState.current_wedge;
                    if (wedge && !isVowelPurchase && letterInstanceCount > 0) {{
                        let cashValue = 0;
                        if (typeof wedge === 'object' && wedge.Cash) {{
                            cashValue = wedge.Cash;
                        }} else if (typeof wedge === 'number') {{
                            cashValue = wedge;
                        }}

                        if (cashValue > 0) {{
                            showLetterValuePopup(cashValue, letterInstanceCount, false);
                        }}
                    }} else if (isVowelPurchase) {{
                        // Show vowel cost deduction
                        showLetterValuePopup(-250, letterInstanceCount, true);
                    }}

                    // Extra celebration for final letter
                    if (isFinalLetter) {{
                        setTimeout(() => {{
                            SoundService.solve();
                        }}, 500);
                    }}
                }}
            }} else {{
                let html = '';
                for (let r = 0; r < 4; r++) {{
                    html += '<div class="puzzle-row">';
                    for (let i = 0; i < ROW_SIZES[r]; i++) {{
                        html += '<div class="letter-tile blank"></div>';
                    }}
                    html += '</div>';
                }}
                board.innerHTML = html;
            }}
            prevRevealed = currentRevealed;

            // Wheel result
            const wedge = gameState.current_wedge;
            let resultText = '-';
            if (wedge !== null && wedge !== undefined) {{
                if (typeof wedge === 'object') {{
                    if (wedge.Cash) resultText = '$' + wedge.Cash;
                    else if (wedge.Prize) resultText = wedge.Prize.name || 'Prize';
                    else if ('Bankrupt' in wedge) resultText = 'BANKRUPT';
                    else if ('LoseTurn' in wedge) resultText = 'LOSE A TURN';
                    else if ('FreePlay' in wedge) resultText = 'FREE PLAY';
                    else if ('Mystery' in wedge) resultText = 'MYSTERY';
                    else if ('Express' in wedge) resultText = 'EXPRESS';
                    else {{
                        const key = Object.keys(wedge)[0] || '';
                        resultText = key.replace(/([A-Z])/g, ' $1').trim();
                    }}
                }} else if (typeof wedge === 'string') {{
                    resultText = wedge.replace(/([A-Z])/g, ' $1').trim();
                }} else {{
                    resultText = '$' + wedge;
                }}
            }}

            if (isWheelSpinning) {{
                pendingWheelResult = resultText;
            }} else {{
                document.getElementById('wheelResult').textContent = resultText;
            }}

            // ========== PLAYERS WITH SCORE TRACKING ==========
            const playerList = document.getElementById('playerList');
            if (gameState.players && gameState.players.length > 0) {{
                // Track score changes
                gameState.players.forEach((p, idx) => {{
                    const newScore = (p.total || 0) + (p.round_bank || 0);
                    const oldScore = prevScores[idx] || 0;
                    if (oldScore !== 0 && newScore !== oldScore) {{
                        setTimeout(() => showScoreChange(idx, newScore - oldScore), 100);
                    }}
                    prevScores[idx] = newScore;
                }});

                playerList.innerHTML = gameState.players.map((p, idx) => {{
                    const wildcards = p.wild_cards || 0;
                    const wildcardHtml = wildcards > 0 ?
                        `<div class="player-wildcards">${{'<div class="wildcard-icon">🃏</div>'.repeat(wildcards)}}</div>` : '';
                    const gameTotal = p.total || 0;
                    const roundTotal = p.round_bank || 0;
                    const allPrizes = [...(p.prizes || []), ...(p.round_prizes || [])];
                    const prizesHtml = allPrizes.length > 0 ?
                        `<div class="player-prizes">${{allPrizes.map(pr => `<span class="player-prize">${{pr.name}}</span>`).join('')}}</div>` : '';
                    const avatar = getAvatarEmoji(p.avatar_id);
                    const isActive = idx === gameState.active_idx;
                    return `
                        <div class="player ${{isActive ? 'active' : ''}}">
                            <div class="player-info">
                                <span class="player-avatar ${{isActive ? 'active' : ''}}">${{avatar}}</span>
                                <span class="player-name">${{p.name}}${{idx === myPlayerIdx ? ' (you)' : ''}}${{wildcardHtml}}</span>
                            </div>
                            <div class="player-score-details">
                                <span class="player-score-total">${{formatCash(gameTotal + roundTotal)}}</span>
                                <span class="player-score-round">Game: ${{formatCash(gameTotal)}} | Rnd: ${{formatCash(roundTotal)}}</span>
                                ${{prizesHtml}}
                            </div>
                        </div>
                    `;
                }}).join('');
            }}

            // ========== BONUS ROUND DISPLAY ==========
            const bonusStateData = gameState.bonus;
            if (phase === 'bonus' && bonusStateData && bonusStateData.stage !== 'off') {{
                // Sync server state with local bonus state
                const serverStage = bonusStateData.stage?.toLowerCase() || 'off';

                // Map server stages to client stages
                // Server: Off, Pick, Running, Done
                // Client: pick, solve, result (prize stage removed)

                if (!bonusState.active) {{
                    // First entry into bonus round - open bonus overlay
                    const activePlayer = gameState.players?.[gameState.active_idx];
                    const playerName = activePlayer?.name || 'Player';
                    const avatarId = activePlayer?.avatar_id || 1;
                    openBonusRound(playerName, avatarId);

                    // Sync to server stage
                    if (serverStage === 'pick') {{
                        // Sync any picks already made
                        bonusState.prizeAmount = bonusStateData.jackpot || 10000;
                        bonusState.pickedConsonants = bonusStateData.picks?.consonants || [];
                        bonusState.pickedVowel = bonusStateData.picks?.vowel || null;
                        initLetterPick();  // Re-init to sync UI with server state
                        showBonusStage('pick');
                    }} else if (serverStage === 'running') {{
                        bonusState.prizeAmount = bonusStateData.jackpot || 10000;
                        bonusState.pickedConsonants = bonusStateData.picks?.consonants || [];
                        bonusState.pickedVowel = bonusStateData.picks?.vowel || null;
                        updatePickedLettersDisplay();
                        showBonusStage('solve');
                    }} else if (serverStage === 'done') {{
                        bonusState.prizeAmount = bonusStateData.jackpot || 10000;
                        bonusState.pickedConsonants = bonusStateData.picks?.consonants || [];
                        bonusState.pickedVowel = bonusStateData.picks?.vowel || null;
                        updatePickedLettersDisplay();
                        showBonusStage('result');
                    }}
                }} else {{
                    // Already active - sync timer, picks, and stage
                    if (serverStage === 'pick') {{
                        // Sync picks from server - may have changed from another client
                        const serverConsonants = bonusStateData.picks?.consonants || [];
                        const serverVowel = bonusStateData.picks?.vowel || null;

                        // Only update if server state differs
                        if (JSON.stringify(serverConsonants) !== JSON.stringify(bonusState.pickedConsonants) ||
                            serverVowel !== bonusState.pickedVowel) {{
                            bonusState.pickedConsonants = serverConsonants;
                            bonusState.pickedVowel = serverVowel;
                            initLetterPick();  // Re-init to sync UI
                        }}

                        // Update puzzle board with RSTLNE revealed
                        const answer = gameState.puzzle?.answer || '';
                        const givenLetters = new Set(['R', 'S', 'T', 'L', 'N', 'E']);
                        renderBonusPuzzleBoard(answer, givenLetters);

                        if (bonusState.stage !== 'pick') {{
                            showBonusStage('pick');
                        }}
                    }} else if (serverStage === 'running') {{
                        bonusState.remainingSeconds = bonusStateData.remaining_seconds || 0;
                        updateBonusTimer(bonusState.remainingSeconds);

                        // Update bonus category
                        document.getElementById('bonusCategory').textContent = 'Category: ' + (gameState.puzzle?.category || '-');

                        // Render bonus puzzle board with revealed letters
                        const answer = gameState.puzzle?.answer || '';
                        const givenLetters = new Set(['R', 'S', 'T', 'L', 'N', 'E']);
                        const pickedLetters = new Set([
                            ...(bonusStateData.picks?.consonants || []),
                            bonusStateData.picks?.vowel || ''
                        ].filter(c => c));
                        const revealed = new Set([...givenLetters, ...pickedLetters, ...(gameState.revealed || [])]);
                        renderBonusPuzzleBoard(answer, revealed);

                        if (bonusState.stage !== 'solve') {{
                            showBonusStage('solve');
                        }}
                    }} else if (serverStage === 'done') {{
                        // Check if puzzle was solved
                        const puzzleSolved = gameState.puzzle_solved_by !== null;
                        showBonusResult(puzzleSolved, bonusState.prizeAmount, gameState.puzzle?.answer);

                        if (bonusState.stage !== 'result') {{
                            showBonusStage('result');
                        }}
                    }}
                }}
            }} else if (bonusState.active && (phase !== 'bonus' || !bonusStateData || bonusStateData.stage === 'off')) {{
                // Final round ended - clean up
                closeBonusRound();
            }}

            // ========== GAME OVER HANDLING ==========
            const gameOverData = gameState.game_over;
            const gameOverOverlay = document.getElementById('gameOverOverlay');
            const isGameOverActive = gameOverOverlay.classList.contains('active');

            if (isGameOver && gameOverData) {{
                // Game is in game over phase
                if (!isGameOverActive) {{
                    // Show game over overlay
                    showGameOver(gameOverData, gameState.players);
                }} else {{
                    // Update countdown if already showing
                    const countdownEl = document.getElementById('gameOverCountdown');
                    if (gameOverData.remaining_seconds !== null && gameOverData.remaining_seconds !== undefined) {{
                        countdownEl.textContent = gameOverData.remaining_seconds;
                    }}
                }}
            }} else if (isGameOverActive) {{
                // Game phase changed from game over to something else - close overlay
                closeGameOver();
            }}

            // ========== CONTROLS VISIBILITY ==========
            const isMyTurn = !isSpectating && gameState.active_idx === myPlayerIdx;
            const isTossup = phase === 'tossup';
            const isFinalSpin = phase === 'final';
            const isBonusRound = phase === 'bonus' && bonusStateData && bonusStateData.stage !== 'off';
            const isFinalRound = isBonusRound; // Backwards compat alias
            const canBuzz = !isSpectating && isTossup && myPlayerIdx !== null &&
                !(gameState.tossup?.locked_player_idxs || []).includes(myPlayerIdx);

            // Final Spin state
            const finalSpinState = gameState.final_spin || {{}};
            const finalSpinDone = finalSpinState.spin_done === true;
            const finalSpinValue = finalSpinState.spin_value || 0;

            // Show final spin value display during Final Spin phase after spin is done
            const finalSpinValueEl = document.getElementById('finalSpinValue');
            if (isFinalSpin && finalSpinDone) {{
                finalSpinValueEl.style.display = 'block';
                document.getElementById('finalSpinAmount').textContent = '$' + finalSpinValue.toLocaleString();
            }} else {{
                finalSpinValueEl.style.display = 'none';
            }}

            // Hide all controls when spectating, in bonus round, or game over
            document.getElementById('controls').style.opacity = isSpectating ? '0.5' : '1';
            document.getElementById('controls').style.display = (isFinalRound || isGameOver) ? 'none' : 'flex';

            // ===== PREGAME PHASE =====
            // Show Start Game button during pregame, hide everything else
            const startGameBtn = document.getElementById('startGameBtn');
            if (isPregame || isGameOver) {{
                startGameBtn.style.display = 'inline-block';
                startGameBtn.disabled = isSpectating;
                document.getElementById('spinBtn').style.display = 'none';
                document.getElementById('finalSpinSpinBtn').style.display = 'none';
                document.getElementById('buyVowelBtn').style.display = 'none';
                document.getElementById('solveBtn').style.display = 'none';
                document.getElementById('wildcardBtn').style.display = 'none';
                document.getElementById('guessArea').style.display = 'none';
            }} else {{
                startGameBtn.style.display = 'none';

                // ===== FINAL SPIN PHASE =====
                if (isFinalSpin) {{
                    // Hide normal spin button during final spin
                    document.getElementById('spinBtn').style.display = 'none';

                    // Final Spin spin button - show to ACTIVE PLAYER (not just host) if spin not done
                    // The active player when Final Spin triggers does the spin
                    // The resulting spin value then applies to ALL players taking turns
                    const finalSpinSpinBtn = document.getElementById('finalSpinSpinBtn');
                    if (!finalSpinDone && (isMyTurn || isHost)) {{
                        finalSpinSpinBtn.style.display = 'inline-block';
                        finalSpinSpinBtn.disabled = isSpectating || (!isMyTurn && !isHost);
                        finalSpinSpinBtn.classList.add('waiting');
                    }} else {{
                        finalSpinSpinBtn.style.display = 'none';
                        finalSpinSpinBtn.classList.remove('waiting');
                    }}

                    // During Final Spin: hide Buy Vowel button (vowels are free via single letter input)
                    document.getElementById('buyVowelBtn').style.display = 'none';

                    // Show solve button during final spin (after spin done)
                    document.getElementById('solveBtn').style.display = finalSpinDone ? 'inline-block' : 'none';
                    document.getElementById('solveBtn').disabled = isSpectating || !isMyTurn;

                    // Show guess area during final spin (after spin done) - accepts ANY letter
                    document.getElementById('guessArea').style.display = finalSpinDone ? 'flex' : 'none';
                    document.getElementById('letterInput').disabled = isSpectating || !isMyTurn;
                    // Update label to indicate any letter is allowed (single input for consonants and vowels)
                    const guessLabel = document.querySelector('.guess-label');
                    if (guessLabel && finalSpinDone) {{
                        guessLabel.textContent = 'Pick any letter (vowels free!)';
                    }}

                    // Hide wildcard during final spin
                    document.getElementById('wildcardBtn').style.display = 'none';
                }} else {{
                    // ===== NORMAL/TOSSUP PHASE =====
                    document.getElementById('finalSpinSpinBtn').style.display = 'none';

                    // Normal controls (hide during toss-up and bonus round)
                    document.getElementById('spinBtn').disabled = isSpectating || !isMyTurn || isTossup || isFinalRound;
                    document.getElementById('spinBtn').style.display = (isTossup || isFinalRound) ? 'none' : 'inline-block';

                    document.getElementById('buyVowelBtn').disabled = isSpectating || !isMyTurn || isTossup || isFinalRound;
                    document.getElementById('buyVowelBtn').style.display = (isTossup || isFinalRound) ? 'none' : 'inline-block';
                    document.getElementById('solveBtn').disabled = isSpectating || (!isMyTurn && !canBuzz) || isFinalRound;
                    document.getElementById('solveBtn').style.display = isFinalRound ? 'none' : 'inline-block';

                    // Show guess area during toss-up for the controller (active player who buzzed in)
                    const isTossupController = isTossup && isMyTurn && gameState.tossup?.remaining_seconds > 0;
                    document.getElementById('guessArea').style.display = (isTossup && !isTossupController) || isFinalRound ? 'none' : 'flex';
                    document.getElementById('letterInput').disabled = isSpectating || isFinalRound || (!isMyTurn && !isTossupController);

                    // Reset label for normal play
                    const guessLabel = document.querySelector('.guess-label');
                    if (guessLabel) {{
                        guessLabel.textContent = 'Select a Consonant';
                    }}
                }}
            }}

            // Flash spin button when waiting for player to spin (normal phase only)
            const needsToSpin = isMyTurn && (gameState.current_wedge === null || gameState.current_wedge === undefined) && !isTossup && !isFinalRound && !isFinalSpin && !isPregame;
            document.getElementById('spinBtn').classList.toggle('waiting', needsToSpin);

            // Flash the input when it's player's turn and they need to guess a letter
            const currentWedge = gameState.current_wedge;
            const isWedgeObject = currentWedge !== null && currentWedge !== undefined && typeof currentWedge === 'object';
            const isBankrupt = currentWedge === 'Bankrupt' || (isWedgeObject && 'Bankrupt' in currentWedge);
            const isLoseTurn = currentWedge === 'LoseTurn' || (isWedgeObject && 'LoseTurn' in currentWedge);
            const hasSpunAndCanGuess = currentWedge !== null && currentWedge !== undefined && !isBankrupt && !isLoseTurn;
            const waitingForInput = isMyTurn && hasSpunAndCanGuess && !isTossup && !isFinalRound;
            document.getElementById('guessArea').classList.toggle('waiting', waitingForInput);

            // ========== TURN TIMER DISPLAY ==========
            const turnTimerEl = document.getElementById('turnTimer');
            const turnTimerTextEl = document.getElementById('turnTimerText');
            const turnTimeRemaining = gameState.turn_timer_remaining;

            // Clear any existing timer interval when state updates
            if (turnTimerInterval) {{
                clearInterval(turnTimerInterval);
                turnTimerInterval = null;
            }}

            if (turnTimeRemaining !== null && turnTimeRemaining !== undefined && turnTimeRemaining > 0 && hasSpunAndCanGuess && !isTossup && !isFinalRound) {{
                turnTimerEl.classList.add('active');
                turnTimerLocalRemaining = turnTimeRemaining;
                turnTimerTextEl.textContent = turnTimerLocalRemaining + 's';
                turnTimerEl.classList.toggle('urgent', turnTimerLocalRemaining <= 3);

                // Start local countdown interval
                turnTimerInterval = setInterval(() => {{
                    turnTimerLocalRemaining--;
                    if (turnTimerLocalRemaining > 0) {{
                        turnTimerTextEl.textContent = turnTimerLocalRemaining + 's';
                        turnTimerEl.classList.toggle('urgent', turnTimerLocalRemaining <= 3);
                        // Play tick sound every second
                        if (turnTimerLocalRemaining <= 3) {{
                            SoundService.timerUrgent();
                        }} else {{
                            SoundService.timerTick();
                        }}
                    }} else {{
                        // Timer expired - play buzzer and hide
                        SoundService.timesUp();
                        turnTimerEl.classList.remove('active', 'urgent');
                        clearInterval(turnTimerInterval);
                        turnTimerInterval = null;
                    }}
                }}, 1000);
            }} else {{
                turnTimerEl.classList.remove('active', 'urgent');
                turnTimerLocalRemaining = null;
            }}

            // ========== BUZZ TIMER DISPLAY ==========
            const buzzTimerEl = document.getElementById('buzzTimer');
            const buzzTimerTextEl = document.getElementById('buzzTimerText');
            const buzzTimeRemaining = gameState.tossup?.remaining_seconds;

            // Clear any existing buzz timer interval when state updates
            if (buzzTimerInterval) {{
                clearInterval(buzzTimerInterval);
                buzzTimerInterval = null;
            }}

            if (isTossup && buzzTimeRemaining !== null && buzzTimeRemaining !== undefined && buzzTimeRemaining > 0) {{
                buzzTimerEl.style.display = 'flex';
                buzzTimerEl.classList.add('active');
                buzzTimerLocalRemaining = buzzTimeRemaining;
                buzzTimerTextEl.textContent = buzzTimerLocalRemaining + 's';
                buzzTimerEl.classList.toggle('urgent', buzzTimerLocalRemaining <= 2);

                // Start local countdown interval
                buzzTimerInterval = setInterval(() => {{
                    buzzTimerLocalRemaining--;
                    if (buzzTimerLocalRemaining > 0) {{
                        buzzTimerTextEl.textContent = buzzTimerLocalRemaining + 's';
                        buzzTimerEl.classList.toggle('urgent', buzzTimerLocalRemaining <= 2);
                        // Play tick sound every second
                        if (buzzTimerLocalRemaining <= 2) {{
                            SoundService.timerUrgent();
                        }} else {{
                            SoundService.timerTick();
                        }}
                    }} else {{
                        // Timer expired - play buzzer and hide
                        SoundService.timesUp();
                        buzzTimerEl.style.display = 'none';
                        buzzTimerEl.classList.remove('active', 'urgent');
                        clearInterval(buzzTimerInterval);
                        buzzTimerInterval = null;
                    }}
                }}, 1000);
            }} else {{
                buzzTimerEl.style.display = 'none';
                buzzTimerEl.classList.remove('active', 'urgent');
                buzzTimerLocalRemaining = null;
            }}

            // Buzz button for toss-up
            const buzzBtn = document.getElementById('buzzBtn');
            buzzBtn.style.display = canBuzz && !isFinalRound ? 'inline-block' : 'none';

            // Wild card button
            const myPlayer = gameState.players?.[myPlayerIdx];
            const hasWildCard = !isSpectating && myPlayer && (myPlayer.wild_cards || 0) > 0;
            const wildcardBtn = document.getElementById('wildcardBtn');
            wildcardBtn.classList.toggle('available', isMyTurn && hasWildCard && !isTossup && !isFinalRound);

            // Update host control buttons based on phase
            const tossupBtn = document.getElementById('tossupBtn');
            const finalSpinBtn = document.getElementById('finalSpinBtn');
            const bonusBtn = document.getElementById('bonusBtn');
            if (tossupBtn) {{
                tossupBtn.textContent = isTossup ? 'End Toss-up' : 'Start Toss-up';
                tossupBtn.classList.toggle('btn-danger', isTossup);
            }}
            if (finalSpinBtn) {{
                finalSpinBtn.textContent = isFinalSpin ? 'End Final Spin' : 'Start Final Spin';
                finalSpinBtn.classList.toggle('btn-danger', isFinalSpin);
            }}
            if (bonusBtn) {{
                bonusBtn.textContent = isBonusRound ? 'End Bonus' : 'Start Bonus';
                bonusBtn.classList.toggle('btn-danger', isBonusRound);
            }}

            // Populate active player dropdown
            const activePlayerSelect = document.getElementById('activePlayerSelect');
            if (activePlayerSelect && gameState.players) {{
                const currentValue = activePlayerSelect.value;
                activePlayerSelect.innerHTML = '<option value="">Set Active Player</option>';
                gameState.players.forEach((player, idx) => {{
                    const option = document.createElement('option');
                    option.value = idx;
                    option.textContent = player.name + (idx === gameState.active_idx ? ' (active)' : '');
                    activePlayerSelect.appendChild(option);
                }});
                activePlayerSelect.value = currentValue;
            }}
        }}

        let notificationTimeout = null;
        let turnTimerInterval = null;
        let turnTimerLocalRemaining = null;
        let buzzTimerInterval = null;
        let buzzTimerLocalRemaining = null;

        function showNotification(msg, type = 'info', duration = 5000) {{
            const notif = document.getElementById('notification');

            // Clear any existing timeout
            if (notificationTimeout) {{
                clearTimeout(notificationTimeout);
            }}

            // Remove previous classes
            notif.classList.remove('show', 'hide', 'success', 'error', 'warning', 'letter-result');

            // Check for letter guess results and enhance them
            let displayMsg = msg;
            let detectedType = type;
            const msgLower = msg.toLowerCase();

            // Match letter found pattern: "3 L(s)!" or "1 A(s)!"
            const letterFoundMatch = msg.match(/^(\d+)\s+([A-Z])\(s\)!$/i);
            // Match letter not found pattern: "No Ls" or "No As"
            const letterNotFoundMatch = msg.match(/^No\s+([A-Z])s$/i);

            if (letterFoundMatch) {{
                const count = letterFoundMatch[1];
                const letter = letterFoundMatch[2].toUpperCase();
                displayMsg = `<div class="letter-result-content"><span class="letter-big">${{letter}}</span><span class="letter-count">${{count}} found!</span></div>`;
                detectedType = 'success';
                notif.classList.add('letter-result');
            }} else if (letterNotFoundMatch) {{
                const letter = letterNotFoundMatch[1].toUpperCase();
                displayMsg = `<div class="letter-result-content"><span class="letter-big letter-miss">${{letter}}</span><span class="letter-count">Not in puzzle</span></div>`;
                detectedType = 'error';
                notif.classList.add('letter-result');
                SoundService.letterWrong();
            }} else if (msgLower.includes('correct') || msgLower.includes('solved') || msgLower.includes('win') || msgLower.includes('🎉')) {{
                detectedType = 'success';
            }} else if (msgLower.includes('bankrupt') || msgLower.includes('incorrect') || msgLower.includes('error') || msgLower.includes('failed') || msgLower.includes('invalid')) {{
                detectedType = 'error';
            }} else if (msgLower.includes('lose a turn') || msgLower.includes('locked out') || msgLower.includes('warning')) {{
                detectedType = 'warning';
            }}

            // Add icon based on type (only for non-letter results)
            let icon = '';
            if (!letterFoundMatch && !letterNotFoundMatch) {{
                if (detectedType === 'success') icon = '✓ ';
                else if (detectedType === 'error') icon = '✗ ';
                else if (detectedType === 'warning') icon = '⚠ ';
                else icon = '';
            }}

            // Play sound based on notification type
            if (!letterNotFoundMatch && (msgLower.includes('not in the puzzle') || msgLower.includes('no letters') || msgLower.includes('wrong') || msgLower.includes('incorrect'))) {{
                SoundService.letterWrong();
            }} else if (msgLower.includes('bankrupt')) {{
                SoundService.bankrupt();
            }}

            // Set content
            if (letterFoundMatch || letterNotFoundMatch) {{
                notif.innerHTML = displayMsg;
            }} else {{
                notif.innerHTML = `<span class="icon">${{icon}}</span>${{msg}}`;
            }}

            // Add type class
            if (detectedType !== 'info') {{
                notif.classList.add(detectedType);
            }}

            // Show with animation
            notif.style.display = 'block';
            // Force reflow for animation
            void notif.offsetWidth;
            notif.classList.add('show');

            // Auto-hide after duration
            notificationTimeout = setTimeout(() => {{
                hideNotification();
            }}, duration);
        }}

        function hideNotification() {{
            const notif = document.getElementById('notification');
            notif.classList.remove('show');
            notif.classList.add('hide');
            setTimeout(() => {{
                notif.style.display = 'none';
                notif.classList.remove('hide');
            }}, 200);
        }}

        function spin() {{
            hideNotification();
            // Set spinning state immediately so incoming toasts get queued
            isWheelSpinning = true;
            document.getElementById('wheelResult').textContent = 'Spinning...';
            socket.emit('spin', {{ room }});

            // Fallback: if no animation started within 2 seconds, reset and show pending toasts
            setTimeout(() => {{
                if (isWheelSpinning && !wheelAnimationId) {{
                    isWheelSpinning = false;
                    document.getElementById('wheelResult').textContent = '-';
                    onWheelStopped();
                }}
            }}, 2000);
        }}

        function guessLetter(letterParam) {{
            const input = document.getElementById('letterInput');
            const letter = letterParam || input.value.toUpperCase();
            if (letter && letter.length === 1) {{
                // Use tossup_guess during toss-up mode, regular guess otherwise
                const isTossup = gameState && gameState.phase === 'tossup';
                const eventName = isTossup ? 'tossup_guess' : 'guess';
                socket.emit(eventName, {{ room, letter }});
                // Only clear if not called with a parameter (event listener handles its own clearing)
                if (!letterParam) {{
                    input.value = '';
                }}
            }}
        }}

        function buyVowel() {{
            document.getElementById('vowelModal').classList.add('active');
        }}

        function selectVowel(vowel) {{
            socket.emit('buy_vowel', {{ room, letter: vowel }});
            closeModal('vowelModal');
        }}

        function promptSolve() {{
            document.getElementById('solveInput').value = '';
            document.getElementById('solveModal').classList.add('active');
            document.getElementById('solveInput').focus();
        }}

        function submitSolve() {{
            const solution = document.getElementById('solveInput').value.trim();
            if (solution) {{
                socket.emit('solve', {{ room, attempt: solution }});
            }}
            closeModal('solveModal');
        }}

        function closeModal(modalId) {{
            document.getElementById(modalId).classList.remove('active');
        }}

        // Start game (transitions from pregame to normal)
        function startGame() {{
            socket.emit('start_game', {{ room }});
        }}

        // Final Spin - host does the one and only spin
        function doFinalSpin() {{
            hideNotification();
            isWheelSpinning = true;
            document.getElementById('wheelResult').textContent = 'Final Spin...';
            socket.emit('final_spin_spin', {{ room }});

            // Fallback: if no animation started within 2 seconds, reset
            setTimeout(() => {{
                if (isWheelSpinning && !wheelAnimationId) {{
                    isWheelSpinning = false;
                    document.getElementById('wheelResult').textContent = '-';
                    onWheelStopped();
                }}
            }}, 2000);
        }}

        // Host functions
        let isHost = false;

        function promptClaimHost() {{
            document.getElementById('hostCodeInput').value = '';
            document.getElementById('claimHostModal').classList.add('active');
            document.getElementById('hostCodeInput').focus();
        }}

        function submitClaimHost() {{
            const code = document.getElementById('hostCodeInput').value.trim();
            if (code) {{
                socket.emit('claim_host', {{ room, code }});
            }}
            closeModal('claimHostModal');
        }}

        function newGame() {{
            socket.emit('new_game', {{ room }});
        }}

        function revealAll() {{
            socket.emit('reveal_all', {{ room }});
        }}

        function changePack() {{
            const select = document.getElementById('packSelect');
            const packId = select.value ? parseInt(select.value) : null;
            const packName = select.options[select.selectedIndex]?.textContent?.split(' (')[0] || 'All Packs';
            socket.emit('set_pack', {{ room, pack_id: packId, pack_name: packName }});
        }}

        function newPuzzle() {{
            socket.emit('new_puzzle', {{ room }});
        }}

        function hostSpin() {{
            socket.emit('spin', {{ room }});
        }}

        function toggleTossup() {{
            const phase = (gameState?.phase || '').toLowerCase();
            if (phase === 'tossup') {{
                socket.emit('end_tossup', {{ room }});
            }} else {{
                socket.emit('start_tossup', {{ room }});
            }}
        }}

        function toggleFinalSpin() {{
            const phase = (gameState?.phase || '').toLowerCase();
            if (phase === 'final') {{
                socket.emit('end_final_spin', {{ room }});
            }} else {{
                socket.emit('start_final_spin', {{ room }});
            }}
        }}

        function toggleBonus() {{
            const phase = (gameState?.phase || '').toLowerCase();
            if (phase === 'bonus') {{
                socket.emit('end_bonus', {{ room }});
            }} else {{
                socket.emit('start_bonus', {{ room }});
            }}
        }}

        // Legacy function for backwards compatibility
        function toggleFinal() {{
            toggleBonus();
        }}

        // ========== ROUND MANAGEMENT ==========

        function advanceRound() {{
            socket.emit('advance_round', {{ room }});
        }}

        function setRound() {{
            const select = document.getElementById('roundSelect');
            const roundNum = parseInt(select.value);
            if (roundNum >= 1 && roundNum <= 4) {{
                socket.emit('set_round', {{ room, round: roundNum }});
            }}
        }}

        function setActivePlayer() {{
            const select = document.getElementById('activePlayerSelect');
            const playerIdx = select.value ? parseInt(select.value) : null;
            if (playerIdx !== null) {{
                socket.emit('set_active', {{ room, player_idx: playerIdx }});
            }}
            select.value = ''; // Reset selection
        }}

        async function loadPacks() {{
            try {{
                const token = localStorage.getItem('auth_token');
                const response = await fetch('/auth/api/packs', {{
                    credentials: 'include',
                    headers: token ? {{
                        'Authorization': `Bearer ${{token}}`
                    }} : {{}}
                }});
                const data = await response.json();
                if (data.ok && data.packs) {{
                    const select = document.getElementById('packSelect');
                    // Clear existing options except the first one
                    while (select.options.length > 1) {{
                        select.remove(1);
                    }}
                    // Add pack options
                    data.packs.forEach(pack => {{
                        const option = document.createElement('option');
                        option.value = pack.id;
                        option.textContent = `${{pack.name}} (${{pack.puzzle_count}})`;
                        select.appendChild(option);
                    }});
                }}
            }} catch (e) {{
                console.error('Failed to load packs:', e);
            }}
        }}

        function updateHostUI() {{
            document.getElementById('hostControls').style.display = isHost ? 'block' : 'none';
            document.getElementById('claimHostSection').style.display = isHost ? 'none' : 'block';
        }}

        // ========== NEW GAME MECHANICS ==========

        // Mystery wedge choice
        function mysteryChoice(choice) {{
            const options = document.getElementById('mysteryOptions');
            const result = document.getElementById('mysteryResult');
            const closeBtn = document.getElementById('mysteryClose');

            options.style.display = 'none';
            result.style.display = 'block';

            if (choice === 'keep') {{
                result.innerHTML = '<div style="font-size: 48px; color: #22c55e;">💰 $1,000</div><p>Safe choice!</p>';
                SoundService.letterCorrect();
            }} else {{
                // Simulate 50/50 flip - server will handle actual result
                result.innerHTML = '<div style="font-size: 48px;">🎲</div><p>Flipping...</p>';
            }}

            socket.emit('mystery_choice', {{ room, choice }});

            // Server will send result, we show close button after a delay
            setTimeout(() => {{
                closeBtn.style.display = 'block';
            }}, 1500);
        }}

        // Wild card functions
        function useWildCard() {{
            document.getElementById('wildcardInput').value = '';
            document.getElementById('wildcardModal').classList.add('active');
            document.getElementById('wildcardInput').focus();
        }}

        function submitWildCard() {{
            const input = document.getElementById('wildcardInput');
            const letter = input.value.toUpperCase();
            const consonants = 'BCDFGHJKLMNPQRSTVWXYZ';

            if (letter && letter.length === 1 && consonants.includes(letter)) {{
                socket.emit('use_wild_card', {{ room, letter }});
                closeModal('wildcardModal');
            }} else {{
                alert('Please enter a valid consonant');
            }}
        }}

        // Toggle sound mute/unmute
        function toggleMute() {{
            SoundService.toggle();
        }}

        // Buzz in for toss-up
        function buzz() {{
            SoundService.buzz();
            socket.emit('buzz', {{ room }});
        }}

        // Auto-guess when letter is typed
        document.getElementById('letterInput').addEventListener('input', (e) => {{
            const input = e.target;
            const letter = input.value.toUpperCase();
            if (letter && letter.length === 1 && /[A-Z]/.test(letter)) {{
                input.value = letter; // Show uppercase
                input.disabled = true; // Prevent additional input while showing
                guessLetter(letter);
                // Keep letter visible for 1 second before clearing
                setTimeout(() => {{
                    input.value = '';
                    input.disabled = false;
                    input.focus();
                }}, 1000);
            }}
        }});

        // Enter key to submit solve
        document.getElementById('solveInput').addEventListener('keypress', (e) => {{
            if (e.key === 'Enter') submitSolve();
        }});

        // Enter key to submit host code
        document.getElementById('hostCodeInput').addEventListener('keypress', (e) => {{
            if (e.key === 'Enter') submitClaimHost();
        }});

        // Enter key to submit wild card
        document.getElementById('wildcardInput').addEventListener('keypress', (e) => {{
            if (e.key === 'Enter') submitWildCard();
        }});

        // Close modals on escape key
        document.addEventListener('keydown', (e) => {{
            if (e.key === 'Escape') {{
                closeModal('solveModal');
                closeModal('vowelModal');
                closeModal('claimHostModal');
                closeModal('mysteryModal');
                closeModal('wildcardModal');
            }}
        }});

        // Initialize sound service (must be after user interaction)
        document.addEventListener('click', function initSound() {{
            SoundService.init();
            document.removeEventListener('click', initSound);
        }}, {{ once: true }});

        // Update mute button state on page load (before sound init)
        SoundService.updateMuteButton();

        // Show spectator banner if in spectator mode
        if (isSpectating) {{
            document.getElementById('spectatorBanner').style.display = 'inline-block';
        }}

        connect();
        loadPacks(); // Pre-load packs so they're ready when user becomes host
    </script>
</body>
</html>"##,
        common_styles = COMMON_STYLES
    ))
}

/// Admin page
pub async fn admin() -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Admin</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Mountains+of+Christmas:wght@700&display=swap" rel="stylesheet">
    <style>
        {common_styles}
        body {{ align-items: flex-start; padding: 20px; }}
        .admin-container {{
            width: 100%;
            max-width: 1200px;
        }}
        .admin-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 24px;
            padding-bottom: 16px;
            border-bottom: 2px solid #333;
        }}
        .tabs {{
            display: flex;
            gap: 8px;
            margin-bottom: 24px;
        }}
        .tab {{
            padding: 12px 24px;
            background: #333;
            color: #fff;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-size: 16px;
        }}
        .tab.active {{ background: #d4af37; color: #0d0628; }}
        .tab:hover {{ background: #444; }}
        .tab.active:hover {{ background: #e5c048; }}
        .panel {{
            background: rgba(26, 10, 62, 0.8);
            border-radius: 16px;
            border: 2px solid #333;
            padding: 24px;
            display: none;
        }}
        .panel.active {{ display: block; }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 16px;
        }}
        th, td {{
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #333;
        }}
        th {{ color: #d4af37; }}
        td {{ color: #fff; }}
        .badge {{
            display: inline-block;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 12px;
        }}
        .badge-success {{ background: #2d8f4e; color: #fff; }}
        .badge-warning {{ background: #c9a227; color: #000; }}
        .badge-danger {{ background: #c0392b; }}
        .btn-sm {{
            padding: 6px 12px;
            font-size: 14px;
            margin: 2px;
        }}
        .form-row {{
            display: flex;
            gap: 12px;
            margin-bottom: 16px;
            align-items: flex-end;
        }}
        .form-row .form-group {{ flex: 1; margin-bottom: 0; }}
        .error-msg {{
            background: #c0392b;
            color: #fff;
            padding: 12px;
            border-radius: 8px;
            margin-bottom: 16px;
            display: none;
        }}
        .success-msg {{
            background: #2d8f4e;
            color: #fff;
            padding: 12px;
            border-radius: 8px;
            margin-bottom: 16px;
            display: none;
        }}
        .access-denied {{
            text-align: center;
            padding: 60px;
        }}
        .access-denied h2 {{ color: #c0392b; margin-bottom: 16px; }}

        /* Room cards */
        .room-card {{
            background: #0d0628;
            border: 2px solid #333;
            border-radius: 12px;
            margin-bottom: 16px;
            overflow: hidden;
        }}
        .room-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 16px;
            background: linear-gradient(180deg, #1a0a3e 0%, #0d0628 100%);
            border-bottom: 1px solid #333;
        }}
        .room-info {{
            display: flex;
            align-items: center;
            gap: 16px;
        }}
        .room-name {{
            font-size: 20px;
            font-weight: bold;
            color: #d4af37;
        }}
        .room-meta {{
            display: flex;
            gap: 12px;
        }}
        .room-meta span {{
            color: #888;
            font-size: 14px;
        }}
        .room-actions {{
            display: flex;
            gap: 8px;
        }}
        .room-players {{
            padding: 16px;
        }}
        .room-players h4 {{
            color: #888;
            margin: 0 0 12px 0;
            font-size: 14px;
            text-transform: uppercase;
        }}
        .player-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
            gap: 12px;
        }}
        .player-card {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 12px 16px;
            background: rgba(26, 10, 62, 0.6);
            border-radius: 8px;
            border: 1px solid #333;
        }}
        .player-card.active {{
            border-color: #d4af37;
            box-shadow: 0 0 10px rgba(212, 175, 55, 0.2);
        }}
        .player-details {{
            display: flex;
            flex-direction: column;
            gap: 4px;
        }}
        .player-name-admin {{
            color: #fff;
            font-weight: 500;
            display: flex;
            align-items: center;
            gap: 6px;
        }}
        .player-avatar-admin {{
            font-size: 20px;
        }}
        .player-score-admin {{
            color: #d4af37;
            font-size: 14px;
        }}
        .player-actions {{
            display: flex;
            gap: 4px;
        }}
        .no-rooms {{
            color: #888;
            text-align: center;
            padding: 40px;
        }}

        /* Modal styles */
        .modal {{
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.8);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 1000;
        }}
        .modal-content {{
            background: #1a0a3e;
            border: 2px solid #333;
            border-radius: 16px;
            width: 90%;
            max-height: 90vh;
            overflow-y: auto;
        }}
        .modal-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 20px 24px;
            border-bottom: 1px solid #333;
        }}
        .modal-header h3 {{
            color: #d4af37;
            margin: 0;
        }}
        .modal-close {{
            background: none;
            border: none;
            color: #888;
            font-size: 28px;
            cursor: pointer;
            padding: 0;
            line-height: 1;
        }}
        .modal-close:hover {{
            color: #fff;
        }}
        .modal-body {{
            padding: 24px;
        }}

        /* Room details section */
        .room-details {{
            padding: 16px;
            border-top: 1px solid #333;
            background: rgba(13, 6, 40, 0.5);
            display: none;
        }}
        .room-details.expanded {{
            display: block;
        }}
        .room-details-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
            gap: 16px;
            margin-bottom: 16px;
        }}
        .detail-item {{
            background: rgba(26, 10, 62, 0.6);
            padding: 12px 16px;
            border-radius: 8px;
            border: 1px solid #333;
        }}
        .detail-label {{
            color: #888;
            font-size: 12px;
            text-transform: uppercase;
            margin-bottom: 4px;
        }}
        .detail-value {{
            color: #fff;
            font-size: 16px;
            font-weight: 500;
        }}
        .detail-value.gold {{
            color: #d4af37;
        }}
        .detail-value.green {{
            color: #4caf50;
        }}
        .puzzle-preview {{
            background: #1a5cb8;
            padding: 16px;
            border-radius: 8px;
            margin-top: 16px;
        }}
        .puzzle-preview-label {{
            color: rgba(255,255,255,0.7);
            font-size: 12px;
            text-transform: uppercase;
            margin-bottom: 8px;
        }}
        .puzzle-preview-answer {{
            font-family: monospace;
            font-size: 18px;
            color: #fff;
            word-spacing: 8px;
            letter-spacing: 2px;
        }}
        .connection-dot {{
            display: inline-block;
            width: 8px;
            height: 8px;
            border-radius: 50%;
            margin-right: 6px;
        }}
        .connection-dot.connected {{
            background: #4caf50;
        }}
        .connection-dot.disconnected {{
            background: #888;
        }}
    </style>
</head>
<body>
    <div class="admin-container">
        <div class="admin-header">
            <h1 style="color: #d4af37; margin: 0;">🎡 Admin Panel</h1>
            <div>
                <span id="adminUser" style="color: #888; margin-right: 16px;"></span>
                <a href="/lobby" class="btn btn-secondary">Back to Lobby</a>
            </div>
        </div>

        <div id="accessDenied" class="access-denied" style="display: none;">
            <h2>Access Denied</h2>
            <p style="color: #888;">You must be an admin to access this page.</p>
            <a href="/lobby" class="btn" style="margin-top: 16px;">Go to Lobby</a>
        </div>

        <div id="adminContent" style="display: none;">
            <div class="tabs">
                <button class="tab active" onclick="showTab('users')">Users</button>
                <button class="tab" onclick="showTab('packs')">Puzzle Packs</button>
                <button class="tab" onclick="showTab('puzzles')">Puzzles</button>
                <button class="tab" onclick="showTab('rooms')">Rooms</button>
                <button class="tab" onclick="showTab('database')">Database</button>
            </div>

            <div class="error-msg" id="errorMsg"></div>
            <div class="success-msg" id="successMsg"></div>

            <!-- Users Panel -->
            <div class="panel active" id="panel-users">
                <h2 style="color: #fff; margin-bottom: 16px;">User Management</h2>
                <table>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>Email</th>
                            <th>Display Name</th>
                            <th>Room</th>
                            <th>Status</th>
                            <th>Admin</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody id="usersTable"></tbody>
                </table>
            </div>

            <!-- Packs Panel -->
            <div class="panel" id="panel-packs">
                <h2 style="color: #fff; margin-bottom: 16px;">Puzzle Packs</h2>
                <div class="form-row">
                    <div class="form-group">
                        <label>New Pack Name</label>
                        <input type="text" id="newPackName" placeholder="Enter pack name">
                    </div>
                    <button class="btn" onclick="createPack()">Create Pack</button>
                </div>
                <div class="form-row" style="margin-top: 24px; padding-top: 24px; border-top: 1px solid #333;">
                    <div class="form-group">
                        <label>Import Puzzle Pack (JSON file)</label>
                        <input type="file" id="importFile" accept=".json" style="padding: 8px; background: #0d0628; color: #fff; border: 2px solid #333; border-radius: 8px; width: 100%;">
                    </div>
                    <button class="btn btn-secondary" onclick="importPack()">Import Pack</button>
                </div>
                <p style="color: #888; font-size: 12px; margin-top: 8px;">
                    JSON format: {{ "name": "Pack Name", "puzzles": [{{ "category": "PHRASE", "answer": "HELLO WORLD" }}] }}
                </p>
                <table>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>Name</th>
                            <th>Puzzles</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody id="packsTable"></tbody>
                </table>
            </div>

            <!-- Puzzles Panel -->
            <div class="panel" id="panel-puzzles">
                <h2 style="color: #fff; margin-bottom: 16px;">Puzzles</h2>
                <div class="form-row">
                    <div class="form-group">
                        <label>Pack</label>
                        <select id="puzzlePackSelect" style="width:100%;padding:12px;background:#0d0628;color:#fff;border:2px solid #333;border-radius:8px;">
                            <option value="">All Packs</option>
                        </select>
                    </div>
                    <button class="btn btn-secondary" onclick="loadPuzzles()">Filter</button>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label>Category</label>
                        <input type="text" id="newPuzzleCategory" placeholder="e.g., Phrase, Thing, Person">
                    </div>
                    <div class="form-group">
                        <label>Answer</label>
                        <input type="text" id="newPuzzleAnswer" placeholder="PUZZLE ANSWER" style="text-transform:uppercase;">
                    </div>
                    <div class="form-group">
                        <label>Pack</label>
                        <select id="newPuzzlePack" style="width:100%;padding:12px;background:#0d0628;color:#fff;border:2px solid #333;border-radius:8px;"></select>
                    </div>
                    <button class="btn" onclick="addPuzzle()">Add Puzzle</button>
                </div>
                <table>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>Category</th>
                            <th>Answer</th>
                            <th>Pack</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody id="puzzlesTable"></tbody>
                </table>
            </div>

            <!-- Rooms Panel -->
            <div class="panel" id="panel-rooms">
                <h2 style="color: #fff; margin-bottom: 16px;">Active Rooms</h2>
                <div id="roomsContainer"></div>
            </div>

            <!-- Database Panel -->
            <div class="panel" id="panel-database">
                <h2 style="color: #fff; margin-bottom: 16px;">Database Browser</h2>
                <p style="color: #888; margin-bottom: 16px;">Browse database tables for debugging purposes. Data is read-only.</p>
                <div class="form-row">
                    <div class="form-group">
                        <label>Select Table</label>
                        <select id="dbTableSelect" onchange="loadTableData()" style="width:100%;padding:12px;background:#0d0628;color:#fff;border:2px solid #333;border-radius:8px;">
                            <option value="">-- Select a table --</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label>Rows per page</label>
                        <select id="dbPageSize" onchange="loadTableData()" style="width:100%;padding:12px;background:#0d0628;color:#fff;border:2px solid #333;border-radius:8px;">
                            <option value="25">25</option>
                            <option value="50" selected>50</option>
                            <option value="100">100</option>
                            <option value="250">250</option>
                        </select>
                    </div>
                </div>

                <div id="dbTableInfo" style="margin: 16px 0; color: #888;"></div>

                <div style="overflow-x: auto;">
                    <table id="dbDataTable" style="min-width: 100%;">
                        <thead id="dbTableHead"></thead>
                        <tbody id="dbTableBody"></tbody>
                    </table>
                </div>

                <div id="dbPagination" style="margin-top: 16px; display: flex; gap: 8px; align-items: center; justify-content: center;"></div>
            </div>
        </div>
    </div>

    <!-- Settings Modal -->
    <div id="settingsModal" class="modal" style="display:none;">
        <div class="modal-content" style="max-width: 600px;">
            <div class="modal-header">
                <h3 id="settingsModalTitle">Room Settings</h3>
                <button class="modal-close" onclick="closeSettingsModal()">&times;</button>
            </div>
            <div class="modal-body">
                <input type="hidden" id="settingsRoom" value="">

                <h4 style="color: #d4af37; margin: 0 0 16px;">Puzzle Settings</h4>
                <div class="form-row">
                    <div class="form-group">
                        <label>Puzzle Pack</label>
                        <select id="settingsPackId" style="width:100%;padding:12px;background:#0d0628;color:#fff;border:2px solid #333;border-radius:8px;">
                            <option value="0">All Packs</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label>Puzzle Display Time (seconds)</label>
                        <input type="number" id="settingsPuzzleDisplay" value="30" min="5" max="120" placeholder="30">
                    </div>
                </div>

                <h4 style="color: #d4af37; margin: 24px 0 16px;">Cost Settings</h4>
                <div class="form-row">
                    <div class="form-group">
                        <label>Vowel Cost ($)</label>
                        <input type="number" id="settingsVowelCost" value="250" min="0" placeholder="250">
                    </div>
                </div>

                <h4 style="color: #d4af37; margin: 24px 0 16px;">Final Round Settings</h4>
                <div class="form-row">
                    <div class="form-group">
                        <label>Final Round Timer (seconds)</label>
                        <input type="number" id="settingsFinalSeconds" value="30" min="10" max="120" placeholder="30">
                    </div>
                    <div class="form-group">
                        <label>Final Round Jackpot ($)</label>
                        <input type="number" id="settingsFinalJackpot" value="10000" min="1000" placeholder="10000">
                    </div>
                </div>

                <h4 style="color: #d4af37; margin: 24px 0 16px;">Prize Wedge Names</h4>
                <div class="form-row">
                    <div class="form-group">
                        <label>Prize Wedge Names (comma-separated)</label>
                        <input type="text" id="settingsPrizeWedges" value="GIFT CARD" placeholder="GIFT CARD, TRIP, CAR">
                    </div>
                </div>

                <h4 style="color: #d4af37; margin: 24px 0 16px;">Player Settings</h4>
                <div class="form-row">
                    <div class="form-group">
                        <label>Disconnect Timeout (seconds, 0 = never remove)</label>
                        <input type="number" id="settingsDisconnectTimeout" value="300" min="0" max="3600" placeholder="300">
                        <small style="color:#888;display:block;margin-top:4px;">Disconnected players are removed after this time. Default: 300 (5 minutes)</small>
                    </div>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label>Turn Timer (seconds, 0 = disabled)</label>
                        <input type="number" id="settingsTurnTimer" value="10" min="0" max="60" placeholder="10">
                        <small style="color:#888;display:block;margin-top:4px;">Time limit to guess a letter after spinning. Default: 10 seconds</small>
                    </div>
                    <div class="form-group">
                        <label>Toss-Up Buzz Timer (seconds, 0 = disabled)</label>
                        <input type="number" id="settingsBuzzTimer" value="5" min="0" max="30" placeholder="5">
                        <small style="color:#888;display:block;margin-top:4px;">Time limit to solve after buzzing in during toss-up. Default: 5 seconds</small>
                    </div>
                </div>

                <button class="btn" onclick="saveSettings()" style="margin-top: 24px; width: 100%;">Save Settings</button>
            </div>
        </div>
    </div>

    <!-- Create Room Modal -->
    <div id="createRoomModal" class="modal" style="display:none;">
        <div class="modal-content" style="max-width: 400px;">
            <div class="modal-header">
                <h3>Create New Room</h3>
                <button class="modal-close" onclick="closeCreateRoomModal()">&times;</button>
            </div>
            <div class="modal-body">
                <div class="form-group">
                    <label>Room Name</label>
                    <input type="text" id="newRoomName" placeholder="Enter room name" style="width:100%;padding:12px;background:#0d0628;color:#fff;border:2px solid #333;border-radius:8px;">
                </div>
                <p style="color: #888; font-size: 14px; margin: 16px 0;">
                    Room names should be lowercase letters, numbers, and hyphens only.
                </p>
                <button class="btn" onclick="createRoom()" style="width: 100%;">Create Room</button>
            </div>
        </div>
    </div>

    <script>
        const user = JSON.parse(localStorage.getItem('user') || 'null');
        if (!user) {{ window.location.href = '/'; }}

        let isAdmin = false;
        let packs = [];

        async function checkAdmin() {{
            try {{
                const res = await fetch('/auth/api/admin/users', {{
                    credentials: 'include'
                }});
                if (res.status === 403) {{
                    document.getElementById('accessDenied').style.display = 'block';
                    return;
                }}
                if (res.status === 401) {{
                    // Not authenticated, redirect to login
                    window.location.href = '/';
                    return;
                }}
                if (res.ok) {{
                    isAdmin = true;
                    document.getElementById('adminUser').textContent = user.display_name + ' (Admin)';
                    document.getElementById('adminContent').style.display = 'block';
                    loadUsers();
                    loadPacks();
                    loadRooms();
                }} else {{
                    // Unexpected response
                    const data = await res.json().catch(() => ({{}}));
                    showError(data.error || 'Failed to load admin panel: ' + res.status);
                    document.getElementById('accessDenied').style.display = 'block';
                }}
            }} catch (e) {{
                console.error('Admin check error:', e);
                showError('Failed to verify admin access: ' + e.message);
                document.getElementById('accessDenied').style.display = 'block';
            }}
        }}

        function showTab(tab) {{
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
            document.querySelector(`[onclick="showTab('${{tab}}')"]`).classList.add('active');
            document.getElementById('panel-' + tab).classList.add('active');
        }}

        function showError(msg) {{
            const el = document.getElementById('errorMsg');
            el.textContent = msg;
            el.style.display = 'block';
            setTimeout(() => el.style.display = 'none', 5000);
        }}

        function showSuccess(msg) {{
            const el = document.getElementById('successMsg');
            el.textContent = msg;
            el.style.display = 'block';
            setTimeout(() => el.style.display = 'none', 3000);
        }}

        // Store rooms data for cross-referencing with users
        let roomsData = [];

        // Avatar helper
        const AVATAR_EMOJIS = ['', '\ud83c\udf85', '\ud83e\uddd1\u200d\ud83c\udf84', '\ud83e\udd8c', '\u26c4', '\ud83c\udf84', '\ud83c\udf81', '\ud83d\udd14', '\u2744\ufe0f', '\u2b50', '\ud83d\udd6f\ufe0f', '\ud83e\udddd', '\ud83e\udd34'];
        function getAvatarEmoji(avatarId) {{
            const id = parseInt(avatarId) || 1;
            const validId = Math.min(12, Math.max(1, id));
            return AVATAR_EMOJIS[validId] || AVATAR_EMOJIS[1];
        }}

        async function loadUsers() {{
            const res = await fetch('/auth/api/admin/users', {{
                credentials: 'include'
            }});
            const data = await res.json();
            if (data.ok && data.users) {{
                document.getElementById('usersTable').innerHTML = data.users.map(u => {{
                    // Find if user is in any room
                    const userRoom = u.current_room || null;
                    const roomBadge = userRoom
                        ? `<a href="/game?room=${{encodeURIComponent(userRoom)}}" class="badge badge-success" style="text-decoration:none;color:#fff;">${{userRoom}}</a>`
                        : '<span style="color:#666;">-</span>';
                    return `
                    <tr>
                        <td>${{u.id}}</td>
                        <td>${{u.email}}</td>
                        <td>${{u.display_name}}</td>
                        <td>${{roomBadge}}</td>
                        <td>${{u.verified ? '<span class="badge badge-success">Verified</span>' : '<span class="badge badge-warning">Unverified</span>'}}</td>
                        <td>${{u.is_admin ? '<span class="badge badge-success">Admin</span>' : '-'}}</td>
                        <td>
                            ${{!u.verified ? `<button class="btn btn-sm btn-secondary" onclick="verifyUser(${{u.id}})">Verify</button>` : ''}}
                            <button class="btn btn-sm btn-secondary" onclick="toggleAdmin(${{u.id}}, ${{!u.is_admin}})">${{u.is_admin ? 'Remove Admin' : 'Make Admin'}}</button>
                            ${{userRoom ? `<button class="btn btn-sm" style="background:#e67e22;" onclick="kickUser(${{u.id}}, '${{userRoom}}')">Kick</button>` : ''}}
                            <button class="btn btn-sm" style="background:#c0392b;" onclick="deleteUser(${{u.id}})">Delete</button>
                        </td>
                    </tr>
                `}}).join('');
            }}
        }}

        async function verifyUser(id) {{
            const res = await fetch(`/auth/api/admin/users/${{id}}/verify`, {{
                method: 'POST',
                credentials: 'include'
            }});
            if (res.ok) {{ showSuccess('User verified'); loadUsers(); }}
            else {{ showError('Failed to verify user'); }}
        }}

        async function toggleAdmin(id, makeAdmin) {{
            const res = await fetch(`/auth/api/admin/users/${{id}}/admin`, {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                    credentials: 'include',
                body: JSON.stringify({{ is_admin: makeAdmin }})
            }});
            if (res.ok) {{ showSuccess('Admin status updated'); loadUsers(); }}
            else {{ showError('Failed to update admin status'); }}
        }}

        async function deleteUser(id) {{
            if (!confirm('Delete this user?')) return;
            const res = await fetch(`/auth/api/admin/users/${{id}}`, {{
                method: 'DELETE',
                credentials: 'include'
            }});
            if (res.ok) {{ showSuccess('User deleted'); loadUsers(); }}
            else {{ showError('Failed to delete user'); }}
        }}

        async function loadPacks() {{
            const res = await fetch('/auth/api/admin/packs', {{
                credentials: 'include'
            }});
            const data = await res.json();
            if (data.ok && data.packs) {{
                packs = data.packs;
                document.getElementById('packsTable').innerHTML = data.packs.map(p => `
                    <tr>
                        <td>${{p.id}}</td>
                        <td>${{p.name}}</td>
                        <td>${{p.puzzle_count}}</td>
                        <td>
                            ${{p.id !== 1 ? `<button class="btn btn-sm" style="background:#c0392b;" onclick="deletePack(${{p.id}})">Delete</button>` : '<span style="color:#888;">Default</span>'}}
                        </td>
                    </tr>
                `).join('');
                // Update pack selects
                const options = data.packs.map(p => `<option value="${{p.id}}">${{p.name}}</option>`).join('');
                document.getElementById('puzzlePackSelect').innerHTML = '<option value="">All Packs</option>' + options;
                document.getElementById('newPuzzlePack').innerHTML = options;
                // Update settings pack selector
                document.getElementById('settingsPackId').innerHTML = '<option value="0">All Packs</option>' + options;
            }}
        }}

        async function createPack() {{
            const name = document.getElementById('newPackName').value.trim();
            if (!name) {{ showError('Enter a pack name'); return; }}
            const res = await fetch('/auth/api/admin/packs', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                    credentials: 'include',
                body: JSON.stringify({{ name }})
            }});
            if (res.ok) {{
                showSuccess('Pack created');
                document.getElementById('newPackName').value = '';
                loadPacks();
            }}
            else {{ showError('Failed to create pack'); }}
        }}

        async function deletePack(id) {{
            if (!confirm('Delete this pack and all its puzzles?')) return;
            const res = await fetch(`/auth/api/admin/packs/${{id}}`, {{
                method: 'DELETE',
                credentials: 'include'
            }});
            if (res.ok) {{ showSuccess('Pack deleted'); loadPacks(); loadPuzzles(); }}
            else {{ showError('Failed to delete pack'); }}
        }}

        async function importPack() {{
            const fileInput = document.getElementById('importFile');
            if (!fileInput.files || fileInput.files.length === 0) {{
                showError('Please select a JSON file to import');
                return;
            }}

            try {{
                const file = fileInput.files[0];
                const text = await file.text();
                const data = JSON.parse(text);

                // Support both single pack and multiple packs formats
                let packsToImport = [];
                if (data.packs && Array.isArray(data.packs)) {{
                    // Multiple packs format: {{ "packs": [{{ "name": "...", "puzzles": [...] }}, ...] }}
                    packsToImport = data.packs;
                }} else if (data.name && data.puzzles && Array.isArray(data.puzzles)) {{
                    // Single pack format: {{ "name": "...", "puzzles": [...] }}
                    packsToImport = [data];
                }} else {{
                    showError('Invalid file format. Expected {{ "name": "...", "puzzles": [...] }} or {{ "packs": [...] }}');
                    return;
                }}

                let totalImported = 0;
                let packsCreated = 0;

                for (const packData of packsToImport) {{
                    if (!packData.name || !packData.puzzles || !Array.isArray(packData.puzzles)) {{
                        console.warn('Skipping invalid pack:', packData);
                        continue;
                    }}

                    // Create the pack first
                    const packRes = await fetch('/auth/api/admin/packs', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'include',
                        body: JSON.stringify({{ name: packData.name }})
                    }});

                    if (!packRes.ok) {{
                        const err = await packRes.json();
                        console.warn(`Failed to create pack "${{packData.name}}":`, err.error);
                        continue;
                    }}

                    const pack = await packRes.json();
                    const packId = pack.pack?.id || pack.id;

                    // Import puzzles
                    const importRes = await fetch('/auth/api/admin/puzzles/import', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'include',
                        body: JSON.stringify({{
                            pack_id: packId,
                            puzzles: packData.puzzles.map(p => ({{
                                category: p.category,
                                answer: p.answer.toUpperCase()
                            }}))
                        }})
                    }});

                    if (importRes.ok) {{
                        totalImported += packData.puzzles.length;
                        packsCreated++;
                    }}
                }}

                if (packsCreated > 0) {{
                    showSuccess(`Imported ${{totalImported}} puzzles into ${{packsCreated}} pack(s)`);
                    fileInput.value = '';
                    loadPacks();
                    loadPuzzles();
                }} else {{
                    showError('Failed to import any packs');
                }}
            }} catch (e) {{
                console.error('Import error:', e);
                showError('Failed to parse JSON file: ' + e.message);
            }}
        }}

        async function loadPuzzles() {{
            const packId = document.getElementById('puzzlePackSelect').value;
            const url = packId ? `/auth/api/admin/puzzles?pack_id=${{packId}}` : '/auth/api/admin/puzzles';
            const res = await fetch(url, {{
                credentials: 'include'
            }});
            const data = await res.json();
            if (data.ok && data.puzzles) {{
                const packMap = {{}};
                packs.forEach(p => packMap[p.id] = p.name);
                document.getElementById('puzzlesTable').innerHTML = data.puzzles.map(p => `
                    <tr>
                        <td>${{p.id}}</td>
                        <td>${{p.category}}</td>
                        <td>${{p.answer}}</td>
                        <td>${{packMap[p.pack_id] || p.pack_id}}</td>
                        <td>
                            <button class="btn btn-sm" style="background:#c0392b;" onclick="deletePuzzle(${{p.id}})">Delete</button>
                        </td>
                    </tr>
                `).join('');
            }}
        }}

        async function addPuzzle() {{
            const category = document.getElementById('newPuzzleCategory').value.trim();
            const answer = document.getElementById('newPuzzleAnswer').value.trim().toUpperCase();
            const pack_id = parseInt(document.getElementById('newPuzzlePack').value);
            if (!category || !answer) {{ showError('Enter category and answer'); return; }}
            const res = await fetch('/auth/api/admin/puzzles', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                    credentials: 'include',
                body: JSON.stringify({{ category, answer, pack_id }})
            }});
            if (res.ok) {{
                showSuccess('Puzzle added');
                document.getElementById('newPuzzleCategory').value = '';
                document.getElementById('newPuzzleAnswer').value = '';
                loadPuzzles();
                loadPacks();
            }}
            else {{ showError('Failed to add puzzle'); }}
        }}

        async function deletePuzzle(id) {{
            if (!confirm('Delete this puzzle?')) return;
            const res = await fetch(`/auth/api/admin/puzzles/${{id}}`, {{
                method: 'DELETE',
                credentials: 'include'
            }});
            if (res.ok) {{ showSuccess('Puzzle deleted'); loadPuzzles(); loadPacks(); }}
            else {{ showError('Failed to delete puzzle'); }}
        }}

        async function loadRooms() {{
            const res = await fetch('/auth/api/admin/rooms', {{
                credentials: 'include'
            }});
            const data = await res.json();
            if (data.ok && data.rooms) {{
                roomsData = data.rooms;
                const container = document.getElementById('roomsContainer');

                // Create room button at the top
                let html = `
                    <div style="margin-bottom: 16px;">
                        <button class="btn" onclick="openCreateRoomModal()">+ Create New Room</button>
                    </div>
                `;

                if (data.rooms.length === 0) {{
                    html += '<div class="no-rooms">No active rooms</div>';
                    container.innerHTML = html;
                    return;
                }}

                html += data.rooms.map(r => {{
                    const players = r.players || [];
                    const activeIdx = r.active_idx ?? -1;
                    const activePlayer = activeIdx >= 0 && players[activeIdx] ? players[activeIdx].name : 'None';

                    const playerCards = players.length > 0 ? players.map((p, idx) => `
                        <div class="player-card ${{idx === activeIdx ? 'active' : ''}}">
                            <div class="player-details">
                                <span class="player-name-admin">
                                    <span class="player-avatar-admin">${{getAvatarEmoji(p.avatar_id)}}</span>
                                    <span class="connection-dot ${{p.is_connected ? 'connected' : 'disconnected'}}"></span>
                                    ${{p.name}}${{idx === activeIdx ? ' ▶' : ''}}
                                </span>
                                <span class="player-score-admin">Total: $${{(p.total || 0).toLocaleString()}} | Round: $${{(p.round_bank || 0).toLocaleString()}}</span>
                            </div>
                            <div class="player-actions">
                                <button class="btn btn-sm btn-secondary" onclick="resetPlayerScore('${{r.name}}', ${{idx}})">Reset</button>
                                <button class="btn btn-sm" style="background:#e67e22;" onclick="kickPlayer('${{r.name}}', ${{idx}})">Kick</button>
                            </div>
                        </div>
                    `).join('') : '<div style="color:#666;">No players in room</div>';

                    const roomId = r.name.replace(/[^a-zA-Z0-9]/g, '_');

                    return `
                        <div class="room-card">
                            <div class="room-header">
                                <div class="room-info">
                                    <span class="room-name">${{r.name}}</span>
                                    <div class="room-meta">
                                        <span>Phase: ${{r.phase || 'waiting'}}</span>
                                        <span>Players: ${{r.player_count || players.length}}</span>
                                        ${{r.has_host ? '<span class="badge badge-success">Host</span>' : ''}}
                                    </div>
                                </div>
                                <div class="room-actions">
                                    <button class="btn btn-sm btn-secondary" onclick="toggleRoomDetails('${{roomId}}')">Details</button>
                                    <button class="btn btn-sm btn-secondary" onclick="openSettingsModal('${{r.name}}')">Settings</button>
                                    <a href="/game?room=${{encodeURIComponent(r.name)}}&spectate=true" class="btn btn-sm" style="background:#3498db;" target="_blank">Spectate</a>
                                    <a href="/game?room=${{encodeURIComponent(r.name)}}" class="btn btn-sm" style="background:#27ae60;" target="_blank">Join</a>
                                    <button class="btn btn-sm btn-secondary" onclick="newGameInRoom('${{r.name}}')">New Game</button>
                                    <button class="btn btn-sm" style="background:#c0392b;" onclick="deleteRoom('${{r.name}}')">Delete</button>
                                </div>
                            </div>

                            <!-- Collapsible Details Section -->
                            <div class="room-details" id="details-${{roomId}}">
                                <div class="room-details-grid">
                                    <div class="detail-item">
                                        <div class="detail-label">Active Turn</div>
                                        <div class="detail-value gold">${{activePlayer}}</div>
                                    </div>
                                    <div class="detail-item">
                                        <div class="detail-label">Current Wedge</div>
                                        <div class="detail-value green">${{r.current_wedge || '--'}}</div>
                                    </div>
                                    <div class="detail-item">
                                        <div class="detail-label">Letters Revealed</div>
                                        <div class="detail-value">${{r.revealed_count}} / ${{r.total_letters}}</div>
                                    </div>
                                </div>

                                ${{r.puzzle_category ? `
                                    <div class="puzzle-preview">
                                        <div class="puzzle-preview-label">${{r.puzzle_category}}</div>
                                        <div class="puzzle-preview-answer">${{r.puzzle_answer}}</div>
                                    </div>
                                ` : ''}}

                                <h4 style="color: #888; margin: 16px 0 12px 0; font-size: 14px; text-transform: uppercase;">Players</h4>
                                <div class="player-grid">
                                    ${{playerCards}}
                                </div>
                            </div>
                        </div>
                    `;
                }}).join('');

                container.innerHTML = html;
                updateSettingsPackSelect();
            }}
        }}

        async function deleteRoom(name) {{
            if (!confirm('Delete this room?')) return;
            const res = await fetch(`/auth/api/admin/rooms/${{encodeURIComponent(name)}}`, {{
                method: 'DELETE',
                credentials: 'include'
            }});
            if (res.ok) {{ showSuccess('Room deleted'); loadRooms(); }}
            else {{ showError('Failed to delete room'); }}
        }}

        async function kickPlayer(room, playerIdx) {{
            if (!confirm('Kick this player from the room?')) return;
            const res = await fetch(`/auth/api/admin/rooms/${{encodeURIComponent(room)}}/players/${{playerIdx}}`, {{
                method: 'DELETE',
                credentials: 'include'
            }});
            if (res.ok) {{ showSuccess('Player kicked'); loadRooms(); loadUsers(); }}
            else {{ showError('Failed to kick player'); }}
        }}

        async function kickUser(userId, room) {{
            if (!confirm('Kick this user from the room?')) return;
            const res = await fetch(`/auth/api/admin/rooms/${{encodeURIComponent(room)}}/kick-user/${{userId}}`, {{
                method: 'POST',
                credentials: 'include'
            }});
            if (res.ok) {{ showSuccess('User kicked'); loadRooms(); loadUsers(); }}
            else {{ showError('Failed to kick user'); }}
        }}

        async function resetPlayerScore(room, playerIdx) {{
            if (!confirm("Reset this player's score to $0?")) return;
            const res = await fetch(`/auth/api/admin/rooms/${{encodeURIComponent(room)}}/players/${{playerIdx}}/reset`, {{
                method: 'POST',
                credentials: 'include'
            }});
            if (res.ok) {{ showSuccess('Score reset'); loadRooms(); }}
            else {{ showError('Failed to reset score'); }}
        }}

        async function newGameInRoom(room) {{
            if (!confirm('Start a new game in this room? Current game will be ended.')) return;
            const res = await fetch(`/auth/api/admin/rooms/${{encodeURIComponent(room)}}/new-game`, {{
                method: 'POST',
                credentials: 'include'
            }});
            if (res.ok) {{ showSuccess('New game started'); loadRooms(); }}
            else {{ showError('Failed to start new game'); }}
        }}

        // Room details toggle
        function toggleRoomDetails(roomId) {{
            const details = document.getElementById('details-' + roomId);
            if (details) {{
                details.classList.toggle('expanded');
            }}
        }}

        // Settings modal functions
        function openSettingsModal(room) {{
            document.getElementById('settingsRoom').value = room;
            document.getElementById('settingsModalTitle').textContent = `Settings: ${{room}}`;
            document.getElementById('settingsModal').style.display = 'flex';
            loadRoomSettings(room);
        }}

        function closeSettingsModal() {{
            document.getElementById('settingsModal').style.display = 'none';
        }}

        async function loadRoomSettings(room) {{
            if (!room) room = document.getElementById('settingsRoom').value;
            try {{
                const res = await fetch(`/auth/api/admin/settings/${{encodeURIComponent(room)}}`, {{
                    credentials: 'include'
                }});
                const data = await res.json();
                if (data.ok && data.config) {{
                    document.getElementById('settingsPuzzleDisplay').value = data.config.puzzle_display_seconds || 30;
                    document.getElementById('settingsVowelCost').value = data.config.vowel_cost || 250;
                    document.getElementById('settingsFinalSeconds').value = data.config.final_seconds || 30;
                    document.getElementById('settingsFinalJackpot').value = data.config.final_jackpot || 10000;
                    document.getElementById('settingsPrizeWedges').value = (data.config.prize_wedge_names || ['GIFT CARD']).join(', ');
                    document.getElementById('settingsPackId').value = data.config.pack_id || 0;
                    document.getElementById('settingsDisconnectTimeout').value = data.config.disconnect_timeout_secs ?? 300;
                    document.getElementById('settingsTurnTimer').value = data.config.turn_timer_seconds ?? 10;
                    document.getElementById('settingsBuzzTimer').value = data.config.buzz_timer_seconds ?? 5;
                }}
            }} catch (e) {{
                console.error('Failed to load settings:', e);
            }}
        }}

        async function saveSettings() {{
            const room = document.getElementById('settingsRoom').value;
            const packId = parseInt(document.getElementById('settingsPackId').value) || 0;
            const config = {{
                puzzle_display_seconds: parseInt(document.getElementById('settingsPuzzleDisplay').value) || 30,
                vowel_cost: parseInt(document.getElementById('settingsVowelCost').value) || 250,
                final_seconds: parseInt(document.getElementById('settingsFinalSeconds').value) || 30,
                final_jackpot: parseInt(document.getElementById('settingsFinalJackpot').value) || 10000,
                prize_wedge_names: document.getElementById('settingsPrizeWedges').value.split(',').map(s => s.trim()).filter(s => s),
                pack_id: packId > 0 ? packId : null,
                disconnect_timeout_secs: parseInt(document.getElementById('settingsDisconnectTimeout').value) || 300,
                turn_timer_seconds: parseInt(document.getElementById('settingsTurnTimer').value) ?? 10,
                buzz_timer_seconds: parseInt(document.getElementById('settingsBuzzTimer').value) ?? 5
            }};
            try {{
                const res = await fetch(`/auth/api/admin/settings/${{encodeURIComponent(room)}}`, {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    credentials: 'include',
                    body: JSON.stringify(config)
                }});
                if (res.ok) {{
                    showSuccess('Settings saved');
                    closeSettingsModal();
                }} else {{
                    showError('Failed to save settings');
                }}
            }} catch (e) {{
                showError('Failed to save settings');
            }}
        }}

        function updateSettingsPackSelect() {{
            // Update the pack select in settings modal
            const select = document.getElementById('settingsPackId');
            if (!select) return;
            const currentValue = select.value;
            select.innerHTML = '<option value="0">All Packs</option>';
            packs.forEach(p => {{
                const option = document.createElement('option');
                option.value = p.id;
                option.textContent = p.name;
                select.appendChild(option);
            }});
            if (currentValue) {{
                select.value = currentValue;
            }}
        }}

        // Create room modal functions
        function openCreateRoomModal() {{
            document.getElementById('newRoomName').value = '';
            document.getElementById('createRoomModal').style.display = 'flex';
        }}

        function closeCreateRoomModal() {{
            document.getElementById('createRoomModal').style.display = 'none';
        }}

        async function createRoom() {{
            const roomName = document.getElementById('newRoomName').value.trim();
            if (!roomName) {{
                showError('Please enter a room name');
                return;
            }}
            try {{
                const res = await fetch('/auth/api/admin/rooms', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    credentials: 'include',
                    body: JSON.stringify({{ name: roomName }})
                }});
                if (res.ok) {{
                    showSuccess(`Room "${{roomName}}" created`);
                    closeCreateRoomModal();
                    loadRooms();
                }} else {{
                    const data = await res.json();
                    showError(data.error || 'Failed to create room');
                }}
            }} catch (e) {{
                showError('Failed to create room');
            }}
        }}

        // ========== DATABASE BROWSER FUNCTIONS ==========
        let dbCurrentPage = 1;

        async function loadTables() {{
            try {{
                const res = await fetch('/auth/api/admin/database/tables', {{
                    credentials: 'include'
                }});
                const data = await res.json();
                if (data.ok && data.tables) {{
                    const select = document.getElementById('dbTableSelect');
                    select.innerHTML = '<option value="">-- Select a table --</option>';
                    data.tables.forEach(table => {{
                        const option = document.createElement('option');
                        option.value = table;
                        option.textContent = table;
                        select.appendChild(option);
                    }});
                }}
            }} catch (e) {{
                console.error('Failed to load tables:', e);
            }}
        }}

        async function loadTableData(page = 1) {{
            const tableName = document.getElementById('dbTableSelect').value;
            if (!tableName) {{
                document.getElementById('dbTableHead').innerHTML = '';
                document.getElementById('dbTableBody').innerHTML = '';
                document.getElementById('dbTableInfo').textContent = '';
                document.getElementById('dbPagination').innerHTML = '';
                return;
            }}

            dbCurrentPage = page;
            const pageSize = document.getElementById('dbPageSize').value;

            try {{
                const res = await fetch(`/auth/api/admin/database/tables/${{tableName}}?page=${{page}}&page_size=${{pageSize}}`, {{
                    credentials: 'include'
                }});
                const data = await res.json();
                if (data.ok) {{
                    renderTableData(data);
                }} else {{
                    showError('Failed to load table data');
                }}
            }} catch (e) {{
                console.error('Failed to load table data:', e);
                showError('Failed to load table data');
            }}
        }}

        function renderTableData(data) {{
            const thead = document.getElementById('dbTableHead');
            const tbody = document.getElementById('dbTableBody');
            const info = document.getElementById('dbTableInfo');
            const pagination = document.getElementById('dbPagination');

            // Render header
            thead.innerHTML = '<tr>' + data.columns.map(col => `<th>${{col}}</th>`).join('') + '</tr>';

            // Render rows
            if (data.rows.length === 0) {{
                tbody.innerHTML = '<tr><td colspan="' + data.columns.length + '" style="text-align:center;color:#888;padding:24px;">No data</td></tr>';
            }} else {{
                tbody.innerHTML = data.rows.map(row => {{
                    return '<tr>' + data.columns.map(col => {{
                        let val = row[col];
                        if (val === null || val === undefined) {{
                            return '<td style="color:#666;">NULL</td>';
                        }}
                        // Truncate long values
                        let display = String(val);
                        if (display.length > 100) {{
                            display = display.substring(0, 100) + '...';
                        }}
                        // Escape HTML
                        display = display.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
                        return `<td>${{display}}</td>`;
                    }}).join('') + '</tr>';
                }}).join('');
            }}

            // Info
            const totalPages = Math.ceil(data.total_count / data.page_size);
            const startRow = (data.page - 1) * data.page_size + 1;
            const endRow = Math.min(data.page * data.page_size, data.total_count);
            info.textContent = `Table: ${{data.table}} | Showing ${{startRow}}-${{endRow}} of ${{data.total_count}} rows | Page ${{data.page}} of ${{totalPages}}`;

            // Pagination
            let paginationHtml = '';
            if (data.page > 1) {{
                paginationHtml += `<button class="btn btn-sm btn-secondary" onclick="loadTableData(1)">First</button>`;
                paginationHtml += `<button class="btn btn-sm btn-secondary" onclick="loadTableData(${{data.page - 1}})">Prev</button>`;
            }}
            paginationHtml += `<span style="color:#888;margin:0 12px;">Page ${{data.page}} / ${{totalPages}}</span>`;
            if (data.page < totalPages) {{
                paginationHtml += `<button class="btn btn-sm btn-secondary" onclick="loadTableData(${{data.page + 1}})">Next</button>`;
                paginationHtml += `<button class="btn btn-sm btn-secondary" onclick="loadTableData(${{totalPages}})">Last</button>`;
            }}
            pagination.innerHTML = paginationHtml;
        }}

        // Override showTab to load tables when switching to database tab
        const originalShowTab = showTab;
        function showTab(tab) {{
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
            document.querySelector(`[onclick="showTab('${{tab}}')"]`).classList.add('active');
            document.getElementById('panel-' + tab).classList.add('active');

            // Load tables when switching to database tab
            if (tab === 'database') {{
                loadTables();
            }}
        }}

        // Initial load
        checkAdmin();
    </script>
</body>
</html>"#,
        common_styles = COMMON_STYLES
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use axum::{body::Body, http::Request, routing::get, Router};
    use tokio::sync::{OnceCell, RwLock};
    use tower::ServiceExt;

    use crate::config::Config;
    use crate::db::Database;
    use crate::email::EmailService;
    use crate::game::GameManager;

    /// Create a test app state with a temporary database
    async fn create_test_state() -> Arc<AppState> {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        std::mem::forget(tmp); // Keep temp file alive

        let db = Database::new(&path).await.unwrap();
        let email = EmailService::from_env();
        let game_manager = GameManager::new();
        // Set valid HOST_CODE for tests (required since HOST_CODE security hardening)
        std::env::set_var("HOST_CODE", "test-secure-code-12345");
        let config = Config::from_env_no_dotenv().unwrap();

        Arc::new(AppState {
            game_manager: RwLock::new(game_manager),
            db,
            email,
            io: OnceCell::new(),
            user_sockets: RwLock::new(HashMap::new()),
            config,
            claim_host_limits: RwLock::new(HashMap::new()),
        })
    }

    #[tokio::test]
    async fn test_health_returns_ok_when_healthy() {
        let state = create_test_state().await;

        let app = Router::new()
            .route("/health", get(health))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "healthy");
        assert_eq!(json["checks"]["database"], "ok");
        assert!(json["checks"]["uptime_seconds"].as_u64().is_some());
        assert!(json["version"].is_string());
    }

    #[tokio::test]
    async fn test_health_response_structure() {
        let state = create_test_state().await;

        let app = Router::new()
            .route("/health", get(health))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Verify response structure
        assert!(json.get("status").is_some(), "Missing 'status' field");
        assert!(json.get("version").is_some(), "Missing 'version' field");
        assert!(json.get("checks").is_some(), "Missing 'checks' field");

        let checks = json.get("checks").unwrap();
        assert!(checks.get("database").is_some(), "Missing 'database' check");
        assert!(
            checks.get("uptime_seconds").is_some(),
            "Missing 'uptime_seconds'"
        );
    }

    #[tokio::test]
    async fn test_health_uptime_increases() {
        let state = create_test_state().await;

        let app = Router::new()
            .route("/health", get(health))
            .with_state(state.clone());

        // First request
        let response1 = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX)
            .await
            .unwrap();
        let json1: serde_json::Value = serde_json::from_slice(&body1).unwrap();
        let uptime1 = json1["checks"]["uptime_seconds"].as_u64().unwrap();

        // Sleep briefly
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Second request
        let response2 = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
            .await
            .unwrap();
        let json2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
        let uptime2 = json2["checks"]["uptime_seconds"].as_u64().unwrap();

        // Uptime should be >= first measurement (may be same if < 1 second elapsed)
        assert!(uptime2 >= uptime1, "Uptime should not decrease");
    }
}
