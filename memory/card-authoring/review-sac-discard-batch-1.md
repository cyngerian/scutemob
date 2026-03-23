# Card Review: A-26 activated-sacrifice, A-27 sacrifice-outlet, A-28 discard-effect

**Reviewed**: 2026-03-23
**Cards**: 18
**Findings**: 2 HIGH, 2 MEDIUM, 2 LOW

---

## Card 1: Altar of Dementia
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - CLEAN. TODO for EffectAmount::PowerOfSacrificedCreature is a genuine DSL gap. Empty abilities per W5 policy is correct -- implementing the mill without dynamic amount would produce wrong game state (always mill 0).

## Card 2: Blasting Station
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (partial implementation with TODO)
- **Findings**:
  - F1 (MEDIUM): The first ability (tap + sacrifice creature, deal 1 damage to any target) is implemented, but the second ability (untap on creature ETB) is omitted with TODO. This means the card functions as a one-shot damage source per untap rather than the repeatable combo piece it should be. However, the implemented ability does not produce *wrong* game state -- it just does less than the full card. The sacrifice cost prevents free value. Borderline W5 but the damage ability alone is correct when it fires. Acceptable as partial implementation with clear TODO.
  - F2 (LOW): TODO claims "Effect::Untap { target: Source } is not in DSL." Verify whether an untap effect targeting source exists -- if it does, this is KI-3 (stale TODO). Currently appears to be a genuine gap.

## Card 3: Greater Good
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - CLEAN. Two DSL gaps: EffectAmount::PowerOfSacrificedCreature for draw count, and the discard-three effect as part of a composite resolution. Empty abilities per W5 is correct.

## Card 4: Life's Legacy
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - CLEAN. Two genuine DSL gaps: spell additional sacrifice cost and PowerOfSacrificedCreature for draw count. Empty abilities per W5 is correct.

## Card 5: Perilous Forays
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: MOSTLY (see findings)
- **Findings**:
  - F3 (MEDIUM): `basic_land_filter()` requires `basic: true` (SuperType::Basic), but oracle says "a land card with a basic land type" -- this means any land with subtype Plains/Island/Swamp/Mountain/Forest, including non-basic duals (Tropical Island, Hallowed Fountain, etc.). The filter is too restrictive. The card def acknowledges this in a comment ("closely approximated by basic_land_filter()") but this is a functional difference. In Commander, fetching shock lands / original duals with this card is a significant gameplay pattern. A `has_basic_land_subtype: true` filter (without `basic: true`) would be correct, but that TargetFilter field may not exist. The approximation is documented, so this is MEDIUM not HIGH.
  - F4 (LOW): The `shuffle_before_placing: false` + separate `Effect::Shuffle` pattern is correct for "put onto the battlefield tapped, then shuffle" -- the land is placed first, then shuffle happens. This is fine.

## Card 6: Spawning Pit
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - CLEAN. TODO correctly identifies three DSL gaps: charge counters on source, Cost::RemoveCounters, and custom token spec. Empty abilities per W5 is correct.

## Card 7: Dreamstone Hedron
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - CLEAN. Both abilities fully implemented. Mana ability: tap for {C}{C}{C} via `mana_pool(0,0,0,0,0,3)` -- correct (WUBRGC order). Sacrifice ability: {3}, {T}, sacrifice self for draw 3 -- correct. `Cost::SacrificeSelf` is the right variant for "Sacrifice this artifact."

## Card 8: Miren, the Moaning Well
- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (None for land)
- **DSL correctness**: YES (partial with TODO)
- **Findings**:
  - CLEAN. Tap-for-colorless is implemented. Life-gain ability omitted due to genuine EffectAmount::SacrificedCreatureToughness gap. Per W5, implementing a GainLife(0) placeholder would be wrong; omitting is correct.

## Card 9: Diamond Valley
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes)
- **Mana cost match**: YES (None for land)
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - CLEAN. Same DSL gap as Miren (SacrificedCreatureToughness). Comment correctly notes this land has no mana ability -- it only gains life via sacrifice. Empty abilities per W5 is correct.

## Card 10: Claws of Gix
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES ({0} = `ManaCost { ..Default::default() }`)
- **DSL correctness**: YES
- **Findings**:
  - CLEAN. Sacrifice cost uses `TargetFilter::default()` which matches any permanent -- correct for "Sacrifice a permanent." GainLife with `PlayerTarget::Controller` is correct for "You gain 1 life." Fully implemented.

