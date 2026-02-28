# Ability Plan: Shadow

**Generated**: 2026-02-28
**CR**: 702.28
**Priority**: P4
**Batch**: 1 (Low effort keyword cleanup), item 1.1
**Similar abilities studied**: Fear (CR 702.36), Intimidate (CR 702.13), Flying (CR 702.9) -- all evasion blocking restrictions in `rules/combat.rs`

## CR Rule Text

702.28. Shadow

702.28a Shadow is an evasion ability.

702.28b A creature with shadow can't be blocked by creatures without shadow, and a creature without shadow can't be blocked by creatures with shadow. (See rule 509, "Declare Blockers Step.")

702.28c Multiple instances of shadow on the same creature are redundant.

## Key Edge Cases

- **Bidirectional restriction (CR 702.28b)**: Shadow is unique among evasion abilities because it restricts blocking in BOTH directions. A creature with shadow cannot block a creature without shadow, AND a creature without shadow cannot block a creature with shadow. This is unlike Flying/Fear/Intimidate which only restrict blocking of the attacker.
- **Multiple evasion abilities must ALL be satisfied (Dauthi Voidwalker ruling 2021-06-18)**: "If an attacking creature has multiple evasion abilities, such as shadow and flying, a creature can block it only if that creature satisfies all of the appropriate evasion abilities." A creature with shadow+flying can only be blocked by a creature with shadow AND (flying or reach).
- **Blocking persists after shadow changes (Soltari Monk ruling 2021-03-19)**: "Once a creature has been blocked, that creature remains blocked and will deal and be dealt combat damage even if it gains or loses shadow or if the blocking creature gains or loses shadow." This is standard blocking behavior (CR 509.1h) and already enforced by the engine -- no special Shadow code needed.
- **Redundancy (CR 702.28c)**: Multiple instances of shadow are redundant. Already handled by `im::OrdSet` deduplication in the keywords set.
- **Keyword counters (CR 122.1b)**: Shadow is in the keyword counter list. A keyword counter with shadow causes the object to gain shadow. No special handling needed -- the layer system already processes keyword counters.
- **Multiplayer**: No special multiplayer considerations. Shadow restricts blocking for each individual attacker-blocker pair regardless of the number of players.

## Current State (from ability-wip.md)

ability-wip.md currently tracks Bolster, not Shadow. Shadow has no prior work.

