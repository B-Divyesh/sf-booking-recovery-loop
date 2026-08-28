import "./styles/tokens.css";
import "./styles/app.css";

import railSceneUrl from "./assets/appointment-rail.svg";
import {
  createDemo,
  DEMO_STORAGE_KEY,
  DemoApiError,
  loadDemo,
  recoverDemoAttempt,
  recoveryPermission,
  resetDemo,
  type DemoAttempt,
  type DemoEnvelope
} from "./lib/demo";
import { canonicalUrl, pageFor, routeFor, type SiteRoute } from "./lib/site";

const applicationRoot = document.querySelector<HTMLDivElement>("#app");
if (!applicationRoot) {
  throw new Error("The application root is missing.");
}
const app: HTMLDivElement = applicationRoot;

let demoEnvelope: DemoEnvelope | null = null;
let demoLoading = false;
let demoError: string | null = null;
let selectedAttemptId: string | null = null;
let workingAttemptId: string | null = null;
let demoNotice: string | null = null;

function homeContent(): string {
  return `
    <section class="hero" aria-describedby="hero-summary">
      <div class="hero-copy">
        <p class="eyebrow">Booking follow-up with proof</p>
        <h1 tabindex="-1">Recover paid sessions before they disappear</h1>
        <p id="hero-summary" class="lede">For solo coaches and tutors who need to see why a paid booking stopped and what can happen next.</p>
        <div class="hero-action">
          <a class="button button-primary" href="/demo">Try it with sample data</a>
          <p>Opens a safe workspace with three fictional clients.</p>
        </div>
        <ul class="plain-facts" aria-label="Demo facts">
          <li><span aria-hidden="true">01</span> No account needed</li>
          <li><span aria-hidden="true">02</span> No real messages sent</li>
          <li><span aria-hidden="true">03</span> No payment in the demo</li>
        </ul>
      </div>
      <figure class="hero-scene">
        <img src="${railSceneUrl}" width="920" height="620" fetchpriority="high" alt="A calm appointment rail showing one booking that needs a follow-up." />
        <figcaption>One booking stopped. Consent decides the next step.</figcaption>
      </figure>
    </section>

    <section class="product-preview section-rule" aria-labelledby="preview-title">
      <div class="section-intro">
        <p class="eyebrow">The product</p>
        <h2 id="preview-title">See the break in the booking loop</h2>
        <p>Each ticket keeps the booking state, permission, and delivery evidence together.</p>
      </div>
      <div class="preview-board" aria-label="Sample recovery board preview">
        <div class="preview-ticket preview-ticket-muted">
          <p class="ticket-time">Tue · 14:00</p>
          <h3>Booking started</h3>
          <p>Service and time chosen</p>
          <span class="status status-good">Recorded</span>
        </div>
        <div class="preview-connector" aria-hidden="true"></div>
        <div class="preview-ticket preview-ticket-active">
          <p class="ticket-time">18 minutes ago</p>
          <h3>Deposit not finished</h3>
          <p>Email consent is on record.</p>
          <span class="status status-attention">Needs a follow-up</span>
        </div>
        <div class="preview-connector" aria-hidden="true"></div>
        <div class="preview-ticket preview-ticket-muted">
          <p class="ticket-time">Next</p>
          <h3>Delivery receipt</h3>
          <p>Waiting for a permitted action</p>
          <span class="status status-neutral">Not started</span>
        </div>
      </div>
    </section>

    <section id="how-it-works" class="how-section section-rule" aria-labelledby="how-title">
      <div class="section-intro">
        <p class="eyebrow">How it works</p>
        <h2 id="how-title">Follow one accountable path</h2>
      </div>
      <ol class="process-rail">
        <li><span>1</span><div><h3>Find the stopped booking</h3><p>See the chosen session and where the client left.</p></div></li>
        <li><span>2</span><div><h3>Check permission first</h3><p>A follow-up stays stopped when contact consent is missing.</p></div></li>
        <li><span>3</span><div><h3>Keep the receipt</h3><p>The sample action ends with a labelled delivery record.</p></div></li>
      </ol>
    </section>

    <section class="boundary-section section-rule" aria-labelledby="boundary-title">
      <div>
        <p class="eyebrow">Clear boundaries</p>
        <h2 id="boundary-title">It does not replace your calendar</h2>
      </div>
      <div class="boundary-copy">
        <p>Booking Recovery Loop focuses on the steps after someone chooses a paid session.</p>
        <p>It is not a CRM, a marketplace, or a tool for bulk messages.</p>
        <a href="/privacy">Read how the sample handles data</a>
      </div>
    </section>

    <section id="practice-plan" class="plan-section section-rule" aria-labelledby="plan-title">
      <div>
        <p class="eyebrow">Practice plan</p>
        <h2 id="plan-title">Recovery Loop Practice</h2>
        <p class="plan-price"><strong>$29</strong> / month</p>
      </div>
      <div class="plan-copy">
        <p>For one practice with one to five practitioners.</p>
        <p>The paid plan is not open in M1. Accounts and hosted checkout arrive in M2.</p>
        <a class="button button-secondary" href="/demo">Try the sample first</a>
      </div>
    </section>`;
}

