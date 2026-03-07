# Ability Plan: Fuse

**Generated**: 2026-03-07
**CR**: 702.102
**Priority**: P4
**Similar abilities studied**: Aftermath (702.127) — split card second half with separate cast zone; Entwine (702.42) — "execute all modes" pattern

## CR Rule Text

702.102. Fuse

702.102a Fuse is a static ability found on some split cards (see rule 709, "Split Cards") that applies while the card with fuse is in a player's hand. If a player casts a split card with fuse from their hand, the player may choose to cast both halves of that split card rather than choose one half. This choice is made before putting the split card with fuse onto the stack. The resulting spell is a fused split spell.

702.102b A fused split spell has the combined characteristics of its two halves. (See rule 709.4.)

702.102c The total cost of a fused split spell includes the mana cost of each half.

702.102d As a fused split spell resolves, the controller of the spell follows the instructions of the left half and then follows the instructions of the right half.

### Supporting Rules (709 — Split Cards)

709.3 A player chooses which half of a split card they are casting before putting it onto the stack.
709.3a Only the chosen half is evaluated to see if it can be cast. Only that half is considered to be put onto the stack.
709.3b While on the stack, only the characteristics of the half being cast exist. The other half's characteristics are treated as though they didn't exist.
709.4 In every zone except the stack, the characteristics of a split card are those of its two halves combined.
709.4b The mana cost of a split card is the combined mana costs of its two halves.
709.4d The characteristics of a fused split spell on the stack are also those of its two halves combined (see rule 702.102, "Fuse").

## Key Edge Cases

- **Hand-only restriction (CR 702.102a)**: Fuse can ONLY be used when casting from hand. Casting from any other zone (graveyard, exile, etc.) means you can only cast one half. This is the most critical enforcement rule.
- **Combined cost (CR 702.102c)**: When fused, pay BOTH halves' mana costs. This is additive — not either/or.
- **Combined characteristics on stack (CR 702.102b, 709.4d)**: A fused spell has the combined characteristics of both halves while on the stack. This is an EXCEPTION to 709.3b (which normally says only one half exists on stack). Relevant for counterspells that check MV, color, etc.
- **Left-then-right resolution order (CR 702.102d)**: Left half's instructions execute first, then right half's. This is mandatory order, not player's choice.
- **Fuse + "without paying mana cost" (ruling 2013-04-15)**: If cast from hand without paying its mana cost (e.g., via cascade or similar), you CAN choose to fuse and cast both halves for free.
- **Partial fizzle with multiple targets (ruling 2013-04-15)**: When a fused spell has targets from both halves, if all targets are illegal the entire spell fizzles. If at least one target is legal, the spell resolves but illegal targets are skipped.
- **Same target for both halves (ruling 2013-04-15)**: You can choose the same object as the target of each half.
- **Multiplayer**: No special multiplayer considerations beyond standard targeting rules.
- **Color identity**: A fused spell on the stack has the combined colors of both halves. This matters for protection checks.
- **Not an alternative cost**: Fuse is a static ability that changes how you cast, not an alternative cost. It does NOT conflict with other alt costs (CR 118.9a). You can theoretically fuse + flashback if a card somehow had both (though no printed card does, and flashback requires graveyard which blocks fuse's hand-only requirement).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (n/a — Fuse is a static ability, no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant and Card Definition Support

#### 1a: KeywordAbility::Fuse

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Fuse` variant (unit variant, no N parameter)
**Pattern**: Follow `KeywordAbility::Fabricate(u32)` at line ~1221, add after it
**Discriminant**: 133 (Fabricate = 132 is the last)
**CR**: 702.102a — Fuse is a static ability

```
/// CR 702.102: Fuse — if a split card has fuse, the controller may cast
/// both halves from their hand, paying both costs and executing both effects.
/// Discriminant 133.
Fuse,
```

#### 1b: Hash

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Fuse => 133u8.hash_into(hasher)` to the KeywordAbility match
**Pattern**: Follow `KeywordAbility::Fabricate` hash arm (132u8)

#### 1c: AbilityDefinition::Fuse variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Fuse` variant to store the second half's spell data
**Discriminant**: 51 (Soulbond = 50 is the last)
**Pattern**: Follow `AbilityDefinition::Aftermath` at lines 308-334

The Fuse variant stores the RIGHT half's spell data (the card definition's top-level fields describe the LEFT half, same pattern as Aftermath):

