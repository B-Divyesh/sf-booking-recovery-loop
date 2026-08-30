import AxeBuilder from "@axe-core/playwright";
import { chromium, request } from "@playwright/test";
import { writeFile } from "node:fs/promises";

const base = "https://booking-recovery-loop.sociobot.in";
const routes = ["/", "/demo", "/start", "/app", "/app/settings/data", "/privacy", "/terms", "/missing-page"];
const browser = await chromium.launch({ headless: true });
const report = { routes: [], keyboard: {}, reducedMotion: {}, privacy: {}, headers: {}, auth: {}, api: {} };

for (const viewport of [{ name: "desktop", width: 1440, height: 1000 }, { name: "mobile", width: 390, height: 844 }]) {
  for (let index = 0; index < routes.length; index += 1) {
    const path = routes[index];
    const context = await browser.newContext({
      viewport,
      extraHTTPHeaders: { "X-Forwarded-For": `198.51.${viewport.name === "desktop" ? 31 : 32}.${index + 10}` }
    });
    const page = await context.newPage();
    const consoleErrors = [];
    const pageErrors = [];
    page.on("console", message => { if (message.type() === "error") consoleErrors.push(message.text()); });
    page.on("pageerror", error => pageErrors.push(String(error)));
    const response = await page.goto(`${base}${path}`, { waitUntil: "networkidle", timeout: 60_000 });
    const axe = await new AxeBuilder({ page }).analyze();
    const details = await page.evaluate(() => {
      const visible = element => {
        const box = element.getBoundingClientRect();
        const style = getComputedStyle(element);
        return box.width > 0 && box.height > 0 && style.visibility !== "hidden" && style.display !== "none";
      };
      const smallTargets = [...document.querySelectorAll("a, button, input, select, textarea")]
        .filter(visible)
        .map(element => {
          const box = element.getBoundingClientRect();
          return { text: (element.textContent || element.getAttribute("aria-label") || element.getAttribute("name") || "").trim(), width: box.width, height: box.height };
        })
        .filter(item => item.width < 44 || item.height < 44);
      return {
        title: document.title,
        lang: document.documentElement.lang,
        h1Count: document.querySelectorAll("h1").length,
        h1: document.querySelector("h1")?.textContent?.trim(),
        mainCount: document.querySelectorAll("main").length,
        missingAlt: document.querySelectorAll("img:not([alt])").length,
        overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
        smallTargets
      };
    });
    report.routes.push({ viewport: viewport.name, path, status: response?.status(), ...details,
      seriousCritical: axe.violations.filter(item => ["serious", "critical"].includes(item.impact || "")).map(item => ({ id: item.id, impact: item.impact, nodes: item.nodes.length })),
      consoleErrors, pageErrors });
    await context.close();
  }
}

{
  const context = await browser.newContext({ viewport: { width: 390, height: 844 }, extraHTTPHeaders: { "X-Forwarded-For": "198.51.40.10" } });
  const page = await context.newPage();
  await page.goto(`${base}/demo`, { waitUntil: "networkidle" });
  await page.keyboard.press("Tab");
  const skip = page.getByRole("link", { name: "Skip to main content" });
  report.keyboard.skipFocused = await skip.evaluate(element => element === document.activeElement);
  report.keyboard.skipFocusStyle = await skip.evaluate(element => {
    const style = getComputedStyle(element); return { outline: style.outline, outlineOffset: style.outlineOffset };
  });
  await page.keyboard.press("Enter");
  report.keyboard.mainFocused = await page.locator("main").evaluate(element => element === document.activeElement);
  const jordan = page.locator('[data-action="select-attempt"]').filter({ hasText: "Jordan Lee" });
  await jordan.focus();
  await page.keyboard.press("Space");
  const permission = page.getByRole("button", { name: "Check recovery permission" });
  await permission.focus();
  await page.keyboard.press("Enter");
  report.keyboard.consentMessage = await page.locator(".inline-notice").innerText();
  report.keyboard.noReceipt = await page.locator(".receipt-block").innerText();
  await context.close();
}

{
  const context = await browser.newContext({ viewport: { width: 390, height: 844 }, reducedMotion: "reduce", extraHTTPHeaders: { "X-Forwarded-For": "198.51.40.11" } });
  const page = await context.newPage();
  await page.goto(`${base}/demo`, { waitUntil: "networkidle" });
  report.reducedMotion.mediaMatches = await page.evaluate(() => matchMedia("(prefers-reduced-motion: reduce)").matches);
  report.reducedMotion.longRunning = await page.evaluate(() => [...document.querySelectorAll("*")].map(element => {
    const style = getComputedStyle(element);
    return { animation: style.animationDuration, transition: style.transitionDuration, transform: style.transform };
  }).filter(item => item.animation.split(",").some(value => parseFloat(value) > 0.12) || item.transition.split(",").some(value => parseFloat(value) > 0.12)));
  await context.close();
}

