# Play surface v2 — design brief

<!-- last_updated: 2026-10-09 -->

> Status: **DRAFT for owner approval** (`scutemob-262`). Nothing is built until this is
> approved or amended. Approval is a task comment on `scutemob-262`.

## 1. Why a new surface

The browser play client (`tools/play-server/frontend`) grew out of the replay viewer, a
step debugger: zones are rows of name chips, the history is an engine event log, and
every game condition got its own strip above the board. Two polish passes on
2026-10-09 (`scutemob-260`, `-261`) fixed real things and did not change the owner's
verdict, which was: *"things shift all the time, it's never obvious what to do."*

Both complaints are structural:

- **Shift.** The top of the page is a stack of conditionally rendered bands: pregame
  banner, game-over banner, auto-pass strip, action bar, pickers, chooser, click message,
  stack box, combat box. Each appears and vanishes on its own, so the board and the
  buttons move on nearly every state change. The action buttons also reorder and resize
  as the legal-action list changes.
- **Not obvious.** The decision is a row of server-labelled buttons under a small
  heading. Nothing says, in one sentence, what the game is waiting for and what the
  usual answer is. Pass is at the far right; pickers open underneath; during other
  players' turns the bar says "No plays available."

The server, the API, the store and the picker *logic* are sound and tested (130 HTTP and
source-gate tests, Invariant 7 enforced server-side). What needs replacing is the
presentation layer: `PlayApp.svelte` and everything it lays out.

## 2. Principles

1. **Nothing moves.** Every region has a fixed slot that exists even when empty. Showing
   or hiding content changes what is *in* a slot, never where anything else is.
2. **One prompt.** Exactly one panel, always in the same place, states what the game is
   waiting for in a sentence and offers one primary action. Pickers render *inside* that
   panel at that panel's height. There is no second place to look.
3. **A table, not a log.** Cards are card images. Your board is the largest region.
   Opponents are a row above it. The history is a compact log in a side column, not the
   centrepiece.
4. **Specced for 1080p.** Every size in the layout is chosen against 1920×1080 and
   measured there; 1440p gets more room, not bigger text.
5. **Seats are first-class.** The client has a notion of *who is looking*, so CC-9
   hot-seat (`scutemob-245`) adds a seat-switch curtain and a server parameter, not a
   redesign.

## 3. Layout (1920×1080)

```
┌─────────────────────────────────────────────────────────────┬──────────────┐
│ scutemob · Turn 9 · Bot-3's turn · Main 1          [≡ menu] │ STACK        │  34
├──────────────────┬──────────────────┬───────────────────────┤  (fixed box, │
│ Bot-2  38   H7 L90│ Bot-3  40  H6 L90│ Bot-4  40  H6 L90    │   "empty")   │
│ ★ Neriv          │ ★ Toski  ACTIVE  │ ★ Ayara               │              │ 250
│ [img][img][img]  │ [img][img]       │ [img][img][img][img]  │──────────────│
│ lands: ▮▮▮ ▯     │ lands: ▮▮        │ lands: ▮▮▮▮           │ COMBAT       │
├──────────────────┴──────────────────┴───────────────────────┤  (fixed box) │
│ YOU  Human-1   40   H5 L89   ★ Nadaar (castable)            │              │
│                                                             │──────────────│
│   [img] [img] [img]        creatures / other                │ LOG          │ 420
│                                                             │  compact,    │
│   [img][img][img][img]     lands                            │  newest at   │
│                                                             │  bottom      │
├─────────────────────────────────────────────────────────────┤              │
│ ▶ YOUR MAIN PHASE. Play a land or cast a spell, or pass.    │              │ 110
│   [ Pass priority (space) ]  [ to my next turn ]            │              │
├─────────────────────────────────────────────────────────────┴──────────────┤
│ HAND  [img] [img] [img] [img] [img]          lit = playable now             │ 170
└────────────────────────────────────────────────────────────────────────────┘
   1600 wide                                                   320 wide
```

