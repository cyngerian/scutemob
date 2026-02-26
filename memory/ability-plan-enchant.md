# Ability Plan: Enchant

**Generated**: 2026-02-26
**CR**: 702.5
**Priority**: P1
**Similar abilities studied**: Equip (702.6) — `rules/abilities.rs:118-164`, `effects/mod.rs:1020-1118`, `tests/equip.rs`; existing Aura SBA in `rules/sba.rs:572-660`

## CR Rule Text

### CR 702.5 — Enchant

- **702.5a** Enchant is a static ability, written "Enchant [object or player]." The enchant ability restricts what an Aura spell can target and what an Aura can enchant.
- **702.5b** For more information about Auras, see rule 303, "Enchantments."
- **702.5c** If an Aura has multiple instances of enchant, all of them apply. The Aura's target must follow the restrictions from all the instances of enchant. The Aura can enchant only objects or players that match all of its enchant abilities.
- **702.5d** Auras that can enchant a player can target and be attached to players. Such Auras can't target permanents and can't be attached to permanents.

### CR 303.4 — Aura Rules

- **303.4a** An Aura spell requires a target, which is defined by its enchant ability.
- **303.4b** The object or player an Aura is attached to is called enchanted. The Aura is attached to, or "enchants," that object or player.
- **303.4c** If an Aura is enchanting an illegal object or player as defined by its enchant ability and other applicable effects, the object it was attached to no longer exists, or the player it was attached to has left the game, the Aura is put into its owner's graveyard. (This is a state-based action. See rule 704.)
- **303.4d** An Aura can't enchant itself. An Aura that's also a creature can't enchant anything. An Aura can't enchant more than one object or player.
- **303.4e** An Aura's controller is separate from the enchanted object's controller.
- **303.4f** If an Aura is entering the battlefield under a player's control by any means other than by resolving as an Aura spell, and the effect putting it onto the battlefield doesn't specify the object or player the Aura will enchant, that player chooses what it will enchant as the Aura enters the battlefield. The player must choose a legal object or player according to the Aura's enchant ability.
- **303.4g** If an Aura is entering the battlefield and there is no legal object or player for it to enchant, the Aura remains in its current zone, unless that zone is the stack. In that case, the Aura is put into its owner's graveyard instead of entering the battlefield.
- **303.4i** If an effect attempts to put an Aura onto the battlefield attached to either an object or player it can't legally enchant or an object or player that is undefined, the Aura remains in its current zone, unless that zone is the stack. In that case, the Aura is put into its owner's graveyard.
- **303.4j** If an effect attempts to attach an Aura on the battlefield to an object or player it can't legally enchant, the Aura doesn't move.

### CR 704.5m — Aura SBA

- **704.5m** If an Aura is attached to an illegal object or player, or is not attached to an object or player, that Aura is put into its owner's graveyard.

## Key Edge Cases

