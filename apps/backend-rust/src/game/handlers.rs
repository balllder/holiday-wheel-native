use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use socketioxide::{
    extract::{Data, SocketRef, State},
    SocketIo,
};
use tracing::{info, warn};

use crate::AppState;

use super::state::{GamePhase, GuessResult};
use super::GameManager;

/// Spawn a background task to load a new puzzle after a delay
fn spawn_auto_advance_puzzle(
    state: Arc<AppState>,
    room: String,
    delay_seconds: u64,
) {
    tokio::spawn(async move {
        // Wait for the display time
        tokio::time::sleep(Duration::from_secs(delay_seconds)).await;

        // Get the pack_id from room config
        let pack_id = {
            let manager = state.game_manager.read().await;
            manager.get_room(&room)
                .map(|game| game.config.pack_id)
                .unwrap_or(None)
        };

        // Get a new puzzle from database
        let puzzle = match state.db.get_random_puzzle(&room, pack_id).await {
            Ok(p) => p,
            Err(e) => {
                info!("Failed to auto-advance puzzle in room {}: {}", room, e);
                return;
            }
        };

        // Update game state and broadcast
        let game_state = {
            let mut manager = state.game_manager.write().await;
            if let Some(game) = manager.get_room_mut(&room) {
                // Only advance if puzzle is still solved (hasn't been manually changed)
                if game.puzzle_solved_by.is_some() {
                    game.new_puzzle(puzzle);
                    Some(game.get_state())
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Broadcast to room
        if let Some(game_state) = game_state {
            if let Some(io) = state.io.get() {
                // Send state update
                if let Some(ns) = io.of("/") {
                    let _ = ns.to(room.clone()).emit("state", &game_state);
                }
                // Send toast notification
                if let Some(ns) = io.of("/") {
                    let _ = ns.to(room).emit("toast", &serde_json::json!({ "msg": "New puzzle!" }));
                }
            }
        }
    });
}

// ========== REQUEST TYPES ==========

#[derive(Debug, Deserialize)]
pub struct RoomRequest {
    pub room: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinGameRequest {
    pub room: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClaimHostRequest {
    pub room: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct GuessRequest {
    pub room: String,
    pub letter: String,
}

#[derive(Debug, Deserialize)]
pub struct SolveRequest {
    pub room: String,
    pub attempt: String,
}

#[derive(Debug, Deserialize)]
pub struct SetActivePlayerRequest {
    pub room: String,
    pub player_idx: usize,
}

#[derive(Debug, Deserialize)]
pub struct SetPlayersRequest {
    pub room: String,
    pub names: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SetPrizeNamesRequest {
    pub room: String,
    pub names: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetConfigRequest {
    pub room: String,
    pub config: ConfigUpdate,
}

#[derive(Debug, Deserialize)]
pub struct ConfigUpdate {
    pub vowel_cost: Option<i32>,
    pub final_seconds: Option<i32>,
    pub final_jackpot: Option<i32>,
    pub prize_replace_cash_values: Option<Vec<i32>>,
}

#[derive(Debug, Deserialize)]
pub struct FinalPickRequest {
    pub room: String,
    pub kind: String,
    pub letter: String,
}

#[derive(Debug, Deserialize)]
pub struct FinalPickConsonantsRequest {
    pub room: String,
    pub letters: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct FinalPickVowelRequest {
    pub room: String,
    pub letter: String,
}

#[derive(Debug, Deserialize)]
pub struct SetActivePack {
    pub room: String,
    pub pack_id: Option<i64>,
    pub pack_name: Option<String>,
}

// ========== HELPER MACROS ==========

macro_rules! toast {
    ($socket:expr, $msg:expr) => {
        $socket.emit("toast", &serde_json::json!({ "msg": $msg })).ok()
    };
}

macro_rules! broadcast_state {
    ($socket:expr, $room:expr, $state:expr) => {{
        let game_state = $state;
        $socket.to($room.clone()).emit("state", &game_state).ok();
        $socket.emit("state", &game_state).ok();
    }};
}

// ========== REGISTER ALL HANDLERS ==========

/// Request type for socket authentication
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub token: String,
}

/// Response type for socket authentication
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Validate a JWT/auth token and return user info if valid
/// This is the core authentication logic used by the socket auth handler
pub async fn validate_socket_auth_token(
    state: &Arc<AppState>,
    token: &str,
) -> Result<(i64, String), &'static str> {
    // Empty token is invalid
    if token.is_empty() {
        return Err("Token is required");
    }

    // Verify token against database
    match state.db.get_user_by_token(token).await {
        Ok(Some(user)) => {
            // Check if user is verified
            if !user.verified {
                return Err("Account not verified");
            }
            Ok((user.id, user.display_name))
        }
        Ok(None) => Err("Invalid or expired token"),
        Err(_) => Err("Database error during authentication"),
    }
}

/// Look up the user_id associated with a socket from user_sockets tracking
/// Returns Some(user_id) if the socket has been authenticated, None otherwise
pub async fn get_socket_user_id(state: &Arc<AppState>, socket_id: &str) -> Option<i64> {
    let user_sockets = state.user_sockets.read().await;
    for (user_id, sockets) in user_sockets.iter() {
        if sockets.contains(socket_id) {
            return Some(*user_id);
        }
    }
    None
}

/// Look up the avatar_id for a user
/// Returns the avatar_id (1-12) or None if user not found
pub async fn get_user_avatar_id(state: &Arc<AppState>, user_id: i64) -> Option<i64> {
    match state.db.get_user_by_id(user_id).await {
        Ok(Some(user)) => Some(user.avatar_id),
        _ => None,
    }
}

// ========== TESTABLE HELPER FUNCTIONS ==========
// These functions extract handler logic for unit testing

/// Validate and process a room join request
/// Returns the game state if successful
pub fn handle_join_room(manager: &GameManager, room: &str) -> Option<super::state::GameState> {
    manager.get_room(room).map(|game| game.get_state())
}

/// Process a player joining the game
/// Returns (player_idx, is_reconnect, player_name)
pub fn handle_join_game(
    manager: &mut GameManager,
    room: &str,
    socket_id: &str,
    name: Option<String>,
) -> Result<(usize, bool, String), &'static str> {
    let game = manager.get_or_create_room(room);

    // Check if already in game with this socket
    if let Some(idx) = game.player_idx_by_socket(socket_id) {
        return Ok((idx, true, game.players[idx].name.clone()));
    }

    let name = name.unwrap_or_else(|| format!("Player {}", game.players.len() + 1));

    // Check if there's a disconnected player with the same name to reconnect
    let existing_idx = game
        .players
        .iter()
        .position(|p| p.name == name && p.socket_id.is_none());

    if let Some(idx) = existing_idx {
        // Reconnect to existing player slot
        game.players[idx].socket_id = Some(socket_id.to_string());
        game.players[idx].disconnected_at = None;
        return Ok((idx, true, name));
    }

    // Add new player (avatar_id will be set separately if authenticated)
    let player_idx = game.add_player(name.clone(), Some(socket_id.to_string()), None, None);
    Ok((player_idx, false, name))
}

/// Process host claim request
/// Returns true if host was granted
pub fn handle_claim_host(
    manager: &mut super::GameManager,
    room: &str,
    socket_id: &str,
    code: &str,
    host_code: &str,
) -> bool {
    if code != host_code {
        return false;
    }

    let game = manager.get_or_create_room(room);
    game.host_sid = Some(socket_id.to_string());
    true
}

/// Process spin request
/// Returns Some(wedge, message) if spin was successful
pub fn handle_spin(
    manager: &mut super::GameManager,
    room: &str,
    socket_id: &str,
) -> Result<(super::WedgeValue, String), &'static str> {
    let game = manager.get_room_mut(room).ok_or("Room not found")?;

    if game.phase != super::state::GamePhase::Normal {
        return Err("Spin is only allowed during normal rounds.");
    }

    if !game.is_active_player(socket_id, true) {
        return Err("Only the active player (or host) can spin.");
    }

    if let Some(wedge) = game.spin() {
        let msg = match &wedge {
            super::WedgeValue::Bankrupt => "BANKRUPT! Lost all round earnings.".to_string(),
            super::WedgeValue::LoseTurn => "LOSE A TURN!".to_string(),
            super::WedgeValue::FreePlay => "FREE PLAY! Guess a letter.".to_string(),
            super::WedgeValue::Cash(amount) => format!("${}! Guess a consonant.", amount),
            super::WedgeValue::Prize { name, .. } => {
                format!("{}! Guess a consonant to win it.", name)
            }
        };
        Ok((wedge, msg))
    } else {
        Err("Failed to spin wheel")
    }
}

/// Process guess consonant request
/// Returns (result_message, should_broadcast)
pub fn handle_guess(
    manager: &mut super::GameManager,
    room: &str,
    socket_id: &str,
    letter: char,
) -> Result<String, &'static str> {
    let game = manager.get_room_mut(room).ok_or("Room not found")?;

    if game.phase != super::state::GamePhase::Normal {
        return Err("Letter guesses are only allowed during normal rounds.");
    }

    if !game.is_active_player(socket_id, false) {
        return Err("Only the active player can guess.");
    }

    let result = game.guess_consonant(letter);
    let msg = match result {
        super::state::GuessResult::Correct(count) => {
            format!("{} {}(s)!", count, letter.to_uppercase())
        }
        super::state::GuessResult::Incorrect => format!("No {}s", letter.to_uppercase()),
        super::state::GuessResult::AlreadyUsed => "Letter already used".to_string(),
        super::state::GuessResult::InvalidLetter => {
            "Invalid letter (must be a consonant)".to_string()
        }
        super::state::GuessResult::NotEnoughMoney => "Not enough money".to_string(),
        super::state::GuessResult::NeedToSpin => "Spin before guessing a consonant".to_string(),
    };
    Ok(msg)
}

/// Process solve attempt
/// Returns (solved, message, auto_advance_delay)
pub fn handle_solve(
    manager: &mut super::GameManager,
    room: &str,
    socket_id: &str,
    attempt: &str,
) -> Result<(bool, String, Option<u64>), &'static str> {
    let game = manager.get_room_mut(room).ok_or("Room not found")?;

    match game.phase {
        super::state::GamePhase::Normal => {
            if !game.is_active_player(socket_id, false) {
                return Err("Only the active player can solve.");
            }
            let solved = game.solve(attempt);
            if solved {
                let delay = Some(game.config.puzzle_display_seconds as u64);
                Ok((true, "Correct! Puzzle solved!".to_string(), delay))
            } else {
                Ok((false, "Incorrect, sorry!".to_string(), None))
            }
        }
        super::state::GamePhase::Tossup => {
            if game.tossup.controller_sid.as_deref() != Some(socket_id) {
                return Err("Only the player who buzzed in can solve.");
            }
            let solved = game.solve(attempt);
            if solved {
                game.tossup_correct_answer();
                let delay = Some(game.config.puzzle_display_seconds as u64);
                Ok((true, "Correct! You win the toss-up!".to_string(), delay))
            } else {
                game.tossup_wrong_answer();
                Ok((false, "Incorrect! You're locked out.".to_string(), None))
            }
        }
        super::state::GamePhase::Final => {
            if !game.is_active_player(socket_id, false) {
                return Err("Only the active player can solve.");
            }
            let solved = game.final_solve(attempt);
            let msg = if solved {
                format!("Correct! You win the ${} jackpot!", game.config.final_jackpot)
            } else {
                "Incorrect!".to_string()
            };
            Ok((solved, msg, None))
        }
    }
}

/// Process buzz-in during toss-up
/// Returns (player_idx, player_name) on success
pub fn handle_buzz(
    manager: &mut super::GameManager,
    room: &str,
    socket_id: &str,
) -> Result<(usize, String), &'static str> {
    let game = manager.get_room_mut(room).ok_or("Room not found")?;

    let player_idx = game.tossup_buzz(socket_id)?;
    let name = game.players[player_idx].name.clone();
    Ok((player_idx, name))
}

pub fn register_handlers(io: &SocketIo) {
    io.ns("/", |socket: SocketRef, State(_state): State<Arc<AppState>>| {
        info!("Client connected: {}", socket.id);

        // ========== SOCKET AUTHENTICATION ==========

        // Authenticate socket and register for session invalidation
        // This handler validates the JWT token and associates the user_id with the connection
        socket.on(
            "auth",
            |socket: SocketRef, Data(req): Data<AuthRequest>, State(state): State<Arc<AppState>>| async move {
                info!("Socket {} authenticating with token", socket.id);

                // Validate the token using our helper function
                match validate_socket_auth_token(&state, &req.token).await {
                    Ok((user_id, display_name)) => {
                        // Join a user-specific room for session invalidation broadcasts
                        let user_room = format!("user:{}", user_id);
                        if let Err(e) = socket.join(user_room.clone()) {
                            warn!("Failed to join user room: {:?}", e);
                        }

                        // Track socket in user_sockets for cleanup and session management
                        {
                            let mut user_sockets = state.user_sockets.write().await;
                            user_sockets
                                .entry(user_id)
                                .or_insert_with(HashSet::new)
                                .insert(socket.id.to_string());
                        }

                        info!(
                            "Socket {} authenticated as user {} ({})",
                            socket.id, user_id, display_name
                        );

                        // Send success response with user info
                        let response = AuthResponse {
                            ok: true,
                            user_id: Some(user_id),
                            display_name: Some(display_name),
                            error: None,
                        };
                        socket.emit("auth_ok", &response).ok();
                    }
                    Err(error_msg) => {
                        warn!(
                            "Socket {} authentication failed: {}",
                            socket.id, error_msg
                        );

                        // Send error response
                        let response = AuthResponse {
                            ok: false,
                            user_id: None,
                            display_name: None,
                            error: Some(error_msg.to_string()),
                        };
                        socket.emit("auth_error", &response).ok();

                        // Optionally disconnect the socket after auth failure
                        // This is commented out to allow retry, but can be enabled for stricter security
                        // socket.disconnect().ok();
                    }
                }
            },
        );

        // ========== JOIN/LEAVE ==========

        // Join a room (as spectator)
        socket.on(
            "join",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                info!("Socket {} joining room {}", socket.id, req.room);
                socket.join(req.room.clone()).ok();

                let manager = state.game_manager.read().await;
                if let Some(game) = manager.get_room(&req.room) {
                    let game_state = game.get_state();
                    socket.emit("state", &game_state).ok();
                }
            },
        );

        // Join as a player
        // Associates the authenticated user_id with the player if socket was authenticated
        socket.on(
            "join_game",
            |socket: SocketRef, Data(req): Data<JoinGameRequest>, State(state): State<Arc<AppState>>| async move {
                info!("Socket {} joining game in room {}", socket.id, req.room);
                socket.join(req.room.clone()).ok();

                // Look up user_id from authenticated socket (if they called auth first)
                let user_id = get_socket_user_id(&state, socket.id.as_str()).await;
                // Look up avatar_id if authenticated
                let avatar_id = if let Some(uid) = user_id {
                    info!("Socket {} is authenticated as user_id {:?}", socket.id, user_id);
                    get_user_avatar_id(&state, uid).await
                } else {
                    None
                };

                // Check if room exists and needs initial puzzle (default puzzle has id=0)
                let needs_initial_puzzle = {
                    let manager = state.game_manager.read().await;
                    match manager.get_room(&req.room) {
                        Some(game) => game.puzzle.id == 0,
                        None => true, // Room doesn't exist yet, will need puzzle
                    }
                };

                // Fetch initial puzzle from database if needed (before acquiring write lock)
                let initial_puzzle = if needs_initial_puzzle {
                    match state.db.get_random_puzzle(&req.room, None).await {
                        Ok(p) => Some(p),
                        Err(e) => {
                            warn!("Failed to get initial puzzle for new room: {}", e);
                            None
                        }
                    }
                } else {
                    None
                };

                let mut manager = state.game_manager.write().await;
                let game = manager.get_or_create_room(&req.room);

                // Set initial puzzle if we fetched one and room still has default puzzle
                if let Some(puzzle) = initial_puzzle {
                    if game.puzzle.id == 0 {
                        info!("Setting initial puzzle for room {}: {}", req.room, puzzle.answer);
                        game.new_puzzle(puzzle);
                    }
                }

                // Check if already in game with this socket
                if let Some(idx) = game.player_idx_by_socket(socket.id.as_str()) {
                    // Update user_id association if missing
                    if game.players[idx].user_id.is_none() && user_id.is_some() {
                        game.players[idx].user_id = user_id;
                    }
                    // Update avatar_id if we have one
                    if let Some(aid) = avatar_id {
                        game.players[idx].avatar_id = aid;
                    }
                    socket.emit("you", &serde_json::json!({ "player_idx": idx, "user_id": user_id })).ok();
                    broadcast_state!(socket, req.room, game.get_state());
                    return;
                }

                let name = req.name.unwrap_or_else(|| format!("Player {}", game.players.len() + 1));

                // Check if there's a disconnected player with the same name to reconnect
                // Also check by user_id if authenticated
                let existing_idx = game.players.iter().position(|p| {
                    // Match by name AND disconnected socket
                    let name_match = p.name == name && p.socket_id.is_none();
                    // OR match by user_id if both are present and socket disconnected
                    let user_match = user_id.is_some() && p.user_id == user_id && p.socket_id.is_none();
                    name_match || user_match
                });

                if let Some(idx) = existing_idx {
                    // Reconnect to existing player slot
                    game.players[idx].socket_id = Some(socket.id.to_string());
                    game.players[idx].disconnected_at = None; // Clear disconnect timestamp
                    // Update user_id if newly authenticated
                    if game.players[idx].user_id.is_none() && user_id.is_some() {
                        game.players[idx].user_id = user_id;
                    }
                    // Update avatar_id if we have one
                    if let Some(aid) = avatar_id {
                        game.players[idx].avatar_id = aid;
                    }
                    let reconnected_name = game.players[idx].name.clone();
                    socket.emit("you", &serde_json::json!({ "player_idx": idx, "user_id": user_id })).ok();
                    toast!(socket, &format!("Reconnected as {}!", reconnected_name));
                    broadcast_state!(socket, req.room, game.get_state());
                    return;
                }

                // Add new player with user_id association
                let player_idx = game.add_player(name.clone(), Some(socket.id.to_string()), user_id, avatar_id);

                socket.emit("you", &serde_json::json!({ "player_idx": player_idx, "user_id": user_id })).ok();
                toast!(socket, &format!("Joined as {}!", name));
                broadcast_state!(socket, req.room, game.get_state());
            },
        );

        // Leave the game
        socket.on(
            "leave_game",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if let Some(player) = game.remove_player_by_socket(socket.id.as_str()) {
                        info!("Player {} left room {}", player.name, req.room);
                        socket.emit("you", &serde_json::json!({ "player_idx": null })).ok();
                        toast!(socket, &format!("{} left the game.", player.name));
                        broadcast_state!(socket, req.room, game.get_state());
                    }
                }
            },
        );

        // ========== HOST SYSTEM ==========

        // Claim host
        socket.on(
            "claim_host",
            |socket: SocketRef, Data(req): Data<ClaimHostRequest>, State(state): State<Arc<AppState>>| async move {
                let host_code = std::env::var("HOST_CODE").unwrap_or_else(|_| "holiday".to_string());

                if req.code != host_code {
                    toast!(socket, "Invalid host code.");
                    socket.emit("host_granted", &serde_json::json!({ "granted": false })).ok();
                    return;
                }

                let mut manager = state.game_manager.write().await;
                let game = manager.get_or_create_room(&req.room);
                game.host_sid = Some(socket.id.to_string());

                socket.emit("host_granted", &serde_json::json!({ "granted": true })).ok();
                toast!(socket, "Host mode enabled on this device.");
                broadcast_state!(socket, req.room, game.get_state());
            },
        );

        // Release host
        socket.on(
            "release_host",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Only the host can release host mode.");
                        return;
                    }
                    game.host_sid = None;
                    socket.emit("host_granted", &serde_json::json!({ "granted": false })).ok();
                    toast!(socket, "Host released.");
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // ========== GAME CONTROLS (HOST ONLY) ==========

        // New puzzle
        socket.on(
            "new_puzzle",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                // Get the pack_id from room config first (needs read lock)
                let pack_id = {
                    let manager = state.game_manager.read().await;
                    manager.get_room(&req.room)
                        .map(|game| game.config.pack_id)
                        .unwrap_or(None)
                };

                // Get puzzle from database
                let puzzle = match state.db.get_random_puzzle(&req.room, pack_id).await {
                    Ok(p) => p,
                    Err(e) => {
                        toast!(socket, &format!("Failed to get puzzle: {}", e));
                        return;
                    }
                };

                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }

                    game.new_puzzle(puzzle);
                    toast!(socket, "New puzzle loaded.");
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // New game (reset)
        socket.on(
            "new_game",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                // Get the pack_id from room config first (needs read lock)
                let pack_id = {
                    let manager = state.game_manager.read().await;
                    manager.get_room(&req.room)
                        .map(|game| game.config.pack_id)
                        .unwrap_or(None)
                };

                // Get a new puzzle from database
                let puzzle = match state.db.get_random_puzzle(&req.room, pack_id).await {
                    Ok(p) => p,
                    Err(e) => {
                        toast!(socket, &format!("Failed to get puzzle: {}", e));
                        return;
                    }
                };

                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }
                    game.reset_game();
                    game.new_puzzle(puzzle);
                    toast!(socket, "New game started.");
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // Reveal all letters
        socket.on(
            "reveal_all",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }
                    game.reveal_all();
                    toast!(socket, "All letters revealed.");
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // Set active player
        socket.on(
            "set_active_player",
            |socket: SocketRef, Data(req): Data<SetActivePlayerRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }
                    if req.player_idx >= game.players.len() {
                        toast!(socket, "Invalid player index.");
                        return;
                    }
                    game.active_idx = req.player_idx;
                    let name = game.players[req.player_idx].name.clone();
                    toast!(socket, &format!("Active player set to {}.", name));
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // Set players (names)
        socket.on(
            "set_players",
            |socket: SocketRef, Data(req): Data<SetPlayersRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }
                    if req.names.is_empty() {
                        toast!(socket, "Provide a list of player names.");
                        return;
                    }

                    game.players.clear();
                    for name in req.names.iter() {
                        game.add_player(name[..name.len().min(30)].to_string(), None, None, None);
                    }
                    game.active_idx = 0;
                    toast!(socket, &format!("Set {} players.", game.players.len()));
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // Set config
        socket.on(
            "set_config",
            |socket: SocketRef, Data(req): Data<SetConfigRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }

                    if let Some(v) = req.config.vowel_cost {
                        game.config.vowel_cost = v;
                    }
                    if let Some(v) = req.config.final_seconds {
                        game.config.final_seconds = v;
                    }
                    if let Some(v) = req.config.final_jackpot {
                        game.config.final_jackpot = v;
                    }
                    if let Some(v) = req.config.prize_replace_cash_values {
                        game.config.prize_replace_cash_values = v;
                    }

                    toast!(socket, "Config saved.");
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // Set active puzzle pack for the room
        socket.on(
            "set_pack",
            |socket: SocketRef, Data(req): Data<SetActivePack>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }

                    // Set the active pack (None or 0 means all packs)
                    game.config.pack_id = req.pack_id;
                    game.active_pack_id = req.pack_id;

                    let pack_name = if req.pack_id.is_none() || req.pack_id == Some(0) {
                        "All Packs".to_string()
                    } else {
                        req.pack_name.clone().unwrap_or_else(|| format!("Pack {}", req.pack_id.unwrap()))
                    };
                    game.active_pack_name = Some(pack_name.clone());

                    toast!(socket, &format!("Puzzle pack changed to: {}", pack_name));
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // ========== GAME ACTIONS ==========

        // Spin the wheel
        socket.on(
            "spin",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                info!("Spin request in room {}", req.room);

                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if game.phase != GamePhase::Normal {
                        toast!(socket, "Spin is only allowed during normal rounds.");
                        return;
                    }

                    if !game.is_active_player(socket.id.as_str(), true) {
                        toast!(socket, "Only the active player (or host) can spin.");
                        return;
                    }

                    if let Some(wedge) = game.spin() {
                        let msg = match wedge {
                            super::WedgeValue::Bankrupt => "BANKRUPT! Lost all round earnings.".to_string(),
                            super::WedgeValue::LoseTurn => "LOSE A TURN!".to_string(),
                            super::WedgeValue::FreePlay => "FREE PLAY! Guess a letter.".to_string(),
                            super::WedgeValue::Cash(amount) => format!("${}! Guess a consonant.", amount),
                            super::WedgeValue::Prize { name, .. } => format!("{}! Guess a consonant to win it.", name),
                        };
                        toast!(socket, &msg);
                        // Broadcast to entire room (including spectators)
                        let _ = socket.to(req.room.clone()).emit("toast", &serde_json::json!({ "msg": &msg }));
                    }
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // Spin animation complete - start turn timer
        socket.on(
            "spin_complete",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                info!("Spin complete in room {}", req.room);

                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    // Only start timer if:
                    // 1. In normal phase
                    // 2. There's a current wedge (spin was successful, not Bankrupt/LoseTurn)
                    // 3. Timer is enabled (turn_timer_seconds > 0)
                    if game.phase == GamePhase::Normal
                        && game.current_wedge.is_some()
                        && game.config.turn_timer_seconds > 0
                    {
                        game.start_turn_timer();
                        info!(
                            "Turn timer started: {} seconds in room {}",
                            game.config.turn_timer_seconds, req.room
                        );

                        // Spawn background task to check for timer expiry
                        let state_clone = state.clone();
                        let room_clone = req.room.clone();
                        let timer_seconds = game.config.turn_timer_seconds as u64;
                        tokio::spawn(async move {
                            // Wait for timer to expire (plus small buffer)
                            tokio::time::sleep(Duration::from_secs(timer_seconds + 1)).await;

                            // Check if timer expired and auto-pass
                            let mut manager = state_clone.game_manager.write().await;
                            if let Some(game) = manager.get_room_mut(&room_clone) {
                                if game.turn_timer_expired() && game.current_wedge.is_some() {
                                    // Timer expired - auto-pass turn
                                    let player_name = game.players.get(game.active_idx)
                                        .map(|p| p.name.clone())
                                        .unwrap_or_else(|| "Player".to_string());
                                    info!(
                                        "Turn timer expired for {} in room {}, auto-passing",
                                        player_name, room_clone
                                    );
                                    game.clear_turn_state();
                                    game.advance_turn();

                                    // Broadcast update
                                    if let Some(io) = state_clone.io.get() {
                                        let msg = format!("{} ran out of time!", player_name);
                                        if let Some(ns) = io.of("/") {
                                            let _ = ns.to(room_clone.clone()).emit("toast", &serde_json::json!({ "msg": msg }));
                                        }
                                        if let Some(ns) = io.of("/") {
                                            let _ = ns.to(room_clone).emit("state", &game.get_state());
                                        }
                                    }
                                }
                            }
                        });

                        broadcast_state!(socket, req.room, game.get_state());
                    }
                }
            },
        );

        // Guess a letter (consonant)
        socket.on(
            "guess",
            |socket: SocketRef, Data(req): Data<GuessRequest>, State(state): State<Arc<AppState>>| async move {
                info!("Guess '{}' in room {}", req.letter, req.room);

                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if game.phase != GamePhase::Normal {
                        toast!(socket, "Letter guesses are only allowed during normal rounds.");
                        return;
                    }

                    if !game.is_active_player(socket.id.as_str(), false) {
                        toast!(socket, "Only the active player can guess.");
                        return;
                    }

                    if let Some(letter) = req.letter.chars().next() {
                        let result = game.guess_consonant(letter);
                        let msg = match result {
                            GuessResult::Correct(count) => format!("{} {}(s)!", count, letter.to_uppercase()),
                            GuessResult::Incorrect => format!("No {}s", letter.to_uppercase()),
                            GuessResult::AlreadyUsed => "Letter already used".to_string(),
                            GuessResult::InvalidLetter => "Invalid letter (must be a consonant)".to_string(),
                            GuessResult::NotEnoughMoney => "Not enough money".to_string(),
                            GuessResult::NeedToSpin => "Spin before guessing a consonant".to_string(),
                        };
                        toast!(socket, &msg);
                        // Broadcast to entire room (including spectators)
                        let _ = socket.to(req.room.clone()).emit("toast", &serde_json::json!({ "msg": &msg }));
                        broadcast_state!(socket, req.room, game.get_state());
                    }
                }
            },
        );

        // Buy a vowel
        socket.on(
            "buy_vowel",
            |socket: SocketRef, Data(req): Data<GuessRequest>, State(state): State<Arc<AppState>>| async move {
                info!("Buy vowel '{}' in room {}", req.letter, req.room);

                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if game.phase != GamePhase::Normal {
                        toast!(socket, "Vowels can only be bought during normal rounds.");
                        return;
                    }

                    if !game.is_active_player(socket.id.as_str(), false) {
                        toast!(socket, "Only the active player can buy vowels.");
                        return;
                    }

                    if let Some(letter) = req.letter.chars().next() {
                        let result = game.buy_vowel(letter);
                        let msg = match result {
                            GuessResult::Correct(count) => format!("{} {}(s)!", count, letter.to_uppercase()),
                            GuessResult::Incorrect => format!("No {}s", letter.to_uppercase()),
                            GuessResult::AlreadyUsed => "Letter already used".to_string(),
                            GuessResult::InvalidLetter => "Must be a vowel".to_string(),
                            GuessResult::NotEnoughMoney => format!("Need ${} to buy a vowel", game.config.vowel_cost),
                            GuessResult::NeedToSpin => "Cannot buy vowel now".to_string(),
                        };
                        toast!(socket, &msg);
                        // Broadcast to entire room (including spectators)
                        let _ = socket.to(req.room.clone()).emit("toast", &serde_json::json!({ "msg": &msg }));
                        broadcast_state!(socket, req.room, game.get_state());
                    }
                }
            },
        );

        // Solve attempt
        socket.on(
            "solve",
            |socket: SocketRef, Data(req): Data<SolveRequest>, State(state): State<Arc<AppState>>| async move {
                info!("Solve attempt '{}' in room {}", req.attempt, req.room);

                // Track if we should auto-advance after solving
                let mut auto_advance_delay: Option<u64> = None;

                {
                    let mut manager = state.game_manager.write().await;
                    if let Some(game) = manager.get_room_mut(&req.room) {
                        // Handle solve based on phase
                        match game.phase {
                            GamePhase::Normal => {
                                if !game.is_active_player(socket.id.as_str(), false) {
                                    toast!(socket, "Only the active player can solve.");
                                    return;
                                }
                                let solved = game.solve(&req.attempt);
                                let msg = if solved {
                                    // Schedule auto-advance after puzzle display time
                                    auto_advance_delay = Some(game.config.puzzle_display_seconds as u64);
                                    "Correct! Puzzle solved!"
                                } else {
                                    "Incorrect, sorry!"
                                };
                                toast!(socket, msg);
                                // Broadcast to entire room (including spectators)
                                let _ = socket.to(req.room.clone()).emit("toast", &serde_json::json!({ "msg": msg }));
                            }
                            GamePhase::Tossup => {
                                // During tossup, the controller can solve
                                if game.tossup.controller_sid.as_deref() != Some(socket.id.as_str()) {
                                    toast!(socket, "Only the player who buzzed in can solve.");
                                    return;
                                }
                                let solved = game.solve(&req.attempt);
                                if solved {
                                    game.tossup_correct_answer();
                                    // Schedule auto-advance after puzzle display time
                                    auto_advance_delay = Some(game.config.puzzle_display_seconds as u64);
                                    let msg = "Correct! You win the toss-up!";
                                    toast!(socket, msg);
                                    let _ = socket.to(req.room.clone()).emit("toast", &serde_json::json!({ "msg": msg }));
                                } else {
                                    game.tossup_wrong_answer();
                                    let msg = "Incorrect! Locked out.";
                                    toast!(socket, msg);
                                    let _ = socket.to(req.room.clone()).emit("toast", &serde_json::json!({ "msg": msg }));
                                }
                            }
                            GamePhase::Final => {
                                if !game.is_active_player(socket.id.as_str(), false) {
                                    toast!(socket, "Only the active player can solve.");
                                    return;
                                }
                                let solved = game.final_solve(&req.attempt);
                                let msg = if solved {
                                    format!("Correct! You win the ${} jackpot!", game.config.final_jackpot)
                                } else {
                                    "Incorrect!".to_string()
                                };
                                toast!(socket, &msg);
                                // Broadcast to entire room (including spectators)
                                let _ = socket.to(req.room.clone()).emit("toast", &serde_json::json!({ "msg": &msg }));
                                // No auto-advance for final round
                            }
                        }
                        broadcast_state!(socket, req.room, game.get_state());
                    }
                } // Lock dropped here

                // Spawn auto-advance task if puzzle was solved
                if let Some(delay) = auto_advance_delay {
                    spawn_auto_advance_puzzle(state.clone(), req.room, delay);
                }
            },
        );

        // ========== TOSS-UP MODE ==========

        // Start toss-up
        socket.on(
            "start_tossup",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }
                    game.start_tossup();
                    toast!(socket, "Toss-up started! Buzz in to solve!");
                    broadcast_state!(socket, req.room, game.get_state());

                    // Spawn background task to reveal letters progressively
                    let state_clone = state.clone();
                    let room_clone = req.room.clone();
                    tokio::spawn(async move {
                        // Reveal letters every 1.5 seconds until solved or ended
                        loop {
                            tokio::time::sleep(Duration::from_millis(1500)).await;

                            let mut manager = state_clone.game_manager.write().await;
                            if let Some(game) = manager.get_room_mut(&room_clone) {
                                // Stop if no longer in toss-up
                                if game.phase != GamePhase::Tossup {
                                    break;
                                }

                                // Reveal one letter
                                let revealed = game.tossup_reveal_step(1);
                                if revealed > 0 {
                                    // Broadcast updated state
                                    if let Some(io) = state_clone.io.get() {
                                        if let Some(ns) = io.of("/") {
                                            let _ = ns.to(room_clone.clone()).emit("state", &game.get_state());
                                        }
                                    }
                                } else {
                                    // No more letters to reveal
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    });
                }
            },
        );

        // End toss-up
        socket.on(
            "end_tossup",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }
                    game.end_tossup();
                    toast!(socket, "Toss-up ended.");
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // Buzz in
        socket.on(
            "buzz",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    match game.tossup_buzz(socket.id.as_str()) {
                        Ok(player_idx) => {
                            let name = game.players[player_idx].name.clone();
                            // Broadcast to all in room
                            let _ = socket.to(req.room.clone()).emit("toast", &serde_json::json!({ "msg": format!("{} buzzed in!", name) }));
                            toast!(socket, &format!("{} buzzed in!", name));

                            // Spawn background task to check for buzz timer expiry
                            if game.config.buzz_timer_seconds > 0 {
                                let state_clone = state.clone();
                                let room_clone = req.room.clone();
                                let timer_seconds = game.config.buzz_timer_seconds as u64;
                                let buzzer_name = name.clone();
                                tokio::spawn(async move {
                                    // Wait for timer to expire (plus small buffer)
                                    tokio::time::sleep(Duration::from_secs(timer_seconds + 1)).await;

                                    // Check if timer expired and lock out player
                                    let mut manager = state_clone.game_manager.write().await;
                                    if let Some(game) = manager.get_room_mut(&room_clone) {
                                        if game.phase == GamePhase::Tossup
                                            && game.buzz_timer_expired()
                                            && game.tossup.controller_sid.is_some()
                                        {
                                            // Timer expired - lock out the player
                                            info!(
                                                "Buzz timer expired for {} in room {}, locking out",
                                                buzzer_name, room_clone
                                            );
                                            game.tossup_buzz_timeout();

                                            // Broadcast update
                                            if let Some(io) = state_clone.io.get() {
                                                let msg = format!("{} ran out of time!", buzzer_name);
                                                if let Some(ns) = io.of("/") {
                                                    let _ = ns.to(room_clone.clone()).emit("toast", &serde_json::json!({ "msg": msg }));
                                                }
                                                if let Some(ns) = io.of("/") {
                                                    let _ = ns.to(room_clone).emit("state", &game.get_state());
                                                }
                                            }
                                        }
                                    }
                                });
                            }
                        }
                        Err(msg) => {
                            toast!(socket, msg);
                        }
                    }
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // Guess a letter during toss-up
        socket.on(
            "tossup_guess",
            |socket: SocketRef, Data(req): Data<GuessRequest>, State(state): State<Arc<AppState>>| async move {
                info!("Toss-up guess '{}' in room {}", req.letter, req.room);

                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if game.phase != GamePhase::Tossup {
                        toast!(socket, "Not in toss-up mode.");
                        return;
                    }

                    // Only the controller (who buzzed in) can guess
                    if game.tossup.controller_sid.as_deref() != Some(socket.id.as_str()) {
                        toast!(socket, "Only the player who buzzed in can guess.");
                        return;
                    }

                    if let Some(letter) = req.letter.chars().next() {
                        let result = game.tossup_guess_letter(letter);
                        let msg = match result {
                            GuessResult::Correct(count) => format!("{} {}(s)!", count, letter.to_uppercase()),
                            GuessResult::Incorrect => {
                                format!("No {}s - locked out!", letter.to_uppercase())
                            }
                            GuessResult::AlreadyUsed => "Letter already used".to_string(),
                            GuessResult::InvalidLetter => "Invalid letter".to_string(),
                            _ => "Error".to_string(),
                        };
                        toast!(socket, &msg);
                        let _ = socket.to(req.room.clone()).emit("toast", &serde_json::json!({ "msg": &msg }));
                        broadcast_state!(socket, req.room, game.get_state());
                    }
                }
            },
        );

        // ========== FINAL ROUND ==========

        // Start final round
        socket.on(
            "start_final",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }
                    game.start_final();
                    toast!(socket, "Final round started! Pick 3 consonants and 1 vowel.");
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // End final round
        socket.on(
            "end_final",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }
                    game.end_final();
                    toast!(socket, "Final round ended.");
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // Final pick (consonant or vowel)
        socket.on(
            "final_pick",
            |socket: SocketRef, Data(req): Data<FinalPickRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_active_player(socket.id.as_str(), false) {
                        toast!(socket, "Only the active player can pick.");
                        return;
                    }

                    if let Some(letter) = req.letter.chars().next() {
                        let result = if req.kind == "consonant" {
                            game.final_pick_consonant(letter)
                        } else if req.kind == "vowel" {
                            game.final_pick_vowel(letter)
                        } else {
                            Err("Invalid pick kind")
                        };

                        match result {
                            Ok(()) => {
                                toast!(socket, &format!("Picked {}: {}", req.kind, letter.to_uppercase()));
                                if game.final_all_picks_complete() {
                                    let _ = socket.to(req.room.clone()).emit("toast", &serde_json::json!({ "msg": "All picks complete! Solve now!" }));
                                    toast!(socket, "All picks complete! Solve now!");

                                    // Spawn background task to auto-end final round when timer expires
                                    let state_clone = state.clone();
                                    let room_clone = req.room.clone();
                                    let final_seconds = game.config.final_seconds as u64;
                                    tokio::spawn(async move {
                                        // Wait for timer to expire (plus small buffer)
                                        tokio::time::sleep(Duration::from_secs(final_seconds + 1)).await;

                                        // Check if timer expired and auto-end final round
                                        let mut manager = state_clone.game_manager.write().await;
                                        if let Some(game) = manager.get_room_mut(&room_clone) {
                                            if game.phase == GamePhase::Final && game.final_timer_expired() {
                                                // Timer expired - end final round
                                                let player_name = game.players.get(game.active_idx)
                                                    .map(|p| p.name.clone())
                                                    .unwrap_or_else(|| "Player".to_string());
                                                info!(
                                                    "Final round timer expired for {} in room {}, auto-ending",
                                                    player_name, room_clone
                                                );
                                                game.end_final();

                                                // Broadcast update
                                                if let Some(io) = state_clone.io.get() {
                                                    let msg = format!("Time's up! {} didn't solve in time.", player_name);
                                                    if let Some(ns) = io.of("/") {
                                                        let _ = ns.to(room_clone.clone()).emit("toast", &serde_json::json!({ "msg": msg }));
                                                    }
                                                    if let Some(ns) = io.of("/") {
                                                        let _ = ns.to(room_clone).emit("state", &game.get_state());
                                                    }
                                                }
                                            }
                                        }
                                    });
                                }
                            }
                            Err(msg) => {
                                toast!(socket, msg);
                            }
                        }
                        broadcast_state!(socket, req.room, game.get_state());
                    }
                }
            },
        );

        // Final pick consonants (web client sends array of consonants)
        socket.on(
            "final_pick_consonant",
            |socket: SocketRef, Data(req): Data<FinalPickConsonantsRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_active_player(socket.id.as_str(), false) {
                        toast!(socket, "Only the active player can pick.");
                        return;
                    }

                    for letter in &req.letters {
                        if let Some(l) = letter.chars().next() {
                            let _ = game.final_pick_consonant(l);
                        }
                    }
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // Final pick vowel (web client sends single vowel)
        socket.on(
            "final_pick_vowel",
            |socket: SocketRef, Data(req): Data<FinalPickVowelRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_active_player(socket.id.as_str(), false) {
                        toast!(socket, "Only the active player can pick.");
                        return;
                    }

                    if let Some(letter) = req.letter.chars().next() {
                        let _ = game.final_pick_vowel(letter);
                    }
                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // Final start timer (web client requests to start the solve timer)
        socket.on(
            "final_start_timer",
            |socket: SocketRef, Data(req): Data<RoomRequest>, State(state): State<Arc<AppState>>| async move {
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_active_player(socket.id.as_str(), false) {
                        toast!(socket, "Only the active player can start the timer.");
                        return;
                    }

                    if game.phase != GamePhase::Final {
                        toast!(socket, "Not in final round.");
                        return;
                    }

                    if !game.final_all_picks_complete() {
                        toast!(socket, "Complete all picks first.");
                        return;
                    }

                    // Reveal the picked letters and start the timer
                    game.final_start_solve();

                    let _ = socket.to(req.room.clone()).emit("toast", &serde_json::json!({ "msg": "Time to solve!" }));
                    toast!(socket, "Time to solve!");

                    // Spawn background task to auto-end final round when timer expires
                    let state_clone = state.clone();
                    let room_clone = req.room.clone();
                    let final_seconds = game.config.final_seconds as u64;
                    tokio::spawn(async move {
                        // Wait for timer to expire (plus small buffer)
                        tokio::time::sleep(Duration::from_secs(final_seconds + 1)).await;

                        // Check if timer expired and auto-end final round
                        let mut manager = state_clone.game_manager.write().await;
                        if let Some(game) = manager.get_room_mut(&room_clone) {
                            if game.phase == GamePhase::Final && game.final_timer_expired() {
                                // Timer expired - end final round
                                let player_name = game.players.get(game.active_idx)
                                    .map(|p| p.name.clone())
                                    .unwrap_or_else(|| "Player".to_string());
                                info!(
                                    "Final round timer expired for {} in room {}, auto-ending",
                                    player_name, room_clone
                                );
                                game.end_final();

                                // Broadcast update
                                if let Some(io) = state_clone.io.get() {
                                    let msg = format!("Time's up! {} didn't solve in time.", player_name);
                                    if let Some(ns) = io.of("/") {
                                        let _ = ns.to(room_clone.clone()).emit("toast", &serde_json::json!({ "msg": msg }));
                                    }
                                    if let Some(ns) = io.of("/") {
                                        let _ = ns.to(room_clone).emit("state", &game.get_state());
                                    }
                                }
                            }
                        }
                    });

                    broadcast_state!(socket, req.room, game.get_state());
                }
            },
        );

        // ========== DISCONNECT ==========

        socket.on_disconnect(|socket: SocketRef, State(state): State<Arc<AppState>>| async move {
            info!("Client disconnected: {}", socket.id);

            // Remove socket from user_sockets tracking
            {
                let mut user_sockets = state.user_sockets.write().await;
                for sockets in user_sockets.values_mut() {
                    sockets.remove(socket.id.as_str());
                }
                // Clean up empty entries
                user_sockets.retain(|_, sockets| !sockets.is_empty());
            }

            let mut manager = state.game_manager.write().await;

            // Find and update rooms where this socket was connected
            let rooms_to_update: Vec<String> = manager
                .rooms
                .iter()
                .filter(|(_, game)| {
                    game.host_sid.as_deref() == Some(socket.id.as_str())
                        || game.players.iter().any(|p| p.socket_id.as_deref() == Some(socket.id.as_str()))
                        || game.tossup.controller_sid.as_deref() == Some(socket.id.as_str())
                })
                .map(|(name, _)| name.clone())
                .collect();

            for room_name in rooms_to_update {
                if let Some(game) = manager.get_room_mut(&room_name) {
                    // Clear host if disconnected
                    if game.host_sid.as_deref() == Some(socket.id.as_str()) {
                        game.host_sid = None;
                    }

                    // Clear tossup controller if disconnected
                    if game.tossup.controller_sid.as_deref() == Some(socket.id.as_str()) {
                        game.tossup.controller_sid = None;
                    }

                    // Remove player (keep their slot but clear socket and track disconnect time)
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    for player in &mut game.players {
                        if player.socket_id.as_deref() == Some(socket.id.as_str()) {
                            player.socket_id = None;
                            player.disconnected_at = Some(now);
                            info!("Player {} disconnected from room {}", player.name, room_name);
                        }
                    }

                    // Remove from tossup locked list
                    game.tossup.locked_sids.remove(socket.id.as_str());
                }
            }

            // Cleanup empty rooms
            manager.cleanup_empty_rooms();
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::state::{GamePhase, Puzzle, TOSSUP_AWARD};
    use crate::game::GameManager;

    // ========== TEST FIXTURES ==========

    /// Create a test GameManager with an empty room
    fn create_test_manager() -> GameManager {
        GameManager::new()
    }

    /// Create a test GameManager with a room containing a puzzle
    fn create_test_manager_with_room(room: &str) -> GameManager {
        let mut manager = GameManager::new();
        let game = manager.get_or_create_room(room);
        game.puzzle = Puzzle {
            id: 1,
            category: "Phrase".to_string(),
            answer: "HELLO WORLD".to_string(),
        };
        manager
    }

    /// Create a test GameManager with a room and players
    fn create_test_manager_with_players(room: &str) -> GameManager {
        let mut manager = create_test_manager_with_room(room);
        let game = manager.get_room_mut(room).unwrap();
        game.add_player("Player 1".to_string(), Some("socket1".to_string()), None, None);
        game.add_player("Player 2".to_string(), Some("socket2".to_string()), None, None);
        game.add_player("Player 3".to_string(), Some("socket3".to_string()), None, None);
        manager
    }

    // ========== JOIN ROOM TESTS ==========

    #[test]
    fn test_handle_join_room_nonexistent() {
        let manager = create_test_manager();
        let result = handle_join_room(&manager, "nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_handle_join_room_existing() {
        let manager = create_test_manager_with_room("test-room");
        let result = handle_join_room(&manager, "test-room");
        assert!(result.is_some());
        let state = result.unwrap();
        assert_eq!(state.room, "test-room");
        assert_eq!(state.puzzle.answer, "HELLO WORLD");
    }

    #[test]
    fn test_handle_join_room_returns_player_state() {
        let manager = create_test_manager_with_players("test-room");
        let result = handle_join_room(&manager, "test-room");
        assert!(result.is_some());
        let state = result.unwrap();
        assert_eq!(state.players.len(), 3);
        assert_eq!(state.players[0].name, "Player 1");
    }

    // ========== JOIN GAME TESTS ==========

    #[test]
    fn test_handle_join_game_new_player() {
        let mut manager = create_test_manager();
        let result = handle_join_game(&mut manager, "test-room", "socket1", Some("Alice".to_string()));

        assert!(result.is_ok());
        let (idx, is_reconnect, name) = result.unwrap();
        assert_eq!(idx, 0);
        assert!(!is_reconnect);
        assert_eq!(name, "Alice");

        // Verify player was added
        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.players.len(), 1);
        assert_eq!(game.players[0].name, "Alice");
        assert_eq!(game.players[0].socket_id, Some("socket1".to_string()));
    }

    #[test]
    fn test_handle_join_game_default_name() {
        let mut manager = create_test_manager();
        let result = handle_join_game(&mut manager, "test-room", "socket1", None);

        assert!(result.is_ok());
        let (_, _, name) = result.unwrap();
        assert_eq!(name, "Player 1");
    }

    #[test]
    fn test_handle_join_game_multiple_players() {
        let mut manager = create_test_manager();

        let result1 = handle_join_game(&mut manager, "test-room", "socket1", Some("Alice".to_string()));
        let result2 = handle_join_game(&mut manager, "test-room", "socket2", Some("Bob".to_string()));

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let (idx1, _, _) = result1.unwrap();
        let (idx2, _, _) = result2.unwrap();

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);

        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.players.len(), 2);
    }

    #[test]
    fn test_handle_join_game_same_socket_returns_existing() {
        let mut manager = create_test_manager();

        // First join
        let result1 = handle_join_game(&mut manager, "test-room", "socket1", Some("Alice".to_string()));
        assert!(result1.is_ok());
        let (idx1, is_reconnect1, _) = result1.unwrap();
        assert!(!is_reconnect1);

        // Second join with same socket - should return existing player
        let result2 = handle_join_game(&mut manager, "test-room", "socket1", Some("Different Name".to_string()));
        assert!(result2.is_ok());
        let (idx2, is_reconnect2, name2) = result2.unwrap();

        assert_eq!(idx1, idx2);
        assert!(is_reconnect2);
        assert_eq!(name2, "Alice"); // Original name preserved

        // Should still only have one player
        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.players.len(), 1);
    }

    #[test]
    fn test_handle_join_game_reconnect_disconnected_player() {
        let mut manager = create_test_manager();

        // Add player then disconnect them
        {
            let game = manager.get_or_create_room("test-room");
            game.add_player("Alice".to_string(), Some("old-socket".to_string()), None, None);
            // Simulate disconnect
            game.players[0].socket_id = None;
            game.players[0].disconnected_at = Some(12345);
        }

        // Reconnect with same name, new socket
        let result = handle_join_game(&mut manager, "test-room", "new-socket", Some("Alice".to_string()));
        assert!(result.is_ok());
        let (idx, is_reconnect, name) = result.unwrap();

        assert_eq!(idx, 0);
        assert!(is_reconnect);
        assert_eq!(name, "Alice");

        // Verify reconnection
        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.players.len(), 1);
        assert_eq!(game.players[0].socket_id, Some("new-socket".to_string()));
        assert!(game.players[0].disconnected_at.is_none());
    }

    #[test]
    fn test_handle_join_game_creates_room_if_needed() {
        let mut manager = create_test_manager();
        assert!(manager.get_room("new-room").is_none());

        let result = handle_join_game(&mut manager, "new-room", "socket1", Some("Player".to_string()));
        assert!(result.is_ok());

        assert!(manager.get_room("new-room").is_some());
    }

    // ========== CLAIM HOST TESTS ==========

    #[test]
    fn test_handle_claim_host_correct_code() {
        let mut manager = create_test_manager();
        let result = handle_claim_host(&mut manager, "test-room", "socket1", "holiday", "holiday");

        assert!(result);
        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.host_sid, Some("socket1".to_string()));
    }

    #[test]
    fn test_handle_claim_host_wrong_code() {
        let mut manager = create_test_manager();
        let result = handle_claim_host(&mut manager, "test-room", "socket1", "wrong", "holiday");

        assert!(!result);
        let game = manager.get_room("test-room");
        // Room might not exist since claim failed before creating it
        assert!(game.is_none() || game.unwrap().host_sid.is_none());
    }

    #[test]
    fn test_handle_claim_host_replaces_existing_host() {
        let mut manager = create_test_manager();

        // First host
        handle_claim_host(&mut manager, "test-room", "socket1", "holiday", "holiday");
        // New host
        let result = handle_claim_host(&mut manager, "test-room", "socket2", "holiday", "holiday");

        assert!(result);
        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.host_sid, Some("socket2".to_string()));
    }

    // ========== SPIN WHEEL TESTS ==========

    #[test]
    fn test_handle_spin_active_player() {
        let mut manager = create_test_manager_with_players("test-room");
        let result = handle_spin(&mut manager, "test-room", "socket1");

        assert!(result.is_ok());
        let (wedge, msg) = result.unwrap();

        // Verify message format based on wedge type
        match wedge {
            super::super::WedgeValue::Cash(amount) => {
                assert!(msg.contains(&format!("${}", amount)));
            }
            super::super::WedgeValue::Bankrupt => {
                assert!(msg.contains("BANKRUPT"));
            }
            super::super::WedgeValue::LoseTurn => {
                assert!(msg.contains("LOSE A TURN"));
            }
            super::super::WedgeValue::FreePlay => {
                assert!(msg.contains("FREE PLAY"));
            }
            super::super::WedgeValue::Prize { name, .. } => {
                assert!(msg.contains(&name));
            }
        }
    }

    #[test]
    fn test_handle_spin_host_can_spin() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.host_sid = Some("host-socket".to_string());
        }

        let result = handle_spin(&mut manager, "test-room", "host-socket");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_spin_non_active_player_rejected() {
        let mut manager = create_test_manager_with_players("test-room");
        // Player 1 (socket1) is active, try to spin as Player 2
        let result = handle_spin(&mut manager, "test-room", "socket2");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Only the active player (or host) can spin.");
    }

    #[test]
    fn test_handle_spin_wrong_phase() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.phase = GamePhase::Tossup;
        }

        let result = handle_spin(&mut manager, "test-room", "socket1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Spin is only allowed during normal rounds.");
    }

    #[test]
    fn test_handle_spin_room_not_found() {
        let mut manager = create_test_manager();
        let result = handle_spin(&mut manager, "nonexistent", "socket1");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Room not found");
    }

    // ========== GUESS LETTER TESTS ==========

    #[test]
    fn test_handle_guess_correct_consonant() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.current_wedge = Some(super::super::WedgeValue::Cash(500));
        }

        let result = handle_guess(&mut manager, "test-room", "socket1", 'H');
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("H(s)!")); // 1 H in HELLO WORLD

        // Verify letter was revealed
        let game = manager.get_room("test-room").unwrap();
        assert!(game.revealed.contains(&'H'));
    }

    #[test]
    fn test_handle_guess_incorrect_consonant() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.current_wedge = Some(super::super::WedgeValue::Cash(500));
        }

        let result = handle_guess(&mut manager, "test-room", "socket1", 'X');
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("No Xs"));

        // Turn should advance
        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.active_idx, 1);
    }

    #[test]
    fn test_handle_guess_vowel_rejected() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.current_wedge = Some(super::super::WedgeValue::Cash(500));
        }

        let result = handle_guess(&mut manager, "test-room", "socket1", 'A');
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("must be a consonant"));
    }

    #[test]
    fn test_handle_guess_already_used() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.current_wedge = Some(super::super::WedgeValue::Cash(500));
            game.used_letters.insert('H');
        }

        let result = handle_guess(&mut manager, "test-room", "socket1", 'H');
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("already used"));
    }

    #[test]
    fn test_handle_guess_need_to_spin() {
        let mut manager = create_test_manager_with_players("test-room");
        // No current_wedge set (hasn't spun)

        let result = handle_guess(&mut manager, "test-room", "socket1", 'H');
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("Spin before"));
    }

    #[test]
    fn test_handle_guess_wrong_player() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.current_wedge = Some(super::super::WedgeValue::Cash(500));
        }

        // Player 1 (socket1) is active, try to guess as Player 2
        let result = handle_guess(&mut manager, "test-room", "socket2", 'H');
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Only the active player can guess.");
    }

    #[test]
    fn test_handle_guess_wrong_phase() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.phase = GamePhase::Tossup;
            game.current_wedge = Some(super::super::WedgeValue::Cash(500));
        }

        let result = handle_guess(&mut manager, "test-room", "socket1", 'H');
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("only allowed during normal rounds"));
    }

    // ========== SOLVE PUZZLE TESTS ==========

    #[test]
    fn test_handle_solve_correct_normal_phase() {
        let mut manager = create_test_manager_with_players("test-room");

        let result = handle_solve(&mut manager, "test-room", "socket1", "HELLO WORLD");
        assert!(result.is_ok());

        let (solved, msg, delay) = result.unwrap();
        assert!(solved);
        assert!(msg.contains("Correct"));
        assert!(delay.is_some());

        // Verify puzzle is solved
        let game = manager.get_room("test-room").unwrap();
        assert!(game.is_solved());
    }

    #[test]
    fn test_handle_solve_incorrect_normal_phase() {
        let mut manager = create_test_manager_with_players("test-room");

        let result = handle_solve(&mut manager, "test-room", "socket1", "WRONG ANSWER");
        assert!(result.is_ok());

        let (solved, msg, delay) = result.unwrap();
        assert!(!solved);
        assert!(msg.contains("Incorrect"));
        assert!(delay.is_none());

        // Verify turn advanced
        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.active_idx, 1);
    }

    #[test]
    fn test_handle_solve_wrong_player() {
        let mut manager = create_test_manager_with_players("test-room");

        let result = handle_solve(&mut manager, "test-room", "socket2", "HELLO WORLD");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Only the active player can solve.");
    }

    #[test]
    fn test_handle_solve_tossup_phase_correct() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.start_tossup();
            game.tossup.controller_sid = Some("socket2".to_string());
            game.active_idx = 1;
        }

        let result = handle_solve(&mut manager, "test-room", "socket2", "HELLO WORLD");
        assert!(result.is_ok());

        let (solved, msg, delay) = result.unwrap();
        assert!(solved);
        assert!(msg.contains("toss-up"));
        assert!(delay.is_some());

        // Verify tossup award
        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.players[1].total, TOSSUP_AWARD);
    }

    #[test]
    fn test_handle_solve_tossup_phase_incorrect() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.start_tossup();
            game.tossup.controller_sid = Some("socket2".to_string());
            game.active_idx = 1;
        }

        let result = handle_solve(&mut manager, "test-room", "socket2", "WRONG");
        assert!(result.is_ok());

        let (solved, msg, _) = result.unwrap();
        assert!(!solved);
        assert!(msg.contains("locked out"));

        // Verify player is locked out
        let game = manager.get_room("test-room").unwrap();
        assert!(game.tossup.locked_sids.contains("socket2"));
        assert!(game.tossup.controller_sid.is_none());
    }

    #[test]
    fn test_handle_solve_tossup_wrong_controller() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.start_tossup();
            game.tossup.controller_sid = Some("socket2".to_string());
        }

        // socket1 tries to solve but socket2 has control
        let result = handle_solve(&mut manager, "test-room", "socket1", "HELLO WORLD");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Only the player who buzzed in can solve.");
    }

    #[test]
    fn test_handle_solve_final_phase_correct() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.start_final();
            game.final_state.stage = super::super::state::FinalStage::Running;
        }

        let result = handle_solve(&mut manager, "test-room", "socket1", "HELLO WORLD");
        assert!(result.is_ok());

        let (solved, msg, delay) = result.unwrap();
        assert!(solved);
        assert!(msg.contains("jackpot"));
        assert!(delay.is_none()); // No auto-advance for final

        // Verify jackpot awarded
        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.players[0].total, game.config.final_jackpot);
    }

    // ========== BUZZ (TOSS-UP) TESTS ==========

    #[test]
    fn test_handle_buzz_success() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.start_tossup();
        }

        let result = handle_buzz(&mut manager, "test-room", "socket2");
        assert!(result.is_ok());

        let (player_idx, name) = result.unwrap();
        assert_eq!(player_idx, 1);
        assert_eq!(name, "Player 2");

        // Verify game state
        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.tossup.controller_sid, Some("socket2".to_string()));
        assert_eq!(game.active_idx, 1);
    }

    #[test]
    fn test_handle_buzz_not_in_tossup() {
        let mut manager = create_test_manager_with_players("test-room");
        // Phase is Normal, not Tossup

        let result = handle_buzz(&mut manager, "test-room", "socket1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not in toss-up mode");
    }

    #[test]
    fn test_handle_buzz_already_locked_out() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.start_tossup();
            game.tossup.locked_sids.insert("socket1".to_string());
        }

        let result = handle_buzz(&mut manager, "test-room", "socket1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "You are locked out for this toss-up");
    }

    #[test]
    fn test_handle_buzz_someone_already_buzzed() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.start_tossup();
            game.tossup.controller_sid = Some("socket1".to_string());
        }

        let result = handle_buzz(&mut manager, "test-room", "socket2");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Someone else already buzzed in");
    }

    #[test]
    fn test_handle_buzz_not_a_player() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.start_tossup();
        }

        // Non-existent socket
        let result = handle_buzz(&mut manager, "test-room", "unknown-socket");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "You must claim a player slot first");
    }

    #[test]
    fn test_handle_buzz_room_not_found() {
        let mut manager = create_test_manager();
        let result = handle_buzz(&mut manager, "nonexistent", "socket1");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Room not found");
    }

    // ========== SCORING INTEGRATION TESTS ==========

    #[test]
    fn test_spin_then_guess_awards_money() {
        let mut manager = create_test_manager_with_players("test-room");

        // Spin the wheel
        let spin_result = handle_spin(&mut manager, "test-room", "socket1");
        assert!(spin_result.is_ok());

        // Get the wedge value to verify scoring later
        let (wedge, _) = spin_result.unwrap();

        // If it's a cash wedge, guess a correct letter
        if let super::super::WedgeValue::Cash(amount) = wedge {
            let guess_result = handle_guess(&mut manager, "test-room", "socket1", 'L');
            assert!(guess_result.is_ok());

            // L appears 3 times in "HELLO WORLD"
            let game = manager.get_room("test-room").unwrap();
            assert_eq!(game.players[0].round_bank, amount * 3);
        }
    }

    #[test]
    fn test_solve_awards_round_bank() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.players[0].round_bank = 5000;
        }

        let result = handle_solve(&mut manager, "test-room", "socket1", "HELLO WORLD");
        assert!(result.is_ok());
        assert!(result.unwrap().0);

        let game = manager.get_room("test-room").unwrap();
        assert_eq!(game.players[0].total, 5000);
        assert_eq!(game.players[0].round_bank, 0);
    }

    // ========== EDGE CASES ==========

    #[test]
    fn test_handle_guess_lowercase_normalized() {
        let mut manager = create_test_manager_with_players("test-room");
        {
            let game = manager.get_room_mut("test-room").unwrap();
            game.current_wedge = Some(super::super::WedgeValue::Cash(500));
        }

        let result = handle_guess(&mut manager, "test-room", "socket1", 'h'); // lowercase
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("H(s)!")); // Uppercase in message

        let game = manager.get_room("test-room").unwrap();
        assert!(game.revealed.contains(&'H'));
    }

    #[test]
    fn test_handle_solve_case_insensitive() {
        let mut manager = create_test_manager_with_players("test-room");

        let result = handle_solve(&mut manager, "test-room", "socket1", "hello world");
        assert!(result.is_ok());
        assert!(result.unwrap().0);
    }

    #[test]
    fn test_multiple_rooms_isolated() {
        let mut manager = create_test_manager();

        // Join different rooms
        handle_join_game(&mut manager, "room-a", "socket1", Some("Alice".to_string())).unwrap();
        handle_join_game(&mut manager, "room-b", "socket2", Some("Bob".to_string())).unwrap();

        // Verify rooms are separate
        let room_a = manager.get_room("room-a").unwrap();
        let room_b = manager.get_room("room-b").unwrap();

        assert_eq!(room_a.players.len(), 1);
        assert_eq!(room_b.players.len(), 1);
        assert_eq!(room_a.players[0].name, "Alice");
        assert_eq!(room_b.players[0].name, "Bob");
    }

    // ========== AUTH RESPONSE TESTS ==========

    #[test]
    fn test_auth_response_success_serialization() {
        let response = super::AuthResponse {
            ok: true,
            user_id: Some(42),
            display_name: Some("TestUser".to_string()),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ok\":true"));
        assert!(json.contains("\"user_id\":42"));
        assert!(json.contains("\"display_name\":\"TestUser\""));
        // error should be skipped when None
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_auth_response_error_serialization() {
        let response = super::AuthResponse {
            ok: false,
            user_id: None,
            display_name: None,
            error: Some("Invalid token".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ok\":false"));
        assert!(json.contains("\"error\":\"Invalid token\""));
        // user_id and display_name should be skipped when None
        assert!(!json.contains("\"user_id\""));
        assert!(!json.contains("\"display_name\""));
    }

    #[test]
    fn test_auth_request_deserialization() {
        let json = r#"{"token": "test-token-123"}"#;
        let req: super::AuthRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.token, "test-token-123");
    }

    // ========== USER ID ASSOCIATION TESTS ==========

    #[test]
    fn test_player_with_user_id() {
        let mut manager = create_test_manager();
        let game = manager.get_or_create_room("test-room");

        // Add player with user_id
        let player_idx = game.add_player(
            "AuthUser".to_string(),
            Some("socket1".to_string()),
            Some(123),
            Some(5),
        );

        assert_eq!(player_idx, 0);
        assert_eq!(game.players[0].user_id, Some(123));
        assert_eq!(game.players[0].name, "AuthUser");
    }

    #[test]
    fn test_reconnect_by_user_id() {
        let mut manager = create_test_manager();
        let game = manager.get_or_create_room("test-room");

        // Add player with user_id
        game.add_player(
            "AuthUser".to_string(),
            Some("socket1".to_string()),
            Some(123),
            Some(5),
        );

        // Disconnect the player
        game.players[0].socket_id = None;
        game.players[0].disconnected_at = Some(1000);

        // Verify player is disconnected
        assert!(game.players[0].socket_id.is_none());

        // Player can reconnect - find by user_id
        let reconnect_idx = game.players.iter().position(|p| {
            p.user_id == Some(123) && p.socket_id.is_none()
        });

        assert_eq!(reconnect_idx, Some(0));

        // Simulate reconnection
        if let Some(idx) = reconnect_idx {
            game.players[idx].socket_id = Some("socket2".to_string());
            game.players[idx].disconnected_at = None;
        }

        assert_eq!(game.players[0].socket_id, Some("socket2".to_string()));
        assert!(game.players[0].disconnected_at.is_none());
    }

    #[test]
    fn test_player_without_user_id() {
        let mut manager = create_test_manager();
        let game = manager.get_or_create_room("test-room");

        // Add player without user_id (anonymous/unauthenticated)
        let player_idx = game.add_player(
            "GuestPlayer".to_string(),
            Some("socket1".to_string()),
            None,
            None,
        );

        assert_eq!(player_idx, 0);
        assert_eq!(game.players[0].user_id, None);
        assert_eq!(game.players[0].name, "GuestPlayer");
    }

    #[test]
    fn test_mixed_authenticated_and_anonymous_players() {
        let mut manager = create_test_manager();
        let game = manager.get_or_create_room("test-room");

        // Add authenticated player
        game.add_player(
            "AuthUser".to_string(),
            Some("socket1".to_string()),
            Some(123),
            Some(5),
        );

        // Add anonymous player
        game.add_player(
            "GuestPlayer".to_string(),
            Some("socket2".to_string()),
            None,
            None,
        );

        assert_eq!(game.players.len(), 2);
        assert_eq!(game.players[0].user_id, Some(123));
        assert_eq!(game.players[1].user_id, None);
    }
}
