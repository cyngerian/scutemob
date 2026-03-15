# Ability WIP: PB-10 Return From Zone Effects

ability: Return From Zone (graveyard targeting)
cr: 115.1, 115.7, 400.7, 608.2b
priority: W6-PB-10
started: 2026-03-14
phase: complete
plan_file: memory/abilities/ability-plan-pb10-return-from-zone.md

## Step Checklist
- [x] 1. Add TargetCardInYourGraveyard + TargetCardInGraveyard to TargetRequirement enum
- [x] 2. Add has_subtypes to TargetFilter + update matches_filter
- [x] 3. Update casting.rs validate_object_satisfies_requirement for graveyard zone
- [x] 4. Unit tests (10 tests, cite CR 115.1, CR 608.2b)
- [x] 5. Fix 9 card definitions (8 planned + Den Protector + Grim Harvest bonus)
- [x] 6. Build verification (2054 tests, 0 clippy warnings, workspace builds clean)

## Summary
- 2 new TargetRequirement variants: TargetCardInYourGraveyard, TargetCardInGraveyard
- 1 new TargetFilter field: has_subtypes (Vec<SubType>, OR semantics)
- casting.rs: 2 new arms in validate_object_satisfies_requirement (no hexproof/shroud for GY)
- hash.rs: 3 changes (TargetFilter has_subtypes, 2 TargetRequirement variants)
- effects/mod.rs: has_subtypes check added to matches_filter
- No resolution.rs changes needed (existing DeclaredTarget + MoveZone handles it)
- 10 new tests in graveyard_targeting.rs
- 9 card defs fixed: Bloodline Necromancer, Buried Ruin, Emeria the Sky Ruin,
  Hall of Heliod's Generosity, Nullpriest of Oblivion, Reanimate, Teneb the Harvester,
  Bladewing the Risen, Den Protector; 1 bonus: Grim Harvest
- Remaining TODOs in card defs (separate DSL gaps):
  - Reanimate: "lose life equal to mana value" (EffectAmount::ManaValueOfTarget)
  - Teneb: optional mana payment on trigger (Cost on triggered abilities)
  - Emeria: "7+ Plains" count threshold (Condition::YouControlNOrMore...)
  - Bladewing: "{B}{R}: Dragon creatures get +1/+1 until EOT" (filtered pump)
