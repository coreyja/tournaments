use axum::routing::get;
use maud::html;
use reqwest::StatusCode;

use crate::{components::page::Page, errors::ServerResult, state::AppState};

pub fn routes(app_state: AppState) -> axum::Router {
    axum::Router::new()
        // Public pages
        .route("/", get(root_page))
        // Add trace layer for debugging
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(app_state)
}

async fn root_page() -> ServerResult<Page, StatusCode> {
    Ok(Page::new(
        "Home".to_string(),
        Box::new(html! {
            div {
                "Hello, world!"
            }
        }),
    ))
}
