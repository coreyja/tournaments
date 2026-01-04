use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get};
use maud::html;

use crate::{components::page_factory::PageFactory, errors::ServerResult, state::AppState};

// Include route modules
pub mod auth;
pub mod battlesnake;
pub mod game;
pub mod github_auth;

pub fn routes(app_state: AppState) -> axum::Router {
    axum::Router::new()
        // Public pages
        .route("/", get(root_page))
        // Profile page - requires authentication
        .route("/me", get(profile_page))
        // GitHub OAuth routes
        .route("/auth/github", get(github_auth::github_auth))
        .route(
            "/auth/github/callback",
            get(github_auth::github_auth_callback),
        )
        .route("/auth/logout", get(github_auth::logout))
        // Battlesnake routes
        .route("/battlesnakes", get(battlesnake::list_battlesnakes))
        .route("/battlesnakes/new", get(battlesnake::new_battlesnake))
        .route(
            "/battlesnakes",
            axum::routing::post(battlesnake::create_battlesnake),
        )
        .route(
            "/battlesnakes/{id}/edit",
            get(battlesnake::edit_battlesnake),
        )
        .route(
            "/battlesnakes/{id}/update",
            axum::routing::post(battlesnake::update_battlesnake),
        )
        .route(
            "/battlesnakes/{id}/delete",
            axum::routing::post(battlesnake::delete_battlesnake),
        )
        // Game routes
        .route("/games", get(game::list_games))
        .route("/games/new", get(game::new_game))
        .route("/games/{id}", get(game::view_game))
        .route("/games/flow/{id}", get(game::show_game_flow))
        .route(
            "/games/flow/{id}/reset",
            axum::routing::post(game::reset_snake_selections),
        )
        .route(
            "/games/flow/{id}/create",
            axum::routing::post(game::create_game),
        )
        .route(
            "/games/flow/{id}/add-snake/{snake_id}",
            axum::routing::post(game::add_battlesnake),
        )
        .route(
            "/games/flow/{id}/remove-snake/{snake_id}",
            axum::routing::post(game::remove_battlesnake),
        )
        .route("/games/flow/{id}/search", get(game::search_battlesnakes))
        // Static files
        .route(
            "/static/{*path}",
            get(crate::static_assets::serve_static_file),
        )
        // Add trace layer for debugging
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(app_state)
}

// web-app[impl homepage.route]
// web-app[verify homepage.route]
// web-app[impl homepage.public]
async fn root_page(
    _: State<AppState>,
    auth::OptionalUser(user): auth::OptionalUser,
    page_factory: PageFactory,
) -> ServerResult<impl IntoResponse, StatusCode> {
    Ok(page_factory.create_page(
        "Home".to_string(),
        Box::new(html! {
            div {
                @if let Some(user) = user {
                    div class="user-info" {
                        // web-app[impl homepage.auth.avatar]
                        img src=(user.github_avatar_url.unwrap_or_default()) alt="Avatar" style="width: 50px; height: 50px; border-radius: 50%;" {}
                        // web-app[impl homepage.auth.welcome]
                        p { "Welcome, " (user.github_login) "!" }
                        @if let Some(name) = user.github_name {
                            p { "Name: " (name) }
                        }
                        div class="user-actions" style="margin-top: 10px;" {
                            // web-app[impl homepage.auth.profile-link]
                            a href="/me" class="btn btn-primary" { "Profile" }
                            // web-app[impl homepage.auth.battlesnakes-link]
                            a href="/battlesnakes" class="btn btn-primary" { "Battlesnakes" }
                            // web-app[impl homepage.auth.logout-link]
                            // web-app[impl homepage.auth.no-login-link]
                            a href="/auth/logout" class="btn btn-secondary" { "Logout" }
                        }
                    }
                } @else {
                    div class="login" {
                        // web-app[impl homepage.unauth.message]
                        p { "You are not logged in." }
                        // web-app[impl homepage.unauth.login-link]
                        a href="/auth/github" { "Login with GitHub" }
                    }
                }
                div class="content" style="margin-top: 20px;" {
                    h1 { "Hello, world!" }
                    p { "Welcome to the Tournaments application!" }
                }
            }
        }),
    ))
}

// web-app[impl profile.route]
// web-app[verify profile.route]
// web-app[impl profile.auth-required]
// web-app[impl profile.title]
/// Profile page that requires authentication
async fn profile_page(
    auth::CurrentUser(user): auth::CurrentUser,
    page_factory: PageFactory,
) -> ServerResult<impl IntoResponse, StatusCode> {
    Ok(page_factory.create_page(
        "My Profile".to_string(),
        Box::new(html! {
            div {
                h1 { "My Profile" }

                div class="profile-card" style="border: 1px solid #ddd; border-radius: 8px; padding: 20px; margin: 20px 0; max-width: 600px;" {
                    div class="profile-header" style="display: flex; align-items: center; margin-bottom: 20px;" {
                        // web-app[impl profile.display.avatar]
                        img src=(user.github_avatar_url.unwrap_or_default()) alt="Avatar" style="width: 100px; height: 100px; border-radius: 50%; margin-right: 20px;" {}

                        div {
                            // web-app[impl profile.display.login]
                            h2 style="margin: 0 0 10px 0;" { (user.github_login) }
                            // web-app[impl profile.display.name]
                            @if let Some(name) = user.github_name.as_ref() {
                                p style="margin: 0; color: #666;" { (name) }
                            }
                            // web-app[impl profile.display.email]
                            @if let Some(email) = user.github_email.as_ref() {
                                p style="margin: 0; color: #666;" { (email) }
                            }
                        }
                    }

                    div class="profile-details" {
                        h3 { "Account Details" }
                        p { "GitHub ID: " (user.external_github_id) }
                        p { "Account created: " (user.created_at.format("%Y-%m-%d %H:%M:%S")) }
                        p { "Last updated: " (user.updated_at.format("%Y-%m-%d %H:%M:%S")) }
                    }

                    div class="profile-actions" style="margin-top: 20px;" {
                        // web-app[impl profile.battlesnakes.summary]
                        h3 { "Your Battlesnakes" }
                        p { "Manage your Battlesnake collection for tournaments." }
                        // web-app[impl profile.nav.battlesnakes]
                        a href="/battlesnakes" class="btn btn-primary" { "Manage Battlesnakes" }

                        h3 class="mt-4" { "Games" }
                        p { "Create and view games with your Battlesnakes." }
                        div {
                            // web-app[impl profile.nav.create-game]
                            a href="/games/new" class="btn btn-primary" { "Create New Game" }
                            // web-app[impl profile.nav.view-games]
                            a href="/games" class="btn btn-secondary ms-2" { "View All Games" }
                        }
                    }
                }

                div class="nav" style="margin-top: 20px;" {
                    // web-app[impl profile.nav.home]
                    a href="/" { "Back to Home" }
                    span { " | " }
                    a href="/auth/logout" { "Logout" }
                }
            }
        }),
    ))
}