```rust
/// CR 702.102: Fuse. The second (right) half of a split card with fuse.
/// When fused, both halves' effects execute at resolution (left first,
/// then right — CR 702.102d).
///
/// The card definition's top-level `name`, `mana_cost`, `types`, and
/// `AbilityDefinition::Spell` describe the left half. This variant
/// stores the right half's data.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Fuse)` for quick
/// presence-checking without scanning all abilities. Discriminant 51.
Fuse {
    /// Name of the right half (e.g., "Tear" for "Wear // Tear").
    name: String,
    /// Mana cost of the right half (added to left half's cost when fused).
    cost: ManaCost,
    /// Card type of the right half (Instant, Sorcery, etc.).
    card_type: CardType,
    /// The spell effect of the right half.
    effect: Effect,
    /// Target requirements for the right half's spell.
    targets: Vec<TargetRequirement>,
},
```

#### 1d: AbilityDefinition hash

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `AbilityDefinition::Fuse { .. }` using discriminant 51
**Pattern**: Follow `AbilityDefinition::Soulbond` hash arm (50u8)

#### 1e: StackObject field

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `was_fused: bool` field to `StackObject`
**Pattern**: Follow `cast_with_aftermath: bool` at line ~151

```rust
/// CR 702.102a: If true, this spell was cast as a fused split spell
/// (both halves from hand). At resolution, both halves' effects execute
/// in order (left first, then right — CR 702.102d). The spell has
/// combined characteristics of both halves (CR 702.102b, 709.4d).
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub was_fused: bool,
```

#### 1f: StackObject hash

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.was_fused.hash_into(hasher)` to the StackObject hash impl
**Pattern**: Follow `self.cast_with_aftermath.hash_into(hasher)` line

#### 1g: CastSpell command field

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `fuse: bool` field to `CastSpell` variant
**Pattern**: Follow `entwine_paid: bool` at line ~238

```rust
/// CR 702.102a: If true, the player is casting both halves of a split card
/// with fuse from their hand. The total cost is the sum of both halves'
/// mana costs (CR 702.102c). At resolution, both halves execute in order
/// (left first, then right — CR 702.102d).
///
/// Validated in `handle_cast_spell`: card must be in hand, must have
/// `KeywordAbility::Fuse`, and `AbilityDefinition::Fuse` must exist.
#[serde(default)]
fuse: bool,
```

