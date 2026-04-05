# Primitive Batch Plan: PB-F --- Damage Multiplier

**Generated**: 2026-04-05
**Primitive**: Generalized damage multiplier replacement effects (TripleDamage, conditional source filters, dynamic replacement registration, replacement effect expiry)
**CR Rules**: 614.1, 614.1a, 701.10g, 615.1, 616.1
**Cards affected**: 3 (2 existing fixes + 1 new)
**Dependencies**: PB-37 (UntilYourNextTurn duration, expire sweep) --- DONE
**Deferred items from prior PBs**: `expire_until_next_turn_effects` does not expire replacement effects (only continuous effects + player protections). This gap is addressed in this batch.

## Primitive Specification

The engine currently supports `ReplacementModification::DoubleDamage` with source-side
filtering (`FromControllerSources`, `ToOpponentOrTheirPermanent`). Three new capabilities
are needed:

1. **TripleDamage variant**: Fiery Emancipation says "triple that damage" --- the engine has
   no multiplier other than 2x. Add `ReplacementModification::TripleDamage`.

2. **Target-player-or-their-permanents filter**: Lightning's Stagger creates a replacement
   scoped to "that player or a permanent that player controls". The existing
   `DamageTargetFilter::ToOpponentOrTheirPermanent` checks for opponent relationship;
   we need `ToPlayerOrTheirPermanents(PlayerId)` which checks a specific player.

3. **Source-entered-this-turn filter**: Neriv conditions on "a creature you control that
   entered this turn". The existing `DamageTargetFilter` is target-side only. We need a
   source-side filter that checks both controller and entered-this-turn status. Add
   `DamageTargetFilter::FromControllerCreaturesEnteredThisTurn(PlayerId)` and an
   `entered_turn: Option<u32>` field on `GameObject`.

4. **Dynamic replacement registration effect**: Lightning's Stagger is a triggered ability
   whose effect creates a replacement effect at runtime. Add
   `Effect::RegisterReplacementEffect { trigger, modification, duration }`.

5. **Replacement effect expiry**: `expire_until_next_turn_effects` must also filter
   `state.replacement_effects` with `UntilYourNextTurn` duration.

## CR Rule Text

**CR 614.1**: Some continuous effects are replacement effects. Like prevention effects
(see rule 615), replacement effects apply continuously as events happen --- they aren't
locked in ahead of time. Such effects watch for a particular event that would happen and
completely or partially replace that event with a different event.

**CR 614.1a**: Effects that use the word "instead" are replacement effects. Most
replacement effects use the word "instead" to indicate what events will be replaced
with other events.

**CR 701.10g**: To double an amount of damage a source would deal, that source instead
deals twice that much damage. This is a replacement effect.

**CR 614.15**: Some replacement effects are not continuous effects. Rather, they are an
effect of a resolving spell or ability that replace part or all of that spell or ability's
own effect(s). Such effects are called self-replacement effects.

**CR 616.1**: When multiple replacements apply to the same event, the affected player or
controller chooses order.

## Engine Changes

### Change 1: Add `TripleDamage` variant to `ReplacementModification`

**File**: `crates/engine/src/state/replacement_effect.rs`
**Action**: Add `TripleDamage` variant after `DoubleDamage` (line ~131)
**Pattern**: Follow `DoubleDamage` at line 131

```
/// CR 614.1: Triple damage from a damage event.
/// "If a source you control would deal damage, it deals triple that damage instead."
/// Used by Fiery Emancipation.
TripleDamage,
```

### Change 2: Add `ToPlayerOrTheirPermanents` and `FromControllerCreaturesEnteredThisTurn` to `DamageTargetFilter`

**File**: `crates/engine/src/state/replacement_effect.rs`
**Action**: Add two new variants to `DamageTargetFilter` (after line ~192)

```
/// Only damage dealt to a specific player or permanents that player controls.
/// Used by Lightning Stagger: "damage to that player or a permanent that player controls".
/// Unlike `ToOpponentOrTheirPermanent`, this targets a SPECIFIC player by ID
/// (not "opponents of controller").
ToPlayerOrTheirPermanents(PlayerId),
/// Only damage from creatures controlled by the specified player that entered
/// the battlefield this turn. Used by Neriv, Heart of the Storm.
/// Checks: source is a creature, controlled by PlayerId, and entered_turn == current turn.
FromControllerCreaturesEnteredThisTurn(PlayerId),
```

