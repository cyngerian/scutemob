# Card Review: Removal/Destroy Batch 5

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 1 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Nullmage Shepherd
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities vec![] with TODO)
- **Findings**:
  - F1 (INFO): Card has `abilities: vec![]` with a TODO noting that `Cost::TapCreatures(4)` does not exist. This is a genuine DSL gap — there is no cost variant for "tap N untapped creatures you control" as an activation cost. The TODO is valid and correctly documents the gap. No action needed until the DSL is extended.

## Card 2: Deadly Tempest
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F2 (MEDIUM): W5 policy concern — the card currently only performs `DestroyAll` for creatures but omits the second clause ("Each player loses life equal to the number of creatures they controlled that were destroyed this way"). The TODO correctly identifies this as a per-player destroy-count tracking gap (LastEffectCount is total, not per-player). However, this means casting Deadly Tempest produces **wrong game state**: it destroys creatures but never causes any life loss. Per W5 policy, a partial implementation that produces incorrect game behavior should use `abilities: vec![]`. The DestroyAll-only version is a free board wipe with no downside, which changes the card's strategic profile (the life loss is a real cost in creature-heavy decks). **Recommend changing to `abilities: vec![]` with the existing TODO.**

## Card 3: Brotherhood's End
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - None. Modal spell is correctly structured with `min_modes: 1, max_modes: 1`. Mode 0 uses two `DealDamage` effects in a `Sequence` (one for creatures, one for planeswalkers) which correctly covers "each creature and each planeswalker". Mode 1 uses `DestroyAll` with `has_card_type: Artifact` and `max_cmc: Some(3)`. Clean implementation.

## Card 4: Austere Command
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - None. Modal spell correctly uses `min_modes: 2, max_modes: 2` for "Choose two". All four modes are properly implemented with appropriate `TargetFilter` constraints. Mode 3 uses `min_cmc: Some(4)` for "mana value 4 or greater". Clean implementation.

## Card 5: Damn
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F3 (HIGH): `DestroyPermanent` does not have a `cant_be_regenerated` field — only `DestroyAll` has it. The normal (non-overloaded) mode uses `DestroyPermanent { target: DeclaredTarget { index: 0 } }` which will allow the target creature to regenerate, contrary to oracle text ("A creature destroyed this way can't be regenerated"). The overloaded path correctly uses `DestroyAll { cant_be_regenerated: true }`. **Fix options**: (a) add `cant_be_regenerated: bool` to `DestroyPermanent` in the DSL, or (b) document as a TODO and revert to `abilities: vec![]` per W5 policy since the non-overloaded mode produces incorrect game state. Note: the Overload keyword marker, Overload cost, and Conditional branching on `WasOverloaded` are all correctly structured.

## Summary
- Cards with issues: Deadly Tempest (MEDIUM — W5 policy, partial impl is free wipe), Damn (HIGH — cant_be_regenerated missing on DestroyPermanent for normal mode)
- Clean cards: Nullmage Shepherd (valid TODO), Brotherhood's End, Austere Command
