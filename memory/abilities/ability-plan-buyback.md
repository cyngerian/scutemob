# Ability Plan: Buyback

**Generated**: 2026-02-28
**CR**: 702.27
**Priority**: P3
**Similar abilities studied**: Kicker (additional cost pattern in `casting.rs`, `resolution.rs`, `command.rs`, `stack.rs`; tests in `tests/kicker.rs`)

## CR Rule Text

702.27. Buyback

702.27a Buyback appears on some instants and sorceries. It represents two static abilities that function while the spell is on the stack. "Buyback [cost]" means "You may pay an additional [cost] as you cast this spell" and "If the buyback cost was paid, put this spell into its owner's hand instead of into that player's graveyard as it resolves." Paying a spell's buyback cost follows the rules for paying additional costs in rules 601.2b and 601.2f-h.

### Supporting Rules

- **CR 601.2b**: Player announces intentions to pay additional costs (including buyback) during the announcement step.
- **CR 601.2f**: Total cost = mana cost (or alt cost) + all additional costs + cost increases - cost reductions. Buyback is an additional cost.
- **CR 118.8**: Additional costs are paid at the same time as the spell's mana cost. Any number of additional costs may apply (CR 118.8a). Some are optional (CR 118.8b). Additional costs don't change mana cost/mana value (CR 118.8d).
- **CR 701.6a**: "A countered spell is put into its owner's graveyard." Buyback only applies "as it resolves" -- a countered buyback spell goes to graveyard regardless.
- **CR 702.34a**: Flashback says "exile this card instead of putting it anywhere else any time it would leave the stack." If both flashback and buyback were paid (hypothetical; no printed card has both), flashback's exile clause overrides buyback's return-to-hand.

## Key Edge Cases

1. **Buyback not paid**: Spell resolves normally, goes to graveyard. No return to hand.
2. **Buyback paid, spell countered (CR 701.6a)**: Spell goes to graveyard. Buyback only applies "as it resolves" and a countered spell does not resolve.
3. **Buyback paid, spell fizzles (all targets illegal, CR 608.2b)**: Same as countered -- the spell never resolves, so buyback doesn't apply. Goes to graveyard.
4. **Buyback + Flashback interaction (CR 702.34a)**: Flashback says "exile instead of putting it anywhere else" when it leaves the stack. This overrides buyback. Flashback wins. (No printed card has both, but the engine should handle it correctly.)
5. **Buyback on instants vs sorceries**: Buyback works identically on both. CR 702.27a: "appears on some instants and sorceries."
6. **Buyback + Commander tax**: Buyback is an additional cost (CR 118.8), commander tax is also an additional cost. Both stack. A buyback spell cast from command zone pays: base mana + commander tax + buyback cost.
7. **Buyback + Kicker**: Both are additional costs. They stack. The spell pays: base mana + kicker cost + buyback cost. (No printed card has both Kicker and Buyback, but the engine should handle it.)
8. **Buyback + Convoke/Delve/Improvise**: These cost-reduction mechanics apply after the total cost is computed. The buyback cost is added to the total before convoke/delve/improvise reduce it.
9. **Buyback + copies (CR 707.10)**: Storm/Cascade copies are not cast, so they never pay buyback. `was_buyback_paid` is always `false` on copies.
10. **Multiplayer**: No special multiplayer considerations. Buyback returns the card to the owner's hand, regardless of controller.
11. **Insufficient mana**: If the player declares buyback but cannot pay the additional cost, the cast fails with an error (same as kicker).

## Current State (from ability-wip.md)

- [x] Step 1: Enum variant -- `KeywordAbility::Buyback` at `types.rs:L497-503`, hash at `hash.rs:L430-431` (disc 61), `AbilityDefinition::Buyback { cost: ManaCost }` at `card_definition.rs:L238-245`, `was_buyback_paid: bool` at `stack.rs:L112-118`, view_model at `view_model.rs:L652`
- [ ] Step 2: Rule enforcement (casting + resolution)
- [ ] Step 3: Trigger wiring (n/a -- buyback is not a triggered ability)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant (DONE)

