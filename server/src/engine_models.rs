//! Types for deserializing game data from the legacy Battlesnake Engine database.
//!
//! The Engine stores games and frames as JSONB with PascalCase field names (from protobuf).

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Game metadata from the Engine's `games` table `value` column.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EngineGame {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Width")]
    pub width: i32,
    #[serde(rename = "Height")]
    pub height: i32,
    #[serde(rename = "Ruleset", default)]
    pub ruleset: HashMap<String, String>,
    #[serde(rename = "SnakeTimeout")]
    pub snake_timeout: i32,
    #[serde(rename = "MaxTurns")]
    pub max_turns: i32,
    #[serde(rename = "FoodSpawns", default)]
    pub food_spawns: Vec<PointOnTurn>,
    #[serde(rename = "HazardSpawns", default)]
    pub hazard_spawns: Vec<PointOnTurn>,
    #[serde(rename = "Source")]
    pub source: Option<String>,
    #[serde(rename = "RulesetName")]
    pub ruleset_name: Option<String>,
    #[serde(rename = "RulesStages")]
    pub rules_stages: Option<Vec<String>>,
    #[serde(rename = "Map")]
    pub map: Option<String>,
    /// Unix timestamp in microseconds
    #[serde(rename = "Created")]
    pub created: i64,
}

impl EngineGame {
    /// Convert the Created timestamp (microseconds) to a DateTime
    pub fn created_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_micros(self.created).unwrap_or(DateTime::UNIX_EPOCH)
    }

    /// Get the board size as a string like "11x11"
    pub fn board_size(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }

    /// Get the game type from the ruleset name
    pub fn game_type(&self) -> String {
        self.ruleset_name
            .clone()
            .unwrap_or_else(|| "standard".to_string())
    }
}

/// A single frame/turn of a game from the Engine's `game_frames` table `value` column.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EngineGameFrame {
    #[serde(rename = "Turn")]
    pub turn: i32,
    #[serde(rename = "Snakes", default)]
    pub snakes: Vec<EngineSnake>,
    #[serde(rename = "Food", default)]
    pub food: Vec<Point>,
    #[serde(rename = "Hazards", default)]
    pub hazards: Vec<Point>,
    #[serde(rename = "GameState", default)]
    pub game_state: HashMap<String, String>,
    #[serde(rename = "PointState", default)]
    pub point_state: Vec<PointState>,
}

/// A snake in a game frame.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EngineSnake {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "URL")]
    pub url: Option<String>,
    #[serde(rename = "Body", default)]
    pub body: Vec<Point>,
    #[serde(rename = "Health")]
    pub health: i32,
    #[serde(rename = "Death")]
    pub death: Option<Death>,
    #[serde(rename = "Color")]
    pub color: Option<String>,
    #[serde(rename = "HeadType")]
    pub head_type: Option<String>,
    #[serde(rename = "TailType")]
    pub tail_type: Option<String>,
    #[serde(rename = "Latency")]
    pub latency: Option<String>,
    #[serde(rename = "Shout")]
    pub shout: Option<String>,
    #[serde(rename = "Squad")]
    pub squad: Option<String>,
    #[serde(rename = "APIVersion")]
    pub api_version: Option<String>,
    #[serde(rename = "Author")]
    pub author: Option<String>,
    #[serde(rename = "StatusCode")]
    pub status_code: Option<i32>,
    #[serde(rename = "Error")]
    pub error: Option<String>,
    #[serde(rename = "TimingMicros", default)]
    pub timing_micros: HashMap<String, i32>,
    #[serde(rename = "IsBot")]
    pub is_bot: Option<bool>,
    #[serde(rename = "IsEnvironment")]
    pub is_environment: Option<bool>,
    #[serde(rename = "ProxyURL")]
    pub proxy_url: Option<String>,
}

/// Death information for a snake.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Death {
    #[serde(rename = "Cause")]
    pub cause: String,
    #[serde(rename = "Turn")]
    pub turn: i32,
    #[serde(rename = "EliminatedBy")]
    pub eliminated_by: Option<String>,
}

/// A point on the board.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Point {
    #[serde(rename = "X")]
    pub x: i32,
    #[serde(rename = "Y")]
    pub y: i32,
}

/// A point with an associated turn number.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PointOnTurn {
    #[serde(rename = "X")]
    pub x: i32,
    #[serde(rename = "Y")]
    pub y: i32,
    #[serde(rename = "Turn")]
    pub turn: i32,
}

/// State associated with a point on the board.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PointState {
    #[serde(rename = "X")]
    pub x: i32,
    #[serde(rename = "Y")]
    pub y: i32,
    #[serde(rename = "Value")]
    pub value: i32,
}

/// Combined export format for archiving a complete game to GCS.
#[derive(Debug, Clone, Serialize)]
pub struct GameExport {
    pub game: EngineGame,
    pub frames: Vec<EngineGameFrame>,
    pub exported_at: DateTime<Utc>,
}
