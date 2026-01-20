//! Game engine module using battlesnake-game-types wire representation
//!
//! This module provides game simulation using the official Battlesnake rules.
//! It uses the wire representation types directly for simplicity.

use battlesnake_game_types::types::{Move, RandomReasonableMovesGame};
use battlesnake_game_types::wire_representation::{
    BattleSnake, Board, Game, NestedGame, Position, Ruleset, Settings,
};
use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::VecDeque;
use uuid::Uuid;

use crate::models::game::{GameBoardSize, GameType};
use crate::models::game_battlesnake::GameBattlesnakeWithDetails;

const SNAKE_MAX_HEALTH: i32 = 100;
const SNAKE_START_SIZE: usize = 3;
const MAX_TURNS: i32 = 500;

/// Result of running a game
#[derive(Debug)]
pub struct GameResult {
    /// Snake IDs in order of placement (index 0 = winner/last alive)
    pub placements: Vec<String>,
    /// Final turn number
    pub final_turn: i32,
}

/// Create the initial game state from database models
pub fn create_initial_game(
    game_id: Uuid,
    board_size: GameBoardSize,
    game_type: GameType,
    battlesnakes: &[GameBattlesnakeWithDetails],
) -> Game {
    let (width, height) = match board_size {
        GameBoardSize::Small => (7, 7),
        GameBoardSize::Medium => (11, 11),
        GameBoardSize::Large => (19, 19),
    };

    let ruleset_name = match game_type {
        GameType::Standard => "standard",
        GameType::Royale => "royale",
        GameType::Constrictor => "constrictor",
        GameType::SnailMode => "snail_mode",
    };

    // Generate spawn positions
    let spawn_positions = generate_spawn_positions(width, height, battlesnakes.len());

    // Create snakes at spawn positions
    let snakes: Vec<BattleSnake> = battlesnakes
        .iter()
        .zip(spawn_positions.iter())
        .map(|(bs, pos)| {
            let body: VecDeque<Position> = (0..SNAKE_START_SIZE).map(|_| *pos).collect();
            BattleSnake {
                id: bs.battlesnake_id.to_string(),
                name: bs.name.clone(),
                head: *pos,
                body,
                health: SNAKE_MAX_HEALTH,
                shout: None,
                actual_length: None,
            }
        })
        .collect();

    // Place initial food - one near each snake plus center
    let food = generate_initial_food(width, height, &snakes);

    let board = Board {
        height: height as u32,
        width: width as u32,
        food,
        snakes: snakes.clone(),
        hazards: vec![],
    };

    // Use first snake as "you" (arbitrary for simulation purposes)
    let you = snakes.first().cloned().unwrap_or_else(|| BattleSnake {
        id: "dummy".to_string(),
        name: "Dummy".to_string(),
        head: Position::new(0, 0),
        body: VecDeque::new(),
        health: 0,
        shout: None,
        actual_length: None,
    });

    Game {
        you,
        board,
        turn: 0,
        game: NestedGame {
            id: game_id.to_string(),
            ruleset: Ruleset {
                name: ruleset_name.to_string(),
                version: "v1.0.0".to_string(),
                settings: Some(Settings {
                    food_spawn_chance: 15,
                    minimum_food: 1,
                    hazard_damage_per_turn: 15,
                    hazard_map: None,
                    hazard_map_author: None,
                    royale: None,
                }),
            },
            timeout: 500,
            map: None,
            source: None,
        },
    }
}

