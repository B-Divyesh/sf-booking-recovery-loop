# Adversarial first-read review 1 — FAIL

**Reviewed:** 2026-08-29 UTC  
**Live URL:** `https://booking-recovery-loop.sociobot.in`  
**Live build:** `4bc479de1f4d464bfd071bd3b2f0a4bb7c659aa5`  
**Verdict:** **FAIL**

This is a review of the deployed product as a cold, first-time visitor and of
the checkout at `c121fdb`. Product code was not changed.

## First screen, cold browser

Fresh Chromium contexts at 390 × 844 and 1440 × 1000 loaded `/` with no
scroll, no console errors, no requests off this origin, and no horizontal
overflow at 390 px.

I can answer the three first-read questions from the first screen:

| Question | Answer from the screen |
| --- | --- |
| What does it do? | It helps recover a paid session whose booking stopped. |
| For whom? | “For solo coaches and tutors …” |
| What should I click? | “Try it with sample data”; it says it opens three fictional clients. |

The first-read gate therefore passes for the **sample**. It does not pass for
the actual product in the brief, because the same landing page later says,
“The paid plan is not open in M1. Accounts and hosted checkout arrive in M2.”

## Findings

### F-1-1 — BLOCKING / Critical — the real paid-booking recovery product is absent

**Exact evidence:** live landing, Practice plan: “The paid plan is not open in
M1. Accounts and hosted checkout arrive in M2.” The demo says “Demo actions use
an in-process mailbox. No email leaves this site.” `Start for real` only clears
the demo token and returns to that M1 notice. The README likewise says CIAM,
PostgreSQL customer tenancy, and the `$29/month` subscription are “M2 scope.”

**Why this fails:** The brief's customer is a small practice that must recover
its own abandoned or at-risk paid booking. A visitor cannot create a practice,
publish a branded paid-session page, collect a hosted deposit, detect a real
abandonment, send a consent-aware email/SMS reminder or fallback, inspect real
delivery/bounce evidence, export/delete encrypted customer data, or purchase
the stated plan. The sole end-to-end outcome is a fictional, isolated,
simulated-email sample. It is a demonstration, not the job-to-be-done.

This is the same unfixed Critical finding in `verification.md`,
`verification-3.md`, `verification-4.md`, and the preceding `handoff.md`.
Those reports did not assign an `F-*` identifier; this review assigns the
stable identifier `F-1-1`.

**Concrete fix:** Implement the real practice workflow before release: account
and tenant onboarding; a branded public paid-session page; hosted deposit
collection through the permitted Sociobot billing boundary; consent capture;
scheduled consent-aware recovery/reminder delivery and bounce fallback; real
delivery evidence and outcomes; encrypted contact-data storage; export and
deletion; and the payable `$29/month` plan. Keep `/demo` isolated and
simulated. Add an observable claim test for each new promise.

### F-1-2 — BLOCKING / High — the claims inventory does not cover material landing and README promises

**Exact evidence:** `.factory/claims.json` has eight entries. Its tests cover
demo isolation/lifetime/rate limiting, no account/payment, reset, consent,
simulated receipt, and no external demo requests. It has no entry for the
following visitor-reliant claims:

- Landing: “Opens a safe workspace with three fictional clients.”
- Landing: “Each ticket keeps the booking state, permission, and delivery
  evidence together.”
- Landing: “For one practice with one to five practitioners.” and “$29 / month.”
- README: “Booking Recovery Loop helps … recover a stopped paid booking without
  losing the consent record or delivery evidence.”
- README: “M1 ships a public product page and a working, isolated recovery
  sandbox.”
- README: “It uses three fictional bookings, never sends a real message, and
  needs no account.”
- README: “A server-enforced stop when the sample has no email consent.”
- README: “A simulated, timestamped delivery receipt from an in-process
  mailbox.”
- README: “A reset control … restores the seed.”
- README: all storage, no-personal-data, migration, one-replica, and
  deployment-boundary assertions in the Data and migrations / Deployment
  sections.

The prior `verification.md` reported this gap. The inventory expanded from five
to eight entries, but these unlisted claims remain. In particular, the
`demo-no-external-requests` test can show same-origin browser requests; it
cannot by itself prove every quoted storage, product-outcome, pricing, or
server-behaviour statement.

**Why this fails:** A visitor is asked to rely on these statements, yet the
required single claims ledger cannot point to a test that proves them. This
makes the advertised product hard to audit and violates the claims contract.

