use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::MockUserConfig;

/// Maps authorization codes to the mock user that should be returned
#[derive(Clone, Default)]
pub struct MockOAuthState {
    /// Maps auth code -> MockUserConfig
    codes: Arc<RwLock<HashMap<String, MockUserConfig>>>,
    /// Maps access token -> MockUserConfig
    tokens: Arc<RwLock<HashMap<String, MockUserConfig>>>,
}

impl MockOAuthState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store an auth code with its associated user config
    pub async fn store_code(&self, code: String, user: MockUserConfig) {
        self.codes.write().await.insert(code, user);
    }

    /// Exchange a code for a token, returning the user config
    pub async fn exchange_code(&self, code: &str) -> Option<(String, MockUserConfig)> {
        let user = self.codes.write().await.remove(code)?;
        let token = format!("mock_token_{}", uuid::Uuid::new_v4());
        self.tokens.write().await.insert(token.clone(), user.clone());
        Some((token, user))
    }

    /// Get user config for a token
    pub async fn get_user(&self, token: &str) -> Option<MockUserConfig> {
        self.tokens.read().await.get(token).cloned()
    }
}
