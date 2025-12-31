# Game Management Specification

This document specifies the Game management system for the Tournaments web application.

## Data Model

### Game

r[game.model.id]
Each game MUST have a unique UUID as its primary identifier.

r[game.model.board_size]
Each game MUST have a board size: Small (7x7), Medium (11x11), or Large (19x19).

r[game.model.board_size.default]
The default board size SHOULD be Medium (11x11).

r[game.model.game_type]
Each game MUST have a game type: Standard, Royale, Constrictor.

r[game.model.game_type.default]
The default game type SHOULD be Standard.

r[game.model.status]
Each game MUST have a status: Waiting, Running, or Finished.

r[game.model.status.initial]
New games MUST start with status Waiting.

r[game.model.timestamps]
Each game MUST have `created_at` and `updated_at` timestamps.

### Game Battlesnakes

r[game.battlesnakes.join]
Games and battlesnakes MUST be linked via a join table (game_battlesnakes).

r[game.battlesnakes.placement]
Each game battlesnake entry MAY have a placement (1st, 2nd, 3rd, etc.) set when the game finishes.

## Battlesnake Constraints

r[game.battlesnakes.min]
A game MUST have at least 1 battlesnake.

r[game.battlesnakes.max]
A game MUST NOT have more than 4 battlesnakes.

r[game.battlesnakes.no_duplicates]
A game MUST NOT contain duplicate battlesnakes.

## Game Creation Flow

r[game.flow.model]
The game creation flow MUST be persisted as a database record (game_flows table).

r[game.flow.user_ownership]
Each game flow MUST be associated with a user.

r[game.flow.state]
The game flow MUST track: board_size, game_type, selected_battlesnake_ids, and optional search_query.

### Flow Initiation

r[game.create.new_route]
The game creation MUST be initiated via GET `/games/new`.

r[game.create.new_auth_required]
The initiation endpoint MUST require authentication.

r[game.create.flow_redirect]
Initiating game creation MUST create a flow and redirect to `/games/flow/{flow_id}`.

### Flow Page

r[game.flow.route]
The game flow page MUST be accessible at `/games/flow/{flow_id}`.

r[game.flow.auth_required]
The game flow page MUST require authentication.

r[game.flow.ownership]
The game flow MUST only be accessible by its owner (404 for others).

r[game.flow.board_size_selector]
The flow page MUST display a board size selector with options: 7x7, 11x11, 19x19.

r[game.flow.game_type_selector]
The flow page MUST display a game type selector with options: Standard, Royale, Constrictor.

r[game.flow.user_snakes]
The flow page MUST display the user's own battlesnakes.

r[game.flow.no_snakes_warning]
If the user has no battlesnakes, the flow MUST display a warning and link to create one.

r[game.flow.selection_counter]
The flow page MUST display how many battlesnakes are selected (X of 4).

r[game.flow.selected_list]
The flow page MUST display a list of currently selected battlesnakes.

r[game.flow.no_selection_warning]
If no battlesnakes are selected, the flow MUST display a warning message.

r[game.flow.create_button_hidden]
The "Create Game" button MUST be hidden when no battlesnakes are selected.

### Battlesnake Selection

r[game.flow.add_snake.route]
Adding a battlesnake MUST be handled via POST `/games/flow/{flow_id}/add-snake/{snake_id}`.

r[game.flow.add_snake.auth_required]
The add snake endpoint MUST require authentication.

r[game.flow.add_snake.max_warning]
If adding a snake when already at 4, a warning flash message MUST be displayed.

r[game.flow.remove_snake.route]
Removing a battlesnake MUST be handled via POST `/games/flow/{flow_id}/remove-snake/{snake_id}`.

r[game.flow.remove_snake.auth_required]
The remove snake endpoint MUST require authentication.

r[game.flow.reset.route]
Resetting all selections MUST be handled via POST `/games/flow/{flow_id}/reset`.

r[game.flow.reset.auth_required]
The reset endpoint MUST require authentication.

r[game.flow.reset.clears_all]
Resetting MUST clear all selected battlesnakes.

### Public Battlesnake Search

r[game.flow.search.route]
Searching for public battlesnakes MUST be handled via GET `/games/flow/{flow_id}/search`.

r[game.flow.search.auth_required]
The search endpoint MUST require authentication.

r[game.flow.search.query_param]
The search query MUST be passed as the `q` query parameter.

r[game.flow.search.public_only]
Search results MUST only include public battlesnakes from other users.

r[game.flow.search.private_hidden]
Private battlesnakes from other users MUST NOT appear in search results.

r[game.flow.search.add_result]
Users MUST be able to add search result battlesnakes to their game.

### Game Creation

r[game.create.route]
Creating the game MUST be handled via POST `/games/flow/{flow_id}/create`.

r[game.create.auth_required]
The create endpoint MUST require authentication.

r[game.create.validation]
The system MUST validate that at least 1 battlesnake is selected.

r[game.create.success_redirect]
On successful creation, the user MUST be redirected to the game details page.

r[game.create.success_flash]
On successful creation, a success flash message MUST be displayed.

r[game.create.flow_cleanup]
On successful creation, the game flow record MUST be deleted.

r[game.create.error_redirect]
On validation error, the user MUST be redirected back to the flow page with an error message.

## Game List

r[game.list.route]
The game list MUST be accessible at `/games`.

r[game.list.auth_required]
The game list MUST require authentication (401 if not logged in).

r[game.list.display_id]
The list MUST display each game's ID.

r[game.list.display_board_size]
The list MUST display each game's board size.

r[game.list.display_game_type]
The list MUST display each game's game type.

r[game.list.display_winner]
The list MUST display the winner's name if the game is finished.

r[game.list.display_user_snakes]
If the current user has battlesnakes participating in a game, the list MUST show their snakes' positions.

r[game.list.display_status]
The list MUST display each game's status.

r[game.list.display_created]
The list MUST display each game's creation date.

r[game.list.view_link]
The list MUST provide a View link for each game.

r[game.list.create_link]
The list MUST provide a link to create a new game.

r[game.list.empty_state]
When no games exist, the list MUST display an empty state message.

r[game.list.sorted]
Games MUST be sorted by creation date, newest first.

## Game Details

r[game.view.route]
The game details MUST be accessible at `/games/{game_id}`.

r[game.view.public]
The game details page MUST be publicly accessible without authentication to allow sharing game replays.

r[game.view.not_found]
Viewing a non-existent game MUST return 404.

r[game.view.display_id]
The details page MUST display the game ID.

r[game.view.display_board_size]
The details page MUST display the board size.

r[game.view.display_game_type]
The details page MUST display the game type.

r[game.view.display_started]
The details page MUST display when the game started running.

r[game.view.results_table]
The details page MUST display a results table with all participating battlesnakes.

r[game.view.results_placement]
For each battlesnake, the results table MUST show placement (1st, 2nd, 3rd, etc.) if available.

r[game.view.results_name]
For each battlesnake, the results table MUST show the snake name.

r[game.view.results_owner]
For each battlesnake, the results table MUST show the owner.
