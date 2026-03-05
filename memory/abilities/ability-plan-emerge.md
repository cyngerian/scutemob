# Ability Plan: Emerge

**Generated**: 2026-03-03
**CR**: 702.119
**Priority**: P4
**Batch**: 6.2
**Effort**: Medium
**Similar abilities studied**: Bargain (CR 702.166, additional-cost sacrifice pattern), Evoke (CR 702.74, alternative-cost with creature sacrifice), Convoke (CR 702.51, cost-reduction-by-MV pattern)

## CR Rule Text

CR 702.119 -- Emerge

> 702.119a Emerge is a keyword that represents two static abilities that function
> while the spell with emerge is on the stack. "Emerge [cost]" means "You may cast
> this spell by paying [cost] and sacrificing a creature" and "The total cost to
> cast this spell is reduced by the sacrificed creature's mana value." Paying a
> spell's emerge cost follows the rules for paying alternative costs in rules
> 601.2b and 601.2f-h.
>
> 702.119b You may sacrifice the creature at the same time you pay the spell's emerge cost.

Note: CR 702.119 is short (only 702.119a and 702.119b). There is no 702.119c+.

## Key Edge Cases

1. **Emerge is an alternative cost (CR 118.9)**: Cannot combine with other alternative
   costs (flashback, evoke, bestow, miracle, etc.). Only one alternative cost per spell.

2. **Cost reduction by sacrificed creature's MV**: The emerge cost is paid, then reduced
   by the sacrificed creature's mana value. The reduction applies to generic mana first,
   then colored mana pips if generic is fully paid. Per CR 601.2f, the total cost is
   determined, and the reduction from the creature's MV is part of that determination.

3. **The sacrifice is part of paying the cost**: The creature is sacrificed as part of
   cost payment (CR 601.2h), not at resolution. The creature leaves the battlefield
   during casting, before the spell even goes on the stack. This means:
   - The sacrificed creature's "dies" triggers will fire
   - The creature is in the graveyard before the emerge spell resolves
   - If the cast is countered, the creature is still gone (costs are non-refundable)

4. **MV reduction can reduce colored costs**: If the emerge cost is {5}{U}{U} and the
   sacrificed creature has MV 7, the reduction of 7 first eliminates the 5 generic,
   then reduces 2 colored pips. The remaining cost would be {0}. If the creature's MV
   exceeds the emerge cost's total, the remaining cost is {0} (cannot go negative).

5. **Creature's MV is checked at sacrifice time**: Use the creature's MV from its
   characteristics at the moment of sacrifice (layer-resolved). A creature with no mana
   cost (e.g., tokens, face-down creatures) has MV 0, providing no reduction.

6. **The sacrificed creature can be ANY creature**: Unlike bargain (artifact/enchantment/token
   only), emerge only requires a creature. Any creature the caster controls works.

7. **Commander interaction**: Casting a spell with emerge from the command zone applies
   commander tax on TOP of the emerge cost (CR 118.9d). The MV reduction from the
   sacrificed creature applies to the emerge cost before tax is added.
   Wait -- actually, per CR 601.2f, the total cost is:
   - Start with emerge cost (alternative base)
   - Subtract sacrificed creature's MV
   - Add commander tax
   - Add additional costs (kicker, etc.)
   So the MV reduction happens to the emerge base, then tax is layered on.

8. **Multiplayer**: No special multiplayer considerations beyond standard Commander rules.
   The sacrificed creature must be controlled by the caster (standard sacrifice rules).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- KeywordAbility::Emerge not yet added
- [ ] Step 2: Rule enforcement -- No emerge handling in casting.rs
- [ ] Step 3: Trigger wiring -- N/A (emerge is a cost, not a trigger)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant + AltCostKind + AbilityDefinition

**Files**:
- `crates/engine/src/state/types.rs` -- add `KeywordAbility::Emerge` after `Bargain` (line ~903)
- `crates/engine/src/state/types.rs` -- add `AltCostKind::Emerge` after `Impending` (line ~112)
- `crates/engine/src/cards/card_definition.rs` -- add `AbilityDefinition::Emerge { cost: ManaCost }` after existing ability variants (after `Impending` at ~line 396)
- `crates/engine/src/state/hash.rs` -- add hash discriminant for `KeywordAbility::Emerge` (discriminant 101, after Bargain=100 at line ~536)
- `crates/engine/src/state/hash.rs` -- add hash discriminant for `AbilityDefinition::Emerge` (discriminant 33, after Impending=32 at line ~3233)