function demoContent(): string {
  if (demoLoading && !demoEnvelope) {
    return `
      <section class="demo-heading">
        <p class="eyebrow">North Star Coaching · sample</p>
        <h1 tabindex="-1">Recover a sample booking</h1>
        <div class="state-panel" role="status" aria-live="polite">
          <span class="state-marker" aria-hidden="true"></span>
          <div><h2>Preparing the sample workspace</h2><p>Adding three fictional bookings and their consent records.</p></div>
        </div>
      </section>`;
  }
  if (demoError && !demoEnvelope) {
    const offline = !navigator.onLine;
    return `
      <section class="demo-heading">
        <p class="eyebrow">Sample workspace</p>
        <h1 tabindex="-1">Recover a sample booking</h1>
        <div class="state-panel state-panel-error" role="alert">
          <div><h2>${offline ? "The demo is offline" : "The demo did not load"}</h2><p>${escapeHtml(demoError)}</p></div>
          <button class="button button-primary" type="button" data-action="retry-demo">Try the demo again</button>
        </div>
      </section>`;
  }
  if (!demoEnvelope) {
    return "";
  }

  const { workspace } = demoEnvelope;
  const selected =
    workspace.attempts.find((attempt) => attempt.id === selectedAttemptId) ??
    workspace.attempts[0];
  if (!selected) {
    return `<section class="demo-heading"><div><p class="eyebrow">Sample workspace</p><h1 tabindex="-1">Recover a sample booking</h1><div class="state-panel state-panel-error" role="alert"><div><h2>The sample is incomplete</h2><p>Reset the demo to restore its sample bookings.</p></div><button class="button button-primary" type="button" data-action="reset-demo">Reset demo</button></div></div></section>`;
  }
  selectedAttemptId = selected?.id ?? null;
  const openCount = workspace.attempts.filter((attempt) => attempt.state === "unfinished").length;

  return `
    <section class="demo-heading">
      <div>
        <p class="eyebrow">${escapeHtml(workspace.practice.name)} · sample</p>
        <h1 tabindex="-1">Recover a sample booking</h1>
        <p class="lede">Choose a ticket, check its consent record, then run one simulated follow-up.</p>
      </div>
      <dl class="service-summary" aria-label="Sample service">
        <div><dt>Service</dt><dd>${escapeHtml(workspace.service.name)}</dd></div>
        <div><dt>Deposit</dt><dd>${formatMoney(workspace.service.depositCents, workspace.service.currency)}</dd></div>
        <div><dt>Needs review</dt><dd>${openCount}</dd></div>
      </dl>
    </section>
    ${demoNotice ? `<p class="inline-notice" role="status" aria-live="polite">${escapeHtml(demoNotice)}</p>` : ""}
    <div class="recovery-board">
      <section class="ticket-rail" aria-labelledby="rail-title">
        <div class="rail-heading"><h2 id="rail-title">Booking rail</h2><p>Times shown for London</p></div>
        <ul class="ticket-list">
          ${workspace.attempts.map((attempt) => ticket(attempt, attempt.id === selected.id)).join("")}
        </ul>
        <div class="empty-state" aria-label="Delivery failures">
          <span aria-hidden="true">✓</span>
          <div><h3>No missed delivery receipts</h3><p>Sample delivery problems would appear here.</p></div>
        </div>
      </section>
      <aside class="case-detail" aria-labelledby="case-title">
        ${caseDetail(selected)}
      </aside>
    </div>`;
}

