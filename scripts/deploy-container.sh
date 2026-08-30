#!/usr/bin/env bash
# Delegate this product's release to the factory deployer. The fleet owns
# durable-share provisioning and mounts deploy.data_dir before updating the
# single product application.
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
CONFIG="$ROOT/deploy/containerapp.m1.json"

command -v jq >/dev/null || {
  echo "Missing required deployment command: jq" >&2
  exit 1
}

jq -e '
  .productSlug == "booking-recovery-loop" and
  .artifactClass == "web-with-backend" and
  .containerPort == 8080 and
  .deploy.data_dir == "/data" and
  .database.engine == "sqlite" and
  .database.path == "/data/state/booking-recovery-loop.sqlite3" and
  .scale.minReplicas == 1 and
  .scale.maxReplicas == 1
' "$CONFIG" >/dev/null || {
  echo "Refusing to deploy without the one-replica durable SQLite contract." >&2
  exit 1
}

SLUG=$(jq -r '.productSlug' "$CONFIG")
PORT=$(jq -r '.containerPort' "$CONFIG")
DATA_DIR=$(jq -r '.deploy.data_dir' "$CONFIG")
FLEET_DEPLOYER=${FACTORY_CONTAINER_DEPLOYER:-/opt/fleet/lib/deploy-container.sh}

if [ ! -x "$FLEET_DEPLOYER" ]; then
  echo "Factory container deployer is unavailable: $FLEET_DEPLOYER" >&2
  exit 1
fi

# WO_DATA_DIR is the factory deployer's durable-storage contract. When it is
# /data, the fleet creates or adopts the product share, mounts it, and pins the
# application to one replica before release.
export WO_DATA_DIR="$DATA_DIR"
exec "$FLEET_DEPLOYER" \
  "$SLUG" \
  "$ROOT" \
  "Dockerfile" \
  "$PORT" \
  "${PREBUILT_IMAGE:-}"
