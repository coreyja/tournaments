use color_eyre::eyre::Context as _;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// API token stored in the database (hashed)
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ApiToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub name: String,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Result of creating a new token - includes the raw secret (only shown once)
pub struct NewApiToken {
    pub token: ApiToken,
    /// The raw secret token to give to the user. Only shown once!
    pub secret: String,
}

/// Generate a random 32-byte token and return it as a hex string (64 chars)
fn generate_token_secret() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Hash a token secret using SHA-256, returning the hex-encoded hash
fn hash_token(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hex::encode(hasher.finalize())
}

/// Create a new API token for a user
pub async fn create_api_token(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
) -> cja::Result<NewApiToken> {
    let secret = generate_token_secret();
    let token_hash = hash_token(&secret);

    let token: ApiToken = sqlx::query_as(
        r#"
        INSERT INTO api_tokens (user_id, token_hash, name)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, token_hash, name, last_used_at, created_at, revoked_at
        "#,
    )
    .bind(user_id)
    .bind(&token_hash)
    .bind(name)
    .fetch_one(pool)
    .await
    .wrap_err("Failed to create API token")?;

    Ok(NewApiToken { token, secret })
}

/// Get all active (non-revoked) tokens for a user
pub async fn list_user_tokens(pool: &PgPool, user_id: Uuid) -> cja::Result<Vec<ApiToken>> {
    let tokens: Vec<ApiToken> = sqlx::query_as(
        r#"
        SELECT id, user_id, token_hash, name, last_used_at, created_at, revoked_at
        FROM api_tokens
        WHERE user_id = $1 AND revoked_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .wrap_err("Failed to list API tokens")?;

    Ok(tokens)
}

/// Validate a raw token secret and return the associated user_id if valid (not revoked)
///
/// This function hashes the token internally to prevent accidentally passing unhashed tokens.
pub async fn validate_token(pool: &PgPool, token_secret: &str) -> cja::Result<Option<Uuid>> {
    let token_hash = hash_token(token_secret);

    let result: Option<Uuid> = sqlx::query_scalar(
        r#"
        UPDATE api_tokens
        SET last_used_at = NOW()
        WHERE token_hash = $1 AND revoked_at IS NULL
        RETURNING user_id
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .wrap_err("Failed to validate API token")?;

    Ok(result)
}

/// Revoke a token by ID (must belong to the user)
pub async fn revoke_token(pool: &PgPool, token_id: Uuid, user_id: Uuid) -> cja::Result<bool> {
    let result = sqlx::query(
        r#"
        UPDATE api_tokens
        SET revoked_at = NOW()
        WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL
        "#,
    )
    .bind(token_id)
    .bind(user_id)
    .execute(pool)
    .await
    .wrap_err("Failed to revoke API token")?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_secret_length() {
        let secret = generate_token_secret();
        assert_eq!(secret.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_generate_token_secret_is_random() {
        let secret1 = generate_token_secret();
        let secret2 = generate_token_secret();
        assert_ne!(secret1, secret2);
    }

    #[test]
    fn test_hash_token_consistency() {
        let secret = "test_secret_value";
        let hash1 = hash_token(secret);
        let hash2 = hash_token(secret);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_length() {
        let hash = hash_token("test");
        assert_eq!(hash.len(), 64); // SHA-256 = 64 hex chars
    }
}
