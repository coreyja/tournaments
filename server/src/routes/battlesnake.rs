use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use color_eyre::eyre::Context as _;
use maud::html;
use uuid::Uuid;

use crate::{
    components::page_factory::PageFactory,
    errors::{ServerResult, WithStatus},
    models::battlesnake::{self, CreateBattlesnake, UpdateBattlesnake, Visibility},
    models::session,
    routes::auth::{CurrentUser, CurrentUserWithSession},
    state::AppState,
};

// web-app[impl battlesnake.list.route]
// web-app[verify battlesnake.list.route]
// web-app[impl battlesnake.list.auth-required]
// List all battlesnakes for the current user
pub async fn list_battlesnakes(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    page_factory: PageFactory,
) -> ServerResult<impl IntoResponse, StatusCode> {
    // Get all battlesnakes for the current user
    let battlesnakes = battlesnake::get_battlesnakes_by_user_id(&state.db, user.user_id)
        .await
        .wrap_err("Failed to get battlesnakes")?;

    // Use flash from page_factory (already extracted and cleared from DB)
    let flash = page_factory.flash.clone();

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
                    // web-app[impl battlesnake.list.empty-state]
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
                                    th { "Visibility" }
                                    th { "Actions" }
                                }
                            }
                            tbody {
                                @for snake in &battlesnakes {
                                    tr {
                                        // web-app[impl battlesnake.list.display-name]
                                        td { (snake.name) }
                                        td {
                                            a href=(snake.url) target="_blank" { (snake.url) }
                                        }
                                        // web-app[impl battlesnake.list.display-visibility]
                                        td {
                                            @if snake.visibility == Visibility::Public {
                                                span class="badge bg-success text-white" { "Public" }
                                            } @else {
                                                span class="badge bg-secondary text-white" { "Private" }
                                            }
                                        }
                                        td class="actions" {
                                            // web-app[impl battlesnake.list.edit-button]
                                            a href={"/battlesnakes/"(snake.battlesnake_id)"/edit"} class="btn btn-sm btn-primary" { "Edit" }
                                            // web-app[impl battlesnake.list.delete-button]
                                            // web-app[impl battlesnake.delete.confirmation]
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
                    // web-app[impl battlesnake.list.add-button]
                    a href="/battlesnakes/new" class="btn btn-primary" { "Add New Battlesnake" }
                    a href="/me" class="btn btn-secondary" { "Back to Profile" }
                }
            }
        }),
        flash,
    ))
}

// web-app[impl battlesnake.create.form-route]
// web-app[impl battlesnake.create.form-auth-required]
// Show the form to create a new battlesnake
pub async fn new_battlesnake(
    CurrentUser(_): CurrentUser,
    page_factory: PageFactory,
) -> ServerResult<impl IntoResponse, StatusCode> {
    // Use flash from page_factory (already extracted and cleared from DB)
    let flash = page_factory.flash.clone();

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

                // web-app[impl battlesnake.create.fields]
                form action="/battlesnakes" method="post" {
                    // web-app[impl battlesnake.create.name-required]
                    div class="form-group" {
                        label for="name" { "Name" }
                        input type="text" id="name" name="name" class="form-control" required {}
                    }

                    // web-app[impl battlesnake.create.url-required]
                    div class="form-group" {
                        label for="url" { "URL" }
                        input type="url" id="url" name="url" class="form-control" required placeholder="https://your-battlesnake-server.com" {}
                        small class="form-text text-muted" { "The URL of your Battlesnake server" }
                    }

                    // web-app[impl battlesnake.create.visibility-required]
                    div class="form-group" {
                        label for="visibility" { "Visibility" }
                        select id="visibility" name="visibility" class="form-control" required {
                            option value="public" selected { "Public (Available to all users)" }
                            option value="private" { "Private (Only available to you)" }
                        }
                        small class="form-text text-muted" { "Control who can add this snake to games" }
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

// web-app[impl battlesnake.create.post-route]
// web-app[verify battlesnake.create.post-route]
// web-app[impl battlesnake.create.post-auth-required]
// Handle the creation of a new battlesnake
pub async fn create_battlesnake(
    State(state): State<AppState>,
    CurrentUserWithSession { user, session }: CurrentUserWithSession,
    Form(create_data): Form<CreateBattlesnake>,
) -> ServerResult<impl IntoResponse, StatusCode> {
    tracing::info!(
        "create_battlesnake: session_id={}, user_id={}, has_flash={:?}",
        session.session_id,
        user.user_id,
        session.flash_message.is_some()
    );

    // Create the new battlesnake in the database
    let battlesnake_result =
        battlesnake::create_battlesnake(&state.db, user.user_id, create_data.clone()).await;

    match battlesnake_result {
        Ok(_) => {
            // web-app[impl battlesnake.create.success-flash]
            // Flash message for success and redirect
            let updated_session = session::set_flash_message(
                &state.db,
                session.session_id,
                "Battlesnake created successfully!".to_string(),
                session::FLASH_TYPE_SUCCESS,
            )
            .await
            .wrap_err("Failed to set flash message")?;

            tracing::info!(
                "Flash set: session_id={}, flash_message={:?}",
                updated_session.session_id,
                updated_session.flash_message
            );

            // web-app[impl battlesnake.create.success-redirect]
            Ok(Redirect::to("/battlesnakes").into_response())
        }
        Err(err) => {
            // web-app[impl battlesnake.create.duplicate-name-error]
            // Check if it's a name uniqueness error
            if err.to_string().contains("already have a battlesnake named") {
                // Set error flash message
                session::set_flash_message(
                    &state.db,
                    session.session_id,
                    err.to_string(),
                    session::FLASH_TYPE_ERROR,
                )
                .await
                .wrap_err("Failed to set flash message")?;

                // Redirect back to the form
                Ok(Redirect::to("/battlesnakes/new").into_response())
            } else {
                // For other errors, propagate them
                Err(err).wrap_err("Failed to create battlesnake")?
            }
        }
    }
}

// web-app[impl battlesnake.edit.form-route]
// Show the form to edit an existing battlesnake
pub async fn edit_battlesnake(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(battlesnake_id): Path<Uuid>,
    page_factory: PageFactory,
) -> ServerResult<impl IntoResponse, StatusCode> {
    // web-app[impl battlesnake.edit.form-not-found]
    // Get the battlesnake by ID
    let battlesnake = battlesnake::get_battlesnake_by_id(&state.db, battlesnake_id)
        .await
        .wrap_err("Failed to get battlesnake")?
        .ok_or_else(|| "Battlesnake not found".to_string())
        .with_status(StatusCode::NOT_FOUND)?;

    // web-app[impl battlesnake.edit.form-ownership]
    // Check if the battlesnake belongs to the current user
    if battlesnake.user_id != user.user_id {
        return Err("You don't have permission to edit this battlesnake".to_string())
            .with_status(StatusCode::FORBIDDEN);
    }

    // Use flash from page_factory (already extracted and cleared from DB)
    let flash = page_factory.flash.clone();

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

                // web-app[impl battlesnake.edit.form-prefilled]
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

                    div class="form-group" {
                        label for="visibility" { "Visibility" }
                        select id="visibility" name="visibility" class="form-control" required {
                            option value="public" selected=(battlesnake.visibility == Visibility::Public) { "Public (Available to all users)" }
                            option value="private" selected=(battlesnake.visibility == Visibility::Private) { "Private (Only available to you)" }
                        }
                        small class="form-text text-muted" { "Control who can add this snake to games" }
                    }

                    div class="form-group" style="margin-top: 20px;" {
                        button type="submit" class="btn btn-primary" { "Update Battlesnake" }
                        // web-app[impl battlesnake.edit.cancel]
                        a href="/battlesnakes" class="btn btn-secondary" { "Cancel" }
                    }
                }
            }
        }),
        flash,
    ))
}

