# Ability Plan: Changeling

**Generated**: 2026-02-26
**CR**: 702.73
**Priority**: P2
**Similar abilities studied**: Layer 4 type-changing (Blood Moon `SetTypeLine`, Urborg `AddSubtypes`) in `rules/layers.rs`; Protection subtype matching in `rules/protection.rs:163`; CDA infrastructure (`is_cda` flag on `ContinuousEffect`)

## CR Rule Text

> **702.73. Changeling**
>
> **702.73a** Changeling is a characteristic-defining ability. "Changeling" means "This object is every creature type." This ability works everywhere, even outside the game. See rule 604.3.

Referenced rules:

> **604.3.** Some static abilities are characteristic-defining abilities. A characteristic-defining ability conveys information about an object's characteristics that would normally be found elsewhere on that object (such as in its mana cost, type line, or power/toughness box). Characteristic-defining abilities can add to or override information found elsewhere on that object. Characteristic-defining abilities function in all zones. They also function outside the game and before the game begins.
>
> **604.3a** A static ability is a characteristic-defining ability if it meets the following criteria: (1) It defines an object's colors, subtypes, power, or toughness; (2) it is printed on the card it affects, it was granted to the token it affects by the effect that created the token, or it was acquired by the object it affects as the result of a copy effect or text-changing effect; (3) it does not directly affect the characteristics of any other objects; (4) it is not an ability that an object grants to itself; and (5) it does not set the values of such characteristics only if certain conditions are met.

Layer system context:

> **613.1d** Layer 4: Type-changing effects are applied. These include effects that change an object's card type, subtype, and/or supertype.
>
> **613.3.** Within layers 2-6, apply effects from characteristic-defining abilities first (see rule 604.3), then all other effects in timestamp order (see rule 613.7).

Creature type master list: CR 205.3m (approximately 290+ types including two-word "Time Lord").

## Key Edge Cases

1. **Changeling applies in ALL zones (CR 604.3, 702.73a)** -- not just the battlefield. A Changeling card in the graveyard is still every creature type. This matters for effects like "Return target Goblin card from your graveyard to your hand." The engine's `calculate_characteristics` is primarily called for battlefield objects; the CDA must also be applied when checking creature types in other zones.

2. **Layer ordering: CDA in Layer 4 applies before Layer 6 ability removal (CR 613.3).** If a Changeling creature loses all abilities (e.g., Humility, Dress Down), it remains all creature types because:
   - Layer 4 (CDA): Changeling sets all creature subtypes
   - Layer 6: RemoveAllAbilities removes the Changeling keyword
   - But the Layer 4 type-change already happened and is not undone by Layer 6.
   - Confirmed by Maskwood Nexus ruling (2021-02-05): "If an effect causes a creature with changeling to lose all abilities, it will remain all creature types, even though it will no longer have changeling."

3. **Changeling and type-setting effects.** Per Maskwood Nexus ruling (2021-02-05): "If an effect causes a creature with changeling to become a new creature type, it will be only that new creature type." This means a `SetTypeLine` effect at Layer 4 that comes after the CDA (by timestamp) will override Changeling's subtypes. Since CDAs apply first within Layer 4 (CR 613.3), a non-CDA `SetTypeLine` will always override Changeling within Layer 4. This is already handled correctly by `resolve_layer_order` which partitions CDAs before non-CDAs.

4. **Changeling on non-creature cards.** Cards with Changeling that are not creatures (e.g., Kindred tribal spells) still have all creature types. The subtypes apply regardless of card type per CR 702.73a: "This object is every creature type." However, CR 205.3d says "An object can't gain a subtype that doesn't correspond to one of that object's types." Creature subtypes correspond to both Creature and Kindred card types (CR 205.3m). For objects that are neither creatures nor kindred, the CDA should still list the types (MTG handles this at a rules level, not by preventing the CDA from functioning). In practice, most Changeling cards are creatures or kindred, so this edge case is low priority.

