# Tournament/Bracket System Plan

## Overview

The Tournament system will allow users to organize competitive events where multiple Battlesnakes compete against each other in a structured bracket format. This document outlines the design and implementation plan for this feature.

## Core Concepts

### Tournament

A Tournament is the primary entity that encapsulates the entire competition.

**Properties:**

- `tournament_id`: Unique identifier (UUID)
- `name`: Name of the tournament
- `description`: Tournament description
- `user_id`: The tournament creator/owner
- `game_type`: Type of game (Standard, Royale, etc.)
- `board_size`: Size of the game board (Small, Medium, Large)
- `registration_status`: Controls who can register snakes ("open", "closed", "owner_only")
- `visibility`: Controls who can view the tournament ("public", "participants_only")
- `status`: Current state of the tournament ("created", "registration", "in_progress", "completed", "canceled")
- `match_style`: How matches are determined ("single_game", "best_of_3", "first_to_3")
- `max_snakes_per_user`: Maximum number of snakes a single user can register
- `required_participants`: Minimum number of snakes required to start the tournament
- `current_round`: The current active round number (0 during registration)
- `created_at`: Timestamp of creation
- `updated_at`: Timestamp of last update

### Tournament Registration

Tracks which snakes are registered for the tournament.

**Properties:**

- `registration_id`: Unique identifier (UUID)
- `tournament_id`: Reference to the tournament
- `battlesnake_id`: Reference to the registered battlesnake
- `user_id`: Reference to the user who registered the snake
- `seed`: Optional seeding value for tournament brackets (lower seeds face higher seeds)
- `registered_at`: Timestamp of registration

### Match

Represents a single match between two or more snakes within the tournament bracket structure.

**Properties:**

- `match_id`: Unique identifier (UUID)
- `tournament_id`: Reference to the tournament
- `round`: The round number in the tournament
- `position`: Position within the round (for bracket structure)
- `status`: Current state of the match ("scheduled", "in_progress", "completed", "canceled")
- `next_match_id`: Reference to the next match in the bracket (winner advances to this match)
- `game_id`: Reference to the game (null if not yet played)
- `winner_id`: Reference to the winning battlesnake (if completed, null otherwise)
- `winner_counts`: JSON map tracking wins for each snake in multi-game matches
- `visual_column`: Horizontal position for bracket visualization
- `visual_row`: Vertical position for bracket visualization
- `created_at`: Timestamp of creation
- `updated_at`: Timestamp of last update

### Match Participant

Represents a snake participating in a match.

**Properties:**

- `match_participant_id`: Unique identifier (UUID)
- `match_id`: Reference to the match
- `battlesnake_id`: Reference to the battlesnake
- `source_match_id`: Reference to the match from which this snake advanced (null for first round)
- `participant_type`: Type of participation ("winner", "loser", "seed", "wildcard")
- `seed_position`: Position in the seeding (for initial round matchups)
- `created_at`: Timestamp of creation

### Match Game

Represents individual games for matches using best-of-N or first-to-N match styles.

**Properties:**

- `match_game_id`: Unique identifier (UUID)
- `match_id`: Reference to the match
- `game_id`: Reference to the actual game
- `game_number`: The game number within the match
- `winner_id`: Reference to the winning battlesnake of this specific game
- `created_at`: Timestamp of creation

## Status Transitions and Validations

### Tournament Status Transitions

- **created** → **registration**: Tournament is open for snake registrations
- **registration** → **in_progress**: Tournament has started, bracket is generated
- **in_progress** → **completed**: All matches are finished, tournament is completed
- **Any Status** → **canceled**: Tournament is canceled (irreversible)

### Match Status Transitions

- **scheduled** → **in_progress**: Match is being played
- **in_progress** → **completed**: Match is finished with a winner
- **scheduled/in_progress** → **canceled**: Match is canceled (walkover/disqualification)

## User Flows

### Tournament Creation Flow:

1. User creates a new tournament with basic settings (name, description, game type, board size)
2. User configures tournament settings (registration_status, visibility, match_style, max_snakes_per_user, required_participants)
3. User submits the form to create the tournament
4. System creates the tournament record with status "created"
5. User is redirected to the tournament management page

### Registration Flow:

1. User views a tournament in "created" or "registration" status
2. If registration is "open" or if user is the owner (for "owner_only"), they can register snakes
3. System checks if user has reached their max_snakes_per_user limit
4. User selects one or more of their snakes to register (up to their limit)
5. System creates registration records for each snake and assigns sequential seed numbers
6. Tournament page updates to show registered snakes

### Tournament Start Flow:

1. Tournament owner views tournament in "registration" status
2. System validates that the required_participants threshold is met
3. Owner clicks "Start Tournament" button
4. System generates bracket structure based on registered snakes and seeding
   - Creates match records for all rounds with appropriate next_match_id connections
   - Sets visual_column and visual_row values for proper bracket rendering
   - Sets tournament.current_round = 1
