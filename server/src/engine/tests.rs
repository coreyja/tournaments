use super::*;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_game_initialization() {
    let engine = StandardEngine::new();
    let snake_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

    let state = engine.initialize_game(snake_ids.clone(), 11, 11);

    assert_eq!(state.board.width, 11);
    assert_eq!(state.board.height, 11);
    assert_eq!(state.snakes.len(), 2);
    assert_eq!(state.turn, 0);

    // All snakes should be alive initially
    for snake_id in &snake_ids {
        let snake = state.snakes.get(snake_id).unwrap();
        assert!(snake.is_alive);
        assert_eq!(snake.health, 100);
        assert_eq!(snake.body.len(), 3); // Initial length
    }

    // Should have some initial food
    assert!(!state.food.is_empty());
}

#[test]
fn test_snake_movement() {
    let engine = StandardEngine::new();
    let snake_id = Uuid::new_v4();
    let snake_ids = vec![snake_id];

    let mut state = engine.initialize_game(snake_ids, 11, 11);

    // Get initial position
    let initial_head = *state.snakes.get(&snake_id).unwrap().get_head();

    // Move the snake up
    let mut moves = HashMap::new();
    moves.insert(snake_id, Direction::Up);

    let events = engine.process_turn(&mut state, moves);

    // Check snake moved
    let new_head = state.snakes.get(&snake_id).unwrap().get_head();
    assert_eq!(new_head.x, initial_head.x);
    assert_eq!(new_head.y, initial_head.y + 1);

    // Check events
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SnakeMoved { .. }))
    );

    // Turn should increment
    assert_eq!(state.turn, 1);
}

#[test]
fn test_wall_collision() {
    let engine = StandardEngine::new();
    let snake_id = Uuid::new_v4();

    let mut state = GameState::new(11, 11);
    // Place snake at top edge
    let snake = Snake::new(snake_id, Coordinate { x: 5, y: 10 });
    state.snakes.insert(snake_id, snake);

    // Move into wall (up from y=10 in 11x11 board)
    let mut moves = HashMap::new();
    moves.insert(snake_id, Direction::Up);

    let events = engine.process_turn(&mut state, moves);

    // Snake should die from wall collision
    let snake = state.snakes.get(&snake_id).unwrap();
    assert!(!snake.is_alive);

    // Check death event
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::SnakeDied {
            cause: DeathCause::WallCollision,
            ..
        }
    )));
}

#[test]
fn test_food_consumption() {
    let engine = StandardEngine::new();
    let snake_id = Uuid::new_v4();

    let mut state = GameState::new(11, 11);

    // Place snake
    let snake = Snake::new(snake_id, Coordinate { x: 5, y: 5 });
    state.snakes.insert(snake_id, snake);

    // Place food where snake will move
    state.food.push(Food {
        position: Coordinate { x: 5, y: 6 },
    });

    let initial_length = state.snakes.get(&snake_id).unwrap().body.len();

    // Move snake to food
    let mut moves = HashMap::new();
    moves.insert(snake_id, Direction::Up);

    let events = engine.process_turn(&mut state, moves);

    // Snake should grow
    let snake = state.snakes.get(&snake_id).unwrap();
    assert_eq!(snake.body.len(), initial_length + 1);
    assert_eq!(snake.health, 100); // Health restored

    // Food should be eaten
    assert!(
        !state
            .food
            .iter()
            .any(|f| f.position == Coordinate { x: 5, y: 6 })
    );

    // Check food eaten event
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SnakeAteFood { .. }))
    );
}

#[test]
fn test_game_over_detection() {
    let engine = StandardEngine::new();
    let snake_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

    let mut state = engine.initialize_game(snake_ids.clone(), 11, 11);

    // Kill one snake - game should not be over with 2 alive
    state.snakes.get_mut(&snake_ids[0]).unwrap().is_alive = false;
    assert!(!engine.is_game_over(&state));

    // Kill second snake - game should be over with 1 alive
    state.snakes.get_mut(&snake_ids[1]).unwrap().is_alive = false;
    assert!(engine.is_game_over(&state));

    // Kill last snake - game should still be over
    state.snakes.get_mut(&snake_ids[2]).unwrap().is_alive = false;
    assert!(engine.is_game_over(&state));
    assert_eq!(engine.get_winner(&state), None); // No winner if all dead
}

#[test]
fn test_winner_detection() {
    let engine = StandardEngine::new();
    let winner_id = Uuid::new_v4();
    let loser_id = Uuid::new_v4();
    let snake_ids = vec![winner_id, loser_id];

    let mut state = engine.initialize_game(snake_ids, 11, 11);

    // Kill one snake
    state.snakes.get_mut(&loser_id).unwrap().is_alive = false;

    assert!(engine.is_game_over(&state));
    assert_eq!(engine.get_winner(&state), Some(winner_id));
}
