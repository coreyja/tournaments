use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
    routing::get,
};
use color_eyre::eyre::Context as _;
use maud::html;
use uuid::Uuid;

use crate::{
    components::{github_auth::User, page::Page},
    cookies::CookieJar,
    errors::ServerResult,
    state::AppState,
};

// Include route modules
pub mod auth;
pub mod github_auth;

pub fn routes(app_state: AppState) -> axum::Router {
    axum::Router::new()
        // Public pages
        .route("/", get(root_page))
        // Profile page - requires authentication
        .route("/me", get(profile_page))
        // GitHub OAuth routes
        .route("/auth/github", get(github_auth::github_auth))
        .route(
            "/auth/github/callback",
            get(github_auth::github_auth_callback),
        )
        .route("/auth/logout", get(github_auth::logout))
        // Add trace layer for debugging
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(app_state)
}

/// Extractor for optionally getting the current user
///
/// Unlike CurrentUser, this won't return an error if the user is not logged in.
/// Instead, it will return Option<User> which will be None if not logged in.
pub struct OptionalUser(pub Option<User>);

#[axum::async_trait]
impl<S> FromRequestParts<S> for OptionalUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Get cookie jar - return None if cookie jar can't be extracted
        let cookie_jar_result = CookieJar::from_request_parts(parts, &app_state).await;
        let cookie_jar = match cookie_jar_result {
            Ok(jar) => jar,
            Err(_) => return Ok(OptionalUser(None)),
        };

        // Try to get user_id from cookie
        let user_id_option = cookie_jar
            .get(github_auth::USER_COOKIE_NAME)
            .and_then(|cookie| cookie.value().parse::<Uuid>().ok());

        // If no user_id in cookie, return None
        let user_id = match user_id_option {
            Some(id) => id,
            None => return Ok(OptionalUser(None)),
        };

        // Get user from database
        let user_result = crate::components::github_auth::get_user_by_id(&app_state.db, user_id)
            .await
            .wrap_err("Failed to fetch user from database");

        // Return None if there was any error getting the user or if user doesn't exist
        let user = match user_result {
            Ok(Some(user)) => user,
            Ok(None) | Err(_) => return Ok(OptionalUser(None)),
        };

        Ok(OptionalUser(Some(user)))
    }
}

async fn root_page(
    _: State<AppState>,
    OptionalUser(user): OptionalUser,
) -> ServerResult<impl IntoResponse, StatusCode> {
    Ok(Page::new(
        "Home".to_string(),
        Box::new(html! {
            div {
                @if let Some(user) = user {
                    div class="user-info" {
                        img src=(user.github_avatar_url.unwrap_or_default()) alt="Avatar" style="width: 50px; height: 50px; border-radius: 50%;" {}
                        p { "Welcome, " (user.github_login) "!" }
                        @if let Some(name) = user.github_name {
                            p { "Name: " (name) }
                        }
                        a href="/auth/logout" { "Logout" }
                        p { "View your " a href="/me" { "profile page" } }
                    }
                } @else {
                    div class="login" {
                        p { "You are not logged in." }
                        a href="/auth/github" { "Login with GitHub" }
                    }
                }
                div class="content" style="margin-top: 20px;" {
                    h1 { "Hello, world!" }
                    p { "Welcome to the Tournaments application!" }
                }
            }
        }),
    ))
}

/// Profile page that requires authentication
async fn profile_page(
    auth::CurrentUser(user): auth::CurrentUser,
) -> ServerResult<impl IntoResponse, StatusCode> {
    Ok(Page::new(
        "My Profile".to_string(),
        Box::new(html! {
            div {
                h1 { "My Profile" }

                div class="profile-card" style="border: 1px solid #ddd; border-radius: 8px; padding: 20px; margin: 20px 0; max-width: 600px;" {
                    div class="profile-header" style="display: flex; align-items: center; margin-bottom: 20px;" {
                        img src=(user.github_avatar_url.unwrap_or_default()) alt="Avatar" style="width: 100px; height: 100px; border-radius: 50%; margin-right: 20px;" {}

                        div {
                            h2 style="margin: 0 0 10px 0;" { (user.github_login) }
                            @if let Some(name) = user.github_name.as_ref() {
                                p style="margin: 0; color: #666;" { (name) }
                            }
                            @if let Some(email) = user.github_email.as_ref() {
                                p style="margin: 0; color: #666;" { (email) }
                            }
                        }
                    }

                    div class="profile-details" {
                        h3 { "Account Details" }
                        p { "GitHub ID: " (user.external_github_id) }
                        p { "Account created: " (user.created_at.format("%Y-%m-%d %H:%M:%S")) }
                        p { "Last updated: " (user.updated_at.format("%Y-%m-%d %H:%M:%S")) }
                    }
                }

                div class="nav" style="margin-top: 20px;" {
                    a href="/" { "Back to Home" }
                    span { " | " }
                    a href="/auth/logout" { "Logout" }
                }
            }
        }),
    ))
}
