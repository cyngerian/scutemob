# Ability Plan: Skulk

**Generated**: 2026-02-28
**CR**: 702.118
**Priority**: P4
**Similar abilities studied**: Shadow (CR 702.28) -- `state/types.rs:586-590`, `state/hash.rs:450-451`, `rules/combat.rs:491-501`, `tests/shadow.rs`; Flying (CR 702.9) -- `rules/combat.rs:427-438`; Fear (CR 702.36) -- `rules/combat.rs:475-488`; Intimidate (CR 702.13) -- `rules/combat.rs:453-472`

## CR Rule Text

702.118. Skulk

702.118a Skulk is an evasion ability.

702.118b A creature with skulk can't be blocked by creatures with greater power. (See rule 509, "Declare Blockers Step.")

702.118c Multiple instances of skulk on the same creature are redundant.

## Key Edge Cases

- **One-directional restriction (unlike Shadow)**: Skulk only prevents creatures with greater power from blocking the skulk creature. It does NOT prevent the skulk creature from blocking anything. This contrasts with Shadow, which is bidirectional.
- **Power comparison uses the blocker's power vs. the attacker's power**: A creature with skulk (the attacker) can't be blocked by a creature whose power is strictly GREATER than the skulk creature's power. Equal power IS allowed to block.
- **Zero or negative power**: If the skulk creature has 0 or negative power (via effects like Sudden Spoiling), use the actual value. A creature with skulk and 0 power can only be blocked by creatures with 0 or less power. A creature with skulk and -1 power can only be blocked by creatures with -1 or less power. (Ruling 2016-04-08 on Furtive Homunculus)
- **Checked only at blocker declaration time**: Modifying either creature's power after blockers are chosen won't cause the attacking creature to become unblocked. (Ruling 2016-04-08, multiple cards) The engine naturally handles this because the block-legality check runs in `handle_declare_blockers` at declaration time.
- **Multiple evasion abilities stack**: A creature with both Skulk and Flying requires blockers to satisfy BOTH restrictions (must have flying/reach AND must not have greater power). This is the standard evasion stacking behavior per CR 509.1b.
- **Redundant instances**: Multiple instances of skulk on the same creature are redundant (CR 702.118c). Auto-handled by `OrdSet` deduplication.
- **Uses `calculate_characteristics` for power**: Power comparison must use post-layer-system values from `calculate_characteristics`, not base P/T. This handles pump effects, Humility, etc.
- **Multiplayer**: No special multiplayer considerations beyond the standard combat structure (each defending player independently declares blockers for attackers targeting them).

## Current State (from ability-wip.md)

