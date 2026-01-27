//! Stress test binary for Arena - generates load via the Create Game API.
//!
//! Supports configurable load patterns (steady stream, batch), periodic stats output,
//! and structured tracing events for Eyes integration.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use clap::Parser;
use color_eyre::eyre::{eyre, Context as _};
use reqwest::StatusCode;
use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Parser)]
#[command(name = "stress-test")]
#[command(about = "Stress test Arena by generating game creation load")]
struct Cli {
    /// Arena API base URL
    #[arg(long, default_value = "http://localhost:3000")]
    url: String,

    /// Comma-separated snake UUIDs to use for games
    #[arg(long)]
    snakes: String,

    /// API token for authentication
    #[arg(long, env = "ARENA_TOKEN")]
    token: String,

    /// Steady stream rate: N/s (e.g., "10/s" for 10 games per second)
    #[arg(long)]
    steady: Option<String>,

    /// Batch pattern: games,interval (e.g., "100,30s" for 100 games every 30 seconds)
    #[arg(long)]
    batch: Option<String>,

    /// Test duration (e.g., "5m", "1h", "30s")
    #[arg(long, default_value = "1m")]
    duration: String,

    /// Stats output interval in seconds
    #[arg(long, default_value = "10")]
    stats_interval: u64,

    /// Board size for games
    #[arg(long, default_value = "11x11")]
    board: String,

    /// Game type
    #[arg(long = "type", default_value = "standard")]
    game_type: String,
}

// ============================================================================
// Duration Parsing
// ============================================================================

fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if let Some(stripped) = s.strip_suffix('s') {
        let secs: u64 = stripped
            .parse()
            .map_err(|_| "Invalid seconds".to_string())?;
        Ok(Duration::from_secs(secs))
    } else if let Some(stripped) = s.strip_suffix('m') {
        let mins: u64 = stripped
            .parse()
            .map_err(|_| "Invalid minutes".to_string())?;
        Ok(Duration::from_secs(mins * 60))
    } else if let Some(stripped) = s.strip_suffix('h') {
        let hours: u64 = stripped
            .parse()
            .map_err(|_| "Invalid hours".to_string())?;
        Ok(Duration::from_secs(hours * 3600))
    } else {
        Err("Duration must end with 's', 'm', or 'h'".to_string())
    }
}

// ============================================================================
// HTTP Client
// ============================================================================

fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .pool_max_idle_per_host(100)
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

#[derive(Debug)]
struct CreateGameResult {
    game_id: Uuid,
    latency: Duration,
}

#[derive(Debug)]
enum GameCreationError {
    Request(reqwest::Error),
    Api { status: StatusCode, body: String },
    Parse(String),
}

impl std::fmt::Display for GameCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(e) => write!(f, "Request error: {}", e),
            Self::Api { status, body } => write!(f, "API error {}: {}", status, body),
            Self::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for GameCreationError {}

async fn create_game(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    snakes: &[Uuid],
    board: &str,
    game_type: &str,
) -> Result<CreateGameResult, GameCreationError> {
    let start = Instant::now();

    let response = client
        .post(format!("{}/api/games", base_url))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "snakes": snakes,
            "board": board,
            "game_type": game_type,
        }))
        .send()
        .await;

    let latency = start.elapsed();

    match response {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| GameCreationError::Parse(e.to_string()))?;

            let game_id = body["id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| GameCreationError::Parse("Missing game id".to_string()))?;

            Ok(CreateGameResult { game_id, latency })
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(GameCreationError::Api { status, body })
        }
        Err(e) => Err(GameCreationError::Request(e)),
    }
}

// ============================================================================
// Stats Tracking
// ============================================================================

struct Stats {
    total_games: AtomicU64,
    successful: AtomicU64,
    failed: AtomicU64,
    start_time: Instant,
    latencies: Mutex<Vec<u64>>, // Latencies in microseconds
}

impl Stats {
    fn new() -> Self {
        Self {
            total_games: AtomicU64::new(0),
            successful: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            start_time: Instant::now(),
            latencies: Mutex::new(Vec::with_capacity(10000)),
        }
    }

    fn record_success(&self, latency: Duration) {
        self.total_games.fetch_add(1, Ordering::Relaxed);
        self.successful.fetch_add(1, Ordering::Relaxed);
        let latency_us = latency.as_micros() as u64;
        self.latencies.lock().unwrap().push(latency_us);
    }

