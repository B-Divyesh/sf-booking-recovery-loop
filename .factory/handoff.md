# Review 1 handoff — FAIL

**Work order:** `booking-recovery-loop-review-1`
**Reviewed live build:** `4bc479de1f4d464bfd071bd3b2f0a4bb7c659aa5`
**Result:** **FAIL — do not release**

No product code was modified. The full adversarial review is in
[`review-1.md`](review-1.md).

What was verified:

- Cold live `/` at 390 px and desktop: first-read sample path is clear,
  responsive, same-origin, and console-clean.
- One-click `/demo`: realistic seeded board, persistent safety banner,
  simulated receipt, reset/token rotation, and same-origin request log all
  work.
- All eight exact `.factory/claims.json` commands passed from a detached clean
  worktree after `npm ci`. `npm test`, `npm run build`, and
  `npm run check:backend` also passed there.
- Rechecked earlier findings: concurrency, clean claim startup, mobile 200%
  reflow, footer targets, static cache headers, unknown-route status, Docker
  Rust tag, and deployment rate limiting are fixed.

Remaining release blockers:

1. `F-1-1`: The deployed product is only a fictional M1 demo. It has no real
   practice, public booking, hosted deposit, consent-aware real delivery and
   fallback, delivery/bounce proof, encrypted customer-data/export/delete, or
   purchasable plan.
2. `F-1-2`: Material landing and README promises remain outside
   `.factory/claims.json` and do not have individual observable tests.

Minor copy findings are also recorded: two metaphor section titles and an
unexplained “proof” eyebrow. The next builder must implement the real brief,
retain the isolated demo, register every claim, then repeat the full cold
review.
