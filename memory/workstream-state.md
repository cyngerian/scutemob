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
| W6: Primitive + Card Authoring | — | available | — | v4 queue CLOSED at rank 21 (PB-DX57 `scutemob-236` last). Course-correction coordinator batch: CC-1/2/3/4 DONE 2026-09-05; CC-17 → CC-15 → CC-14 remain, then CC-5 (owner). History: `CHANGELOG.md` |

## Last Handoff (oversight session, 2026-09-05) — course correction; DX57 collected
## Last Handoff (coordinator session, 2026-09-05) — CC-1..CC-4 shipped inline

**Date**: 2026-09-05 (coordinator session, self-assigned tasks, no dispatch)
**Workstream**: course correction (`docs/course-correction-2026-09.md` §9.2 coordinator batch)
**Task**: `scutemob-237` (`4815467c`), `scutemob-238` (`1a4a9cb5`), `scutemob-239` (`8d59044e`),
`scutemob-240` (`87ac78c7`) — all done and merged; entries in `CHANGELOG.md`.

**Completed**:
- CC-1: CLAUDE.md 5,281 → 244 lines; this file 8,438 → under 60; three verbatim archives
  (diff-verified) under `memory/archive/*-2026-09-05.md`; `CHANGELOG.md` seeded.
- CC-2: 53 version-literal assertions gone (33 tests deleted, 11 in place, 4 renamed); suite
  5,363 → 5,330 / 0 / 6 with the name diff reconciled; SR-8 rule written; A5 gates untouched.
- CC-3: skills/agents repointed to CHANGELOG + notes file; `/eot` 250-line guard enforced;
  `/dispatch` inlined + brief step + Monitor recipe; reviewers have read-only Bash;
  `/review` ranks; `/author-wave --cards` added for CC-13.
- CC-4: change-class table verbatim in `memory/conventions.md`; brief cites it by section.

**Not done / deferred**:
- CC-17 (`scutemob-254`), CC-15 (`scutemob-252`), CC-14 (`scutemob-251`) — the rest of the
  coordinator batch. CC-5 decklists still need the owner.

**Next session candidates** (highest-yield first):
1. CC-17 (doc-only, CC-15's module doc must cite it) → CC-15 (simulator ratchet, the one Rust
   item) → CC-14 (move dormant agents/skills; `/dispatch` already no longer needs `/spawn`).
2. CC-5 with the owner, which unblocks CC-6/7/8.

**Operator-delta line** (what can a player observe now that they could not at the last
handoff?): nothing — four housekeeping batches, 0 engine lines. Second empty entry in a row:
per doc §5.2 a pod-facing item (CC-9 hot-seat) goes to the front once the batch closes.

**Hazards** (carrying forward):
- The bare `cargo clippy -- -D warnings` skips test targets; CC-2's 38 unused-import errors
  were invisible to it. Only the CI bar (`--workspace --all-targets`) counts.
- ESM's merge guard requires four CLAUDE.md headings to survive any rewrite (Worker Detection,
  Workflow Rules, Required Attestations, Project Info); the TUI parses three Current State keys.
- `.claude/docs.yaml` triggers on `crates/engine/tests/**` fire for any test edit; CC-2's were
  dismissed (sentinel deletion changes no corner-case or stress coverage).
- Every dispatch still needs owner approval; the v4 queue is closed.

**Commit prefix used**: `scutemob-N:` / `merge:` / `chore:`
