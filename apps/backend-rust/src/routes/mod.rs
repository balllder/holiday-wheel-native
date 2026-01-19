use axum::extract::Query;
use axum::response::Html;
use axum::Json;
use serde::{Deserialize, Serialize};

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

/// Health check endpoint
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Join query parameters
#[derive(Deserialize)]
pub struct JoinQuery {
    room: Option<String>,
}

/// Universal link join page - tries app first, falls back to web
pub async fn join(Query(query): Query<JoinQuery>) -> Html<String> {
    let room = query.room.unwrap_or_else(|| "main".to_string());
    let room_escaped = room.replace('\"', "&quot;").replace('<', "&lt;").replace('>', "&gt;");

    Html(format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Join Holiday Wheel</title>
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
            color: #d4af37;
            margin-bottom: 16px;
            font-size: 28px;
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
</html>"#, room_escaped = room_escaped, server = "{server}"))
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
    h1 {
        color: #d4af37;
        text-align: center;
        margin-bottom: 8px;
        font-size: 28px;
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
    Html(format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Login</title>
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
</html>"##, common_styles = COMMON_STYLES, google_client_id = google_client_id))
}

/// Register page
pub async fn register() -> Html<String> {
    Html(format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Register</title>
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
    </style>
</head>
<body>
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

        document.getElementById('registerForm').addEventListener('submit', async (e) => {{
            e.preventDefault();
            const errorDiv = document.getElementById('error');
            errorDiv.style.display = 'none';

            const display_name = document.getElementById('displayName').value;
            const email = document.getElementById('email').value;
            const password = document.getElementById('password').value;
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
                    body: JSON.stringify({{ email, password, display_name }}),
                    credentials: 'include'
                }});
                const data = await res.json();

                if (res.ok && data.user) {{
                    localStorage.setItem('user', JSON.stringify(data.user));
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
</html>"##, common_styles = COMMON_STYLES))
}

/// Lobby page
pub async fn lobby() -> Html<String> {
    Html(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Lobby</title>
    <style>
        {common_styles}
        .lobby-header {{
            text-align: center;
            margin-bottom: 24px;
            padding-bottom: 16px;
            border-bottom: 1px solid #333;
        }}
        .lobby-header h1 {{
            font-size: 36px;
            margin-bottom: 8px;
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
        }}
        .hidden {{ display: none !important; }}
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
    <div class="container wide">
        <div class="lobby-header">
            <h1>🎡 Holiday Wheel</h1>
            <div class="user-row">
                <span>Welcome, <span class="user-name" id="userName">Player</span>!</span>
                <div class="header-buttons">
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
            }}
        }});
    </script>
</body>
</html>"#, common_styles = COMMON_STYLES))
}

/// Game page
pub async fn game() -> Html<String> {
    Html(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Game</title>
    <style>
        {common_styles}
        body {{ align-items: flex-start; padding: 20px; }}

        /* ========== MODERN THEME COLORS ========== */
        :root {{
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
                0 0 20px var(--color-primary-glow),
                inset 0 2px 4px rgba(255, 255, 255, 0.1);
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
        }}
        .letter-tile.hidden {{
            background: linear-gradient(180deg, #ffffff 0%, #e8e8e8 100%);
            color: transparent;
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
        }}

        /* Letter reveal animation */
        .letter-tile.revealing {{
            animation: letterReveal 0.6s ease-out forwards;
        }}
        @keyframes letterReveal {{
            0% {{ transform: rotateY(90deg); opacity: 0.5; }}
            50% {{ transform: rotateY(-10deg); }}
            100% {{ transform: rotateY(0deg); opacity: 1; box-shadow: 0 0 20px var(--color-primary-glow); }}
        }}
        .letter-tile.just-revealed {{
            box-shadow: 0 0 15px var(--color-primary-glow), 0 3px 6px rgba(0,0,0,0.3);
        }}
        .category {{ color: #d4af37; text-align: center; font-size: 18px; margin-bottom: 10px; }}
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
        }}
        .wheel-container {{
            position: relative;
            display: flex;
            flex-direction: column;
            align-items: center;
            width: min(320px, 25vw);
            height: min(320px, 25vw);
            min-width: 200px;
            min-height: 200px;
        }}
        .wheel-pointer {{
            position: absolute;
            top: -8px;
            z-index: 10;
            width: 0;
            height: 0;
            border-left: 15px solid transparent;
            border-right: 15px solid transparent;
            border-top: 25px solid #d4af37;
            filter: drop-shadow(0 2px 3px rgba(0,0,0,0.5));
        }}
        .wheel-svg {{
            width: 100%;
            height: 100%;
            filter: drop-shadow(0 4px 8px rgba(0,0,0,0.4));
        }}
        .wheel-result {{
            font-size: clamp(20px, 2vw, 32px);
            color: #d4af37;
            margin-top: 12px;
            text-shadow: 0 2px 4px rgba(0,0,0,0.3);
            text-align: center;
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
                width: min(280px, 60vw);
                height: min(280px, 60vw);
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
        .player-name {{ color: #fff; font-weight: 500; }}
        .player-score {{
            color: var(--color-primary);
            font-weight: bold;
            font-size: 18px;
            position: relative;
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
        .guess-input {{
            display: flex;
            gap: 10px;
            margin-top: 20px;
        }}
        .guess-input input {{ flex: 1; text-transform: uppercase; }}
        .notification {{
            background: #d4af37;
            color: #0d0628;
            padding: 12px;
            border-radius: 8px;
            text-align: center;
            margin-bottom: 16px;
            display: none;
        }}
        .phase-indicator {{
            text-align: center;
            color: #888;
            margin-bottom: 10px;
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
            color: #d4af37;
            margin: 0;
            font-size: 32px;
        }}
        .game-header .leave-btn {{
            position: absolute;
            right: 0;
            top: 50%;
            transform: translateY(-50%);
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

        /* ========== PHASE TRANSITION OVERLAY ========== */
        .phase-overlay {{
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0, 0, 0, 0.9);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 5000;
            opacity: 0;
            pointer-events: none;
            transition: opacity 0.5s ease;
        }}
        .phase-overlay.active {{
            opacity: 1;
            pointer-events: auto;
        }}
        .phase-content {{
            text-align: center;
            animation: phaseZoom 0.5s ease-out;
        }}
        .phase-title {{
            font-size: 48px;
            font-weight: bold;
            color: var(--color-primary);
            text-shadow: 0 0 30px var(--color-primary-glow);
            margin-bottom: 16px;
        }}
        .phase-subtitle {{
            font-size: 24px;
            color: #fff;
            opacity: 0.8;
        }}
        @keyframes phaseZoom {{
            0% {{ transform: scale(0.5); opacity: 0; }}
            100% {{ transform: scale(1); opacity: 1; }}
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
            border: 3px solid var(--color-primary);
            border-radius: 16px;
            padding: 16px 24px;
            text-align: center;
            margin-bottom: 16px;
            box-shadow: 0 0 20px rgba(239, 68, 68, 0.4);
        }}
        .tossup-display.active {{
            display: block;
            animation: tossupPulse 0.5s ease-out;
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
    </style>
</head>
<body>
    <!-- Confetti Container -->
    <div id="confetti-container"></div>

    <!-- Phase Transition Overlay -->
    <div class="phase-overlay" id="phaseOverlay">
        <div class="phase-content">
            <div class="phase-title" id="phaseTitle">TOSS-UP!</div>
            <div class="phase-subtitle" id="phaseSubtitle">Buzz in to answer!</div>
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
                <a href="/lobby" class="btn btn-secondary leave-btn">Leave Room</a>
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

            <div class="notification" id="notification"></div>
            <div class="phase-indicator">Phase: <span id="phase">Connecting...</span></div>

            <div class="game-layout">
                <div class="wheel-area">
                    <div class="wheel-container">
                        <div class="wheel-pointer"></div>
                        <svg id="wheelSvg" class="wheel-svg" width="280" height="280" viewBox="0 0 280 280"></svg>
                    </div>
                    <div class="wheel-result" id="wheelResult">Spin!</div>
                </div>

                <div class="puzzle-section">
                    <div class="category">Category: <span id="category">-</span></div>
                    <div class="puzzle-board" id="puzzleBoard">
                        <p style="color: #fff;">Connecting to game...</p>
                    </div>
                </div>
            </div>

            <div class="controls" id="controls">
                <button class="btn" id="spinBtn" onclick="spin()">Spin</button>
                <button class="btn btn-secondary" id="buyVowelBtn" onclick="buyVowel()">Buy Vowel ($250)</button>
                <button class="btn btn-secondary" id="solveBtn" onclick="promptSolve()">Solve</button>
                <button class="btn wildcard-btn" id="wildcardBtn" onclick="useWildCard()">
                    <span class="icon">🃏</span> Wild Card
                </button>
                <button class="btn btn-danger" id="buzzBtn" onclick="buzz()" style="display: none;">🔔 BUZZ IN!</button>
            </div>

            <div class="guess-input" id="guessArea">
                <input type="text" id="letterInput" maxlength="1" placeholder="Guess a letter">
                <button class="btn" onclick="guessLetter()">Guess</button>
            </div>

            <div class="host-controls" id="hostControls" style="display: none; margin-top: 20px; padding-top: 16px; border-top: 1px solid #333;">
                <p style="color: #d4af37; margin-bottom: 12px; font-weight: bold;">Host Controls</p>
                <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                    <button class="btn" onclick="newGame()">New Game</button>
                    <button class="btn btn-secondary" onclick="revealAll()">Reveal All</button>
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
            </div>
        </div>
    </div>

    <!-- Solve Modal -->
    <div class="modal-overlay" id="solveModal">
        <div class="modal">
            <h2>Solve the Puzzle</h2>
            <input type="text" id="solveInput" placeholder="Enter your solution" autocomplete="off">
            <div class="modal-buttons">
                <button class="btn" onclick="submitSolve()">Submit</button>
                <button class="btn btn-secondary" onclick="closeModal('solveModal')">Cancel</button>
            </div>
        </div>
    </div>

    <!-- Buy Vowel Modal -->
    <div class="modal-overlay" id="vowelModal">
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

        const urlParams = new URLSearchParams(window.location.search);
        const room = urlParams.get('room') || 'main';
        document.getElementById('roomName').textContent = room;

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
        let prevScores = {{}};

        // ========== SOUND EFFECTS ==========
        const SoundService = {{
            enabled: true,
            sounds: {{}},
            audioContext: null,

            init() {{
                try {{
                    this.audioContext = new (window.AudioContext || window.webkitAudioContext)();
                }} catch (e) {{
                    console.log('Web Audio API not supported');
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
            solve() {{
                [523, 659, 784, 1047].forEach((f, i) => {{
                    setTimeout(() => this.playTone(f, 0.2, 'sine'), i * 100);
                }});
            }},
            buzz() {{ this.playTone(440, 0.1, 'square'); }},
        }};

        // ========== CONFETTI SYSTEM ==========
        function launchConfetti(count = 100) {{
            const container = document.getElementById('confetti-container');
            const colors = ['#d4af37', '#ffd700', '#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4', '#ff8c00'];

            for (let i = 0; i < count; i++) {{
                setTimeout(() => {{
                    const confetti = document.createElement('div');
                    confetti.className = 'confetti';
                    confetti.style.left = Math.random() * 100 + 'vw';
                    confetti.style.background = colors[Math.floor(Math.random() * colors.length)];
                    confetti.style.animationDuration = (2 + Math.random() * 2) + 's';
                    confetti.style.animationDelay = Math.random() * 0.5 + 's';
                    container.appendChild(confetti);

                    setTimeout(() => confetti.remove(), 4000);
                }}, i * 20);
            }}
        }}

        // ========== PHASE TRANSITIONS ==========
        function showPhaseTransition(phase) {{
            const overlay = document.getElementById('phaseOverlay');
            const title = document.getElementById('phaseTitle');
            const subtitle = document.getElementById('phaseSubtitle');

            const phases = {{
                'tossup': {{ title: 'TOSS-UP!', subtitle: 'Buzz in to answer!' }},
                'final': {{ title: 'BONUS ROUND!', subtitle: 'Pick your letters wisely...' }},
                'normal': {{ title: 'SPIN THE WHEEL!', subtitle: 'Good luck!' }},
            }};

            const config = phases[phase] || {{ title: phase.toUpperCase(), subtitle: '' }};
            title.textContent = config.title;
            subtitle.textContent = config.subtitle;

            overlay.classList.add('active');
            setTimeout(() => overlay.classList.remove('active'), 2000);
        }}

        // ========== SCORE CHANGE ANIMATION ==========
        function showScoreChange(playerIdx, amount) {{
            const playerEl = document.querySelectorAll('.player')[playerIdx];
            if (!playerEl) return;

            const change = document.createElement('span');
            change.className = 'score-change ' + (amount >= 0 ? 'positive' : 'negative');
            change.textContent = (amount >= 0 ? '+' : '') + '$' + Math.abs(amount).toLocaleString();

            const scoreEl = playerEl.querySelector('.player-score');
            if (scoreEl) {{
                scoreEl.style.position = 'relative';
                scoreEl.appendChild(change);
                setTimeout(() => change.remove(), 1500);
            }}
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
                svg.innerHTML = "<text x='140' y='140' text-anchor='middle' fill='#888'>No wheel data</text>";
                return;
            }}

            const size = 280;
            const radius = size / 2 - 8;
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

                // Dynamic font size based on label length and number of slots
                const baseSize = numSlots > 20 ? 11 : numSlots > 16 ? 12 : 14;
                let fontSize = baseSize;
                if (label.length > 10) fontSize = baseSize - 4;
                else if (label.length > 7) fontSize = baseSize - 2;
                else if (label.length > 5) fontSize = baseSize - 1;

                // Use white text with black stroke for visibility on any background
                const textFill = (isBankrupt || color === '#0047ab' || color === '#9932cc' || color === '#9400d3' || color === '#8a2be2') ? '#fff' : (isLoseTurn ? '#000' : '#fff');
                const strokeColor = textFill === '#fff' ? '#000' : '#fff';

                html += "<text x='" + textX + "' y='" + textY + "' fill='" + textFill + "' stroke='" + strokeColor + "' stroke-width='0.5' font-size='" + fontSize + "' font-weight='bold' text-anchor='middle' dominant-baseline='middle' transform='rotate(" + rotation + ", " + textX + ", " + textY + ")' style='paint-order: stroke fill'>" + label + "</text>";
            }});

            // Center hub with gradient effect
            html += "<circle cx='" + centerX + "' cy='" + centerY + "' r='22' fill='#2a2a2a' stroke='#d4af37' stroke-width='4'/>";
            html += "<circle cx='" + centerX + "' cy='" + centerY + "' r='12' fill='#d4af37'/>";
            html += "<circle cx='" + centerX + "' cy='" + centerY + "' r='6' fill='#fff' opacity='0.3'/>";

            svg.innerHTML = html;
        }}

        function animateWheelTo(targetIdx, slots) {{
            if (wheelAnimationId) {{
                cancelAnimationFrame(wheelAnimationId);
            }}

            isWheelSpinning = true;
            document.getElementById('wheelResult').textContent = 'Spinning...';

            const numSlots = slots.length;
            const anglePerSlot = 360 / numSlots;

            // Calculate the angle where the target wedge center should be at top (pointer position)
            // Wedges are drawn starting at -90 degrees, so wedge N's center is at -90 + N*anglePerSlot + anglePerSlot/2
            // To bring that to the pointer (top, -90 degrees), we rotate by -(N*anglePerSlot + anglePerSlot/2)
            const wedgeCenterAngle = targetIdx * anglePerSlot + anglePerSlot / 2;
            const finalAngle = (360 - wedgeCenterAngle) % 360;  // equivalent to -wedgeCenterAngle in positive form

            const currentAngle = ((wheelRotation % 360) + 360) % 360;  // normalize to 0-360
            let delta = (finalAngle - currentAngle + 360) % 360;
            if (delta < 30) delta += 360;  // ensure visible rotation on last partial spin

            const spins = 3;  // full spins before landing
            const targetRotation = wheelRotation + spins * 360 + delta;

            const startRotation = wheelRotation;
            const totalDelta = targetRotation - startRotation;
            const duration = 3000;
            const startTime = performance.now();

            function animate(currentTime) {{
                const elapsed = currentTime - startTime;
                const progress = Math.min(elapsed / duration, 1);
                // Ease out cubic for natural deceleration
                const eased = 1 - Math.pow(1 - progress, 3);
                wheelRotation = startRotation + totalDelta * eased;

                document.getElementById('wheelSvg').style.transform = `rotate(${{wheelRotation}}deg)`;

                if (progress < 1) {{
                    wheelAnimationId = requestAnimationFrame(animate);
                }} else {{
                    wheelAnimationId = null;
                    isWheelSpinning = false;
                    onWheelStopped();
                }}
            }}

            wheelAnimationId = requestAnimationFrame(animate);
        }}

        function onWheelStopped() {{
            // Show the wheel result
            if (pendingWheelResult !== null) {{
                document.getElementById('wheelResult').textContent = pendingWheelResult;
                pendingWheelResult = null;
            }}
            // Show any pending toasts
            while (pendingToasts.length > 0) {{
                const msg = pendingToasts.shift();
                showNotification(msg);
            }}
        }}

        function connect() {{
            socket = io(window.location.origin, {{ transports: ['websocket'] }});

            socket.on('connect', () => {{
                console.log('Connected:', socket.id);
                socket.emit('join_game', {{ room, name: user.display_name || user.email }});
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

            // ========== PHASE TRANSITION ==========
            if (prevPhase && prevPhase !== phase) {{
                showPhaseTransition(phase);
            }}
            prevPhase = phase;

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
            document.getElementById('phase').textContent = phase;

            // Category
            document.getElementById('category').textContent = gameState.puzzle?.category || '-';

            // ========== ROUND PROGRESS ==========
            const roundState = gameState.round;
            const roundIndicator = document.getElementById('roundIndicator');
            if (roundState && roundState.enabled && roundState.total_rounds > 0) {{
                roundIndicator.classList.add('active');
                document.getElementById('roundNumber').textContent = 'ROUND ' + roundState.current_round;
                document.getElementById('roundTotal').textContent = 'of ' + roundState.total_rounds;

                // Render round dots
                let dotsHtml = '';
                for (let i = 1; i <= roundState.total_rounds; i++) {{
                    const dotClass = i < roundState.current_round ? 'completed' :
                                    i === roundState.current_round ? 'current' : '';
                    dotsHtml += `<div class="round-dot ${{dotClass}}"></div>`;
                }}
                document.getElementById('roundDots').innerHTML = dotsHtml;

                // Round badges
                let badgesHtml = '';
                const currentConfig = roundState.rounds?.[roundState.current_round - 1];
                if (currentConfig) {{
                    if (currentConfig.type && currentConfig.type !== 'normal') {{
                        const typeLabels = {{ tossup: 'TOSS-UP', speed: 'SPEED', bonus: 'BONUS' }};
                        badgesHtml += `<span class="round-badge type">${{typeLabels[currentConfig.type] || currentConfig.type.toUpperCase()}}</span>`;
                    }}
                    if (currentConfig.value_multiplier > 1) {{
                        badgesHtml += `<span class="round-badge multiplier">${{currentConfig.value_multiplier}}x</span>`;
                    }}
                }}
                document.getElementById('roundBadges').innerHTML = badgesHtml;
            }} else {{
                roundIndicator.classList.remove('active');
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

            // ========== PUZZLE BOARD WITH LETTER ANIMATIONS ==========
            const board = document.getElementById('puzzleBoard');
            const ROW_SIZES = [12, 14, 14, 12];
            const currentRevealed = new Set(gameState.revealed || []);
            const newlyRevealed = [...currentRevealed].filter(c => !prevRevealed.has(c));

            if (gameState.puzzle?.answer) {{
                const answer = gameState.puzzle.answer.toUpperCase();
                const words = answer.split(' ');

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
                                const animClass = isNew ? 'revealing' : 'just-revealed';
                                const delay = isNew ? `animation-delay: ${{revealDelay * 0.15}}s` : '';
                                if (isNew) revealDelay++;
                                html += `<div class="letter-tile revealed ${{animClass}}" style="${{delay}}">${{char}}</div>`;
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
                    SoundService.letterCorrect();
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
                    return `
                        <div class="player ${{idx === gameState.active_idx ? 'active' : ''}}">
                            <span class="player-name">${{p.name}}${{idx === myPlayerIdx ? ' (you)' : ''}}${{wildcardHtml}}</span>
                            <span class="player-score">${{(p.total || 0) + (p.round_bank || 0)}}</span>
                        </div>
                    `;
                }}).join('');
            }}

            // ========== CONTROLS VISIBILITY ==========
            const isMyTurn = gameState.active_idx === myPlayerIdx;
            const isTossup = phase === 'tossup';
            const canBuzz = isTossup && myPlayerIdx !== null &&
                !(gameState.tossup?.locked_player_idxs || []).includes(myPlayerIdx);

            // Normal controls
            document.getElementById('spinBtn').disabled = !isMyTurn || isTossup;
            document.getElementById('spinBtn').style.display = isTossup ? 'none' : 'inline-block';
            document.getElementById('buyVowelBtn').disabled = !isMyTurn || isTossup;
            document.getElementById('buyVowelBtn').style.display = isTossup ? 'none' : 'inline-block';
            document.getElementById('solveBtn').disabled = !isMyTurn && !canBuzz;
            document.getElementById('guessArea').style.display = isTossup ? 'none' : 'flex';

            // Buzz button for toss-up
            const buzzBtn = document.getElementById('buzzBtn');
            buzzBtn.style.display = canBuzz ? 'inline-block' : 'none';

            // Wild card button
            const myPlayer = gameState.players?.[myPlayerIdx];
            const hasWildCard = myPlayer && (myPlayer.wild_cards || 0) > 0;
            const wildcardBtn = document.getElementById('wildcardBtn');
            wildcardBtn.classList.toggle('available', isMyTurn && hasWildCard && !isTossup);
        }}

        function showNotification(msg) {{
            const notif = document.getElementById('notification');
            notif.textContent = msg;
            notif.style.display = 'block';
            // Notification persists until next action or new notification
        }}

        function hideNotification() {{
            document.getElementById('notification').style.display = 'none';
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

        function guessLetter() {{
            hideNotification();
            const input = document.getElementById('letterInput');
            const letter = input.value.toUpperCase();
            if (letter && letter.length === 1) {{
                socket.emit('guess', {{ room, letter }});
                input.value = '';
            }}
        }}

        function buyVowel() {{
            hideNotification();
            document.getElementById('vowelModal').classList.add('active');
        }}

        function selectVowel(vowel) {{
            socket.emit('buy_vowel', {{ room, letter: vowel }});
            closeModal('vowelModal');
        }}

        function promptSolve() {{
            hideNotification();
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

        function updateHostUI() {{
            document.getElementById('hostControls').style.display = isHost ? 'block' : 'none';
            document.getElementById('claimHostSection').style.display = isHost ? 'none' : 'block';
        }}

        // ========== NEW GAME MECHANICS ==========

        // Mystery wedge choice
        function mysteryChoice(choice) {{
            hideNotification();
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
            hideNotification();
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

        // Buzz in for toss-up
        function buzz() {{
            hideNotification();
            SoundService.buzz();
            socket.emit('buzz', {{ room }});
        }}

        // Enter key to guess
        document.getElementById('letterInput').addEventListener('keypress', (e) => {{
            if (e.key === 'Enter') guessLetter();
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

        connect();
    </script>
</body>
</html>"#, common_styles = COMMON_STYLES))
}

/// Admin page
pub async fn admin() -> Html<String> {
    Html(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Admin</title>
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
        .badge-success {{ background: #2d8f4e; }}
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
                <button class="tab" onclick="showTab('settings')">Settings</button>
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
                <table>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Players</th>
                            <th>Phase</th>
                            <th>Host</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody id="roomsTable"></tbody>
                </table>
            </div>

            <!-- Settings Panel -->
            <div class="panel" id="panel-settings">
                <h2 style="color: #fff; margin-bottom: 16px;">Game Settings</h2>
                <p style="color: #888; margin-bottom: 24px;">Configure default settings for game rooms. These settings will apply to new games.</p>

                <div class="form-row">
                    <div class="form-group">
                        <label>Room</label>
                        <select id="settingsRoom" style="width:100%;padding:12px;background:#0d0628;color:#fff;border:2px solid #333;border-radius:8px;" onchange="loadRoomSettings()">
                            <option value="main">main (default)</option>
                        </select>
                    </div>
                </div>

                <h3 style="color: #d4af37; margin: 24px 0 16px;">Puzzle Settings</h3>
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

                <h3 style="color: #d4af37; margin: 24px 0 16px;">Cost Settings</h3>
                <div class="form-row">
                    <div class="form-group">
                        <label>Vowel Cost ($)</label>
                        <input type="number" id="settingsVowelCost" value="250" min="0" placeholder="250">
                    </div>
                </div>

                <h3 style="color: #d4af37; margin: 24px 0 16px;">Final Round Settings</h3>
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

                <h3 style="color: #d4af37; margin: 24px 0 16px;">Prize Wedge Names</h3>
                <div class="form-row">
                    <div class="form-group">
                        <label>Prize Wedge Names (comma-separated)</label>
                        <input type="text" id="settingsPrizeWedges" value="GIFT CARD" placeholder="GIFT CARD, TRIP, CAR">
                    </div>
                </div>

                <button class="btn" onclick="saveSettings()" style="margin-top: 24px;">Save Settings</button>
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
                if (res.ok) {{
                    isAdmin = true;
                    document.getElementById('adminUser').textContent = user.display_name + ' (Admin)';
                    document.getElementById('adminContent').style.display = 'block';
                    loadUsers();
                    loadPacks();
                    loadRooms();
                }}
            }} catch (e) {{
                showError('Failed to verify admin access');
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

        async function loadUsers() {{
            const res = await fetch('/auth/api/admin/users', {{
                credentials: 'include'
            }});
            const data = await res.json();
            if (data.ok && data.users) {{
                document.getElementById('usersTable').innerHTML = data.users.map(u => `
                    <tr>
                        <td>${{u.id}}</td>
                        <td>${{u.email}}</td>
                        <td>${{u.display_name}}</td>
                        <td>${{u.verified ? '<span class="badge badge-success">Verified</span>' : '<span class="badge badge-warning">Unverified</span>'}}</td>
                        <td>${{u.is_admin ? '<span class="badge badge-success">Admin</span>' : '-'}}</td>
                        <td>
                            ${{!u.verified ? `<button class="btn btn-sm btn-secondary" onclick="verifyUser(${{u.id}})">Verify</button>` : ''}}
                            <button class="btn btn-sm btn-secondary" onclick="toggleAdmin(${{u.id}}, ${{!u.is_admin}})">${{u.is_admin ? 'Remove Admin' : 'Make Admin'}}</button>
                            <button class="btn btn-sm" style="background:#c0392b;" onclick="deleteUser(${{u.id}})">Delete</button>
                        </td>
                    </tr>
                `).join('');
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
                document.getElementById('roomsTable').innerHTML = data.rooms.length > 0 ? data.rooms.map(r => `
                    <tr>
                        <td>${{r.name}}</td>
                        <td>${{r.player_count}}</td>
                        <td>${{r.phase}}</td>
                        <td>${{r.has_host ? '<span class="badge badge-success">Yes</span>' : '-'}}</td>
                        <td>
                            <a href="/game?room=${{encodeURIComponent(r.name)}}" class="btn btn-sm btn-secondary">View</a>
                            <button class="btn btn-sm" style="background:#c0392b;" onclick="deleteRoom('${{r.name}}')">Delete</button>
                        </td>
                    </tr>
                `).join('') : '<tr><td colspan="5" style="color:#888;">No active rooms</td></tr>';
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

        // Settings functions
        async function loadRoomSettings() {{
            const room = document.getElementById('settingsRoom').value;
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
                pack_id: packId > 0 ? packId : null
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
                }} else {{
                    showError('Failed to save settings');
                }}
            }} catch (e) {{
                showError('Failed to save settings');
            }}
        }}

        function updateRoomSelect() {{
            // Update the room select with active rooms
            const select = document.getElementById('settingsRoom');
            const currentValue = select.value;
            select.innerHTML = '<option value="main">main (default)</option>';
            // Add other rooms from the rooms list if available
        }}

        // Initial load
        checkAdmin();
        loadRoomSettings();
    </script>
</body>
</html>"#, common_styles = COMMON_STYLES))
}
