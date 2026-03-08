# Ability Plan: Forage

**Generated**: 2026-03-07
**CR**: 701.61
**Priority**: P4
**Similar abilities studied**: Escape (graveyard exile cost in `casting.rs`), Scavenge (activated ability from graveyard in `abilities.rs`), Outlast (activated ability expansion in `replay_harness.rs`), Bargain (sacrifice-a-permanent cost on `CastSpell`)

## CR Rule Text

> **701.61.** Forage
>
> **701.61a** To forage means "Exile three cards from your graveyard or sacrifice a Food."

That is the complete rule. Forage is a keyword action (section 701), not a keyword ability (section 702). It defines a composite cost with player choice between two options.

## Key Edge Cases

- **Player choice**: The player must choose ONE of the two options (exile 3 OR sacrifice Food), not both. If neither option is available (fewer than 3 cards in graveyard AND no Food you control), forage cannot be performed and the ability cannot be activated.
- **Food is an artifact subtype, not a creature type** (ruling 2024-11-08). Any artifact with `SubType("Food")` qualifies -- not just Food tokens. E.g., Heaped Harvest is an Artifact - Food that counts.
- **Cannot sacrifice a Food to pay multiple costs** (ruling 2024-11-08). A Food sacrificed to forage cannot simultaneously be sacrificed for another cost.
- **Exile 3 cards**: The 3 cards must be from the activating player's graveyard. Any cards qualify (not restricted by type). The player chooses which 3.
- **Forage appears as an activated ability cost**: Cards like Camellia, the Seedmiser use "{2}, Forage: [effect]". The mana and forage are both part of the activation cost.
- **Forage can also appear as an additional cost on spells** or as a one-shot effect instruction, though the Bloomburrow cards primarily use it as an activated ability cost.
- **No triggers from forage itself**: Forage is a cost action, not an effect that generates triggers. However, sacrificing a Food will trigger "whenever you sacrifice a Food" abilities (e.g., Camellia's own triggered ability), and exiling cards from the graveyard may trigger relevant abilities.
- **Multiplayer**: No special multiplayer considerations. Each player forages from their own resources.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant (keyword action -- no KeywordAbility variant needed)
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (N/A -- forage itself is not a trigger)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Cost Infrastructure -- Add `forage: bool` to `ActivationCost`

**No KeywordAbility variant is needed.** Forage is a keyword action (CR 701.x), not a keyword ability (CR 702.x). Per `gotchas-infra.md`: "Keyword actions (Surveil, Scry, etc.) are Effects, NOT KeywordAbility enum variants."

However, forage is primarily a **cost**, not an effect. The implementation needs:

#### Step 1a: Add `forage` field to `ActivationCost`

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `pub forage: bool` field to `ActivationCost` struct (line ~95), with `#[serde(default)]` like `sacrifice_self`.
**Pattern**: Follow `sacrifice_self` field at line 102.

```
#[serde(default)]
pub forage: bool,
```

#### Step 1b: Update `hash.rs` for `ActivationCost`

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.forage.hash_into(hasher);` to the `ActivationCost` `HashInto` impl.
**Pattern**: Find `sacrifice_self.hash_into(hasher)` and add after it.

#### Step 1c: Add `AbilityDefinition::Forage` variant (optional, for card DSL convenience)

**File**: `crates/engine/src/state/types.rs`
**Action**: This step is NOT needed if cards can express forage through `AbilityDefinition::Activated` with `cost.forage: true`. The existing `Activated` variant with `ActivationCost { forage: true, mana_cost: Some(...), ... }` is sufficient.

**Decision**: Do NOT add an `AbilityDefinition::Forage` variant. Use `AbilityDefinition::Activated` with `cost.forage: true`. This avoids a new discriminant and keeps the pattern simple. Camellia's forage ability is just an activated ability with a forage cost component.

### Step 2: Rule Enforcement -- Handle Forage Cost in `handle_activate_ability`

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add forage cost payment logic after the sacrifice-self block (after line ~336) and before target validation (line ~338).

**CR**: 701.61a -- "Exile three cards from your graveyard or sacrifice a Food."

The forage cost handler must:

1. **Check availability**: Count cards in player's graveyard (excluding the source if it was just sacrificed) and count Food artifacts the player controls on the battlefield.
2. **Deterministic choice** (for M9.5 -- interactive choice deferred to M10+):
   - If player has 3+ cards in graveyard AND controls a Food: deterministic fallback should sacrifice a Food (simpler, fewer state changes).
   - If player has 3+ cards in graveyard but no Food: exile 3 cards from graveyard.
   - If player has fewer than 3 cards in graveyard but controls a Food: sacrifice Food.
   - If neither: return `Err(GameStateError::InvalidCommand("cannot forage: insufficient resources"))`.
3. **Execute the chosen option**:
   - **Sacrifice Food**: Find a Food artifact on the battlefield controlled by the player (deterministic: smallest ObjectId). Move it to graveyard. Emit `CreatureDied` or `PermanentDestroyed` event as appropriate (Food tokens are artifacts, not creatures, so `PermanentDestroyed`). Also emit `PermanentSacrificed` if that event exists.
   - **Exile 3 from graveyard**: Find 3 cards in player's graveyard (deterministic: smallest ObjectId order). Move each to exile. Emit appropriate zone-change events.

**Pattern**: Follow the sacrifice-self cost block at lines 308-336 for the Food sacrifice path. Follow `apply_escape_exile_cost` in `casting.rs` (line ~2757) for the graveyard exile path.

**Important**: The forage cost is paid at activation time (CR 602.2 -- costs are paid before the ability goes on the stack), just like tap and mana costs.

#### Step 2a: Add `Command::ActivateForage` or extend `ActivateAbility`

**Decision**: Do NOT add a new Command variant. The existing `Command::ActivateAbility` is sufficient. The `forage: bool` on `ActivationCost` tells `handle_activate_ability` to process the forage cost. The player's choice (exile vs sacrifice) can be specified via an additional field on the command OR use deterministic fallback.

For now (M9.5), use deterministic fallback. The handler in `abilities.rs` reads `ability_cost.forage` and performs the cost automatically.

#### Step 2b: Harness support for forage

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In `enrich_spec_from_def`, detect `AbilityDefinition::Activated` entries where `cost.forage: true` and expand them into `ActivatedAbility` on the `ObjectSpec`. The forage field on `ActivationCost` will carry through naturally since `enrich_spec_from_def` already copies `AbilityDefinition::Activated` into `activated_abilities`.

No new harness action type is needed -- the standard `activate_ability` action with `ability_index` works. The forage cost is paid automatically by `handle_activate_ability`.

### Step 3: Trigger Wiring

**N/A** -- Forage itself is not a triggered ability. However, the *consequences* of foraging (sacrificing a Food, exiling cards from graveyard) will naturally fire existing triggers:
- Sacrificing a Food fires "whenever you sacrifice" triggers via `PermanentDestroyed`/`CreatureSacrificed` events.
- Exiling cards fires zone-change triggers if any exist.

No new trigger wiring is needed.

### Step 4: Unit Tests

**File**: `crates/engine/tests/forage.rs`
**Tests to write**:

- `test_forage_sacrifice_food` -- CR 701.61a: Activate a forage ability by sacrificing a Food token. Verify Food leaves battlefield, ability resolves, effect applies.
- `test_forage_exile_three_from_graveyard` -- CR 701.61a: Activate a forage ability by exiling 3 cards from graveyard (when no Food is available). Verify 3 cards move to exile, ability resolves.
- `test_forage_insufficient_resources` -- CR 701.61a: Cannot forage with fewer than 3 graveyard cards AND no Food. Verify `InvalidCommand` error.
- `test_forage_with_mana_cost` -- CR 602.2: Forage + mana cost paid together (e.g., "{2}, Forage: effect"). Both costs must be satisfied.
- `test_forage_food_is_artifact_subtype` -- Ruling 2024-11-08: A non-token artifact with subtype Food (e.g., Heaped Harvest) can be sacrificed to forage, not just Food tokens.
- `test_forage_sacrifice_triggers_food_sacrifice` -- When foraging by sacrificing a Food, "whenever you sacrifice a Food" triggers should fire.
- `test_forage_prefers_food_when_both_available` -- Deterministic fallback: when both options are available, verify the engine's consistent choice behavior.

**Pattern**: Follow tests for Scavenge in `crates/engine/tests/scavenge.rs` and Outlast in `crates/engine/tests/outlast.rs` for activated ability cost testing patterns.

### Step 5: Card Definition

**Suggested card**: Camellia, the Seedmiser

**Oracle text**:
> Menace
> Other Squirrels you control have menace.
> Whenever you sacrifice one or more Foods, create a 1/1 green Squirrel creature token.
> {2}, Forage: Put a +1/+1 counter on each other Squirrel you control.

**Why**: Camellia uses Forage as an activated ability cost with a clear, testable effect (putting +1/+1 counters on other Squirrels). It also has a triggered ability that fires when sacrificing Foods, which tests the forage-sacrifice-Food interaction path. The menace keyword and lord ability are already implemented.

**Card lookup**: Use `card-definition-author` agent.

**Note on DSL**: The forage ability maps to:
```rust
AbilityDefinition::Activated {
    cost: ActivationCost {
        requires_tap: false,
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        sacrifice_self: false,
        forage: true,
    },
    effect: Effect::AddCounter {
        target: EffectTarget::Controller, // each other Squirrel -- needs ForEach
        counter: CounterType::PlusOnePlusOne,
        count: 1,
    },
    timing_restriction: None,
}
```

The "each other Squirrel you control" effect may need `Effect::ForEach` with a creature-type filter. This is a card-definition complexity, not a forage-specific issue.

### Step 6: Game Script

**Suggested scenario**: Forage activated ability with both payment paths.
**Subsystem directory**: `test-data/generated-scripts/baseline/`

**Scenario description**:
1. Player 1 controls Camellia, the Seedmiser and a Food token on the battlefield, with 3+ cards in graveyard.
2. Player 1 activates Camellia's forage ability by paying {2} and sacrificing the Food token.
3. Verify the Food token is sacrificed, the forage ability resolves, and +1/+1 counters are placed on other Squirrels.
4. Additionally verify that Camellia's "whenever you sacrifice a Food" triggered ability fires and creates a 1/1 Squirrel token.

**Sequence number**: Next available in baseline directory.

## Interactions to Watch

- **Food sacrifice triggers**: Sacrificing a Food to forage should trigger "whenever you sacrifice a Food/artifact" abilities. The sacrifice event path must be the standard sacrifice path (not a special "forage exile" path that bypasses triggers).
- **Graveyard exile interaction with delve/escape/other GY-exile costs**: If a player is foraging by exiling 3 cards, those cards cannot simultaneously be exiled for another cost. This is naturally handled because each card can only be in one zone.
- **Forage as a cost vs. as an effect**: Some future cards might use "Forage" as an effect instruction rather than a cost. The current implementation focuses on the cost path. An `Effect::Forage` variant could be added later if needed, following the Scry/Surveil pattern.
- **`is_phased_in()` filter**: When searching for Food artifacts to sacrifice, must filter for phased-in permanents only (CR 702.26b).
- **Layer-resolved types for Food check**: Use `calculate_characteristics` to check for `SubType("Food")` -- a permanent might gain or lose the Food subtype due to continuous effects.

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/game_object.rs` | Add `forage: bool` to `ActivationCost` |
| `crates/engine/src/state/hash.rs` | Hash the new `forage` field |
| `crates/engine/src/rules/abilities.rs` | Add forage cost payment in `handle_activate_ability` |
| `crates/engine/src/testing/replay_harness.rs` | No changes needed (forage flows through existing `ActivationCost`) |
| `crates/engine/tests/forage.rs` | New test file with 7 tests |
| `tools/replay-viewer/src/view_model.rs` | No changes needed (no new KW or SOK variant) |
| `tools/tui/src/play/panels/stack_view.rs` | No changes needed (no new SOK variant) |

## Discriminant Chain

No new discriminants needed:
- **KeywordAbility**: No variant added (keyword action, not keyword ability). Last = 136 (Discover).
- **AbilityDefinition**: No variant added (using existing `Activated` with `forage: bool`). Last = 53 (CollectEvidence).
- **StackObjectKind**: No variant added (standard `ActivatedAbility` SOK). Last = 51 (BloodrushAbility).
