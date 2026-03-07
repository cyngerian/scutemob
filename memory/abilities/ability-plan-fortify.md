# Ability Plan: Fortify

**Generated**: 2026-03-07
**CR**: 702.67
**Priority**: P4
**Similar abilities studied**: Equip (CR 702.6) — `types.rs:224`, `abilities.rs:133-183`, `effects/mod.rs:1964-2060`, `card_definition.rs:834-849`, `tests/equip.rs`

## CR Rule Text

702.67. Fortify

702.67a Fortify is an activated ability of Fortification cards. "Fortify [cost]" means "[Cost]: Attach this Fortification to target land you control. Activate only as a sorcery."

702.67b For more information about Fortifications, see rule 301, "Artifacts."

702.67c If a Fortification has multiple instances of fortify, any of its fortify abilities may be used.

### Related Rules

- CR 301.6: "Some artifacts have the subtype 'Fortification.' A Fortification can be attached to a land. It can't legally be attached to an object that isn't a land. Fortification's analog to the equip keyword ability is the fortify keyword ability. Rules 301.5a-f apply to Fortifications in relation to lands just as they apply to Equipment in relation to creatures, with one clarification relating to rule 301.5c: a Fortification that's also a creature (not a land) can't fortify a land."
- CR 704.5n: "If an Equipment or Fortification is attached to an illegal permanent, it becomes unattached." (Already implemented in `sba.rs:909-1009`.)

## Key Edge Cases

