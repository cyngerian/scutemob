# Ability Plan: Soulbond

**Generated**: 2026-03-07
**CR**: 702.95
**Priority**: P4
**Similar abilities studied**: Backup (Layer 6 CE grant, `abilities.rs:2288+`, `resolution.rs:3005+`), Champion (ObjectId tracking on GameObject, `game_object.rs:560-570`, LTB trigger across multiple departure events in `abilities.rs:3506+`)

## CR Rule Text

> **702.95. Soulbond**
>
> **702.95a** Soulbond is a keyword that represents two triggered abilities. "Soulbond" means "When this creature enters, if you control both this creature and another creature and both are unpaired, you may pair this creature with another unpaired creature you control for as long as both remain creatures on the battlefield under your control" and "Whenever another creature you control enters, if you control both that creature and this one and both are unpaired, you may pair that creature with this creature for as long as both remain creatures on the battlefield under your control."
>
> **702.95b** A creature becomes "paired" with another as the result of a soulbond ability. Abilities may refer to a paired creature, the creature another creature is paired with, or whether a creature is paired. An "unpaired" creature is one that is not paired.
>
> **702.95c** When the soulbond ability resolves, if either object that would be paired is no longer a creature, no longer on the battlefield, or no longer under the control of the player who controls the soulbond ability, neither object becomes paired.
>
> **702.95d** A creature can be paired with only one other creature.
>
> **702.95e** A paired creature becomes unpaired if any of the following occur: another player gains control of it or the creature it's paired with; it or the creature it's paired with stops being a creature; or it or the creature it's paired with leaves the battlefield.

## Key Edge Cases

- **Intervening-if on both triggers (CR 702.95a)**: "if you control both this creature and another creature and both are unpaired" -- checked both at trigger time AND at resolution time (CR 603.4 intervening-if pattern)
- **Resolution fizzle (CR 702.95c)**: If either creature changed controller, stopped being a creature, or left the battlefield between trigger and resolution, no pairing occurs
- **One pair only (CR 702.95d)**: A creature can be in at most one pair. If a creature is already paired, it cannot be paired with another creature
- **Unpairing conditions (CR 702.95e)**: Control change, stops being a creature (e.g., Humility + non-creature type), or leaves battlefield -- ALL break the pair
- **Two triggers (CR 702.95a)**: Self-ETB trigger AND other-creature-ETB trigger. Both are on the soulbond creature, not on the entering creature
- **Deadeye Navigator ruling**: Blinking a paired creature breaks the pair immediately (zone change = new object, CR 400.7), then the ETB trigger fires again and they can re-pair
- **Flowering Lumberknot ruling**: If the paired partner loses soulbond, the pairing remains but the granted abilities stop (the "as long as paired" static ability on the soulbond creature is inactive since it lost soulbond)
- **Multiplayer**: Soulbond only pairs with creatures you control -- no opponent pairing. Control changes to an opponent break the pair (CR 702.95e)
- **Soulbond grants are static abilities**: "As long as this creature is paired with another creature, [effect]" is a static continuous effect from the card definition, NOT a one-shot grant. It applies via the layer system as long as the pairing exists and the source has soulbond
- **Two soulbond creatures**: Each can pair with the other via either creature's ETB trigger. The pairing is symmetric -- both are "paired with" each other

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant + State Fields

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Soulbond` variant
**Pattern**: Follow `KeywordAbility::LivingMetal` at line 1185
**Discriminant**: 129

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `paired_with: Option<ObjectId>` field to `GameObject`, after `champion_exiled_card` (line 570)
**CR**: 702.95b -- "A creature becomes 'paired' with another"
**Notes**:
- `#[serde(default)]` annotation required
- Unlike `champion_exiled_card`, `paired_with` should be RESET to `None` on zone changes (CR 400.7 -- new object identity breaks the pair; CR 702.95e -- leaving battlefield breaks pairing)
- Must also unpair the OTHER creature when one leaves (see Step 2 SBA)

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.paired_with.hash_into(hasher);` in `GameObject` hash impl, after `champion_exiled_card`
**Action**: Add `KeywordAbility::Soulbond` arm in the keyword hash match

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add `paired_with: None,` in `ObjectSpec` construction (near line 964, after `champion_exiled_card: None`)

**File**: `crates/engine/src/state/mod.rs`
**Action**: In BOTH `move_object_to_zone` call sites (lines ~361 and ~502), set `paired_with: None` (NOT preserved, unlike `champion_exiled_card`)
**Action**: When setting `paired_with: None` on the departing object, also clear `paired_with` on the OTHER creature (the partner). This requires: before creating the new object, check `old_object.paired_with`, and if `Some(partner_id)`, mutate `state.objects.get_mut(&partner_id)` to set its `paired_with = None`. This is the primary unpairing mechanism for zone changes.

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add `paired_with: None,` in token creation `GameObject` construction (wherever `champion_exiled_card: None` appears)

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `paired_with: None,` in any `GameObject` construction (wherever `champion_exiled_card: None` appears)

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `SoulbondSelfETB` and `SoulbondOtherETB` variants to `PendingTriggerKind` (after `ChampionLTB`, line 102)
**Action**: Add `soulbond_pair_target: Option<ObjectId>` field to `PendingTrigger` (after `champion_exiled_card`, line 334). This carries the auto-selected pairing target from trigger collection to SOK creation.

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `SoulbondTrigger` variant to `StackObjectKind`:
```
/// CR 702.95a: Soulbond ETB trigger -- pair source with target.
/// Discriminant 49.
SoulbondTrigger {
    source_object: ObjectId,      // The soulbond creature
    pair_target: ObjectId,        // The creature to pair with
    soulbond_owner: ObjectId,     // Which creature has soulbond (= source for self-ETB, = source for other-ETB)
},
```
**Notes**: We need to track which creature has soulbond because the "as long as paired" abilities come from the soulbond creature's card definition, not the paired partner.

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::Soulbond => "Soulbond".to_string()` in keyword display match (after `LivingMetal`, line 845)
**Action**: Add `StackObjectKind::SoulbondTrigger { source_object, .. } => { ... }` in `stack_kind_info()` match

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add `StackObjectKind::SoulbondTrigger { source_object, .. } => { ("Soulbond: ".to_string(), Some(*source_object)) }` in the SOK match

