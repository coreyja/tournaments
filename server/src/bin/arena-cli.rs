use clap::{Parser, Subcommand};
use color_eyre::eyre::{Context as _, eyre};

// Include the cli module from the library
use arena::cli::config::{AuthConfig, CliConfig};

#[derive(Parser)]
#[command(name = "arena")]
#[command(about = "Battlesnake Arena CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication commands
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Battlesnake management commands
    Snakes {
        #[command(subcommand)]
        command: SnakesCommands,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Login via GitHub OAuth and store API token
    Login,
    /// Logout and clear stored token
    Logout,
    /// API token management
    Token {
        #[command(subcommand)]
        command: TokenCommands,
    },
}

#[derive(Subcommand)]
enum TokenCommands {
    /// Create a new API token
    Create {
        /// Name for the token (e.g., "My laptop", "CI")
        #[arg(short, long)]
        name: Option<String>,
    },
    /// List all active API tokens
    List,
    /// Revoke an API token
    Revoke {
        /// Token ID to revoke
        id: String,
    },
}

#[derive(Subcommand)]
enum SnakesCommands {
    /// List all your snakes
    List,
    /// Create a new snake
    Create {
        /// Name for the snake
        name: String,
        /// URL for the snake server
        url: String,
        /// Make the snake public (visible to other users)
        #[arg(long)]
        public: bool,
    },
    /// Show details of a snake
    Show {
        /// Snake ID
        id: String,
    },
    /// Edit an existing snake
    Edit {
        /// Snake ID
        id: String,
        /// New name for the snake
        #[arg(long)]
        name: Option<String>,
        /// New URL for the snake server
        #[arg(long)]
        url: Option<String>,
        /// Make the snake public
        #[arg(long, conflicts_with = "private")]
        public: bool,
        /// Make the snake private
        #[arg(long, conflicts_with = "public")]
        private: bool,
    },
    /// Delete a snake
    Delete {
        /// Snake ID
        id: String,
    },
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { command } => handle_auth_command(command).await?,
        Commands::Snakes { command } => handle_snakes_command(command).await?,
    }

    Ok(())
}

async fn handle_auth_command(command: AuthCommands) -> color_eyre::Result<()> {
    match command {
        AuthCommands::Login => {
            login().await?;
        }
        AuthCommands::Logout => {
            logout()?;
        }
        AuthCommands::Token { command } => {
            handle_token_command(command).await?;
        }
    }
    Ok(())
}

async fn handle_token_command(command: TokenCommands) -> color_eyre::Result<()> {
    let config = CliConfig::load()?;
    let token = config
        .auth
        .as_ref()
        .and_then(|a| a.token.as_ref())
        .ok_or_else(|| eyre!("Not logged in. Run 'arena auth login' first."))?;

    let client = reqwest::Client::new();
    let base_url = config.api_url();

    match command {
        TokenCommands::Create { name } => {
            let name = name.unwrap_or_else(|| {
                hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok())
                    .unwrap_or_else(|| "CLI Token".to_string())
            });

            let response = client
                .post(format!("{}/api/tokens", base_url))
                .bearer_auth(token)
                .json(&serde_json::json!({ "name": name }))
                .send()
                .await
                .wrap_err("Failed to create token")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(eyre!("Failed to create token: {} - {}", status, body));
            }

            let result: serde_json::Value = response.json().await?;
            println!("Token created successfully!");
            println!("ID: {}", result["id"]);
            println!("Name: {}", result["name"]);
            println!("\nSecret (save this - it won't be shown again):");
            println!("{}", result["secret"]);
        }
        TokenCommands::List => {
            let response = client
                .get(format!("{}/api/tokens", base_url))
                .bearer_auth(token)
                .send()
                .await
                .wrap_err("Failed to list tokens")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(eyre!("Failed to list tokens: {} - {}", status, body));
            }

            let tokens: Vec<serde_json::Value> = response.json().await?;

            if tokens.is_empty() {
                println!("No active tokens found.");
            } else {
                println!("{:<38} {:<20} {:<20}", "ID", "NAME", "LAST USED");
                println!("{}", "-".repeat(78));
                for token in tokens {
                    let last_used = token["last_used_at"].as_str().unwrap_or("Never");
                    println!(
                        "{:<38} {:<20} {:<20}",
                        token["id"].as_str().unwrap_or(""),
                        token["name"].as_str().unwrap_or(""),
                        last_used
                    );
                }
            }
        }
        TokenCommands::Revoke { id } => {
            let response = client
                .delete(format!("{}/api/tokens/{}", base_url, id))
                .bearer_auth(token)
                .send()
                .await
                .wrap_err("Failed to revoke token")?;

            if response.status() == reqwest::StatusCode::NO_CONTENT {
                println!("Token revoked successfully.");
            } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(eyre!("Token not found or already revoked."));
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(eyre!("Failed to revoke token: {} - {}", status, body));
            }
        }
    }

    Ok(())
}