- **Aura fizzle (Rancor ruling)**: If the creature an Aura spell would enchant is an illegal target at resolution, the Aura spell fizzles. It goes to the graveyard as a spell that didn't resolve, NOT as "from the battlefield." (Rancor's return-to-hand trigger does not fire in this case.)
- **Enchant creature vs Enchant permanent vs Enchant land**: The `EnchantTarget` enum must be extensible. Most common Auras use "Enchant creature", but some use "Enchant land", "Enchant permanent", "Enchant artifact", or even "Enchant player."
- **Self-enchantment prohibited (303.4d)**: An Aura can't enchant itself. If this occurs somehow, the Aura is put into its owner's graveyard as an SBA.
- **Enchant restriction checked at TWO points**: (1) At cast time when targets are chosen (CR 601.2c via 303.4a), and (2) continuously as an SBA (CR 704.5m via 303.4c) — if the target loses the required quality, the Aura falls off.
- **Multiple Enchant restrictions (702.5c)**: If an Aura has multiple Enchant abilities, all must be satisfied simultaneously. Not relevant for MVP but the data model should not preclude it.
- **Non-cast ETB (303.4f)**: Auras entering the battlefield without being cast (e.g., from graveyard via Replenish) must choose a legal target on entry or fail to enter. Deferred — not in scope for this implementation.
- **Protection interaction (702.16c)**: Already handled in existing SBA code (`attachment_is_illegal_due_to_protection`). Enchant implementation should not break this.
- **Multiplayer**: No special multiplayer considerations — an Aura can target/enchant any legal object on the battlefield regardless of controller.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant — `KeywordAbility::Enchant` EXISTS at `state/types.rs:124`; hash at `state/hash.rs:279`. BUT it has no payload (no `EnchantTarget` to specify what it can enchant). The `enchants_creatures` boolean on `GameObject` is a workaround.
- [ ] 2. Rule enforcement — PARTIAL. Aura SBA (704.5m) exists in `rules/sba.rs:572-660` but uses only the `enchants_creatures` boolean. No Enchant restriction enforcement at cast time. No Aura attachment on resolution.
- [ ] 3. Trigger wiring — N/A (Enchant is a static ability, not a trigger)
- [ ] 4. Unit tests — Existing tests for SBA (`test_sba_704_5m_unattached_aura_goes_to_graveyard`, `test_cc31_aura_falls_off_after_type_change_ends`), protection interaction (`test_protection_from_red_aura_falls_off`). Missing: cast-time targeting validation, resolution attachment, Enchant restriction with typed EnchantTarget.
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Add `EnchantTarget` Enum and Refactor `KeywordAbility::Enchant`

**File**: `crates/engine/src/state/types.rs`
**Action**: Create an `EnchantTarget` enum to specify what an Aura can legally enchant. Change `KeywordAbility::Enchant` from a bare variant to `Enchant(EnchantTarget)`.

```rust
/// CR 702.5a: What an Aura can legally target and enchant.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EnchantTarget {
    /// "Enchant creature" (most common)
    Creature,
    /// "Enchant permanent"
    Permanent,
    /// "Enchant artifact"
    Artifact,
    /// "Enchant enchantment"
    Enchantment,
    /// "Enchant land"
    Land,
    /// "Enchant planeswalker"
    Planeswalker,
    /// "Enchant player"
    Player,
    /// "Enchant creature or planeswalker"
    CreatureOrPlaneswalker,
}
```

Change `Enchant,` to `Enchant(EnchantTarget),` at line 124.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Update the `HashInto` impl for `KeywordAbility::Enchant` at line 279 to hash the `EnchantTarget` payload. Add a `HashInto` impl for `EnchantTarget`.
**Pattern**: Follow `Landwalk(LandwalkType)` hashing pattern.

**File**: All match arms on `KeywordAbility`
**Action**: Grep for `KeywordAbility::Enchant` across the codebase and update all match arms from `Enchant` to `Enchant(_)` or `Enchant(target)` as appropriate.
Expected files:
- `state/hash.rs:279`
- Any display/debug code
- `testing/replay_harness.rs` (in `enrich_spec_from_def` keyword propagation)

### Step 2: Remove `enchants_creatures` Boolean from `GameObject`

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Remove the `enchants_creatures: bool` field from `GameObject` (lines 252-261). The `EnchantTarget` on the Aura's `KeywordAbility::Enchant(target)` replaces this.

**IMPORTANT**: All code that reads `obj.enchants_creatures` must be refactored to instead look up the `KeywordAbility::Enchant(target)` from the object's keywords (layer-computed characteristics).

**Files to update** (grep for `enchants_creatures`):
- `state/game_object.rs:261` — remove field
- `state/hash.rs:464` — remove from HashInto
- `state/mod.rs:274` — remove from `move_object_to_zone` new object construction
- `state/mod.rs:344` — same (second move path)
- `state/builder.rs:447` — remove from ObjectSpec build
- `effects/mod.rs:1521` — remove from token creation
- `rules/sba.rs:600,615` — replace with keyword lookup (see Step 3)
- `tests/sba.rs:1235` — update test to set keyword instead of boolean
- `tests/snapshot_perf.rs:127` — remove field
- `tests/commander_damage.rs:415` — remove field

