# M1 repair handoff — release blockers repaired

- **Work order:** `booking-recovery-loop-repair-2`
- **Verifier report:** `337ffd093c05bd6929ccf26ebbbe906f27838c59`
- **Rejected candidate:** `b6ca2c781ddd603ff08c582b66f4b1970df783d4`
- **Repair source:** `361c10a8f0070bf19338d4b84b65090561d3a487`
- **Live URL:** `https://booking-recovery-loop.sociobot.in`
- **Live revision:** `sf-booking-recovery-loop--0000006` (100% traffic)
- **Verified:** 2026-08-28 UTC

## Outcome

The controller identifies this as M1 QA, so the verifier's final-product scope
critique is not an M1 defect. No planned M2–M6 account, billing, real booking,
provider messaging, or customer persistence work was added.

The remaining release blocker is repaired. The live service now enforces the
published 12-write allowance for one first-hop `X-Forwarded-For` client across
the whole M1 deployment. Request 13 returns `429`, `X-RateLimit-Limit: 12`, and
a positive `Retry-After`. The passing M1 sample, isolation, consent, privacy,
responsive, accessibility, and performance behavior remains intact.

## Failure reproduced before repair

Against the rejected live candidate, a fresh fixed forwarded identity received
16 successful workspace writes before its first `429`; the rejection reported
`Retry-After: 0`. The independent verifier observed 36 successful writes before
the first rejection when three replicas were active. Rejection responses also
advertised the outer read allowance (`X-RateLimit-Limit: 40`) instead of the
write allowance.

Root causes:

- The generic deployment allowed three replicas, but each replica owned a
  separate in-memory bucket and local SQLite store.
- The 200 ms replenishment interval refilled during a sequential probe.
- `tower-governor` rounded a sub-second wait down to zero.
- The outer read limiter overwrote write-policy response headers.

## Repairs

- M1 now declares one ingress-routed replica in
  `deploy/containerapp.m1.json`. This makes the process-local M1 limit and
  SQLite sandbox service-wide. M2 must introduce shared infrastructure before
  scale-out.
- Write routes allow 12 immediate requests per first forwarded IP and restore
  one allowance every 60 seconds.
- The custom rejection response rounds the delay up and guarantees
  `Retry-After >= 1`; it reports write limit `12` and remaining `0`.
- Read and write limiters are on separate route branches, so policy headers do
  not overwrite one another.
- The claim regression sends 13 requests from one forwarded identity, checks
  all 12 successful allowance headers, checks the 13th rejection and matching
  positive retry headers, and proves another first-hop identity remains
  allowed.
- Browser tests now give each isolated test context a distinct documentation IP
  so the complete suite tests separate clients instead of exhausting one
  localhost bucket.

## Clean local verification

All commands passed from a clean `npm ci` (62 packages, 0 vulnerabilities):

```text
npm test                    2 files, 9 tests
npm run check:backend       rustfmt + 9 Rust tests
npm run test:deployment     M1 min/max replicas both 1
npm run test:e2e            17 Chromium tests
npm run build               dist/ produced
npm run check:size          JS 8,392 B gzip; CSS 19,123 B raw
npm run build:backend       optimized Rust release build
```

Every exact command in `.factory/claims.json` passed individually. A runtime
started with only `PORT=4191`; it logged a generated default database and served
`/health` and `/`. A real HTTP probe returned 201 for writes 1–12 and 429 for
write 13 with `Retry-After: 60` and limit 12.

Local desktop and 390 px demo runs had no console errors, no horizontal
overflow, and zero serious/critical axe findings. The factory URL verifier
passed title, language, one h1/main, image alternatives, control names, and
console checks. Lighthouse scored Performance 100, Accessibility 100, Best
Practices 100, and SEO 100; FCP was 1.1 s, LCP 1.7 s, CLS 0, and TBT 0 ms.
Evidence is in `.factory/repair-evidence/repair-2-local/`.

## Live verification

- `/health` returned build SHA
  `361c10a8f0070bf19338d4b84b65090561d3a487`.
- Azure reports image tag `sf-booking-recovery-loop:361c10a8f007`, min/max
  replicas `1/1`, and 100% traffic on revision `0000006`.
- From one fresh `X-Forwarded-For` identity, writes 1–12 returned 201. Writes
  13–20 all returned 429 with limit 12, remaining 0, and `Retry-After: 60`.
- `/`, `/demo`, `/privacy`, `/terms`, `/robots.txt`, and `/sitemap.xml` return
  200; an unknown route returns 404. Hashed JavaScript is immutable for one year.
- Desktop and 390 px live demos had no console errors, no overflow, zero
  serious/critical axe findings, and only same-origin requests.
- Keyboard Tab exposed and activated the skip link, focus moved to `main`, and
  the full sample recovery completed from the keyboard. Reduced motion matched.
- At 390 px with 200% text, document width remained 390 px. Storage contained
  only `demo:workspace-token`; there was no service worker.

Live evidence is in `.factory/repair-evidence/repair-2-live/`.

## Applicability and remaining work

This container web product is not a package, CLI, or PWA, so clean-consumer
package checks and service-worker update/offline reload checks do not apply. Its
explicit offline error state and absence of a service worker were verified.
M1 has no sign-in, payment, real provider, analytics, or AI integration, so no
live CIAM, billing, provider, or model identity exists to test.

There are no known M1 release blockers. M2–M6 remain planned in
`.factory/plan.md` and were deliberately not implemented in this repair.
