# Booking Recovery Loop — venture plan

**Status:** Repair 9 deployed the shared PostgreSQL multi-replica boundary and passed live reset/rate probes; production delivery credentials and the dedicated deposit product remain factory-gated.
**Product URL:** `https://booking-recovery-loop.sociobot.in`
**Planning date:** 2026-08-28

## Polish 1 implementation note

The adversarial review required the real job before the planned milestone
sequence could continue. Polish 1 therefore implements the core M2–M5 path in
one bounded release: an owner-token practice workspace, a public paid-session
page, timestamped channel consent, hosted-payment handoff and authenticated
payment confirmation, provider delivery receipts, one permitted bounce
fallback, encrypted contact fields, tenant-scoped export, and deletion. The
original `/demo` boundary remains unchanged.

The release uses a 256-bit private practice key instead of Entra because the
shared callback is not registered for this product. It does not advertise or
link the $29 subscription because the required Sociobot product endpoint
currently returns 404. These external registrations cannot be created from the
repository; the product remains usable without pretending either gateway is
active.

This is the execution contract for every milestone worker. Read this file, the
brief, the current design thesis, and every prior milestone handoff before
changing product code. A milestone may move to the next one only after its
claims, review, polish pass, and handoff are complete.

## 1. Product requirements document

### Customer, situation, and current workaround

The buyer is a solo tutor, coach, or consultant, usually in a one-to-five-person
practice. They sell paid appointments. A prospect can abandon the booking page,
or a paid client can miss a session because a reminder failed. Each lost session
is direct lost income and a hard-to-reconstruct support problem.

Today they combine Calendly or Cal.com, Stripe, manual texts, and email logs.
They must discover an unfinished booking or failed reminder themselves, decide
whether consent allows a follow-up, and then prove what was sent. They do not
need another calendar, CRM, or marketplace.

### Promise

**Turn an unfinished or at-risk paid session into a consented, provable next
step.**

### The three jobs the product must nail

1. **Make a paid session easy to finish.** An owner publishes a branded session
   page with a clear service, available times, consent wording, and a
   Sociobot/Dodo-hosted deposit step. The product records the attempt and never handles
   card data.
2. **Recover an unfinished booking responsibly.** When a known visitor stops
   before payment or confirmation, the product applies a practice-defined,
   consent-aware rule and sends one useful email or SMS fallback. An owner can
   see and stop that recovery case.
3. **Prove attendance protection worked.** The product schedules reminders,
   records provider receipts and bounces, then switches to the permitted
   fallback channel. The owner can see whether a booking is paid, reminded,
   delivered, bounced, recovered, cancelled, or needs attention.

### Monetisation

The launch subscription is the named **Recovery Loop Practice** tier at
**$29/month per practice** for one to five practitioners. It includes the
branded booking page, recovery cases, delivery proof, and the allowance shown
at checkout. The hosted Sociobot billing flow is backed by Dodo; it is the only
way the product sells its own subscription. The product never embeds Dodo or a
payment form.

There is no permanent free production tier. The public, no-account demo is a
safe sample, not a trial storing customer data. A higher tier is intentionally
deferred until pilots establish a usage boundary that justifies it.

End-client session deposits are a distinct flow: the server creates a
per-booking checkout through the approved Sociobot billing API, backed by Dodo.
No card number, CVC, or payment form is collected by Booking Recovery Loop.

### Deliberately out of scope

- Replacing Google Calendar, Outlook, Cal.com, Calendly, or a full scheduling
  engine.
- CRM pipelines, discovery marketplaces, campaigns, bulk marketing, or
  unlimited SMS blasts.
- Taking custody of a practice's session revenue or storing card data.
- Sending messages without recorded channel consent, or using contact data to
  train a model.
- AI-generated messages in the first release. The recovery rules are
  deterministic and easier to audit. A later opt-in drafting assistant is only
  worth adding if pilots show owners spend material time editing recovery copy.

### Product success and guardrails

The pilot target is, over 60 days, either at least 10% of abandoned attempts
recovered or a 20% reduction in paid-session no-shows versus the practice's
prior 60 days. This is a pilot outcome, not landing-page copy until measured.

