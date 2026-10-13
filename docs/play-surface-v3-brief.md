# Play surface v3 — the alpha UI

<!-- last_updated: 2026-10-12 -->

> Status: **APPROVED by the owner 2026-10-12** as "the alpha UI … the ui I will use for
> testing". The design is the static mockup `tools/play-server/mockups/07-table6.html`,
> frozen as `tools/play-server/mockups/snapshots/07-combat-2026-10-12.html` and git tag
> `mockup-7-combat`. This brief turns it into a build. It supersedes
> `docs/play-surface-v2-brief.md` (v2 = B1/B2 behind `?v2`, judged "busted from its
> inception" on 2026-10-12 — partly because every verdict until then was made at 150 %
> browser zoom; the owner now plays at 100 % on the 1440p monitor, 2560×~1340 CSS px).

## 1. What the mockup settled (do not re-litigate)

Seven rated rounds on one page produced these rules. The mockup is the reference for every
pixel decision below; when the brief and the mockup disagree, the mockup wins.

1. **You in the middle.** Your board is the centre of the screen: creatures, other
   permanents, lands (smaller) in three rows, hand row under it (fixed 70px cards, pages
   with arrows only when the width runs out).
2. **The board across the table.** One opponent is "focused" at the top of the centre
   column, mirrored (lands row at the top, creatures nearest yours). Clicking any opponent's
   cell brings them across; a turn change brings the new active player across. Their hand is
   card backs (36px) on their lands row; revealed cards face up.
3. **Fixed seats.** Every opponent keeps a cell in the side columns (left, right, left …
   in seat order; six seats = three + two). The focused one's cell is greyed out (dimmed and
   desaturated, "across" tag) so it is obvious that board is the one at the top; the top
   copy is in addition, never instead. The grey-out lifts whenever that cell's cards can be
   chosen — while a spell or ability is being targeted (arrow or decision box), while blockers
   are being declared, and on hover — and returns when the choice is done. A cell is mirrored like the board across: under the
   1×5 strip come lands, then other permanents, then creatures last (owner, 2026-10-12).
4. **Zones are piles with the top card showing.** Your board and the focused board each
   carry a 2×2 box (title row: name + life; then command zone, library, graveyard, exile)
   pinned to a corner — bottom-right for you, top-right across. Side cells carry a 1×5 strip
   on the name line (command, hand, library, graveyard, exile). Piles are **fixed size**
   (55px in the boxes; strip sized once from the cell width, 34–56px), never touched by
   autosize. The command slot has its own tint; empty = dashed frame with a star. Any pile
   opens a popover listing its cards.
5. **Autosize is battlefield-only, measured, fills, never scrolls.** Per board: rows fill
   the width (never fewer than 8 per row) and the browser is asked whether the box scrolls;
   shrink until it does not, grow until it would. Your board and the focused board share one
   size (the smaller); all side cells share one size. Side cells may drop below 8 per row to
   use their height ("opp auto-size" toggle). The size slider is authoritative when moved
   (auto-fit off; boards scroll).
6. **Card face.** Name band on top (two full lines, ellipsis, bold sans-serif, scaled with
   the card, floor 8px), P/T bottom-right scaled with the card (14.5 % of width), counts
   bottom-right on land stacks, commander star bottom-left, tapped = whole-card tilt plus
   dimmed art, no "T" tag, targeted = red outline.
7. **One hover.** A single fixed overlay (240px, the 488px printing) beside the hovered
   card, flipped near edges, never clipped. The same hover feeds a popped-out card window
   (`BroadcastChannel`) for the second monitor. *C1 amendment (2026-10-12): the build feeds it
   by a `localStorage` write + `storage` event instead — the existing frontend gate bans
   `.postMessage(` outright and the ban was kept (`memory/play-v3-c1-2026-10-12.md` §Gates).*
8. **Active and priority recolour the container's own border**, nothing added: active =
   blue, priority = gold pulsing; the gold wins when both. Applies to the cell, the focused
   board, your board and their 2×2 boxes.
9. **Phase track**, not a dial: a row under the header with the twelve steps, past / future /
   combat each a real colour, current white; turn and whose turn left, priority right.
10. **Dock (right column, 19rem):** stack (top item lit, "resolves next"), **combat card**
    (one bordered box per defending player; each attacker with its blockers beside it; click
    to enlarge to a centred lane-per-defender view), **decision**.