/// Generate spawn positions using the official Battlesnake algorithm
/// For <=8 snakes on boards >=7x7, uses fixed corner/cardinal positions
fn generate_spawn_positions(width: i32, _height: i32, num_snakes: usize) -> Vec<Position> {
    let mut rng = rand::thread_rng();

    // mn = 1, md = (width-1)/2, mx = width-2
    let mn = 1;
    let md = (width - 1) / 2;
    let mx = width - 2;

    // Corner positions
    let mut corner_points = vec![
        Position::new(mn, mn),
        Position::new(mn, mx),
        Position::new(mx, mn),
        Position::new(mx, mx),
    ];

    // Cardinal positions (edges)
    let mut cardinal_points = vec![
        Position::new(mn, md),
        Position::new(md, mn),
        Position::new(md, mx),
        Position::new(mx, md),
    ];

    // Shuffle both lists
    corner_points.shuffle(&mut rng);
    cardinal_points.shuffle(&mut rng);

    // Randomly decide whether to prioritize corners or cardinals
    let mut start_points = if rng.gen_bool(0.5) {
        let mut points = corner_points;
        points.extend(cardinal_points);
        points
    } else {
        let mut points = cardinal_points;
        points.extend(corner_points);
        points
    };

    // Take as many positions as we need
    start_points.truncate(num_snakes);
    start_points
}

/// Generate initial food positions
fn generate_initial_food(width: i32, height: i32, snakes: &[BattleSnake]) -> Vec<Position> {
    let mut rng = rand::thread_rng();
    let mut food: Vec<Position> = Vec::new();
    let center = Position::new((width - 1) / 2, (height - 1) / 2);

    // Place food near each snake (diagonal from head, away from center)
    for snake in snakes {
        let head = snake.head;
        let possible_food_locations = [
            Position::new(head.x - 1, head.y - 1),
            Position::new(head.x - 1, head.y + 1),
            Position::new(head.x + 1, head.y - 1),
            Position::new(head.x + 1, head.y + 1),
        ];

        // Filter valid positions
        let available: Vec<Position> = possible_food_locations
            .iter()
            .filter(|p| {
                // Must be on board
                p.x >= 0 && p.x < width && p.y >= 0 && p.y < height
                    // Not the center
                    && **p != center
                    // Not already food
                    && !food.contains(p)
                    // Not a corner
                    && !((p.x == 0 || p.x == width - 1) && (p.y == 0 || p.y == height - 1))
            })
            .copied()
            .collect();

        if let Some(pos) = available.choose(&mut rng) {
            food.push(*pos);
        }
    }

    // Always place food in center
    if !snakes.iter().any(|s| s.body.contains(&center)) {
        food.push(center);
    }

    food
}

/// Run a complete game with random moves, returning placements
pub fn run_game_with_random_moves(mut game: Game) -> GameResult {
    let mut rng = rand::thread_rng();
    let mut elimination_order: Vec<String> = Vec::new();

    while !is_game_over(&game) && game.turn < MAX_TURNS {
        // Get random reasonable moves for each alive snake
        let moves: Vec<(String, Move)> = game
            .random_reasonable_move_for_each_snake(&mut rng)
            .collect();

        // Apply the moves
        game = apply_turn(game, &moves);
        game.turn += 1;

        // Track newly eliminated snakes
        for snake in &game.board.snakes {
            if snake.health <= 0 && !elimination_order.contains(&snake.id) {
                elimination_order.push(snake.id.clone());
            }
        }
    }

    // Build placements: last eliminated = winner (placement 1)
    // Snakes still alive at the end go first
    let mut placements: Vec<String> = game
        .board
        .snakes
        .iter()
        .filter(|s| s.health > 0)
        .map(|s| s.id.clone())
        .collect();

    // Then add eliminated snakes in reverse order (last eliminated = better placement)
    elimination_order.reverse();
    placements.extend(elimination_order);

    GameResult {
        placements,
        final_turn: game.turn,
    }
}

/// Check if the game is over (1 or fewer snakes alive)
fn is_game_over(game: &Game) -> bool {
    let alive_count = game.board.snakes.iter().filter(|s| s.health > 0).count();
    alive_count <= 1
}

