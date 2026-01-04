# Arena

A tournament management application with GitHub OAuth authentication.

## Setup

### Prerequisites

- Rust 1.72 or later
- PostgreSQL 12 or later

### Environment Variables

Create a `.envrc` file in the root directory with the following environment variables:

```
export DATABASE_URL="postgresql://localhost:5432/arena"
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
   - **GitHub App name**: Arena (or any name you prefer)
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

### Database Commands

- Create database: `cargo sqlx db create`
- Drop database: `cargo sqlx db drop`
- Run all migrations: `cargo sqlx migrate run`
- Revert latest migration: `cargo sqlx mig revert`
- Create new migration: `cargo sqlx migrate add --source migrations <migration_name>`
- Recreate DB from scratch: `cargo sqlx db drop -y && cargo sqlx db create && cargo sqlx migrate run`
- Update query cache: `DATABASE_URL="postgresql://localhost:5432/arena" cargo sqlx prepare --workspace`

Note: Always ensure the DATABASE_URL environment variable is set when working with SQLx commands, especially for migration reversion: `DATABASE_URL="postgresql://localhost:5432/arena" cargo sqlx mig revert`

### E2E Testing

End-to-end tests use Playwright and are located in the `e2e/` directory.

#### Setup

```bash
cd e2e
npm install
npx playwright install chromium
```

#### Test Database

E2E tests use a separate database (`arena_test`). Create it before running tests:

```bash
DATABASE_URL="postgresql://localhost:5432/arena_test" cargo sqlx db create
DATABASE_URL="postgresql://localhost:5432/arena_test" cargo sqlx migrate run
```

#### Running Tests

From the `e2e/` directory:

```bash
# Run tests headless (default)
npm test

# Run tests with browser visible
npm run test:headed

# Run tests in debug mode (step through)
npm run test:debug

# Run tests in UI mode (interactive)
npm run test:ui
```

Note: Tests automatically start the server using `cargo run` with the test database. The first run may take longer due to compilation.

### Spec-to-Code Tracing with Tracey

This project uses [Tracey](https://github.com/bearcove/tracey) for spec-to-code tracing, linking technical specifications to both implementations and tests.

#### Specifications

Specifications are written in Markdown and located in `specs/web_app/`:

- `auth.md` - Authentication (GitHub OAuth, sessions, logout)
- `battlesnakes.md` - Battlesnake CRUD operations and validation
- `games.md` - Game creation, listing, and viewing
- `profiles.md` - User profiles and homepage

Each spec uses the `r[rule.id]` syntax to define requirements, for example:
```markdown
r[auth.oauth.initiation]
The system provides a `/auth/github` route that initiates the OAuth flow.
```

#### Markers

- **Implementation markers** (`[impl rule.id]`) are placed in Rust source code doc comments
- **Verification markers** (`[verify rule.id]`) are placed in E2E test comments

#### Running Tracey Locally

Install Tracey:
```bash
cargo install tracey
```

Run the report:
```bash
tracey --config .config/tracey/config.kdl
```

If Tracey is not installed, the CI workflow will still pass (with a warning) and report generation will be skipped.

#### CI Integration

Tracey runs automatically in CI on every push and pull request. The workflow:
1. Installs Tracey if not cached
2. Generates a coverage report
3. Uploads the report as an artifact

Note: The Tracey job is configured to not fail the build, it only generates reports.