#### 1h: Replay viewer view_model.rs

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::Fuse` arm to the keyword display match
**Pattern**: Follow other keyword arms in the file

#### 1i: TUI stack_view.rs

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: No new `StackObjectKind` variant needed — fuse uses the existing `Spell` SOK
**Note**: Only needed if a new SOK is added. Fuse does NOT add a new SOK.

#### 1j: Default `was_fused: false` at all StackObject construction sites

**Action**: Grep for all StackObject construction sites and add `was_fused: false`
**Pattern**: Follow `cast_with_aftermath: false` — every site that sets `cast_with_aftermath` needs `was_fused` too

Sites to update (from Aftermath grep results):
- `crates/engine/src/rules/casting.rs` — main cast path AND all copy/trigger StackObject constructions (~8 sites)
- `crates/engine/src/rules/copy.rs` — copy construction (~2 sites)
- `crates/engine/src/rules/abilities.rs` — triggered/activated ability StackObject construction (~10 sites)
- `crates/engine/src/rules/resolution.rs` — suspend free-cast site (~1 site)

### Step 2: Rule Enforcement (Casting)

**File**: `crates/engine/src/rules/casting.rs`
**CR**: 702.102a, 702.102c

#### 2a: Extract `fuse` flag from CastSpell command

Near line ~90, where `cast_with_aftermath` is extracted, add:

```rust
let casting_with_fuse = fuse;
```

#### 2b: Validate fuse preconditions

After the aftermath validation block (around line ~277-305), add a new fuse validation block:

```rust
// CR 702.102a: Fuse — allowed if fuse is true, card has KeywordAbility::Fuse,
// and the card is being cast from the player's hand.
if casting_with_fuse {
    if !chars.keywords.contains(&KeywordAbility::Fuse) {
        return Err(GameStateError::InvalidCommand(
            "fuse: card does not have the Fuse keyword (CR 702.102a)".into(),
        ));
    }
    // CR 702.102a: Fuse only applies when cast from hand.
    if !casting_from_hand {
        return Err(GameStateError::InvalidCommand(
            "fuse: can only fuse when casting from hand (CR 702.102a)".into(),
        ));
    }
    // Validate the fuse ability definition exists.
    if get_fuse_data(&card_id, &state.card_registry).is_none() {
        return Err(GameStateError::InvalidCommand(
            "fuse: card has Fuse keyword but no AbilityDefinition::Fuse defined".into(),
        ));
    }
}
```

#### 2c: Mutual exclusion with alternative costs

Fuse is NOT an alternative cost (it's a static ability that changes how you cast). However, since fuse requires casting from hand, and some alt costs require other zones (flashback from graveyard, aftermath from graveyard), they are naturally mutually exclusive. Still, add an explicit guard:

```rust
if casting_with_fuse && alt_cost.is_some() {
    // Fuse requires casting from hand. Most alt costs that change zone
    // (flashback, aftermath, etc.) are incompatible. For safety, reject all.
    return Err(GameStateError::InvalidCommand(
        "cannot combine fuse with an alternative cost (CR 702.102a: from hand only)".into(),
    ));
}
```

#### 2d: Combined mana cost

In the mana cost calculation section (around line ~1704), add a fuse cost path:

```rust
} else if casting_with_fuse {
    // CR 702.102c: The total cost of a fused split spell includes the mana cost of each half.
    let fuse_data = get_fuse_data(&card_id, &state.card_registry);
    match fuse_data {
        Some(right_cost) => {
            let mut total = mana_cost.unwrap_or_default();
            total.white += right_cost.white;
            total.blue += right_cost.blue;
            total.black += right_cost.black;
            total.red += right_cost.red;
            total.green += right_cost.green;
            total.generic += right_cost.generic;
            total.colorless += right_cost.colorless;
            Some(total)
        }
        None => {
            return Err(GameStateError::InvalidCommand(
                "card has Fuse keyword but no fuse cost defined".into(),
            ));
        }
    }
}
```

#### 2e: Timing validation for fused spells

A fused spell contains both halves. For timing, the fused spell should be castable at instant speed if EITHER half is an instant (or has Flash). Check the right half's card type:

```rust
// CR 702.102b + 709.4d: A fused spell has combined characteristics.
// If either half is an instant, the fused spell can be cast at instant speed.
let is_instant_speed = if casting_with_fuse {
    let right_type = get_fuse_card_type(&card_id, &state.card_registry);
    chars.card_types.contains(&CardType::Instant)
        || right_type == Some(CardType::Instant)
        || chars.keywords.contains(&KeywordAbility::Flash)
} else if casting_with_aftermath {
    // existing aftermath logic...
```

Actually, more carefully: the fused spell is a single spell with combined types. If BOTH halves are instants, the spell is an instant. If one is instant and one is sorcery, the combined spell has both types (709.4c). The key question is: does having Instant in the combined types make it castable at instant speed? Per CR 304.1 (sorceries are sorcery speed) and 303.1 (instants are instant speed), a spell that has both types follows... the more permissive rule. Per 709.4c, a fused spell has both types. In practice, all existing Fuse cards have both halves as instants or both as sorceries (Turn // Burn is Instant // Instant, Wear // Tear is Instant // Instant). But for correctness, if either half is Instant, the fused spell should be castable at instant speed.

#### 2f: Target merging for fused spells

Fused spells can have targets from BOTH halves. The CastSpell `targets` vec must contain targets for both halves. At cast time, validate targets against both halves' requirements.

In the target validation section, when `casting_with_fuse`:
- The left half's `TargetRequirement` list comes from `AbilityDefinition::Spell.targets`
- The right half's `TargetRequirement` list comes from `AbilityDefinition::Fuse.targets`
- Targets in the `targets` vec are ordered: left half targets first, then right half targets

#### 2g: Record fuse state on StackObject

At the StackObject construction (around line ~2900):

```rust
was_fused: casting_with_fuse,
```

#### 2h: Helper function

Add `get_fuse_data` and `get_fuse_card_type` helper functions, following the pattern of `get_aftermath_cost` and `get_aftermath_card_type` (lines ~3616-3660):

```rust
/// CR 702.102c: Look up the fuse (right half) cost from the card's `AbilityDefinition`.
fn get_fuse_data(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Fuse { cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

fn get_fuse_card_type(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<CardType> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Fuse { card_type, .. } = a {
                    Some(card_type.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

### Step 3: Rule Enforcement (Resolution)

**File**: `crates/engine/src/rules/resolution.rs`
**CR**: 702.102d

At resolution (around line ~161-192 in the spell effect dispatch), add a fuse path:

When `stack_obj.was_fused` is true:
1. Get the left half's `AbilityDefinition::Spell.effect` (normal path)
2. Get the right half's `AbilityDefinition::Fuse.effect`
3. Execute left effect first, then right effect (CR 702.102d)

```rust
if stack_obj.was_fused {
    // CR 702.102d: Execute left half first, then right half.
    let left_effect = def.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Spell { effect, .. } = a {
            Some(effect.clone())
        } else {
            None
        }
    });
    let right_effect = def.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Fuse { effect, .. } = a {
            Some(effect.clone())
        } else {
            None
        }
    });
    // Execute left, then right
    if let Some(eff) = left_effect {
        // execute with left-half targets
        state = execute_effect(state, &eff, &ctx)?;
    }
    if let Some(eff) = right_effect {
        // execute with right-half targets
        state = execute_effect(state, &eff, &ctx)?;
    }
    (None, None) // skip normal spell dispatch
} else if stack_obj.cast_with_aftermath {
    // existing aftermath path...
}
```

**Important**: Target splitting. The left half targets are indices 0..left_count, right half targets are indices left_count..total. The `EffectContext` must have the correct targets for each half's execution. This may require splitting `ctx.targets` and creating separate contexts for each half.

### Step 4: Trigger Wiring

**n/a** — Fuse is a static ability. No triggers involved.

### Step 5: Replay Harness Action

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `cast_spell_fuse` action type
**Pattern**: Follow `cast_spell_entwine` at line ~1047 area

The harness action sets `fuse: true` on CastSpell. Card must be in hand. Targets include targets for both halves.

```rust
"cast_spell_fuse" => {
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
        alt_cost: None,
        escape_exile_cards: vec![],
        retrace_discard_land: None,
        jump_start_discard: None,
        prototype: false,
        bargain_sacrifice: None,
        emerge_sacrifice: None,
        casualty_sacrifice: None,
        assist_player: None,
        assist_amount: 0,
        replicate_count: 0,
        splice_cards: vec![],
        entwine_paid: false,
        escalate_modes: 0,
        devour_sacrifices: vec![],
        fuse: true,
        modes_chosen: vec![],
    })
}
```

### Step 6: Unit Tests

**File**: `crates/engine/tests/fuse.rs`
**Tests to write**:

1. **`test_fuse_basic_both_halves_execute`** — Cast a fuse card from hand with `fuse: true`. Verify both halves' effects execute (left first, then right). Assert mana paid = sum of both costs.
   - CR 702.102a, 702.102c, 702.102d

2. **`test_fuse_single_half_cast`** — Cast a fuse card from hand with `fuse: false`. Verify only the left half's effect executes and only the left half's cost is paid.
   - CR 709.3, 709.3a

3. **`test_fuse_from_hand_only`** — Attempt to cast a fuse card with `fuse: true` from a zone other than hand (e.g., graveyard). Verify it is rejected with an appropriate error.
   - CR 702.102a

4. **`test_fuse_no_keyword_rejected`** — Attempt `fuse: true` on a card without `KeywordAbility::Fuse`. Verify rejection.

5. **`test_fuse_combined_mana_cost`** — Verify that the total mana paid for a fused spell equals the sum of both halves' mana costs.
   - CR 702.102c

6. **`test_fuse_resolution_order_left_then_right`** — Create a fuse card where left half deals damage and right half gains life. Verify damage happens first (left), then life gain (right).
   - CR 702.102d

7. **`test_fuse_with_targets_both_halves`** — Cast a fused spell where both halves have targets. Verify targets are validated for both halves and effects apply to correct targets.

8. **`test_fuse_alt_cost_rejected`** — Attempt `fuse: true` combined with an alt cost (e.g., `alt_cost: Some(AltCostKind::Flashback)`). Verify rejection.

**Pattern**: Follow tests in `crates/engine/tests/entwine.rs` and `crates/engine/tests/aftermath.rs` for structure.

### Step 7: Card Definition (later phase)

**Suggested card**: Wear // Tear (Dragon's Maze)
- Wear: {1}{R} Instant — Destroy target artifact.
- Tear: {W} Instant — Destroy target enchantment.
- Fuse (You may cast one or both halves of this card from your hand.)

Simple, clean effects. Both halves target different permanent types. Good for testing target splitting.

**Alternative card**: Turn // Burn ({2}{U} / {1}{R})
- Turn: Target creature loses all abilities and becomes a red Weird with base power and toughness 0/1 until end of turn.
- Burn: Burn deals 2 damage to target creature or player.

More complex (Turn involves layers), but good for edge case testing.

**Card lookup**: use `card-definition-author` agent for `Wear // Tear`.

### Step 8: Game Script (later phase)

**Suggested scenario**: "Wear // Tear fused destroys both artifact and enchantment"
- P1 has Wear // Tear in hand, sufficient mana ({1}{R}{W})
- P2 has an artifact and an enchantment on battlefield
- P1 casts Wear // Tear fused, targeting both
- Assert: both artifact and enchantment are destroyed

**Subsystem directory**: `test-data/generated-scripts/stack/`

## Discriminant Assignments

| Type | Name | Discriminant | Notes |
|------|------|-------------|-------|
| KeywordAbility | Fuse | 133 | Unit variant (no N) |
| AbilityDefinition | Fuse { .. } | 51 | Stores right half data |
| StackObjectKind | (none) | — | Fuse uses existing Spell SOK |
| Effect | (none) | — | No new Effect variant needed |
| GameEvent | (none) | — | No new event needed |

## Interactions to Watch

- **Aftermath and Fuse on the same card**: No printed card has both. If one did, they'd conflict (Aftermath casts from graveyard, Fuse requires hand). The mutual exclusion check should handle this.
- **Split card name resolution in harness**: The harness `card_name_to_id` strips " // " for split cards (line ~1680 in replay_harness.rs). Fuse cards follow the same pattern.
- **Copy effects on fused spells**: When copying a fused spell on the stack, the copy should also be fused (it has combined characteristics per 709.4d). The `was_fused` flag should propagate. However, copies are "not cast" so `is_copy: true` overrides. Check `copy.rs` to ensure `was_fused` is propagated for copies.
- **Counter-spell interaction**: A fused spell is one spell. Countering it counters both halves.
- **MV of fused spell on stack**: Per 702.102b and 709.4d, the MV = sum of both halves' MVs. This matters for cards like Spell Blast. The engine should already handle this if the mana cost is set correctly on the stack object.
- **Target legality at resolution**: Per ruling 2013-04-15, if all targets of the fused spell are illegal, the whole spell fizzles. If at least one target is legal across either half, the spell resolves but illegal targets are skipped. This is standard partial fizzle (CR 608.2b) applied to the combined spell.
- **`was_fused` default in all StackObject construction sites**: Must add `was_fused: false` to every site that constructs a StackObject. There are ~15+ sites across casting.rs, copy.rs, abilities.rs, resolution.rs.

## File Modification Summary

| File | Changes |
|------|---------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Fuse` (disc 133) |
| `crates/engine/src/state/hash.rs` | Hash arms for KW::Fuse (133u8), AbilDef::Fuse (51u8), StackObject.was_fused |
| `crates/engine/src/state/stack.rs` | Add `was_fused: bool` field |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Fuse { .. }` (disc 51) |
| `crates/engine/src/rules/casting.rs` | Fuse validation, cost calculation, timing, helpers |
| `crates/engine/src/rules/resolution.rs` | Fuse resolution: left effect then right effect |
| `crates/engine/src/rules/command.rs` | Add `fuse: bool` to CastSpell |
| `crates/engine/src/rules/copy.rs` | Add `was_fused: false` to copy StackObject sites |
| `crates/engine/src/rules/abilities.rs` | Add `was_fused: false` to ability StackObject sites |
| `crates/engine/src/testing/replay_harness.rs` | Add `cast_spell_fuse` action type |
| `tools/replay-viewer/src/view_model.rs` | Add KW::Fuse arm |
| `crates/engine/tests/fuse.rs` | 8 unit tests |
