# Primitive Batch Plan: PB-31 -- Cost Primitives (RemoveCounter, SpellSacrificeCost)

**Generated**: 2026-03-26
**Primitives**: (1) `Cost::RemoveCounter` for activated ability counter-removal costs; (2) `SpellAdditionalCost` enum on `CardDefinition` for spell casting sacrifice costs
**CR Rules**: CR 118.3 (must have resources), CR 118.8 (additional costs on spells), CR 602.2/602.2b (activated ability costs), CR 601.2b/601.2h (paying costs at cast time)
**Cards affected**: ~23 (~19 existing fixes + ~4 complex/deferred)
**Dependencies**: PB-4 (sacrifice-as-activation-cost -- done), RC-1 type consolidation (AdditionalCost vec -- done)
**Deferred items from prior PBs**: None specific to PB-31. PB-30 deferred items (Heartstone, Training Grounds, etc.) are assigned to PB-37.

## Primitive Specification

### G-16: Cost::RemoveCounter

Many cards have activated abilities whose cost includes "Remove N [type] counters from [this permanent]." The engine has `Effect::RemoveCounter` for effect-side counter removal but no `Cost::RemoveCounter` variant for cost-side. Cards like Dragon's Hoard, Spawning Pit, Ominous Seas, Gemstone Array, Spike Weaver, Ramos, and Golgari Grave-Troll are all blocked.

The new `Cost::RemoveCounter { counter: CounterType, count: u32 }` variant goes in the `Cost` enum (card_definition.rs). The `flatten_cost_into()` function translates it to a new field on `ActivationCost`. The `handle_activate_ability()` function validates sufficient counters and removes them before putting the ability on the stack.

### G-17: Spell Additional Sacrifice Cost

Cards like Village Rites ("As an additional cost to cast this spell, sacrifice a creature") require the caster to sacrifice a permanent at cast time. The existing `AdditionalCost::Sacrifice` on `CastSpell` is used by Bargain/Emerge/Casualty/Devour, which are keyword-dispatched. For generic "sacrifice a [type]" spell costs, we need:

1. A new `spell_additional_costs: Vec<SpellAdditionalCost>` field on `CardDefinition` to declare the required cost.
2. A new `SpellAdditionalCost` enum (e.g., `SacrificeCreature`, `SacrificeLand`, `SacrificeArtifactOrCreature`, `SacrificeSubtype(SubType)`, `SacrificeColorPermanent(Color)`) to express the filter.
3. Validation in `casting.rs` that checks the card def's required additional costs are satisfied by the `additional_costs` vec on CastSpell.

The existing `AdditionalCost::Sacrifice(Vec<ObjectId>)` already carries the sacrifice target IDs. The card def's `spell_additional_costs` declares the requirement; casting.rs validates it.

## CR Rule Text

**CR 118.3**: A player can't pay a cost without having the necessary resources to pay it fully. For example, a player with only 1 life can't pay a cost of 2 life, and a permanent that's already tapped can't be tapped to pay a cost.

**CR 118.8**: Some spells and abilities have additional costs. An additional cost is a cost listed in a spell's rules text, or applied to a spell or ability from another effect, that its controller must pay at the same time they pay the spell's mana cost or the ability's activation cost.

**CR 602.2**: To activate an ability is to put it onto the stack and pay its costs, so that it will eventually resolve and have its effect.

**CR 602.2b**: The remainder of the process for activating an ability is identical to the process for casting a spell listed in rules 601.2b-i.

**CR 601.2h**: The player pays the total cost. First, they pay all costs that don't involve random elements or moving objects from the library to a public zone, in any order. Then they pay all remaining costs in any order. Partial payments are not allowed. Unpayable costs can't be paid.

## Engine Changes

### Change 1: Add `Cost::RemoveCounter` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Line**: ~918 (after `Cost::Sequence`)
**Action**: Add new variant to the `Cost` enum:
```rust
/// Remove N counters of the specified type from the source permanent as a cost (CR 602.2).
/// CR 118.3: The permanent must have at least `count` counters of that type.
RemoveCounter { counter: CounterType, count: u32 },
```
**Pattern**: Follow `Cost::PayLife(u32)` at line 908.

