use axum::{extract::FromRequestParts, http::request::Parts, response::Response};

use crate::{
    components::flash::{Flash, FlashType},
    cookies::CookieJar,
    state::AppState,
};

pub struct Flasher {
    cookie_jar: CookieJar,
}

impl Flasher {
    pub fn new(cookie_jar: CookieJar) -> Self {
        Self { cookie_jar }
    }
}

#[async_trait::async_trait]
impl FromRequestParts<AppState> for Flasher {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = CookieJar::from_request_parts(parts, state).await?;
        Ok(Self { cookie_jar })
    }
}

impl Flasher {
    /// Add a flash message with default type (Primary)
    pub fn add_flash(&self, message: impl Into<String>) {
        Flash::add(&self.cookie_jar, message.into(), FlashType::Primary);
    }

    /// Add a success flash message
    pub fn success(&self, message: impl Into<String>) {
        Flash::add(&self.cookie_jar, message.into(), FlashType::Success);
    }

    /// Add an error flash message
    pub fn error(&self, message: impl Into<String>) {
        Flash::add(&self.cookie_jar, message.into(), FlashType::Error);
    }

    /// Add an info flash message
    pub fn info(&self, message: impl Into<String>) {
        Flash::add(&self.cookie_jar, message.into(), FlashType::Info);
    }

    /// Add a warning flash message
    pub fn warning(&self, message: impl Into<String>) {
        Flash::add(&self.cookie_jar, message.into(), FlashType::Warning);
    }
}
