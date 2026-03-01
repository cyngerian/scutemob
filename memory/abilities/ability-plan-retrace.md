# Ability Plan: Retrace

**Generated**: 2026-03-01
**CR**: 702.81
**Priority**: P4
**Batch**: 4.1 (Alt-cast graveyard)
**Similar abilities studied**: Flashback (CR 702.34) -- casting.rs, resolution.rs, stack.rs, hash.rs, flashback.rs tests

## CR Rule Text

702.81. Retrace

702.81a Retrace is a static ability that functions while the card with retrace is in a player's graveyard. "Retrace" means "You may cast this card from your graveyard by discarding a land card as an additional cost to cast it." Casting a spell using its retrace ability follows the rules for paying additional costs in rules 601.2b and 601.2f-h.

## Key Edge Cases

1. **Card returns to graveyard, NOT exile** (ruling 2008-08-01 on all Retrace cards): "When a retrace card you cast from your graveyard resolves, fails to resolve, or is countered, it's put back into your graveyard. You may use the retrace ability to cast it again." This is the critical difference from Flashback.

2. **Discard is an ADDITIONAL cost, not an alternative cost** (CR 702.81a): The player pays the card's normal mana cost PLUS discards a land card. This means Retrace can combine with kicker, buyback, and other additional costs. It can also combine with cost reducers like convoke/improvise/delve.

3. **Normal timing rules apply** (ruling 2008-08-01): "A retrace card cast from your graveyard follows the normal timing rules for its card type." Sorceries with retrace can only be cast at sorcery speed, even from the graveyard.

4. **The discarded card must be a land card** (CR 702.81a): Specifically a card with CardType::Land in its type line. Not "a card named" -- any land card works.

5. **Re-castable loop** (ruling 2008-08-01): "If the active player casts a spell that has retrace, that player may cast that card again after it resolves, before another player can remove the card from the graveyard." The active player has priority after resolution, so they can immediately re-cast. This is naturally handled by the engine's priority system.

6. **Retrace is NOT an alternative cost** (CR 702.81a says "additional cost"): This means Retrace CAN combine with Flashback, Escape, and other alternative costs if a card somehow has both. However, for Retrace spells, the engine uses the card's normal mana cost (not a separate Retrace cost), so no `AbilityDefinition::Retrace { cost }` is needed -- just the keyword.

7. **Multiplayer**: No special multiplayer considerations. The land discard cost is personal to the caster. Normal APNAP priority applies.

8. **Only instants and sorceries in practice**: All printed Retrace cards are instants or sorceries. The CR text doesn't explicitly restrict to instants/sorceries (unlike Flashback which says "if the resulting spell is an instant or sorcery spell"), but all printed cards with Retrace are instants/sorceries. The engine should not add a type restriction that the CR doesn't require.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Retrace vs Flashback: Design Differences

| Aspect | Flashback | Retrace |
|--------|-----------|---------|
| Cost type | Alternative cost (CR 118.9) | Additional cost (CR 118.8) |
| Mana paid | Flashback cost (from AbilityDefinition) | Normal mana cost |
| Extra cost | None | Discard a land card from hand |
| On resolution | Exile | Graveyard (normal) |
| On counter | Exile | Graveyard (normal) |
| On fizzle | Exile | Graveyard (normal) |
| Re-castable | No (exiled) | Yes (returns to graveyard) |
| Combines with alt costs | No (IS an alt cost) | Yes (is additional cost) |
| AbilityDefinition | `Flashback { cost: ManaCost }` | Not needed (no cost data) |
| StackObject flag | `cast_with_flashback: bool` | `cast_with_retrace: bool` -- BUT only used for the discard event, NOT for exile replacement |

**Key insight**: Because Retrace does not change where the card goes on resolution/counter/fizzle, we do NOT need a StackObject flag at all for resolution routing. The card follows the normal instant/sorcery path (to graveyard). However, we DO need the flag to suppress the Flashback exile path if somehow both keywords are on one card. Actually -- a card with both Flashback and Retrace would use one at a time: Flashback is an alternative cost, Retrace is additional cost + normal mana. If cast via Flashback, the `cast_with_flashback` flag handles exile. If cast via Retrace, no exile happens. The detection logic already handles this: the engine checks for Flashback keyword in graveyard first. We need Retrace to be an independent path.

