use std::collections::{HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::wheel::{create_standard_wheel, shuffle_wheel_with_spacing, WedgeValue};

/// Default configuration values
pub const DEFAULT_VOWEL_COST: i32 = 250;
pub const DEFAULT_BONUS_SECONDS: i32 = 30;
pub const DEFAULT_BONUS_JACKPOT: i32 = 10000;
pub const TOSSUP_AWARD: i32 = 1000;
pub const BONUS_RSTLNE: &[char] = &['R', 'S', 'T', 'L', 'N', 'E'];

/// Game phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GamePhase {
    Pregame, // Waiting for someone to press "Start Game" before beginning
    Normal,
    Tossup,
    Final,    // Final Spin round - host spins once, players take turns calling letters
    Bonus,    // Bonus round - RSTLNE revealed, pick 3 consonants + 1 vowel, solve
    GameOver, // Game has ended - show results and winner
}

/// Bonus round stage (formerly FinalStage)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BonusStage {
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
    #[serde(alias = "final_seconds")]
    pub bonus_seconds: i32,
    #[serde(alias = "final_jackpot")]
    pub bonus_jackpot: i32,
    pub prize_replace_cash_values: Vec<i32>,
    pub puzzle_display_seconds: i32,
    pub prize_wedge_names: Vec<String>,
    #[serde(default)]
    pub pack_id: Option<i64>, // None or 0 = all packs
    /// Seconds before disconnected players are removed (0 = never, default 300 = 5 minutes)
    #[serde(default = "default_disconnect_timeout")]
    pub disconnect_timeout_secs: i64,
    /// Seconds for turn timer after spin (0 = disabled, default 10)
    #[serde(default = "default_turn_timer")]
    pub turn_timer_seconds: i32,
    /// Seconds for toss-up buzz timer after buzzing in (0 = disabled, default 5)
    #[serde(default = "default_buzz_timer")]
    pub buzz_timer_seconds: i32,
}

fn default_disconnect_timeout() -> i64 {
    300 // 5 minutes default
}

fn default_turn_timer() -> i32 {
    10 // 10 seconds default
}

fn default_buzz_timer() -> i32 {
    5 // 5 seconds default for toss-up buzz timer
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            vowel_cost: DEFAULT_VOWEL_COST,
            bonus_seconds: DEFAULT_BONUS_SECONDS,
            bonus_jackpot: DEFAULT_BONUS_JACKPOT,
            prize_replace_cash_values: vec![500, 1000, 1500, 2000, 2500, 3000, 3500],
            puzzle_display_seconds: 30,
            prize_wedge_names: vec!["GIFT CARD".to_string()],
            pack_id: None,                // All packs by default
            disconnect_timeout_secs: 300, // 5 minutes
            turn_timer_seconds: 10,       // 10 seconds to guess after spin
            buzz_timer_seconds: 5,        // 5 seconds to solve after buzzing in
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
    /// Unix timestamp when buzz timer expires (None = no timer active)
    pub buzz_timer_end_ts: Option<f64>,
}

/// Bonus round state (formerly FinalState)
#[derive(Debug, Clone)]
pub struct BonusState {
    pub stage: BonusStage,
    pub picks_consonants: Vec<char>,
    pub pick_vowel: Option<char>,
    pub end_ts: Option<f64>,
}

impl Default for BonusState {
    fn default() -> Self {
        Self {
            stage: BonusStage::Off,
            picks_consonants: Vec::new(),
            pick_vowel: None,
            end_ts: None,
        }
    }
}

/// Game over state
#[derive(Debug, Clone, Default)]
pub struct GameOverState {
    /// Index of the winning player (highest total score)
    pub winner_idx: Option<usize>,
    /// Winner's name
    pub winner_name: Option<String>,
    /// Winner's total score
    pub winner_score: i32,
    /// Whether the bonus round was won
    pub bonus_won: bool,
    /// Timestamp when game over will auto-reset to pregame (Unix timestamp in seconds)
    pub reset_at: Option<f64>,
}

/// Default game over display time in seconds before auto-reset
pub const GAME_OVER_DISPLAY_SECONDS: u64 = 15;

/// Bonus per letter during Final Spin (added to spin value)
pub const FINAL_SPIN_LETTER_BONUS: i32 = 1000;

/// Final Spin round state
#[derive(Debug, Clone)]
pub struct FinalSpinState {
    /// The fixed spin value for all consonants in this round
    pub spin_value: i32,
    /// Whether the host has done the initial spin
    pub spin_done: bool,
    /// Random threshold: trigger Final Spin after this many consonants guessed in Round 4
    pub trigger_after_consonants: u8,
    /// Counter for consonants guessed in the current round (Round 4)
    pub consonants_guessed: u8,
}

