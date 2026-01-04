#!/usr/bin/env bash
# Fetches secrets from GCP Secret Manager and runs the server locally.
# Requires: gcloud CLI authenticated with access to battlesnake-production project.
#
# Usage: ./scripts/run-with-secrets.sh [cargo args...]
# Examples:
#   ./scripts/run-with-secrets.sh              # runs: cargo run
#   ./scripts/run-with-secrets.sh --release    # runs: cargo run --release

set -euo pipefail

PROJECT="battlesnake-production"
GCS_BUCKET="battlesnake-game-backups"

echo "Fetching secrets from GCP Secret Manager..."

DATABASE_URL=$(gcloud secrets versions access latest \
  --secret="tf-arena-database-url" \
  --project="$PROJECT")

ENGINE_DATABASE_URL=$(gcloud secrets versions access latest \
  --secret="tf-arena-engine-database-url" \
  --project="$PROJECT")

COOKIE_KEY=$(gcloud secrets versions access latest \
  --secret="tf-arena-cookie-key" \
  --project="$PROJECT")

echo "Secrets loaded. Starting server..."

export DATABASE_URL
export ENGINE_DATABASE_URL
export COOKIE_KEY
export GCS_BUCKET
export RUST_LOG="${RUST_LOG:-info,arena=debug}"

export SQLX_OFFLINE=true

cd "$(dirname "$0")/.."
exec cargo run "$@"
