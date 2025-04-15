use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use super::game::{Game, GameBoardSize, GameStatus, GameType};

// GameBattlesnake model for our application
#[derive(Debug, Serialize, Deserialize)]
pub struct GameBattlesnake {
    pub game_battlesnake_id: Uuid,
    pub game_id: Uuid,
    pub battlesnake_id: Uuid,
    pub placement: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// For adding a battlesnake to a game
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddBattlesnakeToGame {
    pub battlesnake_id: Uuid,
}

// For setting the result of a game for a battlesnake
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SetGameResult {
    pub placement: i32,
}

// Extended GameBattlesnake with battlesnake details
#[derive(Debug, Serialize, Deserialize)]
pub struct GameBattlesnakeWithDetails {
    pub game_battlesnake_id: Uuid,
    pub game_id: Uuid,
    pub battlesnake_id: Uuid,
    pub placement: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    // Battlesnake details
    pub name: String,
    pub url: String,
    pub user_id: Uuid,
}

// Database functions for game battlesnake management

// Get all battlesnakes in a game
pub async fn get_battlesnakes_by_game_id(
    pool: &PgPool,
    game_id: Uuid,
) -> cja::Result<Vec<GameBattlesnakeWithDetails>> {
    let game_battlesnakes = sqlx::query_as!(
        GameBattlesnakeWithDetails,
        r#"
        SELECT
            gb.game_battlesnake_id,
            gb.game_id,
            gb.battlesnake_id,
            gb.placement,
            gb.created_at,
            gb.updated_at,
            b.name,
            b.url,
            b.user_id
        FROM game_battlesnakes gb
        JOIN battlesnakes b ON gb.battlesnake_id = b.battlesnake_id
        WHERE gb.game_id = $1
        ORDER BY gb.placement NULLS LAST, gb.created_at ASC
        "#,
        game_id
    )
    .fetch_all(pool)
    .await
    .wrap_err("Failed to fetch battlesnakes for game from database")?;

    Ok(game_battlesnakes)
}

// Get all games for a battlesnake
pub async fn get_games_by_battlesnake_id(
    pool: &PgPool,
    battlesnake_id: Uuid,
) -> cja::Result<Vec<Game>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            g.game_id,
            g.board_size,
            g.game_type,
            g.status,
            g.created_at,
            g.updated_at
        FROM games g
        JOIN game_battlesnakes gb ON g.game_id = gb.game_id
        WHERE gb.battlesnake_id = $1
        ORDER BY g.created_at DESC
        "#,
        battlesnake_id
    )
    .fetch_all(pool)
    .await
    .wrap_err("Failed to fetch games for battlesnake from database")?;

    let games = rows
        .into_iter()
        .map(|row| {
            let board_size = GameBoardSize::from_str(&row.board_size)
                .wrap_err_with(|| format!("Invalid board size: {}", row.board_size))?;
            let game_type = GameType::from_str(&row.game_type)
                .wrap_err_with(|| format!("Invalid game type: {}", row.game_type))?;
            let status = GameStatus::from_str(&row.status)
                .wrap_err_with(|| format!("Invalid game status: {}", row.status))?;

            Ok(Game {
                game_id: row.game_id,
                board_size,
                game_type,
                status,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        })
        .collect::<cja::Result<Vec<_>>>()?;

    Ok(games)
}

// Add a battlesnake to a game
pub async fn add_battlesnake_to_game(
    pool: &PgPool,
    game_id: Uuid,
    data: AddBattlesnakeToGame,
) -> cja::Result<GameBattlesnake> {
    // Check if the game already has 4 battlesnakes
    let count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM game_battlesnakes
        WHERE game_id = $1
        "#,
        game_id
    )
    .fetch_one(pool)
    .await
    .wrap_err("Failed to count battlesnakes in game")?;

    if count.count.unwrap_or(0) >= 4 {
        return Err(cja::color_eyre::eyre::eyre!(
            "Game already has the maximum of 4 battlesnakes"
        ));
    }

    // Add the battlesnake to the game using the executor pattern
    super::game::add_battlesnake_to_game(pool, game_id, data.clone()).await?;

    // Fetch and return the newly created game_battlesnake
    let game_battlesnake = sqlx::query_as!(
        GameBattlesnake,
        r#"
        SELECT
            game_battlesnake_id,
            game_id,
            battlesnake_id,
            placement,
            created_at,
            updated_at
        FROM game_battlesnakes
        WHERE game_id = $1 AND battlesnake_id = $2
        "#,
        game_id,
        data.battlesnake_id
    )
    .fetch_one(pool)
    .await
    .wrap_err("Failed to fetch newly created game battlesnake")?;

    Ok(game_battlesnake)
}

// Remove a battlesnake from a game
pub async fn remove_battlesnake_from_game(
    pool: &PgPool,
    game_id: Uuid,
    battlesnake_id: Uuid,
) -> cja::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM game_battlesnakes
        WHERE game_id = $1 AND battlesnake_id = $2
        "#,
        game_id,
        battlesnake_id
    )
    .execute(pool)
    .await
    .wrap_err("Failed to remove battlesnake from game")?;

    Ok(())
}

// Set the result of a game for a battlesnake
pub async fn set_game_result(
    pool: &PgPool,
    game_id: Uuid,
    battlesnake_id: Uuid,
    data: SetGameResult,
) -> cja::Result<GameBattlesnake> {
    // Validate placement is between 1 and 4
    if data.placement < 1 || data.placement > 4 {
        return Err(cja::color_eyre::eyre::eyre!(
            "Placement must be between 1 and 4"
        ));
    }

    let game_battlesnake = sqlx::query_as!(
        GameBattlesnake,
        r#"
        UPDATE game_battlesnakes
        SET placement = $3
        WHERE game_id = $1 AND battlesnake_id = $2
        RETURNING
            game_battlesnake_id,
            game_id,
            battlesnake_id,
            placement,
            created_at,
            updated_at
        "#,
        game_id,
        battlesnake_id,
        data.placement
    )
    .fetch_one(pool)
    .await
    .wrap_err("Failed to set game result")?;

    Ok(game_battlesnake)
}

// Get a game with all its battlesnakes
pub async fn get_game_with_battlesnakes(
    pool: &PgPool,
    game_id: Uuid,
) -> cja::Result<(Game, Vec<GameBattlesnakeWithDetails>)> {
    // Get the game
    let game = super::game::get_game_by_id(pool, game_id)
        .await?
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("Game not found"))?;

    // Get the battlesnakes for the game
    let battlesnakes = get_battlesnakes_by_game_id(pool, game_id).await?;

    Ok((game, battlesnakes))
}
