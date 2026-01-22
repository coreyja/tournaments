use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::game_channels::{GameChannels, TurnNotification};

/// A turn in a game with its frame data
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Turn {
    pub turn_id: Uuid,
    pub game_id: Uuid,
    pub turn_number: i32,
    pub frame_data: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Get all turns for a game, ordered by turn number
pub async fn get_turns_by_game_id(pool: &PgPool, game_id: Uuid) -> cja::Result<Vec<Turn>> {
    let turns = sqlx::query_as::<_, Turn>(
        r#"
        SELECT
            turn_id,
            game_id,
            turn_number,
            frame_data,
            created_at
        FROM turns
        WHERE game_id = $1
        ORDER BY turn_number ASC
        "#,
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
    .wrap_err("Failed to fetch turns from database")?;

    Ok(turns)
}

/// Get turns for a game starting from a specific turn number
/// Used for reconnection catch-up
pub async fn get_turns_from(
    pool: &PgPool,
    game_id: Uuid,
    from_turn: i32,
) -> cja::Result<Vec<Turn>> {
    let turns = sqlx::query_as::<_, Turn>(
        r#"
        SELECT
            turn_id,
            game_id,
            turn_number,
            frame_data,
            created_at
        FROM turns
        WHERE game_id = $1 AND turn_number >= $2
        ORDER BY turn_number ASC
        "#,
    )
    .bind(game_id)
    .bind(from_turn)
    .fetch_all(pool)
    .await
    .wrap_err("Failed to fetch turns from database")?;

    Ok(turns)
}

/// Create a new turn for a game
pub async fn create_turn(
    pool: &PgPool,
    game_id: Uuid,
    turn_number: i32,
    frame_data: Option<serde_json::Value>,
) -> cja::Result<Turn> {
    let turn = sqlx::query_as::<_, Turn>(
        r#"
        INSERT INTO turns (game_id, turn_number, frame_data)
        VALUES ($1, $2, $3)
        RETURNING turn_id, game_id, turn_number, frame_data, created_at
        "#,
    )
    .bind(game_id)
    .bind(turn_number)
    .bind(frame_data)
    .fetch_one(pool)
    .await
    .wrap_err("Failed to create turn")?;

    Ok(turn)
}

/// Create a new turn for a game and notify WebSocket subscribers
pub async fn create_turn_and_notify(
    pool: &PgPool,
    game_channels: &GameChannels,
    game_id: Uuid,
    turn_number: i32,
    frame_data: Option<serde_json::Value>,
) -> cja::Result<Turn> {
    let turn = create_turn(pool, game_id, turn_number, frame_data).await?;

    game_channels
        .notify(TurnNotification {
            game_id,
            turn_number,
        })
        .await;

    Ok(turn)
}

/// Update turn frame data (used after computing game state)
pub async fn update_turn_frame_data(
    pool: &PgPool,
    turn_id: Uuid,
    frame_data: serde_json::Value,
) -> cja::Result<()> {
    sqlx::query!(
        r#"
        UPDATE turns
        SET frame_data = $2
        WHERE turn_id = $1
        "#,
        turn_id,
        frame_data
    )
    .execute(pool)
    .await
    .wrap_err("Failed to update turn frame data")?;

    Ok(())
}

/// A snake's move for a specific turn
#[derive(Debug, Serialize, Deserialize)]
pub struct SnakeTurn {
    pub snake_turn_id: Uuid,
    pub turn_id: Uuid,
    pub game_battlesnake_id: Uuid,
    pub direction: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Create a snake turn record
pub async fn create_snake_turn(
    pool: &PgPool,
    turn_id: Uuid,
    game_battlesnake_id: Uuid,
    direction: &str,
) -> cja::Result<SnakeTurn> {
    let row = sqlx::query!(
        r#"
        INSERT INTO snake_turns (turn_id, game_battlesnake_id, direction)
        VALUES ($1, $2, $3)
        RETURNING snake_turn_id, turn_id, game_battlesnake_id, direction, created_at
        "#,
        turn_id,
        game_battlesnake_id,
        direction
    )
    .fetch_one(pool)
    .await
    .wrap_err("Failed to create snake turn")?;

    Ok(SnakeTurn {
        snake_turn_id: row.snake_turn_id,
        turn_id: row.turn_id,
        game_battlesnake_id: row.game_battlesnake_id,
        direction: row.direction,
        created_at: row.created_at,
    })
}

/// Get all snake turns for a specific turn
pub async fn get_snake_turns_by_turn_id(
    pool: &PgPool,
    turn_id: Uuid,
) -> cja::Result<Vec<SnakeTurn>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            snake_turn_id,
            turn_id,
            game_battlesnake_id,
            direction,
            created_at
        FROM snake_turns
        WHERE turn_id = $1
        "#,
        turn_id
    )
    .fetch_all(pool)
    .await
    .wrap_err("Failed to fetch snake turns")?;

    let turns = rows
        .into_iter()
        .map(|row| SnakeTurn {
            snake_turn_id: row.snake_turn_id,
            turn_id: row.turn_id,
            game_battlesnake_id: row.game_battlesnake_id,
            direction: row.direction,
            created_at: row.created_at,
        })
        .collect();

    Ok(turns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_struct_serialization() {
        let turn = Turn {
            turn_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            game_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            turn_number: 42,
            frame_data: Some(serde_json::json!({"test": "data"})),
            created_at: chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
        };

        let json = serde_json::to_string(&turn).unwrap();
        assert!(json.contains("\"turn_id\":"));
        assert!(json.contains("\"game_id\":"));
        assert!(json.contains("\"turn_number\":42"));
        assert!(json.contains("\"frame_data\":{\"test\":\"data\"}"));
    }

    #[test]
    fn test_turn_struct_deserialization() {
        let json = r#"{
            "turn_id": "550e8400-e29b-41d4-a716-446655440000",
            "game_id": "550e8400-e29b-41d4-a716-446655440001",
            "turn_number": 10,
            "frame_data": null,
            "created_at": "2024-01-01T00:00:00Z"
        }"#;

        let turn: Turn = serde_json::from_str(json).unwrap();
        assert_eq!(turn.turn_number, 10);
        assert!(turn.frame_data.is_none());
    }

    #[test]
    fn test_turn_with_frame_data() {
        let frame_data = serde_json::json!({
            "Turn": 5,
            "Snakes": [{"ID": "snake-1", "Health": 100}],
            "Food": [{"X": 5, "Y": 5}],
            "Hazards": []
        });

        let turn = Turn {
            turn_id: Uuid::new_v4(),
            game_id: Uuid::new_v4(),
            turn_number: 5,
            frame_data: Some(frame_data.clone()),
            created_at: chrono::Utc::now(),
        };

        assert_eq!(turn.frame_data.as_ref().unwrap()["Turn"], 5);
        assert!(turn.frame_data.as_ref().unwrap()["Snakes"].is_array());
    }

    #[test]
    fn test_snake_turn_struct_serialization() {
        let snake_turn = SnakeTurn {
            snake_turn_id: Uuid::new_v4(),
            turn_id: Uuid::new_v4(),
            game_battlesnake_id: Uuid::new_v4(),
            direction: "up".to_string(),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&snake_turn).unwrap();
        assert!(json.contains("\"direction\":\"up\""));
    }

    #[test]
    fn test_snake_turn_directions() {
        for direction in ["up", "down", "left", "right"] {
            let snake_turn = SnakeTurn {
                snake_turn_id: Uuid::new_v4(),
                turn_id: Uuid::new_v4(),
                game_battlesnake_id: Uuid::new_v4(),
                direction: direction.to_string(),
                created_at: chrono::Utc::now(),
            };
            assert_eq!(snake_turn.direction, direction);
        }
    }
}
