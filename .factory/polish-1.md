# Perfection loop — polish 1

**Candidate repaired:** `4bc479de1f4d464bfd071bd3b2f0a4bb7c659aa5`

**Review source:** `5298990ff778394f8bebdccb3d7c92b3e2f5d7e8`

**Live build checked:** `e504d7f743c661f457aa52c80b3c315492cc3ffe`

## Finding map

| Finding | Change made | Evidence |
| --- | --- | --- |
| `F-1-1` | Replaced the demo-only dead end with owner-scoped setup, public session pages, consent capture, hosted-payment handoff, authenticated payment callbacks, provider delivery and receipts, bounce-to-SMS fallback, encrypted contacts, export, and deletion. Demo remains separate. | `@claim:practice-publish`; `@claim:booking-consent-record`; Rust tests `production_contacts_are_encrypted_and_tenant_scoped`, `hosted_payment_requires_verified_callback_and_is_idempotent`, `delivery_acceptance_bounce_fallback_and_receipts_are_idempotent`, `occupied_slot_cannot_be_double_booked`, and `export_and_delete_cover_the_complete_practice_record`; `.factory/evidence/polish-1-live/live-check.json`. |
| `F-1-2` | Pruned unsupported copy and expanded the ledger from 8 to 16 narrow claims. Each command ran separately from a clean clone before the full suite. | `.factory/claims.json`; `.factory/evidence/polish-1-clean/claims.log` ends with `ALL CLAIM COMMANDS PASSED`; `full-suite.log`. |
| `F-1-3` | Renamed the headings to “Sample recovery board” and “How booking recovery works.” | Playwright landing route test; local and live home screenshots. |
| `F-1-4` | Removed “Booking follow-up with proof” and standardized on “delivery receipt.” | `.factory/copy-audit.md`; live verifier HTML and screenshot. |

## Earlier findings rechecked

| Earlier finding | Evidence now |
| --- | --- |
| Concurrent sample recovery returned 500 | Rust `eight_concurrent_recoveries_never_return_server_error` passes. |
| First clean claim timed out | All 16 exact commands pass from a fresh clone. |
| Claims inventory was incomplete | 16 ledger entries, 16 passing commands. |
| 200% mobile reflow broke | Playwright `390px at 200 percent text reflows...` and live mobile checks pass. |
| Footer targets were under 44 px | The same Playwright test measures every footer target. |
| Static assets lacked immutable caching | Playwright cache-header test passes. |
| Unknown paths returned 200 | Local and live `/not-a-real-place` return 404. |
| Expected consent stop logged an error | Dedicated console test passes. |
| Rust image was pinned | Both Dockerfiles use `rust:1-slim`; ACR build `chrh` passed. |
| Live rate limit missed 12 writes | Live writes 1–12 returned 201; write 13 returned 429 with `Retry-After: 60`. |

## Additional live defect caught

The first deployed setup run exposed an invalid HTML `pattern` under Chromium’s
Unicode-set regex rules. The hyphen is now escaped, and
`@claim:practice-publish` asserts zero console errors after editing it. The
container was rebuilt; the second cold live run returned `passed: true`.

## Evidence paths

- Local screenshots and Lighthouse: `.factory/evidence/polish-1-local/`.
- Clean-clone claim and suite logs: `.factory/evidence/polish-1-clean/`.
- Live screenshots, verifier output, Lighthouse, health, rate limit, and scale:
  `.factory/evidence/polish-1-live/`.

## External factory boundary

The source-level portion of `F-1-1` is implemented and live. The factory-owned
Sociobot billing registry still returns 404 for this slug, and the shared Entra
redirect is not registered. The UI does not pretend either works. Exact
operator evidence and safe current behaviour are recorded in the handoff.
