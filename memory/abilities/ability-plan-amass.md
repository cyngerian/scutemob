# Ability Plan: Amass

**Generated**: 2026-03-06
**CR**: 701.47
**Priority**: P4
**Similar abilities studied**: `Effect::Investigate` (effects/mod.rs:520), `Effect::Bolster` (effects/mod.rs:1008), `Effect::Proliferate` (effects/mod.rs)

## CR Rule Text

701.47. Amass

701.47a To amass [subtype] N means "If you don't control an Army creature, create a 0/0 black [subtype] Army creature token. Choose an Army creature you control. Put N +1/+1 counters on that creature. If it isn't a [subtype], it becomes a [subtype] in addition to its other types."

701.47b A player "amassed" after the process described in rule 701.47a is complete, even if some or all of those actions were impossible.

701.47c The phrases "the Army you amassed" and "the amassed Army" refer to the creature you chose, whether or not it received counters.

701.47d Some older cards were printed with amass N without including a subtype. Those cards have received errata in the Oracle card reference so that they read "amass Zombies N."

## Key Edge Cases

- **Token enters as 0/0 before counters** (ruling 2023-06-16): "the Zombie Army token you create enters the battlefield as a 0/0 creature before receiving counters." Abilities like Mentor of the Meek see it as 0/0 at ETB. SBAs are NOT checked between token creation and counter placement (same resolution context).
- **Multiple Army creatures** (ruling 2023-06-16): "In the rare case that you control multiple Army creatures (perhaps because you played a creature with changeling), you choose which of your Army creatures to put the +1/+1 counters on." Deterministic fallback: choose smallest ObjectId among Army creatures.
- **Subtype addition** (CR 701.47a): "If it isn't a [subtype], it becomes a [subtype] in addition to its other types." If you amass Orcs on a Zombie Army, it becomes an Orc Zombie Army.
- **Amass 0** (implied by CR 701.47b): If N=0, the process still completes (you still "amassed"). You still create the token if no Army exists. You put 0 counters on it (no counters added). You still add the subtype if needed. An Amassed event should still be emitted (CR 701.47b says the process is complete "even if some or all of those actions were impossible").
- **Multiplayer**: Each player has their own Army creatures. Amass only looks at creatures YOU control.
- **Subtype parameter**: Cards say "amass Zombies N" or "amass Orcs N". The Effect variant must parameterize the subtype.

## Current State (from ability-wip.md)

- [ ] Step 1: Effect variant
- [ ] Step 2: Rule enforcement (effect execution)
- [ ] Step 3: Event + trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Effect Variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Effect::Amass` variant in the `// -- Permanents --` section, near `Effect::Investigate` (line ~617).

```rust
/// CR 701.47a: Amass [subtype] N -- If you don't control an Army creature,
/// create a 0/0 black [subtype] Army creature token. Choose an Army creature
/// you control. Put N +1/+1 counters on that creature. If it isn't a
/// [subtype], it becomes a [subtype] in addition to its other types.
///
/// Deterministic fallback for multiple Armies: choose smallest ObjectId.
Amass {
    /// The creature subtype to add (e.g., "Zombie", "Orc").
    subtype: String,
    /// Number of +1/+1 counters to place.
    count: EffectAmount,
},
```

**No KeywordAbility variant needed**: Amass is a keyword action (like Investigate, Proliferate), not a keyword ability on permanents. Cards use it as an effect in their ability definitions.

### Step 2: Hash Discriminant

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `Effect::Amass` arm to the `HashInto for Effect` impl, after `Effect::Bolster` (line ~3335). Use **discriminant 41** (next after Bolster at 40).

```rust
// CR 701.47a: Amass (discriminant 41)
Effect::Amass { subtype, count } => {
    41u8.hash_into(hasher);
    subtype.hash_into(hasher);
    count.hash_into(hasher);
}
```

### Step 3: Effect Execution

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add execution logic for `Effect::Amass` in the main `execute_effect` match. Place after `Effect::Bolster` (line ~1060). Pattern follows Bolster (battlefield scan + counter placement) combined with Investigate (token creation).

**Logic** (CR 701.47a):
1. Resolve `count` via `resolve_amount(state, count, ctx)`.
2. Find all Army creatures on the battlefield controlled by `ctx.controller`:
   - Filter `state.objects` for `zone == Battlefield`, `is_phased_in()`, `controller == ctx.controller`.
   - Use `calculate_characteristics` to check if the object is a Creature with subtype "Army" (layer-aware, handles Changeling).
3. If no Army creatures found:
   - Create a 0/0 black `[subtype] Army` creature token via a new `army_token_spec(subtype)` helper.
   - Call `make_token` + `state.add_object` + emit `TokenCreated` and `PermanentEnteredBattlefield` events.
   - The newly created token's ObjectId becomes the chosen Army.
4. If one or more Army creatures found:
   - Choose one (deterministic: smallest ObjectId, matching Bolster pattern).
5. Put N +1/+1 counters on the chosen Army (if N > 0):
   - `obj.counters.entry(CounterType::PlusOnePlusOne).or_insert(0) += n`
   - Emit `GameEvent::CounterAdded { object_id, counter: PlusOnePlusOne, count: n }`
6. If the chosen Army does not have `SubType(subtype)` in its characteristics, add it:
   - `obj.characteristics.subtypes.insert(SubType(subtype.clone()))`
7. Emit `GameEvent::Amassed { player, army_id, count }` (CR 701.47b: always emitted).

**New helper function** in `crates/engine/src/cards/card_definition.rs`:

