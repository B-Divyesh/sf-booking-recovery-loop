# Verification handoff — FAIL

**Candidate:** `4bc479de1f4d464bfd071bd3b2f0a4bb7c659aa5`
**URL:** https://booking-recovery-loop.sociobot.in
**Result:** **FAIL — do not release**

The deployment-only problem reported for the preceding candidate is fixed: fresh live evidence shows the configured 12-write forwarded-IP allowance and 429 responses with `Retry-After: 60`. The live health build SHA is the tested candidate, and a SHA-for-SHA frontend rebuild matches its deployed HTML and JavaScript.

All eight `.factory/claims.json` commands passed individually. Local unit, Rust, browser, build, bundle, runtime, accessibility, privacy, mobile, keyboard, security-header, cache, and Lighthouse checks passed. Details and exact commands are in [verification-4.md](verification-4.md).

The candidate still fails the researched product contract at **Critical** severity. It is only an isolated fictional M1 demo: it has no real practice account, branded paid-session page, deposit collection, real consent-aware recovery/reminder delivery, delivery/bounce evidence, encrypted client-data workflow, export/delete, or payable $29/month plan. The page explicitly says the paid plan, accounts, and hosted checkout are deferred to M2. A user cannot use it for the brief's real paid-booking recovery job.

The next builder must implement that real workflow while retaining the one-click isolated demo, then rerun independent verification. Docker image building was not executed because this worker has no Docker/Podman/Buildah binary; all other listed checks are documented in the report.
