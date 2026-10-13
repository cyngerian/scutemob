# Play surface v3 — owner playtest findings

<!-- last_updated: 2026-10-12 -->

> **Status: IN PROGRESS — first owner session 2026-10-12 (coordinator filling in live).** Written by the `scutemob-268` worker; the
> coordinator fills it in from the owner's session (guide: `docs/play-v3-playtest-guide.md`) and
> files one task per finding that needs work. Nothing below the header is a result until the
> "Session" block is filled in.

## Session

| | |
|---|---|
| Date | 2026-10-12 (evening) |
| Build (commit) | main `7bf35d8e` (C3 merge `01f4ac59`) |
| Command line (seed, players, bot) | `--port 3040 --players 4 --bot heuristic --seed 3`, dealt via `POST /api/game` (seed 3) |
| Mulligans taken | |
| Browser, zoom, viewport (CSS px) | (owner plays at 100 % on 2560×~1340) |
| Image store filled before play? (`/api/card-image/status`) | NO — cold store; server prefetch stuck on api.scryfall.com 429s (41 cards in ~4 min); coordinator ran a CDN-by-id filler alongside |
| Games played / turns reached / how each ended | |
| Bug reports exported (`GET /api/game/report` files) | |

## Verdict (owner's words)

> …

Usable for alpha testing as is? **yes / yes with the fixes marked BLOCKER / no**

## Findings

One row per finding. `Where` is the surface region (table, cell, hand, zone box, stack, combat card,
decision, auto-pass, arrows, card menu, images, card window, server). `Layer` is the owner's or the
coordinator's best guess — **UI** (presentation only), **server** (view / route), **engine** (rules,
with the CR number if known), **bot**. `Repro` is the seed + mulligan count + turn / step, or the
exported report file. Severity: BLOCKER (cannot keep testing), HIGH (wrong or misleading), MEDIUM
(friction), LOW (polish).

| # | Severity | Where | Layer | What happened / what was expected | Repro | Task |
|---|---|---|---|---|---|---|
| 1 | BLOCKER | images | server | Cold store + rate-limited lookup path: "untestable without art". Prefetch must not depend on `api.scryfall.com` (ids are in `cards.sqlite`). | seed 3, first deal, ~23:20 | `scutemob-270` (dispatched) |
| 2 | HIGH | hand, board | UI | After the store filled, hand/board cards stayed chips while hover showed art: `CardFace` remembers the failed small-image URL until a full reload. Expected: faces pick the art up when the queue drains. | seed 3, turn 1, after `queued` fell | (playtest batch — not in `-270`) |
| 3 | MEDIUM | server | server | The server starts with no game: `GET /api/game` → `no_session` until `POST /api/game`; the owner's first page load read "no game is running". `--seed/--players/--bot` are only defaults. Expected: a start-up deal when `--seed` is given, or a guide line. | server start | (file at session end) |
| 4 | HIGH | board | UI | Equipped Swiftfoot Boots to Nadaar: nothing on the table shows the attachment (engine correct: `attached_to`/`attachments` in the view, Nadaar gained Hexproof + Haste). Expected: an Aura/Equipment visibly attached to its host (tucked under / badge / line), on every seat's board and in hover. Mockup had no attachment rule — design it. | seed 3, turn 13 | (playtest batch) |
| 5 | MEDIUM | decision | UI | Minimising the decision box: wants a toggle "stay minimised" — today a new decision re-opens the centred float every time (C1 rule). Expected: a pin that keeps it docked until the player maximises it. | seed 3, turn 13 | (playtest batch) |

## The watch list (portfolio session's suggestion, task comment 2026-10-12)

Answer each, even with "did not come up":

| Watch item | Observed |
|---|---|
| Priority stops vs auto-pass noise at this table size (does the panel stop too often / not enough?) | |
| 3–4 deep response chains in the decision panel (still readable? stack box keeps up?) | |
| Simultaneous triggers: ordering asked for and APNAP respected? | |
| Bot pass / respond timing (does a bot's action get lost between two of your decisions?) | |
| Arrow targeting vs the list: which did you reach for? any target you could not point at? | |
| Card menus (right-click): anything missing from "Use" / "Target with"? | |
| Rearranging: did the order survive the game? a new game? | |
| Per-player exile and commander tax: correct and where you looked for them? | |
| Images: any card left as a chip after the store was filled? (name it) | |
| A big combat: the dock's combat card scrolls inside itself past 60 % of the window height (the worker's number) — acceptable, or should it shrink / summarise instead? | |
| Blocking: a block you wanted and could not express (e.g. one creature blocking two attackers, CR 509.1b — known picker limit) | |

## Not findings (already known, do not re-file)

See `memory/play-v3-c3-2026-10-12.md` §Deferred and §LOW, and the same sections of the C1 / C2 notes.

## Tasks filed

| Task | Finding # | Title |
|---|---|---|
| | | |