**KeywordAbility::Emerge variant**:
```rust
/// CR 702.119: Emerge [cost] -- alternative cost: pay [cost] and sacrifice a creature.
/// The total cost is reduced by the sacrificed creature's mana value.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The emerge cost is stored in `AbilityDefinition::Emerge { cost }`.
/// The sacrifice target is provided via `CastSpell.emerge_sacrifice`.
Emerge,
```

**AltCostKind::Emerge variant**:
```rust
Emerge,
```

**AbilityDefinition::Emerge variant**:
```rust
/// CR 702.119: Emerge [cost]. The card may be cast by paying this cost and
/// sacrificing a creature (alternative cost, CR 118.9). The total cost is
/// reduced by the sacrificed creature's mana value.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Emerge)` for quick
/// presence-checking without scanning all abilities.
Emerge { cost: ManaCost },
```

**Hash discriminants**: KeywordAbility::Emerge = 101, AbilityDefinition::Emerge = 33.

**Match arms to update** (grep for exhaustive matches on `KeywordAbility` and `AltCostKind`):
- `state/hash.rs`: KeywordAbility match (add disc 101)
- `state/hash.rs`: AbilityDefinition match (add disc 33)
- `state/hash.rs`: AltCostKind is derived Hash (automatic, no manual arm)
- `rules/layers.rs`: check if KeywordAbility match arms are exhaustive (likely wildcard)
- `cards/builder.rs`: check if KeywordAbility match arms are exhaustive

### Step 2: Command Extension

**File**: `crates/engine/src/rules/command.rs`

**Action**: Add `emerge_sacrifice: Option<ObjectId>` field to `Command::CastSpell` variant, after `bargain_sacrifice` (line ~174).

```rust
/// CR 702.119a: The ObjectId of a creature on the battlefield to sacrifice
/// as part of the emerge alternative cost. `None` means not using emerge.
///
/// When `Some`, the identified creature must be:
/// - On the battlefield, controlled by the caster
/// - A creature (by current characteristics)
///
/// The spell's total cost is reduced by the sacrificed creature's mana value.
/// `alt_cost` must be `Some(AltCostKind::Emerge)` when this is `Some`.
#[serde(default)]
emerge_sacrifice: Option<ObjectId>,
```

**Also update**: Every `Command::CastSpell { ... }` construction site throughout the codebase
must include `emerge_sacrifice: None` (or the appropriate value). Grep for `Command::CastSpell`
to find all sites. Key files:
- `rules/engine.rs` (command dispatch)
- `rules/casting.rs` (function signature)
- `testing/replay_harness.rs` (all `cast_spell*` action types)
- `rules/suspend.rs` (free cast from suspend)
- `rules/resolution.rs` (cascade free cast)
- All test files that construct `Command::CastSpell`

### Step 3: Rule Enforcement in casting.rs

**File**: `crates/engine/src/rules/casting.rs`

#### 3a: Function signature update

Add `emerge_sacrifice: Option<ObjectId>` parameter to `handle_cast_spell()` (after `bargain_sacrifice` at line ~69).

#### 3b: Alt-cost boolean derivation

Add at line ~84 (after `cast_with_impending`):
```rust
let cast_with_emerge = alt_cost == Some(AltCostKind::Emerge);
```

#### 3c: Alt-cost mutual exclusion validation

Add a new step (Step 1n, after Step 1m for Impending at ~line 1080) following the
established pattern. Emerge is an alternative cost, so it must be mutually exclusive
with ALL other alternative costs:
- flashback, evoke, bestow, madness, miracle, escape, foretell, overload, retrace,
  jump-start, aftermath, dash, blitz, plot, impending

Also validate that the card has `AbilityDefinition::Emerge { cost }` via a
`get_emerge_cost()` helper function.

#### 3d: Emerge sacrifice validation

Add a validation block (similar to the Bargain validation at lines ~1368-1413) BEFORE
the casting window validation. When `emerge_sacrifice` is `Some`:

1. Validate `alt_cost == Some(AltCostKind::Emerge)` (sacrifice without emerge flag is invalid)
2. Validate the sacrifice target is on the battlefield
3. Validate the sacrifice target is controlled by the caster
4. Validate the sacrifice target is a creature (by layer-resolved characteristics)
5. Compute the creature's MV from its `mana_cost` (using `ManaCost::mana_value()`)
   - Use `calculate_characteristics()` to get the layer-resolved characteristics
   - If `mana_cost` is `None` (tokens, etc.), MV = 0

Also validate the inverse: if `alt_cost == Some(AltCostKind::Emerge)`, then
`emerge_sacrifice` MUST be `Some` (emerge requires sacrificing a creature).

#### 3e: Base cost selection

In the `base_cost_before_tax` chain (lines ~1109-1202), add an `else if` for emerge
BEFORE the final `else` fallback:

```rust
} else if casting_with_emerge {
    // CR 702.119a: Pay emerge cost, reduced by sacrificed creature's MV.
    let emerge_cost = get_emerge_cost(&card_id, &state.card_registry);
    if let (Some(cost), Some(sac_mv)) = (emerge_cost, emerge_creature_mv) {
        Some(reduce_cost_by_mv(&cost, sac_mv))
    } else {
        return Err(GameStateError::InvalidCommand(
            "emerge: card has Emerge keyword but no emerge cost defined".into(),
        ));
    }
}
```

#### 3f: Cost reduction helper function

Add a new function `reduce_cost_by_mv(cost: &ManaCost, mv: u32) -> ManaCost`:

```rust
/// CR 702.119a: Reduce a mana cost by a creature's mana value.
/// Reduces generic mana first, then colored pips (WUBRG order) if generic is exhausted.
/// The cost cannot go below zero in any component.
fn reduce_cost_by_mv(cost: &ManaCost, mv: u32) -> ManaCost {
    let mut reduced = cost.clone();
    let mut remaining_reduction = mv;

    // Reduce generic first
    let generic_reduction = remaining_reduction.min(reduced.generic);
    reduced.generic -= generic_reduction;
    remaining_reduction -= generic_reduction;

    // Then reduce colorless
    let colorless_reduction = remaining_reduction.min(reduced.colorless);
    reduced.colorless -= colorless_reduction;
    remaining_reduction -= colorless_reduction;

    // Then reduce colored pips (WUBRG order)
    for field in [&mut reduced.white, &mut reduced.blue, &mut reduced.black,
                  &mut reduced.red, &mut reduced.green] {
        let reduction = remaining_reduction.min(*field);
        *field -= reduction;
        remaining_reduction -= reduction;
        if remaining_reduction == 0 { break; }
    }

    reduced
}
```

**Note on reduction order**: CR 601.2f says the player determines the total cost. The
reduction applies to the total emerge cost. Generic mana is reduced first, then colored
costs. This matches the standard MTG cost-reduction convention used by similar effects.

#### 3g: Sacrifice execution

Add a block (similar to Bargain sacrifice at lines ~1750-1763) that performs the actual
sacrifice during cost payment. This should happen AFTER mana cost determination but
BEFORE moving the spell to the stack:

```rust
// CR 702.119a / CR 601.2f-h: Pay the emerge cost -- sacrifice a creature.
// The sacrifice is part of cost payment (CR 601.2h).
if let Some(sac_id) = emerge_sacrifice_id {
    let sac_owner = state.object(sac_id)?.owner;
    let (new_sac_id, _) = state.move_object_to_zone(sac_id, ZoneId::Graveyard(sac_owner))?;
    events.push(GameEvent::ObjectPutInGraveyard {
        player,
        object_id: sac_id,
        new_grave_id: new_sac_id,
    });
}
```

#### 3h: StackObject flag

No new `was_emerged` flag is needed on StackObject or GameObject. Unlike Evoke (which
needs a flag for the ETB sacrifice trigger) or Bargain (which needs a flag for
`Condition::WasBargained`), Emerge has no post-cast effects that depend on knowing
whether emerge was used. The alternative cost is fully handled at cast time.

If a future card needs "if this spell's emerge cost was paid" conditional logic, a
`was_emerged` flag can be added at that time. For now, skip it -- YAGNI.

#### 3i: get_emerge_cost() helper

Add at the end of `casting.rs` (near other `get_X_cost()` functions at ~line 2300+):

```rust
/// CR 702.119a: Look up the emerge cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Emerge { cost }`, or `None`
/// if the card has no definition or no emerge ability defined.
fn get_emerge_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Emerge { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

### Step 4: Replay Harness Support

**File**: `crates/engine/src/testing/replay_harness.rs`

Add a `"cast_spell_emerge"` action type to `translate_player_action()`. Follow the
pattern of `"cast_spell_bargain"` (lines ~956-990) and `"cast_spell_evoke"` (lines ~330-350).

The action needs:
- `card_name` -- the spell being cast (find in hand)
- `emerge_sacrifice` -- name of the creature to sacrifice (find on battlefield)
- `targets` -- any targets for the spell

```rust
"cast_spell_emerge" => {
    let card_id = find_in_hand(state, player, card_name?)?;
    let target_list = resolve_targets(targets, state, players);
    let emerge_sac_name = action.get("emerge_sacrifice")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "cast_spell_emerge requires emerge_sacrifice field".to_string())?;
    let emerge_sac_id = find_on_battlefield(state, player, emerge_sac_name)?;
    Some(Command::CastSpell {
        player,
        card: card_id,
        targets: target_list,
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: Some(AltCostKind::Emerge),
        escape_exile_cards: vec![],
        retrace_discard_land: None,
        jump_start_discard: None,
        prototype: false,
        bargain_sacrifice: None,
        emerge_sacrifice: Some(emerge_sac_id),
    })
}
```

**Also update**: All other `"cast_spell*"` arms must include `emerge_sacrifice: None` in their
`Command::CastSpell` construction. There are ~20 such arms (cast_spell, cast_spell_evoke,
cast_spell_bestow, cast_spell_miracle, cast_spell_escape, cast_spell_foretell,
cast_spell_plot, cast_spell_overload, etc.).

### Step 5: TUI stack_view.rs (No new StackObjectKind needed)

Emerge does NOT introduce a new `StackObjectKind` variant (unlike Evoke which has
`EvokeSacrificeTrigger`). Emerge has no triggered abilities or delayed triggers. The
sacrifice happens during cost payment, and the spell on the stack is a normal `Spell`.

Therefore, `tools/tui/src/play/panels/stack_view.rs` does NOT need any changes for Emerge.

### Step 6: Unit Tests

**File**: `crates/engine/tests/emerge.rs`

**Tests to write**:

1. `test_emerge_basic_sacrifice_reduces_cost` -- CR 702.119a
   - Set up a creature with MV 3 on battlefield, an emerge spell with emerge cost {5}{U}{U}
   - Cast the emerge spell sacrificing the creature
   - Verify: creature is sacrificed (in graveyard), spell resolves, player paid only {2}{U}{U}
   - Pattern: follow `test_bargain_basic_instant_with_sacrifice`

2. `test_emerge_sacrifice_token_mv_zero` -- CR 702.119a + token MV
   - Set up a creature token (MV = 0) on battlefield
   - Cast emerge spell sacrificing the token
   - Verify: token is sacrificed, but no cost reduction (full emerge cost paid)

3. `test_emerge_sacrifice_high_mv_creature` -- CR 702.119a cost floor
   - Set up a creature with MV 10, emerge spell with emerge cost {5}{U}{U} (total MV 7)
   - Cast the emerge spell sacrificing the creature
   - Verify: cost is reduced to {0} (cannot go negative), spell resolves for free

4. `test_emerge_sacrifice_must_be_creature` -- CR 702.119a
   - Set up an artifact (non-creature) on battlefield
   - Attempt to cast emerge spell sacrificing the artifact
   - Verify: error (InvalidCommand), artifact is not a creature

5. `test_emerge_sacrifice_must_be_own_creature` -- CR 702.119a
   - Set up a creature controlled by opponent
   - Attempt to cast emerge spell sacrificing opponent's creature
   - Verify: error (InvalidCommand)

6. `test_emerge_without_sacrifice_fails` -- CR 702.119a
   - Attempt to cast with `alt_cost: Some(AltCostKind::Emerge)` but `emerge_sacrifice: None`
   - Verify: error (emerge requires a sacrifice)

7. `test_emerge_mutual_exclusion_with_flashback` -- CR 118.9a
   - Attempt to combine emerge with flashback
   - Verify: error (cannot combine alternative costs)

8. `test_emerge_mutual_exclusion_with_evoke` -- CR 118.9a
   - Attempt to combine emerge with evoke
   - Verify: error (cannot combine alternative costs)

9. `test_emerge_no_keyword_rejects_sacrifice` -- engine validation
   - Provide `alt_cost: Some(AltCostKind::Emerge)` for a spell without the Emerge keyword
   - Verify: error

10. `test_emerge_normal_cast_without_emerge` -- negative test
    - Cast an emerge spell by paying its normal mana cost (alt_cost: None, emerge_sacrifice: None)
    - Verify: spell resolves normally at full cost, no creature sacrificed

**Pattern**: Follow `crates/engine/tests/bargain.rs` structure:
- Helper functions: `p()`, `find_object()`, `find_object_in_zone()`, `pass_all()`
- Synthetic card definitions for emerge spells
- `GameStateBuilder` with manual mana pool setup
- CR citations on every test

### Step 7: Card Definition (later phase)

**Suggested card**: Elder Deep-Fiend
- Oracle: Emerge {5}{U}{U}; Flash; When you cast this spell, tap up to four target permanents.
- Types: Creature -- Eldrazi Octopus
- P/T: 5/6
- MV: 8 (normal cost: {8})
- Uses: Emerge prominently; flash for surprise tempo plays; cast trigger for tapping

**Alternative**: Lashweed Lurker
- Oracle: Emerge {5}{G}{U}; When you cast this spell, you may exile target nonland permanent. If you do, its controller returns it under their control.
- Types: Creature -- Eldrazi Horror
- P/T: 5/4
- MV: 7

### Step 8: Game Script (later phase)

**Suggested scenario**: "Emerge creature cast by sacrificing a smaller creature"
- Player casts Elder Deep-Fiend with emerge, sacrificing a 3-MV creature
- Taps opponent's creatures on cast trigger
- Elder Deep-Fiend resolves and enters battlefield

**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

1. **Cost reduction order**: Emerge cost reduction happens BEFORE commander tax and before
   convoke/improvise/delve further reductions. The pipeline order in `casting.rs` is:
   - Base = emerge cost
   - Subtract sacrificed creature's MV
   - Add commander tax (CR 118.9d)
   - Add kicker cost (if applicable)
   - Apply convoke reduction
   - Apply improvise reduction
   - Apply delve reduction
   - Pay remaining mana

2. **Emerge + Convoke interaction**: A player could theoretically cast an emerge creature,
   sacrifice one creature for emerge's cost reduction, and tap other creatures for convoke.
   This is legal -- emerge consumes one creature (via sacrifice), convoke taps others.

3. **Creature dies during casting**: The sacrificed creature "dies" as part of cost payment.
   Death triggers fire. If the emerge spell is then countered, the creature is still dead --
   costs are not refundable (CR 601.2h).

4. **Layer-resolved MV**: The sacrificed creature's MV must be calculated from its
   layer-resolved characteristics (via `calculate_characteristics()`), not from the
   card registry. This handles cases where continuous effects have changed the creature's
   mana cost (e.g., Mycosynth Lattice).

5. **No post-cast effects**: Unlike Evoke (ETB sacrifice trigger), Dash (haste + end-step
   return), or Bargain (conditional effects), Emerge has no resolution-time or post-cast
   behavior. The entire mechanic is handled at cast time. No new `was_emerged` flag needed
   on StackObject or GameObject unless a card requires "if this spell's emerge cost was paid"
   conditional logic.

## Files Changed Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Emerge`, `AltCostKind::Emerge` |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Emerge { cost }` |
| `crates/engine/src/state/hash.rs` | Add hash discriminants (KW=101, AbilDef=33) |
| `crates/engine/src/rules/command.rs` | Add `emerge_sacrifice: Option<ObjectId>` to CastSpell |
| `crates/engine/src/rules/casting.rs` | Alt-cost validation, sacrifice validation, cost reduction, `get_emerge_cost()`, `reduce_cost_by_mv()` |
| `crates/engine/src/rules/engine.rs` | Pass `emerge_sacrifice` through to `handle_cast_spell` |
| `crates/engine/src/testing/replay_harness.rs` | Add `"cast_spell_emerge"` action type; add `emerge_sacrifice: None` to all other arms |
| `crates/engine/tests/emerge.rs` | New test file with ~10 tests |
| `crates/engine/src/cards/helpers.rs` | No change needed (Emerge types are already importable) |

**Note**: `tools/tui/src/play/panels/stack_view.rs` does NOT need changes -- no new StackObjectKind.
