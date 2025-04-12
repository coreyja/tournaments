use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use color_eyre::eyre::{Context as _, eyre};
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};

use crate::{
    components::flash::WithFlash as _,
    errors::{ServerError, ServerResult},
    github::auth::{GitHubAuthParams, GitHubTokenResponse, GitHubUser},
    models::{
        session::{
            associate_user_with_session, clear_github_oauth_state, disassociate_user_from_session,
            set_github_oauth_state,
        },
        user::create_or_update_user,
    },
    state::AppState,
};

use super::auth::CurrentSession;

// Constants
const GITHUB_OAUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_API_URL: &str = "https://api.github.com";

// Route handler for initiating GitHub OAuth flow
pub async fn github_auth(
    State(state): State<AppState>,
    current_session: CurrentSession,
) -> ServerResult<Redirect, StatusCode> {
    // Generate a random state for CSRF protection
    let oauth_state = format!("{}", uuid::Uuid::new_v4());

    // Store the state in the session
    set_github_oauth_state(
        &state.db,
        current_session.session.session_id,
        oauth_state.clone(),
    )
    .await
    .wrap_err("Failed to store OAuth state in session")?;

    // Build OAuth URL using the AppState's github_oauth_config
    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&state={}&scope={}",
        GITHUB_OAUTH_URL,
        state.github_oauth_config.client_id,
        urlencoding::encode(&state.github_oauth_config.redirect_uri),
        oauth_state,
        "user:email"
    );

    Ok(Redirect::to(&auth_url))
}

// Route handler for GitHub OAuth callback
pub async fn github_auth_callback(
    State(state): State<AppState>,
    Query(params): Query<GitHubAuthParams>,
    current_session: CurrentSession,
) -> ServerResult<impl IntoResponse, StatusCode> {
    // Verify the state parameter to prevent CSRF attacks
    let session_oauth_state = current_session.session.github_oauth_state;

    let session_state = match session_oauth_state {
        Some(state) => state,
        None => {
            return Err(ServerError(
                eyre!("GitHub OAuth state not found in session"),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    if params.state != session_state {
        return Err(ServerError(
            eyre!("GitHub OAuth state mismatch"),
            StatusCode::BAD_REQUEST,
        ));
    }

    // Clear the state from the session since it's no longer needed
    clear_github_oauth_state(&state.db, current_session.session.session_id)
        .await
        .wrap_err("Failed to clear OAuth state from session")?;

    // Exchange code for access token
    let client = reqwest::Client::new();
    let token_response = client
        .post(GITHUB_TOKEN_URL)
        .json(&serde_json::json!({
            "client_id": state.github_oauth_config.client_id,
            "client_secret": state.github_oauth_config.client_secret,
            "code": params.code,
            "redirect_uri": state.github_oauth_config.redirect_uri,
        }))
        .header(ACCEPT, "application/json")
        .send()
        .await
        .wrap_err("Failed to send token request to GitHub")?
        .json::<GitHubTokenResponse>()
        .await
        .wrap_err("Failed to parse GitHub token response")?;

    // Get user data from GitHub
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token_response.access_token))
            .wrap_err("Failed to create Authorization header")?,
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github.v3+json"),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("tournaments-app"));

    let github_user = client
        .get(format!("{}/user", GITHUB_API_URL))
        .headers(headers.clone())
        .send()
        .await
        .wrap_err("Failed to send user request to GitHub")?
        .json::<GitHubUser>()
        .await
        .wrap_err("Failed to parse GitHub user response")?;

    // Create or update user in the database
    let user = create_or_update_user(&state.db, github_user, token_response)
        .await
        .wrap_err("Failed to create or update user")?;

    // Associate the user with the current session
    associate_user_with_session(&state.db, current_session.session.session_id, user.user_id)
        .await
        .wrap_err("Failed to associate user with session")?;

    // Redirect to home page with success message
    Ok(Redirect::to("/").with_flash("Successfully logged in with GitHub!"))
}

// Route handler for logging out
pub async fn logout(
    State(state): State<AppState>,
    current_session: CurrentSession,
) -> impl IntoResponse {
    // Disassociate user from the session (if logged in)
    if current_session.user.is_some() {
        let _ = disassociate_user_from_session(&state.db, current_session.session.session_id).await;
    }

    Redirect::to("/").with_flash("You have been logged out")
}
