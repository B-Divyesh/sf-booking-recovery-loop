import { readFile } from "node:fs/promises";

import { describe, expect, it } from "vitest";

type Claim = { id: string; test: string };
type PackageJson = { scripts?: Record<string, string> };

describe("clean-clone claims gate", () => {
  it("installs Playwright dependencies before every exact browser claim command", async () => {
    const [claimsFile, packageFile] = await Promise.all([
      readFile(new URL("../../.factory/claims.json", import.meta.url), "utf8"),
      readFile(new URL("../../package.json", import.meta.url), "utf8")
    ]);
    const claims = JSON.parse(claimsFile) as Claim[];
    const packageJson = JSON.parse(packageFile) as PackageJson;
    const browserClaims = claims.filter((claim) => claim.test.includes("@claim:"));

    expect(browserClaims).toHaveLength(9);
    expect(browserClaims.every((claim) => claim.test.startsWith("npm run test:claim:e2e --"))).toBe(true);
    expect(packageJson.scripts?.["test:claim:e2e"]).toMatch(/^npm ci --ignore-scripts && playwright test$/);
  });
});
