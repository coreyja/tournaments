# Battlesnake Management Specification

This document specifies the Battlesnake management system for the Tournaments web application.

## Data Model

r[battlesnake.model.id]
Each battlesnake MUST have a unique UUID as its primary identifier.

r[battlesnake.model.user_id]
Each battlesnake MUST be associated with a user (owner) via user_id.

r[battlesnake.model.name]
Each battlesnake MUST have a name (non-empty string).

r[battlesnake.model.url]
Each battlesnake MUST have a URL pointing to the Battlesnake API server.

r[battlesnake.model.visibility]
Each battlesnake MUST have a visibility setting of either `public` or `private`.

r[battlesnake.model.visibility.default]
The default visibility for new battlesnakes SHOULD be `public`.

r[battlesnake.model.timestamps]
Each battlesnake MUST have `created_at` and `updated_at` timestamps.

## Name Uniqueness

r[battlesnake.name.unique_per_user]
A user MUST NOT have two battlesnakes with the same name.

r[battlesnake.name.unique_across_users]
Different users MAY have battlesnakes with the same name.

r[battlesnake.name.reuse_after_delete]
After deleting a battlesnake, the user MAY create a new battlesnake with the same name.

## Visibility

r[battlesnake.visibility.public]
Public battlesnakes MUST be discoverable and selectable by other users for games.

r[battlesnake.visibility.private]
Private battlesnakes MUST only be visible and selectable by their owner.

r[battlesnake.visibility.list_own_only]
The battlesnake list page MUST only show battlesnakes owned by the current user, regardless of visibility.

## List View

r[battlesnake.list.route]
The battlesnake list MUST be accessible at `/battlesnakes`.

r[battlesnake.list.auth_required]
The battlesnake list page MUST require authentication (return 401 if not logged in).

r[battlesnake.list.display_name]
The list MUST display each battlesnake's name.

r[battlesnake.list.display_url]
The list MUST display each battlesnake's URL as a clickable link.

r[battlesnake.list.display_visibility]
The list MUST display each battlesnake's visibility with a badge (Public/Private).

r[battlesnake.list.edit_button]
The list MUST provide an Edit button/link for each battlesnake.

r[battlesnake.list.delete_button]
The list MUST provide a Delete button for each battlesnake.

r[battlesnake.list.add_button]
The list MUST provide a button to add a new battlesnake.

r[battlesnake.list.empty_state]
When the user has no battlesnakes, the list MUST display an empty state message.

r[battlesnake.list.sorted]
Battlesnakes MUST be sorted alphabetically by name.

## Create Flow

r[battlesnake.create.form_route]
The create form MUST be accessible at `/battlesnakes/new`.

r[battlesnake.create.form_auth_required]
The create form MUST require authentication.

r[battlesnake.create.post_route]
The create submission MUST be handled via POST to `/battlesnakes`.

r[battlesnake.create.post_auth_required]
The create POST endpoint MUST require authentication.

r[battlesnake.create.fields]
The create form MUST have fields for: name, URL, and visibility.

r[battlesnake.create.name_required]
The name field MUST be required.

r[battlesnake.create.url_required]
The URL field MUST be required.

r[battlesnake.create.visibility_required]
The visibility field MUST be required.

r[battlesnake.create.success_redirect]
On successful creation, the user MUST be redirected to the battlesnake list.

r[battlesnake.create.success_flash]
On successful creation, a success flash message MUST be displayed.

r[battlesnake.create.duplicate_name_error]
If the name is a duplicate, the user MUST be redirected back to the form with an error message.

## Edit Flow

r[battlesnake.edit.form_route]
The edit form MUST be accessible at `/battlesnakes/{id}/edit`.

r[battlesnake.edit.form_auth_required]
The edit form MUST require authentication.

r[battlesnake.edit.form_ownership]
The edit form MUST only be accessible by the battlesnake's owner (403 for others).

r[battlesnake.edit.form_not_found]
The edit form MUST return 404 for non-existent battlesnake IDs.

r[battlesnake.edit.form_prefilled]
The edit form MUST be pre-populated with the current battlesnake values.

r[battlesnake.edit.post_route]
The update submission MUST be handled via POST to `/battlesnakes/{id}/update`.

r[battlesnake.edit.post_auth_required]
The update POST endpoint MUST require authentication.

r[battlesnake.edit.post_ownership]
The update POST MUST verify ownership (403 for non-owners).

r[battlesnake.edit.success_redirect]
On successful update, the user MUST be redirected to the battlesnake list.

r[battlesnake.edit.success_flash]
On successful update, a success flash message MUST be displayed.

r[battlesnake.edit.duplicate_name_error]
If the updated name is a duplicate, the user MUST be redirected back to the edit form with an error message.

r[battlesnake.edit.cancel]
The edit form MUST have a cancel button that returns to the list without saving.

## Delete Flow

r[battlesnake.delete.route]
Deletion MUST be handled via POST to `/battlesnakes/{id}/delete`.

r[battlesnake.delete.auth_required]
The delete endpoint MUST require authentication.

r[battlesnake.delete.ownership]
The delete endpoint MUST verify ownership (403 for non-owners).

r[battlesnake.delete.confirmation]
The UI MUST show a confirmation dialog before deletion.

r[battlesnake.delete.success_redirect]
On successful deletion, the user MUST be redirected to the battlesnake list.

r[battlesnake.delete.success_flash]
On successful deletion, a success flash message MUST be displayed.

r[battlesnake.delete.cancel_preserves]
Dismissing the confirmation dialog MUST NOT delete the battlesnake.

## Permissions

r[battlesnake.permission.own_only_edit]
Users MUST only be able to edit their own battlesnakes.

r[battlesnake.permission.own_only_delete]
Users MUST only be able to delete their own battlesnakes.

r[battlesnake.permission.forbidden_status]
Attempting to access another user's battlesnake for edit/update/delete MUST return 403 Forbidden.
