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

CURRENT_APP=$(az containerapp show --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" -o json)
APP_ID=$(jq -r '.id' <<<"$CURRENT_APP")
DATA_VOLUME="sf-$SLUG-data"
CONFIG_ENV=$(jq -c '[.environment | to_entries[] | {name:.key,value:.value}]' "$CONFIG")
PATCH_BODY=$(jq -c \
  --arg image "$IMAGE" \
  --arg storage "$DATA_VOLUME" \
  --argjson config_env "$CONFIG_ENV" '
  .properties.template as $template |
  ($template.containers[0]) as $container |
  {
    properties: {
      template: {
        containers: [($container
          | .image = $image
          | .env = ($config_env + [
              ($container.env // [])[]
              | select(.name == "DELIVERY_PROVIDER_URL" or .name == "DELIVERY_PROVIDER_TOKEN" or .name == "DELIVERY_CALLBACK_SECRET")
            ])
          | .volumeMounts = (((.volumeMounts // []) | map(select(.mountPath != "/data"))) + [{volumeName:"data",mountPath:"/data"}])
        )],
        scale: (($template.scale // {}) | .minReplicas = 1 | .maxReplicas = 1),
        volumes: ((($template.volumes // []) | map(select(.name != "data"))) + [{name:"data",storageType:"AzureFile",storageName:$storage}])
      }
    }
  }
' <<<"$CURRENT_APP")

echo "Deploying one replica with the factory-managed /data volume."
az rest --method patch \
  --url "https://management.azure.com${APP_ID}?api-version=2024-03-01" \
  --body "$PATCH_BODY" \
  --output none

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

DEPLOYED_APP=$(az containerapp show --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" -o json)
jq -e --arg storage "$DATA_VOLUME" '
  .properties.template.scale.minReplicas == 1 and
  .properties.template.scale.maxReplicas == 1 and
  any(.properties.template.containers[0].volumeMounts[]; .mountPath == "/data" and .volumeName == "data") and
  any(.properties.template.volumes[]; .name == "data" and .storageType == "AzureFile" and .storageName == $storage)
' <<<"$DEPLOYED_APP" >/dev/null || {
  echo "The deployed app did not retain its one-replica /data contract." >&2
  exit 1
}

# Remove obsolete app-local secrets after the healthy revision no longer
# refers to them. Only secret names are queried; secret values are never read.
mapfile -t APP_SECRET_NAMES < <(jq -r '.properties.configuration.secrets[]?.name' <<<"$DEPLOYED_APP")
LEGACY_DB_SECRET="data""base-url"
for obsolete in "$LEGACY_DB_SECRET" contact-encryption-key; do
  for present in "${APP_SECRET_NAMES[@]}"; do
    if [ "$present" = "$obsolete" ]; then
      az containerapp secret remove --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
        --secret-names "$obsolete" --output none
    fi
  done
done

echo "Deployment complete. Verify /health, persistence, rate limits, and reset revocation."
