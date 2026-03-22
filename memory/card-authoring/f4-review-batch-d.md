# Card Review: F-4 Session 1 Batch D

**Reviewed**: 2026-03-22
**Cards**: 6
**Findings**: 1 HIGH, 3 MEDIUM, 3 LOW

---

## Card 1: Skemfar Elderhall

- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: MOSTLY
- **Findings**:
  - F1 (LOW): Oracle says "up to one target creature you don't control" but DSL has mandatory `TargetCreatureWithFilter`. No "up to" / optional target mechanism exists in `TargetRequirement`. The ability cannot be activated with zero targets. True DSL gap -- document only.
  - ETB tapped: YES (EntersTapped replacement present, matches oracle)
  - Mana ability: Correct ({G} via mana_pool(0,0,0,0,1,0))
  - Sacrifice cost: Correct (Cost::SacrificeSelf in Sequence with mana+tap)
  - Token spec: Correct (2x 1/1 green Elf Warrior creature tokens)
  - Sorcery speed restriction: Correct (TimingRestriction::SorcerySpeed)
  - Mana cost on activated: Correct ({2}{B}{B}{G} = generic:2, black:2, green:1)

## Card 2: Gnarlroot Trapper

- **Oracle match**: YES
- **Types match**: YES (Creature -- Elf Druid)
- **Mana cost match**: YES ({B})
- **P/T match**: YES (1/1)
- **DSL correctness**: PARTIAL (1 of 2 abilities implemented)
- **Findings**:
  - F2 (MEDIUM): Second ability TODO claims "DSL gap: EffectTarget has no AttackingCreatureWithSubtype variant." This is a genuine gap -- `TargetFilter` lacks an `attacking: bool` field, and `TargetRequirement` has no attacking-creature variant. The TODO is valid. However, the card is missing half its abilities. Per W5 policy, a creature with only one of two abilities implemented could produce wrong game state -- but this is a mana-producing ability + a combat trick, and the mana ability alone does not create incorrect combat results (it just makes the card less powerful). The TODO is acceptable.
  - Mana restriction: Correct. Oracle says "cast an Elf creature spell" and def uses `ManaRestriction::CreatureWithSubtype(SubType("Elf"))`, which checks both creature type and Elf subtype. Correct variant choice.

## Card 3: The Seedcore

- **Oracle match**: YES
- **Types match**: YES (Land -- Sphere)
- **Mana cost match**: YES (none)
- **DSL correctness**: MOSTLY
- **Findings**:
  - F3 (LOW): TODO on line 64 says "Target should be '1/1 creature' -- TargetFilter lacks exact P/T constraint." This is a genuine DSL gap. `TargetFilter` has `max_power` and `min_power` but no `exact_power`/`exact_toughness` fields. However, you COULD approximate "1/1 creature" by setting `max_power: Some(1), min_power: Some(1)` -- this constrains power to exactly 1. Toughness would need a similar pair of fields (`max_toughness`/`min_toughness`) which do not exist. True gap for toughness constraint. LOW severity since the buff is not harmful to non-1/1 creatures.
  - Corrupted activation condition: Correct (`Condition::OpponentHasPoisonCounters(3)`)
  - +2/+1 implementation: Correct. Uses two separate `ApplyContinuousEffect` with `ModifyPower(2)` and `ModifyToughness(1)` since `ModifyBoth` only supports symmetric values.
  - Mana restriction: Correct. Oracle says "cast Phyrexian creature spells" and def uses `ManaRestriction::CreatureWithSubtype(SubType("Phyrexian"))`. Phyrexian is indeed a creature type in MTG.

## Card 4: Voldaren Estate

- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: MOSTLY
- **Findings**:
  - F4 (MEDIUM): TODO on line 38 says "Cost reduction per Vampire not expressible (G-27). Using full {5} cost." This is a genuine DSL gap. `SelfCostReduction` only exists on `CardDefinition` for spell casting costs, not on `AbilityDefinition::Activated` for activated ability costs. The activated ability always costs {5} instead of {5} minus Vampire count. The effect itself is correct (`blood_token_spec(1)`). The cost errs on the side of being too expensive, not too cheap.
  - F5 (MEDIUM): W5 policy assessment: the {5} flat cost means controlling Vampires gives no discount. In practice this makes the ability nearly unusable (5 mana + tap for a Blood token is terrible rate). The card's other two abilities (colorless mana and restricted any-color mana) work correctly. Borderline W5 -- the card functions but one ability is strictly worse than oracle. MEDIUM rather than HIGH since the effect is correct, only the cost is wrong, and it errs on the expensive side.
  - Mana restriction: Correct. Oracle says "cast a Vampire spell" (not "Vampire creature spell"), so `ManaRestriction::SubtypeOnly(SubType("Vampire"))` is correct.

