# Ability Plan: Fear

**Generated**: 2026-02-27
**CR**: 702.36
**Priority**: P3
**Similar abilities studied**: Intimidate (CR 702.13) -- same evasion pattern, validated, `state/types.rs:114`, `rules/combat.rs:453-474`, 7 tests in `tests/keywords.rs:654-991`, card def Bladetusk Boar, script `combat/009`

## CR Rule Text

```
702.36. Fear

702.36a Fear is an evasion ability.

702.36b A creature with fear can't be blocked except by artifact creatures and/or
        black creatures. (See rule 509, "Declare Blockers Step.")

702.36c Multiple instances of fear on the same creature are redundant.
```

## Key Edge Cases

- **Artifact creatures always qualify**: Any creature with CardType::Artifact + CardType::Creature can block regardless of color (CR 702.36b "artifact creatures and/or black creatures"). Colorless artifact creatures, colored artifact creatures, black artifact creatures -- all qualify.
- **Black creatures always qualify**: Any creature with Color::Black can block regardless of other types. A multicolored creature that includes black qualifies.
- **Attacker's color is irrelevant**: Unlike Intimidate (which checks shared colors), Fear only checks if the blocker is artifact or black. A red creature with Fear cannot be blocked by a red creature -- shared color does not help (already tested).
- **Colorless non-artifact creatures cannot block**: An Eldrazi Spawn token (colorless, non-artifact) cannot block a Fear creature (already tested).
- **Multiple evasion stacking**: If a creature has both Fear and Flying, the blocker must satisfy BOTH restrictions (must be artifact/black AND have flying/reach). Same pattern as Intimidate + Flying (tested at `tests/keywords.rs:944-991`).
- **Redundant instances**: Multiple instances of Fear have no additional effect (CR 702.36c). No special engine handling needed -- the blocking check is boolean (has Fear or not).
- **Multiplayer**: Fear works identically in multiplayer. Each defending player's blockers are independently checked against the restriction. No special multiplayer logic needed.

## Current State (from ability-wip.md + code verification)

- [x] Step 1: Enum variant -- exists at `crates/engine/src/state/types.rs:385` with CR comment (702.36a/b/c)
- [x] Step 2: Rule enforcement -- exists at `crates/engine/src/rules/combat.rs:475-489` (CR 702.36b blocking restriction)
- [x] Step 3: Hash discriminant -- exists at `crates/engine/src/state/hash.rs:394-395` (discriminant 46)
- [x] Step 3b: View model -- exists at `tools/replay-viewer/src/view_model.rs:625` ("Fear" string)
- [x] Step 4: Unit tests -- 5 tests exist at `crates/engine/tests/keywords.rs:1818-2047`:
  - `test_702_36_fear_blocks_non_matching_creature` (white creature cannot block)
  - `test_702_36_fear_allows_artifact_creature_blocker` (artifact creature can block)
  - `test_702_36_fear_allows_black_creature_blocker` (black creature can block)
  - `test_702_36_fear_attacker_color_irrelevant` (red attacker + red blocker = still blocked)
  - `test_702_36_fear_colorless_non_artifact_cannot_block` (colorless non-artifact cannot block)
- [x] Step 5: Card definition -- Severed Legion exists at `crates/engine/src/cards/definitions.rs:1761-1773` ({1}{B}{B}, Creature -- Zombie 2/2, Fear)
- [x] Step 6: Game script -- exists at `test-data/generated-scripts/combat/080_fear_blocking_restriction.json` (review_status: `pending_review`; uses Severed Legion + Solemn Simulacrum + Bladetusk Boar + Bog Raiders; 3 CR scenarios)
- [ ] Step 7: Coverage doc update -- `docs/mtg-engine-ability-coverage.md` line 74 still shows `none`

## Implementation Steps

### Steps 1-3: COMPLETE (enum, enforcement, hash, view model)

All infrastructure is already in place. No code changes needed.

