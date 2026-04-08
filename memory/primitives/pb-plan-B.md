# Primitive Batch Plan: PB-B -- Play from GY/Exile Zone-Play Permissions

**Generated**: 2026-04-08
**Primitive**: Zone-play permission system for graveyard (and emblem-sourced graveyard permissions)
**CR Rules**: CR 601.2 (casting from zones), CR 601.3 (permission to cast), CR 305.1 (playing lands)
**Cards affected**: 6 existing fixes + 0 new
**Dependencies**: PB-A (PlayFromTopPermission/PlayFromTopFilter pattern -- DONE)
**Deferred items from prior PBs**: Wrenn and Realmbreaker -7 emblem (from card authoring), Ancient Greenwarden + Perennial Behemoth graveyard land play (from card authoring)

## Scope Decision: Pitch Alt-Costs OUT OF SCOPE

Force of Negation, Force of Vigor, and Force of Will use "exile a [color] card from your hand rather than pay this spell's mana cost." This is fundamentally different from zone-play permissions:
- Pitch costs cast FROM HAND (normal zone), not from another zone.
- The exile-a-card part is the COST, not a zone permission.
- They need a new `AltCostKind::PitchExile` + condition checking (not your turn / always).
- This is a separate primitive (new AltCostKind variant + casting.rs validation).

**Recommendation**: Defer pitch-alt-cost to a separate small PB batch. PB-B focuses purely on graveyard-zone play permissions.

## Scope Decision: Brokkos + Squee + Oathsworn Vampire ARE Self-Cast-From-GY

These three cards grant themselves permission to be cast from the graveyard. They are NOT continuous permissions granted by another permanent. They are self-referential abilities on the card itself:
- **Oathsworn Vampire**: "You may cast this card from your graveyard if you gained life this turn."
- **Squee, Dubious Monarch**: "You may cast this card from your graveyard by paying {3}{R} and exiling four other cards from your graveyard."
- **Brokkos, Apex of Forever**: "You may cast this card from your graveyard using its mutate ability."

These are similar to Flashback/Escape/Disturb (self-cast-from-GY with special conditions/costs) but don't fit existing AltCostKind variants. The right approach is:
- A new `AbilityDefinition` variant: `CastSelfFromGraveyard` that signals "this card may be cast from the graveyard" with optional condition and optional alt cost.
- The casting.rs graveyard-casting validation checks for this ability on the card.

The other three cards (Ancient Greenwarden, Perennial Behemoth, Wrenn emblem) grant a continuous permission to play from the graveyard -- these need the permission system parallel to PB-A.

## Primitive Specification

PB-B adds two capabilities:

### Capability 1: PlayFromGraveyardPermission (continuous permission, parallel to PB-A)

A `PlayFromGraveyardPermission` struct on `GameState`, registered by a new `AbilityDefinition::StaticPlayFromGraveyard` variant. When active, the controller may play lands and/or cast spells from their graveyard (filtered by `PlayFromTopFilter` -- reuse the same filter enum). Cleaned up when the source leaves the battlefield (same sweep as PB-A).

This handles: Ancient Greenwarden, Perennial Behemoth, Wrenn and Realmbreaker emblem.

### Capability 2: CastSelfFromGraveyard (self-referential ability)

A new `AbilityDefinition::CastSelfFromGraveyard` variant that says "this card may be cast from the graveyard" with:
- Optional `Condition` (e.g., "if you gained life this turn" for Oathsworn Vampire)
- Optional `ManaCost` override (e.g., {3}{R} for Squee)
- Optional `AdditionalCost` requirements (e.g., exile 4 GY cards for Squee)
- Optional `AltCostKind` restriction (e.g., Mutate only for Brokkos)

casting.rs checks: if card is in graveyard AND has this ability AND condition is met, allow casting.

### Capability 3: Condition::ControllerGainedLifeThisTurn + PlayerState tracker

