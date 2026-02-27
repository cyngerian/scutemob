# Ability Plan: Split Second

**Generated**: 2026-02-26
**CR**: 702.61
**Priority**: P2
**Similar abilities studied**: Flash (casting permission keyword in `casting.rs:L129-130`), Cycling (activated ability blocked by split second in `abilities.rs:L366`)

## CR Rule Text

702.61. Split Second

702.61a Split second is a static ability that functions only while the spell with split second is on the stack. "Split second" means "As long as this spell is on the stack, players can't cast other spells or activate abilities that aren't mana abilities."

702.61b Players may activate mana abilities and take special actions while a spell with split second is on the stack. Triggered abilities trigger and are put on the stack as normal while a spell with split second is on the stack.

702.61c Multiple instances of split second on the same spell are redundant.

## Key Edge Cases

- **Triggered abilities still fire (CR 702.61b).** Chalice of the Void, prowess, Ward, etc. all trigger normally. Only casting spells and activating non-mana abilities are prohibited. The engine's triggered ability infrastructure (`check_triggers` / `flush_pending_triggers`) is internal and does not go through `process_command`, so no changes are needed there.
- **Mana abilities are allowed (CR 702.61b).** The `TapForMana` command is a mana ability (CR 605) and must remain usable. No split second check in `mana.rs`.
- **Special actions are allowed (CR 702.61b).** `PlayLand` is a special action (CR 116.2a), but it already requires an empty stack (CR 305.1), so it's self-blocking when split second is on the stack. No special handling needed. Face-down creature flipping (morph) is also allowed but not yet implemented.
- **Existing spells/abilities on the stack are unaffected.** Casting a spell with split second does not remove or counter things already on the stack. The restriction only prevents NEW spells from being cast and NEW abilities from being activated (Krosan Grip ruling 2021-03-19).
- **Split second only functions on the stack.** It is a static ability of the spell, not the card. Once the spell resolves (or is countered/fizzled), the restriction immediately ends and players can again cast spells and activate abilities (Krosan Grip ruling 2021-03-19).
- **Cycling is an activated ability, not a mana ability.** It IS blocked by split second. The `CycleCard` command handler must check for split second.
- **Cascade trigger resolution cannot cast if split second is on stack.** If the resolution of a triggered ability involves casting a spell (e.g., cascade), that spell cannot be cast if a spell with split second is on the stack (Krosan Grip ruling 2021-03-19). However, this is a niche interaction: cascade resolves from the stack, and if a split second spell is ALSO on the stack, the cascade-found card can't be cast. The engine's cascade implementation in `copy.rs:resolve_cascade` casts internally (not via the `Command::CastSpell` path), so this edge case needs separate analysis. For the initial implementation, this interaction is deferred -- cascade + split second on the same stack is extremely rare.
- **Multiplayer: applies to all players.** Split second restricts ALL players, not just opponents. The caster of the split second spell also cannot cast additional spells or activate non-mana abilities while it's on the stack.
- **Kicker declared at cast time.** A spell with both kicker and split second (e.g., Molten Disaster) declares kicker at cast time before split second takes effect. Once on the stack with split second, no other player can respond with spells/abilities. This is already correctly handled because kicker is part of the `CastSpell` command, not a separate activation.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (n/a -- split second is not a trigger)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::SplitSecond` variant after `Kicker` (line ~248)
**Pattern**: Follow `KeywordAbility::Convoke` at line 237 (simple unit variant, no parameters)

```rust
/// CR 702.61: Split second -- as long as this spell is on the stack,
/// players can't cast other spells or activate abilities that aren't
/// mana abilities.
/// CR 702.61c: Multiple instances are redundant.
SplitSecond,
```

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` `HashInto` impl for `KeywordAbility`
**Action**: Add discriminant 33 (next after Kicker = 32) at line ~346

```rust
// SplitSecond (discriminant 33) -- CR 702.61
KeywordAbility::SplitSecond => 33u8.hash_into(hasher),
```

**Match arms**: Grep for `KeywordAbility` match expressions across the codebase. The `hash.rs` impl is exhaustive, so the compiler will flag any other exhaustive matches that need a new arm. No other exhaustive matches on `KeywordAbility` are expected outside `hash.rs` (the `OrdSet` checks use `.contains()`, not pattern matching).

### Step 2: Rule Enforcement -- Helper Function

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Add a public helper function `has_split_second_on_stack` that scans `state.stack_objects` for any spell whose source object has `KeywordAbility::SplitSecond`.
**CR**: 702.61a -- "As long as this spell is on the stack..."

The helper must:
1. Iterate over `state.stack_objects`.
2. For each `StackObjectKind::Spell { source_object }`, look up the source object in `state.objects`.
3. Check if that object's `characteristics.keywords` contains `KeywordAbility::SplitSecond`.
4. Use `calculate_characteristics` (not raw characteristics) to respect continuous effects that might grant or remove split second (e.g., a very unusual scenario, but correct per the layer system).
5. Return `true` if any such spell exists.