- **Enum**: `KeywordAbility::Fear` at `crates/engine/src/state/types.rs:385`
- **Combat enforcement**: `crates/engine/src/rules/combat.rs:475-489` -- checks `blocker_is_artifact_creature` (CardType::Artifact + CardType::Creature) and `blocker_is_black` (Color::Black); returns `InvalidCommand` if neither is true
- **Hash**: discriminant 46 at `crates/engine/src/state/hash.rs:394-395`
- **View model**: "Fear" string at `tools/replay-viewer/src/view_model.rs:625`

### Step 4: Unit Tests -- MOSTLY COMPLETE (5 of 7 tests exist)

Five unit tests already exist at `crates/engine/tests/keywords.rs:1818-2047`. Two additional tests should be added for completeness parity with Intimidate (which has 7 tests):

**File**: `crates/engine/tests/keywords.rs`
**Location**: Append after line 2047 (end of existing Fear section)
**Imports**: All needed imports already present in the file header (lines 15-21)

#### Test 6: `test_702_36_fear_allows_black_artifact_creature_blocker`

**CR**: 702.36b -- documents the "and/or" in the rule text
**Action**: A creature that is both black AND an artifact creature can block a Fear attacker. This is trivially true (either condition alone suffices) but documents the dual-qualifying case.
**Pattern**: Same as `test_702_36_fear_allows_artifact_creature_blocker` at line 1864 but add `.with_colors(vec![Color::Black])` to the blocker's `ObjectSpec`:

```rust
#[test]
/// CR 702.36b -- A black artifact creature satisfies both exceptions and can block.
fn test_702_36_fear_allows_black_artifact_creature_blocker() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Fear Attacker", 2, 2)
                .with_keyword(KeywordAbility::Fear)
                .with_colors(vec![Color::Black]),
        )
        .object(
            ObjectSpec::creature(p2, "Black Artifact Blocker", 1, 4)
                .with_types(vec![CardType::Artifact, CardType::Creature])
                .with_colors(vec![Color::Black]),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Fear Attacker");
    let blocker_id = find_object(&state, "Black Artifact Blocker");

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

    assert!(
        result.is_ok(),
        "A black artifact creature should be able to block a fear attacker (CR 702.36b): {:?}",
        result.err()
    );
}
```

#### Test 7: `test_702_36_fear_plus_flying_both_must_be_satisfied`

**CR**: 702.36b + CR 702.9a -- combined evasion; blocker must satisfy ALL restrictions (CR 509.1b)
**Action**: A creature with both Fear and Flying. A black ground creature satisfies Fear but not Flying. Assert `is_err()`.
**Pattern**: Follow `test_702_13_intimidate_plus_flying_both_must_be_satisfied` at line 944-991.

```rust
#[test]
/// CR 702.36b + CR 702.9a -- A creature with both fear and flying requires
/// the blocker to satisfy BOTH restrictions. A black ground creature fails
/// because it satisfies fear but not flying.
fn test_702_36_fear_plus_flying_both_must_be_satisfied() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Flying Fear Creature", 3, 2)
                .with_keyword(KeywordAbility::Fear)
                .with_keyword(KeywordAbility::Flying)
                .with_colors(vec![Color::Black]),
        )
        .object(
            ObjectSpec::creature(p2, "Black Ground Creature", 2, 2)
                .with_colors(vec![Color::Black]),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Flying Fear Creature");
    let blocker_id = find_object(&state, "Black Ground Creature");

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

    assert!(
        result.is_err(),
        "A ground creature should not block flying+fear even if it is black"
    );
}
```

### Step 5: Card Definition -- COMPLETE

Severed Legion exists at `crates/engine/src/cards/definitions.rs:1761-1773`.
- Name: "Severed Legion"
- Card ID: `cid("severed-legion")`
- Mana cost: {1}{B}{B}
- Types: `creature_types(&["Zombie"])`
- Oracle text: "Fear (...)"
- P/T: 2/2
- Abilities: `[AbilityDefinition::Keyword(KeywordAbility::Fear)]`

No additional card definition needed.

### Step 6: Game Script -- COMPLETE (needs validation and approval)

Script exists at `test-data/generated-scripts/combat/080_fear_blocking_restriction.json`.

