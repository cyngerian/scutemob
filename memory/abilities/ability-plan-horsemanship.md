# Ability Plan: Horsemanship

**Generated**: 2026-02-28
**CR**: 702.31 (note: ability-wip.md says 702.30, but MCP lookup confirms the correct CR number is 702.31; 702.30 is Echo)
**Priority**: P4
**Similar abilities studied**: Shadow (CR 702.28) -- `crates/engine/src/state/types.rs:586-590`, `crates/engine/src/state/hash.rs:450-451`, `crates/engine/src/rules/combat.rs:491-501`, `crates/engine/tests/shadow.rs`

## CR Rule Text

```
702.31. Horsemanship

702.31a Horsemanship is an evasion ability.

702.31b A creature with horsemanship can't be blocked by creatures without
        horsemanship. A creature with horsemanship can block a creature with
        or without horsemanship. (See rule 509, "Declare Blockers Step.")

702.31c Multiple instances of horsemanship on the same creature are redundant.
```

## Key Edge Cases

- **Unidirectional, not bidirectional (CR 702.31b)**: Unlike Shadow (which is bidirectional -- shadow can't block non-shadow AND non-shadow can't block shadow), Horsemanship is unidirectional. A creature WITH horsemanship can block a creature WITHOUT horsemanship. Only the "attacker has horsemanship, blocker does not" direction is restricted.
- **Does NOT interact with Flying or Reach (ruling 2009-10-01 on Lu Bu and Sun Quan)**: Despite similarities to Flying, Horsemanship is an independent evasion ability. A creature with Reach cannot block a creature with Horsemanship (unless it also has Horsemanship). A creature with Flying cannot block a creature with Horsemanship (unless it also has Horsemanship).
- **Multiple evasion abilities stack (CR 509.1b)**: A creature with both Horsemanship and Flying requires a blocker that satisfies BOTH restrictions (must have Horsemanship AND Flying/Reach).
- **Redundancy (CR 702.31c)**: Multiple instances on the same creature are redundant. This is automatically handled by `OrdSet` deduplication on keywords.
- **Multiplayer**: No special multiplayer considerations. Block restriction applies per-blocker as usual.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant -- does not exist anywhere in the codebase
- [ ] 2. Rule enforcement -- no combat.rs code
- [ ] 3. Trigger wiring -- n/a (horsemanship is a static evasion ability, not a trigger)
- [ ] 4. Unit tests -- no test file exists
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Horsemanship` variant after `Overload` (line 599), before the closing `}` of the enum (line 600).
**Pattern**: Follow `KeywordAbility::Shadow` at line 586-590.

Add:
```rust
    /// CR 702.31: Horsemanship -- evasion ability (unidirectional).
    /// "A creature with horsemanship can't be blocked by creatures without horsemanship.
    /// A creature with horsemanship can block a creature with or without horsemanship."
    /// CR 702.31c: Multiple instances are redundant (auto-deduped by OrdSet).
    Horsemanship,
```

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add discriminant 71 after the Overload entry (line 458), before the closing `}` of the match (line 459).

Add:
```rust
            // Horsemanship (discriminant 71) -- CR 702.31
            KeywordAbility::Horsemanship => 71u8.hash_into(hasher),
```

**View model**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add arm to the exhaustive match on `KeywordAbility` at line 679, after the `Overload` arm.

Add:
```rust
        KeywordAbility::Horsemanship => "Horsemanship".to_string(),
```

**Match arms**: No other files have exhaustive matches on `KeywordAbility` that need updating. The TUI does not match on `KeywordAbility`. The `combat.rs`, `casting.rs`, `abilities.rs`, etc. use `.contains()` checks, not exhaustive matches. The `hash.rs` and `view_model.rs` are the only two exhaustive match sites.

### Step 2: Rule Enforcement

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs`
**Action**: Add horsemanship block restriction check after the Shadow check (line 501), before the protection check (line 503).
**Pattern**: Follow the Shadow enforcement at lines 491-501, but simplified to be unidirectional.
**CR**: 702.31b -- "A creature with horsemanship can't be blocked by creatures without horsemanship."

Add (after line 501, before line 503):
```rust
        // CR 702.31b: Horsemanship is a unidirectional evasion ability.
        // A creature with horsemanship can't be blocked by creatures without horsemanship.
        // Unlike Shadow, a creature with horsemanship CAN block creatures without horsemanship.
        if attacker_chars
            .keywords
            .contains(&KeywordAbility::Horsemanship)
            && !blocker_chars
                .keywords
                .contains(&KeywordAbility::Horsemanship)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} cannot block {:?} (attacker has horsemanship; \
                 blocker does not have horsemanship)",
                blocker_id, attacker_id
            )));
        }
```

**Key difference from Shadow**: Shadow checks `attacker_has_shadow != blocker_has_shadow` (bidirectional mismatch). Horsemanship only checks `attacker_has && !blocker_has` (unidirectional -- only restricts when the attacker has horsemanship).

### Step 3: Trigger Wiring