// web-app[impl battlesnake.edit.post-route]
// web-app[verify battlesnake.edit.post-route]
// Handle the update of an existing battlesnake
pub async fn update_battlesnake(
    State(state): State<AppState>,
    CurrentUserWithSession { user, session }: CurrentUserWithSession,
    Path(battlesnake_id): Path<Uuid>,
    Form(update_data): Form<UpdateBattlesnake>,
) -> ServerResult<impl IntoResponse, StatusCode> {
    // web-app[impl battlesnake.edit.post-ownership]
    // web-app[impl battlesnake.permission.own-only-edit]
    // First check if the battlesnake exists and belongs to the user
    let exists = battlesnake::belongs_to_user(&state.db, battlesnake_id, user.user_id)
        .await
        .wrap_err("Failed to check battlesnake ownership")?;

    if !exists {
        return Err("Battlesnake not found or you don't have permission to update it".to_string())
            .with_status(StatusCode::FORBIDDEN);
    }

    // Update the battlesnake
    let update_result = battlesnake::update_battlesnake(
        &state.db,
        battlesnake_id,
        user.user_id,
        update_data.clone(),
    )
    .await;

    match update_result {
        Ok(_) => {
            // web-app[impl battlesnake.edit.success-flash]
            // Flash message for success and redirect
            session::set_flash_message(
                &state.db,
                session.session_id,
                "Battlesnake updated successfully!".to_string(),
                session::FLASH_TYPE_SUCCESS,
            )
            .await
            .wrap_err("Failed to set flash message")?;

            // web-app[impl battlesnake.edit.success-redirect]
            Ok(Redirect::to("/battlesnakes").into_response())
        }
        Err(err) => {
            // web-app[impl battlesnake.edit.duplicate-name-error]
            // Check if it's a name uniqueness error
            if err.to_string().contains("already have a battlesnake named") {
                // Set error flash message
                session::set_flash_message(
                    &state.db,
                    session.session_id,
                    err.to_string(),
                    session::FLASH_TYPE_ERROR,
                )
                .await
                .wrap_err("Failed to set flash message")?;

                // Redirect back to the edit form
                Ok(Redirect::to(&format!("/battlesnakes/{}/edit", battlesnake_id)).into_response())
            } else {
                // For other errors, propagate them
                Err(err).wrap_err("Failed to update battlesnake")?
            }
        }
    }
}

// web-app[impl battlesnake.delete.route]
// web-app[verify battlesnake.delete.route]
// Handle the deletion of a battlesnake
pub async fn delete_battlesnake(
    State(state): State<AppState>,
    CurrentUserWithSession { user, session }: CurrentUserWithSession,
    Path(battlesnake_id): Path<Uuid>,
) -> ServerResult<impl IntoResponse, StatusCode> {
    // web-app[impl battlesnake.delete.ownership]
    // web-app[impl battlesnake.permission.own-only-delete]
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

    // web-app[impl battlesnake.delete.success-flash]
    // Flash message for success and redirect
    session::set_flash_message(
        &state.db,
        session.session_id,
        "Battlesnake deleted successfully!".to_string(),
        session::FLASH_TYPE_SUCCESS,
    )
    .await
    .wrap_err("Failed to set flash message")?;

    // web-app[impl battlesnake.delete.success-redirect]
    Ok(Redirect::to("/battlesnakes").into_response())
}
