use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::{HeaderValue, request::Parts},
    response::{IntoResponse, Redirect},
};
use color_eyre::eyre::{ContextCompat, eyre};
use reqwest::{StatusCode, header::LOCATION};
use serde::Deserialize;

use crate::errors::{ServerError, ServerResult};

#[derive(Debug, Deserialize)]
pub struct FlashQuery {
    #[serde(default)]
    flash: Option<String>,
}

/// Extractor for flash messages
///
/// This extractor retrieves the flash message from the query parameters
/// and provides methods to create a `Page` with the flash message.
#[derive(Debug, Clone)]
pub struct Flash {
    pub message: Option<String>,
}

impl Flash {
    /// Get the flash message if it exists
    pub fn message(&self) -> &Option<String> {
        &self.message
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Flash
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(flash_query) = Query::<FlashQuery>::from_request_parts(parts, state)
            .await
            .unwrap_or(Query(FlashQuery { flash: None }));

        Ok(Self {
            message: flash_query.flash,
        })
    }
}
// Add these new structures at the bottom of the file
#[derive(Debug)]
pub struct FlashRedirect {
    redirect: Redirect,
    flash_message: String,
}

impl IntoResponse for FlashRedirect {
    fn into_response(self) -> axum::response::Response {
        fn inner(s: FlashRedirect) -> ServerResult<axum::response::Response, StatusCode> {
            let mut redirect_response = s.redirect.into_response();

            let headers = redirect_response.headers_mut();
            let location = headers
                .get(LOCATION)
                .wrap_err("No location header in redirect")?
                .to_str()
                .map_err(|_| {
                    ServerError(
                        eyre!("Failed to convert location header to string"),
                        StatusCode::BAD_REQUEST,
                    )
                })?;
            let new_location = if location.contains('?') {
                format!("{}&flash={}", location, s.flash_message)
            } else {
                format!("{}?flash={}", location, s.flash_message)
            };
            headers.insert(LOCATION, HeaderValue::from_str(&new_location).unwrap());

            Ok(redirect_response)
        }

        inner(self).into_response()
    }
}

pub trait WithFlash {
    fn with_flash(self, message: impl Into<String>) -> FlashRedirect;
}

impl WithFlash for Redirect {
    fn with_flash(self, message: impl Into<String>) -> FlashRedirect {
        FlashRedirect {
            redirect: self,
            flash_message: message.into(),
        }
    }
}
