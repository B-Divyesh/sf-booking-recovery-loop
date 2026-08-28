# Demo contract (planned for M1)

This repository currently contains a planning and engineering foundation only.
The customer-facing demo described below is not implemented until M1.

## Required M1 behavior

- **Entry:** `/demo` and `/?demo=1` create or enter an isolated demo workspace
  without an account.
- **Seed:** a realistic coach practice, one 45-minute service, three booking
  attempts (one consented unfinished attempt, one missing-consent attempt, and
  one completed booking), plus sample delivery events.
- **Storage:** browser state uses the `demo:` namespace. Server records use a
  random, unguessable demo workspace token, have `is_demo=true`, and expire
  after 24 hours. A demo token must not read or write a real practice.
- **Safety:** demo payment and delivery outcomes are in-process fakes. No
  Stripe, messaging, Entra, Dodo/Sociobot billing, or AI request is permitted.
- **Reset:** the persistent banner’s Reset demo control replaces the demo token
  and restores the original seed. Leaving demo discards browser demo state.
- **Claims:** the acceptance tests are the five entries in `claims.json`.

The M1 handoff replaces this planned-contract note with the implemented API
routes, seed IDs (non-sensitive), reset evidence, expiry cleanup evidence, and
the browser storage keys.