## Card 11: Altar of Bone
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES ({G}{W})
- **DSL correctness**: NO (see findings)
- **Findings**:
  - F5 (HIGH): **W5 violation -- wrong game state.** The card implements the SearchLibrary effect (tutor a creature to hand) WITHOUT the mandatory additional sacrifice cost. Oracle says "As an additional cost to cast this spell, sacrifice a creature." Without this cost, the card is a 2-mana creature tutor with no sacrifice required -- strictly better than intended. The TODO correctly notes the DSL gap, but per W5 policy, the card should have `abilities: vec![]` since the partial implementation produces wrong game state (free tutor). Compare with Life's Legacy which correctly uses empty abilities for the same pattern.
  - F6 (HIGH): **Inconsistency with Life's Legacy.** Life's Legacy (same pattern: spell + additional sacrifice cost) correctly uses `abilities: vec![]`. Altar of Bone implements the spell effect. Both have the same DSL gap (no spell additional sacrifice cost), but Altar of Bone breaks W5 while Life's Legacy follows it. This should be `abilities: vec![]` with a TODO.

## Card 12: Waste Not
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - CLEAN. TODO correctly identifies WheneverOpponentDiscards as a genuine DSL gap. The card-type conditional logic (creature/land/noncreature-nonland) is also a gap. Empty abilities per W5 is correct.

## Card 13: Megrim
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - CLEAN. Same WheneverOpponentDiscards gap. "That player" reference passing is also correctly identified as missing. Empty abilities per W5 is correct.

## Card 14: Liliana's Caress
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - CLEAN. Identical pattern to Megrim. Empty abilities per W5 is correct.

## Card 15: Raiders' Wake
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - CLEAN. Two abilities, both with genuine DSL gaps (WheneverOpponentDiscards + Condition::YouAttackedThisTurn). Empty abilities per W5 is correct.

## Card 16: Tinybones, Trinket Thief
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Skeleton Rogue)
- **Mana cost match**: YES
- **P/T match**: YES (1/2)
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - CLEAN. Both abilities have genuine DSL gaps (OpponentDiscardedThisTurn condition, OpponentHasNoCardsInHand filter). Empty abilities per W5 is correct. Legendary supertype is present. Subtypes Skeleton and Rogue are present.

## Card 17: Fell Specter
- **Oracle match**: YES
- **Types match**: YES (Creature -- Specter)
- **Mana cost match**: YES
- **P/T match**: YES (1/3)
- **DSL correctness**: PARTIAL (see findings)
- **Findings**:
  - F7 (LOW): ETB ability uses `TargetRequirement::TargetPlayer` but oracle says "target opponent." This means the controller could target themselves or an ally. `TargetRequirement::TargetOpponent` does not exist in the DSL (confirmed -- only appears in TODOs elsewhere). This is a known DSL approximation, not a card authoring error. Documented as LOW since the typical use case (targeting an opponent) works correctly.
  - The third ability (whenever an opponent discards, that player loses 2 life) is correctly omitted with TODO due to WheneverOpponentDiscards gap.
  - Flying keyword is present. ETB discard trigger is correctly implemented with WhenEntersBattlefield + DiscardCards targeting DeclaredTarget{index:0}.

## Card 18: Burglar Rat
- **Oracle match**: YES
- **Types match**: YES (Creature -- Rat)
- **Mana cost match**: YES
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - CLEAN. ETB triggers ForEach::EachOpponent with DiscardCards{count:1} per opponent. `PlayerTarget::DeclaredTarget { index: 0 }` inside ForEach::EachOpponent is the correct pattern (references the iteration variable, not targets vec -- per `feedback_foreach_player_target.md`). Fully implemented.

---

## Summary

- **Cards with issues**:
  - **Altar of Bone** -- 2 HIGH findings (W5 violation: implements search without sacrifice cost; inconsistent with Life's Legacy pattern)
  - **Perilous Forays** -- 1 MEDIUM (basic_land_filter too restrictive for "basic land type" search)
  - **Blasting Station** -- 1 MEDIUM (partial implementation missing untap-on-ETB), 1 LOW (verify untap DSL gap)
  - **Fell Specter** -- 1 LOW (TargetPlayer instead of TargetOpponent, DSL limitation)

- **Clean cards** (14): Altar of Dementia, Greater Good, Life's Legacy, Spawning Pit, Dreamstone Hedron, Miren the Moaning Well, Diamond Valley, Claws of Gix, Waste Not, Megrim, Liliana's Caress, Raiders' Wake, Tinybones Trinket Thief, Burglar Rat

### Fix Actions Required
1. **Altar of Bone (HIGH)**: Change to `abilities: vec![]` -- the spell effect without its mandatory sacrifice cost produces wrong game state. Add TODO matching Life's Legacy pattern.
2. **Perilous Forays (MEDIUM)**: Document in comment that the filter misses non-basic lands with basic land types (shocks, duals). No code change needed unless a `has_basic_land_subtype` filter field is added to the DSL.
3. **Blasting Station (MEDIUM)**: Acceptable as-is with TODO. The implemented damage ability is correct in isolation. Verify whether Effect::UntapPermanent or similar exists; if so, implement the untap trigger.
