# Booking Recovery Loop — design thesis

**Direction:** twilight appointment carousel
**Mode:** intentionally single-mode, dark “blue hour” interface

Booking Recovery Loop is a small practice’s calm control room at the moment an
appointment might slip away. It should feel like looking through the window of
a quiet late-evening studio: appointments pass along a lit carousel, and an
amber signal shows where a human needs to act. This is not a generic SaaS
dashboard, a neon cyberpunk board, or a gradient hero. The carousel gives the
product a physical idea: an attempt moves from interest, to deposit, to
reminder, to attendance; a break in that loop is visible.

## Why this direction fits

Tutors, coaches, and consultants do not need more scheduler chrome. They need
a calm, accountable view when a paid session is at risk. The deep blue setting
reduces visual noise during after-hours admin. Warm amber reserves attention
for the next recovery action. Thin rail and ticket motifs explain status and
sequence without turning each booking into a generic card. The view stays
human: names, consent, and delivery evidence are prominent; automation is
subordinate and never presented as magic.

The dark theme is a deliberate product mode, not an unfinished light theme.
It uses explicit background surfaces at every layer and is checked for 4.5:1
text contrast. Marketing, booking, settings, and error pages all remain in the
same twilight world so the public link feels connected to the owner’s console.

## Token system

The executable CSS variables live in [src/styles/tokens.css](../src/styles/tokens.css).
Do not introduce arbitrary hex values in components.

| Token family | Value / rule | Use |
| --- | --- | --- |
| `--color-night` | `#0D1324` | blue-hour page background |
| `--color-cove` | `#151E35` | primary surface and header plate |
| `--color-slate` | `#202C49` | raised rail, inputs, static panels |
| `--color-rail` | `#536481` | quiet separators and carousel track |
| `--color-paper` | `#F8F3E8` | primary text and ticket face |
| `--color-mist` | `#CED4E5` | secondary text; never below 4.5:1 on night |
| `--color-amber` | `#FFBE5C` | the one primary action and attention marker |
| `--color-amber-ink` | `#1E2434` | text on amber controls |
| `--color-mint` | `#82E2C7` | delivered/healthy state with text label |
| `--color-rose` | `#FF9BA2` | failed/blocked state with text label |
| `--color-sky` | `#9CCBFF` | links, info, focus outline companion |
| Space | 4 px base: `4, 8, 12, 16, 24, 32, 48, 64, 96` | all gaps/padding/margins |
| Radius | `8, 14, 22, 999px` | tickets are 14 px; pills only for brief status |
| Shadows | soft navy, never black; one elevated `--shadow-float` | visually separates an active case without glass blur |

The primary “Recover this booking” amber button uses `--color-amber-ink`, and
the default body copy uses `--color-paper`; these token pairs meet or exceed
4.5:1. Colour never communicates a state alone: every status has a word,
icon shape, and a text alternative in the delivery timeline.

## Typography

The headline face is **Fraunces** (OFL), a gently editorial serif used only for
page titles, booking service titles, and meaningful numbers. Body/UI is
**Atkinson Hyperlegible Next** (OFL), selected for clear names, times, and
delivery evidence. M1 must self-host subset WOFF2 files in `public/fonts/`,
preload at most the normal body and display variable files, use
`font-display: swap`, and commit their OFL license text in
`public/fonts/LICENSES.md`. No font loads from a CDN.

M1 ships the Latin variable subsets in `public/fonts/`, preloads both files,
and keeps the full OFL texts beside them. The files came from the Fontsource
5.3.0 packages; no font network request is made. Type scale (with 1.25-ish
steps): 14/20 label, 16/24 body, 20/28 section, 25/32 screen, 32/40 display,
40/48 landing. Body measure is 45–70 characters; tabular numerals are used for
times, prices, and counts.

## Layout and shape language

- The wide desktop board has a narrow left navigation rail, a central
  appointment carousel, and a contextual detail panel. It is a sequence, not
  a grid of unrelated metric cards.
- The public booking page puts the selected session ticket before contact
  fields. On mobile, the carousel becomes an ordered vertical timeline and the
  detail panel follows the active ticket; no horizontal data-table scroll is
  required for the primary action.
- Ticket surfaces use a 14 px rounded outer edge, a restrained dotted or
  notched divider only where it separates action from evidence, and a 1 px
  slate border. Standard containers remain quiet surfaces—not every group
  becomes a card.
- The carousel rail is a thin line with station nodes. A bright amber station
  is always paired with a verb and a clear next step. A delivered station is
  mint plus “Delivered”; an error station is rose plus “Needs attention.”
- 390 px is the first layout. Header navigation stacks into a labelled row;
  status chips wrap; primary actions become full width; touch targets are at
  least 44 by 44 px with 8 px between controls.