### Step 2: SBA-Based Unpairing

**File**: `crates/engine/src/rules/sba.rs`
**Action**: Add a new SBA check function `check_soulbond_unpairing` that implements CR 702.95e
**CR**: 702.95e -- creatures become unpaired when controller changes, stops being a creature, or leaves battlefield
**Pattern**: Add near other SBA checks, call from the main `check_state_based_actions` function

**Logic**:
```
For each object on the battlefield with paired_with = Some(partner_id):
  1. Check partner still exists and is on the battlefield
  2. Check partner is still a creature (use calculate_characteristics)
  3. Check both are controlled by the same player
  4. Check source is still a creature
  If any check fails:
    - Set source.paired_with = None
    - Set partner.paired_with = None (if partner still exists)
```

**Important**: This is NOT a triggered ability. It's a state-based action that cleans up invalid pairings. No trigger goes on the stack for unpairing. The continuous effects just stop applying because `paired_with` becomes `None`.

**Note**: The `move_object_to_zone` unpairing (Step 1) handles the zone-change case directly. The SBA handles the remaining cases: control change (via `ObjectControlChanged` or steal effects) and "stops being a creature" (via Humility/type-changing effects). Both mechanisms are needed because SBAs are only checked at specific times, but zone changes happen immediately.

### Step 3: Trigger Wiring

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `PermanentEnteredBattlefield` event handler within `check_triggers`:

**Trigger 1 -- Self-ETB (CR 702.95a first sentence)**:
When a creature with soulbond enters the battlefield:
- Check intervening-if: controller controls at least one OTHER unpaired creature (besides this one)
- If yes, create `PendingTrigger` with `kind: PendingTriggerKind::SoulbondSelfETB`
- Auto-select pairing target: first unpaired creature controlled by the same player (deterministic default)
- Store in `soulbond_pair_target`

**Trigger 2 -- Other-ETB (CR 702.95a second sentence)**:
When ANY creature enters the battlefield:
- For each OTHER creature with soulbond controlled by the entering creature's controller:
  - Check intervening-if: both the soulbond creature and the entering creature are unpaired
  - If yes, create `PendingTrigger` with `kind: PendingTriggerKind::SoulbondOtherETB`
  - `source` = the soulbond creature (NOT the entering creature)
  - `soulbond_pair_target` = the entering creature's ObjectId
  - `entering_object_id` = the entering creature's ObjectId

