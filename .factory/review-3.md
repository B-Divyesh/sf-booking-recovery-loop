# Adversarial first-read review 3 — FAIL

**Reviewed:** 2026-08-29 UTC

**Live URL:** `https://booking-recovery-loop.sociobot.in`

**Live build:** `256bde53b0e8107421ceda018d4b3a61203ce894`

**Verdict:** **FAIL**

No product code was changed for this review.

## Cold first screen

Fresh Chromium contexts at 390 × 844 and 1440 × 1000 opened `/` without
scrolling. Both had zero console errors, same-origin requests only, and no
horizontal overflow.

| Question | Answer visible before scrolling |
| --- | --- |
| What does it do? | It recovers a paid-session booking that stopped. |
| For whom? | “For solo coaches, tutors, and consultants…” |
| What should I click first? | **Try it with sample data**; the adjacent sentence says it opens three fictional bookings and can be reset. |

The first-read gate passes at both sizes. On the 390 px screen, the headline,
audience, sample action, explanation, all three demo facts, and real setup link
are visible within the first 844 px.

## Findings

### F-3-1 — BLOCKING / High — the live 12-write rate-limit claim has regressed

**Exact quote/location:** `.factory/claims.json`, `forwarded-rate-limit`:
“Demo writes allow 12 immediate requests per first forwarded client IP, then
return a retry time of at least one second.” The same behavior is promised on
`/terms`: “A limited request returns a retry time.”

**Observed:** From one new `X-Forwarded-For` identity, 24 sequential valid
`POST /api/v1/demo/workspaces` requests returned `201`. Request 25 was the first
`429`, with `Retry-After: 56`. Each response advertised
`X-RateLimit-Limit: 12`, while remaining counts repeated in pairs: 11, 11, 10,
10, through 0, 0. An initial independent run also allowed request 13. This is
the same multi-bucket pattern found in earlier verification, despite
`deploy/containerapp.m1.json` specifying one replica.

**Why this misleads:** The deployed product allows twice the stated immediate
writes. A passing single-router unit test does not prove the production
boundary that the visitor receives.

**Concrete fix:** Enforce the limit in shared storage or ensure one effective
production limiter, then add a post-deploy claim test that sends 13 valid writes
through the public ingress from one client identity and requires requests 1–12
to return `201` and request 13 to return `429` with `Retry-After >= 1`.

### F-3-2 — BLOCKING / High — F-2-2 and the remaining F-1-1 are still open

**Exact quote/location:** live `/start` requires empty **Hosted deposit URL**
and **Delivery connection URL** fields. Its help says, “Use the hosted deposit
page your practice already uses” and “This receives automatic recovery and
reminder requests, then reports delivery receipts to the callback shown after
setup.” The landing page says, “Recovery Loop Practice is $29 per month for one
practice with one to five practitioners,” immediately followed by
“Subscription checkout is not available yet.”

**Why this blocks the audience:** A solo tutor, coach, or consultant cannot
finish setup unless they already operate both a hosted payment page and a
custom HTTPS message endpoint that accepts requests and sends callbacks. The
product offers no provider connection flow, no test message, no practitioner
management, and no purchasable $29 plan. The blank fields are more honest than
the old `example.com` default, but they do not fix the earlier finding. The
repository test substitutes `payments.example.test` and
`messages.example.test`, so it proves form wiring rather than a normal
customer’s end-to-end setup.

**Concrete fix:** Provide a supported, non-developer setup path for hosted
deposit and email/SMS delivery, including connection verification and a test
message. Register and open the $29 practice plan through the Sociobot billing
API. Add the promised practitioner access or remove “one to five
practitioners.” Verify a fresh practice through real supported provider
configuration, automatic recovery, receipt, export/deletion, and subscription
checkout.

### F-3-3 — BLOCKING / High — the claims ledger is incomplete again (F-1-2 / F-2-3)

**Exact quote/location:** README line 13: “The service queues an
abandoned-booking recovery after 15 minutes.” Live `/start`: “Recovery starts
automatically 15 minutes after an unpaid booking.” README lines 23–24 then say,
“Every product promise above is listed with its executable evidence in
`.factory/claims.json`.”

**Why this misleads:** No claim entry states the 15-minute delay. The
`automatic-recovery` test overwrites every job’s `due_at` with `1`, so it proves
consent gating and idempotent execution but never checks the advertised delay.
The README’s claim-completeness statement is therefore false. This revives the
earlier incomplete-ledger finding.

**Concrete fix:** Add a separate `automatic-recovery-delay` claim. Its test
must create a booking with a controlled clock and assert the scheduled due time
and allowed delivery window. Rewrite the README sentence in plain words, for
example: “The service schedules a consented recovery message 15 minutes after
an unpaid booking.”

