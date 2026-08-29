import { expect, test } from "@playwright/test";

import { setForwardedClient } from "./client-identity";

test.beforeEach(async ({ page }, testInfo) => {
  await setForwardedClient(page, testInfo);
});

const demoTokenKey = "demo:workspace-token";

async function openReadyDemo(page: import("@playwright/test").Page) {
  await page.goto("/demo");
  await expect(page.getByRole("heading", { name: "Recover a sample booking" })).toBeVisible();
  await expect(page.locator('[data-action="select-attempt"]')).toHaveCount(3);
}

function ticketFor(page: import("@playwright/test").Page, clientName: string) {
  return page.locator('[data-action="select-attempt"]').filter({ hasText: clientName });
}

test("browser demo requests remain scoped to their sample token", async ({ page }) => {
  const apiRequests: Array<{ url: string; token?: string }> = [];
  page.on("request", (request) => {
    if (request.url().includes("/api/v1/demo/")) {
      apiRequests.push({
        url: request.url(),
        token: request.headers()["x-demo-workspace"]
      });
    }
  });

  await openReadyDemo(page);
  const token = await page.evaluate((key) => localStorage.getItem(key), demoTokenKey);
  expect(token).toMatch(/^v1\.[A-Za-z0-9_-]{43}\.\d{10}\.fresh$/);
  expect(await page.evaluate(() => Object.keys(localStorage))).toEqual([demoTokenKey]);
  await ticketFor(page, "Maya Patel").click();
  await page.getByRole("button", { name: "Run sample follow-up" }).click();
  await expect(page.getByText("Sample follow-up delivered. A simulated receipt was added.")).toBeVisible();

  const workspaceRequests = apiRequests.filter(({ url }) => !url.endsWith("/workspaces"));
  expect(workspaceRequests.length).toBeGreaterThan(0);
  expect(workspaceRequests.every((request) => request.token === token)).toBe(true);
  await expect(page.getByText("Private Practice")).toHaveCount(0);
});

test("browser displays the server-issued 24-hour workspace token", async ({ page }) => {
  await openReadyDemo(page);
  const token = await page.evaluate((key) => localStorage.getItem(key), demoTokenKey);
  expect(token).toMatch(/^v1\.[A-Za-z0-9_-]{43}\.\d{10}\.fresh$/);
  const createdSeconds = Number(token?.split(".")[2]);
  const expires = await page.locator("body").evaluate(() => {
    const stored = localStorage.getItem("demo:workspace-token");
    return Number(stored?.split(".")[2]) + 24 * 60 * 60;
  });
  expect(expires - createdSeconds).toBe(24 * 60 * 60);
});

test("@claim:demo-no-account-payment opens without account or payment", async ({ page }) => {
  const requests: string[] = [];
  page.on("request", (request) => requests.push(request.url()));
  await page.goto("/");
  await expect(page.getByText("Demo needs no account")).toBeVisible();
  await page.getByRole("link", { name: "Try it with sample data" }).click();
  await expect(page.locator('[data-action="select-attempt"]')).toHaveCount(3);
  expect(requests.some((url) => /login|checkout|payment/i.test(url))).toBe(false);
});

test("@claim:demo-reset restores fresh sample bookings", async ({ page, request }) => {
  await openReadyDemo(page);
  const oldToken = await page.evaluate((key) => localStorage.getItem(key), demoTokenKey);
  await ticketFor(page, "Maya Patel").click();
  await page.getByRole("button", { name: "Run sample follow-up" }).click();
  await expect(ticketFor(page, "Maya Patel")).toContainText("Recovered in demo");
  const activeToken = await page.evaluate((key) => localStorage.getItem(key), demoTokenKey);

  await Promise.all([
    page.waitForResponse((response) => response.url().endsWith("/api/v1/demo/reset")),
    page.getByRole("button", { name: "Reset demo" }).click()
  ]);
  await expect(page.getByText("Demo reset. The original sample bookings are ready.")).toBeVisible();
  await expect(ticketFor(page, "Maya Patel")).toContainText("Needs a follow-up");
  const newToken = await page.evaluate((key) => localStorage.getItem(key), demoTokenKey);
  expect(newToken).not.toBe(oldToken);
  expect(newToken).not.toBe(activeToken);
  const oldWorkspace = await request.get("/api/v1/demo/workspace", {
    headers: { "X-Demo-Workspace": activeToken ?? "" }
  });
  expect(oldWorkspace.status()).toBe(404);
});

