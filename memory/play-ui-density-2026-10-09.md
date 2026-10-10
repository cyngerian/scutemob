# Play-server UI density pass — `scutemob-261` (2026-10-09)

Self-assigned (`/task`), frontend only, class "tooling". Branch `feat/play-ui-density`.
Owner, after seeing `scutemob-260` on the 1080p monitor: *"this still sucks. the
battlefield should be the largest section and its tiny on my 1080. it should be specced
for the 1080, it looks better on the 2k, but lots of text is too big and takes up so much
space, especially the life total"*.

## Measured (headless, four seats, turn 9, combat box showing)

| Band | Before (1080p) | After (1080p) | After (1440p) |
|---|---|---|---|
| header | 36 | 28 | 28 |
| phase bar | 70 | 26 | 26 |
| seat row | 145 | 69 | 69 |
| action bar | ~70 (+ hint line) | 29 | 29 |
| header..action bar | ~41% | **23%** | 17% |
| `.body` (board + feed) | ~45% | **71%** | 79% |

Probe: scratchpad `measure.py` (reads `getBoundingClientRect` per band at 1920×1080 and
2560×1440 on the :3041 probe server).

## Changes

- **`PhaseStrip.svelte`** (NEW, play-local): the turn structure as one row — turn, active
  player, priority, and the twelve step pips grouped by phase. Replaces the viewer's
  two-line `PhaseIndicator` on this surface; the viewer's component is untouched.
- **`SeatCard.svelte`** rewritten: no longer wraps `$viewer/PlayerPanel`. Two rows (name +
  badges + life at 1rem right-aligned; `H n L n GY n drop n` + pool pips + poison +
  commander damage) and a command-chip row with the drawer toggle. `PlayerPanel.svelte`
  is unmodified and still the replay viewer's.
- **`$viewer/ZoneBattlefield`** gains `compact` (opt-in): hides its own "Battlefield · N
  permanents" header (the cell label already says who and now how many) and tightens
  padding. `PlayBoard` passes it and shows the count in its label.
- **`PlayBoard`**: the grid takes the whole board region (`flex: 1`, `grid-auto-rows:
  minmax(0, 1fr)`, cells stretch) so four boards tile the space rather than sitting as
  content-sized boxes over a dead area.
- **`ActionBar`**: heading is one line (seq moved to a `title`), the "space = pass …" hint
  line is gone (the pass button carries its `space` badge; every picker has Back), tighter
  paddings and 0.72rem buttons.
- **`EventFeed`**: text and kind tag on one line; the player tag is dropped (the sentence
  names the player).
- **`PlayApp`**: header, dock, stack dock, hand bar and feed pane shrunk; `top-dock` cap
  62vh → 48vh; feed pane 22rem → 19rem.
- **`app.css`**: base 14px → 13px. Everything is rem-sized, so both monitors scale.

## Not done

- No per-resolution breakpoints beyond the existing 60rem one-column collapse; the same
  rem sizes serve both monitors, which is what the owner asked for ("specced for the
  1080").
- The combat box (shown only with attackers since `-260`) is still ~80px; it is the one
  band left that could fold into the stack dock's header.
