import { readFile } from "node:fs/promises";

const path = new URL("../deploy/containerapp.m1.json", import.meta.url);
const config = JSON.parse(await readFile(path, "utf8"));

if (config.artifactClass !== "web-with-backend") {
  throw new Error("The deployment must remain web-with-backend.");
}
if (config.productSlug !== "booking-recovery-loop") {
  throw new Error("The deployment contract must identify this exact product.");
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
if (config.environment?.SOCIOBOT_BILLING_BASE_URL !== "https://api.sociobot.in/api/v1") {
  throw new Error("Booking checkout must use the approved Sociobot billing boundary.");
}
if (config.environment?.SOCIOBOT_BOOKING_PRODUCT_SLUG !== "booking-recovery-loop-deposit") {
  throw new Error("Booking deposits must not reuse the practice subscription product.");
}
if (config.environment?.DATABASE_URL !== "secretref:database-url" || config.environment?.REQUIRE_SHARED_DATABASE !== "1") {
  throw new Error("Production must inject the shared database secret and refuse replica-local fallback.");
}
if (config.environment?.STATIC_DIR !== "/app/dist") {
  throw new Error("The deployed container must serve its copied production assets.");
}
if (Object.keys(config.environment ?? {}).some((name) => name.startsWith("DELIVERY_PROVIDER_"))) {
  throw new Error("Unprovisioned delivery values must not be deployed as placeholder configuration.");
}
if (config.integrations?.delivery?.status !== "requires-factory-provisioning") {
  throw new Error("The deployment must declare the missing credentialed delivery boundary honestly.");
}
if (config.integrations?.billing?.status !== "requires-factory-product-registration") {
  throw new Error("The deployment must declare the dedicated deposit-product registration boundary honestly.");
}
if (config.secrets?.DATABASE_URL !== "database-url" || !config.secrets?.CONTACT_ENCRYPTION_KEY) {
  throw new Error("The deployment must map the managed database and stable contact-encryption secrets.");
}

console.log("Production deployment boundary: shared PostgreSQL, shared contact key, and multi-replica-safe API");