async fn handle_snakes_command(command: SnakesCommands) -> color_eyre::Result<()> {
    let config = CliConfig::load()?;
    let token = config
        .auth
        .as_ref()
        .and_then(|a| a.token.as_ref())
        .ok_or_else(|| eyre!("Not logged in. Run 'arena auth login' first."))?;

    let client = reqwest::Client::new();
    let base_url = config.api_url();

    match command {
        SnakesCommands::List => {
            let response = client
                .get(format!("{}/api/snakes", base_url))
                .bearer_auth(token)
                .send()
                .await
                .wrap_err("Failed to list snakes")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(eyre!("Failed to list snakes: {} - {}", status, body));
            }

            let snakes: Vec<serde_json::Value> = response.json().await?;
            // Output as JSON
            println!("{}", serde_json::to_string_pretty(&snakes)?);
        }
        SnakesCommands::Create { name, url, public } => {
            let response = client
                .post(format!("{}/api/snakes", base_url))
                .bearer_auth(token)
                .json(&serde_json::json!({
                    "name": name,
                    "url": url,
                    "is_public": public
                }))
                .send()
                .await
                .wrap_err("Failed to create snake")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(eyre!("Failed to create snake: {} - {}", status, body));
            }

            let snake: serde_json::Value = response.json().await?;
            println!("{}", serde_json::to_string_pretty(&snake)?);
        }
        SnakesCommands::Show { id } => {
            let response = client
                .get(format!("{}/api/snakes/{}", base_url, id))
                .bearer_auth(token)
                .send()
                .await
                .wrap_err("Failed to get snake")?;

            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(eyre!("Snake not found."));
            } else if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(eyre!("Failed to get snake: {} - {}", status, body));
            }

            let snake: serde_json::Value = response.json().await?;
            println!("{}", serde_json::to_string_pretty(&snake)?);
        }
        SnakesCommands::Edit {
            id,
            name,
            url,
            public,
            private,
        } => {
            // Build the update payload with only provided fields
            let mut update: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            if let Some(name) = name {
                update.insert("name".to_string(), serde_json::Value::String(name));
            }
            if let Some(url) = url {
                update.insert("url".to_string(), serde_json::Value::String(url));
            }
            if public {
                update.insert("is_public".to_string(), serde_json::Value::Bool(true));
            } else if private {
                update.insert("is_public".to_string(), serde_json::Value::Bool(false));
            }

            let response = client
                .put(format!("{}/api/snakes/{}", base_url, id))
                .bearer_auth(token)
                .json(&update)
                .send()
                .await
                .wrap_err("Failed to update snake")?;

            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(eyre!("Snake not found."));
            } else if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(eyre!("Failed to update snake: {} - {}", status, body));
            }

            let snake: serde_json::Value = response.json().await?;
            println!("{}", serde_json::to_string_pretty(&snake)?);
        }
        SnakesCommands::Delete { id } => {
            let response = client
                .delete(format!("{}/api/snakes/{}", base_url, id))
                .bearer_auth(token)
                .send()
                .await
                .wrap_err("Failed to delete snake")?;

            if response.status() == reqwest::StatusCode::NO_CONTENT {
                println!("Snake deleted successfully.");
            } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(eyre!("Snake not found."));
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(eyre!("Failed to delete snake: {} - {}", status, body));
            }
        }
    }

    Ok(())
}

async fn login() -> color_eyre::Result<()> {
    let config = CliConfig::load()?;
    let base_url = config.api_url();

    println!("Opening browser for GitHub authentication...");
    println!(
        "If the browser doesn't open, visit: {}/auth/github?cli=true",
        base_url
    );

    // Try to open browser
    let _ = open::that(format!("{}/auth/github?cli=true", base_url));

    // For now, prompt user to enter the token manually
    println!("\nAfter authenticating, you'll receive an API token.");
    println!("Enter your API token:");

    let mut token = String::new();
    std::io::stdin().read_line(&mut token)?;
    let token = token.trim().to_string();

    if token.is_empty() {
        return Err(eyre!("No token provided"));
    }

    // Validate the token by trying to list tokens
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/tokens", base_url))
        .bearer_auth(&token)
        .send()
        .await
        .wrap_err("Failed to validate token")?;

    if !response.status().is_success() {
        return Err(eyre!("Invalid token"));
    }

    // Save the token
    let mut config = config;
    config.auth = Some(AuthConfig { token: Some(token) });
    config.save()?;

    println!("Login successful! Token saved.");
    Ok(())
}

fn logout() -> color_eyre::Result<()> {
    let mut config = CliConfig::load()?;
    config.auth = None;
    config.save()?;
    println!("Logged out successfully.");
    Ok(())
}
