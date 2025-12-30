use axum::{extract::FromRequestParts, http::request::Parts, response::Response};
use maud::Render;

use crate::{
    components::{flash::Flash, page::Page},
    state::AppState,
};

/// PageFactory extractor
///
/// This extractor is responsible for creating Page instances with all necessary components
/// like flash messages. It extracts FlashMessage and uses it when creating pages.
pub struct PageFactory {
    /// The flash message extracted from the session (already cleared from DB)
    pub flash: Flash,
}

impl PageFactory {
    /// Create a new Page with the extracted flash message (if any)
    pub fn create_page(self, title: String, content: Box<dyn Render>) -> Page {
        Page {
            title,
            content,
            flash: self.flash.message,
        }
    }

    /// Create a new Page with an explicit flash message
    /// This is useful when you want to use the FlashData extractor but also
    /// add it to the page later
    pub fn create_page_with_flash(
        self,
        title: String,
        content: Box<dyn Render>,
        flash: Flash,
    ) -> Page {
        Page {
            title,
            content,
            flash: flash.message,
        }
    }
}

impl FromRequestParts<AppState> for PageFactory {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let flash = Flash::from_request_parts(parts, state).await?;
        Ok(Self { flash })
    }
}
