import { chromium } from "@playwright/test";
import { mkdir, writeFile } from "node:fs/promises";

const baseURL = process.argv[2] ?? "https://booking-recovery-loop.sociobot.in";
const evidenceDir = process.argv[3] ?? ".factory/evidence/polish-1-live";
await mkdir(evidenceDir, { recursive: true });

const browser = await chromium.launch();
const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
const page = await context.newPage();
const consoleErrors = [];
const demoRequests = [];
page.on("console", (message) => { if (message.type() === "error") consoleErrors.push(message.text()); });
page.on("request", (request) => { if (page.url().includes("demo=1")) demoRequests.push(request.url()); });

const checks = {};
const response = await page.goto(`${baseURL}/`, { waitUntil: "networkidle" });
checks.homeStatus = response?.status();
checks.homeTitle = await page.title();
checks.homeH1 = await page.locator("h1").textContent();
await page.screenshot({ path: `${evidenceDir}/home-desktop.png`, fullPage: true });

await page.goto(`${baseURL}/?demo=1`, { waitUntil: "networkidle" });
checks.demoTitle = await page.title();
checks.demoTickets = await page.locator('[data-action="select-attempt"]').count();
checks.demoBanner = await page.locator(".demo-banner").innerText();
const firstToken = await page.evaluate(() => localStorage.getItem("demo:workspace-token"));
await page.locator('[data-action="select-attempt"]').filter({ hasText: "Maya Patel" }).click();
await page.getByRole("button", { name: "Run sample follow-up" }).click();
await page.getByText("Delivered · simulated email").waitFor();
checks.demoReceipt = await page.locator(".receipt-timeline li").first().innerText();
await page.getByRole("button", { name: "Reset demo" }).click();
await page.getByText("Demo reset. The original sample bookings are ready.").waitFor();
const secondToken = await page.evaluate(() => localStorage.getItem("demo:workspace-token"));
checks.demoResetRotatedToken = firstToken !== secondToken;
checks.demoSameOrigin = demoRequests.filter((url) => url.startsWith("http")).every((url) => new URL(url).origin === new URL(baseURL).origin);
await page.screenshot({ path: `${evidenceDir}/demo-desktop.png`, fullPage: true });

const checkout = await context.request.get("https://api.sociobot.in/api/v1/products/booking-recovery-loop/checkout", { maxRedirects: 0 });
checks.subscriptionCheckoutStatus = checkout.status();
checks.startExplainsIdentity = await page.goto(`${baseURL}/start`, { waitUntil: "networkidle" }).then(() => page.getByText("Your Sociobot account owns the workspace across devices.").isVisible());

const routeResults = {};
for (const route of ["/", "/demo", "/start", "/app", "/app/settings/data", "/privacy", "/terms", "/not-a-real-place"]) {
  const routeResponse = await context.request.get(`${baseURL}${route}`);
  routeResults[route] = routeResponse.status();
}
checks.routes = routeResults;

const mobile = await context.newPage();
await mobile.setViewportSize({ width: 390, height: 844 });
await mobile.goto(`${baseURL}/`, { waitUntil: "networkidle" });
checks.mobileNoOverflow = await mobile.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth);
await mobile.screenshot({ path: `${evidenceDir}/home-mobile.png`, fullPage: true });

checks.consoleErrors = consoleErrors;
const rateClient = `198.51.100.${(Date.now() % 200) + 20}`;
const rateStatuses = [];
const retryAfter = [];
for (let i = 0; i < 13; i += 1) {
  const rateResponse = await context.request.post(`${baseURL}/api/v1/demo/workspaces`, {
    headers: { "X-Forwarded-For": rateClient, "Idempotency-Key": `live-rate-${Date.now()}-${i}` }
  });
  rateStatuses.push(rateResponse.status());
  retryAfter.push(rateResponse.headers()["retry-after"] ?? null);
}
checks.rateLimit = { client: rateClient, statuses: rateStatuses, retryAfter };
checks.rateLimitPassed = rateStatuses.slice(0, 12).every((status) => status === 201) && rateStatuses[12] === 429 && Number(retryAfter[12]) >= 1;
const passed = checks.homeStatus === 200 && checks.demoTickets === 3 && checks.demoResetRotatedToken && checks.demoSameOrigin && checks.subscriptionCheckoutStatus === 303 && checks.startExplainsIdentity && checks.routes["/not-a-real-place"] === 404 && checks.mobileNoOverflow && checks.rateLimitPassed && consoleErrors.length === 0;
await writeFile(`${evidenceDir}/live-check.json`, JSON.stringify({ baseURL, passed, checks }, null, 2));
await browser.close();
console.log(JSON.stringify({ passed, checks }, null, 2));
if (!passed) process.exit(1);