**Concrete fix:** Either remove each unsupported statement or add a separate
claims entry and one clean-demo observable test per claim. Do not use one broad
network test as evidence for unrelated server, pricing, or product claims. Once
F-1-1 is fixed, register and test the real delivery, billing, data-rights, and
recovery claims too.

### F-1-3 — Minor — two section headings use product metaphor instead of naming their contents

**Exact evidence:** landing `/`, `h2` “See the break in the booking loop” and
`h2` “Follow one accountable path.”

**Why a visitor is lost:** Heard alone in a screen-reader heading list, neither
names the section. “Break” and “path” are metaphors; the first title does not
say that it is a sample board and the second does not say it contains recovery
steps.

**Concrete fix:** Change them to “Sample recovery board” and “How booking
recovery works.”

### F-1-4 — Minor — “Booking follow-up with proof” is unexplained jargon on the first screen

**Exact evidence:** landing eyebrow above the h1: “Booking follow-up with
proof.”

**Why a visitor is lost:** “Proof” does not identify what is proven and the
phrase adds no usable instruction beyond the h1. The product elsewhere calls
the thing a delivery receipt/evidence, creating competing terms.

**Concrete fix:** Delete the eyebrow, or replace it with the concrete label
“Recovery status and delivery receipts.” Use “delivery receipt” consistently.

## Demo and sandbox checks

`/demo` opens in one click from the first screen and immediately shows North
Star Coaching, three named fictional bookings, a £35 deposit, consent records,
and an unfinished Maya Patel case. The persistent banner says “Demo — sample
data, nothing is saved” and includes **Reset demo** and **Start for real**.

From a fresh mobile context, I selected Maya, clicked **Run sample follow-up**,
and observed a timestamped “Delivered · simulated email” receipt. **Reset
demo** changed `demo:workspace-token` and restored the original unfinished
Maya state. The complete logged demo flow used only
`booking-recovery-loop.sociobot.in` requests. This passes for the implemented
demo; it does not cure F-1-1.

## Declared claims: clean-clone results

A detached clean worktree at `c121fdb`, with `npm ci`, was used. Every exact
command in `.factory/claims.json` passed:

| Claim | Result |
| --- | --- |
| `demo-isolated` | PASS — 1 Rust test |
| `demo-lifetime` | PASS — 1 Rust test |
| `forwarded-rate-limit` | PASS — 1 Rust test |
| `demo-no-account-payment` | PASS — 1 Playwright test (9.8 s cold) |
| `demo-reset` | PASS — 1 Playwright test |
| `consent-gates-recovery` | PASS — 1 Playwright test |
| `demo-recovery-receipt` | PASS — 1 Playwright test |
| `demo-no-external-requests` | PASS — 1 Playwright test |

`npm test`, `npm run build`, and `npm run check:backend` also passed in that
worktree (9 Vitest and 9 Rust tests; `dist/` produced; initial JavaScript
8.39 kB gzip).

## History confirmation

Every earlier `verification*.md` and `handoff*.md` was read. There are no
earlier `review-*.md` or `polish-*.md` files. Rechecked results:

| Earlier finding | Current status and evidence |
| --- | --- |
| Real product absent | **Unfixed:** F-1-1. |
| Concurrent recovery returned 500 | Fixed: clean backend test `eight_concurrent_recoveries_never_return_server_error` passes. |
| First clean claim run timed out | Fixed: cold clean-worktree account/payment claim passed in 9.8 s. |
| Claims inventory incomplete | **Unfixed:** F-1-2. |
| 200% mobile reflow / footer targets | Fixed: at 390 px and 200% root text, `scrollWidth === clientWidth === 390`; Privacy and Terms targets are 90×44 and 76×44 px. |
| Static cache headers | Fixed: live hashed JS and WOFF2 return `Cache-Control: public, max-age=31536000, immutable`. |
| Unknown route returned 200 | Fixed: `/not-a-real-place` returns HTTP 404. |
| Expected consent rejection logged as console error | Fixed by existing E2E coverage; no cold-load console errors observed. |
| Pinned Rust base | Fixed: `backend/Dockerfile` uses `rust:1-slim`. |
| Live 12-write limit did not hold | Fixed in prior live verification; the local claim test passes. |

## Copy audit

Counts use visible reader-facing units; labels and buttons are included so no
action wording is skipped. No landing item exceeds 22 words. “Flag” records
the copy findings above or a consistency issue, not a claim test result.

### Landing `/`