impl Default for FinalSpinState {
    fn default() -> Self {
        Self {
            spin_value: 0,
            spin_done: false,
            trigger_after_consonants: rand::thread_rng().gen_range(5..=10),
            consonants_guessed: 0,
        }
    }
}

/// Game state for a room
#[derive(Debug)]
pub struct Game {
    pub room_name: String,
    pub phase: GamePhase,
    /// Current round number (1-4, where 4 is typically Final Spin round)
    pub round: u8,
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
    pub active_pack_name: Option<String>,

    // Toss-up state
    pub tossup: TossupState,

    // Bonus round state
    pub bonus_state: BonusState,

    // Final Spin round state
    pub final_spin: FinalSpinState,

    // Game over state
    pub game_over_state: GameOverState,

    // Turn timer - timestamp when turn expires (started after spin animation completes)
    pub turn_timer_end_ts: Option<f64>,

    // Spin history for anti-clustering (stores last N wedge indices)
    pub spin_history: VecDeque<usize>,
}

impl Game {
    pub fn new(room_name: &str) -> Self {
        let wheel_slots = shuffle_wheel_with_spacing(create_standard_wheel());

        Self {
            room_name: room_name.to_string(),
            phase: GamePhase::Pregame, // Start in pregame, waiting for someone to start
            round: 1,                  // Start at round 1
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
            active_pack_name: None,
            tossup: TossupState::default(),
            bonus_state: BonusState::default(),
            final_spin: FinalSpinState::default(),
            game_over_state: GameOverState::default(),
            turn_timer_end_ts: None,
            spin_history: VecDeque::with_capacity(8),
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
    pub fn add_player(
        &mut self,
        name: String,
        socket_id: Option<String>,
        user_id: Option<i64>,
        avatar_id: Option<i64>,
    ) -> usize {
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

    /// Spin the wheel with physics-based randomization and anti-clustering
    ///
    /// Uses multiple techniques to ensure fair, varied results:
    /// 1. Proper random range (no modulo bias)
    /// 2. Physics simulation with variable force and friction
    /// 3. Anti-clustering: reduces probability of landing on recently-hit wedges
    /// 4. Entropy from multiple sources for better randomness
    pub fn spin(&mut self) -> Option<WedgeValue> {
        if self.wheel_slots.is_empty() {
            return None;
        }

        let num_slots = self.wheel_slots.len();
        let mut rng = rand::thread_rng();

        // Physics-based spin simulation
        // Generate random force (like different spin strengths from players)
        let min_force = 800.0_f64; // Minimum rotation in degrees
        let max_force = 2200.0_f64; // Maximum rotation in degrees
        let force = rng.gen_range(min_force..max_force);

        // Add friction variation (simulates wheel wear, environmental factors)
        let friction_variation = rng.gen_range(0.85..1.15);
        let effective_rotation = force * friction_variation;

        // Calculate starting position (use last spin position or random start)
        let start_angle = self
            .last_spin_index
            .map(|idx| idx as f64 * 360.0 / num_slots as f64)
            .unwrap_or_else(|| rng.gen_range(0.0..360.0));

        // Calculate final position based on physics
        let final_angle = (start_angle + effective_rotation) % 360.0;
        let angle_per_slot = 360.0 / num_slots as f64;

        // Convert final angle to slot index
        let raw_idx = ((360.0 - final_angle + angle_per_slot / 2.0) % 360.0 / angle_per_slot)
            as usize
            % num_slots;

        // Anti-clustering: if we've hit this wedge recently, consider adjusting
        let idx = self.apply_anti_clustering(raw_idx, &mut rng);

        self.wheel_index = Some(idx);
        self.last_spin_index = Some(idx);

        // Track spin history (keep last 6 spins)
        if self.spin_history.len() >= 6 {
            self.spin_history.pop_front();
        }
        self.spin_history.push_back(idx);

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

    /// Apply anti-clustering to avoid landing on the same wedge repeatedly
    /// Returns the original index most of the time, but may adjust if there's clustering
    fn apply_anti_clustering(&self, raw_idx: usize, rng: &mut impl Rng) -> usize {
        let num_slots = self.wheel_slots.len();
        if num_slots == 0 {
            return raw_idx;
        }

        // Count how many times this exact wedge appears in recent history
        let exact_hits = self
            .spin_history
            .iter()
            .filter(|&&idx| idx == raw_idx)
            .count();

        // Count hits on adjacent wedges (including this one) in last 3 spins
        let recent_window = self.spin_history.iter().rev().take(3);
        let adjacent_hits = recent_window
            .filter(|&&idx| {
                let diff = (raw_idx as i32 - idx as i32).abs();
                diff <= 1 || diff >= (num_slots as i32 - 1) // Adjacent or same
            })
            .count();

        // If we've hit this exact wedge 2+ times in history, or 3+ adjacent hits recently,
        // there's a chance to nudge to a different position
        let should_adjust = if exact_hits >= 2 {
            rng.gen_bool(0.7) // 70% chance to adjust if hit twice
        } else if adjacent_hits >= 2 {
            rng.gen_bool(0.4) // 40% chance to adjust if adjacent clustering
        } else {
            false
        };

        if should_adjust {
            // Find a wedge we haven't hit recently
            let offset = rng.gen_range(3..num_slots - 2); // Move at least 3 positions
            (raw_idx + offset) % num_slots
        } else {
            raw_idx
        }
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

            // Track consonants guessed in Round 4 for Final Spin trigger
            if self.round == 4 && self.phase == GamePhase::Normal {
                self.final_spin.consonants_guessed =
                    self.final_spin.consonants_guessed.saturating_add(1);
            }

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
        self.turn_timer_end_ts = None;
        // Note: last_spin_index is NOT cleared - used for prize wedge replacement
    }

    /// Advance to the next player's turn
    pub fn advance_turn(&mut self) {
        self.clear_turn_state();
        if !self.players.is_empty() {
            self.active_idx = (self.active_idx + 1) % self.players.len();
        }
    }

    // ========== ROUND MANAGEMENT ==========

    /// Get current round number (1-4)
    pub fn current_round(&self) -> u8 {
        self.round
    }

    /// Advance to the next round (caps at round 4)
    pub fn advance_round(&mut self) {
        if self.round < 4 {
            self.round += 1;
        }
    }

    /// Set the round number (1-4)
    pub fn set_round(&mut self, round: u8) {
        self.round = round.clamp(1, 4);
    }

    /// Reset round to 1
    pub fn reset_round(&mut self) {
        self.round = 1;
    }

    /// Check if this is the final spin round (round 4)
    pub fn is_final_spin_round(&self) -> bool {
        self.round == 4
    }

    /// Start a new puzzle
    pub fn new_puzzle(&mut self, puzzle: Puzzle) {
        self.puzzle = puzzle;
        self.revealed.clear();
        self.used_letters.clear();
        self.clear_turn_state();
        self.last_spin_index = None;
        self.puzzle_solved_by = None;
        self.spin_history.clear();

        // Reset consonants guessed counter for Final Spin trigger tracking
        self.final_spin.consonants_guessed = 0;

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
        self.round = 1; // Reset to round 1
        self.wheel_slots = shuffle_wheel_with_spacing(create_standard_wheel());
        self.revealed.clear();
        self.used_letters.clear();
        self.clear_turn_state();
        self.last_spin_index = None;
        self.spin_history.clear();

        self.phase = GamePhase::Pregame;
        self.tossup = TossupState::default();
        self.bonus_state = BonusState::default();
        self.final_spin = FinalSpinState::default();
        self.game_over_state = GameOverState::default();
    }

    /// Start the game - transitions from Pregame to Tossup
    /// Anyone can call this (not just host)
    pub fn start_game(&mut self) -> Result<(), &'static str> {
        if self.phase != GamePhase::Pregame {
            return Err("Game is not in pregame phase");
        }
        self.start_tossup();
        Ok(())
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
        let mut letters: Vec<char> = answer_upper.chars().filter(|c| c.is_alphabetic()).collect();
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

        // Start buzz timer if configured
        if self.config.buzz_timer_seconds > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            self.tossup.buzz_timer_end_ts = Some(now + self.config.buzz_timer_seconds as f64);
        }

        Ok(player_idx)
    }

    /// Handle wrong toss-up answer
    pub fn tossup_wrong_answer(&mut self) {
        if let Some(sid) = self.tossup.controller_sid.take() {
            self.tossup.locked_sids.insert(sid);
        }
        self.tossup.buzz_timer_end_ts = None;
    }

    /// Handle correct toss-up answer
    /// Auto-transitions to Normal phase Round 1, with the winner remaining as active player
    pub fn tossup_correct_answer(&mut self) {
        // Award toss-up points
        if let Some(player) = self.players.get_mut(self.active_idx) {
            player.total += TOSSUP_AWARD;
        }
        // Reveal all
        self.reveal_all();
        self.tossup.buzz_timer_end_ts = None;

        // Auto-transition to Normal phase Round 1
        // The toss-up winner (active_idx) remains as the active player for Round 1
        self.phase = GamePhase::Normal;
        self.round = 1;
        self.tossup = TossupState::default();
    }

    /// Handle buzz timer expiry - locks out the player who buzzed
    pub fn tossup_buzz_timeout(&mut self) {
        if let Some(sid) = self.tossup.controller_sid.take() {
            self.tossup.locked_sids.insert(sid);
        }
        self.tossup.buzz_timer_end_ts = None;
    }

    /// Get remaining buzz timer seconds (None if no timer or expired)
    pub fn buzz_timer_remaining(&self) -> Option<f64> {
        self.tossup.buzz_timer_end_ts.and_then(|end_ts| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            let remaining = end_ts - now;
            if remaining > 0.0 {
                Some(remaining)
            } else {
                None
            }
        })
    }

    /// Check if buzz timer has expired (and there was a timer active)
    pub fn buzz_timer_expired(&self) -> bool {
        if let Some(end_ts) = self.tossup.buzz_timer_end_ts {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            now >= end_ts
        } else {
            false
        }
    }

    /// Guess a letter during toss-up (no spin required, no vowel cost)
    /// Returns (correct, count) - correct means the letter was found
    pub fn tossup_guess_letter(&mut self, letter: char) -> GuessResult {
        let letter = letter.to_ascii_uppercase();

        if !letter.is_ascii_alphabetic() {
            return GuessResult::InvalidLetter;
        }

        if self.used_letters.contains(&letter) {
            return GuessResult::AlreadyUsed;
        }

        self.used_letters.insert(letter);

        let answer_upper = self.puzzle.answer.to_uppercase();
        let count = answer_upper.chars().filter(|&c| c == letter).count();

        if count > 0 {
            self.revealed.insert(letter);
            GuessResult::Correct(count)
        } else {
            // Wrong guess during toss-up = locked out
            self.tossup_wrong_answer();
            GuessResult::Incorrect
        }
    }

    /// End toss-up mode
    pub fn end_tossup(&mut self) {
        self.phase = GamePhase::Normal;
        self.tossup = TossupState::default();
    }

    // ========== FINAL SPIN ROUND METHODS ==========

    /// Check if Final Spin should be triggered automatically
    /// Returns true when:
    /// - Round is 4
    /// - Phase is Normal
    /// - Consonants guessed >= trigger_after_consonants threshold
    pub fn should_trigger_final_spin(&self) -> bool {
        self.round == 4
            && self.phase == GamePhase::Normal
            && self.final_spin.consonants_guessed >= self.final_spin.trigger_after_consonants
    }

    /// Trigger Final Spin mode (transition from Normal to Final phase in Round 4)
    /// This sets up the state but the host still needs to do the actual spin
    pub fn trigger_final_spin(&mut self) {
        self.phase = GamePhase::Final;
        // Keep the current trigger_after_consonants value, but reset spin_done
        self.final_spin.spin_done = false;
        self.final_spin.spin_value = 0;
        // Generate new threshold for next time (if game is reset)
        self.final_spin.trigger_after_consonants = rand::thread_rng().gen_range(5..=10);
        self.clear_turn_state();
    }

    /// Start Final Spin round - host spins once to set the value for all letters
    pub fn start_final_spin(&mut self) {
        self.phase = GamePhase::Final;
        self.final_spin = FinalSpinState {
            spin_value: 0,
            spin_done: false,
            trigger_after_consonants: rand::thread_rng().gen_range(5..=10),
            consonants_guessed: 0,
        };
        self.clear_turn_state();
    }

    /// Perform the Final Spin (done by active player, not host)
    /// Spins the wheel and sets spin_done = true with the spin value
    /// Automatically re-spins on special wedges (Bankrupt, LoseTurn, Prize, FreePlay)
    /// until a normal cash value is obtained
    /// Returns the wedge value that was landed on
    pub fn final_spin_do_spin(&mut self) -> Option<WedgeValue> {
        if self.phase != GamePhase::Final {
            return None;
        }

        // Perform the actual spin
        let result = self.spin();

        // Extract the cash value from the wedge
        // Only normal Cash wedges count - re-spin on everything else
        let spin_value = match &result {
            Some(WedgeValue::Cash(amount)) => *amount,
            Some(WedgeValue::Prize { .. })
            | Some(WedgeValue::FreePlay)
            | Some(WedgeValue::Bankrupt)
            | Some(WedgeValue::LoseTurn) => {
                // Re-spin on any special wedge during Final Spin
                // Keep spinning until we get a normal cash value
                return self.final_spin_do_spin();
            }
            None => return None,
        };

        self.final_spin.spin_value = spin_value;
        self.final_spin.spin_done = true;

        result
    }

    /// Set the Final Spin value after host spins (manual override)
    pub fn set_final_spin_value(&mut self, value: i32) {
        self.final_spin.spin_value = value;
        self.final_spin.spin_done = true;
    }

    /// Guess a consonant during Final Spin (uses fixed spin value + $1000 bonus per letter)
    /// No spinning required after the initial host spin - players just call letters
    pub fn final_spin_guess_consonant(&mut self, letter: char) -> GuessResult {
        let letter = letter.to_ascii_uppercase();

        if !self.final_spin.spin_done {
            return GuessResult::NeedToSpin;
        }

        if Self::is_vowel(letter) {
            return GuessResult::InvalidLetter;
        }

        if self.used_letters.contains(&letter) {
            return GuessResult::AlreadyUsed;
        }

        self.used_letters.insert(letter);

        let answer_upper = self.puzzle.answer.to_uppercase();
        let count = answer_upper.chars().filter(|&c| c == letter).count();

        if count > 0 {
            self.revealed.insert(letter);

            // Award spin_value + $1000 bonus per letter (Final Spin rules)
            let amount_per_letter = self.final_spin.spin_value + FINAL_SPIN_LETTER_BONUS;
            if let Some(player) = self.players.get_mut(self.active_idx) {
                player.round_bank += amount_per_letter * count as i32;
            }

            GuessResult::Correct(count)
        } else {
            // Wrong consonant - advance to next player
            self.advance_turn();
            GuessResult::Incorrect
        }
    }

    /// Buy a vowel during Final Spin (FREE - no cost)
    pub fn final_spin_buy_vowel(&mut self, letter: char) -> GuessResult {
        let letter = letter.to_ascii_uppercase();

        if !self.final_spin.spin_done {
            return GuessResult::NeedToSpin;
        }

        if !Self::is_vowel(letter) {
            return GuessResult::InvalidLetter;
        }

        if self.used_letters.contains(&letter) {
            return GuessResult::AlreadyUsed;
        }

        // No cost check - vowels are FREE during Final Spin
        self.used_letters.insert(letter);

        let answer_upper = self.puzzle.answer.to_uppercase();
        let count = answer_upper.chars().filter(|&c| c == letter).count();

        if count > 0 {
            self.revealed.insert(letter);
            // No cash awarded for vowels, but player keeps turn
            GuessResult::Correct(count)
        } else {
            // Wrong vowel - advance to next player
            self.advance_turn();
            GuessResult::Incorrect
        }
    }

    /// Guess any letter during Final Spin (unified - consonants earn money, vowels are free)
    /// This method accepts either consonants or vowels in a single input
    pub fn final_spin_guess_letter(&mut self, letter: char) -> GuessResult {
        let letter = letter.to_ascii_uppercase();

        if !self.final_spin.spin_done {
            return GuessResult::NeedToSpin;
        }

        if !letter.is_ascii_alphabetic() {
            return GuessResult::InvalidLetter;
        }

        if self.used_letters.contains(&letter) {
            return GuessResult::AlreadyUsed;
        }

        self.used_letters.insert(letter);

        let answer_upper = self.puzzle.answer.to_uppercase();
        let count = answer_upper.chars().filter(|&c| c == letter).count();

        if count > 0 {
            self.revealed.insert(letter);

            // Award spin_value + $1000 bonus per letter for consonants only
            // Vowels are FREE (no money awarded but correct)
            if !Self::is_vowel(letter) {
                let amount_per_letter = self.final_spin.spin_value + FINAL_SPIN_LETTER_BONUS;
                if let Some(player) = self.players.get_mut(self.active_idx) {
                    player.round_bank += amount_per_letter * count as i32;
                }
            }

            GuessResult::Correct(count)
        } else {
            // Wrong letter - advance to next player
            self.advance_turn();
            GuessResult::Incorrect
        }
    }

    /// End Final Spin round
    pub fn end_final_spin(&mut self) {
        self.phase = GamePhase::Normal;
        self.final_spin = FinalSpinState::default();
    }

    // ========== BONUS ROUND METHODS ==========

    /// Start bonus round (pick phase)
    pub fn start_bonus(&mut self) {
        self.phase = GamePhase::Bonus;
        self.bonus_state = BonusState {
            stage: BonusStage::Pick,
            picks_consonants: Vec::new(),
            pick_vowel: None,
            end_ts: None,
        };
        self.clear_turn_state();

        // Auto-reveal RSTLNE
        for &ch in BONUS_RSTLNE {
            self.revealed.insert(ch);
            self.used_letters.insert(ch);
        }
    }

    /// Pick a consonant for bonus round
    pub fn bonus_pick_consonant(&mut self, letter: char) -> Result<(), &'static str> {
        let letter = letter.to_ascii_uppercase();

        if self.phase != GamePhase::Bonus || self.bonus_state.stage != BonusStage::Pick {
            return Err("Not in bonus pick phase");
        }

        if Self::is_vowel(letter) {
            return Err("That's a vowel");
        }

        if self.used_letters.contains(&letter) {
            return Err("Letter already used");
        }

        if self.bonus_state.picks_consonants.len() >= 3 {
            return Err("Already picked 3 consonants");
        }

        self.bonus_state.picks_consonants.push(letter);
        self.used_letters.insert(letter);

        // Check if all picks complete
        if self.bonus_all_picks_complete() {
            self.bonus_start_running();
        }

        Ok(())
    }

