# User Profile Specification

This document specifies the User Profile system for the Tournaments web application.

## Profile Page

r[profile.route]
The user profile MUST be accessible at `/me`.

r[profile.auth_required]
The profile page MUST require authentication (return 401 if not logged in).

r[profile.title]
The profile page MUST display "My Profile" as the page title/heading.

## User Information Display

r[profile.display.login]
The profile MUST display the user's GitHub login (username).

r[profile.display.avatar]
The profile MUST display the user's GitHub avatar image.

r[profile.display.avatar.alt]
The avatar image MUST have an "Avatar" alt text.

r[profile.display.name]
The profile SHOULD display the user's GitHub display name if available.

r[profile.display.email]
The profile SHOULD display the user's GitHub email if available.

## Account Details Section

r[profile.details.heading]
The profile MUST have an "Account Details" section heading.

r[profile.details.github_id]
The profile MUST display the user's GitHub ID.

r[profile.details.created_at]
The profile MUST display when the account was created.

r[profile.details.updated_at]
The profile MUST display when the account was last updated.

## Navigation Links

r[profile.nav.battlesnakes]
The profile MUST have a link to manage battlesnakes (`/battlesnakes`).

r[profile.nav.create_game]
The profile MUST have a link to create a new game (`/games/new`).

r[profile.nav.view_games]
The profile MUST have a link to view all games (`/games`).

r[profile.nav.home]
The profile MUST have a link to return to the home page (`/`).

r[profile.nav.logout]
The profile MUST have a logout link (`/auth/logout`).

## Homepage

r[homepage.route]
The homepage MUST be accessible at `/`.

r[homepage.public]
The homepage MUST be accessible without authentication.

### Unauthenticated State

r[homepage.unauth.message]
When not logged in, the homepage MUST display "You are not logged in."

r[homepage.unauth.login_link]
When not logged in, the homepage MUST display a "Login with GitHub" link.

### Authenticated State

r[homepage.auth.welcome]
When logged in, the homepage MUST display a welcome message with the user's login.

r[homepage.auth.avatar]
When logged in, the homepage MUST display the user's avatar.

r[homepage.auth.profile_link]
When logged in, the homepage MUST have a link to the profile page.

r[homepage.auth.battlesnakes_link]
When logged in, the homepage MUST have a link to battlesnakes.

r[homepage.auth.logout_link]
When logged in, the homepage MUST have a logout link.

r[homepage.auth.no_login_link]
When logged in, the homepage MUST NOT display the login link.