### F-3-4 — High — the privacy summary contradicts the product’s own data list

**Exact quote/location:** live `/privacy`: “A real practice stores only the
booking and consent records needed for recovery.” The same page later says,
“The service stores practice settings, booking attempts, email or SMS consent,
and delivery receipts.” The schema also stores scheduled jobs and provider
configuration.

**Why this misleads:** “Only” gives a narrower data inventory than the product
actually stores. A visitor deciding whether to enter client data receives two
different answers on one page.

**Concrete fix:** Replace the summary with “A real workspace stores practice
settings, bookings, consent records, scheduled messages, and delivery
receipts.” Add a `practice-data-inventory` claim and a test that compares the
stored/exported record types with the privacy-page list.

### F-3-5 — High — the card-data privacy promise is not in `claims.json`

**Exact quote/location:** live `/start`: “Card details never enter this
product.” Live `/privacy`: “Payment card details stay on the practice’s hosted
payment page and never enter this product.” Live `/terms`: “This product does
not collect card details…”

**Why this misleads:** This is an absolute privacy claim. The existing
`booking-consent-record` entry proves that a configured hosted URL opens, but
the ledger does not list or test the stronger card-data boundary.

**Concrete fix:** Add a `card-data-excluded` claims entry and a browser/server
test that confirms the product renders no card fields and sends no card data in
the booking request before navigating to the configured payment origin. Or
remove the absolute promise.

### F-3-6 — Minor — the headline uses a metaphor instead of the exact job

**Exact quote/location:** landing `h1`: “Recover paid sessions before they
disappear.”

**Why this slows a first read:** Sessions do not literally disappear. The
product recovers unfinished bookings, so the metaphor makes the scope less
precise than the sentence beneath it.

**Concrete fix:** Use “Recover unfinished paid-session bookings.”

### F-3-7 — Minor — the README heading does not name its section

**Exact quote/location:** README line 9: “Use it.”

**Why this is weak out of context:** A heading list does not reveal that this
section explains the booking recovery workflow.

**Concrete fix:** Rename it “How booking recovery works.”

### F-3-8 — Minor — “queues” is implementation jargon in customer workflow copy

**Exact quote/location:** README line 13: “The service queues an
abandoned-booking recovery after 15 minutes.”

**Why this is harder to use:** A practice owner needs to know what will happen,
not which internal queue operation occurs.

**Concrete fix:** After adding the timing claim test, write: “The service
schedules a consented recovery message 15 minutes after an unpaid booking.”

### F-3-9 — Minor — “verified payment event” is implementation jargon

**Exact quote/location:** README line 14: “A verified payment event queues a
session reminder.”

**Why this is harder to use:** “Event” and “queues” describe the implementation
rather than the practice-visible result.

**Concrete fix:** Write: “After the payment provider confirms the deposit, the
service schedules one session reminder.”

## Demo and sandbox

The first-screen action opens `/demo` in one click. The first completed screen
already shows North Star Coaching, a 45-minute session, a £35 deposit, and
three named bookings: Maya Patel needs follow-up, Jordan Lee lacks email
consent, and Alex Morgan is complete.

The persistent banner says **Demo — sample data, nothing is saved** and exposes
**Reset demo** and **Start for real**. Maya’s action produced a timestamped
“Delivered · simulated email” receipt. Reset changed the
`demo:workspace-token` and restored all three original states. Start for real
removed the demo token and opened `/start`.

For browser isolation, I seeded a `practice:access-token` sentinel before
entering the demo. It remained unchanged through recovery and reset. Every
network request in the flow stayed on
`https://booking-recovery-loop.sociobot.in`; the only API requests were demo
workspace creation, sample recovery, and reset. There were no console errors.
The backend isolation test also passed from the clean clone.

## Declared claims

The repository was cloned without local objects into
`/tmp/booking-recovery-review3.8ZBrtp`, then `npm ci` and every exact command in
`.factory/claims.json` were run in ledger order. All 19 entries passed locally.
F-3-1 records the separate live-production mismatch.

| Claim id | Result |
| --- | --- |
| `demo-isolated` | PASS — named Rust test |
| `demo-lifetime` | PASS — named Rust test |
| `forwarded-rate-limit` | PASS locally; **fails live behavior**, F-3-1 |
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
| `automatic-recovery` | PASS — named Rust test |
| `automatic-reminder` | PASS — named Rust test |
| `practice-plan-price` | PASS — 1 Playwright test |

F-3-3 and F-3-5 are unlisted claims, so no declared test exists for them.

## Copy audit

Counts treat hyphenated strings and URLs as one word. Navigation, headings,
labels, statuses, and actions are included so that no visible copy is skipped.
No item exceeds 22 words and no banned marketing adjective appears.

### Landing `/`