Heights are fixed on `100vh` (sum 984 + gaps; everything inside scrolls, the page never
does). The right column is `320px`, three fixed boxes: stack (top), combat (middle), log
(rest). The stack and combat boxes keep their height when empty and say "empty" in muted
text, so a spell appearing never moves the board.

**Opponent row.** One cell per opponent seat, equal widths, 250px tall. Name, life, hand
and library counts, commander chip, then permanents as small images (Scryfall `small`,
146×204, rendered at ~60px wide) grouped non-land / land; lands as a row of tap-state
pips with a count, not images, because a bot's twelve basics are not information. A seat
whose board overflows scrolls inside its cell. An eliminated seat greys and shrinks to
its header line; the row does not reflow.

**Your board.** 420px, permanents as images at ~90px wide with overlays: tapped
(rotated 90°, dimmed), summoning sick (dashed ring), counters (small badge), P/T
(bottom-right pill, red when damaged), attached (link glyph). Lands are a separate bottom
row, same-name lands stacked with a count (G13 kept). A card the current decision offers
an action for is lit (bright ring); clicking it is the click-through (reused as is).

**Prompt panel.** 110px, always present. Contents by decision:

| Decision | Sentence | Primary | Secondary |
|---|---|---|---|
| Priority, your turn, main phase | "Your main phase. Play a land or cast a spell, or pass." | Pass priority | to my next turn · to next step |
| Priority, your turn, other step | "Your [step]. Nothing to do here? Pass." | Pass | same |
| Priority, opponent's turn, nothing castable | "Bot-3's [step]. You have nothing to cast. Pass." | Pass | to my next turn (default-focused) |
| Priority, opponent's turn, something castable | "Bot-3's [step]. You could respond, or pass." | Pass | lit cards in hand/board |
| DeclareAttackers | "Declare attackers." + the attacker picker inline | Attack with N / with nothing | Back |
| DeclareBlockers | "Bot-2 attacks you with X. Declare blockers." + picker | Block with N / no blocks | Back |
| CleanupDiscard | "Discard down to 7." + picker | Discard N | use default |
| TriggerTargets / EffectChoice | the server's `prompt` + picker | Confirm | Back |
| Mulligan | "Keep this hand?" | Keep | Mulligan |
| CommanderZoneChoice | the server's prompt | the two choices |
| game over | "Game over — winner X." | New game | Export report |

The sentence is composed client-side from `decision.kind`, `state.turn`, and whether any
non-pass action exists. The primary button is always the leftmost, same size, same
place. Mana sources, concede and "new game" live in the menu, not the prompt panel
(G10 and G8 both kept: `TapForMana` stays reachable, concede stays away from the
prompt). A picker chain (X, modes, costs, targets) runs inside the panel one stage at a
time with a stage indicator ("targets 1/2"), reusing the existing picker components
unchanged.

**Hand.** 170px, cards as images at ~110px wide, playable ones lit, hover shows the full
card (existing `cardTooltip`). Click is the click-through. With more than ~14 cards the
row scrolls horizontally.

**Header.** One line: turn, whose turn, step name, and a menu (new game, seed, players,
export report, concede with its confirmation). The phase strip from `-261` is dropped;
the step name in the sentence does its job.

**Log.** The existing feed, folded and humanised as of `-260`, in its box, newest at the
bottom, tier chips kept.

## 4. Cards as images

- Source: Scryfall named-image endpoint, `version=small` for board and hand,
  `version=normal` for hover (already in use). Browser-cached per name; a four-seat game
  is ~60 distinct names.
- **Fallback is mandatory:** if the image fails or has not arrived, the card renders as a
  text chip with name, type and P/T in the same box, so the layout is identical offline.
  This is the pattern `cardTooltip`'s `onerror` already uses.
