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
import {
  createBookingAttempt,
  createPractice,
  deletePractice,
  loadPractice,
  publicPractice,
  recoverPracticeAttempt,
  testDeliveryConnection,
  type Practice,
  type PublicPractice,
  type ScheduledJob
} from "./lib/practice";
import { accessToken, initialiseIdentity, signIn, signOut, signedInName } from "./lib/auth";

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
let practice: Practice | null = null;
let publicPage: PublicPractice | null = null;
let practiceLoading = false;
let practiceError: string | null = null;
let practiceNotice: string | null = null;
let identityName: string | null = null;

function homeContent(): string {
  return `
    <section class="hero" aria-describedby="hero-summary">
      <div class="hero-copy">
        <h1 tabindex="-1">Recover unfinished paid-session bookings</h1>
        <p id="hero-summary" class="lede">For solo coaches, tutors, and consultants who need to act when a paid booking stops.</p>
        <div class="hero-action">
          <a class="button button-primary" href="/demo">Try it with sample data</a>
          <p>See three fictional bookings, then reset the sample at any time.</p>
        </div>
        <ul class="plain-facts" aria-label="Demo facts">
          <li><span aria-hidden="true">01</span> Demo needs no account</li>
          <li><span aria-hidden="true">02</span> Demo sends no messages</li>
          <li><span aria-hidden="true">03</span> Demo opens no payment</li>
        </ul>
        <p class="real-start">Ready to use your own booking? <a href="/start">Set up your practice</a>.</p>
      </div>
      <figure class="hero-scene">
        <img src="${railSceneUrl}" width="920" height="620" fetchpriority="high" alt="A calm appointment rail showing one booking that needs a follow-up." />
        <figcaption>One booking stopped. Email consent decides the next step.</figcaption>
      </figure>
    </section>

    <section class="product-preview section-rule" aria-labelledby="preview-title">
      <div class="section-intro">
        <p class="eyebrow">Sample view</p>
        <h2 id="preview-title">Sample recovery board</h2>
        <p>Review a booking, its email consent, and each delivery receipt.</p>
      </div>
      <div class="preview-board" aria-label="Sample recovery board preview">
        <div class="preview-ticket preview-ticket-muted">
          <p class="ticket-time">Tue · 14:00</p>
          <h3>Booking started</h3>
          <p>Service and time chosen</p>
          <span class="status status-good">Booking recorded</span>
        </div>
        <div class="preview-connector" aria-hidden="true"></div>
        <div class="preview-ticket preview-ticket-active">
          <p class="ticket-time">18 minutes ago</p>
          <h3>Deposit not finished</h3>
          <p>Email consent recorded.</p>
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
        <p class="eyebrow">Three steps</p>
        <h2 id="how-title">How booking recovery works</h2>
      </div>
      <ol class="process-rail">
        <li><span>1</span><div><h3>Find the stopped booking</h3><p>See the chosen session and where the client left.</p></div></li>
        <li><span>2</span><div><h3>Check email consent first</h3><p>A follow-up stays stopped when email consent is missing.</p></div></li>
        <li><span>3</span><div><h3>Read the delivery receipt</h3><p>The sample action ends with a timestamped simulated receipt.</p></div></li>
      </ol>
    </section>

    <section class="boundary-section section-rule" aria-labelledby="boundary-title">
      <div>
        <p class="eyebrow">Product scope</p>
        <h2 id="boundary-title">It does not replace your calendar</h2>
      </div>
      <div class="boundary-copy">
        <p>It is not a CRM, a marketplace, or a tool for bulk messages.</p>
        <a href="/privacy">Read how booking data is handled</a>
      </div>
    </section>

    <section id="practice-plan" class="plan-section section-rule" aria-labelledby="plan-title">
      <div>
        <p class="eyebrow">Use your own bookings</p>
        <h2 id="plan-title">Create a practice workspace</h2>
      </div>
      <div class="plan-copy">
        <p>Recovery Loop Practice is $29 per month for one practice.</p>
        <p>Publish one session page, record email or SMS consent, and review delivery receipts.</p>
        <p class="action-note">The hosted checkout starts the $29 monthly practice subscription. Sign in to create the workspace after checkout.</p>
        <div class="button-row"><a class="button button-primary" href="https://api.sociobot.in/api/v1/products/booking-recovery-loop/checkout">Start Recovery Loop Practice</a><a class="button button-secondary" href="/start">Set up your practice</a></div>
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
      <div class="evidence-heading"><h3 id="consent-title">Email consent</h3><span class="status ${attempt.consent.email ? "status-good" : "status-blocked"}">${attempt.consent.email ? "Recorded" : "Missing"}</span></div>
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

function scheduledJobs(jobs: readonly ScheduledJob[]): string {
  if (jobs.length === 0) return "";
  return `<section class="evidence-block" aria-label="Automatic recovery schedule"><h4>Automatic schedule</h4><ol class="schedule-list">${jobs.map((job) => `<li><strong>${job.kind === "abandoned_recovery" ? "Booking recovery" : "Session reminder"}</strong><span>${titleCase(job.status)} · ${formatDateTime(job.dueAt)}</span>${job.lastError ? `<p>${escapeHtml(job.lastError)}</p>` : ""}</li>`).join("")}</ol></section>`;
}

function startContent(): string {
  return `<article class="setup-page">
    <p class="eyebrow">Practice setup</p>
    <h1 tabindex="-1">Set up your booking recovery page</h1>
    <p class="policy-lede">Create one paid-session page. Your Sociobot account owns the workspace across devices.</p>
    ${practiceNotice ? `<p class="inline-notice" role="status" aria-live="polite">${escapeHtml(practiceNotice)}</p>` : ""}
    <form class="setup-form" data-form="create-practice">
      <fieldset><legend>Practice</legend>
        <label>Practice name<input name="name" required minlength="2" maxlength="80" autocomplete="organization" value="North Star Coaching"></label>
        <label>Booking link<span class="field-prefix">${escapeHtml(window.location.origin)}/b/</span><input name="publicSlug" required pattern="[a-z0-9\\-]{3,40}" value="north-star-${Date.now().toString().slice(-6)}" aria-describedby="slug-help"></label>
        <p id="slug-help" class="field-help">Use lowercase letters, numbers, and hyphens.</p>
        <label>Timezone<input name="timezone" required value="Europe/London"></label>
      </fieldset>
      <fieldset><legend>Paid session</legend>
        <label>Session name<input name="serviceName" required maxlength="100" value="45-minute focus session"></label>
        <label>Length in minutes<input name="durationMinutes" type="number" min="15" max="480" required value="45"></label>
        <label>Deposit in minor units<input name="depositCents" type="number" min="0" max="1000000" required value="3500" aria-describedby="deposit-help"></label>
        <p id="deposit-help" class="field-help">For £35, enter 3500.</p>
        <label>Currency<input name="currency" required minlength="3" maxlength="3" value="GBP"></label>
        <label>Hosted deposit URL<input name="paymentUrl" type="url" inputmode="url" required placeholder="https://payments.your-practice.example/session" aria-describedby="payment-help"></label>
        <p id="payment-help" class="field-help">Paste the secure payment page you already use. The booking form has no card fields.</p>
      </fieldset>
      <fieldset><legend>Delivery connection</legend>
        <p id="delivery-help" class="field-help">Live email and SMS delivery are not enabled in this deployment. The product will not accept client contact data for a provider connection until a credentialed provider is configured.</p>
        <input name="deliveryWebhookUrl" type="hidden" value="">
      </fieldset>
      <button class="button button-primary" type="submit">Create practice workspace</button>
    </form>
  </article>`;
}

function appContent(): string {
  if (practiceLoading) return statePage("Review bookings that need action", "Loading your practice workspace.");
  if (!identityName || practiceError) {
    return `<article class="setup-page"><p class="eyebrow">Private practice</p><h1 tabindex="-1">Open your recovery queue</h1>
      <p class="policy-lede">Sign in with your Sociobot account to open your practice workspace.</p>
      ${practiceError ? `<p class="inline-notice notice-error" role="alert">${escapeHtml(practiceError)}</p>` : ""}
      <div class="button-row"><button class="button button-primary" type="button" data-action="sign-in">Sign in</button><a class="button button-secondary" href="/start">Create a practice workspace</a></div></article>`;
  }
  if (!practice) return statePage("Review bookings that need action", "Loading your practice workspace.");
  return `<section class="practice-heading"><div><p class="eyebrow">${escapeHtml(practice.name)}</p><h1 tabindex="-1">Review bookings that need action</h1><p class="lede">Recovery and reminders run automatically when their due time arrives. Email or SMS consent decides the channel.</p></div>
    <div class="button-row"><a class="button button-secondary" href="/b/${escapeHtml(practice.publicSlug)}">Open public booking page</a><a class="button button-secondary" href="/app/settings/data">Manage data</a></div></section>
    ${practiceNotice ? `<p class="inline-notice" role="status" aria-live="polite">${escapeHtml(practiceNotice)}</p>` : ""}
    <section class="practice-board" aria-labelledby="attempts-title"><h2 id="attempts-title">Booking attempts</h2>
      ${practice.attempts.length === 0 ? `<div class="state-panel"><div><h3>No bookings need attention</h3><p>New bookings from your public page will appear here.</p></div></div>` : `<ul class="practice-attempts">${practice.attempts.map((attempt) => `<li><article><div class="attempt-top"><h3>${escapeHtml(attempt.clientName)}</h3><span class="status ${attempt.state === "recovered" ? "status-good" : "status-attention"}">${escapeHtml(attempt.state.replaceAll("_", " "))}</span></div><p>${formatDateTime(attempt.scheduledFor)}</p><p>Email consent: <strong>${attempt.emailConsent ? "Recorded" : "Missing"}</strong> · SMS consent: <strong>${attempt.smsConsent ? "Recorded" : "Missing"}</strong></p><p class="evidence-time">Consent recorded ${formatDateTime(attempt.consentRecordedAt)}</p>${scheduledJobs(attempt.scheduledJobs)}${attempt.events.length ? `<ol class="receipt-timeline">${attempt.events.map((event) => `<li><span class="receipt-node" aria-hidden="true">✓</span><div><strong>${titleCase(event.status)} · ${escapeHtml(event.channel)}</strong><time datetime="${escapeHtml(event.occurredAt)}">${formatDateTime(event.occurredAt)}</time><p>${escapeHtml(event.detail)}</p></div></li>`).join("")}</ol>` : `<p>No delivery receipt yet.</p>`}${attempt.scheduledJobs.some((job) => job.status === "failed") ? `<button class="button button-secondary" type="button" data-action="recover-practice" data-attempt-id="${escapeHtml(attempt.id)}">Retry delivery now</button>` : ""}</article></li>`).join("")}</ul>`}
    </section>`;
}

function bookingContent(): string {
  if (practiceLoading) return statePage("Finish your paid session booking", "Loading this booking page.");
  if (practiceError) return `<article class="policy-page"><h1 tabindex="-1">This booking page is unavailable</h1><p role="alert">${escapeHtml(practiceError)}</p><a href="/">Return home</a></article>`;
  if (!publicPage) return statePage("Finish your paid session booking", "Loading this booking page.");
  const tomorrow = new Date(Date.now() + 24 * 60 * 60 * 1000); tomorrow.setMinutes(0, 0, 0);
  return `<article class="booking-page"><p class="eyebrow">${escapeHtml(publicPage.name)}</p><h1 tabindex="-1">Finish your paid session booking</h1>
    <section class="service-ticket" aria-labelledby="service-title"><div><h2 id="service-title">${escapeHtml(publicPage.serviceName)}</h2><p>${publicPage.durationMinutes} minutes · ${formatMoney(publicPage.depositCents, publicPage.currency)} deposit</p></div><span class="status status-attention">Deposit required</span></section>
    ${practiceNotice ? `<p class="inline-notice" role="status">${escapeHtml(practiceNotice)}</p>` : ""}
    <form class="booking-form" data-form="create-booking"><label>Your name<input name="clientName" required minlength="2" maxlength="100" autocomplete="name"></label><label>Email address<input name="email" type="email" autocomplete="email"></label><label>Mobile number<input name="phone" type="tel" autocomplete="tel"></label><label>Session time<input name="scheduledFor" type="datetime-local" required value="${tomorrow.toISOString().slice(0,16)}" aria-describedby="slot-help"></label><p id="slot-help" class="field-help">A time already held by another active booking cannot be selected.</p>
      <fieldset><legend>Contact consent</legend><p>${escapeHtml(publicPage.consentWording)}</p><label class="check-label"><input name="emailConsent" type="checkbox"> I give email consent for this booking</label><label class="check-label"><input name="smsConsent" type="checkbox"> I give SMS consent for this booking</label></fieldset>
      <button class="button button-primary" type="submit">Save booking and open payment</button><p class="action-note">This form records no card number, card expiry, or security code. You will leave this site for the practice’s hosted payment page.</p></form></article>`;
}

function completeContent(): string {
  return `<article class="policy-page"><p class="eyebrow">Payment status</p><h1 tabindex="-1">Your deposit is being checked</h1><p class="policy-lede">Returning from checkout does not prove payment. The practice will confirm your booking after its payment provider reports the deposit.</p><a class="button button-primary" href="/">Return home</a></article>`;
}

function dataContent(): string {
  if (practiceLoading) return statePage("Export or delete practice data", "Loading your practice workspace.");
  return `<article class="policy-page"><p class="eyebrow">Practice data</p><h1 tabindex="-1">Export or delete practice data</h1><p class="policy-lede">Export stays available without a delivery connection. Deletion removes the practice, booking attempts, encrypted contacts, and receipts.</p>${identityName ? `<section><h2>Account access</h2><p>${escapeHtml(identityName)} is signed in through Sociobot Entra External ID. No access key or callback secret is stored in this browser.</p></section><div class="button-row"><button class="button button-secondary" type="button" data-action="export-practice">Export practice JSON</button><button class="button button-danger" type="button" data-action="delete-practice">Delete practice data</button></div>` : `<p>Sign in to export or delete a practice.</p><button class="button button-primary" type="button" data-action="sign-in">Sign in</button>`}</article>`;
}

function authCallbackContent(): string {
  return `<article class="policy-page"><p class="eyebrow">Account access</p><h1 tabindex="-1">Completing sign-in</h1><p class="policy-lede">Your Sociobot account is being checked. You can return to your recovery queue when this page updates.</p><a class="button button-primary" href="/app">Open recovery queue</a></article>`;
}

function statePage(title: string, message: string): string { return `<section class="demo-heading"><h1 tabindex="-1">${escapeHtml(title)}</h1><div class="state-panel" role="status"><div><h2>Please wait</h2><p>${escapeHtml(message)}</p></div></div></section>`; }

function privacyContent(): string {
  return `
    <article class="policy-page">
      <p class="eyebrow">Privacy</p>
      <h1 tabindex="-1">Control your booking data</h1>
      <p class="policy-lede">The demo stays separate. A real workspace stores practice settings, bookings, consent records, scheduled messages, and delivery receipts.</p>
      <section><h2>What the demo stores</h2><p>Your browser keeps one random demo token under <code>demo:workspace-token</code>.</p><p>The server keeps the matching sample workspace for up to 24 hours.</p></section>
      <section><h2>What the demo does not contact</h2><p>Demo actions do not call payment, messaging, sign-in, billing, or AI services.</p><p>The simulated receipt comes from this product’s own server.</p></section>
      <section><h2>How to remove the sample</h2><p>Reset demo makes the current workspace inaccessible and creates a fresh one.</p><p>Start for real removes the browser token. The inaccessible server copy expires automatically.</p></section>
      <section><h2>What a practice stores</h2><p>The service stores practice settings, booking attempts, email or SMS consent, scheduled messages, and delivery receipts.</p><p>Client names, email addresses, and phone numbers are encrypted before database storage.</p><p>When a permitted message is due, the service sends the selected contact field and message text to the supported delivery provider. That provider controls its own retention.</p><p>The booking form has no card fields. Payment card details stay on the practice’s hosted payment page.</p></section>
      <section><h2>Export and deletion</h2><p>A practice owner can export its records or delete the full practice from the data controls page.</p></section>
      <div class="button-row"><a class="button button-primary" href="/demo">Open the sample workspace</a><a class="button button-secondary" href="/app/settings/data">Open data controls</a></div>
    </article>`;
}

function termsContent(): string {
  return `
    <article class="policy-page">
      <p class="eyebrow">Terms</p>
      <h1 tabindex="-1">Terms for using Booking Recovery Loop</h1>
      <p class="policy-lede">Use the demo with its fictional records. Use a real workspace only for bookings you are allowed to manage.</p>
      <section><h2>Use the sample safely</h2><p>Use only the fictional records already provided. Do not enter client contact details.</p></section>
      <section><h2>Hosted payments</h2><p>Practices provide their own hosted payment link. The booking form has no card fields and does not confirm payment from a return URL.</p></section>
      <section><h2>Messages and consent</h2><p>Automatic recovery and reminders use only recorded email or SMS consent.</p><p>A delivery receipt reports provider status. It is not a guarantee that a person read the message.</p></section>
      <section><h2>Availability</h2><p>The sample may reset during maintenance. Use Reset demo whenever its state is unclear.</p></section>
      <section><h2>Fair use</h2><p>Automated abuse may be rate limited. A limited request returns a retry time.</p></section>
      <a class="button button-primary" href="/demo">Try the sample workspace</a>
    </article>`;
}

function notFoundContent(): string {
  return `
    <section class="not-found-page">
      <div class="lost-ticket" aria-hidden="true"><span></span><span></span><span></span></div>
      <p class="eyebrow">Page not found</p>
      <h1 tabindex="-1">That page is not here</h1>
      <p>Check the address, return home, or open the sample workspace.</p>
      <div class="button-row"><a class="button button-primary" href="/">Go to the home page</a><a class="button button-secondary" href="/demo">Try the sample</a></div>
    </section>`;
}

function contentFor(route: SiteRoute): string {
  switch (route) {
    case "home":
      return homeContent();
    case "demo":
      return demoContent();
    case "start":
      return startContent();
    case "app":
      return appContent();
    case "data":
      return dataContent();
    case "auth-callback":
      return authCallbackContent();
    case "booking":
      return bookingContent();
    case "complete":
      return completeContent();
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
    ["/start", "start", "Set up"],
    ["/app", "app", "Recovery queue"],
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
    <div><button type="button" data-action="reset-demo" ${demoLoading ? "disabled" : ""}>${demoLoading ? "Resetting…" : "Reset demo"}</button><a href="/start" data-action="leave-demo">Start for real</a></div>
  </aside>`;
}

