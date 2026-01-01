// Battlesnake Game Engine
// This module implements the core game logic for running Battlesnake games

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod game_state;
pub mod rules;
pub mod runner;

#[cfg(test)]
mod tests;

pub use game_state::{Board, Coordinate, Direction, Food, GameState, Snake};
pub use rules::StandardRules;
pub use runner::run_and_store_game;

// Main engine trait that different rule sets can implement
pub trait GameEngine {
    fn initialize_game(
        &self,
        snake_ids: Vec<Uuid>,
        board_width: u32,
        board_height: u32,
    ) -> GameState;
    fn process_turn(
        &self,
        state: &mut GameState,
        moves: HashMap<Uuid, Direction>,
    ) -> Vec<GameEvent>;
    fn is_game_over(&self, state: &GameState) -> bool;
    fn get_winner(&self, state: &GameState) -> Option<Uuid>;
}

// Events that occur during gameplay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    SnakeMoved {
        snake_id: Uuid,
        direction: Direction,
        new_head: Coordinate,
    },
    SnakeAteFood {
        snake_id: Uuid,
        food_position: Coordinate,
    },
    SnakeDied {
        snake_id: Uuid,
        cause: DeathCause,
    },
    FoodSpawned {
        position: Coordinate,
    },
    GameOver {
        winner: Option<Uuid>,
    },
}

// Reasons a snake can die
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeathCause {
    WallCollision,
    SnakeCollision { other_snake_id: Uuid },
    SelfCollision,
    HeadToHeadCollision { other_snake_id: Uuid },
    Starvation,
}

// Standard Battlesnake game engine
pub struct StandardEngine {
    rules: StandardRules,
}

impl StandardEngine {
    pub fn new() -> Self {
        Self {
            rules: StandardRules::default(),
        }
    }
}

impl GameEngine for StandardEngine {
    fn initialize_game(
        &self,
        snake_ids: Vec<Uuid>,
        board_width: u32,
        board_height: u32,
    ) -> GameState {
        let mut state = GameState::new(board_width, board_height);

        // Place snakes in corners/edges with some spacing
        let starting_positions =
            self.get_starting_positions(snake_ids.len(), board_width, board_height);

        for (snake_id, start_pos) in snake_ids.into_iter().zip(starting_positions) {
            let snake = Snake::new(snake_id, start_pos);
            state.snakes.insert(snake_id, snake);
        }

        // Spawn initial food
        state.food = self.spawn_food(&state, self.rules.initial_food_count);

        state
    }

    fn process_turn(
        &self,
        state: &mut GameState,
        moves: HashMap<Uuid, Direction>,
    ) -> Vec<GameEvent> {
        let mut events = Vec::new();
        state.turn += 1;

        // Move all snakes
        let mut new_heads: HashMap<Uuid, Coordinate> = HashMap::new();
        for (snake_id, snake) in &state.snakes {
            if !snake.is_alive {
                continue;
            }

            let direction = moves.get(snake_id).copied().unwrap_or_else(|| {
                // Random move if no move provided
                self.get_random_move(snake, &state.board)
            });

            let new_head = snake.get_next_head(direction);
            new_heads.insert(*snake_id, new_head);

            events.push(GameEvent::SnakeMoved {
                snake_id: *snake_id,
                direction,
                new_head,
            });
        }

        // Check for collisions and apply deaths
        let deaths = self.check_collisions(&new_heads, state);
        for (snake_id, cause) in &deaths {
            if let Some(snake) = state.snakes.get_mut(snake_id) {
                snake.is_alive = false;
                events.push(GameEvent::SnakeDied {
                    snake_id: *snake_id,
                    cause: cause.clone(),
                });
            }
        }

        // Move surviving snakes
        let mut food_eaten = Vec::new();
        for (snake_id, new_head) in new_heads {
            if deaths.contains_key(&snake_id) {
                continue;
            }

            if let Some(snake) = state.snakes.get_mut(&snake_id) {
                // Check if snake ate food
                if let Some(food_idx) = state.food.iter().position(|f| f.position == new_head) {
                    let food = state.food.remove(food_idx);
                    food_eaten.push(food.position);
                    snake.health = 100; // Reset health on food
                    snake.grow(new_head);

                    events.push(GameEvent::SnakeAteFood {
                        snake_id,
                        food_position: food.position,
                    });
                } else {
                    snake.move_to(new_head);
                    snake.health = snake.health.saturating_sub(1);

                    // Check for starvation
                    if snake.health == 0 {
                        snake.is_alive = false;
                        events.push(GameEvent::SnakeDied {
                            snake_id,
                            cause: DeathCause::Starvation,
                        });
                    }
                }
            }
        }

        // Spawn new food to replace eaten food
        let new_food = self.spawn_food(state, food_eaten.len());
        for food in new_food {
            events.push(GameEvent::FoodSpawned {
                position: food.position,
            });
            state.food.push(food);
        }

        // Check for game over
        if self.is_game_over(state) {
            events.push(GameEvent::GameOver {
                winner: self.get_winner(state),
            });
        }

        events
    }

