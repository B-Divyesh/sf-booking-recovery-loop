# Component inventory

This is the implementation checklist for the twilight appointment carousel
system. Components are intentionally named for the product’s job, not copied
from a generic dashboard library. The released demo and practice workflow use
the same state and semantic contracts.

| Component | Surface | Required states and contract |
| --- | --- | --- |
| App shell | Foundation / M1 | header, main, footer, skip link; one h1 per route |
| Wordmark and navigation | Foundation / M1 | current page, collapsed mobile navigation, keyboard links |
| Demo safety banner | M1 | visible, reset pending/success/failure, start-for-real link |
| Appointment rail | M1 | desktop rail / mobile timeline, selected, at-risk, complete, inaccessible label fallback |
| Appointment ticket | M1 | default, selected, needs action, completed, cancelled; 44 px target |
| Station marker | M1 | planned, sent, delivered, bounced, blocked; visible text plus color/shape |
| Recovery action panel | M1 | eligible, missing consent, running, simulated receipt, stopped, error |
| Delivery receipt timeline | M1 | empty, accepted, delivered, bounced, retrying, unknown; timestamps in tabular figures |
| Public service ticket | Booking | available, selected, unavailable, error |
| Slot selector | Booking | selected, held, unavailable; labelled native date-time control |
| Contact and consent form | Booking | pristine, invalid, submitting, server error, consent withdrawn |
| Hosted-deposit handoff | Booking | ready, opening, awaiting verified callback, paid, failed |
| Status label | M1 | semantic text/icon always accompanies mint/rose/amber color |
| Field and validation message | M1 | label, hint, required, invalid, disabled, async error linked by ID |
| Button and text link | Foundation / M1 | default, hover, focus, pressed, disabled, loading; no fake links |
| Inline notice and toast | M1 | info, success, warning, error; announced once without stealing focus |
| Empty state | M1 | explains what will appear and the direct next action |
| Progress/loading state | M1 | static label, reserved layout, no mandatory animation |
| Error/offline panel | M1 | what failed, whether data changed, retry only when safe |
| Delete confirmation | Data controls | native modal confirmation names the irreversible target |

Use the values in `src/styles/tokens.css` and the rationale in
`.factory/design.md`. New components require a row here before they reach a
customer-facing screen.
