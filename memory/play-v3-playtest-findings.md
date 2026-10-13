# Play surface v3 — owner playtest findings

<!-- last_updated: 2026-10-12 -->

> **Status: TEMPLATE — no playtest has been run yet.** Written by the `scutemob-268` worker; the
> coordinator fills it in from the owner's session (guide: `docs/play-v3-playtest-guide.md`) and
> files one task per finding that needs work. Nothing below the header is a result until the
> "Session" block is filled in.

## Session

| | |
|---|---|
| Date | |
| Build (commit) | |
| Command line (seed, players, bot) | |
| Mulligans taken | |
| Browser, zoom, viewport (CSS px) | (owner plays at 100 % on 2560×~1340) |
| Image store filled before play? (`/api/card-image/status`) | |
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
| 1 | | | | | | |

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
| Blocking: a block you wanted and could not express (e.g. one creature blocking two attackers, CR 509.1b — known picker limit) | |

## Not findings (already known, do not re-file)

See `memory/play-v3-c3-2026-10-12.md` §Deferred and §LOW, and the same sections of the C1 / C2 notes.

## Tasks filed

| Task | Finding # | Title |
|---|---|---|
| | | |