**Keyword detection**: Use layer-resolved characteristics (`calculate_characteristics`) to check for `KeywordAbility::Soulbond`, NOT the card registry. This ensures Humility/ability-removal is respected (per gotchas-infra.md parameterized keyword pattern).

**File**: `crates/engine/src/rules/abilities.rs` (in `flush_pending_triggers`)
**Action**: Add match arms for `PendingTriggerKind::SoulbondSelfETB` and `PendingTriggerKind::SoulbondOtherETB` to create `StackObjectKind::SoulbondTrigger`:
```
PendingTriggerKind::SoulbondSelfETB | PendingTriggerKind::SoulbondOtherETB => {
    StackObjectKind::SoulbondTrigger {
        source_object: trigger.source,
        pair_target: trigger.soulbond_pair_target.unwrap_or(trigger.source),
        soulbond_owner: trigger.source,  // The soulbond creature is always the source
    }
}
```

### Step 4: Trigger Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::SoulbondTrigger` resolution handler

**Resolution logic (CR 702.95c)**:
```
1. Fizzle check: both source_object and pair_target must still be:
   - On the battlefield
   - Creatures (layer-resolved)
   - Controlled by the same player (the soulbond ability's controller)
   - Both unpaired (paired_with == None)
2. If all checks pass:
   - Set source_object.paired_with = Some(pair_target)
   - Set pair_target.paired_with = Some(source_object)
3. Emit GameEvent::AbilityResolved
```

**Action**: Add `SoulbondTrigger` to the counter-spell catch-all arm (line ~4998)

### Step 5: Layer 6 Continuous Effect for Paired Abilities

**Design Decision**: Soulbond's "as long as paired" grant is a STATIC ability from the card definition, not a registered `ContinuousEffect`. This is because:
- The grant is conditional on pairing state (`paired_with.is_some()`)
- It comes from the card's printed text, not from a resolved trigger
- It must respond dynamically to pairing/unpairing (no cleanup needed)
- It must be removed if soulbond itself is removed (Humility)

**File**: `crates/engine/src/rules/layers.rs`
**Action**: In the Layer 6 (Ability) processing section, add inline soulbond grant logic:

**Logic** (runs during `calculate_characteristics` for each object):
```
After processing all ContinuousEffects for Layer 6:
For each object on the battlefield with paired_with = Some(partner_id):
  1. Check that this object has KeywordAbility::Soulbond in its layer-resolved keywords
     (already computed by this point in Layer 6 -- but NOTE: we need soulbond to survive
     Layer 6 removal effects like Humility. If Humility removes soulbond, no grant.)
  2. Look up the soulbond creature's card definition
  3. Find the soulbond_grants from the card definition (see Step 6 below for how these
     are specified)
  4. Apply those grants to BOTH the soulbond creature and its paired partner
```

**IMPORTANT**: This is tricky because Layer 6 effects from soulbond should apply to both creatures in the pair, and BOTH creatures might have soulbond (each granting different abilities). The grants must be cumulative.

**Alternative approach (simpler, recommended)**: Instead of inline Layer 6 logic, use `EffectDuration::WhileSourceOnBattlefield` with a custom activity check. But we need a new duration variant:

**File**: `crates/engine/src/state/continuous_effect.rs`
**Action**: Add `EffectDuration::WhilePaired(ObjectId, ObjectId)` variant:
```
/// Active as long as both ObjectIds are paired with each other on the battlefield.
/// Used for soulbond "as long as paired" grants (CR 702.95a).
WhilePaired(ObjectId, ObjectId),
```