function ticket(attempt: DemoAttempt, selected: boolean): string {
  const status = statusFor(attempt);
  return `<li>
    <button class="appointment-ticket ${selected ? "is-selected" : ""}" type="button" aria-pressed="${selected}" data-action="select-attempt" data-attempt-id="${escapeHtml(attempt.id)}">
      <span class="ticket-date">${formatDate(attempt.scheduledFor)}</span>
      <strong>${escapeHtml(attempt.clientName)}</strong>
      <span>${escapeHtml(attempt.reason)}</span>
      <span class="status ${status.className}">${status.label}</span>
    </button></li>`;
}

function caseDetail(attempt: DemoAttempt): string {
  const permission = recoveryPermission(attempt);
  const isWorking = workingAttemptId === attempt.id;
  const consentTime = attempt.consent.recordedAt
    ? formatDateTime(attempt.consent.recordedAt)
    : "Not recorded";
  const action =
    attempt.state === "completed" || attempt.state === "recovered"
      ? ""
      : `<button class="button ${permission.allowed ? "button-primary" : "button-secondary"}" type="button" data-action="recover-attempt" data-attempt-id="${escapeHtml(attempt.id)}" ${isWorking ? "disabled" : ""}>${isWorking ? "Running sample…" : permission.allowed ? "Run sample follow-up" : "Check recovery permission"}</button>`;
  return `
    <div class="case-topline"><span class="case-number">Selected ticket</span><span>${formatDateTime(attempt.scheduledFor)}</span></div>
    <h2 id="case-title" tabindex="-1">${escapeHtml(attempt.clientName)}</h2>
    <p>${escapeHtml(attempt.reason)}.</p>
    <section class="evidence-block" aria-labelledby="consent-title">
      <div class="evidence-heading"><h3 id="consent-title">Email permission</h3><span class="status ${attempt.consent.email ? "status-good" : "status-blocked"}">${attempt.consent.email ? "Recorded" : "Missing"}</span></div>
      <p class="evidence-quote">${attempt.consent.wording ? `“${escapeHtml(attempt.consent.wording)}”` : "No email wording was accepted."}</p>
      <p class="evidence-time">${consentTime}</p>
    </section>
    <section class="action-block" aria-labelledby="action-title">
      <h3 id="action-title">Next permitted step</h3>
      <p class="permission-copy ${permission.allowed ? "" : "permission-blocked"}">${escapeHtml(permission.explanation)}</p>
      ${action}
      <p class="action-note">Demo actions use an in-process mailbox. No email leaves this site.</p>
    </section>
    <section class="receipt-block" aria-labelledby="receipt-title">
      <h3 id="receipt-title">Delivery evidence</h3>
      ${receiptTimeline(attempt)}
    </section>`;
}

function receiptTimeline(attempt: DemoAttempt): string {
  if (attempt.receipts.length === 0) {
    return `<div class="empty-receipt"><span aria-hidden="true">○</span><p>No receipt yet. A permitted sample action will add one here.</p></div>`;
  }
  return `<ol class="receipt-timeline">${attempt.receipts
    .map(
      (receipt) => `<li>
        <span class="receipt-node" aria-hidden="true">✓</span>
        <div><strong>${titleCase(receipt.status)} · simulated ${escapeHtml(receipt.channel)}</strong><time datetime="${escapeHtml(receipt.occurredAt)}">${formatDateTime(receipt.occurredAt)}</time><p>${escapeHtml(receipt.detail)}</p></div>
      </li>`
    )
    .join("")}</ol>`;
}

