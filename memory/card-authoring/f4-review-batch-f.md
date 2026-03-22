# Card Review: F-4 Session 6 Batch

**Reviewed**: 2026-03-22
**Cards**: 11
**Findings**: 0 HIGH, 1 MEDIUM, 1 LOW

---

## Card 1: Valakut, the Molten Pinnacle
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**:
  - ETB tapped replacement: correct (`EntersTapped`, `is_self: true`).
  - `{T}: Add {R}` mana ability: correct. `mana_pool(0,0,0,1,0,0)` = 0W 0U 0B 1R 0G 0C.
  - TODO for triggered ability (Mountain-enters + 5-other-Mountains intervening-if + any-target damage): genuine DSL gap (subtype-filtered permanent-ETB trigger, count-based intervening-if, "any target" damage). Valid.
- Clean.

## Card 2: Spinerock Knoll
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**:
  - ETB tapped replacement: correct.
  - `{T}: Add {R}` mana ability: correct.
  - TODO for Hideaway 4: valid DSL gap (look-at-top-N-exile-one ETB sequence).
  - TODO for `{R}, {T}:` play-from-exile with damage-threshold condition: valid DSL gap.
- Clean.

## Card 3: Creeping Tar Pit
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**:
  - ETB tapped replacement: correct.
  - `{T}: Add {U} or {B}` via `Effect::Choose` with two `AddMana` branches: correct. `mana_pool(0,1,0,0,0,0)` = {U}, `mana_pool(0,0,1,0,0,0)` = {B}. WUBRGC order verified.
  - TODO for `{1}{U}{B}` land animation: valid DSL gap (land-becomes-creature effect).
- Clean.

## Card 4: Temple of the False God
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**:
  - `{T}: Add {C}{C}` with `mana_pool(0,0,0,0,0,2)` = 2 colorless. Correct.
  - `activation_condition: Some(Condition::ControlAtLeastNOtherLands(4))`: Oracle says "five or more lands." Temple itself is a land, so 4 other lands + Temple = 5 total. Comment in def explains this reasoning correctly. Correct.
- Clean.

## Card 5: Den of the Bugbear
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**:
  - Conditional ETB tapped: `unless_condition: Some(Condition::Not(Box::new(Condition::ControlAtLeastNOtherLands(2))))`. Semantics: enters tapped UNLESS you do NOT control 2+ other lands = enters tapped IF you control 2+ other lands. Matches oracle "If you control two or more other lands, this land enters tapped." Correct.
  - `{T}: Add {R}` mana ability: correct.
  - TODO for `{3}{R}` land animation: valid DSL gap.
- Clean.

## Card 6: Cryptic Coat
- **Oracle match**: YES (minor formatting with line continuation in string literal, concatenated text matches Scryfall)
- **Types match**: YES (Artifact -- Equipment)
- **Mana cost match**: YES (`{2}{U}`)
- **DSL correctness**: YES
- **Findings**:
  - ETB trigger (Cloak + AttachEquipment via Sequence): correct pattern.
  - `{1}{U}: Return this Equipment to its owner's hand`: uses `Effect::MoveZone` with `ZoneTarget::Hand { owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::Source)) }`. Correctly targets owner, not controller (KI-11 compliant).
  - TODO for static grant (+1/+0, can't be blocked on equipped creature): genuine DSL gap.
  - F1 (LOW): Header comment at lines 9-11 says "{1}{U}: Return to hand -- no ReturnToHand activated ability. TODO." but this ability IS now implemented in the abilities vec (lines 44-57). The header comment is stale.

## Card 7: Arixmethes, Slumbering Isle
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Kraken, uses `full_types` with `SuperType::Legendary`)
- **Mana cost match**: YES (`{2}{G}{U}`)
- **P/T**: Correct (12/12)
- **DSL correctness**: ISSUE
- **Findings**:
  - `{T}: Add {G}{U}`: `mana_pool(0,1,0,0,1,0)` = 0W 1U 0B 0R 1G 0C = {G}{U}. Correct.
  - F2 (MEDIUM): W5 concern -- Arixmethes has a mana ability but is missing "enters tapped with five slumber counters." Without the ETB replacement, it enters untapped as a 12/12 creature that can immediately tap for {G}{U}. The card is fundamentally incomplete without slumber counters + type-change + counter-removal trigger, so the mana ability in isolation creates a creature that provides free mana on entry. This is not subtly wrong (the card is obviously incomplete), but it does produce wrong game state in a technical sense. The three TODOs (ETB-with-counters, conditional type-change, whenever-you-cast-a-spell trigger) are all genuine DSL gaps and correctly documented.

