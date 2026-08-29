# Verification 9 handoff — FAIL

**Candidate:** `06c4b50fbc1f5b3eaae13b38bf2f11789e8d7d07`

**Live URL:** <https://booking-recovery-loop.sociobot.in>

**Report:** [verification-9.md](verification-9.md)

**Decision:** **FAIL — do not release.**

## Blocking evidence

1. Production reports both required real-job integrations unavailable:
   `billing.configured: false` for the dedicated booking-deposit product and
   `delivery.configured: false`. The exact deposit checkout endpoint returns
   404, so a real paid booking/recovery/reminder/fallback loop cannot complete.
2. Separate-connection production probes show the promised shared deployment
   is still replica-local. Forty writes produced 36×201 and 4×429 instead of
   12×201 and 28×429. One hundred sixty reads produced 120×200 and 40×429
   instead of 40×200 and 120×429.
3. One demo token resolved to three workspace IDs. After reset, 24 of 36 reads
   using the old token still returned 200 from two replicas. This falsifies the
   live `demo-reset` privacy claim.
4. The literal pre-install claims run recorded 17 passing Rust commands and
   nine Playwright startup failures because `@playwright/test` was not yet
   installed. After `npm ci`, all 26 exact claim commands passed locally.

The repository's supplied live script reported the write/reset checks passing,
but it reused one pooled request context and stayed sticky to one replica. The
independent verifier repeated the probes with one context/connection per
request, exposing the cross-replica failures. See
[`verification-9-evidence/topology-probes.json`](verification-9-evidence/topology-probes.json).

## What passed

- Cold first-read and visible one-click sample gate.
- Candidate identity: `/health`, footer, and byte-identical HTML/JS/CSS all
  match `06c4b50f…`.
- `npm ci`, 10 Vitest tests, 29 Rust tests plus rustfmt, strict Clippy,
  deployment-contract check, 28 Playwright tests, exact frontend build, bundle
  check, and locked optimized backend build.
- Local runtime with only `PORT`, plus health/static serving and graceful
  shutdown.
- Live sample recovery, consent rejection, receipt, visible reset, route/link
  checks, input boundaries, protected-route 401, and disabled production test
  identity.
- Sociobot Entra authority, tenant, client, redirect, PKCE S256, and scopes.
- Desktop, 390px, 200% text, keyboard/focus, reduced motion, zero serious or
  critical axe findings, and no valid-route console/page errors.
- Same-origin demo traffic, security headers, compression, immutable asset
  caching, HTTPS redirect, and legal/metadata routes.
- Fresh mobile Lighthouse: 100/100/100/100; LCP 1.663 s, CLS 0, TBT 0.
- Bundle: 79,446-byte gzip JS; 22,252-byte CSS; self-hosted fonts.

Docker tooling was unavailable, so the image was inspected but not executed.
No login credential was available, and the absent production integrations made
a real owner-to-client paid booking flow impossible to complete.

## Next steps

Provision and verify the dedicated client-deposit product and credentialed
delivery relay. Then correct the deployed database/runtime topology and repeat
write, read, and reset tests across independent connections. Do not rely on a
single sticky browser request context as evidence of multi-replica behavior.

No product code was modified. Verification report and evidence only were added.
