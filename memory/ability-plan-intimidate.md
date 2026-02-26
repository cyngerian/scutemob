# Ability Plan: Intimidate

**Generated**: 2026-02-25
**CR**: 702.13
**Priority**: P1
**Similar abilities studied**: Flying (CR 702.9) in `rules/combat.rs` lines 427-439; Menace (CR 702.110) in `rules/combat.rs` lines 463-493; CantBeBlocked in `rules/combat.rs` lines 441-451; Protection blocking check in `rules/protection.rs` lines 147-152

## CR Rule Text

```
702.13. Intimidate

702.13a  Intimidate is an evasion ability.

702.13b  A creature with intimidate can't be blocked except by artifact creatures
         and/or creatures that share a color with it. (See rule 509, "Declare
         Blockers Step.")

702.13c  Multiple instances of intimidate on the same creature are redundant.
```

## Key Edge Cases

1. **Colorless attacker with intimidate**: A colorless creature with intimidate has no colors
   to share (CR 105.2c: "A colorless object has no color"). It can only be blocked by artifact
   creatures. No non-artifact creature can share a color with it because it has no colors.

2. **Multicolored attacker with intimidate**: A white-blue creature with intimidate can be
   blocked by any creature that shares *at least one* color with it (white or blue), in
   addition to artifact creatures. The blocker does not need to share ALL colors. (Ruling on
   Hideous Visage, 2011-09-22: "A multicolored creature with intimidate can be blocked by any
   creature that shares a color with it.")

3. **Artifact creature blocker**: An artifact creature can always block a creature with
   intimidate, regardless of colors. The blocker must have the card type `Artifact` AND
   `Creature` in its card_types. A non-creature artifact cannot block (it can't block
   anything -- only creatures can block).

4. **Color identity vs. current colors**: Intimidate checks the *current* colors of both the
   attacker and potential blocker at declare-blockers time, NOT color identity or printed
   colors. Effects that change colors (e.g., turning a creature blue) affect the check.
   (Rulings on Surrakar Marauder, Guul Draz Vampire, Bladetusk Boar, 2009-10-01: "Intimidate
   looks at the current colors of a creature that has it.")

5. **Once blocked, color changes don't matter**: If a creature with intimidate is legally
   blocked, changing its colors after blockers are declared does not change or undo the block.
   (Rulings on Guul Draz Vampire, Surrakar Marauder, Halo Hunter, 2009-10-01.) This is
   already handled by the engine -- blocking legality is checked only at declare-blockers time.

6. **Intimidate + Protection interaction**: Both are checked independently. A creature with
   intimidate AND protection from red can't be blocked by non-artifact non-color-sharing
   creatures (intimidate) AND can't be blocked by red creatures (protection). Both restrictions
   must be satisfied independently. The engine already checks protection separately at
   `combat.rs:455`.

7. **Intimidate + Flying**: Both evasion abilities stack. A creature with both flying and
   intimidate can only be blocked by a creature that satisfies BOTH restrictions (has
   flying/reach AND is artifact or shares a color). The current code checks flying first, then
   the intimidate check runs on the same pair. This works correctly as-is because both checks
   are in sequence in the per-blocker loop.

8. **Redundancy (702.13c)**: Multiple instances are redundant. No special handling needed --
   the engine uses `OrdSet<KeywordAbility>` which deduplicates automatically.

9. **Multiplayer**: Intimidate checks are per-blocker, per-attacker. In multiplayer, each
   defending player independently checks their blockers. The engine already handles this
   because `handle_declare_blockers` is called per-player and iterates the blocker list.

## Current State (from ability-wip.md)

- [x] Step 1: Enum variant -- exists at `crates/engine/src/state/types.rs:114`
- [x] Step 1a: Hash discriminant -- exists at `crates/engine/src/state/hash.rs:274` (discriminant 11)
- [ ] Step 2: Rule enforcement in `rules/combat.rs`
- [ ] Step 3: Trigger wiring (n/a -- intimidate is a static evasion ability, no triggers)
- [ ] Step 4: Unit tests in `crates/engine/tests/keywords.rs`
- [ ] Step 5: Card definition (Bladetusk Boar)
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant (DONE)

**File**: `crates/engine/src/state/types.rs`
**Status**: Already exists at line 114: `Intimidate,`
**Hash**: Already exists at `crates/engine/src/state/hash.rs:274` with discriminant `11u8`
**No action needed.**

### Step 2: Rule Enforcement — Blocking Restriction

**File**: `crates/engine/src/rules/combat.rs`
**Action**: Add intimidate blocking check in the `handle_declare_blockers` function, in the
per-blocker validation loop, after the existing flying check (line ~439) and before or after
the CantBeBlocked check.

**Pattern**: Follow the Flying check at lines 427-439. The intimidate check is structurally
similar: read both attacker and blocker characteristics, check if the attacker has the keyword,
then validate the blocker meets one of the two exceptions.

**Insertion point**: After the flying check block (line 439) and after the CantBeBlocked check
(line 451), but before the protection check (line 455). Insert between lines 451 and 453
(before the protection check).

**CR**: 702.13b -- "A creature with intimidate can't be blocked except by artifact creatures
and/or creatures that share a color with it."

**Logic**:
```
// CR 702.13b: A creature with intimidate can't be blocked except by
// artifact creatures and/or creatures that share a color with it.
if attacker_chars.keywords.contains(&KeywordAbility::Intimidate) {
    let blocker_is_artifact_creature =
        blocker_chars.card_types.contains(&CardType::Artifact)
        && blocker_chars.card_types.contains(&CardType::Creature);
    let shares_a_color = attacker_chars.colors.iter()
        .any(|c| blocker_chars.colors.contains(c));
    if !blocker_is_artifact_creature && !shares_a_color {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} cannot block {:?} (attacker has intimidate; blocker is neither an artifact creature nor shares a color)",
            blocker_id, attacker_id
        )));
    }
}
```

**Important notes**:
- `attacker_chars` is already computed at line 429 via `calculate_characteristics(state, *attacker_id)` -- reuse it.
- `blocker_chars` is already computed earlier in the loop via `calculate_characteristics(state, *blocker_id)` -- reuse it.
- Check `blocker_chars.card_types` for BOTH `Artifact` and `Creature` -- a non-creature artifact on the battlefield cannot block.
- Use `attacker_chars.colors` (the layer-calculated colors, not printed colors) per the rulings.
- The `OrdSet::contains` method works for color checks.
- The `blocker_chars.colors` intersection check implements "shares a color" -- any single shared color suffices.

### Step 3: Trigger Wiring

**Not applicable.** Intimidate is a static evasion ability (CR 702.13a). It does not trigger,
does not use the stack, and does not need trigger dispatch. It is purely a blocking restriction
checked at declare-blockers time.

### Step 4: Unit Tests

**File**: `crates/engine/tests/keywords.rs`
**Insertion point**: After the menace tests (line ~635), before the lifelink tests. Add a new
section header: `// -- CR 702.13: Intimidate --`

**Tests to write** (7 tests):

1. **`test_702_13_intimidate_blocks_non_matching_creature`**
   - CR 702.13b basic enforcement
   - Red creature with intimidate attacks. White creature (non-artifact) tries to block.
   - Expected: `Err` -- blocker shares no color and is not an artifact creature.
   - Setup: p1 has red 3/2 with `Intimidate` + `Color::Red`; p2 has white 2/2 with `Color::White`

2. **`test_702_13_intimidate_allows_artifact_creature_blocker`**
   - CR 702.13b artifact creature exception
   - Red creature with intimidate attacks. Artifact creature (colorless) blocks.
   - Expected: `Ok` -- artifact creatures can always block intimidate.
   - Setup: p1 has red 3/2 with `Intimidate` + `Color::Red`; p2 has 1/1 with `card_types: [Artifact, Creature]` and no colors

3. **`test_702_13_intimidate_allows_same_color_blocker`**
   - CR 702.13b color-sharing exception
   - Red creature with intimidate attacks. Red creature blocks.
   - Expected: `Ok` -- blocker shares a color (red).
   - Setup: p1 has red 3/2 with `Intimidate` + `Color::Red`; p2 has red 2/2 with `Color::Red`

4. **`test_702_13_intimidate_multicolor_attacker_allows_partial_color_match`**
   - CR 702.13b + multicolor ruling (Hideous Visage 2011-09-22)
   - White-blue creature with intimidate attacks. Green-white creature blocks.
   - Expected: `Ok` -- blocker shares white with the attacker.
   - Setup: p1 has 2/2 with `Intimidate` + `Color::White, Color::Blue`; p2 has 2/2 with `Color::Green, Color::White`

5. **`test_702_13_intimidate_colorless_attacker_only_artifact_can_block`**
   - CR 702.13b + CR 105.2c (colorless has no colors to share)
   - Colorless creature with intimidate attacks. Non-artifact creature (any color) tries to block.
   - Expected: `Err` -- colorless attacker shares no colors with any creature.
   - Setup: p1 has 2/2 with `Intimidate` and no colors; p2 has red 3/3 with `Color::Red`

6. **`test_702_13_intimidate_colorless_attacker_artifact_creature_blocks`**
   - CR 702.13b + CR 105.2c -- artifact creature can still block colorless intimidate
   - Colorless creature with intimidate attacks. Artifact creature blocks.
   - Expected: `Ok` -- artifact creature exception still applies.
   - Setup: p1 has 2/2 with `Intimidate` and no colors; p2 has 1/1 with `card_types: [Artifact, Creature]`

7. **`test_702_13_intimidate_plus_flying_both_must_be_satisfied`**
   - Interaction: flying + intimidate stack
   - Red creature with intimidate AND flying attacks. Red ground creature (no flying/reach) tries to block.
   - Expected: `Err` -- satisfies intimidate (shares red) but fails flying check.
   - Setup: p1 has 3/2 with `Intimidate, Flying` + `Color::Red`; p2 has 2/2 with `Color::Red` (no flying/reach)

**Pattern**: Follow the Flying tests at lines 238-321 and Menace tests at lines 543-635.
Each test:
1. Creates a `GameStateBuilder` with two players
2. Places creatures with appropriate keywords and colors
3. Sets `at_step(Step::DeclareBlockers)` and `active_player(p1)`
4. Manually sets up `state.combat` with attackers
5. Calls `process_command` with `Command::DeclareBlockers`
6. Asserts `is_err()` or `is_ok()` with descriptive message

**Imports needed**: `Color` and `CardType` must be imported (check existing imports at top
of `keywords.rs`).

### Step 5: Card Definition (later phase)

**Suggested card**: Bladetusk Boar
- **Oracle text**: "Intimidate (This creature can't be blocked except by artifact creatures and/or creatures that share a color with it.)"
- **Type**: Creature -- Boar
- **Mana cost**: {3}{R}
- **P/T**: 3/2
- **Colors**: Red
- **Card ID**: `bladetusk-boar`
- **Keywords**: `[KeywordAbility::Intimidate]`
- **Abilities**: `[AbilityDefinition::Keyword(KeywordAbility::Intimidate)]`
- **This is a vanilla intimidate creature** -- ideal for testing the core mechanic.
- **File**: `crates/engine/src/cards/definitions.rs` -- add as card #58 (after existing 57 cards)
- **Pattern**: Follow Birds of Paradise definition at line 1354 (simple creature with keyword)

**Card definition sketch**:
```rust
// 58. Bladetusk Boar -- {3R}, Creature -- Boar 3/2; Intimidate
CardDefinition {
    card_id: cid("bladetusk-boar"),
    name: "Bladetusk Boar".to_string(),
    mana_cost: Some(ManaCost { red: 1, generic: 3, ..Default::default() }),
    types: creature_types(&["Boar"]),
    oracle_text: "Intimidate (This creature can't be blocked except by artifact creatures and/or creatures that share a color with it.)".to_string(),
    power: Some(3),
    toughness: Some(2),
    abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Intimidate)],
    ..Default::default()
},
```

### Step 6: Game Script (later phase)

**Suggested scenario**: "Bladetusk Boar with intimidate attacks; non-matching creature fails
to block; artifact creature successfully blocks"
**Subsystem directory**: `test-data/generated-scripts/combat/`
**Filename**: `009_intimidate_blocking_restriction.json`
**Script ID**: `script_combat_009`

**Scenario outline**:
1. P1 controls Bladetusk Boar (3/2, red, intimidate) on the battlefield
2. P2 controls a white 2/2 creature AND a 1/1 artifact creature on the battlefield
3. P1 attacks with Bladetusk Boar targeting P2
4. P2 attempts to block with the white creature -- this should be rejected or skipped
5. P2 blocks with the artifact creature -- this should succeed
6. Combat damage resolves; assert artifact creature took damage

**Note**: The exact script format and step sequence depend on the game-script-generator agent.
The scenario should test both the negative case (color mismatch rejection) and the positive
case (artifact creature exception). If the harness doesn't support "expected failure" steps,
split into two scripts or use the positive case only and rely on unit tests for the negative.

## Interactions to Watch

1. **Flying + Intimidate**: Both are checked independently in the per-blocker loop. A blocker
   must satisfy BOTH restrictions. The sequential check in `handle_declare_blockers` handles
   this correctly -- flying is checked first (line 431-439), then intimidate would be checked.
   If either fails, the blocker is rejected.

2. **Protection + Intimidate**: Protection's blocking restriction (CR 702.16f) is also checked
   independently at line 455. The `can_block` check in `protection.rs` and the intimidate check
   are separate. Both must pass.

3. **CantBeBlocked + Intimidate**: If a creature has both `CantBeBlocked` and `Intimidate`, the
   `CantBeBlocked` check fires first (line 443-451) and rejects ALL blockers. Intimidate is
   redundant in this case. No special handling needed.

4. **Menace + Intimidate**: Both apply. Menace requires 2+ blockers; intimidate restricts which
   creatures can block. All blockers must individually satisfy intimidate, and the total count
   must be >= 2. The engine handles this because intimidate is checked per-blocker (in the
   per-pair loop) and menace is checked per-attacker (in the aggregate count loop after).

5. **Layer system**: Intimidate is checked via `calculate_characteristics`, which applies all
   continuous effects. If an effect removes intimidate (e.g., Humility), it won't appear in
   `keywords` and the check won't fire. If an effect adds intimidate (e.g., Hideous Visage),
   it will appear. This is correct behavior.

6. **Colorless creatures (Eldrazi)**: A colorless creature with intimidate is extremely hard
   to block -- only artifact creatures can block it. This is intentional per the rules (no
   colors to share means the "shares a color" exception never fires).
