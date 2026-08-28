# Independent product verification — FAIL

**Candidate:** `4bc479de1f4d464bfd071bd3b2f0a4bb7c659aa5`
**Live URL:** https://booking-recovery-loop.sociobot.in
**Verified:** 2026-08-28 UTC
**Work order:** `booking-recovery-loop-verify-4`

## Verdict

**FAIL — do not release as the researched Booking Recovery Loop product.**

The previous deployment-only rate-limit failure is repaired. This candidate is live, matches its source exactly, is accessible, private in demo mode, and all declared claims pass. It nevertheless ships only a fictional M1 sandbox. It does not perform the real job-to-be-done in the researched brief.

## Mandatory first-read gate — PASS

A fresh desktop browser visit to `/` displayed:

- **What:** “Recover paid sessions before they disappear.”
- **For whom:** “For solo coaches and tutors who need to see why a paid booking stopped and what can happen next.”
- **First action:** visible **Try it with sample data**, with “Opens a safe workspace with three fictional clients.”

The action opens `/demo` in one click. It has three fictional bookings and a persistent demo banner with reset/start-for-real controls. Thus this gate does not cause the failure.

## Release-blocking finding

### Critical — the real paid-booking recovery product is absent

The brief's smallest useful product requires a branded paid-session page, Stripe-hosted deposit collection, consent-aware automatic email/SMS fallback for abandoned or bounced reminders, delivery evidence, encrypted real contact data, deletion/export, and the `$29/month` practice subscription.

Fresh live evidence shows the opposite for every real workflow:

- The landing page states: “The paid plan is not open in M1. Accounts and hosted checkout arrive in M2.”
- **Start for real** only removes the `demo:workspace-token`; it does not open onboarding or a real practice workspace.
- The only recovery calls `/api/v1/demo/*`; Maya's receipt is visibly labelled “Delivered · simulated email,” and the UI says “No email leaves this site.”
- There is no account/CIAM login, real booking/session page, hosted checkout, Stripe deposit, real message provider, reminder/bounce fallback, persistence for a real practice, encryption boundary for client contacts, export, or deletion flow.

This is a direct failure of the brief and repository definition of done, not a deployment problem or a missing optional feature.

## Claims gate — PASS

`npm ci` completed from the clean candidate (62 packages; 0 reported npm vulnerabilities). Every command listed in `.factory/claims.json` was run individually and passed:

| Claim | Exact command | Evidence |
| --- | --- | --- |
| `demo-isolated` | `cargo test --manifest-path backend/Cargo.toml demo_never_reads_or_mutates_real_practice_fixture` | 1 passed |
| `demo-lifetime` | `cargo test --manifest-path backend/Cargo.toml portable_token_has_256_random_bits_and_24_hour_expiry` | 1 passed |
| `forwarded-rate-limit` | `cargo test --manifest-path backend/Cargo.toml write_limit_uses_forwarded_ip_and_returns_retry_after` | 1 passed |
| `demo-no-account-payment` | `npm run test:e2e -- --grep @claim:demo-no-account-payment` | 1 passed |
| `demo-reset` | `npm run test:e2e -- --grep @claim:demo-reset` | 1 passed |
| `consent-gates-recovery` | `npm run test:e2e -- --grep @claim:consent-gates-recovery` | 1 passed |
| `demo-recovery-receipt` | `npm run test:e2e -- --grep @claim:demo-recovery-receipt` | 1 passed |
| `demo-no-external-requests` | `npm run test:e2e -- --grep @claim:demo-no-external-requests` | 1 passed |

## Fresh live verification — PASS for the implemented demo

- `/health` returned `200` and `{"status":"ok","build_sha":"4bc479de1f4d464bfd071bd3b2f0a4bb7c659aa5"}`.
- A build with `VITE_BUILD_SHA=4bc479de1f4d464bfd071bd3b2f0a4bb7c659aa5` exactly matched the deployment: both `index.html` hashes were `f69c4e3264c989165ec3b86d47e8c804b79ca9f2234c95c53916f0092264e9e3`; both application JS hashes were `1d9d06d86e142f5ffa2c8a193ec82b61beaa813e9adf39cf9767a4c956273ef7`.
- `/`, `/demo`, `/privacy`, `/terms`, `/robots.txt`, and `/sitemap.xml` return 200; an unknown route returns 404.
- On desktop and 390px mobile: Maya's permitted action showed a timestamped simulated receipt; Jordan's missing email consent stopped recovery; reset rotated the demo token and restored the seed. No horizontal overflow or console/page errors occurred.
- Keyboard testing reached the skip link first; Enter moved focus to `main`. Focus was visibly styled. The live app honours reduced motion. Axe reported zero serious/critical violations on desktop and mobile. The factory `verify-url.sh` reported title, `lang=en`, exactly one h1, a main landmark, zero missing image alternatives, zero unnamed buttons, and no console errors.
- The complete fresh demo request log (load, recovery, consent stop, reset) contained only `https://booking-recovery-loop.sociobot.in`. No payment, messaging, Entra, billing, analytics, AI, or font-CDN request occurred. The demo's browser storage key was `demo:workspace-token`.
- Responses include `nosniff`, strict-origin referrer policy, `DENY` framing, restrictive permissions policy, and CSP with `frame-ancestors 'none'`. Hashed JavaScript is `public, max-age=31536000, immutable`.
- Live `POST /api/v1/demo/workspaces`, from one new forwarded client identity, returned 201 twelve times (`X-RateLimit-Limit: 12`, remaining 11 through 0), then 429 on requests 13 and 14 with `Retry-After: 60`. This repairs the earlier deployment-only finding and meets the documented allowance.

## Local quality checks — PASS

| Check | Result |
| --- | --- |
| `npm test` | 2 files, 9 tests passed |
| `npm run check:backend` | rustfmt and 9 Rust tests passed |
| `npm run test:deployment` | passed: one ingress-routed replica on port 8080 |
| `npm run test:e2e -- --workers=4` | complete 17-test browser suite ran; all claim, accessibility, keyboard, responsive, cache, security, and offline-state cases passed (the final two were also rerun directly: 2 passed) |
| `npm run build` | passed; `dist/` produced |
| `npm run check:size` | JS 8,392 bytes gzip; CSS 19,123 bytes raw |
| `npm run build:backend` | passed, optimized release profile |
| runtime with only `PORT=4191` | passed: generated default DB, `/health` and `/` both 200 |

Fresh live Lighthouse: Performance 100, Accessibility 100, Best Practices 100, SEO 100; FCP 1.2 s, LCP 1.5 s, CLS 0, TBT 0 ms.

No Docker, Podman, or Buildah executable is installed in this worker, so the exact container-image build could not be executed. The normal production web and optimized backend builds did pass; the Dockerfile remains an unverified environment limitation.

This is a web-with-backend product, not a library, CLI, or PWA. Clean-consumer package checks and service-worker update/offline reload are therefore not applicable. There is no sign-in to test against CIAM; the lack of a real account flow is included in the critical scope finding.

## Required next step

Implement the brief's real end-to-end practice workflow before release: Sociobot Entra account/tenant isolation, branded paid session page, Stripe-hosted deposit through the permitted architecture, consent-aware real email/SMS delivery and bounce fallback with receipts, encrypted real contact data, export/delete, and the $29/month subscription. Preserve the isolated one-click demo and add claim tests for every new visitor-facing promise.
