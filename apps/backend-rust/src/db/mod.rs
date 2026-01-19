use sqlx::{migrate::MigrateDatabase, FromRow, Row, Sqlite, SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::info;

use crate::game::{Puzzle, RoomConfig};

// ============================================================================
// DATABASE MIGRATIONS
// ============================================================================
//
// This module uses sqlx migrations for schema management. Migrations are stored
// in the `migrations/` directory at the crate root.
//
// ## Migration Process
//
// 1. **Creating new migrations:**
//    ```bash
//    # Using sqlx-cli (recommended)
//    cargo install sqlx-cli
//    sqlx migrate add <migration_name>
//
//    # Or manually create: migrations/NNNN_description.sql
//    ```
//
// 2. **Running migrations:**
//    Migrations run automatically on startup via `sqlx::migrate!()`.
//    The `_sqlx_migrations` table tracks which migrations have been applied.
//
// 3. **Migration file naming:**
//    - Format: `NNN_description.sql` (e.g., `001_initial_schema.sql`)
//    - Files are executed in lexicographic order
//    - Each migration runs in a transaction
//
// 4. **Checking migration status:**
//    ```bash
//    sqlx migrate info --database-url sqlite:puzzles.db
//    ```
//
// 5. **Reverting migrations:**
//    SQLx doesn't support down migrations by default. For rollbacks:
//    - Create a new migration that reverses the changes
//    - Or restore from backup
//
// ## Current Schema
//
// The initial migration (001_initial_schema.sql) creates:
// - users: User accounts with auth data (email, password, tokens)
// - rooms: Game room metadata
// - packs: Puzzle pack definitions
// - puzzles: Individual puzzles with categories
// - used_puzzles: Per-room puzzle usage tracking
// - room_config: Per-room game configuration
// - passkey_credentials: WebAuthn/Passkey storage
// - oauth_accounts: OAuth provider linkage
// - webauthn_challenges: Temporary WebAuthn challenge storage
//
// ============================================================================

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

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
    pub password_hash: Option<String>, // Optional for OAuth-only users
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

// ========== PASSKEY & OAUTH TYPES ==========

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct PasskeyCredential {
    pub id: String,           // credential ID (base64url)
    pub user_id: i64,
    pub public_key: Vec<u8>,  // COSE public key
    pub counter: i64,         // signature counter
    pub transports: Option<String>, // JSON array of transports
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct OAuthAccount {
    pub id: i64,
    pub user_id: i64,
    pub provider: String,     // 'google' or 'apple'
    pub provider_user_id: String, // sub claim from ID token
    pub email: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct WebAuthnChallenge {
    pub challenge: String,
    pub user_id: Option<i64>,
    pub email: Option<String>,
    pub challenge_type: String, // 'registration' or 'authentication'
    pub created_at: i64,
    pub expires_at: i64,
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
    /// Create a new database connection and run migrations
    ///
    /// This method:
    /// 1. Creates the database file if it doesn't exist
    /// 2. Runs any pending sqlx migrations from the `migrations/` directory
    /// 3. Performs runtime initialization (admin bootstrap, default data)
    pub async fn new(db_path: &str) -> Result<Self, DbError> {
        // Handle both "sqlite:/path" and "/path" formats
        let db_url = if db_path.starts_with("sqlite:") {
            db_path.to_string()
        } else {
            format!("sqlite:{}", db_path)
        };

        // Extract file path for logging
        let file_path = db_url.strip_prefix("sqlite:").unwrap_or(&db_url);

        // Create database if it doesn't exist
        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            info!("Creating database at {}", file_path);
            Sqlite::create_database(&db_url).await?;
        }

        let pool = SqlitePool::connect(&db_url).await?;

        // Run sqlx migrations from the migrations/ directory
        // The migrate!() macro embeds migrations at compile time
        info!("Running database migrations...");
        sqlx::migrate!("./migrations").run(&pool).await?;
        info!("Database migrations completed");

        let db = Self { pool };

        // Run runtime initialization (not in migrations because they depend on env vars)
        db.init_runtime_data().await?;

        Ok(db)
    }

    /// Connect to an existing database (for compatibility)
    pub async fn connect(url: &str) -> Result<Self, DbError> {
        // Extract path from sqlite: URL if present
        let db_path = url.strip_prefix("sqlite:").unwrap_or(url);
        Self::new(db_path).await
    }

    /// Ping the database to verify connectivity
    pub async fn ping(&self) -> Result<(), DbError> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    /// Initialize runtime data that cannot be in migrations
    ///
    /// This handles:
    /// - Admin user bootstrap from ADMIN_EMAIL env var
    /// - Default puzzles if database is empty
    ///
    /// Note: Schema creation is now handled by sqlx migrations in ./migrations/
    async fn init_runtime_data(&self) -> Result<(), DbError> {
        // Bootstrap admin from environment variable
        // This runs on every startup to ensure the admin is always set
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

            let puzzle_count_inserted = default_puzzles.len();
            for (category, answer) in default_puzzles {
                sqlx::query(
                    "INSERT INTO puzzles (category, answer, pack_id) VALUES (?, ?, 1)",
                )
                .bind(category)
                .bind(answer)
                .execute(&self.pool)
                .await?;
            }
            info!("Inserted {} default puzzles", puzzle_count_inserted);
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
    /// If pack_id is None or Some(0), puzzles from all packs are used
    pub async fn get_random_puzzle(
        &self,
        room_name: &str,
        pack_id: Option<i64>,
    ) -> Result<Puzzle, DbError> {
        // None or 0 means "all packs"
        let use_all_packs = pack_id.is_none() || pack_id == Some(0);

        let row = if use_all_packs {
            sqlx::query(
                r#"
                SELECT id, category, answer
                FROM puzzles
                WHERE enabled = 1
                  AND id NOT IN (SELECT puzzle_id FROM used_puzzles WHERE room_name = ?)
                ORDER BY RANDOM()
                LIMIT 1
                "#,
            )
            .bind(room_name)
            .fetch_optional(&self.pool)
            .await?
        } else {
            sqlx::query(
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
            .bind(pack_id.unwrap())
            .bind(room_name)
            .fetch_optional(&self.pool)
            .await?
        };

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

            let row = if use_all_packs {
                sqlx::query(
                    r#"
                    SELECT id, category, answer
                    FROM puzzles
                    WHERE enabled = 1
                    ORDER BY RANDOM()
                    LIMIT 1
                    "#,
                )
                .fetch_optional(&self.pool)
                .await?
            } else {
                sqlx::query(
                    r#"
                    SELECT id, category, answer
                    FROM puzzles
                    WHERE pack_id = ? AND enabled = 1
                    ORDER BY RANDOM()
                    LIMIT 1
                    "#,
                )
                .bind(pack_id.unwrap())
                .fetch_optional(&self.pool)
                .await?
            };

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
            "SELECT active_pack_id, vowel_cost, final_seconds, final_jackpot, prize_replace_csv, puzzle_display_seconds, prize_wedge_names, disconnect_timeout_secs FROM room_config WHERE room_name = ?",
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
                pack_id: row.get::<Option<i64>, _>("active_pack_id"),
                disconnect_timeout_secs: row.get::<Option<i64>, _>("disconnect_timeout_secs").unwrap_or(300),
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
            INSERT INTO room_config (room_name, active_pack_id, vowel_cost, final_seconds, final_jackpot, prize_replace_csv, puzzle_display_seconds, prize_wedge_names, disconnect_timeout_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(room_name) DO UPDATE SET
                active_pack_id = excluded.active_pack_id,
                vowel_cost = excluded.vowel_cost,
                final_seconds = excluded.final_seconds,
                final_jackpot = excluded.final_jackpot,
                prize_replace_csv = excluded.prize_replace_csv,
                puzzle_display_seconds = excluded.puzzle_display_seconds,
                prize_wedge_names = excluded.prize_wedge_names,
                disconnect_timeout_secs = excluded.disconnect_timeout_secs
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
        .bind(config.disconnect_timeout_secs)
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

    // ========== PASSKEY METHODS ==========

    /// Create a passkey credential
    pub async fn create_passkey(
        &self,
        id: &str,
        user_id: i64,
        public_key: &[u8],
        counter: i64,
        transports: Option<&str>,
        device_name: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO passkey_credentials (id, user_id, public_key, counter, transports, created_at, device_name)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(public_key)
        .bind(counter)
        .bind(transports)
        .bind(now_secs())
        .bind(device_name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get passkey credential by ID
    pub async fn get_passkey(&self, credential_id: &str) -> Result<Option<PasskeyCredential>, DbError> {
        let cred = sqlx::query_as::<_, PasskeyCredential>(
            "SELECT * FROM passkey_credentials WHERE id = ?",
        )
        .bind(credential_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(cred)
    }

    /// Get all passkeys for a user
    pub async fn get_user_passkeys(&self, user_id: i64) -> Result<Vec<PasskeyCredential>, DbError> {
        let creds = sqlx::query_as::<_, PasskeyCredential>(
            "SELECT * FROM passkey_credentials WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(creds)
    }

    /// Update passkey counter and last_used
    pub async fn update_passkey_counter(
        &self,
        credential_id: &str,
        counter: i64,
    ) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE passkey_credentials SET counter = ?, last_used_at = ? WHERE id = ?",
        )
        .bind(counter)
        .bind(now_secs())
        .bind(credential_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete a passkey credential
    pub async fn delete_passkey(&self, credential_id: &str, user_id: i64) -> Result<bool, DbError> {
        let result = sqlx::query(
            "DELETE FROM passkey_credentials WHERE id = ? AND user_id = ?",
        )
        .bind(credential_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    // ========== OAUTH ACCOUNT METHODS ==========

    /// Create or link an OAuth account
    pub async fn create_oauth_account(
        &self,
        user_id: i64,
        provider: &str,
        provider_user_id: &str,
        email: Option<&str>,
    ) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO oauth_accounts (user_id, provider, provider_user_id, email, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .bind(provider_user_id)
        .bind(email)
        .bind(now_secs())
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// Get OAuth account by provider and provider user ID
    pub async fn get_oauth_account(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Option<OAuthAccount>, DbError> {
        let account = sqlx::query_as::<_, OAuthAccount>(
            "SELECT * FROM oauth_accounts WHERE provider = ? AND provider_user_id = ?",
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(account)
    }

    /// Get all OAuth accounts for a user
    pub async fn get_user_oauth_accounts(&self, user_id: i64) -> Result<Vec<OAuthAccount>, DbError> {
        let accounts = sqlx::query_as::<_, OAuthAccount>(
            "SELECT * FROM oauth_accounts WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(accounts)
    }

    /// Delete an OAuth account link
    pub async fn delete_oauth_account(&self, id: i64, user_id: i64) -> Result<bool, DbError> {
        let result = sqlx::query(
            "DELETE FROM oauth_accounts WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    // ========== WEBAUTHN CHALLENGE METHODS ==========

    /// Store a WebAuthn challenge
    pub async fn store_challenge(
        &self,
        challenge: &str,
        user_id: Option<i64>,
        email: Option<&str>,
        challenge_type: &str,
        expires_secs: i64,
    ) -> Result<(), DbError> {
        let now = now_secs();
        sqlx::query(
            r#"
            INSERT INTO webauthn_challenges (challenge, user_id, email, type, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(challenge)
        .bind(user_id)
        .bind(email)
        .bind(challenge_type)
        .bind(now)
        .bind(now + expires_secs)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get and delete a challenge (one-time use)
    pub async fn consume_challenge(&self, challenge: &str) -> Result<Option<WebAuthnChallenge>, DbError> {
        // First get the challenge
        let row = sqlx::query(
            "SELECT challenge, user_id, email, type, created_at, expires_at FROM webauthn_challenges WHERE challenge = ?",
        )
        .bind(challenge)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let challenge_data = WebAuthnChallenge {
                challenge: row.get("challenge"),
                user_id: row.get("user_id"),
                email: row.get("email"),
                challenge_type: row.get("type"),
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
            };

            // Delete it (one-time use)
            sqlx::query("DELETE FROM webauthn_challenges WHERE challenge = ?")
                .bind(&challenge_data.challenge)
                .execute(&self.pool)
                .await?;

            // Check if expired
            if challenge_data.expires_at < now_secs() {
                return Ok(None);
            }

            Ok(Some(challenge_data))
        } else {
            Ok(None)
        }
    }

    /// Clean up expired challenges
    pub async fn cleanup_expired_challenges(&self) -> Result<u64, DbError> {
        let result = sqlx::query("DELETE FROM webauthn_challenges WHERE expires_at < ?")
            .bind(now_secs())
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    // ========== USER METHODS (OAUTH ADDITIONS) ==========

    /// Create user from OAuth (no password)
    pub async fn create_oauth_user(
        &self,
        email: &str,
        display_name: &str,
        verified: bool,
    ) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO users (email, password_hash, display_name, verified, created_at)
            VALUES (?, NULL, ?, ?, ?)
            "#,
        )
        .bind(email.to_lowercase())
        .bind(display_name)
        .bind(verified)
        .bind(now_secs())
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// Check if user has a password set
    pub async fn user_has_password(&self, user_id: i64) -> Result<bool, DbError> {
        let row = sqlx::query("SELECT password_hash FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row
            .and_then(|r| r.get::<Option<String>, _>("password_hash"))
            .is_some())
    }

    /// Set password for user (for OAuth users adding password)
    pub async fn set_user_password(&self, user_id: i64, password_hash: &str) -> Result<(), DbError> {
        sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a temporary test database
    async fn create_test_db() -> Database {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        // Keep the tempfile handle alive by leaking it (tests are short-lived)
        std::mem::forget(tmp);
        Database::new(&path).await.unwrap()
    }

    // ========== USER TESTS ==========

    #[tokio::test]
    async fn test_create_user() {
        let db = create_test_db().await;

        let user_id = db
            .create_user(NewUser {
                email: "test@example.com".to_string(),
                password_hash: "hash123".to_string(),
                display_name: "Test User".to_string(),
                verification_token: "token123".to_string(),
                verification_token_expires: 9999999999,
            })
            .await
            .unwrap();

        assert!(user_id > 0);
    }

    #[tokio::test]
    async fn test_user_exists() {
        let db = create_test_db().await;

        // User doesn't exist yet
        assert!(!db.user_exists("test@example.com").await.unwrap());

        // Create user
        db.create_user(NewUser {
            email: "test@example.com".to_string(),
            password_hash: "hash123".to_string(),
            display_name: "Test User".to_string(),
            verification_token: "token123".to_string(),
            verification_token_expires: 9999999999,
        })
        .await
        .unwrap();

        // Now user exists
        assert!(db.user_exists("test@example.com").await.unwrap());
    }

    #[tokio::test]
    async fn test_user_exists_case_insensitive() {
        let db = create_test_db().await;

        db.create_user(NewUser {
            email: "Test@Example.COM".to_string(),
            password_hash: "hash123".to_string(),
            display_name: "Test User".to_string(),
            verification_token: "token123".to_string(),
            verification_token_expires: 9999999999,
        })
        .await
        .unwrap();

        // Should find with different case
        assert!(db.user_exists("test@example.com").await.unwrap());
        assert!(db.user_exists("TEST@EXAMPLE.COM").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_user_by_email() {
        let db = create_test_db().await;

        db.create_user(NewUser {
            email: "test@example.com".to_string(),
            password_hash: "hash123".to_string(),
            display_name: "Test User".to_string(),
            verification_token: "token123".to_string(),
            verification_token_expires: 9999999999,
        })
        .await
        .unwrap();

        let user = db.get_user_by_email("test@example.com").await.unwrap();
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.display_name, "Test User");
        assert_eq!(user.password_hash, Some("hash123".to_string()));
    }

    #[tokio::test]
    async fn test_get_user_by_email_not_found() {
        let db = create_test_db().await;

        let user = db.get_user_by_email("nonexistent@example.com").await.unwrap();
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        let db = create_test_db().await;

        let user_id = db
            .create_user(NewUser {
                email: "test@example.com".to_string(),
                password_hash: "hash123".to_string(),
                display_name: "Test User".to_string(),
                verification_token: "token123".to_string(),
                verification_token_expires: 9999999999,
            })
            .await
            .unwrap();

        let user = db.get_user_by_id(user_id).await.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn test_verify_user() {
        let db = create_test_db().await;

        let user_id = db
            .create_user(NewUser {
                email: "test@example.com".to_string(),
                password_hash: "hash123".to_string(),
                display_name: "Test User".to_string(),
                verification_token: "token123".to_string(),
                verification_token_expires: 9999999999,
            })
            .await
            .unwrap();

        // User not verified initially
        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert!(!user.verified);
        assert!(user.verification_token.is_some());

        // Verify user
        db.verify_user(user_id).await.unwrap();

        // User now verified, token cleared
        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert!(user.verified);
        assert!(user.verification_token.is_none());
    }

    #[tokio::test]
    async fn test_get_user_by_verification_token() {
        let db = create_test_db().await;

        db.create_user(NewUser {
            email: "test@example.com".to_string(),
            password_hash: "hash123".to_string(),
            display_name: "Test User".to_string(),
            verification_token: "unique-token-123".to_string(),
            verification_token_expires: 9999999999,
        })
        .await
        .unwrap();

        let user = db
            .get_user_by_verification_token("unique-token-123")
            .await
            .unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn test_remember_token() {
        let db = create_test_db().await;

        let user_id = db
            .create_user(NewUser {
                email: "test@example.com".to_string(),
                password_hash: "hash123".to_string(),
                display_name: "Test User".to_string(),
                verification_token: "token123".to_string(),
                verification_token_expires: 9999999999,
            })
            .await
            .unwrap();

        // Set remember token
        db.set_remember_token(user_id, "remember-me-token").await.unwrap();

        // Get user by token
        let user = db.get_user_by_token("remember-me-token").await.unwrap();
        assert!(user.is_some());

        // Clear token
        db.clear_remember_token(user_id).await.unwrap();

        // Token no longer works
        let user = db.get_user_by_token("remember-me-token").await.unwrap();
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn test_create_oauth_user() {
        let db = create_test_db().await;

        let user_id = db
            .create_oauth_user("oauth@example.com", "OAuth User", true)
            .await
            .unwrap();

        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert_eq!(user.email, "oauth@example.com");
        assert_eq!(user.display_name, "OAuth User");
        assert!(user.verified);
        assert!(user.password_hash.is_none()); // No password for OAuth users
    }

    #[tokio::test]
    async fn test_user_has_password() {
        let db = create_test_db().await;

        // OAuth user without password
        let oauth_user_id = db
            .create_oauth_user("oauth@example.com", "OAuth User", true)
            .await
            .unwrap();
        assert!(!db.user_has_password(oauth_user_id).await.unwrap());

        // Regular user with password
        let regular_user_id = db
            .create_user(NewUser {
                email: "regular@example.com".to_string(),
                password_hash: "hash123".to_string(),
                display_name: "Regular User".to_string(),
                verification_token: "token".to_string(),
                verification_token_expires: 9999999999,
            })
            .await
            .unwrap();
        assert!(db.user_has_password(regular_user_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let db = create_test_db().await;

        let user_id = db
            .create_user(NewUser {
                email: "test@example.com".to_string(),
                password_hash: "hash123".to_string(),
                display_name: "Test User".to_string(),
                verification_token: "token123".to_string(),
                verification_token_expires: 9999999999,
            })
            .await
            .unwrap();

        assert!(db.get_user_by_id(user_id).await.unwrap().is_some());

        let deleted = db.delete_user(user_id).await.unwrap();
        assert!(deleted);

        assert!(db.get_user_by_id(user_id).await.unwrap().is_none());
    }

    // ========== PUZZLE TESTS ==========

    #[tokio::test]
    async fn test_get_random_puzzle() {
        let db = create_test_db().await;

        // Default puzzles are inserted during init
        let puzzle = db.get_random_puzzle("test-room", None).await.unwrap();

        assert!(!puzzle.category.is_empty());
        assert!(!puzzle.answer.is_empty());
    }

    #[tokio::test]
    async fn test_puzzle_not_repeated() {
        let db = create_test_db().await;

        // Get first puzzle
        let puzzle1 = db.get_random_puzzle("test-room", None).await.unwrap();

        // Get more puzzles, should not repeat the first one until we've used them all
        let mut seen_ids = vec![puzzle1.id];

        // Get up to 9 more puzzles (we have 10 default puzzles)
        for _ in 0..9 {
            let puzzle = db.get_random_puzzle("test-room", None).await.unwrap();
            if seen_ids.contains(&puzzle.id) {
                // We've cycled through all puzzles
                break;
            }
            seen_ids.push(puzzle.id);
        }

        // Should have seen multiple unique puzzles
        assert!(seen_ids.len() > 1);
    }

    #[tokio::test]
    async fn test_clear_used_puzzles() {
        let db = create_test_db().await;

        // Get all puzzles once
        for _ in 0..10 {
            db.get_random_puzzle("test-room", None).await.unwrap();
        }

        // Clear used puzzles
        db.clear_used_puzzles("test-room").await.unwrap();

        // Should be able to get puzzles again
        let puzzle = db.get_random_puzzle("test-room", None).await.unwrap();
        assert!(puzzle.id > 0);
    }

    #[tokio::test]
    async fn test_add_puzzle() {
        let db = create_test_db().await;

        let puzzle_id = db.add_puzzle("TEST CATEGORY", "test answer", 1).await.unwrap();
        assert!(puzzle_id > 0);

        // Verify puzzle was added with uppercase answer
        let puzzles = db.list_all_puzzles(Some(1)).await.unwrap();
        let added = puzzles.iter().find(|p| p.id == puzzle_id).unwrap();
        assert_eq!(added.category, "TEST CATEGORY");
        assert_eq!(added.answer, "TEST ANSWER"); // Should be uppercased
    }

    #[tokio::test]
    async fn test_delete_puzzle() {
        let db = create_test_db().await;

        let puzzle_id = db.add_puzzle("TEST", "DELETE ME", 1).await.unwrap();

        let deleted = db.delete_puzzle(puzzle_id).await.unwrap();
        assert!(deleted);

        // Should no longer exist
        let puzzles = db.list_all_puzzles(None).await.unwrap();
        assert!(!puzzles.iter().any(|p| p.id == puzzle_id));
    }

    // ========== PACK TESTS ==========

    #[tokio::test]
    async fn test_list_packs() {
        let db = create_test_db().await;

        let packs = db.list_packs().await.unwrap();
        // Default pack should exist
        assert!(!packs.is_empty());
        assert!(packs.iter().any(|p| p.name == "Default"));
    }

    #[tokio::test]
    async fn test_get_or_create_pack() {
        let db = create_test_db().await;

        // Create new pack
        let pack_id = db.get_or_create_pack("Test Pack").await.unwrap();
        assert!(pack_id > 1); // Not the default pack

        // Get same pack again
        let pack_id2 = db.get_or_create_pack("Test Pack").await.unwrap();
        assert_eq!(pack_id, pack_id2);
    }

    #[tokio::test]
    async fn test_delete_pack() {
        let db = create_test_db().await;

        // Create a pack
        let pack_id = db.get_or_create_pack("Deletable Pack").await.unwrap();

        // Add a puzzle to it
        db.add_puzzle("TEST", "TEST PUZZLE", pack_id).await.unwrap();

        // Delete pack
        let deleted = db.delete_pack(pack_id).await.unwrap();
        assert!(deleted);

        // Pack's puzzles should be deleted too
        let puzzles = db.list_all_puzzles(Some(pack_id)).await.unwrap();
        assert!(puzzles.is_empty());
    }

    #[tokio::test]
    async fn test_cannot_delete_default_pack() {
        let db = create_test_db().await;

        let deleted = db.delete_pack(1).await.unwrap();
        assert!(!deleted); // Should return false
    }

    // ========== OAUTH ACCOUNT TESTS ==========

    #[tokio::test]
    async fn test_create_oauth_account() {
        let db = create_test_db().await;

        let user_id = db
            .create_oauth_user("test@example.com", "Test User", true)
            .await
            .unwrap();

        let account_id = db
            .create_oauth_account(user_id, "google", "google-user-123", Some("test@example.com"))
            .await
            .unwrap();

        assert!(account_id > 0);
    }

    #[tokio::test]
    async fn test_get_oauth_account() {
        let db = create_test_db().await;

        let user_id = db
            .create_oauth_user("test@example.com", "Test User", true)
            .await
            .unwrap();

        db.create_oauth_account(user_id, "google", "google-user-123", Some("test@example.com"))
            .await
            .unwrap();

        let account = db.get_oauth_account("google", "google-user-123").await.unwrap();
        assert!(account.is_some());
        let account = account.unwrap();
        assert_eq!(account.user_id, user_id);
        assert_eq!(account.provider, "google");
    }

    #[tokio::test]
    async fn test_get_user_oauth_accounts() {
        let db = create_test_db().await;

        let user_id = db
            .create_oauth_user("test@example.com", "Test User", true)
            .await
            .unwrap();

        // Add multiple OAuth accounts
        db.create_oauth_account(user_id, "google", "google-123", Some("test@example.com"))
            .await
            .unwrap();
        db.create_oauth_account(user_id, "apple", "apple-456", Some("test@example.com"))
            .await
            .unwrap();

        let accounts = db.get_user_oauth_accounts(user_id).await.unwrap();
        assert_eq!(accounts.len(), 2);
    }

    // ========== PASSKEY TESTS ==========

    #[tokio::test]
    async fn test_create_passkey() {
        let db = create_test_db().await;

        let user_id = db
            .create_oauth_user("test@example.com", "Test User", true)
            .await
            .unwrap();

        db.create_passkey(
            "credential-id-123",
            user_id,
            &[1, 2, 3, 4],
            0,
            Some("[\"internal\"]"),
            Some("My Device"),
        )
        .await
        .unwrap();

        // Verify passkey was created
        let passkey = db.get_passkey("credential-id-123").await.unwrap();
        assert!(passkey.is_some());
        let passkey = passkey.unwrap();
        assert_eq!(passkey.user_id, user_id);
        assert_eq!(passkey.device_name, Some("My Device".to_string()));
    }

    #[tokio::test]
    async fn test_get_user_passkeys() {
        let db = create_test_db().await;

        let user_id = db
            .create_oauth_user("test@example.com", "Test User", true)
            .await
            .unwrap();

        db.create_passkey("cred-1", user_id, &[1, 2], 0, None, Some("Device 1"))
            .await
            .unwrap();
        db.create_passkey("cred-2", user_id, &[3, 4], 0, None, Some("Device 2"))
            .await
            .unwrap();

        let passkeys = db.get_user_passkeys(user_id).await.unwrap();
        assert_eq!(passkeys.len(), 2);
    }

    #[tokio::test]
    async fn test_update_passkey_counter() {
        let db = create_test_db().await;

        let user_id = db
            .create_oauth_user("test@example.com", "Test User", true)
            .await
            .unwrap();

        db.create_passkey("cred-123", user_id, &[1, 2, 3], 0, None, None)
            .await
            .unwrap();

        // Update counter
        db.update_passkey_counter("cred-123", 5).await.unwrap();

        let passkey = db.get_passkey("cred-123").await.unwrap().unwrap();
        assert_eq!(passkey.counter, 5);
        assert!(passkey.last_used_at.is_some());
    }

    #[tokio::test]
    async fn test_delete_passkey() {
        let db = create_test_db().await;

        let user_id = db
            .create_oauth_user("test@example.com", "Test User", true)
            .await
            .unwrap();

        db.create_passkey("cred-to-delete", user_id, &[1, 2], 0, None, None)
            .await
            .unwrap();

        let deleted = db.delete_passkey("cred-to-delete", user_id).await.unwrap();
        assert!(deleted);

        assert!(db.get_passkey("cred-to-delete").await.unwrap().is_none());
    }

    // ========== WEBAUTHN CHALLENGE TESTS ==========

    #[tokio::test]
    async fn test_store_and_consume_challenge() {
        let db = create_test_db().await;

        db.store_challenge(
            "challenge-abc",
            Some(1),
            Some("test@example.com"),
            "registration",
            300, // 5 minutes
        )
        .await
        .unwrap();

        // Consume challenge
        let challenge = db.consume_challenge("challenge-abc").await.unwrap();
        assert!(challenge.is_some());
        let challenge = challenge.unwrap();
        assert_eq!(challenge.challenge, "challenge-abc");
        assert_eq!(challenge.challenge_type, "registration");

        // Challenge should be deleted (one-time use)
        let challenge_again = db.consume_challenge("challenge-abc").await.unwrap();
        assert!(challenge_again.is_none());
    }

    #[tokio::test]
    async fn test_expired_challenge() {
        let db = create_test_db().await;

        // Store challenge that's already expired
        db.store_challenge(
            "expired-challenge",
            None,
            Some("test@example.com"),
            "authentication",
            -1, // Already expired
        )
        .await
        .unwrap();

        // Should not be consumable
        let challenge = db.consume_challenge("expired-challenge").await.unwrap();
        assert!(challenge.is_none());
    }

    // ========== ROOM CONFIG TESTS ==========

    #[tokio::test]
    async fn test_get_room_config_default() {
        let db = create_test_db().await;

        let config = db.get_room_config("new-room").await.unwrap();

        // Should return defaults
        assert_eq!(config.vowel_cost, 250);
        assert_eq!(config.final_seconds, 30);
        assert_eq!(config.final_jackpot, 10000);
    }

    #[tokio::test]
    async fn test_set_room_config() {
        let db = create_test_db().await;

        let config = RoomConfig {
            vowel_cost: 500,
            final_seconds: 60,
            final_jackpot: 50000,
            prize_replace_cash_values: vec![1000, 2000, 3000],
            puzzle_display_seconds: 45,
            prize_wedge_names: vec!["PRIZE 1".to_string(), "PRIZE 2".to_string()],
            pack_id: Some(1),
            disconnect_timeout_secs: 300,
        };

        db.set_room_config("custom-room", &config, Some(1)).await.unwrap();

        let loaded = db.get_room_config("custom-room").await.unwrap();
        assert_eq!(loaded.vowel_cost, 500);
        assert_eq!(loaded.final_seconds, 60);
        assert_eq!(loaded.final_jackpot, 50000);
        assert_eq!(loaded.prize_replace_cash_values, vec![1000, 2000, 3000]);
        assert_eq!(loaded.prize_wedge_names, vec!["PRIZE 1", "PRIZE 2"]);
    }

    // ========== ROOM ACTIVITY TESTS ==========

    #[tokio::test]
    async fn test_update_room_activity() {
        let db = create_test_db().await;

        db.update_room_activity("active-room", None).await.unwrap();

        let rooms = db.list_active_rooms(24).await.unwrap();
        assert!(rooms.iter().any(|r| r.name == "active-room"));
    }

    #[tokio::test]
    async fn test_delete_room() {
        let db = create_test_db().await;

        db.update_room_activity("room-to-delete", None).await.unwrap();

        let deleted = db.delete_room("room-to-delete").await.unwrap();
        assert!(deleted);

        let rooms = db.list_active_rooms(24).await.unwrap();
        assert!(!rooms.iter().any(|r| r.name == "room-to-delete"));
    }

    // ========== ADMIN TESTS ==========

    #[tokio::test]
    async fn test_list_all_users() {
        let db = create_test_db().await;

        db.create_user(NewUser {
            email: "user1@example.com".to_string(),
            password_hash: "hash".to_string(),
            display_name: "User 1".to_string(),
            verification_token: "token1".to_string(),
            verification_token_expires: 9999999999,
        })
        .await
        .unwrap();

        db.create_user(NewUser {
            email: "user2@example.com".to_string(),
            password_hash: "hash".to_string(),
            display_name: "User 2".to_string(),
            verification_token: "token2".to_string(),
            verification_token_expires: 9999999999,
        })
        .await
        .unwrap();

        let users = db.list_all_users().await.unwrap();
        assert_eq!(users.len(), 2);
    }

    #[tokio::test]
    async fn test_set_user_admin() {
        let db = create_test_db().await;

        let user_id = db
            .create_user(NewUser {
                email: "test@example.com".to_string(),
                password_hash: "hash".to_string(),
                display_name: "Test".to_string(),
                verification_token: "token".to_string(),
                verification_token_expires: 9999999999,
            })
            .await
            .unwrap();

        // Initially not admin
        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert!(!user.is_admin);

        // Make admin
        db.set_user_admin(user_id, true).await.unwrap();

        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert!(user.is_admin);

        // Remove admin
        db.set_user_admin(user_id, false).await.unwrap();

        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert!(!user.is_admin);
    }

    #[tokio::test]
    async fn test_get_puzzle_counts() {
        let db = create_test_db().await;

        let counts = db.get_puzzle_counts().await.unwrap();

        // Default pack should have puzzles
        assert!(!counts.is_empty());
        let default_pack = counts.iter().find(|(_, name, _)| name == "Default").unwrap();
        assert!(default_pack.2 > 0); // Should have default puzzles
    }

    #[tokio::test]
    async fn test_import_puzzles() {
        let db = create_test_db().await;

        let puzzles = vec![
            ("Phrase".to_string(), "test puzzle one".to_string()),
            ("Thing".to_string(), "test puzzle two".to_string()),
        ];

        let count = db.import_puzzles(puzzles, 1).await.unwrap();
        assert_eq!(count, 2);

        // Verify puzzles were added with uppercase
        let all = db.list_all_puzzles(Some(1)).await.unwrap();
        assert!(all.iter().any(|p| p.answer == "TEST PUZZLE ONE"));
        assert!(all.iter().any(|p| p.answer == "TEST PUZZLE TWO"));
    }

    // ========== ADDITIONAL EDGE CASE TESTS ==========

    #[tokio::test]
    async fn test_duplicate_email_fails() {
        let db = create_test_db().await;

        db.create_user(NewUser {
            email: "duplicate@example.com".to_string(),
            password_hash: "hash".to_string(),
            display_name: "User 1".to_string(),
            verification_token: "token1".to_string(),
            verification_token_expires: 9999999999,
        })
        .await
        .unwrap();

        // Attempting to create another user with the same email should fail
        let result = db
            .create_user(NewUser {
                email: "duplicate@example.com".to_string(),
                password_hash: "hash".to_string(),
                display_name: "User 2".to_string(),
                verification_token: "token2".to_string(),
                verification_token_expires: 9999999999,
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_password_reset_token_flow() {
        let db = create_test_db().await;

        let user_id = db
            .create_user(NewUser {
                email: "reset@example.com".to_string(),
                password_hash: "old_hash".to_string(),
                display_name: "Reset User".to_string(),
                verification_token: "token".to_string(),
                verification_token_expires: 9999999999,
            })
            .await
            .unwrap();

        // Set reset token
        db.set_password_reset_token(user_id, "reset-token-123", 9999999999)
            .await
            .unwrap();

        // Verify reset token was set
        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert_eq!(user.reset_token, Some("reset-token-123".to_string()));
        assert_eq!(user.reset_token_expires, Some(9999999999));

        // Update password (should clear reset token)
        db.update_password(user_id, "new_hash").await.unwrap();

        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert_eq!(user.password_hash, Some("new_hash".to_string()));
        assert!(user.reset_token.is_none());
        assert!(user.reset_token_expires.is_none());
    }

    #[tokio::test]
    async fn test_update_last_login() {
        let db = create_test_db().await;

        let user_id = db
            .create_user(NewUser {
                email: "login@example.com".to_string(),
                password_hash: "hash".to_string(),
                display_name: "Login User".to_string(),
                verification_token: "token".to_string(),
                verification_token_expires: 9999999999,
            })
            .await
            .unwrap();

        // Initially no last_login_at
        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert!(user.last_login_at.is_none());

        // Update last login
        db.update_last_login(user_id).await.unwrap();

        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert!(user.last_login_at.is_some());
    }

    #[tokio::test]
    async fn test_set_verification_token() {
        let db = create_test_db().await;

        let user_id = db
            .create_user(NewUser {
                email: "verify@example.com".to_string(),
                password_hash: "hash".to_string(),
                display_name: "Verify User".to_string(),
                verification_token: "old-token".to_string(),
                verification_token_expires: 1000000000,
            })
            .await
            .unwrap();

        // Verify user first
        db.verify_user(user_id).await.unwrap();
        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert!(user.verified);

        // Resend verification (set new token)
        db.set_verification_token(user_id, "new-token", 9999999999)
            .await
            .unwrap();

        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert!(!user.verified); // Should be unverified again
        assert_eq!(user.verification_token, Some("new-token".to_string()));
        assert_eq!(user.verification_token_expires, Some(9999999999));
    }

    #[tokio::test]
    async fn test_set_user_password() {
        let db = create_test_db().await;

        // Create OAuth user (no password)
        let user_id = db
            .create_oauth_user("oauth-pw@example.com", "OAuth User", true)
            .await
            .unwrap();

        assert!(!db.user_has_password(user_id).await.unwrap());

        // Add password
        db.set_user_password(user_id, "new_password_hash")
            .await
            .unwrap();

        assert!(db.user_has_password(user_id).await.unwrap());
        let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
        assert_eq!(user.password_hash, Some("new_password_hash".to_string()));
    }

    #[tokio::test]
    async fn test_get_random_puzzle_with_pack_filter() {
        let db = create_test_db().await;

        // Create a new pack
        let pack_id = db.get_or_create_pack("Test Pack").await.unwrap();

        // Add puzzles to the new pack
        db.add_puzzle("Test Category", "TEST ANSWER ONE", pack_id)
            .await
            .unwrap();
        db.add_puzzle("Test Category", "TEST ANSWER TWO", pack_id)
            .await
            .unwrap();

        // Get puzzle from specific pack
        let puzzle = db.get_random_puzzle("pack-test-room", Some(pack_id)).await.unwrap();

        // Should be from our test pack
        assert!(puzzle.answer == "TEST ANSWER ONE" || puzzle.answer == "TEST ANSWER TWO");
    }

    #[tokio::test]
    async fn test_get_random_puzzle_pack_id_zero_uses_all() {
        let db = create_test_db().await;

        // Pack ID 0 should use all packs (like None)
        let puzzle = db.get_random_puzzle("all-packs-room", Some(0)).await.unwrap();

        // Should work and return a puzzle
        assert!(!puzzle.answer.is_empty());
    }

    #[tokio::test]
    async fn test_puzzle_enabled_toggle() {
        let db = create_test_db().await;

        let puzzle_id = db.add_puzzle("Test", "TOGGLE TEST", 1).await.unwrap();

        // Initially enabled
        let puzzles = db.list_all_puzzles(Some(1)).await.unwrap();
        let puzzle = puzzles.iter().find(|p| p.id == puzzle_id).unwrap();
        assert!(puzzle.enabled);

        // Disable puzzle
        db.set_puzzle_enabled(puzzle_id, false).await.unwrap();

        let puzzles = db.list_all_puzzles(Some(1)).await.unwrap();
        let puzzle = puzzles.iter().find(|p| p.id == puzzle_id).unwrap();
        assert!(!puzzle.enabled);

        // Re-enable puzzle
        db.set_puzzle_enabled(puzzle_id, true).await.unwrap();

        let puzzles = db.list_all_puzzles(Some(1)).await.unwrap();
        let puzzle = puzzles.iter().find(|p| p.id == puzzle_id).unwrap();
        assert!(puzzle.enabled);
    }

    #[tokio::test]
    async fn test_disabled_puzzles_not_returned() {
        let db = create_test_db().await;

        // Create a new pack with only one puzzle
        let pack_id = db.get_or_create_pack("Single Puzzle Pack").await.unwrap();
        let puzzle_id = db.add_puzzle("Test", "ONLY PUZZLE", pack_id).await.unwrap();

        // Get the puzzle once (should work)
        let puzzle = db.get_random_puzzle("disabled-test", Some(pack_id)).await.unwrap();
        assert_eq!(puzzle.answer, "ONLY PUZZLE");

        // Clear used puzzles and disable the puzzle
        db.clear_used_puzzles("disabled-test").await.unwrap();
        db.set_puzzle_enabled(puzzle_id, false).await.unwrap();

        // Should return error (no enabled puzzles)
        let result = db.get_random_puzzle("disabled-test", Some(pack_id)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_oauth_account() {
        let db = create_test_db().await;

        let user_id = db
            .create_oauth_user("oauth-delete@example.com", "OAuth Delete", true)
            .await
            .unwrap();

        let account_id = db
            .create_oauth_account(user_id, "google", "google-delete-123", None)
            .await
            .unwrap();

        // Verify account exists
        let accounts = db.get_user_oauth_accounts(user_id).await.unwrap();
        assert_eq!(accounts.len(), 1);

        // Delete account
        let deleted = db.delete_oauth_account(account_id, user_id).await.unwrap();
        assert!(deleted);

        // Verify account is gone
        let accounts = db.get_user_oauth_accounts(user_id).await.unwrap();
        assert!(accounts.is_empty());
    }

    #[tokio::test]
    async fn test_delete_oauth_account_wrong_user() {
        let db = create_test_db().await;

        let user1_id = db
            .create_oauth_user("user1@example.com", "User 1", true)
            .await
            .unwrap();
        let user2_id = db
            .create_oauth_user("user2@example.com", "User 2", true)
            .await
            .unwrap();

        let account_id = db
            .create_oauth_account(user1_id, "google", "google-user1", None)
            .await
            .unwrap();

        // Try to delete with wrong user ID
        let deleted = db.delete_oauth_account(account_id, user2_id).await.unwrap();
        assert!(!deleted);

        // Account should still exist
        let accounts = db.get_user_oauth_accounts(user1_id).await.unwrap();
        assert_eq!(accounts.len(), 1);
    }

    #[tokio::test]
    async fn test_set_active_pack() {
        let db = create_test_db().await;

        let pack_id = db.get_or_create_pack("Active Pack").await.unwrap();

        // Initially no active pack
        let active = db.get_active_pack_id("pack-room").await.unwrap();
        assert!(active.is_none());

        // Set active pack
        db.set_active_pack("pack-room", Some(pack_id)).await.unwrap();

        let active = db.get_active_pack_id("pack-room").await.unwrap();
        assert_eq!(active, Some(pack_id));

        // Clear active pack
        db.set_active_pack("pack-room", None).await.unwrap();

        let active = db.get_active_pack_id("pack-room").await.unwrap();
        assert!(active.is_none());
    }

    #[tokio::test]
    async fn test_list_active_rooms_respects_hours() {
        let db = create_test_db().await;

        db.update_room_activity("recent-room", None).await.unwrap();

        // Room should appear in 24-hour window
        let rooms = db.list_active_rooms(24).await.unwrap();
        assert!(rooms.iter().any(|r| r.name == "recent-room"));

        // Room should appear in 1-hour window (just created)
        let rooms = db.list_active_rooms(1).await.unwrap();
        assert!(rooms.iter().any(|r| r.name == "recent-room"));
    }

    #[tokio::test]
    async fn test_cleanup_expired_challenges() {
        let db = create_test_db().await;

        // Store an already-expired challenge
        db.store_challenge("expired-1", None, None, "auth", -60)
            .await
            .unwrap();
        db.store_challenge("expired-2", None, None, "auth", -30)
            .await
            .unwrap();
        // Store a valid challenge
        db.store_challenge("valid", None, None, "auth", 300)
            .await
            .unwrap();

        // Cleanup expired
        let cleaned = db.cleanup_expired_challenges().await.unwrap();
        assert_eq!(cleaned, 2);

        // Valid challenge should still be consumable
        let challenge = db.consume_challenge("valid").await.unwrap();
        assert!(challenge.is_some());
    }

    #[tokio::test]
    async fn test_add_puzzles_batch() {
        let db = create_test_db().await;

        let pack_id = db.get_or_create_pack("Batch Pack").await.unwrap();

        let puzzles = vec![
            ("Phrase".to_string(), "BATCH ONE".to_string()),
            ("Thing".to_string(), "BATCH TWO".to_string()),
            ("Place".to_string(), "BATCH THREE".to_string()),
        ];

        let count = db.add_puzzles(puzzles, pack_id).await.unwrap();
        assert_eq!(count, 3);

        let all = db.list_all_puzzles(Some(pack_id)).await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_database_connect_method() {
        // Test the connect method (alternate constructor)
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        std::mem::forget(tmp);

        let db = Database::connect(&format!("sqlite:{}", path)).await.unwrap();

        // Should work
        let packs = db.list_packs().await.unwrap();
        assert!(!packs.is_empty());
    }

    #[tokio::test]
    async fn test_get_user_by_id_not_found() {
        let db = create_test_db().await;

        let user = db.get_user_by_id(99999).await.unwrap();
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_user() {
        let db = create_test_db().await;

        let deleted = db.delete_user(99999).await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_puzzle() {
        let db = create_test_db().await;

        let deleted = db.delete_puzzle(99999).await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_room() {
        let db = create_test_db().await;

        let deleted = db.delete_room("nonexistent-room").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_passkey_wrong_user_delete() {
        let db = create_test_db().await;

        let user1 = db.create_oauth_user("pk1@example.com", "User 1", true).await.unwrap();
        let user2 = db.create_oauth_user("pk2@example.com", "User 2", true).await.unwrap();

        db.create_passkey("cred-user1", user1, &[1, 2, 3], 0, None, None)
            .await
            .unwrap();

        // Try to delete with wrong user
        let deleted = db.delete_passkey("cred-user1", user2).await.unwrap();
        assert!(!deleted);

        // Should still exist
        assert!(db.get_passkey("cred-user1").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_list_all_puzzles_no_filter() {
        let db = create_test_db().await;

        // Create additional pack with puzzles
        let pack_id = db.get_or_create_pack("Extra Pack").await.unwrap();
        db.add_puzzle("Test", "EXTRA PUZZLE", pack_id).await.unwrap();

        // List all puzzles (no pack filter)
        let all = db.list_all_puzzles(None).await.unwrap();

        // Should include default puzzles AND our extra puzzle
        assert!(all.len() > 10);
        assert!(all.iter().any(|p| p.answer == "EXTRA PUZZLE"));
    }

    #[tokio::test]
    async fn test_room_config_update_preserves_values() {
        let db = create_test_db().await;

        // Create packs first to satisfy foreign key constraint
        db.get_or_create_pack("Test Pack 1").await.unwrap();
        db.get_or_create_pack("Test Pack 2").await.unwrap();

        let config1 = RoomConfig {
            vowel_cost: 300,
            final_seconds: 45,
            final_jackpot: 20000,
            prize_replace_cash_values: vec![1000, 2000],
            puzzle_display_seconds: 60,
            prize_wedge_names: vec!["CAR".to_string()],
            pack_id: Some(1),
            disconnect_timeout_secs: 600,
        };

        db.set_room_config("config-test", &config1, Some(1)).await.unwrap();

        // Update with different values
        let config2 = RoomConfig {
            vowel_cost: 400,
            final_seconds: 60,
            final_jackpot: 30000,
            prize_replace_cash_values: vec![500],
            puzzle_display_seconds: 90,
            prize_wedge_names: vec!["TRIP".to_string(), "BOAT".to_string()],
            pack_id: Some(2),
            disconnect_timeout_secs: 120,
        };

        db.set_room_config("config-test", &config2, Some(2)).await.unwrap();

        let loaded = db.get_room_config("config-test").await.unwrap();
        assert_eq!(loaded.vowel_cost, 400);
        assert_eq!(loaded.final_seconds, 60);
        assert_eq!(loaded.final_jackpot, 30000);
        assert_eq!(loaded.prize_replace_cash_values, vec![500]);
        assert_eq!(loaded.puzzle_display_seconds, 90);
        assert_eq!(loaded.prize_wedge_names, vec!["TRIP", "BOAT"]);
        assert_eq!(loaded.disconnect_timeout_secs, 120);
    }
}
