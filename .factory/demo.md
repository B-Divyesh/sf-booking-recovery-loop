# Demo sandbox — M1 implementation

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
browser stores it under `demo:workspace-token`; each database replica stores
only its SHA-256 hash.

An existing workspace is selected with all three conditions:

1. the token hash matches;
2. `is_demo = 1`;
3. `expires_at` is still in the future.

The API test inserts a non-demo practice fixture and proves a demo request
cannot read it. Expired demo workspaces are rejected and purged when a new
workspace is created.

Factory ingress may send two requests to different container replicas. A
valid, unexpired portable token can therefore recreate only the fixed
fictional seed on a new replica. A successful sample recovery rotates its
marker to `recovered`, so a later replica recreates the simulated receipt too.
The token never identifies or authorizes a real workspace. A three-database
API test covers create, recover, and reload across separate replicas.

The M1 database is SQLite because the deployed container receives no database
configuration and stores no real customer data. Migration
`0001_demo_workspaces.up.sql` is tracked and safe to rerun. Its matching down
migration removes the schema. M2 introduces the planned shared PostgreSQL
customer store before production practice data exists.

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
and expires the prior workspace on the serving replica. **Start for real**
removes the browser token. Inaccessible server copies expire after 24 hours.

The API routes are limited by client IP. The first `X-Forwarded-For` hop is
used behind factory ingress. Write routes allow a burst of 12 and return `429`
with `Retry-After` after that allowance.

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