### Change 3: Add `entered_turn` field to `GameObject`

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `entered_turn: Option<u32>` field near `has_summoning_sickness` (line ~686). Default `None`. Set to `Some(state.turn.turn_number)` at every ETB site.

```rust
/// The turn number on which this permanent entered the battlefield.
/// Used by Neriv to check "entered this turn" (compare with state.turn.turn_number).
/// Set at ETB in resolution.rs and lands.rs. None for objects not on the battlefield.
#[serde(default)]
pub entered_turn: Option<u32>,
```

### Change 4: Set `entered_turn` at all ETB sites

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: In the permanent ETB path (near line ~6553 where `has_summoning_sickness = true`), also set `obj.entered_turn = Some(state.turn.turn_number);`

**File**: `crates/engine/src/rules/lands.rs`
**Action**: In `handle_play_land`, set `entered_turn` on the land object when it enters the battlefield. Find where `has_summoning_sickness = true` is set and add `entered_turn` assignment next to it.

### Change 5: Add `Effect::RegisterReplacementEffect`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new Effect variant (near the end of the Effect enum, after other registration-style effects)

```rust
/// Dynamically register a replacement effect at runtime.
/// Used by triggered abilities that create replacement effects (e.g., Lightning's Stagger).
/// The replacement is pushed to `state.replacement_effects` with the specified
/// trigger, modification, and duration.
RegisterReplacementEffect {
    trigger: ReplacementTrigger,
    modification: ReplacementModification,
    duration: crate::state::continuous_effect::EffectDuration,
},
```

### Change 6: Handle `Effect::RegisterReplacementEffect` in effect execution

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm for the new Effect variant. Generate a new `ReplacementId`, resolve any placeholder `PlayerId(0)` filters to the actual controller, and push to `state.replacement_effects`.

```rust
Effect::RegisterReplacementEffect { trigger, modification, duration } => {
    let id = state.next_replacement_id();
    // Resolve placeholder PlayerId(0) to actual controller
    let resolved_trigger = resolve_damage_trigger_placeholders(&trigger, ctx.controller);
    state.replacement_effects.push_back(ReplacementEffect {
        id,
        source: Some(ctx.source),
        controller: ctx.controller,
        duration: duration.clone(),
        is_self_replacement: false,
        trigger: resolved_trigger,
        modification: modification.clone(),
    });
    events.push(GameEvent::ReplacementEffectApplied {
        effect_id: id,
        description: format!("registered damage replacement: {:?}", modification),
    });
}
```

The `resolve_damage_trigger_placeholders` helper should handle binding `PlayerId(0)` in
`FromControllerSources`, `ToOpponentOrTheirPermanent`, `FromControllerCreaturesEnteredThisTurn`,
and `ToPlayerOrTheirPermanents` to the actual controller. Also, for Lightning specifically,
the `DamageTargetFilter::ToPlayerOrTheirPermanents` uses `PlayerTarget::DamagedPlayer` to
identify the target player at trigger resolution time. The trigger effect needs to resolve
this to the actual player ID from `ctx.combat_data`.

**Important**: Lightning's Stagger trigger must resolve `PlayerTarget::DamagedPlayer` to
the actual `PlayerId` at trigger resolution time (from `ctx.combat_data.damaged_player`).
The `DamageTargetFilter::ToPlayerOrTheirPermanents` stores the resolved `PlayerId`, not a
placeholder. This means the Effect must use a `PlayerTarget` for the filter, not a raw
`PlayerId`. Alternative approach: use `DamageTargetFilter::ToPlayerOrTheirPermanents(PlayerId(0))`
as placeholder, and in the RegisterReplacementEffect handler, resolve it using
`resolve_player_target(state, &PlayerTarget::DamagedPlayer, &ctx)`. Check what
`PlayerTarget::DamagedPlayer` resolves to and use that.

Let me re-check the PlayerTarget enum for DamagedPlayer:

