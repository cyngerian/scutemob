# Ability Plan: Delve

**Generated**: 2026-02-26
**CR**: 702.66
**Priority**: P2
**Similar abilities studied**: Convoke (CR 702.51) -- `crates/engine/src/rules/casting.rs`, `crates/engine/src/state/types.rs:237`, `crates/engine/tests/convoke.rs`

## CR Rule Text

702.66. Delve

- **702.66a**: Delve is a static ability that functions while the spell with delve is on the stack. "Delve" means "For each generic mana in this spell's total cost, you may exile a card from your graveyard rather than pay that mana."
- **702.66b**: The delve ability isn't an additional or alternative cost and applies only after the total cost of the spell with delve is determined.
- **702.66c**: Multiple instances of delve on the same spell are redundant.

## Key Edge Cases

1. **Delve only pays generic mana** (CR 702.66a) -- unlike Convoke, Delve cannot pay for colored mana pips. Each exiled card pays for exactly {1} generic. This is the critical difference from Convoke's implementation.
2. **Delve does not change mana cost or mana value** (ruling on every Delve card) -- "Delve doesn't change a spell's mana cost or mana value. For example, Treasure Cruise's mana value is 8 even if you exiled three cards to cast it."
3. **Delve is not an additional or alternative cost** (CR 702.66b) -- it can be used in conjunction with alternative costs (flashback) and additional costs (commander tax, kicker). Applies AFTER total cost is determined.
4. **Cannot exile more cards than the generic mana requirement** (Treasure Cruise ruling) -- "You can exile cards to pay only for generic mana, and you can't exile more cards than the generic mana requirement of a spell with delve."
5. **Delve works with flashback** (Treasure Cruise ruling) -- "Because delve isn't an alternative cost, it can be used in conjunction with alternative costs, such as flashback."
6. **Delve works with commander tax** -- Commander tax is an additional cost added before delve applies (CR 702.66b). Delve can reduce the total generic portion including tax.
7. **Multiple instances of delve are redundant** (CR 702.66c) -- no extra benefit from having two instances.
8. **Multiplayer considerations** -- In Commander, graveyards tend to fill quickly (mill, self-discard, creature deaths). Delve is a powerful cost reduction. Each exiled card is its own zone-change (CR 400.7), producing a new ObjectId in exile. The engine must handle exiling N cards atomically as part of casting cost.
9. **Cards must be in the caster's graveyard** (CR 702.66a: "your graveyard") -- cannot exile cards from an opponent's graveyard.
10. **Ethereal Forager ruling** -- cards exiled with delve can be tracked by the engine (the exile zone retains ObjectIds), enabling future cards that care about "cards exiled with delve."

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (N/A -- Delve is a static ability / cost modification)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Delve` variant after `Convoke` at line 237.
**Pattern**: Follow `KeywordAbility::Convoke` (simple unit variant, no payload).

```rust
/// CR 702.66: Delve -- exile cards from your graveyard to pay for generic mana.
/// "For each generic mana in this spell's total cost, you may exile a card
/// from your graveyard rather than pay that mana."
/// CR 702.66b: Not an additional or alternative cost; applies after total cost determined.
/// CR 702.66c: Multiple instances are redundant.
Delve,
```

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` after Convoke (discriminant 30). Use discriminant 31:

```rust
// Delve (discriminant 31) -- CR 702.66
KeywordAbility::Delve => 31u8.hash_into(hasher),
```

**Match arms**: Grep for `KeywordAbility::Convoke` match expressions engine-wide and add `KeywordAbility::Delve` wherever Convoke appears in a match. Since Delve is a unit variant like Convoke, this is straightforward. Key locations:
- `state/hash.rs` (discriminant -- covered above)
- Any exhaustive match on `KeywordAbility` that was added for Convoke

### Step 2: Rule Enforcement

This step has two sub-parts: (A) add the `delve_cards` field to the `CastSpell` command, and (B) implement `apply_delve_reduction()` in `casting.rs`.

#### Step 2A: CastSpell Command -- Add `delve_cards` Field

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `delve_cards: Vec<ObjectId>` field to `Command::CastSpell` variant.
**Pattern**: Follow `convoke_creatures: Vec<ObjectId>` at line 71.

