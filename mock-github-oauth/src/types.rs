use serde::{Deserialize, Serialize};

/// Configuration for a mock user that will be returned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockUserConfig {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
}

impl Default for MockUserConfig {
    fn default() -> Self {
        Self {
            id: 12345,
            login: "mock_user".to_string(),
            name: Some("Mock User".to_string()),
            email: Some("mock@example.com".to_string()),
            avatar_url: "https://example.com/avatar.png".to_string(),
        }
    }
}

/// Query params for /login/oauth/authorize
#[derive(Debug, Deserialize)]
pub struct AuthorizeParams {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    #[allow(dead_code)]
    pub scope: Option<String>,
    // Custom params for testing - these control what mock user is returned
    pub mock_user_id: Option<i64>,
    pub mock_user_login: Option<String>,
    pub mock_user_name: Option<String>,
    pub mock_user_email: Option<String>,
}

/// POST body for /login/oauth/access_token
#[derive(Debug, Deserialize)]
pub struct TokenParams {
    #[allow(dead_code)]
    pub client_id: String,
    #[allow(dead_code)]
    pub client_secret: String,
    pub code: String,
    #[allow(dead_code)]
    pub redirect_uri: String,
}

/// Response for access token
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

/// GitHub user API response
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
}