Only `Spell` stack objects can have split second (it's a property of the spell card). Activated abilities, triggered abilities, storm triggers, and cascade triggers cannot have split second.

**Proposed signature and location** (at the bottom of `casting.rs`, before `get_flashback_cost`):

```rust
/// CR 702.61a: Check if any spell on the stack has split second.
///
/// While a spell with split second is on the stack, players can't cast
/// other spells or activate abilities that aren't mana abilities.
/// Mana abilities and special actions are still allowed (CR 702.61b).
/// Triggered abilities still trigger and resolve normally (CR 702.61b).
pub fn has_split_second_on_stack(state: &GameState) -> bool {
    use crate::rules::layers::calculate_characteristics;

    state.stack_objects.iter().any(|stack_obj| {
        if let StackObjectKind::Spell { source_object } = &stack_obj.kind {
            let chars = calculate_characteristics(state, *source_object)
                .unwrap_or_else(|| {
                    state
                        .object(*source_object)
                        .map(|o| o.characteristics.clone())
                        .unwrap_or_default()
                });
            chars.keywords.contains(&KeywordAbility::SplitSecond)
        } else {
            false
        }
    })
}
```

### Step 3: Rule Enforcement -- CastSpell Gate

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Add a split second check in `handle_cast_spell`, immediately after the priority check (line ~64) and before the card fetch (line ~69).
**CR**: 702.61a -- players can't cast other spells while split second is on the stack.

```rust
// CR 702.61a: If a spell with split second is on the stack, no spells
// can be cast (mana abilities and special actions are still allowed).
if has_split_second_on_stack(state) {
    return Err(GameStateError::InvalidCommand(
        "a spell with split second is on the stack; no spells can be cast (CR 702.61a)"
            .into(),
    ));
}
```

**Placement**: After the priority check (`state.turn.priority_holder != Some(player)`) at line 59-64, before the card fetch block starting at line 69. This ensures the error is clear and early -- there's no point validating the card if casting is entirely prohibited.

### Step 4: Rule Enforcement -- ActivateAbility Gate

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add a split second check in `handle_activate_ability`, immediately after the priority check (line ~56-61) and before the source validation (line ~64).
**CR**: 702.61a -- players can't activate abilities that aren't mana abilities.

```rust
// CR 702.61a: If a spell with split second is on the stack, no
// non-mana abilities can be activated.
if crate::rules::casting::has_split_second_on_stack(state) {
    return Err(GameStateError::InvalidCommand(
        "a spell with split second is on the stack; non-mana abilities cannot be activated (CR 702.61a)"
            .into(),
    ));
}
```

**Import**: Use the full path `crate::rules::casting::has_split_second_on_stack` or add `use super::casting::has_split_second_on_stack;` at the top of `abilities.rs`.

### Step 5: Rule Enforcement -- CycleCard Gate

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add a split second check in `handle_cycle_card`, immediately after the priority check (line ~372-377) and before the zone check (line ~380).
**CR**: 702.61a -- cycling is an activated ability, not a mana ability; it is blocked by split second.

```rust
// CR 702.61a: Cycling is an activated ability (not a mana ability);
// it cannot be activated while a spell with split second is on the stack.
if crate::rules::casting::has_split_second_on_stack(state) {
    return Err(GameStateError::InvalidCommand(
        "a spell with split second is on the stack; cycling cannot be activated (CR 702.61a)"
            .into(),
    ));
}
```

### Step 6: Trigger Wiring

**Not applicable.** Split second is a static ability, not a triggered ability. It does not use triggers, the stack, or any event dispatch. The enforcement is purely through validation gates in Steps 3-5.

### Step 7: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/split_second.rs`
**Pattern**: Follow tests in `/home/airbaggie/scutemob/crates/engine/tests/casting.rs` (lines 22-80) for test setup with `GameStateBuilder`, `ObjectSpec`, and `process_command`.

**Tests to write**:

1. **`test_split_second_blocks_casting_spells`** -- CR 702.61a
   - Setup: 4-player game, P1 active at PreCombatMain. Place an instant with `SplitSecond` keyword on the stack (cast by P1). P2 has an instant in hand.
   - Action: P2 tries to `CastSpell` with their instant.
   - Assert: Returns `Err(GameStateError::InvalidCommand(...))` containing "split second".
   - Note: To get a split second spell on the stack, have P1 cast it first (which succeeds because split second isn't on the stack yet at that point). Then verify P2 can't cast.

2. **`test_split_second_blocks_activated_abilities`** -- CR 702.61a
   - Setup: Same as above, split second spell on stack. P2 has a creature on battlefield with an activated ability.
   - Action: P2 tries to `ActivateAbility`.
   - Assert: Returns `Err(GameStateError::InvalidCommand(...))` containing "split second".

3. **`test_split_second_blocks_cycling`** -- CR 702.61a
   - Setup: Split second spell on stack. P2 has a card with Cycling in hand.
   - Action: P2 tries to `CycleCard`.
   - Assert: Returns `Err(GameStateError::InvalidCommand(...))` containing "split second".

4. **`test_split_second_allows_mana_abilities`** -- CR 702.61b
   - Setup: Split second spell on stack. P2 has a land on battlefield with a mana ability.
   - Action: P2 uses `TapForMana`.
   - Assert: Succeeds (no error). Mana is added to pool.

5. **`test_split_second_allows_pass_priority`** -- CR 702.61b (players still get priority)
   - Setup: Split second spell on stack. P2 has priority.
   - Action: P2 uses `PassPriority`.
   - Assert: Succeeds. Priority passes to next player.

6. **`test_split_second_blocks_caster_too`** -- CR 702.61a (applies to ALL players)
   - Setup: P1 casts a split second spell. P1 still has priority (active player). P1 has another instant in hand.
   - Action: P1 tries to `CastSpell` with another spell.
   - Assert: Returns `Err(GameStateError::InvalidCommand(...))` containing "split second".

7. **`test_split_second_does_not_block_after_resolution`** -- Krosan Grip ruling
   - Setup: P1 casts a split second spell. All players pass priority, the spell resolves.
   - Action: P2 (or next priority holder) tries to cast a spell.
   - Assert: Succeeds. The split second restriction ended when the spell left the stack.

8. **`test_split_second_triggered_abilities_still_fire`** -- CR 702.61b
   - Setup: P1 has a creature with prowess on battlefield. P1 casts a split second instant.
   - Assert: Prowess trigger fires as normal (prowess triggers on "whenever you cast a noncreature spell"). The triggered ability is put on the stack above the split second spell.

9. **`test_split_second_multiple_instances_redundant`** -- CR 702.61c
   - This is automatically handled by `OrdSet` deduplication of keywords. No special code needed. Include a comment-only test or skip.

**Test setup pattern** for getting a split second spell on the stack:
- Create an instant card with `KeywordAbility::SplitSecond` in its keywords.
- Place it in P1's hand.
- Use `Command::CastSpell` to cast it (succeeds because no split second is on the stack yet).
- The returned state now has the split second spell on the stack.
- Attempt the blocked action on the returned state.

### Step 8: Card Definition (later phase)

**Suggested card**: Krosan Grip
- Mana cost: {2}{G}
- Type: Instant
- Keywords: Split second
- Effect: Destroy target artifact or enchantment
- This is the most commonly played split second card in Commander and is a clean test case (single target, simple effect).

**Card lookup**: use `card-definition-author` agent with "Krosan Grip"

### Step 9: Game Script (later phase)

**Suggested scenario**: "Krosan Grip destroys Sol Ring, opponent cannot respond"
- P1 controls a Sol Ring on the battlefield.
- P2 has Krosan Grip in hand.
- P2 casts Krosan Grip targeting Sol Ring.
- P1 attempts to activate Sol Ring's mana ability (allowed -- mana ability).
- P1 attempts to cast a counterspell (blocked by split second).
- All players pass, Krosan Grip resolves, Sol Ring is destroyed.

**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Cascade + Split Second**: If a cascade trigger resolves while a split second spell is also on the stack, the cascade-found card technically can't be cast (Krosan Grip ruling). The engine's `resolve_cascade` in `copy.rs` handles cascade internally, not through `Command::CastSpell`. This interaction is deferred -- document as a known gap, address when cascade is tested with split second.
- **Storm + Split Second**: A spell could theoretically have both storm and split second (via an effect granting one of them). Storm copies are NOT cast (CR 702.40c), so split second doesn't affect them. The copies would go on the stack normally. No special handling needed.
- **Ward + Split Second**: If a split second spell targets a creature with Ward, the Ward trigger fires normally (CR 702.61b). However, the Ward trigger asks the opponent to pay a cost -- if that cost involves casting or activating, it would be blocked. Standard Ward costs are mana payments, which are mana abilities and are allowed. No special handling needed.
- **Split Second doesn't affect the split second spell itself.** The restriction begins when the spell is put on the stack. The act of casting the split second spell itself is legal (split second wasn't on the stack when casting began).

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::SplitSecond` variant |
| `crates/engine/src/state/hash.rs` | Add discriminant 33 to `HashInto for KeywordAbility` |
| `crates/engine/src/rules/casting.rs` | Add `has_split_second_on_stack()` helper + gate in `handle_cast_spell` |
| `crates/engine/src/rules/abilities.rs` | Add gate in `handle_activate_ability` + `handle_cycle_card` |
| `crates/engine/tests/split_second.rs` | New test file with 8 tests |
