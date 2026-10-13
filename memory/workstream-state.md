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

## Last Handoff (primary session, 2026-10-12) — v3 C1–C3 collected; the alpha table awaits the owner playtest

**Date**: 2026-10-12 (primary; dispatched + collected `scutemob-266` `5025a70a`, `-267` `2436e4e7`, `-268` `01f4ac59`, `-270` `fc9eb147`; pushed)
**Workstream**: play client (`tools/play-server` + `crates/view-model`), no engine / wire change (PROTOCOL 44 / HASH 85)

**Completed**: data refresh (`e5382c5a`). C1 table, C2 interactions (see archive). C3: view-model DTOs
(`exile_by_owner`, `commander_tax`, `stack[].target_refs`, `summary.game_id`, `pass_until` server param;
each with a redaction case), the LOCAL IMAGE STORE (`src/images.rs`; `GET /api/card-image` serves only;
filled at deal or `--prefetch-images`; `SCUTEMOB_IMAGE_STORE`; v3 makes 0 Scryfall requests — gate +
defeat), `?v3` wired to all of it, `probes/v3game.py` seed 3 to game over at both viewports (turn 55,
891 decisions, 0 errors), `verify.py` flow re-expressed (69 checks), interact 122 → 280. Reviews C3:
0H + 3M fixed. Tests 5,356 → 5,407. Notes `memory/play-v3-c3-2026-10-12.md`. Guide
`docs/play-v3-playtest-guide.md` + template `memory/play-v3-playtest-findings.md`. Owner idea filed:
C4 classic layout `scutemob-269` (you bottom-left; every board its own hand + piles) — brief §4 row.
Owner playtest STARTED (seed 3 on 3040, old C3 binary): findings 1–5 in `memory/play-v3-playtest-findings.md`
(cold-store blocker → `-270` SHIPPED: CDN by Scryfall id from `cards.sqlite`, 238 cards in 72 s, retry route;
sticky chips until reload, no-game-at-start, attachments not drawn, decision-box pin → playtest batch).

**Not done**: `-268` criterion 8617's owner half — the playtest, the recorded verdict, the filed tasks;
task stays `in_review` until then. Three owner calls pending (C3 notes): image-store 200/404 reveals
union-of-decks membership (fine for one human; close before CC-9 hot-seat by prefetching all pod decks);
combat card 60vh inner scroll (only scrolling box on the surface); playtest friction-log hotkey.
`cockatrice*.png` (3) at the repo root → `mockups/reference/` when C4 dispatches.

**Next session candidates**: (1) owner plays `?v3` per the guide (restart the server: the running
3040 binary predates C3); (2) record verdict in the findings file, file tasks, transition `-268` done;
(3) `/dispatch` C4 (`-269`); (4) CC-9 hot-seat (`-245`) still awaits owner approval. Every dispatch
needs explicit owner approval.

**Operator-delta line**: the alpha table is feature-complete for a one-human table with bots; images
local; one seeded game runs to game over headless. Pod play still needs decklists (CC-5) and hot-seat (CC-9).

**Hazards**: the old 3040 server + a rebuilt dist = chips and missing fields — restart the server
first; probes on 3041+; first deal on a seed takes minutes to fill the store (chips meanwhile);
`.postMessage(` banned; v3 root 13px. CLAUDE.md is 249/250.

**Commit prefix used**: `scutemob-266/267/268:` / `merge:` / `chore:`
