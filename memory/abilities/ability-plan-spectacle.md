# Ability Plan: Spectacle

**Generated**: 2026-03-04
**CR**: 702.137
**Priority**: P4
**Similar abilities studied**: Dash (CR 702.109, `AltCostKind::Dash`), Emerge (CR 702.119, `AltCostKind::Emerge`), Bargain (CR 702.166, additional cost with `Condition::WasBargained`)

## CR Rule Text

**702.137. Spectacle**

702.137a: Spectacle is a static ability that functions on the stack. "Spectacle [cost]" means "You may pay [cost] rather than pay this spell's mana cost if an opponent lost life this turn."

702.137b: To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a spectacle cost), add any cost increases, then apply any cost reductions. The resulting total cost is what you pay.

702.137c: A card's spectacle cost is found in its ability.

## Key Edge Cases

- **"An opponent lost life this turn"** includes ALL sources: combat damage, spell/effect damage, paying life as a cost, life loss from effects. Checked at cast time, not resolution time.
- **Spectacle is an alternative cost (CR 118.9)** -- mutually exclusive with other alternative costs (flashback, dash, emerge, etc.). Cannot combine.
- **Commander tax still applies** -- CR 118.9d: additional costs (like commander tax) are added on top of the spectacle cost.
- **Spectacle cost can be greater or less than the printed cost** -- it's not always cheaper.
- **Damage dealt by Infect does NOT cause life loss** -- it gives poison counters instead (CR 702.90b). So infect damage alone does not enable spectacle.
- **Multiple instances are redundant** -- a card can only have one spectacle cost.
- **Multiplayer: "an opponent" means any ONE opponent of the caster** -- if any opponent of the casting player lost life this turn, spectacle is enabled. In 4-player Commander, this is easy to satisfy.
- **"This turn" means the current game turn** -- life lost in previous turns does not count. The counter resets at each turn boundary for all players (scoped to game turn, not player turn).
- **Conditional effects**: Some spectacle cards have "if you cast this spell for its spectacle cost" effects (e.g., Rix Maadi Reveler). This requires tracking the spectacle status through to resolution. The existing `cast_alt_cost: Option<AltCostKind>` on `GameObject` handles this.
- **Life payment as a cost** (e.g., Phyrexian mana, fetchlands) counts as life loss (CR 119.4). However, `Cost::PayLife` is defined in the DSL but not yet implemented in `rules/`. If/when implemented, it must increment `life_lost_this_turn`.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- nothing exists
- [ ] Step 2: Rule enforcement -- nothing exists
- [ ] Step 3: Trigger wiring -- N/A (Spectacle has no triggers)
- [ ] Step 4: Unit tests -- nothing exists
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Add Infrastructure -- `life_lost_this_turn` Counter

**Rationale**: Spectacle's precondition is "an opponent lost life this turn." The engine currently has no per-player turn-scoped life loss tracking. This is the prerequisite infrastructure.

#### Step 1a: Add `life_lost_this_turn` field to `PlayerState`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/player.rs`
**Action**: Add a new field `pub life_lost_this_turn: u32` to `PlayerState` after `has_citys_blessing` (line ~123).
**Pattern**: Follow `cards_drawn_this_turn: u32` at line 110 and `spells_cast_this_turn: u32` at line 118.
**CR**: 702.137a -- "if an opponent lost life this turn" requires per-player, per-turn tracking.

```rust
/// Amount of life this player has lost this turn (CR 702.137a, CR 118.4).
///
/// Incremented whenever this player's life total decreases due to damage
/// or life loss effects. Reset to 0 at the start of each turn in
/// `reset_turn_state`. Used by Spectacle to check if an opponent
/// lost life this turn.
#[serde(default)]
pub life_lost_this_turn: u32,
```

#### Step 1b: Hash the new field

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: In the `HashInto for PlayerState` implementation, add `self.life_lost_this_turn.hash_into(hasher);` after `self.has_citys_blessing.hash_into(hasher);`.
**Pattern**: Follow the existing per-turn counters (`cards_drawn_this_turn`, `spells_cast_this_turn`).

