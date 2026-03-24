# Primitive WIP: PB-24 -- Conditional Statics ("as long as X")

batch: PB-24
title: Conditional statics ("as long as X")
cards_affected: ~201
started: 2026-03-23
phase: close
plan_file: memory/primitives/pb-plan-24.md

## Gap Reference
G-2 from `docs/dsl-gap-closure-plan.md`: Add `condition: Option<Condition>` field to
`AbilityDefinition::Static` so cards with "as long as you control X" / "as long as it's
your turn" / threshold conditions can gate their static abilities.

## Deferred from Prior PBs
none

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch)
  - Added `condition: Option<Condition>` to `ContinuousEffectDef` and `ContinuousEffect`
  - Added 5 new `Condition` variants: `OpponentLifeAtMost`, `SourceIsUntapped`, `IsYourTurn`, `YouControlNOrMoreWithFilter`, `DevotionToColorsLessThan`
  - Added `RemoveCardTypes(OrdSet<CardType>)` to `LayerModification`
  - Added `CounterType::Quest` and `CounterType::Slumber`
  - Added `check_static_condition` and `calculate_devotion_to_colors` to effects/mod.rs
  - Updated `is_effect_active` in layers.rs to check condition field
  - Updated `register_static_continuous_effects` in replacement.rs to resolve `EffectFilter::Source` and propagate condition
  - Added Layer 4 `RemoveCardTypes` arm in layers.rs
  - Updated all exhaustive matches (hash.rs, replay-viewer view_model.rs)
  - Added `condition: None` to all existing ContinuousEffect and ContinuousEffectDef struct literals
  - Workspace compiles cleanly
- [x] 2. Card definition fixes
  - serra_ascendant.rs: +5/+5 and Flying when life >= 30 (ControllerLifeAtLeast)
  - dragonlord_ojutai.rs: hexproof while untapped (SourceIsUntapped)
  - bloodghast.rs: haste when opponent <= 10 life (OpponentLifeAtMost)
  - purphoros_god_of_the_forge.rs: not a creature when devotion to red < 5 (DevotionToColorsLessThan + RemoveCardTypes)
  - athreos_god_of_passage.rs: not a creature when devotion to W+B < 7
  - iroas_god_of_victory.rs: not a creature when devotion to R+W < 7
  - nadaar_selfless_paladin.rs: +1/+1 to others when dungeon completed (CompletedADungeon)
  - beastmaster_ascension.rs: +5/+5 to creatures when 7+ quest counters (SourceHasCounters + CounterType::Quest)
  - quest_for_the_goblin_lord.rs: +2/+0 to creatures when 5+ quest counters
  - arixmethes_slumbering_isle.rs: is a land + not a creature when slumber counter (CounterType::Slumber)
  - razorkin_needlehead.rs: first strike during your turn (IsYourTurn)
  - mox_opal.rs: activation condition — 3+ artifacts (YouControlNOrMoreWithFilter)
  - indomitable_archangel.rs: updated TODO (condition expressible, blocked on EffectFilter PB-25)
- [x] 3. New card definitions (if any) — none needed
- [x] 4. Unit tests
  - crates/engine/tests/conditional_statics.rs: 11 tests, all passing
  - test_conditional_static_life_threshold (Serra Ascendant, ControllerLifeAtLeast)
  - test_conditional_static_untapped (Dragonlord Ojutai, SourceIsUntapped)
  - test_conditional_static_counter_threshold (Beastmaster Ascension, SourceHasCounters + CounterType::Quest)
  - test_conditional_static_dungeon (Nadaar, CompletedADungeon)
  - test_conditional_static_opponent_life (Bloodghast, OpponentLifeAtMost)
  - test_conditional_static_is_your_turn (Razorkin Needlehead, IsYourTurn)
  - test_conditional_static_devotion_single (Purphoros, DevotionToColorsLessThan single color)
  - test_conditional_static_devotion_multicolor (Athreos, DevotionToColorsLessThan multi-color)
  - test_conditional_static_remove_type (RemoveCardTypes isolation test)
  - test_conditional_static_toggles_midgame (immediate condition toggle)
  - test_conditional_static_source_filter_resolved (EffectFilter::Source → SingleObject)
- [x] 5. Workspace build verification
  - cargo test -p mtg-engine: all tests pass, 0 failures, 11 new tests added (conditional_statics.rs)
  - cargo clippy -- -D warnings: 0 warnings
  - cargo build --workspace: clean (engine + simulator + network + replay-viewer + tui)
  - cargo fmt --check: clean

## Review
findings: 8 (HIGH: 1, MEDIUM: 2, LOW: 5)
verdict: needs-fix
review_file: memory/primitives/pb-review-24.md

## Fix Phase
- [x] HIGH-4: nadaar_selfless_paladin.rs — mana cost {3}{W} → {2}{W} (header comment + ManaCost field)
- [x] MEDIUM-1: effects/mod.rs calculate_devotion_to_colors — added CR 700.5a known-deviation doc comment
- [x] MEDIUM-2: effects/mod.rs YouControlNOrMoreWithFilter — added re-entrancy safety doc comment
- LOW findings: informational only, no fixes applied
- cargo test --all: all pass, 0 failures
- cargo clippy -- -D warnings: 0 warnings
- cargo build --workspace: clean