```rust
/// CR 702.66: Cards in the caster's graveyard to exile for delve cost reduction.
/// Empty vec for non-delve spells. Each card must be:
/// - In the caster's graveyard (not opponent's)
/// - Not duplicated (no ObjectId appears twice)
/// Each exiled card pays for {1} generic mana. Cannot exceed the generic
/// mana component of the spell's total cost.
/// Validated in handle_cast_spell -> apply_delve_reduction.
delve_cards: Vec<ObjectId>,
```

**Propagation**: The new field must be added everywhere `Command::CastSpell` is constructed or destructured:

1. **`/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`** -- Add `delve_cards` to the destructure at the `Command::CastSpell` arm (around line 70-80), and pass it to `handle_cast_spell`.
2. **`/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`** -- Add `delve_cards: Vec<ObjectId>` parameter to `handle_cast_spell` signature (around line 47-53).
3. **`/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`** -- Add `delve_cards: vec![]` to the `cast_spell_flashback` arm, and for `cast_spell`, resolve `delve_names` from the script JSON. Also add `delve_names: &[String]` parameter to `translate_player_action`.
4. **Every test file that constructs `Command::CastSpell`** -- Add `delve_cards: vec![]`. Grep for `convoke_creatures:` in `crates/engine/tests/` to find all ~20 files.
5. **`/home/airbaggie/scutemob/crates/engine/src/testing/script_schema.rs`** -- Add `delve: Vec<String>` field to `ScriptAction::Action` (parallel to `convoke: Vec<String>` at line 222).
6. **`/home/airbaggie/scutemob/crates/engine/tests/script_replay.rs`** -- Pass the new `delve` field from `ScriptAction` through to `translate_player_action`.

#### Step 2B: apply_delve_reduction() in casting.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Add `apply_delve_reduction()` function and call it in `handle_cast_spell` after convoke but before mana payment.
**Pattern**: Follow `apply_convoke_reduction()` at line 499, but SIMPLER -- Delve only reduces generic mana.
**CR**: CR 702.66a -- each exiled card pays for {1} generic mana. CR 702.66b -- applies after total cost determined.

**Call site** (in `handle_cast_spell`, after the convoke block around line 254-273):

```rust
// CR 702.66a / 702.66b: Apply delve cost reduction AFTER total cost is determined.
// Delve is not an additional or alternative cost -- it applies to the total cost.
// Order: base_mana_cost -> commander_tax -> flashback -> CONVOKE -> DELVE -> pay.
let mut delve_events: Vec<GameEvent> = Vec::new();
let mana_cost = if !delve_cards.is_empty() {
    if !chars.keywords.contains(&KeywordAbility::Delve) {
        return Err(GameStateError::InvalidCommand(
            "spell does not have delve".into(),
        ));
    }
    apply_delve_reduction(
        state,
        player,
        &delve_cards,
        mana_cost,
        &mut delve_events,
    )?
} else {
    mana_cost
};
```

**The `apply_delve_reduction` function**:

```rust
/// CR 702.66a: Validate delve cards and compute the reduced mana cost.
///
/// For each card in `delve_cards`:
/// - Must exist in `state.objects` in the caster's graveyard.
/// - Must not appear twice (no duplicates).
///
/// Reduction (CR 702.66a):
/// - Each card reduces one generic pip. Cannot exceed total generic mana.
///
/// Exiles each card via `state.move_object_to_zone(id, ZoneId::Exile)` and
/// emits `ObjectExiled` events for each.
/// Returns the reduced `Option<ManaCost>`.
fn apply_delve_reduction(
    state: &mut GameState,
    player: PlayerId,
    delve_cards: &[ObjectId],
    cost: Option<ManaCost>,
    events: &mut Vec<GameEvent>,
) -> Result<Option<ManaCost>, GameStateError> {
    // Validate uniqueness
    // Validate each card is in caster's graveyard
    // Check that delve_cards.len() <= cost.generic
    // Reduce generic by delve_cards.len()
    // Exile each card: state.move_object_to_zone(id, ZoneId::Exile)
    // Emit ObjectExiled event for each
    // Return reduced cost
}
```