#### Step 1c: Reset in `reset_turn_state`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/turn_actions.rs`
**Action**: In `reset_turn_state` (line 699), add resetting `life_lost_this_turn` for ALL players (same scope as `cards_drawn_this_turn` reset at lines 719-724).
**CR**: 702.137a -- "this turn" scoped to the current game turn, not player turn.

Add inside the `for pid in player_ids` loop (after line 723):

```rust
// CR 702.137a: per-turn life-loss counter resets at the start of each
// turn for all players (like cards_drawn_this_turn for miracle).
p.life_lost_this_turn = 0;
```

#### Step 1d: Increment at all life-loss sites

There are exactly 4 sites where `life_total` is decremented. Each must increment `life_lost_this_turn` by the actual loss amount.

**Site 1 -- Effect damage to player (non-infect)**
**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` (around line 192)
**Action**: After `player.life_total -= final_dmg as i32;`, add `player.life_lost_this_turn += final_dmg;`

**Site 2 -- LoseLife effect**
**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` (around line 334)
**Action**: After `ps.life_total -= loss as i32;`, add `ps.life_lost_this_turn += loss;`

**Site 3 -- DrainLife effect (opponent drain)**
**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` (around line 361)
**Action**: After `ps.life_total -= loss as i32;`, add `ps.life_lost_this_turn += loss;`

**Site 4 -- Combat damage to player (non-infect)**
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs` (around line 1437)
**Action**: After `player.life_total -= final_dmg as i32;`, add `player.life_lost_this_turn += final_dmg;`

**Note**: Infect damage (combat.rs:1427, effects/mod.rs:185) does NOT cause life loss -- it gives poison counters instead (CR 702.90b). Do NOT increment `life_lost_this_turn` for infect damage.

### Step 2: Enum Variant -- `KeywordAbility::Spectacle`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Spectacle` variant to `KeywordAbility` enum after `Emerge` (line 912).
**Pattern**: Follow `KeywordAbility::Emerge` at line 912.

```rust
/// CR 702.137a: Spectacle [cost] -- alternative cost if an opponent lost life this turn.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The spectacle cost is stored in `AbilityDefinition::Spectacle { cost }`.
Spectacle,
```

**Hash**: In `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`, add to `HashInto for KeywordAbility` (after line 538):

```rust
// Spectacle (discriminant 102) -- CR 702.137
KeywordAbility::Spectacle => 102u8.hash_into(hasher),
```

### Step 3: `AltCostKind::Spectacle` Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Spectacle` variant to `AltCostKind` enum after `Emerge` (line 114).
**Pattern**: Follow `AltCostKind::Emerge` at line 114.

```rust
/// CR 702.137a: Spectacle alternative cost -- pay spectacle cost instead of mana cost.
Spectacle,
```

**Note**: `AltCostKind` is hashed via `cast_alt_cost.map(|k| k as u8)` which uses the enum's discriminant order. Adding `Spectacle` at the end of the enum is safe.

### Step 4: `AbilityDefinition::Spectacle` Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Spectacle { cost: ManaCost }` variant to `AbilityDefinition` enum after `Emerge { cost }` (line 399).
**Pattern**: Follow `AbilityDefinition::Emerge { cost: ManaCost }` at line 399.

