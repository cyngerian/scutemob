# Primitive Batch Plan: PB-32 -- Static/Effect Primitives (Lands, Prevention, Control, Animation)

**Generated**: 2026-03-26
**Primitive**: Four DSL gaps: additional land plays (G-18), damage prevention effects (G-19), control change effects (G-20), land animation convenience (G-21)
**CR Rules**: 305.2, 305.2a, 305.2b (additional land plays); 615.1-615.12 (prevention effects); 613.1b (Layer 2 control-changing); 701.12, 701.12a-701.12b (exchange control); 613.1d, 613.1f, 613.4b (type-changing / animation layers)
**Cards affected**: ~33 (6 G-18 + 8 G-19 + 11 G-20 + 8 G-21)
**Dependencies**: PB-31 (DONE -- Cost::RemoveCounter used by Spike Weaver ability 1)
**Deferred items from prior PBs**: Spike Weaver ability 2 (PreventAllCombatDamage -- explicitly deferred from PB-31, now in scope for G-19)

---

## Primitive Specification

This batch adds four related DSL capabilities:

1. **G-18: Additional land plays** -- A new `Effect::AdditionalLandPlay` that increments `land_plays_remaining` on the controller. For static permanents, a new `AbilityDefinition::AdditionalLandPlays` that is registered in `register_static_continuous_effects` to increment land plays at the start of each turn (CR 305.2).

2. **G-19: Damage prevention effects** -- A new `Effect::PreventAllCombatDamage` that registers a replacement effect preventing all combat damage this turn. Also `Effect::PreventDamageToTarget` for per-creature prevention (Maze of Ith, Kor Haven). These leverage the existing `apply_damage_prevention` infrastructure.

3. **G-20: Control change effects** -- A new `Effect::GainControl` that creates a Layer 2 `SetController` continuous effect on the target permanent. The `LayerModification::SetController` and `EffectLayer::Control` already exist; this adds the Effect variant to wire them from card definitions.

4. **G-21: Land animation** -- No new engine Effect needed. The multi-step `ApplyContinuousEffect` pattern already works (proven by Inkmoth Nexus, Blinkmoth Nexus). Cards with TODO markers just need the verbose pattern authored. A helper function `animate_land_effects()` can reduce boilerplate.

---

## CR Rule Text

### CR 305.2 (Additional Land Plays)

> 305.2. A player can normally play one land during their turn; however, continuous effects may increase this number.
>
> 305.2a To determine whether a player can play a land, compare the number of lands the player can play this turn with the number of lands they have already played this turn (including lands played as special actions and lands played during the resolution of spells and abilities). If the number of lands the player can play is greater, the play is legal.
>
> 305.2b A player can't play a land, for any reason, if the number of lands the player can play this turn is equal to or less than the number of lands they have already played this turn. Ignore any part of an effect that instructs a player to do so.

### CR 615.1 (Prevention Effects)

> 615.1. Some continuous effects are prevention effects. Like replacement effects, prevention effects apply continuously as events happen -- they aren't locked in ahead of time. Such effects watch for a damage event that would happen and completely or partially prevent the damage that would be dealt. They act like "shields" around whatever they're affecting.

### CR 613.1b (Layer 2: Control-Changing)

> 613.1b Layer 2: Control-changing effects are applied.

### CR 701.12 (Exchange)

> 701.12a A spell or ability may instruct players to exchange something (for example, life totals or control of two permanents) as part of its resolution. When such a spell or ability resolves, if the entire exchange can't be completed, no part of the exchange occurs.
>
> 701.12b When control of two permanents is exchanged, if those permanents are controlled by different players, each of those players simultaneously gains control of the permanent that was controlled by the other player. If, on the other hand, those permanents are controlled by the same player, the exchange effect does nothing.

---

## Engine Changes

### Change 1: Effect::AdditionalLandPlay (G-18)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new Effect variant after `CreateEmblem` (line ~1637):

```rust
/// CR 305.2: Grant the controller one additional land play this turn.
///
/// Increments `land_plays_remaining` on the effect's controller by 1.
/// Used by Explore ("You may play an additional land this turn") and
/// similar one-shot effects from spells.
AdditionalLandPlay,
```

**Pattern**: Follow `Effect::BecomeMonarch` (simple player-targeted effect, line 1321)

### Change 2: AbilityDefinition::AdditionalLandPlays (G-18)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new AbilityDefinition variant after `CdaPowerToughness`:

```rust
/// CR 305.2: Static "You may play an additional land on each of your turns."
///
/// Registered in `register_static_continuous_effects`. When the source permanent
/// is on the battlefield, the controller's `land_plays_remaining` is incremented
/// by `count` at the start of each of their turns.
///
/// Stacks with multiple sources (two Aesi = 3 total land plays per turn).
AdditionalLandPlays { count: u32 },
```

### Change 3: Effect::AdditionalLandPlay dispatch (G-18)

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add dispatch arm for `Effect::AdditionalLandPlay` (after `Effect::Repeat`, line ~2143):

