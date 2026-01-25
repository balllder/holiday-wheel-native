use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Environment Variables
    /// - `DATABASE_URL`: PostgreSQL connection string (required)
    /// - `PORT`: Server port (default: 3000)
    ///
    /// # Errors
    /// Returns an error if required environment variables are missing
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| anyhow::anyhow!("PORT must be a valid u16"))?;

        Ok(Self {
            database_url,
            port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_requires_database_url() {
        env::remove_var("DATABASE_URL");
        let result = Config::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_default_port() {
        env::set_var("DATABASE_URL", "postgres://localhost/test");
        env::remove_var("PORT");

        let config = Config::from_env().unwrap();
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_custom_port() {
        env::set_var("DATABASE_URL", "postgres://localhost/test");
        env::set_var("PORT", "8080");

        let config = Config::from_env().unwrap();
        assert_eq!(config.port, 8080);
    }
}
