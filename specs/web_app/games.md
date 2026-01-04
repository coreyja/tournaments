# Game Management Specification

This document specifies the Game management system for the Tournaments web application.

## Data Model

### Game

r[game.model.id]
Each game MUST have a unique UUID as its primary identifier.

r[game.model.board-size]
Each game MUST have a board size: Small (7x7), Medium (11x11), or Large (19x19).

r[game.model.board-size.default]
The default board size SHOULD be Medium (11x11).

r[game.model.game-type]
Each game MUST have a game type: Standard, Royale, Constrictor.

r[game.model.game-type.default]
The default game type SHOULD be Standard.

r[game.model.status]
Each game MUST have a status: Waiting, Running, or Finished.

r[game.model.status.initial]
New games MUST start with status Waiting.

r[game.model.timestamps]
Each game MUST have `created-at` and `updated-at` timestamps.

### Game Battlesnakes

r[game.battlesnakes.join]
Games and battlesnakes MUST be linked via a join table (game-battlesnakes).

r[game.battlesnakes.placement]
Each game battlesnake entry MAY have a placement (1st, 2nd, 3rd, etc.) set when the game finishes.

## Battlesnake Constraints

r[game.battlesnakes.min]
A game MUST have at least 1 battlesnake.

r[game.battlesnakes.max]
A game MUST NOT have more than 4 battlesnakes.

r[game.battlesnakes.no-duplicates]
A game MUST NOT contain duplicate battlesnakes.

## Game Creation Flow

r[game.flow.model]
The game creation flow MUST be persisted as a database record (game-flows table).

r[game.flow.user-ownership]
Each game flow MUST be associated with a user.

r[game.flow.state]
The game flow MUST track: board-size, game-type, selected-battlesnake-ids, and optional search-query.

### Flow Initiation

r[game.create.new-route]
The game creation MUST be initiated via GET `/games/new`.

r[game.create.new-auth-required]
The initiation endpoint MUST require authentication.

r[game.create.flow-redirect]
Initiating game creation MUST create a flow and redirect to `/games/flow/{flow-id}`.

### Flow Page

r[game.flow.route]
The game flow page MUST be accessible at `/games/flow/{flow-id}`.

r[game.flow.auth-required]
The game flow page MUST require authentication.

r[game.flow.ownership]
The game flow MUST only be accessible by its owner (404 for others).

r[game.flow.board-size-selector]
The flow page MUST display a board size selector with options: 7x7, 11x11, 19x19.

r[game.flow.game-type-selector]
The flow page MUST display a game type selector with options: Standard, Royale, Constrictor.

r[game.flow.user-snakes]
The flow page MUST display the user's own battlesnakes.

r[game.flow.no-snakes-warning]
If the user has no battlesnakes, the flow MUST display a warning and link to create one.

r[game.flow.selection-counter]
The flow page MUST display how many battlesnakes are selected (X of 4).

r[game.flow.selected-list]
The flow page MUST display a list of currently selected battlesnakes.

r[game.flow.no-selection-warning]
If no battlesnakes are selected, the flow MUST display a warning message.

r[game.flow.create-button-hidden]
The "Create Game" button MUST be hidden when no battlesnakes are selected.

### Battlesnake Selection

r[game.flow.add-snake.route]
Adding a battlesnake MUST be handled via POST `/games/flow/{flow-id}/add-snake/{snake-id}`.

r[game.flow.add-snake.auth-required]
The add snake endpoint MUST require authentication.

r[game.flow.add-snake.max-warning]
If adding a snake when already at 4, a warning flash message MUST be displayed.

r[game.flow.remove-snake.route]
Removing a battlesnake MUST be handled via POST `/games/flow/{flow-id}/remove-snake/{snake-id}`.

r[game.flow.remove-snake.auth-required]
The remove snake endpoint MUST require authentication.

r[game.flow.reset.route]
Resetting all selections MUST be handled via POST `/games/flow/{flow-id}/reset`.

r[game.flow.reset.auth-required]
The reset endpoint MUST require authentication.

r[game.flow.reset.clears-all]
Resetting MUST clear all selected battlesnakes.

### Public Battlesnake Search

r[game.flow.search.route]
Searching for public battlesnakes MUST be handled via GET `/games/flow/{flow-id}/search`.

r[game.flow.search.auth-required]
The search endpoint MUST require authentication.

r[game.flow.search.query-param]
The search query MUST be passed as the `q` query parameter.

r[game.flow.search.public-only]
Search results MUST only include public battlesnakes from other users.

r[game.flow.search.private-hidden]
Private battlesnakes from other users MUST NOT appear in search results.

r[game.flow.search.add-result]
Users MUST be able to add search result battlesnakes to their game.

### Game Creation

r[game.create.route]
Creating the game MUST be handled via POST `/games/flow/{flow-id}/create`.

r[game.create.auth-required]
The create endpoint MUST require authentication.

r[game.create.validation]
The system MUST validate that at least 1 battlesnake is selected.

r[game.create.success-redirect]
On successful creation, the user MUST be redirected to the game details page.

r[game.create.success-flash]
On successful creation, a success flash message MUST be displayed.

r[game.create.flow-cleanup]
On successful creation, the game flow record MUST be deleted.

r[game.create.error-redirect]
On validation error, the user MUST be redirected back to the flow page with an error message.

## Game List

r[game.list.route]
The game list MUST be accessible at `/games`.

r[game.list.auth-required]
The game list MUST require authentication (401 if not logged in).

r[game.list.display-id]
The list MUST display each game's ID.

r[game.list.display-board-size]
The list MUST display each game's board size.

r[game.list.display-game-type]
The list MUST display each game's game type.

r[game.list.display-winner]
The list MUST display the winner's name if the game is finished.

r[game.list.display-user-snakes]
If the current user has battlesnakes participating in a game, the list MUST show their snakes' positions.

r[game.list.display-status]
The list MUST display each game's status.

r[game.list.display-created]
The list MUST display each game's creation date.

r[game.list.view-link]
The list MUST provide a View link for each game.

r[game.list.create-link]
The list MUST provide a link to create a new game.

r[game.list.empty-state]
When no games exist, the list MUST display an empty state message.

r[game.list.sorted]
Games MUST be sorted by creation date, newest first.

## Game Details

r[game.view.route]
The game details MUST be accessible at `/games/{game-id}`.

r[game.view.public]
The game details page MUST be publicly accessible without authentication to allow sharing game replays.

r[game.view.not-found]
Viewing a non-existent game MUST return 404.

r[game.view.display-id]
The details page MUST display the game ID.

r[game.view.display-board-size]
The details page MUST display the board size.

r[game.view.display-game-type]
The details page MUST display the game type.

r[game.view.display-started]
The details page MUST display when the game started running.

r[game.view.results-table]
The details page MUST display a results table with all participating battlesnakes.

r[game.view.results-placement]
For each battlesnake, the results table MUST show placement (1st, 2nd, 3rd, etc.) if available.

r[game.view.results-name]
For each battlesnake, the results table MUST show the snake name.

r[game.view.results-owner]
For each battlesnake, the results table MUST show the owner.