function privacyContent(): string {
  return `
    <article class="policy-page">
      <p class="eyebrow">Privacy</p>
      <h1 tabindex="-1">Your sample stays separate</h1>
      <p class="policy-lede">The demo uses fictional people and a temporary workspace. It never opens a real practice record.</p>
      <section><h2>What the demo stores</h2><p>Your browser keeps one random demo token under <code>demo:workspace-token</code>.</p><p>The server keeps the matching sample workspace for up to 24 hours.</p></section>
      <section><h2>What the demo does not contact</h2><p>Demo actions do not call payment, messaging, sign-in, billing, or AI services.</p><p>The simulated receipt comes from this product’s own server.</p></section>
      <section><h2>How to remove the sample</h2><p>Reset demo makes the current workspace inaccessible and creates a fresh one.</p><p>Start for real removes the browser token. The inaccessible server copy expires automatically.</p></section>
      <section><h2>Production data</h2><p>M1 has no customer account, payment, or real contact-data flow.</p><p>This notice will change before those features open.</p></section>
      <a class="button button-primary" href="/demo">Open the sample workspace</a>
    </article>`;
}

function termsContent(): string {
  return `
    <article class="policy-page">
      <p class="eyebrow">Terms</p>
      <h1 tabindex="-1">Terms for the sample workspace</h1>
      <p class="policy-lede">The M1 demo is a product sample. It does not create a practice account or send a real message.</p>
      <section><h2>Use the sample safely</h2><p>Use only the fictional records already provided. Do not enter client contact details.</p></section>
      <section><h2>No payment in M1</h2><p>The sample does not take deposits or sell a subscription.</p><p>The planned practice plan will use a hosted Sociobot checkout in a later milestone.</p></section>
      <section><h2>Availability</h2><p>The sample may reset during maintenance. Use Reset demo whenever its state is unclear.</p></section>
      <section><h2>Fair use</h2><p>Automated abuse may be rate limited. A limited request returns a retry time.</p></section>
      <a class="button button-primary" href="/demo">Try the sample workspace</a>
    </article>`;
}

function notFoundContent(): string {
  return `
    <section class="not-found-page">
      <div class="lost-ticket" aria-hidden="true"><span></span><span></span><span></span></div>
      <p class="eyebrow">404 · off the rail</p>
      <h1 tabindex="-1">That page is not here</h1>
      <p>The booking rail ends here. Return home or open the sample workspace.</p>
      <div class="button-row"><a class="button button-primary" href="/">Go to the home page</a><a class="button button-secondary" href="/demo">Try the sample</a></div>
    </section>`;
}

function contentFor(route: SiteRoute): string {
  switch (route) {
    case "home":
      return homeContent();
    case "demo":
      return demoContent();
    case "privacy":
      return privacyContent();
    case "terms":
      return termsContent();
    case "not-found":
      return notFoundContent();
  }
}

function navigation(currentRoute: SiteRoute): string {
  const links: ReadonlyArray<readonly [string, SiteRoute | null, string]> = [
    ["/demo", "demo", "Demo"],
    ["/#how-it-works", null, "How it works"],
    ["/privacy", "privacy", "Privacy"]
  ];
  return links
    .map(
      ([href, route, label]) =>
        `<a href="${href}"${currentRoute === route ? ' aria-current="page"' : ""}>${label}</a>`
    )
    .join("");
}

function demoBanner(): string {
  return `<aside class="demo-banner" aria-label="Demo notice">
    <p><strong>Demo</strong> — sample data, nothing is saved</p>
    <div><button type="button" data-action="reset-demo" ${demoLoading ? "disabled" : ""}>${demoLoading ? "Resetting…" : "Reset demo"}</button><a href="/#practice-plan" data-action="leave-demo">Start for real</a></div>
  </aside>`;
}

function setDocumentMetadata(pathname: string, search: string): void {
  const page = pageFor(pathname, search);
  const canonical = canonicalUrl(page.canonicalPath);
  document.title = page.title;
  setMeta('meta[name="description"]', "content", page.description);
  setMeta('meta[property="og:title"]', "content", page.title);
  setMeta('meta[property="og:description"]', "content", page.description);
  setMeta('meta[property="og:url"]', "content", canonical);
  setMeta('meta[name="twitter:title"]', "content", page.title);
  setMeta('meta[name="twitter:description"]', "content", page.description);
  setMeta('link[rel="canonical"]', "href", canonical);
}

