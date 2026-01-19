use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::wheel::{create_standard_wheel, shuffle_wheel_with_spacing, WedgeValue};

/// Default configuration values
pub const DEFAULT_VOWEL_COST: i32 = 250;
pub const DEFAULT_FINAL_SECONDS: i32 = 30;
pub const DEFAULT_FINAL_JACKPOT: i32 = 10000;
pub const TOSSUP_AWARD: i32 = 1000;
pub const FINAL_RSTLNE: &[char] = &['R', 'S', 'T', 'L', 'N', 'E'];

/// Game phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GamePhase {
    Normal,
    Tossup,
    Final,
}

/// Final round stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FinalStage {
    Off,
    Pick,
    Running,
    Done,
}

/// A prize won by a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prize {
    pub name: String,
    pub value: i32,
}

/// A player in the game
#[derive(Debug, Clone, Serialize)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub total: i32,
    pub prizes: Vec<Prize>,
    pub round_bank: i32,
    pub round_prizes: Vec<Prize>,
    #[serde(skip)]
    pub socket_id: Option<String>,
    #[serde(skip)]
    pub user_id: Option<i64>,
    /// Timestamp when the player disconnected (None if connected)
    #[serde(skip)]
    pub disconnected_at: Option<i64>,
    /// Avatar ID (1-12), default 1
    pub avatar_id: i64,
}

impl Player {
    pub fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            total: 0,
            prizes: Vec::new(),
            round_bank: 0,
            round_prizes: Vec::new(),
            socket_id: None,
            user_id: None,
            disconnected_at: None,
            avatar_id: 1, // Default avatar
        }
    }

    /// Calculate total prize value
    pub fn prize_value_total(&self) -> i32 {
        self.prizes.iter().map(|p| p.value).sum()
    }

    /// Calculate round prize value
    pub fn round_prize_value_total(&self) -> i32 {
        self.round_prizes.iter().map(|p| p.value).sum()
    }

    /// Calculate TV total (cash + prizes)
    pub fn tv_total(&self) -> i32 {
        self.total + self.prize_value_total()
    }
}

/// Current puzzle
#[derive(Debug, Clone, Serialize)]
pub struct Puzzle {
    pub id: i64,
    pub category: String,
    pub answer: String,
}

impl Default for Puzzle {
    fn default() -> Self {
        Self {
            id: 0,
            category: "Phrase".to_string(),
            answer: "JINGLE ALL THE WAY".to_string(),
        }
    }
}

/// Room configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    pub vowel_cost: i32,
    pub final_seconds: i32,
    pub final_jackpot: i32,
    pub prize_replace_cash_values: Vec<i32>,
    pub puzzle_display_seconds: i32,
    pub prize_wedge_names: Vec<String>,
    #[serde(default)]
    pub pack_id: Option<i64>, // None or 0 = all packs
    /// Seconds before disconnected players are removed (0 = never, default 300 = 5 minutes)
    #[serde(default = "default_disconnect_timeout")]
    pub disconnect_timeout_secs: i64,
}

fn default_disconnect_timeout() -> i64 {
    300 // 5 minutes default
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            vowel_cost: DEFAULT_VOWEL_COST,
            final_seconds: DEFAULT_FINAL_SECONDS,
            final_jackpot: DEFAULT_FINAL_JACKPOT,
            prize_replace_cash_values: vec![500, 1000, 1500, 2000, 2500, 3000, 3500],
            puzzle_display_seconds: 30,
            prize_wedge_names: vec!["GIFT CARD".to_string()],
            pack_id: None, // All packs by default
            disconnect_timeout_secs: 300, // 5 minutes
        }
    }
}

/// Toss-up state
#[derive(Debug, Clone, Default)]
pub struct TossupState {
    pub controller_sid: Option<String>,
    pub locked_sids: HashSet<String>,
    pub reveal_order: Vec<char>,
    pub allowed_player_idxs: Vec<usize>,
    pub is_tiebreaker: bool,
}

/// Final round state
#[derive(Debug, Clone)]
pub struct FinalState {
    pub stage: FinalStage,
    pub picks_consonants: Vec<char>,
    pub pick_vowel: Option<char>,
    pub end_ts: Option<f64>,
}

impl Default for FinalState {
    fn default() -> Self {
        Self {
            stage: FinalStage::Off,
            picks_consonants: Vec::new(),
            pick_vowel: None,
            end_ts: None,
        }
    }
}

/// Game state for a room
#[derive(Debug)]
pub struct Game {
    pub room_name: String,
    pub phase: GamePhase,
    pub players: Vec<Player>,
    pub active_idx: usize,
    pub puzzle: Puzzle,
    pub revealed: HashSet<char>,
    pub used_letters: HashSet<char>,
    pub wheel_slots: Vec<WedgeValue>,
    pub wheel_index: Option<usize>,
    pub last_spin_index: Option<usize>,
    pub current_wedge: Option<WedgeValue>,

    // Puzzle solved state
    pub puzzle_solved_by: Option<String>,

    // Host
    pub host_sid: Option<String>,

    // Configuration
    pub config: RoomConfig,
    pub active_pack_id: Option<i64>,

    // Toss-up state
    pub tossup: TossupState,

    // Final round state
    pub final_state: FinalState,
}

impl Game {
    pub fn new(room_name: &str) -> Self {
        let wheel_slots = shuffle_wheel_with_spacing(create_standard_wheel());

        Self {
            room_name: room_name.to_string(),
            phase: GamePhase::Normal,
            players: Vec::new(),
            active_idx: 0,
            puzzle: Puzzle::default(),
            revealed: HashSet::new(),
            used_letters: HashSet::new(),
            wheel_slots,
            wheel_index: None,
            last_spin_index: None,
            current_wedge: None,
            puzzle_solved_by: None,
            host_sid: None,
            config: RoomConfig::default(),
            active_pack_id: None,
            tossup: TossupState::default(),
            final_state: FinalState::default(),
        }
    }

    /// Check if socket is the host
    pub fn is_host(&self, socket_id: &str) -> bool {
        self.host_sid.as_deref() == Some(socket_id)
    }

    /// Check if socket is the active player (or host acting on behalf)
    pub fn is_active_player(&self, socket_id: &str, allow_host: bool) -> bool {
        if allow_host && self.is_host(socket_id) {
            return true;
        }
        self.players
            .get(self.active_idx)
            .map(|p| p.socket_id.as_deref() == Some(socket_id))
            .unwrap_or(false)
    }

