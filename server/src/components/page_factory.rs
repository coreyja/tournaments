use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use maud::Render;

use crate::components::{flash::Flash, page::Page};

/// PageFactory extractor
///
/// This extractor is responsible for creating Page instances with all necessary components
/// like flash messages. It extracts FlashMessage and uses it when creating pages.
#[derive(Debug, Clone)]
pub struct PageFactory {
    flash: Flash,
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
}

#[async_trait]
impl<S> FromRequestParts<S> for PageFactory
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let flash = Flash::from_request_parts(parts, state).await?;

        Ok(Self { flash })
    }
}
