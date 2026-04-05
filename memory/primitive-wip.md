# Primitive WIP: PB-C — Extra Turns

batch: PB-C
title: Extra turns
cards_affected: 4
started: 2026-04-05
phase: closed

## Review
findings: 3 (HIGH: 0, MEDIUM: 1, LOW: 2)
verdict: needs-fix
review_file: memory/primitives/pb-review-C.md
plan_file: memory/primitives/pb-plan-C.md

## Cards
1. Nexus of Fate — "Take an extra turn after this one." + shuffle into library
2. Temporal Trespass — Delve + "Take an extra turn after this one. Exile ~."
3. Temporal Mastery — Miracle + "Take an extra turn after this one. Exile ~."
4. Teferi, Master of Time — existing def stub, -10 loyalty: "Take two extra turns after this one."

Also fixes TODO in: emrakul_the_promised_end.rs (extra turn part only; gain-control remains blocked)

## Existing Infrastructure
- `extra_turns: Vector<PlayerId>` on TurnState (LIFO queue, CR 614.10)
- `advance_turn()` in turn_structure.rs already pops from extra_turns
- `GameEvent::ExtraTurnAdded { player }` already exists
- `GiftType::ExtraTurn` variant exists (Gift mechanic, deferred)
- 4 existing tests in extra_turns.rs verify LIFO, designated player, resumption, multi-stack

## Deferred from Prior PBs
- Cavern of Souls "can't be countered" (CounterRestriction) — unrelated
- activated_ability_cost_reductions index off-by-one — unrelated

## Step Checklist
- [x] 1. Engine changes (Effect::ExtraTurn variant, dispatch in effects/mod.rs, hash in state/hash.rs, GiftType::ExtraTurn wired in resolution.rs)
  - Added Effect::ExtraTurn { player: PlayerTarget, count: EffectAmount } to card_definition.rs (after SolveCase)
  - Dispatch in effects/mod.rs: resolve_player_target_list + resolve_amount, pushes to extra_turns, emits ExtraTurnAdded
  - Hash in state/hash.rs: discriminant 76
  - GiftType::ExtraTurn wired in resolution.rs execute_gift_effect()
  - Added self_exile_on_resolution and self_shuffle_on_resolution flags to CardDefinition
  - Resolution.rs destination selection checks both flags before flashback/buyback
  - Python script updated 131 card defs + 14 test files with new explicit-constructor fields
- [x] 2. Card definition fixes (Teferi -10, Emrakul TODO comment)
  - teferi_master_of_time.rs: Added LoyaltyAbility for -10 with Effect::ExtraTurn Fixed(2), updated TODO comments
  - emrakul_the_promised_end.rs: Updated TODO comment to note ExtraTurn now expressible via PB-C
- [x] 3. New card definitions (Nexus of Fate, Temporal Trespass, Temporal Mastery)
  - nexus_of_fate.rs: instant, ExtraTurn Fixed(1), self_shuffle_on_resolution: true
  - temporal_trespass.rs: sorcery, Delve keyword, ExtraTurn Fixed(1), self_exile_on_resolution: true
  - temporal_mastery.rs: sorcery, Miracle {1}{U}, ExtraTurn Fixed(1), self_exile_on_resolution: true
- [x] 4. Unit tests (6 new tests in extra_turns.rs, 10 total)
  - test_effect_extra_turn_basic: CR 500.7 — ExtraTurn pushes to queue, emits event
  - test_effect_extra_turn_two_turns: CR 500.7 — count=2 adds two turns
  - test_gift_extra_turn: CR 702.174g — GiftType::ExtraTurn gives opponent extra turn
  - test_self_exile_on_resolution: self_exile flag — card goes to exile not graveyard
  - test_self_shuffle_on_resolution: self_shuffle flag — card goes to library not graveyard
  - test_effect_extra_turn_resolves_and_taken: end-to-end extra turn taken
- [x] 5. Workspace build verification
  - cargo test --all: 2480 passing, 0 failing
  - cargo clippy -- -D warnings: clean
  - cargo build --workspace: clean
  - cargo fmt --check: clean
