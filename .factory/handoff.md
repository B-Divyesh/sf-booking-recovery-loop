# Foundation handoff — Booking Recovery Loop

**Work order:** `venture-booking-recovery-loop-plan`

**Date:** 2026-08-28

**Status:** planning and working skeleton complete; M1 product behavior has not started.

## What was done

- Wrote the executable venture contract in `.factory/plan.md`: PRD, sourced
  evidence, Rust/axum + Vite architecture, tenancy/security boundaries,
  Sociobot Entra CIAM and Dodo subscription plan, Stripe-hosted session-payment
  boundary, data model, jobs, operations, design requirements, M1–M6 scope,
  claim/test/DoD lists, and risk-retirement experiments.
- Wrote the twilight appointment carousel visual thesis in
  `.factory/design.md`, including palette, system type plan, spacing, shape and
  motion rules, component/state requirements, responsive/accessibility rules,
  original-art provenance policy, and the stack decision.
- Added M1’s five planned, browser-verifiable claims to `.factory/claims.json`
  and documented the exact isolated-demo contract in `.factory/demo.md`.
- Added a Vite/strict-TypeScript application shell with route metadata,
  History API navigation, title/canonical updates, focus restoration, skip
  link, mobile/reduced-motion styling, design tokens, and a component inventory.
  It intentionally shows only foundation placeholders, not a mocked product.
- Added a minimal Rust/axum health service, structured logs, graceful shutdown,
  `PORT` default, Dockerfile, local Postgres compose scaffold, and backend test.
- Added GitHub Actions for web tests/build artifact and Rust format/test/release
  build; added deployment metadata, CSP/security headers, favicon, robots,
  sitemap, fonts placeholder/license instructions, and designed 404 shell.
- Updated README and retained the existing MIT license.

## Verification performed

All commands ran successfully in `/work/repo`:

```text
npm audit --json                         # 0 vulnerabilities
npm test                                 # 1 file, 4 tests passed
npm run build                            # dist/ created; 2.30 kB gzip JS
npm run check:backend                    # rustfmt + 1 Rust test passed
npm run build:backend                    # release build passed
```

The Rust release build was rerun once after the initial long compile and exited
successfully. The working tree’s ignored `graphify-out/` cache is environment
generated and is not part of this handoff or commit.

## Known gaps (intentional)

- No M1 public landing, isolated demo workspace, booking attempt, recovery
  state machine, real policy pages, or Playwright claim tests exists yet. The
  current `/demo`, `/privacy`, and `/terms` pages accurately say they are
  placeholders.
- No database migrations, authentication, billing, Stripe integration,
  messaging provider, persistence, rate-limited product API, AI feature, or
  external call exists yet. The API has only an exempt `/health` endpoint.
- `npm run test:e2e` is intentionally not a passing gate yet because M1 must
  implement exactly one tagged Playwright test for each claim. CI correctly
  gates the executable foundation tests and builds now.
- Font assets and the hand-made appointment-rail SVG are intentionally deferred
  to M1; no external font or art request is made by the scaffold.

## Next builder: M1

Read `.factory/plan.md`, `.factory/design.md`, `.factory/claims.json`, and this
handoff before editing. Build only M1: public landing, isolated 24-hour demo
tenant/token boundary, realistic seed, reset flow, consent-gated simulated
recovery receipt, public policy pages, and the five tagged claim tests. Replace
the placeholder text honestly. Add the required font subsets and original art
with provenance. Run every M1 quality gate, write `.factory/copy-audit.md` and
`.factory/handoff-m1.md`, update plan status, then obtain review/polish PASS
before M2.

## Future operator action

Before M2 production acceptance, register
`https://booking-recovery-loop.sociobot.in/auth/callback` on the shared
Sociobot Entra CIAM SPA application and register the `$29/month` Recovery Loop
Practice subscription in the Sociobot product/billing registry. No operator
action is needed for this foundation commit.