Key differences from `apply_convoke_reduction`:
1. **No color matching** -- Delve only reduces generic mana, never colored pips.
2. **Zone is graveyard, not battlefield** -- cards must be in `ZoneId::Graveyard(player)`.
3. **Cards are exiled, not tapped** -- use `state.move_object_to_zone(id, ZoneId::Exile)` and emit `ObjectExiled` events (not `PermanentTapped`).
4. **No creature requirement** -- any card in the graveyard can be exiled for delve (lands, instants, creatures, etc.).
5. **Simpler validation** -- only need to check graveyard membership and duplicates (no tapped check, no creature check, no controller check since graveyard is per-player).

**Event emission**: The events pushed to `events` vec should be `GameEvent::ObjectExiled` for each card exiled. These events are emitted AFTER mana payment (same pattern as convoke's `PermanentTapped` events at line 294-296 in casting.rs). The `delve_events` are extended into the main events vec after `ManaCostPaid`.

**Important**: `move_object_to_zone` changes ObjectIds (CR 400.7). The old ObjectId from `delve_cards` becomes dead. The new ObjectId is returned from `move_object_to_zone` and should be used in the `ObjectExiled` event's `new_object_id` field.

### Step 3: Trigger Wiring

**N/A** -- Delve is a static ability that modifies cost payment (CR 702.66a). It is not a triggered ability and has no trigger to wire. The exile happens as part of casting cost payment, not as a stack-based effect.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/delve.rs`
**Pattern**: Follow `crates/engine/tests/convoke.rs` (12 tests, same structure).

**Tests to write** (10 tests):

1. **`test_delve_basic_exile_cards_reduce_generic_cost`** -- CR 702.66a
   - Treasure Cruise-like spell: {7}{U}. Exile 7 cards from graveyard, pay {U} from pool.
   - Assert: spell on stack, 7 cards in exile, graveyard size decreased by 7, mana pool empty.

2. **`test_delve_partial_reduction`** -- CR 702.66a
   - {4}{B} spell (Murderous Cut-like). Exile 3 cards, pay {1}{B} from pool.
   - Assert: spell on stack, 3 cards exiled, 1 generic + 1 black paid from pool.

3. **`test_delve_object_exiled_events`** -- CR 702.66a + CR 400.7
   - Cast a delve spell exiling 3 cards. Assert that 3 `ObjectExiled` events are emitted.
   - Verify old ObjectIds are retired (no longer in `state.objects`).

4. **`test_delve_reject_no_keyword`** -- CR 702.66a
   - Attempt delve on a spell without the Delve keyword. Assert error.

5. **`test_delve_reject_too_many_cards`** -- CR 702.66a / Treasure Cruise ruling
   - {2}{U} spell (2 generic). Try to exile 3 cards. Assert error ("exceeds generic mana").

6. **`test_delve_reject_card_not_in_graveyard`** -- CR 702.66a
   - Pass an ObjectId that is on the battlefield (not in graveyard). Assert error.

7. **`test_delve_reject_opponents_graveyard`** -- CR 702.66a ("your graveyard")
   - Pass an ObjectId from an opponent's graveyard. Assert error.

8. **`test_delve_reject_duplicate_cards`** -- Validation
   - Pass the same ObjectId twice. Assert error ("duplicate card").

9. **`test_delve_zero_cards_normal_cast`** -- CR 702.66a
   - A spell with Delve can be cast normally with empty `delve_cards`. Full mana payment.

10. **`test_delve_with_commander_tax`** -- CR 702.66b + CR 903.8
    - Commander with Delve: {4}{U}{U}. After 1 previous cast, tax = {2}. Total = {6}{U}{U}.
    - Exile 6 cards, pay {U}{U} from pool. Assert: spell on stack, tax incremented, 6 cards exiled.

**Helper functions**:

```rust
/// Create a delve spell in hand.
fn delve_spell_spec(owner: PlayerId, name: &str, generic: u32, blue: u32) -> ObjectSpec

/// Create a card in the graveyard (any card type -- lands, creatures, instants).
fn graveyard_card(owner: PlayerId, name: &str) -> ObjectSpec
```

### Step 5: Card Definition

**Suggested card**: Treasure Cruise

**Card**: Treasure Cruise
- **Card ID**: `treasure-cruise`
- **Mana Cost**: {7}{U}
- **Type**: Sorcery
- **Oracle Text**: "Delve (Each card you exile from your graveyard while casting this spell pays for {1}.)\nDraw three cards."
- **Color Identity**: ["U"]
- **Keywords**: [Delve]
- **Abilities**: `AbilityDefinition::Keyword(KeywordAbility::Delve)`
- **Effect**: `Effect::DrawCards { player: EffectTarget::Controller, amount: EffectAmount::Fixed(3) }`

**Why Treasure Cruise**: It is the most iconic delve card (banned/restricted in multiple formats), has a simple effect (draw 3), high generic cost ({7}), and exactly one colored pip ({U}). This makes it ideal for testing -- you can verify that delve exiles up to 7 cards paying the generic portion, while the blue pip must come from the mana pool. The draw-3 effect is already implemented (`Effect::DrawCards`).

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs`
**Action**: Add after the last card definition, following the pattern of existing sorceries.

**Alternative card** (if a second card is desired): Murderous Cut ({4}{B}, Instant, "Delve. Destroy target creature.") -- tests delve with targeting and instant-speed casting.

### Step 6: Game Script

**Suggested scenario**: "Delve basic -- Treasure Cruise with full delve"

**Subsystem directory**: `test-data/generated-scripts/stack/`

**Scenario description**:
1. Player 1 starts with Treasure Cruise in hand, 7 assorted cards in graveyard (mix of creatures, lands, instants), and {U} in mana pool.
2. Player 1 casts Treasure Cruise using delve, exiling all 7 graveyard cards.
3. All players pass priority.
4. Treasure Cruise resolves -- Player 1 draws 3 cards.

**Assertions**:
- After casting: spell on stack, graveyard empty (all 7 exiled), exile has 7 cards, mana pool empty.
- After resolution: 3 cards drawn (hand size increased), spell in graveyard (sorcery), stack empty.

**Script action format** (new `delve` field parallel to `convoke`):
```json
{
  "action": "cast_spell",
  "player": "p1",
  "card": "Treasure Cruise",
  "delve": ["Lightning Bolt", "Mountain", "Grizzly Bears", "Forest", "Cancel", "Island", "Elvish Mystic"]
}
```

## Interactions to Watch

1. **Convoke + Delve on the same spell**: Theoretically possible if a future card has both. The implementation should handle both reductions sequentially: convoke first (reduces colored and generic), then delve (reduces remaining generic). The current ordering in `handle_cast_spell` (convoke -> delve -> pay) handles this naturally.

2. **Flashback + Delve**: CR ruling confirms this works. When casting from graveyard via flashback, the spell being cast is MOVED from graveyard to stack first (before costs are paid). The delve cards are OTHER cards in the graveyard -- the spell itself is already on the stack and cannot be exiled for its own delve. No special handling needed.

3. **Commander tax + Delve**: Tax increases generic cost. Delve reduces generic cost. The total cost calculation order (base -> tax -> convoke -> delve -> pay) ensures delve can reduce the tax-inflated generic portion. A commander with {2}{U}{U} base cost and 2 tax would have {4}{U}{U} total; delve could exile up to 4 cards.

4. **Empty graveyard**: If the player has no cards in graveyard, `delve_cards` must be empty. The spell can still be cast normally (same as convoke with no creatures -- Step 9 test).

5. **Mana value unchanged**: The engine's `mana_value()` method reads from the card's mana cost, not from what was actually paid. Since delve doesn't modify `ManaCost` on the card itself (it modifies the remaining payment), mana value stays correct. Confirm this is true for `calculate_characteristics` too.

6. **Object identity (CR 400.7)**: Each exiled card gets a new ObjectId. The old graveyard ObjectIds in the `delve_cards` vec become dead after exile. Tests must search by name in the exile zone, not by the old ObjectId.

7. **Graveyard order**: In MTG, graveyard order matters in some formats but not in Commander. The engine uses `im::Vector` for zone contents. Delve removes specific cards by ObjectId, not from top/bottom, so order is irrelevant.