    /// Get player index by socket ID
    pub fn player_idx_by_socket(&self, socket_id: &str) -> Option<usize> {
        self.players
            .iter()
            .position(|p| p.socket_id.as_deref() == Some(socket_id))
    }

    /// Add a player to the game
    pub fn add_player(&mut self, name: String, socket_id: Option<String>, user_id: Option<i64>, avatar_id: Option<i64>) -> usize {
        let id = self.players.len();
        let mut player = Player::new(id, name);
        player.socket_id = socket_id;
        player.user_id = user_id;
        player.avatar_id = avatar_id.unwrap_or(1).clamp(1, 12);
        self.players.push(player);
        id
    }

    /// Remove a player by socket ID
    pub fn remove_player_by_socket(&mut self, socket_id: &str) -> Option<Player> {
        if let Some(pos) = self
            .players
            .iter()
            .position(|p| p.socket_id.as_deref() == Some(socket_id))
        {
            let player = self.players.remove(pos);
            // Renumber remaining players
            for (i, p) in self.players.iter_mut().enumerate() {
                p.id = i;
            }
            // Adjust active_idx if needed
            if self.active_idx >= self.players.len() && !self.players.is_empty() {
                self.active_idx = 0;
            }
            Some(player)
        } else {
            None
        }
    }

    /// Remove a player by index
    pub fn remove_player(&mut self, idx: usize) -> Option<Player> {
        if idx >= self.players.len() {
            return None;
        }
        let player = self.players.remove(idx);
        // Renumber remaining players
        for (i, p) in self.players.iter_mut().enumerate() {
            p.id = i;
        }
        // Adjust active_idx if needed
        if self.active_idx >= self.players.len() && !self.players.is_empty() {
            self.active_idx = 0;
        } else if idx < self.active_idx {
            self.active_idx -= 1;
        }
        Some(player)
    }

    /// Remove players who have been disconnected longer than the timeout
    /// Returns the names of removed players
    pub fn cleanup_timed_out_players(&mut self) -> Vec<String> {
        if self.config.disconnect_timeout_secs <= 0 {
            return Vec::new(); // Timeout disabled
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let timeout = self.config.disconnect_timeout_secs;
        let mut removed_names = Vec::new();

        // Find indices of timed-out players (in reverse order for safe removal)
        let timed_out_indices: Vec<usize> = self
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if let Some(disconnected_at) = p.disconnected_at {
                    now - disconnected_at >= timeout
                } else {
                    false
                }
            })
            .map(|(i, _)| i)
            .collect();

        // Remove in reverse order to preserve indices
        for idx in timed_out_indices.into_iter().rev() {
            if let Some(player) = self.remove_player(idx) {
                removed_names.push(player.name);
            }
        }