Actually, a simpler approach: the card def trigger effect for Lightning should use
`PlayerTarget::DamagedPlayer` to get the player ID, and pass it into the
`DamageTargetFilter::ToPlayerOrTheirPermanents`. The `RegisterReplacementEffect` handler
should resolve `PlayerTarget::DamagedPlayer` from `ctx.combat_data` and substitute into the filter.

The cleanest design: have the `RegisterReplacementEffect` effect contain a
`damage_target_player: Option<PlayerTarget>` field that, when `Some`, overrides a
`PlayerId(0)` in the `ToPlayerOrTheirPermanents` filter. The effect handler resolves
`PlayerTarget::DamagedPlayer` at execution time. Alternatively, keep it simple and add a
special-case: if the `DamageTargetFilter` contains `PlayerId(0)`, resolve it from
`ctx.combat_data.damaged_player`.

**Recommended design**: In the `RegisterReplacementEffect` handler, if
`DamageTargetFilter::ToPlayerOrTheirPermanents(PlayerId(0))` is found in the trigger,
look up `ctx.combat_data` and substitute the damaged player's ID. If
`FromControllerSources(PlayerId(0))` or `FromControllerCreaturesEnteredThisTurn(PlayerId(0))`
is found, substitute `ctx.controller`.

### Change 7: Extend `apply_damage_doubling` to handle `TripleDamage` and new filters

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: Modify `apply_damage_doubling` (line ~2498) to:
1. Also match `ReplacementModification::TripleDamage` (multiply by 3)
2. Handle `DamageTargetFilter::ToPlayerOrTheirPermanents(pid)` in the matching logic
3. Handle `DamageTargetFilter::FromControllerCreaturesEnteredThisTurn(pid)` --- check source
   controller AND `entered_turn == Some(state.turn.turn_number)` AND source is a creature

Rename function to `apply_damage_multipliers` (optional but clearer).

```rust
// Inside the for loop over replacement_effects:
match &effect.modification {
    ReplacementModification::DoubleDamage => { multiplier = 2; }
    ReplacementModification::TripleDamage => { multiplier = 3; }
    _ => continue,
}
// ... existing filter matching, extended with new variants
DamageTargetFilter::ToPlayerOrTheirPermanents(pid) => {
    match damage_target {
        Some(CombatDamageTarget::Player(p)) => *p == *pid,
        Some(CombatDamageTarget::Creature(id)) | Some(CombatDamageTarget::Planeswalker(id)) => {
            state.objects.get(id).map(|o| o.controller == *pid).unwrap_or(false)
        }
        None => true,
    }
}
DamageTargetFilter::FromControllerCreaturesEnteredThisTurn(pid) => {
    source_controller == Some(*pid)
        && state.objects.get(&source).map(|o| {
            o.entered_turn == Some(state.turn.turn_number)
                && calculate_characteristics(state, source)
                    .map(|c| c.card_types.contains(&CardType::Creature))
                    .unwrap_or(false)
        }).unwrap_or(false)
}
```

Apply the multiplier: `modified *= multiplier;`

### Change 8: Handle `TripleDamage` in `apply_damage_prevention` path

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: In the `match modification` block inside `apply_damage_prevention` (line ~1935),
add arm for `TripleDamage`:

```rust
Some(ReplacementModification::TripleDamage) => {
    remaining *= 3;
    events.push(GameEvent::ReplacementEffectApplied {
        effect_id: id,
        description: format!("tripled damage to {}", remaining),
    });
}
```

### Change 9: Expire replacement effects in `expire_until_next_turn_effects`

**File**: `crates/engine/src/rules/layers.rs`
**Action**: In `expire_until_next_turn_effects` (line ~1287), add filtering of
`state.replacement_effects` with `UntilYourNextTurn` duration, matching the pattern
for `continuous_effects`:

```rust
// Expire UntilYourNextTurn replacement effects for this player.
let keep_repl: im::Vector<ReplacementEffect> = state
    .replacement_effects
    .iter()
    .filter(|e| e.duration != EffectDuration::UntilYourNextTurn(active_player))
    .cloned()
    .collect();
state.replacement_effects = keep_repl;
```

Add required imports: `ReplacementEffect` from `crate::state::replacement_effect`.

### Change 10: Exhaustive match updates

