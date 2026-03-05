# Ability Plan: Surge

**Generated**: 2026-03-05
**CR**: 702.117
**Priority**: P4
**Similar abilities studied**: Spectacle (CR 702.137) -- same pattern: alternative cost with a turn-based precondition. Files: `crates/engine/src/rules/casting.rs`, `crates/engine/src/state/types.rs`, `crates/engine/src/state/hash.rs`, `crates/engine/src/cards/card_definition.rs`, `crates/engine/tests/spectacle.rs`

## CR Rule Text

702.117. Surge

702.117a Surge is a static ability that functions while the spell with surge is on the stack. "Surge [cost]" means "You may pay [cost] rather than pay this spell's mana cost as you cast this spell if you or one of your teammates has cast another spell this turn." Casting a spell for its surge cost follows the rules for paying alternative costs in rules 601.2b and 601.2f-h.

## Key Edge Cases

- **"Another spell this turn"**: The surge spell itself does not count. The precondition is that a *different* spell was cast this turn by you (or a teammate). Since `spells_cast_this_turn` is incremented AFTER the spell enters the stack (line ~2282 of casting.rs), the check must verify `spells_cast_this_turn >= 1` at the START of casting (before the counter is incremented for the surge spell itself). This means at least one other spell was already cast.
- **Teammates in Commander**: Commander is a free-for-all format with no teams. There are no teammates (CR 810 defines teammates only for team variants like Two-Headed Giant). In Commander, Surge's precondition simplifies to "you have cast another spell this turn." The engine should check only the caster's `spells_cast_this_turn`.
- **"Another spell" includes resolved, countered, or still on the stack** (ruling 2016-01-22 on Crush of Tentacles). Only requires a successful cast, not resolution.
- **Surge cost does not change mana cost or mana value** (ruling 2016-01-22). CR 118.9c: alternative costs only change what you pay, not the spell's characteristics.
- **Copies of surge spells are considered to have had surge cost paid** (ruling 2016-01-22). The copy inherits `cast_alt_cost: Some(AltCostKind::Surge)` from the original via copy infrastructure.
- **Some surge cards have "if surge cost was paid" effects** (e.g., Crush of Tentacles creates token, Reckless Bushwhacker grants haste/+1+0). These use `cast_alt_cost == Some(AltCostKind::Surge)` on the resolved permanent/spell, same pattern as Dash/Blitz/Spectacle.
- **Mutual exclusion with all other alternative costs** (CR 118.9a): Surge cannot combine with Flashback, Evoke, Bestow, Madness, Miracle, Escape, Foretell, Overload, Retrace, Jump-Start, Aftermath, Dash, Blitz, Plot, Impending, Emerge, Spectacle.
- **Commander tax stacks on top of surge cost** (CR 118.9d): Additional costs apply to alternative costs.
- **Multiplayer turn scope**: `spells_cast_this_turn` resets at each player's turn start. On another player's turn, the counter reflects spells cast during THAT game turn by the active player. But the surge caster needs to check THEIR OWN counter, not the active player's. This works because the caster is the one whose `spells_cast_this_turn` was incremented when they cast their earlier spell (even on another player's turn, e.g., casting an instant).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (n/a -- Surge is a static ability, no triggers)
- [ ] Step 4: Unit tests

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`

**Action 1a**: Add `Surge` to `AltCostKind` enum (after `Spectacle`):
```rust
/// CR 702.117a: Surge alternative cost -- pay surge cost instead of mana cost
/// if you or a teammate has cast another spell this turn.
Surge,
```

**Action 1b**: Add `Surge` to `KeywordAbility` enum (after `Spectacle`, before the closing brace):
```rust
/// CR 702.117a: Surge [cost] -- alternative cost if you or a teammate cast another spell this turn.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The surge cost is stored in `AbilityDefinition::Surge { cost }`.
Surge,
```

**File**: `crates/engine/src/cards/card_definition.rs`

**Action 1c**: Add `Surge { cost: ManaCost }` variant to `AbilityDefinition` enum (after `Spectacle { cost: ManaCost }`):
```rust
/// CR 702.117: Surge [cost]. The card may be cast by paying this cost
/// instead of its mana cost if you or a teammate has cast another spell this turn.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Surge)` for quick
/// presence-checking without scanning all abilities.
Surge { cost: ManaCost },
```

**File**: `crates/engine/src/state/hash.rs`

**Action 1d**: Add hash arm for `KeywordAbility::Surge` -- discriminant **103**:
```rust
// Surge (discriminant 103) -- CR 702.117
KeywordAbility::Surge => 103u8.hash_into(hasher),
```
Pattern: follows `KeywordAbility::Spectacle => 102u8` at line ~540.

**Action 1e**: Add hash arm for `AbilityDefinition::Surge { cost }` -- discriminant **35**:
```rust
// Surge (discriminant 35) -- CR 702.117
AbilityDefinition::Surge { cost } => {
    35u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```
Pattern: follows `AbilityDefinition::Spectacle` at line ~3246.

**File**: `tools/tui/src/play/panels/stack_view.rs`

**Action 1f**: If `AltCostKind` is exhaustively matched in stack_view.rs, add `AltCostKind::Surge` arm. Check whether this file matches on `AltCostKind` -- if not, skip.

### Step 2: Rule Enforcement

**File**: `crates/engine/src/rules/casting.rs`

**Action 2a**: Add `let cast_with_surge = alt_cost == Some(AltCostKind::Surge);` near line ~87 (alongside the other `cast_with_*` flags).

**Action 2b**: Add mutual exclusion checks. In each existing alt-cost validation block (Dash, Blitz, Plot, Impending, Emerge, Spectacle), add a check rejecting combination with Surge. Pattern -- copy the Spectacle mutual-exclusion pattern exactly, replacing "spectacle" with "surge":
- In the Dash block: `if cast_with_surge { return Err(...) }`
- In the Blitz block: `if cast_with_surge { return Err(...) }`
- In the Plot block: `if cast_with_surge { return Err(...) }`
- In the Impending block: `if cast_with_surge { return Err(...) }`
- In the Emerge block: `if cast_with_surge { return Err(...) }`
- In the Spectacle block: `if cast_with_surge { return Err(...) }`

**Action 2c**: Add the Surge validation block (after the Spectacle block, around line ~1405). Follow the Spectacle pattern exactly:

```rust
// Step 1p: Validate surge mutual exclusion and precondition (CR 702.117a / CR 118.9a).
// Surge is an alternative cost -- cannot combine with other alternative costs.
let casting_with_surge = if cast_with_surge {
    // Mutual exclusion with all other alt costs (same pattern as Spectacle).
    if casting_with_flashback {
        return Err(GameStateError::InvalidCommand(
            "cannot combine surge with flashback (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    // ... (all other alt costs: evoke, bestow, madness, miracle, escape, foretell,
    //      overload, retrace, jump_start, aftermath, dash, blitz, plot, impending,
    //      emerge, spectacle)

    // Validate the card has the Surge keyword.
    if !chars.keywords.contains(&KeywordAbility::Surge) {
        return Err(GameStateError::InvalidCommand(
            "spell does not have surge (CR 702.117a)".into(),
        ));
    }
    // Validate the card has a surge cost defined.
    if get_surge_cost(&card_id, &state.card_registry).is_none() {
        return Err(GameStateError::InvalidCommand(
            "card has Surge keyword but no surge cost defined (CR 702.117a)".into(),
        ));
    }
    // CR 702.117a: Validate that the caster has cast another spell this turn.
    // spells_cast_this_turn is incremented AFTER the spell enters the stack,
    // so at this point it reflects spells cast BEFORE this one.
    // >= 1 means at least one other spell was already cast.
    let caster_cast_count = state
        .players
        .get(&player)
        .map(|ps| ps.spells_cast_this_turn)
        .unwrap_or(0);
    if caster_cast_count < 1 {
        return Err(GameStateError::InvalidCommand(
            "surge: you have not cast another spell this turn (CR 702.117a)".into(),
        ));
    }
    true
} else {
    false
};
```

**Action 2d**: In the mana cost determination section (around line ~1534), add an `else if` branch for surge (before the plot branch):

```rust
} else if casting_with_surge {
    // CR 702.117a: Pay surge cost instead of mana cost.
    // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
    get_surge_cost(&card_id, &state.card_registry)
}
```

**Action 2e**: Add `get_surge_cost()` helper function at the bottom of casting.rs (follow `get_spectacle_cost` pattern exactly):

```rust
/// CR 702.117a: Look up the surge cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Surge { cost }`, or `None`
/// if the card has no definition or no surge ability defined.
fn get_surge_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Surge { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

**Action 2f**: Ensure `cast_alt_cost` is set on the stack object/permanent for surge. This already happens generically via `alt_cost` -> `cast_alt_cost` propagation in the existing CastSpell flow (same mechanism as Spectacle/Dash/Blitz). Verify that the `AltCostKind::Surge` value flows through to `game_object.cast_alt_cost`. No new code needed if the existing flow handles all `AltCostKind` variants generically.

### Step 3: Trigger Wiring

**N/A** -- Surge is a static ability that functions on the stack. It has no triggers. Cards with "if surge cost was paid" effects use conditional ETB triggers that check `cast_alt_cost`, which is already supported by the existing infrastructure.

### Step 4: Unit Tests

**File**: `crates/engine/tests/surge.rs`

**Tests to write** (follow `spectacle.rs` pattern exactly):

1. **`test_surge_basic_cast_with_surge_cost`** -- CR 702.117a: Cast a spell for its surge cost after casting another spell this turn. Verify correct mana payment (surge cost, not printed cost).

2. **`test_surge_rejected_no_prior_spell`** -- CR 702.117a: Attempt to surge without having cast another spell this turn. Verify error.

3. **`test_surge_optional_normal_cost`** -- CR 702.117a: Cast a surge card for its normal mana cost (without alt_cost). Verify it works even when surge is available.

4. **`test_surge_after_resolved_spell`** -- Ruling 2016-01-22: The prior spell can have already resolved. Cast a spell, resolve it (pass_all), then surge. Verify success.

5. **`test_surge_after_countered_spell`** -- Ruling 2016-01-22: The prior spell can have been countered. Verify `spells_cast_this_turn` still counts it.

6. **`test_surge_mutual_exclusion_with_flashback`** -- CR 118.9a: Cannot combine surge with flashback. Verify error.

7. **`test_surge_mutual_exclusion_with_spectacle`** -- CR 118.9a: Cannot combine surge with spectacle. Verify error.

8. **`test_surge_card_without_keyword_rejected`** -- Engine validation: A card without `KeywordAbility::Surge` rejects `alt_cost: Some(AltCostKind::Surge)`.

9. **`test_surge_reset_at_turn_start`** -- CR 702.117a + turn structure: After turn boundary, `spells_cast_this_turn` resets. Surge is rejected because no spells cast in new turn.

10. **`test_surge_commander_tax_stacks`** -- CR 118.9d: Commander tax adds on top of surge cost. Cast commander with surge, verify total cost = surge cost + tax.

11. **`test_surge_cast_alt_cost_tracked`** -- Verify `cast_alt_cost == Some(AltCostKind::Surge)` on the resolved permanent (for "if surge cost was paid" effects).

**Pattern**: Follow `spectacle.rs` test structure -- synthetic card definitions with `CardId("surge-creature")`, `find_object` helper, `GameStateBuilder::four_player()`, `process_command` calls.

**Synthetic card definitions needed**:
- `surge_creature_def()`: Creature, cost {3}{R}, Surge {1}{R}, P/T 3/2
- `plain_spell_def()`: Reuse from spectacle tests or define a simple instant
- `cheap_cantrip_def()`: A 1-mana spell to cast first (enabling surge)

### Step 5: Card Definition (later phase)

**Suggested card**: Crush of Tentacles
- Sorcery, {4}{U}{U}, Surge {3}{U}{U}
- Return all nonland permanents to their owners' hands
- If surge cost was paid, create an 8/8 blue Octopus creature token
- Uses `cast_alt_cost` conditional, complex effect (bounce all + conditional token)

**Alternative simpler card**: Reckless Bushwhacker
- Creature {2}{R}, Surge {1}{R}, 2/1, Haste
- ETB: if surge cost was paid, other creatures get +1/+0 and haste until end of turn
- Uses conditional ETB trigger based on `cast_alt_cost`

**Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: Cast a cheap instant, then cast a Surge creature for its surge cost. Verify the creature enters the battlefield and the correct mana was paid.
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **`spells_cast_this_turn` timing**: The counter is incremented AFTER the spell enters the stack (casting.rs line ~2282). The surge precondition check happens BEFORE the increment, so `>= 1` correctly means "at least one OTHER spell was cast before this one."
- **Storm interaction**: Storm also uses `spells_cast_this_turn`. No conflict -- storm reads the count, surge reads the count, neither modifies the other's behavior.
- **Cascade interaction**: Cascade free-casts increment `spells_cast_this_turn` (copy.rs line ~411). A cascade free-cast would enable surge for subsequent spells in the same turn.
- **Suspend free-cast**: Suspend also increments `spells_cast_this_turn` (resolution.rs line ~1771). Casting from suspend enables surge.
- **Copy inheritance**: Spell copies inherit `cast_alt_cost` from the original. If a surge spell is copied (e.g., by Twincast), the copy has `cast_alt_cost: Some(AltCostKind::Surge)`, matching the ruling that copies are considered to have had surge cost paid.
- **No teammate support needed**: Commander is free-for-all. The engine does not model teams or teammates. If Two-Headed Giant support is added later, the surge precondition would need to also check teammates' `spells_cast_this_turn`.