Oathsworn Vampire's condition "if you gained life this turn" requires:
- A `life_gained_this_turn: u32` field on `PlayerState`
- Incremented wherever `LifeGained` events are emitted
- Reset in `reset_turn_state`
- A new `Condition::ControllerGainedLifeThisTurn` variant

## CR Rule Text

**CR 601.2**: "To cast a spell is to take it from where it is (usually the hand), put it on the stack, and pay its costs..."
- Key: "from where it is" -- spells can be cast from zones other than hand if a rule or effect permits.

**CR 601.3**: "A player can begin to cast a spell only if a rule or effect allows that player to cast it and no rule or effect prohibits that player from casting it."
- Key: effects on cards and permanents can grant permission to cast from non-hand zones.

**CR 305.1**: "A player who has priority may play a land card from their hand during a main phase of their turn when the stack is empty."
- Modified by effects that allow playing lands from other zones (e.g., "You may play lands from your graveyard").

## Engine Changes

### Change 1: PlayFromGraveyardPermission struct

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `PlayFromGraveyardPermission` struct after `PlayFromTopPermission` (around line 665).
**Pattern**: Follow `PlayFromTopPermission` at line 645.

```rust
/// An active play-from-graveyard permission.
///
/// CR 601.3: A player can begin to cast a spell only if a rule or effect allows it.
/// CR 305.1 (modified): Playing a land from graveyard requires explicit permission.
///
/// Registered by `AbilityDefinition::StaticPlayFromGraveyard` when the source permanent
/// enters the battlefield, or by emblem creation. Cleaned up when source leaves battlefield.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayFromGraveyardPermission {
    /// ObjectId of the source permanent or emblem.
    pub source: ObjectId,
    /// The player who receives this permission.
    pub controller: PlayerId,
    /// Which cards this permission applies to (reuse PlayFromTopFilter).
    pub filter: PlayFromTopFilter,
    /// Optional condition that must be true for this permission to be active.
    pub condition: Option<crate::cards::card_definition::Condition>,
}
```

Note: No `look_at_top`/`reveal_top`/`pay_life_instead`/`on_cast_effect` -- graveyard is public info, and none of the current cards have these properties.

### Change 2: play_from_graveyard_permissions field on GameState

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add `play_from_graveyard_permissions: Vector<PlayFromGraveyardPermission>` field after `play_from_top_permissions` (around line 138).

### Change 3: Export PlayFromGraveyardPermission

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add `PlayFromGraveyardPermission` to the `pub use stubs::{}` import (line ~48).

**File**: `crates/engine/src/lib.rs`
**Action**: Add `PlayFromGraveyardPermission` to the re-export list (around line 41).

### Change 4: GameStateBuilder initialization

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add `play_from_graveyard_permissions: im::Vector::new()` in the build method (around line 322).

### Change 5: AbilityDefinition::StaticPlayFromGraveyard variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant after `StaticPlayFromTop` (line ~949). Discriminant 74.

```rust
/// Static play-from-graveyard permission (CR 601.3, CR 305.1).
///
/// Registers a `PlayFromGraveyardPermission` when the permanent enters the battlefield.
/// Cleaned up when the source leaves the battlefield (via reset_turn_state sweep).
/// Follows the same pattern as `StaticPlayFromTop`.
///
/// Discriminant 74.
StaticPlayFromGraveyard {
    filter: crate::state::stubs::PlayFromTopFilter,
    /// Optional condition that gates this permission.
    condition: Option<Box<Condition>>,
},
```

### Change 6: AbilityDefinition::CastSelfFromGraveyard variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant after `StaticPlayFromGraveyard`. Discriminant 75.