function setMeta(selector: string, attribute: string, value: string): void {
  document.querySelector<HTMLElement>(selector)?.setAttribute(attribute, value);
}

function render({ focusHeading = false }: { focusHeading?: boolean } = {}): void {
  const route = routeFor(window.location.pathname, window.location.search);
  const page = pageFor(window.location.pathname, window.location.search);
  setDocumentMetadata(window.location.pathname, window.location.search);
  app.innerHTML = `
    <a class="skip-link" href="#main">Skip to main content</a>
    ${route === "demo" ? demoBanner() : ""}
    <header class="site-header">
      <a class="wordmark" href="/" aria-label="Booking Recovery Loop home"><span aria-hidden="true"></span>Booking Recovery Loop</a>
      <nav aria-label="Primary navigation">${navigation(route)}</nav>
    </header>
    <main id="main" class="main-${route}" tabindex="-1">${contentFor(route)}</main>
    <footer class="site-footer">
      <p>Booking Recovery Loop keeps consent and recovery evidence on one rail.</p>
      <div><a href="/privacy">Privacy</a><a href="/terms">Terms</a><span>Built by Param Factory</span><span>${escapeHtml(import.meta.env.VITE_BUILD_SHA ?? "M1 local")}</span></div>
      <p class="art-credit">Original rail artwork made for this product.</p>
    </footer>
    <p class="sr-only" aria-live="polite" aria-atomic="true" id="route-announcement">${escapeHtml(page.heading)}</p>`;

  if (focusHeading) {
    document.querySelector<HTMLElement>("main h1")?.focus();
  }
  if (route === "demo" && !demoEnvelope && !demoLoading && !demoError) {
    void initialiseDemo();
  }
}

async function initialiseDemo(forceCreate = false): Promise<void> {
  demoLoading = true;
  demoError = null;
  render();
  try {
    const token = forceCreate ? null : localStorage.getItem(DEMO_STORAGE_KEY);
    let envelope: DemoEnvelope;
    if (token) {
      try {
        envelope = await loadDemo(token);
      } catch (error) {
        if (!(error instanceof DemoApiError) || error.status !== 404) {
          throw error;
        }
        localStorage.removeItem(DEMO_STORAGE_KEY);
        envelope = await createDemo();
      }
    } else {
      envelope = await createDemo();
    }
    acceptDemo(envelope);
  } catch (error) {
    demoError = messageFor(error);
  } finally {
    demoLoading = false;
    render();
  }
}

async function performReset(): Promise<void> {
  if (demoLoading) return;
  demoLoading = true;
  demoNotice = "Restoring the original sample bookings.";
  render();
  try {
    const token = demoEnvelope?.workspaceToken ?? localStorage.getItem(DEMO_STORAGE_KEY);
    const envelope = token ? await resetDemo(token) : await createDemo();
    acceptDemo(envelope);
    selectedAttemptId = envelope.workspace.attempts[0]?.id ?? null;
    demoNotice = "Demo reset. The original sample bookings are ready.";
  } catch (error) {
    demoNotice = messageFor(error);
  } finally {
    demoLoading = false;
    render();
  }
}

async function performRecovery(attemptId: string): Promise<void> {
  if (!demoEnvelope || workingAttemptId) return;
  const attempt = demoEnvelope.workspace.attempts.find((item) => item.id === attemptId);
  if (attempt && !recoveryPermission(attempt).allowed) {
    demoNotice = recoveryPermission(attempt).explanation;
    render();
    return;
  }
  workingAttemptId = attemptId;
  demoNotice = "Checking the recorded permission.";
  render();
  try {
    const envelope = await recoverDemoAttempt(demoEnvelope.workspaceToken, attemptId);
    acceptDemo(envelope);
    selectedAttemptId = attemptId;
    demoNotice = "Sample follow-up delivered. A simulated receipt was added.";
  } catch (error) {
    demoNotice = messageFor(error);
  } finally {
    workingAttemptId = null;
    render();
  }
}