### Change 2: Add `remove_counter_cost` field to `ActivationCost`

**File**: `crates/engine/src/state/game_object.rs`
**Line**: ~235 (after `sacrifice_filter` field)
**Action**: Add new field to `ActivationCost`:
```rust
/// CR 602.2: Remove counters from the source permanent as part of the activation cost.
/// E.g., "Remove a charge counter: ..." = `Some((CounterType::Charge, 1))`.
#[serde(default)]
pub remove_counter_cost: Option<(CounterType, u32)>,
```

### Change 3: Wire `Cost::RemoveCounter` in `flatten_cost_into()`

**File**: `crates/engine/src/testing/replay_harness.rs`
**Line**: ~2963 (after `Cost::PayLife` arm in `flatten_cost_into` at line 2933)
**Action**: Add match arm:
```rust
Cost::RemoveCounter { counter, count } => {
    ac.remove_counter_cost = Some((counter.clone(), *count));
}
```

### Change 4: Validate and pay RemoveCounter cost in `handle_activate_ability()`

**File**: `crates/engine/src/rules/abilities.rs`
**Line**: ~578 (after the sacrifice-another-permanent cost block, before stack push)
**Action**: Add counter removal cost validation and payment. Use existing `GameEvent::CounterRemoved` (already defined in `rules/events.rs` line 424):
```rust
// CR 602.2 / CR 118.3: Pay remove-counter cost.
if let Some((ref counter_type, count)) = ability_cost.remove_counter_cost {
    let obj = state.object(source)?;
    let current = obj.counters.get(counter_type).copied().unwrap_or(0);
    if current < count {
        return Err(GameStateError::InvalidCommand(format!(
            "remove-counter cost: need {} {:?} counters, have {} (CR 118.3)",
            count, counter_type, current
        )));
    }
    if let Some(obj) = state.objects.get_mut(&source) {
        let new_count = current - count;
        if new_count == 0 {
            obj.counters.remove(counter_type);
        } else {
            obj.counters.insert(counter_type.clone(), new_count);
        }
    }
    events.push(GameEvent::CounterRemoved {
        object_id: source,
        counter: counter_type.clone(),
        count,
    });
}
```
**CR**: CR 118.3 -- a player can't pay a cost without having the necessary resources.

### Change 5: Add `SpellAdditionalCost` enum and `spell_additional_costs` field on `CardDefinition`

**File**: `crates/engine/src/cards/card_definition.rs`
**Line**: ~118 (after `starting_loyalty` field on `CardDefinition`; enum near the `Cost` enum section ~line 918)
**Action**:
1. Add field to `CardDefinition`:
```rust
/// CR 118.8: Required additional costs to cast this spell.
/// E.g., Village Rites: "As an additional cost to cast this spell, sacrifice a creature."
/// The caster must include a matching `AdditionalCost::Sacrifice` in the CastSpell command.
#[serde(default)]
pub spell_additional_costs: Vec<SpellAdditionalCost>,
```
2. Add new enum (near the Cost enum section, ~line 918):
```rust
/// Required additional cost on a spell card definition (CR 118.8).
/// Declares what the caster must sacrifice/pay as an additional cost to cast.
/// Validated in `casting.rs` against `additional_costs` on `CastSpell`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellAdditionalCost {
    /// "Sacrifice a creature" (Village Rites, Altar of Bone, etc.)
    SacrificeCreature,
    /// "Sacrifice a land" (Crop Rotation)
    SacrificeLand,
    /// "Sacrifice an artifact or creature" (Deadly Dispute)
    SacrificeArtifactOrCreature,
    /// "Sacrifice a [subtype]" (Goblin Grenade: "Sacrifice a Goblin")
    SacrificeSubtype(SubType),
    /// "Sacrifice a [color] permanent" (Abjure: "Sacrifice a blue permanent")
    SacrificeColorPermanent(Color),
}
```

### Change 6: Validate spell additional costs in `casting.rs`

