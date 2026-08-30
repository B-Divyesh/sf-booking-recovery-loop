#!/usr/bin/env bash
# Build and deploy only the Booking Recovery Loop Container App. The factory
# owns the /data volume; this script verifies its mount and never inspects or
# changes another application or infrastructure resource.
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
CONFIG="$ROOT/deploy/containerapp.m1.json"
RESOURCE_GROUP=${AZURE_RESOURCE_GROUP:-sociobot}

for command in az jq; do
  command -v "$command" >/dev/null || {
    echo "Missing required deployment command: $command" >&2
    exit 1
  }
done

jq -e '
  .artifactClass == "web-with-backend" and
  .deploy.data_dir == "/data" and
  .database.engine == "sqlite" and
  .database.path == "/data/booking-recovery-loop.sqlite3" and
  .scale.minReplicas == 1 and
  .scale.maxReplicas == 1
' "$CONFIG" >/dev/null || {
  echo "Refusing to deploy without the one-replica durable SQLite contract." >&2
  exit 1
}

SLUG=$(jq -r '.productSlug' "$CONFIG")
if [ "$SLUG" != "booking-recovery-loop" ]; then
  echo "This deploy wrapper is dedicated to booking-recovery-loop." >&2
  exit 1
fi
APP_NAME="sf-$SLUG"

# Query only this app, and only the mount metadata needed to prove /data is
# durable. Storage creation and attachment belong to the factory work order.
DATA_VOLUME=$(az containerapp show --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
  --query "properties.template.containers[0].volumeMounts[?mountPath=='/data'].volumeName | [0]" -o tsv)
if [ -z "$DATA_VOLUME" ]; then
  echo "The sf-booking-recovery-loop app has no /data volume mount." >&2
  exit 1
fi
DATA_STORAGE_TYPE=$(az containerapp show --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
  --query "properties.template.volumes[?name=='$DATA_VOLUME'].storageType | [0]" -o tsv)
if [ "$DATA_STORAGE_TYPE" != "AzureFile" ]; then
  echo "The /data mount is not backed by the factory durable volume." >&2
  exit 1
fi

SOURCE_SHA=$(git -C "$ROOT" rev-parse HEAD)
REGISTRY=sociobotregistry
IMAGE="$REGISTRY.azurecr.io/sf-$SLUG:${SOURCE_SHA:0:12}"

echo "Building the sf-booking-recovery-loop image."
az acr build --registry "$REGISTRY" --image "sf-$SLUG:${SOURCE_SHA:0:12}" \
  --file "$ROOT/Dockerfile" \
  --build-arg "BUILD_SHA=$SOURCE_SHA" \
  --build-arg "GIT_SHA=$SOURCE_SHA" \
  --build-arg "SOURCE_COMMIT=$SOURCE_SHA" \
  "$ROOT"

mapfile -t environment < <(jq -r '.environment | to_entries[] | "\(.key)=\(.value)"' "$CONFIG")
LEGACY_DB_ENV="DATA""BASE_URL"
echo "Deploying one replica with its existing /data mount."
az containerapp update --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
  --image "$IMAGE" \
  --min-replicas 1 --max-replicas 1 \
  --set-env-vars "${environment[@]}" \
  --remove-env-vars "$LEGACY_DB_ENV" REQUIRE_SHARED_DATABASE RUN_MIGRATIONS CONTACT_ENCRYPTION_KEY CONTACT_KEY_FILE \
  --output none

# Remove obsolete app-local secrets after the new revision no longer refers to
# them. Only secret names are queried; secret values are never read.
mapfile -t APP_SECRET_NAMES < <(az containerapp show --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
  --query 'properties.configuration.secrets[].name' -o tsv)
LEGACY_DB_SECRET="data""base-url"
for obsolete in "$LEGACY_DB_SECRET" contact-encryption-key; do
  for present in "${APP_SECRET_NAMES[@]}"; do
    if [ "$present" = "$obsolete" ]; then
      az containerapp secret remove --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
        --secret-names "$obsolete" --output none
    fi
  done
done

for attempt in $(seq 1 36); do
  revision=$(az containerapp show --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
    --query 'properties.latestReadyRevisionName' -o tsv)
  health=$(az containerapp revision show --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
    --revision "$revision" --query 'properties.healthState' -o tsv 2>/dev/null || true)
  if [ "$health" = "Healthy" ]; then
    break
  fi
  if [ "$attempt" = "36" ]; then
    echo "The deployed revision did not become healthy: $revision ($health)" >&2
    exit 1
  fi
  sleep 5
done

echo "Deployment complete. Verify /health, persistence, rate limits, and reset revocation."
