import { expect, test } from "@playwright/test";

const demoTokenKey = "demo:workspace-token";

async function openReadyDemo(page: import("@playwright/test").Page) {
  await page.goto("/demo");
  await expect(page.getByRole("heading", { name: "Recover a sample booking" })).toBeVisible();
  await expect(page.locator('[data-action="select-attempt"]')).toHaveCount(3);
}

function ticketFor(page: import("@playwright/test").Page, clientName: string) {
  return page.locator('[data-action="select-attempt"]').filter({ hasText: clientName });
}

test("@claim:demo-isolated keeps the sample in its own workspace", async ({ page }) => {
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
  expect(token).toMatch(/^[A-Za-z0-9_-]{43}$/);
  expect(await page.evaluate(() => Object.keys(localStorage))).toEqual([demoTokenKey]);
  await ticketFor(page, "Maya Patel").click();
  await page.getByRole("button", { name: "Run sample follow-up" }).click();
  await expect(page.getByText("Sample follow-up delivered. A simulated receipt was added.")).toBeVisible();

  const workspaceRequests = apiRequests.filter(({ url }) => !url.endsWith("/workspaces"));
  expect(workspaceRequests.length).toBeGreaterThan(0);
  expect(workspaceRequests.every((request) => request.token === token)).toBe(true);
  await expect(page.getByText("Private Practice")).toHaveCount(0);
});

test("@claim:demo-reset restores fresh sample bookings", async ({ page }) => {
  await openReadyDemo(page);
  const oldToken = await page.evaluate((key) => localStorage.getItem(key), demoTokenKey);
  await ticketFor(page, "Maya Patel").click();
  await page.getByRole("button", { name: "Run sample follow-up" }).click();
  await expect(ticketFor(page, "Maya Patel")).toContainText("Recovered in demo");

  await Promise.all([
    page.waitForResponse((response) => response.url().endsWith("/api/v1/demo/reset")),
    page.getByRole("button", { name: "Reset demo" }).click()
  ]);
  await expect(page.getByText("Demo reset. The original sample bookings are ready.")).toBeVisible();
  await expect(ticketFor(page, "Maya Patel")).toContainText("Needs a follow-up");
  const newToken = await page.evaluate((key) => localStorage.getItem(key), demoTokenKey);
  expect(newToken).not.toBe(oldToken);
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