    fn record_failure(&self) {
        self.total_games.fetch_add(1, Ordering::Relaxed);
        self.failed.fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> StatsSnapshot {
        let total = self.total_games.load(Ordering::Relaxed);
        let successful = self.successful.load(Ordering::Relaxed);
        let failed = self.failed.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();

        let latencies = self.latencies.lock().unwrap();
        let (avg_latency, p50, p95, p99) = calculate_percentiles(&latencies);

        StatsSnapshot {
            total_games: total,
            successful,
            failed,
            elapsed,
            rate: if elapsed.as_secs_f64() > 0.0 {
                total as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            },
            success_rate: if total > 0 {
                successful as f64 / total as f64 * 100.0
            } else {
                0.0
            },
            avg_latency_ms: avg_latency,
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
        }
    }
}

struct StatsSnapshot {
    total_games: u64,
    successful: u64,
    failed: u64,
    elapsed: Duration,
    rate: f64,
    success_rate: f64,
    avg_latency_ms: f64,
    p50_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
}

fn calculate_percentiles(latencies: &[u64]) -> (f64, f64, f64, f64) {
    if latencies.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }

    let mut sorted = latencies.to_vec();
    sorted.sort_unstable();

    let len = sorted.len();
    let avg = sorted.iter().sum::<u64>() as f64 / len as f64 / 1000.0; // us to ms
    let p50 = sorted[len * 50 / 100] as f64 / 1000.0;
    let p95 = sorted[len * 95 / 100] as f64 / 1000.0;
    let p99_idx = (len * 99 / 100).min(len.saturating_sub(1));
    let p99 = sorted[p99_idx] as f64 / 1000.0;

    (avg, p50, p95, p99)
}

// ============================================================================
// Load Patterns
// ============================================================================

#[derive(Clone)]
struct LoadConfig {
    base_url: String,
    token: String,
    snakes: Vec<Uuid>,
    board: String,
    game_type: String,
}

#[async_trait]
trait LoadPattern: Send + Sync {
    async fn run(
        &self,
        client: &reqwest::Client,
        config: &LoadConfig,
        stats: &Arc<Stats>,
        cancel: CancellationToken,
    );
}

// Steady stream pattern
struct SteadyStreamPattern {
    rate_per_second: f64,
}

impl SteadyStreamPattern {
    fn from_str(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if !s.ends_with("/s") {
            return Err("Steady rate must end with '/s' (e.g., '10/s')".to_string());
        }
        let rate: f64 = s[..s.len() - 2]
            .parse()
            .map_err(|_| "Invalid rate number".to_string())?;
        if rate <= 0.0 {
            return Err("Rate must be positive".to_string());
        }
        Ok(Self { rate_per_second: rate })
    }
}

#[async_trait]
impl LoadPattern for SteadyStreamPattern {
    async fn run(
        &self,
        client: &reqwest::Client,
        config: &LoadConfig,
        stats: &Arc<Stats>,
        cancel: CancellationToken,
    ) {
        let interval_duration = Duration::from_secs_f64(1.0 / self.rate_per_second);
        let mut interval = tokio::time::interval(interval_duration);
        interval.set_missed_tick_behavior(MissedTickBehavior::Burst);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = interval.tick() => {
                    let client = client.clone();
                    let config = config.clone();
                    let stats = stats.clone();

                    tokio::spawn(async move {
                        match create_game(
                            &client,
                            &config.base_url,
                            &config.token,
                            &config.snakes,
                            &config.board,
                            &config.game_type,
                        )
                        .await
                        {
                            Ok(result) => {
                                stats.record_success(result.latency);
                                tracing::info!(
                                    game_id = %result.game_id,
                                    latency_ms = result.latency.as_millis() as u64,
                                    "game_created"
                                );
                            }
                            Err(e) => {
                                stats.record_failure();
                                tracing::warn!(error = %e, "game_creation_failed");
                            }
                        }
                    });
                }
            }
        }
    }
}

// Batch pattern
struct BatchPattern {
    batch_size: u32,
    interval: Duration,
}

impl BatchPattern {
    fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err("Batch format: 'count,interval' (e.g., '100,30s')".to_string());
        }
        let batch_size: u32 = parts[0]
            .trim()
            .parse()
            .map_err(|_| "Invalid batch size".to_string())?;
        let interval = parse_duration(parts[1].trim())?;
        if batch_size == 0 {
            return Err("Batch size must be positive".to_string());
        }
        Ok(Self { batch_size, interval })
    }
}