function setDocumentMetadata(pathname: string, search: string): void {
  const page = pageFor(pathname, search);
  const route = routeFor(pathname, search);
  const canonical = canonicalUrl(route === "booking" || route === "complete" ? pathname : page.canonicalPath);
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
      <nav aria-label="Primary navigation">${navigation(route)}${identityName ? `<button class="nav-account" type="button" data-action="sign-out">Sign out</button>` : `<button class="nav-account" type="button" data-action="sign-in">Sign in</button>`}</nav>
    </header>
    <main id="main" class="main-${route}" tabindex="-1">${contentFor(route)}</main>
    <footer class="site-footer">
      <p>Review stopped bookings, email or SMS consent, and delivery receipts.</p>
      <div><a href="/privacy">Privacy</a><a href="/terms">Terms</a><span>Built by Param Factory</span><span>${escapeHtml(import.meta.env.VITE_BUILD_SHA ?? "local build")}</span></div>
      <p class="art-credit">Original rail artwork made for this product.</p>
    </footer>
    <p class="sr-only" aria-live="polite" aria-atomic="true" id="route-announcement">${escapeHtml(page.heading)}</p>`;

  if (focusHeading) {
    document.querySelector<HTMLElement>("main h1")?.focus();
  }
  if (route === "demo" && !demoEnvelope && !demoLoading && !demoError) {
    void initialiseDemo();
  }
  if ((route === "app" || route === "data") && identityName && !practice && !practiceLoading && !practiceError) {
    void initialisePractice();
  }
  if (route === "booking" && !publicPage && !practiceLoading && !practiceError) {
    void initialisePublicPage();
  }
}

async function initialisePractice(): Promise<void> {
  practiceLoading = true; practiceError = null; render();
  try { practice = await loadPractice(); }
  catch (error) { practiceError = messageFor(error); practice = null; }
  finally { practiceLoading = false; render(); }
}

async function initialisePublicPage(): Promise<void> {
  const slug = window.location.pathname.split("/")[2];
  if (!slug) return;
  practiceLoading = true; practiceError = null; render();
  try { publicPage = await publicPractice(slug); }
  catch (error) { practiceError = messageFor(error); }
  finally { practiceLoading = false; render(); }
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
  if (action === "recover-practice") {
    const attemptId = actionTarget?.dataset.attemptId;
    if (attemptId) void (async () => {
      practiceNotice = "Checking consent and asking the delivery service to send."; render();
      try { await recoverPracticeAttempt(attemptId); practiceNotice = "The delivery service accepted the recovery message."; practice = await loadPractice(); }
      catch (error) { practiceNotice = messageFor(error); }
      render();
    })();
    return;
  }
  if (action === "test-delivery") {
    if (identityName) void (async () => {
      practiceNotice = "Sending a delivery connection test without client data."; render();
      try { await testDeliveryConnection(); practiceNotice = "The delivery service accepted the connection test. No client data was sent."; }
      catch (error) { practiceNotice = messageFor(error); }
      render();
    })();
    return;
  }
  if (action === "export-practice") {
    if (identityName) void exportPractice();
    return;
  }
  if (action === "delete-practice") {
    if (identityName && window.confirm("Delete this practice and every booking record? This cannot be undone.")) void (async () => {
      try { await deletePractice(); practice = null; practiceNotice = null; navigate(new URL("/", window.location.origin), true); }
      catch (error) { practiceNotice = messageFor(error); render(); }
    })();
    return;
  }
  if (action === "sign-in") { void signIn(); return; }
  if (action === "sign-out") { void signOut(); return; }

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

document.addEventListener("submit", (event) => {
  const form = event.target;
  if (!(form instanceof HTMLFormElement)) return;
  event.preventDefault();
  const data = new FormData(form);
  if (form.dataset.form === "create-practice") void (async () => {
    practiceNotice = "Creating the private practice workspace."; render();
    try {
      const result = await createPractice({ name: data.get("name"), publicSlug: data.get("publicSlug"), timezone: data.get("timezone"), serviceName: data.get("serviceName"), durationMinutes: Number(data.get("durationMinutes")), depositCents: Number(data.get("depositCents")), currency: data.get("currency"), paymentUrl: data.get("paymentUrl"), deliveryWebhookUrl: data.get("deliveryWebhookUrl") });
      practice = result.practice; practiceNotice = "Practice created for your signed-in account.";
      navigate(new URL("/app", window.location.origin), true);
    } catch (error) { practiceNotice = messageFor(error); render(); }
  })();
  if (form.dataset.form === "create-booking") void (async () => {
    const slug = window.location.pathname.split("/")[2] ?? "";
    const scheduled = new Date(String(data.get("scheduledFor"))).toISOString();
    practiceNotice = "Saving the booking and consent record."; render();
    try {
      const result = await createBookingAttempt(slug, { clientName: data.get("clientName"), email: data.get("email") || null, phone: data.get("phone") || null, scheduledFor: scheduled, emailConsent: data.get("emailConsent") === "on", smsConsent: data.get("smsConsent") === "on" });
      window.location.assign(result.paymentUrl);
    } catch (error) { practiceNotice = messageFor(error); render(); }
  })();
});

async function exportPractice(): Promise<void> {
  const token = await accessToken();
  if (!token) { practiceNotice = "Sign in to download this export."; render(); return; }
  const response = await fetch("/api/v1/practice/export", { headers: { Authorization: `Bearer ${token}` } });
  if (!response.ok) { practiceNotice = "The export did not finish. Try again."; render(); return; }
  const url = URL.createObjectURL(await response.blob()); const link = document.createElement("a"); link.href = url; link.download = "booking-recovery-export.json"; link.click(); URL.revokeObjectURL(url);
  practiceNotice = "Practice export downloaded."; render();
}

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

void initialiseIdentity().then(async () => {
  identityName = await signedInName();
  render({ focusHeading: window.location.pathname === "/auth/callback" });
});
render();
