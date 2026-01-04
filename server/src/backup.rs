//! Game backup module for archiving games from the Engine database to GCS.

use std::io::Write;

use chrono::{Duration, Utc};
use color_eyre::eyre::{Context as _, eyre};
use google_cloud_storage::{
    client::{Client as GcsClient, ClientConfig},
    http::objects::upload::{Media, UploadObjectRequest, UploadType},
};
use sqlx::{FromRow, PgPool};

use crate::engine_models::{EngineGame, EngineGameFrame, GameExport};
use crate::jobs::BackupSingleGameJob;
use crate::state::AppState;
use cja::jobs::Job;

/// Row from Engine's games table
#[derive(FromRow)]
struct EngineGameRow {
    id: String,
    value: serde_json::Value,
    /// Engine DB uses TIMESTAMP (no timezone), not TIMESTAMPTZ
    created: chrono::NaiveDateTime,
}

/// Fetch completed games from the Engine database within the given time window.
async fn fetch_completed_games(
    engine_db: &PgPool,
    hours_ago: i64,
) -> cja::Result<Vec<EngineGameRow>> {
    // Engine DB uses TIMESTAMP (no timezone), so use NaiveDateTime
    let since = (Utc::now() - Duration::hours(hours_ago)).naive_utc();

    // Note: We use query_as (not the macro) because this is a different database
    // with a different schema that sqlx doesn't know about at compile time.
    // Limit to 5000 as a safety valve - if we hit this, we'll catch the rest next run.
    let rows: Vec<EngineGameRow> = sqlx::query_as(
        r#"
        SELECT id, value, created
        FROM games
        WHERE value->>'Status' IN ('complete', 'error')
          AND created >= $1
        ORDER BY created ASC
        LIMIT 5000
        "#,
    )
    .bind(since)
    .fetch_all(engine_db)
    .await
    .wrap_err("Failed to fetch completed games from Engine")?;

    Ok(rows)
}

/// Fetch a single game from the Engine database by ID.
async fn fetch_game_by_id(engine_db: &PgPool, game_id: &str) -> cja::Result<Option<EngineGameRow>> {
    let row: Option<EngineGameRow> = sqlx::query_as(
        r#"
        SELECT id, value, created
        FROM games
        WHERE id = $1
        "#,
    )
    .bind(game_id)
    .fetch_optional(engine_db)
    .await
    .wrap_err("Failed to fetch game from Engine")?;

    Ok(row)
}

/// Fetch all frames for a game from the Engine database.
async fn fetch_game_frames(engine_db: &PgPool, game_id: &str) -> cja::Result<Vec<EngineGameFrame>> {
    let rows: Vec<(serde_json::Value,)> = sqlx::query_as(
        r#"
        SELECT value
        FROM game_frames
        WHERE id = $1
        ORDER BY turn ASC
        "#,
    )
    .bind(game_id)
    .fetch_all(engine_db)
    .await
    .wrap_err("Failed to fetch game frames from Engine")?;

    let frames: Vec<EngineGameFrame> = rows
        .into_iter()
        .map(|(value,)| serde_json::from_value(value))
        .collect::<Result<Vec<_>, _>>()
        .wrap_err("Failed to deserialize game frames")?;

    Ok(frames)
}

/// Check if a game has already been archived (exists in local games table with archived_at set).
async fn is_already_archived(db: &PgPool, engine_game_id: &str) -> cja::Result<bool> {
    let result = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM games
            WHERE engine_game_id = $1 AND archived_at IS NOT NULL
        ) as "exists!"
        "#,
        engine_game_id
    )
    .fetch_one(db)
    .await
    .wrap_err("Failed to check if game is already archived")?;

    Ok(result)
}

/// Generate the GCS path for a game based on its creation date.
fn gcs_path(game: &EngineGame) -> String {
    let created = game.created_at();
    format!(
        "games/{}/{:02}/{:02}/{}.json.zst",
        created.format("%Y"),
        created.format("%m"),
        created.format("%d"),
        game.id
    )
}

/// Compress JSON with zstd and upload to GCS.
async fn compress_and_upload_to_gcs(
    client: &GcsClient,
    bucket: &str,
    path: &str,
    export: &GameExport,
) -> cja::Result<()> {
    // Serialize to JSON
    let json = serde_json::to_vec(export).wrap_err("Failed to serialize game export")?;

    // Compress with zstd (level 3 is a good balance of speed/compression)
    let mut encoder =
        zstd::Encoder::new(Vec::new(), 3).wrap_err("Failed to create zstd encoder")?;
    encoder
        .write_all(&json)
        .wrap_err("Failed to write to zstd encoder")?;
    let compressed = encoder
        .finish()
        .wrap_err("Failed to finish zstd compression")?;

    tracing::debug!(
        game_id = %export.game.id,
        json_size = json.len(),
        compressed_size = compressed.len(),
        ratio = format!("{:.1}%", (compressed.len() as f64 / json.len() as f64) * 100.0),
        "Compressed game for upload"
    );

    // Upload to GCS
    let upload_type = UploadType::Simple(Media::new(path.to_string()));
    client
        .upload_object(
            &UploadObjectRequest {
                bucket: bucket.to_string(),
                ..Default::default()
            },
            compressed,
            &upload_type,
        )
        .await
        .wrap_err("Failed to upload to GCS")?;

    Ok(())
}

