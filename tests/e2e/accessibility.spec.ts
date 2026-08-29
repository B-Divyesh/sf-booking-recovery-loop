import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

import { setForwardedClient } from "./client-identity";

test.beforeEach(async ({ page }, testInfo) => {
  await setForwardedClient(page, testInfo);
});

const routes = [
  ["/", "Booking Recovery Loop — recover unfinished bookings"],
  ["/demo", "Demo — Booking Recovery Loop"],
  ["/start", "Start a practice — Booking Recovery Loop"],
  ["/app", "Recovery queue — Booking Recovery Loop"],
  ["/app/settings/data", "Data controls — Booking Recovery Loop"],
  ["/privacy", "Privacy — Booking Recovery Loop"],
  ["/terms", "Terms — Booking Recovery Loop"],
  ["/missing-page", "Page not found — Booking Recovery Loop"]
] as const;

for (const [path, title] of routes) {
  test(`${path} has route metadata and no serious accessibility violations`, async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (message) => {
      if (message.type() === "error") errors.push(message.text());
    });
    await page.goto(path);
    if (path === "/demo") {
      await expect(page.locator('[data-action="select-attempt"]')).toHaveCount(3);
    }
    await expect(page).toHaveTitle(title);
    await expect(page.locator("main")).toHaveCount(1);
    await expect(page.locator("h1")).toHaveCount(1);
    await expect(page.locator('img:not([alt])')).toHaveCount(0);
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""))).toEqual([]);
    if (path === "/missing-page") {
      expect(errors).toEqual([expect.stringContaining("404")]);
    } else {
      expect(errors).toEqual([]);
    }
  });
}

test("keyboard users can skip, select, and run the recovery", async ({ page }) => {
  await openDemo(page);
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "Skip to main content" })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.locator("main")).toBeFocused();

  const maya = page.locator('[data-action="select-attempt"]').filter({ hasText: "Maya Patel" });
  await maya.focus();
  await page.keyboard.press("Space");
  const recovery = page.getByRole("button", { name: "Run sample follow-up" });
  await recovery.focus();
  await page.keyboard.press("Enter");
  await expect(page.locator(".receipt-timeline li")).toContainText("Delivered");
});

test("expected consent stop does not create a console error", async ({ page }) => {
  const errors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  await openDemo(page);
  await page.locator('[data-action="select-attempt"]').filter({ hasText: "Jordan Lee" }).click();
  await page.getByRole("button", { name: "Check recovery permission" }).click();
  await expect(page.locator(".inline-notice")).toContainText("stays stopped");
  expect(errors).toEqual([]);
});

test("390px at 200 percent text reflows and keeps 44px footer targets", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await page.evaluate(() => {
    document.documentElement.style.fontSize = "200%";
  });
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
  for (const link of await page.locator("footer a").all()) {
    const box = await link.boundingBox();
    expect(box?.width).toBeGreaterThanOrEqual(44);
    expect(box?.height).toBeGreaterThanOrEqual(44);
  }
});

test("unknown pages return 404 and immutable assets are cached", async ({ request, page }) => {
  const missing = await request.get("/missing-page");
  expect(missing.status()).toBe(404);
  await page.goto("/");
  const assetPath = await page.locator('script[type="module"]').getAttribute("src");
  expect(assetPath).toBeTruthy();
  const asset = await request.get(assetPath ?? "");
  expect(asset.headers()["cache-control"]).toBe("public, max-age=31536000, immutable");
});

test("security response policy and offline error state are explicit", async ({ page, request, context }) => {
  const response = await request.get("/");
  expect(response.headers()["content-security-policy"]).toContain("frame-ancestors 'none'");
  expect(response.headers()["x-content-type-options"]).toBe("nosniff");
  expect(response.headers()["referrer-policy"]).toBe("strict-origin-when-cross-origin");
  await page.goto("/");
  expect(await page.evaluate(() => navigator.serviceWorker.getRegistrations().then((items) => items.length))).toBe(0);
  await context.setOffline(true);
  await page.getByRole("link", { name: "Try it with sample data" }).click();
  await expect(page.getByRole("heading", { name: "The demo is offline" })).toBeVisible();
});

test("history navigation restores the route and focuses its heading", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("link", { name: "Set up", exact: true }).click();
  await expect(page).toHaveURL(/\/start$/);
  await expect(page.getByRole("heading", { level: 1 })).toBeFocused();
  await page.goBack();
  await expect(page).toHaveURL(/\/$/);
  await expect(page.getByRole("heading", { level: 1 })).toBeFocused();
});

test("public booking page is accessible and reflows at 390px", async ({ page, request }, testInfo) => {
  const slug = `a11y-booking-${Date.now()}`;
  const ip = `203.0.113.${testInfo.workerIndex + 120}`;
  const created = await request.post("/api/v1/practices", { headers: { "X-Forwarded-For": ip, "X-Test-Oid": "playwright-sociobot-entra-user" }, data: {
    name: "North Star Coaching", publicSlug: slug, timezone: "Europe/London", serviceName: "Focus session",
    durationMinutes: 45, depositCents: 3500, currency: "GBP", paymentUrl: "https://example.com/pay", deliveryWebhookUrl: ""
  }});
  expect(created.status()).toBe(201);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`/b/${slug}`);
  await expect(page.getByRole("heading", { name: "Finish your paid session booking" })).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""))).toEqual([]);
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
  const button = await page.getByRole("button", { name: "Save booking and open payment" }).boundingBox();
  expect(button?.height).toBeGreaterThanOrEqual(44);
});

async function openDemo(page: import("@playwright/test").Page) {
  await page.goto("/?demo=1");
  await expect(page.locator('[data-action="select-attempt"]')).toHaveCount(3);
}