```rust
/// CR 702.137: Spectacle [cost]. The card may be cast by paying this cost
/// instead of its mana cost if an opponent lost life this turn.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Spectacle)` for quick
/// presence-checking without scanning all abilities.
Spectacle { cost: ManaCost },
```

**Hash**: In `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`, add to `HashInto for AbilityDefinition` (after line 3240):

```rust
// Spectacle (discriminant 34) -- CR 702.137
AbilityDefinition::Spectacle { cost } => {
    34u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

### Step 5: Rule Enforcement in `casting.rs`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`

#### Step 5a: Add `cast_with_spectacle` flag

Near line 86, after `let cast_with_emerge = ...`:

```rust
let cast_with_spectacle = alt_cost == Some(AltCostKind::Spectacle);
```

#### Step 5b: Validate mutual exclusion (CR 118.9a)

Add a new validation block (Step 1p or similar) after the emerge block (~line 1195):

```rust
// Step 1p: Validate spectacle mutual exclusion (CR 702.137a / CR 118.9a).
// Spectacle is an alternative cost -- cannot combine with other alternative costs.
let casting_with_spectacle = if cast_with_spectacle {
    if casting_with_flashback {
        return Err(GameStateError::InvalidCommand(
            "cannot combine spectacle with flashback (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_evoke { /* same pattern */ }
    if casting_with_bestow { /* same pattern */ }
    if casting_with_overload { /* same pattern */ }
    if casting_with_retrace { /* same pattern */ }
    if casting_with_jump_start { /* same pattern */ }
    if casting_with_aftermath { /* same pattern */ }
    if casting_with_dash { /* same pattern */ }
    if casting_with_blitz { /* same pattern */ }
    if casting_with_plot { /* same pattern */ }
    if cast_with_impending { /* same pattern */ }
    if casting_with_emerge { /* same pattern */ }
    // Validate the card has the Spectacle keyword
    if !chars.keywords.contains(&KeywordAbility::Spectacle) {
        return Err(GameStateError::InvalidCommand(
            "spell does not have spectacle (CR 702.137a)".into(),
        ));
    }
    if get_spectacle_cost(&card_id, &state.card_registry).is_none() {
        return Err(GameStateError::InvalidCommand(
            "card has Spectacle keyword but no spectacle cost defined (CR 702.137a)".into(),
        ));
    }
    // CR 702.137a: Validate that an opponent of the caster lost life this turn.
    let any_opponent_lost_life = state.players.iter().any(|(pid, ps)| {
        *pid != player && ps.life_lost_this_turn > 0
    });
    if !any_opponent_lost_life {
        return Err(GameStateError::InvalidCommand(
            "spectacle: no opponent has lost life this turn (CR 702.137a)".into(),
        ));
    }
    true
} else {
    false
};
```

**Note**: Also add `if casting_with_spectacle { return Err(...) }` arms to ALL existing alt cost blocks (dash, blitz, plot, impending, emerge, etc.) for the reverse direction mutual exclusion.

#### Step 5c: Base cost substitution

In the base cost selection chain (after `casting_with_emerge` around line 1383):

```rust
} else if casting_with_spectacle {
    // CR 702.137a: Pay spectacle cost instead of mana cost.
    // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
    get_spectacle_cost(&card_id, &state.card_registry)
}
```

#### Step 5d: Add `get_spectacle_cost` helper

At the end of `casting.rs`, add:

```rust
/// CR 702.137a: Look up the spectacle cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Spectacle { cost }`, or `None`
/// if the card has no definition or no spectacle ability defined.
fn get_spectacle_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Spectacle { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

**Pattern**: Identical to `get_dash_cost` at line 2508, `get_emerge_cost` at line 3486.

#### Step 5e: Record spectacle status on StackObject

No new boolean needed on `StackObject` -- the existing `cast_alt_cost` pattern handles this. In the StackObject construction (around line 2014):

The `alt_cost` field on the CastSpell command already carries `Some(AltCostKind::Spectacle)`. The `cast_alt_cost` on `GameObject` is set from the alt_cost during resolution. No additional boolean is needed unless conditional Spectacle effects are implemented (see Step 5f).

#### Step 5f: (Optional) Add `Condition::WasSpectacled`

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `WasSpectacled` variant to `Condition` enum (after `WasBargained`).
**Pattern**: Follow `Condition::WasBargained` at line 963.

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Add evaluation in `evaluate_condition` (after `WasBargained` around line 2777).

This is needed for cards like Rix Maadi Reveler that have "if this spell was cast for its spectacle cost" conditional effects. Implementation:
- Check `ctx.cast_alt_cost == Some(AltCostKind::Spectacle)` or a dedicated `was_spectacled` boolean on `EffectContext`.
- Since the existing pattern for Bargain uses a separate `was_bargained` boolean (because Bargain is an additional cost that can combine with alt costs), Spectacle as an alt cost could use `cast_alt_cost` directly.
- However, for consistency with the `WasBargained` pattern and to keep `EffectContext` fields explicit, add a `was_spectacled: bool` to `EffectContext`.

**Decision**: Defer this to the card definition step. The core ability (paying alt cost when opponent lost life) works without the condition variant. Add `Condition::WasSpectacled` + `EffectContext.was_spectacled` only when authoring a card that needs it.

### Step 6: Replay Harness `cast_spell_spectacle` Action Type

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"cast_spell_spectacle"` arm to `translate_player_action` (around line 1007).
**Pattern**: Follow `"cast_spell_dash"` at line 910.

```rust
// CR 702.137a: Cast a spell with spectacle from the player's hand.
// The spectacle cost (an alternative cost) is paid instead of the mana cost.
"cast_spell_spectacle" => {
    let card_id = find_in_hand(state, player, card_name?)?;
    let target_list = resolve_targets(targets, state, players);
    Some(Command::CastSpell {
        player,
        card: card_id,
        targets: target_list,
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: Some(AltCostKind::Spectacle),
        escape_exile_cards: vec![],
        retrace_discard_land: None,
        jump_start_discard: None,
        prototype: false,
        bargain_sacrifice: None,
        emerge_sacrifice: None,
    })
}
```

### Step 7: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/spectacle.rs`
**Pattern**: Follow `/home/airbaggie/scutemob/crates/engine/tests/dash.rs` (lines 119-690) and `/home/airbaggie/scutemob/crates/engine/tests/emerge.rs` (lines 170-820).

**Tests to write**:

1. `test_spectacle_basic_cast_after_opponent_life_loss` -- CR 702.137a
   - Set up: P1 has a Spectacle card in hand. P2 has `life_lost_this_turn = 3`.
   - Action: P1 casts the card with `alt_cost: Some(AltCostKind::Spectacle)`.
   - Assert: Spell resolves successfully, spectacle cost was paid (not printed cost).

2. `test_spectacle_rejected_when_no_opponent_lost_life` -- CR 702.137a
   - Set up: P1 has a Spectacle card in hand. No opponent has lost life.
   - Action: P1 tries to cast with `alt_cost: Some(AltCostKind::Spectacle)`.
   - Assert: Error "no opponent has lost life this turn."

3. `test_spectacle_normal_cast_without_spectacle` -- CR 702.137a (spectacle is optional)
   - Set up: P1 has a Spectacle card in hand.
   - Action: P1 casts the card with `alt_cost: None` (paying normal mana cost).
   - Assert: Spell resolves normally. No spectacle behavior.

4. `test_spectacle_combat_damage_enables_spectacle` -- CR 702.137a + CR 120.3a
   - Set up: P1 dealt combat damage to P2 (P2's life decreased). P2's `life_lost_this_turn > 0`.
   - Action: P1 casts a Spectacle spell for its spectacle cost.
   - Assert: Cast succeeds.

5. `test_spectacle_effect_damage_enables_spectacle` -- CR 702.137a + CR 118.4
   - Set up: P1 dealt effect damage (Lightning Bolt) to P2. P2's `life_lost_this_turn > 0`.
   - Action: P1 casts a Spectacle spell for its spectacle cost.
   - Assert: Cast succeeds.

6. `test_spectacle_mutual_exclusion_with_flashback` -- CR 118.9a
   - Set up: Card has both Spectacle and Flashback. Card is in graveyard.
   - Action: Try to cast with both alt costs.
   - Assert: Error about combining alternative costs.

7. `test_spectacle_no_keyword_rejects` -- validation
   - Set up: Card without Spectacle keyword.
   - Action: Try `alt_cost: Some(AltCostKind::Spectacle)`.
   - Assert: Error "spell does not have spectacle."

8. `test_spectacle_commander_tax_applies` -- CR 118.9d
   - Set up: Commander with Spectacle, cast once before (tax = {2}).
   - Action: Cast with spectacle from command zone.
   - Assert: Total cost includes commander tax on top of spectacle cost.

9. `test_spectacle_life_lost_counter_resets_on_turn_boundary` -- per-turn scoping
   - Set up: P2 lost life last turn. New turn starts.
   - Assert: P2's `life_lost_this_turn` is 0 after `reset_turn_state`.

10. `test_spectacle_infect_damage_does_not_enable` -- CR 702.90b
    - Set up: P1 dealt infect damage to P2. P2 got poison counters but `life_lost_this_turn == 0`.
    - Action: P1 tries Spectacle cast.
    - Assert: Error "no opponent has lost life."

11. `test_spectacle_multiplayer_any_opponent` -- multiplayer
    - Set up: 4 players. P3 lost life (not P2 or P4).
    - Action: P1 casts Spectacle spell.
    - Assert: Cast succeeds (P3 is an opponent who lost life).

**Helper card definition for tests**: Create a simple test card with Spectacle. Follow the test helper pattern from `dash.rs`:

```rust
fn spectacle_test_card() -> CardDefinition {
    CardDefinition {
        name: "Spectacle Test Creature".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: creature_types(&["Devil"]),
        oracle_text: "Spectacle {1}{R}".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Spectacle),
            AbilityDefinition::Spectacle {
                cost: ManaCost { generic: 1, red: 1, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
```

### Step 8: Card Definition (later phase)

**Suggested cards**:
- **Light Up the Stage** ({2}{R} sorcery, Spectacle {R}) -- simple, prominent Spectacle card. Effect: exile top two cards, play them until end of next turn.
- **Skewer the Critics** ({2}{R} sorcery, Spectacle {R}) -- simpler effect: deals 3 damage to any target.
- **Rix Maadi Reveler** ({1}{R} creature 2/2, Spectacle {B}{R}) -- has conditional "if cast for spectacle cost" effect.

**Card lookup**: Use `card-definition-author` agent.

### Step 9: Game Script (later phase)

**Suggested scenario**: P1 deals damage to P2 with a creature, then casts a Spectacle spell for its spectacle cost in second main phase. Verify the cost paid matches the spectacle cost, not the printed cost.
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Spectacle + Commander Tax**: Commander tax is added ON TOP of the spectacle cost (CR 118.9d). The cost pipeline in `casting.rs` already handles this -- commander tax is applied after the base cost is selected.
- **Spectacle + Convoke/Improvise/Delve**: These cost-reduction mechanics apply AFTER the total cost (including spectacle as base) is determined. No special interaction -- they work normally.
- **Spectacle + Prototype**: Prototype is NOT an alternative cost (ruling 2022-10-14). Spectacle IS an alternative cost. These should be mutually exclusive via the `alt_cost` mechanism. However, no existing Spectacle card also has Prototype, so this is theoretical.
- **`life_lost_this_turn` and Surge**: Surge (CR 702.117, same batch) needs "you or a teammate cast a spell this turn" -- a different tracking mechanism (`spells_cast_this_turn` per player). The `life_lost_this_turn` infrastructure does NOT overlap with Surge.
- **Multiplayer**: In Commander, checking "any opponent lost life" is easy -- iterate `state.players` and check if any non-caster player has `life_lost_this_turn > 0`. In Two-Headed Giant, teammates are not opponents, but the engine is Commander-first (no THG support needed).
- **Future PayLife cost implementation**: If `Cost::PayLife` is implemented in `rules/`, it must increment `life_lost_this_turn` on the paying player. This is noted as a future concern.

## File Summary

| File | Action |
|------|--------|
| `crates/engine/src/state/player.rs` | Add `life_lost_this_turn: u32` field |
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Spectacle`, `AltCostKind::Spectacle` |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Spectacle { cost }` |
| `crates/engine/src/state/hash.rs` | Hash `life_lost_this_turn`, `KeywordAbility::Spectacle` (disc 102), `AbilityDefinition::Spectacle` (disc 34) |
| `crates/engine/src/rules/turn_actions.rs` | Reset `life_lost_this_turn` in `reset_turn_state` |
| `crates/engine/src/effects/mod.rs` | Increment `life_lost_this_turn` at 3 sites |
| `crates/engine/src/rules/combat.rs` | Increment `life_lost_this_turn` at 1 site |
| `crates/engine/src/rules/casting.rs` | Add spectacle validation, mutual exclusion, cost substitution, `get_spectacle_cost` helper |
| `crates/engine/src/testing/replay_harness.rs` | Add `cast_spell_spectacle` action type |
| `crates/engine/tests/spectacle.rs` | 11 unit tests |

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| `KeywordAbility` | `Spectacle` | 102 |
| `AbilityDefinition` | `Spectacle` | 34 |
| `AltCostKind` | `Spectacle` | (auto, appended after Emerge) |
