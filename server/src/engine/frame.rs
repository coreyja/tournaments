//! Frame data serialization for the Battlesnake board viewer
//!
//! This module converts the internal game state to the PascalCase JSON format
//! expected by the board viewer.

use battlesnake_game_types::wire_representation::{Game, Position};
use serde::Serialize;
use std::collections::VecDeque;

/// Information about a snake's death
#[derive(Debug, Clone)]
pub struct DeathInfo {
    /// The snake's ID
    pub snake_id: String,
    /// The turn on which the snake died
    pub turn: i32,
    /// The cause of death (e.g., "wall-collision", "head-collision")
    pub cause: String,
    /// The ID of the snake that eliminated this snake (if applicable)
    /// TODO: Pass eliminated_by from the game engine once head-to-head collision tracking is implemented
    pub eliminated_by: String,
}

/// Convert a VecDeque of Positions to a Vec of FrameCoords
fn body_to_coords(body: &VecDeque<Position>) -> Vec<FrameCoord> {
    body.iter().map(|p| FrameCoord { x: p.x, y: p.y }).collect()
}

/// Frame data in PascalCase format for the board viewer
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EngineGameFrame {
    pub turn: i32,
    pub snakes: Vec<FrameSnake>,
    pub food: Vec<FrameCoord>,
    pub hazards: Vec<FrameCoord>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FrameSnake {
    #[serde(rename = "ID")]
    pub id: String,
    pub name: String,
    pub body: Vec<FrameCoord>,
    pub health: i32,
    pub color: String,
    pub head_type: String,
    pub tail_type: String,
    pub latency: String,
    pub shout: String,
    pub squad: String,
    #[serde(rename = "APIVersion")]
    pub api_version: String,
    pub author: String,
    pub death: Option<FrameDeath>,
    pub eliminated_cause: String,
    pub eliminated_by: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FrameCoord {
    #[serde(rename = "X")]
    pub x: i32,
    #[serde(rename = "Y")]
    pub y: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FrameDeath {
    pub cause: String,
    pub turn: i32,
    pub eliminated_by: String,
}

impl From<Position> for FrameCoord {
    fn from(pos: Position) -> Self {
        FrameCoord { x: pos.x, y: pos.y }
    }
}

/// Convert a Game state to a frame for the board viewer
pub fn game_to_frame(game: &Game, death_info: &[DeathInfo]) -> EngineGameFrame {
    EngineGameFrame {
        turn: game.turn,
        snakes: game
            .board
            .snakes
            .iter()
            .map(|s| {
                let death = death_info
                    .iter()
                    .find(|d| d.snake_id == s.id)
                    .map(|d| FrameDeath {
                        cause: d.cause.clone(),
                        turn: d.turn,
                        eliminated_by: d.eliminated_by.clone(),
                    });

                let (eliminated_cause, eliminated_by) = if s.health <= 0 {
                    death_info
                        .iter()
                        .find(|d| d.snake_id == s.id)
                        .map(|d| (d.cause.clone(), d.eliminated_by.clone()))
                        .unwrap_or_default()
                } else {
                    Default::default()
                };

                FrameSnake {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    body: body_to_coords(&s.body),
                    health: s.health,
                    color: generate_snake_color(&s.id),
                    head_type: "default".to_string(),
                    tail_type: "default".to_string(),
                    latency: "0".to_string(),
                    shout: s.shout.clone().unwrap_or_default(),
                    squad: "".to_string(),
                    api_version: "1".to_string(),
                    author: "".to_string(),
                    death,
                    eliminated_cause,
                    eliminated_by,
                }
            })
            .collect(),
        food: game.board.food.iter().map(|p| (*p).into()).collect(),
        hazards: game.board.hazards.iter().map(|p| (*p).into()).collect(),
    }
}

/// Generate a consistent color for a snake based on its ID
fn generate_snake_color(id: &str) -> String {
    // Generate a color from the hash of the ID
    let hash: u32 = id
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));

    // Use the hash to generate a hue
    let hue = (hash % 360) as f32;
    let saturation: f32 = 0.7;
    let lightness: f32 = 0.5;

    // HSL to RGB conversion
    let c: f32 = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let x: f32 = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m: f32 = lightness - c / 2.0;

    let (r, g, b) = match hue as i32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let r = ((r + m) * 255.0) as u8;
    let g = ((g + m) * 255.0) as u8;
    let b = ((b + m) * 255.0) as u8;

    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_coord_serialization() {
        let coord = FrameCoord { x: 5, y: 10 };
        let json = serde_json::to_string(&coord).unwrap();
        assert!(json.contains("\"X\":5"));
        assert!(json.contains("\"Y\":10"));
    }

    #[test]
    fn test_generate_snake_color() {
        let color1 = generate_snake_color("snake-1");
        let color2 = generate_snake_color("snake-2");

        // Colors should be different for different IDs
        assert_ne!(color1, color2);

        // Color should be consistent
        assert_eq!(color1, generate_snake_color("snake-1"));

        // Should be a valid hex color
        assert!(color1.starts_with('#'));
        assert_eq!(color1.len(), 7);
    }
}