        removed_names
    }

    /// Spin the wheel
    pub fn spin(&mut self) -> Option<WedgeValue> {
        if self.wheel_slots.is_empty() {
            return None;
        }

        let idx = rand::random::<usize>() % self.wheel_slots.len();
        self.wheel_index = Some(idx);
        self.last_spin_index = Some(idx);
        let wedge = self.wheel_slots[idx].clone();
        self.current_wedge = Some(wedge.clone());

        // Handle special wedges
        match &wedge {
            WedgeValue::Bankrupt => {
                if let Some(player) = self.players.get_mut(self.active_idx) {
                    player.round_bank = 0;
                    player.round_prizes.clear();
                }
                self.clear_turn_state();
                self.advance_turn();
            }
            WedgeValue::LoseTurn => {
                self.clear_turn_state();
                self.advance_turn();
            }
            _ => {}
        }

        Some(wedge)
    }

    /// Check if a letter is a vowel
    pub fn is_vowel(letter: char) -> bool {
        matches!(letter.to_ascii_uppercase(), 'A' | 'E' | 'I' | 'O' | 'U')
    }

    /// Guess a consonant
    pub fn guess_consonant(&mut self, letter: char) -> GuessResult {
        let letter = letter.to_ascii_uppercase();

        if Self::is_vowel(letter) {
            return GuessResult::InvalidLetter;
        }

        if self.used_letters.contains(&letter) {
            return GuessResult::AlreadyUsed;
        }

        if self.current_wedge.is_none() {
            return GuessResult::NeedToSpin;
        }

        self.used_letters.insert(letter);

        let answer_upper = self.puzzle.answer.to_uppercase();
        let count = answer_upper.chars().filter(|&c| c == letter).count();

        if count > 0 {
            self.revealed.insert(letter);

            // Handle different wedge types
            match &self.current_wedge {
                Some(WedgeValue::Cash(amount)) => {
                    if let Some(player) = self.players.get_mut(self.active_idx) {
                        player.round_bank += amount * count as i32;
                    }
                }
                Some(WedgeValue::Prize { name, .. }) => {
                    // Award prize
                    let prize_value = self
                        .config
                        .prize_replace_cash_values
                        .choose(&mut rand::thread_rng())
                        .copied()
                        .unwrap_or(1000);

                    if let Some(player) = self.players.get_mut(self.active_idx) {
                        // Only add if not already won
                        if !player.round_prizes.iter().any(|p| p.name == *name) {
                            player.round_prizes.push(Prize {
                                name: name.clone(),
                                value: prize_value,
                            });
                        }
                    }

                    // Replace prize wedge with cash
                    if let Some(idx) = self.last_spin_index {
                        if idx < self.wheel_slots.len() {
                            let replacement = self
                                .config
                                .prize_replace_cash_values
                                .choose(&mut rand::thread_rng())
                                .copied()
                                .unwrap_or(500);
                            self.wheel_slots[idx] = WedgeValue::Cash(replacement);
                        }
                    }
                }
                Some(WedgeValue::FreePlay) => {
                    // No money awarded, but turn continues
                }
                _ => {}
            }

            self.clear_turn_state();
            GuessResult::Correct(count)
        } else {
            // Wrong guess - handle FREE PLAY specially
            if self.current_wedge != Some(WedgeValue::FreePlay) {
                self.advance_turn();
            }
            self.clear_turn_state();
            GuessResult::Incorrect
        }
    }

    /// Buy a vowel
    pub fn buy_vowel(&mut self, letter: char) -> GuessResult {
        let letter = letter.to_ascii_uppercase();

        if !Self::is_vowel(letter) {
            return GuessResult::InvalidLetter;
        }

        if self.used_letters.contains(&letter) {
            return GuessResult::AlreadyUsed;
        }

        // Check if player has enough money
        if let Some(player) = self.players.get(self.active_idx) {
            if player.round_bank < self.config.vowel_cost {
                return GuessResult::NotEnoughMoney;
            }
        } else {
            return GuessResult::InvalidLetter;
        }

        // Deduct cost
        if let Some(player) = self.players.get_mut(self.active_idx) {
            player.round_bank -= self.config.vowel_cost;
        }

        self.used_letters.insert(letter);

        let answer_upper = self.puzzle.answer.to_uppercase();
        let count = answer_upper.chars().filter(|&c| c == letter).count();

        if count > 0 {
            self.revealed.insert(letter);
            GuessResult::Correct(count)
        } else {
            self.advance_turn();
            GuessResult::Incorrect
        }
    }

    /// Attempt to solve the puzzle
    pub fn solve(&mut self, attempt: &str) -> bool {
        let normalized_answer = self
            .puzzle
            .answer
            .to_uppercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();

        let normalized_attempt = attempt
            .to_uppercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();

        if normalized_answer == normalized_attempt {
            // Reveal all letters
            for c in self.puzzle.answer.chars() {
                if c.is_alphabetic() {
                    self.revealed.insert(c.to_ascii_uppercase());
                }
            }

            // Record who solved it
            if let Some(player) = self.players.get(self.active_idx) {
                self.puzzle_solved_by = Some(player.name.clone());
            }

            // Award round to active player
            self.award_round_to_active();
            true
        } else {
            self.advance_turn();
            false
        }
    }

    /// Award round bank and prizes to active player
    pub fn award_round_to_active(&mut self) {
        if let Some(player) = self.players.get_mut(self.active_idx) {
            player.total += player.round_bank;
            player.round_bank = 0;
            player.prizes.append(&mut player.round_prizes);
        }
    }

    /// Clear turn state
    pub fn clear_turn_state(&mut self) {
        self.current_wedge = None;
        self.wheel_index = None;
        // Note: last_spin_index is NOT cleared - used for prize wedge replacement
    }

    /// Advance to the next player's turn
    pub fn advance_turn(&mut self) {
        self.clear_turn_state();
        if !self.players.is_empty() {
            self.active_idx = (self.active_idx + 1) % self.players.len();
        }
    }

    /// Start a new puzzle
    pub fn new_puzzle(&mut self, puzzle: Puzzle) {
        self.puzzle = puzzle;
        self.revealed.clear();
        self.used_letters.clear();
        self.clear_turn_state();
        self.last_spin_index = None;
        self.puzzle_solved_by = None;

        // Reset round banks
        for player in &mut self.players {
            player.round_bank = 0;
            player.round_prizes.clear();
        }

        // Reshuffle wheel
        self.wheel_slots = shuffle_wheel_with_spacing(create_standard_wheel());
    }

    /// Reveal all letters in the puzzle
    pub fn reveal_all(&mut self) {
        for c in self.puzzle.answer.chars() {
            if c.is_alphabetic() {
                self.revealed.insert(c.to_ascii_uppercase());
            }
        }
    }

    /// Check if puzzle is solved
    pub fn is_solved(&self) -> bool {
        let answer_upper = self.puzzle.answer.to_uppercase();
        answer_upper
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| self.revealed.contains(&c))
    }

    /// Reset the game
    pub fn reset_game(&mut self) {
        for player in &mut self.players {
            player.total = 0;
            player.prizes.clear();
            player.round_bank = 0;
            player.round_prizes.clear();
        }

        self.active_idx = 0;
        self.wheel_slots = shuffle_wheel_with_spacing(create_standard_wheel());
        self.revealed.clear();
        self.used_letters.clear();
        self.clear_turn_state();
        self.last_spin_index = None;

        self.phase = GamePhase::Normal;
        self.tossup = TossupState::default();
        self.final_state = FinalState::default();
    }

    // ========== TOSS-UP METHODS ==========

    /// Start toss-up mode
    pub fn start_tossup(&mut self) {
        self.phase = GamePhase::Tossup;
        self.tossup = TossupState::default();
        self.revealed.clear();
        self.used_letters.clear();
        self.clear_turn_state();
        self.build_tossup_reveal_order();
    }

    /// Build the reveal order for toss-up (randomized letters)
    pub fn build_tossup_reveal_order(&mut self) {
        let answer_upper = self.puzzle.answer.to_uppercase();
        let mut letters: Vec<char> = answer_upper
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect();
        letters.shuffle(&mut rand::thread_rng());
        self.tossup.reveal_order = letters;
    }

    /// Reveal next letter(s) in toss-up
    pub fn tossup_reveal_step(&mut self, n: usize) -> usize {
        let mut newly_revealed = 0;
        for _ in 0..n {
            if let Some(ch) = self.tossup.reveal_order.pop() {
                if !self.revealed.contains(&ch) {
                    self.revealed.insert(ch);
                    newly_revealed += 1;
                }
            } else {
                break;
            }
        }
        newly_revealed
    }

    /// Handle buzz-in during toss-up
    pub fn tossup_buzz(&mut self, socket_id: &str) -> Result<usize, &'static str> {
        if self.phase != GamePhase::Tossup {
            return Err("Not in toss-up mode");
        }

        if self.tossup.locked_sids.contains(socket_id) {
            return Err("You are locked out for this toss-up");
        }

        if self.tossup.controller_sid.is_some() {
            return Err("Someone else already buzzed in");
        }

        let player_idx = self
            .player_idx_by_socket(socket_id)
            .ok_or("You must claim a player slot first")?;

        if !self.tossup.allowed_player_idxs.is_empty()
            && !self.tossup.allowed_player_idxs.contains(&player_idx)
        {
            return Err("You are not allowed to buzz in this round");
        }

        self.tossup.controller_sid = Some(socket_id.to_string());
        self.active_idx = player_idx;
        Ok(player_idx)
    }

    /// Handle wrong toss-up answer
    pub fn tossup_wrong_answer(&mut self) {
        if let Some(sid) = self.tossup.controller_sid.take() {
            self.tossup.locked_sids.insert(sid);
        }
    }

    /// Handle correct toss-up answer
    pub fn tossup_correct_answer(&mut self) {
        // Award toss-up points
        if let Some(player) = self.players.get_mut(self.active_idx) {
            player.total += TOSSUP_AWARD;
        }
        // Reveal all
        self.reveal_all();
    }

    /// End toss-up mode
    pub fn end_tossup(&mut self) {
        self.phase = GamePhase::Normal;
        self.tossup = TossupState::default();
    }

    // ========== FINAL ROUND METHODS ==========

    /// Start final round (pick phase)
    pub fn start_final(&mut self) {
        self.phase = GamePhase::Final;
        self.final_state = FinalState {
            stage: FinalStage::Pick,
            picks_consonants: Vec::new(),
            pick_vowel: None,
            end_ts: None,
        };
        self.clear_turn_state();

        // Auto-reveal RSTLNE
        for &ch in FINAL_RSTLNE {
            self.revealed.insert(ch);
            self.used_letters.insert(ch);
        }
    }

    /// Pick a consonant for final round
    pub fn final_pick_consonant(&mut self, letter: char) -> Result<(), &'static str> {
        let letter = letter.to_ascii_uppercase();

        if self.phase != GamePhase::Final || self.final_state.stage != FinalStage::Pick {
            return Err("Not in final pick phase");
        }

        if Self::is_vowel(letter) {
            return Err("That's a vowel");
        }

        if self.used_letters.contains(&letter) {
            return Err("Letter already used");
        }

        if self.final_state.picks_consonants.len() >= 3 {
            return Err("Already picked 3 consonants");
        }

        self.final_state.picks_consonants.push(letter);
        self.used_letters.insert(letter);

        // Check if all picks complete
        if self.final_all_picks_complete() {
            self.final_start_running();
        }

        Ok(())
    }

    /// Pick a vowel for final round
    pub fn final_pick_vowel(&mut self, letter: char) -> Result<(), &'static str> {
        let letter = letter.to_ascii_uppercase();

        if self.phase != GamePhase::Final || self.final_state.stage != FinalStage::Pick {
            return Err("Not in final pick phase");
        }

        if !Self::is_vowel(letter) {
            return Err("That's not a vowel");
        }

        if self.used_letters.contains(&letter) {
            return Err("Letter already used");
        }

        if self.final_state.pick_vowel.is_some() {
            return Err("Already picked a vowel");
        }

        self.final_state.pick_vowel = Some(letter);
        self.used_letters.insert(letter);

        // Check if all picks complete
        if self.final_all_picks_complete() {
            self.final_start_running();
        }

        Ok(())
    }

    /// Check if all final picks are complete
    pub fn final_all_picks_complete(&self) -> bool {
        self.final_state.picks_consonants.len() >= 3 && self.final_state.pick_vowel.is_some()
    }

    /// Start the running phase of final round
    fn final_start_running(&mut self) {
        // Reveal picked letters
        for &ch in &self.final_state.picks_consonants {
            self.revealed.insert(ch);
        }
        if let Some(ch) = self.final_state.pick_vowel {
            self.revealed.insert(ch);
        }

        // Start timer
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        self.final_state.end_ts = Some(now + self.config.final_seconds as f64);
        self.final_state.stage = FinalStage::Running;
    }

    /// Get remaining seconds in final round
    pub fn final_remaining_seconds(&self) -> Option<i32> {
        self.final_state.end_ts.map(|end_ts| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            (end_ts - now).max(0.0) as i32
        })
    }

    /// Check if final timer expired
    pub fn final_timer_expired(&self) -> bool {
        if let Some(remaining) = self.final_remaining_seconds() {
            remaining <= 0
        } else {
            false
        }
    }

    /// End final round
    pub fn end_final(&mut self) {
        self.phase = GamePhase::Normal;
        self.final_state = FinalState::default();
    }

    /// Solve in final round
    pub fn final_solve(&mut self, attempt: &str) -> bool {
        if self.solve(attempt) {
            // Award jackpot
            if let Some(player) = self.players.get_mut(self.active_idx) {
                player.total += self.config.final_jackpot;
            }
            self.final_state.stage = FinalStage::Done;
            true
        } else {
            false
        }
    }

    /// Get the game state for sending to clients
    pub fn get_state(&self) -> GameState {
        let tossup_controller_idx = self
            .tossup
            .controller_sid
            .as_ref()
            .and_then(|sid| self.player_idx_by_socket(sid));

        let tossup_locked_idxs: Vec<usize> = self
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.socket_id
                    .as_ref()
                    .map(|sid| self.tossup.locked_sids.contains(sid))
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect();

        GameState {
            room: self.room_name.clone(),
            phase: self.phase,
            players: self
                .players
                .iter()
                .map(|p| PlayerState {
                    id: p.id,
                    name: p.name.clone(),
                    total: p.total,
                    prizes: p.prizes.clone(),
                    prize_value_total: p.prize_value_total(),
                    round_bank: p.round_bank,
                    round_prizes: p.round_prizes.clone(),
                    round_prize_value_total: p.round_prize_value_total(),
                    claimed: p.socket_id.is_some(),
                    avatar_id: p.avatar_id,
                })
                .collect(),
            active_idx: self.active_idx,
            puzzle: PuzzleState {
                id: self.puzzle.id,
                category: self.puzzle.category.clone(),
                answer: self.puzzle.answer.clone(),
            },
            revealed: self.revealed.iter().cloned().collect(),
            used: self.used_letters.iter().cloned().collect(),
            current_wedge: self.current_wedge.clone(),
            wheel_index: self.wheel_index,
            wheel_slots: self.wheel_slots.clone(),
            last_spin_index: self.last_spin_index,
            puzzle_solved_by: self.puzzle_solved_by.clone(),
            host: HostState {
                claimed: self.host_sid.is_some(),
            },
            config: self.config.clone(),
            active_pack_id: self.active_pack_id,
            tossup: TossupStateClient {
                controller_player_idx: tossup_controller_idx,
                locked_player_idxs: tossup_locked_idxs,
                allowed_player_idxs: self.tossup.allowed_player_idxs.clone(),
                is_tiebreaker: self.tossup.is_tiebreaker,
            },
            final_round: FinalStateClient {
                stage: self.final_state.stage,
                picks: FinalPicks {
                    consonants: self.final_state.picks_consonants.clone(),
                    vowel: self.final_state.pick_vowel,
                },
                remaining_seconds: self.final_remaining_seconds(),
                jackpot: self.config.final_jackpot,
            },
        }
    }
}

