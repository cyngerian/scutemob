# Primitive WIP: PB-AC1 — Counter / Untap / Once-per-turn

batch: PB-AC1
title: Counter / untap / once-per-turn primitives
cards_affected: ~22 (discounted; real roster to be identified from oracle text)
started: 2026-07-07
phase: closed
plan_file: memory/primitives/pb-plan-AC1.md

## Task reference
- ESM task: scutemob-43
- Branch: feat/pb-ac1-counter-untap-once-per-turn-primitives
- Acceptance criteria:
  - 4154: Engine primitives implemented (UntapAll, WheneverPermanentUntaps +
    WhenCounterPlaced triggers, once-per-turn limiter, doesn't-untap static) each
    with tests citing CR sections
  - 4155: Review pass complete; all HIGH/MEDIUM findings fixed
  - 4156: Backfill complete; all unblocked cards re-authored, TODO/ENGINE-BLOCKED
    markers removed, reviewed by card-batch-reviewer
  - 4157: All gates green; authoring-report rerun and coverage delta posted

## Scope (from campaign-plan-2026-05-16.md §2)
Primitives to add:
- `Effect::UntapAll { filter }` — untap all permanents matching a filter (CR 701.20/701.21)
- `TriggerCondition::WheneverPermanentUntaps` (CR 603.2)
- `TriggerCondition::WhenCounterPlaced` (CR 603.2, 122)
- Generic `once_per_turn` limiter on triggered abilities (e.g. Morbid Opportunist
  "triggers only once each turn") (CR 603.2)
- "Doesn't untap during untap step" static (CR 502.4 / 702.x)

CR refs (701.20, 701.21, 603.2) are ADVISORY — verify against the CR via the
mtg-rules MCP. Card rosters in the plan are advisory; identify the real roster
from oracle text (feedback_oversight_primitive_category_not_cards).

## Hazards (from task description)
1. Verify KW/AbilDef/SOK discriminant chain from current code before adding variants.
2. Exhaustive matches in tools/tui/src/play/panels/stack_view.rs AND
   tools/replay-viewer/src/view_model.rs must gain arms for every new enum variant —
   verify with `cargo build --workspace`.
3. Do NOT commit phantom `.claude/skills/*/SKILL.md` deletions in the worktree.

## Deferred from Prior PBs
none applicable

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch) — Effect::UntapAll (disc 87),
  TriggerCondition::WheneverPermanentUntaps (disc 42) + WhenCounterPlaced (disc 43),
  runtime TriggerEvent::AnyPermanentUntaps (disc 45) + CounterPlaced (disc 46),
  KeywordAbility::DoesNotUntap (disc 162), once_per_turn limiter (flush_pending_triggers
  gate + turn-reset sweep), DoesNotUntap untap-step enforcement (layer-resolved).
  HASH_SCHEMA_VERSION 27->28 + all 20 scattered parity-sentinel tests updated.
- [x] 2. Card definition fixes / backfill — ONLY the 4 integration cards done this
  phase (Morbid Opportunist, Mesmeric Orb, Goblin Sharpshooter, Sharktocrab); full
  backfill roster (partial-clean + blocked cards from the plan) is OUT OF SCOPE for
  this engine-primitives phase per task instructions — deferred to the backfill phase.
- [x] 3. New card definitions (if any) — none (all 4 integration cards pre-existed
  as TODO/ENGINE-BLOCKED stubs; re-authored in place).
- [x] 4. Unit tests — crates/engine/tests/pb_ac1_untap_counter.rs, 20 tests, all
  passing (full plan test list covered + a direct-construction wiring test + a
  matches_filter sanity test).
- [x] 5. Workspace build verification — `cargo build --workspace`, `cargo test --all`
  (2893 passed, 0 failed), `cargo clippy --workspace --all-targets -- -D warnings`
  (clean), `cargo fmt --check` (clean).

## Fix phase complete (2026-07-07)
- All HIGH/MEDIUM findings from `memory/primitives/pb-review-AC1.md` resolved:
  1. HIGH — `state/hash.rs`: `GameObject::triggered_abilities_fired_this_turn` now hashed
     (end of `HashInto for GameObject`, after `skip_untap_steps`).
  2. MEDIUM — `state/hash.rs`: `TriggeredAbilityDef::hash_into` now hashes `once_per_turn`,
     `counter_filter`, `counter_on_self`.
  3. MEDIUM — CR 122.6 enters-with-counters gap tracked as new issue `MR-AC1-01` (LOW, OPEN)
     in `docs/mtg-engine-milestone-reviews.md`; test comment in
     `crates/engine/tests/pb_ac1_untap_counter.rs` updated to cite it.
- LOW findings 4-6 left open (non-blocking per task instructions).
- `cargo build --workspace`, `cargo test -p mtg-engine` (all pass, HASH_SCHEMA_VERSION
  still 28), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`
  all clean.

## Known residual (flagged, not fixed in this phase)
- `test-data/generated-scripts/baseline/105_sharktocrab_adapt.json` demoted from
  `approved` to `pending_review`: Sharktocrab's newly-authored WhenCounterPlaced
  ability (tap target opponent creature + PreventNextUntap) now fires during the
  script's Phase 1 Adapt resolution and lingers on the stack into Phase 2, breaking
  the script's `zones.stack.count` assertions. Needs script regeneration (extra
  priority_round + stack_resolve steps) before re-approval — tracked in the script's
  `generation_notes`.
