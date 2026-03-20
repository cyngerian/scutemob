# Primitive WIP: PB-21 -- Fight & Bite

batch: PB-21
title: Fight & Bite
cards_affected: 5+
started: 2026-03-19
phase: closed
plan_file: memory/primitives/pb-plan-21.md

## Deferred from Prior PBs
none

## Review
findings: 6 (HIGH: 0, MEDIUM: 3, LOW: 3)
verdict: needs-fix → fixed
review_file: memory/primitives/pb-review-21.md

## Fix Phase (2026-03-19)
- [x] Finding 1 (MEDIUM): is_creature_on_battlefield now calls calculate_characteristics(state, id) via layer system — effects/mod.rs:3835-3852
- [x] Finding 5 (MEDIUM): bridgeworks_battle.rs: added back_face: Some(CardFace { ... }) for Tanglespan Bridgeworks (Land, enters tapped, {T}: Add {G}) — bridgeworks_battle.rs
- [x] Finding 2 (LOW): Added test_fight_target_not_creature in fight_bite.rs — uses SetTypeLine ContinuousEffect to make Fighter B an Enchantment before resolution; verifies CR 701.14b all-or-nothing
- [x] Finding 3 (LOW): Added test_bite_negative_power in fight_bite.rs — uses ModifyPower(-3) on a 2/2 for net -1 power; verifies clamped to 0 damage
- [ ] Finding 4 (MEDIUM): "Up to one" optional targeting — DSL gap, TODO remains in bridgeworks_battle.rs
- [ ] Finding 6 (LOW): "Another target" filter — DSL gap, documented in brash_taunter.rs
- Post-fix verification: 14/14 fight_bite tests pass; cargo test --all clean (0 failures); cargo clippy -- -D warnings clean; cargo build --workspace clean; cargo fmt --check clean

## Step Checklist
- [x] 1. Engine changes (Effect::Fight, Effect::Bite, dispatch in effects/mod.rs)
  - Added Effect::Fight { attacker, defender } and Effect::Bite { source, target } to card_definition.rs (after Effect::Meld)
  - Added deal_creature_power_damage() + is_creature_on_battlefield() + get_creature_power() helpers in effects/mod.rs
  - Added Effect::Fight and Effect::Bite dispatch arms in execute_effect_inner
  - Added hash discriminants 58 (Fight) and 59 (Bite) in state/hash.rs
  - No exhaustive match changes needed in replay-viewer or TUI (Effect is not matched there)
- [x] 2. Card definition fixes (existing cards with fight/bite TODOs)
  - brash_taunter.rs: Added fight activated ability {2}{R},{T} with Cost::Sequence([Mana, Tap])
  - bridgeworks_battle.rs: Added full Spell with PT boost + Fight effect (mandatory 2nd target, "up to one" TODO)
  - ram_through.rs: Added Bite spell with TargetController::You/Opponent (trample overflow TODO remains)
  - frontier_siege.rs: Updated TODO to clarify Fight is now available, modal ETB is the blocking gap
- [x] 3. New card definitions (if any): N/A (plan specified none)
- [x] 4. Unit tests: 12 tests in crates/engine/tests/fight_bite.rs — all pass
  - test_fight_basic, test_fight_one_dies, test_fight_both_die, test_fight_self
  - test_fight_creature_left_battlefield, test_fight_non_combat_damage
  - test_fight_deathtouch, test_fight_lifelink
  - test_bite_basic, test_bite_zero_power, test_bite_lifelink
  - test_bite_source_creature_killed_before_resolution
- [x] 5. Workspace build verification
  - cargo test --all: 2204 tests pass (was 2184 before PB-21)
  - cargo clippy -- -D warnings: clean
  - cargo build --workspace: clean
  - cargo fmt --check: clean