/// Result of a guess attempt
#[derive(Debug)]
pub enum GuessResult {
    Correct(usize),
    Incorrect,
    AlreadyUsed,
    InvalidLetter,
    NotEnoughMoney,
    NeedToSpin,
}

/// Player state for client
#[derive(Debug, Clone, Serialize)]
pub struct PlayerState {
    pub id: usize,
    pub name: String,
    pub total: i32,
    pub prizes: Vec<Prize>,
    pub prize_value_total: i32,
    pub round_bank: i32,
    pub round_prizes: Vec<Prize>,
    pub round_prize_value_total: i32,
    pub claimed: bool,
    pub avatar_id: i64,
}

/// Puzzle state for client
#[derive(Debug, Clone, Serialize)]
pub struct PuzzleState {
    pub id: i64,
    pub category: String,
    pub answer: String,
}

/// Host state for client
#[derive(Debug, Clone, Serialize)]
pub struct HostState {
    pub claimed: bool,
}

/// Toss-up state for client
#[derive(Debug, Clone, Serialize)]
pub struct TossupStateClient {
    pub controller_player_idx: Option<usize>,
    pub locked_player_idxs: Vec<usize>,
    pub allowed_player_idxs: Vec<usize>,
    pub is_tiebreaker: bool,
}

/// Final picks for client
#[derive(Debug, Clone, Serialize)]
pub struct FinalPicks {
    pub consonants: Vec<char>,
    pub vowel: Option<char>,
}

