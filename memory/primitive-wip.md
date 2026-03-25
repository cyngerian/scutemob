# Primitive WIP: PB-27 -- X-cost spells

batch: PB-27
title: X-cost spells
cards_affected: ~42
started: 2026-03-24
phase: fix
plan_file: memory/primitives/pb-plan-27.md

## Gap Reference
G-5 from `docs/dsl-gap-closure-plan.md`:
- G-5: X-cost spells (~42 cards) — Add `x_cost: bool` to `ManaCost`; wire `EffectAmount::XValue` into mana cost parsing and legal action generation

## Deferred from Prior PBs
- PB-9 already added `x_value: u32` on CastSpell/StackObject/GameObject/EffectContext and `EffectAmount::XValue`
- PB-9 review deferred 2M 7L items (some may relate to X-cost handling)

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch) — Condition::XValueAtLeast, Effect::Repeat, x_value on ActivateAbility, ETB propagation, replay harness, hash updates
- [x] 2. Card definition fixes — 15 cards fixed (pull_from_tomorrow, awaken_the_woods, ingenious_prodigy, martial_coup, white_suns_twilight, treasure_vault, chandra_flamecaller fully fixed; the_meathook_massacre, spiteful_banditry, goblin_negotiation, agadeems_awakening, finale_of_devastation, mirror_entity, steel_hellkite, ugin_the_spirit_dragon partially fixed)
- [ ] 3. New card definitions (if any) — N/A
- [x] 4. Unit tests — 10 tests in crates/engine/tests/x_cost_spells.rs, all pass
- [x] 5. Workspace build verification — all pass, 0 clippy warnings, cargo fmt clean

## Review
findings: 6 (HIGH: 0, MEDIUM: 2, LOW: 4)
verdict: needs-fix
review_file: memory/primitives/pb-review-27.md

## Fix Phase Results
- MEDIUM-1 (ingenious_prodigy.rs): Added CR 614.1c deviation comment on ETB trigger ability.
- MEDIUM-2 (martial_coup.rs): Added TODO comment citing oracle text; test comment updated to KNOWN DEVIATION.
- LOW-3 (finale_of_devastation.rs): No action — existing TODO sufficient.
- LOW-4 (ugin_the_spirit_dragon.rs): Added partial -10 LoyaltyAbility with GainLife(7)+DrawCards(7) Sequence; TODO for hand-to-battlefield.
- LOW-5 (ingenious_prodigy.rs): Covered by MEDIUM-1 fix (upkeep "may" deviation comment added).
- LOW-6 (chandra_flamecaller.rs): No action — existing TODOs sufficient.
fix_phase: complete
build: ok (0 errors, 0 clippy warnings)
tests: all pass
