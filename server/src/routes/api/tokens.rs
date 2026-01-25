use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::api_token::{self, ApiToken},
    routes::auth::ApiUser,
    state::AppState,
};

/// Request body for creating a new token
#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
}

/// Response for a newly created token (includes the secret)
#[derive(Debug, Serialize)]
pub struct CreateTokenResponse {
    pub id: Uuid,
    pub name: String,
    pub secret: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Response for listing tokens (no secrets)
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub id: Uuid,
    pub name: String,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ApiToken> for TokenResponse {
    fn from(token: ApiToken) -> Self {
        Self {
            id: token.id,
            name: token.name,
            last_used_at: token.last_used_at,
            created_at: token.created_at,
        }
    }
}

/// POST /api/v1/tokens - Create a new API token
pub async fn create_token(
    State(state): State<AppState>,
    ApiUser(user): ApiUser,
    Json(request): Json<CreateTokenRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let new_token = api_token::create_api_token(&state.db, user.user_id, &request.name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create API token: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((
        StatusCode::CREATED,
        Json(CreateTokenResponse {
            id: new_token.token.id,
            name: new_token.token.name,
            secret: new_token.secret,
            created_at: new_token.token.created_at,
        }),
    ))
}

/// GET /api/v1/tokens - List all active tokens for the current user
pub async fn list_tokens(
    State(state): State<AppState>,
    ApiUser(user): ApiUser,
) -> Result<impl IntoResponse, StatusCode> {
    let tokens = api_token::list_user_tokens(&state.db, user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list API tokens: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response: Vec<TokenResponse> = tokens.into_iter().map(TokenResponse::from).collect();
    Ok(Json(response))
}

/// DELETE /api/v1/tokens/:id - Revoke a token
pub async fn revoke_token(
    State(state): State<AppState>,
    ApiUser(user): ApiUser,
    Path(token_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let revoked = api_token::revoke_token(&state.db, token_id, user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to revoke API token: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if revoked {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