    fn is_game_over(&self, state: &GameState) -> bool {
        let alive_count = state.snakes.values().filter(|s| s.is_alive).count();
        alive_count <= 1
    }

    fn get_winner(&self, state: &GameState) -> Option<Uuid> {
        let alive_snakes: Vec<_> = state.snakes.iter().filter(|(_, s)| s.is_alive).collect();

        if alive_snakes.len() == 1 {
            Some(*alive_snakes[0].0)
        } else {
            None
        }
    }
}

impl StandardEngine {
    fn get_starting_positions(
        &self,
        snake_count: usize,
        width: u32,
        height: u32,
    ) -> Vec<Coordinate> {
        // Simple starting positions - corners and edges
        let positions = vec![
            Coordinate { x: 1, y: 1 }, // Bottom-left
            Coordinate {
                x: width - 2,
                y: height - 2,
            }, // Top-right
            Coordinate {
                x: 1,
                y: height - 2,
            }, // Top-left
            Coordinate { x: width - 2, y: 1 }, // Bottom-right
        ];

        positions.into_iter().take(snake_count).collect()
    }

    fn spawn_food(&self, state: &GameState, count: usize) -> Vec<Food> {
        let mut food = Vec::new();
        let mut attempts = 0;

        while food.len() < count && attempts < 100 {
            attempts += 1;

            let x = rand::random::<u32>() % state.board.width;
            let y = rand::random::<u32>() % state.board.height;
            let position = Coordinate { x, y };

            // Check position is empty
            let occupied = state.snakes.values().any(|s| s.body.contains(&position))
                || state.food.iter().any(|f| f.position == position)
                || food.iter().any(|f: &Food| f.position == position);

            if !occupied {
                food.push(Food { position });
            }
        }

        food
    }

    fn get_random_move(&self, snake: &Snake, board: &Board) -> Direction {
        use rand::seq::SliceRandom;

        let directions = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let head = &snake.body[0];

        // Filter out moves that would immediately hit a wall
        let valid_moves: Vec<_> = directions
            .into_iter()
            .filter(|&dir| {
                let next = head.apply_direction(dir);
                next.x < board.width && next.y < board.height
            })
            .collect();

        valid_moves
            .choose(&mut rand::thread_rng())
            .copied()
            .unwrap_or(Direction::Up) // Fallback if no valid moves
    }

    fn check_collisions(
        &self,
        new_heads: &HashMap<Uuid, Coordinate>,
        state: &GameState,
    ) -> HashMap<Uuid, DeathCause> {
        let mut deaths = HashMap::new();

        // Check each snake for collisions
        for (snake_id, new_head) in new_heads {
            let snake = match state.snakes.get(snake_id) {
                Some(s) if s.is_alive => s,
                _ => continue,
            };

            // Wall collision
            if new_head.x >= state.board.width || new_head.y >= state.board.height {
                deaths.insert(*snake_id, DeathCause::WallCollision);
                continue;
            }

            // Self collision (with body, not the tail that will move)
            let body_without_tail: Vec<_> = snake.body.iter().take(snake.body.len() - 1).collect();
            if body_without_tail.contains(&new_head) {
                deaths.insert(*snake_id, DeathCause::SelfCollision);
                continue;
            }

            // Collision with other snakes
            for (other_id, other_snake) in &state.snakes {
                if other_id == snake_id || !other_snake.is_alive {
                    continue;
                }

                // Check collision with other snake's body
                if other_snake.body.contains(new_head) {
                    deaths.insert(
                        *snake_id,
                        DeathCause::SnakeCollision {
                            other_snake_id: *other_id,
                        },
                    );
                }
            }
        }

        // Check for head-to-head collisions
        let mut head_collisions: HashMap<Coordinate, Vec<Uuid>> = HashMap::new();
        for (snake_id, new_head) in new_heads {
            if !deaths.contains_key(snake_id) {
                head_collisions
                    .entry(*new_head)
                    .or_default()
                    .push(*snake_id);
            }
        }

        for (_, snake_ids) in head_collisions {
            if snake_ids.len() > 1 {
                // Head-to-head collision - smaller snake(s) die
                let mut sizes: Vec<_> = snake_ids
                    .iter()
                    .filter_map(|id| state.snakes.get(id).map(|s| (id, s.body.len())))
                    .collect();
                sizes.sort_by_key(|&(_, size)| size);
                sizes.reverse(); // Largest first

                let max_size = sizes[0].1;
                for (snake_id, size) in &sizes {
                    if *size < max_size {
                        // Smaller snake dies
                        deaths.insert(
                            **snake_id,
                            DeathCause::HeadToHeadCollision {
                                other_snake_id: *sizes[0].0,
                            },
                        );
                    }
                }

                // If all same size, all die
                if sizes.iter().all(|&(_, s)| s == max_size) {
                    for (snake_id, _) in &sizes {
                        if let Some(other_id) = snake_ids.iter().find(|id| **id != **snake_id) {
                            deaths.insert(
                                **snake_id,
                                DeathCause::HeadToHeadCollision {
                                    other_snake_id: *other_id,
                                },
                            );
                        }
                    }
                }
            }
        }

        deaths
    }
}
