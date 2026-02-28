# Ability Plan: Affinity

**Generated**: 2026-02-27
**CR**: 702.41
**Priority**: P3
**Similar abilities studied**: Improvise (CR 702.126) -- `crates/engine/src/rules/casting.rs:1396-1508`, `crates/engine/tests/improvise.rs`, `crates/engine/src/state/types.rs:333-338`

## CR Rule Text

> **702.41. Affinity**
>
> **702.41a** Affinity is a static ability that functions while the spell with affinity is on the stack. "Affinity for [text]" means "This spell costs {1} less to cast for each [text] you control."
>
> **702.41b** If a spell has multiple instances of affinity, each of them applies.

## Key Edge Cases

- **Multiple instances are cumulative** (CR 702.41b). If a spell has two instances of "affinity for artifacts," each one independently reduces the cost by 1 per artifact. (Confirmed by Mycosynth Golem ruling 2004-12-01: "Two or more instances of the affinity ability are cumulative, even if they're both affinity for the same thing.")
- **Cost cannot go below {0}** (CR 601.2f): "If the mana component of the total cost is reduced to nothing by cost reduction effects, it is considered to be {0}. It can't be reduced to less than {0}." Affinity only reduces generic mana. Color pips are preserved.
- **Affinity reduces generic mana only.** The reminder text "costs {1} less" means one generic mana per qualifying permanent. Colored/colorless pips are unaffected.
- **Counts ALL permanents of the type you control** -- tapped or untapped. Unlike Improvise (which taps artifacts), Affinity just counts. An artifact tapped for Improvise still contributes to the Affinity count.
- **Affinity is automatic** -- unlike Convoke/Improvise/Delve, the player does not choose objects. The engine counts the qualifying permanents and reduces the cost. No new `CastSpell` parameter needed.
- **Affinity applies as part of total cost determination** (CR 601.2f), BEFORE convoke/improvise/delve payment methods. The pipeline order: base cost -> alt cost -> commander tax -> kicker -> **affinity reduction** -> convoke -> improvise -> delve -> pay.
- **Affinity for a sacrificed permanent**: If the spell being cast requires sacrificing a permanent as an additional cost, and that permanent was counted for affinity, the cost reduction was already locked in (Mycosynth Golem ruling 2004-12-01).
- **Multiplayer**: Affinity counts only permanents the caster controls, not all players' permanents. No special multiplayer considerations.
- **Affinity for different qualities**: "Affinity for artifacts" is the most common, but "affinity for [quality]" can be any permanent type or characteristic. The `AffinityTarget` enum should support at least `Artifacts` (vast majority of cards), with extensibility for other targets.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (N/A -- Affinity is a static ability, not triggered)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant + AffinityTarget Type

**File**: `crates/engine/src/state/types.rs`
**Action**: Add an `AffinityTarget` enum and a `KeywordAbility::Affinity(AffinityTarget)` variant.

```rust
/// CR 702.41a: Specifies the quality for affinity cost reduction.
///
/// "Affinity for [text]" means "This spell costs {1} less to cast for
/// each [text] you control." The quality determines which permanents
/// on the battlefield are counted for the reduction.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AffinityTarget {
    /// "Affinity for artifacts" -- count artifacts you control (most common).
    Artifacts,
    /// "Affinity for [basic land type]" -- count lands of that subtype you control.
    /// Example: "Affinity for Plains" counts all lands with the Plains subtype.
    BasicLandType(SubType),
}
```

Add `AffinityTarget` before `KeywordAbility` (after `EnchantTarget`, around line 137).

Add `KeywordAbility::Affinity(AffinityTarget)` variant to the `KeywordAbility` enum:

```rust
/// CR 702.41: Affinity for [quality] -- this spell costs {1} less for each
/// [quality] you control.
///
/// Static ability that functions while the spell is on the stack (CR 702.41a).
/// Automatically reduces generic mana in the total cost based on the count
/// of qualifying permanents the caster controls. Multiple instances are
/// cumulative (CR 702.41b).
Affinity(AffinityTarget),
```

**Insert location**: After `Unearth` (line ~415), or alphabetically among the keyword variants. Since existing keywords are roughly grouped by function, insert after `Improvise` (line 338) as it is another cost-modifier keyword.

**Pattern**: Follow `KeywordAbility::Annihilator(u32)` at line 269 for a parameterized variant, but with a custom enum payload instead of `u32`.

### Step 2: Hash Implementation

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `HashInto` implementation for `AffinityTarget` and add the `KeywordAbility::Affinity` match arm.