- **Fortification can't fortify itself** (CR 301.5c analog via 301.6): A Fortification that's also a creature can't fortify a land. A Fortification can't fortify itself (trivially true since it's an artifact, not a land).
- **Target must be a land you control** (CR 702.67a): Parallels Equip's "target creature you control."
- **Sorcery speed only** (CR 702.67a): "Activate only as a sorcery."
- **Multiple instances** (CR 702.67c): If a Fortification has multiple fortify abilities, any may be used. Each is a separate activated ability.
- **Unattach from previous land** (CR 301.5c analog via 301.6): A Fortification can't be attached to more than one land. Attaching to a new land detaches from the old one.
- **SBA already implemented**: `check_equipment_sbas` in `sba.rs` already checks Fortification subtype and validates target is a Land (line 963-966).
- **Phasing already handled**: `turn_actions.rs:949` already phases out attached Fortifications indirectly.
- **"Fortified land" in card text**: Darksteel Garrison uses "fortified land" — needs `EffectFilter::AttachedLand` (analogous to `EffectFilter::AttachedCreature`).

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Fortify` variant after `Soulbond` (line ~1194).
**Discriminant**: 130
**Doc comment**: `/// CR 702.67a: Fortify [cost] -- "[Cost]: Attach this Fortification to target land you control. Activate only as a sorcery." Discriminant 130.`
**Pattern**: Follow `KeywordAbility::Equip` at line 224 — simple unit variant, no parameters.

**Hash**: In `crates/engine/src/state/hash.rs`, add arm to `KeywordAbility` match:
```
KeywordAbility::Fortify => 130u8.hash_into(hasher),
```
Follow the pattern at line 329 (`KeywordAbility::Equip => 4u8.hash_into(hasher)`).

**Match arms to update**:
1. `tools/replay-viewer/src/view_model.rs` — `KeywordAbility` display match: add `KeywordAbility::Fortify => "Fortify".to_string()`

### Step 2: Effect Variant — `Effect::AttachFortification`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Effect::AttachFortification { equipment: EffectTarget, target: EffectTarget }` variant near `AttachEquipment` (line ~844).
**CR**: 702.67a / 701.3a — Attach the source Fortification to the target land.
**Doc comment**: Mirror the `AttachEquipment` doc but reference CR 702.67a and "target land you control" instead of "target creature you control."
**No new AbilityDefinition discriminant needed**: Fortify uses `AbilityDefinition::Activated` (like Equip), not a dedicated AbilityDefinition variant.
**No new StackObjectKind needed**: The activated ability goes on the stack as a regular `StackObjectKind::ActivatedAbility`.

**Hash**: In `crates/engine/src/state/hash.rs`, add arm to `Effect` match for `AttachFortification`. Follow the `AttachEquipment` pattern — hash a new discriminant for the Effect enum, then hash the two EffectTarget fields. Find the last Effect discriminant and use the next one.

### Step 3: EffectFilter::AttachedLand

**File**: `crates/engine/src/state/continuous_effect.rs`
**Action**: Add `AttachedLand` variant to `EffectFilter` enum (after `AttachedCreature`, line ~91).
**Doc comment**: `/// Resolved at characteristic-calculation time: the source object's 'attached_to' field points to the target land. Used for Fortification static abilities such as Darksteel Garrison ("fortified land has indestructible").`

**File**: `crates/engine/src/rules/layers.rs`
**Action**: Add match arm for `EffectFilter::AttachedLand` in the filter-matching function (after `AttachedCreature` at line ~333). Logic is identical to `AttachedCreature` — check `effect.source -> attached_to == object_id`. No card-type check needed (the SBA already ensures Fortifications are only attached to lands).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `EffectFilter::AttachedLand => 13u8.hash_into(hasher)` to the `EffectFilter` hash match (after line 964).

### Step 4: Effect Execution — `AttachFortification`

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add handler for `Effect::AttachFortification` near `Effect::AttachEquipment` (line ~1964). The logic mirrors `AttachEquipment` exactly with these differences:
- Target validation: check `CardType::Land` instead of `CardType::Creature` (line ~2012)
- Error message: "fortify target must be a land" instead of "equip target must be a creature"
- CR citation: CR 702.67a instead of CR 702.6a

**Pattern**: Copy the `AttachEquipment` handler (lines 1964-2060) and change:
1. `CardType::Creature` -> `CardType::Land`
2. All CR citations from 702.6 to 702.67
3. Comments from "equip"/"equipment" to "fortify"/"fortification"

### Step 5: Activation Validation

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add a parallel validation block for `AttachFortification` (after the `AttachEquipment` block at line ~140-183). The block should:
1. Match `Effect::AttachFortification { .. }` in the effect check
2. Validate target is a **land** on the battlefield controlled by the activating player
3. Use layer-computed characteristics for the land type check (same pattern as Equip)
4. Error messages: "fortify target must be a land you control on the battlefield", "fortify target must be a land"

**CR**: 702.67a — "target land you control"

### Step 6: Unit Tests

**File**: `crates/engine/tests/fortify.rs` (new file)
**Tests to write**:

- `test_fortify_basic_attaches_to_land` — CR 702.67a: Activate Fortify, pay cost, Fortification attaches to target land. Verify `attached_to` and `attachments` fields. Pattern: follow `test_equip_basic_attaches_to_creature` in `tests/equip.rs`.

- `test_fortify_sorcery_speed_only` — CR 702.67a: Cannot activate Fortify during opponent's turn or with items on stack. Expect `GameStateError`. Pattern: follow `test_equip_sorcery_speed_only` in `tests/equip.rs`.

- `test_fortify_target_must_be_land` — CR 702.67a: Attempting to fortify a creature (not a land) returns error. Expect `GameStateError::InvalidTarget`.

- `test_fortify_target_must_be_controlled` — CR 702.67a: Cannot fortify an opponent's land. Expect error.

- `test_fortify_moves_between_lands` — CR 301.6 (via 301.5c analog): Fortifying a new land detaches from the old one. Verify old land's `attachments` is empty, new land has the Fortification.

- `test_fortify_sba_unattaches_from_nonland` — CR 704.5n: If the fortified permanent stops being a land, the Fortification becomes unattached. (SBA already implemented; this test validates the existing code path for Fortifications specifically.)

- `test_fortify_static_ability_grants_to_land` — CR 301.6 + 604.2: A Fortification with a static ability (e.g., "fortified land has indestructible") grants that ability to the attached land via `EffectFilter::AttachedLand`. Uses `calculate_characteristics` to verify.

**Test helper**: `fortify_ability(generic_mana: u32) -> ActivatedAbility` — mirrors `equip_ability()` in `tests/equip.rs` but uses `Effect::AttachFortification` instead of `Effect::AttachEquipment`.

**Pattern**: Follow `tests/equip.rs` structure closely.

### Step 7: Card Definition (later phase)

**Suggested card**: Darksteel Garrison
- Artifact -- Fortification, {2}
- "Fortified land has indestructible."
- "Whenever fortified land becomes tapped, target creature gets +1/+1 until end of turn."
- Fortify {3}

**Card lookup result**:
- Mana Cost: {2}
- Type: Artifact -- Fortification
- Oracle Text: Fortified land has indestructible. / Whenever fortified land becomes tapped, target creature gets +1/+1 until end of turn. / Fortify {3}

**Card definition approach**:
```rust
abilities: vec![
    // Static: fortified land has indestructible
    AbilityDefinition::Static {
        continuous_effect: ContinuousEffectDef {
            layer: EffectLayer::Ability,
            modification: LayerModification::AddKeywords(
                [KeywordAbility::Indestructible].into_iter().collect(),
            ),
            filter: EffectFilter::AttachedLand,
            duration: EffectDuration::WhileSourceOnBattlefield,
        },
    },
    // Triggered: whenever fortified land becomes tapped (may need new TriggerEvent)
    // NOTE: This trigger ("whenever fortified land becomes tapped") requires
    // TriggerEvent infrastructure that may not exist yet. Defer or implement
    // as a simplified version for the initial card definition.
    // Fortify {3}
    AbilityDefinition::Activated {
        cost: Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
        effect: Effect::AttachFortification {
            equipment: EffectTarget::Source,
            target: EffectTarget::DeclaredTarget { index: 0 },
        },
        timing_restriction: Some(TimingRestriction::SorcerySpeed),
    },
],
```

**File**: `crates/engine/src/cards/defs/darksteel_garrison.rs`
**Agent**: Use `card-definition-author` agent.

### Step 8: Game Script (later phase)

**Suggested scenario**: Fortify activation and land attachment
**Subsystem directory**: `test-data/generated-scripts/baseline/`
**Script**: Player casts Darksteel Garrison (artifact), then activates Fortify {3} targeting a basic land. Verify the land gains indestructible. Optionally, a Wrath of God or similar "destroy all" effect to confirm the fortified land survives.

## Interactions to Watch

- **SBA for Fortification already exists** (`sba.rs:909-1009`): No new SBA work needed. The existing `check_equipment_sbas` already handles Fortification subtype checking `CardType::Land` (line 963-966).
- **Phasing already handles Fortifications** (`turn_actions.rs:949`): Fortifications phase out/in with their attached land indirectly.
- **Protection**: The existing protection check in `check_equipment_sbas` (line 971-977) applies to Fortifications too — if a land has protection from a quality the Fortification matches, the Fortification becomes unattached.
- **`EffectFilter::AttachedCreature` vs `AttachedLand`**: The existing `AttachedCreature` filter has no type check (it just checks `attached_to == object_id`), so it would technically work for Fortifications. However, adding a distinct `AttachedLand` variant is cleaner for semantic clarity and matches the Equipment/Fortification split in the rules.
- **No trigger wiring needed for the keyword itself**: Fortify is purely an activated ability. No triggers fire on attach/detach (those would be card-specific, like Darksteel Garrison's "whenever fortified land becomes tapped").
- **Multiplayer**: No special multiplayer considerations. "Target land you control" is controller-specific, same as Equip.

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Fortify` (disc 130) |
| `crates/engine/src/state/hash.rs` | Add hash arms for `Fortify`, `AttachFortification`, `AttachedLand` |
| `crates/engine/src/state/continuous_effect.rs` | Add `EffectFilter::AttachedLand` variant |
| `crates/engine/src/cards/card_definition.rs` | Add `Effect::AttachFortification` variant |
| `crates/engine/src/effects/mod.rs` | Add `AttachFortification` effect handler |
| `crates/engine/src/rules/abilities.rs` | Add activation validation for `AttachFortification` |
| `crates/engine/src/rules/layers.rs` | Add `AttachedLand` filter matching |
| `tools/replay-viewer/src/view_model.rs` | Add `KeywordAbility::Fortify` display arm |
| `crates/engine/tests/fortify.rs` | New test file with 7 tests |
