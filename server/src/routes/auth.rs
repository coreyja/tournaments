use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use color_eyre::eyre::{Context as _, eyre};
use uuid::Uuid;

use crate::{
    components::github_auth::{User, get_user_by_id},
    cookies::CookieJar,
    errors::ServerError,
    routes::github_auth::USER_COOKIE_NAME,
    state::AppState,
};

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

#[async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ServerError<StatusCode>;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract cookie jar from the request
        let cookie_jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                ServerError(
                    eyre!("Failed to extract cookies"),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            })?;

        // Check if user is logged in
        let user_id = cookie_jar
            .get(USER_COOKIE_NAME)
            .and_then(|cookie| cookie.value().parse::<Uuid>().ok())
            .ok_or_else(|| ServerError(eyre!("Not authenticated"), StatusCode::UNAUTHORIZED))?;

        // Get user from database
        let user = get_user_by_id(&state.db, user_id)
            .await
            .wrap_err("Failed to get current user")?
            .ok_or_else(|| ServerError(eyre!("User not found"), StatusCode::UNAUTHORIZED))?;

        Ok(CurrentUser(user))
    }
}