/// Final state for client
#[derive(Debug, Clone, Serialize)]
pub struct FinalStateClient {
    pub stage: FinalStage,
    pub picks: FinalPicks,
    pub remaining_seconds: Option<i32>,
    pub jackpot: i32,
}

/// Game state for sending to clients
#[derive(Debug, Clone, Serialize)]
pub struct GameState {
    pub room: String,
    pub phase: GamePhase,
    pub players: Vec<PlayerState>,
    pub active_idx: usize,
    pub puzzle: PuzzleState,
    pub revealed: Vec<char>,
    pub used: Vec<char>,
    pub current_wedge: Option<WedgeValue>,
    pub wheel_index: Option<usize>,
    pub wheel_slots: Vec<WedgeValue>,
    pub last_spin_index: Option<usize>,
    pub puzzle_solved_by: Option<String>,
    pub host: HostState,
    pub config: RoomConfig,
    pub active_pack_id: Option<i64>,
    pub tossup: TossupStateClient,
    #[serde(rename = "final")]
    pub final_round: FinalStateClient,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test game with a simple puzzle
    fn create_test_game() -> Game {
        let mut game = Game::new("test-room");
        game.puzzle = Puzzle {
            id: 1,
            category: "Phrase".to_string(),
            answer: "HELLO WORLD".to_string(),
        };
        game
    }

    /// Create a test game with players
    fn create_test_game_with_players() -> Game {
        let mut game = create_test_game();
        game.add_player("Player 1".to_string(), Some("socket1".to_string()), None, Some(1));
        game.add_player("Player 2".to_string(), Some("socket2".to_string()), None, Some(2));
        game.add_player("Player 3".to_string(), Some("socket3".to_string()), None, Some(3));
        game
    }

    // ========== is_vowel tests ==========

    #[test]
    fn test_is_vowel_lowercase() {
        assert!(Game::is_vowel('a'));
        assert!(Game::is_vowel('e'));
        assert!(Game::is_vowel('i'));
        assert!(Game::is_vowel('o'));
        assert!(Game::is_vowel('u'));
    }

    #[test]
    fn test_is_vowel_uppercase() {
        assert!(Game::is_vowel('A'));
        assert!(Game::is_vowel('E'));
        assert!(Game::is_vowel('I'));
        assert!(Game::is_vowel('O'));
        assert!(Game::is_vowel('U'));
    }

    #[test]
    fn test_is_consonant() {
        assert!(!Game::is_vowel('B'));
        assert!(!Game::is_vowel('C'));
        assert!(!Game::is_vowel('D'));
        assert!(!Game::is_vowel('X'));
        assert!(!Game::is_vowel('Z'));
    }

    // ========== solve tests ==========

    #[test]
    fn test_solve_exact_match() {
        let mut game = create_test_game_with_players();
        assert!(game.solve("HELLO WORLD"));
        assert!(game.is_solved());
    }

    #[test]
    fn test_solve_case_insensitive() {
        let mut game = create_test_game_with_players();
        assert!(game.solve("hello world"));
        assert!(game.is_solved());
    }

    #[test]
    fn test_solve_mixed_case() {
        let mut game = create_test_game_with_players();
        assert!(game.solve("HeLLo WoRLd"));
        assert!(game.is_solved());
    }

    #[test]
    fn test_solve_ignores_punctuation() {
        let mut game = create_test_game_with_players();
        // Should match even with extra punctuation
        assert!(game.solve("HELLO, WORLD!"));
        assert!(game.is_solved());
    }

    #[test]
    fn test_solve_ignores_spaces() {
        let mut game = create_test_game_with_players();
        // Should match without spaces
        assert!(game.solve("HELLOWORLD"));
        assert!(game.is_solved());
    }

    #[test]
    fn test_solve_wrong_answer() {
        let mut game = create_test_game_with_players();
        let initial_active = game.active_idx;
        assert!(!game.solve("WRONG ANSWER"));
        assert!(!game.is_solved());
        // Turn should advance on wrong solve
        assert_ne!(game.active_idx, initial_active);
    }

    #[test]
    fn test_solve_partial_answer() {
        let mut game = create_test_game_with_players();
        assert!(!game.solve("HELLO"));
        assert!(!game.is_solved());
    }

    #[test]
    fn test_solve_awards_round_bank() {
        let mut game = create_test_game_with_players();
        // Give player some round bank
        game.players[0].round_bank = 500;
        game.active_idx = 0;

        assert!(game.solve("HELLO WORLD"));

        // Round bank should transfer to total
        assert_eq!(game.players[0].total, 500);
        assert_eq!(game.players[0].round_bank, 0);
    }