```rust
Effect::AdditionalLandPlay => {
    // CR 305.2: Increment controller's land plays remaining.
    let controller = ctx.controller;
    if let Some(p) = state.players.get_mut(&controller) {
        p.land_plays_remaining += 1;
    }
}
```

### Change 4: AdditionalLandPlays registration (G-18)

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: Add match arm in `register_static_continuous_effects` (after `CdaPowerToughness`, line ~1786):

```rust
AbilityDefinition::AdditionalLandPlays { count } => {
    // Store on a per-permanent tracker; applied in reset_turn_state.
    state.additional_land_play_sources.push_back(
        crate::state::stubs::AdditionalLandPlaySource {
            source: new_id,
            controller,
            count: *count,
        }
    );
}
```

### Change 5: AdditionalLandPlaySource struct + GameState field (G-18)

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add struct:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdditionalLandPlaySource {
    pub source: ObjectId,
    pub controller: PlayerId,
    pub count: u32,
}
```

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add field to `GameState`:

```rust
/// Static "additional land play" sources from permanents on the battlefield (CR 305.2).
pub additional_land_play_sources: im::Vector<crate::state::stubs::AdditionalLandPlaySource>,
```

### Change 6: Apply additional land plays at turn start (G-18)

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: After `p.land_plays_remaining = 1;` (line 1297), add:

```rust
// CR 305.2: Apply static additional land play effects.
let additional: u32 = state
    .additional_land_play_sources
    .iter()
    .filter(|s| s.controller == player
        && state.objects.get(&s.source)
            .map(|o| state.zones.battlefield.contains(&s.source))
            .unwrap_or(false))
    .map(|s| s.count)
    .sum();
