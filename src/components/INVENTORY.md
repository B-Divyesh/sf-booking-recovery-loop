# Component inventory

This is the implementation checklist for the twilight appointment carousel
system. Components are intentionally named for the product’s job, not copied
from a generic dashboard library. M1 creates the public/demo components;
later milestones complete the rest without breaking their states or semantic
contracts.

| Component | First milestone | Required states and contract |
| --- | --- | --- |
| App shell | Foundation / M1 | header, main, footer, skip link; one h1 per route |
| Wordmark and navigation | Foundation / M1 | current page, collapsed mobile navigation, keyboard links |
| Demo safety banner | M1 | visible, reset pending/success/failure, start-for-real link |
| Appointment rail | M1 | desktop rail / mobile timeline, selected, at-risk, complete, inaccessible label fallback |
| Appointment ticket | M1 | default, selected, needs action, completed, cancelled; 44 px target |
| Station marker | M1 | planned, sent, delivered, bounced, blocked; visible text plus color/shape |
| Recovery action panel | M1 | eligible, missing consent, running, simulated receipt, stopped, error |
| Delivery receipt timeline | M1 | empty, accepted, delivered, bounced, retrying, unknown; timestamps in tabular figures |
| Public service ticket | M3 | available, inactive, selected, sold out, error |
| Slot selector | M3 | loading, available, selected, held, unavailable, expired; keyboard list semantics |
| Contact and consent form | M3 | pristine, invalid, submitting, server error, consent withdrawn |
| Hosted-deposit handoff | M3 | ready, opening, awaiting verified webhook, paid, failed, expired |
| Status label | M1 | semantic text/icon always accompanies mint/rose/amber color |
| Field and validation message | M1 | label, hint, required, invalid, disabled, async error linked by ID |
| Button and text link | Foundation / M1 | default, hover, focus, pressed, disabled, loading; no fake links |
| Inline notice and toast | M1 | info, success, warning, error; announced once without stealing focus |
| Empty state | M1 | explains what will appear and the direct next action |
| Progress/loading state | M1 | static label, reserved layout, no mandatory animation |
| Error/offline panel | M1 | what failed, whether data changed, retry only when safe |
| Confirmation dialog | M5 | focus trap, explicit target, cancel/default focus, irreversible confirmation |

Use the values in `src/styles/tokens.css` and the rationale in
`.factory/design.md`. New components require a row here before they reach a
customer-facing screen.