test("@claim:consent-gates-recovery stops a message without consent", async ({ page }) => {
  await openReadyDemo(page);
  await ticketFor(page, "Jordan Lee").click();
  await expect(page.getByRole("heading", { name: "Jordan Lee" })).toBeVisible();
  await page.getByRole("button", { name: "Check recovery permission" }).click();
  await expect(page.locator(".inline-notice")).toHaveText("No email consent was recorded. This recovery stays stopped.");
  await expect(page.locator(".receipt-block")).toContainText("No receipt yet");

  await page.reload();
  await expect(page.locator('[data-action="select-attempt"]')).toHaveCount(3);
  await ticketFor(page, "Jordan Lee").click();
  await expect(page.locator(".receipt-block")).toContainText("No receipt yet");
});

test("@claim:demo-recovery-receipt adds a timestamped simulated receipt", async ({ page }) => {
  await openReadyDemo(page);
  await ticketFor(page, "Maya Patel").click();
  await page.getByRole("button", { name: "Run sample follow-up" }).click();
  const receipt = page.locator(".receipt-timeline li");
  await expect(receipt).toContainText("Delivered · simulated email");
  await expect(receipt.locator("time")).toHaveAttribute("datetime", /T\d{2}:\d{2}:\d{2}Z$/);
  await expect(ticketFor(page, "Maya Patel")).toContainText("Recovered in demo");
});

test("@claim:demo-no-external-requests keeps the full demo flow same-origin", async ({ page, baseURL }) => {
  const requestUrls: string[] = [];
  page.on("request", (request) => requestUrls.push(request.url()));
  await openReadyDemo(page);
  await ticketFor(page, "Maya Patel").click();
  await page.getByRole("button", { name: "Run sample follow-up" }).click();
  await expect(page.locator(".receipt-timeline li")).toBeVisible();
  await page.getByRole("button", { name: "Reset demo" }).click();
  await expect(page.getByText("Demo reset. The original sample bookings are ready.")).toBeVisible();

  const expectedOrigin = new URL(baseURL ?? "http://127.0.0.1:4173").origin;
  const networkUrls = requestUrls.filter((url) => url.startsWith("http"));
  expect(networkUrls.length).toBeGreaterThan(0);
  expect(networkUrls.every((url) => new URL(url).origin === expectedOrigin)).toBe(true);
});

test("@claim:sample-three-bookings opens one-click sample data", async ({ page }) => {
  await page.goto("/?demo=1");
  await expect(page.getByText("Demo — sample data, nothing is saved")).toBeVisible();
  await expect(page.locator('[data-action="select-attempt"]')).toHaveCount(3);
  await expect(ticketFor(page, "Maya Patel")).toContainText("Needs a follow-up");
});

test("@claim:practice-plan-price shows the priced plan and current checkout state", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("Recovery Loop Practice is $29 per month for one practice.")).toBeVisible();
  await expect(page.getByText("Checkout is not available yet. You can create a workspace and test the delivery connection now.")).toBeVisible();
  await expect(page.getByRole("link", { name: "Set up your practice" }).last()).toHaveAttribute("href", "/start");
});