**Revised approach**: We need `casting_with_retrace` as a local variable in `handle_cast_spell` to:
1. Allow casting from graveyard when the card has `KeywordAbility::Retrace`
2. Validate and discard a land card from hand as additional cost
3. NOT set any exile flag -- normal resolution applies

We also need a new field on `Command::CastSpell`: `retrace_discard_land: Option<ObjectId>` so the player specifies which land card from hand to discard.

## Implementation Steps

### Step 1: Enum Variant — KeywordAbility::Retrace

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Retrace` variant after `CommanderNinjutsu` (discriminant 88).
**Pattern**: Follow `KeywordAbility::Flashback` at its location.
**Doc comment**:
```rust
/// CR 702.81: Retrace -- card may be cast from the owner's graveyard
/// by discarding a land card as an additional cost (CR 118.8).
/// Unlike Flashback, the card returns to the graveyard on resolution
/// (not exiled). The normal mana cost is paid (not an alternative cost).
///
/// This variant is a marker for quick presence-checking (`keywords.contains`).
/// No `AbilityDefinition::Retrace` is needed because there is no separate
/// cost to store -- the card uses its normal mana cost.
Retrace,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant for `KeywordAbility::Retrace` at discriminant 89 (next after CommanderNinjutsu=88).
**Pattern**: Follow the `KeywordAbility::Flashback => 27u8.hash_into(hasher)` pattern.
```rust
// Retrace (discriminant 89) -- CR 702.81
KeywordAbility::Retrace => 89u8.hash_into(hasher),
```

**Note**: No `AbilityDefinition::Retrace` variant is needed. Retrace uses the card's normal mana cost, and the "discard a land" additional cost is not a mana cost -- it is validated/paid inline in `casting.rs`. This contrasts with Flashback which needs `AbilityDefinition::Flashback { cost }` to store the alternative mana cost.

### Step 2: Command Extension — retrace_discard_land

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add a new field to `Command::CastSpell`:
```rust
/// CR 702.81a: If casting via retrace, the ObjectId of a land card in
/// the player's hand to discard as an additional cost. Must be a card
/// with CardType::Land in the player's hand. None if not using retrace.
#[serde(default)]
retrace_discard_land: Option<ObjectId>,
```
**Location**: After `cast_with_overload: bool` (line ~166 in command.rs).

**Grep for all `Command::CastSpell {` construction sites** and add `retrace_discard_land: None` to each. Key files:
- `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs` (storm trigger, cascade trigger construction)
- `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs` (cascade free cast, suspend free cast)
- `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs` (madness cast)
- `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs` (all action type handlers)
- `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs` (CastSpell destructure)
- All test files that construct `Command::CastSpell`

**Critical**: Grep for `CastSpell {` across the entire `crates/engine/src/` and `crates/engine/tests/` trees. Every construction site needs the new field. Because the field has `#[serde(default)]` and is `Option<ObjectId>` (defaults to `None`), existing JSON scripts will continue to work without modification.

### Step 3: Rule Enforcement in casting.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Wire Retrace into the graveyard-casting and additional-cost machinery.

#### Step 3a: Detection (near line 87-200)

After the existing `casting_with_flashback` detection block, add Retrace detection:

```rust
// CR 702.81a: Retrace -- allowed if card has the Retrace keyword and is in graveyard.
// Retrace is an additional cost (not alternative), so it does NOT conflict with
// Flashback. However, if the card is being cast via Flashback (alternative cost),
// Retrace's additional cost (discard land) is NOT required -- Flashback replaces
// the normal casting method entirely.
let casting_with_retrace = casting_from_graveyard
    && card_obj.characteristics.keywords.contains(&KeywordAbility::Retrace)
    && !casting_with_flashback  // Flashback takes priority over Retrace
    && retrace_discard_land.is_some();  // Player must provide a land to discard
```

Also update the zone validation block (near line 184-195) to allow Retrace:
```rust
if card_obj.zone != ZoneId::Hand(player)
    && !casting_from_command_zone
    && !casting_with_flashback
    && !casting_with_retrace  // <-- ADD THIS
    && !casting_with_madness
    && !casting_with_escape_auto
    && !cast_with_escape
    && !cast_with_foretell
{
    return Err(GameStateError::InvalidCommand("card is not in your hand".into()));
}
```