Every contact action must be attributable to a practice, consent record,
recovery rule, and delivery event. A recovery is capped by default at one email
and one permitted SMS fallback per attempt; a cancelled or opted-out contact
cannot be messaged. The customer can export and request deletion of their
practice data.

## 2. Evidence and wedge

| Signal | What it says | Product implication |
| --- | --- | --- |
| [HN discussion, 2025-05-12](https://hn.algolia.com/api/v1/items/43959652) | A tutor says scheduling tools feel bloated and clients still text; a legal consultant wants a booking experience that converts and includes Stripe. | The booking surface must be short, plain, branded, and deposit-aware—not a calendar administration product. |
| [Cal.com issue #28811, 2026-04-09](https://github.com/calcom/cal.diy/issues/28811) | Workflows configured for 24-hour and one-hour reminders reportedly never executed. | Delivery receipts, bounce handling, and a visible fallback are core records, not hidden automation. |
| Repeated workaround in the brief | Owners stitch a scheduler, Stripe, manual messages, and email logs together. | The product owns the handoff from booking intent to attendance proof. |

The wedge is accountable recovery for a tiny practice. Calendly, Cal.com,
SavvyCal, and Stripe each handle a piece. Booking Recovery Loop makes the
conversion-to-attendance chain observable: an owner can identify the attempt,
see the consent basis, see the delivery result, and intervene before revenue is
lost.

## 3. Architecture

### Stack decision

- **Web client:** Vite 6 + strict TypeScript + vanilla DOM/CSS. This public
  booking surface and compact operations UI need fast load and accessible,
  explicit state—not a component runtime. The initial JavaScript budget is
  150 KB gzip (hard maximum 200 KB).
- **API and worker:** Rust 2021, axum, tokio, sqlx, PostgreSQL. This is shared,
  sensitive multi-tenant data with scheduled work and webhook verification;
  Rust and Postgres make ownership, query boundaries, and job claiming
  explicit. A single service image runs the HTTP API and a supervised worker
  loop initially; split worker deployment only when queue load warrants it.
- **M1 demo store:** SQLite holds only fictional, expiring demo workspaces. The
  runtime receives no database connection string, and no customer record exists
  in M1. This keeps the public demo deployable with only `PORT`; M2 introduces
  PostgreSQL before accounts or real practice data are accepted.
- **Deployment shape:** one container serving the API on `PORT` (default
  `8080`) plus the built web assets, behind the factory ingress. `dist/` is
  always produced by `npm run build` for static inspection. Docker is
  multi-stage and runs as non-root. No infrastructure changes live here.
- **Local developer services:** M1 starts with its generated SQLite demo store.
  Docker Compose retains the planned Postgres service for M2. API tests use an
  isolated in-memory database, and browser tests run the built client through
  axum so same-origin boundaries match production.

M1 turns the initial Vite and axum shells into the first useful demo. Later
milestones extend this boundary instead of replacing the public workflow.

### Routing and site contract

Every screen has an address-bar URL, a route-specific `<title>`, one plain-word
`<h1>`, canonical URL, description, and announced route change. The public
shell is always header → main → footer and includes a skip link.

| Route | Milestone | Purpose and title |
| --- | --- | --- |
| `/` | M1 | Landing and entry point — `Booking Recovery Loop — recover paid sessions` |
| `/demo` and `/?demo=1` | M1 | Isolated sample workspace — `Demo — Booking Recovery Loop` |
| `/privacy` | M1 | Plain-language data use — `Privacy — Booking Recovery Loop` |
| `/terms` | M1 | Service and billing terms — `Terms — Booking Recovery Loop` |
| `/404` | M1 | Branded way back — `Page not found — Booking Recovery Loop` |
| `/auth/callback` | M2 | Entra redirect receiver; no customer content |
| `/app` | M2 | Authenticated practice home |
| `/app/settings/billing` | M2 | Subscription, hosted checkout, restore/manage subscription |
| `/b/:public_slug` | M3 | Public branded paid-session page |
| `/b/:public_slug/complete` | M3 | Sociobot return reconciliation page |
| `/app/recovery` | M4 | Recovery queue and delivery proof |
| `/app/settings/data` | M5 | Export and deletion controls |
| `/app/settings/integrations` | M6 | Booking-link, calendar, and webhook connections |

`robots.txt`, `sitemap.xml`, a real `404.html`, SVG favicon, Apple touch icon,
Open Graph/Twitter image, and CSP/security headers ship in M1. The app uses
History API navigation, restores focus to the new h1, announces navigation,
and never leaves a dead internal link.

### Identity and tenancy

M2 adds Sociobot Microsoft Entra External ID exactly as follows:

- Use `@azure/msal-browser` with PKCE, `loginRedirect`,
  `acquireTokenSilent`, scopes `openid profile email`, and `sessionStorage`
  cache. Public landing, booking pages, and demo remain unguarded.
- Default tenant config is `ENTRA_TENANT_ID=35c6fe40-0ec0-46b6-98c6-213ad4de6650`,
  `ENTRA_TENANT_SUBDOMAIN=sociobotcustomers`, and
  `ENTRA_CLIENT_ID=25c704f4-465a-47af-80ab-2c489466b697`. The browser uses the
  shared authority from the auth contract.
- The API gets discovery at startup, caches its issuer/JWKS one hour, and
  validates RS256, `aud`, `tid`, dynamic discovery `iss`, `exp`, and `nbf`.
  It keys users by immutable `oid`, never email. Invalid tokens receive `401`
  and `WWW-Authenticate: Bearer`.
- The factory must register
  `https://booking-recovery-loop.sociobot.in/auth/callback` on the shared SPA
  app before production M2 acceptance. The M2 handoff names this as operator
  action unless registration is confirmed.

Each query receives a `PracticeId` only after a membership lookup. Public
booking endpoints resolve only a non-guessable public page slug and never
return other practice data. Database access is tenant-scoped in repository
methods and PostgreSQL row-level security is enabled before production data.

### Data model

All IDs are UUIDv7. Timestamps are UTC with a named IANA timezone stored on the
practice and session page. Contact PII is encrypted before it reaches the
database with envelope encryption; the envelope key lives in the factory secret
store in production and a generated local development key lives outside Git.
Searchable contact lookup uses a separately salted, normalized hash. Logs redact
contact values and tokens.

| Entity | Ownership | Essential fields / relationships |
| --- | --- | --- |
| `user` | Entra `oid` | display name, created timestamp; no email as identity |
| `practice` | customer tenant | name, timezone, public brand, recovery defaults, deletion status |
| `practice_member` | practice + user | role `owner` or `operator`; unique membership |
| `service` | practice | name, duration, price/deposit amount, active flag |
| `booking_page` | practice + service | public slug, consent copy/version, availability source, Sociobot checkout product reference |
| `availability_slot` | booking page | start/end, source reference, held/available state; this is a display cache, not a calendar replacement |
| `booking_attempt` | practice + page | anonymous/session token, selected slot, state, contact reference after consent, attribution, expiry |
| `contact` | practice | encrypted name/email/phone, normalized hashes, channel consent state; never shared across practices |
| `consent_record` | contact + attempt | channel, wording/version, timestamp, IP hash, source, withdrawal timestamp |
| `booking` | practice + attempt | scheduled time, payment state, attendance state, cancellation state, public reference |
| `payment_session` | booking | Sociobot intent ID, checkout URL, deposit amount/currency, verification status; no card data |
| `recovery_rule` | practice | trigger, delay, permitted channel order, cap, active version |
| `recovery_case` | attempt or booking | reason, state, next action time, rule version, owner override, resolved reason |
| `outbound_message` | recovery case | channel, approved template/version, idempotency key, provider message reference, send state |
| `delivery_event` | outbound message | accepted, delivered, bounced, failed, clicked, timestamp, provider payload digest |
| `subscription` | practice | Sociobot product/plan reference, state, renewal/expiry; never a Dodo secret |
| `audit_event` | practice | actor, action, resource, redacted metadata, timestamp |
| `data_request` | practice | export/delete request state, expiry, operator evidence |
| `job` | system | typed payload reference, run time, attempts, locked-at, idempotency key, dead-letter reason |

### Billing and payment boundaries

- Product subscription: M2 registers the named **Recovery Loop Practice**
  subscription price ($29/month) in the Sociobot product registry. The client
  opens only a Sociobot-hosted checkout URL for the allowlisted plan. Dodo is
  merchant infrastructure behind Sociobot, never a browser SDK.
- The server maps the verified billing webhook/event to one `subscription` for
  one practice. It verifies the Sociobot signature, is idempotent, and logs no
  raw event body containing PII. A stale or cancelled subscription makes paid
  writes read-only after a plainly stated grace rule; export, deletion, and
  accessibility features stay available.
- The exact subscription checkout and webhook field names are taken from the
  factory's registered Sociobot billing contract in M2. Plan IDs live in server
  configuration and are allowlisted; the browser may request a plan but cannot
  set a price, product, or practice ID.
- Session deposits: M3 creates one Dodo-hosted checkout through Sociobot on
  the server, binds its intent to the booking, and verifies the returned
  Sociobot license before recording payment. It stores only intent IDs,
  license hashes, and status. It does not collect card data or trust a return
  URL alone.

### API, jobs, integrations, and operations

- Versioned JSON API under `/api/v1`; request and response schemas are
  validated at the edge. Every non-health endpoint has an IP-based limiter
  keyed by the first `X-Forwarded-For` hop (otherwise socket peer). Default is
  20 requests/second with burst 40; auth, booking create, checkout, webhook,
  export, and message actions have stricter route limits. A limit response is
  `429` with `Retry-After`. Health is the single exempt endpoint.
- Public booking writes use a short-lived, signed attempt token and a stricter
  per-IP/per-page limiter. Webhook routes additionally validate the provider
  signature before persistence. All writes have idempotency keys.
- A transactional outbox inserts `job` work with the database change. A worker
  claims due jobs using `FOR UPDATE SKIP LOCKED`, sends at most once by
  idempotency key, stores receipts, backs off retryable failures, and moves
  exhausted work to a visible dead-letter state. No cron task may silently
  disappear.
- Transactional email/SMS is configured through a server-side provider adapter
  with opt-in only. Development and demo use an in-process receipt provider;
  production credentials are factory provisioned. The product never contacts a
  messaging vendor from the browser. SMS cost and regional consent requirements
  are enforced by the provider configuration before enabling a practice.
- There is no runtime AI feature in M1–M6. If later research earns a recovery
  message drafting assistant, it must call only the Sociobot gateway from the
  server with a spend cap, rate limit, explicit preview of transmitted text,
  explicit user action, undo, and deterministic template fallback. Demo and
  tests use canned replies and never spend.
- `/health` returns status and build SHA. `/metrics` is internal/authenticated
  before exposure. JSON structured logs include request ID, route, status,
  latency, and redacted practice/user IDs. Sentry-like third-party tracking is
  not added; operational page views are aggregate and privacy-preserving.
- Postgres has daily encrypted backups and a quarterly restore drill. Exports
  are one-time, expiring download URLs. Deletion removes contacts/messages and
  redacts audit records after the required operational window; a nightly job
  purges expired demo tenants after 24 hours.

### Demo sandbox contract

`/demo` and `?demo=1` provision or select a random, unguessable demo workspace
seeded with a realistic coach practice, one 45-minute session, three booking
attempts, and delivery outcomes. It is marked `is_demo`, expires in 24 hours,
and is protected by the same public rate limiter. The API rejects any operation
that could read or write a real practice while its demo workspace token is
present. Browser-only state uses the `demo:` namespace, never a production key.

The persistent banner says **“Demo — sample data, nothing is saved”** and has
**Reset demo** and **Start for real**. Reset replaces the demo token and seeds
a fresh workspace. Demo uses a fake payment result and in-process delivery
receipt; it does not send email/SMS, create a checkout, authenticate, or
spend AI money. M1 writes `.factory/demo.md` with the seed, reset behavior,
TTL, API boundary, and all claim paths.

## 4. Design system

The visual system is defined in [.factory/design.md](design.md). It is part of
the product requirement, not later polish. M1 must implement the token file,
the responsive public shell, the demo banner, and the key state components.
Later milestones use the component inventory instead of introducing a generic
dashboard kit.

Minimum acceptance for every screen: 16 px body text, one visible primary
action, 44 px touch targets, designed focus rings, contrast of at least 4.5:1
for text, semantic landmarks, keyboard operation, announced async errors, no
motion beyond opacity/instant state under `prefers-reduced-motion`, and a
usable 390 px layout. Decorative scene elements have empty alt; meaningful art
explains the calm handoff from booking intent to confirmed appointment.

## 5. Milestones

Every milestone is sized for one focused 3–4 hour builder session. The
builder updates its status here only after automated verification and review
pass. Future claims are added to `.factory/claims.json` before the related copy
appears on a screen; one tagged test proves each claim from a clean sandbox.

### M1 — Public promise and isolated recovery-loop demo

**Status:** built, deployed, and verified; independent review/polish pending
**Goal:** A stranger can open the public page, start a sample workspace in one
click, run a representative abandoned-booking recovery loop, and understand
that it has not sent a real message.

**Routes/screens:** `/`, `/demo`, `?demo=1`, `/privacy`, `/terms`, `/404`;
landing, persistent demo banner, sample recovery board, sample booking view,
delivery receipt timeline, empty/loading/error/offline state examples.

**Build scope:** Implement the standard landing information order, plain-words
copy audit, demo server workspace with 24-hour expiry, `demo:` browser state,
seed/reset endpoint, and the following observable sequence: select the sample
client’s unfinished attempt → review recorded email consent → start its
accelerated recovery → see a simulated delivery receipt and outcome. A missing
consent must prevent the recovery action and explain why. The sample must not
call Stripe, a messaging provider, Entra, Dodo, or AI. Add sitemap, robots,
metadata, all security headers, and product-native 404. Use the design tokens
and component inventory, not a starter-template layout.

**M1 claims:** `demo-isolated`, `demo-reset`, `consent-gates-recovery`,
`demo-recovery-receipt`, and `demo-no-external-requests`, defined in
`.factory/claims.json` in this planning commit.

**Tests:**

- Vitest for sample seed, state transitions, consent policy, title metadata,
  and route selection.
- API integration tests for demo creation/reset/expiry boundary and the
  prohibition on real-tenant lookup from a demo token.
- Playwright test for every `@claim:` entry from a fresh browser context;
  capture screenshot and trace on failure.
- Playwright + axe for landing, demo, privacy, terms, and 404; keyboard-only
  smoke for skip link, banner controls, and recovery action.
- Record request URLs during the complete demo path and prove only same-origin
  URLs are used. Run `npm run build`, bundle-size check, and mobile Lighthouse
  (performance ≥90, accessibility ≥95).

**Definition of done:** All five M1 claims pass; the sample is useful without
an account; reset produces clean seed data; no demo action reaches real data or
an external paid service; all five public pages have title, one h1, main,
metadata, and working links; `npm test`, `npm run test:e2e`, `npm run build`,
and `cargo test --manifest-path backend/Cargo.toml` pass; `.factory/demo.md`,
`.factory/copy-audit.md`, and `.factory/handoff-m1.md` are written. Review and
polish must pass before M2.

### M2 — Practice accounts, persistence, and subscription access

**Status:** core persistence shipped in Polish 1; external Entra and Sociobot product registration unavailable
**Goal:** An owner can sign in, create a practice, retain its setup safely, and
start or manage the $29/month Recovery Loop Practice subscription.

**Routes/screens:** `/auth/callback`, `/app`, `/app/onboarding`,
`/app/settings/billing`.

**Build scope:** Implement Entra CIAM PKCE and server JWT validation, practice
and membership migrations with tenant-scoped repositories/RLS, onboarding for
practice name/timezone/service, session handling and sign-out, and the
Sociobot/Dodo hosted subscription checkout/manage path. Receive verified
billing events idempotently and show a current subscription state. The demo
continues to use its isolated workspace and does not require sign-in.

**Claims to add before copy:** authenticated owner isolation; practice setup
survives a new session; Recovery Loop Practice costs $29/month; cancelled
subscription makes new paid setup read-only while export remains available.

**Tests:** mock Entra discovery/JWKS for token accept/reject cases; migration
up/down test; API cross-tenant read/write denial; Playwright sign-in callback
fixture, checkout-link allowlist test, billing-event idempotency test; rate
limit test asserts `429` and `Retry-After`.

**Definition of done:** A registered shared Entra redirect URI is confirmed or
called out in handoff; no browser sees billing secrets; tenant isolation and
rate limits pass; subscription status comes from a verified server event; all
new claims, tests, migration reversal, demo regression, review, and handoff
pass.

### M3 — Real branded paid-session page

**Status:** repaired in Repair 8 with server-created Sociobot/Dodo checkout and verified completion
**Goal:** A practice can publish one focused session page that creates a real,
consented booking attempt and completes a Sociobot/Dodo-hosted deposit flow.

**Routes/screens:** `/b/:public_slug`, `/b/:public_slug/complete`,
`/app/pages`, `/app/pages/:id`.

**Build scope:** Create service/page editor, available-slot display/hold,
contact form with explicit email/SMS consent by channel, consent versioning,
attempt expiry, and a server-created Sociobot/Dodo checkout. Reconcile only
Sociobot-verified completion tokens into bookings/payment status. A return page
states that confirmation is being checked rather than claiming payment early.
The product owns a compact availability cache or imported slots only; it does
not become a full calendar.

**Claims to add before copy:** public page records a booking attempt; recovery
messages require channel consent; deposit completion appears only after a
verified webhook; an occupied/expired slot cannot be double-booked.

**Tests:** public form validation and keyboard flow; server-side intent and
slot concurrency integration test; signed webhook accept/reject/idempotency
tests; Playwright against billing fixtures/test mode with no card data in DOM or
logs; per-page/IP write-limit test; responsive/axe regression.

**Definition of done:** A pilot can create a paid booking without platform card
handling; consent wording and evidence are stored; payment state is webhook
truth; browser/API never leak cross-practice information; all claims, tests,
demo regression, migration reversal, review, and handoff pass.

### M4 — Reminder delivery and accountable fallback

**Status:** repaired in Repair 8 with a credentialed relay contract, HMAC callbacks, durable receipts, and one-bounce fallback; live provider provisioning remains operator-gated
**Goal:** The practice can protect a booked session with reminders, visible
delivery proof, and one consented fallback when the first delivery bounces or
fails.

**Routes/screens:** `/app/recovery`, `/app/recovery/:case_id`,
`/app/settings/recovery-rules`.

**Build scope:** Add rule editor with safe default caps, transactional outbox
worker, scheduler, provider adapter, receipt webhook endpoint, bounce/failure
state machine, fallback selection, stop/resolve action, and customer-facing
opt-out link. Seed monitoring for overdue jobs/dead letters. Messages use
practice-approved, plain-language templates—not AI—and never send in demo.

**Claims to add before copy:** scheduled reminder has a visible receipt;
bounced permitted email creates exactly one permitted fallback; stopped/opted
out recovery sends nothing; duplicate provider webhook does not duplicate a
message.

**Tests:** deterministic clock job tests; provider fake acceptance/delivery/
bounce fixtures; idempotency and retry/dead-letter integration tests; browser
recovery proof and stop test; 100 rps route-limit/load smoke documented in
handoff; outbound provider calls blocked in demo.

**Definition of done:** Every real send has consent and an audit trail; failure
is visible instead of silently lost; fallback caps work; job recovery survives
restart; claim suite, rate-limit verification, operations dashboard review,
and handoff pass.

### M5 — Data rights and operational control

**Status:** core export and deletion shipped in Polish 1
**Goal:** A practice can manage what is happening, export its records, and
request deletion without support intervention.

**Routes/screens:** `/app/settings/data`, `/app/settings/team`,
`/app/operations` (owner-only).

**Build scope:** Add CSV/JSON export with expiring download, deletion request
and clear confirmation, retention/purge jobs, practice-member management,
message/provider health summary, dead-letter retry/resolve controls, audit log,
health/metrics guard, backup/restore runbook, and privacy/terms refresh to
match actual processing.

**Claims to add before copy:** export contains the practice’s records; export
does not include another practice; deletion request stops new messages; a
failed job is visible and retryable.

**Tests:** export schema/data-isolation test; one-time download expiry;
deletion race with scheduled job; owner-role authorization; restore a
sanitized backup into a temporary database; operational UI axe test.

**Definition of done:** Customer data rights work without an operator; exports
remain available after billing lock; recurring backup and restore evidence is
in handoff; production logs remain redacted; all claims/tests/review pass.

### M6 — Distribution and lightweight integrations

**Status:** planned
**Goal:** A practice can put its recovery-aware session page where clients
already start and connect only the scheduling context it needs.

**Routes/screens:** `/app/settings/integrations`, `/app/share`,
`/embed/:public_slug`.

**Build scope:** Add copyable public link and accessible embed snippet,
calendar hold/ICS and selected calendar availability import, signed outbound
webhooks for booking/recovery events, integration disconnect/revocation, and
an installable lightweight page shell only if offline viewing adds proven
value. Keep source calendars authoritative; never grow a replacement calendar.

**Claims to add before copy:** copied link opens the published page; an embed
preserves keyboard access; an imported unavailable slot cannot be booked;
webhook signatures verify and retries are idempotent.

**Tests:** link/embed browser flow with responsive iframe constraints;
availability fixture import; webhook signature/retry tests; disconnect removes
tokens and stops sync; privacy request log; demo regression.

**Definition of done:** Sharing is one copy action, integrations fail visibly
and safely, revocation works, non-goal calendar boundaries remain intact, and
all new claims/tests/review/handoff pass.

## 6. Risks, unknowns, and retirement experiments

| Risk / unknown | Why it matters | Experiment / decision gate |
| --- | --- | --- |
| Owners may see abandoned attempts but lack a lawful contact route. | Recovery cannot be sent without consent. | In M1–M3 prototype, measure consent completion and simulation eligibility. Interview five pilots on wording. Do not enable real fallback if consent capture is confusing or low. |
| A $29 subscription may not cover message volume or fail to recover a session. | The price must sustain delivery and be worth paying. | Track (without marketing it as proof) message cost, eligible attempts, recoveries, and no-shows for 60 days across pilots. Keep a hard allowance and revisit tier only with evidence. |
| Sociobot/Dodo deposit products vary by amount and currency. | Payment truth must be reliable and compliant. | Register allowlisted deposit products through the factory billing process and verify each returned license before expanding currencies. |
| Delivery provider receipts are incomplete or delayed. | “Proof” becomes misleading. | M4 adapter contract distinguishes accepted, delivered, and unknown. Run controlled mailbox/phone tests across supported channels; show “awaiting receipt,” never infer delivery. |
| Reminder rules create annoyance or regulatory exposure. | Trust and compliance damage exceed recovered revenue. | Default cap of one recovery plus one allowed fallback, explicit opt-out, quiet-hours/timezone test, and review every pilot template before send. |
| Entra shared-app redirect registration blocks production sign-in. | M2 cannot authenticate a real owner. | Request/verify the specified callback registration before starting M2; keep M1 entirely public and demoable. |
| API abuse can create bogus booking attempts or costs. | Public pages and messaging are attack surfaces. | Load-test public/write limits, CAPTCHA-free abuse heuristics only after evidence, enforce server idempotency and `429`/`Retry-After` from M2 onward. |
| Small practices may prefer existing booking tools and reject a new page. | The wedge requires a low-switch path. | M6 validates embed/link and availability import before considering deeper calendar integrations; success is a live page beside—not replacing—their scheduler. |
| AI message drafting may make copy less compliant or increase cost. | It adds risk without solving the core workflow. | Do not build it. Reconsider only if pilots log substantial manual edits; then test a gateway-only, explicit-preview draft against a fixed template baseline. |

## 7. Release and handoff checklist

For every milestone: update this plan’s status, update claims before adding
claim-like copy, execute all listed unit/API/browser/accessibility checks,
record build SHA and verification results in `.factory/handoff-mN.md`, preserve
the demo contract, and run the review → polish loop. The final handoff includes
the deployment configuration names (never values), migration/rollback notes,
backup/restore evidence, known gaps, and a concise next-builder brief.
