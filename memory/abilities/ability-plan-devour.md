# Ability Plan: Devour

**Generated**: 2026-03-06
**CR**: 702.82
**Priority**: P4
**Similar abilities studied**: Amplify (702.38) in `resolution.rs:679-777`, `lands.rs:217-296`, `tests/amplify.rs`; Bloodthirst (702.54) in `resolution.rs:779-837`, `lands.rs:299-350`

## CR Rule Text

702.82. Devour

[702.82a] Devour is a static ability. "Devour N" means "As this object enters, you may sacrifice any number of creatures. This permanent enters with N +1/+1 counters on it for each creature sacrificed this way."

[702.82b] Some objects have abilities that refer to the number of creatures the permanent devoured. "It devoured" means "sacrificed as a result of its devour ability as it entered the battlefield."

[702.82c] Devour [quality] is a variant of devour. "Devour [quality] N" means "As this object enters, you may sacrifice any number of [quality] permanents. This permanent enters with N +1/+1 counters on it for each permanent sacrificed this way."

## Key Edge Cases

- **Optional sacrifice**: "you may sacrifice any number of creatures" -- zero is valid (no sacrifice, no counters). The player chooses.
- **Only creatures already on the battlefield**: Cannot devour creatures entering at the same time. Cannot devour itself. (Ruling on Mycoloth, Thorn-Thrash Viashino, etc.)
- **Multiple Devour instances**: CR 702.82c implies variants work separately. Each instance processes its own sacrifice pool independently.
- **Sacrifice, not reveal**: Unlike Amplify (reveal from hand, non-destructive), Devour sacrifices creatures (destructive -- they go to the graveyard). This means the sacrificed creatures trigger "when this creature dies" abilities.
- **Thromok the Insatiable**: "Devour X, where X is the number of creatures devoured" -- X squared counters. This is a special variant we do NOT need to support in B10 (it's a single card with a unique formula).
- **Token with Devour entering**: Works the same as a card with Devour. The token may devour the creature that created it (e.g., Dragon Broodmother's token can devour Dragon Broodmother itself).
- **Multiple Devour creatures entering simultaneously**: Each can devour, but a creature can only be devoured by one of them. All sacrifices happen at the same time. (Ruling 2008-10-01 on Mycoloth.)
- **Multiplayer**: Devour only sacrifices creatures the controller controls (implied by "you may sacrifice"). Opponents' creatures are not eligible.
- **CR 702.82b "it devoured"**: Some cards reference the number of creatures devoured. Need `creatures_devoured: u32` on `GameObject` to track this count.
- **Choice happens at resolution**: "If you cast this as a spell, you choose how many and which creatures to devour as part of the resolution of that spell." (Ruling on Thunder-Thrash Elder.)

## Implementation Approach

Devour differs from Amplify in a critical way: Amplify is non-destructive (reveal), so auto-maximizing is always optimal. Devour is destructive (sacrifice), so auto-sacrificing all creatures is NOT always desirable. The engine needs the player to specify which creatures to sacrifice.

**Chosen approach**: Add `devour_sacrifices: Vec<ObjectId>` to `CastSpell` command (like `convoke_creatures`). The sacrifice IDs are validated at cast time (creatures on battlefield, controlled by caster, no duplicates, not the spell being cast) but the actual sacrifice and counter placement happen at resolution time in `resolution.rs` as an ETB replacement effect. This matches the CR: "you choose how many and which creatures to devour as part of the resolution of that spell."

For the `lands.rs` ETB hook: lands with Devour are not printed in Magic, but the hook is added for consistency (same as Amplify, Bloodthirst). Lands can't be creatures, so `devour_sacrifices` passed from `CastSpell` won't apply -- the lands.rs hook reads from the `StackObject` or handles it as a no-op.

For the harness: Add `cast_spell_devour` action type with `devour_creatures: [name, ...]` array (like `convoke_creatures`).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (n/a -- Devour is a replacement effect, not a trigger)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Devour(u32)` variant after `Bloodthirst(u32)` (line ~1134).
**Doc comment**:
```
/// CR 702.82: Devour N -- "As this object enters, you may sacrifice any number of
/// creatures. This permanent enters with N +1/+1 counters on it for each creature
/// sacrificed this way."
///
/// Static ability / ETB replacement effect (CR 614.1c). Multiple instances work
/// separately (CR 702.82c).
///
/// Discriminant 124.
Devour(u32),
```
**Pattern**: Follow `KeywordAbility::Amplify(u32)` at line 1126 and `KeywordAbility::Bloodthirst(u32)` at line 1134.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm after Bloodthirst (disc 123):
```rust
// Devour (discriminant 124) -- CR 702.82
KeywordAbility::Devour(n) => {
    124u8.hash_into(hasher);
    n.hash_into(hasher);
}
```
**Pattern**: Follow `KeywordAbility::Amplify(n)` at hash.rs line 614-617.

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add TWO match arms:
1. `KeywordAbility::Devour(n) => format!("Devour {n}")` in the keyword display function.
2. No new `StackObjectKind` variant needed (Devour doesn't create stack objects).
**Pattern**: Search for `KeywordAbility::Amplify` in view_model.rs and add after it.

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: No change needed (no new `StackObjectKind` variant).

### Step 2: Command Extension + Rule Enforcement

#### Step 2a: CastSpell Field

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `devour_sacrifices: Vec<ObjectId>` field to `CastSpell` variant, after `escalate_modes`:
```rust
/// CR 702.82a: Creatures on the battlefield to sacrifice as the devour
/// ETB replacement effect resolves. Empty vec = no sacrifice (devour 0).
///
/// When non-empty, each ObjectId must be:
/// - On the battlefield, controlled by the caster
/// - A creature (by current characteristics)
/// - Not duplicated (no ObjectId appears twice)
/// - Not the card being cast (can't devour itself)
///
/// The sacrifice happens at resolution time as an ETB replacement effect,
/// not at cast time. Validated at cast time for early error detection.
#[serde(default)]
devour_sacrifices: Vec<ObjectId>,
```

**File**: `crates/engine/src/rules/casting.rs`
**Action**:
1. Add `devour_sacrifices: Vec<ObjectId>` parameter to `handle_cast_spell` function signature (after `escalate_modes`).
2. Add validation block: verify spell has `KeywordAbility::Devour(_)`, each creature is on battlefield + controlled by caster + is a creature + no duplicates + not the cast card.
3. Store the `devour_sacrifices` on the `StackObject` (new field, see Step 2b).
4. Do NOT sacrifice at cast time -- the sacrifice happens at resolution.

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Pass `devour_sacrifices` from `Command::CastSpell` destructure to `handle_cast_spell` call.

#### Step 2b: StackObject Field

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add field to `StackObject`:
```rust
/// CR 702.82a: Creatures to sacrifice when this permanent enters the battlefield.
/// Populated from CastSpell.devour_sacrifices at cast time; consumed at resolution
/// time in resolution.rs for the Devour ETB replacement.
#[serde(default)]
pub devour_sacrifices: Vec<ObjectId>,
```
**Hash**: Add to hash.rs `StackObject` hash impl.

#### Step 2c: GameObject Field

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `creatures_devoured: u32` field to `GameObject`:
```rust
/// CR 702.82b: The number of creatures this permanent devoured as it entered
/// the battlefield. "It devoured" means "sacrificed as a result of its devour
/// ability as it entered the battlefield."
///
/// Used by abilities that reference the number of creatures devoured
/// (e.g., Mycoloth's upkeep trigger uses +1/+1 counter count, but other
/// cards like Preyseizer Dragon reference devour count directly).
#[serde(default)]
pub creatures_devoured: u32,
```
**Hash**: Add to hash.rs `GameObject` hash impl.
**Init sites**: Initialize to `0` in:
- `crates/engine/src/state/builder.rs` (ObjectSpec conversion)
- `crates/engine/src/effects/mod.rs` (token creation)
- `crates/engine/src/rules/resolution.rs` (any new-object construction)
- Both `move_object_to_zone` sites in `state/mod.rs` (reset on zone change, CR 400.7)

#### Step 2d: ETB Replacement in resolution.rs

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add Devour ETB replacement block after the Bloodthirst block (~line 837). Pattern follows Amplify/Bloodthirst closely:

```
// CR 702.82a: Devour N -- "As this object enters, you may sacrifice any number
// of creatures. This permanent enters with N +1/+1 counters on it for each
// creature sacrificed this way."
// CR 702.82c: Multiple instances work separately.
// CR 614.1c: This is a static/replacement ability, not a triggered ability.
//
// Implementation: consume devour_sacrifices from the StackObject. For each
// Devour(N) instance, multiply N by the number of sacrificed creatures.
// The sacrifice happens HERE (during ETB), not at cast time.
```

Logic:
1. Collect `Devour(n)` instances from the card definition (same pattern as Amplify).
2. If any instances exist AND `stack_obj.devour_sacrifices` is non-empty:
   a. Validate each sacrifice target still exists on battlefield, is a creature, controlled by the entering creature's controller (re-validate at resolution since state may have changed).
   b. For each valid sacrifice: move the creature to its owner's graveyard via `move_object_to_zone`. Emit `GameEvent::CreatureDied` for each (triggers "when a creature dies").
   c. Count the number of successfully sacrificed creatures.
   d. For each Devour(N) instance: add N * sacrifice_count counters.
   e. Set `obj.creatures_devoured = sacrifice_count` on the entering permanent.
3. Emit `GameEvent::CounterAdded` with the total count.

**CR**: 702.82a (sacrifice + counters), 702.82b (creatures_devoured tracking), 702.82c (multiple instances).

#### Step 2e: ETB Hook in lands.rs

**File**: `crates/engine/src/rules/lands.rs`
**Action**: Add Devour ETB hook after the Bloodthirst block (~line 350). Since lands never have Devour and there's no `StackObject` for lands, this is a no-op stub for consistency:

```rust
// CR 702.82a: Devour N -- lands with Devour are not printed in Magic.
// ETB hook exists here for consistency with resolution.rs (gotchas-infra.md).
// No-op: lands have no StackObject to carry devour_sacrifices.
```

#### Step 2f: Replay Harness Action

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `cast_spell_devour` action type handler in `translate_player_action()`:
- Parse `devour_creatures: [name, ...]` array from the action JSON.
- Resolve each name to an ObjectId on the battlefield.
- Build `CastSpell` with `devour_sacrifices` populated, all other fields default.
**Pattern**: Follow `cast_spell_replicate` or `cast_spell_splice` handlers.

#### Step 2g: All CastSpell Call Sites

**Action**: Grep for all `CastSpell {` or `Command::CastSpell` construction sites and add `devour_sacrifices: vec![]` to each:
- `crates/engine/src/rules/casting.rs` (internal construction)
- `crates/engine/src/testing/replay_harness.rs` (all existing `cast_spell` variants)
- `crates/engine/tests/*.rs` (all test files that construct CastSpell)
- `crates/engine/src/rules/copy.rs` (spell copy construction -- copies don't devour)
- Any other sites found by grep.

### Step 3: Trigger Wiring

**N/A** -- Devour is a replacement effect (CR 614.1c), not a triggered ability. No trigger dispatch needed. The sacrificed creatures' death triggers fire via the normal `CreatureDied` event pathway.

### Step 4: Unit Tests

**File**: `crates/engine/tests/devour.rs`
**Tests to write**:

1. `test_devour_basic_one_sacrifice` -- CR 702.82a: Devour 1 creature enters, sacrifice 1 creature, enters with 1 +1/+1 counter. Verify counter on permanent, verify sacrificed creature is in graveyard.

2. `test_devour_multiple_sacrifices` -- CR 702.82a: Devour 1 creature enters, sacrifice 3 creatures, enters with 3 +1/+1 counters.

3. `test_devour_n_multiplier` -- CR 702.82a: Devour 3 creature enters, sacrifice 2 creatures, enters with 6 +1/+1 counters (3 x 2).

4. `test_devour_zero_sacrifice` -- CR 702.82a: Player passes empty `devour_sacrifices`, creature enters with 0 counters (optional sacrifice).

5. `test_devour_no_eligible_creatures` -- CR 702.82a: No other creatures on battlefield, empty `devour_sacrifices`, creature enters normally.

6. `test_devour_only_own_creatures` -- Verify that only creatures controlled by the entering creature's controller can be sacrificed (validation rejects opponent's creatures).

7. `test_devour_cannot_sacrifice_self` -- Verify that the creature being cast cannot be in `devour_sacrifices`.

8. `test_devour_creatures_go_to_graveyard` -- CR 702.82a: Sacrificed creatures should be in their owner's graveyard after resolution.

9. `test_devour_multiple_instances` -- CR 702.82c (by analogy): Creature with Devour 1 + Devour 2, sacrifice 2 creatures = (1*2) + (2*2) = 6 counters. (Artificial test case -- no printed cards have multiple Devour instances.)

10. `test_devour_creatures_devoured_tracking` -- CR 702.82b: Verify `creatures_devoured` field is set correctly on the permanent after resolution.

**Pattern**: Follow `crates/engine/tests/amplify.rs` for test structure, helpers, card definitions, and assertion patterns.

### Step 5: Card Definition (later phase)

**Suggested card**: Thunder-Thrash Elder -- {2}{R}, Creature - Lizard Warrior, 1/1, Devour 3
- Simple, clean Devour with a high N value. Good for testing.
- No additional abilities that complicate the card definition.
**Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: Thunder-Thrash Elder enters the battlefield, devouring 2 creatures.
- Initial state: 3 creatures on battlefield under P1's control + Thunder-Thrash Elder in hand.
- P1 casts Thunder-Thrash Elder with `devour_sacrifices` listing 2 of the 3 creatures.
- Assert: Thunder-Thrash Elder on battlefield with 6 +1/+1 counters (Devour 3 x 2 creatures).
- Assert: 2 sacrificed creatures in graveyard, 1 creature still on battlefield.
**Subsystem directory**: `test-data/generated-scripts/replacement/`

## Interactions to Watch

- **Death triggers**: Sacrificed creatures dying from Devour should trigger "when a creature dies" abilities (e.g., Blood Artist). The sacrifice is part of the ETB replacement, which does NOT use the stack, but the resulting death events DO go through the normal trigger system.
- **Sacrifice vs. destroy**: Devour sacrifices, not destroys. Indestructible creatures CAN be sacrificed. Regeneration does NOT prevent sacrifice.
- **Replacement effects on death**: If a sacrificed creature has "if this would die, exile it instead" (e.g., Rest in Peace), the replacement applies to the sacrifice result normally.
- **Commander death**: If a commander is sacrificed to Devour, the commander zone-change SBA applies as usual (CR 903.9a).
- **Simultaneous ETB**: If two Devour creatures enter at the same time, each can devour different creatures but a creature can only be devoured by one. The current engine doesn't support simultaneous spell resolution, so this edge case is deferred.
- **Layer interactions**: Devour's +1/+1 counters are placed as part of ETB, before any continuous effects from the permanent itself. This is consistent with the current ETB replacement ordering in resolution.rs (counters before static continuous effect registration).

## Discriminant Chain

- **KeywordAbility::Devour** = discriminant 124 (next after Bloodthirst=123)
- **No new StackObjectKind** needed (Devour doesn't create stack objects)
- **No new AbilityDefinition** variant needed (uses `AbilityDefinition::Keyword(KeywordAbility::Devour(n))`)
- **No new GameEvent** variant needed (uses existing `CounterAdded`, `CreatureDied`)
- **Next available after this**: KW=125, AbilDef=49, SOK=46
