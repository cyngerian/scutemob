# Ability Plan: Gravestorm

**Generated**: 2026-03-05
**CR**: 702.69
**Priority**: P4
**Similar abilities studied**: Storm (CR 702.40) in `casting.rs:2625-2682`, `resolution.rs:773-792`, `copy.rs:245-272`; Replicate (CR 702.56) in `casting.rs:2784-2831`, `resolution.rs:829-859`

## CR Rule Text

> **702.69. Gravestorm**
>
> **702.69a** Gravestorm is a triggered ability that functions on the stack. "Gravestorm" means "When you cast this spell, copy it for each permanent that was put into a graveyard from the battlefield this turn. If the spell has any targets, you may choose new targets for any of the copies."
>
> **702.69b** If a spell has multiple instances of gravestorm, each triggers separately.

## Key Edge Cases

- **Global count**: Gravestorm counts ALL permanents put into graveyards from the battlefield this turn, regardless of controller, owner, or whether they were tokens (ruling 2024-02-02).
- **Copies are NOT cast**: The copies created by gravestorm are created on the stack, so they do not trigger "whenever you cast a spell" abilities and do not increment `spells_cast_this_turn` (ruling 2024-02-02). Already handled by `create_storm_copies` / `copy_spell_on_stack` which sets `is_copy: true`.
- **Trigger can be countered**: The triggered ability itself can be countered (e.g., by Stifle). If countered, no copies are created (ruling 2024-02-02).
- **Each copy is independent**: Countering the original spell or a copy does not affect other copies (ruling 2024-02-02).
- **Count captured at trigger time**: Like storm, the gravestorm count should be captured when the trigger is created (at cast time), not when it resolves. This prevents permanents dying between cast and resolution from affecting the count. (Follows storm pattern -- `storm_count` is captured at trigger creation in `casting.rs:2641`.)
- **Tokens count**: Tokens that go to a graveyard from the battlefield count (they briefly exist in the graveyard before ceasing to exist as an SBA -- CR 704.5d). The counter should increment before the token-cessation SBA runs.
- **Multiple instances**: If a spell has multiple instances of gravestorm, each triggers separately (702.69b). Each trigger independently creates copies equal to the gravestorm count.
- **Multiplayer**: In a 4-player game, ALL permanents from ALL players that went to graveyards from battlefield this turn count. This is inherently a global counter, not per-player.
- **"This turn" scope**: The counter resets at the start of each turn (same cadence as `spells_cast_this_turn` and `life_lost_this_turn`).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (tracking counter)
- [ ] Step 3: Trigger wiring (casting.rs trigger + resolution.rs handler)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Gravestorm` variant.
**Pattern**: Follow `KeywordAbility::Replicate` (the most recently added keyword).
**Note**: Gravestorm is a simple keyword (no parameter). No `AbilityDefinition` variant needed -- use `AbilityDefinition::Keyword(KeywordAbility::Gravestorm)` like Storm does.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Gravestorm => 107u8.hash_into(hasher)` to the KeywordAbility HashInto impl.
**Discriminant**: KW=107 (next available after Replicate=106).

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::GravestormTrigger` variant:
```rust
GravestormTrigger {
    source_object: ObjectId,
    original_stack_id: ObjectId,
    gravestorm_count: u32,
},
```
**Discriminant**: SOK=36 (next available after ReplicateTrigger=35).
**Pattern**: Follow `StackObjectKind::StormTrigger` and `StackObjectKind::ReplicateTrigger`.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `StackObjectKind::GravestormTrigger` with discriminant 36.

**Match arms to update** (grep for exhaustive matches on `StackObjectKind`):
- `tools/tui/src/play/panels/stack_view.rs` -- add display arm: `("Gravestorm: ".to_string(), Some(*source_object))`
- `tools/replay-viewer/src/view_model.rs` -- add arm: `("gravestorm_trigger", Some(*source_object))`
- `crates/engine/src/rules/resolution.rs` -- add resolution handler (Step 3)
- `crates/engine/src/testing/script_schema.rs` -- if there's a match on StackObjectKind, add arm

### Step 2: Tracking Counter (Rule Enforcement)

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add field `pub permanents_put_into_graveyard_this_turn: u32` to `GameState`.
**CR**: 702.69a -- "each permanent that was put into a graveyard from the battlefield this turn"
**Rationale**: This is a global count (not per-player). All permanents from all players, including tokens, regardless of owner/controller. `GameState` is the correct location, not `PlayerState` or `TurnState`.
**Default**: Initialize to 0 in `GameStateBuilder::build()`.
**Serde**: Add `#[serde(default)]` for backwards compatibility with existing scripts.

