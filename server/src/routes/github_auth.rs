use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use color_eyre::eyre::{Context as _, eyre};
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use time::Duration;
use tower_cookies::Cookie;
use tower_cookies::cookie::SameSite;

use crate::{
    components::github_auth::{
        GitHubAuthParams, GitHubTokenResponse, GitHubUser, create_or_update_user,
    },
    cookies::CookieJar,
    errors::{ServerError, ServerResult},
    state::AppState,
};

// Constants
const GITHUB_OAUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_API_URL: &str = "https://api.github.com";
pub const USER_COOKIE_NAME: &str = "user_id";
const GITHUB_OAUTH_STATE_COOKIE: &str = "github_oauth_state";
const COOKIE_MAX_AGE_SECONDS: i64 = 60 * 60 * 24 * 30; // 30 days

// Route handler for initiating GitHub OAuth flow
pub async fn github_auth(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
) -> ServerResult<Redirect, StatusCode> {
    // Generate a random state for CSRF protection
    let oauth_state = format!("{}", uuid::Uuid::new_v4());

    // Store the state in a cookie for verification
    let mut cookie = Cookie::new(GITHUB_OAUTH_STATE_COOKIE, oauth_state.clone());
    cookie.set_same_site(SameSite::Strict);
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie.set_max_age(Duration::minutes(10));
    cookie_jar.add(cookie);

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
    cookie_jar: CookieJar,
) -> ServerResult<Redirect, StatusCode> {
    // Verify the state parameter to prevent CSRF attacks
    let state_cookie = cookie_jar.get(GITHUB_OAUTH_STATE_COOKIE);
    let cookie_state = match state_cookie {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return Err(ServerError(
                eyre!("GitHub OAuth state cookie not found"),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    if params.state != cookie_state {
        return Err(ServerError(
            eyre!("GitHub OAuth state mismatch"),
            StatusCode::BAD_REQUEST,
        ));
    }

    // Remove the state cookie since it's no longer needed
    cookie_jar.remove(Cookie::build(GITHUB_OAUTH_STATE_COOKIE).build());

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

    // Set user_id cookie
    let mut cookie = Cookie::new(USER_COOKIE_NAME, user.user_id.to_string());
    cookie.set_max_age(Duration::seconds(COOKIE_MAX_AGE_SECONDS));
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Lax);
    cookie_jar.add(cookie);

    // Redirect to home page after successful login
    Ok(Redirect::to("/"))
}

// Route handler for logging out
pub async fn logout(cookie_jar: CookieJar) -> impl IntoResponse {
    // Remove the user_id cookie
    cookie_jar.remove(Cookie::build(USER_COOKIE_NAME).build());

    Redirect::to("/")
}
