# Adversarial first-read review 2 — FAIL

**Reviewed:** 2026-08-29 UTC
**Live URL:** https://booking-recovery-loop.sociobot.in
**Live build:** `7de273d65e1e9f34354d03ee9070a6a4fc4793be`
**Verdict:** **FAIL**

No product code was changed for this review.

## Cold first screen

Fresh Chromium contexts at 390 × 844 and 1440 × 1000 loaded `/` without scrolling. The live build made same-origin requests only and had no console errors or 390 px horizontal overflow.

| Question | Answer visible before scrolling |
| --- | --- |
| What does it do? | “Recover paid sessions before they disappear.” |
| For whom? | “For solo coaches, tutors, and consultants who need to act when a paid booking stops.” |
| What should I click first? | **Try it with sample data**; adjacent copy says it will show three fictional bookings and can be reset. |

The first-read gate passes. The mobile action is full-width and high contrast. The desktop appointment-rail artwork is original and product-specific, not a generic SaaS template.

## Findings

### F-2-1 (continuation of F-1-1) — BLOCKING / Critical — the automatic recovery loop is absent

**Exact evidence:** `.factory/brief.json` requires “a reminder delivery receipt, and an **automatic SMS/email fallback when a booking is abandoned or a reminder bounces**.” The live owner queue instead has the manual button **“Send permitted recovery.”** `src/main.ts` calls `recoverPracticeAttempt` only from that click. Repository search finds no scheduler, job queue, timer, or abandoned-booking trigger; the backend exposes only a manual recovery route and provider callbacks.

**Why this fails:** A solo practitioner must still notice an unpaid booking and send recovery themselves. An `awaiting_deposit` booking gets no delayed recovery or reminder. Bounce-to-SMS code helps after a provider callback, but does not repair the missing automatic abandoned-booking/reminder flow. This is the brief’s core job, not optional polish.

**Concrete fix:** Add a durable per-practice schedule: set a due time when an unpaid attempt is created, enqueue consent-checked recovery, schedule the permitted reminder, and make one SMS fallback conditional on an email bounce. Show queued/sent/stopped states and receipts. Add clean-sandbox claims with an accelerated clock for abandoned deposit, reminder, bounce, no consent, and idempotent restart/retry. Keep manual send only as an explicit retry.

### F-2-2 — BLOCKING / High — setup requires developer infrastructure and does not offer the stated plan

**Exact evidence:** `/start` requires **“Hosted payment URL”**, prefilled with `https://example.com/hosted-payment`, and has **“Delivery webhook URL (optional)”** with “Connect an HTTPS endpoint that accepts recovery messages and returns delivery receipts.” The brief specifies `$29/month per practice`; the landing, setup, and README show neither a price nor a Sociobot billing route.

**Why this fails:** The stated audience is a one-to-five-person coaching, tutoring, or consulting practice. It cannot finish first-run setup on a phone without independently operating checkout and message-webhook systems. A blank webhook cannot send recovery; `example.com` is not a payment service. There is no disclosed cost or way to obtain the promised practice subscription.

**Concrete fix:** Provision supported payment and delivery connections in practice setup with tested configuration and clear failures. Use the required Sociobot billing API for the subscription; show the exact $29/month price and an honest unavailable state only if setup is disabled. Test a new practice through an actual supported checkout/delivery configuration, not `example.com` or a hand-built webhook.

### F-2-3 (continuation of F-1-2) — BLOCKING / High — README assertions remain outside the claims ledger

**Exact evidence:** the 16-entry `.factory/claims.json` has no entry for these README statements: “Each statement above has one named test in `.factory/claims.json`.”; “The factory deploys that non-root container on `PORT`.”; “`/health` reports the build SHA.”; and “No secret is stored in this repository.”

**Why this fails:** The claims contract requires an entry and exact test for every assertion a reader can rely on. These are operational and security assertions, not instructions. In particular, the no-secret assertion cannot be audited through the stated claims contract.

**Concrete fix:** Delete assertions unnecessary to an operator, or add narrow entries and tests: a tracked-secret scan and a container/runtime check for the deployment and health assertions. Do not use a broad application test as unstated evidence.

### F-2-4 — Minor — one permission record has three names

**Exact evidence:** landing copy says “its **contact consent**” and “**Email consent** decides the next step”; the README says “**channel consent**”; and the demo panel says **“Email permission.”**

**Why this fails:** A first-time visitor cannot know whether these are one record, separate permissions, or an account setting. The product’s terminology table says “email consent or SMS consent.”

**Concrete fix:** Use **email consent** and **SMS consent** everywhere. Rewrite “Review a booking state, its contact consent, and each delivery receipt.” as “Review a booking, its email consent, and its delivery receipts.” Rename the demo panel “Email consent”; rewrite README “channel consent” as “email or SMS consent.”

## Demo and sandbox behaviour