if let Some(p) = state.players.get_mut(&player) {
    p.land_plays_remaining += additional;
}
```

### Change 7: Clean up departed AdditionalLandPlaySources (G-18)

**File**: `crates/engine/src/rules/layers.rs`
**Action**: In `remove_expired_continuous_effects` (or `check_and_remove_stale_statics`), remove sources whose source object is no longer on the battlefield. Follow the existing pattern for `trigger_doublers` / `etb_suppressors` / `restrictions` cleanup.

### Change 8: Effect::PreventAllCombatDamage (G-19)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new Effect variant:

```rust
/// CR 615.1: Prevent all combat damage that would be dealt this turn.
///
/// Registers a blanket damage prevention replacement that prevents
/// ALL combat damage events for the remainder of the turn. The
/// prevention is applied in `apply_combat_damage_assignments` in
/// combat.rs by checking a `prevent_all_combat_damage_this_turn` flag.
///
/// Used by Fog, Spike Weaver ability 2, Inkshield, etc.
PreventAllCombatDamage,
```

### Change 9: PreventAllCombatDamage flag on GameState (G-19)

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add field to `GameState`:

```rust
/// CR 615.1: When true, all combat damage is prevented for the rest of the turn.
/// Set by Effect::PreventAllCombatDamage. Reset in reset_turn_state / cleanup.
#[serde(default)]
pub prevent_all_combat_damage: bool,
```

### Change 10: PreventAllCombatDamage dispatch (G-19)

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add dispatch arm:

```rust
Effect::PreventAllCombatDamage => {
    state.prevent_all_combat_damage = true;
}
```

### Change 11: Enforce prevention in combat damage (G-19)

**File**: `crates/engine/src/rules/combat.rs`
**Action**: In `apply_combat_damage_assignments` (line ~1458), BEFORE the damage-doubling and individual prevention checks, add an early-return path:

```rust
// CR 615.1: If all combat damage is prevented this turn, skip all assignments.
if state.prevent_all_combat_damage {
    // Emit events for each would-be assignment showing prevention.
    // Return empty assignment list -- no damage dealt.
    return (state, vec![/* prevention events */]);
}
```

This check goes before the per-assignment loop at line ~1484.

### Change 12: Reset prevention flag (G-19)

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: In `reset_turn_state` (around line 1297), add:

```rust
state.prevent_all_combat_damage = false;
```

### Change 13: Effect::PreventCombatDamageFromTarget (G-19)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new Effect variant for per-creature prevention:

```rust
/// CR 615.1: Prevent all combat damage that would be dealt to and/or by
/// the target creature this turn.
///
/// Used by Maze of Ith ("prevent all combat damage dealt to and by that
/// creature"), Kor Haven ("prevent all combat damage dealt by target
/// attacking creature").
PreventCombatDamageFromOrTo {
    target: EffectTarget,
    /// If true, prevent damage dealt BY the target.
    prevent_from: bool,
    /// If true, prevent damage dealt TO the target.
    prevent_to: bool,
},
```

### Change 14: PreventCombatDamageFromOrTo dispatch + tracking (G-19)

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Resolve the target to an ObjectId, add it to a set on GameState:

```rust
Effect::PreventCombatDamageFromOrTo { target, prevent_from, prevent_to } => {
    let obj_ids = resolve_effect_targets(state, target, ctx);
    for obj_id in obj_ids {
        if *prevent_from {
            state.combat_damage_prevented_from.insert(obj_id);
        }
        if *prevent_to {
            state.combat_damage_prevented_to.insert(obj_id);
        }
    }
}
```

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add fields:

```rust
/// Objects whose combat damage output is prevented this turn (CR 615).
pub combat_damage_prevented_from: im::OrdSet<ObjectId>,
/// Objects whose combat damage input is prevented this turn (CR 615).
pub combat_damage_prevented_to: im::OrdSet<ObjectId>,
```

Reset both in `reset_turn_state`.

### Change 15: Enforce per-creature prevention in combat.rs (G-19)

**File**: `crates/engine/src/rules/combat.rs`
**Action**: In the per-assignment loop (~line 1490), before `apply_damage_prevention`, check:

```rust
// CR 615: Per-creature combat damage prevention
if state.combat_damage_prevented_from.contains(&source_id) {
    // Skip this assignment -- damage from this source is prevented
    continue;
}
if state.combat_damage_prevented_to.contains(&target_id) {
    // Skip this assignment -- damage to this target is prevented
    continue;
}
```

### Change 16: Effect::GainControl (G-20)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new Effect variant:

```rust
/// CR 613.1b: Gain control of a target permanent (Layer 2 control-changing effect).
///
/// Creates a `ContinuousEffect` with `LayerModification::SetController` on the
/// target permanent for the specified duration.
///
/// Used by Zealous Conscripts ("gain control ... until end of turn"),
/// Connive ("gain control ... indefinitely"), Dragonlord Silumgar
/// ("gain control ... for as long as you control [this]").
GainControl {
    target: EffectTarget,
    duration: EffectDuration,
},
```

### Change 17: Effect::GainControl dispatch (G-20)

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add dispatch arm (after `ApplyContinuousEffect`):

```rust
Effect::GainControl { target, duration } => {
    let obj_ids = resolve_effect_targets(state, target, ctx);
    for obj_id in obj_ids {
        // CR 613.1b: Create a Layer 2 control-changing continuous effect.
        let id_inner = state.next_object_id().0;
        let ts = state.timestamp_counter;
        state.timestamp_counter += 1;
        let controller = ctx.controller;
        let eff = crate::state::continuous_effect::ContinuousEffect {
            id: crate::state::continuous_effect::EffectId(id_inner),
            source: Some(ctx.source),
            layer: crate::state::continuous_effect::EffectLayer::Control,
            modification: crate::state::continuous_effect::LayerModification::SetController(controller),
            filter: crate::state::continuous_effect::EffectFilter::SingleObject(obj_id),
            duration: *duration,
            is_cda: false,
            timestamp: ts,
            condition: None,
        };
        state.continuous_effects.push_back(eff);

        // Update the object's controller immediately for correct game state.
        if let Some(obj) = state.objects.get_mut(&obj_id) {
            obj.controller = controller;
        }
    }
}
```

### Change 18: Effect::ExchangeControl (G-20)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new Effect variant:

```rust
/// CR 701.12b: Exchange control of two permanents.
///
/// If the permanents are controlled by different players, each player
/// simultaneously gains control of the other's permanent. If controlled
/// by the same player, nothing happens (CR 701.12b).
///
/// CR 701.12a: If the entire exchange can't be completed, no part occurs.
ExchangeControl {
    target_a: EffectTarget,
    target_b: EffectTarget,
    duration: EffectDuration,
},
```

### Change 19: ExchangeControl dispatch (G-20)

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add dispatch arm:

```rust
Effect::ExchangeControl { target_a, target_b, duration } => {
    let a_ids = resolve_effect_targets(state, target_a, ctx);
    let b_ids = resolve_effect_targets(state, target_b, ctx);
    if let (Some(&a_id), Some(&b_id)) = (a_ids.first(), b_ids.first()) {
        let a_ctrl = state.objects.get(&a_id).map(|o| o.controller);
        let b_ctrl = state.objects.get(&b_id).map(|o| o.controller);
        if let (Some(ac), Some(bc)) = (a_ctrl, b_ctrl) {
            if ac != bc {
                // CR 701.12b: Exchange -- each gets the other's controller.
                // Create two Layer 2 control-changing effects.
                for (obj_id, new_controller) in [(a_id, bc), (b_id, ac)] {
                    let id_inner = state.next_object_id().0;
                    let ts = state.timestamp_counter;
                    state.timestamp_counter += 1;
                    state.continuous_effects.push_back(
                        crate::state::continuous_effect::ContinuousEffect {
                            id: crate::state::continuous_effect::EffectId(id_inner),
                            source: Some(ctx.source),
                            layer: crate::state::continuous_effect::EffectLayer::Control,
                            modification: crate::state::continuous_effect::LayerModification::SetController(new_controller),
                            filter: crate::state::continuous_effect::EffectFilter::SingleObject(obj_id),
                            duration: *duration,
                            is_cda: false,
                            timestamp: ts,
                            condition: None,
                        },
                    );
                    if let Some(obj) = state.objects.get_mut(&obj_id) {
                        obj.controller = new_controller;
                    }
                }
            }
            // CR 701.12b: Same controller = do nothing.
        }
    }
}
```

### Change 20: Layer 2 enforcement in calculate_characteristics (G-20)

**File**: `crates/engine/src/rules/layers.rs`
**Action**: Verify that `SetController` effects are applied to `obj.controller` during the layer loop. Currently the match arm at line 911 is a no-op comment saying "Handled outside calculate_characteristics." This is correct for static abilities (whose controller is set at registration). For dynamic control-change effects from spells, the controller is set immediately in the Effect dispatch (Change 17). No additional layer enforcement needed -- the existing design is sufficient.

### Change 21: Exhaustive match updates for new Effect variants

Files requiring new match arms for `Effect::AdditionalLandPlay`, `Effect::PreventAllCombatDamage`, `Effect::PreventCombatDamageFromOrTo`, `Effect::GainControl`, `Effect::ExchangeControl`:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/state/hash.rs` | Effect HashInto | L4766+ | Add discriminants 68-72 for 5 new variants |
| `crates/engine/src/effects/mod.rs` | execute_effect_inner | L211+ | Add 5 dispatch arms (Changes 3, 10, 14, 17, 19) |