    /// Pick a vowel for bonus round
    pub fn bonus_pick_vowel(&mut self, letter: char) -> Result<(), &'static str> {
        let letter = letter.to_ascii_uppercase();

        if self.phase != GamePhase::Bonus || self.bonus_state.stage != BonusStage::Pick {
            return Err("Not in bonus pick phase");
        }

        if !Self::is_vowel(letter) {
            return Err("That's not a vowel");
        }

        if self.used_letters.contains(&letter) {
            return Err("Letter already used");
        }

        if self.bonus_state.pick_vowel.is_some() {
            return Err("Already picked a vowel");
        }

        self.bonus_state.pick_vowel = Some(letter);
        self.used_letters.insert(letter);

        // Check if all picks complete
        if self.bonus_all_picks_complete() {
            self.bonus_start_running();
        }

        Ok(())
    }

    /// Check if all final picks are complete
    pub fn bonus_all_picks_complete(&self) -> bool {
        self.bonus_state.picks_consonants.len() >= 3 && self.bonus_state.pick_vowel.is_some()
    }

    /// Start the running phase of final round (called automatically when picks complete,
    /// or can be called manually via bonus_start_solve)
    pub fn bonus_start_solve(&mut self) {
        if self.bonus_state.stage == BonusStage::Pick && self.bonus_all_picks_complete() {
            self.bonus_start_running();
        }
    }

    /// Start the running phase of final round
    fn bonus_start_running(&mut self) {
        // Reveal picked letters
        for &ch in &self.bonus_state.picks_consonants {
            self.revealed.insert(ch);
        }
        if let Some(ch) = self.bonus_state.pick_vowel {
            self.revealed.insert(ch);
        }

        // Start timer
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        self.bonus_state.end_ts = Some(now + self.config.bonus_seconds as f64);
        self.bonus_state.stage = BonusStage::Running;
    }

    /// Get remaining seconds in final round
    pub fn bonus_remaining_seconds(&self) -> Option<i32> {
        self.bonus_state.end_ts.map(|end_ts| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            (end_ts - now).max(0.0) as i32
        })
    }

    /// Check if final timer expired
    pub fn bonus_timer_expired(&self) -> bool {
        if let Some(remaining) = self.bonus_remaining_seconds() {
            remaining <= 0
        } else {
            false
        }
    }

    /// Get remaining seconds in turn timer
    pub fn turn_timer_remaining_seconds(&self) -> Option<i32> {
        self.turn_timer_end_ts.map(|end_ts| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            (end_ts - now).max(0.0) as i32
        })
    }

    /// Check if turn timer expired
    pub fn turn_timer_expired(&self) -> bool {
        if let Some(remaining) = self.turn_timer_remaining_seconds() {
            remaining <= 0
        } else {
            false
        }
    }

    /// Start the turn timer after spin animation completes
    pub fn start_turn_timer(&mut self) {
        if self.config.turn_timer_seconds > 0 && self.current_wedge.is_some() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            self.turn_timer_end_ts = Some(now + self.config.turn_timer_seconds as f64);
        }
    }

    /// Clear the turn timer
    pub fn clear_turn_timer(&mut self) {
        self.turn_timer_end_ts = None;
    }

    /// End final round - transitions to GameOver phase
    pub fn end_bonus(&mut self) {
        // Determine the winner (player with highest total score)
        let bonus_won =
            self.bonus_state.stage == BonusStage::Done && self.puzzle_solved_by.is_some();

        let (winner_idx, winner_name, winner_score) = self
            .players
            .iter()
            .enumerate()
            .max_by_key(|(_, p)| p.total)
            .map(|(idx, p)| (Some(idx), Some(p.name.clone()), p.total))
            .unwrap_or((None, None, 0));

        // Calculate reset timestamp (15 seconds from now)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let reset_at = now + GAME_OVER_DISPLAY_SECONDS as f64;

        // Set game over state
        self.game_over_state = GameOverState {
            winner_idx,
            winner_name,
            winner_score,
            bonus_won,
            reset_at: Some(reset_at),
        };

        self.phase = GamePhase::GameOver;
        self.bonus_state.stage = BonusStage::Done;
    }

    /// Check if game over timer has expired and should auto-reset
    pub fn game_over_should_reset(&self) -> bool {
        if self.phase != GamePhase::GameOver {
            return false;
        }
        if let Some(reset_at) = self.game_over_state.reset_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            now >= reset_at
        } else {
            false
        }
    }

    /// Get remaining seconds until game over auto-reset
    pub fn game_over_remaining_seconds(&self) -> Option<i32> {
        if self.phase != GamePhase::GameOver {
            return None;
        }
        self.game_over_state.reset_at.map(|reset_at| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            ((reset_at - now).max(0.0).ceil() as i32).max(0)
        })
    }

    /// Solve in final round
    pub fn bonus_solve(&mut self, attempt: &str) -> bool {
        if self.solve(attempt) {
            // Award jackpot
            if let Some(player) = self.players.get_mut(self.active_idx) {
                player.total += self.config.bonus_jackpot;
            }
            self.bonus_state.stage = BonusStage::Done;
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
            round: self.round,
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
            active_pack_name: self.active_pack_name.clone(),
            tossup: TossupStateClient {
                controller_player_idx: tossup_controller_idx,
                locked_player_idxs: tossup_locked_idxs,
                allowed_player_idxs: self.tossup.allowed_player_idxs.clone(),
                is_tiebreaker: self.tossup.is_tiebreaker,
                remaining_seconds: self.buzz_timer_remaining().map(|r| r.ceil() as i32),
            },
            bonus_round: BonusStateClient {
                stage: self.bonus_state.stage,
                picks: BonusPicks {
                    consonants: self.bonus_state.picks_consonants.clone(),
                    vowel: self.bonus_state.pick_vowel,
                },
                remaining_seconds: self.bonus_remaining_seconds(),
                jackpot: self.config.bonus_jackpot,
            },
            final_spin: FinalSpinStateClient {
                spin_value: self.final_spin.spin_value,
                spin_done: self.final_spin.spin_done,
                consonants_guessed: self.final_spin.consonants_guessed,
                trigger_after_consonants: self.final_spin.trigger_after_consonants,
            },
            game_over: GameOverStateClient {
                winner_idx: self.game_over_state.winner_idx,
                winner_name: self.game_over_state.winner_name.clone(),
                winner_score: self.game_over_state.winner_score,
                bonus_won: self.game_over_state.bonus_won,
                remaining_seconds: self.game_over_remaining_seconds(),
            },
            turn_timer_remaining: self.turn_timer_remaining_seconds(),
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
    pub remaining_seconds: Option<i32>,
}

/// Final picks for client
#[derive(Debug, Clone, Serialize)]
pub struct BonusPicks {
    pub consonants: Vec<char>,
    pub vowel: Option<char>,
}

/// Bonus state for client
#[derive(Debug, Clone, Serialize)]
pub struct BonusStateClient {
    pub stage: BonusStage,
    pub picks: BonusPicks,
    pub remaining_seconds: Option<i32>,
    pub jackpot: i32,
}

/// Final Spin state for client
#[derive(Debug, Clone, Serialize)]
pub struct FinalSpinStateClient {
    pub spin_value: i32,
    pub spin_done: bool,
    /// Number of consonants guessed so far in Round 4 (for UI progress)
    pub consonants_guessed: u8,
    /// Threshold that triggers Final Spin (for UI progress display)
    pub trigger_after_consonants: u8,
}

/// Game Over state for client
#[derive(Debug, Clone, Serialize)]
pub struct GameOverStateClient {
    /// Index of the winning player
    pub winner_idx: Option<usize>,
    /// Winner's name
    pub winner_name: Option<String>,
    /// Winner's total score
    pub winner_score: i32,
    /// Whether the bonus round was won
    pub bonus_won: bool,
    /// Remaining seconds until auto-reset
    pub remaining_seconds: Option<i32>,
}

/// Game state for sending to clients
#[derive(Debug, Clone, Serialize)]
pub struct GameState {
    pub room: String,
    pub phase: GamePhase,
    /// Current round number (1-4)
    pub round: u8,
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
    pub active_pack_name: Option<String>,
    pub tossup: TossupStateClient,
    #[serde(rename = "bonus")]
    pub bonus_round: BonusStateClient,
    /// Final Spin round state
    pub final_spin: FinalSpinStateClient,
    /// Game Over state (winner, reset timer)
    pub game_over: GameOverStateClient,
    /// Remaining seconds in turn timer (None if not active)
    pub turn_timer_remaining: Option<i32>,
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
        game.add_player(
            "Player 1".to_string(),
            Some("socket1".to_string()),
            None,
            Some(1),
        );
        game.add_player(
            "Player 2".to_string(),
            Some("socket2".to_string()),
            None,
            Some(2),
        );
        game.add_player(
            "Player 3".to_string(),
            Some("socket3".to_string()),
            None,
            Some(3),
        );
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
        let idx = game.add_player(
            "Test Player".to_string(),
            Some("socket123".to_string()),
            Some(42),
            Some(7),
        );

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
        assert_eq!(game.tossup.controller_sid, Some("socket2".to_string()));
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

    // ========== bonus round tests ==========

    #[test]
    fn test_start_bonus() {
        let mut game = create_test_game_with_players();

        game.start_bonus();

        assert_eq!(game.phase, GamePhase::Bonus);
        assert_eq!(game.bonus_state.stage, BonusStage::Pick);
        // RSTLNE should be revealed
        for c in ['R', 'S', 'T', 'L', 'N', 'E'] {
            assert!(game.revealed.contains(&c));
            assert!(game.used_letters.contains(&c));
        }
    }

    #[test]
    fn test_bonus_pick_consonant() {
        let mut game = create_test_game_with_players();
        game.start_bonus();

        let result = game.bonus_pick_consonant('B');
        assert!(result.is_ok());
        assert!(game.bonus_state.picks_consonants.contains(&'B'));
    }

    #[test]
    fn test_bonus_pick_consonant_vowel_rejected() {
        let mut game = create_test_game_with_players();
        game.start_bonus();

        let result = game.bonus_pick_consonant('A');
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "That's a vowel");
    }

    #[test]
    fn test_bonus_pick_consonant_already_used() {
        let mut game = create_test_game_with_players();
        game.start_bonus();
        // R is already used (RSTLNE)

        let result = game.bonus_pick_consonant('R');
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Letter already used");
    }

    #[test]
    fn test_bonus_pick_vowel() {
        let mut game = create_test_game_with_players();
        game.start_bonus();

        let result = game.bonus_pick_vowel('I');
        assert!(result.is_ok());
        assert_eq!(game.bonus_state.pick_vowel, Some('I'));
    }

    #[test]
    fn test_bonus_all_picks_starts_running() {
        let mut game = create_test_game_with_players();
        game.start_bonus();

        game.bonus_pick_consonant('B').unwrap();
        game.bonus_pick_consonant('C').unwrap();
        game.bonus_pick_consonant('D').unwrap();
        game.bonus_pick_vowel('I').unwrap();

        assert_eq!(game.bonus_state.stage, BonusStage::Running);
        assert!(game.bonus_state.end_ts.is_some());
    }

    #[test]
    fn test_bonus_solve_awards_jackpot() {
        let mut game = create_test_game_with_players();
        game.start_bonus();
        game.bonus_state.stage = BonusStage::Running;
        game.active_idx = 0;

        assert!(game.bonus_solve("HELLO WORLD"));

        assert_eq!(game.players[0].total, game.config.bonus_jackpot);
        assert_eq!(game.bonus_state.stage, BonusStage::Done);
    }

    // ========== reset_game tests ==========

    #[test]
    fn test_reset_game() {
        let mut game = create_test_game_with_players();
        game.players[0].total = 5000;
        game.players[0].round_bank = 1000;
        game.active_idx = 2;
        game.phase = GamePhase::Bonus;
        game.revealed.insert('H');

        game.reset_game();

        assert_eq!(game.players[0].total, 0);
        assert_eq!(game.players[0].round_bank, 0);
        assert_eq!(game.active_idx, 0);
        assert_eq!(game.phase, GamePhase::Pregame);
        assert!(game.revealed.is_empty());
    }

    // ========== start_game tests ==========

    #[test]
    fn test_start_game_from_pregame() {
        let mut game = create_test_game_with_players();
        game.reset_game();
        assert_eq!(game.phase, GamePhase::Pregame);

        let result = game.start_game();
        assert!(result.is_ok());
        assert_eq!(game.phase, GamePhase::Tossup);
    }

    #[test]
    fn test_start_game_not_in_pregame() {
        let mut game = create_test_game_with_players();
        game.phase = GamePhase::Normal;

        let result = game.start_game();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Game is not in pregame phase");
    }

    #[test]
    fn test_tossup_correct_answer_transitions_to_normal() {
        let mut game = create_test_game_with_players();
        game.start_tossup();
        game.active_idx = 1; // Player 2 buzzes in and wins

        game.tossup_correct_answer();

        // Should transition to Normal phase Round 1
        assert_eq!(game.phase, GamePhase::Normal);
        assert_eq!(game.round, 1);
        // Winner should remain as active player
        assert_eq!(game.active_idx, 1);
        // Should have been awarded toss-up points
        assert_eq!(game.players[1].total, TOSSUP_AWARD);
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