Skulk is not the current WIP ability (Horsemanship is). All steps for Skulk are fresh:

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (n/a -- skulk is purely static)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Skulk` variant after `Overload` (line ~599)
**Pattern**: Follow `KeywordAbility::Shadow` at line 590 -- simple unit variant, no parameters
**Doc comment**:
```rust
/// CR 702.118: Skulk -- evasion ability.
/// "A creature with skulk can't be blocked by creatures with greater power."
/// CR 702.118c: Multiple instances are redundant (auto-deduped by OrdSet).
Skulk,
```

**Hash**: Add to `state/hash.rs` `HashInto` impl after `Overload` (discriminant 70, line ~458):
```rust
// Skulk (discriminant 71) -- CR 702.118
KeywordAbility::Skulk => 71u8.hash_into(hasher),
```

**Replay viewer `format_keyword`**: Add arm to `tools/replay-viewer/src/view_model.rs` in the `format_keyword` function (after `KeywordAbility::Overload` at line ~679):
```rust
KeywordAbility::Skulk => "Skulk".to_string(),
```

**Match arms**: Grep for exhaustive `KeywordAbility` match expressions across the codebase. The three known sites:
1. `state/hash.rs` -- covered above
2. `tools/replay-viewer/src/view_model.rs:format_keyword` -- covered above
3. Any other exhaustive matches -- grep `KeywordAbility::Overload` to find all arms that need a new case

### Step 2: Rule Enforcement

**File**: `crates/engine/src/rules/combat.rs`
**Action**: Add skulk blocking restriction check in `handle_declare_blockers`, in the per-blocker-attacker loop, after the Shadow check (line ~501) and before the protection check (line ~503).
**Pattern**: Follow Shadow check pattern at lines 491-501, but with a power comparison instead of a symmetric keyword check.
**CR**: 702.118b -- "A creature with skulk can't be blocked by creatures with greater power."

The check should be:
```rust
// CR 702.118b: Skulk -- a creature with skulk can't be blocked by creatures
// with greater power. Unlike Shadow, this is one-directional: it only restricts
// what can block the skulk creature, not what the skulk creature can block.
if attacker_chars.keywords.contains(&KeywordAbility::Skulk) {
    let attacker_power = attacker_chars.power.unwrap_or(0);
    let blocker_power = blocker_chars.power.unwrap_or(0);
    if blocker_power > attacker_power {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} cannot block {:?} (attacker has skulk with power {}; \
             blocker has greater power {})",
            blocker_id, attacker_id, attacker_power, blocker_power
        )));
    }
}
```

**Key details**:
- Uses `attacker_chars.power` and `blocker_chars.power` from `calculate_characteristics` (already computed earlier in the loop -- `attacker_chars` at line 429, `blocker_chars` at line 370)
- `unwrap_or(0)` handles the case where power is None (non-creature objects, though this should not happen since both are validated as creatures earlier in the loop)
- Comparison is strictly greater (`blocker_power > attacker_power`), not greater-or-equal. Equal power CAN block.
- The check only fires when the ATTACKER has skulk. The skulk creature can freely block anything.

### Step 3: Trigger Wiring

**Not applicable.** Skulk is a static evasion ability (CR 702.118a). It has no triggered ability component. No wiring needed in `builder.rs` or `abilities.rs`.

### Step 4: Unit Tests

**File**: `crates/engine/tests/skulk.rs`
**Tests to write**:

1. `test_702_118_skulk_creature_cannot_be_blocked_by_greater_power`
   - CR 702.118b -- Basic case: 2/1 skulk attacker cannot be blocked by a 3/3 blocker
   - Setup: p1 has a 2/1 skulk creature, p2 has a 3/3 creature
   - Expect: `process_command(DeclareBlockers)` returns `Err`

2. `test_702_118_skulk_creature_can_be_blocked_by_equal_power`
   - CR 702.118b -- Equal power IS allowed: 2/1 skulk attacker CAN be blocked by a 2/2 blocker
   - Setup: p1 has a 2/1 skulk creature, p2 has a 2/2 creature
   - Expect: `process_command(DeclareBlockers)` returns `Ok`

3. `test_702_118_skulk_creature_can_be_blocked_by_lesser_power`
   - CR 702.118b -- Lesser power IS allowed: 3/3 skulk attacker CAN be blocked by a 1/4 blocker
   - Setup: p1 has a 3/3 skulk creature, p2 has a 1/4 creature
   - Expect: `process_command(DeclareBlockers)` returns `Ok`

4. `test_702_118_skulk_is_one_directional`
   - CR 702.118b -- Skulk only restricts what blocks IT. A skulk creature can block a non-skulk creature freely.
   - Setup: p1 has a 5/5 non-skulk attacker, p2 has a 2/1 skulk blocker
   - Expect: `process_command(DeclareBlockers)` returns `Ok` (skulk creature can block anything)

5. `test_702_118_skulk_plus_flying_both_must_be_satisfied`
   - CR 702.118b + CR 702.9a -- A skulk+flying creature requires the blocker to satisfy BOTH
   - Setup: p1 has a 2/1 skulk+flying creature, p2 has a 1/1 non-flying creature
   - Expect: `process_command(DeclareBlockers)` returns `Err` (no flying/reach)
   - Also test: p2 has a 1/1 with flying -- should succeed (has flying, power not greater)
   - Also test: p2 has a 3/3 with flying -- should fail (has flying, but power IS greater)

6. `test_702_118_skulk_zero_power_attacker`
   - Ruling 2016-04-08 -- Zero or negative power: a 0/1 skulk creature can only be blocked by creatures with 0 or less power
   - Setup: p1 has a 0/1 skulk creature, p2 has a 1/1 creature
   - Expect: `process_command(DeclareBlockers)` returns `Err` (blocker power 1 > attacker power 0)
   - Also test: p2 has a 0/3 creature -- should succeed (0 is not greater than 0)

7. `test_702_118_skulk_with_power_pump`
   - Skulk uses post-layer power via `calculate_characteristics`
   - Setup: p1 has a 2/1 skulk creature with a continuous effect giving +2/+0 (making it 4/1), p2 has a 3/3 creature
   - Expect: `process_command(DeclareBlockers)` returns `Ok` (blocker power 3 is not greater than pumped attacker power 4)

**Pattern**: Follow tests in `tests/shadow.rs` -- same file structure, same `find_object` helper, same `GameStateBuilder` setup with `.at_step(Step::DeclareBlockers)`, same combat state initialization pattern.

### Step 5: Card Definition (later phase)

**Suggested card**: Furtive Homunculus ({1}{U}, 2/1 Creature - Homunculus, Skulk)
- Simple vanilla skulk creature, ideal for testing
- Low mana cost, clean oracle text
- Alternative: Pale Rider of Trostad ({1}{B}, 3/3, Skulk + ETB discard trigger) -- tests skulk on a higher-power creature

**Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: "Skulk creature cannot be blocked by higher-power creature; can be blocked by equal-power creature"
- Turn 1: p1 attacks with a 2/1 skulk creature
- p2 attempts to block with a 3/3 -- blocked (error, illegal block)
- p2 blocks with a 2/2 -- allowed
- Validates CR 702.118b boundary condition (greater vs. equal)

**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

- **Skulk + Flying/Reach**: Both evasion restrictions must be satisfied independently. The engine's sequential check pattern in `handle_declare_blockers` handles this naturally -- each evasion ability check runs independently and any failure rejects the block.
- **Skulk + Menace**: A skulk creature with menace requires at least two blockers (menace) AND none of those blockers can have greater power (skulk). The menace check is separate (lines 540+, counted after all per-pair checks pass), so this works correctly without special handling.
- **Skulk + Pump effects (Giant Growth, etc.)**: The power comparison uses `calculate_characteristics`, which includes all continuous effects. A pumped skulk creature has a higher threshold for what can block it. This is correct per the rulings.
- **Skulk + Humility**: Humility removes all abilities including skulk (Layer 6). If skulk is removed, the check won't fire. Correct behavior -- `calculate_characteristics` returns the post-Humility keyword set.
- **Skulk + Power-reducing effects (e.g., Sudden Spoiling)**: If the skulk creature's power is reduced to 0 or negative, the check still works correctly because `unwrap_or(0)` yields the actual computed value. Creatures with any positive power cannot block it.
- **Multiplayer**: No special considerations. Each defending player declares blockers independently. The skulk check fires per blocker-attacker pair within each declaration.
