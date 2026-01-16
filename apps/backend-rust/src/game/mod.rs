pub mod handlers;
pub mod state;
pub mod wheel;

use std::collections::HashMap;

pub use state::{Game, Puzzle, RoomConfig};
pub use wheel::WedgeValue;

/// Manages all active game rooms
pub struct GameManager {
    pub rooms: HashMap<String, Game>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    /// Get or create a room
    pub fn get_or_create_room(&mut self, room_name: &str) -> &mut Game {
        self.rooms
            .entry(room_name.to_string())
            .or_insert_with(|| Game::new(room_name))
    }

    /// Get a room if it exists
    pub fn get_room(&self, room_name: &str) -> Option<&Game> {
        self.rooms.get(room_name)
    }

    /// Get a mutable room if it exists
    pub fn get_room_mut(&mut self, room_name: &str) -> Option<&mut Game> {
        self.rooms.get_mut(room_name)
    }

    /// List all rooms with player counts
    pub fn list_rooms(&self) -> Vec<RoomInfo> {
        self.rooms
            .iter()
            .map(|(name, game)| RoomInfo {
                name: name.clone(),
                player_count: game.players.len(),
                has_host: game.host_sid.is_some(),
            })
            .collect()
    }

    /// Remove empty rooms
    pub fn cleanup_empty_rooms(&mut self) {
        self.rooms.retain(|_, game| !game.players.is_empty());
    }

    /// Remove a room
    pub fn remove_room(&mut self, room_name: &str) -> Option<Game> {
        self.rooms.remove(room_name)
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Room info for listing
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoomInfo {
    pub name: String,
    pub player_count: usize,
    pub has_host: bool,
}
