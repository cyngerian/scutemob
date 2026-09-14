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
| W6: Primitive + Card Authoring | — | available | — | v4 queue CLOSED at rank 21. CC batch COMPLETE; LL-3 (`-257`, `9a560313`) and LL-1 (`-255`, `01ce7e2c`) collected. STOP: CC-9 (`-245`) awaits owner approval; LL-2 (`-256`) after. LL-4 (`-258`) blocked on CC-5 (owner). History: `CHANGELOG.md` |

## Last Handoff (coordinator session, 2026-09-05 evening) — landscape assessment; LL-3 + LL-1 collected

**Date**: 2026-09-05 (coordinator; `/collect scutemob-257` merge `9a560313`)
**Workstream**: landscape lessons (`docs/mtg-engine-landscape-assessment.md`, commit `9677fa0c`)

**Completed**: assessment of phase.rs / Manabrew / XMage / Forge vs scutemob (survey §6 tasks);
LL-1..LL-4 filed (`scutemob-255..258`); LL-3 and LL-1 dispatched, reviewed, collected. LL-3 — new-variant
checklist (`memory/checklists/new-effect-variant.md`, 31 sites, two the brief missed:
`state/stack_registry.rs`, `rules/mana.rs::is_mana_producing_effect`), "## Landscape rules" in
`memory/conventions.md`, most-prohibited-pattern line in CLAUDE.md (now **249 lines** — one under
the `/eot` guard; the next CLAUDE.md edit must shave), exit-3 rule in `/dispatch`.

LL-1 (`01ce7e2c`, class rows 1+2+4): SR-39 (every def names `completeness:`; 964 swept, ZERO
defaulted; tally 1,143 / 413 / 147 / 100); `DestroyPermanent` records the departed (controller,
owner) in `EffectContext` so `ControllerOf` resolves under CR 608.2h — 11 defs repaired (not 6:
`natures_claim` and `boseiju_who_endures` were EXPLICIT-Complete with a dead clause, the class SR-39
cannot see). Suite 5,333 → 5,346 / 0 / 6; golden 208 → 209; wire 44/85 unchanged (predicted, met).
One relaxation on record: `MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED` 1 → 2, ablation-attributed to the
corpus re-deal (`OOS-CARDS2-3`), with a written stop. Seed candidate named, not filed: three
exile-shape `ControllerOf` defs (`swords_to_plowshares`, `reality_shift`, `path_to_exile`) and
`DestroyAll` recording no departures.

**Not done / deferred**: seed NOT filed for the exile-shape `ControllerOf` defs + `DestroyAll`
departures (named in `ll-1-execution-notes.md`; grep the registry before filing). `chord_of_calling`
shows as marker drift in `authoring-status.md` (prose TODO, false positive — reword the comment).

**Next session candidates**: (1) CC-9 hot-seat (`scutemob-245`) — pod-facing, AWAITS OWNER
APPROVAL; (2) LL-2 CR-cite verifier (`-256`) after it; (3) CC-5 decklists (owner) → LL-4 (`-258`),
CC-6/7/8. The portfolio agent's critique (process ahead of pod-facing work) was accepted.

**Operator-delta line**: Beast Within, Generous Gift, Nature's Claim and eight siblings now give
the token / life to the destroyed permanent's controller at a four-player table. First non-empty
entry since the course correction. CC-9 is next and awaits owner approval.

**Hazards**: `esm worktree check` flags `.claude/skills/dispatch/SKILL.md` as provisioned damage
whenever a task edits it on purpose — inspect, then `--allow-provisioned-changes`. `esm doctor`
`missing: end, spawn` is expected. Landscape clones live at `~/projects/scutemob-landscape/`.
A defs-only fix for "its controller …" after a zone move CANNOT work (dead id, CR 400.7; LKI store
is sparse by design, SR-24) — see `memory/gotchas-rules.md`. CLAUDE.md is 249/250.

**Note (2026-09-14)**: the 2026-09-04/05 chain-coordinator session (PB-DX52..DX57) was resumed and closed; it verified its final collect (`scutemob-236`, `cb6980f2`) had been done by a later session and made no project changes.
**Commit prefix used**: `scutemob-N:` / `merge:` / `chore:`