### Step 3: Generalize Aura SBA to Use `EnchantTarget`

**File**: `crates/engine/src/rules/sba.rs`
**Action**: Refactor `check_aura_sbas` (lines 572-660) to derive the enchant restriction from the Aura's keywords instead of the removed `enchants_creatures` boolean.

**New logic in `check_aura_sbas`**:
1. For each Aura on the battlefield, compute its characteristics via `calculate_characteristics`.
2. Find `KeywordAbility::Enchant(target)` in its computed keywords.
3. If `attached_to` is `None` -> illegal (unchanged).
4. If `attached_to` target is gone/not on battlefield -> illegal (unchanged).
5. Match the `EnchantTarget` variant to validate the target's type:
   - `EnchantTarget::Creature` -> target must have `CardType::Creature`
   - `EnchantTarget::Land` -> target must have `CardType::Land`
   - `EnchantTarget::Artifact` -> target must have `CardType::Artifact`
   - `EnchantTarget::Enchantment` -> target must have `CardType::Enchantment`
   - `EnchantTarget::Planeswalker` -> target must have `CardType::Planeswalker`
   - `EnchantTarget::Permanent` -> target must be on battlefield (any type)
   - `EnchantTarget::Player` -> deferred (player attachment not yet supported)
   - `EnchantTarget::CreatureOrPlaneswalker` -> target must be creature or planeswalker
6. Protection check (unchanged — already uses `attachment_is_illegal_due_to_protection`).

**CR**: 704.5m / 303.4c — enchant restriction checked as SBA.

**Helper function** to extract:
```rust
/// Extract the EnchantTarget from an Aura's computed keywords.
/// Returns None if the object has no Enchant keyword.
fn get_enchant_target(keywords: &OrdSet<KeywordAbility>) -> Option<&EnchantTarget> {
    keywords.iter().find_map(|kw| {
        if let KeywordAbility::Enchant(target) = kw {
            Some(target)
        } else {
            None
        }
    })
}
```

**Validation helper**:
```rust
/// CR 702.5a: Check if a target object matches the Enchant restriction.
/// target_chars are the layer-computed characteristics of the attached object.
fn matches_enchant_target(enchant: &EnchantTarget, target_chars: &Characteristics) -> bool {
    match enchant {
        EnchantTarget::Creature => target_chars.card_types.contains(&CardType::Creature),
        EnchantTarget::Land => target_chars.card_types.contains(&CardType::Land),
        EnchantTarget::Artifact => target_chars.card_types.contains(&CardType::Artifact),
        EnchantTarget::Enchantment => target_chars.card_types.contains(&CardType::Enchantment),
        EnchantTarget::Planeswalker => target_chars.card_types.contains(&CardType::Planeswalker),
        EnchantTarget::Permanent => true, // any permanent type
        EnchantTarget::Player => false, // can't be attached to an object (needs player)
        EnchantTarget::CreatureOrPlaneswalker => {
            target_chars.card_types.contains(&CardType::Creature)
                || target_chars.card_types.contains(&CardType::Planeswalker)
        }
    }
}
```

Place both helpers in `rules/sba.rs` (private to module). The `matches_enchant_target` helper will also be used by casting validation in Step 4.

### Step 4: Enforce Enchant Restriction at Cast Time

**File**: `crates/engine/src/rules/casting.rs`
**Action**: When an Aura spell is cast, the `TargetRequirement` must enforce the Enchant restriction from the card's `KeywordAbility::Enchant(target)`.

**Approach A (preferred — simpler)**: Add Aura-specific target validation in `handle_cast_spell` after the general target validation. When the spell being cast is an Aura (check `subtypes` for "Aura"), look up its `Enchant` keyword ability and validate that each target matches the enchant restriction.

**Approach B (alternative)**: Add new `TargetRequirement` variants for Aura targets (`TargetRequirement::EnchantCreature`, etc.) and set them on Aura card definitions. This couples the restriction to the card definition DSL.