function acceptDemo(envelope: DemoEnvelope): void {
  demoEnvelope = envelope;
  localStorage.setItem(DEMO_STORAGE_KEY, envelope.workspaceToken);
  selectedAttemptId ??= envelope.workspace.attempts[0]?.id ?? null;
}

function leaveDemo(destination: URL): void {
  localStorage.removeItem(DEMO_STORAGE_KEY);
  demoEnvelope = null;
  demoError = null;
  demoNotice = null;
  selectedAttemptId = null;
  navigate(destination, true);
}

function navigate(destination: URL, focusHeading: boolean): void {
  window.history.pushState({}, "", `${destination.pathname}${destination.search}${destination.hash}`);
  render({ focusHeading });
  if (destination.hash) {
    window.requestAnimationFrame(() => document.querySelector(destination.hash)?.scrollIntoView());
  } else {
    window.scrollTo({ top: 0 });
  }
}

function internalAnchor(event: MouseEvent): HTMLAnchorElement | null {
  const target = event.target;
  if (!(target instanceof Element)) return null;
  const anchor = target.closest<HTMLAnchorElement>("a[href]");
  if (
    !anchor ||
    event.defaultPrevented ||
    event.button !== 0 ||
    event.metaKey ||
    event.ctrlKey ||
    event.shiftKey ||
    event.altKey
  ) {
    return null;
  }
  const destination = new URL(anchor.href, window.location.href);
  return destination.origin === window.location.origin ? anchor : null;
}

document.addEventListener("click", (event) => {
  const target = event.target;
  if (!(target instanceof Element)) return;
  const actionTarget = target.closest<HTMLElement>("[data-action]");
  const action = actionTarget?.dataset.action;
  if (action === "select-attempt") {
    selectedAttemptId = actionTarget?.dataset.attemptId ?? null;
    render();
    document.querySelector<HTMLElement>("#case-title")?.focus({ preventScroll: true });
    return;
  }
  if (action === "reset-demo") {
    void performReset();
    return;
  }
  if (action === "retry-demo") {
    demoError = null;
    void initialiseDemo();
    return;
  }
  if (action === "recover-attempt") {
    const attemptId = actionTarget?.dataset.attemptId;
    if (attemptId) void performRecovery(attemptId);
    return;
  }

  const anchor = internalAnchor(event);
  if (!anchor) return;
  if (anchor.classList.contains("skip-link")) {
    event.preventDefault();
    document.querySelector<HTMLElement>("#main")?.focus();
    return;
  }
  const destination = new URL(anchor.href, window.location.href);
  event.preventDefault();
  if (action === "leave-demo") {
    leaveDemo(destination);
  } else {
    navigate(destination, true);
  }
});

window.addEventListener("popstate", () => render({ focusHeading: true }));
window.addEventListener("offline", () => {
  if (routeFor(window.location.pathname, window.location.search) === "demo") {
    demoNotice = "You are offline. Viewing works, but sample actions need a connection.";
    render();
  }
});
window.addEventListener("online", () => {
  if (routeFor(window.location.pathname, window.location.search) === "demo") {
    demoNotice = "You are back online. Sample actions are available.";
    render();
  }
});

function statusFor(attempt: DemoAttempt): { label: string; className: string } {
  if (attempt.state === "recovered") return { label: "Recovered in demo", className: "status-good" };
  if (attempt.state === "completed") return { label: "Booking complete", className: "status-good" };
  if (!attempt.consent.email) return { label: "Stopped — no consent", className: "status-blocked" };
  return { label: "Needs a follow-up", className: "status-attention" };
}

function formatDate(value: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    weekday: "short",
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
    timeZone: "Europe/London"
  }).format(new Date(value));
}

function formatDateTime(value: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
    timeZone: "Europe/London",
    timeZoneName: "short"
  }).format(new Date(value));
}

function formatMoney(cents: number, currency: string): string {
  return new Intl.NumberFormat("en-GB", { style: "currency", currency }).format(cents / 100);
}

function titleCase(value: string): string {
  return `${value.charAt(0).toUpperCase()}${value.slice(1)}`;
}

function messageFor(error: unknown): string {
  if (error instanceof Error) return error.message;
  return "The sample action failed. Try it again.";
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

render();