**Important edge case**: A card in the graveyard with Retrace but no `retrace_discard_land` provided should NOT auto-cast. The player must explicitly provide the land card. If they provide `retrace_discard_land: Some(land_id)`, validate and proceed. If `retrace_discard_land: None`, fall through to "card is not in your hand" unless another graveyard permission (Flashback, Escape) applies.

**However**, consider the case where a card has BOTH Retrace and Escape: the player provides `cast_with_escape: true` and `retrace_discard_land: None` -- they want Escape, not Retrace. This works naturally because `casting_with_retrace` requires `retrace_discard_land.is_some()`.

#### Step 3b: Discard land validation and payment (near line 660-700, after buyback cost)

After the buyback cost block, add Retrace land discard validation and payment:

```rust
// CR 702.81a / 601.2f: Retrace -- discard a land card as additional cost.
// The discard happens as part of cost payment (CR 601.2f-h), before the
// spell goes on the stack. This is similar to how cycling discards the
// card itself as part of the cost.
let mut retrace_discard_events: Vec<GameEvent> = Vec::new();
if casting_with_retrace {
    let land_id = retrace_discard_land.expect("casting_with_retrace requires retrace_discard_land");

    // Validate the land card:
    // 1. Must be in the player's hand
    // 2. Must have CardType::Land
    let land_obj = state.object(land_id)?;
    if land_obj.zone != ZoneId::Hand(player) {
        return Err(GameStateError::InvalidCommand(
            "retrace: discarded card must be in your hand (CR 702.81a)".into(),
        ));
    }
    if !land_obj.characteristics.card_types.contains(&CardType::Land) {
        return Err(GameStateError::InvalidCommand(
            "retrace: discarded card must be a land card (CR 702.81a)".into(),
        ));
    }

    // Pay the cost: discard (move from hand to graveyard)
    let land_owner = land_obj.owner;
    let (new_land_id, _) = state.move_object_to_zone(land_id, ZoneId::Graveyard(land_owner))?;
    retrace_discard_events.push(GameEvent::CardDiscarded {
        player,
        object_id: land_id,
        new_id: new_land_id,
    });
}
```

