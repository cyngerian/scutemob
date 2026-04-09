# Primitive WIP: PB-E — Mana Doubling

batch: PB-E
title: Mana doubling — mana trigger interception system
cards_affected: 9
started: 2026-04-08
phase: DONE
plan_file: memory/primitives/pb-plan-E.md

## Deferred from Prior PBs
- A-38 blocked "mana-doubling" category (~9 cards)
- A-42 may have additional mana-doubling cards

## Known Cards with Mana Doubling TODOs
Existing defs with TODOs:
- Mirari's Wake: "Whenever you tap a land for mana, add one mana of any type that land produced"
- Crypt Ghast: "Whenever you tap a Swamp for mana, add an additional {B}"
- Wild Growth: "Whenever enchanted land is tapped for mana, add {G}"
- Leyline of Abundance: "Whenever you tap a creature for mana, add {G}"
- Badgermole Cub: "Whenever you tap a creature for mana, add an additional {G}"
- Forbidden Orchard: "Whenever you tap this land for mana, target opponent creates a 1/1"

Cards that may need new defs:
- Nyxbloom Ancient: "If you tap a permanent for mana, it produces three times as much"
- Mana Reflection: "If you tap a permanent for mana, it produces twice as much"
- Zendikar Resurgent: "Whenever you tap a land for mana, add one mana of any type that land produced"

## Step Checklist
- [x] 1. Engine changes — GameEvent::ManaAdded source field; TriggerCondition::WhenTappedForMana + ManaSourceFilter; ReplacementTrigger::ManaWouldBeProduced + ReplacementModification::MultiplyMana; Effect::AddManaMatchingType; EffectContext.mana_produced; mana.rs: apply_mana_production_replacements + fire_mana_triggered_abilities + mana_source_matches + is_mana_producing_effect; hash.rs; helpers.rs; replay_harness.rs; tui/app.rs
- [x] 2. Card definition fixes — miraris_wake.rs, crypt_ghast.rs, wild_growth.rs, leyline_of_abundance.rs, badgermole_cub.rs, forbidden_orchard.rs (6 files)
- [x] 3. New card definitions — nyxbloom_ancient.rs, mana_reflection.rs, zendikar_resurgent.rs (3 new files)
- [x] 4. Unit tests — crates/engine/tests/mana_triggers.rs (10 tests, all pass); also fixed mana_and_lands.rs, treasure_tokens.rs, mana_filter.rs, primitive_pb37.rs for GameEvent::ManaAdded source field + EffectContext.mana_produced
- [x] 5. Workspace build verification — cargo test --all: 0 failed; cargo clippy -- -D warnings: 0 errors; cargo build --workspace: clean; cargo fmt --check: clean
