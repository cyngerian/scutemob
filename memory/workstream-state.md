# Workstream State

> Coordination file for sessions: the Active Claims table plus the LAST handoff only, capped at
> 60 lines (`docs/course-correction-2026-09.md` §3.1 item 3). Older handoffs are rotated verbatim
> to `memory/archive/workstream-state-<date>.md` (latest: `workstream-state-2026-10-12.md`).
> Per-batch narrative lives in `CHANGELOG.md` + `memory/primitives/pb-<id>-execution-notes.md`.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | All abilities done (B16 Dungeon + Ring last) |
| W2: TUI & Simulator | — | available | — | Phase 1 done; hardening pending |
| W3: LOW Remediation | — | available | — | LOW Sweep COMPLETE 2026-05-16; 6 LOWs remain, deferred |
| W4: M10 Networking | — | not-started | — | P3 of the course correction; after hot-seat |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6 |
| W6: Primitive + Card Authoring | — | available | — | v4 queue CLOSED at rank 21. CC batch + LL-3/LL-1 COMPLETE; data sync `scutemob-259` (`5ce2795a`) 2026-09-14. STOP: CC-9 (`-245`) awaits owner approval; LL-2 (`-256`) after (baseline: `data-freshness.py cites`). LL-4 (`-258`) blocked on CC-5 (owner). History: `CHANGELOG.md` |

## Last Handoff (primary session, 2026-10-12) — v3 C1 + C2 collected: the alpha table is interactive on `?v3`

**Date**: 2026-10-12 (primary; dispatched + collected `scutemob-266` (`5025a70a`) and `-267` (`2436e4e7`), pushed)
**Workstream**: play client (`tools/play-server`), no engine / wire change (PROTOCOL 44 / HASH 85)

**Completed**: data refresh (`e5382c5a`: CR 2026-09-25, Scryfall 2026-10-12; `cites` 33/307 unchanged).
C1: mockup 7 as `?v3` + `?card-window`, pickers hosted unmodified, probes `v3sweep.py` / `v3interact.py`,
3 gates. C2: arrow targeting (cards, opponent cells, seats), card right-click menu (Use / Target with /
counters), blocker arrows, drag-to-rearrange hand + battlefield (per-game client order), summoning-sick
outline, auto-pass panel (4 modes, 3 stops; `startPassUntil` takes a predicate, additive); ONE submission
path through the mounted pickers (gate + executed defeat); `v3scenario.py`; interact 35 → 122 checks.
Reviews: C1 1H+3M, C2 1H+1M fixed in-cycle. Tests 5,352 → 5,356. Notes `memory/play-v3-c{1,2}-2026-10-12.md`.
Owner ruling recorded (`memory/decisions.md`, brief §5, comment on `-268`): card images are downloaded
when a deck is added; play never calls Scryfall. Collect chore also fixed the `BlockerPicker` select
(numeric option vs String value — showed blank on every surface; C2 found it).

**Not done / deferred (C3, `-268`)**: image store + deck-time prefetch (replaces the lazy proxy),
per-player exile, commander tax, `GameSummary.game_id` (order key), `StackItemView` target ids (the
`targeted` outline), pass-until-step server param, revealed cards, seed-3 game to game-over,
`verify.py` on v3, owner playtest. Portfolio suggestion on `-268`: playtest friction log (owner to rule).
C2 LOW list in its notes (CR 509.1b double-block inherited from `BlockerPicker`; unexercised arrow paths).
`cockatrice*.png` (3 files) at the repo root are the owner's untracked reference shots.

**Next session candidates**: (1) owner plays `?v3` on 3040 (bundle rebuilt in place after each collect;
no server restart needed); (2) `/dispatch` C3 (`-268`) — rows 1/3/4: server DTO additions are a wire
change (predict the PROTOCOL bump), image store, game-over run, then the playtest; (3) CC-9 hot-seat
(`-245`) still awaits owner approval. Every dispatch needs explicit owner approval.

**Operator-delta line**: `?v3` is playable with arrows, menus, blocker arrows, rearranging and auto-pass;
images still come live from Scryfall until C3's store (chips on 429).

**Hazards**: probes on 3041/3042 only; `.postMessage(` banned by a gate (storage-event transport);
v3 root 13px (pickers `zoom: 1.23`); `reorder.js` swallows the click after a drag (arrow drop deferred
two timer turns); no game id in `GameSummary` yet. CLAUDE.md is 249/250.

**Commit prefix used**: `scutemob-266:` / `scutemob-267:` / `merge:` / `chore:`