**Recommended: Approach A** because it automatically derives the restriction from the keyword, which is how the CR works (the Enchant keyword defines the targeting restriction — 702.5a, 303.4a).

**Implementation detail**:
After `validate_targets` succeeds (line ~202), add a block:
```rust
// CR 702.5a / 303.4a: Aura spell target must match the Enchant restriction.
if chars.subtypes.contains(&SubType("Aura".to_string())) {
    if let Some(enchant_target) = get_enchant_target(&chars.keywords) {
        for target in &spell_targets {
            if let Target::Object(target_id) = target.target {
                let target_chars = calculate_characteristics(state, target_id)
                    .or_else(|| state.objects.get(&target_id).map(|o| o.characteristics.clone()));
                if let Some(tc) = target_chars {
                    if !matches_enchant_target(enchant_target, &tc) {
                        return Err(GameStateError::InvalidTarget(
                            format!("target does not match Enchant restriction ({:?})", enchant_target)
                        ));
                    }
                }
            }
        }
    }
}
```

**Note**: The helper functions `get_enchant_target` and `matches_enchant_target` should be made `pub(crate)` and placed in a shared location (either `rules/sba.rs` exported, or a new small helper in `rules/mod.rs`) so both `casting.rs` and `sba.rs` can use them.

**Also needed in casting.rs**: Aura spells MUST have a target. If an Aura spell is cast with zero targets, it should be rejected. Currently the casting code's `TargetRequirement` comes from the `AbilityDefinition::Spell { targets, .. }` — but Aura spells are permanents, not instants/sorceries. They don't have a `Spell` ability definition; their target requirement comes from the Enchant keyword.

**Critical addition**: For Aura cards, `handle_cast_spell` must derive a `TargetRequirement` from the Enchant keyword if the card definition has no explicit `Spell` targets. The target count should be exactly 1 (each Aura requires exactly one target). Map `EnchantTarget::Creature` to `TargetRequirement::TargetCreature`, `EnchantTarget::Permanent` to `TargetRequirement::TargetPermanent`, etc.

Alternatively, the `CardDefinition` for each Aura card should explicitly include the target requirement in a new variant. **Recommended**: Derive it from the keyword in `handle_cast_spell` to avoid redundancy.

### Step 5: Attach Aura to Target on Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: After an Aura permanent spell resolves and enters the battlefield (in the `is_permanent` branch, after `move_object_to_zone`), attach it to the spell's target.

