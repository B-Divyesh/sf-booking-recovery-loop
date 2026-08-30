import { readFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";

const claims = JSON.parse(
  await readFile(new URL("../.factory/claims.json", import.meta.url), "utf8"),
);
const failures = [];

for (const claim of claims) {
  process.stdout.write(`\n[claim:${claim.id}] ${claim.test}\n`);
  const result = spawnSync(claim.test, {
    cwd: new URL("../", import.meta.url),
    env: process.env,
    shell: true,
    stdio: "inherit",
  });
  if (result.status !== 0) failures.push(claim.id);
}

if (failures.length) {
  throw new Error(`Claim checks failed: ${failures.join(", ")}`);
}
console.log(`\nAll ${claims.length} claim commands passed individually.`);
