#!/usr/bin/env bash
# Deploy Booking Recovery Loop without dropping its shared-store runtime
# configuration. It builds the image directly and updates the existing app,
# rather than using the generic factory deployer that knows only PORT and
# replaces the whole runtime configuration.
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
CONFIG="$ROOT/deploy/containerapp.m1.json"
RESOURCE_GROUP=${AZURE_RESOURCE_GROUP:-sociobot}
KEY_VAULT=${AZURE_KEY_VAULT:-sociobot-keyvault1}

for command in az jq openssl; do
  command -v "$command" >/dev/null || {
    echo "Missing required deployment command: $command" >&2
    exit 1
  }
done

jq -e '
  .artifactClass == "web-with-backend" and
  .database.engine == "postgresql" and
  .database.connectionStringEnv == "DATABASE_URL" and
  .environment.DATABASE_URL == "secretref:database-url" and
  .environment.CONTACT_ENCRYPTION_KEY == "secretref:contact-encryption-key" and
  .environment.REQUIRE_SHARED_DATABASE == "1"
' "$CONFIG" >/dev/null || {
  echo "Refusing to deploy without the shared PostgreSQL configuration contract." >&2
  exit 1
}

SLUG=$(jq -r '.productSlug // "booking-recovery-loop"' "$CONFIG")
if [ "$SLUG" = "booking-recovery-loop" ]; then
  :
else
  echo "This deploy wrapper is dedicated to booking-recovery-loop." >&2
  exit 1
fi
APP_NAME="sf-$SLUG"
PORT=$(jq -r '.containerPort' "$CONFIG")
DATABASE_SECRET_NAME=$(jq -r '.secrets.DATABASE_URL' "$CONFIG")
CONTACT_SECRET_NAME=$(jq -r '.secrets.CONTACT_ENCRYPTION_KEY' "$CONFIG")

# Keep the database value in a shell variable only. Azure CLI redacts secret
# values in its response; this script intentionally never echoes either value.
DATABASE_URL=$(az keyvault secret show \
  --vault-name "$KEY_VAULT" \
  --name "$DATABASE_SECRET_NAME" \
  --query value -o tsv)

# The worker has read-only access to the shared vault. A Container App secret
# is durable across `az containerapp update` calls, so create a random key only
# on the first configuration of this app and retain that exact secret later.
CONTACT_SECRET_EXISTS=$(az containerapp show --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
  --query "length(properties.configuration.secrets[?name=='contact-encryption-key'] || \`[]\`)" -o tsv)
if [ "$CONTACT_SECRET_EXISTS" = "0" ]; then
  CONTACT_KEY=$(openssl rand -hex 32)
  az containerapp secret set --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
    --secrets "contact-encryption-key=$CONTACT_KEY" --output none
fi

SOURCE_SHA=$(git -C "$ROOT" rev-parse HEAD)
REGISTRY=sociobotregistry
IMAGE="$REGISTRY.azurecr.io/sf-$SLUG:${SOURCE_SHA:0:12}"
echo "Building and publishing the container image."
az acr build --registry "$REGISTRY" --image "sf-$SLUG:${SOURCE_SHA:0:12}" \
  --file "$ROOT/Dockerfile" \
  --build-arg "BUILD_SHA=$SOURCE_SHA" \
  --build-arg "GIT_SHA=$SOURCE_SHA" \
  --build-arg "SOURCE_COMMIT=$SOURCE_SHA" \
  "$ROOT"

echo "Applying shared runtime secrets and the one-replica migration revision."
az containerapp secret set --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
  --secrets "database-url=$DATABASE_URL" \
  --output none

mapfile -t environment < <(jq -r '.environment | to_entries[] | "\(.key)=\(.value)"' "$CONFIG")
az containerapp update --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
  --image "$IMAGE" \
  --min-replicas 1 --max-replicas 1 \
  --set-env-vars "${environment[@]}" RUN_MIGRATIONS=1 \
  --remove-env-vars DELIVERY_PROVIDER_URL DELIVERY_PROVIDER_TOKEN DELIVERY_CALLBACK_SECRET \
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
    echo "The migration revision did not become healthy: $revision ($health)" >&2
    exit 1
  fi
  sleep 5
done

# Migrations run once on a single replica. The serving revision must not carry
# RUN_MIGRATIONS because PostgreSQL may be behind transaction-pooling.
echo "Promoting the normal multi-replica serving revision."
az containerapp update --name "$APP_NAME" --resource-group "$RESOURCE_GROUP" \
  --max-replicas "$(jq -r '.scale.maxReplicas' "$CONFIG")" \
  --remove-env-vars RUN_MIGRATIONS \
  --output none

echo "Deployment complete. Verify /health and independent-connection topology probes before release."
