# Ability Plan: Undaunted

**Generated**: 2026-02-27
**CR**: 702.125 (NOT 702.124 -- that is Partner; the ability-coverage doc has a stale CR number)
**Priority**: P3
**Similar abilities studied**: Affinity (CR 702.41) -- `crates/engine/src/rules/casting.rs:1917-1986`, `crates/engine/tests/affinity.rs`

## CR Rule Text

```
702.125. Undaunted

702.125a Undaunted is a static ability that functions while the spell with
         undaunted is on the stack. Undaunted means "This spell costs {1} less
         to cast for each opponent you have."

702.125b Players who have left the game are not counted when determining how
         many opponents you have.

702.125c If a spell has multiple instances of undaunted, each of them applies.
```

## Key Edge Cases

- **CR 702.125b -- Eliminated opponents do not count.** Only active (non-lost, non-conceded)
  opponents reduce the cost. The engine already tracks this via `has_lost` and `has_conceded`
  on `PlayerState`. Use `state.active_players()` and filter out the caster to get the
  opponent count. This is the primary Commander-relevant edge case: as players are eliminated,
  Undaunted reduces less.
- **CR 702.125c -- Multiple instances stack.** If a spell somehow has two instances of
  Undaunted (e.g., via a copy/gain effect), each applies independently. With 3 opponents
  and 2 instances: total reduction = 6. This mirrors Affinity's CR 702.41b cumulative behavior.
- **Generic mana floor at 0 (CR 601.2f).** Undaunted reduces only generic mana. Colored and
  colorless pips are unaffected. The generic component cannot go below 0.
- **Cost is locked at announcement time (ruling from multiple Undaunted cards).** "Causing an
  opponent to lose the game after you've announced that you're casting a spell with undaunted
  and determined its total cost won't cause you to have to pay more mana." This is inherently
  handled by the engine's casting pipeline -- cost is determined once during `handle_cast_spell`
  and never re-evaluated.
- **Multiplayer scaling.** In a 4-player Commander game: 3 opponents = {3} reduction. In 1v1:
  1 opponent = {1} reduction. In a 6-player game: 5 opponents = {5} reduction. If 2 players
  have been eliminated in a 4-player game, only 1 opponent remains = {1} reduction.
- **Interaction with commander tax.** Undaunted applies AFTER the total cost is determined
  (including commander tax), same as Affinity. A commander with Undaunted that costs {5}{W}
  with 2 commander tax ({2} added) = {7}{W} total, then Undaunted reduces by opponent count.
- **No player decisions required.** Like Affinity, Undaunted is automatic -- the engine counts
  opponents and applies the reduction. No UI/command interaction needed.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- `KeywordAbility::Undaunted` does NOT exist in `types.rs`