```rust
/// Self-referential graveyard cast permission.
///
/// "You may cast this card from your graveyard [condition] [with alt cost]."
/// Unlike StaticPlayFromGraveyard (which grants a continuous permission for
/// ALL matching cards), this grants permission to cast only THIS card.
///
/// casting.rs checks: card is in owner's graveyard, has this ability,
/// condition (if any) is met, and applies the specified cost/restrictions.
///
/// Discriminant 75.
CastSelfFromGraveyard {
    /// Optional condition (e.g., ControllerGainedLifeThisTurn for Oathsworn Vampire).
    condition: Option<Box<Condition>>,
    /// If Some, this is the mana cost to pay (instead of the card's normal mana cost).
    /// If None, pay the card's normal mana cost.
    alt_mana_cost: Option<ManaCost>,
    /// Additional costs required (e.g., exile 4 other GY cards for Squee).
    additional_costs: Vec<CastFromGraveyardAdditionalCost>,
    /// If Some, must cast using this specific alt cost kind (e.g., Mutate for Brokkos).
    required_alt_cost: Option<crate::state::types::AltCostKind>,
},
```

### Change 7: CastFromGraveyardAdditionalCost enum

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new enum near the CastSelfFromGraveyard variant.

```rust
/// Additional costs for CastSelfFromGraveyard ability.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CastFromGraveyardAdditionalCost {
    /// Exile N other cards from your graveyard.
    ExileOtherGraveyardCards(u32),
}
```

### Change 8: Condition::ControllerGainedLifeThisTurn

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to `Condition` enum (after `TopCardIsCreatureOfChosenType`, line ~2721).

```rust
/// "if you gained life this turn" — Oathsworn Vampire.
///
/// True when the controller's `life_gained_this_turn > 0`.
/// Does not care about net life change — only whether ANY life was gained.
/// Ruling 2018-01-19: cares only about gain, not whether you also lost life.
ControllerGainedLifeThisTurn,
```

### Change 9: PlayerState.life_gained_this_turn tracker

**File**: `crates/engine/src/state/player.rs`
**Action**: Add `pub life_gained_this_turn: u32` field after `life_lost_this_turn` (around line 356).

### Change 10: life_gained_this_turn reset in reset_turn_state

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: Add `p.life_gained_this_turn = 0;` in the reset loop (around line 1493, after `life_lost_this_turn`).

### Change 11: life_gained_this_turn increment on LifeGained events

**Files**: `crates/engine/src/effects/mod.rs` (GainLife effect + DrainLife effect + Fight/Bite lifelink), `crates/engine/src/rules/combat.rs` (lifelink combat damage)
**Action**: After every `events.push(GameEvent::LifeGained { player, amount })`, add:
```rust
if let Some(ps) = state.players.get_mut(&player_id) {
    ps.life_gained_this_turn += amount;
}
```
Note: Some LifeGained events already modify `life_total` right before pushing the event. Add the tracker increment at the same location.

### Change 12: Registration in replacement.rs

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: Add match arm for `StaticPlayFromGraveyard` after the `StaticPlayFromTop` arm (around line 1846).

```rust
AbilityDefinition::StaticPlayFromGraveyard { filter, condition } => {
    state.play_from_graveyard_permissions.push_back(
        crate::state::stubs::PlayFromGraveyardPermission {
            source: new_id,
            controller,
            filter: filter.clone(),
            condition: condition.as_ref().map(|c| *c.clone()),
        },
    );
}
```

### Change 13: Cleanup in turn_actions.rs

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: Add retain sweep for `play_from_graveyard_permissions` after the `play_from_top_permissions` sweep (around line 1545).

```rust
state.play_from_graveyard_permissions.retain(|perm| {
    state
        .objects
        .get(&perm.source)
        .map(|o| matches!(o.zone, crate::state::ZoneId::Battlefield) || o.is_emblem)
        .unwrap_or(false)
});
```

Note: Emblem sources live in command zone, not battlefield. The retain must keep permissions whose source is an emblem (command zone) OR a permanent (battlefield).

### Change 14: Casting validation in casting.rs

**File**: `crates/engine/src/rules/casting.rs`
**Action**: In the zone-validation section (around line 240-550), add logic:

