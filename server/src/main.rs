use cja::{
    server::run_server,
    setup::{setup_sentry, setup_tracing},
};
use state::AppState;
use tracing::info;

mod cron;
mod errors;
mod jobs;
mod routes;
mod state;
mod cookies;

mod components {
    pub mod github_auth;
    pub mod page;
}

fn main() -> color_eyre::Result<()> {
    // Initialize Sentry for error tracking
    let _sentry_guard = setup_sentry();

    // Create and run the tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()?
        .block_on(async { run_application().await })
}

async fn run_application() -> cja::Result<()> {
    // Initialize tracing
    setup_tracing("tournaments")?;

    let app_state = AppState::from_env().await?;

    // Spawn application tasks
    info!("Spawning application tasks");
    let futures = spawn_application_tasks(app_state).await?;

    // Wait for all tasks to complete
    futures::future::try_join_all(futures).await?;

    Ok(())
}

/// Spawn all application background tasks
async fn spawn_application_tasks(
    app_state: AppState,
) -> cja::Result<Vec<tokio::task::JoinHandle<cja::Result<()>>>> {
    let mut futures = vec![];

    if is_feature_enabled("SERVER") {
        info!("Server Enabled");
        futures.push(tokio::spawn(run_server(routes::routes(app_state.clone()))));
    } else {
        info!("Server Disabled");
    }

    // Initialize job worker if enabled
    if is_feature_enabled("JOBS") {
        info!("Jobs Enabled");
        futures.push(tokio::spawn(cja::jobs::worker::job_worker(
            app_state.clone(),
            jobs::Jobs,
        )));
    } else {
        info!("Jobs Disabled");
    }

    // Initialize cron worker if enabled
    if is_feature_enabled("CRON") {
        info!("Cron Enabled");
        futures.push(tokio::spawn(cron::run_cron(app_state.clone())));
    } else {
        info!("Cron Disabled");
    }

    info!("All application tasks spawned successfully");
    Ok(futures)
}

/// Check if a feature is enabled based on environment variables
fn is_feature_enabled(feature: &str) -> bool {
    let env_var_name = format!("{}_DISABLED", feature);
    let value = std::env::var(&env_var_name).unwrap_or_else(|_| "false".to_string());

    value != "true"
}
