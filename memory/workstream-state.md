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
| W6: Primitive + Card Authoring | `scutemob-255` | dispatching | 2026-09-05 | v4 queue CLOSED at rank 21. CC batch COMPLETE; LL-3 (`-257`) collected `9a560313`. Approved chain: LL-1 (`-255`) → STOP, ask for CC-9 (`-245`) → LL-2 (`-256`). LL-4 (`-258`) blocked on CC-5 (owner). History: `CHANGELOG.md` |

## Last Handoff (coordinator session, 2026-09-05 evening) — landscape assessment; LL-3 collected

**Date**: 2026-09-05 (coordinator; `/collect scutemob-257` merge `9a560313`)
**Workstream**: landscape lessons (`docs/mtg-engine-landscape-assessment.md`, commit `9677fa0c`)

**Completed**: assessment of phase.rs / Manabrew / XMage / Forge vs scutemob (survey §6 tasks);
LL-1..LL-4 filed (`scutemob-255..258`); LL-3 dispatched, reviewed, collected — new-variant
checklist (`memory/checklists/new-effect-variant.md`, 31 sites, two the brief missed:
`state/stack_registry.rs`, `rules/mana.rs::is_mana_producing_effect`), "## Landscape rules" in
`memory/conventions.md`, most-prohibited-pattern line in CLAUDE.md (now **249 lines** — one under
the `/eot` guard; the next CLAUDE.md edit must shave), exit-3 rule in `/dispatch`.

**Finding worth a player's attention**: `beast_within.rs` and `generous_gift.rs` are deck-legal and
give the token to the CASTER (`TokenSpec.recipient` defaulted); golden `tokens/002` retired with a
stale reason. LL-1 fixes it and makes `completeness:` mandatory (964 defs default today).

**Next**: LL-1 (`scutemob-255`, approved) → then STOP and ask the owner for CC-9 (`-245`) before
LL-2 (`-256`) — the portfolio agent's critique (process ahead of pod-facing work) was accepted.
LL-4 (`-258`) and everything pod-facing wait on CC-5 (owner: six decklists).

**Operator-delta line**: nothing yet — LL-1 will be the first (Beast Within right at a 4-player
table). Fourth empty entry; CC-9 is next after LL-1 by agreement.

**Hazards**: `esm worktree check` flags `.claude/skills/dispatch/SKILL.md` as provisioned damage
whenever a task edits it on purpose — inspect, then `--allow-provisioned-changes`. `esm doctor`
`missing: end, spawn` is expected. Landscape clones live at `~/projects/scutemob-landscape/`.