**File**: `crates/engine/src/rules/layers.rs`
**Action**: Add `WhilePaired` handling in `is_effect_active`:
```
EffectDuration::WhilePaired(a, b) => {
    // Both must be on battlefield, both must have paired_with pointing to each other
    let a_ok = state.objects.get(&a)
        .map(|o| o.zone == ZoneId::Battlefield && o.is_phased_in() && o.paired_with == Some(b))
        .unwrap_or(false);
    let b_ok = state.objects.get(&b)
        .map(|o| o.zone == ZoneId::Battlefield && o.is_phased_in() && o.paired_with == Some(a))
        .unwrap_or(false);
    a_ok && b_ok
}
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `WhilePaired(a, b)` to the `EffectDuration` hash impl

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: Add `WhilePaired` handling in the `is_replacement_active` duration check (line ~225), same logic as `is_effect_active`

Then, in the SoulbondTrigger resolution (Step 4), after setting `paired_with` on both creatures:
- Look up the soulbond creature's card definition
- Extract the soulbond grants (continuous effects defined in the card def)
- Register them as `ContinuousEffect` entries with `duration: WhilePaired(source, target)` and `filter: SingleObject(target)` (for the partner) plus `filter: SingleObject(source)` (for self)

### Step 6: Card Definition Support for Soulbond Grants

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Soulbond` variant:
```
/// CR 702.95a: Soulbond -- two triggered abilities (self-ETB and other-ETB pairing)
/// plus a static "as long as paired" grant.
///
/// `grants` specifies what continuous effects apply to both paired creatures.
/// These are registered as WhilePaired CEs when the pairing is established.
///
/// Discriminant 50.
Soulbond {
    /// Continuous effects granted to both paired creatures.
    /// E.g., for Wolfir Silverheart: LayerModification::ModifyPT(4, 4)
    /// E.g., for Silverblade Paladin: LayerModification::AddKeywords({DoubleStrike})
    grants: Vec<SoulbondGrant>,
},
```

**File**: `crates/engine/src/cards/card_definition.rs` (or `types.rs`)
**Action**: Define `SoulbondGrant` enum/struct:
```
/// A continuous effect granted by soulbond while paired.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoulbondGrant {
    pub layer: EffectLayer,
    pub modification: LayerModification,
}
```

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: Export `SoulbondGrant` for use in card definitions

**File**: `crates/engine/src/cards/card_definition.rs` (in `enrich_spec_from_def`)
**Action**: When encountering `AbilityDefinition::Soulbond { .. }`, add `KeywordAbility::Soulbond` to the object's keywords

### Step 7: Unit Tests

**File**: `crates/engine/tests/soulbond.rs`
**Tests to write**:

1. `test_soulbond_self_etb_pairs_with_unpaired_creature` -- CR 702.95a first sentence
   - Creature with soulbond enters, controller has another unpaired creature
   - After trigger resolves, both creatures have `paired_with` pointing to each other
   - Pattern: Follow Backup tests for ETB trigger resolution

2. `test_soulbond_other_etb_pairs_with_entering_creature` -- CR 702.95a second sentence
   - Soulbond creature already on battlefield, another creature enters
   - Trigger fires and pairs them
   - Verify both have `paired_with` set

3. `test_soulbond_grants_apply_while_paired` -- CR 702.95a "for as long as"
   - Wolfir Silverheart-style: pair should grant +4/+4 to both creatures
   - Use `calculate_characteristics` to verify P/T modification

4. `test_soulbond_unpair_on_zone_change` -- CR 702.95e + CR 400.7
   - Pair two creatures, then one dies/is exiled/bounced
   - Remaining creature's `paired_with` should be `None`
   - Grants should no longer apply

5. `test_soulbond_no_pair_if_no_unpaired_available` -- CR 702.95a intervening-if
   - Soulbond creature enters but controller has no other unpaired creatures
   - No trigger should fire

6. `test_soulbond_already_paired_cannot_repiar` -- CR 702.95d
   - Creature already paired cannot be chosen as pairing target

7. `test_soulbond_resolution_fizzle` -- CR 702.95c
   - Target creature leaves battlefield between trigger and resolution
   - Neither creature becomes paired

8. `test_soulbond_unpair_on_control_change` -- CR 702.95e
   - Paired creature changes controller (e.g., steal effect)
   - Pairing breaks for both creatures

9. `test_soulbond_grants_removed_when_unpaired`
   - Verify that continuous effects with `WhilePaired` duration stop applying when unpairing occurs

10. `test_soulbond_two_soulbond_creatures_both_grant`
    - Two different soulbond creatures paired together
    - Both should receive grants from BOTH creatures' soulbond definitions

**Pattern**: Follow Champion tests (ObjectId tracking, ETB/LTB triggers) and Backup tests (Layer 6 CE grant)

### Step 8: Card Definition (later phase)