    #[test]
    fn test_solve_records_solver_name() {
        let mut game = create_test_game_with_players();
        game.active_idx = 1; // Player 2
        assert!(game.solve("HELLO WORLD"));
        assert_eq!(game.puzzle_solved_by, Some("Player 2".to_string()));
    }

    // ========== guess_consonant tests ==========

    #[test]
    fn test_guess_consonant_correct() {
        let mut game = create_test_game_with_players();
        game.current_wedge = Some(WedgeValue::Cash(500));

        let result = game.guess_consonant('H');
        assert!(matches!(result, GuessResult::Correct(1)));
        assert!(game.revealed.contains(&'H'));
        assert!(game.used_letters.contains(&'H'));
    }

    #[test]
    fn test_guess_consonant_correct_multiple() {
        let mut game = create_test_game_with_players();
        game.current_wedge = Some(WedgeValue::Cash(500));

        let result = game.guess_consonant('L');
        assert!(matches!(result, GuessResult::Correct(3))); // 3 L's in HELLO WORLD
        assert!(game.revealed.contains(&'L'));
    }

    #[test]
    fn test_guess_consonant_awards_money() {
        let mut game = create_test_game_with_players();
        game.current_wedge = Some(WedgeValue::Cash(500));
        game.active_idx = 0;

        game.guess_consonant('L'); // 3 L's
        assert_eq!(game.players[0].round_bank, 1500); // 500 * 3
    }

    #[test]
    fn test_guess_consonant_incorrect() {
        let mut game = create_test_game_with_players();
        game.current_wedge = Some(WedgeValue::Cash(500));
        let initial_active = game.active_idx;

        let result = game.guess_consonant('X');
        assert!(matches!(result, GuessResult::Incorrect));
        assert!(!game.revealed.contains(&'X'));
        assert!(game.used_letters.contains(&'X'));
        // Turn should advance
        assert_ne!(game.active_idx, initial_active);
    }

    #[test]
    fn test_guess_consonant_already_used() {
        let mut game = create_test_game_with_players();
        game.current_wedge = Some(WedgeValue::Cash(500));
        game.used_letters.insert('H');

        let result = game.guess_consonant('H');
        assert!(matches!(result, GuessResult::AlreadyUsed));
    }

    #[test]
    fn test_guess_consonant_vowel_rejected() {
        let mut game = create_test_game_with_players();
        game.current_wedge = Some(WedgeValue::Cash(500));

        let result = game.guess_consonant('A');
        assert!(matches!(result, GuessResult::InvalidLetter));
    }

    #[test]
    fn test_guess_consonant_need_to_spin() {
        let mut game = create_test_game_with_players();
        // No current wedge (hasn't spun)

        let result = game.guess_consonant('H');
        assert!(matches!(result, GuessResult::NeedToSpin));
    }

    #[test]
    fn test_guess_consonant_lowercase_normalized() {
        let mut game = create_test_game_with_players();
        game.current_wedge = Some(WedgeValue::Cash(500));

        let result = game.guess_consonant('h');
        assert!(matches!(result, GuessResult::Correct(1)));
        assert!(game.revealed.contains(&'H'));
    }

    #[test]
    fn test_guess_consonant_free_play_wrong_keeps_turn() {
        let mut game = create_test_game_with_players();
        game.current_wedge = Some(WedgeValue::FreePlay);
        let initial_active = game.active_idx;

        let result = game.guess_consonant('X');
        assert!(matches!(result, GuessResult::Incorrect));
        // Turn should NOT advance on Free Play
        assert_eq!(game.active_idx, initial_active);
    }

    // ========== buy_vowel tests ==========

    #[test]
    fn test_buy_vowel_correct() {
        let mut game = create_test_game_with_players();
        game.players[0].round_bank = 500;
        game.active_idx = 0;

        let result = game.buy_vowel('O');
        assert!(matches!(result, GuessResult::Correct(2))); // 2 O's in HELLO WORLD
        assert!(game.revealed.contains(&'O'));
        assert_eq!(game.players[0].round_bank, 250); // 500 - 250
    }

    #[test]
    fn test_buy_vowel_incorrect() {
        let mut game = create_test_game_with_players();
        game.players[0].round_bank = 500;
        game.active_idx = 0;
        let initial_active = game.active_idx;

        // Change puzzle to not have 'U'
        let result = game.buy_vowel('U');
        assert!(matches!(result, GuessResult::Incorrect));
        assert_eq!(game.players[0].round_bank, 250); // Still charged
        // Turn should advance
        assert_ne!(game.active_idx, initial_active);
    }

    #[test]
    fn test_buy_vowel_not_enough_money() {
        let mut game = create_test_game_with_players();
        game.players[0].round_bank = 100; // Less than 250
        game.active_idx = 0;

        let result = game.buy_vowel('E');
        assert!(matches!(result, GuessResult::NotEnoughMoney));
        assert!(!game.revealed.contains(&'E'));
        assert_eq!(game.players[0].round_bank, 100); // Not charged
    }

    #[test]
    fn test_buy_vowel_consonant_rejected() {
        let mut game = create_test_game_with_players();
        game.players[0].round_bank = 500;
        game.active_idx = 0;

        let result = game.buy_vowel('H');
        assert!(matches!(result, GuessResult::InvalidLetter));
    }

    #[test]
    fn test_buy_vowel_already_used() {
        let mut game = create_test_game_with_players();
        game.players[0].round_bank = 500;
        game.active_idx = 0;
        game.used_letters.insert('E');

        let result = game.buy_vowel('E');
        assert!(matches!(result, GuessResult::AlreadyUsed));
    }

    #[test]
    fn test_buy_vowel_lowercase_normalized() {
        let mut game = create_test_game_with_players();
        game.players[0].round_bank = 500;
        game.active_idx = 0;

        let result = game.buy_vowel('e');
        assert!(matches!(result, GuessResult::Correct(1))); // 1 E in HELLO
        assert!(game.revealed.contains(&'E'));
    }

    // ========== turn management tests ==========

    #[test]
    fn test_advance_turn() {
        let mut game = create_test_game_with_players();
        assert_eq!(game.active_idx, 0);

        game.advance_turn();
        assert_eq!(game.active_idx, 1);

        game.advance_turn();
        assert_eq!(game.active_idx, 2);

        game.advance_turn();
        assert_eq!(game.active_idx, 0); // Wraps around
    }

