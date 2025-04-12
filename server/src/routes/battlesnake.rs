use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Form,
};
use color_eyre::eyre::Context as _;
use maud::html;
use uuid::Uuid;

use crate::{
    components::page_factory::PageFactory,
    components::flash::Flash,
    cookies::CookieJar,
    errors::{ServerResult, WithStatus},
    models::battlesnake::{self, CreateBattlesnake, UpdateBattlesnake},
    routes::auth::CurrentUser,
    state::AppState,
};

// List all battlesnakes for the current user
pub async fn list_battlesnakes(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    page_factory: PageFactory,
    flash: Flash,
) -> ServerResult<impl IntoResponse, StatusCode> {
    // Get all battlesnakes for the current user
    let battlesnakes = battlesnake::get_battlesnakes_by_user_id(&state.db, user.user_id)
        .await
        .wrap_err("Failed to get battlesnakes")?;

    // Render the battlesnake list page
    Ok(page_factory.create_page_with_flash(
        "Your Battlesnakes".to_string(),
        Box::new(html! {
            div class="container" {
                h1 { "Your Battlesnakes" }

                @if let Some(message) = flash.message() {
                    div class=(flash.class()) {
                        p { (message) }
                    }
                }

                @if battlesnakes.is_empty() {
                    div class="empty-state" {
                        p { "You don't have any battlesnakes yet." }
                    }
                } @else {
                    div class="battlesnakes-list" {
                        table class="table" {
                            thead {
                                tr {
                                    th { "Name" }
                                    th { "URL" }
                                    th { "Actions" }
                                }
                            }
                            tbody {
                                @for snake in &battlesnakes {
                                    tr {
                                        td { (snake.name) }
                                        td { 
                                            a href=(snake.url) target="_blank" { (snake.url) }
                                        }
                                        td class="actions" {
                                            a href={"/battlesnakes/"(snake.battlesnake_id)"/edit"} class="btn btn-sm btn-primary" { "Edit" }
                                            form action={"/battlesnakes/"(snake.battlesnake_id)"/delete"} method="post" style="display: inline;" {
                                                button type="submit" class="btn btn-sm btn-danger" onclick="return confirm('Are you sure you want to delete this battlesnake?');" { "Delete" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div class="actions" style="margin-top: 20px;" {
                    a href="/battlesnakes/new" class="btn btn-primary" { "Add New Battlesnake" }
                    a href="/me" class="btn btn-secondary" { "Back to Profile" }
                }
            }
        }),
        flash,
    ))
}

// Show the form to create a new battlesnake
pub async fn new_battlesnake(
    CurrentUser(_): CurrentUser,
    page_factory: PageFactory,
    flash: Flash,
) -> ServerResult<impl IntoResponse, StatusCode> {
    Ok(page_factory.create_page_with_flash(
        "Add New Battlesnake".to_string(),
        Box::new(html! {
            div class="container" {
                h1 { "Add New Battlesnake" }

                @if let Some(message) = flash.message() {
                    div class=(flash.class()) {
                        p { (message) }
                    }
                }

                form action="/battlesnakes" method="post" {
                    div class="form-group" {
                        label for="name" { "Name" }
                        input type="text" id="name" name="name" class="form-control" required {}
                    }
                    
                    div class="form-group" {
                        label for="url" { "URL" }
                        input type="url" id="url" name="url" class="form-control" required placeholder="https://your-battlesnake-server.com" {}
                        small class="form-text text-muted" { "The URL of your Battlesnake server" }
                    }
                    
                    div class="form-group" style="margin-top: 20px;" {
                        button type="submit" class="btn btn-primary" { "Create Battlesnake" }
                        a href="/battlesnakes" class="btn btn-secondary" { "Cancel" }
                    }
                }
            }
        }),
        flash,
    ))
}

// Handle the creation of a new battlesnake
pub async fn create_battlesnake(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    cookie_jar: CookieJar,
    Form(create_data): Form<CreateBattlesnake>,
) -> ServerResult<impl IntoResponse, StatusCode> {
    // Create the new battlesnake in the database
    battlesnake::create_battlesnake(&state.db, user.user_id, create_data)
        .await
        .wrap_err("Failed to create battlesnake")?;

    // Flash message for success and redirect
    let (message, flash_type) = state.flasher.success("Battlesnake created successfully!".to_string());
    Flash::add(&cookie_jar, message, flash_type);

    Ok(Redirect::to("/battlesnakes").into_response())
}

// Show the form to edit an existing battlesnake
pub async fn edit_battlesnake(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(battlesnake_id): Path<Uuid>,
    page_factory: PageFactory,
    flash: Flash,
) -> ServerResult<impl IntoResponse, StatusCode> {
    // Get the battlesnake by ID
    let battlesnake = battlesnake::get_battlesnake_by_id(&state.db, battlesnake_id)
        .await
        .wrap_err("Failed to get battlesnake")?
        .ok_or_else(|| "Battlesnake not found".to_string())
        .with_status(StatusCode::NOT_FOUND)?;

    // Check if the battlesnake belongs to the current user
    if battlesnake.user_id != user.user_id {
        return Err("You don't have permission to edit this battlesnake".to_string())
            .with_status(StatusCode::FORBIDDEN);
    }

    Ok(page_factory.create_page_with_flash(
        format!("Edit Battlesnake: {}", battlesnake.name),
        Box::new(html! {
            div class="container" {
                h1 { "Edit Battlesnake: " (battlesnake.name) }

                @if let Some(message) = flash.message() {
                    div class=(flash.class()) {
                        p { (message) }
                    }
                }

                form action={"/battlesnakes/"(battlesnake_id)"/update"} method="post" {
                    div class="form-group" {
                        label for="name" { "Name" }
                        input type="text" id="name" name="name" class="form-control" required value=(battlesnake.name) {}
                    }
                    
                    div class="form-group" {
                        label for="url" { "URL" }
                        input type="url" id="url" name="url" class="form-control" required value=(battlesnake.url) {}
                        small class="form-text text-muted" { "The URL of your Battlesnake server" }
                    }
                    
                    div class="form-group" style="margin-top: 20px;" {
                        button type="submit" class="btn btn-primary" { "Update Battlesnake" }
                        a href="/battlesnakes" class="btn btn-secondary" { "Cancel" }
                    }
                }
            }
        }),
        flash,
    ))
}

// Handle the update of an existing battlesnake
pub async fn update_battlesnake(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(battlesnake_id): Path<Uuid>,
    cookie_jar: CookieJar,
    Form(update_data): Form<UpdateBattlesnake>,
) -> ServerResult<impl IntoResponse, StatusCode> {
    // First check if the battlesnake exists and belongs to the user
    let exists = battlesnake::belongs_to_user(&state.db, battlesnake_id, user.user_id)
        .await
        .wrap_err("Failed to check battlesnake ownership")?;

    if !exists {
        return Err("Battlesnake not found or you don't have permission to update it".to_string())
            .with_status(StatusCode::FORBIDDEN);
    }

    // Update the battlesnake
    battlesnake::update_battlesnake(&state.db, battlesnake_id, user.user_id, update_data)
        .await
        .wrap_err("Failed to update battlesnake")?;

    // Flash message for success and redirect
    let (message, flash_type) = state.flasher.success("Battlesnake updated successfully!".to_string());
    Flash::add(&cookie_jar, message, flash_type);

    Ok(Redirect::to("/battlesnakes").into_response())
}

// Handle the deletion of a battlesnake
pub async fn delete_battlesnake(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(battlesnake_id): Path<Uuid>,
    cookie_jar: CookieJar,
) -> ServerResult<impl IntoResponse, StatusCode> {
    // First check if the battlesnake exists and belongs to the user
    let exists = battlesnake::belongs_to_user(&state.db, battlesnake_id, user.user_id)
        .await
        .wrap_err("Failed to check battlesnake ownership")?;

    if !exists {
        return Err("Battlesnake not found or you don't have permission to delete it".to_string())
            .with_status(StatusCode::FORBIDDEN);
    }

    // Delete the battlesnake
    battlesnake::delete_battlesnake(&state.db, battlesnake_id, user.user_id)
        .await
        .wrap_err("Failed to delete battlesnake")?;

    // Flash message for success and redirect
    let (message, flash_type) = state.flasher.success("Battlesnake deleted successfully!".to_string());
    Flash::add(&cookie_jar, message, flash_type);

    Ok(Redirect::to("/battlesnakes").into_response())
} 