| Text | Words | Flag / proposed rewrite |
| --- | ---: | --- |
| Booking follow-up with proof | 4 | F-1-4; delete or “Recovery status and delivery receipts”. |
| Recover paid sessions before they disappear | 6 | Clear h1. |
| For solo coaches and tutors who need to see why a paid booking stopped and what can happen next. | 19 | Clear. |
| Try it with sample data | 5 | Result-naming action; clear. |
| Opens a safe workspace with three fictional clients. | 8 | Unlisted claim; add claim/test or remove “safe”. |
| No account needed | 3 | Covered by `demo-no-account-payment`. |
| No real messages sent | 4 | Covered only in part by network claim; register exact claim. |
| No payment in the demo | 5 | Covered by `demo-no-account-payment`. |
| One booking stopped. | 3 | Clear. |
| Consent decides the next step. | 5 | Useful but uses “consent” rather than “email consent”; use the latter. |
| The product | 2 | Clear label. |
| See the break in the booking loop | 7 | F-1-3; “Sample recovery board”. |
| Each ticket keeps the booking state, permission, and delivery evidence together. | 10 | Unlisted claim; test or remove. |
| Booking started | 2 | Clear status. |
| Service and time chosen | 4 | Clear status. |
| Recorded | 1 | Ambiguous alone; “Booking recorded”. |
| Deposit not finished | 3 | Clear status. |
| Email consent is on record. | 5 | Unlisted claim; use “Email permission recorded” consistently. |
| Needs a follow-up | 4 | Clear status. |
| Delivery receipt | 2 | Clear label. |
| Waiting for a permitted action | 5 | Clear enough. |
| Not started | 2 | Clear status. |
| How it works | 3 | Clear label. |
| Follow one accountable path | 4 | F-1-3; “How booking recovery works”. |
| Find the stopped booking | 4 | Clear imperative heading. |
| See the chosen session and where the client left. | 9 | Clear. |
| Check permission first | 3 | Prefer consistent “Check email consent first”. |
| A follow-up stays stopped when contact consent is missing. | 9 | Clear but terminology differs from email consent. |
| Keep the receipt | 3 | Clear imperative. |
| The sample action ends with a labelled delivery record. | 9 | Covered by `demo-recovery-receipt`. |
| Clear boundaries | 2 | Generic label; “What the product does not do”. |
| It does not replace your calendar | 6 | Clear. |
| Booking Recovery Loop focuses on the steps after someone chooses a paid session. | 12 | Unlisted scope claim. |
| It is not a CRM, a marketplace, or a tool for bulk messages. | 12 | Clear scope statement; test is not needed if retained as a non-capability statement. |
| Read how the sample handles data | 6 | Clear link. |
| Practice plan | 2 | Clear label. |
| Recovery Loop Practice | 3 | Product-plan name; context supplied. |
| $29 / month | 3 | Unlisted price claim; the plan cannot be bought. |
| For one practice with one to five practitioners. | 8 | Unlisted plan claim. |
| The paid plan is not open in M1. | 9 | Honest but confirms F-1-1. |
| Accounts and hosted checkout arrive in M2. | 7 | Honest but confirms F-1-1; remove milestone jargon from customer copy and implement the service. |
| Try the sample first | 4 | Result-naming action; clear. |

### README.md

All prose sentences/reader-facing bullets are listed below. Command lines and
section titles are not sentences. None exceeds 22 words after URLs and links
are counted as one token; several are nevertheless technical claims covered by
F-1-2.

