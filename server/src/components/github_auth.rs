use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// Config structure for GitHub OAuth
#[derive(Clone, Debug)]
pub struct GitHubOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl GitHubOAuthConfig {
    pub fn from_env() -> cja::Result<Self> {
        let client_id =
            std::env::var("GITHUB_CLIENT_ID").wrap_err("GITHUB_CLIENT_ID must be set")?;
        let client_secret =
            std::env::var("GITHUB_CLIENT_SECRET").wrap_err("GITHUB_CLIENT_SECRET must be set")?;
        let redirect_uri =
            std::env::var("GITHUB_REDIRECT_URI").wrap_err("GITHUB_REDIRECT_URI must be set")?;

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
        })
    }
}

// GitHub OAuth parameters
#[derive(Debug, Deserialize)]
pub struct GitHubAuthParams {
    pub code: String,
    pub state: String,
}

// GitHub API response for token exchange
#[derive(Debug, Deserialize)]
pub struct GitHubTokenResponse {
    pub access_token: String,
    // These fields are required for proper deserialization of GitHub's API response
    // but are not used in our code
    #[allow(dead_code)]
    pub token_type: String,
    #[allow(dead_code)]
    pub scope: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<i64>,
}

// GitHub API response for user data
#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    pub avatar_url: String,
}

// User model for our application
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: Uuid,
    pub external_github_id: i64,
    pub github_login: String,
    pub github_avatar_url: Option<String>,
    pub github_name: Option<String>,
    pub github_email: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// Database functions for user management
pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> cja::Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
            user_id,
            external_github_id,
            github_login,
            github_avatar_url,
            github_name,
            github_email,
            created_at,
            updated_at
        FROM users
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
    .wrap_err("Failed to fetch user from database")?;

    Ok(user)
}

pub async fn create_or_update_user(
    pool: &PgPool,
    github_user: GitHubUser,
    token: GitHubTokenResponse,
) -> cja::Result<User> {
    let token_expires_at = token
        .expires_in
        .map(|expires_in| chrono::Utc::now() + chrono::Duration::seconds(expires_in));

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (
            external_github_id,
            github_login,
            github_avatar_url,
            github_name,
            github_email,
            github_access_token,
            github_refresh_token,
            github_token_expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (external_github_id) DO UPDATE SET
            github_login = $2,
            github_avatar_url = $3,
            github_name = $4,
            github_email = $5,
            github_access_token = $6,
            github_refresh_token = $7,
            github_token_expires_at = $8,
            updated_at = NOW()
        RETURNING
            user_id,
            external_github_id,
            github_login,
            github_avatar_url,
            github_name,
            github_email,
            created_at,
            updated_at
        "#,
        github_user.id,
        github_user.login,
        github_user.avatar_url,
        github_user.name,
        github_user.email,
        token.access_token,
        token.refresh_token,
        token_expires_at
    )
    .fetch_one(pool)
    .await
    .wrap_err("Failed to create or update user in database")?;

    Ok(user)
}
