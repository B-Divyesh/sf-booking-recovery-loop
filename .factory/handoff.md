# Current handoff — Booking Recovery Loop M1

M1 is built and locally verified. The product now has a public landing page,
an isolated server-backed demo, consent-gated sample recovery, simulated
delivery receipts, complete policy/error routes, and claim-level tests.

Run the complete gate:

```sh
npm ci
npm test
npm run check:backend
npm run test:e2e
npm run build
npm run check:size
```

Detailed evidence, migration notes, scope decisions, deployment configuration,
known boundaries, and the M2 brief are in
[.factory/handoff-m1.md](handoff-m1.md).

## Needs operator action for M2

- Register `https://booking-recovery-loop.sociobot.in/auth/callback` on the
  shared Sociobot Entra CIAM SPA application.
- Register the `$29/month` Recovery Loop Practice subscription in the Sociobot
  billing catalog and provide the verified event contract.
- Provision the production PostgreSQL connection and backup policy before any
  real practice data is accepted.