1. If card is in player's graveyard AND does NOT have a standard GY-casting keyword (Flashback, Escape, Disturb, Retrace, JumpStart, Aftermath):
   a. Check if the card has `CastSelfFromGraveyard` ability -- if yes, validate condition + apply alt cost.
   b. Check if there's a matching `PlayFromGraveyardPermission` -- if yes, allow normal casting.
   c. If neither, reject (existing behavior).

2. For `CastSelfFromGraveyard`:
   - If `required_alt_cost` is Some, set `alt_cost` on the CastSpell accordingly.
   - If `alt_mana_cost` is Some, use that instead of the card's normal mana cost.
   - If `additional_costs` includes `ExileOtherGraveyardCards(n)`, validate N other cards exist in graveyard.

### Change 15: Land play validation in lands.rs

**File**: `crates/engine/src/rules/lands.rs`
**Action**: Add `has_play_from_graveyard_land_permission()` function parallel to `has_play_from_top_land_permission()` (around line 431). In the land-play validation, if the land is in the player's graveyard AND there's a matching permission, allow play.

The main `handle_play_land` function must also check: if the land object is in the graveyard zone, validate against `play_from_graveyard_permissions` instead of requiring it to be in hand.

### Change 16: Emblem support for graveyard permissions

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Extend `CreateEmblem` to include an optional `play_from_graveyard: Option<PlayFromTopFilter>` field.

**File**: `crates/engine/src/effects/mod.rs`
**Action**: In the `CreateEmblem` handler (around line 4060), if `play_from_graveyard` is Some, register a `PlayFromGraveyardPermission` with the emblem as source.

### Change 17: Condition evaluation

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm in `check_condition` for `ControllerGainedLifeThisTurn`.

```rust
Condition::ControllerGainedLifeThisTurn => {
    state.players.get(&ctx.controller)
        .map(|p| p.life_gained_this_turn > 0)
        .unwrap_or(false)
}
```

### Change 18: Exhaustive match updates

Files requiring new match arms for the new types/variants:

| File | Match expression | Approx Line | Action |
|------|-----------------|-------------|--------|
| `state/hash.rs` | `AbilityDefinition::*` | ~5741 | Hash `StaticPlayFromGraveyard` (disc 74) and `CastSelfFromGraveyard` (disc 75) |
| `state/hash.rs` | `Condition::*` | ~4530 | Hash `ControllerGainedLifeThisTurn` (disc 42) |
| `state/hash.rs` | `GameState` HashInto | ~5900 | Hash `play_from_graveyard_permissions` |
| `state/hash.rs` | `PlayerState` HashInto | ~varies | Hash `life_gained_this_turn` |
| `state/hash.rs` | `Effect::CreateEmblem` | ~varies | Hash new `play_from_graveyard` field |
| `state/hash.rs` | `CastFromGraveyardAdditionalCost` | new | Implement `HashInto` for new enum |
| `rules/replacement.rs` | AbilityDefinition match | ~1825 | Add `StaticPlayFromGraveyard` arm |
| `effects/mod.rs` | `check_condition` | ~6315 | Add `ControllerGainedLifeThisTurn` arm |
| `effects/mod.rs` | `CreateEmblem` handler | ~4060 | Register graveyard permission from emblem |
| `cards/helpers.rs` | exports | top | Export `PlayFromGraveyardPermission`, `CastFromGraveyardAdditionalCost` |

**NOTE**: The `_ => {}` wildcard in `replacement.rs` (line 1847) already catches new AbilityDefinition variants, so the new variants won't cause a compile error there -- but the runner MUST add explicit arms to actually register the permissions.

## Card Definition Fixes