| Location | Words | Sentence / bullet | Audit note |
| --- | ---: | --- | --- |
| 3–4 | 22 | Booking Recovery Loop helps solo tutors, coaches, and consultants recover a stopped paid booking without losing the consent record or delivery evidence. | Unlisted product claim; F-1-2. |
| 5 | 12 | M1 ships a public product page and a working, isolated recovery sandbox. | “M1” is internal jargon; unlisted claim. |
| 7–9 | 14 | Try the deployed sample at … It uses three fictional bookings, never sends a real message, and needs no account. | Split into two sentences; claims need entries. |
| 13 | 8 | A one-click demo of a consented abandoned-booking recovery. | Fragment; rewrite “The demo recovers a consented abandoned booking in one click.” |
| 14 | 10 | A server-enforced stop when the sample has no email consent. | Fragment; unlisted claim. |
| 15 | 9 | A simulated, timestamped delivery receipt from an in-process mailbox. | Fragment; unlisted claim. |
| 16 | 12 | Portable demo workspace tokens with 256 random bits and a 24-hour expiry. | Claim is listed. |
| 17 | 13 | A reset control that replaces the current browser workspace and restores the seed. | Fragment; unlisted claim. |
| 18 | 7 | Plain-language privacy, terms, and product-native not-found routes. | Fragment; “product-native” jargon. |
| 19–20 | 10 / 13 | A 12-write per-client allowance keyed from the first X-Forwarded-For hop. It restores one write each minute and returns a positive Retry-After when full. | First sentence listed; second has no separate claim; technical README detail. |
| 22–25 | 7 / 16 / 15 | The demo is not a production account. It does not call Entra, Sociobot billing, Dodo, Stripe, a messaging provider, or an AI service. CIAM, PostgreSQL customer tenancy, and the $29/month hosted subscription flow are M2 scope … | Second is partly listed; jargon and F-1-1 confirmation. |
| 29 | 11 | Vite 6 and strict TypeScript with semantic HTML and product-native CSS. | Fragment; technical inventory. |
| 30 | 11 | Rust 2021, axum, sqlx, and SQLite for temporary M1 demo workspaces. | Fragment; technical inventory. |
| 31 | 12 | A single non-root container serving both the API and built dist assets. | Fragment; deployment claim. |
| 33–35 | 15 / 16 | The shared production customer store moves to PostgreSQL in M2, before real practice data exists. The M1 container starts with only PORT and creates its demo database in the working directory. | F-1-1 confirmation; unlisted technical claims. |
| 39 | 8 | Requirements: Node 22+, npm, and current stable Rust. | Clear instruction. |
| 47–48 | 4 / 6 | Open http://127.0.0.1:8080. The service creates booking-recovery-loop.db locally. | Clear instruction; server-storage claim. |
| 50 | 7 | PORT — HTTP port, default 8080. | Clear reference item. |
| 51–52 | 7 | legacy database setting — SQLite URL, default sqlite://booking-recovery-loop.db. | Clear reference item. |
| 53–54 | 14 | STATIC_DIR — built web directory, default dist locally and /app/dist in the container. | Clear reference item. |
| 67–69 | 10 / 18 | The Playwright command builds and starts the complete service itself. It runs one browser test per claim … plus axe and keyboard checks for every public route. | First is a technical claim; second is inaccurate as a ledger statement because F-1-2 claims are not registered. |
| 71 | 5 | To build the production image: | Clear instruction lead-in. |
| 80–85 | 11 / 17 / 14 / 14 | Migration 0001… creates … . Migration 0002… retains … . Matching down migrations … . Each portable token contains no personal data; server replicas store only its SHA-256 hash. | Technical/storage claims; register or move out of visitor-facing README. |
| 87–89 | 15 | See demo.md for the sandbox boundary and design.md for the visual system and asset provenance. | Clear link sentence. |
| 93–98 | 11 / 10 / 19 / 13 / 7 | The factory deploys … . The Dockerfile … . M1's deployment contract … . M2 must move … . Do not add secrets … . | Internal deployment jargon; final instruction is clear. |
| 102–103 | 18 | MIT. Fraunces and Atkinson Hyperlegible Next use the SIL Open Font License; their license texts are in public/fonts/. | Clear legal information. |

## Structure, routing, accessibility, and visual checks

- `/`, `/demo`, `/privacy`, `/terms`, and `/404` return 200; an unknown URL
  returns a designed 404 with HTTP 404. `robots.txt` and `sitemap.xml` are
  present. Crawled internal links resolve.
- Titles, descriptions, canonicals, Open Graph/Twitter metadata, favicon, one
  `h1`, `main`, `lang=en`, header/footer, Privacy/Terms links, skip link,
  route-change focus, and aria-live announcements were present. Back navigation
  restored the previous route and moved focus to its h1.
- The product uses its distinct twilight appointment-rail identity rather than
  a generic gradient/card SaaS template. Self-hosted fonts and art load from
  this origin.
- The 390 px board has no horizontal overflow. The focused route and skip-link
  behaviour are correct. The visual spacing between the demo banner controls
  is tight in copied text (“Reset demoStart for real”), but they remain separate
  44 px controls and this is not a finding.
- AI is not a necessary addition to this brief; a decorative AI feature would
  not improve the core flow. The obvious missing leverage is the real public
  booking/deposit/recovery/export flow already covered by F-1-1.

## What would make this perfect

Implement F-1-1 completely, then prune or register every visitor-reliant claim
in F-1-2. Rename the two metaphor headings and remove the unexplained eyebrow.
After a clean, real-practice end-to-end run confirms payment, consent,
delivery/fallback, evidence, export/delete, data isolation, and subscription
behaviour, repeat this full cold review. Only then can the product be PASS.
