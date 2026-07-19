# Ability WIP — IDLE (no ability in progress)

<!-- last_updated: 2026-07-18 -->

**Status: IDLE.** There is no active `/implement-ability` run. The W1 ability campaign is
closed (~199 validated; 42/42 P1, 17/17 P2, 40/40 P3, 95/95 P4 — see CLAUDE.md → Current
State → Abilities). This file is the live single-slot scratchpad the ability pipeline
reuses; when the next ability starts, `/implement-ability` overwrites it.

Do **not** read the previous occupant's checklist below as current work — it is a completed
March 2026 batch, kept only as a pointer.

## Last occupant (COMPLETE — historical)

- **PB-10 Return From Zone Effects** — started 2026-03-14, phase `complete`.
- Full record: `memory/abilities/ability-plan-pb10-return-from-zone.md`.
- Shipped: 2 `TargetRequirement` variants (TargetCardInYourGraveyard / TargetCardInGraveyard),
  `TargetFilter::has_subtypes`, casting.rs graveyard-zone validation, 10 tests, 9 card-def
  fixes. Follow-on DSL gaps it surfaced were folded into the later EF/OS queues.