**New code block** (after `obj.controller = controller;` at line ~178):
```rust
// CR 303.4a / 303.4b: If the resolved permanent is an Aura, attach it to its target.
let is_aura = card_types.iter().any(|t| matches!(t, CardType::Enchantment))
    && state.objects.get(&new_id)
        .map(|o| o.characteristics.subtypes.contains(&SubType("Aura".to_string())))
        .unwrap_or(false);

if is_aura {
    // The Aura's target is the first legal target from the stack object.
    let aura_target = stack_obj.targets.iter()
        .filter(|t| is_target_legal(state, t))
        .find_map(|t| {
            if let Target::Object(target_id) = t.target {
                Some(target_id)
            } else {
                None
            }
        });

    if let Some(target_id) = aura_target {
        // Set attached_to on the Aura.
        if let Some(aura_obj) = state.objects.get_mut(&new_id) {
            aura_obj.attached_to = Some(target_id);
        }
        // Add to target's attachments list.
        if let Some(target_obj) = state.objects.get_mut(&target_id) {
            if !target_obj.attachments.contains(&new_id) {
                target_obj.attachments.push_back(new_id);
            }
        }
        events.push(GameEvent::AuraAttached {
            aura_id: new_id,
            target_id,
            player: controller,
        });
    }
    // If no legal target exists, the Aura is unattached. SBA 704.5m will handle it.
}
```

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add a new `GameEvent::AuraAttached` variant (analogous to `EquipmentAttached`):
```rust
/// An Aura resolved and was attached to its target (CR 303.4a, 303.4b).
AuraAttached {
    aura_id: ObjectId,
    target_id: ObjectId,
    player: PlayerId,
},
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `AuraAttached` to the `GameEvent` HashInto impl (follow `EquipmentAttached` pattern).

### Step 6: Update `enrich_spec_from_def` for Aura Cards

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In `enrich_spec_from_def`, when propagating keywords, ensure `KeywordAbility::Enchant(target)` is carried through. Also propagate subtypes from the card definition (currently subtypes are not enriched from the definition, which means Aura cards built with `ObjectSpec::card()` lack the "Aura" subtype needed for SBA checks).

Add subtype propagation:
```rust
// Apply subtypes (Aura, Equipment, etc.)
if !def.types.subtypes.is_empty() {
    spec.subtypes = def.types.subtypes.iter().cloned().collect();
}
```

This ensures that when game scripts use `ObjectSpec::card(p1, "Pacifism")` with `enrich_spec_from_def`, the resulting object has the `Aura` subtype and the `Enchant(Creature)` keyword.

### Step 7: Unit Tests

**File**: `crates/engine/tests/enchant.rs` (NEW FILE)
**Tests to write**:

1. **`test_enchant_aura_spell_requires_target_creature`** — CR 303.4a / 702.5a: Cast an "Enchant creature" Aura with a creature target; succeeds. Cast with a non-creature target; fails with `InvalidTarget`.

2. **`test_enchant_aura_spell_resolves_and_attaches`** — CR 303.4b: Cast an Aura targeting a creature, pass priority to resolve, verify the Aura has `attached_to` set and the creature has the Aura in `attachments`. Verify `AuraAttached` event is emitted.

3. **`test_enchant_aura_fizzles_when_target_illegal_at_resolution`** — CR 608.2b: Cast an Aura targeting a creature, kill the creature before resolution, verify the Aura fizzles (`SpellFizzled` event) and goes to graveyard (not battlefield).

4. **`test_enchant_sba_aura_on_non_creature_falls_off`** — CR 704.5m: Place an Aura with `Enchant(Creature)` attached to a land via manual state setup. Run SBAs. Verify `AuraFellOff` event. (Replaces/refactors existing `test_cc31_aura_falls_off_after_type_change_ends` to use `EnchantTarget` instead of `enchants_creatures` boolean.)

5. **`test_enchant_sba_enchant_land_aura_on_creature_falls_off`** — CR 704.5m: An "Enchant land" Aura attached to a creature falls off.

6. **`test_enchant_sba_enchant_permanent_stays_on_creature`** — CR 704.5m: An "Enchant permanent" Aura attached to a creature stays (not illegal).

7. **`test_enchant_casting_rejected_without_target`** — CR 303.4a: An Aura spell cast with no targets should be rejected.

8. **`test_enchant_continuous_effect_on_attached_creature`** — Aura with a static ability (e.g., Pacifism) grants the effect to the enchanted creature. Verify layer-computed characteristics of the attached creature reflect the Aura's effect.

**Pattern**: Follow `tests/equip.rs` structure (GameStateBuilder with card registry, ObjectSpec for hand/battlefield, CastSpell + pass priority for resolution).

### Step 8: Card Definition (later phase)

**Suggested card**: Pacifism (Enchant creature, {1W})
**Oracle text**: "Enchant creature. Enchanted creature can't attack or block."

**File**: `crates/engine/src/cards/definitions.rs`
**Definition sketch**:
```rust
CardDefinition {
    card_id: cid("pacifism"),
    name: "Pacifism".to_string(),
    mana_cost: Some(ManaCost { white: 1, generic: 1, ..Default::default() }),
    types: TypeLine {
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: vec![SubType("Aura".to_string())],
    },
    oracle_text: "Enchant creature\nEnchanted creature can't attack or block.".to_string(),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
        AbilityDefinition::Static {
            continuous_effect: ContinuousEffectDef {
                layer: EffectLayer::Layer7b, // or appropriate layer for attack/block restriction
                filter: EffectFilter::AttachedCreature,
                modification: LayerModification::AddKeyword(KeywordAbility::CantAttackOrBlock), // needs new keyword or effect
                duration: EffectDuration::WhileSourceOnBattlefield,
            },
        },
    ],
    ..Default::default()
}
```

**Note**: Pacifism's "can't attack or block" effect may need a new keyword or combat restriction mechanism. If that's too complex, Rancor is a simpler alternative (Enchant creature, +2/+0 and trample, plus graveyard-to-hand trigger).

**Alternative card**: Rancor ({G}, Enchant creature, +2/+0, trample)
- Simpler to model: `LayerModification::AddPowerToughness { power: 2, toughness: 0 }` + `LayerModification::AddKeyword(KeywordAbility::Trample)`
- Has a graveyard trigger (good for testing), but that's a separate ability, not part of Enchant

**Use `card-definition-author` agent** for final implementation.

### Step 9: Game Script (later phase)

**Suggested scenario**: Cast Pacifism (or Rancor) targeting a creature, verify attachment on resolution, verify continuous effect applies, verify SBA if the creature is killed (Aura falls off).
**Subsystem directory**: `test-data/generated-scripts/stack/` (Aura casting + resolution) or new `aura/` directory.

## Interactions to Watch

### Existing Code That Reads `enchants_creatures`
After Step 2, all references to `enchants_creatures` are dead. **Every** file listed in Step 2 must be updated simultaneously. Compile the project after this step to verify no lingering references.

### EffectFilter::AttachedCreature
This existing filter (in `rules/layers.rs:225-238`) works by checking `source.attached_to`. It is used by Equipment static abilities. Aura static abilities that grant effects to the enchanted creature will use the SAME filter. No changes needed to this filter. Confirm it works for Aura sources too (it should — it just checks `attached_to` on the source object).

### Protection (DEBT)
The existing protection check in `check_aura_sbas` (lines 624-637) already calls `attachment_is_illegal_due_to_protection`. This must continue to work after the refactor. The protection check runs AFTER the enchant-type check, which is correct.

### Target Validation in Casting
The casting code currently looks up `TargetRequirement` from `AbilityDefinition::Spell { targets }`. Aura cards are permanents and don't have a `Spell` ability definition. The new code must generate an implicit `TargetRequirement` from the `Enchant` keyword. This is the most architecturally significant change.

### Resolution Attachment Must Happen Before ETB Replacements
The Aura must be attached to its target BEFORE `register_static_continuous_effects` is called (line ~211 in resolution.rs), because the continuous effect uses `EffectFilter::AttachedCreature` which needs `attached_to` to be set. Insert the attachment code right after `obj.controller = controller;` and before the ETB replacement calls.

### Existing SBA Tests
The existing tests in `tests/sba.rs` and `tests/protection.rs` that use the `enchants_creatures` boolean must be updated to use the new `KeywordAbility::Enchant(EnchantTarget::Creature)` on the Aura's keywords instead.

### Aura Subtype Detection
The SBA currently checks `obj.characteristics.subtypes.contains(&SubType("Aura".to_string()))`. This should use **layer-computed** characteristics from `calculate_characteristics` to handle cases where a type-changing effect adds or removes the Aura subtype. For the initial implementation, raw characteristics are acceptable since Aura subtype removal is extremely rare.

## Risk Assessment

- **Breaking change**: `KeywordAbility::Enchant` gaining a payload is a breaking change to every match arm. Must update all in one atomic step.
- **`enchants_creatures` removal**: Removing a field from `GameObject` is a breaking change to all code that constructs `GameObject` directly. Most construction goes through builders/helpers, but `tests/snapshot_perf.rs` and `tests/commander_damage.rs` construct directly.
- **Resolution attachment ordering**: The attachment must happen at the right point in the resolution flow. Too early = ETB replacements don't see the permanent on the battlefield. Too late = static continuous effects can't resolve the `AttachedCreature` filter.