11. **The decision opens in the centre** when you hold priority (minimise to the dock; a new
    priority re-opens it). Options are **cards** with a caption, grouped (cast / activate /
    mana muted), hover zoom on them. Declaring blockers: attackers on top, all your untapped
    creatures below, click a creature then the attacker to draw an arrow; finish when done.
12. **Auto-pass panel** under the right column's cells: off / until end of turn / until my
    next turn / until a chosen step, with stop-conditions (a spell is cast, an ability is
    activated, attackers declared at you) and a sentence stating the rule.
13. **Legibility pass** (1080p screenshot): no grey-on-grey; row labels, pile labels, stack
    text, panel titles, hand label all readable.
14. **Peek.** Holding the pointer on an opponent's cell for one second grows it to double size
    in place — anchored on its own corner, over its neighbours, not a centred window — with its
    cards autoscaled to the bigger box and hoverable as usual; it closes when the pointer
    leaves. It arms only while the pointer rests on the cell itself, never while a card in
    the cell is being hovered (owner, 2026-10-12). Lets you check a board without bringing that player across (owner, 2026-10-12).

## 2. Must land in the build (owner, 2026-10-12 — not possible in the mockup)

1. **Arrow targeting.** Drag an arrow from an action (or from a card) to a target card
   anywhere on the table, including cards in opponent cells, instead of enumerating every
   target in the decision box. The decision box still lists targets, but in a large game the
   arrow is the primary way.
2. **Card context menus.** Right-click on a card lists the actions that could use it as a
   legal target, and the actions it is the source of. Things like moving counters are chosen
   on the card, not by enumerating all potential targets. Most actions should be reachable
   by context menu and arrows as well as by the decision box.
3. **Rearranging.** Drag to reorder cards in your hand and on your battlefield within the
   available grid. Client-side order, remembered per game.
4. **Summoning sickness** gets an indicating outline on creatures.

## 3. Architecture

- Same bundle, same server. Mounted behind `?v3` from `main.js`; v2 stays behind `?v2` and
  the old surface stays default until the owner says otherwise. Removing them is a later task.
- **Reuse (wiring):** `lib/api.js`, `lib/stores.js` (seat view, decision, act, pass-until,
  errors), `lib/names.js`, `lib/v2/prompt.js` (sentences, loose actions, answerable), the
  picker components' logic (targets, X/modes, attackers, blockers, costs, discard, search,
  partition, confirm, binary).
- **New (presentation):** `lib/v3/` — `TableV3.svelte` (grid), `SideCell`, `FocusedBoard`,
  `YourBoard`, `HandRow`, `ZoneBox`, `ZoneStrip`, `Pile`, `PilePopover`, `CardFace`,
  `HoverCard` (+ `card-window` route), `PhaseTrack`, `Dock` (`StackBox`, `CombatCard`,
  `CombatFloat`, `DecisionPanel`), `AutoPass`, `ContextMenu`, `Arrows` (SVG overlay),
  `fit.js` (the measured fit, ported from the mockup), `layout.js` (zone split by card type,
  seat order, focus rules).
- **Invariant 7** unchanged: the client renders what the seat view gives; no hidden-info
  inference. Opponent hands are backs because the view gives backs.
- **Data the view lacks today** (server tasks, §5): per-player exile, commander tax, revealed
  cards (hand / library top), oracle text (not needed: images), a card image proxy.

## 4. Phases

| Phase | Task | Scope | Acceptance |
|---|---|---|---|
| C1 | `scutemob-266` | the table from live state: layout, seats, focus, boards, hand, zone boxes/strips/popovers, fit, hover + card window, phase track, dock with stack / combat card / decision (card items, centred, minimise), seat right-click menu; priority playable; existing pickers reachable | headless 10-turn drive on two seeds at 2560×1340 and 1707×880: no page scroll, no box scroll in auto mode, no page errors; source gates in `main.rs`; all cargo gates + frontend build green |
| C2 | `scutemob-267` | arrow targeting (drag from action/card to any card), card context menus (source + target actions, counters), blocker arrows, auto-pass panel (client stop-conditions; step param), summoning-sick outline, drag-to-rearrange hand and battlefield (persisted per game) | each interaction exercised headless on a real decision; sweep as C1 |
| C3 | `scutemob-268` | server gaps (per-player exile, commander tax, revealed cards, image proxy, pass-until-step), full game to game-over on seed 3, `verify.py` on v3, owner playtest → tasks | game over reached; `verify.py` green; owner verdict recorded |
| C4 | `scutemob-269` | **classic layout** (owner idea 2026-10-12): a layout switch on `?v3` divides the table into N equal boards, Cockatrice-style — no focused board, no side cells, no peek; every C1–C3 component reused; reference `mockups/reference/cockatrice*.png` | owner rates classic vs focus on one seed after the first playtest; sweep + interact green in both layouts |

