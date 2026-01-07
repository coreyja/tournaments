// Board viewer integration for Battlesnake
// Provides endpoints compatible with board.battlesnake.com viewer
//
// The board viewer expects:
// - GET /games/{id} - game metadata
// - WS /games/{id}/events - real-time frame stream
//
// Event format: {"EventType": "frame"|"game_end", "Data": {...}}
// Important: Uses uppercase X/Y in coordinates!

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::Response,
    Json,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// Game event type for board viewer (uses uppercase X/Y)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "EventType")]
pub enum BoardViewerEvent {
    #[serde(rename = "frame")]
    Frame { #[serde(rename = "Data")] data: FrameData },
    #[serde(rename = "game_end")]
    GameEnd { #[serde(rename = "Data")] data: GameEndData },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameData {
    #[serde(rename = "Turn")]
    pub turn: u32,
    #[serde(rename = "Snakes")]
    pub snakes: Vec<BoardSnake>,
    #[serde(rename = "Food")]
    pub food: Vec<BoardCoordinate>,
    #[serde(rename = "Hazards")]
    pub hazards: Vec<BoardCoordinate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardSnake {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Body")]
    pub body: Vec<BoardCoordinate>,
    #[serde(rename = "Health")]
    pub health: u32,
    #[serde(rename = "Death")]
    pub death: Option<SnakeDeath>,
    #[serde(rename = "Color")]
    pub color: String,
    #[serde(rename = "HeadType")]
    pub head_type: String,
    #[serde(rename = "TailType")]
    pub tail_type: String,
    #[serde(rename = "Latency")]
    pub latency: String,
    #[serde(rename = "Shout")]
    pub shout: String,
    #[serde(rename = "Author")]
    pub author: String,
    #[serde(rename = "IsBot")]
    pub is_bot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnakeDeath {
    #[serde(rename = "Cause")]
    pub cause: String,
    #[serde(rename = "Turn")]
    pub turn: u32,
    #[serde(rename = "EliminatedBy")]
    pub eliminated_by: String,
}

/// Coordinate with uppercase X/Y for board viewer compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardCoordinate {
    #[serde(rename = "X")]
    pub x: u32,
    #[serde(rename = "Y")]
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEndData {
    #[serde(rename = "Game")]
    pub game: GameMetadata,
}

/// Game metadata returned by GET /games/{id}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Width")]
    pub width: u32,
    #[serde(rename = "Height")]
    pub height: u32,
    #[serde(rename = "Ruleset")]
    pub ruleset: serde_json::Value,
    #[serde(rename = "RulesetName")]
    pub ruleset_name: String,
    #[serde(rename = "Map")]
    pub map: String,
}

/// Registry for active game broadcast channels
/// Each game has its own channel to broadcast frames to spectators
pub struct GameRegistry {
    games: RwLock<HashMap<Uuid, broadcast::Sender<BoardViewerEvent>>>,
}

impl GameRegistry {
    pub fn new() -> Self {
        Self {
            games: RwLock::new(HashMap::new()),
        }
    }

    /// Get or create a broadcast channel for a game
    pub async fn get_or_create_channel(&self, game_id: Uuid) -> broadcast::Sender<BoardViewerEvent> {
        let read = self.games.read().await;
        if let Some(sender) = read.get(&game_id) {
            return sender.clone();
        }
        drop(read);

        let mut write = self.games.write().await;
        // Double-check after acquiring write lock
        if let Some(sender) = write.get(&game_id) {
            return sender.clone();
        }

        // Create new channel with buffer for ~10 seconds of frames at 4fps
        let (tx, _rx) = broadcast::channel(40);
        write.insert(game_id, tx.clone());
        info!("Created broadcast channel for game {}", game_id);
        tx
    }

    /// Subscribe to a game's event stream
    pub async fn subscribe(&self, game_id: Uuid) -> broadcast::Receiver<BoardViewerEvent> {
        let sender = self.get_or_create_channel(game_id).await;
        sender.subscribe()
    }

    /// Broadcast an event to all spectators of a game
    pub async fn broadcast(&self, game_id: Uuid, event: BoardViewerEvent) {
        let read = self.games.read().await;
        if let Some(sender) = read.get(&game_id) {
            // If no receivers, that's fine - just means no spectators
            let _ = sender.send(event);
        }
    }

    /// Clean up a game's channel when the game ends
    pub async fn cleanup(&self, game_id: Uuid) {
        let mut write = self.games.write().await;
        if write.remove(&game_id).is_some() {
            info!("Cleaned up broadcast channel for game {}", game_id);
        }
    }
}

/// Global game registry - shared across all handlers
static GAME_REGISTRY: std::sync::OnceLock<Arc<GameRegistry>> = std::sync::OnceLock::new();

pub fn game_registry() -> &'static Arc<GameRegistry> {
    GAME_REGISTRY.get_or_init(|| Arc::new(GameRegistry::new()))
}

/// GET /api/games/{id} - Returns game metadata for board viewer
pub async fn get_game_metadata(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
) -> Result<Json<GameMetadata>, StatusCode> {
    use crate::models::game;

    let game_data = game::get_game_by_id(&state.db, game_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let (width, height) = match game_data.board_size {
        game::GameBoardSize::Small => (7, 7),
        game::GameBoardSize::Medium => (11, 11),
        game::GameBoardSize::Large => (19, 19),
    };

    let status = match game_data.status {
        game::GameStatus::Waiting => "waiting",
        game::GameStatus::Running => "running",
        game::GameStatus::Finished => "complete",
    };

    Ok(Json(GameMetadata {
        id: game_id.to_string(),
        status: status.to_string(),
        width,
        height,
        ruleset: json!({}),
        ruleset_name: "standard".to_string(),
        map: "standard".to_string(),
    }))
}

/// WS /api/games/{id}/events - WebSocket endpoint for frame streaming
pub async fn game_events_ws(
    State(_state): State<AppState>,
    Path(game_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_game_socket(socket, game_id))
}

async fn handle_game_socket(socket: WebSocket, game_id: Uuid) {
    info!("New WebSocket connection for game {}", game_id);

    let (mut sender, mut receiver) = socket.split();
    let mut game_rx = game_registry().subscribe(game_id).await;

    // Spawn task to forward game events to WebSocket
    let send_task = tokio::spawn(async move {
        loop {
            match game_rx.recv().await {
                Ok(event) => {
                    let json = serde_json::to_string(&event).unwrap_or_default();
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break; // Client disconnected
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("WebSocket client lagged by {} messages", n);
                    // Continue - we'll catch up
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("Game broadcast channel closed");
                    break;
                }
            }
        }
    });

    // Handle incoming messages (we don't expect any, but need to keep connection alive)
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {} // Ignore other messages
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }

    info!("WebSocket connection closed for game {}", game_id);
}

/// Convert internal GameState to board viewer frame format
pub fn game_state_to_frame(
    state: &crate::engine::GameState,
    snake_colors: &HashMap<Uuid, String>,
) -> FrameData {
    let snakes: Vec<BoardSnake> = state
        .snakes
        .iter()
        .map(|(id, snake)| {
            let default_color = "#888888".to_string();
            let color = snake_colors.get(id).unwrap_or(&default_color).clone();

            BoardSnake {
                id: id.to_string(),
                name: format!("Snake {}", &id.to_string()[..8]),
                body: snake
                    .body
                    .iter()
                    .map(|c| BoardCoordinate { x: c.x, y: c.y })
                    .collect(),
                health: snake.health,
                death: if snake.is_alive {
                    None
                } else {
                    Some(SnakeDeath {
                        cause: "collision".to_string(),
                        turn: state.turn,
                        eliminated_by: String::new(),
                    })
                },
                color,
                head_type: "default".to_string(),
                tail_type: "default".to_string(),
                latency: "0".to_string(),
                shout: String::new(),
                author: String::new(),
                is_bot: true,
            }
        })
        .collect();

    let food: Vec<BoardCoordinate> = state
        .food
        .iter()
        .map(|f| BoardCoordinate {
            x: f.position.x,
            y: f.position.y,
        })
        .collect();

    FrameData {
        turn: state.turn,
        snakes,
        food,
        hazards: vec![], // No hazards in standard mode
    }
}

// Snake colors - a palette for assigning colors to snakes
const SNAKE_COLORS: &[&str] = &[
    "#FF5733", // Red-orange
    "#33FF57", // Green
    "#3357FF", // Blue
    "#FF33F5", // Pink
    "#F5FF33", // Yellow
    "#33FFF5", // Cyan
    "#FF8C33", // Orange
    "#8C33FF", // Purple
];

/// Get a color for a snake based on its position in the game
pub fn get_snake_color(index: usize) -> String {
    SNAKE_COLORS[index % SNAKE_COLORS.len()].to_string()
}