- [ ] Step 1: Enum variant -- does NOT exist in `KeywordAbility`
- [ ] Step 2: Rule enforcement -- no blocking check in `combat.rs`
- [ ] Step 3: Trigger wiring -- N/A (shadow is a static keyword, no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Shadow` variant
**Pattern**: Follow `KeywordAbility::Fear` at line 392 (simple unit variant, no parameters)
**Insert location**: After `Hideaway(u32)` (line 559), before the closing brace of the enum. Or alphabetically near the other evasion keywords. Recommended: add after `Fear` (line 392) to keep evasion abilities grouped, but any position within the enum is fine since ordering doesn't affect semantics. Best approach: add at the end of the enum (before line 560 closing brace) to minimize diff noise, matching the convention used by all recent additions.

Add this variant:

```rust
/// CR 702.28: Shadow -- evasion ability.
/// "A creature with shadow can't be blocked by creatures without shadow,
/// and a creature without shadow can't be blocked by creatures with shadow."
/// CR 702.28c: Multiple instances are redundant (auto-deduped by OrdSet).
Shadow,
```

### Step 1b: Hash discriminant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Shadow` arm to `HashInto for KeywordAbility` match block
**Pattern**: Follow `KeywordAbility::Hideaway(n)` at line 441 (last entry, discriminant 66)
**Next discriminant**: 67

Add:

```rust
// Shadow (discriminant 67) -- CR 702.28
KeywordAbility::Shadow => 67u8.hash_into(hasher),
```

### Step 1c: View model format

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::Shadow` arm to `format_keyword` match block
**Pattern**: Follow `KeywordAbility::Hideaway(n)` at line 672 (last entry)

Add:

```rust
KeywordAbility::Shadow => "Shadow".to_string(),
```

### Step 1d: Match arm exhaustiveness

**Action**: Grep for all `match` expressions on `KeywordAbility` and add `Shadow` arm where needed. The compiler will catch these as exhaustiveness errors if any are missed.

```
Grep pattern="KeywordAbility::" path="crates/engine/src/" output_mode="files_with_matches"
```

Key files to check:
- `state/hash.rs` (covered in Step 1b)
- `state/builder.rs` -- keyword-to-trigger translations; Shadow has no trigger, so it should be handled by the existing catch-all `_ => {}` arm or explicitly listed as a no-op
- `rules/layers.rs` -- `calculate_characteristics` keyword processing; Shadow is not a CDA, no layer interaction needed
- `effects/mod.rs` -- if there's a keyword-dispatch match, add Shadow as no-op
- Any other file with a match on KeywordAbility variants

### Step 2: Rule Enforcement

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs`
**Action**: Add Shadow blocking restriction check in `handle_declare_blockers`
**Location**: After the Fear check (line 489) and before the Protection check (line 491). This groups all evasion keyword checks together.
**CR**: 702.28b -- bidirectional blocking restriction

The Shadow check is unique because it has TWO restrictions (unlike Fear/Intimidate which have one):

1. If the attacker has shadow and the blocker does not have shadow, the block is illegal.
2. If the attacker does NOT have shadow and the blocker DOES have shadow, the block is illegal.

These can be combined into a single check: if attacker_has_shadow != blocker_has_shadow, the block is illegal.

Insert after line 489 (after the Fear check closing brace):

```rust
// CR 702.28b: Shadow is a bidirectional evasion ability.
// A creature with shadow can't be blocked by creatures without shadow,
// and a creature without shadow can't be blocked by creatures with shadow.
let attacker_has_shadow = attacker_chars.keywords.contains(&KeywordAbility::Shadow);
let blocker_has_shadow = blocker_chars.keywords.contains(&KeywordAbility::Shadow);
if attacker_has_shadow != blocker_has_shadow {
    return Err(GameStateError::InvalidCommand(format!(
        "Object {:?} cannot block {:?} (shadow mismatch: attacker shadow={}, blocker shadow={})",
        blocker_id, attacker_id, attacker_has_shadow, blocker_has_shadow
    )));
}
```

**Note on ordering**: The check MUST be placed in the per-pair loop (lines 388-525), after `blocker_chars` and `attacker_chars` are computed (line 429+). Placing it after Fear (line 489) and before Protection (line 491) is ideal.

### Step 3: Trigger Wiring

**N/A** -- Shadow is a purely static evasion keyword. It has no triggered, activated, or replacement effect components. No wiring needed in `builder.rs`, `abilities.rs`, or `effects/mod.rs`.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/keywords.rs`
**Pattern**: Follow the Fear test block (lines 1818-2149) exactly
**Insert location**: After the Fear+Flying combined test (line 2149), before the Wither section (line 2151). Add a new section header comment.

**Tests to write** (7 tests, matching the Fear/Intimidate pattern):

#### 4.1: `test_702_28_shadow_creature_cannot_be_blocked_by_non_shadow`
- CR 702.28b -- first half of bidirectional restriction
- Setup: P1 has a shadow creature attacking, P2 has a non-shadow creature blocking
- Assert: DeclareBlockers returns Err

#### 4.2: `test_702_28_shadow_creature_can_be_blocked_by_shadow`
- CR 702.28b -- shadow creatures CAN block each other
- Setup: P1 has a shadow creature attacking, P2 has a shadow creature blocking
- Assert: DeclareBlockers returns Ok

#### 4.3: `test_702_28_non_shadow_creature_cannot_be_blocked_by_shadow`
- CR 702.28b -- second half of bidirectional restriction (unique to shadow)
- Setup: P1 has a non-shadow creature attacking, P2 has a shadow creature trying to block
- Assert: DeclareBlockers returns Err

#### 4.4: `test_702_28_non_shadow_can_block_non_shadow`
- Baseline: two non-shadow creatures can block each other normally
- Setup: P1 has a non-shadow creature attacking, P2 has a non-shadow creature blocking
- Assert: DeclareBlockers returns Ok

#### 4.5: `test_702_28_shadow_plus_flying_both_must_be_satisfied`
- CR 702.28b + CR 702.9a -- multiple evasion abilities compound
- Setup: P1 has a creature with shadow+flying attacking, P2 has a shadow (but no flying/reach) creature blocking
- Assert: DeclareBlockers returns Err (shadow satisfied but flying not satisfied)

#### 4.6: `test_702_28_shadow_plus_flying_satisfied_by_shadow_flying`
- CR 702.28b + CR 702.9a -- blocker with both shadow and flying can block
- Setup: P1 has a creature with shadow+flying, P2 has a creature with shadow+flying
- Assert: DeclareBlockers returns Ok

#### 4.7: `test_702_28_shadow_plus_flying_satisfied_by_shadow_reach`
- CR 702.28b + CR 702.9a + CR 702.17 -- shadow+reach satisfies shadow+flying
- Setup: P1 has a creature with shadow+flying, P2 has a creature with shadow+reach
- Assert: DeclareBlockers returns Ok

**Test structure** (each test follows the exact same pattern as `test_702_36_fear_blocks_non_matching_creature` at line 1826):

```rust
#[test]
/// CR 702.28b — <description>
fn test_702_28_shadow_<scenario>() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "<Attacker Name>", <p>, <t>)
                .with_keyword(KeywordAbility::Shadow),
        )
        .object(
            ObjectSpec::creature(p2, "<Blocker Name>", <p>, <t>),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "<Attacker Name>");
    let blocker_id = find_object(&state, "<Blocker Name>");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(result.is_err() /* or is_ok() */, "<CR citation message>");
}
```

### Step 5: Card Definition (later phase)

**Suggested card**: Dauthi Slayer
- **Oracle text**: "Shadow / Dauthi Slayer attacks each combat if able."
- **Why**: Simple 2-drop with shadow + one straightforward non-keyword ability. The "attacks each combat if able" restriction is a nice secondary ability to test but does not require new engine infrastructure (it's a constraint on the controller, not enforced by the rules engine in the current scope).
- **Alternative**: Soltari Monk ({W}{W} 2/1, Protection from black, Shadow) -- slightly more interesting but depends on Protection + Shadow interacting cleanly.
- **Stretch**: Dauthi Voidwalker ({B}{B} 3/2, Shadow + graveyard exile replacement + activated ability) -- complex, better for later.

**Card lookup**: use `card-definition-author` agent with "Dauthi Slayer"

### Step 6: Game Script (later phase)

**Suggested scenario**: "Shadow Evasion"
- P1 controls a 2/2 Shadow creature and a 3/3 non-shadow creature, both attacking
- P2 controls a 2/2 Shadow creature and a 2/2 non-shadow creature
- P2 declares blockers: shadow blocks shadow, non-shadow blocks non-shadow (both legal)
- Verify: combat damage resolves correctly with proper pairings
- Edge case step: P2 tries to have non-shadow block shadow -- rejected

**Subsystem directory**: `test-data/generated-scripts/combat/`
**Suggested filename**: `110_shadow_evasion.json` (next available number in combat scripts)

## Interactions to Watch

- **Shadow + Flying (ruling)**: Both evasion abilities must be satisfied. The engine already processes evasion checks sequentially (Flying check at line 434, then Fear at line 477, etc.). Adding Shadow as another check in the sequence automatically enforces the compound requirement -- if any single check fails, the block is rejected. No special "combine evasion" logic needed.
- **Shadow + Protection**: A creature with shadow and protection from a quality still follows both restrictions. Protection prevents blocking by matching sources (DEBT), and shadow prevents blocking by non-shadow creatures. Both checks are independent and sequential.
- **Shadow + CantBeBlocked**: CantBeBlocked is checked at line 443 (before all evasion checks). A creature with both shadow and CantBeBlocked is simply unblockable. No conflict.
- **Shadow + Menace**: Menace requires 2+ blockers (checked at line 528, after the per-pair loop). If a shadow creature also has menace, all blockers must have shadow AND there must be 2+. The sequential checks handle this correctly.
- **Shadow + Changeling**: Changeling adds all creature types but does not interact with shadow (shadow is a keyword ability, not a creature type). No conflict.
- **Multiplayer**: No special handling. Each blocker-attacker pair is validated independently in the per-pair loop (lines 388-525). Shadow check applies to each pair.

## Estimated Effort

**Low** -- this is a textbook "add enum variant + add one check in combat.rs + write tests" ability. The blocking restriction is simpler than Fear/Intimidate (no color/type checks, just a boolean match). The bidirectional aspect is the only wrinkle and it's handled by a single `!=` comparison.

**Files modified**: 4 (types.rs, hash.rs, combat.rs, keywords.rs test file)
**Files touched for exhaustiveness**: view_model.rs + any other match blocks (compiler will flag)
**New lines of code**: ~15 (enforcement) + ~250 (7 tests) + ~10 (enum/hash/view_model)
**Risk**: Minimal. No new patterns, no new infrastructure, no trigger wiring.