Files requiring new match arm for `AbilityDefinition::AdditionalLandPlays`:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/state/hash.rs` | AbilityDefinition HashInto | L5211+ | Add discriminant 71 |
| `crates/engine/src/rules/replacement.rs` | register_static_continuous_effects | L1787 | Add registration arm (Change 4) |
| `crates/engine/src/rules/abilities.rs` | Any exhaustive AbilityDefinition matches | Search | Add `_ => {}` or specific arm |

### Change 22: Hash updates for new GameState fields (G-18, G-19)

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add to GameState HashInto:
- `additional_land_play_sources` (vector of structs -- hash each entry)
- `prevent_all_combat_damage` (bool)
- `combat_damage_prevented_from` (OrdSet)
- `combat_damage_prevented_to` (OrdSet)

### Change 23: GameStateBuilder defaults (G-18, G-19)

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add defaults for new GameState fields:
- `additional_land_play_sources: im::Vector::new()`
- `prevent_all_combat_damage: false`
- `combat_damage_prevented_from: im::OrdSet::new()`
- `combat_damage_prevented_to: im::OrdSet::new()`

### Change 24: Helpers.rs exports (G-18)

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: No new exports needed -- `EffectDuration` is already exported.

### Change 25: animate_land_effects helper function (G-21)

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: Add convenience function to reduce boilerplate in land animation card defs:

```rust
/// Build the standard "this land becomes a P/T [subtypes] creature with [keywords]
/// until end of turn, it's still a land" effect sequence.
///
/// Produces a Sequence of ApplyContinuousEffect calls:
/// - Layer 4: AddCardTypes({Creature} + extra_types)
/// - Layer 4: AddSubtypes(subtypes)
/// - Layer 7b: SetPowerToughness
/// - Layer 6: AddKeywords (if any)
/// - Layer 5: SetColors (if any)
pub fn animate_land_effects(
    power: i32,
    toughness: i32,
    subtypes: &[&str],
    keywords: &[KeywordAbility],
    colors: &[Color],
    extra_card_types: &[CardType],
    duration: EffectDuration,
) -> Effect {
    // ... builds Effect::Sequence of ApplyContinuousEffect calls
}
```

---

## Card Definition Fixes

### G-18: Additional Land Plays (6 cards)

#### explore.rs
**Oracle text**: "You may play an additional land this turn. Draw a card."
**Current state**: TODO -- draw only, additional land play omitted
**Fix**: Replace spell effect with `Effect::Sequence(vec![Effect::AdditionalLandPlay, Effect::DrawCards { ... }])`

#### urban_evolution.rs
**Oracle text**: "Draw three cards. You may play an additional land this turn."
**Current state**: TODO -- additional land play omitted
**Fix**: Add `Effect::AdditionalLandPlay` to the spell effect sequence

#### aesi_tyrant_of_gyre_strait.rs
**Oracle text**: "You may play an additional land on each of your turns."
**Current state**: TODO -- static additional land play omitted
**Fix**: Add `AbilityDefinition::AdditionalLandPlays { count: 1 }` to abilities

#### mina_and_denn_wildborn.rs
**Oracle text**: "You may play an additional land on each of your turns."
**Current state**: TODO -- static additional land play omitted
**Fix**: Add `AbilityDefinition::AdditionalLandPlays { count: 1 }` to abilities

#### wayward_swordtooth.rs
**Oracle text**: "You may play an additional land on each of your turns."
**Current state**: TODO -- additional land play omitted
**Fix**: Add `AbilityDefinition::AdditionalLandPlays { count: 1 }` to abilities

#### druid_class.rs
**Oracle text**: Level 2: "You may play an additional land on each of your turns."
**Current state**: TODO -- level 2 additional land play not expressible
**Fix**: Add `AbilityDefinition::AdditionalLandPlays { count: 1 }` as effect of ClassLevel 2 activation. Note: Class levels are activated abilities that gain static effects -- the AdditionalLandPlays effect should be granted when level 2 is achieved. This may require the runner to check how ClassLevel resolution works and wire accordingly.

### G-19: Damage Prevention (8 cards)

#### spike_weaver.rs
**Oracle text**: "{1}, Remove a +1/+1 counter: Prevent all combat damage that would be dealt this turn."
**Current state**: TODO -- ability 2 omitted, deferred from PB-31
**Fix**: Add third ability:
```rust
AbilityDefinition::Activated {
    cost: Cost::Sequence(vec![
        Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
        Cost::RemoveCounter { counter: CounterType::PlusOnePlusOne, count: 1 },
    ]),
    effect: Effect::PreventAllCombatDamage,
    timing_restriction: None,
    targets: vec![],
    activation_condition: None,
},
```

#### maze_of_ith.rs
**Oracle text**: "{T}: Untap target attacking creature. Prevent all combat damage that would be dealt to and dealt by that creature this turn."
**Current state**: Empty abilities -- stripped per W5 policy
**Fix**: Add activated ability with `Effect::Sequence(vec![Effect::UntapPermanent { target: DeclaredTarget(0) }, Effect::PreventCombatDamageFromOrTo { target: DeclaredTarget(0), prevent_from: true, prevent_to: true }])`. Note: targeting "attacking creature" needs `TargetRequirement::TargetAttackingCreature` -- check if it exists, otherwise use `TargetCreature` as approximation.

#### kor_haven.rs
**Oracle text**: "{1}{W}, {T}: Prevent all combat damage that would be dealt by target attacking creature this turn."
**Current state**: TODO -- prevention effect omitted
**Fix**: Add activated ability with `Effect::PreventCombatDamageFromOrTo { target: DeclaredTarget(0), prevent_from: true, prevent_to: false }`.

#### iroas_god_of_victory.rs
**Oracle text**: "Prevent all damage that would be dealt to attacking creatures you control."
**Current state**: TODO -- blanket prevention for attacking creatures omitted
**Fix**: This is a STATIC prevention, not a one-shot. It requires a replacement effect registered as `AbilityDefinition::Replacement` that filters on "damage dealt to attacking creatures you control." This is more complex than the one-shot Fog pattern. Defer to a note: the runner should register a replacement effect with `ReplacementTrigger::WouldDealDamage` filtered to attacking creatures controlled by the source's controller. If the replacement infrastructure doesn't support this filter scope, leave as TODO with a note that it needs a new `DamageTargetFilter` variant.

#### galadhrim_ambush.rs
**Oracle text**: "Prevent all combat damage that would be dealt this turn by non-Elf creatures."
**Current state**: TODO -- both token creation and prevention omitted
**Fix**: The prevention here is filtered ("by non-Elf creatures") which the blanket `PreventAllCombatDamage` does not support. Leave the prevention as TODO. Fix only the token creation part if `EffectAmount::AttackingCreatureCount` exists (check PB-26/28 -- likely does not). If not expressible, leave as TODO.

#### inkshield.rs
**Oracle text**: "Prevent all combat damage that would be dealt to you this turn. For each 1 damage prevented this way, create a 2/1 white and black Inkling creature token with flying."
**Current state**: TODO -- prevention + variable token creation
**Fix**: The "for each 1 damage prevented" token creation requires tracking prevented damage amount, which is beyond the scope of the simple `PreventAllCombatDamage` flag. Leave as TODO -- this needs a more sophisticated prevention tracking mechanism.

#### crystal_barricade.rs
**Oracle text**: "Prevent all noncombat damage that would be dealt to other creatures you control."
**Current state**: TODO -- noncombat damage prevention not expressible
**Fix**: This is NON-combat damage prevention, which is a different filter than what G-19 addresses. Leave as TODO.

#### teferis_protection.rs
**Oracle text**: "Until your next turn, your life total can't change and you gain protection from everything."
**Current state**: Multiple TODOs -- life total lock + player protection + phase out
**Fix**: The damage prevention aspect is only part of this card. Leave as TODO -- requires multiple engine features not in scope (player protection, "until your next turn" duration, phase-out-all).

#### bonecrusher_giant.rs
**Oracle text**: Stomp: "Damage can't be prevented this turn."
**Current state**: TODO(3) -- prevention removal
**Fix**: "Damage can't be prevented" is the OPPOSITE of damage prevention (CR 615.12). Leave as TODO -- requires `state.damage_cant_be_prevented_this_turn` flag and wiring in `apply_damage_prevention`. Not strictly in G-19 scope.

**Net G-19 fixable cards**: 3 definitively fixable (Spike Weaver, Maze of Ith, Kor Haven), 1 partially fixable (Iroas -- complex static prevention). The remaining 4 have filters or mechanics beyond the basic prevention primitive.

### G-20: Control Change Effects (11 cards)

#### zealous_conscripts.rs
**Oracle text**: "When this creature enters, gain control of target permanent until end of turn. Untap that permanent. It gains haste until end of turn."
**Current state**: TODO -- ETB control change
**Fix**: Add ETB triggered ability:
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenEntersBattlefield,
    effect: Effect::Sequence(vec![
        Effect::GainControl { target: EffectTarget::DeclaredTarget { index: 0 }, duration: EffectDuration::UntilEndOfTurn },
        Effect::UntapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } },
        Effect::ApplyContinuousEffect { effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::Ability,
            modification: LayerModification::AddKeyword(KeywordAbility::Haste),
            filter: EffectFilter::DeclaredTarget { index: 0 },
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }) },
    ]),
    intervening_if: None,
    targets: vec![TargetRequirement::TargetPermanent],
},
```

