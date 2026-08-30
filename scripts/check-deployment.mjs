import { readFile, readdir } from "node:fs/promises";
import { extname, relative } from "node:path";

const root = new URL("../", import.meta.url);
const contractPath = new URL("../deploy/containerapp.m1.json", import.meta.url);
const config = JSON.parse(await readFile(contractPath, "utf8"));

if (config.artifactClass !== "web-with-backend") {
  throw new Error("The deployment must remain web-with-backend.");
}
if (config.productSlug !== "booking-recovery-loop") {
  throw new Error("The deployment contract must identify this exact product.");
}
if (config.containerPort !== 8080) {
  throw new Error("The deployment must expose port 8080.");
}
if (config.deploy?.data_dir !== "/data") {
  throw new Error("The durable data mount must remain /data.");
}
if (config.scale?.minReplicas !== 1 || config.scale?.maxReplicas !== 1) {
  throw new Error("SQLite deployment must remain pinned to exactly one replica.");
}
if (
  config.database?.engine !== "sqlite" ||
  config.database?.path !== "/data/booking-recovery-loop.sqlite3" ||
  config.database?.journalMode !== "WAL"
) {
  throw new Error("Production must use the WAL-enabled SQLite file under /data.");
}
if (!config.database?.backup) {
  throw new Error("Production needs a backup and restore plan.");
}
if (config.environment?.SOCIOBOT_BILLING_BASE_URL !== "https://api.sociobot.in/api/v1") {
  throw new Error("Booking checkout must use the approved Sociobot billing boundary.");
}
if (config.environment?.SOCIOBOT_BOOKING_PRODUCT_SLUG !== "booking-recovery-loop-deposit") {
  throw new Error("Booking deposits must not reuse the practice subscription product.");
}
if (config.environment?.STATIC_DIR !== "/app/dist") {
  throw new Error("The deployed container must serve its copied production assets.");
}
if (Object.keys(config.environment ?? {}).some((name) => name.startsWith("DELIVERY_PROVIDER_"))) {
  throw new Error("Unprovisioned delivery values must not be deployed as placeholders.");
}

// Construct these sentinels so the regression test does not itself retain a
// forbidden deployment identifier. Scan source, tests, manifests, docs, and
// lockfiles; generated dependencies and version-control internals are skipped.
const sentinels = [
  ["sociobot", "db"].join("-"),
  ["sociobot", "v2"].join("-"),
  ["sociobot", "keyvault1"].join("-"),
  ["DATA", "BASE_URL"].join(""),
  ["Pg", "Bouncer"].join(""),
  ["shared", "post" + "gresql"].join(" "),
  ["post" + "gresql", "://"].join(""),
  ["sqlx", "post" + "gres"].join("-"),
  ["sqlx", "mysql"].join("-"),
];
const skippedDirectories = new Set([".git", "node_modules", "target", "dist", "test-results", "playwright-report"]);
const binaryExtensions = new Set([".avif", ".db", ".gif", ".ico", ".jpeg", ".jpg", ".png", ".sqlite", ".sqlite3", ".webp", ".woff", ".woff2"]);
const violations = [];

async function scan(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.isDirectory() && skippedDirectories.has(entry.name)) continue;
    const path = new URL(`${entry.name}${entry.isDirectory() ? "/" : ""}`, directory);
    if (entry.isDirectory()) {
      await scan(path);
      continue;
    }
    if (binaryExtensions.has(extname(entry.name))) continue;
    const body = await readFile(path, "utf8");
    for (const sentinel of sentinels) {
      if (body.toLowerCase().includes(sentinel.toLowerCase())) {
        violations.push(`${relative(root.pathname, path.pathname)} contains ${sentinel}`);
      }
    }
  }
}

await scan(root);
if (violations.length) {
  throw new Error(`Forbidden external-database references remain:\n${violations.join("\n")}`);
}

console.log("Production deployment boundary: one replica with durable SQLite state under /data");
