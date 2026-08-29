# Perfection loop — polish 3

**Review source:** `5a81103e3e69dd244189f5ddc6a86dffc78bfd13`
**Candidate reviewed:** `256bde53b0e8107421ceda018d4b3a61203ce894`
**Repair commits:** `99a98ed668fb68f258edf0ba3c52b9aff1fc53db`,
`d43b9635a11fb551d81fb8f5026d2ba1c1843393`,
`15bd99b9765cdbfc6cf25316948b37615323cf25`
**Live source verified:** `15bd99b9765cdbfc6cf25316948b37615323cf25`

## Finding map

| Finding | Change made | Evidence |
| --- | --- | --- |
| `F-1-1` | Preserved the real practice setup, public session page, consent capture, scheduled recovery/reminder jobs, encrypted data, receipts, export, and deletion introduced in prior repairs. | Clean `@claim:practice-publish`, `@claim:booking-consent-record`, and `npm run test:e2e`; live `polish-3-live-final/live-check.json`. |
| `F-1-2` | Kept the narrow claims ledger and added the missing timing, data-inventory, card-boundary, delivery-connection, and independent automatic-reminder claim tests. | All 23 exact commands passed in clean clone; `.factory/claims.json`. |
| `F-1-3` | Retained “Sample recovery board” and “How booking recovery works.” | Live home screenshot: `polish-3-live-final/home-desktop.png`. |
| `F-1-4` | Retained the removed “proof” eyebrow and the consistent “delivery receipt” term. | `copy-audit.md`; live home screenshot. |
| `F-2-1` | Preserved durable, atomic scheduled recovery and reminder processing. | Clean Rust `automatic_recovery_is_durable_consent_gated_and_idempotent`; live real-practice flow. |
| `F-2-2` | Added an owner-visible delivery connection test that sends no client data and reports a clear connection failure. Removed a dead billing link after the official checkout returned 404. | Clean Rust `delivery_connection_test_verifies_the_provider_without_client_data`; billing response evidence. **Not fully closed**: an enabled Sociobot product and first-party non-developer delivery connection require factory operation. |
| `F-2-3` | Removed unsupported operational promises from README and retained named evidence for every reader-facing product promise. | Clean-clone ledger run; README and `.factory/claims.json`. |
| `F-2-4` | Retained email consent / SMS consent consistently in the UI and README. | `@claim:booking-consent-record`; live booking flow. |
| `F-3-1` | Rechecked the effective deployment boundary through the public ingress with a new verifier probe. | `polish-3-live-final/live-check.json`: writes 1–12 = `201`, write 13 = `429`, `Retry-After: 60`. |
| `F-3-2` | See `F-2-2`: delivery test is implemented and the UI no longer sends a visitor to a known 404 checkout. | `.factory/repair-evidence/polish-3-billing-check.json`. **External operator action remains required.** |
| `F-3-3` | Added `automatic-recovery-delay` and a controlled-clock assertion for the exact 15-minute schedule. Rewrote customer copy without “queues.” | Clean Rust `automatic_recovery_is_scheduled_exactly_15_minutes_after_unpaid_booking`; `polish-3-live-copy-check.json`. |
| `F-3-4` | Replaced the contradictory privacy summary and tested the exported record types against its inventory. | Clean Rust `practice_data_inventory_matches_the_exported_record_types`; live privacy copy check. |
| `F-3-5` | Added no-card-field wording and a browser claim that captures the booking request before hosted navigation. | Clean `@claim:card-data-excluded`; booking-page accessibility test. |
| `F-3-6` | Replaced the metaphor headline with the exact job. | Live verifier reports `Recover unfinished paid-session bookings`. |
| `F-3-7` | Renamed README “Use it” to “How booking recovery works.” | README source and clean-clone test run. |
| `F-3-8` | Rewrote “queues” as “schedules a consented recovery message.” | README, start-page live copy check, timing claim. |
| `F-3-9` | Rewrote payment-event jargon as “After the payment provider confirms the deposit…” | README source and clean-clone test run. |

## Earlier non-ID findings rechecked

| Earlier issue | Result / evidence |
| --- | --- |
| Concurrent sample recoveries | Preserved: `eight_concurrent_recoveries_never_return_server_error` passes. |
| Clean-clone timing | Fresh clone installed and executed all claims plus the full 27-test browser suite. |
| 200% mobile reflow and footer targets | Preserved by Playwright; final live 390 px screenshot is `polish-3-live-final/home-mobile.png`. |
| Cache headers, 404, console errors, route focus, titles, metadata, legal links | Preserved by browser suite and final `verify-url.sh` output. |
| Dark twilight rail identity and self-hosted art/type | Preserved; no generic template or third-party assets were introduced. |

## Result

Every repository-controlled finding except the external parts of `F-3-2` is
repaired and verified live. `F-3-2` cannot truthfully be marked complete until
the factory registers the billing product and provides a supported
non-developer delivery connection; the product deliberately does not disguise
either absence with a fake checkout or simulated production delivery.