#### connive.rs (Connive half)
**Oracle text**: "Gain control of target creature with power 2 or less."
**Current state**: `Effect::Nothing` placeholder
**Fix**: Replace with `Effect::GainControl { target: DeclaredTarget(0), duration: EffectDuration::Indefinite }`

#### dragonlord_silumgar.rs
**Oracle text**: "When Dragonlord Silumgar enters, gain control of target creature or planeswalker for as long as you control Dragonlord Silumgar."
**Current state**: TODO -- ETB control effect omitted
**Fix**: Add ETB triggered ability with `Effect::GainControl { target: DeclaredTarget(0), duration: EffectDuration::WhileSourceOnBattlefield }`. Note: `WhileSourceOnBattlefield` is functionally equivalent to "as long as you control this" since the source leaving the battlefield removes the effect.

#### sarkhan_vol.rs
**Oracle text**: "-2: Gain control of target creature until end of turn. Untap that creature. It gains haste until end of turn."
**Current state**: TODO -- loyalty ability effect empty
**Fix**: Replace `Effect::Nothing` with threaten sequence (same pattern as Zealous Conscripts but as loyalty ability effect)

#### alexios_deimos_of_kosmos.rs
**Oracle text**: Upkeep trigger with GainControl + untap + counters + haste
**Current state**: TODO -- complex upkeep trigger
**Fix**: The gain control part is now expressible. Add the upkeep triggered ability with `Effect::GainControl`. Note: this triggers on EACH player's upkeep (not just controller's), which needs `TriggerCondition::AtBeginningOfUpkeep` without controller restriction. Check if this exists.