test("@claim:card-data-excluded has no card fields before hosted payment", async ({ page, request }) => {
  const slug = `claim-card-data-${Date.now()}`;
  const created = await request.post("/api/v1/practices", { headers: { "X-Forwarded-For": "203.0.113.171" }, data: {
    name: "North Star Coaching", publicSlug: slug, timezone: "Europe/London", serviceName: "Focus session",
    durationMinutes: 45, depositCents: 3500, currency: "GBP", paymentUrl: "https://example.com/hosted-payment", deliveryWebhookUrl: ""
  }});
  expect(created.status()).toBe(201);
  let submitted: Record<string, unknown> | null = null;
  page.on("request", (request) => {
    if (request.url().includes(`/api/v1/public/${slug}/attempts`)) submitted = request.postDataJSON() as Record<string, unknown>;
  });
  await page.route("https://example.com/**", (route) => route.fulfill({ contentType: "text/html", body: "<title>Hosted payment</title><h1>Hosted payment</h1>" }));
  await page.goto(`/b/${slug}`);
  await expect(page.locator('input[name*="card" i], input[autocomplete^="cc-"]')).toHaveCount(0);
  await page.getByLabel("Your name").fill("Taylor Reed");
  await page.getByLabel("Email address").fill("taylor@example.test");
  await page.getByLabel("I give email consent for this booking").check();
  await page.getByRole("button", { name: "Save booking and open payment" }).click();
  await expect(page).toHaveURL("https://example.com/hosted-payment");
  expect(submitted).not.toBeNull();
  expect(Object.keys(submitted ?? {}).some((key) => /card|pan|cvc|cvv|expiry/i.test(key))).toBe(false);
});

test("@claim:practice-publish creates a private workspace and public page", async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on("console", (message) => { if (message.type() === "error") consoleErrors.push(message.text()); });
  const slug = `claim-practice-${Date.now()}`;
  await page.goto("/start");
  await page.getByLabel("Booking link").fill(slug);
  await page.getByLabel("Hosted deposit URL").fill("https://payments.example.test/session");
  await page.getByLabel("Delivery service").selectOption("resend");
  await page.getByRole("button", { name: "Create practice workspace" }).click();
  await expect(page).toHaveURL(/\/app$/);
  await expect(page.getByRole("heading", { name: "Review bookings that need action" })).toBeVisible();
  const token = await page.evaluate((key) => localStorage.getItem(key), "practice:access-token");
  expect(token).toMatch(/^owner_/);
  await page.getByRole("link", { name: "Open public booking page" }).click();
  await expect(page).toHaveURL(new RegExp(`/b/${slug}$`));
  await expect(page.getByRole("heading", { name: "Finish your paid session booking" })).toBeVisible();
  await expect(page.getByText("45-minute focus session")).toBeVisible();
  expect(consoleErrors).toEqual([]);
});

test("@claim:booking-consent-record saves consent before hosted payment", async ({ page, request }) => {
  const slug = `claim-booking-${Date.now()}`;
  const clientIp = `198.51.100.${Math.floor(Math.random() * 100) + 100}`;
  const created = await request.post("/api/v1/practices", { headers: { "X-Forwarded-For": clientIp }, data: {
    name: "North Star Coaching", publicSlug: slug, timezone: "Europe/London", serviceName: "45-minute focus session",
    durationMinutes: 45, depositCents: 3500, currency: "GBP", paymentUrl: "https://example.com/hosted-payment", deliveryWebhookUrl: ""
  }});
  expect(created.status()).toBe(201);
  const owner = await created.json();
  await page.route("https://example.com/**", (route) => route.fulfill({ contentType: "text/html", body: "<title>Hosted payment</title><h1>Hosted payment</h1>" }));
  await page.goto(`/b/${slug}`);
  await page.getByLabel("Your name").fill("Taylor Reed");
  await page.getByLabel("Email address").fill("taylor@example.test");
  await page.getByLabel("I give email consent for this booking").check();
  await page.getByRole("button", { name: "Save booking and open payment" }).click();
  await expect(page).toHaveURL("https://example.com/hosted-payment");
  const practiceResponse = await request.get("/api/v1/practice", { headers: { Authorization: `Bearer ${owner.accessToken}`, "X-Forwarded-For": clientIp } });
  const practice = await practiceResponse.json();
  expect(practice.attempts).toHaveLength(1);
  expect(practice.attempts[0]).toMatchObject({ clientName: "Taylor Reed", emailConsent: true, smsConsent: false, state: "awaiting_deposit" });
  expect(practice.attempts[0].consentRecordedAt).toMatch(/Z$/);
});
