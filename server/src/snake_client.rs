//! HTTP client for communicating with Battlesnake servers
//!
//! This module handles all HTTP communication with snake servers following
//! the official Battlesnake API specification.

use battlesnake_game_types::types::Move;
use battlesnake_game_types::wire_representation::{BattleSnake, Game};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Response from a snake's /move endpoint
#[derive(Debug, Deserialize)]
pub struct MoveResponse {
    #[serde(rename = "move")]
    pub direction: String,
    pub shout: Option<String>,
}

/// Result of a move request including timing info
#[derive(Debug, Clone)]
pub struct MoveResult {
    pub snake_id: String,
    pub direction: Move,
    pub latency_ms: Option<i64>,
    pub timed_out: bool,
    pub shout: Option<String>,
}

/// Build the request body for a specific snake
///
/// The Battlesnake API expects the `you` field to be set to the snake
/// that the request is being sent to.
fn build_request_for_snake(game: &Game, snake: &BattleSnake) -> Game {
    Game {
        you: snake.clone(),
        board: game.board.clone(),
        turn: game.turn,
        game: game.game.clone(),
    }
}

/// Parse a direction string into a Move enum
fn parse_direction(s: &str) -> Option<Move> {
    match s.to_lowercase().as_str() {
        "up" => Some(Move::Up),
        "down" => Some(Move::Down),
        "left" => Some(Move::Left),
        "right" => Some(Move::Right),
        _ => None,
    }
}

/// Call a snake's /move endpoint
///
/// On timeout or error, falls back to the last direction (or Up if no last direction).
pub async fn request_move(
    client: &Client,
    url: &str,
    game: &Game,
    snake: &BattleSnake,
    timeout: Duration,
    last_direction: Option<Move>,
) -> MoveResult {
    let request_body = build_request_for_snake(game, snake);
    let move_url = format!("{}/move", url.trim_end_matches('/'));

    let start = Instant::now();

    let result = tokio::time::timeout(timeout, client.post(&move_url).json(&request_body).send())
        .await;

    let elapsed = start.elapsed().as_millis() as i64;

    match result {
        Ok(Ok(response)) => {
            match response.json::<MoveResponse>().await {
                Ok(move_response) => {
                    let direction = parse_direction(&move_response.direction)
                        .unwrap_or_else(|| last_direction.unwrap_or(Move::Up));
                    MoveResult {
                        snake_id: snake.id.clone(),
                        direction,
                        latency_ms: Some(elapsed),
                        timed_out: false,
                        shout: move_response.shout,
                    }
                }
                Err(e) => {
                    // JSON parse error - use fallback
                    tracing::warn!(
                        snake_id = %snake.id,
                        error = %e,
                        "Failed to parse move response, using fallback"
                    );
                    MoveResult {
                        snake_id: snake.id.clone(),
                        direction: last_direction.unwrap_or(Move::Up),
                        latency_ms: Some(elapsed),
                        timed_out: false,
                        shout: None,
                    }
                }
            }
        }
        Ok(Err(e)) => {
            // Network error - continue in same direction
            tracing::warn!(
                snake_id = %snake.id,
                error = %e,
                "Network error calling snake, using fallback"
            );
            MoveResult {
                snake_id: snake.id.clone(),
                direction: last_direction.unwrap_or(Move::Up),
                latency_ms: None,
                timed_out: true,
                shout: None,
            }
        }
        Err(_) => {
            // Timeout - continue in same direction
            tracing::warn!(
                snake_id = %snake.id,
                timeout_ms = timeout.as_millis(),
                "Snake timed out, using fallback"
            );
            MoveResult {
                snake_id: snake.id.clone(),
                direction: last_direction.unwrap_or(Move::Up),
                latency_ms: None,
                timed_out: true,
                shout: None,
            }
        }
    }
}

