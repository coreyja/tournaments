#![allow(dead_code)]

use cja::{
    jobs::worker::DEFAULT_LOCK_TIMEOUT,
    server::run_server,
    setup::{setup_sentry, setup_tracing},
};
use color_eyre::eyre::eyre;
use state::AppState;
use tokio_util::sync::CancellationToken;
use tracing::info;

mod backup;
mod cron;
mod engine;
mod engine_models;
mod errors;
mod flasher;
mod game_channels;
mod github;
mod jobs;
mod models;
mod routes;
mod state;
mod static_assets;

/// Frontend UI components only - do not place backend logic here
mod components {
    pub mod flash;
    pub mod page;
    pub mod page_factory;
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
    // Initialize tracing (returns Eyes shutdown handle if configured)
    let eyes_shutdown_handle = setup_tracing("arent")?;

    let app_state = AppState::from_env().await?;

    // Spawn application tasks
    info!("Spawning application tasks");
    let tasks = spawn_application_tasks(app_state).await?;

    // Wait for any task to complete - they all run forever, so if one exits it's an error
    if !tasks.is_empty() {
        let (name, result) = wait_for_first_task(tasks).await;

        match result {
            Ok(Ok(())) => {
                tracing::error!(task = name, "Task exited unexpectedly");
                return Err(eyre!("Task '{}' exited unexpectedly", name));
            }
            Ok(Err(e)) => {
                tracing::error!(task = name, error = ?e, "Task failed with error");
                return Err(e);
            }
            Err(join_error) => {
                tracing::error!(task = name, error = ?join_error, "Task panicked");
                return Err(eyre!("Task '{}' panicked: {}", name, join_error));
            }
        }
    }

    // Graceful shutdown of Eyes tracing if configured
    if let Some(handle) = eyes_shutdown_handle {
        info!("Shutting down Eyes tracing...");
        if let Err(e) = handle.shutdown().await {
            tracing::warn!("Error shutting down Eyes: {e}");
        }
    }

    Ok(())
}

struct NamedTask {
    name: &'static str,
    handle: tokio::task::JoinHandle<cja::Result<()>>,
}

impl NamedTask {
    fn spawn<F>(name: &'static str, future: F) -> Self
    where
        F: std::future::Future<Output = cja::Result<()>> + Send + 'static,
    {
        Self {
            name,
            handle: tokio::spawn(future),
        }
    }
}

/// Wait for the first task to complete and return its name and result
async fn wait_for_first_task(
    tasks: Vec<NamedTask>,
) -> (
    &'static str,
    Result<cja::Result<()>, tokio::task::JoinError>,
) {
    let (handles, names): (Vec<_>, Vec<_>) = tasks.into_iter().map(|t| (t.handle, t.name)).unzip();

    let (result, index, _remaining) = futures::future::select_all(handles).await;
    (names[index], result)
}

/// Spawn all application background tasks
async fn spawn_application_tasks(app_state: AppState) -> cja::Result<Vec<NamedTask>> {
    let mut tasks = vec![];

    if is_feature_enabled("SERVER") {
        info!("Server Enabled");
        tasks.push(NamedTask::spawn(
            "server",
            run_server(routes::routes(app_state.clone())),
        ));
    } else {
        info!("Server Disabled");
    }

    if is_feature_enabled("JOBS") {
        info!("Jobs Enabled");
        // Allow configuring job poll interval via env var (default 60 seconds)
        let job_poll_interval_secs: u64 = std::env::var("JOB_POLL_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);
        info!("Job poll interval: {} seconds", job_poll_interval_secs);
        tasks.push(NamedTask::spawn(
            "jobs",
            cja::jobs::worker::job_worker(
                app_state.clone(),
                jobs::Jobs,
                std::time::Duration::from_secs(job_poll_interval_secs),
                20,
                CancellationToken::new(),
                DEFAULT_LOCK_TIMEOUT,
            ),
        ));
    } else {
        info!("Jobs Disabled");
    }

    if is_feature_enabled("CRON") {
        info!("Cron Enabled");
        tasks.push(NamedTask::spawn("cron", cron::run_cron(app_state.clone())));
    } else {
        info!("Cron Disabled");
    }

    info!("All application tasks spawned successfully");
    Ok(tasks)
}

/// Check if a feature is enabled based on environment variables
fn is_feature_enabled(feature: &str) -> bool {
    let env_var_name = format!("{}_DISABLED", feature);
    let value = std::env::var(&env_var_name).unwrap_or_else(|_| "false".to_string());

    value != "true"
}
