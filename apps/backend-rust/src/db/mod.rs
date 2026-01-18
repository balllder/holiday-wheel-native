use sqlx::{migrate::MigrateDatabase, FromRow, Row, Sqlite, SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::info;

use crate::game::{Puzzle, RoomConfig};

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,
}

/// Database wrapper
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

// ========== USER TYPES ==========

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub verified: bool,
    pub verification_token: Option<String>,
    pub verification_token_expires: Option<i64>,
    pub reset_token: Option<String>,
    pub reset_token_expires: Option<i64>,
    pub created_at: i64,
    pub last_login_at: Option<i64>,
    pub remember_token: Option<String>,
    #[sqlx(default)]
    pub is_admin: bool,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub verification_token: String,
    pub verification_token_expires: i64,
}

// ========== PUZZLE PACK TYPES ==========

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Pack {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct DbPuzzle {
    pub id: i64,
    pub category: String,
    pub answer: String,
    pub pack_id: i64,
    pub enabled: bool,
}

// ========== ROOM TYPES ==========

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Room {
    pub name: String,
    pub created_by: Option<i64>,
    pub created_at: i64,
    pub last_activity_at: i64,
    pub is_public: bool,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

impl Database {
    /// Create a new database connection
    pub async fn new(db_path: &str) -> Result<Self, DbError> {
        let db_url = format!("sqlite:{}", db_path);

        // Create database if it doesn't exist
        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            info!("Creating database at {}", db_path);
            Sqlite::create_database(&db_url).await?;
        }

        let pool = SqlitePool::connect(&db_url).await?;

        let db = Self { pool };
        db.init_tables().await?;

        Ok(db)
    }

    /// Connect to an existing database (for compatibility)
    pub async fn connect(url: &str) -> Result<Self, DbError> {
        // Extract path from sqlite: URL if present
        let db_path = url.strip_prefix("sqlite:").unwrap_or(url);
        Self::new(db_path).await
    }

    /// Initialize all tables
    async fn init_tables(&self) -> Result<(), DbError> {
        // Users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT NOT NULL UNIQUE COLLATE NOCASE,
                password_hash TEXT NOT NULL,
                display_name TEXT NOT NULL,
                verified INTEGER NOT NULL DEFAULT 0,
                verification_token TEXT,
                verification_token_expires INTEGER,
                reset_token TEXT,
                reset_token_expires INTEGER,
                created_at INTEGER NOT NULL,
                last_login_at INTEGER,
                remember_token TEXT,
                is_admin INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Add is_admin column if it doesn't exist (migration for existing databases)
        sqlx::query("ALTER TABLE users ADD COLUMN is_admin INTEGER NOT NULL DEFAULT 0")
            .execute(&self.pool)
            .await
            .ok(); // Ignore error if column already exists

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_users_verification_token ON users(verification_token)",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_remember_token ON users(remember_token)")
            .execute(&self.pool)
            .await?;

        // Rooms table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rooms (
                name TEXT PRIMARY KEY,
                created_by INTEGER,
                created_at INTEGER NOT NULL,
                last_activity_at INTEGER NOT NULL,
                is_public INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY(created_by) REFERENCES users(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_rooms_last_activity ON rooms(last_activity_at)")
            .execute(&self.pool)
            .await?;

        // Packs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS packs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Puzzles table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS puzzles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                answer TEXT NOT NULL,
                pack_id INTEGER NOT NULL DEFAULT 1,
                enabled INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY(pack_id) REFERENCES packs(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_puzzles_pack_id ON puzzles(pack_id)")
            .execute(&self.pool)
            .await?;

        // Used puzzles tracking (per room)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS used_puzzles (
                room_name TEXT NOT NULL,
                puzzle_id INTEGER NOT NULL,
                used_at INTEGER NOT NULL,
                PRIMARY KEY(room_name, puzzle_id),
                FOREIGN KEY(puzzle_id) REFERENCES puzzles(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Room config
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS room_config (
                room_name TEXT PRIMARY KEY,
                active_pack_id INTEGER,
                vowel_cost INTEGER,
                final_seconds INTEGER,
                final_jackpot INTEGER,
                prize_replace_csv TEXT,
                puzzle_display_seconds INTEGER,
                prize_wedge_names TEXT,
                FOREIGN KEY(active_pack_id) REFERENCES packs(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Add new columns if they don't exist (migration for existing databases)
        sqlx::query("ALTER TABLE room_config ADD COLUMN puzzle_display_seconds INTEGER")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("ALTER TABLE room_config ADD COLUMN prize_wedge_names TEXT")
            .execute(&self.pool)
            .await
            .ok();

        // Create default pack if none exists
        sqlx::query("INSERT OR IGNORE INTO packs (id, name) VALUES (1, 'Default')")
            .execute(&self.pool)
            .await?;

        // Bootstrap admin from environment variable
        if let Ok(admin_email) = std::env::var("ADMIN_EMAIL") {
            sqlx::query("UPDATE users SET is_admin = 1 WHERE email = ?")
                .bind(admin_email.to_lowercase())
                .execute(&self.pool)
                .await
                .ok();
        }

        // Insert some default puzzles if none exist
        let puzzle_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM puzzles")
            .fetch_one(&self.pool)
            .await?;

        if puzzle_count == 0 {
            let default_puzzles = vec![
                ("Phrase", "JINGLE ALL THE WAY"),
                ("Phrase", "HAPPY HOLIDAYS"),
                ("Thing", "CHRISTMAS TREE"),
                ("Place", "NORTH POLE"),
                ("Phrase", "DECK THE HALLS"),
                ("Thing", "SNOWFLAKE"),
                ("Person", "SANTA CLAUS"),
                ("Phrase", "LET IT SNOW"),
                ("Thing", "CANDY CANE"),
                ("Event", "NEW YEARS EVE"),
            ];

            for (category, answer) in default_puzzles {
                sqlx::query(
                    "INSERT INTO puzzles (category, answer, pack_id) VALUES (?, ?, 1)",
                )
                .bind(category)
                .bind(answer)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    // ========== USER METHODS ==========

    /// Check if user with email exists
    pub async fn user_exists(&self, email: &str) -> Result<bool, DbError> {
        let result = sqlx::query("SELECT 1 FROM users WHERE email = ?")
            .bind(email.to_lowercase())
            .fetch_optional(&self.pool)
            .await?;
        Ok(result.is_some())
    }

    /// Create a new user
    pub async fn create_user(&self, user: NewUser) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO users (email, password_hash, display_name, verification_token,
                             verification_token_expires, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user.email.to_lowercase())
        .bind(&user.password_hash)
        .bind(&user.display_name)
        .bind(&user.verification_token)
        .bind(user.verification_token_expires)
        .bind(now_secs())
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DbError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(email.to_lowercase())
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>, DbError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    /// Get user by verification token
    pub async fn get_user_by_verification_token(
        &self,
        token: &str,
    ) -> Result<Option<User>, DbError> {
        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE verification_token = ?")
                .bind(token)
                .fetch_optional(&self.pool)
                .await?;
        Ok(user)
    }

    /// Get user by remember token
    pub async fn get_user_by_token(&self, token: &str) -> Result<Option<User>, DbError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE remember_token = ?")
            .bind(token)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    /// Verify user
    pub async fn verify_user(&self, user_id: i64) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE users SET verified = 1, verification_token = NULL, verification_token_expires = NULL WHERE id = ?",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Set remember token
    pub async fn set_remember_token(&self, user_id: i64, token: &str) -> Result<(), DbError> {
        sqlx::query("UPDATE users SET remember_token = ? WHERE id = ?")
            .bind(token)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Clear remember token
    pub async fn clear_remember_token(&self, user_id: i64) -> Result<(), DbError> {
        sqlx::query("UPDATE users SET remember_token = NULL WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Update last login
    pub async fn update_last_login(&self, user_id: i64) -> Result<(), DbError> {
        sqlx::query("UPDATE users SET last_login_at = ? WHERE id = ?")
            .bind(now_secs())
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Set password reset token
    pub async fn set_password_reset_token(
        &self,
        user_id: i64,
        token: &str,
        expires: i64,
    ) -> Result<(), DbError> {
        sqlx::query("UPDATE users SET reset_token = ?, reset_token_expires = ? WHERE id = ?")
            .bind(token)
            .bind(expires)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Update password
    pub async fn update_password(&self, user_id: i64, password_hash: &str) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE users SET password_hash = ?, reset_token = NULL, reset_token_expires = NULL WHERE id = ?",
        )
        .bind(password_hash)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Set verification token (for resending)
    pub async fn set_verification_token(
        &self,
        user_id: i64,
        token: &str,
        expires: i64,
    ) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE users SET verification_token = ?, verification_token_expires = ?, verified = 0 WHERE id = ?",
        )
        .bind(token)
        .bind(expires)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ========== PUZZLE METHODS ==========

    /// Get a random puzzle from a pack (excluding already used ones)
    pub async fn get_random_puzzle(
        &self,
        room_name: &str,
        pack_id: Option<i64>,
    ) -> Result<Puzzle, DbError> {
        let pack_id = pack_id.unwrap_or(1);

        let row = sqlx::query(
            r#"
            SELECT id, category, answer
            FROM puzzles
            WHERE pack_id = ?
              AND enabled = 1
              AND id NOT IN (SELECT puzzle_id FROM used_puzzles WHERE room_name = ?)
            ORDER BY RANDOM()
            LIMIT 1
            "#,
        )
        .bind(pack_id)
        .bind(room_name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let puzzle = Puzzle {
                id: row.get("id"),
                category: row.get("category"),
                answer: row.get("answer"),
            };

            // Mark as used
            self.mark_puzzle_used(room_name, puzzle.id).await?;

            Ok(puzzle)
        } else {
            // If no unused puzzles, clear used list and try again
            self.clear_used_puzzles(room_name).await?;

            let row = sqlx::query(
                r#"
                SELECT id, category, answer
                FROM puzzles
                WHERE pack_id = ? AND enabled = 1
                ORDER BY RANDOM()
                LIMIT 1
                "#,
            )
            .bind(pack_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(row) = row {
                let puzzle = Puzzle {
                    id: row.get("id"),
                    category: row.get("category"),
                    answer: row.get("answer"),
                };
                self.mark_puzzle_used(room_name, puzzle.id).await?;
                Ok(puzzle)
            } else {
                Err(DbError::NotFound)
            }
        }
    }

    /// Mark a puzzle as used in a room
    pub async fn mark_puzzle_used(
        &self,
        room_name: &str,
        puzzle_id: i64,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT OR REPLACE INTO used_puzzles (room_name, puzzle_id, used_at) VALUES (?, ?, ?)",
        )
        .bind(room_name)
        .bind(puzzle_id)
        .bind(now_secs())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Clear used puzzles for a room
    pub async fn clear_used_puzzles(&self, room_name: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM used_puzzles WHERE room_name = ?")
            .bind(room_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ========== PACK METHODS ==========

    /// List all packs
    pub async fn list_packs(&self) -> Result<Vec<Pack>, DbError> {
        let packs = sqlx::query_as::<_, Pack>("SELECT id, name FROM packs ORDER BY name")
            .fetch_all(&self.pool)
            .await?;
        Ok(packs)
    }

    /// Get or create a pack by name
    pub async fn get_or_create_pack(&self, name: &str) -> Result<i64, DbError> {
        // Try to get existing pack
        let existing = sqlx::query("SELECT id FROM packs WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = existing {
            Ok(row.get("id"))
        } else {
            // Create new pack
            let result = sqlx::query("INSERT INTO packs (name) VALUES (?)")
                .bind(name)
                .execute(&self.pool)
                .await?;
            Ok(result.last_insert_rowid())
        }
    }

    /// Add puzzles to a pack
    pub async fn add_puzzles(
        &self,
        puzzles: Vec<(String, String)>,
        pack_id: i64,
    ) -> Result<usize, DbError> {
        let mut count = 0;
        for (category, answer) in puzzles {
            sqlx::query("INSERT INTO puzzles (category, answer, pack_id) VALUES (?, ?, ?)")
                .bind(&category)
                .bind(&answer)
                .bind(pack_id)
                .execute(&self.pool)
                .await?;
            count += 1;
        }
        Ok(count)
    }

    /// Delete a pack and its puzzles
    pub async fn delete_pack(&self, pack_id: i64) -> Result<bool, DbError> {
        // Don't allow deleting default pack
        if pack_id == 1 {
            return Ok(false);
        }

        sqlx::query("DELETE FROM puzzles WHERE pack_id = ?")
            .bind(pack_id)
            .execute(&self.pool)
            .await?;

        let result = sqlx::query("DELETE FROM packs WHERE id = ?")
            .bind(pack_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========== ROOM CONFIG METHODS ==========

    /// Get room config
    pub async fn get_room_config(&self, room_name: &str) -> Result<RoomConfig, DbError> {
        let row = sqlx::query(
            "SELECT active_pack_id, vowel_cost, final_seconds, final_jackpot, prize_replace_csv, puzzle_display_seconds, prize_wedge_names FROM room_config WHERE room_name = ?",
        )
        .bind(room_name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let prize_csv: Option<String> = row.get("prize_replace_csv");
            let prize_values = prize_csv
                .map(|csv| {
                    csv.split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect()
                })
                .unwrap_or_else(|| vec![500, 1000, 1500, 2000, 2500, 3000, 3500]);

            let prize_wedge_csv: Option<String> = row.get("prize_wedge_names");
            let prize_wedge_names = prize_wedge_csv
                .map(|csv| {
                    csv.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_else(|| vec!["GIFT CARD".to_string()]);

            Ok(RoomConfig {
                vowel_cost: row.get::<Option<i32>, _>("vowel_cost").unwrap_or(250),
                final_seconds: row.get::<Option<i32>, _>("final_seconds").unwrap_or(30),
                final_jackpot: row.get::<Option<i32>, _>("final_jackpot").unwrap_or(10000),
                prize_replace_cash_values: prize_values,
                puzzle_display_seconds: row.get::<Option<i32>, _>("puzzle_display_seconds").unwrap_or(30),
                prize_wedge_names,
            })
        } else {
            Ok(RoomConfig::default())
        }
    }

    /// Get active pack ID for a room
    pub async fn get_active_pack_id(&self, room_name: &str) -> Result<Option<i64>, DbError> {
        let row = sqlx::query("SELECT active_pack_id FROM room_config WHERE room_name = ?")
            .bind(room_name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|r| r.get("active_pack_id")))
    }

    /// Set room config
    pub async fn set_room_config(
        &self,
        room_name: &str,
        config: &RoomConfig,
        active_pack_id: Option<i64>,
    ) -> Result<(), DbError> {
        let prize_csv = config
            .prize_replace_cash_values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let prize_wedge_csv = config.prize_wedge_names.join(",");

        sqlx::query(
            r#"
            INSERT INTO room_config (room_name, active_pack_id, vowel_cost, final_seconds, final_jackpot, prize_replace_csv, puzzle_display_seconds, prize_wedge_names)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(room_name) DO UPDATE SET
                active_pack_id = excluded.active_pack_id,
                vowel_cost = excluded.vowel_cost,
                final_seconds = excluded.final_seconds,
                final_jackpot = excluded.final_jackpot,
                prize_replace_csv = excluded.prize_replace_csv,
                puzzle_display_seconds = excluded.puzzle_display_seconds,
                prize_wedge_names = excluded.prize_wedge_names
            "#,
        )
        .bind(room_name)
        .bind(active_pack_id)
        .bind(config.vowel_cost)
        .bind(config.final_seconds)
        .bind(config.final_jackpot)
        .bind(&prize_csv)
        .bind(config.puzzle_display_seconds)
        .bind(&prize_wedge_csv)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Set active pack for a room
    pub async fn set_active_pack(
        &self,
        room_name: &str,
        pack_id: Option<i64>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO room_config (room_name, active_pack_id)
            VALUES (?, ?)
            ON CONFLICT(room_name) DO UPDATE SET active_pack_id = excluded.active_pack_id
            "#,
        )
        .bind(room_name)
        .bind(pack_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ========== ROOM ACTIVITY METHODS ==========

    /// List active rooms
    pub async fn list_active_rooms(&self, hours: i64) -> Result<Vec<Room>, DbError> {
        let cutoff = now_secs() - (hours * 3600);

        let rooms = sqlx::query_as::<_, Room>(
            "SELECT * FROM rooms WHERE last_activity_at > ? ORDER BY last_activity_at DESC",
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await?;
        Ok(rooms)
    }

    /// Update room activity
    pub async fn update_room_activity(
        &self,
        room_name: &str,
        user_id: Option<i64>,
    ) -> Result<(), DbError> {
        let now = now_secs();

        sqlx::query(
            r#"
            INSERT INTO rooms (name, created_by, created_at, last_activity_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(name) DO UPDATE SET last_activity_at = excluded.last_activity_at
            "#,
        )
        .bind(room_name)
        .bind(user_id)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete a room
    pub async fn delete_room(&self, room_name: &str) -> Result<bool, DbError> {
        // Also clear used puzzles
        self.clear_used_puzzles(room_name).await?;

        // Delete room config
        sqlx::query("DELETE FROM room_config WHERE room_name = ?")
            .bind(room_name)
            .execute(&self.pool)
            .await?;

        // Delete room
        let result = sqlx::query("DELETE FROM rooms WHERE name = ?")
            .bind(room_name)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========== ADMIN METHODS ==========

    /// List all users (admin)
    pub async fn list_all_users(&self) -> Result<Vec<User>, DbError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    /// Set user admin status
    pub async fn set_user_admin(&self, user_id: i64, is_admin: bool) -> Result<(), DbError> {
        sqlx::query("UPDATE users SET is_admin = ? WHERE id = ?")
            .bind(is_admin)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Delete a user
    pub async fn delete_user(&self, user_id: i64) -> Result<bool, DbError> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// List all puzzles (optionally filtered by pack)
    pub async fn list_all_puzzles(&self, pack_id: Option<i64>) -> Result<Vec<DbPuzzle>, DbError> {
        let puzzles = if let Some(pack_id) = pack_id {
            sqlx::query_as::<_, DbPuzzle>(
                "SELECT * FROM puzzles WHERE pack_id = ? ORDER BY category, answer"
            )
            .bind(pack_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, DbPuzzle>(
                "SELECT * FROM puzzles ORDER BY pack_id, category, answer"
            )
            .fetch_all(&self.pool)
            .await?
        };
        Ok(puzzles)
    }

    /// Add a single puzzle
    pub async fn add_puzzle(&self, category: &str, answer: &str, pack_id: i64) -> Result<i64, DbError> {
        let result = sqlx::query(
            "INSERT INTO puzzles (category, answer, pack_id) VALUES (?, ?, ?)"
        )
        .bind(category)
        .bind(answer.to_uppercase())
        .bind(pack_id)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// Delete a puzzle
    pub async fn delete_puzzle(&self, puzzle_id: i64) -> Result<bool, DbError> {
        let result = sqlx::query("DELETE FROM puzzles WHERE id = ?")
            .bind(puzzle_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Toggle puzzle enabled status
    pub async fn set_puzzle_enabled(&self, puzzle_id: i64, enabled: bool) -> Result<(), DbError> {
        sqlx::query("UPDATE puzzles SET enabled = ? WHERE id = ?")
            .bind(enabled)
            .bind(puzzle_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get puzzle count per pack
    pub async fn get_puzzle_counts(&self) -> Result<Vec<(i64, String, i64)>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT p.id, p.name, COUNT(pz.id) as count
            FROM packs p
            LEFT JOIN puzzles pz ON pz.pack_id = p.id AND pz.enabled = 1
            GROUP BY p.id
            ORDER BY p.name
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|r| {
            (r.get("id"), r.get("name"), r.get("count"))
        }).collect())
    }

    /// Bulk import puzzles from a list
    pub async fn import_puzzles(&self, puzzles: Vec<(String, String)>, pack_id: i64) -> Result<usize, DbError> {
        let mut count = 0;
        for (category, answer) in puzzles {
            sqlx::query(
                "INSERT INTO puzzles (category, answer, pack_id) VALUES (?, ?, ?)"
            )
            .bind(&category)
            .bind(answer.to_uppercase())
            .bind(pack_id)
            .execute(&self.pool)
            .await?;
            count += 1;
        }
        Ok(count)
    }
}
