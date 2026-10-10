# Play-server UI usability pass — `scutemob-260` (2026-10-09)

Self-assigned (`/task`), frontend only, class 1 (no engine, no wire, no Rust). Branch
`feat/play-ui-usability`. Owner asked: "audit the play server for usability and make
improvements … mostly interested in the UI".

## Method

The surface was driven headless with `playwright` (python, already present for webber) on
`/usr/bin/chromium` against a real `play-server` on :3040, seed 3, four seats, heuristic
bots. Scripts in the session scratchpad: `shot.py` (first screens), `drive.py` (greedy
play until a picker, stack or combat appears), `verify.py` (asserts every criterion after
the change). Screenshots were read directly; nothing below was reasoned about from source
alone.

## Findings → changes

| # | Finding (screenshot) | Change |
|---|---|---|
| 0 | Served `dist/` was built 2026-09-05-ish and lacked G8/G10 (Cancel vs Back, `TapForMana` in the play list). `dist/` is gitignored. | `npm run build`; README now says to rebuild after pulling. Not a repo change. |
| 1 | Nothing marks which hand / battlefield / command cards can be used; every card looks the same until clicked. | `offeredIds` (Set) derived in `PlayApp` from `decision.actions` (+ declare `eligible`, − `TapForMana`); opt-in prop on `$viewer/ZoneHand`, `$viewer/ZoneBattlefield`, `SeatCard`; green ring. |
| 2 | Three identical "Play Plains" buttons. | `playRows` in `ActionBar`, folded by `(kind, label)` → "Play Plains ×3". |
| 3 | Feed lines "PreCombatMain — PreCombatMain", "Combat — BeginningOfCombat"; stop strip "the PreCombatMain phase moved on"; heading "DECLAREATTACKERS". | NEW `lib/names.js` (`stepName`, `phaseName`, `decisionKindName`, `eventKindName`, `stepChangedText`) used by `EventFeed`, `stores.js`, `ActionBar`. |
| 4 | Full-width red "COMBAT 0 attackers · No attackers declared" box for every combat phase. | `combatToShow` — box only when `attackers.length > 0`. |
| 5 | Declare-attackers needs a click on "Declare attackers (1 eligible)" before the picker. | Auto-open once per `seq` when exactly one declare action; Back still closes and stays closed. |
| 6 | After pass-until runs the feed stopped following; "↓ jump to latest" stuck. | Async scroll-event race; `autoScrollTop` guard in `EventFeed`. Reproduced only under fast appends — a slow probe passed, which is why it was not found by eye. |
| 7 | Eight lines per priority round (has priority / passes ×4). | Fold `PriorityGiven` into the adjacent same-player `PriorityPassed`. Display only. |
| 8 | "STOPPED — your turn began" strip stays up after the human has acted on that turn. | `act()` clears a finished run. |
| 9 | Chip names truncate at 11 chars ("Dusk Legion…", "Marble Diam…"); a land stack's "×2" fell off; combat row's attacker name truncated. | Widths in `$viewer/ZoneBattlefield` (100→150, land 70→110, name 90→140) and `$viewer/CombatView` (100→180). |
| 10 | "Lands: 1" on the seat card reads as a land count. | `$viewer/PlayerPanel`: "Land drops: 1". |

## `$viewer` edits, stated precisely

`ZoneHand.svelte`, `ZoneBattlefield.svelte` (prop + styles + widths), `CombatView.svelte`
(one width), `PlayerPanel.svelte` (one label). All additive or cosmetic; the replay viewer
builds (`npm run build` in `tools/replay-viewer/frontend`, 2026-10-09) and passes no
`offeredIds`, so its chips never carry the `offered` class.

## Not done, and why

- **Mana cost on hand chips.** `CardInZoneView` has no `mana_cost`; adding one is a
  view-model change (`crates/view-model`), out of a frontend-only task. Worth a follow-up:
  it is the single most useful fact a hand chip could show.
- **Finer pass-until modes** ("until Bot-3's end step") — limitation 21 stands.
- **Component tests** — limitation 17 stands; `verify.py` is a probe, not a harness. Its
  one false negative: `.decision-kind` is CSS-uppercased, so `inner_text()` reads
  "PRIORITY"; the screenshot shows the humanised label.
- Concede's disabled reason wraps awkwardly in the header at 1600px; left.

## Gates

`cargo test -p play-server` 130 passed (the frontend source gates walk the edited
`$viewer` files); `npm run build` both frontends; `cargo clippy` / `cargo fmt --check` /
`tools/check-defs-fmt.sh` unaffected (no Rust or defs touched) but run at `/done`.
