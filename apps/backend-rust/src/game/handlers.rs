use std::sync::Arc;

use serde::Deserialize;
use socketioxide::{
    extract::{Data, SocketRef, State},
    SocketIo,
};
use tracing::info;

use crate::AppState;

use super::state::{GamePhase, GuessResult, Puzzle};

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
#[allow(dead_code)]
pub struct SetActivePack {
    pub room: String,
    pub pack_id: Option<i64>,
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

pub fn register_handlers(io: &SocketIo) {
    io.ns("/", |socket: SocketRef, State(_state): State<Arc<AppState>>| {
        info!("Client connected: {}", socket.id);

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
        socket.on(
            "join_game",
            |socket: SocketRef, Data(req): Data<JoinGameRequest>, State(state): State<Arc<AppState>>| async move {
                info!("Socket {} joining game in room {}", socket.id, req.room);
                socket.join(req.room.clone()).ok();

                let mut manager = state.game_manager.write().await;
                let game = manager.get_or_create_room(&req.room);

                // Check if already in game with this socket
                if let Some(idx) = game.player_idx_by_socket(socket.id.as_str()) {
                    socket.emit("you", &serde_json::json!({ "player_idx": idx })).ok();
                    broadcast_state!(socket, req.room, game.get_state());
                    return;
                }

                let name = req.name.unwrap_or_else(|| format!("Player {}", game.players.len() + 1));

                // Check if there's a disconnected player with the same name to reconnect
                let existing_idx = game.players.iter().position(|p| {
                    p.name == name && p.socket_id.is_none()
                });

                if let Some(idx) = existing_idx {
                    // Reconnect to existing player slot
                    game.players[idx].socket_id = Some(socket.id.to_string());
                    socket.emit("you", &serde_json::json!({ "player_idx": idx })).ok();
                    toast!(socket, &format!("Reconnected as {}!", name));
                    broadcast_state!(socket, req.room, game.get_state());
                    return;
                }

                // Add new player
                let player_idx = game.add_player(name.clone(), Some(socket.id.to_string()), None);

                socket.emit("you", &serde_json::json!({ "player_idx": player_idx })).ok();
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
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }

                    // For now, use a default puzzle - will integrate with DB later
                    let puzzle = Puzzle {
                        id: rand::random::<i64>().abs(),
                        category: "Phrase".to_string(),
                        answer: "HAPPY HOLIDAYS".to_string(),
                    };
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
                let mut manager = state.game_manager.write().await;
                if let Some(game) = manager.get_room_mut(&req.room) {
                    if !game.is_host(socket.id.as_str()) {
                        toast!(socket, "Host only.");
                        return;
                    }
                    game.reset_game();
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
                        game.add_player(name[..name.len().min(30)].to_string(), None, None);
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
                    }
                    broadcast_state!(socket, req.room, game.get_state());
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
                                "Correct! Puzzle solved!"
                            } else {
                                "Incorrect, sorry!"
                            };
                            toast!(socket, msg);
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
                                toast!(socket, "Correct! You win the toss-up!");
                            } else {
                                game.tossup_wrong_answer();
                                toast!(socket, "Incorrect! You're locked out.");
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
                        }
                    }
                    broadcast_state!(socket, req.room, game.get_state());
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
                        }
                        Err(msg) => {
                            toast!(socket, msg);
                        }
                    }
                    broadcast_state!(socket, req.room, game.get_state());
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

        // ========== DISCONNECT ==========

        socket.on_disconnect(|socket: SocketRef, State(state): State<Arc<AppState>>| async move {
            info!("Client disconnected: {}", socket.id);

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

                    // Remove player (keep their slot but clear socket)
                    for player in &mut game.players {
                        if player.socket_id.as_deref() == Some(socket.id.as_str()) {
                            player.socket_id = None;
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