Files requiring new match arms for the new variants:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `state/hash.rs` | `HashInto for ReplacementModification` | L1651 | Add `TripleDamage => 18u8.hash_into(hasher)` |
| `state/hash.rs` | `HashInto for DamageTargetFilter` | L1549 | Add `ToPlayerOrTheirPermanents(pid) => { 6u8; pid }` and `FromControllerCreaturesEnteredThisTurn(pid) => { 7u8; pid }` |
| `state/hash.rs` | `HashInto for GameObject` | (find site) | Add `entered_turn` to hash |
| `rules/replacement.rs` | `apply_damage_prevention` match modification | L1935 | Add `TripleDamage` arm (Change 8) |
| `rules/replacement.rs` | `register_permanent_replacement_abilities` trigger binding | L1639 | Add binding arms for `FromControllerCreaturesEnteredThisTurn(PlayerId(0))` |
| `effects/mod.rs` | Effect dispatch | (find site) | Add `RegisterReplacementEffect` arm (Change 6) |
| `cards/helpers.rs` | re-exports | L25 | Already exports `DamageTargetFilter`, `ReplacementModification`, `ReplacementTrigger` -- no change needed unless new types added |

## Card Definition Fixes

### lightning_army_of_one.rs
**Oracle text**: First strike, trample, lifelink; Stagger --- Whenever Lightning deals combat damage to a player, until your next turn, if a source would deal damage to that player or a permanent that player controls, it deals double that damage instead.
**Current state**: TODO --- Stagger triggered ability omitted. Keywords (first strike, trample, lifelink) are present.
**Fix**: Add triggered ability with `TriggerEvent::SelfDealsCombatDamageToPlayer`, effect is `RegisterReplacementEffect` with:
- `trigger: ReplacementTrigger::DamageWouldBeDealt { target_filter: DamageTargetFilter::ToPlayerOrTheirPermanents(PlayerId(0)) }` --- PlayerId(0) is resolved at trigger resolution time to the damaged player via `ctx.combat_data.damaged_player`
- `modification: ReplacementModification::DoubleDamage`
- `duration: EffectDuration::UntilYourNextTurn(PlayerId(0))` --- PlayerId(0) resolved to controller

Note: The `RegisterReplacementEffect` handler must resolve TWO placeholders differently:
- `ToPlayerOrTheirPermanents(PlayerId(0))` -> resolved to `ctx.combat_data.damaged_player`
- `UntilYourNextTurn(PlayerId(0))` -> resolved to `ctx.controller`

This means the handler needs context about which placeholders map to which values. Consider using `PlayerId(0)` for controller and `PlayerId(1)` for damaged player, or adding an explicit field like `target_player: PlayerTarget` on the Effect variant.

**Recommended approach**: Add `target_player: Option<PlayerTarget>` to `RegisterReplacementEffect`. When `Some`, the handler resolves it and substitutes into any `PlayerId(0)` in the `DamageTargetFilter`. The `duration` placeholder is always resolved to `ctx.controller`.

### neriv_heart_of_the_storm.rs
**Oracle text**: Flying; If a creature you control that entered this turn would deal damage, it deals twice that much damage instead.
**Current state**: TODO --- replacement effect omitted, Flying keyword present.
**Fix**: Add `AbilityDefinition::Replacement` with:
- `trigger: ReplacementTrigger::DamageWouldBeDealt { target_filter: DamageTargetFilter::FromControllerCreaturesEnteredThisTurn(PlayerId(0)) }`
- `modification: ReplacementModification::DoubleDamage`
- `is_self: false`
- `unless_condition: None`

This is a static replacement (registered on ETB via `register_permanent_replacement_abilities`), not a triggered ability. PlayerId(0) is bound to controller at registration time.

## New Card Definitions

### fiery_emancipation.rs
**Oracle text**: If a source you control would deal damage to a permanent or player, it deals triple that damage to that permanent or player instead.
**CardDefinition sketch**:
```rust
CardDefinition {
    card_id: cid("fiery-emancipation"),
    name: "Fiery Emancipation".to_string(),
    mana_cost: Some(ManaCost { generic: 3, red: 3, ..Default::default() }),
    types: basic_types(&[CardType::Enchantment]),
    oracle_text: "If a source you control would deal damage to a permanent or player, it deals triple that damage to that permanent or player instead.".to_string(),
    abilities: vec![
        AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::DamageWouldBeDealt {
                target_filter: DamageTargetFilter::FromControllerSources(PlayerId(0)),
            },
            modification: ReplacementModification::TripleDamage,
            is_self: false,
            unless_condition: None,
        },
    ],
    ..Default::default()
}
```

