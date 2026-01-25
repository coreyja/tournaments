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

    /// Returns the (width, height) dimensions of the board
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            GameBoardSize::Small => (7, 7),
            GameBoardSize::Medium => (11, 11),
            GameBoardSize::Large => (19, 19),
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

// Game status enum
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Waiting,
    Running,
    Finished,
}

impl GameStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameStatus::Waiting => "waiting",
            GameStatus::Running => "running",
            GameStatus::Finished => "finished",
        }
    }
}

impl FromStr for GameStatus {
    type Err = color_eyre::eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "waiting" => Ok(GameStatus::Waiting),
            "running" => Ok(GameStatus::Running),
            "finished" => Ok(GameStatus::Finished),
            _ => Err(color_eyre::eyre::eyre!("Invalid game status: {}", s)),
        }
    }
}

// Game model for our application
#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub game_id: Uuid,
    pub board_size: GameBoardSize,
    pub game_type: GameType,
    pub status: GameStatus,
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

// Struct to hold the game with winner query result
#[derive(Debug)]
struct GameWithWinnerRow {
    game_id: Uuid,
    board_size: String,
    game_type: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    winner_name: Option<String>,
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
            status,
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

// Get a single game by ID
pub async fn get_game_by_id(pool: &PgPool, game_id: Uuid) -> cja::Result<Option<Game>> {
    let row = sqlx::query!(
        r#"
        SELECT
            game_id,
            board_size,
            game_type,
            status,
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
            let status = GameStatus::from_str(&row.status)
                .wrap_err_with(|| format!("Invalid game status: {}", row.status))?;

            Some(Game {
                game_id: row.game_id,
                board_size,
                game_type,
                status,
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
    let status_str = GameStatus::Waiting.as_str();

    let row = sqlx::query!(
        r#"
        INSERT INTO games (
            board_size,
            game_type,
            status
        )
        VALUES ($1, $2, $3)
        RETURNING
            game_id,
            board_size,
            game_type,
            status,
            created_at,
            updated_at
        "#,
        board_size_str,
        game_type_str,
        status_str
    )
    .fetch_one(&mut *tx) // Access the connection inside the transaction
    .await
    .wrap_err("Failed to create game in database")?;

    let game = Game {
        game_id: row.game_id,
        board_size: data.board_size,
        game_type: data.game_type,
        status: GameStatus::from_str(&row.status)
            .wrap_err_with(|| format!("Invalid game status: {}", row.status))?,
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
    let status_str = GameStatus::Waiting.as_str();

    let row = sqlx::query!(
        r#"
        INSERT INTO games (
            board_size,
            game_type,
            status
        )
        VALUES ($1, $2, $3)
        RETURNING
            game_id,
            board_size,
            game_type,
            status,
            created_at,
            updated_at
        "#,
        board_size_str,
        game_type_str,
        status_str
    )
    .fetch_one(executor)
    .await
    .wrap_err("Failed to create game in database")?;

    Ok(Game {
        game_id: row.game_id,
        board_size: data.board_size,
        game_type: data.game_type,
        status: GameStatus::from_str(&row.status)
            .wrap_err_with(|| format!("Invalid game status: {}", row.status))?,
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

// Update the status of a game
pub async fn update_game_status(
    pool: &PgPool,
    game_id: Uuid,
    status: GameStatus,
) -> cja::Result<Game> {
    let status_str = status.as_str();

    let row = sqlx::query!(
        r#"
        UPDATE games
        SET status = $2
        WHERE game_id = $1
        RETURNING
            game_id,
            board_size,
            game_type,
            status,
            created_at,
            updated_at
        "#,
        game_id,
        status_str
    )
    .fetch_one(pool)
    .await
    .wrap_err_with(|| format!("Failed to update status for game {}", game_id))?;

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
}

/// Run a game with turn-by-turn DB persistence and WebSocket notifications
///
/// This function calls the actual snake APIs to get moves, with timeout handling.
/// On timeout, snakes continue in the same direction as their last move.
pub async fn run_game(app_state: &crate::state::AppState, game_id: Uuid) -> cja::Result<()> {
    use crate::engine::MAX_TURNS;
    use crate::engine::frame::{DeathInfo, game_to_frame, game_to_frame_with_latency};
    use crate::snake_client::{
        request_end_parallel, request_moves_parallel, request_start_parallel,
    };
    use battlesnake_game_types::types::Move;
    use std::collections::HashMap;

    let pool = &app_state.db;
    let game_channels = &app_state.game_channels;
    let http_client = &app_state.http_client;

    tracing::info!(game_id = %game_id, "Starting run_game");

    // Get the game details
    let game = get_game_by_id(pool, game_id)
        .await?
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("Game not found"))?;

    // Update status to running
    update_game_status(pool, game_id, GameStatus::Running).await?;

    // Get all the battlesnakes in the game with their URLs
    let battlesnakes = crate::models::game_battlesnake::get_battlesnakes_by_game_id(pool, game_id)
        .await
        .wrap_err("Failed to get battlesnakes for game")?;

    if battlesnakes.is_empty() {
        return Err(cja::color_eyre::eyre::eyre!("No battlesnakes in the game"));
    }

    // Build snake_id -> url mapping
    let snake_urls: Vec<(String, String)> = battlesnakes
        .iter()
        .map(|bs| (bs.battlesnake_id.to_string(), bs.url.clone()))
        .collect();

    // Build snake_id -> game_battlesnake_id mapping for DB storage
    let snake_db_ids: HashMap<String, Uuid> = battlesnakes
        .iter()
        .map(|bs| (bs.battlesnake_id.to_string(), bs.game_battlesnake_id))
        .collect();

    // Create the initial game state
    let mut engine_game =
        crate::engine::create_initial_game(game_id, game.board_size, game.game_type, &battlesnakes);

    // Get timeout from game settings (default 500ms)
    let timeout = std::time::Duration::from_millis(engine_game.game.timeout as u64);

    // Call /start for all snakes in parallel (fire and forget)
    tracing::info!(game_id = %game_id, "Calling /start for all snakes");
    request_start_parallel(http_client, &engine_game, &snake_urls, timeout).await;

    let mut death_info: Vec<DeathInfo> = Vec::new();
    let mut elimination_order: Vec<String> = Vec::new();
    let mut last_moves: HashMap<String, Move> = HashMap::new();

    // Helper to check if game is over
    let is_game_over = |g: &battlesnake_game_types::wire_representation::Game| {
        g.board.snakes.iter().filter(|s| s.health > 0).count() <= 1
    };

    // Store turn 0 (initial state)
    let frame_0 = game_to_frame(&engine_game, &death_info);
    let frame_0_json =
        serde_json::to_value(&frame_0).wrap_err("Failed to serialize initial frame")?;

    tracing::info!(game_id = %game_id, "Storing turn 0");
    crate::models::turn::create_turn_and_notify(
        pool,
        game_channels,
        game_id,
        0,
        Some(frame_0_json),
    )
    .await?;
    tracing::info!(game_id = %game_id, "Turn 0 stored successfully");

    // Run the game turn by turn
    while !is_game_over(&engine_game) && engine_game.turn < MAX_TURNS {
        // Request moves from all alive snakes in parallel
        let move_results =
            request_moves_parallel(http_client, &engine_game, &snake_urls, timeout, &last_moves)
                .await;

        // Convert to move vector for engine
        let moves: Vec<(String, Move)> = move_results
            .iter()
            .map(|r| (r.snake_id.clone(), r.direction))
            .collect();

        // Store last moves for timeout fallback on next turn
        for result in &move_results {
            last_moves.insert(result.snake_id.clone(), result.direction);
        }

        // Apply the moves using the engine
        engine_game = crate::engine::apply_turn(engine_game, &moves);
        engine_game.turn += 1;

        // Track newly eliminated snakes
        for snake in &engine_game.board.snakes {
            if snake.health <= 0 && !elimination_order.contains(&snake.id) {
                elimination_order.push(snake.id.clone());
                death_info.push(DeathInfo {
                    snake_id: snake.id.clone(),
                    turn: engine_game.turn,
                    cause: "eliminated".to_string(),
                    eliminated_by: String::new(),
                });
            }
        }

        // Store the turn frame with latency info and notify subscribers
        let frame = game_to_frame_with_latency(&engine_game, &death_info, &move_results);
        let frame_json = serde_json::to_value(&frame)
            .wrap_err_with(|| format!("Failed to serialize frame {}", engine_game.turn))?;

        tracing::debug!(game_id = %game_id, turn = engine_game.turn, "Storing turn");
        let turn = crate::models::turn::create_turn_and_notify(
            pool,
            game_channels,
            game_id,
            engine_game.turn,
            Some(frame_json),
        )
        .await?;

        // Store individual snake moves with latency
        for result in &move_results {
            if let Some(game_battlesnake_id) = snake_db_ids.get(&result.snake_id) {
                crate::models::turn::create_snake_turn(
                    pool,
                    turn.turn_id,
                    *game_battlesnake_id,
                    &result.direction.to_string(),
                    result.latency_ms,
                    result.timed_out,
                )
                .await?;
            }
        }
    }

    // Call /end for all snakes in parallel (fire and forget)
    tracing::info!(game_id = %game_id, "Calling /end for all snakes");
    request_end_parallel(http_client, &engine_game, &snake_urls, timeout).await;

    tracing::info!(
        game_id = %game_id,
        final_turn = engine_game.turn,
        "Game completed with persistence"
    );

    // Build placements: last eliminated = winner (placement 1)
    // Snakes still alive at the end go first
    let mut placements: Vec<String> = engine_game
        .board
        .snakes
        .iter()
        .filter(|s| s.health > 0)
        .map(|s| s.id.clone())
        .collect();

    // Then add eliminated snakes in reverse order (last eliminated = better placement)
    elimination_order.reverse();
    placements.extend(elimination_order);

    // Assign placements to database
    for (i, snake_id) in placements.iter().enumerate() {
        let placement = (i + 1) as i32;

        let battlesnake_id: Uuid = snake_id
            .parse()
            .wrap_err_with(|| format!("Invalid battlesnake ID: {}", snake_id))?;

        crate::models::game_battlesnake::set_game_result(
            pool,
            game_id,
            battlesnake_id,
            crate::models::game_battlesnake::SetGameResult { placement },
        )
        .await
        .wrap_err_with(|| {
            format!(
                "Failed to set game result for battlesnake {}",
                battlesnake_id
            )
        })?;
    }

    // Update status to finished
    update_game_status(pool, game_id, GameStatus::Finished).await?;

    // Clean up game channel (will be removed when no subscribers)
    game_channels.cleanup(game_id).await;

    Ok(())
}

// Get all games with their winners (if available)
pub async fn get_all_games_with_winners(pool: &PgPool) -> cja::Result<Vec<(Game, Option<String>)>> {
    let rows = sqlx::query_as!(
        GameWithWinnerRow,
        r#"
        SELECT
            g.game_id,
            g.board_size,
            g.game_type,
            g.status,
            g.created_at,
            g.updated_at,
            b.name as "winner_name?"
        FROM games g
        LEFT JOIN game_battlesnakes gb ON g.game_id = gb.game_id AND gb.placement = 1
        LEFT JOIN battlesnakes b ON gb.battlesnake_id = b.battlesnake_id
        ORDER BY g.created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
    .wrap_err("Failed to fetch games with winners from database")?;

    let games_with_winners = rows
        .into_iter()
        .map(|row| {
            let board_size = GameBoardSize::from_str(&row.board_size)
                .wrap_err_with(|| format!("Invalid board size: {}", row.board_size))?;
            let game_type = GameType::from_str(&row.game_type)
                .wrap_err_with(|| format!("Invalid game type: {}", row.game_type))?;
            let status = GameStatus::from_str(&row.status)
                .wrap_err_with(|| format!("Invalid game status: {}", row.status))?;

            let game = Game {
                game_id: row.game_id,
                board_size,
                game_type,
                status,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };

            Ok((game, row.winner_name))
        })
        .collect::<cja::Result<Vec<_>>>()?;

    Ok(games_with_winners)
}
