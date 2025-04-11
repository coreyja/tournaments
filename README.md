# Tournaments

A tournament management application with GitHub OAuth authentication.

## Setup

### Prerequisites

- Rust 1.72 or later
- PostgreSQL 12 or later

### Environment Variables

Create a `.envrc` file in the root directory with the following environment variables:

```
export DATABASE_URL="postgresql://localhost:5432/tournaments"
export GITHUB_CLIENT_ID="your_github_client_id"
export GITHUB_CLIENT_SECRET="your_github_client_secret"
export GITHUB_REDIRECT_URI="http://localhost:3000/auth/github/callback"
```

If you're using [direnv](https://direnv.net/), run `direnv allow` to load these environment variables.

### Creating a GitHub App

If you want to create a GitHub App instead of an OAuth App:

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click on "New GitHub App"
3. Fill in the required fields:
   - **GitHub App name**: Tournaments (or any name you prefer)
   - **Homepage URL**: http://localhost:3000
   - **Callback URL**: http://localhost:3000/auth/github/callback
   - **Setup URL**: (Optional) Leave blank
   - **Webhook URL**: (Optional) Leave blank for local development
   - **Webhook secret**: (Optional) Leave blank for local development
   - **Permissions**: Set the following permissions:
     - **User permissions**: Read access to email addresses and profile information
   - **Where can this GitHub App be installed?**: Any account
4. Click "Create GitHub App"
5. On the next page, note your **Client ID**
6. Generate a client secret by clicking "Generate a new client secret"
7. Update your `.envrc` file with the new credentials

### Database Setup

Run the following commands to set up the database:

```bash
cargo sqlx db create
cargo sqlx migrate run
```

### Running the Application

To run the application:

```bash
cargo run
```

The application will be available at http://localhost:3000

## Development

### Build/Lint/Test Commands

- Build: `cargo build`
- Run: `cargo run`
- Check: `cargo check`
- Lint: `cargo clippy`
- Fix auto-correctable lints: `cargo clippy --fix`
- Format: `cargo fmt`
- Test: `cargo test`
- Run database migrations: `cargo sqlx migrate run`
- Create migration: `cargo sqlx migrate add --source migrations <migration_name>`
