use axum::{extract::FromRequestParts, http::request::Parts, response::Response};

use crate::{
    models::session::{
        self, FLASH_TYPE_ERROR, FLASH_TYPE_INFO, FLASH_TYPE_PRIMARY, FLASH_TYPE_SUCCESS,
        FLASH_TYPE_WARNING,
    },
    routes::auth::CurrentSession,
    state::AppState,
};

/// Flasher extractor for setting flash messages
pub struct Flasher {
    session_id: uuid::Uuid,
    db_pool: sqlx::PgPool,
}

impl Flasher {
    pub fn new(session_id: uuid::Uuid, db_pool: sqlx::PgPool) -> Self {
        Self {
            session_id,
            db_pool,
        }
    }
}

impl FromRequestParts<AppState> for Flasher {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let current_session = CurrentSession::from_request_parts(parts, state).await?;
        let session_id = current_session.session.session_id;

        Ok(Self {
            session_id,
            db_pool: state.db.clone(),
        })
    }
}

impl Flasher {
    /// Add a flash message with default type (Primary)
    pub async fn add_flash(&self, message: impl Into<String>) -> cja::Result<()> {
        session::set_flash_message(
            &self.db_pool,
            self.session_id,
            message.into(),
            FLASH_TYPE_PRIMARY,
        )
        .await?;
        Ok(())
    }

    /// Add a success flash message
    pub async fn success(&self, message: impl Into<String>) -> cja::Result<()> {
        session::set_flash_message(
            &self.db_pool,
            self.session_id,
            message.into(),
            FLASH_TYPE_SUCCESS,
        )
        .await?;
        Ok(())
    }

    /// Add an error flash message
    pub async fn error(&self, message: impl Into<String>) -> cja::Result<()> {
        session::set_flash_message(
            &self.db_pool,
            self.session_id,
            message.into(),
            FLASH_TYPE_ERROR,
        )
        .await?;
        Ok(())
    }

    /// Add an info flash message
    pub async fn info(&self, message: impl Into<String>) -> cja::Result<()> {
        session::set_flash_message(
            &self.db_pool,
            self.session_id,
            message.into(),
            FLASH_TYPE_INFO,
        )
        .await?;
        Ok(())
    }

    /// Add a warning flash message
    pub async fn warning(&self, message: impl Into<String>) -> cja::Result<()> {
        session::set_flash_message(
            &self.db_pool,
            self.session_id,
            message.into(),
            FLASH_TYPE_WARNING,
        )
        .await?;
        Ok(())
    }
}