**Placement**: This must happen AFTER mana cost determination (the mana cost is the card's normal mana cost, not modified by Retrace) but BEFORE mana payment. Actually, per CR 601.2f, the total cost is determined first, then all costs are paid. The discard is part of cost payment. Place the discard block alongside the mana payment block, or just before the `pay_cost` call. Follow the buyback pattern (validate and bind the cost, then pay alongside mana).

**Alternative placement**: Actually, the cleanest approach is to perform the discard AFTER mana payment but BEFORE pushing the spell onto the stack. This matches the cycling pattern where the discard happens as cost payment. The retrace_discard_events are then appended to the events list alongside mana payment events.

#### Step 3c: Mana cost selection (near line 538-548)

Retrace uses the card's NORMAL mana cost (not an alternative). The existing code path already falls through to `base_mana_cost` when no alternative cost is selected. Verify that when `casting_with_retrace` is true and `casting_with_flashback` is false, the normal mana cost path is taken. This should work automatically -- no change needed here.

#### Step 3d: Append events and return (near line 920-1148)

Add the `retrace_discard_events` to the events list alongside the mana payment events:

```rust
events.extend(retrace_discard_events);
```

**Location**: After mana payment events, before the SpellCast event emission.

#### Step 3e: NO StackObject flag needed

Unlike Flashback, Retrace does NOT change where the card goes on resolution/counter/fizzle. The card goes to the graveyard normally (standard instant/sorcery resolution path). Therefore:
- NO new `cast_with_retrace` field on `StackObject`
- NO changes to `resolution.rs` fizzle/resolution/counter paths
- NO changes to `effects/mod.rs` CounterSpell effect
- NO changes to `hash.rs` StackObject hashing

This is the simplest part of the implementation: the absence of special handling IS the correct behavior.

### Step 4: Replay Harness — cast_spell_retrace action

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add a new action type `"cast_spell_retrace"` for game scripts.

**Pattern**: Follow `"cast_spell_flashback"` (line ~298).

```rust
// CR 702.81a: Cast a spell with retrace from the player's graveyard.
// The player discards a land card from hand as an additional cost.
// The spell uses its normal mana cost.
"cast_spell_retrace" => {
    let card_id = find_in_graveyard(state, player, card_name?)?;
    let target_list = resolve_targets(targets, state, players);
    // The land to discard is specified in a "discard_land" field in the action JSON.
    let discard_land_name = action.get("discard_land")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("cast_spell_retrace requires 'discard_land' field"))?;
    let land_id = find_in_hand(state, player, discard_land_name)?;
    Some(Command::CastSpell {
        player,
        card: card_id,
        targets: target_list,
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        cast_with_evoke: false,
        cast_with_bestow: false,
        cast_with_miracle: false,
        cast_with_escape: false,
        escape_exile_cards: vec![],
        cast_with_foretell: false,
        cast_with_buyback: false,
        cast_with_overload: false,
        retrace_discard_land: Some(land_id),
    })
}
```

### Step 5: Trigger Wiring

**Not applicable.** Retrace is a static ability that grants permission to cast from the graveyard. It has no triggers. The `CardDiscarded` event from the land discard will naturally trigger any "whenever a player discards a card" triggers through the existing trigger infrastructure.

### Step 6: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/retrace.rs`
**Pattern**: Follow `/home/airbaggie/scutemob/crates/engine/tests/flashback.rs` structure.

**Card definitions for tests**:

```rust
/// Flame Jab: Sorcery {R}, "Flame Jab deals 1 damage to any target. Retrace."
fn flame_jab_def() -> CardDefinition { ... }

/// A simple land card for the discard cost.
fn mountain_def() -> CardDefinition { ... }

/// A non-land card for negative testing.
fn lightning_bolt_def() -> CardDefinition { ... }
```

**Tests to write**:

1. `test_retrace_basic_cast_from_graveyard` -- CR 702.81a: Cast Flame Jab from graveyard by paying {R} + discarding a Mountain. Verify SpellCast event, card on stack, mana paid, land discarded (CardDiscarded event).

2. `test_retrace_card_returns_to_graveyard_on_resolution` -- CR 702.81a + ruling 2008-08-01: After resolution, Flame Jab is back in the graveyard (not exile). Verify it can be found in `ZoneId::Graveyard(p1)` and NOT in `ZoneId::Exile`.

3. `test_retrace_card_returns_to_graveyard_when_countered` -- CR 702.81a + ruling: When a retrace spell is countered, it goes to the graveyard normally. Set up: p1 casts Flame Jab via retrace, p2 counters with Counterspell. Verify Flame Jab in graveyard.

4. `test_retrace_recast_after_resolution` -- Ruling 2008-08-01: Cast Flame Jab via retrace, let it resolve (goes to graveyard), then cast it again via retrace with another land. Verifies the re-castable loop.

5. `test_retrace_normal_timing_sorcery` -- Ruling 2008-08-01: A sorcery with retrace follows sorcery-speed timing from the graveyard. Attempt to cast during opponent's turn; verify error.

6. `test_retrace_discard_must_be_land` -- CR 702.81a: Attempting to discard a non-land card (e.g., Lightning Bolt) as the retrace cost produces an error.

7. `test_retrace_discard_must_be_in_hand` -- CR 702.81a: Attempting to discard a land that is not in the player's hand (e.g., on battlefield) produces an error.

8. `test_retrace_no_retrace_keyword_cannot_cast` -- Negative test: A card in the graveyard without Retrace keyword cannot be cast via retrace (providing `retrace_discard_land` on a non-Retrace card).

9. `test_retrace_pays_normal_mana_cost` -- CR 702.81a: Retrace pays the card's normal mana cost, not a separate retrace cost. Verify mana pool is reduced by {R} (Flame Jab's mana cost).

10. `test_retrace_without_land_in_hand_fails` -- CR 702.81a: If the player has no land cards in hand, they cannot cast via retrace. Verify error.

11. `test_retrace_normal_hand_cast_no_land_discard_needed` -- When casting a Retrace card from hand (normal cast), no land discard is required. Verify normal cast works with `retrace_discard_land: None`.

### Step 7: Card Definition (later phase)

**Suggested card**: Flame Jab
- Simple: Sorcery, {R}, "Flame Jab deals 1 damage to any target."
- Tests the targeting + damage effect + retrace combination.
- Oracle text is straightforward -- no complex effects.

**Alternative**: Raven's Crime ({B}, "Target player discards a card.") -- also simple, but discard-target effects are slightly more complex for testing.

