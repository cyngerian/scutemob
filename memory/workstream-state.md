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

## Last Handoff (primary session, 2026-10-12) — play surface v3 C1 collected: the alpha table is live on `?v3`

**Date**: 2026-10-12 (primary; dispatched `scutemob-266`, collected; merge `5025a70a`, pushed)
**Workstream**: play client (`tools/play-server`), no engine / wire change (PROTOCOL 44 / HASH 85)

**Completed**: data refresh first (`e5382c5a`: CR 2026-09-25 + Scryfall 2026-10-12; SR-37 fixture
4 wording tweaks; `cites` 33 / 307 dangling — unchanged baseline). Then C1: mockup 7 built as the
third surface behind `?v3` (`frontend/src/lib/v3/`, 25 files) incl. the three late brief rules
(mirrored cells, greyed focused cell + lift, peek); `?card-window` pop-out; v2 `PromptPanel` hosted
unmodified in the decision panel; NEW `probes/v3sweep.py` + `v3interact.py` (0 scroll / 0 errors,
35/35); three source gates with executed defeats; tests 5,352 → 5,355. Opus review: 1 HIGH + 3
MEDIUM fixed in-cycle, 4 LOW logged. Full record: `memory/play-v3-c1-2026-10-12.md`; CHANGELOG entry.
Brief rule 7 amended (card window fed by a `storage` event, not `BroadcastChannel`).

**Not done / deferred**: C2 (`-267`) arrows, card context menus, blocker arrows, auto-pass panel,
summoning-sick outline, drag-rearrange; C3 (`-268`) server gaps (per-player exile, commander tax,
image proxy — Scryfall rate-limits the probes, pass-until-step), game-over run, owner playtest.
Portfolio session's suggestion recorded on `-268`: a playtest friction log (hotkey → note + state
snapshot). Owner to rule: ride with C3 or its own small task before the first playtest.
`cockatrice*.png` at the repo root are still the owner's untracked reference shots (6 MB).

**Next session candidates**: (1) owner looks at `?v3` on 3040 (the six-seat side-cell card size
~34px is a LOW worth an owner look before C2); (2) `/dispatch` C2 (`-267`) — single worker, change
class row 3 (+ row 4 per gate); (3) C3 (`-268`) incl. the friction log if ruled in. CC-9 hot-seat
(`-245`) still awaits owner approval. Every dispatch needs explicit owner approval.

**Operator-delta line**: the alpha table is playable through priority on `?v3` (pass, pass-until,
card-item actions, every existing picker); targeting is still by decision box until C2.

**Hazards**: one play-server per game — probes on 3041/3042, never the owner's 3040; Scryfall 429s
on back-to-back probes (chips are the fallback; C3 proxy fixes it); the frontend gate bans
`.postMessage(` — use the storage-event transport; v3 page root is 13px (pickers `zoom: 1.23`).
CLAUDE.md is 249/250.

**Commit prefix used**: `scutemob-266:` / `merge:` / `chore:`
