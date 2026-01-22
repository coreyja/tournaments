use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// A turn in a game with its frame data
#[derive(Debug, Serialize, Deserialize)]
pub struct Turn {
    pub turn_id: Uuid,
    pub game_id: Uuid,
    pub turn_number: i32,
    pub frame_data: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Get all turns for a game, ordered by turn number
pub async fn get_turns_by_game_id(pool: &PgPool, game_id: Uuid) -> cja::Result<Vec<Turn>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            turn_id,
            game_id,
            turn_number,
            frame_data,
            created_at
        FROM turns
        WHERE game_id = $1
        ORDER BY turn_number ASC
        "#,
        game_id
    )
    .fetch_all(pool)
    .await
    .wrap_err("Failed to fetch turns from database")?;

    let turns = rows
        .into_iter()
        .map(|row| Turn {
            turn_id: row.turn_id,
            game_id: row.game_id,
            turn_number: row.turn_number,
            frame_data: row.frame_data,
            created_at: row.created_at,
        })
        .collect();

    Ok(turns)
}

/// Get turns for a game starting from a specific turn number
/// Used for reconnection catch-up
pub async fn get_turns_from(
    pool: &PgPool,
    game_id: Uuid,
    from_turn: i32,
) -> cja::Result<Vec<Turn>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            turn_id,
            game_id,
            turn_number,
            frame_data,
            created_at
        FROM turns
        WHERE game_id = $1 AND turn_number >= $2
        ORDER BY turn_number ASC
        "#,
        game_id,
        from_turn
    )
    .fetch_all(pool)
    .await
    .wrap_err("Failed to fetch turns from database")?;

    let turns = rows
        .into_iter()
        .map(|row| Turn {
            turn_id: row.turn_id,
            game_id: row.game_id,
            turn_number: row.turn_number,
            frame_data: row.frame_data,
            created_at: row.created_at,
        })
        .collect();

    Ok(turns)
}

/// Create a new turn for a game
pub async fn create_turn(
    pool: &PgPool,
    game_id: Uuid,
    turn_number: i32,
    frame_data: Option<serde_json::Value>,
) -> cja::Result<Turn> {
    let row = sqlx::query!(
        r#"
        INSERT INTO turns (game_id, turn_number, frame_data)
        VALUES ($1, $2, $3)
        RETURNING turn_id, game_id, turn_number, frame_data, created_at
        "#,
        game_id,
        turn_number,
        frame_data
    )
    .fetch_one(pool)
    .await
    .wrap_err("Failed to create turn")?;

    Ok(Turn {
        turn_id: row.turn_id,
        game_id: row.game_id,
        turn_number: row.turn_number,
        frame_data: row.frame_data,
        created_at: row.created_at,
    })
}

/// Update turn frame data (used after computing game state)
pub async fn update_turn_frame_data(
    pool: &PgPool,
    turn_id: Uuid,
    frame_data: serde_json::Value,
) -> cja::Result<()> {
    sqlx::query!(
        r#"
        UPDATE turns
        SET frame_data = $2
        WHERE turn_id = $1
        "#,
        turn_id,
        frame_data
    )
    .execute(pool)
    .await
    .wrap_err("Failed to update turn frame data")?;

    Ok(())
}

/// A snake's move for a specific turn
#[derive(Debug, Serialize, Deserialize)]
pub struct SnakeTurn {
    pub snake_turn_id: Uuid,
    pub turn_id: Uuid,
    pub game_battlesnake_id: Uuid,
    pub direction: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Create a snake turn record
pub async fn create_snake_turn(
    pool: &PgPool,
    turn_id: Uuid,
    game_battlesnake_id: Uuid,
    direction: &str,
) -> cja::Result<SnakeTurn> {
    let row = sqlx::query!(
        r#"
        INSERT INTO snake_turns (turn_id, game_battlesnake_id, direction)
        VALUES ($1, $2, $3)
        RETURNING snake_turn_id, turn_id, game_battlesnake_id, direction, created_at
        "#,
        turn_id,
        game_battlesnake_id,
        direction
    )
    .fetch_one(pool)
    .await
    .wrap_err("Failed to create snake turn")?;

    Ok(SnakeTurn {
        snake_turn_id: row.snake_turn_id,
        turn_id: row.turn_id,
        game_battlesnake_id: row.game_battlesnake_id,
        direction: row.direction,
        created_at: row.created_at,
    })
}

/// Get all snake turns for a specific turn
pub async fn get_snake_turns_by_turn_id(
    pool: &PgPool,
    turn_id: Uuid,
) -> cja::Result<Vec<SnakeTurn>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            snake_turn_id,
            turn_id,
            game_battlesnake_id,
            direction,
            created_at
        FROM snake_turns
        WHERE turn_id = $1
        "#,
        turn_id
    )
    .fetch_all(pool)
    .await
    .wrap_err("Failed to fetch snake turns")?;

    let turns = rows
        .into_iter()
        .map(|row| SnakeTurn {
            snake_turn_id: row.snake_turn_id,
            turn_id: row.turn_id,
            game_battlesnake_id: row.game_battlesnake_id,
            direction: row.direction,
            created_at: row.created_at,
        })
        .collect();

    Ok(turns)
}