/// Apply a single turn: move snakes, reduce health, feed, eliminate
fn apply_turn(mut game: Game, moves: &[(String, Move)]) -> Game {
    // 1. Move snakes
    for snake in &mut game.board.snakes {
        if snake.health <= 0 {
            continue;
        }

        // Find the move for this snake
        let snake_move = moves
            .iter()
            .find(|(id, _)| id == &snake.id)
            .map(|(_, m)| *m)
            .unwrap_or(Move::Up);

        // Calculate new head position
        let new_head = snake.head.add_vec(snake_move.to_vector());

        // Move: add new head, remove tail
        snake.body.push_front(new_head);
        snake.body.pop_back();
        snake.head = new_head;
    }

    // 2. Reduce health
    for snake in &mut game.board.snakes {
        if snake.health > 0 {
            snake.health -= 1;
        }
    }

    // 3. Feed snakes (before elimination check)
    let mut eaten_food = Vec::new();
    for snake in &mut game.board.snakes {
        if snake.health <= 0 {
            continue;
        }

        // Check if head is on food
        if let Some(food_idx) = game.board.food.iter().position(|f| *f == snake.head) {
            // Eat the food
            eaten_food.push(food_idx);
            snake.health = SNAKE_MAX_HEALTH;
            // Grow by duplicating tail
            if let Some(tail) = snake.body.back().copied() {
                snake.body.push_back(tail);
            }
        }
    }

    // Remove eaten food (in reverse order to preserve indices)
    eaten_food.sort();
    eaten_food.reverse();
    for idx in eaten_food {
        game.board.food.remove(idx);
    }

    // 4. Eliminate snakes
    eliminate_snakes(&mut game);

    // Update "you" to match the board state
    if let Some(you_snake) = game.board.snakes.iter().find(|s| s.id == game.you.id) {
        game.you = you_snake.clone();
    }

    game
}