**Validation command** (run from `/home/airbaggie/scutemob/crates/engine`):
```bash
SCRIPT_FILTER=080_fear_blocking_restriction ~/.cargo/bin/cargo test --test run_all_scripts -- --nocapture
```

**Actions after validation**:
1. If passing: update `review_status` from `"pending_review"` to `"approved"`, set `"reviewed_by": "ability-impl-runner"`, set `"review_date": "2026-02-27"`.
2. If failing: diagnose the failure. **Known potential issue**: the JSON at lines 73-76, 153-154, and 210-211 has duplicate `zones.battlefield.p2` keys in the same assertions object. JSON spec says duplicate keys produce undefined behavior; most parsers keep only the last one. If this causes assertion failures, the assertions need restructuring (use a single `zones.battlefield.p2` with `"includes"` containing multiple cards in one array).

### Step 7: Coverage Doc Update

**File**: `docs/mtg-engine-ability-coverage.md`

**Line 74 -- replace**:
```
| Fear | 702.36 | P3 | `none` | --- | --- | --- | --- | Can't be blocked except by artifact/black creatures |
```
**with**:
```
| Fear | 702.36 | P3 | `validated` | `state/types.rs:385`, `rules/combat.rs:475-489` | Severed Legion | `combat/080` | --- | CR 702.36b blocking restriction enforced (artifact creature OR black creature); 7 unit tests in `tests/keywords.rs:1818`; game script approved |
```

**Line ~387 -- update evasion pattern row to include Fear**:
```
| Evasion / blocking restriction | `rules/combat.rs` | Flying, Menace, Intimidate, Fear |
```

### Step 8: Update ability-wip.md

**File**: `memory/ability-wip.md`

Mark all steps as complete and set `phase: done`:
```markdown
# Ability WIP: Fear

ability: Fear
cr: 702.36
priority: P3
started: 2026-02-27
phase: done
plan_file: memory/ability-plan-fear.md

## Step Checklist
- [x] 1. Enum variant
- [x] 2. Rule enforcement
- [x] 3. Trigger wiring (N/A -- static evasion)
- [x] 4. Unit tests
- [x] 5. Card definition
- [x] 6. Game script
- [x] 7. Coverage doc update
```

## Interactions to Watch

- **Fear + Flying**: Both restrictions apply simultaneously. Blocker must be (artifact OR black) AND (flying OR reach). The combat.rs code checks each evasion restriction independently in sequence, which correctly enforces the conjunction.
- **Fear + Menace**: Blocker count must be >= 2 AND each blocker must individually be artifact or black. Menace is checked separately in combat.rs; Fear check applies per-blocker.
- **Fear + Protection**: Protection from a quality prevents blocking by creatures with that quality (CR 702.16f). If a creature has both Fear and Protection from green, it is doubly unblockable by green non-artifact non-black creatures. Each check is independent.
- **Intimidate vs Fear on the same creature**: Both would apply. A blocker must satisfy both: (artifact OR black) AND (artifact OR shares-a-color). The intersection depends on the attacker's colors. No code change needed.
- **Changeling + Fear interaction**: A Changeling creature is every creature type, but Changeling does not add `CardType::Artifact`. The Changeling's color determines whether it can block a fear creature. No special handling needed.
- **Layer system**: Fear is a static keyword. If a layer effect removes all abilities (Humility), Fear is removed and the blocking restriction no longer applies. Handled automatically by `calculate_characteristics`.

## Runner Summary

The runner's workload is minimal -- Fear is essentially fully implemented. Remaining tasks:

1. **Add 2 unit tests** for completeness (black artifact creature blocker, Fear + Flying combined evasion). Append after line 2047 in `crates/engine/tests/keywords.rs`. This brings the test count to 7, matching Intimidate.
2. **Validate and approve the game script** (`080_fear_blocking_restriction.json`). Watch for the duplicate JSON key issue in assertion blocks.
3. **Update the coverage doc** (`docs/mtg-engine-ability-coverage.md` line 74 and the evasion pattern row around line 387).
4. **Update `memory/ability-wip.md`** to mark all steps complete and set phase to `done`.
5. **Run `cargo test --all` and `cargo clippy -- -D warnings`** to confirm everything passes.