The first-screen action reaches `/demo` in one click. Its first completed screen already shows North Star Coaching, three named fictional bookings, a £35 deposit, consent records, and a recovery case. The persistent banner says **“Demo — sample data, nothing is saved”** and includes **Reset demo** and **Start for real**.

In a fresh 390 px context, Maya Patel’s **Run sample follow-up** showed **“Delivered · simulated email”** with a timestamp. Reset changed `demo:workspace-token` and restored Maya’s unfinished state. The browser stored only that `demo:` key. The full load/recover/reset request log was same-origin only: no payment, delivery, sign-in, billing, analytics, font-CDN, or AI request. This confirms the isolated demo, but not F-2-1 or F-2-2.

## Declared claims

I cloned the current checkout to a fresh temporary directory, ran `npm ci`, and ran every exact `.factory/claims.json` command individually. All 16 passed. The final Playwright run reported `{"status":"passed","failedTests":[]}`.

| Claim id | Exact command result |
| --- | --- |
| `demo-isolated` | PASS — named Rust test |
| `demo-lifetime` | PASS — named Rust test |
| `forwarded-rate-limit` | PASS — named Rust test |
| `demo-no-account-payment` | PASS — 1 Playwright test |
| `demo-reset` | PASS — 1 Playwright test |
| `consent-gates-recovery` | PASS — 1 Playwright test |
| `demo-recovery-receipt` | PASS — 1 Playwright test |
| `demo-no-external-requests` | PASS — 1 Playwright test |
| `sample-three-bookings` | PASS — 1 Playwright test |
| `practice-publish` | PASS — 1 Playwright test |
| `booking-consent-record` | PASS — 1 Playwright test |
| `encrypted-tenant-data` | PASS — named Rust test |
| `export-delete` | PASS — named Rust test |
| `delivery-fallback-receipts` | PASS — named Rust test |
| `verified-deposit` | PASS — named Rust test |
| `no-double-booking` | PASS — named Rust test |

The tests prove the implemented manual flow, isolation, and fallback callback. They cannot prove the missing scheduled automation in F-2-1, because no such claim exists.

## Copy audit

Counts include visible headings, labels, and buttons. No landing or README item exceeds 22 words. “Mapped” means an instruction, scope boundary, or content covered by a semantic claim; the exceptions are findings above.

### Landing `/`

| Words | Copy | Result |
| ---: | --- | --- |
| 6 | Recover paid sessions before they disappear | Clear h1 |
| 15 | For solo coaches, tutors, and consultants who need to act when a paid booking stops. | Clear |
| 5 | Try it with sample data | Result-naming verb |
| 11 | See three fictional bookings, then reset the sample at any time. | Mapped |
| 4 | Demo needs no account | Mapped |
| 4 | Demo sends no messages | Mapped |
| 4 | Demo opens no payment | Mapped |
| 6 | Ready to use your own booking? | Clear prompt |
| 4 | Set up your practice. | Result-naming link |
| 3 | One booking stopped. | Clear |
| 6 | Email consent decides the next step. | Clear terminology baseline |
| 2 | Sample view | Useful label |
| 3 | Sample recovery board | Clear heading |
| 11 | Review a booking state, its contact consent, and each delivery receipt. | F-2-4 |
| 3 | Tue · 14:00 | Sample time |
| 2 | Booking started | Clear status |
| 4 | Service and time chosen | Clear status |
| 2 | Booking recorded | Clear status |
| 3 | 18 minutes ago | Sample time |
| 3 | Deposit not finished | Clear status |
| 3 | Email consent recorded. | Clear |
| 3 | Needs a follow-up | Clear status |
| 1 | Next | Contextual status |
| 2 | Delivery receipt | Clear label |
| 5 | Waiting for a permitted action | Clear status |
| 2 | Not started | Clear status |
| 2 | Three steps | Useful label |
| 4 | How booking recovery works | Clear heading |
| 4 | Find the stopped booking | Clear step |
| 9 | See the chosen session and where the client left. | Clear |
| 4 | Check email consent first | Clear step |
| 9 | A follow-up stays stopped when email consent is missing. | Mapped |
| 4 | Read the delivery receipt | Clear step |
| 9 | The sample action ends with a timestamped simulated receipt. | Mapped |
| 2 | Product scope | Useful label |
| 6 | It does not replace your calendar | Clear boundary |
| 13 | It is not a CRM, a marketplace, or a tool for bulk messages. | Clear boundary |
| 6 | Read how booking data is handled | Result-naming link |
| 4 | Use your own bookings | Useful label |
| 4 | Create a practice workspace | Clear heading |
| 11 | Publish one session page, capture channel consent, and review delivery receipts. | F-2-4 |
| 4 | Set up your practice | Result-naming link |
| 8 | Review stopped bookings, contact consent, and delivery receipts. | F-2-4 |
| 1 | Privacy | Clear link |
| 1 | Terms | Clear link |
| 4 | Built by Param Factory | Attribution |
| 7 | Original rail artwork made for this product. | Provenance |

