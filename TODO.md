# Tournament System Implementation TODO

## Database Setup

- [ ] Create migration for `tournaments` table
  - [ ] Add `max_snakes_per_user` field
  - [ ] Add `required_participants` field
  - [ ] Add `current_round` field
- [ ] Create migration for `tournament_registrations` table
  - [ ] Include `seed` field for bracket seeding
- [ ] Create migration for `tournament_matches` table
  - [ ] Include `next_match_id` for bracket structure
  - [ ] Include `game_id` to link with existing Game model
  - [ ] Add `winner_counts` JSON field for multi-game matches
  - [ ] Add `visual_column` and `visual_row` for bracket positioning
- [ ] Create migration for `match_participants` table
  - [ ] Include `seed_position` for proper seeding
- [ ] Create migration for `match_games` table
  - [ ] Add `winner_id` field to track game-specific winners
- [ ] Run migrations

## Models Implementation

- [ ] Create `tournament.rs` model file with basic types

  - [ ] Implement `Tournament` struct
  - [ ] Implement `TournamentStatus` enum with proper transitions
    - [ ] Add "canceled" status
    - [ ] Add status transition validation
  - [ ] Implement `RegistrationStatus` enum
  - [ ] Implement `Visibility` enum
  - [ ] Implement `MatchStyle` enum
  - [ ] Implement CRUD operations for tournaments
  - [ ] Add tournament reset functionality
  - [ ] Add tournament cancellation functionality

- [ ] Create `tournament_registration.rs` model file

  - [ ] Implement `TournamentRegistration` struct
  - [ ] Implement registration/unregistration operations
  - [ ] Add validation for `max_snakes_per_user` limit
  - [ ] Add seeding functionality

- [ ] Create `tournament_match.rs` model file

  - [ ] Implement `TournamentMatch` struct
  - [ ] Implement `MatchStatus` enum with canceled state
  - [ ] Implement match operations (create, update, etc.)
  - [ ] Implement bracket generation algorithm for:
    - [ ] Power-of-2 number of participants
    - [ ] Non-power-of-2 number of participants with byes
  - [ ] Add functions to traverse the bracket structure
  - [ ] Implement visual positioning calculation for bracket rendering
  - [ ] Implement winner determination logic for multi-game matches

- [ ] Create `match_participant.rs` model file

  - [ ] Implement `MatchParticipant` struct
  - [ ] Implement `ParticipantType` enum with wildcard type
  - [ ] Implement participant operations
  - [ ] Add seeding position logic

- [ ] Create `match_game.rs` model file
  - [ ] Implement `MatchGame` struct
  - [ ] Implement match game operations for multi-game matches
  - [ ] Add game winner tracking

## Transaction Safety

- [ ] Implement transaction wrappers for critical operations:
  - [ ] Tournament start and bracket generation
  - [ ] Match execution and winner determination
  - [ ] Tournament status updates

## Job System

- [ ] Create `RunTournamentRoundJob`

  - [ ] Job for running a round of tournament matches
  - [ ] Include transactional integrity
  - [ ] Add to job registry

- [ ] Create `RunMatchJob`

  - [ ] Job for running a single match (with multiple games if needed)
  - [ ] Support different match styles (single_game, best_of_3, first_to_3)
  - [ ] Implement proper winner determination logic
  - [ ] Handle failure scenarios with retries
  - [ ] Add to job registry

- [ ] Create `UpdateTournamentStatusJob`
  - [ ] Check for tournament completion or round advancement
  - [ ] Update current_round field
  - [ ] Add to job registry

## Error Handling

- [ ] Implement error handling for common scenarios:
  - [ ] Snake API unavailability
  - [ ] Required participant validation
  - [ ] Concurrent operation protection
  - [ ] Job failure recovery

## API Implementation

### Tournament Routes

- [ ] List all tournaments
- [ ] View single tournament details
- [ ] Create tournament with all settings
- [ ] Update tournament
- [ ] Delete tournament
- [ ] Start tournament (generate bracket)
- [ ] Run tournament round
- [ ] Reset tournament to registration phase
- [ ] Cancel tournament

### Registration Routes

- [ ] Register snake for tournament with max_snakes_per_user validation
- [ ] Unregister snake from tournament

### Match Routes

- [ ] View all matches in a tournament
- [ ] View details of a specific match
- [ ] Add admin override for match winners

## Frontend Implementation

### UI Components

- [ ] Create tournament list page
- [ ] Create tournament creation form with all settings
  - [ ] Add max_snakes_per_user setting
  - [ ] Add required_participants input
  - [ ] Add bracket preview based on expected participation
- [ ] Create tournament details page
  - [ ] Add status and progress indicators
  - [ ] Add tournament statistics
- [ ] Create tournament registration component with user limits
- [ ] Create bracket visualization component
  - [ ] Implement visual coordinate-based rendering
  - [ ] Add winner path indicators
  - [ ] Support responsive design
- [ ] Create match history component
- [ ] Add tournament admin controls
  - [ ] Reset tournament button
  - [ ] Cancel tournament button
  - [ ] Match winner override

### User Flows

- [ ] Implement tournament creation flow with all settings
- [ ] Implement registration workflow with validation
- [ ] Implement tournament start flow with bracket generation
- [ ] Implement match execution flow for different match styles
- [ ] Implement tournament reset and cancellation flows
- [ ] Implement dropout/disqualification handling

## Testing

- [ ] Unit tests for tournament models
- [ ] Unit tests for bracket generation algorithm
  - [ ] Test with power-of-2 participants
  - [ ] Test with non-power-of-2 participants
- [ ] Unit tests for match processing with different match styles
  - [ ] Test single_game determination
  - [ ] Test best_of_3 determination
  - [ ] Test first_to_3 determination
- [ ] Integration tests for tournament flows
- [ ] Test transaction safety
- [ ] Test error handling scenarios
- [ ] End-to-end tests for complete tournament lifecycle

## Documentation

- [ ] Document API endpoints
- [ ] Document tournament creation process
- [ ] Document bracket generation algorithm
- [ ] Document match styles and processing
- [ ] Document status transitions and validations
- [ ] Document error handling approaches
- [ ] Add user guide for tournament creation and management
