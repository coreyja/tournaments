use std::time::Duration;

use cja::cron::{CronRegistry, Worker};

use crate::backup;
use crate::state::AppState;

fn cron_registry() -> CronRegistry<AppState> {
    let mut registry = CronRegistry::new();

    // Game backup: runs every 12 hours, exports games from the last 36 hours
    registry.register(
        "game_backup",
        Duration::from_secs(12 * 60 * 60), // 12 hours
        |app_state, _context| Box::pin(async move { backup::run_backup(&app_state).await }),
    );

    registry
}

pub(crate) async fn run_cron(app_state: AppState) -> cja::Result<()> {
    Ok(Worker::new(app_state, cron_registry()).run().await?)
}
