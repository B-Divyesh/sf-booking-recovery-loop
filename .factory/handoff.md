# Verification handoff — FAIL

**Candidate:** `d03d83db200435a8582ea5fac676139abfb139cb`

**URL:** `https://booking-recovery-loop.sociobot.in`

**Independent result:** **FAIL — not releasable against the supplied product contract.**

Fresh verification confirms that production is online and matches the exact
candidate SHA. The first-read/demo gate passes, all warm automated suites pass,
the normal demo flow works on desktop and 390px mobile, privacy requests stay
same-origin, rate limiting returns 429 with `Retry-After`, and mobile Lighthouse
scores 100 in all four tested categories.

Release blockers:

1. The candidate is only an M1 fictional demo. It has none of the real account,
   paid booking, deposit, reminder/fallback, delivery, subscription, or customer
   data-rights workflow required by the brief.
2. Concurrent valid recoveries return HTTP 500: 5/8 failed live and 7/8 failed
   against one fresh local candidate process.
3. The first exact claim command failed from the clean environment because the
   120-second Playwright server timeout elapsed during the cold Rust build. It
   passed after compilation, as did all other claim tests.
4. Material landing/privacy/README claims are absent from `claims.json`, and
   the `demo-isolated` tagged browser test does not perform the fixture
   read/mutation check its declared sandbox promises.

Additional defects: 200% text causes 649px document width at a 390px viewport;
footer Privacy/Terms targets are under 44px tall; hashed assets and fonts have
no cache policy; unknown routes render the 404 UI with HTTP 200; the handled
409 consent path creates a Chromium console error; and the Dockerfile pins
`rust:1.98-slim-bookworm` contrary to the supplied major-only base-image rule.

Full commands, evidence, headers, hashes, claim-by-claim outcomes, and severity
details are in [.factory/verification.md](verification.md). Key artifacts are
in `.factory/verification-artifacts/`.

No product code was modified. Verification added only this handoff, the report,
and evidence artifacts.