    #[test]
    fn test_advance_turn_clears_wedge() {
        let mut game = create_test_game_with_players();
        game.current_wedge = Some(WedgeValue::Cash(500));
        game.wheel_index = Some(5);

        game.advance_turn();

        assert!(game.current_wedge.is_none());
        assert!(game.wheel_index.is_none());
    }

    #[test]
    fn test_advance_turn_single_player() {
        let mut game = create_test_game();
        game.add_player("Solo".to_string(), None, None, None);
        assert_eq!(game.active_idx, 0);

        game.advance_turn();
        assert_eq!(game.active_idx, 0); // Stays at 0
    }

    // ========== player management tests ==========

    #[test]
    fn test_add_player() {
        let mut game = create_test_game();
        let idx = game.add_player("Test Player".to_string(), Some("socket123".to_string()), Some(42), Some(7));

        assert_eq!(idx, 0);
        assert_eq!(game.players.len(), 1);
        assert_eq!(game.players[0].name, "Test Player");
        assert_eq!(game.players[0].socket_id, Some("socket123".to_string()));
        assert_eq!(game.players[0].user_id, Some(42));
    }

    #[test]
    fn test_remove_player_renumbers() {
        let mut game = create_test_game_with_players();
        // Remove player 1 (middle)
        game.remove_player(1);

        assert_eq!(game.players.len(), 2);
        assert_eq!(game.players[0].id, 0);
        assert_eq!(game.players[0].name, "Player 1");
        assert_eq!(game.players[1].id, 1);
        assert_eq!(game.players[1].name, "Player 3");
    }

    #[test]
    fn test_remove_player_adjusts_active_idx() {
        let mut game = create_test_game_with_players();
        game.active_idx = 1; // Player 2 is active

        // Remove player before active (at index 0)
        game.remove_player(0);

        // After removal: 2 players remain (P2, P3), active_idx should decrease from 1 to 0
        assert_eq!(game.active_idx, 0); // Adjusted down
    }

    #[test]
    fn test_remove_player_wraps_active_idx() {
        let mut game = create_test_game_with_players();
        game.active_idx = 2; // Last player

        // Remove last player
        game.remove_player(2);

        // active_idx was 2, now >= len (2), so wraps to 0
        assert_eq!(game.active_idx, 0);
    }

