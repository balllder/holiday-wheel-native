//! Configuration module for environment variable validation and loading.
//!
//! This module provides a centralized configuration system that:
//! - Validates required environment variables exist
//! - Validates format of URLs and ports
//! - Provides sensible defaults for optional variables
//! - Logs warnings for missing optional variables

use std::fmt;
use thiserror::Error;
use tracing::{info, warn};

/// Configuration errors that can occur during validation
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Required environment variable '{0}' is not set")]
    MissingRequired(String),

    #[error("Invalid port '{value}' for '{name}': {reason}")]
    InvalidPort {
        name: String,
        value: String,
        reason: String,
    },

    #[error("Invalid URL '{value}' for '{name}': {reason}")]
    InvalidUrl {
        name: String,
        value: String,
        reason: String,
    },

    #[error("Invalid boolean '{value}' for '{name}': expected 'true', 'false', '1', or '0'")]
    InvalidBoolean { name: String, value: String },

    #[error("Insecure HOST_CODE: {0}")]
    InsecureHostCode(String),

    #[error("Multiple configuration errors: {0:?}")]
    Multiple(Vec<ConfigError>),
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub ssl_enabled: bool,
    pub ssl_cert: Option<String>,
    pub ssl_key: Option<String>,
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

/// Email/SMTP configuration
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_email: String,
    pub base_url: String,
}

/// WebAuthn/Passkey configuration
#[derive(Debug, Clone)]
pub struct WebAuthnConfig {
    pub rp_id: String,
    pub rp_origin: String,
}

/// OAuth configuration
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub google_client_id: Option<String>,
    pub apple_client_id: Option<String>,
    pub apple_client_id_web: Option<String>,
    pub apple_redirect_uri: Option<String>,
}

/// Admin configuration
#[derive(Debug, Clone)]
pub struct AdminConfig {
    pub admin_email: Option<String>,
}

/// CORS configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins for CORS. Empty means allow all (development only).
    /// In production, this should be explicitly set via CORS_ALLOWED_ORIGINS.
    pub allowed_origins: Vec<String>,
    /// Whether CORS is in permissive mode (allows any origin)
    pub permissive: bool,
}

/// Game configuration
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Host code for claiming host privileges
    pub host_code: String,
    /// Whether using the default (insecure) host code
    pub using_default_host_code: bool,
}

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub email: EmailConfig,
    pub webauthn: WebAuthnConfig,
    pub oauth: OAuthConfig,
    pub admin: AdminConfig,
    pub cors: CorsConfig,
    pub game: GameConfig,
    pub rust_log: String,
}

impl Config {
    /// Load and validate configuration from environment variables.
    ///
    /// This method will:
    /// 1. Load the .env file if present
    /// 2. Validate required environment variables
    /// 3. Validate format of ports and URLs
    /// 4. Log warnings for missing optional variables
    /// 5. Return errors for any validation failures
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file (ignore if not present)
        dotenvy::dotenv().ok();