#### karrthus_tyrant_of_jund.rs
**Oracle text**: "When Karrthus enters, gain control of all Dragons, then untap all Dragons. Dragonlord creatures have haste."
**Current state**: TODO -- ETB gain control of all Dragons
**Fix**: Add ETB triggered ability. "Gain control of all Dragons" = `Effect::GainControl { target: EffectTarget::AllPermanentsMatching(TargetFilter { has_subtype: Some(SubType("Dragon")), ..Default::default() }), duration: EffectDuration::Indefinite }`. Check if `EffectTarget::AllPermanentsMatching` is supported by `resolve_effect_targets`.

#### kellogg_dangerous_mind.rs
**Oracle text**: "Sacrifice five Treasures: Gain control of target creature for as long as you control Kellogg."
**Current state**: TODO -- sacrifice cost + gain control
**Fix**: The sacrifice-Treasures cost is complex (requires sacrificing specific token types). The GainControl part is now expressible. Leave sacrifice cost as TODO if `Cost::SacrificeNWithFilter` doesn't exist.

#### captivating_vampire.rs
**Oracle text**: "Tap five untapped Vampires you control: Gain control of target creature."
**Current state**: TODO -- tap-vampires cost + gain control + type change
**Fix**: The GainControl part is now expressible but the cost (`Cost::TapNCreaturesWithSubtype`) likely doesn't exist. Leave cost as TODO, add GainControl in a comment showing intended structure.

#### olivia_voldaren.rs
**Oracle text**: "{3}{B}{B}: Gain control of target Vampire for as long as you control Olivia Voldaren."
**Current state**: TODO -- activated gain control
**Fix**: Add activated ability with `Effect::GainControl { target: DeclaredTarget(0), duration: EffectDuration::WhileSourceOnBattlefield }` targeting Vampires.

#### hellkite_tyrant.rs
**Oracle text**: "Whenever this deals combat damage to a player, gain control of all artifacts that player controls."
**Current state**: TODO -- combat damage trigger with gain control
**Fix**: The trigger exists (`WheneverThisCreatureDealsCombatDamageToPlayer`). The gain-all-artifacts part needs `EffectTarget::AllPermanentsMatching` with a controller filter pointing to the damaged player. This may need `EffectTarget::AllPermanentsControlledByDamagedPlayer` or similar. Check feasibility.

#### thieving_skydiver.rs
**Oracle text**: "When Thieving Skydiver enters, if it was kicked, gain control of target artifact with mana value X or less."
**Current state**: TODO -- kicked ETB + gain control
**Fix**: The ETB trigger with kicker condition + GainControl. The kicker intervening-if exists. Add triggered ability with `Effect::GainControl { target: DeclaredTarget(0), duration: EffectDuration::Indefinite }`.

