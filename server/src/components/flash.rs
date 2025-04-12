use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{HeaderValue, header::SET_COOKIE, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use color_eyre::eyre::eyre;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    cookies::CookieJar,
    errors::{ServerError, ServerResult},
    state::AppState,
};

// Cookie names for flash messages
const FLASH_COOKIE_NAME: &str = "flash_message";
const FLASH_TYPE_COOKIE_NAME: &str = "flash_type";

/// Flash message types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FlashType {
    Success,
    Error,
    Info,
    Warning,
    Primary,
}

impl FlashType {
    /// Convert FlashType to CSS class
    pub fn to_class(self) -> &'static str {
        match self {
            FlashType::Success => "alert alert-success",
            FlashType::Error => "alert alert-danger",
            FlashType::Info => "alert alert-info",
            FlashType::Warning => "alert alert-warning",
            FlashType::Primary => "alert alert-primary",
        }
    }

    /// Parse a string to FlashType
    fn from_str(s: &str) -> Self {
        match s {
            "success" => FlashType::Success,
            "error" => FlashType::Error,
            "info" => FlashType::Info,
            "warning" => FlashType::Warning,
            _ => FlashType::Primary,
        }
    }

    /// Convert FlashType to string
    fn to_str(self) -> &'static str {
        match self {
            FlashType::Success => "success",
            FlashType::Error => "error",
            FlashType::Info => "info",
            FlashType::Warning => "warning",
            FlashType::Primary => "primary",
        }
    }
}

/// Extractor for flash messages
///
/// This extractor retrieves the flash message from the cookies
/// and provides methods to create a `Page` with the flash message.
/// It also clears the cookie after reading the flash message.
#[derive(Debug, Clone)]
pub struct Flash {
    pub message: Option<String>,
    pub flash_type: Option<FlashType>,
}

// Type alias for Flash to be used in the routes
pub type FlashData = Flash;

impl Flash {
    /// Get the flash message if it exists
    pub fn message(&self) -> &Option<String> {
        &self.message
    }

    /// Get the CSS class based on flash type
    pub fn class(&self) -> &'static str {
        match self.flash_type {
            Some(flash_type) => flash_type.to_class(),
            None => "",
        }
    }

    /// Add a flash message with type to the cookies
    pub fn add(cookie_jar: &CookieJar, message: String, flash_type: FlashType) {
        // Set message cookie
        let mut message_cookie = tower_cookies::Cookie::new(FLASH_COOKIE_NAME, message);
        message_cookie.set_http_only(true);
        message_cookie.set_secure(true);
        message_cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
        message_cookie.set_max_age(time::Duration::seconds(60));
        message_cookie.set_path("/");
        cookie_jar.add(message_cookie);

        // Set type cookie
        let mut type_cookie =
            tower_cookies::Cookie::new(FLASH_TYPE_COOKIE_NAME, flash_type.to_str());
        type_cookie.set_http_only(true);
        type_cookie.set_secure(true);
        type_cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
        type_cookie.set_max_age(time::Duration::seconds(60));
        type_cookie.set_path("/");
        cookie_jar.add(type_cookie);
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

        // Get message from cookie
        let flash_cookie = cookie_jar.get(FLASH_COOKIE_NAME);
        let message = flash_cookie.map(|cookie| cookie.value().to_string());

        // Get type from cookie
        let flash_type_cookie = cookie_jar.get(FLASH_TYPE_COOKIE_NAME);
        let flash_type = flash_type_cookie.map(|cookie| FlashType::from_str(cookie.value()));

        // Remove both cookies
        cookie_jar.remove_by_name(FLASH_COOKIE_NAME);
        cookie_jar.remove_by_name(FLASH_TYPE_COOKIE_NAME);

        Ok(Self {
            message,
            flash_type,
        })
    }
}

#[derive(Debug)]
pub struct FlashRedirect {
    redirect: Redirect,
    flash_message: String,
    flash_type: FlashType,
}

impl FlashRedirect {
    pub fn new(redirect: Redirect, message: String, flash_type: FlashType) -> Self {
        Self {
            redirect,
            flash_message: message,
            flash_type,
        }
    }
}

impl IntoResponse for FlashRedirect {
    fn into_response(self) -> axum::response::Response {
        fn inner(s: FlashRedirect) -> ServerResult<axum::response::Response, StatusCode> {
            let mut redirect_response = s.redirect.into_response();

            // Set the flash message cookie
            let encoded_flash = urlencoding::encode(&s.flash_message);
            let message_cookie = format!(
                "{}={}; Path=/; HttpOnly; SameSite=Lax",
                FLASH_COOKIE_NAME, encoded_flash
            );

            // Set the flash type cookie
            let type_cookie = format!(
                "{}={}; Path=/; HttpOnly; SameSite=Lax",
                FLASH_TYPE_COOKIE_NAME,
                s.flash_type.to_str()
            );

            // Add message cookie
            redirect_response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::from_str(&message_cookie).map_err(|_| {
                    ServerError(
                        eyre!("Failed to create cookie header"),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })?,
            );

            // Add type cookie
            redirect_response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::from_str(&type_cookie).map_err(|_| {
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