- [ ] Step 2: Rule enforcement -- no cost reduction logic in `casting.rs`
- [ ] Step 3: Trigger wiring -- N/A (static ability, not a trigger)
- [ ] Step 4: Unit tests -- none
- [ ] Step 5: Card definition -- none
- [ ] Step 6: Game script -- none
- [ ] Step 7: Coverage doc update -- none

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Undaunted` variant after the `Affinity(AffinityTarget)` variant
(currently at line 363). It is a simple unit variant with no parameters (unlike Affinity which
takes an `AffinityTarget`).

**Doc comment**:
```rust
/// CR 702.125: Undaunted -- "This spell costs {1} less to cast for each
/// opponent you have."
///
/// Static ability that functions while the spell is on the stack (CR 702.125a).
/// Automatically reduces generic mana in the total cost based on the number
/// of opponents the caster has. Multiple instances are cumulative (CR 702.125c).
/// Players who have left the game are not counted (CR 702.125b).
Undaunted,
```

**Pattern**: Follow `KeywordAbility::Affinity` at types.rs line 353-363 (static cost-reduction
keyword, no parameters).

### Step 1b: Hash Discriminant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add a new arm to the `HashInto for KeywordAbility` match block. The current highest
discriminant is 53 (Affinity). Use discriminant **54** for Undaunted.

**Code**:
```rust
// Undaunted (discriminant 54) -- CR 702.125
KeywordAbility::Undaunted => 54u8.hash_into(hasher),
```

**Location**: After the `KeywordAbility::Affinity(target)` arm at hash.rs line 409-412, before
the closing `}` at line 413.

### Step 1c: View Model Display

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add a new arm to the `keyword_to_string` match (or the inline match near line 632).

**Code**:
```rust
KeywordAbility::Undaunted => "Undaunted".to_string(),
```

**Location**: After the `KeywordAbility::Affinity(target)` arm (currently the last arm around
line 632-636).

### Step 1d: Exhaustiveness Check

After adding the new variant, the compiler will flag any other match expressions on
`KeywordAbility` that are missing the new arm. Grep for all match expressions:

```
Grep pattern="KeywordAbility::" path="crates/engine/src/" output_mode="files_with_matches"
```

Known locations that need new arms (besides hash.rs and view_model.rs):
- `crates/engine/src/state/builder.rs` -- triggered ability translation (Undaunted does NOT
  generate a trigger, so the match arm should be a no-op / fall-through with the other
  non-trigger keywords)

### Step 2: Rule Enforcement -- Cost Reduction in casting.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Add a new function `apply_undaunted_reduction` and call it in the casting pipeline
immediately after `apply_affinity_reduction` (line 695) and before the convoke section (line 697).

**CR**: 702.125a -- "This spell costs {1} less to cast for each opponent you have."
**CR**: 702.125b -- "Players who have left the game are not counted."
**CR**: 702.125c -- "If a spell has multiple instances of undaunted, each of them applies."
**CR**: 601.2f -- "The generic mana component cannot be reduced below 0."

**Pipeline insertion point** (after line 695):
```rust
// CR 702.125a: Apply undaunted cost reduction AFTER total cost is determined
// (including commander tax, kicker, affinity) and BEFORE convoke/improvise/delve.
// Undaunted is a static ability -- the engine counts opponents automatically.
// CR 702.125c: Multiple instances of undaunted are cumulative.
let mana_cost = apply_undaunted_reduction(state, player, &chars, mana_cost);
```

**New function** (add after `apply_affinity_reduction` / `matches_affinity_target`, near line 1987):
```rust
/// CR 702.125a: Apply undaunted cost reduction to the total mana cost.
///
/// For each instance of `KeywordAbility::Undaunted` on the spell,
/// count the number of opponents the caster has (CR 702.125b: only active
/// players who have not left the game), and reduce the generic mana
/// component by that count.
///
/// CR 702.125c: Multiple instances of undaunted are cumulative -- each one
/// independently counts opponents and reduces. Two instances with 3 opponents
/// = 6 generic mana reduction.
///
/// CR 601.2f: The generic mana component cannot be reduced below 0.
/// Colored and colorless pips are unaffected.
fn apply_undaunted_reduction(
    state: &GameState,
    player: PlayerId,
    chars: &Characteristics,
    cost: Option<ManaCost>,
) -> Option<ManaCost> {
    // Count how many instances of Undaunted the spell has.
    let undaunted_count = chars
        .keywords
        .iter()
        .filter(|kw| matches!(kw, KeywordAbility::Undaunted))
        .count() as u32;

    if undaunted_count == 0 {
        return cost;
    }

    // CR 601.2f: If the spell has no mana cost (None), there is nothing to reduce.
    let mut reduced = cost?;

    // CR 702.125b: Count only active players (not lost/conceded) who are NOT the caster.
    let opponent_count = state
        .active_players()
        .iter()
        .filter(|&&pid| pid != player)
        .count() as u32;

    // CR 702.125c: Each instance applies independently.
    let total_reduction = undaunted_count * opponent_count;

    // CR 601.2f: Generic cannot go below 0.
    let reduction = total_reduction.min(reduced.generic);
    reduced.generic -= reduction;

    Some(reduced)
}
```

**Pattern**: Directly follows `apply_affinity_reduction` (lines 1917-1952). Same structure:
count keyword instances, early return if none, early return if no mana cost, compute
reduction, floor at 0.

**Key difference from Affinity**: Undaunted counts opponents (from `state.active_players()`
filtered by `!= player`), not permanents. No `AffinityTarget` parameter -- always counts
opponents. Simpler than Affinity.

### Step 3: Trigger Wiring

**N/A** -- Undaunted is a static ability that modifies the casting cost. It does not generate
triggers. No work needed in `builder.rs` trigger translation, `abilities.rs` trigger dispatch,
or `events.rs`.

The `builder.rs` match on KeywordAbility will need a no-op arm (or wildcard coverage) for
`Undaunted`, but that is covered in Step 1d's exhaustiveness check.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/undaunted.rs`
**Pattern**: Follow `crates/engine/tests/affinity.rs` (12 tests, 861 lines). Undaunted tests
will be simpler because there is no target parameterization.

**Tests to write**:

1. **`test_undaunted_basic_4player_reduce_generic_cost`** -- CR 702.125a
   4-player game (3 opponents). Spell {6}{W} with Undaunted. Reduced cost: {3}{W}. Pay {3}{W}.
   Verify spell goes on stack and mana pool is depleted correctly.

2. **`test_undaunted_reduce_to_zero`** -- CR 702.125a + CR 601.2f
   4-player game (3 opponents). Spell {3}{W} with Undaunted. Generic {3} - 3 = {0}. Pay {W} only.

3. **`test_undaunted_excess_opponents_floors_at_zero`** -- CR 601.2f
   6-player game (5 opponents). Spell {3}{W} with Undaunted. Generic {3} - 5 = {0} (floors, not -2).
   Pay {W} only.

4. **`test_undaunted_does_not_reduce_colored_pips`** -- CR 702.125a + CR 601.2f
   4-player game (3 opponents). Spell {2}{W}{U} with Undaunted. Generic {2} - 3 = {0}. Must
   still pay {W}{U}. Colored pips unaffected.

5. **`test_undaunted_no_keyword_no_reduction`** -- Negative test
   4-player game. Spell {6}{W} WITHOUT Undaunted. Full {6}{W} must be paid. Should fail with
   only {3}{W} in pool.