1. Add `AffinityTarget` to the imports at line 33 (in the `use super::types` block):
   ```rust
   use super::types::{
       AffinityTarget, CardType, Color, CounterType, EnchantTarget, KeywordAbility,
       LandwalkType, ManaColor, ProtectionQuality, SubType, SuperType,
   };
   ```

2. Add `HashInto for AffinityTarget` impl (near the other type impls, around line 280):
   ```rust
   impl HashInto for AffinityTarget {
       fn hash_into(&self, hasher: &mut Hasher) {
           match self {
               AffinityTarget::Artifacts => 0u8.hash_into(hasher),
               AffinityTarget::BasicLandType(st) => {
                   1u8.hash_into(hasher);
                   st.hash_into(hasher);
               }
           }
       }
   }
   ```

3. Add `KeywordAbility::Affinity` match arm in the `HashInto for KeywordAbility` impl (after `Unearth` discriminant 52, line 395):
   ```rust
   // Affinity (discriminant 53) -- CR 702.41
   KeywordAbility::Affinity(target) => {
       53u8.hash_into(hasher);
       target.hash_into(hasher);
   }
   ```

### Step 3: Rule Enforcement (Cost Reduction in casting.rs)

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add `apply_affinity_reduction` function and wire it into the cost pipeline between kicker and convoke.

**CR**: 702.41a -- "This spell costs {1} less to cast for each [text] you control."
**CR**: 601.2f -- Cost reductions are applied as part of total cost determination.
**CR**: 702.41b -- Multiple instances each apply.

#### 3a. Add the `apply_affinity_reduction` function

Insert after the kicker cost addition block (around line 594) and before the convoke block (line 691):

```rust
// CR 702.41a / 601.2f: Apply affinity cost reduction AFTER total cost is determined
// (including commander tax, kicker) and BEFORE convoke/improvise/delve.
// Affinity is a static ability — the engine counts qualifying permanents automatically.
// CR 702.41b: Multiple instances of affinity are cumulative.
let mana_cost = apply_affinity_reduction(state, player, &chars, mana_cost);
```

#### 3b. Implement the function

```rust
/// CR 702.41a: Apply affinity cost reduction to the total mana cost.
///
/// For each instance of `KeywordAbility::Affinity(target)` on the spell,
/// count the number of permanents matching `target` that the caster controls,
/// and reduce the generic mana component by that count.
///
/// CR 702.41b: Multiple instances of affinity are cumulative — each one
/// independently counts and reduces. Two instances of "affinity for artifacts"
/// with 3 artifacts = 6 generic mana reduction.
///
/// CR 601.2f: The generic mana component cannot be reduced below 0.
/// Colored and colorless pips are unaffected.
fn apply_affinity_reduction(
    state: &GameState,
    player: PlayerId,
    chars: &Characteristics,
    cost: Option<ManaCost>,
) -> Option<ManaCost> {
    // Collect all affinity instances from the spell's keywords.
    let affinity_targets: Vec<&AffinityTarget> = chars
        .keywords
        .iter()
        .filter_map(|kw| {
            if let KeywordAbility::Affinity(target) = kw {
                Some(target)
            } else {
                None
            }
        })
        .collect();

    if affinity_targets.is_empty() {
        return cost;
    }

    let mut reduced = match cost {
        Some(c) => c,
        None => return None, // No mana cost to reduce (e.g., free spell)
    };

    // CR 702.41b: Each instance applies independently.
    for target in &affinity_targets {
        let count = count_affinity_permanents(state, player, target);
        // CR 601.2f: Generic cannot go below 0.
        let reduction = count.min(reduced.generic);
        reduced.generic -= reduction;
    }

    Some(reduced)
}

/// Count permanents on the battlefield matching the affinity target
/// that are controlled by the given player.
///
/// CR 702.41a: Counts ALL matching permanents — tapped or untapped.
fn count_affinity_permanents(
    state: &GameState,
    player: PlayerId,
    target: &AffinityTarget,
) -> u32 {
    state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.controller == player
                && matches_affinity_target(state, obj, target)
        })
        .count() as u32
}

/// Check if a game object matches the given affinity target.
///
/// Uses `calculate_characteristics` for layer-correct type checking.
fn matches_affinity_target(
    state: &GameState,
    obj: &crate::state::game_object::GameObject,
    target: &AffinityTarget,
) -> bool {
    let chars = calculate_characteristics(state, obj.id)
        .unwrap_or_else(|| obj.characteristics.clone());
    match target {
        AffinityTarget::Artifacts => chars.card_types.contains(&CardType::Artifact),
        AffinityTarget::BasicLandType(subtype) => {
            chars.card_types.contains(&CardType::Land)
                && chars.subtypes.contains(subtype)
        }
    }
}
```

