use color_eyre::eyre::Context as _;
use serde::Deserialize;

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
