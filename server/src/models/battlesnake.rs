use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// Battlesnake model for our application
#[derive(Debug, Serialize, Deserialize)]
pub struct Battlesnake {
    pub battlesnake_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub url: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// For creating a new battlesnake
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBattlesnake {
    pub name: String,
    pub url: String,
}

// For updating an existing battlesnake
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBattlesnake {
    pub name: String,
    pub url: String,
}

// Database functions for battlesnake management

// Get all battlesnakes for a user
pub async fn get_battlesnakes_by_user_id(
    pool: &PgPool,
    user_id: Uuid,
) -> cja::Result<Vec<Battlesnake>> {
    let battlesnakes = sqlx::query_as!(
        Battlesnake,
        r#"
        SELECT
            battlesnake_id,
            user_id,
            name,
            url,
            created_at,
            updated_at
        FROM battlesnakes
        WHERE user_id = $1
        ORDER BY name ASC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .wrap_err("Failed to fetch battlesnakes from database")?;

    Ok(battlesnakes)
}

// Get a single battlesnake by ID
pub async fn get_battlesnake_by_id(
    pool: &PgPool,
    battlesnake_id: Uuid,
) -> cja::Result<Option<Battlesnake>> {
    let battlesnake = sqlx::query_as!(
        Battlesnake,
        r#"
        SELECT
            battlesnake_id,
            user_id,
            name,
            url,
            created_at,
            updated_at
        FROM battlesnakes
        WHERE battlesnake_id = $1
        "#,
        battlesnake_id
    )
    .fetch_optional(pool)
    .await
    .wrap_err("Failed to fetch battlesnake from database")?;

    Ok(battlesnake)
}

// Create a new battlesnake
pub async fn create_battlesnake(
    pool: &PgPool,
    user_id: Uuid,
    data: CreateBattlesnake,
) -> cja::Result<Battlesnake> {
    let battlesnake = sqlx::query_as!(
        Battlesnake,
        r#"
        INSERT INTO battlesnakes (
            user_id,
            name,
            url
        )
        VALUES ($1, $2, $3)
        RETURNING
            battlesnake_id,
            user_id,
            name,
            url,
            created_at,
            updated_at
        "#,
        user_id,
        data.name,
        data.url
    )
    .fetch_one(pool)
    .await
    .wrap_err("Failed to create battlesnake in database")?;

    Ok(battlesnake)
}

// Update an existing battlesnake
pub async fn update_battlesnake(
    pool: &PgPool,
    battlesnake_id: Uuid,
    user_id: Uuid,
    data: UpdateBattlesnake,
) -> cja::Result<Battlesnake> {
    let battlesnake = sqlx::query_as!(
        Battlesnake,
        r#"
        UPDATE battlesnakes
        SET
            name = $3,
            url = $4
        WHERE
            battlesnake_id = $1
            AND user_id = $2
        RETURNING
            battlesnake_id,
            user_id,
            name,
            url,
            created_at,
            updated_at
        "#,
        battlesnake_id,
        user_id,
        data.name,
        data.url
    )
    .fetch_one(pool)
    .await
    .wrap_err("Failed to update battlesnake in database")?;

    Ok(battlesnake)
}

// Delete a battlesnake
pub async fn delete_battlesnake(
    pool: &PgPool,
    battlesnake_id: Uuid,
    user_id: Uuid,
) -> cja::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM battlesnakes
        WHERE
            battlesnake_id = $1
            AND user_id = $2
        "#,
        battlesnake_id,
        user_id
    )
    .execute(pool)
    .await
    .wrap_err("Failed to delete battlesnake from database")?;

    Ok(())
}

// Check if a battlesnake belongs to a user
pub async fn belongs_to_user(
    pool: &PgPool,
    battlesnake_id: Uuid,
    user_id: Uuid,
) -> cja::Result<bool> {
    let result = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM battlesnakes
            WHERE
                battlesnake_id = $1
                AND user_id = $2
        ) as "exists!"
        "#,
        battlesnake_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .wrap_err("Failed to check if battlesnake belongs to user")?;

    Ok(result.exists)
}