#### 3c. Add imports

Add `AffinityTarget` to the imports in `casting.rs` (line 33):
```rust
use crate::state::types::{AffinityTarget, CardType, EnchantTarget, KeywordAbility, SubType};
```

### Step 4: Replay Viewer (view_model.rs)

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::Affinity` arm to the `format_keyword` function (around line 631).

```rust
KeywordAbility::Affinity(target) => {
    match target {
        AffinityTarget::Artifacts => "Affinity for artifacts".to_string(),
        AffinityTarget::BasicLandType(st) => format!("Affinity for {}", st.0),
    }
}
```

Add `AffinityTarget` to the import of `mtg_engine` at line 11:
```rust
use mtg_engine::{
    calculate_characteristics, AffinityTarget, AttackTarget, CombatState, CounterType,
    GameState, KeywordAbility, ObjectId, PlayerId, StackObjectKind, ZoneId,
};
```

### Step 5: lib.rs Re-export

**File**: `crates/engine/src/lib.rs`
**Action**: Add `AffinityTarget` to the public re-exports. Grep for `KeywordAbility` in the re-export block and add `AffinityTarget` alongside it.

### Step 6: Match Arm Exhaustiveness

Grep for all `KeywordAbility` match expressions that need a new arm. Known locations:

- `crates/engine/src/state/hash.rs` -- `HashInto for KeywordAbility` (Step 2 above)
- `tools/replay-viewer/src/view_model.rs` -- `format_keyword` (Step 4 above)
- `crates/engine/src/state/builder.rs` -- keyword-to-trigger translation (Affinity is a static ability, NOT a triggered ability; add an empty/no-op arm or let the wildcard handle it)

**Action**: Compile and fix any exhaustiveness errors by adding `KeywordAbility::Affinity(_)` arms. Since Affinity is a static cost-reduction keyword (not triggered, not evasion, not protection), most match arms need only a no-op/pass-through arm.

### Step 7: Unit Tests

**File**: `crates/engine/tests/affinity.rs` (new file)
**Tests to write** (following the pattern from `crates/engine/tests/improvise.rs`):

1. **`test_affinity_basic_reduce_generic_cost`** -- CR 702.41a
   - Spell {4} with Affinity for artifacts. Player controls 3 artifacts.
   - Cast with {1} in pool. Should succeed (4 - 3 = 1 generic).
   - Verify spell on stack, mana pool empty.

2. **`test_affinity_reduce_to_zero`** -- CR 702.41a + CR 601.2f
   - Spell {4} with Affinity for artifacts. Player controls 4 artifacts.
   - Cast with {0} in pool. Should succeed (4 - 4 = 0).
   - Verify spell on stack, mana pool empty.

3. **`test_affinity_excess_artifacts_no_negative_cost`** -- CR 601.2f
   - Spell {4} with Affinity for artifacts. Player controls 6 artifacts.
   - Cast with {0} in pool. Should succeed (cost floors at 0, does not go negative).
   - Verify spell on stack.

4. **`test_affinity_does_not_reduce_colored_pips`** -- CR 702.41a + CR 601.2f
   - Spell {4}{U} with Affinity for artifacts. Player controls 4 artifacts.
   - Must have {U} in pool (colored pips not reduced). Cast with only {U}.
   - Verify spell on stack.

5. **`test_affinity_no_keyword_no_reduction`** -- Negative test
   - Spell {4} WITHOUT affinity keyword. Player controls 4 artifacts.
   - Must have {4} in pool to cast. Verify full cost is paid.

6. **`test_affinity_counts_tapped_artifacts`** -- CR 702.41a (ALL artifacts, not just untapped)
   - Spell {4} with Affinity for artifacts. Player controls 2 untapped + 2 tapped artifacts.
   - Cast with {0} in pool. Should succeed (counts all 4, regardless of tapped state).

7. **`test_affinity_only_counts_controlled_permanents`** -- CR 702.41a ("you control")
   - Spell {4} with Affinity for artifacts. Player controls 1 artifact. Opponent controls 3 artifacts.
   - Must have {3} in pool (only own artifacts count: 4 - 1 = 3).

8. **`test_affinity_combined_with_improvise`** -- Interaction test
   - Spell {6}{U} with Affinity for artifacts AND Improvise. Player controls 4 artifacts.
   - Affinity reduces {6} to {2} (4 artifacts). Then Improvise taps 2 artifacts to pay remaining {2}. Pay {U} from pool.
   - Verify all artifacts counted for affinity, 2 tapped for improvise.

9. **`test_affinity_with_commander_tax`** -- CR 702.41a + CR 903.8
   - Commander spell {2}{U} with Affinity for artifacts, cast from command zone with 1 prior cast (tax = {2}).
   - Total cost: {4}{U}. Player controls 4 artifacts. Affinity reduces {4} to {0}. Pay {U} from pool.

10. **`test_affinity_multiple_instances_cumulative`** -- CR 702.41b
    - Spell {8} with TWO instances of Affinity for artifacts. Player controls 3 artifacts.
    - Each instance reduces by 3: 8 - 3 - 3 = 2. Must have {2} in pool.

11. **`test_affinity_artifact_creature_counts`** -- Ruling clarification
    - Spell with Affinity for artifacts. Player controls artifact creatures.
    - Artifact creatures ARE artifacts and should be counted.

**Pattern**: Follow `crates/engine/tests/improvise.rs` for helper functions (`find_object`, `p()`, `cid()`) and test structure. Use `ObjectSpec::artifact()` for artifact permanents and `ObjectSpec::card().with_keyword(KeywordAbility::Affinity(AffinityTarget::Artifacts))` for the spell.

### Step 8: Card Definition (later phase)

**Suggested card**: Frogmite
- Oracle text: `Affinity for artifacts (This spell costs {1} less to cast for each artifact you control.)`
- Mana cost: {4}
- Type: Artifact Creature -- Frog
- P/T: 2/2
- Abilities: `AbilityDefinition::Keyword(KeywordAbility::Affinity(AffinityTarget::Artifacts))`

**File**: `crates/engine/src/cards/definitions.rs`
**Card lookup**: use `card-definition-author` agent

### Step 9: Game Script (later phase)

**Suggested scenario**: Player 1 controls 4 artifacts (Sol Ring, Mana Vault, Chromatic Star, Ichor Wellspring) on the battlefield. Player 1 casts Frogmite from hand. Affinity for artifacts reduces {4} to {0}. Frogmite enters the battlefield as a 2/2. Assert: Frogmite on battlefield, mana pool unchanged (no mana spent), 4 artifacts still on battlefield.

**Subsystem directory**: `test-data/generated-scripts/stack/` (cost reduction is part of the casting/stack subsystem)

## Interactions to Watch

- **Affinity + Improvise**: Affinity counts ALL artifacts (tapped or not). Improvise then taps some of those same artifacts. Both apply to the same spell. Affinity reduction happens BEFORE Improvise in the cost pipeline (affinity is a cost reduction per CR 601.2f; improvise is a payment method per CR 702.126b). The artifacts tapped for Improvise are still counted by Affinity because Affinity was already locked in.
- **Affinity + Convoke**: Similar to Improvise. Artifact creatures counted by Affinity can also be tapped for Convoke. No conflict.
- **Affinity + Delve**: No interaction -- Delve exiles from graveyard, Affinity counts battlefield permanents.
- **Affinity + Commander tax**: Tax increases the total cost first, then Affinity reduces generic from the total. E.g., base {4} + tax {2} = {6} total, Affinity for artifacts with 5 artifacts = {1} remaining generic.
- **Affinity + Kicker**: Kicker adds to total cost, then Affinity reduces. E.g., base {4} + kicker {2} = {6}, Affinity with 4 artifacts = {2} remaining.
- **Cost floor**: Affinity reduces generic mana only. Color pips are never touched. Even if Affinity "over-reduces," the generic component floors at 0, and color pips remain.
- **Multiple Affinity instances**: CR 702.41b says they are cumulative. Each instance independently counts and reduces. This is already handled by iterating over all `Affinity` keyword instances in the spell's keywords.

## Design Notes

### Why Affinity does NOT need a new CastSpell parameter

Unlike Convoke (player chooses which creatures to tap), Improvise (player chooses which artifacts to tap), and Delve (player chooses which cards to exile), Affinity is fully automatic. The engine counts all qualifying permanents the caster controls and reduces the cost accordingly. No player decision is involved. Therefore:

- No new field on `Command::CastSpell`
- No new field on `ScriptAction::PlayerAction`
- No new parameter on `translate_player_action`
- No new harness action type

The reduction is computed inside `handle_cast_spell` and applied transparently.

### Cost pipeline order (after this change)

```
base_mana_cost (or alt cost: flashback/evoke/bestow/madness/miracle/escape/foretell)
  -> commander_tax (CR 903.8)
  -> kicker (additional cost, CR 702.33)
  -> AFFINITY reduction (CR 702.41, cost reduction per CR 601.2f)
  -> convoke (CR 702.51, payment method)
  -> improvise (CR 702.126, payment method)
  -> delve (CR 702.66, payment method)
  -> escape exile (CR 702.138)
  -> mana payment from pool (CR 601.2f-h)
```
