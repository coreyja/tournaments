# User Profile Specification

This document specifies the User Profile system for the Tournaments web application.

## Profile Page

r[profile.route]
The user profile MUST be accessible at `/me`.

r[profile.auth-required]
The profile page MUST require authentication (return 401 if not logged in).

r[profile.title]
The profile page MUST display "My Profile" as the page title/heading.

## User Information Display

r[profile.display.login]
The profile MUST display the user's GitHub login (username).

r[profile.display.avatar]
The profile MUST display the user's GitHub avatar image.

r[profile.display.name]
The profile SHOULD display the user's GitHub display name if available.

r[profile.display.email]
The profile SHOULD display the user's GitHub email if available.

## Battlesnake Summary

r[profile.battlesnakes.summary]
The profile MUST display a summary of the user's 5 most active battlesnakes.

## Navigation Links

r[profile.nav.battlesnakes]
The profile MUST have a link to manage battlesnakes (`/battlesnakes`).

r[profile.nav.create-game]
The profile MUST have a link to create a new game (`/games/new`).

r[profile.nav.view-games]
The profile MUST have a link to view all games (`/games`).

r[profile.nav.home]
The profile MUST have a link to return to the home page (`/`).

## Homepage

r[homepage.route]
The homepage MUST be accessible at `/`.

r[homepage.public]
The homepage MUST be accessible without authentication.

### Unauthenticated State

r[homepage.unauth.message]
When not logged in, the homepage MUST display "You are not logged in."

r[homepage.unauth.login-link]
When not logged in, the homepage MUST display a "Login with GitHub" link.

### Authenticated State

r[homepage.auth.welcome]
When logged in, the homepage MUST display a welcome message with the user's login.

r[homepage.auth.avatar]
When logged in, the homepage MUST display the user's avatar.

r[homepage.auth.profile-link]
When logged in, the homepage MUST have a link to the profile page.

r[homepage.auth.battlesnakes-link]
When logged in, the homepage MUST have a link to battlesnakes.

r[homepage.auth.logout-link]
When logged in, the homepage MUST have a logout link.

r[homepage.auth.no-login-link]
When logged in, the homepage MUST NOT display the login link.