**Card definition pattern**:
```rust
CardDefinition {
    card_id: CardId("flame-jab".to_string()),
    name: "Flame Jab".to_string(),
    mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
    types: TypeLine {
        card_types: [CardType::Sorcery].into_iter().collect(),
        ..Default::default()
    },
    oracle_text: "Flame Jab deals 1 damage to any target.\nRetrace".to_string(),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Retrace),
        AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![TargetRequirement::TargetAny],
            modes: None,
            cant_be_countered: false,
        },
    ],
    ..Default::default()
}
```

### Step 8: Game Script (later phase)

**Suggested scenario**: "Retrace loop -- cast, resolve, recast"
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Sequence number**: Next available in stack/ directory

**Scenario**:
1. p1 has Flame Jab in graveyard, two Mountains in hand, {R}{R} in mana pool.
2. p1 casts Flame Jab via retrace targeting p2, discarding Mountain #1.
3. All pass -- Flame Jab resolves, deals 1 damage to p2. Flame Jab returns to graveyard.
4. p1 casts Flame Jab again via retrace targeting p2, discarding Mountain #2.
5. All pass -- resolves, deals 1 damage to p2. Flame Jab in graveyard again.
6. Assert: p2 life = 38 (40 - 1 - 1), Flame Jab in graveyard, both Mountains in graveyard.

## Interactions to Watch

1. **Retrace + Flashback on the same card**: Theoretically possible via external effects granting keywords. If a card has both, the player chooses which to use. Flashback = alternative cost (uses flashback cost, exiles on departure). Retrace = additional cost (uses normal mana cost + land discard, no exile). The `casting_with_flashback` check takes priority via position -- `casting_with_retrace` explicitly excludes `casting_with_flashback == true`.

2. **Retrace + Escape on the same card**: Same principle. Escape is an alternative cost. If the player provides `cast_with_escape: true`, they use Escape (not Retrace). If they provide `retrace_discard_land: Some(id)`, they use Retrace (not Escape). If both are provided simultaneously, need a clear error or precedence rule. Recommend: error if both `cast_with_escape: true` and `retrace_discard_land: Some(_)`.

3. **Retrace + Kicker/Buyback**: Legal combinations. Retrace's land discard is additional cost. Kicker is additional cost. Buyback is additional cost. All three can apply simultaneously. The total cost = normal mana cost + kicker cost + buyback cost, PLUS discard a land card. No special handling needed -- existing additional cost infrastructure handles this.

4. **Retrace + Convoke/Improvise/Delve**: Legal. These are cost reducers applied after the total cost is determined. The land discard is separate from mana cost reduction. No conflict.

5. **Discard triggers**: The `CardDiscarded` event from the land discard will naturally fire "whenever a player discards" triggers (e.g., Waste Not, Bone Miser). This is correct per CR -- the discard is a real discard as part of cost payment.

6. **Madness interaction**: If the discarded land card has Madness, the madness replacement applies (exile instead of graveyard, put madness trigger on stack). This should work automatically through the existing madness discard replacement in `replacement.rs` or `abilities.rs`. However, note that in the casting.rs implementation, we move the land directly to graveyard. We may need to check if the land has Madness and route through the madness replacement. **CHECK**: Does the existing discard path in casting.rs handle madness? The cycling handler in `abilities.rs` (line 460-502) does handle madness for the cycling discard. We should follow the same pattern for the retrace land discard.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Retrace` variant |
| `crates/engine/src/state/hash.rs` | Add hash discriminant 89 for Retrace |
| `crates/engine/src/rules/command.rs` | Add `retrace_discard_land: Option<ObjectId>` to CastSpell |
| `crates/engine/src/rules/casting.rs` | Retrace detection, zone validation, land discard cost payment |
| `crates/engine/src/rules/engine.rs` | Destructure `retrace_discard_land` in CastSpell handler |
| `crates/engine/src/testing/replay_harness.rs` | Add `cast_spell_retrace` action, add `retrace_discard_land: None` to all existing CastSpell constructions |
| `crates/engine/tests/retrace.rs` | 11 unit tests |
| All files constructing `Command::CastSpell` | Add `retrace_discard_land: None` field |

## Estimated Complexity

**Low**. Retrace is simpler than Flashback because:
- No alternative cost (uses normal mana cost)
- No exile replacement (card goes to graveyard normally)
- No new StackObject field
- No changes to resolution.rs, effects/mod.rs, or sba.rs
- The only new behavior is: allow cast from graveyard + validate/pay land discard cost

The bulk of the work is mechanical: adding the new Command field to all CastSpell construction sites (grep-and-add pattern), plus the casting.rs enforcement logic.
