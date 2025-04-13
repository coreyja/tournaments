use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool, Postgres};
use std::str::FromStr;
use uuid::Uuid;

use super::game_battlesnake::AddBattlesnakeToGame;

// Game board size enum
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum GameBoardSize {
    Small,  // 7x7
    Medium, // 11x11
    Large,  // 19x19
}

impl GameBoardSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameBoardSize::Small => "7x7",
            GameBoardSize::Medium => "11x11",
            GameBoardSize::Large => "19x19",
        }
    }
}

impl FromStr for GameBoardSize {
    type Err = color_eyre::eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "7x7" => Ok(GameBoardSize::Small),
            "11x11" => Ok(GameBoardSize::Medium),
            "19x19" => Ok(GameBoardSize::Large),
            _ => Err(color_eyre::eyre::eyre!("Invalid board size: {}", s)),
        }
    }
}

// Game type enum
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
    Standard,
    Royale,
    Constrictor,
    SnailMode,
}

impl GameType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameType::Standard => "Standard",
            GameType::Royale => "Royale",
            GameType::Constrictor => "Constrictor",
            GameType::SnailMode => "Snail Mode",
        }
    }
}

impl FromStr for GameType {
    type Err = color_eyre::eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Standard" => Ok(GameType::Standard),
            "Royale" => Ok(GameType::Royale),
            "Constrictor" => Ok(GameType::Constrictor),
            "Snail Mode" => Ok(GameType::SnailMode),
            _ => Err(color_eyre::eyre::eyre!("Invalid game type: {}", s)),
        }
    }
}

// Game model for our application
#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub game_id: Uuid,
    pub board_size: GameBoardSize,
    pub game_type: GameType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// For creating a new game
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateGame {
    pub board_size: GameBoardSize,
    pub game_type: GameType,
}

// Create a game with battlesnakes in a single transaction
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateGameWithSnakes {
    pub board_size: GameBoardSize,
    pub game_type: GameType,
    pub battlesnake_ids: Vec<Uuid>,
}

// Database functions for game management