All pre-edits are in place:
- `KeywordAbility::Buyback` in `crates/engine/src/state/types.rs:L497-503`
- Hash discriminant 61 in `crates/engine/src/state/hash.rs:L430-431`
- `AbilityDefinition::Buyback { cost: ManaCost }` discriminant 19 in `crates/engine/src/state/hash.rs:L2634-2638`
- `was_buyback_paid: bool` on `StackObject` in `crates/engine/src/state/stack.rs:L112-118`
- View model display in `tools/replay-viewer/src/view_model.rs:L652`

### Step 2: Casting Support (Rule Enforcement -- Additional Cost)

**File**: `crates/engine/src/rules/casting.rs`
**CR**: 702.27a, 601.2b, 601.2f, 118.8

#### Step 2a: Add `cast_with_buyback: bool` parameter to `handle_cast_spell`

**Action**: Add `cast_with_buyback: bool` as a new parameter to the function signature at line 52.
**Pattern**: Follow `cast_with_foretell: bool` at line 66 (the last current parameter).

#### Step 2b: Add `get_buyback_cost` helper function

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add a `get_buyback_cost` function near `get_kicker_cost` (line 1054-1077).
**Pattern**: Follow `get_kicker_cost` exactly, but match on `AbilityDefinition::Buyback { cost }` and return `Option<ManaCost>` (no `is_multikicker` -- buyback is always one-shot).

