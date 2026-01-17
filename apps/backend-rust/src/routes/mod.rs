use axum::response::Html;
use axum::Json;
use serde::Serialize;

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
    Html(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Login</title>
    <style>{}</style>
</head>
<body>
    <div class="container">
        <h1>🎡 Holiday Wheel</h1>
        <p class="subtitle">Sign in to play</p>
        <div class="error" id="error"></div>
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
    <script>
        // Check if already logged in
        if (localStorage.getItem('token')) {{
            window.location.href = '/lobby';
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
                    body: JSON.stringify({{ email, password }})
                }});
                const data = await res.json();

                if (res.ok && data.token) {{
                    localStorage.setItem('token', data.token);
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
</html>"#, COMMON_STYLES))
}

/// Register page
pub async fn register() -> Html<String> {
    Html(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Register</title>
    <style>{}</style>
</head>
<body>
    <div class="container">
        <h1>🎡 Holiday Wheel</h1>
        <p class="subtitle">Create your account</p>
        <div class="error" id="error"></div>
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
                    body: JSON.stringify({{ email, password, display_name }})
                }});
                const data = await res.json();

                if (res.ok && data.token) {{
                    localStorage.setItem('token', data.token);
                    localStorage.setItem('user', JSON.stringify(data.user));
                    window.location.href = '/lobby';
                }} else if (res.ok || data.ok) {{
                    // Registration succeeded but needs email verification
                    errorDiv.style.background = '#4CAF50';
                    errorDiv.textContent = data.message || 'Registration successful! Check your email to verify.';
                    errorDiv.style.display = 'block';
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
</html>"#, COMMON_STYLES))
}

/// Lobby page
pub async fn lobby() -> Html<String> {
    Html(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel - Lobby</title>
    <style>{}</style>
</head>
<body>
    <div class="container wide">
        <div class="user-info">
            <div>
                <h1>🎡 Holiday Wheel</h1>
                <span>Welcome, <span class="user-name" id="userName">Player</span>!</span>
            </div>
            <button class="btn btn-secondary" onclick="logout()">Logout</button>
        </div>

        <h2 style="color: #fff; margin-bottom: 16px;">Active Rooms</h2>
        <div class="rooms-grid" id="roomsGrid">
            <p style="color: #888;">Loading rooms...</p>
        </div>

        <div class="join-form">
            <input type="text" id="roomName" placeholder="Enter room name" value="main">
            <button class="btn" onclick="joinRoom()">Join Room</button>
        </div>
    </div>
    <script src="https://cdn.socket.io/4.7.5/socket.io.min.js"></script>
    <script>
        // Check auth
        const token = localStorage.getItem('token');
        const user = JSON.parse(localStorage.getItem('user') || 'null');

        if (!token || !user) {{
            window.location.href = '/';
        }} else {{
            document.getElementById('userName').textContent = user.display_name || user.email;
        }}

        // Load rooms
        async function loadRooms() {{
            try {{
                const res = await fetch('/auth/api/rooms', {{
                    headers: {{ 'Authorization': 'Bearer ' + token }}
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

        function logout() {{
            localStorage.removeItem('token');
            localStorage.removeItem('user');
            window.location.href = '/';
        }}

        // Initial load and refresh every 5 seconds
        loadRooms();
        setInterval(loadRooms, 5000);
    </script>
</body>
</html>"#, COMMON_STYLES))
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
        .game-container {{
            width: 100%;
            max-width: 1200px;
            display: grid;
            grid-template-columns: 1fr 300px;
            gap: 20px;
        }}
        .main-area {{
            background: rgba(26, 10, 62, 0.8);
            border-radius: 16px;
            border: 2px solid #333;
            padding: 24px;
        }}
        .sidebar {{
            background: rgba(26, 10, 62, 0.8);
            border-radius: 16px;
            border: 2px solid #333;
            padding: 24px;
        }}
        .puzzle-board {{
            background: #1a5cb8;
            border-radius: 12px;
            padding: 20px;
            margin: 20px 0;
            min-height: 200px;
            display: flex;
            flex-wrap: wrap;
            justify-content: center;
            align-items: center;
            gap: 8px;
        }}
        .letter-tile {{
            width: 50px;
            height: 60px;
            background: #fff;
            border-radius: 4px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 32px;
            font-weight: bold;
            color: #000;
        }}
        .letter-tile.hidden {{ background: #2a6cc8; color: transparent; }}
        .letter-tile.space {{ background: transparent; width: 30px; }}
        .category {{ color: #d4af37; text-align: center; font-size: 18px; margin-bottom: 10px; }}
        .wheel-area {{ text-align: center; margin: 20px 0; }}
        .wheel-result {{
            font-size: 48px;
            color: #d4af37;
            margin: 20px 0;
        }}
        .controls {{ display: flex; gap: 10px; flex-wrap: wrap; justify-content: center; margin-top: 20px; }}
        .player-list {{ margin-top: 20px; }}
        .player {{
            display: flex;
            justify-content: space-between;
            padding: 12px;
            background: #0d0628;
            border-radius: 8px;
            margin-bottom: 8px;
            border: 2px solid #333;
        }}
        .player.active {{ border-color: #d4af37; }}
        .player-name {{ color: #fff; }}
        .player-score {{ color: #d4af37; font-weight: bold; }}
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
    </style>
</head>
<body>
    <div class="game-container">
        <div class="main-area">
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <h1 style="color: #d4af37; margin: 0;">🎡 Holiday Wheel</h1>
                <a href="/lobby" class="btn btn-secondary">Leave Room</a>
            </div>

            <div class="notification" id="notification"></div>
            <div class="phase-indicator">Phase: <span id="phase">Connecting...</span></div>
            <div class="category">Category: <span id="category">-</span></div>

            <div class="puzzle-board" id="puzzleBoard">
                <p style="color: #fff;">Connecting to game...</p>
            </div>

            <div class="wheel-area">
                <div class="wheel-result" id="wheelResult">-</div>
            </div>

            <div class="controls" id="controls">
                <button class="btn" id="spinBtn" onclick="spin()">Spin</button>
                <button class="btn btn-secondary" id="buyVowelBtn" onclick="buyVowel()">Buy Vowel ($250)</button>
                <button class="btn btn-secondary" id="solveBtn" onclick="promptSolve()">Solve</button>
            </div>

            <div class="guess-input" id="guessArea">
                <input type="text" id="letterInput" maxlength="1" placeholder="Guess a letter">
                <button class="btn" onclick="guessLetter()">Guess</button>
            </div>
        </div>

        <div class="sidebar">
            <h2 style="color: #fff; margin-bottom: 16px;">Players</h2>
            <div class="player-list" id="playerList">
                <p style="color: #888;">No players yet</p>
            </div>

            <div style="margin-top: 24px; padding-top: 16px; border-top: 1px solid #333;">
                <p style="color: #888; font-size: 14px;">Room: <span id="roomName">-</span></p>
            </div>
        </div>
    </div>

    <script src="https://cdn.socket.io/4.7.5/socket.io.min.js"></script>
    <script>
        const token = localStorage.getItem('token');
        const user = JSON.parse(localStorage.getItem('user') || 'null');
        if (!token || !user) {{ window.location.href = '/'; }}

        const urlParams = new URLSearchParams(window.location.search);
        const room = urlParams.get('room') || 'main';
        document.getElementById('roomName').textContent = room;

        let socket;
        let gameState = null;
        let myPlayerIdx = null;

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
                gameState = state;
                renderGame();
            }});

            socket.on('notification', (msg) => {{
                showNotification(msg);
            }});

            socket.on('disconnect', () => {{
                document.getElementById('phase').textContent = 'Disconnected';
            }});
        }}

        function renderGame() {{
            if (!gameState) return;

            // Phase
            document.getElementById('phase').textContent = gameState.phase || 'normal';

            // Category
            document.getElementById('category').textContent = gameState.puzzle?.category || '-';

            // Puzzle board
            const board = document.getElementById('puzzleBoard');
            if (gameState.puzzle?.display) {{
                board.innerHTML = gameState.puzzle.display.split('').map(char => {{
                    if (char === ' ') return '<div class="letter-tile space"></div>';
                    if (char === '_') return '<div class="letter-tile hidden"></div>';
                    return `<div class="letter-tile">${{char}}</div>`;
                }}).join('');
            }}

            // Wheel result
            const lastSpin = gameState.last_spin_value;
            if (lastSpin !== null && lastSpin !== undefined) {{
                if (typeof lastSpin === 'number') {{
                    document.getElementById('wheelResult').textContent = '$' + lastSpin;
                }} else {{
                    document.getElementById('wheelResult').textContent = lastSpin;
                }}
            }}

            // Players
            const playerList = document.getElementById('playerList');
            if (gameState.players && gameState.players.length > 0) {{
                playerList.innerHTML = gameState.players.map((p, idx) => `
                    <div class="player ${{idx === gameState.active_player ? 'active' : ''}}">
                        <span class="player-name">${{p.name}}${{idx === myPlayerIdx ? ' (you)' : ''}}</span>
                        <span class="player-score">${{p.score}}</span>
                    </div>
                `).join('');
            }}

            // Update controls based on turn
            const isMyTurn = gameState.active_player === myPlayerIdx;
            document.getElementById('spinBtn').disabled = !isMyTurn;
            document.getElementById('buyVowelBtn').disabled = !isMyTurn;
            document.getElementById('solveBtn').disabled = !isMyTurn;
        }}

        function showNotification(msg) {{
            const notif = document.getElementById('notification');
            notif.textContent = msg;
            notif.style.display = 'block';
            setTimeout(() => {{ notif.style.display = 'none'; }}, 3000);
        }}

        function spin() {{
            socket.emit('spin', {{ room }});
        }}

        function guessLetter() {{
            const input = document.getElementById('letterInput');
            const letter = input.value.toUpperCase();
            if (letter && letter.length === 1) {{
                socket.emit('guess', {{ room, letter }});
                input.value = '';
            }}
        }}

        function buyVowel() {{
            const vowel = prompt('Enter a vowel (A, E, I, O, U):');
            if (vowel && 'AEIOU'.includes(vowel.toUpperCase())) {{
                socket.emit('buy_vowel', {{ room, letter: vowel.toUpperCase() }});
            }}
        }}

        function promptSolve() {{
            const solution = prompt('Enter your solution:');
            if (solution) {{
                socket.emit('solve', {{ room, solution }});
            }}
        }}

        // Enter key to guess
        document.getElementById('letterInput').addEventListener('keypress', (e) => {{
            if (e.key === 'Enter') guessLetter();
        }});

        connect();
    </script>
</body>
</html>"#, common_styles = COMMON_STYLES))
}
