# Workstream State

> Coordination file for sessions: the Active Claims table plus the LAST handoff only, capped at
> 60 lines (`docs/course-correction-2026-09.md` §3.1 item 3). Older handoffs are rotated verbatim
> to `memory/archive/workstream-state-<date>.md` (latest: `workstream-state-2026-09-05.md`).
> Per-batch narrative lives in `CHANGELOG.md` + `memory/primitives/pb-<id>-execution-notes.md`.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | All abilities done (B16 Dungeon + Ring last) |
| W2: TUI & Simulator | — | available | — | Phase 1 done; hardening pending |
| W3: LOW Remediation | — | available | — | LOW Sweep COMPLETE 2026-05-16; 6 LOWs remain, deferred |
| W4: M10 Networking | — | not-started | — | P3 of the course correction; after hot-seat |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6 |
| W6: Primitive + Card Authoring | — | available | — | v4 queue CLOSED at rank 21. Course-correction coordinator batch COMPLETE 2026-09-05 (CC-1/2/3/4/14/15/17). Next: CC-5 decklists (owner), then CC-6/7/8; CC-9 hot-seat needs a dispatch with owner approval. History: `CHANGELOG.md` |

## Last Handoff (oversight session, 2026-09-05) — course correction; DX57 collected
## Last Handoff (coordinator session, 2026-09-05) — coordinator batch COMPLETE, CC-1..4 + 17/15/14

**Date**: 2026-09-05 (coordinator session, all seven tasks self-assigned inline, no dispatch)
**Workstream**: course correction (`docs/course-correction-2026-09.md` §9.2 coordinator batch)
**Task**: `scutemob-237/238/239/240` (morning) and `scutemob-254` (`d2be14f9`), `-252` (`cdaffd6e`),
`-251` (`294072ad`) — all done and merged; one `CHANGELOG.md` entry each.

**Completed** (this half; the CC-1..4 half is in the previous rotation of this handoff):
- CC-17: pair-or-demote rule in `memory/conventions.md` (probe under the same revert or reason +
  seed ID; no sweep; re-key-after-defeat is the moment), cross-referenced from engine-invariants.
- CC-15: `crates/simulator/tests/cc15_raw_characteristics_ratchet.rs` — two-sided per-file pins
  + directory walk; five executed defeats; paired with the SR-38 probes. **Finding**: the
  addendum's grep-line "43" counted 3 comment mentions and missed 6 line-wrapped chains; the
  whitespace-blind count is 47 (28 in `legal_actions.rs`). Suite 5,330 → 5,333 / 0 / 6, 73 targets.
- CC-14: 8 agents → `.claude/agents-dormant/`, 5 skills → `.claude/skills-dormant/` (READMEs with
  the restore recipe); `start-work`/`end`/`spawn` deleted; Agents table == disk == dispatch roster.

**Not done / deferred**:
- Nothing left in the coordinator batch. CC-5 (six pod decklists) needs the owner; everything
  else in backlog is blocked on it or on a dispatch.

**Next session candidates** (highest-yield first):
1. CC-5 with the owner (`scutemob-241`) → unblocks CC-6/7/8 (`242/243/244`, dispatchable).
2. CC-9 hot-seat (`scutemob-245`) — pod-facing, needs owner approval to dispatch.

**Operator-delta line** (what can a player observe now that they could not at the last
handoff?): nothing — housekeeping only, 0 engine lines all day. THIRD empty entry in a row:
doc §5.2 says a pod-facing item goes to the front, and CC-9 is it.

**Hazards** (carrying forward):
- `esm doctor` now reports `missing: end, spawn` at every `/start` — expected (deleted by
  CC-14, both ESM-provisioned). Never `esm update` to clear it; it re-adds them.
- The eight dormant agents stay listed as `subagent_type` values until a session restart.
- `/implement-ability` depends on dormant agents (banner at its top); `/author-wave` has a new
  `--cards` mode for CC-13; `/dispatch` now writes `.esm/brief.md` and watches via Monitor.
- Every dispatch still needs owner approval; the v4 queue is closed.

**Commit prefix used**: `scutemob-N:` / `merge:` / `chore:`
