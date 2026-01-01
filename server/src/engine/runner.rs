// Game runner - orchestrates the game engine and manages game execution

use super::{Direction, GameEngine, GameEvent, GameState, StandardEngine};
use crate::models::game::GameStatus;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;
use uuid::Uuid;

pub struct GameRunner {
    engine: Box<dyn GameEngine + Send + Sync>,
    turn_delay: Duration,
}

impl GameRunner {
    pub fn new() -> Self {
        Self {
            engine: Box::new(StandardEngine::new()),
            turn_delay: Duration::from_millis(500), // 500ms between turns for visualization
        }
    }

    pub fn with_turn_delay(mut self, delay: Duration) -> Self {
        self.turn_delay = delay;
        self
    }

    /// Run a game with random moves for all snakes
    pub async fn run_game_with_random_moves(
        &self,
        game_id: Uuid,
        snake_ids: Vec<Uuid>,
        board_width: u32,
        board_height: u32,
        state_callback: impl Fn(GameState, Vec<GameEvent>) + Send,
    ) -> cja::Result<GameResult> {
        info!("Starting game {} with {} snakes", game_id, snake_ids.len());

        // Initialize game
        let mut game_state =
            self.engine
                .initialize_game(snake_ids.clone(), board_width, board_height);
        state_callback(game_state.clone(), vec![]);

        // Game loop
        let max_turns = 500; // Prevent infinite games
        let mut winner = None;

        for turn in 0..max_turns {
            // Generate random moves for all alive snakes
            let mut moves = HashMap::new();
            for snake in game_state.get_alive_snakes() {
                let direction = self.get_random_valid_move(snake, &game_state);
                moves.insert(snake.id, direction);
            }

            // Process turn
            let events = self.engine.process_turn(&mut game_state, moves);

            // Send state update
            state_callback(game_state.clone(), events.clone());

            // Check for game over
            if self.engine.is_game_over(&game_state) {
                winner = self.engine.get_winner(&game_state);
                info!(
                    "Game {} ended at turn {} with winner: {:?}",
                    game_id, turn, winner
                );
                break;
            }

            // Delay for visualization
            sleep(self.turn_delay).await;
        }

        // Calculate placements
        let mut placements = HashMap::new();
        let mut placement = 1;

        // Winner gets placement 1
        if let Some(winner_id) = winner {
            placements.insert(winner_id, placement);
            placement += 1;
        }

        // Other snakes get placements based on when they died
        // (This is simplified - in a real game we'd track elimination order)
        for snake_id in game_state.snakes.keys() {
            if !placements.contains_key(snake_id) {
                placements.insert(*snake_id, placement);
                placement += 1;
            }
        }

        Ok(GameResult {
            winner,
            placements,
            final_turn: game_state.turn,
        })
    }

    /// Get a random valid move for a snake
    fn get_random_valid_move(&self, snake: &super::Snake, state: &GameState) -> Direction {
        use rand::seq::SliceRandom;

        let directions = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let head = snake.get_head();

        // Filter out moves that would immediately cause death
        let valid_moves: Vec<_> = directions
            .into_iter()
            .filter(|&dir| {
                let next = head.apply_direction(dir);

                // Check wall collision
                if !state.board.is_valid_position(&next) {
                    return false;
                }

                // Check self collision (but not with tail since it will move)
                let body_without_tail = &snake.body[..snake.body.len() - 1];
                if body_without_tail.contains(&next) {
                    return false;
                }

                true
            })
            .collect();

        // Choose a random valid move, or Up as fallback
        valid_moves
            .choose(&mut rand::thread_rng())
            .copied()
            .unwrap_or(Direction::Up)
    }
}

pub struct GameResult {
    pub winner: Option<Uuid>,
    pub placements: HashMap<Uuid, u32>,
    pub final_turn: u32,
}

/// Run a game and update the database
pub async fn run_and_store_game(
    pool: &sqlx::PgPool,
    game_id: Uuid,
    websocket_sender: Option<tokio::sync::mpsc::Sender<String>>,
) -> cja::Result<()> {
    use crate::models::{game, game_battlesnake};

    info!("Running game {}", game_id);

    // Update status to running
    game::update_game_status(pool, game_id, GameStatus::Running).await?;

    // Get battlesnakes for the game
    let battlesnakes = game_battlesnake::get_battlesnakes_by_game_id(pool, game_id).await?;
    if battlesnakes.is_empty() {
        return Err(color_eyre::eyre::eyre!("No battlesnakes in game"));
    }

    let snake_ids: Vec<_> = battlesnakes.iter().map(|b| b.battlesnake_id).collect();

    // Create game runner
    let runner = GameRunner::new();

    // Run the game
    let result = runner
        .run_game_with_random_moves(
            game_id,
            snake_ids,
            11, // 11x11 board for now
            11,
            |state, events| {
                // Send state updates via WebSocket if available
                if let Some(sender) = &websocket_sender {
                    let message = serde_json::json!({
                        "type": "game_state",
                        "state": state.to_api_format(),
                        "events": events,
                    });

                    let _ = sender.try_send(message.to_string());
                }
            },
        )
        .await?;

    // Store results in database
    for (snake_id, placement) in result.placements {
        game_battlesnake::set_game_result(
            pool,
            game_id,
            snake_id,
            game_battlesnake::SetGameResult {
                placement: placement as i32,
            },
        )
        .await?;
    }

    // Update game status to finished
    game::update_game_status(pool, game_id, GameStatus::Finished).await?;

    info!("Game {} completed successfully", game_id);
    Ok(())
}
