import { readFile } from "node:fs/promises";

const path = new URL("../deploy/containerapp.m1.json", import.meta.url);
const config = JSON.parse(await readFile(path, "utf8"));

if (config.artifactClass !== "web-with-backend") {
  throw new Error("The deployment must remain web-with-backend.");
}
if (config.containerPort !== 8080) {
  throw new Error("The deployment must expose port 8080.");
}
if (config.scale?.minReplicas !== 1 || config.scale?.maxReplicas < 2) {
  throw new Error(
    "Production must allow a second replica only when the storage boundary is shared.",
  );
}
if (config.database?.engine !== "postgresql" || config.database?.connectionStringEnv !== "DATABASE_URL") {
  throw new Error("Production must use the shared PostgreSQL DATABASE_URL contract.");
}
if (!config.database?.backup || !config.secrets?.CONTACT_ENCRYPTION_KEY) {
  throw new Error("Production needs a backup/restore plan and a shared contact encryption key.");
}
if (!config.secrets?.DELIVERY_PROVIDER_TOKEN || !config.secrets?.DELIVERY_CALLBACK_SECRET) {
  throw new Error("Production needs credentialed delivery and authenticated callback secrets.");
}
if (config.environment?.SOCIOBOT_BILLING_BASE_URL !== "https://api.sociobot.in/api/v1") {
  throw new Error("Booking checkout must use the approved Sociobot billing boundary.");
}
if (config.environment?.SOCIOBOT_BOOKING_PRODUCT_SLUG !== "booking-recovery-loop-deposit") {
  throw new Error("Booking deposits must not reuse the practice subscription product.");
}
if (config.environment?.DATABASE_URL !== "secretref:database-url" || config.environment?.REQUIRE_SHARED_DATABASE !== "1") {
  throw new Error("Production must inject the shared database secret and refuse replica-local fallback.");
}

console.log("Production deployment boundary: shared PostgreSQL, shared contact key, and multi-replica-safe API");
