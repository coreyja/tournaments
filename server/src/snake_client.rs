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
use url::Url;

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

/// Build a URL for a snake endpoint, properly handling query parameters
///
/// This appends the endpoint path (e.g., "move", "start", "end") to the base URL
/// while preserving any query parameters in the correct position.
fn build_endpoint_url(base_url: &str, endpoint: &str) -> String {
    // Try to parse as a proper URL
    if let Ok(mut url) = Url::parse(base_url) {
        // Get the current path, trim trailing slashes, and append the endpoint
        let current_path = url.path().trim_end_matches('/');
        let new_path = format!("{}/{}", current_path, endpoint);
        url.set_path(&new_path);
        url.to_string()
    } else {
        // Fallback to simple string concatenation if URL parsing fails
        format!("{}/{}", base_url.trim_end_matches('/'), endpoint)
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
    let move_url = build_endpoint_url(url, "move");

    let start = Instant::now();

    let result =
        tokio::time::timeout(timeout, client.post(&move_url).json(&request_body).send()).await;

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
pub async fn request_start(
    client: &Client,
    url: &str,
    game: &Game,
    snake: &BattleSnake,
    timeout: Duration,
) {
    let request_body = build_request_for_snake(game, snake);
    let start_url = build_endpoint_url(url, "start");

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
pub async fn request_end(
    client: &Client,
    url: &str,
    game: &Game,
    snake: &BattleSnake,
    timeout: Duration,
) {
    let request_body = build_request_for_snake(game, snake);
    let end_url = build_endpoint_url(url, "end");

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
    use battlesnake_game_types::wire_representation::{Board, NestedGame, Position, Ruleset};
    use std::collections::VecDeque;

    #[test]
    fn test_build_endpoint_url_simple() {
        let url = build_endpoint_url("https://example.com", "move");
        assert_eq!(url, "https://example.com/move");
    }

    #[test]
    fn test_build_endpoint_url_with_trailing_slash() {
        let url = build_endpoint_url("https://example.com/", "move");
        assert_eq!(url, "https://example.com/move");
    }

    #[test]
    fn test_build_endpoint_url_with_path() {
        let url = build_endpoint_url("https://example.com/api/v1", "move");
        assert_eq!(url, "https://example.com/api/v1/move");
    }

    #[test]
    fn test_build_endpoint_url_with_query_params() {
        let url = build_endpoint_url("https://example.com?token=secret", "move");
        assert_eq!(url, "https://example.com/move?token=secret");
    }

    #[test]
    fn test_build_endpoint_url_with_path_and_query_params() {
        let url = build_endpoint_url("https://example.com/api?token=secret&version=2", "move");
        assert_eq!(url, "https://example.com/api/move?token=secret&version=2");
    }

    #[test]
    fn test_build_endpoint_url_with_trailing_slash_and_query_params() {
        let url = build_endpoint_url("https://example.com/api/?token=secret", "start");
        assert_eq!(url, "https://example.com/api/start?token=secret");
    }

    #[test]
    fn test_build_endpoint_url_all_endpoints() {
        let base = "https://snake.example.com?auth=abc123";
        assert_eq!(
            build_endpoint_url(base, "move"),
            "https://snake.example.com/move?auth=abc123"
        );
        assert_eq!(
            build_endpoint_url(base, "start"),
            "https://snake.example.com/start?auth=abc123"
        );
        assert_eq!(
            build_endpoint_url(base, "end"),
            "https://snake.example.com/end?auth=abc123"
        );
    }

    fn create_test_snake(id: &str) -> BattleSnake {
        BattleSnake {
            id: id.to_string(),
            name: format!("Snake {}", id),
            head: Position::new(5, 5),
            body: VecDeque::from([
                Position::new(5, 5),
                Position::new(5, 4),
                Position::new(5, 3),
            ]),
            health: 100,
            shout: None,
            actual_length: None,
        }
    }

    fn create_test_game_with_snakes(snakes: Vec<BattleSnake>) -> Game {
        let you = snakes
            .first()
            .cloned()
            .unwrap_or_else(|| create_test_snake("default"));
        Game {
            you,
            board: Board {
                height: 11,
                width: 11,
                food: vec![Position::new(3, 3)],
                snakes,
                hazards: vec![],
            },
            turn: 5,
            game: NestedGame {
                id: "test-game".to_string(),
                ruleset: Ruleset {
                    name: "standard".to_string(),
                    version: "v1.0.0".to_string(),
                    settings: None,
                },
                timeout: 500,
                map: None,
                source: None,
            },
        }
    }

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

    #[test]
    fn test_build_request_for_snake_sets_you_field() {
        let snake1 = create_test_snake("snake-1");
        let snake2 = create_test_snake("snake-2");
        let game = create_test_game_with_snakes(vec![snake1.clone(), snake2.clone()]);

        // Build request for snake2 - the `you` field should be snake2
        let request = build_request_for_snake(&game, &snake2);

        assert_eq!(request.you.id, "snake-2");
        assert_eq!(request.you.name, "Snake snake-2");
        // Board should be preserved
        assert_eq!(request.board.snakes.len(), 2);
        assert_eq!(request.turn, 5);
        assert_eq!(request.game.id, "test-game");
    }

    #[test]
    fn test_build_request_for_snake_preserves_board() {
        let snake1 = create_test_snake("snake-1");
        let game = create_test_game_with_snakes(vec![snake1.clone()]);

        let request = build_request_for_snake(&game, &snake1);

        // All board properties should be preserved
        assert_eq!(request.board.height, 11);
        assert_eq!(request.board.width, 11);
        assert_eq!(request.board.food.len(), 1);
        assert_eq!(request.board.food[0].x, 3);
        assert_eq!(request.board.food[0].y, 3);
    }

    #[test]
    fn test_move_response_deserialization() {
        let json = r#"{"move": "up"}"#;
        let response: MoveResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.direction, "up");
        assert!(response.shout.is_none());
    }

    #[test]
    fn test_move_response_deserialization_with_shout() {
        let json = r#"{"move": "down", "shout": "I'm coming for you!"}"#;
        let response: MoveResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.direction, "down");
        assert_eq!(response.shout, Some("I'm coming for you!".to_string()));
    }

    #[test]
    fn test_move_response_deserialization_case_sensitivity() {
        // The API spec says "move" should be lowercase, but snakes might return different cases
        let json = r#"{"move": "LEFT"}"#;
        let response: MoveResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.direction, "LEFT");
        // parse_direction handles case normalization
        assert_eq!(parse_direction(&response.direction), Some(Move::Left));
    }
}
