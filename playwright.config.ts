import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/e2e",
  testMatch: "**/*.spec.ts",
  fullyParallel: false,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [["html", { open: "never" }], ["list"]] : "list",
  workers: 1,
  use: {
    baseURL: "http://127.0.0.1:4173",
    trace: "retain-on-failure",
    screenshot: "only-on-failure"
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] }
    }
  ],
  webServer: [
    {
      command: "node tests/fixtures/integration-server.mjs",
      url: "http://127.0.0.1:4174/health",
      env: { INTEGRATION_FIXTURE_PORT: "4174" },
      reuseExistingServer: !process.env.CI,
      timeout: 30_000
    },
    {
      command: "npm run test:server",
      url: "http://127.0.0.1:4173",
      env: {
        PORT: "4173",
        DATABASE_URL: "sqlite://booking-recovery-loop-e2e-v3.db?mode=rwc",
        STATIC_DIR: "dist",
        TEST_ENTRA_OID: "playwright-sociobot-entra-user",
        SOCIOBOT_BILLING_BASE_URL: "http://127.0.0.1:4174",
        SOCIOBOT_BOOKING_PRODUCT_SLUG: "booking-recovery-loop-deposit"
      },
      reuseExistingServer: !process.env.CI,
      // A clean Rust toolchain can spend several minutes compiling sqlx. Claims
      // are required to pass from a cold clone, so the harness must wait for the
      // product rather than timing out while it is still building.
      timeout: 360_000
    }
  ]
});