{
  const context = await browser.newContext({ viewport: { width: 390, height: 844 }, extraHTTPHeaders: { "X-Forwarded-For": "198.51.40.12" } });
  const page = await context.newPage();
  const requests = [];
  const responses = [];
  page.on("request", req => requests.push({ method: req.method(), url: req.url() }));
  page.on("response", res => responses.push({ status: res.status(), url: res.url(), headers: res.headers() }));
  await page.goto(`${base}/?demo=1`, { waitUntil: "networkidle" });
  await page.locator('[data-action="select-attempt"]').filter({ hasText: "Maya Patel" }).click();
  await page.getByRole("button", { name: "Run sample follow-up" }).click();
  await page.getByText("Delivered · simulated email").waitFor();
  await page.getByRole("button", { name: "Reset demo" }).click();
  await page.getByText("Demo reset. The original sample bookings are ready.").waitFor();
  await page.evaluate(() => { document.documentElement.style.fontSize = "200%"; });
  report.privacy = {
    requests,
    allSameOrigin: requests.filter(item => item.url.startsWith("http")).every(item => new URL(item.url).origin === new URL(base).origin),
    responseHeaders: responses.map(item => ({ status: item.status, url: item.url, cacheControl: item.headers["cache-control"], csp: item.headers["content-security-policy"], referrerPolicy: item.headers["referrer-policy"] })),
    localStorage: await page.evaluate(() => Object.keys(localStorage)),
    sessionStorage: await page.evaluate(() => Object.keys(sessionStorage)),
    cookies: await context.cookies(),
    zoom200Overflow: await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)
  };
  await page.screenshot({ path: ".factory/verification-artifacts/live/demo-mobile-200.png", fullPage: true });
  await context.close();
}

{
  const client = await request.newContext();
  for (const path of ["/", "/api/v1/integrations/status", "/missing-page", "/robots.txt", "/sitemap.xml", "/assets/index-Bqqmy4wO.js", "/fonts/atkinson-next-latin-variable.woff2"]) {
    const response = await client.get(`${base}${path}`);
    report.headers[path] = { status: response.status(), headers: response.headers() };
  }
  const httpResponse = await client.get("http://booking-recovery-loop.sociobot.in", { maxRedirects: 0 });
  report.headers.httpRedirect = { status: httpResponse.status(), location: httpResponse.headers().location };
  const unauthenticated = await client.get(`${base}/api/v1/practice`, { headers: { "X-Forwarded-For": "198.51.40.13" } });
  report.api.unauthenticated = { status: unauthenticated.status(), headers: unauthenticated.headers(), body: await unauthenticated.text() };
  const deposit = await client.get("https://api.sociobot.in/api/v1/products/booking-recovery-loop-deposit/checkout", { maxRedirects: 0 });
  report.api.depositCheckout = { status: deposit.status(), headers: deposit.headers(), body: await deposit.text() };
  await client.dispose();
}

{
  const context = await browser.newContext({ viewport: { width: 390, height: 844 }, extraHTTPHeaders: { "X-Forwarded-For": "198.51.40.14" } });
  const page = await context.newPage();
  await page.goto(`${base}/`, { waitUntil: "networkidle" });
  await page.getByRole("button", { name: "Sign in" }).click();
  await page.waitForURL(url => url.hostname.endsWith("ciamlogin.com"), { timeout: 30_000 });
  const authUrl = new URL(page.url());
  report.auth = {
    host: authUrl.hostname,
    tenantInPath: authUrl.pathname.includes("35c6fe40-0ec0-46b6-98c6-213ad4de6650"),
    clientId: authUrl.searchParams.get("client_id"),
    redirectUri: authUrl.searchParams.get("redirect_uri"),
    responseType: authUrl.searchParams.get("response_type"),
    codeChallengeMethod: authUrl.searchParams.get("code_challenge_method"),
    scope: authUrl.searchParams.get("scope")
  };
  await context.close();
}

await writeFile(".factory/verification-artifacts/live-audit.json", JSON.stringify(report, null, 2));
console.log(JSON.stringify({
  routeFailures: report.routes.filter(item => item.seriousCritical.length || item.missingAlt || item.h1Count !== 1 || item.mainCount !== 1 || item.lang !== "en" || item.overflow > 0 || item.pageErrors.length),
  smallTargetCounts: report.routes.map(item => ({ viewport: item.viewport, path: item.path, count: item.smallTargets.length })),
  keyboard: report.keyboard,
  reducedMotion: report.reducedMotion,
  privacy: { allSameOrigin: report.privacy.allSameOrigin, localStorage: report.privacy.localStorage, sessionStorage: report.privacy.sessionStorage, cookies: report.privacy.cookies, zoom200Overflow: report.privacy.zoom200Overflow },
  headers: report.headers,
  auth: report.auth,
  api: report.api
}, null, 2));
await browser.close();