**Suggested card**: Wolfir Silverheart
- Simple P/T grant (+4/+4 to both paired creatures)
- Clean test of soulbond mechanics without complex interactions
- Card lookup: use `card-definition-author` agent

**Secondary card**: Silverblade Paladin
- Keyword grant (double strike) tests the `AddKeywords` path

### Step 9: Game Script (later phase)

**Suggested scenario**: Wolfir Silverheart enters battlefield, pairs with an existing creature, both get +4/+4. Then the partner dies, unpairing occurs, Silverheart loses the buff.
**Subsystem directory**: `test-data/generated-scripts/abilities/`

## Interactions to Watch

- **Humility (Layer 6 ability removal)**: If Humility removes soulbond, the "as long as paired" grants should stop. The `WhilePaired` duration handles this indirectly -- the CEs still exist but the soulbond keyword is gone. However, CR 702.95e says unpairing occurs when the creature "stops being a creature" -- Humility doesn't cause that. The pairing itself should persist (objects still have `paired_with`), but the grants (which come from soulbond's static ability, which was removed) should not apply. **Resolution**: The `WhilePaired` duration checks that both are on battlefield and paired -- it does NOT check that soulbond is present. The grants continue even if soulbond is removed. This matches the Flowering Lumberknot ruling: "If the creature Flowering Lumberknot is paired with loses soulbond, Flowering Lumberknot will remain paired but won't be able to attack or block." -- wait, this says the Lumberknot is paired but doesn't have the grant. The grant comes from the SOULBOND creature's ability, which was removed. So the grant stops because the source lost the ability, not because of unpairing.

  **CORRECTION**: The `WhilePaired` CE should ALSO check that the soulbond source creature still has soulbond. Add `soulbond_source: ObjectId` to the CE or use a combined check. Alternatively, use `WhileSourceOnBattlefield` combined with a paired-check in the filter. The simplest approach: add a source-has-soulbond check in `is_effect_active` for `WhilePaired`, or store the soulbond owner ID and verify the keyword. Update the `WhilePaired` variant to `WhilePaired { creature_a: ObjectId, creature_b: ObjectId, soulbond_source: ObjectId }` and check that `soulbond_source` still has the Soulbond keyword in layer-resolved characteristics.

  **PROBLEM**: Checking layer-resolved characteristics inside `is_effect_active` during `calculate_characteristics` creates a circular dependency. The layer system is computing characteristics and would need to re-enter itself. **Solution**: Don't check for soulbond keyword presence in `is_effect_active`. Instead, rely on the SBA to clean up: when soulbond is removed (by Humility), the SBA doesn't automatically unpair (CR 702.95e only lists control change, stops being creature, or leaves battlefield). The pairing persists but the grants are from a static ability that no longer exists. This is handled naturally if we register the CEs with `duration: WhileSourceOnBattlefield` (the source is the soulbond creature) AND an additional paired check.

  **FINAL APPROACH**: Use `EffectDuration::WhileSourceOnBattlefield` for the CEs, with `source: Some(soulbond_creature_id)`. In addition, override the filter to check pairing state. Actually, the cleanest design:
  1. Register CEs with `duration: WhileSourceOnBattlefield` and `source: Some(soulbond_owner)`
  2. Use `EffectFilter::PairedWith(soulbond_owner)` -- a new filter variant that matches any object paired with the given ID
  3. For the self-grant, use `EffectFilter::SingleObject(soulbond_owner)` with an additional `WhilePaired` condition

  **SIMPLEST CORRECT APPROACH**: Skip `WhilePaired` entirely. Use the SBA (Step 2) to clear `paired_with` when unpairing conditions are met. In `calculate_characteristics`, add inline soulbond-grant logic in Layer 6 that checks:
  - Object has soulbond keyword (already in current characteristics after Layer 6 removals are applied -- but this is mid-Layer-6, timing matters)
  - Object has `paired_with = Some(partner_id)`
  - Partner is on battlefield
  - Apply grants to BOTH self and partner

  **PROBLEM**: Inline Layer 6 logic for soulbond grants means the grants are computed during characteristic calculation, not from registered CEs. This is the same pattern as Changeling (CDA in Layer 4). It works but doesn't use the CE system. The downside: the grants don't have timestamps and can't interact with other Layer 6 effects in timestamp order.

  **RECOMMENDED FINAL APPROACH**: Register CEs at pairing resolution time (Step 4). Use `EffectDuration::WhilePaired(ObjectId, ObjectId)`. In `is_effect_active`, check that both objects are on battlefield and paired with each other. Do NOT check for soulbond keyword presence (avoids circular dependency). If soulbond is removed by Humility, the CEs remain active as long as the pairing exists. This is a minor inaccuracy relative to the Flowering Lumberknot ruling, but it's the simplest correct implementation for V1. The Lumberknot ruling is about a creature WHOSE PARTNER lost soulbond -- the Lumberknot itself never had soulbond, it just has "as long as paired" conditions. For creatures that DO have soulbond, the grants come from their own static ability. Accept this edge case as a known V1 gap.

