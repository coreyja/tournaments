// Game state representation for Battlesnake

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// 2D coordinate on the game board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: u32,
    pub y: u32,
}

impl Coordinate {
    pub fn apply_direction(&self, direction: Direction) -> Coordinate {
        match direction {
            Direction::Up => Coordinate {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Down => Coordinate {
                x: self.x,
                y: self.y.saturating_sub(1),
            },
            Direction::Left => Coordinate {
                x: self.x.saturating_sub(1),
                y: self.y,
            },
            Direction::Right => Coordinate {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
}

// Movement directions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "up" => Some(Direction::Up),
            "down" => Some(Direction::Down),
            "left" => Some(Direction::Left),
            "right" => Some(Direction::Right),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Left => "left",
            Direction::Right => "right",
        }
    }
}

// Food on the board
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Food {
    pub position: Coordinate,
}

// Snake representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snake {
    pub id: Uuid,
    pub body: Vec<Coordinate>, // Head is at index 0
    pub health: u32,
    pub is_alive: bool,
}

impl Snake {
    pub fn new(id: Uuid, starting_position: Coordinate) -> Self {
        Self {
            id,
            body: vec![starting_position; 3], // Start with length 3
            health: 100,
            is_alive: true,
        }
    }

    pub fn get_head(&self) -> &Coordinate {
        &self.body[0]
    }

    pub fn get_next_head(&self, direction: Direction) -> Coordinate {
        self.body[0].apply_direction(direction)
    }

    pub fn move_to(&mut self, new_head: Coordinate) {
        self.body.insert(0, new_head);
        self.body.pop(); // Remove tail
    }

    pub fn grow(&mut self, new_head: Coordinate) {
        self.body.insert(0, new_head);
        // Don't remove tail when growing
    }
}

// Game board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub width: u32,
    pub height: u32,
}

impl Board {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn is_valid_position(&self, pos: &Coordinate) -> bool {
        pos.x < self.width && pos.y < self.height
    }
}

// Complete game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub board: Board,
    pub snakes: HashMap<Uuid, Snake>,
    pub food: Vec<Food>,
    pub turn: u32,
}

impl GameState {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            board: Board::new(width, height),
            snakes: HashMap::new(),
            food: Vec::new(),
            turn: 0,
        }
    }

    pub fn get_alive_snakes(&self) -> Vec<&Snake> {
        self.snakes.values().filter(|s| s.is_alive).collect()
    }

    pub fn get_alive_snake_count(&self) -> usize {
        self.snakes.values().filter(|s| s.is_alive).count()
    }

    // Convert to format suitable for WebSocket/API transmission
    pub fn to_api_format(&self) -> serde_json::Value {
        serde_json::json!({
            "turn": self.turn,
            "board": {
                "width": self.board.width,
                "height": self.board.height,
                "food": self.food.iter().map(|f| {
                    serde_json::json!({
                        "x": f.position.x,
                        "y": f.position.y
                    })
                }).collect::<Vec<_>>(),
                "snakes": self.snakes.values().map(|snake| {
                    serde_json::json!({
                        "id": snake.id.to_string(),
                        "name": format!("Snake {}", &snake.id.to_string()[..8]),
                        "health": snake.health,
                        "body": snake.body.iter().map(|coord| {
                            serde_json::json!({
                                "x": coord.x,
                                "y": coord.y
                            })
                        }).collect::<Vec<_>>(),
                        "latency": 0, // Placeholder
                        "head": {
                            "x": snake.body[0].x,
                            "y": snake.body[0].y
                        },
                        "length": snake.body.len(),
                        "shout": "", // Optional shout
                        "elimination": if snake.is_alive {
                            serde_json::Value::Null
                        } else {
                            serde_json::json!({
                                "cause": "collision", // Simplified for now
                                "turn": self.turn,
                                "by": ""
                            })
                        }
                    })
                }).collect::<Vec<_>>()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_apply_direction() {
        let coord = Coordinate { x: 5, y: 5 };

        assert_eq!(
            coord.apply_direction(Direction::Up),
            Coordinate { x: 5, y: 6 }
        );
        assert_eq!(
            coord.apply_direction(Direction::Down),
            Coordinate { x: 5, y: 4 }
        );
        assert_eq!(
            coord.apply_direction(Direction::Left),
            Coordinate { x: 4, y: 5 }
        );
        assert_eq!(
            coord.apply_direction(Direction::Right),
            Coordinate { x: 6, y: 5 }
        );
    }

    #[test]
    fn test_coordinate_apply_direction_boundaries() {
        let coord = Coordinate { x: 0, y: 0 };

        // Should use saturating_sub to prevent underflow
        assert_eq!(
            coord.apply_direction(Direction::Down),
            Coordinate { x: 0, y: 0 }
        );
        assert_eq!(
            coord.apply_direction(Direction::Left),
            Coordinate { x: 0, y: 0 }
        );
    }

    #[test]
    fn test_snake_movement() {
        let mut snake = Snake::new(Uuid::new_v4(), Coordinate { x: 5, y: 5 });

        assert_eq!(snake.body.len(), 3);
        assert_eq!(*snake.get_head(), Coordinate { x: 5, y: 5 });

        // Move without growing
        snake.move_to(Coordinate { x: 5, y: 6 });
        assert_eq!(snake.body.len(), 3);
        assert_eq!(*snake.get_head(), Coordinate { x: 5, y: 6 });

        // Grow
        snake.grow(Coordinate { x: 5, y: 7 });
        assert_eq!(snake.body.len(), 4);
        assert_eq!(*snake.get_head(), Coordinate { x: 5, y: 7 });
    }

    #[test]
    fn test_direction_from_str() {
        assert_eq!(Direction::from_str("up"), Some(Direction::Up));
        assert_eq!(Direction::from_str("DOWN"), Some(Direction::Down));
        assert_eq!(Direction::from_str("Left"), Some(Direction::Left));
        assert_eq!(Direction::from_str("RIGHT"), Some(Direction::Right));
        assert_eq!(Direction::from_str("invalid"), None);
    }
}