/// Call /start endpoint (fire and forget, no response expected)
pub async fn request_start(client: &Client, url: &str, game: &Game, snake: &BattleSnake, timeout: Duration) {
    let request_body = build_request_for_snake(game, snake);
    let start_url = format!("{}/start", url.trim_end_matches('/'));

    // Fire and forget - ignore result but log errors
    match tokio::time::timeout(timeout, client.post(&start_url).json(&request_body).send()).await {
        Ok(Ok(_)) => {
            tracing::debug!(snake_id = %snake.id, "Called /start successfully");
        }
        Ok(Err(e)) => {
            tracing::warn!(snake_id = %snake.id, error = %e, "Failed to call /start");
        }
        Err(_) => {
            tracing::warn!(snake_id = %snake.id, "Timeout calling /start");
        }
    }
}

/// Call /end endpoint (fire and forget, no response expected)
pub async fn request_end(client: &Client, url: &str, game: &Game, snake: &BattleSnake, timeout: Duration) {
    let request_body = build_request_for_snake(game, snake);
    let end_url = format!("{}/end", url.trim_end_matches('/'));

    // Fire and forget - ignore result but log errors
    match tokio::time::timeout(timeout, client.post(&end_url).json(&request_body).send()).await {
        Ok(Ok(_)) => {
            tracing::debug!(snake_id = %snake.id, "Called /end successfully");
        }
        Ok(Err(e)) => {
            tracing::warn!(snake_id = %snake.id, error = %e, "Failed to call /end");
        }
        Err(_) => {
            tracing::warn!(snake_id = %snake.id, "Timeout calling /end");
        }
    }
}

/// Request moves from all alive snakes in parallel
///
/// Returns a MoveResult for each alive snake.
pub async fn request_moves_parallel(
    client: &Client,
    game: &Game,
    snake_urls: &[(String, String)], // (snake_id, url)
    timeout: Duration,
    last_moves: &HashMap<String, Move>,
) -> Vec<MoveResult> {
    let futures: Vec<_> = game
        .board
        .snakes
        .iter()
        .filter(|s| s.health > 0)
        .filter_map(|snake| {
            snake_urls
                .iter()
                .find(|(id, _)| id == &snake.id)
                .map(|(_, url)| {
                    let last_direction = last_moves.get(&snake.id).copied();
                    request_move(client, url, game, snake, timeout, last_direction)
                })
        })
        .collect();

    futures::future::join_all(futures).await
}

/// Call /start for all snakes in parallel
pub async fn request_start_parallel(
    client: &Client,
    game: &Game,
    snake_urls: &[(String, String)],
    timeout: Duration,
) {
    let futures: Vec<_> = game
        .board
        .snakes
        .iter()
        .filter_map(|snake| {
            snake_urls
                .iter()
                .find(|(id, _)| id == &snake.id)
                .map(|(_, url)| request_start(client, url, game, snake, timeout))
        })
        .collect();

    futures::future::join_all(futures).await;
}

/// Call /end for all snakes in parallel
pub async fn request_end_parallel(
    client: &Client,
    game: &Game,
    snake_urls: &[(String, String)],
    timeout: Duration,
) {
    let futures: Vec<_> = game
        .board
        .snakes
        .iter()
        .filter_map(|snake| {
            snake_urls
                .iter()
                .find(|(id, _)| id == &snake.id)
                .map(|(_, url)| request_end(client, url, game, snake, timeout))
        })
        .collect();

    futures::future::join_all(futures).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_direction() {
        assert_eq!(parse_direction("up"), Some(Move::Up));
        assert_eq!(parse_direction("UP"), Some(Move::Up));
        assert_eq!(parse_direction("Down"), Some(Move::Down));
        assert_eq!(parse_direction("left"), Some(Move::Left));
        assert_eq!(parse_direction("RIGHT"), Some(Move::Right));
        assert_eq!(parse_direction("invalid"), None);
        assert_eq!(parse_direction(""), None);
    }

    #[test]
    fn test_move_result_clone() {
        let result = MoveResult {
            snake_id: "test".to_string(),
            direction: Move::Up,
            latency_ms: Some(100),
            timed_out: false,
            shout: Some("hello".to_string()),
        };
        let cloned = result.clone();
        assert_eq!(cloned.snake_id, "test");
        assert_eq!(cloned.direction, Move::Up);
        assert_eq!(cloned.latency_ms, Some(100));
        assert!(!cloned.timed_out);
        assert_eq!(cloned.shout, Some("hello".to_string()));
    }
}