### ancient_greenwarden.rs
**Oracle text**: "Reach\nYou may play lands from your graveyard.\nIf a land entering causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
**Current state**: Only Reach implemented. Two TODOs: graveyard land play + land-ETB trigger doubling.
**Fix**: Add `AbilityDefinition::StaticPlayFromGraveyard { filter: PlayFromTopFilter::LandsOnly, condition: None }`. The trigger doubling is a SEPARATE primitive (PB-M Panharmonicon-style doubling filtered to land ETBs) -- add a TODO noting it's deferred to PB-M or similar.

### perennial_behemoth.rs
**Oracle text**: "You may play lands from your graveyard.\nUnearth {G}{G}"
**Current state**: Only Unearth keyword. TODO for graveyard land play.
**Fix**: Add `AbilityDefinition::StaticPlayFromGraveyard { filter: PlayFromTopFilter::LandsOnly, condition: None }`. The Unearth keyword marker is already present; add the `AltCastAbility` for Unearth cost if missing.

### wrenn_and_realmbreaker.rs
**Oracle text**: "-7: You get an emblem with 'You may play lands and cast permanent spells from your graveyard.'"
**Current state**: -7 creates an empty emblem with a TODO.
**Fix**: Change the -7 `CreateEmblem` to include `play_from_graveyard: Some(PlayFromTopFilter::PermanentsAndLands)`. This requires adding a new `PlayFromTopFilter::PermanentsAndLands` variant (creatures + artifacts + enchantments + planeswalkers + lands). The -2 mill ability and static mana grant are separate TODOs -- leave those.

### oathsworn_vampire.rs
**Oracle text**: "This creature enters tapped.\nYou may cast this card from your graveyard if you gained life this turn."
**Current state**: Enters-tapped replacement implemented. TODO for graveyard cast.
**Fix**: Add `AbilityDefinition::CastSelfFromGraveyard { condition: Some(Box::new(Condition::ControllerGainedLifeThisTurn)), alt_mana_cost: None, additional_costs: vec![], required_alt_cost: None }`.

### squee_dubious_monarch.rs
**Oracle text**: "You may cast this card from your graveyard by paying {3}{R} and exiling four other cards from your graveyard rather than paying its mana cost."
**Current state**: Haste + attack trigger implemented. TODO for graveyard cast.
**Fix**: Add `AbilityDefinition::CastSelfFromGraveyard { condition: None, alt_mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }), additional_costs: vec![CastFromGraveyardAdditionalCost::ExileOtherGraveyardCards(4)], required_alt_cost: None }`.

### brokkos_apex_of_forever.rs
**Oracle text**: "You may cast this card from your graveyard using its mutate ability."
**Current state**: Mutate keyword + cost implemented. TODO for graveyard cast.
**Fix**: Add `AbilityDefinition::CastSelfFromGraveyard { condition: None, alt_mana_cost: None, additional_costs: vec![], required_alt_cost: Some(AltCostKind::Mutate) }`. casting.rs must enforce: when `required_alt_cost` is `Some(AltCostKind::Mutate)`, the cast MUST use Mutate (ruling 2020-04-17).

## New PlayFromTopFilter Variant

### PlayFromTopFilter::PermanentsAndLands

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add variant to `PlayFromTopFilter` enum.

```rust
/// Permanent spells (creature, artifact, enchantment, planeswalker) and lands.
/// Used by Wrenn and Realmbreaker emblem: "play lands and cast permanent spells."
PermanentsAndLands,
```

**Files needing new match arm**: `casting.rs` (two places: `has_play_from_top_permission` and `find_play_from_top_on_cast_effect`), `lands.rs` (`has_play_from_top_land_permission`).

For casting filter: `PermanentsAndLands` matches Creature, Artifact, Enchantment, Planeswalker (but NOT Instant, NOT Sorcery).
For land filter: `PermanentsAndLands` includes lands.

## Unit Tests

