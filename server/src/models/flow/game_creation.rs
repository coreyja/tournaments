use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;

use crate::models::battlesnake::{self, Battlesnake};
use crate::models::game::{self, CreateGameWithSnakes, GameBoardSize, GameType};

// Flow model for the game creation process
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameCreationFlow {
    pub flow_id: Uuid,
    pub board_size: GameBoardSize,
    pub game_type: GameType,
    pub selected_battlesnake_ids: HashSet<Uuid>,
    pub search_query: Option<String>,
    pub user_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl GameCreationFlow {
    // Create a new flow for a user
    pub async fn create_for_user(pool: &PgPool, user_id: Uuid) -> cja::Result<Self> {
        // Insert a new flow with default values
        let flow = sqlx::query_as!(
            GameCreationFlowRaw,
            r#"
            INSERT INTO game_flows (
                user_id,
                board_size,
                game_type,
                selected_battlesnakes,
                search_query
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                flow_id,
                board_size,
                game_type,
                selected_battlesnakes,
                search_query,
                user_id,
                created_at,
                updated_at
            "#,
            user_id,
            GameBoardSize::Medium.as_str(),
            GameType::Standard.as_str(),
            &Vec::<Uuid>::new(),
            None::<String>
        )
        .fetch_one(pool)
        .await
        .wrap_err("Failed to create game flow")?;

        Ok(flow.into())
    }

    // Get a flow by ID, ensuring it belongs to the user
    pub async fn get_by_id(
        pool: &PgPool,
        flow_id: Uuid,
        user_id: Uuid,
    ) -> cja::Result<Option<Self>> {
        let flow = sqlx::query_as!(
            GameCreationFlowRaw,
            r#"
            SELECT
                flow_id,
                board_size,
                game_type,
                selected_battlesnakes,
                search_query,
                user_id,
                created_at,
                updated_at
            FROM game_flows
            WHERE flow_id = $1 AND user_id = $2
            "#,
            flow_id,
            user_id
        )
        .fetch_optional(pool)
        .await
        .wrap_err("Failed to get game flow")?;

        Ok(flow.map(|f| f.into()))
    }

    // Update the flow with new values
    pub async fn update(&self, pool: &PgPool) -> cja::Result<Self> {
        // Convert selected_battlesnake_ids to a Vec for SQL
        let selected_battlesnakes: Vec<Uuid> =
            self.selected_battlesnake_ids.iter().copied().collect();

        let flow = sqlx::query_as!(
            GameCreationFlowRaw,
            r#"
            UPDATE game_flows
            SET
                board_size = $1,
                game_type = $2,
                selected_battlesnakes = $3,
                search_query = $4
            WHERE flow_id = $5 AND user_id = $6
            RETURNING
                flow_id,
                board_size,
                game_type,
                selected_battlesnakes,
                search_query,
                user_id,
                created_at,
                updated_at
            "#,
            self.board_size.as_str(),
            self.game_type.as_str(),
            &selected_battlesnakes,
            self.search_query.as_deref(),
            self.flow_id,
            self.user_id
        )
        .fetch_one(pool)
        .await
        .wrap_err("Failed to update game flow")?;

        Ok(flow.into())
    }

    // Delete a flow
    pub async fn delete(pool: &PgPool, flow_id: Uuid, user_id: Uuid) -> cja::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM game_flows
            WHERE flow_id = $1 AND user_id = $2
            "#,
            flow_id,
            user_id
        )
        .execute(pool)
        .await
        .wrap_err("Failed to delete game flow")?;

        Ok(())
    }

    // Add a battlesnake to the selection
    pub fn add_battlesnake(&mut self, battlesnake_id: Uuid) -> bool {
        // Only add if we have fewer than 4 snakes selected and it's not already selected
        if self.selected_battlesnake_ids.len() < 4
            && !self.selected_battlesnake_ids.contains(&battlesnake_id)
        {
            self.selected_battlesnake_ids.insert(battlesnake_id);
            true
        } else {
            false // Already have 4 snakes or snake is already selected
        }
    }

    // Remove a battlesnake from the selection
    pub fn remove_battlesnake(&mut self, battlesnake_id: Uuid) -> bool {
        // Remove the battlesnake if it's in the selection
        self.selected_battlesnake_ids.remove(&battlesnake_id)
    }

    // Check if a battlesnake is selected
    pub fn is_battlesnake_selected(&self, battlesnake_id: &Uuid) -> bool {
        self.selected_battlesnake_ids.contains(battlesnake_id)
    }

    // Get count of selected snakes
    pub fn selected_count(&self) -> usize {
        self.selected_battlesnake_ids.len()
    }

    // Validate the flow state before creating a game
    pub fn validate(&self) -> cja::Result<()> {
        if self.selected_battlesnake_ids.is_empty() {
            return Err(cja::color_eyre::eyre::eyre!(
                "At least one battlesnake is required"
            ));
        }

        if self.selected_battlesnake_ids.len() > 4 {
            return Err(cja::color_eyre::eyre::eyre!(
                "Maximum of 4 battlesnakes allowed"
            ));
        }

        Ok(())
    }

    // Convert the flow to a CreateGameWithSnakes request
    pub fn to_create_game_request(&self) -> cja::Result<CreateGameWithSnakes> {
        self.validate()?;

        Ok(CreateGameWithSnakes {
            board_size: self.board_size,
            game_type: self.game_type,
            battlesnake_ids: self.selected_battlesnake_ids.iter().copied().collect(),
        })
    }

    // Create the game from the flow
    pub async fn create_game(&self, pool: &PgPool) -> cja::Result<Uuid> {
        let create_request = self.to_create_game_request()?;

        let game = game::create_game_with_snakes(pool, create_request)
            .await
            .wrap_err("Failed to create game")?;

        // Enqueue a job to run the game asynchronously
        game::run_game(pool, game.game_id)
            .await
            .wrap_err("Failed to run game")?;

        Ok(game.game_id)
    }

    // Get all battlesnakes for the current user
    pub async fn get_user_battlesnakes(&self, pool: &PgPool) -> cja::Result<Vec<Battlesnake>> {
        battlesnake::get_battlesnakes_by_user_id(pool, self.user_id)
            .await
            .wrap_err("Failed to get user's battlesnakes")
    }

    // Search for public battlesnakes
    pub async fn search_public_battlesnakes(&self, pool: &PgPool) -> cja::Result<Vec<Battlesnake>> {
        if let Some(query) = &self.search_query {
            if query.is_empty() {
                return Ok(Vec::new());
            }

            // Search for public battlesnakes by name (case-insensitive)
            // This SQL query finds public battlesnakes that match the search query
            // and are not owned by the current user
            let battlesnakes = sqlx::query_as!(
                Battlesnake,
                r#"
                SELECT
                    battlesnake_id,
                    user_id,
                    name,
                    url,
                    visibility as "visibility: _",
                    created_at,
                    updated_at
                FROM battlesnakes
                WHERE 
                    visibility = 'public'
                    AND user_id != $1
                    AND name ILIKE $2
                ORDER BY name ASC
                LIMIT 10
                "#,
                self.user_id,
                format!("%{}%", query)
            )
            .fetch_all(pool)
            .await
            .wrap_err("Failed to search public battlesnakes")?;

            Ok(battlesnakes)
        } else {
            Ok(Vec::new())
        }
    }

    // Get details of the selected battlesnakes
    pub async fn get_selected_battlesnakes(&self, pool: &PgPool) -> cja::Result<Vec<Battlesnake>> {
        if self.selected_battlesnake_ids.is_empty() {
            return Ok(Vec::new());
        }

        let ids: Vec<Uuid> = self.selected_battlesnake_ids.iter().copied().collect();

        let battlesnakes = sqlx::query_as!(
            Battlesnake,
            r#"
            SELECT
                battlesnake_id,
                user_id,
                name,
                url,
                visibility as "visibility: _",
                created_at,
                updated_at
            FROM battlesnakes
            WHERE battlesnake_id = ANY($1)
            ORDER BY name ASC
            "#,
            &ids
        )
        .fetch_all(pool)
        .await
        .wrap_err("Failed to get selected battlesnakes")?;

        Ok(battlesnakes)
    }
}

// Raw database model
#[derive(Debug)]
struct GameCreationFlowRaw {
    pub flow_id: Uuid,
    pub board_size: String,
    pub game_type: String,
    pub selected_battlesnakes: Vec<Uuid>,
    pub search_query: Option<String>,
    pub user_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// Convert raw database model to domain model
impl From<GameCreationFlowRaw> for GameCreationFlow {
    fn from(raw: GameCreationFlowRaw) -> Self {
        let board_size =
            std::str::FromStr::from_str(&raw.board_size).unwrap_or(GameBoardSize::Medium);

        let game_type = std::str::FromStr::from_str(&raw.game_type).unwrap_or(GameType::Standard);

        Self {
            flow_id: raw.flow_id,
            board_size,
            game_type,
            selected_battlesnake_ids: raw.selected_battlesnakes.into_iter().collect(),
            search_query: raw.search_query,
            user_id: raw.user_id,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
        }
    }
}