    #[test]
    fn test_remove_player_by_socket() {
        let mut game = create_test_game_with_players();

        let removed = game.remove_player_by_socket("socket2");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "Player 2");
        assert_eq!(game.players.len(), 2);
    }

    // ========== scoring tests ==========

    #[test]
    fn test_award_round_to_active() {
        let mut game = create_test_game_with_players();
        game.active_idx = 0;
        game.players[0].round_bank = 5000;
        game.players[0].round_prizes.push(Prize {
            name: "Car".to_string(),
            value: 25000,
        });

        game.award_round_to_active();

        assert_eq!(game.players[0].total, 5000);
        assert_eq!(game.players[0].round_bank, 0);
        assert_eq!(game.players[0].prizes.len(), 1);
        assert_eq!(game.players[0].round_prizes.len(), 0);
    }

    #[test]
    fn test_player_prize_value_total() {
        let mut player = Player::new(0, "Test".to_string());
        player.prizes.push(Prize {
            name: "Prize 1".to_string(),
            value: 1000,
        });
        player.prizes.push(Prize {
            name: "Prize 2".to_string(),
            value: 2000,
        });

        assert_eq!(player.prize_value_total(), 3000);
    }

    #[test]
    fn test_player_tv_total() {
        let mut player = Player::new(0, "Test".to_string());
        player.total = 5000;
        player.prizes.push(Prize {
            name: "Prize".to_string(),
            value: 2000,
        });

        assert_eq!(player.tv_total(), 7000);
    }

    // ========== new_puzzle tests ==========

    #[test]
    fn test_new_puzzle_resets_state() {
        let mut game = create_test_game_with_players();
        game.revealed.insert('H');
        game.used_letters.insert('H');
        game.players[0].round_bank = 1000;
        game.puzzle_solved_by = Some("Player 1".to_string());

        game.new_puzzle(Puzzle {
            id: 2,
            category: "Thing".to_string(),
            answer: "BICYCLE".to_string(),
        });

        assert!(game.revealed.is_empty());
        assert!(game.used_letters.is_empty());
        assert_eq!(game.players[0].round_bank, 0);
        assert!(game.puzzle_solved_by.is_none());
        assert_eq!(game.puzzle.answer, "BICYCLE");
    }

    // ========== is_solved tests ==========

    #[test]
    fn test_is_solved_all_revealed() {
        let mut game = create_test_game();
        // Reveal all letters
        for c in ['H', 'E', 'L', 'O', 'W', 'R', 'D'] {
            game.revealed.insert(c);
        }

        assert!(game.is_solved());
    }

    #[test]
    fn test_is_solved_partial() {
        let mut game = create_test_game();
        game.revealed.insert('H');
        game.revealed.insert('E');

        assert!(!game.is_solved());
    }

    // ========== reveal_all tests ==========

    #[test]
    fn test_reveal_all() {
        let mut game = create_test_game();

        game.reveal_all();

        assert!(game.revealed.contains(&'H'));
        assert!(game.revealed.contains(&'E'));
        assert!(game.revealed.contains(&'L'));
        assert!(game.revealed.contains(&'O'));
        assert!(game.revealed.contains(&'W'));
        assert!(game.revealed.contains(&'R'));
        assert!(game.revealed.contains(&'D'));
    }

    // ========== host tests ==========

    #[test]
    fn test_is_host() {
        let mut game = create_test_game();
        game.host_sid = Some("host123".to_string());

        assert!(game.is_host("host123"));
        assert!(!game.is_host("other"));
    }

    #[test]
    fn test_is_active_player() {
        let mut game = create_test_game_with_players();
        game.active_idx = 1;

        assert!(game.is_active_player("socket2", false));
        assert!(!game.is_active_player("socket1", false));
    }

    #[test]
    fn test_is_active_player_host_allowed() {
        let mut game = create_test_game_with_players();
        game.active_idx = 0;
        game.host_sid = Some("host123".to_string());

        assert!(game.is_active_player("host123", true));
        assert!(!game.is_active_player("host123", false));
    }

    // ========== tossup tests ==========

    #[test]
    fn test_start_tossup() {
        let mut game = create_test_game();

        game.start_tossup();

        assert_eq!(game.phase, GamePhase::Tossup);
        assert!(game.revealed.is_empty());
        assert!(!game.tossup.reveal_order.is_empty());
    }

    #[test]
    fn test_tossup_buzz() {
        let mut game = create_test_game_with_players();
        game.start_tossup();

        let result = game.tossup_buzz("socket2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Player index 1
        assert_eq!(game.active_idx, 1);
        assert_eq!(
            game.tossup.controller_sid,
            Some("socket2".to_string())
        );
    }

    #[test]
    fn test_tossup_buzz_locked_out() {
        let mut game = create_test_game_with_players();
        game.start_tossup();
        game.tossup.locked_sids.insert("socket1".to_string());

        let result = game.tossup_buzz("socket1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "You are locked out for this toss-up");
    }

    #[test]
    fn test_tossup_buzz_already_buzzed() {
        let mut game = create_test_game_with_players();
        game.start_tossup();
        game.tossup.controller_sid = Some("socket1".to_string());

        let result = game.tossup_buzz("socket2");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Someone else already buzzed in");
    }

    #[test]
    fn test_tossup_wrong_answer_locks() {
        let mut game = create_test_game_with_players();
        game.start_tossup();
        game.tossup.controller_sid = Some("socket1".to_string());

        game.tossup_wrong_answer();

        assert!(game.tossup.controller_sid.is_none());
        assert!(game.tossup.locked_sids.contains("socket1"));
    }

    #[test]
    fn test_tossup_correct_answer_awards() {
        let mut game = create_test_game_with_players();
        game.start_tossup();
        game.active_idx = 1;

        game.tossup_correct_answer();

        assert_eq!(game.players[1].total, TOSSUP_AWARD);
        assert!(game.is_solved()); // reveal_all called
    }

    // ========== final round tests ==========

    #[test]
    fn test_start_final() {
        let mut game = create_test_game_with_players();

        game.start_final();

        assert_eq!(game.phase, GamePhase::Final);
        assert_eq!(game.final_state.stage, FinalStage::Pick);
        // RSTLNE should be revealed
        for c in ['R', 'S', 'T', 'L', 'N', 'E'] {
            assert!(game.revealed.contains(&c));
            assert!(game.used_letters.contains(&c));
        }
    }

    #[test]
    fn test_final_pick_consonant() {
        let mut game = create_test_game_with_players();
        game.start_final();

        let result = game.final_pick_consonant('B');
        assert!(result.is_ok());
        assert!(game.final_state.picks_consonants.contains(&'B'));
    }

    #[test]
    fn test_final_pick_consonant_vowel_rejected() {
        let mut game = create_test_game_with_players();
        game.start_final();

        let result = game.final_pick_consonant('A');
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "That's a vowel");
    }

    #[test]
    fn test_final_pick_consonant_already_used() {
        let mut game = create_test_game_with_players();
        game.start_final();
        // R is already used (RSTLNE)

        let result = game.final_pick_consonant('R');
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Letter already used");
    }

    #[test]
    fn test_final_pick_vowel() {
        let mut game = create_test_game_with_players();
        game.start_final();

        let result = game.final_pick_vowel('I');
        assert!(result.is_ok());
        assert_eq!(game.final_state.pick_vowel, Some('I'));
    }

    #[test]
    fn test_final_all_picks_starts_running() {
        let mut game = create_test_game_with_players();
        game.start_final();

        game.final_pick_consonant('B').unwrap();
        game.final_pick_consonant('C').unwrap();
        game.final_pick_consonant('D').unwrap();
        game.final_pick_vowel('I').unwrap();

        assert_eq!(game.final_state.stage, FinalStage::Running);
        assert!(game.final_state.end_ts.is_some());
    }

    #[test]
    fn test_final_solve_awards_jackpot() {
        let mut game = create_test_game_with_players();
        game.start_final();
        game.final_state.stage = FinalStage::Running;
        game.active_idx = 0;

        assert!(game.final_solve("HELLO WORLD"));

        assert_eq!(game.players[0].total, game.config.final_jackpot);
        assert_eq!(game.final_state.stage, FinalStage::Done);
    }

    // ========== reset_game tests ==========

    #[test]
    fn test_reset_game() {
        let mut game = create_test_game_with_players();
        game.players[0].total = 5000;
        game.players[0].round_bank = 1000;
        game.active_idx = 2;
        game.phase = GamePhase::Final;
        game.revealed.insert('H');

        game.reset_game();

        assert_eq!(game.players[0].total, 0);
        assert_eq!(game.players[0].round_bank, 0);
        assert_eq!(game.active_idx, 0);
        assert_eq!(game.phase, GamePhase::Normal);
        assert!(game.revealed.is_empty());
    }

    // ========== edge cases ==========

    #[test]
    fn test_solve_puzzle_with_numbers() {
        let mut game = create_test_game_with_players();
        game.puzzle = Puzzle {
            id: 1,
            category: "Phrase".to_string(),
            answer: "21ST CENTURY".to_string(),
        };

        assert!(game.solve("21st century"));
        assert!(game.is_solved());
    }

    #[test]
    fn test_solve_puzzle_with_apostrophe() {
        let mut game = create_test_game_with_players();
        game.puzzle = Puzzle {
            id: 1,
            category: "Phrase".to_string(),
            answer: "IT'S A WRAP".to_string(),
        };

        assert!(game.solve("its a wrap"));
        assert!(game.is_solved());
    }

    #[test]
    fn test_guess_consonant_on_prize_wedge() {
        let mut game = create_test_game_with_players();
        game.current_wedge = Some(WedgeValue::prize("GIFT CARD"));
        game.last_spin_index = Some(0);
        game.active_idx = 0;

        let result = game.guess_consonant('H');
        assert!(matches!(result, GuessResult::Correct(1)));
        assert!(!game.players[0].round_prizes.is_empty());
    }

    #[test]
    fn test_empty_game_advance_turn() {
        let mut game = create_test_game();
        // No players

        game.advance_turn(); // Should not panic
        assert_eq!(game.active_idx, 0);
    }
}
