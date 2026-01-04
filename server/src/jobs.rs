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

/// Job to discover games that need backup and enqueue individual backup jobs.
/// Runs as a cron job every hour, checking games from the last 4 hours.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameBackupJob;

#[async_trait::async_trait]
impl Job<AppState> for GameBackupJob {
    const NAME: &'static str = "GameBackupJob";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        crate::backup::run_backup_discovery(&app_state)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
        Ok(())
    }
}

/// Job to backup a single game from the Engine database to GCS.
/// Enqueued by GameBackupJob for each game that needs archiving.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackupSingleGameJob {
    pub engine_game_id: String,
}

#[async_trait::async_trait]
impl Job<AppState> for BackupSingleGameJob {
    const NAME: &'static str = "BackupSingleGameJob";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        crate::backup::backup_single_game(&app_state, &self.engine_game_id)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
        Ok(())
    }
}

cja::impl_job_registry!(AppState, NoopJob, GameRunnerJob, GameBackupJob, BackupSingleGameJob);