| Words | Exact copy | Audit |
| ---: | --- | --- |
| 3 | Booking Recovery Loop | Wordmark; clear |
| 1 | Demo | Navigation; clear |
| 2 | Set up | Navigation; clear |
| 2 | Recovery queue | Navigation; clear |
| 1 | Privacy | Navigation; clear |
| 6 | Recover paid sessions before they disappear | F-3-6 |
| 15 | For solo coaches, tutors, and consultants who need to act when a paid booking stops. | Clear |
| 5 | Try it with sample data | Result-naming action |
| 11 | See three fictional bookings, then reset the sample at any time. | Clear; claims mapped |
| 4 | Demo needs no account | Claim mapped |
| 4 | Demo sends no messages | Claim mapped |
| 4 | Demo opens no payment | Claim mapped |
| 6 | Ready to use your own booking? | Clear prompt |
| 4 | Set up your practice. | Result-naming action |
| 3 | One booking stopped. | Clear |
| 6 | Email consent decides the next step. | Clear |
| 2 | Sample view | Clear label |
| 3 | Sample recovery board | Clear heading |
| 10 | Review a booking, its email consent, and each delivery receipt. | Clear |
| 3 | Tue · 14:00 | Sample time |
| 2 | Booking started | Clear state |
| 4 | Service and time chosen | Clear state |
| 2 | Booking recorded | Clear state |
| 3 | 18 minutes ago | Sample time |
| 3 | Deposit not finished | Clear state |
| 3 | Email consent recorded. | Clear state |
| 3 | Needs a follow-up | Clear state |
| 1 | Next | Contextual label |
| 2 | Delivery receipt | Clear label |
| 5 | Waiting for a permitted action | Clear state |
| 2 | Not started | Clear state |
| 2 | Three steps | Clear label |
| 4 | How booking recovery works | Clear heading |
| 4 | Find the stopped booking | Clear step heading |
| 9 | See the chosen session and where the client left. | Clear |
| 4 | Check email consent first | Clear step heading |
| 9 | A follow-up stays stopped when email consent is missing. | Claim mapped |
| 4 | Read the delivery receipt | Clear step heading |
| 9 | The sample action ends with a timestamped simulated receipt. | Claim mapped |
| 2 | Product scope | Clear label |
| 6 | It does not replace your calendar | Clear heading |
| 13 | It is not a CRM, a marketplace, or a tool for bulk messages. | Clear boundary |
| 6 | Read how booking data is handled | Result-naming action |
| 4 | Use your own bookings | Clear label |
| 4 | Create a practice workspace | Clear heading |
| 15 | Recovery Loop Practice is $29 per month for one practice with one to five practitioners. | F-3-2 |
| 13 | Publish one session page, record email or SMS consent, and review delivery receipts. | F-3-2 |
| 6 | Subscription checkout is not available yet. | Honest, but confirms F-3-2 |
| 8 | You can set up a practice workspace now. | F-3-2: setup requires provider infrastructure |
| 4 | Set up your practice | Result-naming action |
| 10 | Review stopped bookings, email or SMS consent, and delivery receipts. | Clear footer summary |
| 1 | Privacy | Clear link |
| 1 | Terms | Clear link |
| 4 | Built by Param Factory | Clear attribution |
| 7 | Original rail artwork made for this product. | Useful provenance; documented in `.factory/design.md` |

### README.md

| Words | Exact copy | Audit |
| ---: | --- | --- |
| 3 | Booking Recovery Loop | Clear title |
| 14 | Recover a stopped paid booking with recorded email or SMS consent and delivery receipts. | Clear; mapped claims |
| 8 | It is for solo coaches, tutors, and consultants. | Clear audience |
| 6 | Try the isolated sample at `https://booking-recovery-loop.sociobot.in/?demo=1`. | Clear instruction |
| 2 | Use it | F-3-7 |
| 11 | Create a practice at `/start` and publish its `/b/<slug>` booking page. | Claim mapped |
| 11 | A client records email or SMS consent before hosted payment opens. | Claim mapped |
| 9 | The service queues an abandoned-booking recovery after 15 minutes. | F-3-3 and F-3-8 |
| 8 | A verified payment event queues a session reminder. | F-3-9 |
| 7 | Delivery receipts appear in the practice queue. | Claim mapped |
| 9 | A bounced email can use one permitted SMS fallback. | Claim mapped |
| 7 | Export or delete practice data from `/app/settings/data`. | Result-naming instruction; claim mapped |
| 13 | Recovery Loop Practice is $29/month for one practice with one to five people. | F-3-2 |
| 4 | Checkout is currently unavailable. | Honest, but confirms F-3-2 |
| 6 | The demo has separate sample storage. | Claim mapped |
| 7 | It sends no real messages or payments. | Claims mapped |
| 12 | Every product promise above is listed with its executable evidence in `.factory/claims.json`. | False because of F-3-3 |
| 2 | Run locally | Clear heading |
| 8 | Requirements: Node 22+, npm, and current stable Rust. | Clear instruction |
| 2 | Open `http://127.0.0.1:8080`. | Clear instruction |
| 1 | Verify | Clear heading |
| 1 | Deploy | Clear heading |
| 10 | Set `PORT` when you need a port other than 8080. | Clear instruction |
| 6 | Use `/health` for a health check. | Clear instruction |
| 1 | License | Clear heading |
| 1 | MIT. | Clear license |
| 17 | Fraunces and Atkinson Hyperlegible Next use the SIL Open Font License; their license texts are in `public/fonts/`. | Clear legal information |