5. System creates match_participant records for the first round based on seeding
6. Tournament status changes to "in_progress"
7. Tournament page updates to show bracket

### Match Processing Flow:

1. Tournament owner views tournament in "in_progress" status
2. Owner can click "Run Next Round" to start the current round's matches
3. System enqueues jobs for each match in the current round
4. Jobs execute according to match_style:
   - For "single_game": Create and run one game for each match
   - For "best_of_3": Create and run up to 3 games, first to 2 wins
   - For "first_to_3": Create and run up to 5 games, first to 3 wins
5. As games complete:
   - Match records are updated with results and winner information
   - winner_counts is updated for multi-game matches
   - Match status is updated to "completed" when a winner is determined
6. System updates match_participant records for the next round's matches
7. When all matches in the current round are complete:
   - System increments tournament.current_round
   - If no matches exist for the new round, tournament status is set to "completed"
8. Tournament page updates to show current state of the bracket
9. Owner can start the next round if available

### Handling Dropouts/Disqualifications:

1. Tournament admin can remove a snake from an upcoming match
2. System marks the match as completed with the remaining snake as winner
3. Winner advances to the next match automatically
4. For completed matches, admin can override the winner if needed

### Tournament Reset Flow:

1. Tournament owner can reset a tournament if it's in "in_progress" status
2. System deletes all match data and resets tournament to "registration" status
3. Registered snakes are preserved unless explicitly removed
4. Tournament can be restarted from registration phase

## Database Schema

### tournaments Table