**Net G-20 fixable cards**: ~8 definitively fixable (Zealous Conscripts, Connive, Dragonlord Silumgar, Sarkhan Vol, Olivia Voldaren, Thieving Skydiver, and partially Karrthus, Alexios). 3 have cost gaps unrelated to G-20 (Captivating Vampire, Kellogg, Hellkite Tyrant).

### G-21: Land Animation (8 cards with TODOs)

#### den_of_the_bugbear.rs
**Oracle text**: "{3}{R}: Until end of turn, this land becomes a 3/2 red Goblin creature with 'Whenever this creature attacks, create a 1/1 red Goblin creature token that's tapped and attacking.' It's still a land."
**Current state**: TODO -- animation ability omitted
**Fix**: Add activated ability using `animate_land_effects()` helper or manual `Effect::Sequence` of `ApplyContinuousEffect` calls (Layer 4 AddCardTypes, Layer 4 AddSubtypes, Layer 7b SetPowerToughness, Layer 5 SetColors, Layer 6 AddKeyword). Note: the triggered ability grant ("Whenever this creature attacks, create a token") requires granting a triggered ability via Layer 6, which may need a new `LayerModification::AddTriggeredAbility` or can be approximated.

#### creeping_tar_pit.rs
**Oracle text**: "{1}{U}{B}: Until end of turn, this land becomes a 3/2 blue and black Elemental creature. It's still a land. It can't be blocked this turn."
**Current state**: TODO -- animation ability omitted
**Fix**: Add activated ability using manual animation sequence + "can't be blocked this turn" via `ContinuousRestriction` or Layer 6 evasion grant.

#### destiny_spinner.rs
**Oracle text**: "{3}{G}: Target land you control becomes an X/X Elemental creature with trample and haste until end of turn, where X is the number of enchantments you control. It's still a land."
**Current state**: TODO -- animation with dynamic X + "can't be countered" static
**Fix**: Animation with dynamic P/T requires `LayerModification::SetPtDynamic` using `EffectAmount::PermanentCount` filtered to enchantments. Add activated ability targeting a land.