- **Object identity (CR 400.7)**: When a paired creature changes zones, it becomes a new object. The `move_object_to_zone` handling (Step 1) clears `paired_with` on both the departing creature AND its partner. The CEs with `WhilePaired` duration automatically become inactive.

- **Multiple soulbond triggers**: If a creature with soulbond enters and there are multiple soulbond creatures on the battlefield, each soulbond creature generates an Other-ETB trigger. Only one can succeed (first to resolve pairs, subsequent ones fizzle because the target is now paired). APNAP ordering determines which resolves first.

- **Phasing**: A phased-out creature is "treated as though it does not exist" (CR 702.26). The `is_phased_in()` check in `is_effect_active` for `WhilePaired` ensures grants don't apply while phased out. When the creature phases back in, `paired_with` is still set (phasing preserves state), so the pairing resumes. CR 702.95e does NOT list phasing as an unpairing condition -- correct behavior.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| `KeywordAbility` | `Soulbond` | 129 |
| `AbilityDefinition` | `Soulbond { grants }` | 50 |
| `StackObjectKind` | `SoulbondTrigger { .. }` | 49 |
| `PendingTriggerKind` | `SoulbondSelfETB` | (auto) |
| `PendingTriggerKind` | `SoulbondOtherETB` | (auto) |
| `EffectDuration` | `WhilePaired(ObjectId, ObjectId)` | (auto) |

## Files Modified Summary

| File | Changes |
|------|---------|
| `crates/engine/src/state/types.rs` | `KeywordAbility::Soulbond` (disc 129) |
| `crates/engine/src/state/game_object.rs` | `paired_with: Option<ObjectId>` field |
| `crates/engine/src/state/hash.rs` | Hash `paired_with`, `Soulbond`, `WhilePaired`, `SoulbondTrigger` |
| `crates/engine/src/state/builder.rs` | `paired_with: None` in ObjectSpec |
| `crates/engine/src/state/mod.rs` | Reset `paired_with` in both `move_object_to_zone` sites + unpair partner |
| `crates/engine/src/state/stubs.rs` | `SoulbondSelfETB`/`SoulbondOtherETB` PTK; `soulbond_pair_target` field |
| `crates/engine/src/state/stack.rs` | `SoulbondTrigger` SOK (disc 49) |
| `crates/engine/src/state/continuous_effect.rs` | `WhilePaired(ObjectId, ObjectId)` duration |
| `crates/engine/src/cards/card_definition.rs` | `AbilityDefinition::Soulbond { grants }` (disc 50); `SoulbondGrant` struct |
| `crates/engine/src/cards/helpers.rs` | Export `SoulbondGrant` |
| `crates/engine/src/effects/mod.rs` | `paired_with: None` in token creation |
| `crates/engine/src/rules/sba.rs` | `check_soulbond_unpairing` SBA function |
| `crates/engine/src/rules/abilities.rs` | ETB trigger detection + flush PTK match arms |
| `crates/engine/src/rules/resolution.rs` | `SoulbondTrigger` resolution + CE registration; counter-spell arm |
| `crates/engine/src/rules/layers.rs` | `WhilePaired` in `is_effect_active` |
| `crates/engine/src/rules/replacement.rs` | `WhilePaired` in duration check |
| `crates/engine/tests/soulbond.rs` | 10 unit tests |
| `tools/replay-viewer/src/view_model.rs` | `Soulbond` KW display + `SoulbondTrigger` SOK display |
| `tools/tui/src/play/panels/stack_view.rs` | `SoulbondTrigger` SOK display |