- Tokens render as text chips (no printing to fetch), as the tooltip does today.
- A hidden card (`hidden: true`) renders as a card back. Opponents' hands are never
  shown as cards, only as a count, which is the current rule (Invariant 7 is server-side;
  the client only ever sees what it is entitled to).
- Rate limit: Scryfall asks for ≤10 requests/s; a first render of a full board may fire
  ~40. Images are requested lazily (`loading="lazy"`) and only for cards in view.
  Follow-up, not in scope: a tiny image proxy in `play-server` with a disk cache, which
  would also make the surface work offline.

## 5. Seat model (hot-seat readiness)

The client renders for a **viewer seat**. Today that is always `summary.human` from the
server. The redesign keeps `viewerSeat` as one store value and routes everything that
differs by seat (whose board is "yours", whose hand is the hand bar, which prompt is
shown) through it. CC-9 then adds: a server-side seat parameter on the three routes, a
**curtain** screen ("Hand the keyboard to Bot-2 — press space when ready") that blanks
the surface between seats, and nothing else in the layout changes. This is the reason to
design it now rather than after CC-9.

## 6. Reused as is

`api.js`, `stores.js` (including pass-until, the error strip logic, global error
reporting), `plainClone.svelte.js`, `names.js`, all nine pickers
(`TargetPicker`, `AttackerPicker`, `BlockerPicker`, `ValuePrompt`, `CostPicker`,
`DiscardPicker`, `SearchPicker`, `PartitionPicker`, `ConfirmPicker`,
`BinaryChoicePicker`), `cardTooltip.js`, `EventFeed.svelte`, and `ActionBar`'s chain
state machine (`pickerNeeded` / `advanceChain` / `beginExternal`), which moves into the
prompt panel.

**Dropped from this surface:** `PlayBoard`, `SeatCard`, `PhaseStrip`, the `$viewer`
zone components (`ZoneBattlefield`, `ZoneHand`, `ZoneStack`, `CombatView`,
`PlayerPanel`, `PhaseIndicator`). The replay viewer keeps all of them; the `$viewer`
alias stays for `cardTooltip` and `CardDisplay`.

**Server: no change.** Same six routes, same `SeatView`. The one thing the client would
like and does not get is a mana cost on `CardInZoneView`; the hand image carries it, so
it is not needed for this surface.

## 7. Build plan

Built as `frontend/src/lib/v2/` behind a `?v2` query flag on the same bundle, so both
surfaces run against one server and can be compared on the same seed. The old surface
is removed only when the owner says the new one is better.

| Phase | Delivers | Acceptance |
|---|---|---|
| B1 Skeleton | fixed-slot layout, header, prompt panel with Priority sentences and pass; boards as text chips | headless probe at 1920×1080: every region's rect identical across 20 random state transitions |
| B2 Cards | images with overlays and fallback, lit playable cards, click-through, hand bar | screenshot sweep across seeds 0..5 at turn 1, 5, 10; fallback verified with Scryfall blocked |
| B3 Decisions | all pickers inside the prompt panel, sentences per decision kind, combat and stack boxes | the `-260` `verify.py` flow passes on v2; one full game to game-over on seed 3 by the greedy driver |
| B4 Playtest | owner plays a game; findings become tasks | owner verdict recorded on the task |

Class "tooling" (frontend only): suite + clippy at each phase; `npm run build` green;
the existing frontend source gates (`title` ban on tooltip anchors, no
`structuredClone`, picker error wiring) extend to `v2/` automatically because they walk
`frontend/src`.

## 8. Open questions for the owner

1. **Opponent lands as pips, not images** (§3). Agree, or images everywhere?
2. **Mana sources in the menu** rather than the prompt panel. They are needed for
   activation costs (`OOS-SIM6-3`) until auto-tap covers activations. Agree?
3. **Log column at 320px.** Keep, narrower, or collapsible to a button?
4. **Scryfall dependency** for a playable surface. Accept for now, with the proxy as a
   follow-up?