6. **`test_undaunted_2player_one_opponent`** -- CR 702.125a
   2-player game (1 opponent). Spell {6}{W} with Undaunted. Reduced cost: {5}{W}. Pay {5}{W}.

7. **`test_undaunted_eliminated_opponents_not_counted`** -- CR 702.125b
   4-player game. One opponent has `has_lost = true`. Only 2 active opponents remain. Spell
   {6}{W} with Undaunted. Reduced cost: {4}{W}. Pay {4}{W}.

8. **`test_undaunted_multiple_instances_cumulative`** -- CR 702.125c
   4-player game (3 opponents). Spell {8} with TWO instances of Undaunted (note: OrdSet
   deduplicates, so this test must verify the OrdSet behavior -- if it deduplicates, the test
   documents that limitation; if multiple instances are stored, it verifies cumulative behavior).
   **Important note**: Because `keywords` is likely an `OrdSet<KeywordAbility>`, the unit
   variant `Undaunted` will be deduplicated. This means CR 702.125c cannot be naturally
   exercised with the current data model. The test should document this as a known limitation
   (same as Affinity's note about OrdSet deduplication in test 10 of affinity.rs). In practice,
   no printed card has two instances of Undaunted, so this is theoretical only.

9. **`test_undaunted_with_commander_tax`** -- CR 702.125a + CR 903.8
   4-player game (3 opponents). Commander {4}{W} with Undaunted in command zone. 1 prior cast
   (tax = {2}). Total cost: {6}{W}. Undaunted: {6} - 3 = {3}. Pay {3}{W}.

10. **`test_undaunted_combined_with_affinity`** -- Interaction test
    4-player game (3 opponents). Spell {8}{U} with BOTH Undaunted AND Affinity for Artifacts.
    Player controls 2 artifacts. Affinity: {8} - 2 = {6}. Undaunted: {6} - 3 = {3}. Pay {3}{U}.
    Verifies both cost reductions compose correctly in the pipeline.

11. **`test_undaunted_6player_game`** -- Multiplayer scaling
    6-player game (5 opponents). Spell {6}{W} with Undaunted. Reduced cost: {1}{W}. Pay {1}{W}.
    Verifies correct scaling at N=6.

**Helper functions** (following affinity.rs pattern):
- `fn p(n: u64) -> PlayerId` -- player ID shorthand
- `fn cid(s: &str) -> CardId` -- card ID shorthand
- `fn find_object(state: &GameState, name: &str) -> ObjectId` -- object lookup
- `fn undaunted_spell_spec(owner: PlayerId, name: &str, generic: u32, white: u32) -> ObjectSpec`
  -- sorcery with Undaunted keyword and specified mana cost
- `fn cast_spell(state, player, card) -> Result<...>` -- CastSpell command shorthand

### Step 5: Card Definition (later phase)

**Suggested card**: Sublime Exhalation
- **Mana cost**: {6}{W}
- **Type**: Sorcery
- **Oracle text**: "Undaunted (This spell costs {1} less to cast for each opponent you have.) / Destroy all creatures."
- **Color identity**: W
- **Keywords**: Undaunted
- **Effect**: DestroyAllCreatures (or equivalent Effect composition)
- **Card lookup**: use `card-definition-author` agent
- **Note**: The user prompt incorrectly stated "Exile all creatures" -- the actual oracle text
  is "Destroy all creatures."

### Step 6: Game Script (later phase)

**Suggested scenario**: 4-player Commander game. Player 1 casts Sublime Exhalation ({6}{W}).
With 3 opponents, Undaunted reduces cost to {3}{W}. Verify the spell resolves and all creatures
are destroyed.

**Secondary scenario**: Same game but one opponent has been eliminated (P4 lost). Undaunted
now reduces by only 2. Cost becomes {4}{W}.

**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Affinity + Undaunted composition**: Both are static cost-reduction abilities. The pipeline
  order matters: Affinity first (line 695), then Undaunted. Both reduce generic mana only. The
  final generic cost is the result of both reductions chained. Tests should verify this works.
- **Commander tax**: Undaunted applies AFTER tax is added to the total cost (same insertion
  point as Affinity). Tax is part of the base cost that gets reduced.
- **Convoke/Improvise/Delve**: These apply AFTER Undaunted in the pipeline. Undaunted reduces
  the generic mana, then Convoke/Improvise/Delve can further reduce or pay the remaining cost.
- **Split Second interaction**: If a Split Second spell is on the stack, players cannot cast
  spells. This blocks casting Undaunted spells entirely -- no special interaction needed.
- **Multiplayer elimination**: As opponents leave the game (`has_lost` / `has_conceded`),
  Undaunted's reduction decreases. This is naturally handled by using `active_players()`.
- **No `OrdSet` deduplication concern for single instances**: Since Undaunted is a unit variant
  (no parameters), OrdSet will deduplicate identical copies. In practice, no card has multiple
  instances of Undaunted. This matches the Affinity precedent where identical-target instances
  are deduplicated by OrdSet.
