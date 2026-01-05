use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse as _, Response},
};
use cja::server::cookies::{Cookie, CookieJar};
use color_eyre::eyre::eyre;
use uuid::Uuid;

use crate::{
    errors::ServerError,
    models::{
        session::{
            SESSION_COOKIE_NAME, SESSION_EXPIRATION_SECONDS, Session, create_session,
            get_session_with_user,
        },
        user::User,
    },
    state::AppState,
};

/// Current session and optional user
///
/// This struct contains the current session and optional user.
/// The session is always present, but the user may be None if not logged in.
pub struct CurrentSession {
    pub session: Session,
    pub user: Option<User>,
}
/// Extract the current session with optional user
impl FromRequestParts<AppState> for CurrentSession {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = match CookieJar::from_request_parts(parts, app_state).await {
            Ok(jar) => jar,
            Err(_) => {
                tracing::error!("Cookie jar extraction failed");
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        };

        // Try to get session_id from cookie
        let session_id = cookie_jar
            .get(SESSION_COOKIE_NAME)
            .and_then(|cookie| cookie.value().parse::<Uuid>().ok());

        // If no session_id in cookie, create a new session
        let session_id = match session_id {
            Some(id) => id,
            None => {
                // No session found, create a new one
                let new_session = match create_session(&app_state.db).await {
                    Ok(session) => session,
                    Err(_e) => {
                        tracing::error!("Session creation failed: {}", _e);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
                    }
                };

                // Set the session cookie
                let mut cookie =
                    Cookie::new(SESSION_COOKIE_NAME, new_session.session_id.to_string());
                cookie.set_http_only(true);
                cookie.set_secure(true);
                cookie.set_same_site(cja::server::cookies::SameSite::Lax);
                cookie.set_max_age(time::Duration::seconds(SESSION_EXPIRATION_SECONDS));
                cookie_jar.add(cookie);

                new_session.session_id
            }
        };

        // Get session and user from database
        let result = match get_session_with_user(&app_state.db, session_id).await {
            Ok(result) => result,
            Err(_e) => {
                tracing::error!("Session fetch failed: {}", _e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        };

        // If session doesn't exist, create a new one
        match result {
            Some((session, user)) => Ok(CurrentSession { session, user }),
            None => {
                // Session expired or doesn't exist, create a new one
                let new_session = match create_session(&app_state.db).await {
                    Ok(session) => session,
                    Err(_e) => {
                        tracing::error!("Session creation failed: {}", _e);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
                    }
                };

                // Set the session cookie
                let mut cookie =
                    Cookie::new(SESSION_COOKIE_NAME, new_session.session_id.to_string());
                cookie.set_http_only(true);
                cookie.set_secure(true);
                cookie.set_same_site(cja::server::cookies::SameSite::Lax);
                cookie.set_max_age(time::Duration::seconds(SESSION_EXPIRATION_SECONDS));
                cookie_jar.add(cookie);

                Ok(CurrentSession {
                    session: new_session,
                    user: None,
                })
            }
        }
    }
}

/// Extractor for requiring an authenticated user
///
/// This extractor will return a 401 Unauthorized error if the user is not logged in.
/// Use this extractor in route handlers that require authentication.
///
/// Example:
/// ```
/// async fn protected_route(
///    CurrentUser(user): CurrentUser,
/// ) -> impl IntoResponse {
///    // User is guaranteed to be logged in here
///    format!("Hello, {}!", user.github_login)
/// }
/// ```
pub struct CurrentUser(pub User);

/// Extractor for optionally getting the current user
///
/// Unlike CurrentUser, this won't return an error if the user is not logged in.
/// Instead, it will return Option<User> which will be None if not logged in.
pub struct OptionalUser(pub Option<User>);

impl FromRequestParts<AppState> for OptionalUser {
    type Rejection = axum::response::Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = CurrentSession::from_request_parts(parts, state).await?;

        Ok(OptionalUser(session.user))
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = axum::response::Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = CurrentSession::from_request_parts(parts, state).await?;

        // Check if user is logged in
        let user = session.user.ok_or_else(|| {
            ServerError(eyre!("Not authenticated"), StatusCode::UNAUTHORIZED).into_response()
        })?;

        Ok(CurrentUser(user))
    }
}

/// Extractor for getting both the authenticated user and their session
///
/// This extractor will return a 401 Unauthorized error if the user is not logged in.
/// Use this extractor in route handlers that require authentication and need the session.
///
/// Example:
/// ```
/// async fn protected_route(
///    CurrentUserWithSession { user, session }: CurrentUserWithSession,
/// ) -> impl IntoResponse {
///    // User is guaranteed to be logged in here and you have access to their session
///    format!("Hello, {}! Your session ID is {}", user.github_login, session.session_id)
/// }
/// ```
pub struct CurrentUserWithSession {
    pub user: User,
    pub session: Session,
}

impl FromRequestParts<AppState> for CurrentUserWithSession {
    type Rejection = axum::response::Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let current_session = CurrentSession::from_request_parts(parts, state).await?;

        // Check if user is logged in
        let user = current_session.user.ok_or_else(|| {
            ServerError(eyre!("Not authenticated"), StatusCode::UNAUTHORIZED).into_response()
        })?;

        Ok(CurrentUserWithSession {
            user,
            session: current_session.session,
        })
    }
}
