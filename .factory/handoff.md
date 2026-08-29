# Verification 6 handoff — FAIL

**Work order:** `booking-recovery-loop-verify-6`
**Candidate:** `649a5e7efd92d84aae17290332337b7e5eebb096`
**Live URL:** https://booking-recovery-loop.sociobot.in
**Decision:** **FAIL — do not release**

## Release blockers

- The live revision has three replicas and only `PORT`. A newly created
  practice returned `200` on 29 of 90 independent reads and `404` on 61.
  Production is still split across local SQLite stores despite the candidate's
  shared-PostgreSQL source contract.
- The only production delivery option is nonfunctional. A fresh consented
  booking's Resend connection test and manual recovery both returned
  `502 delivery_rejected`; automatic recovery, reminders, and SMS fallback
  therefore cannot complete.
- The $29/month Sociobot checkout returns 404. Session deposits use a static
  practice URL and generic callback token, not per-booking Stripe Checkout and
  signed Stripe events.
- Rate limiting is multiplied by the three stores. One client completed 36
  immediate demo writes for the documented 12-write allowance, and 126 API
  reads for the advertised 40-request burst. Limited responses did include
  `Retry-After` (`60` for writes, `1` for general API calls).
- The startup-grade identity boundary is absent: no Sociobot Entra/MSAL/JWT
  flow, `/auth/callback` is 404, and practice access is a browser-stored owner
  key.

## Verification that passed

- All 24 `.factory/claims.json` commands passed locally, but the deployed
  storage, rate-limit, and delivery claims are contradicted by live evidence.
- Cold first read and one-click sample gate passed.
- `npm ci`, 10 frontend tests, rustfmt and 22 backend tests, the deployment
  contract check, 27 browser tests, strict production build, bundle check,
  optimized Rust build, and Clippy all passed.
- The release binary starts with only `PORT`; health, static serving, and
  graceful shutdown passed.
- Live `/health` reports the candidate SHA. Candidate `index.html`, JS, and CSS
  match live byte-for-byte.
- Desktop/390 px axe found zero serious/critical issues. Keyboard, visible
  focus, reduced motion, same-origin demo privacy, response headers, cache
  policy, 404 handling, and normal/invalid input checks passed.
- Mobile Lighthouse scored 100 in all four categories; LCP was 1.59 s, CLS 0,
  TBT 0 ms, and transfer 138,941 B.

The remaining accessibility defects are sub-44 px select/inline-link targets
and poor line wrapping in the demo at 200% text size.

## How to reproduce

```sh
npm ci
npm test
npm run check:backend
npm run test:deployment
npm run test:e2e
VITE_BUILD_SHA=649a5e7efd92d84aae17290332337b7e5eebb096 npm run build
npm run check:size
npm run build:backend
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
```

Run every exact command in `.factory/claims.json` individually before the
broader suite. For the live deployment, use independent HTTP connections when
testing persistence and limits; a reused connection remains pinned to one
replica and hides the failure.

Full report and evidence:
[verification-6.md](verification-6.md) and `verification-evidence-6/`.

No product code was modified during verification.
