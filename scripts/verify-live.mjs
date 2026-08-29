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

await page.goto(`${baseURL}/start`, { waitUntil: "networkidle" });
const slug = `live-check-${Date.now()}`;
await page.getByLabel("Practice name").fill("Live Verification Practice");
await page.getByLabel("Booking link").fill(slug);
await page.getByLabel("Hosted deposit URL").fill("https://payments.example.test/session");
await page.getByLabel("Delivery service").selectOption("resend");
await page.getByRole("button", { name: "Create practice workspace" }).click();
await page.waitForURL(`${baseURL}/app`);
checks.practiceCreated = await page.getByRole("heading", { name: "Review bookings that need action" }).isVisible();
const ownerToken = await page.evaluate(() => localStorage.getItem("practice:access-token"));
checks.ownerTokenIssued = ownerToken?.startsWith("owner_") ?? false;
await page.getByRole("link", { name: "Open public booking page" }).click();
await page.waitForURL(`${baseURL}/b/${slug}`);
checks.publicPageTitle = await page.title();
await page.route("https://payments.example.test/**", (route) => route.fulfill({ contentType: "text/html", body: "<title>Hosted payment</title><h1>Hosted payment</h1>" }));
await page.getByLabel("Your name").fill("Verification Client");
await page.getByLabel("Email address").fill("verification@example.test");
await page.getByLabel("I give email consent for this booking").check();
await page.getByRole("button", { name: "Save booking and open payment" }).click();
await page.waitForURL("https://payments.example.test/session");
checks.hostedPaymentOpened = true;

const practiceResponse = await context.request.get(`${baseURL}/api/v1/practice`, { headers: { Authorization: `Bearer ${ownerToken}` } });
const practice = await practiceResponse.json();
checks.consentRecorded = practice.attempts?.[0]?.emailConsent === true && practice.attempts?.[0]?.state === "awaiting_deposit";
const exportResponse = await context.request.get(`${baseURL}/api/v1/practice/export`, { headers: { Authorization: `Bearer ${ownerToken}` } });
checks.exportStatus = exportResponse.status();
const deleteResponse = await context.request.delete(`${baseURL}/api/v1/practice`, { headers: { Authorization: `Bearer ${ownerToken}` } });
checks.deleteStatus = deleteResponse.status();
const deletedResponse = await context.request.get(`${baseURL}/api/v1/practice`, { headers: { Authorization: `Bearer ${ownerToken}` } });
checks.deletedKeyStatus = deletedResponse.status();

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
const passed = checks.homeStatus === 200 && checks.demoTickets === 3 && checks.demoResetRotatedToken && checks.demoSameOrigin && checks.practiceCreated && checks.ownerTokenIssued && checks.hostedPaymentOpened && checks.consentRecorded && checks.exportStatus === 200 && checks.deleteStatus === 204 && checks.deletedKeyStatus === 401 && checks.routes["/not-a-real-place"] === 404 && checks.mobileNoOverflow && checks.rateLimitPassed && consoleErrors.length === 0;
await writeFile(`${evidenceDir}/live-check.json`, JSON.stringify({ baseURL, passed, checks }, null, 2));
await browser.close();
console.log(JSON.stringify({ passed, checks }, null, 2));
if (!passed) process.exit(1);