/// Eliminate snakes that are out of health, out of bounds, or have collided
fn eliminate_snakes(game: &mut Game) {
    let width = game.board.width as i32;
    let height = game.board.height as i32;

    // Collect elimination info first (can't mutate while iterating)
    let mut eliminations: Vec<(String, &'static str)> = Vec::new();

    // Check each snake
    for snake in &game.board.snakes {
        if snake.health <= 0 {
            continue; // Already eliminated
        }

        let head = snake.head;

        // Out of bounds check
        if head.x < 0 || head.x >= width || head.y < 0 || head.y >= height {
            eliminations.push((snake.id.clone(), "wall-collision"));
            continue;
        }

        // Out of health check (should already be 0 if starved)
        if snake.health <= 0 {
            eliminations.push((snake.id.clone(), "out-of-health"));
            continue;
        }

        // Self collision check (head hitting own body, excluding head position)
        let self_collision = snake.body.iter().skip(1).any(|p| *p == head);
        if self_collision {
            eliminations.push((snake.id.clone(), "snake-self-collision"));
            continue;
        }

        // Body collision with other snakes
        let body_collision = game.board.snakes.iter().any(|other| {
            other.id != snake.id
                && other.health > 0
                && other.body.iter().skip(1).any(|p| *p == head)
        });
        if body_collision {
            eliminations.push((snake.id.clone(), "snake-collision"));
            continue;
        }

        // Head-to-head collision (lose if same size or smaller)
        let head_collision = game.board.snakes.iter().any(|other| {
            other.id != snake.id
                && other.health > 0
                && other.head == head
                && snake.body.len() <= other.body.len()
        });
        if head_collision {
            eliminations.push((snake.id.clone(), "head-collision"));
        }
    }

    // Apply eliminations
    for (snake_id, _cause) in eliminations {
        if let Some(snake) = game.board.snakes.iter_mut().find(|s| s.id == snake_id) {
            snake.health = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_spawn_positions() {
        let positions = generate_spawn_positions(11, 11, 4);
        assert_eq!(positions.len(), 4);

        // All positions should be unique
        for (i, p1) in positions.iter().enumerate() {
            for (j, p2) in positions.iter().enumerate() {
                if i != j {
                    assert_ne!(p1, p2, "Positions should be unique");
                }
            }
        }

        // All positions should be on the board
        for pos in &positions {
            assert!(pos.x >= 0 && pos.x < 11);
            assert!(pos.y >= 0 && pos.y < 11);
        }
    }

    #[test]
    fn test_is_game_over() {
        let game = create_test_game(2);
        assert!(!is_game_over(&game));

        let mut game_one_alive = create_test_game(2);
        game_one_alive.board.snakes[0].health = 0;
        assert!(is_game_over(&game_one_alive));
    }

    #[test]
    fn test_run_full_game() {
        // Run multiple games to ensure consistency
        for _ in 0..10 {
            let game = create_test_game(4);
            let result = run_game_with_random_moves(game);

            // Should have placements for all 4 snakes
            assert_eq!(
                result.placements.len(),
                4,
                "All snakes should have placements"
            );

            // All snake IDs should be unique
            let mut ids = result.placements.clone();
            ids.sort();
            ids.dedup();
            assert_eq!(ids.len(), 4, "All placements should be unique snakes");

            // Game should end within MAX_TURNS
            assert!(
                result.final_turn <= MAX_TURNS,
                "Game should end within MAX_TURNS"
            );

            // Game should have progressed at least a few turns
            assert!(
                result.final_turn > 0,
                "Game should have run for at least one turn"
            );
        }
    }

    #[test]
    fn test_apply_turn_movement() {
        let mut game = create_test_game(1);
        game.board.snakes[0].head = Position::new(5, 5);
        game.board.snakes[0].body = VecDeque::from([
            Position::new(5, 5),
            Position::new(5, 4),
            Position::new(5, 3),
        ]);

        let moves = vec![("snake-0".to_string(), Move::Up)];
        let game = apply_turn(game, &moves);

        // Head should have moved up
        assert_eq!(game.board.snakes[0].head, Position::new(5, 6));
        // Body should follow
        assert_eq!(game.board.snakes[0].body[0], Position::new(5, 6));
        assert_eq!(game.board.snakes[0].body[1], Position::new(5, 5));
        assert_eq!(game.board.snakes[0].body[2], Position::new(5, 4));
    }

    #[test]
    fn test_apply_turn_health_decrease() {
        let mut game = create_test_game(1);
        game.board.snakes[0].health = 100;

        let moves = vec![("snake-0".to_string(), Move::Up)];
        let game = apply_turn(game, &moves);

        // Health should decrease by 1
        assert_eq!(game.board.snakes[0].health, 99);
    }

    #[test]
    fn test_apply_turn_eating_food() {
        let mut game = create_test_game(1);
        game.board.snakes[0].head = Position::new(5, 4);
        game.board.snakes[0].body = VecDeque::from([
            Position::new(5, 4),
            Position::new(5, 3),
            Position::new(5, 2),
        ]);
        game.board.snakes[0].health = 50;
        game.board.food = vec![Position::new(5, 5)];

        let moves = vec![("snake-0".to_string(), Move::Up)];
        let game = apply_turn(game, &moves);

        // Health should be restored to max
        assert_eq!(game.board.snakes[0].health, SNAKE_MAX_HEALTH);
        // Snake should have grown
        assert_eq!(game.board.snakes[0].body.len(), 4);
        // Food should be consumed
        assert!(game.board.food.is_empty());
    }

    #[test]
    fn test_wall_collision_elimination() {
        let mut game = create_test_game(1);
        // Position snake at edge, moving into wall
        game.board.snakes[0].head = Position::new(0, 5);
        game.board.snakes[0].body = VecDeque::from([
            Position::new(0, 5),
            Position::new(1, 5),
            Position::new(2, 5),
        ]);

        let moves = vec![("snake-0".to_string(), Move::Left)];
        let game = apply_turn(game, &moves);

        // Snake should be eliminated (health = 0)
        assert_eq!(game.board.snakes[0].health, 0);
    }

    fn create_test_game(num_snakes: usize) -> Game {
        let snakes: Vec<BattleSnake> = (0..num_snakes)
            .map(|i| BattleSnake {
                id: format!("snake-{}", i),
                name: format!("Snake {}", i),
                head: Position::new(i as i32 * 2 + 1, i as i32 * 2 + 1),
                body: VecDeque::from([Position::new(i as i32 * 2 + 1, i as i32 * 2 + 1); 3]),
                health: 100,
                shout: None,
                actual_length: None,
            })
            .collect();

        Game {
            you: snakes[0].clone(),
            board: Board {
                height: 11,
                width: 11,
                food: vec![Position::new(5, 5)],
                snakes,
                hazards: vec![],
            },
            turn: 0,
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
}