## Unit Tests

**File**: `crates/engine/tests/damage_multiplier.rs` (new file)
**Tests to write**:
- `test_triple_damage_basic` --- Fiery Emancipation triples damage from a source the controller controls. CR 614.1a, CR 701.10g.
- `test_triple_damage_opponent_source_not_tripled` --- Fiery Emancipation does not triple damage from opponents' sources.
- `test_double_and_triple_stack` --- Angrath's Marauders + Fiery Emancipation: double then triple (or triple then double). Both apply: damage * 2 * 3 = 6x. CR 616.1.
- `test_neriv_creatures_entered_this_turn_doubled` --- Neriv: a creature that entered this turn deals double damage. CR 614.1a.
- `test_neriv_creature_from_prior_turn_not_doubled` --- Neriv: a creature that entered on a previous turn is not doubled.
- `test_neriv_noncreature_source_not_doubled` --- Neriv: a noncreature source (e.g., planeswalker) is not doubled even if it entered this turn.
- `test_lightning_stagger_doubles_damage_to_target_player` --- Lightning combat damage triggers Stagger, then subsequent damage to that player is doubled.
- `test_lightning_stagger_expires_at_next_turn` --- Stagger replacement expires at the start of Lightning's controller's next turn.
- `test_lightning_stagger_doubles_damage_to_target_player_permanents` --- Stagger doubles damage dealt to permanents the target player controls.
**Pattern**: Follow tests in `crates/engine/tests/replacement_effects.rs` and `crates/engine/tests/combat_damage.rs`.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (lightning_army_of_one.rs, neriv_heart_of_the_storm.rs)
- [ ] New card def authored (fiery_emancipation.rs)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs

## Risks & Edge Cases

- **Multiple damage multipliers stacking**: CR 616.1 says the affected player chooses order when multiple replacement effects apply. Doubling + tripling = 6x regardless of order (2*3 = 3*2), but three doublings = 8x (2^3). The current implementation applies all matching effects sequentially in iteration order (registration order). This is deterministic but doesn't give the player a choice. The current approach is acceptable for pre-M10 (no interactive choices), and since multiplication is commutative, order doesn't matter for multipliers.

- **Damage prevention vs. multiplier ordering**: Per Fiery Emancipation rulings (2023-09-01), "If multiple replacement or prevention effects try to modify damage, the player or controller chooses the order." The engine applies doubling/tripling first (in `apply_damage_doubling`), then prevention (in `apply_damage_prevention`). This is one valid ordering, but the player should be able to choose. Pre-M10, the deterministic order (multiply first) is conservative (maximizes damage dealt, which matters for the attacker). This is an acceptable simplification.

- **`entered_turn` field on `GameObject`**: Must be set at ALL ETB sites (resolution.rs + lands.rs). Missing one site means Neriv won't work for lands or for spell permanents. Verify both sites during implementation.

- **`expire_until_next_turn_effects` gap**: Currently does not filter `replacement_effects`. Without this fix, Lightning's Stagger replacement would persist forever. This is a bug that affects any future `UntilYourNextTurn`-duration replacement effect, not just Lightning.

- **Trample + triple damage interaction**: Fiery Emancipation ruling (2020-06-23) says unmodified damage is divided first, then tripled per-target. The engine already does this correctly because `apply_damage_doubling` is called per-assignment (per-target) after combat damage is divided.

- **`RegisterReplacementEffect` placeholder resolution**: The Lightning card needs `PlayerId(0)` in the damage filter to resolve to the *damaged player* (from combat data), while `PlayerId(0)` in the duration resolves to the *controller*. Using a `target_player: Option<PlayerTarget>` field on the Effect variant is the cleanest solution. If `target_player` is `Some(PlayerTarget::DamagedPlayer)`, resolve it from `ctx.combat_data` and substitute into the filter.

- **Hash exhaustiveness**: `entered_turn: Option<u32>` must be added to the `HashInto for GameObject` implementation in `state/hash.rs`. Missing this causes nondeterminism in distributed verification.
