# Card Review: Batch A -- Ravnica Bounce Lands

**Reviewed**: 2026-03-22
**Cards**: 6
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW (shared across all 6)

---

## Card 1: Azorius Chancery
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes needed)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Mana production**: `mana_pool(1, 1, 0, 0, 0, 0)` = {W}{U} -- correct
- **Findings**: None

## Card 2: Boros Garrison
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Mana production**: `mana_pool(1, 0, 0, 1, 0, 0)` = {W}{R} -- correct
- **Findings**: None

## Card 3: Dimir Aqueduct
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Mana production**: `mana_pool(0, 1, 1, 0, 0, 0)` = {U}{B} -- correct
- **Findings**: None

## Card 4: Golgari Rot Farm
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Mana production**: `mana_pool(0, 0, 1, 0, 1, 0)` = {B}{G} -- correct
- **Findings**: None

## Card 5: Gruul Turf
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Mana production**: `mana_pool(0, 0, 0, 1, 1, 0)` = {R}{G} -- correct
- **Findings**: None

## Card 6: Izzet Boilerworks
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Mana production**: `mana_pool(0, 1, 0, 1, 0, 0)` = {U}{R} -- correct
- **Findings**: None

---

## Shared Pattern Verification (all 6 cards)

All 6 bounce lands use identical structure, differing only in name, card_id, oracle_text, and mana production. Verified the following for all:

| Check | Result | Notes |
|-------|--------|-------|
| ETB tapped replacement | PASS | `ReplacementModification::EntersTapped` with `is_self: true` |
| ETB bounce trigger | PASS | `WhenEntersBattlefield` trigger with `MoveZone` to `ZoneTarget::Hand` |
| Target filter | PASS | `TargetPermanentWithFilter` with `has_card_type: Some(CardType::Land)`, `controller: TargetController::You` |
| Owner's hand | PASS | Uses `PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))` -- correct for "its owner's hand" |
| Mana ability | PASS | `Cost::Tap` with `Effect::AddMana`, `player: PlayerTarget::Controller` |
| No extra abilities | PASS | Exactly 3 abilities each (replacement, triggered, activated) |
| No supertypes needed | PASS | Plain "Land" type line, not Legendary/Basic/Snow |
| Mana cost absent | PASS | `mana_cost: None` for all lands |

## Shared Observation (LOW)

- F-shared (LOW): The oracle ETB trigger "return a land you control" is non-targeted (no "target" keyword in oracle text). The DSL uses `TargetPermanentWithFilter` as a proxy for resolution-time choice; this is documented in comments. This is an acceptable engine approximation -- the behavioral difference (targeting vs. choosing) only matters for hexproof/shroud on your own lands, which is an extremely rare edge case. No action needed.

## Summary
- Cards with issues: None
- Clean cards: Azorius Chancery, Boros Garrison, Dimir Aqueduct, Golgari Rot Farm, Gruul Turf, Izzet Boilerworks
- All 6 bounce lands are correctly implemented with matching oracle text, correct mana production, proper ETB-tapped replacement, correct bounce trigger with OwnerOf for "owner's hand", and correct target filters.