### README.md

| Words | Sentence or reader-facing item | Result |
| ---: | --- | --- |
| 15 | Booking Recovery Loop helps solo coaches, tutors, and consultants act when a paid booking stops. | F-2-1: automation incomplete |
| 17 | A practice can publish one session page, record channel consent, open hosted payment, and review provider receipts. | F-2-4 |
| 6 | Try the isolated sample at `https://booking-recovery-loop.sociobot.in/?demo=1`. | Clear instruction |
| 10 | It opens three fictional bookings without an account or payment. | Mapped |
| 12 | Demo actions stay on the product origin and send no real message. | Mapped |
| 7 | Create a private practice workspace at `/start`. | Mapped |
| 7 | Publish the generated `/b/<slug>` session page. | Mapped |
| 11 | A client records email or SMS consent before hosted payment opens. | Mapped |
| 10 | A time already held by another active booking is rejected. | Mapped |
| 10 | The payment provider confirms a deposit through the authenticated callback. | Mapped |
| 11 | A connected delivery service returns accepted, delivered, bounced, or failed receipts. | Mapped |
| 9 | One permitted SMS fallback can follow an email bounce. | Mapped |
| 10 | Export or delete the complete practice from `/app/settings/data`. | Mapped |
| 8 | Client contact fields are encrypted before database storage. | Mapped |
| 11 | Owner tokens scope every private read and write to one practice. | Jargon: “A private access key limits every read and write to one practice.” |
| 11 | The demo uses a separate token, schema path, and fictional seed. | Jargon: “The demo uses separate sample storage and fictional bookings.” |
| 11 | Each statement above has one named test in `.factory/claims.json`. | F-2-3 |
| 8 | Requirements: Node 22+, npm, and current stable Rust. | Clear instruction |
| 7 | Open `http://127.0.0.1:8080`. | Clear instruction |
| 12 | Optional settings are `PORT`, `DATABASE_URL`, `STATIC_DIR`, and `CONTACT_KEY_FILE`. | Clear reference |
| 17 | With no settings, the container creates its SQLite database and encryption key under its writable data directory. | Clear deployment behaviour |
| 8 | Playwright starts the complete Rust and Vite build. | Clear verification behaviour |
| 14 | It checks each browser claim plus keyboard, route, mobile, offline, privacy, and axe coverage. | Jargon: replace “axe” with “accessibility” |
| 5 | Build the production image with: | Clear lead-in |
| 8 | The factory deploys that non-root container on `PORT`. | F-2-3 |
| 5 | `/health` reports the build SHA. | F-2-3; “build version” is plainer |
| 7 | No secret is stored in this repository. | F-2-3 |
| 1 | MIT. | Clear legal label |
| 17 | Fraunces and Atkinson Hyperlegible Next use the SIL Open Font License; their texts are in `public/fonts/`. | Clear legal information |

## History, structure, and quality checks

All earlier `.factory/review-*.md`, `.factory/polish-*.md`, verification, and handoff files were read.

| Earlier finding | Current confirmation |
| --- | --- |
| F-1-1, product absent | Partly fixed: setup, public booking, consent, encryption, export/delete, payment callback, and bounce fallback exist. Still blocking as F-2-1 because automatic abandonment/reminder recovery is absent. |
| F-1-2, incomplete claims | Improved from 8 to 16 entries. Still blocking as F-2-3 for the listed README assertions. |
| F-1-3, metaphor headings | Fixed: “Sample recovery board” and “How booking recovery works.” |
| F-1-4, unexplained eyebrow | Fixed: removed. |
| Concurrent 500s; clean timeout; 200% reflow; footer targets; cache headers; 200 unknown route; consent console error; Rust image; rate-limit mismatch | Confirmed fixed by current code/tests or live checks. `/missing-page` is a designed HTTP 404; cold load had no console errors; assets use immutable caching. |

Route checks found route-specific titles, descriptions, canonicals, one `h1`, and one `main` on home, demo, setup, app, data controls, Privacy, Terms, and 404. Header/footer, skip link, Privacy/Terms, favicon, OG image, robots, sitemap, back-button heading focus, and product-native 404 are present. The normal static crawl found no dead internal links. Accessibility tests cover keyboard use and axe serious/critical checks. `npm test`, `npm run build`, and the Rust suite passed locally; the build produced `dist/` with 12.12 kB gzip JavaScript.

AI is not missing leverage: a model would not make this recovery loop more trustworthy than reliable scheduled, consent-gated delivery. No AI key is embedded.

## What would make this perfect

Implement the automatic recovery/reminder schedule and supported first-party onboarding/billing in F-2-1 and F-2-2. Remove or test every remaining README assertion, then use one consent term consistently. Re-run this full review. A PASS requires zero findings.
