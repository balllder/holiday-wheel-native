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
            padding: 16px 8px;
            margin: 20px 0;
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 4px;
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
                console.log('Received state:', state);
                gameState = state;
                renderGame();
            }});

            socket.on('toast', (data) => {{
                console.log('Toast:', data);
                showNotification(data.msg || data);
            }});

            socket.on('notification', (msg) => {{
                showNotification(msg);
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

            // Phase
            document.getElementById('phase').textContent = gameState.phase || 'normal';

            // Category
            document.getElementById('category').textContent = gameState.puzzle?.category || '-';

            // Puzzle board - Wheel of Fortune style with 4 rows (12, 14, 14, 12)
            const board = document.getElementById('puzzleBoard');
            const ROW_SIZES = [12, 14, 14, 12];

            if (gameState.puzzle?.answer) {{
                const revealed = new Set(gameState.revealed || []);
                const answer = gameState.puzzle.answer.toUpperCase();
                const words = answer.split(' ');

                // Lay out words across rows, keeping words together
                const rows = [[], [], [], []];
                let currentRow = 0;

                for (const word of words) {{
                    // Check if word fits on current row
                    const currentLen = rows[currentRow].reduce((sum, w) => sum + w.length + 1, 0) - 1;
                    const spaceNeeded = currentLen > 0 ? word.length + 1 : word.length;

                    if (currentLen + spaceNeeded <= ROW_SIZES[currentRow]) {{
                        rows[currentRow].push(word);
                    }} else {{
                        // Try next row
                        currentRow++;
                        if (currentRow < 4) {{
                            rows[currentRow].push(word);
                        }}
                    }}
                }}

                // Center the content vertically (use middle rows first if puzzle is short)
                const usedRows = rows.filter(r => r.length > 0).length;
                let startRow = 0;
                if (usedRows === 1) startRow = 1;
                else if (usedRows === 2) startRow = 1;

                // Render rows
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
            }} else {{
                // Show empty board
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

            // Wheel result - use current_wedge
            const wedge = gameState.current_wedge;
            if (wedge !== null && wedge !== undefined) {{
                if (typeof wedge === 'object') {{
                    if (wedge.Cash) {{
                        document.getElementById('wheelResult').textContent = '$' + wedge.Cash;
                    }} else if (wedge.Prize) {{
                        document.getElementById('wheelResult').textContent = wedge.Prize.name || 'Prize';
                    }} else {{
                        // Bankrupt, LoseTurn, FreePlay, etc.
                        const key = Object.keys(wedge)[0] || wedge;
                        document.getElementById('wheelResult').textContent = key.replace(/([A-Z])/g, ' $1').trim();
                    }}
                }} else if (typeof wedge === 'string') {{
                    document.getElementById('wheelResult').textContent = wedge.replace(/([A-Z])/g, ' $1').trim();
                }} else {{
                    document.getElementById('wheelResult').textContent = '$' + wedge;
                }}
            }} else {{
                document.getElementById('wheelResult').textContent = '-';
            }}

            // Players - use active_idx and total
            const playerList = document.getElementById('playerList');
            if (gameState.players && gameState.players.length > 0) {{
                playerList.innerHTML = gameState.players.map((p, idx) => `
                    <div class="player ${{idx === gameState.active_idx ? 'active' : ''}}">
                        <span class="player-name">${{p.name}}${{idx === myPlayerIdx ? ' (you)' : ''}}</span>
                        <span class="player-score">${{p.total + (p.round_bank || 0)}}</span>
                    </div>
                `).join('');
            }}

            // Update controls based on turn - use active_idx
            const isMyTurn = gameState.active_idx === myPlayerIdx;
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
        </div>
    </div>

    <script>
        const token = localStorage.getItem('token');
        const user = JSON.parse(localStorage.getItem('user') || 'null');
        if (!token || !user) {{ window.location.href = '/'; }}

        let isAdmin = false;
        let packs = [];

        async function checkAdmin() {{
            try {{
                const res = await fetch('/auth/api/admin/users', {{
                    headers: {{ 'Authorization': 'Bearer ' + token }}
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
                headers: {{ 'Authorization': 'Bearer ' + token }}
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
                headers: {{ 'Authorization': 'Bearer ' + token }}
            }});
            if (res.ok) {{ showSuccess('User verified'); loadUsers(); }}
            else {{ showError('Failed to verify user'); }}
        }}

        async function toggleAdmin(id, makeAdmin) {{
            const res = await fetch(`/auth/api/admin/users/${{id}}/admin`, {{
                method: 'POST',
                headers: {{ 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json' }},
                body: JSON.stringify({{ is_admin: makeAdmin }})
            }});
            if (res.ok) {{ showSuccess('Admin status updated'); loadUsers(); }}
            else {{ showError('Failed to update admin status'); }}
        }}

        async function deleteUser(id) {{
            if (!confirm('Delete this user?')) return;
            const res = await fetch(`/auth/api/admin/users/${{id}}`, {{
                method: 'DELETE',
                headers: {{ 'Authorization': 'Bearer ' + token }}
            }});
            if (res.ok) {{ showSuccess('User deleted'); loadUsers(); }}
            else {{ showError('Failed to delete user'); }}
        }}

        async function loadPacks() {{
            const res = await fetch('/auth/api/admin/packs', {{
                headers: {{ 'Authorization': 'Bearer ' + token }}
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
            }}
        }}

        async function createPack() {{
            const name = document.getElementById('newPackName').value.trim();
            if (!name) {{ showError('Enter a pack name'); return; }}
            const res = await fetch('/auth/api/admin/packs', {{
                method: 'POST',
                headers: {{ 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json' }},
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
                headers: {{ 'Authorization': 'Bearer ' + token }}
            }});
            if (res.ok) {{ showSuccess('Pack deleted'); loadPacks(); loadPuzzles(); }}
            else {{ showError('Failed to delete pack'); }}
        }}

        async function loadPuzzles() {{
            const packId = document.getElementById('puzzlePackSelect').value;
            const url = packId ? `/auth/api/admin/puzzles?pack_id=${{packId}}` : '/auth/api/admin/puzzles';
            const res = await fetch(url, {{
                headers: {{ 'Authorization': 'Bearer ' + token }}
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
                headers: {{ 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json' }},
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
                headers: {{ 'Authorization': 'Bearer ' + token }}
            }});
            if (res.ok) {{ showSuccess('Puzzle deleted'); loadPuzzles(); loadPacks(); }}
            else {{ showError('Failed to delete puzzle'); }}
        }}

        async function loadRooms() {{
            const res = await fetch('/auth/api/admin/rooms', {{
                headers: {{ 'Authorization': 'Bearer ' + token }}
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
                headers: {{ 'Authorization': 'Bearer ' + token }}
            }});
            if (res.ok) {{ showSuccess('Room deleted'); loadRooms(); }}
            else {{ showError('Failed to delete room'); }}
        }}

        // Initial load
        checkAdmin();
    </script>
</body>
</html>"#, common_styles = COMMON_STYLES))
}
