# Perfection loop — polish 2

**Source candidate:** `7de273d65e1e9f34354d03ee9070a6a4fc4793be`  
**Repair commit:** `7e7194b0f1a0d4f0585e55fadf324bbe2ba903b0`

## Finding map

| Finding | Change made | Evidence |
| --- | --- | --- |
| `F-1-1` | Kept the real practice, public booking, encrypted data, export/delete, authenticated payment event, receipt, and bounce fallback path. Added durable abandoned-booking recovery and paid-session reminder jobs. | Rust `automatic_recovery_is_durable_consent_gated_and_idempotent`; `hosted_payment_requires_verified_callback_and_is_idempotent`; live `/health` and `/app` check after deployment. |
| `F-1-2` / `F-2-3` | Expanded `claims.json` to 19 narrow, executable claims. Rewrote README so product promises point to named evidence and removed the previous untestable deployment/security assertions. | Clean-clone `npm ci`, all claims commands, unit, backend, build, deployment-boundary, size, and browser suite. |
| `F-1-3` | Retained the plain section titles “Sample recovery board” and “How booking recovery works.” | Local home screenshot: `.factory/evidence/polish-2-local/home.png`. |
| `F-1-4` | Retained the removed unexplained eyebrow and consistent “delivery receipt” wording. | Local home screenshot and `copy-audit.md`. |
| `F-2-1` | New SQLite `practice_scheduled_jobs` migration persists recovery/reminder jobs. The service worker loop claims due jobs atomically, retries provider failures, stops withdrawn-consent work, and exposes queued/sent/stopped state in the owner queue. | `automatic_recovery_is_durable_consent_gated_and_idempotent`; `hosted_payment_requires_verified_callback_and_is_idempotent`. |
| `F-2-2` | Replaced the fake `example.com` setup value with required explicit connection fields. Added the exact $29/month plan price. The Sociobot product registry currently returns 404, so checkout is explicitly unavailable instead of a dead link. | `@claim:practice-plan-price`; `curl https://api.sociobot.in/api/v1/products/booking-recovery-loop/checkout` returned the registry’s 404 before deploy. |
| `F-2-4` | Standardized visible product language on **email consent** and **SMS consent**; removed “contact consent,” “channel consent,” and “Email permission.” | `@claim:booking-consent-record`; local screenshots; `copy-audit.md`. |
| Earlier concurrent-write, rate-limit, 200% reflow, footer target, cache, 404, console, Rust-image, routing, title, legal-link, metadata, and demo-isolation findings | Preserved the existing repairs and reran their coverage. | `npm run test:e2e`, `npm run check:backend`, `npm run test:deployment`, `npm run check:size`; clean-clone browser result. |

## Screens and live evidence

- Desktop home: `.factory/evidence/polish-2-local/home.png`
- 390 px demo: `.factory/evidence/polish-2-local/demo-mobile.png`
- Live URL: `https://booking-recovery-loop.sociobot.in` — `/health` reports `7e7194b0f1a0d4f0585e55fadf324bbe2ba903b0`; cold verifier passed.

## External boundary

The source cannot register a Sociobot billing product or operate a practice’s
payment/delivery provider. The UI now states that billing checkout is unavailable
rather than making a false or dead purchase promise. The durable delivery path
works against a configured HTTPS delivery connection and records its result.