## Card 5: Oboro, Palace in the Clouds

- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F6 (HIGH): Oracle says "Return Oboro to **its owner's** hand" but def uses `ZoneTarget::Hand { owner: PlayerTarget::Controller }`. In multiplayer Commander, a permanent's controller and owner can differ (e.g., after Bribery, Gilded Drake, or steal effects). `PlayerTarget::Controller` returns it to the controller's hand, not the owner's hand. This is KI-11. **Fix**: change to `ZoneTarget::Hand { owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::Source)) }`. The `PlayerTarget::OwnerOf` variant exists (confirmed at card_definition.rs:1604) and is designed exactly for this use case.
  - Legendary supertype: Correct (present in type line)
  - Mana ability: Correct ({U} via mana_pool(0,1,0,0,0,0))
  - Self-bounce cost: Correct ({1} generic mana, no tap)

## Card 6: Geier Reach Sanitarium

- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**:
  - F7 (LOW): The ForEach loop uses `PlayerTarget::DeclaredTarget { index: 0 }` inside `ForEach { over: ForEachTarget::EachPlayer }`. Initial suspicion was that this would reference a non-existent declared target. However, **this is actually correct** -- the `ForEach` handler in `effects/mod.rs` (lines 2043-2049) creates an inner `EffectContext` for each iteration where `targets[0]` is populated with the current player as a `SpellTarget { target: Target::Player(p) }`. So `DeclaredTarget { index: 0 }` correctly resolves to the current iteration's player. Inside a ForEach loop, using `DeclaredTarget { index: 0 }` is the idiomatic pattern. `PlayerTarget::EachPlayer` is for non-ForEach contexts (e.g., `LoseLife { player: PlayerTarget::EachPlayer }`).
  - Legendary supertype: Correct (present in type line via `full_types`)
  - Mana ability: Correct ({C} via mana_pool(0,0,0,0,0,1))
  - Activation cost: Correct ({2} + tap)

---

## Summary

- **Cards with issues**:
  - Oboro, Palace in the Clouds: 1 HIGH (KI-11: Controller vs Owner for "its owner's hand")
  - Voldaren Estate: 2 MEDIUM (genuine DSL gap for activated ability cost reduction; overcosted ability)
  - Gnarlroot Trapper: 1 MEDIUM (partial implementation, genuine DSL gap for attacking creature targeting)
  - Skemfar Elderhall: 1 LOW (no "up to" optional target mechanism)
  - The Seedcore: 1 LOW (no exact P/T target filter, partially approximable)
  - Geier Reach Sanitarium: 1 LOW (informational -- DeclaredTarget inside ForEach is correct)

- **Clean cards**: Geier Reach Sanitarium (functionally correct)

### Action Items

| ID | Severity | Card | Fix |
|----|----------|------|-----|
| F6 | HIGH | Oboro, Palace in the Clouds | Change `PlayerTarget::Controller` to `PlayerTarget::OwnerOf(Box::new(EffectTarget::Source))` in `ZoneTarget::Hand` |
| F4 | MEDIUM | Voldaren Estate | Genuine DSL gap (G-27). TODO is valid. No fix without engine changes. |
| F2 | MEDIUM | Gnarlroot Trapper | Genuine DSL gap. TODO is valid. No fix without adding `attacking` to TargetFilter. |
| F5 | MEDIUM | Voldaren Estate | Same root cause as F4. |

### Stale TODO Check
- Gnarlroot Trapper TODO: **VALID** -- no attacking creature target filter exists
- The Seedcore TODO: **VALID** -- no exact toughness constraint on TargetFilter
- Voldaren Estate TODO: **VALID** -- no activated ability cost reduction mechanism

### DSL Gaps Identified
1. **G-NEW-1**: `TargetFilter` lacks `attacking: bool` field for targeting attacking creatures
2. **G-NEW-2**: `TargetFilter` lacks `max_toughness`/`min_toughness` for exact P/T matching (power has `max_power`/`min_power` but toughness does not)
3. **G-27** (confirmed): No `SelfCostReduction` equivalent for activated ability costs
4. **G-NEW-3**: No "up to N" optional target mechanism in `TargetRequirement`