**File**: `crates/engine/src/rules/casting.rs`
**Line**: After the card def is loaded and before mana payment (around the validation section, ~line 2700-2800 area where bargain/casualty validation happens)
**Action**: Add validation block that:
1. Loads the card def's `spell_additional_costs` from the card registry.
2. For each required cost, validates that a matching `AdditionalCost::Sacrifice(ids)` exists in the CastSpell's `additional_costs`.
3. Validates the sacrificed permanent matches the filter (creature, land, artifact-or-creature, subtype, color) using layer-resolved characteristics via `calculate_characteristics()`.
4. Performs the sacrifice (move to graveyard, emit `CreatureDied`/`PermanentDestroyed` + `PermanentSacrificed` events) alongside existing bargain/emerge/casualty sacrifice handling.

**Pattern**: Follow the bargain validation block at ~line 2735. The sacrifice-and-event-emission code at ~line 3340 can be reused.

**CR**: CR 118.8 -- additional costs are paid at the same time as the mana cost. CR 601.2h -- costs are paid in any order, partial payments not allowed.

### Change 7: Exhaustive match updates

Files requiring new match arms for the new `Cost::RemoveCounter` variant:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/state/hash.rs` | `Cost` HashInto | L4241 | Add `Cost::RemoveCounter { counter, count }` arm with discriminant `9u8`, hash counter + count |
| `crates/engine/src/testing/replay_harness.rs` | `flatten_cost_into` match | L2934 | Add arm (Change 3 above) |

Files requiring hash updates for `ActivationCost` new field:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/state/hash.rs` | `ActivationCost` HashInto | L1733 | Add `self.remove_counter_cost.hash_into(hasher)` after line 1741 |