**File**: `crates/engine/src/state/mod.rs` (inside `move_object_to_zone`)
**Action**: After the zone move succeeds, check if `old_object.zone` matches `ZoneId::Battlefield` and `to` matches `ZoneId::Graveyard(_)`. If so, increment `self.permanents_put_into_graveyard_this_turn += 1`.
**Also**: Apply the same increment in `move_object_to_bottom_of_zone` (same check).
**CR**: 702.69a -- captures ALL pathways: SBA destroy, sacrifice, destroy effects, damage lethal, etc.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.permanents_put_into_graveyard_this_turn.hash_into(hasher)` to `GameState`'s HashInto impl.

**File**: `crates/engine/src/rules/turn_actions.rs` (inside `reset_turn_state`)
**Action**: Add `state.permanents_put_into_graveyard_this_turn = 0;` to reset at the start of each turn. Place alongside the existing `cards_drawn_this_turn` and `life_lost_this_turn` resets (around line 719-728). Unlike those per-player counters, this is on `GameState` directly, so add it outside the player loop.

### Step 3: Trigger Wiring

#### 3a: Trigger Creation (casting.rs)

**File**: `crates/engine/src/rules/casting.rs`
**Action**: After the storm trigger block (~line 2682) and before the cascade trigger block (~line 2684), add a gravestorm trigger block:
```
// CR 702.69a: Gravestorm -- "When you cast this spell, copy it for each
// permanent that was put into a graveyard from the battlefield this turn."
// Gravestorm is a triggered ability (CR 702.69a). The count is captured
// at trigger creation time (at cast, not resolution).
if chars.keywords.contains(&KeywordAbility::Gravestorm) {
    let count = state.permanents_put_into_graveyard_this_turn;
    let trigger_id = state.next_object_id();
    let trigger_obj = StackObject { ... kind: StackObjectKind::GravestormTrigger { ... } ... };
    state.stack_objects.push_back(trigger_obj);
    events.push(GameEvent::AbilityTriggered { ... });
}
```
**Pattern**: Identical to the storm trigger block at lines 2640-2682. The only differences: (1) keyword is `Gravestorm` not `Storm`, (2) count comes from `state.permanents_put_into_graveyard_this_turn` not `storm_count()`, (3) kind is `GravestormTrigger`.
**Note**: The gravestorm count is NOT `count - 1` (unlike storm). Storm subtracts 1 because the storm spell itself is one of the "spells cast this turn." Gravestorm counts permanents going to graveyards, which is unrelated to the spell being cast.

#### 3b: Trigger Resolution (resolution.rs)

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add a resolution arm for `StackObjectKind::GravestormTrigger` after the `ReplicateTrigger` arm (~line 859):
```rust
// CR 702.69a: Gravestorm trigger resolves -- create copies of the original spell.
StackObjectKind::GravestormTrigger {
    source_object: _,
    original_stack_id,
    gravestorm_count,
} => {
    let controller = stack_obj.controller;
    let copy_events = crate::rules::copy::create_storm_copies(
        state,
        original_stack_id,
        controller,
        gravestorm_count,
    );
    events.extend(copy_events);
    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```
**Pattern**: Identical to the `StormTrigger` arm at lines 774-792 and the `ReplicateTrigger` arm at lines 841-859. Reuses `create_storm_copies` which calls `copy_spell_on_stack` N times.

### Step 4: Unit Tests

**File**: `crates/engine/tests/gravestorm.rs` (new file)
**Pattern**: Follow `crates/engine/tests/storm_copy.rs` structure and `crates/engine/tests/replicate.rs` helpers.
**Tests to write**:

1. **`test_gravestorm_basic_creates_copies`** -- CR 702.69a: Cast a gravestorm spell when `permanents_put_into_graveyard_this_turn = 3`. After resolving the trigger, verify 3 copies + 1 original = 4 stack objects. Verify SpellCopied events (not SpellCast).

2. **`test_gravestorm_zero_count_no_copies`** -- CR 702.69a: Cast a gravestorm spell when `permanents_put_into_graveyard_this_turn = 0`. After resolving the trigger, verify only the original remains (trigger still appears but creates 0 copies).

3. **`test_gravestorm_count_increments_on_permanent_dying`** -- CR 702.69a: Place a creature on the battlefield, destroy it (or use SBA with 0 toughness), verify `permanents_put_into_graveyard_this_turn` incremented to 1.

4. **`test_gravestorm_count_includes_tokens`** -- CR 702.69a + ruling 2024-02-02: Create a token on the battlefield, sacrifice/destroy it, verify the counter increments. Tokens going to graveyard from battlefield count.

5. **`test_gravestorm_count_includes_all_players`** -- CR 702.69a + ruling 2024-02-02: Destroy permanents controlled by different players, verify the counter increments for each regardless of controller.

6. **`test_gravestorm_count_resets_each_turn`** -- Verify that `permanents_put_into_graveyard_this_turn` resets to 0 at the start of each turn via `reset_turn_state`.

7. **`test_gravestorm_copies_not_cast`** -- CR 702.69a + ruling 2024-02-02: Verify copies have `is_copy: true` and do not trigger SpellCast events. (Inherited from `create_storm_copies` behavior, but verify explicitly.)

8. **`test_gravestorm_count_does_not_include_non_battlefield_to_graveyard`** -- Verify that a card discarded from hand to graveyard does NOT increment the counter (only battlefield-to-graveyard counts).

### Step 5: Card Definition (later phase)

**Suggested card**: Follow the Bodies ({2}{U} Sorcery -- Gravestorm, Investigate)
**Card lookup**: use `card-definition-author` agent
**Note**: This is the only card with Gravestorm in MTG. It also has Investigate, which is already implemented as `Effect::Investigate { count }`.

### Step 6: Game Script (later phase)

**Suggested scenario**: Follow the Bodies cast after 2 permanents die. Setup: two creatures on battlefield, both get destroyed (e.g., SBA with 0 toughness via an effect). Then cast Follow the Bodies. Verify: gravestorm trigger appears, resolves to create 2 copies. Each copy creates a Clue token (Investigate). Result: 3 Clue tokens total (1 original + 2 copies).
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Counter is on `GameState`, not `PlayerState`**: Unlike `spells_cast_this_turn`, this is a shared global counter. All zone-move code paths flow through `move_object_to_zone` / `move_object_to_bottom_of_zone`, so a single increment site covers all pathways (SBA, sacrifice, destroy, combat damage lethal, etc.).
- **Tokens**: Tokens briefly exist in the graveyard (CR 704.5d) before ceasing to exist as an SBA. The counter should increment when the token enters the graveyard, not when the SBA removes it. Since the increment is inside `move_object_to_zone`, this is automatically correct -- the increment happens before any subsequent SBA check.
- **Non-permanent cards**: Instants and sorceries that resolve go from the stack to the graveyard, not from the battlefield. They should NOT increment the counter. The `ZoneId::Battlefield` check in `move_object_to_zone` naturally excludes them.
- **Gravestorm trigger resolution reuses `create_storm_copies`**: Same infrastructure as Storm and Replicate. No new copy logic needed.
- **Multiple instances of gravestorm (702.69b)**: Each triggers separately. The current pattern (a single `if chars.keywords.contains(...)` check) only creates one trigger. For strict 702.69b compliance, we'd need to check for multiple instances. However, no card has multiple gravestorm instances, and `KeywordAbility` in `im::OrdSet` deduplicates. This is a known limitation shared with Storm (702.40b) -- accept as LOW gap.