/// Current archive format version. Increment when changing the export format.
const ARCHIVE_VERSION: i32 = 1;

/// Insert or update a game record in the local database after archiving.
async fn upsert_game_record(db: &PgPool, game: &EngineGame, gcs_path: &str) -> cja::Result<()> {
    let now = Utc::now();
    let board_size = game.board_size();
    let game_type = game.game_type();
    let created_at = game.created_at();

    sqlx::query!(
        r#"
        INSERT INTO games (engine_game_id, board_size, game_type, status, created_at, archived_at, gcs_path, archive_version)
        VALUES ($1, $2, $3, 'finished', $4, $5, $6, $7)
        ON CONFLICT (engine_game_id) DO UPDATE SET
            archived_at = $5,
            gcs_path = $6,
            archive_version = $7,
            updated_at = $5
        "#,
        game.id,
        board_size,
        game_type,
        created_at,
        now,
        gcs_path,
        ARCHIVE_VERSION
    )
    .execute(db)
    .await
    .wrap_err("Failed to upsert game record")?;

    Ok(())
}

/// Backup error type that implements std::error::Error for cron compatibility.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct BackupError {
    message: String,
}

impl From<color_eyre::Report> for BackupError {
    fn from(err: color_eyre::Report) -> Self {
        Self {
            message: format!("{err:?}"),
        }
    }
}

/// Hours to look back for games to backup.
const BACKUP_WINDOW_HOURS: i64 = 4;

/// Run the game backup discovery process.
///
/// Finds completed games from the Engine database and enqueues individual
/// backup jobs for each game that hasn't been archived yet.
pub async fn run_backup_discovery(app_state: &AppState) -> Result<(), BackupError> {
    run_backup_discovery_inner(app_state).await.map_err(Into::into)
}

async fn run_backup_discovery_inner(app_state: &AppState) -> cja::Result<()> {
    tracing::info!(window_hours = BACKUP_WINDOW_HOURS, "Starting backup discovery");

    let engine_db = match &app_state.engine_db {
        Some(db) => db,
        None => {
            tracing::warn!("Engine database not configured, skipping backup discovery");
            return Ok(());
        }
    };

    // Fetch games from the lookback window
    let games = fetch_completed_games(engine_db, BACKUP_WINDOW_HOURS).await?;
    tracing::info!(
        count = games.len(),
        "Found completed games to check for archival"
    );

    let mut enqueued_count = 0;
    let mut skipped_count = 0;

    for game_row in games {
        // Check if already archived
        if is_already_archived(&app_state.db, &game_row.id).await? {
            skipped_count += 1;
            continue;
        }

        // Enqueue a job to backup this game
        BackupSingleGameJob {
            engine_game_id: game_row.id.clone(),
        }
        .enqueue(app_state.clone(), format!("backup game {}", game_row.id))
        .await
        .wrap_err_with(|| format!("Failed to enqueue backup job for game {}", game_row.id))?;

        enqueued_count += 1;
    }

    tracing::info!(
        enqueued = enqueued_count,
        skipped = skipped_count,
        "Backup discovery complete"
    );

    Ok(())
}

/// Backup a single game from the Engine database to GCS.
///
/// Called by BackupSingleGameJob. Fetches the game and frames from Engine,
/// compresses and uploads to GCS, and records the archival in the local database.
pub async fn backup_single_game(app_state: &AppState, engine_game_id: &str) -> Result<(), BackupError> {
    backup_single_game_inner(app_state, engine_game_id).await.map_err(Into::into)
}

async fn backup_single_game_inner(app_state: &AppState, engine_game_id: &str) -> cja::Result<()> {
    // Check if already archived (idempotency)
    if is_already_archived(&app_state.db, engine_game_id).await? {
        tracing::debug!(game_id = %engine_game_id, "Game already archived, skipping");
        return Ok(());
    }

    let engine_db = match &app_state.engine_db {
        Some(db) => db,
        None => {
            return Err(eyre!("Engine database not configured"));
        }
    };

    let bucket = match &app_state.gcs_bucket {
        Some(b) => b.clone(),
        None => {
            return Err(eyre!("GCS bucket not configured"));
        }
    };

    // Fetch the game from Engine
    let game_row = fetch_game_by_id(engine_db, engine_game_id)
        .await?
        .ok_or_else(|| eyre!("Game {} not found in Engine database", engine_game_id))?;

    // Parse the game data
    let game: EngineGame = serde_json::from_value(game_row.value)
        .wrap_err_with(|| format!("Failed to parse game data for {}", engine_game_id))?;

    // Fetch frames
    let frames = fetch_game_frames(engine_db, &game.id).await?;

    // Build export
    let export = GameExport {
        game: game.clone(),
        frames,
        exported_at: Utc::now(),
    };

    // Initialize GCS client
    let config = ClientConfig::default()
        .with_auth()
        .await
        .wrap_err("Failed to configure GCS client")?;
    let gcs_client = GcsClient::new(config);

    // Generate path and upload
    let path = gcs_path(&game);
    compress_and_upload_to_gcs(&gcs_client, &bucket, &path, &export).await?;

    // Record in local database
    upsert_game_record(&app_state.db, &game, &path).await?;

    tracing::info!(game_id = %game.id, path = %path, "Archived game");
    Ok(())
}
