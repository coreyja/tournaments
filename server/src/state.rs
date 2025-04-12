use color_eyre::eyre::{Context as _, eyre};
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{components::flash::FlashType, github::auth::GitHubOAuthConfig};

/// App-wide Flasher utility for creating flash messages
#[derive(Clone)]
pub struct Flasher;

impl Flasher {
    /// Create a new success flash message
    pub fn success(&self, message: String) -> (String, FlashType) {
        (message, FlashType::Success)
    }

    /// Create a new error flash message
    pub fn error(&self, message: String) -> (String, FlashType) {
        (message, FlashType::Error)
    }

    /// Create a new info flash message
    pub fn info(&self, message: String) -> (String, FlashType) {
        (message, FlashType::Info)
    }

    /// Create a new warning flash message
    pub fn warning(&self, message: String) -> (String, FlashType) {
        (message, FlashType::Warning)
    }

    /// Create a new primary flash message
    pub fn primary(&self, message: String) -> (String, FlashType) {
        (message, FlashType::Primary)
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::Pool<sqlx::Postgres>,
    pub cookie_key: cja::server::cookies::CookieKey,
    pub github_oauth_config: GitHubOAuthConfig,
    pub flasher: Flasher,
}

impl AppState {
    pub async fn from_env() -> cja::Result<Self> {
        #[tracing::instrument(err)]
        pub async fn setup_db_pool() -> cja::Result<PgPool> {
            const MIGRATION_LOCK_ID: i64 = 0xDB_DB_DB_DB_DB_DB_DB;

            let database_url =
                std::env::var("DATABASE_URL").wrap_err("DATABASE_URL must be set")?;
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await?;

            sqlx::query!("SELECT pg_advisory_lock($1)", MIGRATION_LOCK_ID)
                .execute(&pool)
                .await?;

            sqlx::migrate!("../migrations").run(&pool).await?;

            let unlock_result = sqlx::query!("SELECT pg_advisory_unlock($1)", MIGRATION_LOCK_ID)
                .fetch_one(&pool)
                .await?
                .pg_advisory_unlock;

            match unlock_result {
                Some(b) => {
                    if b {
                        tracing::info!("Migration lock unlocked");
                    } else {
                        tracing::info!("Failed to unlock migration lock");
                    }
                }
                None => return Err(eyre!("Failed to unlock migration lock")),
            }

            Ok(pool)
        }

        let pool = setup_db_pool().await?;

        let cookie_key = cja::server::cookies::CookieKey::from_env_or_generate()?;

        // Initialize GitHub OAuth config - now required
        let github_oauth_config =
            GitHubOAuthConfig::from_env().wrap_err("GitHub OAuth configuration is required")?;

        Ok(Self {
            db: pool,
            cookie_key,
            github_oauth_config,
            flasher: Flasher,
        })
    }
}

impl cja::app_state::AppState for AppState {
    fn version(&self) -> &str {
        env!("VERGEN_GIT_SHA")
    }

    fn db(&self) -> &sqlx::PgPool {
        &self.db
    }

    fn cookie_key(&self) -> &cja::server::cookies::CookieKey {
        &self.cookie_key
    }
}