## Interaction and motion grammar

The signature movement is a ticket joining or leaving the appointment rail:
it slides at most 16 px from the rail with a 180–240 ms transform/opacity
transition. Selecting a case moves the detail emphasis from the selected
ticket, not from the screen edge. A new delivery receipt gently reveals its
timestamp; it does not pulse or flash. There is no endlessly moving carousel,
no autoplay video, and no parallax needed to understand the page.

`prefers-reduced-motion: reduce` removes transforms and uses instant or
120 ms opacity changes. Loading uses a static labelled progress state rather
than a shimmering skeleton. Motion is never the sole signal of a changed
delivery status.

## Original art and provenance

M1 ships the hand-made `src/assets/appointment-rail.svg`: a midnight rail,
three ticket silhouettes, and a small amber station light. Its source includes
the comment **“Hand-made for Booking Recovery Loop, Param Factory, 2026; no
external asset or model output.”** The landing image alt text is: “A calm
appointment rail showing one booking that needs a follow-up.”

`public/social-card.svg` is a second hand-made composition derived from the
same rail. `public/social-card.png` is a 1200×630 Chromium render of that source.
`public/apple-touch-icon.png` is likewise rendered from the hand-made product
mark. No image model, stock asset, brand, or generated bitmap source was used.
If a later milestone needs a hero image, it must use a factory image
model, be reviewed for artifacts, export responsive WebP/AVIF below 300 KB on
mobile, and append its exact prompt, model, date, output filename, and license
status here. Generated imagery is disclosed in the footer/about content. Text
needed to operate the product never appears only inside art.

## Component inventory

The living implementation inventory is [src/components/INVENTORY.md](../src/components/INVENTORY.md).
These components share the tokens above and must expose semantic states:

1. App shell and skip link
2. Wordmark and labelled navigation
3. Demo safety banner
4. Appointment rail / mobile timeline
5. Appointment ticket
6. Station/status marker
7. Recovery action panel
8. Delivery receipt timeline
9. Public booking service ticket
10. Slot selector
11. Contact and channel-consent form
12. Hosted-deposit handoff panel
13. Status label with text/icon
14. Field and validation message
15. Button/link set
16. Toast and inline notice
17. Empty state
18. Loading/progress state
19. Error/offline state
20. Destructive confirmation dialog

## Five key screens in words

1. **Landing / demo entry:** The evening rail occupies the visual half of the
   opening screen. A plain headline, one sentence, a sample-data action, and
   three facts sit on a solid cove plate. A miniature ticket on the rail shows
   “Needs a follow-up,” making the product tangible without a generic feature
   grid.
2. **Demo recovery board:** A persistent sample-data banner sits above a rail
   of seeded appointments. The selected unfinished booking opens a right-side
   evidence panel showing consent, what will happen in demo, and a labelled
   simulated delivery receipt.
3. **Public paid-session page:** A warm paper session ticket floats on night
   background. Service, date, deposit, and selected time are readable before
   client details. Channel consent is an explicit field group, followed by a
   button that says it opens secure payment.
4. **Recovery case detail:** The case progresses vertically through attempt,
   message, receipt, fallback, and outcome. Each station says the time and
   meaning in words. An amber action is available only when a human decision is
   needed.
5. **Settings / data rights:** This is quieter and denser: paper labels on
   cove surfaces, purpose-led sections, and a clearly bounded danger zone.
   Export and delete actions explain their result before any confirmation.

## States and accessibility rules

- Empty recovery queue: “No bookings need a follow-up right now. New at-risk
  bookings will appear here.” Include a link to recovery rules.
- Loading: label the content being loaded and retain surrounding layout.
- Error: state what failed, whether data was changed, and the one safe retry.
  Preserve form inputs and show server-validation errors next to labels.
- Offline: state that booking/payment confirmation needs a connection; never
  pretend a payment or message was sent.
- Focus has a 3 px sky outline with visible offset. Keyboard tabs through rail
  tickets, controls, and dialogs in visual order. A selected ticket uses
  `aria-current` or a labelled pressed/selected state.
- All forms have programmatic labels, required hints, errors linked by
  `aria-describedby`, and live-region announcement for async completion.
- Charts are avoided; any future delivery aggregate includes a table/text
  equivalent. Status icon SVGs are paired with visible text.

## Stack/design note

The frontend is Vite + strict TypeScript + vanilla CSS rather than React. The
interface has focused state and benefits from a small, deliberate component
layer, lower first-load JavaScript, and direct control of the appointment-rail
layout. The backend is Rust/axum/Postgres because multi-tenant contact,
payment, and delayed-message state needs strong server boundaries.
