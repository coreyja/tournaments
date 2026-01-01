// Game rules configuration for Battlesnake

use serde::{Deserialize, Serialize};

// Standard Battlesnake rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardRules {
    pub initial_health: u32,
    pub initial_food_count: usize,
    pub minimum_food: usize,
    pub food_spawn_chance: f32,
    pub snake_start_length: usize,
}

impl Default for StandardRules {
    fn default() -> Self {
        Self {
            initial_health: 100,
            initial_food_count: 1,   // Start with 1 food per snake
            minimum_food: 1,         // Always have at least 1 food
            food_spawn_chance: 0.15, // 15% chance to spawn food each turn
            snake_start_length: 3,
        }
    }
}

// Different game modes could have different rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleSet {
    Standard(StandardRules),
    // Future: Royale, Constrictor, etc.
}

impl Default for RuleSet {
    fn default() -> Self {
        RuleSet::Standard(StandardRules::default())
    }
}
