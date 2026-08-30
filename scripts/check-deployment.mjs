import { execFile } from "node:child_process";
import { chmod, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { extname, join, relative } from "node:path";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

const root = new URL("../", import.meta.url);
const contractPath = new URL("../deploy/containerapp.m1.json", import.meta.url);
const config = JSON.parse(await readFile(contractPath, "utf8"));
const dockerfile = await readFile(new URL("../Dockerfile", import.meta.url), "utf8");
const cargoManifest = await readFile(new URL("../backend/Cargo.toml", import.meta.url), "utf8");
const runtimeSource = await readFile(new URL("../backend/src/main.rs", import.meta.url), "utf8");
const deployScriptUrl = new URL("./deploy-container.sh", import.meta.url);
const deployScript = await readFile(deployScriptUrl, "utf8");

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
  config.database?.path !== "/data/state/booking-recovery-loop.sqlite3" ||
  config.database?.journalMode !== "DELETE"
) {
  throw new Error("Production must use the mounted-filesystem-safe SQLite file under /data.");
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
if (!cargoManifest.includes('sqlx = { path = "sqlx-sqlite-only" }')) {
  throw new Error("The backend must use the SQLite-only database facade.");
}
if (!dockerfile.includes("COPY backend/sqlx-sqlite-only ./sqlx-sqlite-only")) {
  throw new Error("The production image must copy the SQLite-only facade before building.");
}
if (!runtimeSource.includes(".max_connections(1)")) {
  throw new Error("Mounted SQLite must use one in-process connection to avoid competing file locks.");
}
if (runtimeSource.includes("SqliteJournalMode::Wal")) {
  throw new Error("Mounted SQLite must not enable WAL on the network filesystem.");
}
if (!runtimeSource.includes('format!("file:{sqlite_path}?nolock=1")')) {
  throw new Error("One-replica mounted SQLite must disable unsupported network file locking.");
}

// Regression for repair 12: the old wrapper patched a volume that referred to
// a missing environment storage and Azure rejected the release with
// ManagedEnvironmentStorageNotFound. The factory deployer owns creation and
// mounting of deploy.data_dir; repository code must only delegate the exact
// product contract to it.
for (const directCloudMutation of ["az acr", "az containerapp", "az rest", "storageName:"]) {
  if (deployScript.includes(directCloudMutation)) {
    throw new Error(`The product deploy wrapper must not perform direct cloud mutation: ${directCloudMutation}`);
  }
}

const fixtureDirectory = await mkdtemp(join(tmpdir(), "booking-recovery-deploy-"));
try {
  const fakeFleetDeployer = join(fixtureDirectory, "fleet-deploy.sh");
  const capturePath = join(fixtureDirectory, "capture.txt");
  await writeFile(
    fakeFleetDeployer,
    '#!/bin/sh\nprintf "%s\\n" "$WO_DATA_DIR" "$@" > "$DEPLOY_CAPTURE"\n',
    "utf8",
  );
  await chmod(fakeFleetDeployer, 0o755);
  await execFileAsync(deployScriptUrl.pathname, [], {
    cwd: root.pathname,
    env: {
      ...process.env,
      DEPLOY_CAPTURE: capturePath,
      FACTORY_CONTAINER_DEPLOYER: fakeFleetDeployer,
      PREBUILT_IMAGE: "registry.invalid/sf-booking-recovery-loop:test",
    },
  });
  const delegated = (await readFile(capturePath, "utf8")).trim().split("\n");
  const expected = [
    "/data",
    "booking-recovery-loop",
    root.pathname.replace(/\/$/, ""),
    "Dockerfile",
    "8080",
    "registry.invalid/sf-booking-recovery-loop:test",
  ];
  if (JSON.stringify(delegated) !== JSON.stringify(expected)) {
    throw new Error(`Factory deployment delegation mismatch: ${JSON.stringify(delegated)}`);
  }
} finally {
  await rm(fixtureDirectory, { recursive: true, force: true });
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