// Get all games
pub async fn get_all_games(pool: &PgPool) -> cja::Result<Vec<Game>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            game_id,
            board_size,
            game_type,
            created_at,
            updated_at
        FROM games
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
    .wrap_err("Failed to fetch games from database")?;

    let games = rows
        .into_iter()
        .map(|row| {
            let board_size = GameBoardSize::from_str(&row.board_size)
                .wrap_err_with(|| format!("Invalid board size: {}", row.board_size))?;
            let game_type = GameType::from_str(&row.game_type)
                .wrap_err_with(|| format!("Invalid game type: {}", row.game_type))?;

            Ok(Game {
                game_id: row.game_id,
                board_size,
                game_type,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        })
        .collect::<cja::Result<Vec<_>>>()?;

    Ok(games)
}

// Get a single game by ID
pub async fn get_game_by_id(pool: &PgPool, game_id: Uuid) -> cja::Result<Option<Game>> {
    let row = sqlx::query!(
        r#"
        SELECT
            game_id,
            board_size,
            game_type,
            created_at,
            updated_at
        FROM games
        WHERE game_id = $1
        "#,
        game_id
    )
    .fetch_optional(pool)
    .await
    .wrap_err("Failed to fetch game from database")?;

    let game = match row {
        Some(row) => {
            let board_size = GameBoardSize::from_str(&row.board_size)
                .wrap_err_with(|| format!("Invalid board size: {}", row.board_size))?;
            let game_type = GameType::from_str(&row.game_type)
                .wrap_err_with(|| format!("Invalid game type: {}", row.game_type))?;

            Some(Game {
                game_id: row.game_id,
                board_size,
                game_type,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        }
        None => None,
    };

    Ok(game)
}

// Delete a game
pub async fn delete_game(pool: &PgPool, game_id: Uuid) -> cja::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM games
        WHERE game_id = $1
        "#,
        game_id
    )
    .execute(pool)
    .await
    .wrap_err("Failed to delete game from database")?;

    Ok(())
}

// Create a new game with all battlesnakes in a single transaction
pub async fn create_game_with_snakes(
    pool: &PgPool,
    data: CreateGameWithSnakes,
) -> cja::Result<Game> {
    // Validate number of battlesnakes
    if data.battlesnake_ids.is_empty() {
        return Err(cja::color_eyre::eyre::eyre!(
            "At least one battlesnake is required for a game"
        ));
    }

    if data.battlesnake_ids.len() > 4 {
        return Err(cja::color_eyre::eyre::eyre!(
            "A maximum of 4 battlesnakes are allowed in a game"
        ));
    }

    // Check for duplicate battlesnake IDs
    let mut unique_ids = data.battlesnake_ids.clone();
    unique_ids.sort();
    unique_ids.dedup();
    if unique_ids.len() != data.battlesnake_ids.len() {
        return Err(cja::color_eyre::eyre::eyre!(
            "Duplicate battlesnake IDs are not allowed"
        ));
    }

    // Start a transaction
    let mut tx = pool
        .begin()
        .await
        .wrap_err("Failed to start database transaction")?;

    // Create the game
    let board_size_str = data.board_size.as_str();
    let game_type_str = data.game_type.as_str();

    let row = sqlx::query!(
        r#"
        INSERT INTO games (
            board_size,
            game_type
        )
        VALUES ($1, $2)
        RETURNING
            game_id,
            board_size,
            game_type,
            created_at,
            updated_at
        "#,
        board_size_str,
        game_type_str
    )
    .fetch_one(&mut *tx) // Access the connection inside the transaction
    .await
    .wrap_err("Failed to create game in database")?;

    let game = Game {
        game_id: row.game_id,
        board_size: data.board_size,
        game_type: data.game_type,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    // Add each battlesnake to the game
    for battlesnake_id in data.battlesnake_ids {
        sqlx::query!(
            r#"
            INSERT INTO game_battlesnakes (
                game_id,
                battlesnake_id
            )
            VALUES ($1, $2)
            "#,
            game.game_id,
            battlesnake_id
        )
        .execute(&mut *tx) // Access the connection inside the transaction
        .await
        .wrap_err_with(|| format!("Failed to add battlesnake {} to game", battlesnake_id))?;
    }

    // Commit the transaction
    tx.commit()
        .await
        .wrap_err("Failed to commit database transaction")?;

    Ok(game)
}

// Generic function to create a game with any executor
pub async fn create_game<'e, E>(executor: E, data: CreateGame) -> cja::Result<Game>
where
    E: Executor<'e, Database = Postgres>,
{
    let board_size_str = data.board_size.as_str();
    let game_type_str = data.game_type.as_str();

    let row = sqlx::query!(
        r#"
        INSERT INTO games (
            board_size,
            game_type
        )
        VALUES ($1, $2)
        RETURNING
            game_id,
            board_size,
            game_type,
            created_at,
            updated_at
        "#,
        board_size_str,
        game_type_str
    )
    .fetch_one(executor)
    .await
    .wrap_err("Failed to create game in database")?;

    Ok(Game {
        game_id: row.game_id,
        board_size: data.board_size,
        game_type: data.game_type,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

// Generic function to add a battlesnake to a game with any executor
pub async fn add_battlesnake_to_game<'e, E>(
    executor: E,
    game_id: Uuid,
    data: AddBattlesnakeToGame,
) -> cja::Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query!(
        r#"
        INSERT INTO game_battlesnakes (
            game_id,
            battlesnake_id
        )
        VALUES ($1, $2)
        "#,
        game_id,
        data.battlesnake_id
    )
    .execute(executor)
    .await
    .wrap_err_with(|| format!("Failed to add battlesnake {} to game", data.battlesnake_id))?;

    Ok(())
}