#### tatyova_steward_of_tides.rs
**Oracle text**: "Land creatures you control have flying. Whenever a land you control enters, if you control seven or more lands, up to one target land you control becomes a 3/3 Elemental creature with haste. It's still a land."
**Current state**: TODO -- flying grant to land creatures + landfall animation
**Fix**: Static flying grant needs a new `EffectFilter` for "land creatures you control" (objects that are both Land AND Creature and controlled by source's controller). The landfall animation can use the standard sequence. Check if `EffectFilter::LandCreaturesYouControl` exists or needs creation.

#### druid_class.rs
**Oracle text**: Level 3: "target land you control becomes a creature with haste and 'This creature's power and toughness are each equal to the number of lands you control.' It's still a land."
**Current state**: TODO -- level 3 land animation with CDA P/T
**Fix**: Complex -- the animated land gains a CDA ("power and toughness are each equal to the number of lands you control"). This requires `LayerModification::SetPtDynamic` with `EffectAmount::PermanentCount { filter: lands, controller: You }`. Check feasibility.

#### wrenn_and_realmbreaker.rs
**Oracle text**: "+1: Up to one target land you control becomes a 3/3 Elemental creature with vigilance, hexproof, and haste until your next turn. It's still a land."
**Current state**: TODO -- planeswalker loyalty ability animation
**Fix**: Standard animation sequence with keywords. Note: duration "until your next turn" requires `EffectDuration::UntilYourNextTurn` which does not exist. Use `UntilEndOfTurn` as approximation or add new duration.

#### imprisoned_in_the_moon.rs
**Oracle text**: "Enchanted permanent is a colorless land with '{T}: Add {C}' and loses all other card types and abilities."
**Current state**: TODO -- multi-layer type/ability overwrite
**Fix**: This is NOT land animation per se, but type replacement. Requires Layer 4 SetTypeLine (to Land only), Layer 5 BecomeColorless, Layer 6 RemoveAllAbilities. The existing continuous effect system can express this via multiple static effects on the Aura. This is fixable now.

#### oko_thief_of_crowns.rs (partial -- +1 ability)
**Oracle text**: "+1: Target artifact or creature loses all abilities and becomes a green Elk creature with base power and toughness 3/3."
**Current state**: TODO -- planeswalker type/ability overwrite
**Fix**: Standard continuous effect sequence: Layer 4 SetTypeLine (Creature, Elk), Layer 5 SetColors(Green), Layer 6 RemoveAllAbilities, Layer 7b SetPowerToughness(3,3). The -5 exchange control is now also expressible with `ExchangeControl`.

---

## New Card Definitions

None -- all cards already have def files. This batch is fixing existing TODOs.

---

## Unit Tests

**File**: `crates/engine/tests/primitive_pb32.rs`
**Tests to write**:

### G-18 Tests
- `test_additional_land_play_spell` -- Explore grants one extra land play this turn (CR 305.2)
- `test_additional_land_play_static` -- Aesi on battlefield grants +1 land play per turn (CR 305.2)
- `test_additional_land_play_stacks` -- Two static sources = 3 total land plays (CR 305.2a)
- `test_additional_land_play_removed` -- When source leaves battlefield, next turn reverts to 1 (CR 305.2)
- `test_additional_land_play_spell_one_shot` -- Explore only adds for current turn, not next turn

### G-19 Tests
- `test_prevent_all_combat_damage_fog` -- PreventAllCombatDamage prevents all combat damage this turn (CR 615.1)
- `test_prevent_all_combat_damage_resets_next_turn` -- Prevention flag resets next turn
- `test_prevent_combat_damage_from_target` -- Kor Haven: damage BY target is prevented, other combat damage still dealt
- `test_prevent_combat_damage_to_target` -- Maze of Ith: damage TO and BY target prevented
- `test_noncombat_damage_not_prevented` -- PreventAllCombatDamage does NOT prevent spell damage (CR 615)

### G-20 Tests
- `test_gain_control_until_eot` -- Threaten: control changes back at cleanup step (CR 613.1b)
- `test_gain_control_indefinite` -- Connive: permanent control change (CR 613.1b)
- `test_gain_control_while_source` -- Dragonlord Silumgar: control returns when Silumgar leaves (CR 613.1b)
- `test_exchange_control` -- Two permanents swap controllers (CR 701.12b)
- `test_exchange_control_same_controller` -- Same controller = nothing happens (CR 701.12b)
- `test_gain_control_multiplayer` -- Verify control change works in 4-player game

### G-21 Tests
- `test_land_animation_basic` -- Inkmoth Nexus activation: land becomes creature, still land (Layer 4)
- `test_land_animation_wears_off` -- Animation ends at cleanup step (EffectDuration::UntilEndOfTurn)

**Pattern**: Follow tests for combat damage in `crates/engine/tests/combat_tests.rs` and continuous effects in `crates/engine/tests/layer_tests.rs`

---

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (or explicitly noted as beyond scope)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining fixable TODOs in affected card defs
- [ ] Hash discriminants are unique and sequential
- [ ] `reset_turn_state` resets all new per-turn fields
- [ ] New GameState fields added to hash.rs
- [ ] New GameState fields added to builder.rs defaults

---

## Risks & Edge Cases

- **Layer 2 controller change timing**: The CR says Layer 2 effects are applied in the layer system, but our implementation sets `obj.controller` immediately in the Effect dispatch. This is correct for gameplay (the controller change takes effect immediately) but means the layer system's `SetController` handling is effectively a no-op. The layer system still needs the continuous effect registered so it can be removed when the duration expires. Verify the expiration path in `remove_expired_continuous_effects`.

- **WhileSourceOnBattlefield for "as long as you control"**: This duration checks if the source is on the battlefield, not if the source's controller is still the same player. For Dragonlord Silumgar, if Silumgar's control is stolen, the "as long as you control" should end the control-stealing effect. The current `WhileSourceOnBattlefield` does NOT cover this edge case. A proper implementation would need `EffectDuration::WhileControlledByOriginalController` or similar. For now, `WhileSourceOnBattlefield` is a reasonable approximation.

- **Multiple additional land play sources**: The `additional_land_play_sources` vector must be scanned at turn start, filtering for sources that (a) are still on the battlefield and (b) are controlled by the active player. If a permanent that grants additional lands changes controllers, the new controller should benefit on their turn.

- **PreventAllCombatDamage vs. specific damage events**: The blanket flag approach prevents ALL combat damage but does not emit individual `DamagePrevented` events per assignment. Cards that care about "for each 1 damage prevented" (Inkshield) need event-level tracking. The flag approach is correct for Fog/Spike Weaver but not for Inkshield.

- **TargetRequirement::TargetAttackingCreature**: Maze of Ith and Kor Haven target "attacking creature" specifically. Check if this target requirement exists. If not, it needs to be added as a new `TargetRequirement` variant, or the runner can use `TargetCreature` as an approximation.

- **Land animation with granted triggered abilities**: Den of the Bugbear wants to grant "Whenever this creature attacks, create a token tapped and attacking." Granting triggered abilities through the layer system is not currently supported -- `LayerModification` has no `AddTriggeredAbility` variant. Leave this sub-ability as TODO for now; the basic animation (type/P/T/color/keywords) is fixable.

- **EffectDuration::UntilYourNextTurn**: Wrenn and Realmbreaker's +1 lasts "until your next turn" which is longer than `UntilEndOfTurn`. This duration doesn't exist. Use `UntilEndOfTurn` as approximation. Adding a new duration variant is optional but recommended for accuracy.

- **Cleanup of additional_land_play_sources**: When a permanent leaves the battlefield, its entry in `additional_land_play_sources` must be removed. This should happen alongside the existing continuous_effects cleanup in `remove_expired_continuous_effects` or in an SBA sweep for stale sources.
