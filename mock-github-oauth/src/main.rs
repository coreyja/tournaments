use color_eyre::eyre::Context;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mock_github_oauth=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get port from env or default to 8081
    let port: u16 = std::env::var("MOCK_GITHUB_PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse()
        .wrap_err("Invalid MOCK_GITHUB_PORT")?;

    tracing::info!("Starting mock GitHub OAuth server on port {}", port);

    mock_github_oauth::run_server(port).await
}