        Self::from_env_internal()
    }

    /// Load and validate configuration from environment variables without loading .env file.
    /// This is useful for testing where you want to control env vars directly.
    #[cfg(test)]
    pub fn from_env_no_dotenv() -> Result<Self, ConfigError> {
        Self::from_env_internal()
    }

    /// Internal implementation of config loading
    fn from_env_internal() -> Result<Self, ConfigError> {
        let mut errors: Vec<ConfigError> = Vec::new();

        // Server configuration
        let port = Self::parse_port("PORT", "5000", &mut errors);
        let ssl_enabled = Self::parse_bool("SSL_ENABLED", false, &mut errors);
        let ssl_cert = Self::get_optional("SSL_CERT");
        let ssl_key = Self::get_optional("SSL_KEY");

        // Validate SSL config consistency
        if ssl_enabled && (ssl_cert.is_none() || ssl_key.is_none()) {
            errors.push(ConfigError::MissingRequired(
                "SSL_CERT and SSL_KEY (required when SSL_ENABLED=true)".to_string(),
            ));
        }

        // Database configuration
        let db_path = std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("DB_PATH"))
            .unwrap_or_else(|_| {
                warn!("Neither DATABASE_URL nor DB_PATH set, using default 'puzzles.db'");
                "puzzles.db".to_string()
            });

        // Email configuration
        let email_enabled = Self::parse_bool("EMAIL_ENABLED", false, &mut errors);
        let smtp_host =
            Self::get_with_default("SMTP_HOST", "smtp.gmail.com", email_enabled, "SMTP_HOST");
        let smtp_port = Self::parse_port("SMTP_PORT", "587", &mut errors);
        let smtp_user = Self::get_with_default("SMTP_USER", "", email_enabled, "SMTP_USER");
        let smtp_pass = Self::get_with_default("SMTP_PASS", "", email_enabled, "SMTP_PASS");
        let from_email = Self::get_with_default(
            "FROM_EMAIL",
            "noreply@holidaywheel.com",
            false,
            "FROM_EMAIL",
        );
        let base_url =
            Self::get_with_default("BASE_URL", "http://localhost:5000", false, "BASE_URL");

        // Validate email URL format
        if let Err(e) = Self::validate_url(&base_url, "BASE_URL") {
            errors.push(e);
        }

        // Validate SMTP credentials if email is enabled
        if email_enabled && smtp_user.is_empty() {
            errors.push(ConfigError::MissingRequired(
                "SMTP_USER (required when EMAIL_ENABLED=true)".to_string(),
            ));
        }
        if email_enabled && smtp_pass.is_empty() {
            errors.push(ConfigError::MissingRequired(
                "SMTP_PASS (required when EMAIL_ENABLED=true)".to_string(),
            ));
        }

        // WebAuthn configuration
        let webauthn_rp_id =
            Self::get_with_default("WEBAUTHN_RP_ID", "localhost", false, "WEBAUTHN_RP_ID");
        let webauthn_rp_origin = Self::get_with_default(
            "WEBAUTHN_RP_ORIGIN",
            "http://localhost:5000",
            false,
            "WEBAUTHN_RP_ORIGIN",
        );

        // Validate WebAuthn origin URL
        if let Err(e) = Self::validate_url(&webauthn_rp_origin, "WEBAUTHN_RP_ORIGIN") {
            errors.push(e);
        }

        // OAuth configuration (all optional)
        let google_client_id = Self::get_optional_with_warning("GOOGLE_CLIENT_ID", "Google OAuth");
        let apple_client_id =
            Self::get_optional_with_warning("APPLE_CLIENT_ID", "Apple OAuth (native)");
        let apple_client_id_web =
            Self::get_optional_with_warning("APPLE_CLIENT_ID_WEB", "Apple OAuth (web)");
        let apple_redirect_uri = Self::get_optional("APPLE_REDIRECT_URI");

        // Validate Apple redirect URI if provided
        if let Some(ref uri) = apple_redirect_uri {
            if let Err(e) = Self::validate_url(uri, "APPLE_REDIRECT_URI") {
                errors.push(e);
            }
        }

        // Admin configuration
        let admin_email = Self::get_optional_with_warning("ADMIN_EMAIL", "auto-admin assignment");

        // CORS configuration
        let (cors_origins, cors_permissive) = Self::parse_cors_origins();

        // Game configuration
        let (host_code, using_default_host_code) = match Self::parse_host_code() {
            Ok(result) => result,
            Err(e) => {
                errors.push(e);
                // Return placeholder values; startup will fail due to error
                ("placeholder".to_string(), true)
            }
        };

        // Logging configuration
        let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        // Return errors if any validation failed
        if !errors.is_empty() {
            if errors.len() == 1 {
                return Err(errors.remove(0));
            }
            return Err(ConfigError::Multiple(errors));
        }

        Ok(Config {
            server: ServerConfig {
                port,
                ssl_enabled,
                ssl_cert,
                ssl_key,
            },
            database: DatabaseConfig { path: db_path },
            email: EmailConfig {
                enabled: email_enabled,
                smtp_host,
                smtp_port,
                smtp_user,
                smtp_pass,
                from_email,
                base_url,
            },
            webauthn: WebAuthnConfig {
                rp_id: webauthn_rp_id,
                rp_origin: webauthn_rp_origin,
            },
            oauth: OAuthConfig {
                google_client_id,
                apple_client_id,
                apple_client_id_web,
                apple_redirect_uri,
            },
            admin: AdminConfig { admin_email },
            cors: CorsConfig {
                allowed_origins: cors_origins,
                permissive: cors_permissive,
            },
            game: GameConfig {
                host_code,
                using_default_host_code,
            },
            rust_log,
        })
    }

    /// Get an optional environment variable
    fn get_optional(name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    /// Get an optional environment variable, logging a warning if not set
    fn get_optional_with_warning(name: &str, feature: &str) -> Option<String> {
        match std::env::var(name) {
            Ok(val) => Some(val),
            Err(_) => {
                warn!("{} not set, {} will be disabled", name, feature);
                None
            }
        }
    }

    /// Get environment variable with default, optionally warning if required by feature
    fn get_with_default(
        name: &str,
        default: &str,
        warn_if_missing: bool,
        warning_name: &str,
    ) -> String {
        match std::env::var(name) {
            Ok(val) => val,
            Err(_) => {
                if warn_if_missing {
                    warn!("{} not set, using default '{}'", warning_name, default);
                }
                default.to_string()
            }
        }
    }

    /// Parse a port number from environment variable
    fn parse_port(name: &str, default: &str, errors: &mut Vec<ConfigError>) -> u16 {
        let value = std::env::var(name).unwrap_or_else(|_| default.to_string());
        match value.parse::<u16>() {
            Ok(port) => {
                if port == 0 {
                    errors.push(ConfigError::InvalidPort {
                        name: name.to_string(),
                        value,
                        reason: "port cannot be 0".to_string(),
                    });
                    5000 // Return default on error
                } else {
                    port
                }
            }
            Err(e) => {
                errors.push(ConfigError::InvalidPort {
                    name: name.to_string(),
                    value,
                    reason: e.to_string(),
                });
                5000 // Return default on error
            }
        }
    }

    /// Parse a boolean from environment variable
    fn parse_bool(name: &str, default: bool, errors: &mut Vec<ConfigError>) -> bool {
        match std::env::var(name) {
            Ok(value) => {
                let lower = value.to_lowercase();
                match lower.as_str() {
                    "true" | "1" | "yes" => true,
                    "false" | "0" | "no" => false,
                    _ => {
                        errors.push(ConfigError::InvalidBoolean {
                            name: name.to_string(),
                            value,
                        });
                        default
                    }
                }
            }
            Err(_) => default,
        }
    }

    /// Validate URL format
    fn validate_url(url: &str, name: &str) -> Result<(), ConfigError> {
        // Basic URL validation - must have scheme and host
        if url.is_empty() {
            return Err(ConfigError::InvalidUrl {
                name: name.to_string(),
                value: url.to_string(),
                reason: "URL cannot be empty".to_string(),
            });
        }

        // Check for valid scheme
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ConfigError::InvalidUrl {
                name: name.to_string(),
                value: url.to_string(),
                reason: "URL must start with http:// or https://".to_string(),
            });
        }

        // Check for host after scheme
        let after_scheme = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or("");

        if after_scheme.is_empty() || after_scheme.starts_with('/') {
            return Err(ConfigError::InvalidUrl {
                name: name.to_string(),
                value: url.to_string(),
                reason: "URL must have a host".to_string(),
            });
        }

        Ok(())
    }

    /// Parse CORS allowed origins from environment variable.
    ///
    /// CORS_ALLOWED_ORIGINS can be:
    /// - Not set or empty: Permissive mode (allow any origin) - logs warning in production
    /// - "*": Explicitly permissive mode (allow any origin) - logs warning
    /// - Comma-separated URLs: Strict mode with specific allowed origins
    ///
    /// Returns (origins, is_permissive)
    fn parse_cors_origins() -> (Vec<String>, bool) {
        match std::env::var("CORS_ALLOWED_ORIGINS") {
            Ok(value) if value.trim() == "*" => {
                warn!(
                    "CORS_ALLOWED_ORIGINS is set to '*' (allow any origin). \
                    This is insecure for production! Set specific origins."
                );
                (vec![], true)
            }
            Ok(value) if !value.trim().is_empty() => {
                let origins: Vec<String> = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if origins.is_empty() {
                    warn!(
                        "CORS_ALLOWED_ORIGINS was set but contains no valid origins. \
                        Falling back to permissive mode."
                    );
                    (vec![], true)
                } else {
                    info!("CORS configured with {} allowed origin(s)", origins.len());
                    (origins, false)
                }
            }
            _ => {
                warn!(
                    "CORS_ALLOWED_ORIGINS not set. Using permissive mode (allow any origin). \
                    This is insecure for production! Set CORS_ALLOWED_ORIGINS=https://your-domain.com"
                );
                (vec![], true)
            }
        }
    }

    /// Parse HOST_CODE from environment variable.
    ///
    /// HOST_CODE must be at least 8 characters and cannot be the insecure default "holiday".
    /// If HOST_CODE is not set or invalid, returns an error that will fail startup.
    /// Returns (host_code, using_default)
    fn parse_host_code() -> Result<(String, bool), ConfigError> {
        match std::env::var("HOST_CODE") {
            Ok(code) if !code.trim().is_empty() => {
                let code = code.trim().to_string();

                // Check for insecure default value
                if code.to_lowercase() == "holiday" {
                    return Err(ConfigError::InsecureHostCode(
                        "HOST_CODE cannot be 'holiday'. Please set a secure value (minimum 8 characters).".to_string()
                    ));
                }

                // Enforce minimum length
                if code.len() < 8 {
                    return Err(ConfigError::InsecureHostCode(
                        format!(
                            "HOST_CODE must be at least 8 characters (got {} chars). \
                            Please set a secure value.",
                            code.len()
                        )
                    ));
                }

                Ok((code, false))
            }
            _ => {
                Err(ConfigError::InsecureHostCode(
                    "HOST_CODE environment variable is required. \
                    Please set HOST_CODE to a secure value (minimum 8 characters)."
                        .to_string()
                ))
            }
        }
    }

    /// Log the loaded configuration (with sensitive values redacted)
    pub fn log_config(&self) {
        info!("Configuration loaded:");
        info!("  Server:");
        info!("    Port: {}", self.server.port);
        info!("    SSL Enabled: {}", self.server.ssl_enabled);
        if self.server.ssl_enabled {
            info!(
                "    SSL Cert: {}",
                self.server.ssl_cert.as_deref().unwrap_or("not set")
            );
            info!(
                "    SSL Key: {}",
                self.server.ssl_key.as_deref().unwrap_or("not set")
            );
        }
        info!("  Database:");
        info!("    Path: {}", self.database.path);
        info!("  Email:");
        info!("    Enabled: {}", self.email.enabled);
        if self.email.enabled {
            info!("    SMTP Host: {}", self.email.smtp_host);
            info!("    SMTP Port: {}", self.email.smtp_port);
            info!("    SMTP User: {}", redact(&self.email.smtp_user));
            info!("    From: {}", self.email.from_email);
        }
        info!("    Base URL: {}", self.email.base_url);
        info!("  WebAuthn:");
        info!("    RP ID: {}", self.webauthn.rp_id);
        info!("    RP Origin: {}", self.webauthn.rp_origin);
        info!("  OAuth:");
        info!(
            "    Google Client ID: {}",
            self.oauth
                .google_client_id
                .as_ref()
                .map(|s| redact(s))
                .unwrap_or_else(|| "not set".to_string())
        );
        info!(
            "    Apple Client ID: {}",
            self.oauth
                .apple_client_id
                .as_deref()
                .unwrap_or("not set")
        );
        info!(
            "    Apple Client ID (web): {}",
            self.oauth
                .apple_client_id_web
                .as_deref()
                .unwrap_or("not set")
        );
        info!("  Admin:");
        info!(
            "    Admin Email: {}",
            self.admin.admin_email.as_deref().unwrap_or("not set")
        );
        info!("  CORS:");
        if self.cors.permissive {
            info!("    Mode: PERMISSIVE (any origin allowed) - NOT SECURE FOR PRODUCTION");
        } else {
            info!("    Mode: STRICT ({} origins)", self.cors.allowed_origins.len());
            for origin in &self.cors.allowed_origins {
                info!("    - {}", origin);
            }
        }
        info!("  Game:");
        if self.game.using_default_host_code {
            info!("    Host Code: DEFAULT ('holiday') - NOT SECURE FOR PRODUCTION");
        } else {
            info!("    Host Code: {} (custom)", redact(&self.game.host_code));
        }
        info!("  Logging: {}", self.rust_log);
    }
}

