pub mod routes;
pub mod state;
pub mod types;

use axum::{routing::{get, post}, Router};
use state::MockOAuthState;

pub use types::MockUserConfig;

/// Create the mock OAuth server router
pub fn create_router() -> Router {
    let state = MockOAuthState::new();

    Router::new()
        // OAuth endpoints (mimic GitHub)
        .route("/login/oauth/authorize", get(routes::authorize))
        .route("/login/oauth/access_token", post(routes::access_token))
        .route("/user", get(routes::get_user))
        // Admin endpoint for test control
        .route("/_admin/set-user-for-state", post(routes::set_user_for_state))
        .with_state(state)
}

/// Run the mock OAuth server on the specified port
pub async fn run_server(port: u16) -> color_eyre::Result<()> {
    let app = create_router();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Mock GitHub OAuth server running on port {}", port);
    axum::serve(listener, app).await?;
    Ok(())
}
