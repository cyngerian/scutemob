# Ability Plan: Improvise

**Generated**: 2026-02-27
**CR**: 702.126
**Priority**: P3
**Similar abilities studied**: Convoke (CR 702.51) in `crates/engine/src/rules/casting.rs`, `crates/engine/src/state/types.rs`, `crates/engine/src/state/hash.rs`, `crates/engine/tests/convoke.rs`; Delve (CR 702.66) in `crates/engine/src/rules/casting.rs`, `crates/engine/tests/delve.rs`

## CR Rule Text

```
702.126. Improvise

702.126a Improvise is a static ability that functions while the spell with improvise is on
the stack. "Improvise" means "For each generic mana in this spell's total cost, you may tap
an untapped artifact you control rather than pay that mana."

702.126b The improvise ability isn't an additional or alternative cost and applies only after
the total cost of the spell with improvise is determined.

702.126c Multiple instances of improvise on the same spell are redundant.
```

## Key Edge Cases

1. **Generic mana ONLY (CR 702.126a):** Unlike Convoke, Improvise can ONLY reduce generic mana. It cannot pay for colored pips ({W}, {U}, {B}, {R}, {G}) or colorless ({C}). This is the critical distinction from Convoke -- Convoke allows colored creatures to pay matching colored pips; Improvise never does.
2. **Artifacts, not creatures (CR 702.126a):** Improvise taps artifacts, not creatures. The type check must verify `CardType::Artifact` (via layer-correct `calculate_characteristics`), not `CardType::Creature`.
3. **Not additional or alternative cost (CR 702.126b):** Applies AFTER the total cost is determined. Cost pipeline order: `base_mana_cost -> flashback alt cost -> commander tax -> kicker -> convoke -> IMPROVISE -> delve -> payment`. Improvise inserts between convoke and delve in the reduction chain.
4. **Multiple instances are redundant (CR 702.126c):** No need for special handling -- the `improvise_artifacts` list has the same maximum regardless of how many instances exist.
5. **Cannot tap an artifact already tapped for mana (Ruling):** "If an artifact you control has a mana ability with {T} in the cost, activating that ability while casting a spell with improvise will result in the artifact being tapped when you pay the spell's costs. You won't be able to tap it again for improvise." The engine already handles this because TapForMana sets `status.tapped = true` before CastSpell is processed, and the Improvise validation checks `!obj.status.tapped`.
6. **Sacrificed artifacts unavailable (Ruling):** "If you sacrifice an artifact to activate a mana ability while casting a spell with improvise, that artifact won't be on the battlefield when you pay the spell's costs." The engine handles this because the artifact is removed from `state.objects` (or moved to graveyard).
7. **Tapping doesn't stop static abilities (Ruling):** "Tapping an artifact won't cause its abilities to stop applying unless those abilities say so." No engine action needed -- our continuous effects don't check tapped status unless explicitly coded to.
8. **Equipment tapping independent (Ruling):** "Equipment attached to a creature doesn't become tapped when that creature becomes tapped, and tapping that Equipment doesn't cause the creature to become tapped." Already correct in the engine -- there is no implicit tap propagation.
9. **X spells (Ruling):** "When using improvise to cast a spell with {X} in its mana cost, first choose the value for X. That choice, plus any cost increases or decreases, will determine the spell's total cost. Then you can tap artifacts you control to help pay that cost." Already handled by cost pipeline ordering.
10. **Cannot pay for non-casting costs (Ruling):** "Improvise can't be used to pay for anything other than the cost of casting the spell." Only applies during CastSpell -- already correct by design.
11. **Doesn't change mana cost or mana value (Ruling):** The `ManaCost` on the spell is unchanged; only the amount paid from the pool is reduced. This is already correct because `apply_improvise_reduction` modifies a local copy of the cost for payment purposes but does not alter the spell's `characteristics.mana_cost`.
12. **Multiplayer:** No special multiplayer considerations -- each player can only tap their own artifacts (controlled by them, on the battlefield).
13. **Summoning sickness irrelevant:** Like Convoke, summoning sickness does not prevent tapping for Improvise because it is not an activated ability with {T} in the cost (CR 302.6). Artifacts entering this turn can still be tapped for Improvise.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- does NOT exist in `types.rs`
- [ ] Step 2: Rule enforcement -- no `apply_improvise_reduction` in `casting.rs`
- [ ] Step 3: Trigger wiring -- N/A (Improvise is not a trigger)
- [ ] Step 4: Unit tests -- no `crates/engine/tests/improvise.rs`
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Improvise` variant after `Convoke` (line ~237), before `Delve`.
**Pattern**: Follow `KeywordAbility::Convoke` at line 237.
**CR**: 702.126

```rust
/// CR 702.126: Improvise -- tap artifacts to pay generic mana costs.
/// "For each generic mana in this spell's total cost, you may tap an untapped
/// artifact you control rather than pay that mana."
/// CR 702.126b: Not an additional or alternative cost; applies after total cost determined.
/// CR 702.126c: Multiple instances are redundant.
Improvise,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant for `KeywordAbility::Improvise`. Use discriminant **44** (next after Extort's 43).
**Pattern**: Follow `KeywordAbility::Convoke => 30u8.hash_into(hasher)` at line 342.

```rust
// Improvise (discriminant 44) -- CR 702.126
KeywordAbility::Improvise => 44u8.hash_into(hasher),
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `improvise_artifacts: Vec<ObjectId>` field to the `CastSpell` variant, after `convoke_creatures` (line ~71).
**Pattern**: Follow `convoke_creatures: Vec<ObjectId>` at line 71.

```rust
/// CR 702.126: Artifacts to tap for improvise cost reduction.
/// Empty vec for non-improvise spells. Each artifact must be:
/// - Untapped, on the battlefield, controlled by the caster
/// - An artifact (by current characteristics)
/// - Not duplicated (no ObjectId appears twice)
///
/// Each artifact pays for {1} generic mana. Cannot exceed the generic
/// mana component of the spell's total cost (after convoke reduction).
/// Validated in handle_cast_spell -> apply_improvise_reduction.
improvise_artifacts: Vec<ObjectId>,
```

**Match arms**: After adding the field to `Command::CastSpell`, grep for all destructuring patterns of `Command::CastSpell { ... }` and add the new field. Known locations:
- `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs` line ~70
- `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` (hash impl for Command)

### Step 2: Rule Enforcement

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`

#### 2a: Update `handle_cast_spell` signature

**Action**: Add `improvise_artifacts: Vec<ObjectId>` parameter after `convoke_creatures`.
**Pattern**: Follow `convoke_creatures: Vec<ObjectId>` parameter at line 55.

#### 2b: Add improvise reduction step in the cost pipeline

**Action**: Insert the improvise reduction block between convoke and delve (after line ~355, before line ~357).
**CR**: 702.126a / 702.126b -- Improvise applies after total cost is determined, not an additional or alternative cost.

The cost pipeline order becomes:
```
base_mana_cost -> flashback alt cost -> commander tax -> kicker -> convoke -> IMPROVISE -> delve -> payment
```

```rust
// CR 702.126a / 702.126b: Apply improvise cost reduction AFTER total cost is determined.
// Improvise is not an additional or alternative cost -- it applies to the total cost.
// Order: base_mana_cost -> commander_tax -> flashback -> convoke -> IMPROVISE -> delve -> pay.
let mut improvise_events: Vec<GameEvent> = Vec::new();
let mana_cost = if !improvise_artifacts.is_empty() {
    if !chars.keywords.contains(&KeywordAbility::Improvise) {
        return Err(GameStateError::InvalidCommand(
            "spell does not have improvise".into(),
        ));
    }
    apply_improvise_reduction(
        state,
        player,
        &improvise_artifacts,
        mana_cost,
        &mut improvise_events,
    )?
} else {
    mana_cost
};
```

Also add `events.extend(improvise_events);` after the convoke events emission (after line ~393), before delve events.

#### 2c: Implement `apply_improvise_reduction` function

**Action**: Add a new function after `apply_convoke_reduction` (ends at line ~773).
**Pattern**: This is a SIMPLIFIED version of `apply_convoke_reduction` -- Improvise is simpler because it only reduces generic mana, never colored mana.
**CR**: 702.126a

The function must:
1. Validate uniqueness (no duplicate ObjectIds).
2. For each artifact ID:
   - Verify it exists in `state.objects` on `ZoneId::Battlefield`.
   - Verify it is controlled by `player`.
   - Verify it is untapped (`status.tapped == false`).
   - Verify it is an artifact via `calculate_characteristics` (layer-correct type check using `CardType::Artifact`).
3. Validate that `improvise_artifacts.len() <= reduced.generic` (cannot exceed generic mana remaining).
4. Reduce `reduced.generic` by `improvise_artifacts.len()`.
5. Tap each artifact (`obj.status.tapped = true`) and emit `PermanentTapped` events.
6. Return the reduced `Option<ManaCost>`.

**Key difference from Convoke**: No colored mana matching. Each artifact reduces exactly one generic pip. The reduction loop is a simple count, not a per-creature color-matching loop.

```rust
/// CR 702.126a: Validate improvise artifacts and compute the reduced mana cost.
///
/// For each artifact in `improvise_artifacts`:
/// - Must exist in `state.objects` on the battlefield.
/// - Must be controlled by `player`.
/// - Must be an artifact (by current characteristics via `calculate_characteristics`).
/// - Must be untapped (`status.tapped == false`).
/// - Must not appear twice in the list (no duplicates).
///
/// Reduction (CR 702.126a):
/// - Each artifact reduces one generic pip. Cannot exceed total generic mana.
///
/// Taps each artifact in `state.objects` and emits a `PermanentTapped` event.
/// Returns the reduced `Option<ManaCost>`.
///
/// CR 702.126b: Improvise is not an additional or alternative cost -- it applies
/// after the total cost (including commander tax, convoke) is determined.
fn apply_improvise_reduction(
    state: &mut GameState,
    player: PlayerId,
    improvise_artifacts: &[ObjectId],
    cost: Option<ManaCost>,
    events: &mut Vec<GameEvent>,
) -> Result<Option<ManaCost>, GameStateError> {
    // Validate uniqueness (no duplicates in improvise_artifacts).
    let mut seen = std::collections::HashSet::new();
    for &id in improvise_artifacts {
        if !seen.insert(id) {
            return Err(GameStateError::InvalidCommand(format!(
                "duplicate artifact {:?} in improvise_artifacts",
                id
            )));
        }
    }

    // Validate each artifact before mutably borrowing state for tapping.
    for &id in improvise_artifacts {
        let obj = state
            .objects
            .get(&id)
            .ok_or(GameStateError::ObjectNotFound(id))?;

        // Must be on the battlefield.
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::InvalidCommand(format!(
                "improvise artifact {:?} is not on the battlefield",
                id
            )));
        }
        // Must be controlled by the caster.
        if obj.controller != player {
            return Err(GameStateError::InvalidCommand(format!(
                "improvise artifact {:?} is not controlled by the caster",
                id
            )));
        }
        // Must be untapped.
        if obj.status.tapped {
            return Err(GameStateError::InvalidCommand(format!(
                "improvise artifact {:?} is already tapped",
                id
            )));
        }

        // Must be an artifact (use calculate_characteristics for layer-correct check).
        let chars = calculate_characteristics(state, id)
            .or_else(|| state.objects.get(&id).map(|o| o.characteristics.clone()))
            .unwrap_or_default();
        if !chars
            .card_types
            .contains(&crate::state::types::CardType::Artifact)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "improvise artifact {:?} is not an artifact",
                id
            )));
        }
    }

    // Apply cost reduction: each artifact reduces one generic pip (CR 702.126a).
    let had_cost = cost.is_some();
    let mut reduced = cost.unwrap_or_default();

    // Validate that we don't tap more artifacts than the generic portion allows.
    if improvise_artifacts.len() as u32 > reduced.generic {
        return Err(GameStateError::InvalidCommand(format!(
            "improvise_artifacts.len() ({}) exceeds generic mana in cost ({})",
            improvise_artifacts.len(),
            reduced.generic
        )));
    }
    reduced.generic -= improvise_artifacts.len() as u32;

    // Tap each improvise artifact and emit PermanentTapped events.
    for &id in improvise_artifacts {
        if let Some(obj) = state.objects.get_mut(&id) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: id,
        });
    }

    // If the original cost was Some, return Some(reduced); if it was None, return None.
    if had_cost {
        Ok(Some(reduced))
    } else {
        Ok(None)
    }
}
```

#### 2d: Update `handle_cast_spell` callers

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs` (line ~70-90)
**Action**: Destructure `improvise_artifacts` from `Command::CastSpell` and pass it to `handle_cast_spell`.

```rust
Command::CastSpell {
    player,
    card,
    targets,
    convoke_creatures,
    improvise_artifacts,  // NEW
    delve_cards,
    kicker_times,
    cast_with_evoke,
} => {
    // ...
    let mut events = casting::handle_cast_spell(
        &mut state,
        player,
        card,
        targets,
        convoke_creatures,
        improvise_artifacts,  // NEW
        delve_cards,
        kicker_times,
        cast_with_evoke,
    )?;
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/copy.rs`
**Action**: Grep for any `handle_cast_spell` calls (storm/cascade might call it). If they call it, add `vec![]` for `improvise_artifacts`. If they construct `Command::CastSpell`, add `improvise_artifacts: vec![]`.

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: In the `Command::CastSpell` hash arm, add hashing for `improvise_artifacts`. Pattern: follow how `convoke_creatures` is hashed.

#### 2e: Update replay harness

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/script_schema.rs`
**Action**: Add `improvise: Vec<String>` field to `ScriptAction::PlayerAction`, with `#[serde(default)]`.
**Pattern**: Follow `convoke: Vec<String>` at line ~222.

```rust
/// CR 702.126: For `cast_spell` with improvise. Names of untapped artifacts on the
/// battlefield to tap as part of cost payment. Empty for non-improvise casts.
/// Example: ["Sol Ring", "Mana Vault", "Signet"]
#[serde(default)]
improvise: Vec<String>,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**:
1. Add `improvise_names: &[String]` parameter to `translate_player_action` (after `convoke_names`, line ~202).
2. In the `"cast_spell"` arm, resolve `improvise_names` to ObjectIds using `find_on_battlefield`, same as convoke. Pass the resulting `improvise_ids` into `Command::CastSpell { improvise_artifacts: improvise_ids, ... }`.
3. In all other `Command::CastSpell` constructions (e.g., `"cast_spell_flashback"`, `"cast_spell_evoke"`), add `improvise_artifacts: vec![]`.

**File**: `/home/airbaggie/scutemob/crates/engine/tests/script_replay.rs`
**Action**: Update the `ScriptAction::PlayerAction` destructuring to extract `improvise`, and pass it to `translate_player_action`.

#### 2f: Update all existing CastSpell construction sites

Grep for `Command::CastSpell` across the entire codebase. Every construction site needs the new `improvise_artifacts` field (usually `improvise_artifacts: vec![]`).

Known sites:
- All test files that construct `Command::CastSpell` (convoke.rs, delve.rs, kicker.rs, casting.rs, etc.)
- `replay_harness.rs` (handled above)
- `copy.rs` (storm/cascade copies)

**This is the highest-churn step.** Use grep to find ALL `Command::CastSpell` construction sites:
```
Grep pattern="Command::CastSpell" path="crates/engine/" output_mode="files_with_matches"
```

Then add `improvise_artifacts: vec![],` to each one.

### Step 3: Trigger Wiring

**N/A** -- Improvise is a static ability that functions while the spell is on the stack. It is not a trigger. No trigger wiring is needed.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/improvise.rs`
**Pattern**: Follow `crates/engine/tests/convoke.rs` and `crates/engine/tests/delve.rs`.

**Tests to write:**

1. **`test_improvise_basic_tap_artifacts_reduce_generic_cost`** -- CR 702.126a
   - Spell: `{3}{U}{U}` with Improvise. Tap 3 untapped artifacts to pay generic. Pay `{U}{U}` from pool.
   - Assert: spell on stack, all 3 artifacts tapped, mana pool empty, 3 PermanentTapped events.

2. **`test_improvise_cannot_pay_colored_mana`** -- CR 702.126a ("for each GENERIC mana")
   - Spell: `{U}{U}` with Improvise (no generic cost). Provide 2 artifacts.
   - Assert: error -- too many artifacts tapped for improvise (exceeds generic mana 0).
   - This is the CRITICAL difference from Convoke.

3. **`test_improvise_reject_no_keyword`** -- CR 702.126a
   - Spell: plain sorcery {3} without Improvise keyword. Attempt to pass artifacts.
   - Assert: error containing "improvise" or "InvalidCommand".

4. **`test_improvise_reject_tapped_artifact`** -- CR 702.126a ("untapped artifact")
   - Artifact is already tapped. Attempt to use for Improvise.
   - Assert: error containing "tapped" or "InvalidCommand".

5. **`test_improvise_reject_not_artifact`** -- CR 702.126a ("artifact")
   - A creature (non-artifact) is passed in `improvise_artifacts`.
   - Assert: error containing "artifact" or "InvalidCommand".

6. **`test_improvise_reject_opponent_artifact`** -- CR 702.126a ("you control")
   - An artifact controlled by an opponent.
   - Assert: error containing "controller" or "caster" or "InvalidCommand".

7. **`test_improvise_reject_too_many_artifacts`** -- CR 702.126a / Rulings
   - Spell: `{1}{U}` (1 generic). Tap 2 artifacts -- exceeds generic mana.
   - Assert: error.

8. **`test_improvise_zero_artifacts_normal_cast`** -- CR 702.126a
   - Spell with Improvise cast with empty `improvise_artifacts` vec. Full mana payment.
   - Assert: spell on stack, mana pool empty.

9. **`test_improvise_with_commander_tax`** -- CR 702.126b + CR 903.8
   - Commander with Improvise. After 1 previous cast, tax = {2}.
   - Total cost = base + {2} tax. Improvise artifacts pay some generic pips.
   - Assert: spell on stack, commander tax incremented, mana pool correct.

10. **`test_improvise_combined_with_convoke`** -- Edge case: spell has BOTH Convoke and Improvise
    - Not a natural card, but rules allow it (e.g., Inspiring Statuary + creature with Convoke).
    - Convoke reduces first (creatures pay colored/generic), then Improvise reduces remaining generic.
    - Assert: both creature and artifact tapped, spell on stack.

11. **`test_improvise_artifact_creature_can_be_used`** -- Ruling: artifact creatures ARE artifacts
    - An artifact creature on the battlefield can be tapped for Improvise (it's an artifact).
    - Assert: success, artifact creature tapped.

12. **`test_improvise_summoning_sickness_irrelevant`** -- Ruling: no summoning sickness check
    - Newly-entered artifact creature can be tapped for Improvise.
    - Assert: success.

**Helper functions to define:**

```rust
/// Create an improvise spell in hand.
/// Cost: `{generic}{blue}` where blue is the number of blue pips.
fn improvise_spell_spec(owner: PlayerId, name: &str, generic: u32, blue: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            blue,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Improvise)
}

/// Untapped artifact on the battlefield (NOT a creature).
fn artifact_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Artifact])
}

/// Artifact creature on the battlefield (both types).
fn artifact_creature_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1)
        .with_types(vec![CardType::Artifact, CardType::Creature])
}
```

### Step 5: Card Definition (later phase)

**Suggested card**: Reverse Engineer (`{3}{U}{U}`, Sorcery, Improvise, Draw three cards.)
- Simple effect (DrawCards), clear Improvise usage, good test target.
- Alternative: Whir of Invention (`{X}{U}{U}{U}`, Instant, Improvise, search library for artifact) -- more complex due to X cost and SearchLibrary.

**Card lookup**: Use `card-definition-author` agent with "Reverse Engineer".

### Step 6: Game Script (later phase)

**Suggested scenario**: Cast Reverse Engineer with 3 artifacts tapped for Improvise, paying only {U}{U} from pool. Resolve the spell, draw 3 cards. Verify artifacts are tapped and hand size increased.

**Subsystem directory**: `test-data/generated-scripts/stack/` (casting/cost-reduction scripts live here alongside convoke/delve scripts).

## Interactions to Watch

1. **Convoke + Improvise on same spell**: If both keywords are present (e.g., via Inspiring Statuary granting Improvise to a spell that already has Convoke), Convoke applies first (can reduce colored and generic), then Improvise applies to remaining generic. The cost pipeline ordering handles this naturally.

2. **Delve + Improvise on same spell**: Improvise applies first (tapping artifacts), then Delve applies to remaining generic (exiling cards). Both reduce only generic mana, so order between them doesn't matter mathematically, but the pipeline puts Improvise before Delve for consistency.

3. **Commander tax**: Tax is added before Improvise reduction (CR 702.126b: "after total cost is determined"). Improvise can pay for the tax's generic component.

4. **Flashback + Improvise**: Flashback is an alternative cost. CR 702.126b says Improvise applies after total cost is determined. Flashback cost replaces mana cost, then Improvise reduces the generic portion of the flashback cost. Already handled by pipeline ordering.

5. **Kicker + Improvise**: Kicker adds to total cost before Improvise applies. Improvise can pay for generic mana added by kicker. Already handled by pipeline ordering.

6. **Split Second**: Split second prevents casting spells and activating non-mana abilities. TapForMana (mana abilities) is still allowed, but Improvise tapping is part of the CastSpell cost payment, not a separate mana ability. If a spell with split second is on the stack, you cannot cast an Improvise spell (because you cannot cast spells at all), so no interaction issue.

7. **Artifact tokens**: Treasure tokens, Clue tokens, Food tokens, etc. are artifacts and can be tapped for Improvise. They are NOT sacrificed (unlike using Treasure for mana) -- they are just tapped.
