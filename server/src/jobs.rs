use crate::state::AppState;

use cja::jobs::Job;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoopJob;

#[async_trait::async_trait]
impl Job<AppState> for NoopJob {
    const NAME: &'static str = "NoopJob";

    async fn run(&self, _app_state: AppState) -> cja::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameRunnerJob {
    pub game_id: Uuid,
}

#[async_trait::async_trait]
impl Job<AppState> for GameRunnerJob {
    const NAME: &'static str = "GameRunnerJob";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        // Run the game - this will update the status to running, execute the game, and update to finished
        crate::models::game::run_game(&app_state.db, self.game_id).await?;
        Ok(())
    }
}

cja::impl_job_registry!(AppState, NoopJob, GameRunnerJob);