#[async_trait]
impl LoadPattern for BatchPattern {
    async fn run(
        &self,
        client: &reqwest::Client,
        config: &LoadConfig,
        stats: &Arc<Stats>,
        cancel: CancellationToken,
    ) {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = interval.tick() => {
                    // Spawn batch_size concurrent requests
                    let futures: Vec<_> = (0..self.batch_size)
                        .map(|_| {
                            let client = client.clone();
                            let config = config.clone();
                            let stats = stats.clone();
                            async move {
                                match create_game(
                                    &client,
                                    &config.base_url,
                                    &config.token,
                                    &config.snakes,
                                    &config.board,
                                    &config.game_type,
                                )
                                .await
                                {
                                    Ok(result) => {
                                        stats.record_success(result.latency);
                                        tracing::info!(
                                            game_id = %result.game_id,
                                            latency_ms = result.latency.as_millis() as u64,
                                            "game_created"
                                        );
                                    }
                                    Err(e) => {
                                        stats.record_failure();
                                        tracing::warn!(error = %e, "game_creation_failed");
                                    }
                                }
                            }
                        })
                        .collect();

                    futures::future::join_all(futures).await;
                }
            }
        }
    }
}

// ============================================================================
// Stats Output
// ============================================================================

async fn stats_output_task(stats: Arc<Stats>, interval_secs: u64, cancel: CancellationToken) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = interval.tick() => {
                let snapshot = stats.snapshot();

                // Terminal output
                let elapsed = format_duration(snapshot.elapsed);
                println!(
                    "[{}] Games: {} | Rate: {:.1}/s | Success: {:.1}% | Avg: {:.0}ms | p50: {:.0}ms | p95: {:.0}ms | p99: {:.0}ms",
                    elapsed,
                    snapshot.total_games,
                    snapshot.rate,
                    snapshot.success_rate,
                    snapshot.avg_latency_ms,
                    snapshot.p50_latency_ms,
                    snapshot.p95_latency_ms,
                    snapshot.p99_latency_ms,
                );

                // Structured tracing event for Eyes
                tracing::info!(
                    total_games = snapshot.total_games,
                    successful = snapshot.successful,
                    failed = snapshot.failed,
                    rate = snapshot.rate,
                    success_rate = snapshot.success_rate,
                    avg_latency_ms = snapshot.avg_latency_ms,
                    p50_latency_ms = snapshot.p50_latency_ms,
                    p95_latency_ms = snapshot.p95_latency_ms,
                    p99_latency_ms = snapshot.p99_latency_ms,
                    "stress_test_stats"
                );
            }
        }
    }
}

fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Setup tracing (JSON for Eyes compatibility)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("stress_test=info".parse().unwrap()),
        )
        .json()
        .init();

    let cli = Cli::parse();

    // Parse and validate snake UUIDs
    let snakes: Vec<Uuid> = cli
        .snakes
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(Uuid::parse_str)
        .collect::<Result<Vec<_>, _>>()
        .wrap_err("Invalid snake UUID format")?;

    if snakes.is_empty() {
        return Err(eyre!("At least one snake UUID is required"));
    }

    // Parse duration
    let duration =
        parse_duration(&cli.duration).map_err(|e| eyre!("Invalid duration: {}", e))?;

    // Build load patterns
    let mut patterns: Vec<Box<dyn LoadPattern>> = Vec::new();

    if let Some(ref steady) = cli.steady {
        let pattern = SteadyStreamPattern::from_str(steady)
            .map_err(|e| eyre!("Invalid steady pattern: {}", e))?;
        patterns.push(Box::new(pattern));
    }

    if let Some(ref batch) = cli.batch {
        let pattern =
            BatchPattern::from_str(batch).map_err(|e| eyre!("Invalid batch pattern: {}", e))?;
        patterns.push(Box::new(pattern));
    }

    if patterns.is_empty() {
        return Err(eyre!(
            "At least one load pattern (--steady or --batch) is required"
        ));
    }

    // Create shared state
    let client = create_http_client();
    let stats = Arc::new(Stats::new());
    let cancel = CancellationToken::new();

    let config = LoadConfig {
        base_url: cli.url.clone(),
        token: cli.token.clone(),
        snakes,
        board: cli.board.clone(),
        game_type: cli.game_type.clone(),
    };

    println!("Starting stress test against {}", cli.url);
    println!("Duration: {}", cli.duration);
    println!("Patterns: {}", patterns.len());
    println!("Snakes: {:?}", config.snakes);
    println!();

    // Spawn load pattern tasks
    let mut handles = Vec::new();
    for pattern in patterns {
        let client = client.clone();
        let config = config.clone();
        let stats = stats.clone();
        let cancel = cancel.clone();

        handles.push(tokio::spawn(async move {
            pattern.run(&client, &config, &stats, cancel).await;
        }));
    }

    // Spawn stats output task
    let stats_handle = {
        let stats = stats.clone();
        let cancel = cancel.clone();
        tokio::spawn(async move {
            stats_output_task(stats, cli.stats_interval, cancel).await;
        })
    };

    // Wait for duration then cancel
    tokio::time::sleep(duration).await;
    cancel.cancel();

    // Wait for tasks to finish
    for handle in handles {
        let _ = handle.await;
    }
    let _ = stats_handle.await;

    // Final stats output
    let final_snapshot = stats.snapshot();
    println!();
    println!("=== Final Results ===");
    println!("Total games: {}", final_snapshot.total_games);
    println!("Successful: {}", final_snapshot.successful);
    println!("Failed: {}", final_snapshot.failed);
    println!("Success rate: {:.1}%", final_snapshot.success_rate);
    println!("Average rate: {:.1} games/sec", final_snapshot.rate);
    println!("Avg latency: {:.0}ms", final_snapshot.avg_latency_ms);
    println!("p50 latency: {:.0}ms", final_snapshot.p50_latency_ms);
    println!("p95 latency: {:.0}ms", final_snapshot.p95_latency_ms);
    println!("p99 latency: {:.0}ms", final_snapshot.p99_latency_ms);

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_seconds() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("1s").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_duration("0s").unwrap(), Duration::from_secs(0));
    }

    #[test]
    fn test_parse_duration_minutes() {
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1m").unwrap(), Duration::from_secs(60));
    }

    #[test]
    fn test_parse_duration_hours() {
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("30").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("30x").is_err());
    }

    #[test]
    fn test_steady_stream_pattern_parsing() {
        let pattern = SteadyStreamPattern::from_str("10/s").unwrap();
        assert!((pattern.rate_per_second - 10.0).abs() < f64::EPSILON);

        let pattern = SteadyStreamPattern::from_str("0.5/s").unwrap();
        assert!((pattern.rate_per_second - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_steady_stream_pattern_invalid() {
        assert!(SteadyStreamPattern::from_str("10").is_err());
        assert!(SteadyStreamPattern::from_str("abc/s").is_err());
        assert!(SteadyStreamPattern::from_str("0/s").is_err());
        assert!(SteadyStreamPattern::from_str("-1/s").is_err());
    }

    #[test]
    fn test_batch_pattern_parsing() {
        let pattern = BatchPattern::from_str("100,30s").unwrap();
        assert_eq!(pattern.batch_size, 100);
        assert_eq!(pattern.interval, Duration::from_secs(30));

        let pattern = BatchPattern::from_str("50, 1m").unwrap();
        assert_eq!(pattern.batch_size, 50);
        assert_eq!(pattern.interval, Duration::from_secs(60));
    }

    #[test]
    fn test_batch_pattern_invalid() {
        assert!(BatchPattern::from_str("100").is_err());
        assert!(BatchPattern::from_str("100,30s,extra").is_err());
        assert!(BatchPattern::from_str("abc,30s").is_err());
        assert!(BatchPattern::from_str("0,30s").is_err());
    }

    #[test]
    fn test_calculate_percentiles_empty() {
        let (avg, p50, p95, p99) = calculate_percentiles(&[]);
        assert_eq!(avg, 0.0);
        assert_eq!(p50, 0.0);
        assert_eq!(p95, 0.0);
        assert_eq!(p99, 0.0);
    }

    #[test]
    fn test_calculate_percentiles() {
        // 100 values from 1000 to 100000 microseconds (1ms to 100ms)
        let latencies: Vec<u64> = (1..=100).map(|i| i * 1000).collect();
        let (avg, p50, p95, p99) = calculate_percentiles(&latencies);

        // Average of 1..=100 is 50.5, so in ms: 50.5
        assert!((avg - 50.5).abs() < 0.1);

        // For 100 elements: len*50/100 = 50, sorted[50] = 51ms
        // So p50 is around 51ms (integer division floors)
        assert!((p50 - 51.0).abs() < 1.0);

        // p95: len*95/100 = 95, sorted[95] = 96ms
        assert!((p95 - 96.0).abs() < 1.0);

        // p99: min(len*99/100, len-1) = min(99, 99) = 99, sorted[99] = 100ms
        assert!((p99 - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0)), "00:00:00");
        assert_eq!(format_duration(Duration::from_secs(61)), "00:01:01");
        assert_eq!(format_duration(Duration::from_secs(3661)), "01:01:01");
        assert_eq!(format_duration(Duration::from_secs(90)), "00:01:30");
    }
}
