use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{HeaderValue, header::SET_COOKIE, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use color_eyre::eyre::eyre;
use reqwest::StatusCode;

use crate::{
    cookies::CookieJar,
    errors::{ServerError, ServerResult},
    state::AppState,
};

// Cookie name for flash messages
const FLASH_COOKIE_NAME: &str = "flash_message";

/// Extractor for flash messages
///
/// This extractor retrieves the flash message from the cookies
/// and provides methods to create a `Page` with the flash message.
/// It also clears the cookie after reading the flash message.
#[derive(Debug, Clone)]
pub struct Flash {
    pub message: Option<String>,
}

impl Flash {
    /// Get the flash message if it exists
    pub fn message(&self) -> &Option<String> {
        &self.message
    }

    pub fn add(cookie_jar: &CookieJar, message: String) {
        let mut cookie = tower_cookies::Cookie::new(FLASH_COOKIE_NAME, message);
        cookie.set_http_only(true);
        cookie.set_secure(true);
        cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
        cookie.set_max_age(time::Duration::seconds(60));
        cookie.set_path("/");

        cookie_jar.add(cookie);
    }
}

#[async_trait]
impl FromRequestParts<AppState> for Flash {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = CookieJar::from_request_parts(parts, app_state).await?;

        let flash_cookie = cookie_jar.get(FLASH_COOKIE_NAME);
        let message = flash_cookie.map(|cookie| cookie.value().to_string());

        cookie_jar.remove_by_name(FLASH_COOKIE_NAME);

        Ok(Self { message })
    }
}

#[derive(Debug)]
pub struct FlashRedirect {
    redirect: Redirect,
    flash_message: String,
}

impl IntoResponse for FlashRedirect {
    fn into_response(self) -> axum::response::Response {
        fn inner(s: FlashRedirect) -> ServerResult<axum::response::Response, StatusCode> {
            let mut redirect_response = s.redirect.into_response();

            // Set the flash cookie
            let encoded_flash = urlencoding::encode(&s.flash_message);
            let cookie = format!(
                "{}={}; Path=/; HttpOnly; SameSite=Lax",
                FLASH_COOKIE_NAME, encoded_flash
            );

            redirect_response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::from_str(&cookie).map_err(|_| {
                    ServerError(
                        eyre!("Failed to create cookie header"),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })?,
            );

            Ok(redirect_response)
        }

        inner(self).into_response()
    }
}