## 5. Server changes the design asks for

- `PlayerView.commander_tax` (and `in_command_zone` derivable from `command_zone`).
- `ZonesView.exile` per player (owner-keyed like graveyard), keeping the shared list for v1/v2.
- Revealed information: hand cards revealed to all / library top revealed (needs an engine
  concept; may stay a follow-up if the engine has none).
- `GET /api/card-image?name=&version=` served from a **local image store**. *Owner ruling
  (2026-10-12): every card image a deck needs is downloaded when the deck is added (decklist
  resolution time, the sizes the UI uses), so play never depends on Scryfall once the decks are
  determined. The route reads the store; a miss is a visible chip, not a live fetch.*
- Pass-until a chosen step (the loop exists; add the stop step and "unless" conditions server
  side or evaluate them client side from events).

## 6. Verification

- Headless drives (`tools/play-server/mockups/screenshot.py` pattern): region rect stability
  is **not** the metric any more; the metrics are no page scroll, no auto-mode box scroll,
  every card has an image or a chip, zero page errors, decisions answered.
- Screenshots at turns 1 / 5 / 10 per seed, reviewed by eye against the mockup.
- The mockup page stays as the design reference; it is updated only when the owner asks.

## 7. Dispatch notes for C1 (`scutemob-266`)

For the coordinator writing the worker brief:

- **Change class:** new presentation surface, no engine change (`memory/conventions.md` →
  "Change-class acceptance table", the UI row). Gates: `cargo test --workspace`, clippy
  `--workspace --all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh`,
  `npm run build` in `tools/play-server/frontend`, the frontend source gates in
  `tools/play-server/src/main.rs` (`collect_frontend_files`, add a v3 gate beside
  `test_frontend_v2_card_images_are_lazy_and_fall_back`).
- **Start from:** `frontend/src/lib/v3/layout.js` (zone split, seat order, side columns,
  focus rule, combat grouping, step order — written 2026-10-12, use or replace);
  `frontend/src/main.js` (mount behind `?v3`, and a `?card-window` mount for the pop-out);
  `frontend/src/lib/v2/PlayAppV2.svelte` (click-through, offered ids, mana rows, concede —
  port, don't import); `frontend/src/lib/v2/PromptPanel.svelte` (the picker chain and
  `beginExternal`; host it inside the v3 decision panel rather than rewriting the pickers);
  `frontend/src/lib/stores.js`, `lib/api.js`, `lib/names.js`, `lib/v2/prompt.js`,
  `lib/v2/images.js` (add a `normal`-size URL for the hover overlay).
- **Port verbatim from the mockup:** `tools/play-server/mockups/07-table6.html` and
  `common.css` (`.t6` / `.t7` rules) — the grid, `fitBoard` / `fitAll` / `sizeStrips`,
  `showHover`, the dock, the combat card, the decision float, the zone box and strip CSS,
  the legibility block. The mockup's sample data (`common.js`) is NOT ported; the seat view
  is the data.
- **Data notes:** opponent hands are `zones.hand[name]` entries with `hidden: true` → backs;
  exile is one shared list (label it shared in every box until C3); command zone cards are
  `zones.command_zone[name]`, a commander on the battlefield is `is_commander`; combat is
  `state.combat` (`attackers[].target` is `player:<name>` / `planeswalker:<name>`,
  `blockers[]`); P/T for a combat card comes from the battlefield entry with that `object_id`.
- **Probes:** one play-server per game — run probes on 3041/3042, never the owner's 3040;
  the drive script pattern is `tools/play-server/mockups/screenshot.py` plus the B1/B2
  `v2sweep.py` loop (deal, keep hand, answer decisions, screenshot turns 1/5/10); commit
  the v3 sweep under `tools/play-server/probes/`. Seed `OOS-DX20b-1` (Purphoros offered
  as an attacker, engine refuses) is a known engine seed, not a surface bug.
- **Must not:** `git add -A`; touch `?v2` or the default surface; re-filter hidden info on
  the client (Invariant 7); run the CR/Scryfall data refresh.