`SpellAdditionalCost` hash: `CardDefinition` is NOT hashed (it's static registry data). `SpellAdditionalCost` only lives on `CardDefinition`, so NO HashInto impl is needed for it.

**No changes needed in**:
- `tools/replay-viewer/src/view_model.rs` -- does not match on `Cost` or `AdditionalCost`
- `tools/tui/src/play/panels/stack_view.rs` -- does not match on `Cost` or `AdditionalCost`

### Change 8: Counter type decision

`CounterType::Custom(String)` already exists as a catch-all. Dragon's Hoard uses "gold" counters, Ominous Seas uses "foreshadow" counters. These will use `CounterType::Custom("gold".into())` etc. No new named `CounterType` variants needed. `CounterType::Charge` already exists for Spawning Pit, Gemstone Array, Umezawa's Jitte, Druids' Repository.

### Change 9: Wire `sacrifice_card` for spell casting in the replay harness

**File**: `crates/engine/src/testing/replay_harness.rs`
**Line**: ~289 (the base `cast_spell` handler)
**Action**: Extend the base `cast_spell` handler to read `sacrifice_card_name`. If present, resolve to ObjectId on battlefield, add `AdditionalCost::Sacrifice(vec![sac_id])` to the `additional_costs` vec. This avoids adding a new `cast_spell_sacrifice` action type. The `sacrifice_card` field already exists in `PlayerAction` (script_schema.rs line 351) but is currently only wired for `activate_ability`.

### Change 10: Export `SpellAdditionalCost` from helpers.rs

**File**: `crates/engine/src/cards/helpers.rs`
**Line**: In the re-export section
**Action**: Add `SpellAdditionalCost` to the public re-exports so card defs can use it without a full path.

## Card Definition Fixes

### dragons_hoard.rs
**Oracle text**: Whenever a Dragon you control enters, put a gold counter on this artifact. / {T}, Remove a gold counter from this artifact: Draw a card. / {T}: Add one mana of any color.
**Current state**: TODO on line 6 and 35 -- missing {T}, Remove gold counter: Draw a card ability.
**Fix**: Add `AbilityDefinition::Activated` with:
- `cost: Cost::Sequence(vec![Cost::Tap, Cost::RemoveCounter { counter: CounterType::Custom("gold".into()), count: 1 }])`
- `effect: Effect::DrawCards { target: EffectTarget::Controller, count: 1 }`

### spawning_pit.rs
**Oracle text**: Sacrifice a creature: Put a charge counter on this artifact. / {1}, Remove two charge counters from this artifact: Create a 2/2 colorless Spawn artifact creature token.
**Current state**: TODO on line 7 -- missing remove-counter cost ability.
**Fix**: Add activated ability with:
- `cost: Cost::Sequence(vec![Cost::Mana(ManaCost { generic: 1, ..Default::default() }), Cost::RemoveCounter { counter: CounterType::Charge, count: 2 }])`
- `effect: Effect::CreateToken { ... }` (2/2 colorless Spawn artifact creature)

### ominous_seas.rs
**Oracle text**: Whenever you draw a card, put a foreshadow counter on this enchantment. / Remove eight foreshadow counters from this enchantment: Create an 8/8 blue Kraken creature token. / Cycling {2}
**Current state**: TODO on line 27 -- missing remove-counter ability.
**Fix**: Add activated ability with:
- `cost: Cost::RemoveCounter { counter: CounterType::Custom("foreshadow".into()), count: 8 }`
- `effect: Effect::CreateToken { ... }` (8/8 blue Kraken creature token)

### gemstone_array.rs
**Oracle text**: {2}: Put a charge counter on this artifact. / Remove a charge counter from this artifact: Add one mana of any color.
**Current state**: TODO on line 26 -- missing remove-counter mana ability.
**Fix**: Add activated ability with:
- `cost: Cost::RemoveCounter { counter: CounterType::Charge, count: 1 }`
- `effect: Effect::AddMana { ... }` (one mana of any color)
Note: Technically a mana ability (CR 605.1). For this batch, implement as a regular activated ability (uses the stack). Document mana-ability classification gap as PB-37 follow-up.

### golgari_grave_troll.rs
**Oracle text**: This creature enters with a +1/+1 counter on it for each creature card in your graveyard. / {1}, Remove a +1/+1 counter from this creature: Regenerate this creature. / Dredge 6
**Current state**: TODO on line 12 -- missing regeneration cost with counter removal.
**Fix**: Add activated ability with:
- `cost: Cost::Sequence(vec![Cost::Mana(ManaCost { generic: 1, ..Default::default() }), Cost::RemoveCounter { counter: CounterType::PlusOnePlusOne, count: 1 }])`
- `effect: Effect::Regenerate { target: EffectTarget::Source }`
Note: Check if `Effect::Regenerate` exists. If not, this card remains partially blocked on the regeneration effect (counter cost is still useful).

### ghave_guru_of_spores.rs
**Oracle text**: Ghave enters with five +1/+1 counters on it. / {1}, Remove a +1/+1 counter from a creature you control: Create a 1/1 green Saproling creature token. / {1}, Sacrifice a creature: Put a +1/+1 counter on target creature.
**Current state**: TODO on line 20 -- missing both activated abilities.
**Fix**:
- Ability 1: `cost: Cost::Sequence(vec![Cost::Mana(...), Cost::RemoveCounter { counter: CounterType::PlusOnePlusOne, count: 1 }])`, `effect: Effect::CreateToken { ... }` (1/1 green Saproling)
- Ability 2: `cost: Cost::Sequence(vec![Cost::Mana(...), Cost::Sacrifice(TargetFilter { is_creature: Some(true), ..Default::default() })])`, `effect: Effect::AddCounters { target: EffectTarget::Target(0), counter: CounterType::PlusOnePlusOne, count: 1 }`
Note: Ghave's first ability says "from a creature you control" -- this removes from ANY creature you control, not just Ghave. `Cost::RemoveCounter` removes from the source permanent. For this batch, implement as removing from self. Document "from another creature" as PB-37 enhancement.

### spike_weaver.rs
**Oracle text**: This creature enters with three +1/+1 counters on it. / {2}, Remove a +1/+1 counter from this creature: Put a +1/+1 counter on target creature. / {1}, Remove a +1/+1 counter from this creature: Prevent all combat damage that would be dealt this turn.
**Current state**: TODO on lines 29-32 -- both abilities missing.
**Fix**:
- Ability 1: `cost: Cost::Sequence(vec![Cost::Mana({ generic: 2, .. }), Cost::RemoveCounter { counter: PlusOnePlusOne, count: 1 }])`, `effect: Effect::AddCounters { target: EffectTarget::Target(0), counter: PlusOnePlusOne, count: 1 }`
- Ability 2: Blocked on `Effect::PreventAllCombatDamage` (G-19, PB-32). Add cost but leave TODO for the effect.

### ramos_dragon_engine.rs
**Oracle text**: Flying / Whenever you cast a spell, put a +1/+1 counter on Ramos for each of that spell's colors. / Remove five +1/+1 counters from Ramos: Add {W}{W}{U}{U}{B}{B}{R}{R}{G}{G}. Activate only once each turn.
**Current state**: TODO on line 26 -- missing remove-counter mana ability + once-per-turn.
**Fix**: Add activated ability with:
- `cost: Cost::RemoveCounter { counter: CounterType::PlusOnePlusOne, count: 5 }`
- `effect: Effect::AddMana { ... }` (WWUUBBRRGG)
Note: Once-per-turn restriction not expressible yet. Implement without restriction, add TODO. Mana-ability caveat same as Gemstone Array.

### druids_repository.rs
**Oracle text**: Whenever a creature you control attacks, put a charge counter on this enchantment. / Remove a charge counter from this enchantment: Add one mana of any color.
**Current state**: TODO on line 26 -- missing remove-counter mana ability.
**Fix**: Add activated ability with:
- `cost: Cost::RemoveCounter { counter: CounterType::Charge, count: 1 }`
- `effect: Effect::AddMana { ... }` (any color)
Same mana-ability caveat as Gemstone Array.

### umezawas_jitte.rs
**Oracle text**: Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte. / Remove a charge counter: Choose one -- +2/+2, -1/-1, or gain 2 life. / Equip {2}
**Current state**: TODO on line 26 -- missing both combat trigger and modal remove-counter ability.
**Fix**: Add activated ability with:
- `cost: Cost::RemoveCounter { counter: CounterType::Charge, count: 1 }`
- Modal effect -- check if `AbilityDefinition::Activated` supports modes. If not, implement as the first mode (+2/+2) with TODO for full modal support.
Note: Combat trigger ("whenever equipped creature deals combat damage") was added in PB-30 infrastructure. Wire the trigger to put 2 charge counters on Jitte.

### crucible_of_the_spirit_dragon.rs
**Oracle text**: {T}: Add {C}. / {1}, {T}: Put a storage counter. / {T}, Remove X storage counters: Add X mana (Dragon restricted).
**Current state**: TODO on line 24 -- missing remove-X-counters ability.
**Fix**: Defer to PB-37. "Remove X counters" requires X-value integration in `Cost::RemoveCounter` (count driven by x_value).

### tekuthal_inquiry_dominus.rs
**Oracle text**: (Proliferate doubling) / {1}{U/P}{U/P}, Remove three counters from among other permanents: Indestructible counter on self.
**Current state**: TODO on line 33 -- "remove counters from others" is beyond `Cost::RemoveCounter`.
**Fix**: Defer to PB-37. Requires multi-source counter removal.

### village_rites.rs
**Oracle text**: As an additional cost to cast this spell, sacrifice a creature. / Draw two cards.
**Current state**: TODO on line 5 -- sacrifice cost not enforced.
**Fix**: Add `spell_additional_costs: vec![SpellAdditionalCost::SacrificeCreature]` to the card def.

### deadly_dispute.rs
**Oracle text**: As an additional cost to cast this spell, sacrifice an artifact or creature. / Draw two cards and create a Treasure token.
**Current state**: TODO on line 3 -- sacrifice cost not enforced.
**Fix**: Add `spell_additional_costs: vec![SpellAdditionalCost::SacrificeArtifactOrCreature]` to the card def.

### goblin_grenade.rs
**Oracle text**: As an additional cost to cast this spell, sacrifice a Goblin. / Goblin Grenade deals 5 damage to any target.
**Current state**: TODO on line 9 -- sacrifice cost not enforced.
**Fix**: Add `spell_additional_costs: vec![SpellAdditionalCost::SacrificeSubtype(SubType::Goblin)]` to the card def.

### altar_of_bone.rs
**Oracle text**: As an additional cost to cast this spell, sacrifice a creature. / Search library for a creature card.
**Current state**: TODO on line 9 -- sacrifice cost not enforced.
**Fix**: Add `spell_additional_costs: vec![SpellAdditionalCost::SacrificeCreature]` to the card def.

### crop_rotation.rs
**Oracle text**: As an additional cost to cast this spell, sacrifice a land. / Search library for a land card, put onto battlefield.
**Current state**: TODO on line 3 -- sacrifice cost not enforced.
**Fix**: Add `spell_additional_costs: vec![SpellAdditionalCost::SacrificeLand]` to the card def.

### corrupted_conviction.rs
**Oracle text**: As an additional cost to cast this spell, sacrifice a creature. / Draw two cards.
**Current state**: TODO on line 14 -- sacrifice cost not enforced.
**Fix**: Add `spell_additional_costs: vec![SpellAdditionalCost::SacrificeCreature]` to the card def.

### lifes_legacy.rs
**Oracle text**: As an additional cost to cast this spell, sacrifice a creature. / Draw cards equal to the sacrificed creature's power.
**Current state**: TODO on lines 4-8 -- sacrifice cost not enforced + power-based draw missing.
**Fix**:
- Add `spell_additional_costs: vec![SpellAdditionalCost::SacrificeCreature]` to the card def.
- The draw-count-equals-power requires `EffectAmount::SacrificedCreaturePower` or similar. For this batch, add the sacrifice cost. Defer power-based draw count to PB-37.

### abjure.rs
**Oracle text**: As an additional cost to cast this spell, sacrifice a blue permanent. / Counter target spell.
**Current state**: TODO on line 13 -- sacrifice cost not enforced.
**Fix**: Add `spell_additional_costs: vec![SpellAdditionalCost::SacrificeColorPermanent(Color::Blue)]` to the card def.

### plumb_the_forbidden.rs (DEFERRED)
**Oracle text**: As an additional cost, you may sacrifice one or more creatures. When you do, copy this spell for each creature sacrificed this way. / You draw a card and you lose 1 life.
**Current state**: TODO on line 15.
**Fix**: Defer to PB-37. "May sacrifice one or more" + copy-per-sacrifice is beyond the simple mandatory sacrifice model.

### flare_of_fortitude.rs (DEFERRED)
**Oracle text**: You may sacrifice a nontoken white creature rather than pay this spell's mana cost.
**Current state**: TODO on line 14.
**Fix**: Defer to PB-37. This is an alternative cost ("rather than pay"), not an additional cost.

## New Card Definitions

None -- all affected cards already have defs. This batch fixes existing TODOs only.

## Unit Tests

**File**: `crates/engine/tests/cost_primitives.rs`
**Tests to write**:
- `test_remove_counter_cost_basic` -- activate ability with remove-counter cost on a permanent with sufficient counters, verify counter decremented and ability resolves. CR 602.2.
- `test_remove_counter_cost_insufficient` -- try to activate with insufficient counters, verify `InvalidCommand` error. CR 118.3.
- `test_remove_counter_cost_in_sequence` -- `Cost::Sequence` with Tap + Mana + RemoveCounter, verify all costs paid and all events emitted. CR 601.2h.
- `test_remove_counter_cost_exact_zero` -- remove the last counter (e.g., 1 counter, remove 1), verify counter entry removed from map (not set to 0). CR 118.3.
- `test_spell_sacrifice_cost_creature` -- cast Village Rites with `AdditionalCost::Sacrifice`, verify creature sacrificed at cast time and cards drawn on resolution. CR 118.8.
- `test_spell_sacrifice_cost_missing` -- cast Village Rites WITHOUT providing a sacrifice, verify error from casting.rs validation. CR 118.8.
- `test_spell_sacrifice_cost_wrong_type` -- try to sacrifice a non-creature for Village Rites, verify error (filter mismatch).
- `test_spell_sacrifice_cost_land` -- cast Crop Rotation with land sacrifice, verify land sacrificed. CR 118.8.
- `test_spell_sacrifice_cost_artifact_or_creature` -- cast Deadly Dispute with artifact sacrifice, verify artifact sacrificed and draw + treasure created. CR 118.8.
- `test_spell_sacrifice_cost_subtype` -- cast Goblin Grenade sacrificing a Goblin creature, verify goblin sacrificed and 5 damage dealt. CR 118.8.

**Pattern**: Follow tests in `crates/engine/tests/card_def_fixes.rs` for card-specific tests and `crates/engine/tests/activated_abilities.rs` for activation cost tests.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved
- [ ] New card defs authored (if any)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs (except explicitly deferred items)

## Risks & Edge Cases

1. **Mana ability gap**: Gemstone Array, Druids' Repository, and Ramos's "remove counter" abilities produce mana. Technically these are mana abilities (produce mana, no target, not loyalty -- CR 605.1). Mana abilities resolve immediately without the stack and go through `TapForMana`, not `ActivateAbility`. The `Cost::RemoveCounter` is only wired into `handle_activate_ability`. For this batch, implement as regular activated abilities (use the stack). Document the mana-ability classification gap as a follow-up. In practice, the timing difference rarely matters in Commander games.

2. **"Remove from another creature" (Ghave)**: Ghave's first ability says "Remove a +1/+1 counter from a creature you control" -- not necessarily from Ghave itself. The `Cost::RemoveCounter` as designed removes from the SOURCE permanent. Supporting removal from other permanents needs a different mechanism (e.g., a target + cost combo). For this batch, implement Ghave's ability as removing from self. Document the gap.

3. **"Remove X counters" (Crucible of the Spirit Dragon)**: The `count` field is a fixed `u32`. Cards with "Remove X counters" need the count to be the X value. This could be supported by `count: 0` meaning "use x_value" or by a separate variant. Defer X-count to PB-37.

4. **Spell sacrifice validation timing**: The sacrifice must happen at cast time (CR 601.2h), BEFORE the spell goes on the stack. The permanent is sacrificed as a cost -- if the spell is countered, the sacrifice is still lost. The existing Bargain/Emerge/Casualty sacrifice paths in `casting.rs` already handle this correctly. The new validation follows the same pattern.

5. **Life's Legacy power tracking**: "Draw cards equal to the sacrificed creature's power" requires knowing the power at sacrifice time (LKI). This may need a new `EffectAmount` variant or context field. Partially blocked -- implement sacrifice cost, defer power-based draw.

6. **Abjure color check**: "Sacrifice a blue permanent" requires checking the permanent's color using layer-resolved characteristics (a permanent might be blue due to continuous effects). Use `calculate_characteristics()` for the color check.

7. **SacrificeFilter vs SpellAdditionalCost overlap**: `SacrificeFilter` (on `ActivationCost`) and `SpellAdditionalCost` serve similar roles. They are intentionally separate because they live in different contexts (activated ability cost vs spell card definition). The validation logic in `casting.rs` can reuse the same filter-matching helper.

8. **Disambiguation with Bargain/Casualty**: When `casting.rs` sees `AdditionalCost::Sacrifice`, it currently checks for Bargain/Casualty keywords to determine what the sacrifice is for. The new `spell_additional_costs` validation must run BEFORE the keyword-specific checks, or must not conflict with them. Cards with `spell_additional_costs` should NOT also have Bargain/Casualty keywords (they are different mechanics), so there should be no conflict. But verify this during implementation.

## Deferred Items (carry forward to PB-37)

- Ghave "remove counter from another creature you control" -- needs target-based counter cost
- Crucible of the Spirit Dragon "Remove X counters" -- needs X-value integration in Cost
- Tekuthal "remove counters from among others" -- complex multi-source counter removal
- Plumb the Forbidden "may sacrifice one or more" -- variable-count sacrifice + copy-per
- Flare of Fortitude sacrifice as alternative cost -- needs AltCostKind extension
- Ramos once-per-turn activation restriction -- needs ActivatedAbility once_per_turn field
- Mana ability classification for counter-removal abilities that produce mana
- Life's Legacy EffectAmount::SacrificedCreaturePower
- Spike Weaver ability 2 effect (PreventAllCombatDamage -- G-19, PB-32)