**N/A** -- Horsemanship is a static evasion ability with no trigger component. No trigger wiring is needed.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/horsemanship.rs`
**Pattern**: Follow `crates/engine/tests/shadow.rs` exactly (same helper, same `GameStateBuilder` setup, same `CombatState` construction, same `process_command + DeclareBlockers` assertion pattern).

**Tests to write**:

1. `test_702_31_horsemanship_creature_cannot_be_blocked_by_non_horsemanship`
   - CR 702.31b: Attacker has horsemanship, blocker does not. Block is illegal.
   - Pattern: identical to `test_702_28_shadow_creature_cannot_be_blocked_by_non_shadow` but with `KeywordAbility::Horsemanship`.

2. `test_702_31_horsemanship_creature_can_be_blocked_by_horsemanship`
   - CR 702.31b: Both attacker and blocker have horsemanship. Block is legal.
   - Pattern: identical to `test_702_28_shadow_creature_can_be_blocked_by_shadow` but with `KeywordAbility::Horsemanship`.

3. `test_702_31_non_horsemanship_can_be_blocked_by_horsemanship`
   - CR 702.31b second sentence: "A creature with horsemanship can block a creature with or without horsemanship." Attacker has NO horsemanship, blocker HAS horsemanship. Block is LEGAL.
   - **This is the key difference from Shadow.** The analogous Shadow test (`test_702_28_non_shadow_creature_cannot_be_blocked_by_shadow`) asserts `is_err()`. This Horsemanship test must assert `is_ok()`.

4. `test_702_31_non_horsemanship_can_block_non_horsemanship`
   - CR 702.31b baseline: Neither has horsemanship. Block is legal.
   - Pattern: identical to `test_702_28_non_shadow_can_block_non_shadow`.

5. `test_702_31_horsemanship_does_not_interact_with_flying`
   - Ruling 2009-10-01: "Despite the similarities between horsemanship and flying, horsemanship doesn't interact with flying or reach." Attacker has horsemanship, blocker has flying (but not horsemanship). Block is ILLEGAL.
   - This confirms the two abilities are independent evasion checks.

6. `test_702_31_horsemanship_plus_flying_both_must_be_satisfied`
   - CR 702.31b + CR 702.9a: Attacker has BOTH horsemanship and flying. Blocker has only horsemanship (no flying/reach). Block is ILLEGAL (flying restriction not satisfied).
   - Pattern: follows `test_702_28_shadow_plus_flying_both_must_be_satisfied`.

7. `test_702_31_horsemanship_plus_flying_satisfied_by_horsemanship_flying`
   - CR 702.31b + CR 702.9a: Attacker has both horsemanship and flying. Blocker has both horsemanship and flying. Block is LEGAL.
   - Pattern: follows `test_702_28_shadow_plus_flying_satisfied_by_shadow_flying`.

**Imports needed**:
```rust
use mtg_engine::{
    process_command, AttackTarget, Command, GameStateBuilder, KeywordAbility, ObjectSpec, PlayerId,
    Step,
};
```

### Step 5: Card Definition (later phase)

**Suggested card**: Shu Cavalry
- Mana cost: {2}{W}
- Type: Creature -- Human Soldier
- Oracle text: "Horsemanship (This creature can't be blocked except by creatures with horsemanship.)"
- P/T: 2/2
- Keywords: [Horsemanship]
- Simple vanilla creature -- ideal for testing the keyword in isolation.
- **Card lookup**: use `card-definition-author` agent with "Shu Cavalry"

### Step 6: Game Script (later phase)

**Suggested scenario**: Horsemanship evasion in combat -- Shu Cavalry attacks, defender has a non-horsemanship creature that cannot block it, and a horsemanship creature that can.
**Subsystem directory**: `test-data/generated-scripts/combat/`
**Suggested sequence number**: Next available in combat/ directory (check `ls test-data/generated-scripts/combat/`).

## Interactions to Watch

- **Flying/Reach independence**: Horsemanship is explicitly unrelated to Flying and Reach per ruling. The engine's block validation checks evasion abilities sequentially (Flying, Fear, Intimidate, Shadow, protection, Landwalk, Menace). Adding Horsemanship as another sequential check is correct -- each check independently returns an error if violated. No interaction logic is needed.
- **Shadow interaction**: A creature with Shadow AND Horsemanship? Both restrictions apply independently. Shadow prevents non-shadow from blocking (and prevents the creature from blocking non-shadow). Horsemanship prevents non-horsemanship from blocking. The blocker would need BOTH Shadow AND Horsemanship to be legal. This is handled naturally by sequential checks in `validate_blocker_legality`.
- **Protection interaction**: Protection from a quality (DEBT) is checked after evasion abilities in combat.rs. A creature with protection from white cannot be blocked by white creatures, independently of horsemanship. No interaction issues.
- **Layer system**: Horsemanship is a keyword ability. It can be granted or removed by continuous effects in Layer 6. The `calculate_characteristics` function handles this generically. No special layer logic needed.
- **Multiplayer**: No special considerations. Block restriction is per-attacker/per-blocker as with all evasion abilities. Multiple defending players each independently check their blockers.
