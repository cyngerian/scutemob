# Play surface v3 — owner playtest guide

<!-- last_updated: 2026-10-12 -->

How to run the alpha table (`?v3`, built by `scutemob-266/267/268` from mockup 7), what to try, and
how to report what you find. Findings go into `memory/play-v3-playtest-findings.md`; the coordinator
files a task per finding. Design reference: `docs/play-surface-v3-brief.md`.

## 1. Start

```sh
# once: build the frontend (the server serves tools/play-server/dist)
cd tools/play-server/frontend && npm ci && npm run build && cd ../../..

# a stable image store, shared by every checkout and worktree (recommended)
export SCUTEMOB_IMAGE_STORE=~/.cache/scutemob/card-images

cargo run -p play-server -- --port 3040 --players 4 --bot heuristic --seed 0
```

Open **<http://127.0.0.1:3040/?v3>** at **100 % browser zoom** on the 1440p monitor (the surface is
designed and measured at 2560×1340 and 1707×880 CSS px; every earlier verdict at 150 % zoom is void).
Without `?v3` you get the old default surface; `?v2` is the retired B1/B2 surface.

`--seed N` picks the table (decks are random per seat until CC-5 decklists exist); `--players 2..6`.
The menu (top right) deals a new game, exports a bug report, and concedes.

### Card images (first game on a seed takes a few minutes to fill)

The browser never talks to Scryfall on v3. When a game is dealt the server downloads every card of
every deck into the image store in the background (politely: one card every ~120 ms, ≈400 cards for
four seats). Until a card has arrived it is drawn as a **text chip** — that is the designed fallback,
not a bug.

- Progress: <http://127.0.0.1:3040/api/card-image/status> — ready when `queued` is `0`. `failed`
  lists names Scryfall could not resolve (those stay chips: report them).
- A chip turns into a picture on the next render after its image arrives (reload if impatient).
- To fill the store before sitting down: start the server, deal the seed you will play, wait for
  `queued: 0`, then reload. For a known list: `cargo run -p play-server -- --prefetch-images <file | "Name;Name">`.
- `--no-image-prefetch` serves the store without ever fetching (all-chips play works).

## 2. What to try

Play at least one game to the end (or until something blocks you). In rough order:

1. **The table.** You in the middle; one opponent across the top; every opponent keeps a side cell.
   Click a cell to bring that player across; a turn change brings the active player across. Hold the
   pointer on a cell for a second to peek at it double-size. Size slider / auto-fit in the top bar.
2. **Zones.** Each board has a 2×2 box (command, library, graveyard, exile), each cell a 1×5 strip.
   Click any pile for its list. **New in C3:** exile is per player (no more shared `EX*`), and the
   command pile shows the commander tax (`+2`, `+4`, …) once a commander has been cast.
3. **Hover and the card window.** One hover overlay beside the card; "card window" in the top bar
   pops a large view out for the second monitor.
4. **Decisions.** The decision opens in the centre when you hold priority (minimise it to the dock;
   a new priority re-opens it). Options are cards.
5. **Arrow targeting.** Drag from an action (or from one of your cards) to any card or seat on the
   table. The list in the decision box stays as the fallback — note which one you reach for.
6. **Card menus.** Right-click a card: "Use" (what it is the source of), "Target with" (what could
   target it), counters. Right-click a seat for the seat menu.
7. **The stack.** Top item is lit ("resolves next"). **New in C3:** what a stack item targets gets a
   red outline on the table, and hovering the stack item draws arrows to its targets.
8. **Combat.** Declare attackers; when attacked, the block box (attackers on top, your creatures
   below): click a creature then an attacker, or drag. The combat card in the dock shows every
   attacker and blocker per defending player; click it to enlarge.
9. **Rearranging.** Drag cards within your hand row and within each battlefield row. The order is
   remembered per game (**C3:** keyed by the server's game id — a new deal starts clean).
10. **Auto-pass panel** (under the right-hand cells): off / until end of turn / until my next turn /
    until a chosen step, with stop conditions. **C3:** a run is one server request, so the table
    jumps straight to where the run stopped, and a run cannot be interrupted part-way — the stop
    conditions are its brakes. Check that the reason it gives for stopping matches what happened.
11. **Summoning sickness.** A dashed violet inner frame on creatures that cannot attack yet.
12. **Game over.** Play until someone wins or you are eliminated; check the end screen says what
    happened.

Things known to be limited (do not report; see `memory/play-v3-c3-2026-10-12.md` §Deferred / §LOW):
one creature cannot be declared as blocking two attackers (CR 509.1b — a limit of the blocker
picker); revealed hand cards / a revealed library top are not shown (the engine has no "revealed"
state to render); mulligans redeal the whole table.

## 3. How to report

For each thing that is wrong, awkward or missing, note:

1. **What you did and what you expected** — one or two sentences.
2. **Where** — which part of the surface (cell, zone box, stack, decision, combat card, auto-pass…).
3. **Repro** — menu → **Export report** right when it happens (it saves the seed, mulligan count and
   the full command journal; that file reproduces the table exactly). If you cannot export: the
   `--seed`, how many mulligans, and the turn / step from the phase track.
4. **A screenshot** if it is about layout or legibility (say the zoom and window size if not the
   usual 100 % full-screen).
5. **How bad** — blocker (cannot keep testing) / wrong / friction / polish.

Tell the coordinator session (or paste into `memory/play-v3-playtest-findings.md` under "Findings").
The template there also carries a short **watch list** — priority stops vs auto-pass noise, deep
response chains in the decision panel, simultaneous-trigger ordering, bot timing — answer each, even
with "did not come up".

A rules bug (the engine did something Magic does not) is as welcome as a UI finding: say which card
and which rule you believe applies; the export file is what makes it fixable.

## 4. Open question for you

The portfolio session suggested a **friction-log hotkey** on the v3 surface (one keypress writes a
one-line note with a state snapshot: turn, step, stack, seat). It was not built in C3 — it needs your
ruling on whether it is wanted and whether it is its own task (task comment on `scutemob-268`,
2026-10-12).