## Earlier findings rechecked

Every earlier `review-*.md`, `polish-*.md`, handoff, and verification report in
`.factory/` was read. Each prior issue was checked against both current code
and the live deployment.

| Earlier finding | Current result |
| --- | --- |
| F-1-1, real product absent | **Partly fixed, still BLOCKING through F-3-2:** real setup, booking, consent, scheduled recovery/reminder, receipts, export, and deletion now exist; supported provider onboarding and the paid subscription still do not. |
| F-1-2 / F-2-3, incomplete claims | **Regressed, BLOCKING as F-3-3:** the exact 15-minute promise is not listed or tested. The earlier README deployment assertions were removed. |
| F-1-3, metaphor section headings | Fixed: “Sample recovery board” and “How booking recovery works” remain. |
| F-1-4, unexplained “proof” eyebrow | Fixed: the eyebrow remains removed and “delivery receipt” is consistent. |
| F-2-1, automatic recovery absent | Fixed in code: durable jobs, a 30-second worker, consent stop, retry, and reminder scheduling are present; both declared Rust tests pass. |
| F-2-2, developer infrastructure and no plan | **Still BLOCKING as F-3-2.** |
| F-2-4, three consent terms | Fixed: visible product copy consistently uses email consent and SMS consent. |
| Concurrent demo recovery returned 500 | Fixed: `eight_concurrent_recoveries_never_return_server_error` passes. |
| First cold claim command timed out | Fixed: the first clean-clone Rust build and every later exact claim command passed. |
| 200% mobile reflow and footer targets | Fixed: current Playwright coverage passes. |
| Static assets lacked immutable caching | Fixed locally and confirmed by the live hashed asset response. |
| Unknown route returned 200 | Fixed: live `/missing-review-3` returns a designed HTTP 404. |
| Expected consent rejection logged a console error | Fixed: current test and live demo have no unexpected console error. |
| Rust base image was floating | Fixed: both Dockerfiles use `rust:1-slim`. |
| Live limiter allowed more than 12 writes | **Regressed, BLOCKING as F-3-1.** |

## Structure, routing, accessibility, and visual identity

- `/`, `/demo`, `/start`, `/app`, `/app/settings/data`, `/privacy`, `/terms`,
  and `/404` return `200`. A missing route returns the designed app 404 with
  HTTP `404`.
- Every checked route has a route-specific title, description, canonical and
  Open Graph URL, exactly one `h1`, one `main`, a skip link, consistent header
  and footer, and Privacy/Terms links. Titles follow the required pattern.
- All rendered internal anchor targets returned `200`. `robots.txt`,
  `sitemap.xml`, SVG favicon, Apple touch icon, and 1200 × 630 social image are
  present.
- A live navigation to `/start` focused its `h1`; browser Back returned to `/`
  and focused the home `h1`.
- The factory URL verifier passed with zero console errors. Live axe checks at
  1440 px and 390 px found zero violations. The full local 26-test Playwright
  suite passed keyboard, focus, 200% reflow, 44 px targets, offline state,
  reduced request scope, and serious/critical axe checks.
- The twilight rail, ticket shapes, restrained amber signal, self-hosted type,
  and designed lost-ticket 404 are recognizably product-specific. This is not
  a generic centered-gradient SaaS template.
- The clean clone also passed 10 Vitest tests, 15 Rust tests, deployment check,
  build, and size check. `dist/` was produced; JavaScript is 12,323 bytes gzip.

## Missed leverage

AI would not improve this deterministic consent, scheduling, and receipt job.
No AI provider key is embedded. Export and deletion exist. The obvious missing
leverage is the supported payment and delivery connection flow already covered
by F-3-2; requiring a custom webhook transfers the core product work back to
the customer.

## What would make this perfect

Restore the production rate-limit contract, complete non-developer provider
onboarding and Sociobot subscription checkout, register and test the exact
15-minute promise and card-data boundary, correct the privacy inventory, and
apply the four copy rewrites. Then rerun every claim through the public ingress
as well as from a clean clone. A PASS requires zero remaining findings.