**File**: `crates/engine/tests/play_from_graveyard.rs` (new file)
**Tests to write**:
- `test_play_from_graveyard_land_basic` -- Ancient Greenwarden on battlefield allows playing land from GY. CR 601.3, 305.1.
- `test_play_from_graveyard_land_timing` -- Land from GY still follows timing restrictions (main phase, stack empty). CR 305.1.
- `test_play_from_graveyard_land_count` -- Land from GY counts against land-play limit. Ruling 2020-09-25 (Ancient Greenwarden).
- `test_play_from_graveyard_permission_removed` -- When source permanent leaves, permission removed. No more GY land play.
- `test_play_from_graveyard_permanent_spell` -- Wrenn emblem allows casting creature from GY.
- `test_play_from_graveyard_no_instant_sorcery` -- Wrenn emblem does NOT allow instant/sorcery from GY (PermanentsAndLands filter).
- `test_cast_self_from_graveyard_oathsworn_vampire` -- Oathsworn casts from GY when life gained this turn. Ruling 2018-01-19.
- `test_cast_self_from_graveyard_oathsworn_no_life` -- Oathsworn cannot cast from GY when no life gained.
- `test_cast_self_from_graveyard_squee` -- Squee casts from GY paying {3}{R} + exiling 4 GY cards.
- `test_cast_self_from_graveyard_squee_insufficient_exile` -- Squee cannot cast if < 4 other GY cards.
- `test_cast_self_from_graveyard_brokkos` -- Brokkos casts from GY using mutate only.
- `test_life_gained_this_turn_tracking` -- Basic tracking: GainLife increments, reset at turn start.
- `test_play_from_graveyard_emblem_persists` -- Emblem permission survives source PW leaving battlefield.

**Pattern**: Follow tests in `crates/engine/tests/play_from_top.rs`.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (6 cards)
- [ ] New card defs authored (if any) -- none for this batch
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs (except explicitly deferred: Ancient Greenwarden trigger doubling, Wrenn -2 mill, Wrenn static mana grant)

## Risks & Edge Cases

- **Emblem persistence**: Emblems never leave the command zone. The cleanup sweep must NOT remove permissions sourced from emblems. The retain check must allow command-zone emblem sources (`o.is_emblem`).
- **Multiple permissions**: A player with both Ancient Greenwarden and a Wrenn emblem should be able to play lands from GY via either permission. Each permission is independently checked.
- **Squee re-cast after counter**: Ruling 2022-09-09: "If Squee is countered or dies after being cast from your graveyard, it returns to the graveyard. It may be cast this way again later." This works naturally -- the card returns to GY and still has the ability.
- **Brokkos must use Mutate**: Ruling 2020-04-17: "you must pay its mutate cost to cast it." If `required_alt_cost` is `Some(Mutate)`, casting.rs MUST enforce that `alt_cost == Some(Mutate)` -- no normal casting from GY allowed.
- **CastSelfFromGraveyard condition evaluation**: The condition for Oathsworn Vampire must be checked at CAST TIME, not at resolution time. Use a special `EffectContext` built from the casting player's state for condition evaluation in casting.rs.
- **Wrenn emblem filter**: "permanent spells" means creature, artifact, enchantment, and planeswalker -- NOT instants or sorceries. "play lands" is separate from "cast permanent spells." The `PermanentsAndLands` filter must match both.
- **Ancient Greenwarden trigger doubling**: This is NOT part of PB-B. The trigger doubling for land ETBs is a separate primitive (requires intercept in trigger generation). Mark with TODO referencing PB-M or a future batch.
- **Perennial Behemoth Unearth**: The `AltCastAbility` for Unearth may already be present or may need to be added. Verify the card def has both the keyword marker AND the cost entry.
- **CastSelfFromGraveyard with existing GY-cast keywords**: If a card somehow has both CastSelfFromGraveyard AND Flashback, the player chooses which permission to use (CR 601.3). The casting code should not auto-select.
- **legal_actions integration**: The simulator's `legal_actions.rs` does not enumerate graveyard-castable cards. This is a known gap (comment at line 107). PB-B does not need to fix the simulator -- the engine's `process_command` validates legality directly.
