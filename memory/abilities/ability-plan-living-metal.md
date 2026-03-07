# Ability Plan: Living Metal

**Generated**: 2026-03-07
**CR**: 702.161
**Priority**: P4
**Similar abilities studied**: Impending (Layer 4 inline at `layers.rs:86-110`), Crew (Layer 4 via `AddCardTypes` continuous effect at `abilities.rs:5220-5229`)

## CR Rule Text

> **702.161.** Living Metal
>
> **702.161a** Living metal is a keyword ability found on some Vehicles. "Living metal" means "During your turn, this permanent is an artifact creature in addition to its other types."

Note: The ability-wip.md listed CR 702.176 (which is Impending). The correct CR is **702.161** per MCP lookup.

## Key Edge Cases

- **P/T comes from the Vehicle's printed stats, not from Living Metal.** Living Metal only adds the Creature type. The Vehicle already has printed power/toughness (ruling: "While it's a creature, the Vehicle has its printed power and toughness."). No Layer 7b effect needed.
- **March of the Machines interaction**: A Vehicle with Living Metal and March of the Machines on the battlefield is a creature during your turn (from Living Metal) AND a creature during opponents' turns (from March of the Machines, which makes noncreature artifacts into creatures with P/T = mana value). During your turn, Living Metal's type addition makes March's "noncreature" condition false, so the Vehicle uses its printed P/T. During opponents' turns, March applies (Vehicle is noncreature artifact) and sets P/T to mana value. (Ruling from Arcee, Acrobatic Coupe: "Arcee will be a 2/2 artifact creature during your turn and a 3/3 artifact creature (because its mana value is 3) during each opponent's turn.")
- **"Your turn" means controller's turn**, not owner's turn. Check `state.turn.active_player == obj.controller`.
- **Battlefield only**: Like all static abilities on permanents, Living Metal only functions while the permanent is on the battlefield (CR 611.3b). Not a CDA.
- **Summoning sickness applies**: A Vehicle with Living Metal that entered this turn cannot attack on its controller's turn (it becomes a creature but has summoning sickness). It CAN still be crewed normally.
- **Humility interaction**: If Humility removes all abilities (Layer 6), Living Metal is removed. The inline Layer 4 check should use `chars.keywords` (which reflects Layer 6 removals if Layer 6 ran first). However, since Layer 4 runs BEFORE Layer 6, the keyword will still be present at Layer 4 time. This means Humility does NOT prevent Living Metal from adding the Creature type -- the type was already added in Layer 4 before Humility stripped the keyword in Layer 6. This is correct behavior (same as Changeling CDA surviving Humility).
- **Multiplayer**: "Your turn" is the controller's turn specifically. During other players' turns, the Vehicle is NOT a creature (cannot block, cannot be targeted by "destroy target creature" etc.).
- **SBAs**: When the Vehicle is a creature (controller's turn), SBAs for creatures apply (0 toughness = dies, lethal damage = dies, etc.). When it's not a creature (opponents' turns), creature SBAs don't apply.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (N/A -- static ability, no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::LivingMetal` variant after `UmbraArmor` (line ~1174)
**Discriminant**: 128

```rust
/// CR 702.161a: Living metal -- "During your turn, this permanent is an
/// artifact creature in addition to its other types."
///
/// Static ability on Vehicles. Applied inline in `calculate_characteristics`
/// at Layer 4 (TypeChange). Adds the Creature card type when the active
/// player is the permanent's controller. No Layer 7b needed -- the Vehicle
/// already has printed P/T.
///
/// Discriminant 128.
LivingMetal,
```

**Hash**: Add to `state/hash.rs` `HashInto` impl for `KeywordAbility`, after the `UmbraArmor` arm (line ~637):
```rust
// LivingMetal (discriminant 128) -- CR 702.161
KeywordAbility::LivingMetal => 128u8.hash_into(hasher),
```

**Match arms to update**:
1. `tools/replay-viewer/src/view_model.rs` -- `format_keyword` function (after `UmbraArmor` arm, line ~844):
   ```rust
   KeywordAbility::LivingMetal => "Living Metal".to_string(),
   ```

### Step 2: Rule Enforcement (Layer 4 Inline)

**File**: `crates/engine/src/rules/layers.rs`
**Action**: Add an inline check in `calculate_characteristics` at Layer 4 (`EffectLayer::TypeChange`), right after the Impending block (line ~110). This follows the exact same pattern as Impending -- a conditional type modification inline in the layer loop.

**CR**: 702.161a -- "During your turn, this permanent is an artifact creature in addition to its other types."

**Pattern**: Follow Impending at lines 86-110 of `layers.rs`.

**Logic**:
```rust
// CR 702.161a: Living Metal -- "During your turn, this permanent is
// an artifact creature in addition to its other types."
// Applied at Layer 4 (TypeChange) inline, after CDAs, before non-CDA
// Layer 4 effects. Like Impending, this is a static ability that
// modifies types conditionally. The condition is:
// (1) permanent is on the battlefield, AND
// (2) the active player is the permanent's controller.
// Uses chars.keywords (pre-Layer-6) so the check runs at Layer 4 time
// before Humility could strip it. This is intentionally correct: Layer 4
// runs before Layer 6, so Living Metal adds Creature before Humility
// removes abilities. Same behavior as Changeling CDA.
if layer == EffectLayer::TypeChange
    && chars.keywords.contains(&KeywordAbility::LivingMetal)
{
    if let Some(obj_ref) = state.objects.get(&object_id) {
        if obj_ref.zone == ZoneId::Battlefield
            && state.turn.active_player == obj_ref.controller
        {
            chars.card_types.insert(CardType::Creature);
        }
    }
}
```

**Placement**: After the Impending block (line ~110) and before the `let layer_effects` gathering (line ~112). Both Impending and Living Metal are inline Layer 4 checks that run after CDAs but before non-CDA continuous effects.

### Step 3: Trigger Wiring

**N/A** -- Living Metal is a static ability generating a continuous effect. No triggers, no stack objects, no `PendingTriggerKind`, no `StackObjectKind` variant needed.

### Step 4: Unit Tests

**File**: `crates/engine/tests/living_metal.rs`
**Tests to write**:

1. **`test_living_metal_creature_during_controller_turn`** -- CR 702.161a basic. Place a Vehicle with Living Metal on the battlefield. On the controller's turn, `calculate_characteristics` returns `card_types` containing `Creature`. Verify the Vehicle's printed P/T is present.

2. **`test_living_metal_not_creature_during_opponent_turn`** -- CR 702.161a negative. Same Vehicle, but when `active_player` is a different player. `calculate_characteristics` returns `card_types` without `Creature` (still has `Artifact`).

3. **`test_living_metal_retains_other_types`** -- CR 702.161a "in addition to its other types." The Vehicle retains Artifact (and any subtypes like "Vehicle") when Creature is added.

4. **`test_living_metal_not_on_battlefield`** -- Living Metal should NOT add Creature type when the object is not on the battlefield (e.g., in hand, graveyard, exile). Only functions on battlefield per CR 611.3b.

5. **`test_living_metal_uses_printed_pt`** -- Verify that the Vehicle's printed P/T is used when it becomes a creature. No special P/T setting needed.

6. **`test_living_metal_multiplayer_only_controller_turn`** -- In a 4-player game, verify the Vehicle is only a creature during its controller's turn, not during any other player's turn.

7. **`test_living_metal_with_crew`** -- A Vehicle with both Living Metal and Crew. During the controller's turn, it's already a creature (from Living Metal) without needing Crew. Crew can still be activated to add a redundant continuous effect.

**Pattern**: Follow tests in `crates/engine/tests/impending.rs` for structure (helpers, builder setup, `calculate_characteristics` assertions).

### Step 5: Card Definition (later phase)

**Suggested card**: Arcee, Acrobatic Coupe (the back face of Arcee, Sharpshooter). This is the simplest Living Metal card to model as a standalone (ignoring the double-faced aspect). Type: Legendary Artifact -- Vehicle, 2/2, First Strike, Living Metal. However, since ALL Living Metal cards are double-faced Transformers cards, the card definition will need to model just the Vehicle face as a standalone card (the engine does not yet support DFCs).

**Alternative**: Create a test-only synthetic card "Living Metal Test Vehicle" -- a simple `{3}` Artifact -- Vehicle 3/3 with Living Metal. This avoids the DFC complexity entirely and is sufficient for validation.

**Card lookup**: Use `card-definition-author` agent when ready.

### Step 6: Game Script (later phase)

**Suggested scenario**: A 4-player game where a Vehicle with Living Metal is on the battlefield. During the controller's turn, it attacks as a creature. During opponents' turns, it cannot be targeted by "destroy target creature" effects. The turn cycles through multiple players to verify the type toggling.

**Subsystem directory**: `test-data/generated-scripts/layers/` (type-changing continuous effect)

## Interactions to Watch

- **Layer 4 ordering**: Living Metal runs inline before non-CDA Layer 4 effects. If Blood Moon or a `SetTypeLine` effect also applies, the continuous effect ordering (timestamp/dependency) handles it after the inline check. Blood Moon would set the type line to "Mountain" (removing Creature), but Blood Moon applies to nonbasic lands, not artifacts -- so this interaction is unlikely unless the Vehicle is also a land.
- **Humility (Layer 6)**: Humility removes all abilities in Layer 6, including Living Metal. But since Layer 4 runs before Layer 6, the Creature type was already added. The keyword removal in Layer 6 is too late to undo the Layer 4 type addition. This is correct per layer ordering rules.
- **Dress Down (Layer 6)**: Same as Humility -- removes abilities but Layer 4 already ran.
- **SBAs during turn transitions**: When the turn changes from the controller's turn to another player's turn, the Vehicle stops being a creature. If it had damage marked on it, that damage stays but is no longer relevant (damage on a non-creature permanent doesn't cause it to be destroyed by SBAs). If it had -1/-1 counters reducing toughness to 0, the SBA wouldn't fire during opponents' turns because it's not a creature.
- **Combat**: The Vehicle can only attack/block during its controller's turn (when it's a creature). It cannot block during opponents' turns because it's not a creature then.
- **Auras/Equipment**: An Aura with "Enchant creature" that's attached when the Vehicle is a creature would cause an SBA to put the Aura into the graveyard when the turn changes and the Vehicle stops being a creature (CR 704.5m -- Aura enchanting illegal object).