```rust
/// CR 702.27a: Look up the buyback cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Buyback { cost }`, or `None`
/// if the card has no definition or no buyback ability defined.
fn get_buyback_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Buyback { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

#### Step 2c: Add buyback cost to total cost pipeline

**File**: `crates/engine/src/rules/casting.rs`
**Action**: After the kicker cost addition block (lines 579-594), add a similar block for buyback.
**CR**: 601.2f -- buyback cost is an additional cost added to the total before convoke/delve/improvise reduction.
**Pipeline order**: base_mana_cost -> alt_cost -> commander_tax -> kicker -> **buyback** -> affinity -> undaunted -> convoke -> improvise -> delve -> pay.

```rust
// CR 702.27a / 601.2f: If the player declared intention to pay buyback, validate
// the spell has buyback and add the buyback cost to the total.
// CR 118.8d: Additional costs don't change the spell's mana cost, only what is paid.
let was_buyback_paid = if cast_with_buyback {
    match get_buyback_cost(&card_id, &state.card_registry) {
        Some(buyback_cost) => {
            // Validate: buyback is only for instants/sorceries (CR 702.27a).
            // This is inherent to the card having the ability, but double-check.
            true
        }
        None => {
            return Err(GameStateError::InvalidCommand(
                "spell does not have buyback".into(),
            ));
        }
    }
} else {
    false
};

// CR 601.2f: Add buyback cost to the total mana cost.
let mana_cost = if was_buyback_paid {
    let buyback_cost = get_buyback_cost(&card_id, &state.card_registry).unwrap();
    let mut total = mana_cost.unwrap_or_default();
    total.white += buyback_cost.white;
    total.blue += buyback_cost.blue;
    total.black += buyback_cost.black;
    total.red += buyback_cost.red;
    total.green += buyback_cost.green;
    total.generic += buyback_cost.generic;
    total.colorless += buyback_cost.colorless;
    Some(total)
} else {
    mana_cost
};
```

**Note**: The `get_buyback_cost` is called twice -- once for validation and once to add. The runner may optimize to a single call by binding the result.

#### Step 2d: Set `was_buyback_paid` on the StackObject

**File**: `crates/engine/src/rules/casting.rs`
**Action**: At line 860, change `was_buyback_paid: false` to `was_buyback_paid`.
**Pattern**: The `was_buyback_paid` local variable from step 2c flows into the struct literal.

### Step 3: Command Enum -- Add `cast_with_buyback` field

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `cast_with_buyback: bool` field to the `CastSpell` variant, after `cast_with_foretell` (line 149).
**Pattern**: Follow `cast_with_foretell` (lines 141-149).

```rust
/// CR 702.27a: If true, pay the buyback additional cost when casting.
/// If the buyback cost was paid and the spell resolves, the card returns
/// to its owner's hand instead of going to the graveyard.
/// This is an additional cost (not alternative) -- can combine with
/// flashback, kicker, and other costs.
#[serde(default)]
cast_with_buyback: bool,
```

### Step 4: Engine Dispatch -- Wire `cast_with_buyback` through `engine.rs`

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add `cast_with_buyback` to the `Command::CastSpell` destructuring (around line 85) and pass it to `casting::handle_cast_spell` (around line 104).
**Pattern**: Follow `cast_with_foretell` which is already destructured and passed.

### Step 5: Resolution -- Return to Hand Instead of Graveyard

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Modify the instant/sorcery resolution branch (lines 481-496) to check `was_buyback_paid`.
**CR**: 702.27a -- "If the buyback cost was paid, put this spell into its owner's hand instead of into that player's graveyard as it resolves."

Current code (lines 482-488):
```rust
// CR 608.2n: Instant/sorcery -- card moves to owner's graveyard.
// CR 702.34a: If cast with flashback, exile instead of graveyard.
let destination = if stack_obj.cast_with_flashback {
    ZoneId::Exile // CR 702.34a
} else {
    ZoneId::Graveyard(owner)
};
```

New code:
```rust
// CR 608.2n: Instant/sorcery -- card moves to owner's graveyard.
// CR 702.34a: If cast with flashback, exile instead of graveyard.
// Flashback overrides buyback: "exile instead of putting it anywhere
// else any time it would leave the stack" (CR 702.34a).
// CR 702.27a: If buyback was paid (and not flashbacked), return to hand.
let destination = if stack_obj.cast_with_flashback {
    ZoneId::Exile // CR 702.34a -- overrides all other destinations
} else if stack_obj.was_buyback_paid {
    ZoneId::Hand(owner) // CR 702.27a
} else {
    ZoneId::Graveyard(owner)
};
```

**IMPORTANT**: The fizzle path (lines 77-97) should NOT be modified. When a spell fizzles, it does not resolve. Buyback says "as it resolves", so fizzled buyback spells go to graveyard (or exile if flashbacked). The existing fizzle code at lines 89-93 is already correct for this -- it only checks flashback, not buyback.

**IMPORTANT**: The `CounterSpell` effect path in `effects/mod.rs` (lines 718-724) should NOT be modified for buyback. Countered spells go to graveyard per CR 701.6a. Buyback only applies on resolution.

### Step 6: Replay Harness -- Add `cast_spell_buyback` action type

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add a `"cast_spell_buyback"` arm to `translate_player_action()`.
**Pattern**: Follow `"cast_spell"` (line 221) but with `cast_with_buyback: true`.
**Note**: Unlike evoke/bestow/miracle which are alternative costs, buyback is an additional cost. The `"cast_spell_buyback"` action should still cast from hand (not graveyard/exile). It can also be combined with kicker (both additional costs), but for simplicity the dedicated action always sets `cast_with_buyback: true` and can optionally use the `kicked` field.

Alternatively: Add a `buyback: bool` field to the `ScriptPlayerAction::PlayerAction` struct (like `kicked: bool`) so that `"cast_spell"` with `buyback: true` works. This is cleaner and more consistent with how kicker works.

**Recommended approach**: Add `buyback: bool` field to `ScriptPlayerAction` (in `script_schema.rs`) with `#[serde(default)]`. In the `"cast_spell"` arm of the replay harness, read this field and pass it through. This avoids a separate action type and is consistent with the `kicked` pattern.

#### Step 6a: Script Schema

**File**: `crates/engine/src/testing/script_schema.rs`
**Action**: Add a `buyback: bool` field to `ScriptPlayerAction::PlayerAction`, after the `kicked` field (around line 237).

```rust
/// CR 702.27: For `cast_spell` with buyback. If true, the buyback additional
/// cost is paid. If the spell resolves, it returns to the owner's hand.
/// Defaults to false (buyback not paid).
#[serde(default)]
buyback: bool,
```

#### Step 6b: Replay Harness

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In the `"cast_spell"` arm (around line 221), read the `buyback` field and pass `cast_with_buyback` to the `Command::CastSpell` constructor.
**Pattern**: Follow how `kicked` is read and mapped to `kicker_times`.

The existing `"cast_spell"` arm destructures `kicked` from the action. Add `buyback` similarly:
```rust
"cast_spell" => {
    let card_id = find_in_hand(state, player, card_name?)?;
    let target_list = resolve_targets(targets, state, players);
    // ... existing convoke/improvise/delve resolution ...
    Some(Command::CastSpell {
        player,
        card: card_id,
        targets: target_list,
        convoke_creatures: convoke_ids,
        improvise_artifacts: improvise_ids,
        delve_cards: delve_ids,
        kicker_times: if kicked { 1 } else { 0 },
        cast_with_evoke: false,
        cast_with_bestow: false,
        cast_with_miracle: false,
        cast_with_escape: false,
        escape_exile_cards: vec![],
        cast_with_foretell: false,
        cast_with_buyback: buyback,  // NEW
    })
}
```

**Also**: Every other `"cast_spell_*"` arm (flashback, evoke, bestow, madness, miracle, escape, foretell) must add `cast_with_buyback: false` to their `Command::CastSpell` constructors. Count: 7 arms need this addition.

### Step 7: Trigger Wiring

**N/A**: Buyback is not a triggered ability. It has no triggers to wire. It is purely:
1. An additional cost announced at cast time (CR 601.2b).
2. A replacement destination for the card at resolution time (CR 702.27a).

### Step 8: Unit Tests

**File**: `crates/engine/tests/buyback.rs`
**Tests to write**:

#### Test 1: `test_buyback_basic_return_to_hand`
- **CR**: 702.27a -- "If the buyback cost was paid, put this spell into its owner's hand instead of into that player's graveyard as it resolves."
- Setup: P1 has Searing Touch (or Whispers of the Muse) in hand. P1 has enough mana for base + buyback cost.
- Cast with `cast_with_buyback: true`, targeting P2.
- All players pass priority, spell resolves.
- Assert: Spell card is in P1's hand (not graveyard). P2 took damage (or P1 drew a card).

#### Test 2: `test_buyback_not_paid_goes_to_graveyard`
- **CR**: 702.27a -- When buyback is NOT paid, the spell goes to graveyard normally.
- Setup: Same as Test 1, but `cast_with_buyback: false`.
- Assert: Spell card is in P1's graveyard after resolution.

#### Test 3: `test_buyback_paid_spell_countered_goes_to_graveyard`
- **CR**: 701.6a -- "A countered spell is put into its owner's graveyard."
- Setup: P1 casts Searing Touch with buyback. P2 casts Counterspell targeting it.
- Assert: Searing Touch is in P1's graveyard (not hand), even though buyback was paid. Buyback only applies "as it resolves."

#### Test 4: `test_buyback_paid_spell_fizzles_goes_to_graveyard`
- **CR**: 608.2b + 702.27a -- When all targets become illegal, the spell fizzles and does not resolve. Buyback does not apply.
- Setup: P1 casts Searing Touch targeting a creature. Before resolution, the creature leaves the battlefield (is destroyed/sacrificed). All targets illegal -- spell fizzles.
- Assert: Searing Touch is in P1's graveyard (not hand).

#### Test 5: `test_buyback_cost_added_to_total`
- **CR**: 601.2f, 118.8 -- Buyback adds its cost to the total.
- Setup: P1 has Searing Touch ({R}, Buyback {4}). P1 has exactly {4}{R} mana.
- Cast with buyback.
- Assert: Cast succeeds. Mana pool is empty.

#### Test 6: `test_buyback_insufficient_mana_rejected`
- **CR**: 601.2f -- If the player can't pay the total cost, the cast is illegal.
- Setup: P1 has Searing Touch ({R}, Buyback {4}). P1 has only {R} (not enough for buyback).
- Attempt cast with `cast_with_buyback: true`.
- Assert: Error returned (insufficient mana).

#### Test 7: `test_buyback_no_buyback_ability_rejected`
- Setup: P1 has Lightning Bolt (no buyback ability). Casts with `cast_with_buyback: true`.
- Assert: Error returned ("spell does not have buyback").

#### Test 8: `test_buyback_with_flashback_exile_wins`
- **CR**: 702.34a -- Flashback exile overrides buyback return-to-hand.
- Setup: Create a card with both Buyback and Flashback abilities (synthetic test card). Cast from graveyard with flashback. The cast_with_buyback flag is also set.
- Assert: After resolution, the card is in exile (not hand). Flashback's "exile instead of putting it anywhere else" wins.
- **Note**: This is a defensive test for a hypothetical interaction. No printed card has both keywords.

**Pattern**: Follow `crates/engine/tests/kicker.rs` for test structure (helpers, imports, builder, `pass_all`).

**Card Definition needed**: Searing Touch (simple buyback instant with targeting). The `card-definition-author` agent will create this.

### Step 9: Card Definition (Later Phase)

**Suggested card**: Searing Touch
- Name: Searing Touch
- Mana cost: {R}
- Type: Instant
- Buyback {4}
- Effect: Deal 1 damage to any target
- Abilities: `[Keyword(Buyback), Buyback { cost: ManaCost { generic: 4, ..default } }, Spell { effect: DealDamage { target: DeclaredTarget { index: 0 }, amount: Fixed(1) } }]`

**Alternative card**: Whispers of the Muse ({U} Instant, Buyback {5}, Draw a card) -- simpler, no targeting needed.

### Step 10: Game Script (Later Phase)

**Suggested scenario**: "Searing Touch with Buyback -- cast, resolve, verify return to hand, cast again"
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Sequence number**: Next available (likely 091+)

## Files Modified Summary

| File | Action |
|------|--------|
| `crates/engine/src/rules/command.rs` | Add `cast_with_buyback: bool` to `CastSpell` variant |
| `crates/engine/src/rules/engine.rs` | Wire `cast_with_buyback` through dispatch |
| `crates/engine/src/rules/casting.rs` | Add param, `get_buyback_cost()`, cost addition, set `was_buyback_paid` |
| `crates/engine/src/rules/resolution.rs` | Check `was_buyback_paid` for hand destination |
| `crates/engine/src/testing/script_schema.rs` | Add `buyback: bool` field |
| `crates/engine/src/testing/replay_harness.rs` | Wire `buyback` through all `cast_spell*` arms |
| `crates/engine/tests/buyback.rs` | 7-8 unit tests |

## Interactions to Watch

- **Buyback + Flashback (resolution destination priority)**: Flashback exile overrides buyback return-to-hand. Check the resolution path handles the priority correctly (flashback checked first).
- **Buyback + CounterSpell (effects/mod.rs)**: Countered spells go to graveyard. The `CounterSpell` effect path must NOT check `was_buyback_paid`. It currently doesn't -- just verify it stays that way.
- **Buyback + Fizzle (resolution.rs fizzle path)**: Fizzled spells go to graveyard. The fizzle path must NOT check `was_buyback_paid`. It currently doesn't -- verify it stays that way.
- **Buyback + Cost pipeline order**: Buyback cost must be added AFTER commander tax and kicker, BEFORE affinity/undaunted/convoke/improvise/delve. This matches the existing pipeline architecture.
- **All `StackObject` construction sites**: Every place that constructs a `StackObject` must include `was_buyback_paid: false` (for non-spell entries). Currently there are ~10 sites in `casting.rs`, `copy.rs`, `abilities.rs`. These already have the field (added in the pre-edit). Verify they all compile with the new `cast_with_buyback` on `Command::CastSpell`.
- **All `Command::CastSpell` construction sites**: The replay harness has ~8 arms that construct `CastSpell`. All need `cast_with_buyback` added. The copy.rs cascade cast also constructs `CastSpell` indirectly -- verify.
