# Authentication Specification

This document specifies the authentication system for the Tournaments web application.

## Session Management

r[auth.session.creation]
When a request is received without a valid session cookie, the system MUST create a new session and set the session cookie.

r[auth.session.cookie.name]
The session cookie MUST be named `tournaments_session_id`.

r[auth.session.cookie.httponly]
The session cookie MUST have the HttpOnly flag set to prevent client-side JavaScript access.

r[auth.session.cookie.secure]
The session cookie MUST have the Secure flag set to ensure transmission only over HTTPS.

r[auth.session.cookie.samesite]
The session cookie MUST have SameSite set to Lax to prevent CSRF attacks while allowing navigation.

r[auth.session.expiration]
Sessions MUST expire after 30 days from creation.

r[auth.session.persistence]
Sessions MUST be stored in the PostgreSQL database.

## GitHub OAuth Flow

r[auth.oauth.initiation]
When a user navigates to `/auth/github`, the system MUST redirect to GitHub's OAuth authorization endpoint.

r[auth.oauth.state.generation]
The system MUST generate a unique state parameter (UUID) for each OAuth request.

r[auth.oauth.state.storage]
The OAuth state parameter MUST be stored in the user's session before redirecting to GitHub.

r[auth.oauth.scope]
The OAuth request MUST request the `user:email` scope.

r[auth.oauth.callback.route]
The OAuth callback MUST be handled at `/auth/github/callback`.

r[auth.oauth.state.validation]
Upon callback, the system MUST verify that the state parameter matches the one stored in the session.

r[auth.oauth.state.mismatch]
If the state parameter does not match, the system MUST return a 400 Bad Request error.

r[auth.oauth.state.missing]
If the state parameter is missing from the session, the system MUST return a 400 Bad Request error.

r[auth.oauth.state.cleanup]
After successful state validation, the OAuth state MUST be cleared from the session.

r[auth.oauth.token.exchange]
The system MUST exchange the authorization code for an access token using GitHub's token endpoint.

r[auth.oauth.user.fetch]
After obtaining an access token, the system MUST fetch the user's GitHub profile data.

r[auth.oauth.user.creation]
If the GitHub user does not exist in the database, the system MUST create a new user record.

r[auth.oauth.user.update]
If the GitHub user already exists, the system MUST update their profile data (name, email, avatar).

r[auth.oauth.session.association]
After successful authentication, the user MUST be associated with the current session.

r[auth.oauth.success.redirect]
After successful authentication, the user MUST be redirected to the home page with a success flash message.

## Logout

r[auth.logout.route]
Logout MUST be accessible at `/auth/logout`.

r[auth.logout.session.disassociation]
When logging out, the user MUST be disassociated from the session.

r[auth.logout.redirect]
After logout, the user MUST be redirected to the home page.

r[auth.logout.flash]
After logout, a flash message MUST inform the user they have been logged out.

## Protected Routes

r[auth.protected.extraction]
Protected routes MUST use the `CurrentUser` extractor to require authentication.

r[auth.protected.unauthorized]
If a user attempts to access a protected route without authentication, the system MUST return a 401 Unauthorized status.

r[auth.optional.extraction]
Routes that optionally display user information MUST use the `OptionalUser` extractor.

## User Data Model

r[auth.user.github_id]
Each user MUST have a unique external GitHub ID.

r[auth.user.github_login]
Each user MUST have their GitHub login (username) stored.

r[auth.user.avatar]
The user's GitHub avatar URL SHOULD be stored if available.

r[auth.user.name]
The user's GitHub display name SHOULD be stored if available.

r[auth.user.email]
The user's GitHub email SHOULD be stored if available.

r[auth.user.timestamps]
Each user MUST have `created_at` and `updated_at` timestamps.
