use cja::cron::{CronRegistry, Worker};

use crate::state::AppState;

fn cron_registry() -> CronRegistry<AppState> {
    CronRegistry::new()
}

pub(crate) async fn run_cron(app_state: AppState) -> cja::Result<()> {
    Ok(Worker::new(app_state, cron_registry()).run().await?)
}