/// Redact sensitive values for logging
fn redact(value: &str) -> String {
    if value.is_empty() {
        return "not set".to_string();
    }
    if value.len() <= 4 {
        return "****".to_string();
    }
    format!("{}****", &value[..4])
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Config {{ port: {}, ssl: {}, db: {}, email: {} }}",
            self.server.port, self.server.ssl_enabled, self.database.path, self.email.enabled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Mutex to serialize config tests that modify environment variables
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// All environment variables that Config reads - used to isolate tests
    const ALL_CONFIG_VARS: &[&str] = &[
        "PORT",
        "SSL_ENABLED",
        "SSL_CERT",
        "SSL_KEY",
        "DATABASE_URL",
        "DB_PATH",
        "EMAIL_ENABLED",
        "SMTP_HOST",
        "SMTP_PORT",
        "SMTP_USER",
        "SMTP_PASS",
        "FROM_EMAIL",
        "BASE_URL",
        "WEBAUTHN_RP_ID",
        "WEBAUTHN_RP_ORIGIN",
        "GOOGLE_CLIENT_ID",
        "APPLE_CLIENT_ID",
        "APPLE_CLIENT_ID_WEB",
        "APPLE_REDIRECT_URI",
        "ADMIN_EMAIL",
        "CORS_ALLOWED_ORIGINS",
        "HOST_CODE",
        "RUST_LOG",
    ];

    /// Valid HOST_CODE for tests that don't specifically test HOST_CODE validation
    const TEST_HOST_CODE: &str = "test-secure-code-12345";

    /// Helper to run a test with a clean environment, setting only the specified vars.
    /// This ensures tests don't interfere with each other.
    fn with_clean_env<F>(vars: &[(&str, &str)], f: F)
    where
        F: FnOnce(),
    {
        let _lock = ENV_LOCK.lock().unwrap();

        // Store original values of ALL config vars
        let originals: Vec<_> = ALL_CONFIG_VARS
            .iter()
            .map(|k| (*k, env::var(k).ok()))
            .collect();

        // Clear ALL config vars
        for k in ALL_CONFIG_VARS {
            env::remove_var(k);
        }

        // Set only the specified test values
        for (k, v) in vars {
            env::set_var(k, v);
        }

        // Run test
        f();

        // Restore ALL original values
        for (k, original) in originals {
            match original {
                Some(v) => env::set_var(k, v),
                None => env::remove_var(k),
            }
        }
    }

    /// Helper to set environment variables for testing with a valid HOST_CODE.
    /// This wraps with_clean_env and automatically includes a valid HOST_CODE
    /// unless one is explicitly provided in vars.
    fn with_env_vars<F>(vars: &[(&str, &str)], f: F)
    where
        F: FnOnce(),
    {
        // Check if HOST_CODE is explicitly provided
        let has_host_code = vars.iter().any(|(k, _)| *k == "HOST_CODE");

        if has_host_code {
            with_clean_env(vars, f);
        } else {
            let mut all_vars = vec![("HOST_CODE", TEST_HOST_CODE)];
            all_vars.extend(vars.iter().cloned());
            with_clean_env(&all_vars, f);
        }
    }

    /// Helper to run a test with a valid HOST_CODE plus any additional vars.
    /// Use this for tests that don't specifically test HOST_CODE validation.
    fn with_valid_host_code<F>(extra_vars: &[(&str, &str)], f: F)
    where
        F: FnOnce(),
    {
        let mut vars = vec![("HOST_CODE", TEST_HOST_CODE)];
        vars.extend(extra_vars.iter().cloned());
        with_clean_env(&vars, f);
    }

    #[test]
    fn test_default_config() {
        with_valid_host_code(&[], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert_eq!(config.server.port, 5000);
            assert!(!config.server.ssl_enabled);
            assert_eq!(config.database.path, "puzzles.db");
            assert!(!config.email.enabled);
        });
    }

    #[test]
    fn test_custom_port() {
        with_valid_host_code(&[("PORT", "8080")], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert_eq!(config.server.port, 8080);
        });
    }

    #[test]
    fn test_invalid_port_not_a_number() {
        with_env_vars(&[("PORT", "not_a_number")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
            let err = result.unwrap_err();
            match err {
                ConfigError::InvalidPort { name, value, .. } => {
                    assert_eq!(name, "PORT");
                    assert_eq!(value, "not_a_number");
                }
                _ => panic!("Expected InvalidPort error"),
            }
        });
    }

    #[test]
    fn test_invalid_port_zero() {
        with_env_vars(&[("PORT", "0")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
            let err = result.unwrap_err();
            match err {
                ConfigError::InvalidPort { name, value, .. } => {
                    assert_eq!(name, "PORT");
                    assert_eq!(value, "0");
                }
                _ => panic!("Expected InvalidPort error"),
            }
        });
    }

    #[test]
    fn test_invalid_port_negative() {
        with_env_vars(&[("PORT", "-1")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_ssl_enabled_without_certs() {
        // SSL_ENABLED=true but no SSL_CERT or SSL_KEY
        with_env_vars(&[("SSL_ENABLED", "true")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
            let err = result.unwrap_err();
            match err {
                ConfigError::MissingRequired(msg) => {
                    assert!(msg.contains("SSL_CERT"));
                    assert!(msg.contains("SSL_KEY"));
                }
                _ => panic!("Expected MissingRequired error"),
            }
        });
    }

    #[test]
    fn test_ssl_enabled_with_certs() {
        with_env_vars(
            &[
                ("SSL_ENABLED", "true"),
                ("SSL_CERT", "/path/to/cert.pem"),
                ("SSL_KEY", "/path/to/key.pem"),
            ],
            || {
                let config = Config::from_env_no_dotenv().unwrap();
                assert!(config.server.ssl_enabled);
                assert_eq!(
                    config.server.ssl_cert,
                    Some("/path/to/cert.pem".to_string())
                );
                assert_eq!(config.server.ssl_key, Some("/path/to/key.pem".to_string()));
            },
        );
    }

    #[test]
    fn test_boolean_parsing() {
        // Test "true" - SSL_ENABLED=true should fail due to missing certs
        with_env_vars(&[("SSL_ENABLED", "true")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
        });

        // Test "1" - EMAIL_ENABLED=1 should fail due to missing SMTP credentials
        with_env_vars(&[("EMAIL_ENABLED", "1")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
        });

        // Test "yes" - EMAIL_ENABLED=yes should fail due to missing SMTP credentials
        with_env_vars(&[("EMAIL_ENABLED", "yes")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
        });

        // Test "false"
        with_env_vars(&[("EMAIL_ENABLED", "false")], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert!(!config.email.enabled);
        });

        // Test "0"
        with_env_vars(&[("EMAIL_ENABLED", "0")], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert!(!config.email.enabled);
        });
    }

    #[test]
    fn test_invalid_boolean() {
        with_env_vars(&[("SSL_ENABLED", "maybe")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
            let err = result.unwrap_err();
            match err {
                ConfigError::InvalidBoolean { name, value } => {
                    assert_eq!(name, "SSL_ENABLED");
                    assert_eq!(value, "maybe");
                }
                _ => panic!("Expected InvalidBoolean error"),
            }
        });
    }

    #[test]
    fn test_valid_url() {
        assert!(Config::validate_url("http://localhost:5000", "TEST").is_ok());
        assert!(Config::validate_url("https://example.com", "TEST").is_ok());
        assert!(Config::validate_url("https://example.com/path", "TEST").is_ok());
        assert!(Config::validate_url("http://192.168.1.1:8080", "TEST").is_ok());
    }

    #[test]
    fn test_invalid_url_empty() {
        let result = Config::validate_url("", "TEST");
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidUrl { reason, .. } => {
                assert!(reason.contains("empty"));
            }
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    #[test]
    fn test_invalid_url_no_scheme() {
        let result = Config::validate_url("example.com", "TEST");
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidUrl { reason, .. } => {
                assert!(reason.contains("http://") || reason.contains("https://"));
            }
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    #[test]
    fn test_invalid_url_no_host() {
        let result = Config::validate_url("http://", "TEST");
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidUrl { reason, .. } => {
                assert!(reason.contains("host"));
            }
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    #[test]
    fn test_invalid_base_url() {
        with_env_vars(&[("BASE_URL", "not-a-url")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_database_path_priority() {
        // DATABASE_URL takes priority over DB_PATH
        with_env_vars(
            &[
                ("DATABASE_URL", "/path/to/database.db"),
                ("DB_PATH", "/other/path.db"),
            ],
            || {
                let config = Config::from_env_no_dotenv().unwrap();
                assert_eq!(config.database.path, "/path/to/database.db");
            },
        );

        // Falls back to DB_PATH when DATABASE_URL is not set
        with_env_vars(&[("DB_PATH", "/other/path.db")], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert_eq!(config.database.path, "/other/path.db");
        });
    }

    #[test]
    fn test_email_enabled_requires_credentials() {
        // EMAIL_ENABLED=true without credentials should fail
        with_env_vars(&[("EMAIL_ENABLED", "true")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
        });

        // EMAIL_ENABLED=true with credentials should succeed
        with_env_vars(
            &[
                ("EMAIL_ENABLED", "true"),
                ("SMTP_USER", "user@example.com"),
                ("SMTP_PASS", "password"),
            ],
            || {
                let config = Config::from_env_no_dotenv().unwrap();
                assert!(config.email.enabled);
                assert_eq!(config.email.smtp_user, "user@example.com");
            },
        );
    }

    #[test]
    fn test_webauthn_defaults() {
        // With env (no WEBAUTHN vars set), should use defaults
        with_valid_host_code(&[], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert_eq!(config.webauthn.rp_id, "localhost");
            assert_eq!(config.webauthn.rp_origin, "http://localhost:5000");
        });
    }

    #[test]
    fn test_oauth_config_optional() {
        // With env (no OAuth vars), all should be None
        with_valid_host_code(&[], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert!(config.oauth.google_client_id.is_none());
            assert!(config.oauth.apple_client_id.is_none());
            assert!(config.oauth.apple_client_id_web.is_none());
            assert!(config.oauth.apple_redirect_uri.is_none());
        });
    }

    #[test]
    fn test_invalid_apple_redirect_uri() {
        with_env_vars(&[("APPLE_REDIRECT_URI", "not-a-url")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_redact_function() {
        assert_eq!(redact(""), "not set");
        assert_eq!(redact("abc"), "****");
        assert_eq!(redact("abcd"), "****");
        assert_eq!(redact("abcde"), "abcd****");
        assert_eq!(redact("secret_password"), "secr****");
    }

    #[test]
    fn test_config_display() {
        with_valid_host_code(&[], || {
            let config = Config::from_env_no_dotenv().unwrap();
            let display = format!("{}", config);
            assert!(display.contains("port: 5000"));
            assert!(display.contains("ssl: false"));
            assert!(display.contains("puzzles.db"));
            assert!(display.contains("email: false"));
        });
    }

    #[test]
    fn test_multiple_errors() {
        // Multiple errors: invalid port, missing SSL certs, invalid URL
        with_env_vars(
            &[
                ("PORT", "invalid"),
                ("SSL_ENABLED", "true"),
                ("BASE_URL", "not-a-url"),
            ],
            || {
                let result = Config::from_env_no_dotenv();
                assert!(result.is_err());
                match result.unwrap_err() {
                    ConfigError::Multiple(errors) => {
                        assert!(errors.len() >= 2);
                    }
                    _ => panic!("Expected Multiple error"),
                }
            },
        );
    }

    #[test]
    fn test_cors_default_permissive() {
        // Without CORS_ALLOWED_ORIGINS, should be permissive
        with_valid_host_code(&[], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert!(config.cors.permissive);
            assert!(config.cors.allowed_origins.is_empty());
        });
    }

    #[test]
    fn test_cors_explicit_wildcard() {
        // CORS_ALLOWED_ORIGINS="*" should be explicitly permissive
        with_env_vars(&[("CORS_ALLOWED_ORIGINS", "*")], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert!(config.cors.permissive);
            assert!(config.cors.allowed_origins.is_empty());
        });
    }

    #[test]
    fn test_cors_single_origin() {
        with_env_vars(&[("CORS_ALLOWED_ORIGINS", "https://example.com")], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert!(!config.cors.permissive);
            assert_eq!(config.cors.allowed_origins.len(), 1);
            assert_eq!(config.cors.allowed_origins[0], "https://example.com");
        });
    }

    #[test]
    fn test_cors_multiple_origins() {
        with_env_vars(
            &[(
                "CORS_ALLOWED_ORIGINS",
                "https://example.com, https://app.example.com, http://localhost:3000",
            )],
            || {
                let config = Config::from_env_no_dotenv().unwrap();
                assert!(!config.cors.permissive);
                assert_eq!(config.cors.allowed_origins.len(), 3);
                assert_eq!(config.cors.allowed_origins[0], "https://example.com");
                assert_eq!(config.cors.allowed_origins[1], "https://app.example.com");
                assert_eq!(config.cors.allowed_origins[2], "http://localhost:3000");
            },
        );
    }

    #[test]
    fn test_cors_empty_string_fallback() {
        // Empty CORS_ALLOWED_ORIGINS should fall back to permissive
        with_env_vars(&[("CORS_ALLOWED_ORIGINS", "")], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert!(config.cors.permissive);
        });
    }

    #[test]
    fn test_cors_whitespace_only_fallback() {
        // Whitespace-only CORS_ALLOWED_ORIGINS should fall back to permissive
        with_env_vars(&[("CORS_ALLOWED_ORIGINS", "   ")], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert!(config.cors.permissive);
        });
    }

    #[test]
    fn test_host_code_not_set_fails() {
        // Without HOST_CODE, startup should fail
        with_clean_env(&[], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
            let err = result.unwrap_err();
            match err {
                ConfigError::InsecureHostCode(msg) => {
                    assert!(msg.contains("required"));
                }
                ConfigError::Multiple(errors) => {
                    assert!(errors.iter().any(|e| matches!(e, ConfigError::InsecureHostCode(_))));
                }
                _ => panic!("Expected InsecureHostCode error, got {:?}", err),
            }
        });
    }

    #[test]
    fn test_host_code_custom_valid() {
        with_clean_env(&[("HOST_CODE", "my-secure-code-123")], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert!(!config.game.using_default_host_code);
            assert_eq!(config.game.host_code, "my-secure-code-123");
        });
    }

    #[test]
    fn test_host_code_holiday_rejected() {
        // "holiday" is explicitly rejected as insecure
        with_clean_env(&[("HOST_CODE", "holiday")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
            let err = result.unwrap_err();
            match err {
                ConfigError::InsecureHostCode(msg) => {
                    assert!(msg.contains("holiday"));
                }
                ConfigError::Multiple(errors) => {
                    assert!(errors.iter().any(|e| matches!(e, ConfigError::InsecureHostCode(_))));
                }
                _ => panic!("Expected InsecureHostCode error, got {:?}", err),
            }
        });
    }

    #[test]
    fn test_host_code_too_short_rejected() {
        // HOST_CODE must be at least 8 characters
        with_clean_env(&[("HOST_CODE", "short")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
            let err = result.unwrap_err();
            match err {
                ConfigError::InsecureHostCode(msg) => {
                    assert!(msg.contains("8 characters"));
                }
                ConfigError::Multiple(errors) => {
                    assert!(errors.iter().any(|e| matches!(e, ConfigError::InsecureHostCode(_))));
                }
                _ => panic!("Expected InsecureHostCode error, got {:?}", err),
            }
        });
    }

    #[test]
    fn test_host_code_exactly_8_chars_valid() {
        // Exactly 8 characters should be valid
        with_clean_env(&[("HOST_CODE", "12345678")], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert!(!config.game.using_default_host_code);
            assert_eq!(config.game.host_code, "12345678");
        });
    }

    #[test]
    fn test_host_code_empty_fails() {
        // Empty HOST_CODE should fail
        with_clean_env(&[("HOST_CODE", "")], || {
            let result = Config::from_env_no_dotenv();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_host_code_whitespace_trimmed() {
        with_clean_env(&[("HOST_CODE", "  my-secure-code  ")], || {
            let config = Config::from_env_no_dotenv().unwrap();
            assert!(!config.game.using_default_host_code);
            assert_eq!(config.game.host_code, "my-secure-code");
        });
    }
}
