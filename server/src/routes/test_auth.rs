//! Test-only authentication routes
//!
//! These routes are only available when the `E2E_TEST_MODE` environment variable is set.
//! They allow tests to authenticate as a specific user without going through OAuth.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use cja::server::cookies::{Cookie, CookieJar};
use color_eyre::eyre::Context as _;
use uuid::Uuid;

use crate::{
    errors::{ServerResult, WithStatus},
    models::session::{SESSION_COOKIE_NAME, SESSION_EXPIRATION_SECONDS},
    state::AppState,
};

/// Check if test mode is enabled
pub fn is_test_mode() -> bool {
    std::env::var("E2E_TEST_MODE").is_ok()
}

/// Test login endpoint - sets the session cookie for a given session_id
///
/// POST /test/auth/login/{session_id}
///
/// This allows e2e tests to authenticate by:
/// 1. Creating a user and session in the database
/// 2. Calling this endpoint with the session_id
/// 3. The server sets the encrypted cookie correctly
pub async fn test_login(
    State(state): State<AppState>,
    cookie_jar: CookieJar<AppState>,
    Path(session_id): Path<Uuid>,
) -> ServerResult<impl IntoResponse, StatusCode> {
    if !is_test_mode() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    // Verify the session exists in the database
    let session = crate::models::session::get_active_session_by_id(&state.db, session_id)
        .await
        .wrap_err("Failed to fetch session")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

    if session.is_none() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    // Set the session cookie with proper encryption
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, session_id.to_string());
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie.set_same_site(cja::server::cookies::SameSite::Lax);
    cookie.set_max_age(time::Duration::seconds(SESSION_EXPIRATION_SECONDS));
    cookie.set_path("/"); // Important: set path to root so cookie is sent for all routes
    cookie_jar.add(cookie);

    // Redirect to home page
    Ok(Redirect::to("/").into_response())
}