5. **Copy effects and Changeling.** When you copy a creature with Changeling, the copy gets Changeling (it's a copiable value). The copy's Changeling CDA then applies normally. Layer 1 (Copy) sets the copiable values including the Changeling keyword, and then Layer 4 processes the CDA.

6. **Protection from a creature type (e.g., "Protection from Goblins").** A creature with Changeling IS a Goblin, so `ProtectionQuality::FromSubType(SubType("Goblin"))` matches. This is already handled by `protection.rs:163` checking `source_chars.subtypes.contains(st)` -- as long as the subtypes are populated by the layer system, protection works automatically.

7. **Multiplayer: all creature types includes every tribal lord bonus.** In Commander, a Changeling creature benefits from every "creatures of type X get +1/+1" effect on the battlefield. This is already handled by `EffectFilter` matching if the creature has the correct subtype.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Design Decision: Modeling Changeling

Changeling requires **two components**:

### A) `KeywordAbility::Changeling` variant
- Marker for presence-checking, copy effects, display
- Listed in `keywords` on objects that have Changeling
- Removed by "lose all abilities" effects in Layer 6

### B) CDA continuous effect at Layer 4
- A `LayerModification::AddAllCreatureTypes` variant that adds every creature type from CR 205.3m
- Registered as a CDA (`is_cda: true`) so it applies before non-CDA type effects within Layer 4
- Duration: special -- works in all zones, not just battlefield
- Unlike other static abilities, Changeling's CDA needs to be **intrinsic to the object's characteristics**, not registered as a separate `ContinuousEffect` in `state.continuous_effects`

### Implementation Approach: Intrinsic CDA via `calculate_characteristics`

Rather than registering 290+ subtypes as a `ContinuousEffect`, the cleanest approach is:

1. Add `KeywordAbility::Changeling` to the enum
2. In `calculate_characteristics` (in `rules/layers.rs`), at the start of Layer 4 processing, check if the object's current `keywords` set contains `Changeling`. If so, add all creature subtypes to `chars.subtypes` **before** processing any Layer 4 continuous effects. This mimics the CDA-first rule (CR 613.3).
3. After Layer 6 processing (ability removal), the keyword may be gone, but the subtypes were already added in Layer 4 and are not removed. This correctly implements the "lose all abilities but keep all creature types" ruling.

This avoids:
- Creating 290+ `SubType` strings per Changeling creature per characteristic calculation
- Storing a massive `ContinuousEffect` in state
- Special handling for "works in all zones" duration

**But wait** -- there is a subtlety. `calculate_characteristics` starts with the object's base characteristics, then applies Layer 1 (copy), then Layer 4. If Changeling is in the base `keywords`, we can check it at the beginning of Layer 4. But what if Changeling is **granted** by a Layer 6 effect (e.g., Maskwood Nexus creating a token with changeling)? The keyword would not be in `chars.keywords` until after Layer 6, which is too late for Layer 4.

Resolution: Changeling granted by **printed keyword** or **token creation** is a CDA and applies in Layer 4. Changeling granted by a Layer 6 "gain ability" effect is NOT a CDA per CR 604.3a criterion (2) -- it was not printed, not part of the token definition, and not from a copy/text-change. So for granted-Changeling (Maskwood Nexus's first ability, which says "Creatures you control are every creature type"), the correct implementation is a separate Layer 4 continuous effect (which Maskwood Nexus would register, not the Changeling keyword itself). The Changeling keyword implementation only needs to handle the CDA case.

**Therefore: check `chars.keywords.contains(Changeling)` at the start of Layer 4 in `calculate_characteristics` is correct.** The keywords set at that point contains:
- Printed keywords (from base characteristics)
- Keywords from Layer 1 (copy effects)
- But NOT keywords from Layer 6 (ability granting)

This is exactly the CDA behavior we want.

### All-zones handling

`calculate_characteristics` is designed for battlefield objects. For non-battlefield objects, the engine often uses the raw `obj.characteristics` directly. Since Changeling's CDA works in all zones (CR 604.3), we need to ensure that any code checking creature types on non-battlefield objects either:
1. Calls `calculate_characteristics` (which already works for any zone -- it just filters effects by zone)
2. OR has explicit Changeling handling

The existing subtype-checking sites in the engine:
- `protection.rs:163` -- uses `source_chars` which comes from `calculate_characteristics`
- `combat.rs:494` (landwalk) -- uses `calculate_characteristics`
- `casting.rs:264` (Aura check) -- uses `chars` from `calculate_characteristics`
- `sba.rs:638,744,747` (Aura/Equipment checks) -- uses `obj.characteristics` or `chars_map`
- `layers.rs:184` (AllNonAuraEnchantments filter) -- uses `chars` in-flight
- `effects/mod.rs:1875` (target filter has_subtype) -- uses `calculate_characteristics`

For the non-battlefield case (e.g., "return target Goblin from graveyard"), `calculate_characteristics` already works on any zone. The key is that code performing type checks on non-battlefield objects must call `calculate_characteristics` rather than reading raw `characteristics`. This is an existing concern (not introduced by Changeling) and the main sites already use `calculate_characteristics`.

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Changeling` variant after `Undying` (line ~285)
**Pattern**: Follow the simple variants like `Convoke` (no payload)
**Doc comment**: `/// CR 702.73: Changeling -- "This object is every creature type."` + `/// Characteristic-defining ability (CDA). Applied as a type-change in Layer 4` + `/// before non-CDA effects. Functions in all zones (CR 604.3).`

### Step 2: Hash Implementation

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Changeling => 38u8.hash_into(hasher),` at line ~359 (after Undying's discriminant 37)
**Pattern**: Follow `KeywordAbility::Undying` at line 358-359

Also add hash arm for the new `LayerModification::AddAllCreatureTypes` variant (see Step 3).

### Step 3: LayerModification Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/continuous_effect.rs`
**Action**: Add a new `LayerModification::AddAllCreatureTypes` variant in the Layer 4 section (after `LoseAllSubtypes`, line ~139)
**Doc comment**: `/// Adds every creature type from CR 205.3m to the object's subtypes.` + `/// Used by Changeling CDA and effects like Maskwood Nexus.` + `/// No payload needed -- the engine knows the full list.`
**CR**: CR 702.73a, CR 205.3m

**Hash** (`/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`): Add after `SwitchPowerToughness` (discriminant 19, line ~691):
`LayerModification::AddAllCreatureTypes => 20u8.hash_into(hasher),`

### Step 4: Creature Type Constant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add a public function `all_creature_types() -> OrdSet<SubType>` that returns the complete set from CR 205.3m. Use `once_cell::sync::Lazy` or `std::sync::LazyLock` (stable since Rust 1.80) to compute once.

Alternative: Since the engine uses `im::OrdSet`, and constructing 290+ strings per call is expensive, use a `static` `LazyLock<Vec<SubType>>` and clone into an `OrdSet` when needed, or better: just use a `static LazyLock<OrdSet<SubType>>` directly (im::OrdSet is cheaply clonable due to structural sharing).

**Pattern**: New function at the bottom of `types.rs`:
```rust
use std::sync::LazyLock;
use im::OrdSet;

/// All creature types from CR 205.3m.
///
/// Used by Changeling (CR 702.73a) and "is every creature type" effects.
/// Lazily initialized on first use; im::OrdSet clones are O(1) due to
/// structural sharing.
pub static ALL_CREATURE_TYPES: LazyLock<OrdSet<SubType>> = LazyLock::new(|| {
    let types = [
        "Advisor", "Aetherborn", "Alien", "Ally", "Angel", /* ... full list ... */
        "Zombie", "Zubera",
    ];
    types.into_iter().map(|s| SubType(s.to_string())).collect()
});
```

Include the complete list from CR 205.3m (the full list was returned by the MCP lookup above). Include "Time Lord" as a single two-word entry.

### Step 5: Layer System Enforcement

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/layers.rs`
**Action**: Two changes:

**5a)** In `apply_layer_modification` (line ~251), add handling for `LayerModification::AddAllCreatureTypes`:
```rust
LayerModification::AddAllCreatureTypes => {
    for s in crate::state::types::ALL_CREATURE_TYPES.iter() {
        chars.subtypes.insert(s.clone());
    }
}
```
Place after `LoseAllSubtypes` (line ~313).

**5b)** In `calculate_characteristics` (line ~32), at the START of the Layer 4 (`EffectLayer::TypeChange`) processing, check the object's keywords for Changeling and apply the CDA before other Layer 4 effects:

```rust
// Inside the `for &layer in &layers_in_order` loop, before gathering layer_effects:
if layer == EffectLayer::TypeChange {
    // CR 702.73a + CR 613.3: Changeling is a CDA that functions in all zones.
    // Apply before any Layer 4 continuous effects (CDAs apply first per CR 613.3).
    if chars.keywords.contains(&KeywordAbility::Changeling) {
        for s in crate::state::types::ALL_CREATURE_TYPES.iter() {
            chars.subtypes.insert(s.clone());
        }
    }
}
```

This must go BEFORE the `let layer_effects: Vec<...>` gathering and application loop, so that non-CDA Layer 4 effects (like Blood Moon's SetTypeLine) can override the Changeling subtypes if they have a later timestamp -- which is correct per the Maskwood Nexus ruling.

**CR**: 702.73a, 604.3, 613.3, 613.1d

**Note on dependency ordering**: The Changeling CDA adds subtypes. `SetTypeLine` depends on `AddSubtypes` (the dependency `depends_on` function already handles this). Since the Changeling subtypes are applied as an inline mutation (not a ContinuousEffect with dependency tracking), they are naturally applied before non-CDA effects within the layer -- this is correct and matches CR 613.3.

### Step 6: `register_static_continuous_effects` -- is_cda flag support

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/replacement.rs`
**Action**: The current `register_static_continuous_effects` (line ~1175) hardcodes `is_cda: false`. For future CDA-based static abilities (not needed for Changeling since we handle it inline in `calculate_characteristics`), this should eventually be parameterized. However, Changeling does NOT need this change since the CDA is handled in Step 5b.

**No changes needed for this step.** Noted for completeness.

### Step 7: View Model Update

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add match arm in `format_keyword` (line ~562):
```rust
KeywordAbility::Changeling => "Changeling".to_string(),
```
Place after `Undying` (line ~601).

### Step 8: Trigger Wiring

**Not applicable.** Changeling has no triggered or activated ability component. It is a pure static CDA.

### Step 9: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/changeling.rs`
**Tests to write**:

1. **`test_changeling_has_all_creature_types`** -- CR 702.73a
   - Create a creature with `KeywordAbility::Changeling` on the battlefield
   - Call `calculate_characteristics`
   - Assert that `subtypes` contains representative types: "Goblin", "Elf", "Human", "Dragon", "Sliver", "Shapeshifter"
   - Assert count is >= 290 (full list from CR 205.3m)

2. **`test_changeling_lose_all_abilities_keeps_types`** -- CR 613.3, Maskwood Nexus ruling
   - Create a creature with Changeling
   - Add a Layer 6 `RemoveAllAbilities` continuous effect (like Humility)
   - Call `calculate_characteristics`
   - Assert: `keywords` does NOT contain `Changeling` (removed by Layer 6)
   - Assert: `subtypes` DOES contain all creature types (Layer 4 CDA applied before Layer 6)

3. **`test_changeling_overridden_by_set_type_line`** -- Maskwood Nexus ruling, CR 613.3
   - Create a creature with Changeling
   - Add a Layer 4 `SetTypeLine` continuous effect (like Blood Moon making it just a Mountain)
   - Call `calculate_characteristics`
   - Assert: `subtypes` contains ONLY "Mountain" (SetTypeLine non-CDA overrides CDA since SetTypeLine applies after CDA within Layer 4)

4. **`test_changeling_protection_from_subtype_matches`** -- CR 702.16a + 702.73a
   - Create creature A with `ProtectionFrom(FromSubType(SubType("Goblin")))`
   - Create creature B with `Changeling` (attacking creature B, source of potential damage)
   - Verify that protection's `matches_quality` returns true for creature B
   - This tests that Changeling creatures are blocked by "protection from Goblins"

5. **`test_changeling_works_in_graveyard`** -- CR 604.3
   - Create a creature with Changeling in the graveyard
   - Call `calculate_characteristics` on it
   - Assert subtypes contain all creature types (CDA works in all zones)

6. **`test_changeling_negative_no_keyword_no_types`** -- Negative test
   - Create a regular creature WITHOUT Changeling
   - Call `calculate_characteristics`
   - Assert subtypes contains only the printed subtypes (no all-creature-types expansion)

**Pattern**: Follow the layer tests in `/home/airbaggie/scutemob/crates/engine/tests/layers.rs` for setup (`GameStateBuilder`, `add_continuous_effect`, `calculate_characteristics`).

### Step 10: Card Definition (later phase)

**Suggested card**: Universal Automaton (simplest Changeling card -- 1 mana, 1/1 artifact creature)
**Oracle text**: "Changeling (This card is every creature type.)"
**Card lookup**: Use `card-definition-author` agent
**Second card**: Morophon, the Boundless (Changeling + tribal synergy, good for testing)

### Step 11: Game Script (later phase)

**Suggested scenario**: A creature with Changeling attacks. Defending player has "protection from Goblins" on a creature. Verify the Changeling creature cannot be blocked by the protected creature (protection from Goblins matches Changeling because it is a Goblin). Also test interaction with a tribal lord like "other Elves you control get +1/+1" -- the Changeling creature should get the bonus.
**Subsystem directory**: `test-data/generated-scripts/combat/` (protection + changeling) or `test-data/generated-scripts/layers/` (type-changing CDA)

## Interactions to Watch

1. **Layer 4 ordering**: Changeling CDA applies first in Layer 4 (CR 613.3). `SetTypeLine` effects (Blood Moon) apply after CDAs and can override. `AddSubtypes` effects (Urborg) apply after CDAs and add to the set. This is all handled correctly by applying the CDA inline before processing `layer_effects`.

2. **Layer 6 ability removal**: Humility/Dress Down remove Changeling from `keywords`, but this happens in Layer 6 -- subtypes were already set in Layer 4. No interaction bug.

3. **Copy effects (Layer 1)**: Copying a Changeling creature copies the keyword. The copy's `calculate_characteristics` then applies the CDA in Layer 4. Works correctly.

4. **Protection (DEBT)**: All subtype-based protection checks use `chars.subtypes.contains()` on layer-computed characteristics. Since Changeling adds all subtypes in Layer 4, protection checks against any creature type will match Changeling creatures. This affects:
   - `ProtectionQuality::FromSubType` -- will always match a Changeling source
   - Blocking prevention
   - Targeting prevention
   - Damage prevention
   - Enchanting/Equipping prevention

5. **Tribal lords and buffs**: Any `EffectFilter` or `TargetFilter` that checks `has_subtype` will match Changeling creatures. The `effects/mod.rs:1875` filter uses `chars.subtypes.contains(st)` on layer-computed characteristics, so this works automatically.

6. **Commander color identity**: Changeling does not affect color identity. Color identity is based on mana cost, color indicator, and mana symbols in rules text (CR 903.4), not creature types.

7. **Kindred (Tribal) spells**: A Kindred spell with Changeling (e.g., a spell that is "Kindred - Shapeshifter") already has all creature types. The CDA works the same way on non-battlefield objects.

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Changeling` variant + `ALL_CREATURE_TYPES` constant |
| `crates/engine/src/state/hash.rs` | Add hash arms for `Changeling` (38) and `AddAllCreatureTypes` (20) |
| `crates/engine/src/state/continuous_effect.rs` | Add `LayerModification::AddAllCreatureTypes` variant |
| `crates/engine/src/rules/layers.rs` | Inline CDA check at Layer 4 + `AddAllCreatureTypes` application |
| `tools/replay-viewer/src/view_model.rs` | Add `format_keyword` arm for Changeling |
| `crates/engine/tests/changeling.rs` | New test file with 6 tests |