```rust
/// CR 701.47a: Token spec for an Army creature token.
/// Creates a 0/0 black [subtype] Army creature token.
pub fn army_token_spec(subtype: &str) -> TokenSpec {
    TokenSpec {
        name: format!("{} Army", subtype),
        power: 0,
        toughness: 0,
        colors: [Color::Black].into_iter().collect(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [
            SubType(subtype.to_string()),
            SubType("Army".to_string()),
        ].into_iter().collect(),
        keywords: OrdSet::new(),
        count: 1,
        tapped: false,
        mana_color: None,
        mana_abilities: vec![],
        activated_abilities: vec![],
    }
}
```

### Step 4: Game Event

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add `Amassed` variant to `GameEvent` enum. Place near `Investigated` (line ~711).

```rust
/// A player performed an amass action (CR 701.47a).
///
/// Emitted by `Effect::Amass` after the Army creature has received counters
/// and the subtype has been added. CR 701.47b: always emitted even if some
/// or all actions were impossible. Enables "whenever you amass" triggers.
Amassed {
    player: PlayerId,
    /// The Army creature that was chosen (or created).
    army_id: ObjectId,
    /// Number of +1/+1 counters placed.
    count: u32,
},
```

**Hash**: Add to `GameEvent` HashInto impl in `state/hash.rs`. Use the next available GameEvent discriminant (check existing chain).

### Step 5: Match Arm Exhaustiveness

**Files to update** for the new `Effect::Amass` variant:
- `crates/engine/src/state/hash.rs` -- done in Step 2
- Any other `match effect { ... }` arms in `effects/mod.rs` (the main execute_effect function is the only one)
- Grep for `Effect::` match exhaustiveness across the codebase

**Files to update** for the new `GameEvent::Amassed` variant:
- `crates/engine/src/state/hash.rs` -- GameEvent HashInto
- `crates/engine/src/rules/abilities.rs` -- `check_triggers` function (add arm, can be no-op initially)
- `tools/replay-viewer/src/view_model.rs` -- StateViewModel event rendering (if exhaustive match)
- `tools/tui/src/play/panels/stack_view.rs` -- if exhaustive match on events (unlikely but check)

### Step 6: Unit Tests

**File**: `crates/engine/tests/amass.rs` (new file)
**Tests to write**:

- `test_amass_creates_army_token_when_none_exists` -- CR 701.47a: No Army on battlefield, amass Zombies 2 creates a 0/0 black Zombie Army, adds 2 +1/+1 counters. Final state: 2/2 Zombie Army creature token.
- `test_amass_adds_counters_to_existing_army` -- CR 701.47a: Army already exists with 1 +1/+1 counter, amass Zombies 3 adds 3 more counters. Final: 4/4 Army.
- `test_amass_adds_subtype_to_existing_army` -- CR 701.47a: Zombie Army exists, amass Orcs 1 makes it an Orc Zombie Army with 1 additional counter.
- `test_amass_zero_still_creates_token` -- CR 701.47b: Amass Zombies 0 with no Army creates a 0/0 token (which will die to SBAs), but the Amassed event is emitted.
- `test_amass_multiple_armies_chooses_one` -- CR 701.47a: Two Army creatures on battlefield (e.g., via Changeling), deterministic picks smallest ObjectId.
- `test_amass_multiplayer_only_own_armies` -- Amass only looks at armies controlled by the effect's controller, not opponents'.

**Pattern**: Follow `crates/engine/tests/investigate.rs` (if exists) or `crates/engine/tests/bolster.rs` test structure. Use `GameStateBuilder::four_player()`, `ObjectSpec::creature()` for pre-existing armies, and `process_command` to execute effects.

### Step 7: Card Definition (later phase)

**Suggested card**: Dreadhorde Invasion (simple upkeep trigger + amass Zombies 1)
**Alternative**: Lazotep Plating (instant, amass Zombies 1 + hexproof)
**Card lookup**: use `card-definition-author` agent

### Step 8: Game Script (later phase)

**Suggested scenario**: Dreadhorde Invasion upkeep trigger creating an Army token, then a second upkeep adding counters to the existing Army.
**Subsystem directory**: `test-data/generated-scripts/baseline/`

## Interactions to Watch

- **Token creation fires ETB triggers**: The 0/0 Army token enters the battlefield normally. Any "whenever a creature enters the battlefield" triggers see it as 0/0 (before counters). Panharmonicon should double the ETB trigger but NOT the amass action itself.
- **SBAs after amass resolves**: If amass 0 creates a 0/0 token, SBAs will kill it immediately after the effect finishes resolving (CR 704.5f). The token exists briefly (long enough for ETB triggers to see it, and for the Amassed event).
- **Layer system**: The subtype addition in step 6 of execution is a direct modification to `obj.characteristics.subtypes`, NOT a continuous effect. This is correct because amass is a one-shot keyword action, not a continuous effect. The subtype is permanently added to the token/creature.
- **+1/+1 counters interact with -1/-1 counters**: SBA 704.5q annihilates pairs. If the Army had -1/-1 counters, some +1/+1 counters cancel out.
- **Changeling creatures are Army creatures**: A creature with Changeling has all creature types including Army. Amass can choose it. The `calculate_characteristics` check handles this because Changeling adds all subtypes at Layer 4.
- **Copy effects**: If an Army token is copied, the copy is also an Army creature. Amass can choose either.

## Discriminant Summary

- **Effect::Amass**: discriminant **41** (hash.rs, after Bolster at 40)
- **GameEvent::Amassed**: next available GameEvent discriminant (check existing chain in hash.rs)
- **No KeywordAbility variant needed**
- **No AbilityDefinition variant needed**
- **No StackObjectKind variant needed**
