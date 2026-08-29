# Demo sandbox

## Entry and seed

- URL: `https://booking-recovery-loop.sociobot.in/demo`
- Alternate entry: `https://booking-recovery-loop.sociobot.in/?demo=1`
- Practice: North Star Coaching, timezone `Europe/London`.
- Service: 45-minute focus session with a fictional £35 deposit.
- Maya Patel: unfinished attempt with recorded email consent.
- Jordan Lee: unfinished attempt without email consent.
- Alex Morgan: completed booking with a seeded simulated receipt.

All names and records are fictional. No account or manual setup is required.

## Isolation and storage

`POST /api/v1/demo/workspaces` creates a UUIDv7 workspace and returns a
portable demo token. It contains 256 random bits, its issue time, and a
`fresh` sample-state marker. It contains no customer or contact data. The
browser stores it under `demo:workspace-token`; the shared production database
stores only its SHA-256 hash.

An existing workspace is selected with all three conditions:

1. the token hash matches;
2. `is_demo = 1`;
3. `expires_at` is still in the future.

The API test inserts a non-demo practice fixture and proves a demo request
cannot read it. Expired demo workspaces are rejected immediately and purged by
the service's 30-second worker, so cleanup never delays the first sample
response.

Factory ingress may send requests to different container replicas. Production
uses the shared PostgreSQL store declared in `deploy/containerapp.m1.json`, so
each replica resolves the same token hash and workspace. A reset expires that
workspace in the shared store; every later old-token read returns `404`.
Portable-token hydration exists only for a local, intentionally isolated demo
database and can recreate only the fixed fictional seed. The token never
identifies or authorizes a real workspace. The exact cross-replica reset and
rate-limit regressions run against four routers sharing one durable store.

The demo uses dedicated SQLite tables and routes. Real practice records use
separate tables, owner tokens, and encrypted contact fields. Migration
`0001_demo_workspaces.up.sql` is tracked and safe to rerun. Its matching down
migration removes the demo schema.

## Recovery flow

1. Select Maya Patel.
2. Review the exact recorded email wording and timestamp.
3. Select **Run sample follow-up**.
4. The server checks consent, writes one sample message, and writes a simulated
   delivered event.
5. The browser shows its timestamp and clearly labels it as simulated email.

Selecting Jordan Lee and choosing **Check recovery permission** stops in the
browser before a request is sent. The server independently returns
`409 consent_required` if a caller bypasses the interface. No outbound message
or receipt is created.

## Reset and expiry

**Reset demo** creates a new token and fresh seed, replaces the browser token,
and expires the prior workspace in the shared store. **Start for real** removes
the browser token. Inaccessible server copies expire after 24 hours.

The API routes are limited by client IP. The first `X-Forwarded-For` hop is
used behind factory ingress. Write routes allow 12 immediate requests per
client, restore one request every 60 seconds, and then return `429` with a
whole-second `Retry-After` value of at least 1.

## External-service boundary

Demo payment and delivery results are in-process simulations. The complete
Playwright flow records every URL and proves all requests stay same-origin.
The demo never contacts Stripe, a messaging provider, Entra, Sociobot billing,
Dodo, or the Sociobot AI gateway.

## Clean verification

```sh
npm ci
npm run test:e2e
```

Each entry in `.factory/claims.json` names one exact browser or server test.
The suite proves expiry, reset, non-demo isolation, consent enforcement,
migration reversal, and rate-limit headers.
