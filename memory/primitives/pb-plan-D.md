# Primitive Batch Plan: PB-D -- Chosen Creature Type

**Generated**: 2026-04-02
**Primitive**: Dynamic chosen-creature-type filtering for EffectFilter (anthems), TriggerCondition (cast triggers), TargetFilter (destroy/count), SpellCostModifier (colored reduction), and EffectContext (spell-level type choice)
**CR Rules**: CR 205.3m (creature types), CR 601.2f (cost reduction), CR 614.1c ("as this enters" replacement)
**Cards affected**: 12 (4 existing partial fixes, 8 existing with TODOs)
**Dependencies**: None (existing infrastructure: `chosen_creature_type` on GameObject, `ReplacementModification::ChooseCreatureType`, `Effect::ChooseCreatureType`, `SpellCostFilter::HasChosenCreatureSubtype`, `ManaRestriction::ChosenTypeCreaturesOnly`)
**Deferred items from prior PBs**: None

## Primitive Specification

The engine already has `chosen_creature_type: Option<SubType>` on `GameObject` and the ETB replacement that sets it. What is missing:

1. **EffectFilter variants for chosen-type anthems** -- "+1/+1 to creatures you control of the chosen type" (Vanquisher's Banner, Patchwork Banner, Etchings of the Chosen, Morophon). The layer system needs `EffectFilter` variants that dynamically look up the source permanent's `chosen_creature_type` at application time.

2. **TriggerCondition for "whenever you cast a creature spell of the chosen type"** (Vanquisher's Banner). The `WheneverYouCastSpell` trigger condition needs a `chosen_subtype_filter: bool` field.

3. **EffectContext.chosen_creature_type for spell-level type choice** (Kindred Dominance, Pact of the Serpent). These spells choose a type on resolution (not via ETB replacement), so the chosen type must be stored in `EffectContext` for subsequent effects in a `Sequence`.

4. **TargetFilter fields for dynamic chosen-type matching** -- DestroyAll with "not of the chosen type" (Kindred Dominance), PermanentCount with "of the chosen type" (Pact of the Serpent, Three Tree City).

5. **Colored mana cost reduction** (Morophon: "{W}{U}{B}{R}{G} less"). The `SpellCostModifier.change` field only supports generic mana. Morophon needs per-color reduction.

6. **Count-based mana by chosen type** (Three Tree City: "add mana equal to creatures you control of the chosen type").

## CR Rule Text

**CR 205.3m**: Creature types list (authoritative list of all valid creature types).

**CR 601.2f**: "The player determines the total cost of the spell. Usually this is just the mana cost. [...] The total cost is the mana cost or alternative cost, plus all additional costs and cost increases, and minus all cost reductions. If multiple cost reductions apply, the player may apply them in any order."

**CR 614.1c**: "Effects that read '[This permanent] enters with...,' 'As [this permanent] enters...,' or '[This permanent] enters as...' are replacement effects."

## Engine Changes

### Change 1: Add `EffectFilter::CreaturesYouControlOfChosenType` and `EffectFilter::OtherCreaturesYouControlOfChosenType`

**File**: `crates/engine/src/state/continuous_effect.rs`
**Action**: Add two new variants to `EffectFilter` enum after `LandsYouControl`:

```rust
/// Applies to all creature permanents controlled by the source's controller
/// that have the source permanent's chosen_creature_type (INCLUDING the source).
///
/// Used for "Creatures you control of the chosen type get +1/+1" (Vanquisher's Banner,
/// Patchwork Banner, Etchings of the Chosen). Reads chosen_creature_type from the
/// source object at layer-application time.
CreaturesYouControlOfChosenType,
/// Same as above but excludes the source object itself.
///
/// Used for "Other creatures you control of the chosen type get +1/+1" (Morophon).
OtherCreaturesYouControlOfChosenType,
```

**Pattern**: Follow `OtherCreaturesYouControlWithSubtype(SubType)` at line 134. The difference: instead of a compile-time `SubType`, look up `state.objects.get(&source_id)?.chosen_creature_type` dynamically.

### Change 2: Dispatch new EffectFilter variants in layers.rs

**File**: `crates/engine/src/rules/layers.rs`
**Action**: Add match arms after `EffectFilter::LandsYouControl` (line 905) for the two new variants.

```rust
EffectFilter::CreaturesYouControlOfChosenType => {
    if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
        return false;
    }
    if let Some(source_id) = effect.source {
        let source = state.objects.get(&source_id);
        let source_controller = source.map(|s| s.controller);
        let chosen_type = source.and_then(|s| s.chosen_creature_type.as_ref());
        let obj_controller = state.objects.get(&object_id).map(|o| o.controller);
        source_controller.is_some()
            && source_controller == obj_controller
            && chosen_type.map(|ct| chars.subtypes.contains(ct)).unwrap_or(false)
    } else {
        false
    }
}
EffectFilter::OtherCreaturesYouControlOfChosenType => {
    // Same but exclude source
    if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
        return false;
    }
    if let Some(source_id) = effect.source {
        if source_id == object_id { return false; }
        let source = state.objects.get(&source_id);
        let source_controller = source.map(|s| s.controller);
        let chosen_type = source.and_then(|s| s.chosen_creature_type.as_ref());
        let obj_controller = state.objects.get(&object_id).map(|o| o.controller);
        source_controller.is_some()
            && source_controller == obj_controller
            && chosen_type.map(|ct| chars.subtypes.contains(ct)).unwrap_or(false)
    } else {
        false
    }
}
```

### Change 3: Hash the new EffectFilter variants

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms after `EffectFilter::LandsYouControl => 29u8` (line 1266):

```rust
EffectFilter::CreaturesYouControlOfChosenType => 30u8.hash_into(hasher),
EffectFilter::OtherCreaturesYouControlOfChosenType => 31u8.hash_into(hasher),
```

### Change 4: Add `chosen_subtype_filter: bool` to `TriggerCondition::WheneverYouCastSpell`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add field to `WheneverYouCastSpell` variant (line 2161):

```rust
WheneverYouCastSpell {
    during_opponent_turn: bool,
    spell_type_filter: Option<Vec<CardType>>,
    noncreature_only: bool,
    /// If true, only fires for creature spells whose subtype matches the trigger
    /// source's chosen_creature_type. Used by Vanquisher's Banner.
    #[serde(default)]
    chosen_subtype_filter: bool,
},
```

### Change 5: Dispatch `chosen_subtype_filter` in abilities.rs trigger post-filter

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the G-4 post-filter block (around line 3178), extend the `WheneverYouCastSpell` match arm to also check `chosen_subtype_filter`:

After the existing `spell_type_filter`/`noncreature_only` checks (line 3197), add:

```rust
if *chosen_subtype_filter {
    // Look up the trigger source's chosen_creature_type
    let source_chosen = state.objects.get(&t.source)
        .and_then(|o| o.chosen_creature_type.as_ref());
    let spell_has_chosen = source_chosen
        .map(|ct| spell_subtypes.contains(ct))
        .unwrap_or(false);
    if !spell_has_chosen { return false; }
}
```

This requires also extracting `spell_subtypes` from the cast spell (similar to how `spell_card_types` is already extracted at line 3154). Add:

```rust
let spell_subtypes: Vec<SubType> = state.objects.get(source_object_id)
    .map(|obj| obj.characteristics.subtypes.iter().cloned().collect())
    .unwrap_or_default();
```

### Change 6: Hash the new TriggerCondition field

**File**: `crates/engine/src/state/hash.rs`
**Action**: At line 4207, add `chosen_subtype_filter.hash_into(hasher);` inside the `WheneverYouCastSpell` hash block. Update the destructuring pattern to include the new field.

### Change 7: Add `chosen_creature_type: Option<SubType>` to `EffectContext`

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add field to `EffectContext` struct (after line 120):

```rust
/// Chosen creature type for spell-level type choice (Kindred Dominance, Pact of the Serpent).
/// Set by Effect::ChooseCreatureType when the source is a resolving spell.
/// Read by TargetFilter's exclude_chosen_subtype / has_chosen_subtype flags.
pub chosen_creature_type: Option<SubType>,
```

**Also**: Update `Effect::ChooseCreatureType` dispatch (line 2529) to ALSO set `ctx.chosen_creature_type = Some(chosen.clone())` after setting it on the permanent (or instead, for spells where source may not be on battlefield).

**Also**: Update ALL `EffectContext { ... }` construction sites to include `chosen_creature_type: None`:

| File | Line | Context |
|------|------|---------|
| `effects/mod.rs` | 122+ (Default/new) | EffectContext::new() or construction |
| `effects/mod.rs` | ~2224 | inner_ctx for ForEach |
| `rules/resolution.rs` | All EffectContext construction sites (grep for `EffectContext {`) |
| `rules/abilities.rs` | All EffectContext construction sites |
| `rules/replacement.rs` | All EffectContext construction sites |
| `state/stubs.rs` | Any stub construction |

Run `grep -n "EffectContext {" crates/engine/src/` to get exact list.

### Change 8: Add `has_chosen_subtype` and `exclude_chosen_subtype` to `TargetFilter`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add two new boolean fields to `TargetFilter` struct (after `is_attacking` at line 2083):

```rust
/// Must have the chosen creature type from the source/context.
/// Resolved dynamically: in effect context, reads ctx.chosen_creature_type;
/// for activated abilities, reads source permanent's chosen_creature_type.
/// Used for Pact of the Serpent count, Three Tree City count.
#[serde(default)]
pub has_chosen_subtype: bool,
/// Must NOT have the chosen creature type from the source/context.
/// Used for Kindred Dominance "destroy all creatures that aren't of the chosen type."
#[serde(default)]
pub exclude_chosen_subtype: bool,
```

### Change 9: Resolve chosen subtype fields in effect dispatch sites

**File**: `crates/engine/src/effects/mod.rs`
**Action**: At each call site that uses `matches_filter` and has access to `ctx` or a source permanent, add post-filter checks for the new `TargetFilter` fields. Key sites:

1. **`Effect::DestroyAll`** (line 839): After `matches_filter(&chars, filter)`, add:
   ```rust
   && check_chosen_subtype_filter(state, ctx, filter, &chars)
   ```

2. **`EffectAmount::PermanentCount`** (line 5122): Same pattern after `matches_filter`.

Add helper function:
```rust
/// Check has_chosen_subtype / exclude_chosen_subtype fields on TargetFilter.
/// Uses ctx.chosen_creature_type (spell-level) or source permanent's chosen_creature_type.
fn check_chosen_subtype_filter(
    state: &GameState, ctx: &EffectContext, filter: &TargetFilter, chars: &Characteristics
) -> bool {
    if !filter.has_chosen_subtype && !filter.exclude_chosen_subtype {
        return true;
    }
    let chosen = ctx.chosen_creature_type.as_ref().or_else(||
        state.objects.get(&ctx.source).and_then(|o| o.chosen_creature_type.as_ref())
    );
    if let Some(ct) = chosen {
        if filter.has_chosen_subtype && !chars.subtypes.contains(ct) { return false; }
        if filter.exclude_chosen_subtype && chars.subtypes.contains(ct) { return false; }
        true
    } else {
        // No chosen type set -- has_chosen_subtype fails, exclude passes
        !filter.has_chosen_subtype
    }
}
```

### Change 10: Hash the new TargetFilter fields

**File**: `crates/engine/src/state/hash.rs`
**Action**: At line 3910 (after `self.is_attacking.hash_into(hasher);`), add:
```rust
self.has_chosen_subtype.hash_into(hasher);
self.exclude_chosen_subtype.hash_into(hasher);
```

### Change 11: Add `colored_mana_reduction: Option<ManaCost>` to `SpellCostModifier`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add field to `SpellCostModifier` struct (after `exclude_self` at line 2983):

```rust
/// Colored mana reduction (Morophon: "{W}{U}{B}{R}{G} less to cast").
/// Each color field in the ManaCost specifies how much of that color to reduce.
/// Applied independently from `change` (which handles generic). Each colored
/// component cannot be reduced below 0.
#[serde(default)]
pub colored_mana_reduction: Option<ManaCost>,
```

### Change 12: Apply colored mana reduction in casting.rs

**File**: `crates/engine/src/rules/casting.rs`
**Action**: In the cost reduction loop (around line 5862), after collecting `total_change`, also collect colored reductions. After applying generic reduction (line 5870), apply each color:

```rust
// Track colored reductions from Morophon-style modifiers.
let mut colored_reduction = ManaCost::default();
// ... (inside the loop, after filter_matches check):
if let Some(ref cr) = modifier.colored_mana_reduction {
    colored_reduction.white += cr.white;
    colored_reduction.blue += cr.blue;
    colored_reduction.black += cr.black;
    colored_reduction.red += cr.red;
    colored_reduction.green += cr.green;
}
// ... (after generic reduction):
if colored_reduction != ManaCost::default() {
    reduced.white = reduced.white.saturating_sub(colored_reduction.white);
    reduced.blue = reduced.blue.saturating_sub(colored_reduction.blue);
    reduced.black = reduced.black.saturating_sub(colored_reduction.black);
    reduced.red = reduced.red.saturating_sub(colored_reduction.red);
    reduced.green = reduced.green.saturating_sub(colored_reduction.green);
}
```

Also update the early return check: `if total_change == 0 && colored_reduction == ManaCost::default() { return cost; }`.

CR reference: Morophon ruling (2019-06-14): "Morophon's effect reduces the total cost by up to one mana of each color."

### Change 13: Add `EffectAmount::ChosenTypeCreatureCount`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `EffectAmount` enum (after `CombatDamageDealt` at line 1966):

```rust
/// Count of creatures controlled by the target player that have the source
/// permanent's chosen_creature_type. Used for Three Tree City mana, Pact of the
/// Serpent draw/life loss.
ChosenTypeCreatureCount { controller: PlayerTarget },
```

### Change 14: Evaluate `EffectAmount::ChosenTypeCreatureCount` in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm in `resolve_effect_amount` (after `CombatDamageDealt`):

```rust
EffectAmount::ChosenTypeCreatureCount { controller } => {
    let chosen = ctx.chosen_creature_type.as_ref().or_else(||
        state.objects.get(&ctx.source).and_then(|o| o.chosen_creature_type.as_ref())
    );
    let Some(ct) = chosen else { return 0; };
    let players = resolve_player_target_list(state, controller, ctx);
    state.objects.values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_phased_in()
                && players.contains(&obj.controller)
                && {
                    let chars = crate::rules::layers::calculate_characteristics(state, obj.id)
                        .unwrap_or_else(|| obj.characteristics.clone());
                    chars.card_types.contains(&CardType::Creature)
                        && chars.subtypes.contains(ct)
                }
        })
        .count() as i32
}
```

### Change 15: Hash the new EffectAmount variant

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `EffectAmount::ChosenTypeCreatureCount` in the EffectAmount match. Find the last discriminant and add next.

### Change 16: Add `Effect::AddManaOfChosenTypeCount`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `Effect` enum for Three Tree City's unique ability ("Choose a color. Add mana of that color equal to the number of creatures you control of the chosen type").

Actually, this can be expressed as `Effect::AddManaAnyColorScaled { player: PlayerTarget::Controller, amount: EffectAmount::ChosenTypeCreatureCount { controller: PlayerTarget::Controller } }` if we have such an effect. Let me check if `AddManaAnyColorScaled` exists or something similar.

**Alternative approach**: Three Tree City's second activated ability can use:
```rust
Effect::AddMana {
    player: PlayerTarget::Controller,
    mana: ..., // one of any color -- but count-scaled
}
```

The DSL doesn't have "add X mana of any color" where X is dynamic. The simplest approach: add an `Effect::AddManaOfAnyColorAmount { player: PlayerTarget, amount: EffectAmount }` variant. When executed, the engine picks the most common color among the controller's permanents as the deterministic choice.

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to Effect enum:

```rust
/// "Add an amount of mana of [chosen color] equal to [amount]."
/// Deterministic color choice: picks the most common color among the controller's permanents.
/// CR 601.2f: The amount is determined at resolution time.
AddManaOfAnyColorAmount {
    player: PlayerTarget,
    amount: EffectAmount,
},
```

### Change 17: Dispatch `AddManaOfAnyColorAmount` in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add execution arm that resolves the amount, picks a deterministic color, and adds that much mana of that color to the player's pool.

### Change 18: Hash `AddManaOfAnyColorAmount`

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for new Effect variant. Find the last Effect discriminant and add next.

### Change 19: Exhaustive match updates for new enum variants

Files requiring new match arms:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `state/hash.rs` | `EffectFilter` match | L1266 | Add disc 30, 31 for new filter variants |
| `state/hash.rs` | `TriggerCondition::WheneverYouCastSpell` | L4199 | Add `chosen_subtype_filter` to destructuring + hash |
| `state/hash.rs` | `TargetFilter` | L3910 | Add two new bool fields to hash |
| `state/hash.rs` | `EffectAmount` match | (find) | Add disc for `ChosenTypeCreatureCount` |
| `state/hash.rs` | `Effect` match | (find) | Add disc for `AddManaOfAnyColorAmount` |
| `rules/layers.rs` | `EffectFilter` match | L905 | Add arms for both new filter variants |
| `effects/mod.rs` | `Effect` match | (main dispatch) | Arms for `AddManaOfAnyColorAmount` |
| `effects/mod.rs` | `EffectAmount` match | (resolve_effect_amount) | Arm for `ChosenTypeCreatureCount` |
| `tools/replay-viewer/src/view_model.rs` | Check if EffectFilter/Effect/EffectAmount matched | N/A | Likely no exhaustive match -- verify |
| `tools/tui/src/play/panels/stack_view.rs` | Check if Effect matched | N/A | Likely no exhaustive match -- verify |

## Card Definition Fixes

### 1. morophon_the_boundless.rs
**Oracle text**: Changeling. As Morophon enters, choose a creature type. Spells of the chosen type you cast cost {W}{U}{B}{R}{G} less to cast. This effect reduces only the amount of colored mana you pay. Other creatures you control of the chosen type get +1/+1.
**Current state**: Has ChooseCreatureType replacement, missing cost reduction + anthem.
**Fix**:
- Add `spell_cost_modifiers` with `filter: SpellCostFilter::HasChosenCreatureSubtype`, `change: 0`, `colored_mana_reduction: Some(ManaCost { white: 1, blue: 1, black: 1, red: 1, green: 1, ..Default::default() })`, `scope: CostModifierScope::Controller`.
  - Note: Morophon says "Spells of the chosen type" not just creature spells. The `HasChosenCreatureSubtype` filter already checks creature type. Per oracle text "Spells of the chosen type" means any spell with the subtype, not just creatures. Need a new `SpellCostFilter::HasChosenSubtype` (not creature-restricted) OR extend `HasChosenCreatureSubtype` semantics.
  - Actually re-reading oracle text: "Spells of the chosen type you cast cost..." -- this means creature spells that have the chosen creature subtype (since creature type applies to spells via Kindred/Tribal). For simplicity, use `HasChosenCreatureSubtype` -- it already does the right thing.
  - Wait: Morophon has Changeling, so "chosen type" is a creature type. But the cost reduction applies to "Spells" not "Creature spells." A Kindred spell (tribal) with the chosen type would also qualify. For now, keep `HasChosenCreatureSubtype` which requires `CardType::Creature`. If tribal spells matter later, extend.
- Add `AbilityDefinition::Static` with `ContinuousEffectDef { layer: EffectLayer::Layer7b, modification: LayerModification::ModifyBoth(1), filter: EffectFilter::OtherCreaturesYouControlOfChosenType }`.
- Remove existing TODOs.

### 2. vanquishers_banner.rs
**Oracle text**: As this artifact enters, choose a creature type. Creatures you control of the chosen type get +1/+1. Whenever you cast a creature spell of the chosen type, draw a card.
**Current state**: Empty abilities vec with TODO.
**Fix**:
- Add `ChooseCreatureType` replacement ability.
- Add `Static` ability with `CreaturesYouControlOfChosenType` filter + `ModifyBoth(1)` in Layer 7b.
- Add `Triggered` ability with `WheneverYouCastSpell { during_opponent_turn: false, spell_type_filter: Some(vec![CardType::Creature]), noncreature_only: false, chosen_subtype_filter: true }` and `DrawCards(Fixed(1))` effect.

### 3. patchwork_banner.rs
**Oracle text**: As this artifact enters, choose a creature type. Creatures you control of the chosen type get +1/+1. {T}: Add one mana of any color.
**Current state**: Has tap-for-any-color, missing choose type + anthem.
**Fix**:
- Add `ChooseCreatureType` replacement ability.
- Add `Static` ability with `CreaturesYouControlOfChosenType` filter + `ModifyBoth(1)`.
- Keep existing `AddManaAnyColor` activated ability.

### 4. heralds_horn.rs
**Oracle text**: As Herald's Horn enters, choose a creature type. Creature spells you cast of the chosen type cost {1} less to cast. At the beginning of your upkeep, look at the top card of your library. If it's a creature card of the chosen type, you may reveal it and put it into your hand.
**Current state**: Empty abilities vec with TODO.
**Fix**:
- Add `ChooseCreatureType` replacement ability.
- Add `spell_cost_modifiers` with `change: -1, filter: HasChosenCreatureSubtype, scope: Controller`.
- Upkeep trigger: Per ruling "If you don't put the top card into your hand, you put it back without revealing it." This is a look-at-top + conditional put-into-hand. Deterministic engine approximation: if top card is a creature of chosen type, put it into hand; otherwise do nothing (card stays on top). Use `AtBeginningOfYourUpkeep` trigger with a `Sequence` of `ChooseCreatureType` (to refresh ctx) then conditional draw based on top-card check.
- **DSL gap note**: "look at the top card" + "if it's a creature card of the chosen type" requires inspecting the top card's characteristics and comparing to `chosen_creature_type`. This is a Scry-like peek. The simplest approach: add a `Conditional` with a new `Condition::TopCardOfLibraryMatchesChosenType` that peeks at the top card. If true, execute `Effect::DrawCards(Fixed(1))`. If false, `Nothing`.
- **Alternative simpler approach**: Since the engine is deterministic, the bot always takes the best action. If top card matches, draw it. If not, leave it. Implement as: `Condition::TopCardIsCreatureOfChosenType` check + conditional `DrawCards(1)`.
- Add new `Condition::TopCardIsCreatureOfChosenType` variant.

### 5. kindred_dominance.rs
**Oracle text**: Choose a creature type. Destroy all creatures that aren't of the chosen type.
**Current state**: Empty abilities vec with TODO.
**Fix**:
- Spell effect: `Sequence([ChooseCreatureType { default: SubType("Human") }, DestroyAll { filter: TargetFilter { has_card_type: Some(Creature), exclude_chosen_subtype: true, ..Default::default() }, cant_be_regenerated: false }])`.
- The `ChooseCreatureType` sets `ctx.chosen_creature_type`, then `DestroyAll` uses `exclude_chosen_subtype` to skip creatures of that type.

### 6. cavern_of_souls.rs
**Oracle text**: As this land enters, choose a creature type. {T}: Add {C}. {T}: Add one mana of any color. Spend this mana only to cast a creature spell of the chosen type, and that spell can't be countered.
**Current state**: Has choose-type replacement, colorless tap, restricted mana tap. TODO for "can't be countered" rider.
**Fix**: The "can't be countered" rider is a complex interaction (linking mana source to spell uncounterability at payment time). **Defer** this to a future PB. The card is otherwise functional. Remove the TODO and add a comment noting the deferred "can't be countered" behavior.

### 7. unclaimed_territory.rs
**Current state**: Fully implemented (choose type replacement + colorless + restricted mana). No TODOs.
**Fix**: No changes needed. Already complete.

### 8. secluded_courtyard.rs
**Current state**: Fully implemented. No TODOs. The "or activate an ability" part of the mana restriction is noted as not enforced.
**Fix**: No changes needed. Already complete.

### 9. urzas_incubator.rs
**Current state**: Fully implemented (choose type replacement + HasChosenCreatureSubtype cost reduction). No TODOs.
**Fix**: No changes needed. Already complete.

### 10. three_tree_city.rs
**Oracle text**: As Three Tree City enters, choose a creature type. {T}: Add {C}. {2}, {T}: Choose a color. Add an amount of mana of that color equal to the number of creatures you control of the chosen type.
**Current state**: Has choose type replacement + colorless tap. TODO for count-based mana.
**Fix**:
- Add activated ability: `Cost::Sequence(vec![Cost::Mana(ManaCost { generic: 2, ..Default::default() }), Cost::Tap])`, `Effect::AddManaOfAnyColorAmount { player: PlayerTarget::Controller, amount: EffectAmount::ChosenTypeCreatureCount { controller: PlayerTarget::Controller } }`.

### 11. etchings_of_the_chosen.rs
**Oracle text**: As this enchantment enters, choose a creature type. Creatures you control of the chosen type get +1/+1. {1}, Sacrifice a creature of the chosen type: Target creature you control gains indestructible until end of turn.
**Current state**: Has choose type replacement only. TODOs for anthem + activated ability.
**Fix**:
- Add `Static` ability with `CreaturesYouControlOfChosenType` filter + `ModifyBoth(1)`.
- Add `Activated` ability: `Cost::Sequence(vec![Cost::Mana(ManaCost { generic: 1, ..Default::default() }), Cost::Sacrifice(TargetFilter { has_card_type: Some(Creature), has_chosen_subtype: true, ..Default::default() })])`. The `has_chosen_subtype: true` on the sacrifice cost filter means the engine checks the sacrificed creature has the source's `chosen_creature_type`.
  - **Note**: `Cost::Sacrifice(TargetFilter)` with `has_chosen_subtype: true` -- the sacrifice validation code in `replay_harness.rs`/`casting.rs` needs to resolve `has_chosen_subtype` by looking up the source permanent's `chosen_creature_type`. This requires extending the sacrifice validation to support this dynamic field.
- Effect: `ApplyContinuousEffect { effect_def: Box::new(ContinuousEffectDef { layer: EffectLayer::Layer6, modification: LayerModification::AddKeyword(KeywordAbility::Indestructible), filter: EffectFilter::DeclaredTarget { index: 0 }, duration: EffectDuration::UntilEndOfTurn, condition: None }) }`.
- Targets: `vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter { controller: TargetController::You, ..Default::default() })]`.

### 12. pact_of_the_serpent.rs
**Oracle text**: Choose a creature type. Target player draws X cards and loses X life, where X is the number of creatures they control of the chosen type.
**Current state**: Empty spell with TODO.
**Fix**:
- Targets: `vec![TargetRequirement::TargetPlayer]`.
- Effect: `Sequence(vec![ChooseCreatureType { default: SubType("Human") }, DrawCards { amount: ChosenTypeCreatureCount { controller: PlayerTarget::DeclaredTarget(0) }, player: PlayerTarget::DeclaredTarget(0) }, LoseLife { amount: ChosenTypeCreatureCount { controller: PlayerTarget::DeclaredTarget(0) }, player: PlayerTarget::DeclaredTarget(0) }])`.
- Per ruling (2021-02-05): "You choose the target player as you cast, but you don't choose the creature type until the spell resolves." The `ChooseCreatureType` in the Sequence runs at resolution time -- correct.

## New Card Definitions

None -- all 12 cards already have card def files.

## Condition Addition for Herald's Horn

### Change 20: Add `Condition::TopCardIsCreatureOfChosenType`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `Condition` enum (after `And` at line 2532):

```rust
/// "If the top card of your library is a creature card of the chosen type."
/// Used by Herald's Horn's upkeep trigger. Peeks at the controller's library top
/// card and checks if it's a creature with the source's chosen_creature_type.
TopCardIsCreatureOfChosenType,
```

### Change 21: Evaluate `TopCardIsCreatureOfChosenType` in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add arm in `check_condition` (around line 6019):

```rust
Condition::TopCardIsCreatureOfChosenType => {
    let chosen = state.objects.get(&ctx.source)
        .and_then(|o| o.chosen_creature_type.as_ref());
    let lib_zone = ZoneId::Library(ctx.controller);
    // Find top card of library (lowest position index)
    let top_card = state.objects.values()
        .filter(|o| o.zone == lib_zone)
        .min_by_key(|o| o.zone_position);
    match (chosen, top_card) {
        (Some(ct), Some(card)) => {
            card.characteristics.card_types.contains(&CardType::Creature)
                && card.characteristics.subtypes.contains(ct)
        }
        _ => false,
    }
}
```

### Change 22: Hash `Condition::TopCardIsCreatureOfChosenType`

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm with discriminant 41 after `Condition::And`.

### Change 23: Hash `EffectAmount::ChosenTypeCreatureCount`

**File**: `crates/engine/src/state/hash.rs`
**Action**: Find last `EffectAmount` discriminant and add next for `ChosenTypeCreatureCount`. Include `controller.hash_into(hasher)`.

## Sacrifice Cost Validation for Chosen Subtype

### Change 24: Extend sacrifice cost validation for `has_chosen_subtype`

**File**: `crates/engine/src/testing/replay_harness.rs` (and/or `crates/engine/src/rules/casting.rs`)
**Action**: Where `Cost::Sacrifice(filter)` is validated (checking that the sacrificed permanent matches the filter), add support for `has_chosen_subtype: true`. The validation needs to look up the activating permanent's `chosen_creature_type` and verify the sacrifice target has that subtype.

The exact location depends on where sacrifice cost validation happens. Search for `Cost::Sacrifice` dispatch in `replay_harness.rs` and `rules/` to find the validation site.

## Unit Tests

**File**: `crates/engine/tests/chosen_creature_type.rs` (new file)
**Tests to write**:
- `test_chosen_creature_type_etb_sets_type` -- verify `ChooseCreatureType` replacement sets `chosen_creature_type` on the permanent. CR 614.1c.
- `test_chosen_type_anthem_basic` -- verify `CreaturesYouControlOfChosenType` filter applies +1/+1 to matching creatures and not others.
- `test_chosen_type_anthem_other` -- verify `OtherCreaturesYouControlOfChosenType` excludes source.
- `test_chosen_type_cast_trigger` -- verify `chosen_subtype_filter` on `WheneverYouCastSpell` only fires for matching creature spells (Vanquisher's Banner pattern).
- `test_chosen_type_cost_reduction_generic` -- verify Herald's Horn reduces cost by {1} for matching creature spells.
- `test_chosen_type_cost_reduction_colored` -- verify Morophon reduces {W}{U}{B}{R}{G} from colored costs. CR 601.2f.
- `test_chosen_type_destroy_all_except` -- verify Kindred Dominance destroys non-chosen-type creatures and spares chosen-type ones.
- `test_chosen_type_permanent_count` -- verify `ChosenTypeCreatureCount` returns correct count.
- `test_chosen_type_spell_level_choice` -- verify `EffectContext.chosen_creature_type` is set by `ChooseCreatureType` in a spell Sequence.
- `test_pact_of_the_serpent_draw_and_life_loss` -- integration test: cast Pact targeting opponent, verify X draws + X life loss.
- `test_top_card_creature_of_chosen_type` -- verify Herald's Horn upkeep conditional draw.

**Pattern**: Follow tests in `crates/engine/tests/card_def_fixes.rs` for card-level integration tests using `GameStateBuilder`.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved
- [ ] New card defs authored (if any)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs

## Risks and Edge Cases

1. **Morophon colored mana reduction + hybrid mana**: Per ruling (2019-06-14), hybrid mana choices are made before cost reduction. If you choose {W} for a {W/U} hybrid, and Morophon reduces {W}, the pip is reduced. The engine's cost reduction pipeline applies after hybrid resolution, so this should work naturally -- but verify.

2. **Morophon "spells of the chosen type" scope**: Oracle says "Spells of the chosen type" not "creature spells." Kindred/tribal spells could qualify. Current `HasChosenCreatureSubtype` filter requires `CardType::Creature`. If tribal spells exist in the card universe, this is a minor gap. LOW risk since tribal cards are rare in Commander.

3. **Herald's Horn top-card peek is hidden information**: The "look at the top card" doesn't reveal it publicly. In the deterministic engine, this is fine (engine sees all). For networked play (M10+), the top-card check needs to be a private event. LOW risk for current scope.

4. **Cavern of Souls "can't be countered" deferred**: This is a well-known feature of the card. Deferring it is a gameplay correctness gap but the card is otherwise functional (mana restriction works). Document clearly in card def.

5. **`EffectContext.chosen_creature_type` propagation through nested effects**: In `Sequence`, the context is shared, so `ChooseCreatureType` setting `ctx.chosen_creature_type` is visible to subsequent effects. But in `ForEach` with inner contexts, the chosen type from the outer Sequence needs to propagate. Verify `inner_ctx` copies this field.

6. **`has_chosen_subtype` on `TargetFilter` in `Cost::Sacrifice` validation**: The sacrifice cost validation in `replay_harness.rs`/`casting.rs` needs to look up the activating permanent's `chosen_creature_type`. This is a new code path -- verify it exists and handles the case where no type is chosen (activation should be illegal).

7. **Zone change clears `chosen_creature_type`**: This is already handled (line 482, 620, 729, 919 of `state/mod.rs`). If a permanent with a chosen type is flickered, it returns without a chosen type and must re-choose via its ETB replacement. This is correct behavior.