## Card 8: Oathsworn Vampire
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Knight; subtypes unordered, both present)
- **Mana cost match**: YES (`{1}{B}`)
- **P/T**: Correct (2/2)
- **DSL correctness**: YES
- **Findings**:
  - ETB tapped replacement: correct (`EntersTapped`, `is_self: true`).
  - TODO for graveyard casting with life-gained-this-turn condition: valid DSL gap.
- Clean.

## Card 9: Hydroelectric Specimen
- **Oracle match**: YES (front face oracle for DFC)
- **Types match**: YES (Creature -- Weird)
- **Mana cost match**: YES (`{2}{U}`)
- **P/T**: Correct (1/4)
- **DSL correctness**: YES
- **Findings**:
  - Flash keyword: correct.
  - TODO for ETB target-redirection effect: valid DSL gap (no target redirection primitive).
- Clean.

## Card 10: Bloomvine Regent
- **Oracle match**: YES (front face oracle for DFC)
- **Types match**: YES (Creature -- Dragon)
- **Mana cost match**: YES (`{3}{G}{G}`)
- **P/T**: Correct (4/5)
- **DSL correctness**: YES
- **Findings**:
  - Flying keyword: correct.
  - Dragon-enters trigger: `WheneverCreatureEntersBattlefield` with `has_subtype: Some(SubType("Dragon"))` and `controller: TargetController::You`. Oracle says "Whenever this creature or another Dragon you control enters, you gain 3 life." This is equivalent to "any Dragon you control enters" since the source IS a Dragon. The filter correctly matches self-entering and other Dragons you control. `GainLife { player: PlayerTarget::Controller, amount: EffectAmount::Fixed(3) }` is correct.
- Clean.

## Card 11: Marang River Regent
- **Oracle match**: YES (front face oracle for DFC)
- **Types match**: YES (Creature -- Dragon)
- **Mana cost match**: YES (`{4}{U}{U}`)
- **P/T**: Correct (6/7)
- **DSL correctness**: YES
- **Findings**:
  - Flying keyword: correct.
  - TODO for ETB bounce ("return up to two other target nonland permanents to their owners' hands"): valid DSL gap -- "up to N" optional multi-target mechanism is not supported. Correctly documented.
- Clean.

---

## Summary

**Findings**: 0 HIGH, 1 MEDIUM, 1 LOW

### Cards with issues
- **Cryptic Coat**: 1 LOW (F1 -- stale header comment says return-to-hand is TODO but it is now implemented)
- **Arixmethes, Slumbering Isle**: 1 MEDIUM (F2 -- mana ability without enters-tapped creates wrong game state, but card is fundamentally incomplete without slumber mechanics; all TODOs are genuine DSL gaps)

### Clean cards
- Valakut, the Molten Pinnacle
- Spinerock Knoll
- Creeping Tar Pit
- Temple of the False God
- Den of the Bugbear
- Oathsworn Vampire
- Hydroelectric Specimen
- Bloomvine Regent
- Marang River Regent

### Condition Logic Verification
- **Temple of the False God**: `ControlAtLeastNOtherLands(4)` correctly maps "five or more lands" (4 other + self = 5 total). VERIFIED.
- **Den of the Bugbear**: `Not(ControlAtLeastNOtherLands(2))` as `unless_condition` correctly implements "if you control two or more other lands, enters tapped." VERIFIED.

### TODO Validity (KI-3 check)
All remaining TODOs across all 11 cards cite genuine DSL gaps. No stale TODOs found.

### Action Items
1. **Cryptic Coat** (LOW): Update header comment at lines 9-11 to remove the stale "no ReturnToHand activated ability" claim.
2. **Arixmethes** (MEDIUM): Consider whether the mana ability should be removed (making the card `vec![]`) until enters-tapped-with-counters and type-change statics are available. Currently the card is a 12/12 that taps for {G}{U} on entry, which is significantly stronger than intended.
