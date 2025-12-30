use axum::{
    extract::FromRequestParts,
    http::{HeaderValue, header::SET_COOKIE, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use color_eyre::eyre::{Context as _, eyre};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{ServerError, ServerResult},
    models::session::{
        self, FLASH_TYPE_ERROR, FLASH_TYPE_INFO, FLASH_TYPE_PRIMARY, FLASH_TYPE_SUCCESS,
        FLASH_TYPE_WARNING,
    },
    routes::auth::CurrentSession,
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

/// Flash message extractor for web requests
///
/// This extractor retrieves the flash message from the session
/// and clears it after reading to ensure it's only shown once.
#[derive(Debug, Clone)]
pub struct Flash {
    pub message: Option<String>,
    pub flash_type: Option<String>,
}

impl Flash {
    /// Get the flash message if it exists
    pub fn message(&self) -> &Option<String> {
        &self.message
    }

    /// Get the CSS class based on flash type
    pub fn class(&self) -> &'static str {
        match self.flash_type.as_deref() {
            Some(FLASH_TYPE_SUCCESS) => "alert alert-success",
            Some(FLASH_TYPE_ERROR) => "alert alert-danger",
            Some(FLASH_TYPE_INFO) => "alert alert-info",
            Some(FLASH_TYPE_WARNING) => "alert alert-warning",
            Some(FLASH_TYPE_PRIMARY) | Some(_) => "alert alert-primary",
            None => "",
        }
    }
}

impl FromRequestParts<AppState> for Flash {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session_result = CurrentSession::from_request_parts(parts, app_state).await;

        match session_result {
            Ok(CurrentSession { session, .. }) => {
                // Extract the flash data
                let flash = Self {
                    message: session.flash_message.clone(),
                    flash_type: session.flash_type.clone(),
                };

                // If there was a flash message, clear it so it's only shown once
                if flash.message.is_some() {
                    match session::clear_flash_message(&app_state.db, session.session_id)
                        .await
                        .wrap_err("Failed to clear flash message")
                    {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(
                                ServerError(err, StatusCode::INTERNAL_SERVER_ERROR).into_response()
                            );
                        }
                    }
                }

                Ok(flash)
            }
            _ => {
                // Session not found or error, return empty flash
                Ok(Self {
                    message: None,
                    flash_type: None,
                })
            }
        }
    }
}

/// Helper for setting a flash message in the session
///
/// This function will set a flash message in the session
/// and should be called before redirecting the user.
pub async fn set_flash_message(
    db_pool: &sqlx::PgPool,
    session_id: uuid::Uuid,
    message: String,
    flash_type: &str,
) -> cja::Result<()> {
    session::set_flash_message(db_pool, session_id, message, flash_type)
        .await
        .map(|_| ())
}

/// Set a success flash message
pub async fn set_success_flash(
    db_pool: &sqlx::PgPool,
    session_id: uuid::Uuid,
    message: String,
) -> cja::Result<()> {
    set_flash_message(db_pool, session_id, message, FLASH_TYPE_SUCCESS).await
}

/// Set an error flash message
pub async fn set_error_flash(
    db_pool: &sqlx::PgPool,
    session_id: uuid::Uuid,
    message: String,
) -> cja::Result<()> {
    set_flash_message(db_pool, session_id, message, FLASH_TYPE_ERROR).await
}

/// Set an info flash message
pub async fn set_info_flash(
    db_pool: &sqlx::PgPool,
    session_id: uuid::Uuid,
    message: String,
) -> cja::Result<()> {
    set_flash_message(db_pool, session_id, message, FLASH_TYPE_INFO).await
}

/// Set a warning flash message
pub async fn set_warning_flash(
    db_pool: &sqlx::PgPool,
    session_id: uuid::Uuid,
    message: String,
) -> cja::Result<()> {
    set_flash_message(db_pool, session_id, message, FLASH_TYPE_WARNING).await
}

/// Set a primary flash message
pub async fn set_primary_flash(
    db_pool: &sqlx::PgPool,
    session_id: uuid::Uuid,
    message: String,
) -> cja::Result<()> {
    set_flash_message(db_pool, session_id, message, FLASH_TYPE_PRIMARY).await
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