```sql
CREATE TABLE IF NOT EXISTS tournaments (
    tournament_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    game_type TEXT NOT NULL,
    board_size TEXT NOT NULL,
    registration_status TEXT NOT NULL DEFAULT 'open',
    visibility TEXT NOT NULL DEFAULT 'public',
    status TEXT NOT NULL DEFAULT 'created',
    match_style TEXT NOT NULL DEFAULT 'single_game',
    max_snakes_per_user INTEGER NOT NULL DEFAULT 1,
    required_participants INTEGER NOT NULL DEFAULT 2,
    current_round INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### tournament_registrations Table

```sql
CREATE TABLE IF NOT EXISTS tournament_registrations (
    registration_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tournament_id UUID NOT NULL REFERENCES tournaments(tournament_id) ON DELETE CASCADE,
    battlesnake_id UUID NOT NULL REFERENCES battlesnakes(battlesnake_id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    seed INTEGER,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(tournament_id, battlesnake_id)
);
```

### tournament_matches Table

```sql
CREATE TABLE IF NOT EXISTS tournament_matches (
    match_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tournament_id UUID NOT NULL REFERENCES tournaments(tournament_id) ON DELETE CASCADE,
    round INTEGER NOT NULL,
    position INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled',
    next_match_id UUID REFERENCES tournament_matches(match_id),
    game_id UUID REFERENCES games(game_id) ON DELETE SET NULL,
    winner_id UUID REFERENCES battlesnakes(battlesnake_id) ON DELETE SET NULL,
    winner_counts JSONB DEFAULT '{}',
    visual_column INTEGER,
    visual_row INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(tournament_id, round, position)
);
```

### match_participants Table

```sql
CREATE TABLE IF NOT EXISTS match_participants (
    match_participant_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    match_id UUID NOT NULL REFERENCES tournament_matches(match_id) ON DELETE CASCADE,
    battlesnake_id UUID REFERENCES battlesnakes(battlesnake_id) ON DELETE SET NULL,
    source_match_id UUID REFERENCES tournament_matches(match_id) ON DELETE SET NULL,
    participant_type TEXT NOT NULL,
    seed_position INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### match_games Table

```sql
CREATE TABLE IF NOT EXISTS match_games (
    match_game_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    match_id UUID NOT NULL REFERENCES tournament_matches(match_id) ON DELETE CASCADE,
    game_id UUID NOT NULL REFERENCES games(game_id) ON DELETE CASCADE,
    game_number INTEGER NOT NULL,
    winner_id UUID REFERENCES battlesnakes(battlesnake_id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(match_id, game_number)
);
```

## API Endpoints

### Tournament Endpoints:

- `GET /tournaments` - List tournaments
- `GET /tournaments/:id` - View tournament details
- `POST /tournaments` - Create a new tournament
- `PUT /tournaments/:id` - Update tournament details
- `DELETE /tournaments/:id` - Delete a tournament
- `POST /tournaments/:id/start` - Start a tournament (generate bracket)
- `POST /tournaments/:id/run-round` - Run the current round of matches
- `POST /tournaments/:id/reset` - Reset tournament to registration phase
- `POST /tournaments/:id/cancel` - Cancel tournament

### Registration Endpoints:

- `POST /tournaments/:id/register` - Register a snake for a tournament
- `DELETE /tournaments/:id/unregister/:snake_id` - Unregister a snake

### Match Endpoints:

- `GET /tournaments/:id/matches` - View all matches in a tournament
- `GET /tournaments/:id/matches/:match_id` - View details of a specific match
- `POST /tournaments/:id/matches/:match_id/override-winner` - Admin override for match winner

## Job System

### Tournament Jobs:

1. **RunTournamentRoundJob**:

   - Triggered when tournament owner starts a round
   - Enqueues individual match jobs for each match in current round
   - Manages the progression of tournament rounds
   - Handles transactional integrity of the round execution

2. **RunMatchJob**:

   - Runs a specific match according to match_style
   - For single_game: Creates and runs one game
   - For best_of_3: Creates and runs up to 3 games, first to 2 wins
   - For first_to_3: Creates and runs up to 5 games, first to 3 wins
   - Updates match status, winner, and winner_counts
   - Updates participant records for the next match
   - Handles failure scenarios with appropriate error handling

3. **UpdateTournamentStatusJob**:
   - Triggered when all matches in a round are complete
   - Checks if tournament is complete or advances to next round
   - Updates tournament.current_round
   - Updates tournament status if all rounds complete
   - Executed within a transaction for data consistency

## Bracket Generation Algorithm

The bracket generation algorithm will handle the following scenarios:

1. **Power of 2 Participants**: Standard single elimination bracket
2. **Non-power of 2**: Incorporate byes in the first round
   - Example: 6 participants would have 2 byes in the first round
   - Byes are assigned based on seeding (top seeds get byes)
3. **Seeding**: Participants are placed according to seed values
   - 1 vs 8, 4 vs 5, 2 vs 7, 3 vs 6 pattern for 8 participants
   - Seeds are assigned based on registration order if not specified

The algorithm will also calculate visual coordinates for bracket rendering:

- `visual_column`: Based on the round (starts at 0, increases by 1 per round)
- `visual_row`: Position within each round, calculated to ensure proper bracket visualization

## Multi-Game Match Determination

For matches with multiple games:

1. **Best of 3**:

   - Run up to 3 games
   - Track wins in winner_counts
   - First snake to reach 2 wins is the match winner

2. **First to 3**:
   - Run up to 5 games
   - Track wins in winner_counts
   - First snake to reach 3 wins is the match winner

## UI Designs

### Tournament List Page

- List of tournaments with filters (all, my tournaments, active, completed)
- Create tournament button
- Each tournament shows name, owner, status, game type, # of snakes

### Tournament Creation Page

- Form with fields for all tournament settings
- Options for max_snakes_per_user setting
- Input for required_participants
- Preview of tournament structure based on expected participants

### Tournament Detail Page

- Tournament info section (name, description, owner, settings)
- Registration section (if in registration phase)
- Current status and progress indicators
- Bracket visualization (if started)
- Controls for tournament owner (start tournament, run round, reset, cancel)
- Match history and results
- Tournament statistics (total games, average game length, etc.)

### Bracket Visualization

- Interactive bracket showing all matches
- Color coding for completed, in-progress, and pending matches
- Visual indicators for winners and advancement paths
- Links to individual match/game details
- Responsive design that works on different screen sizes

## Implementation Strategy

1. Create database migrations for all new tables with proper constraints
2. Implement basic tournament CRUD operations with status validations
3. Implement registration functionality with per-user snake limits
4. Develop bracket generation algorithm with visual positioning
5. Implement multi-game match determination logic
6. Implement the job system for running matches with proper transaction handling
7. Create UI components for tournament management and bracket visualization
8. Add permissions and visibility controls
9. Implement tournament statistics and history
10. Add admin controls for match overrides and tournament management

## Transaction Safety

All critical operations will be executed within database transactions to ensure data integrity:

1. Tournament start: Bracket generation and match creation
2. Match execution: Game creation and result processing
3. Round advancement: Updating matches and tournament status

## Error Handling

The system will handle common error scenarios:

1. **Snake Unavailability**: If a snake API is unreachable during a match
2. **Tournament Prerequisites**: Validation of required_participants before starting
3. **Concurrent Operations**: Preventing race conditions in tournament management
4. **Job Failures**: Retry mechanism for tournament jobs with proper logging

## Future Enhancements

- Double-elimination tournaments
- Round-robin tournaments
- Tournament leaderboards
- Tournament chat/commenting
- Public tournament listings page
- Tournament templates (save and reuse tournament settings)
- Tournament series (multiple linked tournaments)
- Prize/point systems
- ELO/rating system for snakes based on tournament performance
