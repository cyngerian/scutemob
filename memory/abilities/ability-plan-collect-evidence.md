# Ability Plan: Collect Evidence

**Generated**: 2026-03-07
**CR**: 701.59 (keyword action, NOT keyword ability)
**Priority**: P4
**Similar abilities studied**: Bargain (optional additional cost with `was_bargained` tracking in `casting.rs`, `stack.rs`, `game_object.rs`, `effects/mod.rs`, `resolution.rs`; tests in `crates/engine/tests/bargain.rs`), Delve (graveyard exile mechanic in `casting.rs:apply_delve_reduction`)

## CR Rule Text

> **701.59.** Collect Evidence
>
> **701.59a** To "collect evidence N" means to exile any number of cards from your graveyard with total mana value N or greater.
>
> **701.59b** If a player is given the choice to collect evidence but is unable to exile cards with total mana value N or greater from their graveyard (usually because there aren't enough cards to do so) they can't choose to collect evidence.
>
> **701.59c** A spell that has an ability that allows a player to collect evidence as an additional cost to cast it may have another ability that refers to whether evidence was collected. These abilities are linked. See rule 607, "Linked Abilities."

## Key Edge Cases

- **Minimum total MV**: The exiled cards must have total mana value **N or greater**, not exactly N. Player can over-exile (CR 701.59a).
- **Cannot choose to collect evidence if insufficient MV in graveyard**: If the total mana value of all cards in the graveyard is < N, the player cannot choose to collect evidence at all (CR 701.59b). This is checked at cast time.
- **Linked abilities**: "If evidence was collected" on the same card is a linked ability (CR 701.59c / 607). Modeled as `Condition::EvidenceWasCollected` checked at resolution time.
- **Optional vs. mandatory**: Most cards say "you may collect evidence N" (optional additional cost). Some cards say "collect evidence N" as a mandatory additional cost. The implementation must handle both. For optional: player passes empty `collect_evidence_cards` to decline. For mandatory: engine validates that evidence was collected.
- **Collect evidence is a keyword ACTION, not a keyword ability**: It does NOT get a `KeywordAbility` variant. It IS listed as a keyword on cards in Scryfall, but in the engine it is modeled as a casting cost mechanic (like how Delve exiles cards). The `KeywordAbility` enum should NOT have a `CollectEvidence` variant.
- **Cards exiled for evidence do NOT reduce mana cost**: Unlike Delve, the exiled cards are purely cost payment. The normal mana cost is still paid in full.
- **Multiple instances**: If a card has both "collect evidence as additional cost" AND another source triggers "whenever you collect evidence", each collect-evidence action fires the trigger independently (per Conspiracy Unraveler ruling).
- **Cannot exile the card being cast from graveyard**: If casting from graveyard (e.g., flashback), that card is on the stack and cannot be exiled for evidence. But in normal casting from hand, no conflict.
- **Multiplayer**: No special multiplayer considerations -- collect evidence only involves the caster's own graveyard.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant (N/A -- keyword action, no KW variant needed)
- [ ] Step 2: Rule enforcement (casting.rs: validation + payment)
- [ ] Step 3: Trigger wiring (N/A for the cost itself; `Condition::EvidenceWasCollected` needed)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: No KeywordAbility Variant Needed

Collect Evidence is a keyword **action** (CR 701.59), not a keyword ability. It does NOT get added to `KeywordAbility` enum in `types.rs`. Instead, it is modeled entirely through:
1. A new field on `Command::CastSpell`: `collect_evidence_cards: Vec<ObjectId>`
2. A new field on `StackObject`: `evidence_collected: bool`
3. A new field on `GameObject`: `evidence_collected: bool`
4. A new `Condition::EvidenceWasCollected` variant

**Note**: Scryfall lists "Collect evidence" as a keyword on cards, but in the engine's architecture, keyword actions are modeled as Effects or cost mechanics, not as `KeywordAbility` variants (see `gotchas-infra.md` Agent Workflow Gotchas: "Keyword actions are Effects, NOT `KeywordAbility` enum variants").

### Step 2: Add `collect_evidence_cards` to `Command::CastSpell`

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add a new field after `x_value`:
```rust
/// CR 701.59a: Cards in the caster's graveyard to exile as the collect
/// evidence additional cost. Empty vec = not collecting evidence (either
/// the spell lacks collect evidence or the player chose not to pay).
/// The total mana value of the exiled cards must be >= N where N is the
/// collect evidence threshold on the card.
/// Validated in `handle_cast_spell`.
#[serde(default)]
collect_evidence_cards: Vec<ObjectId>,
```

**Pattern**: Follows `delve_cards: Vec<ObjectId>` at command.rs line ~95.

### Step 3: Add `CollectEvidence(u32)` to `AbilityDefinition`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add a new `AbilityDefinition` variant to store the evidence threshold N:
```rust
/// CR 701.59a: Collect evidence N -- optional additional cost.
/// "As an additional cost to cast this spell, you may collect evidence N."
/// The u32 parameter is N (minimum total mana value of exiled cards).
/// The bool indicates whether this is mandatory (true) or optional (false).
CollectEvidence { threshold: u32, mandatory: bool },
```
**Discriminant**: Next available AbilityDefinition discriminant = 53.
**Hash**: Add to `state/hash.rs` `AbilityDefinition` match arm with discriminant 53.

### Step 4: Add `evidence_collected` to `StackObject`

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add field after `was_bargained`:
```rust
/// CR 701.59c: If true, this spell was cast with collect evidence paid.
/// Used by `Condition::EvidenceWasCollected` to check at resolution time.
#[serde(default)]
pub evidence_collected: bool,
```
**Hash**: Add to `state/hash.rs` `StackObject` hash impl.

### Step 5: Add `evidence_collected` to `GameObject`

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add field after `was_bargained`:
```rust
/// CR 701.59c: If true, this permanent was cast with collect evidence paid.
/// Propagated from `StackObject.evidence_collected` at resolution time.
#[serde(default)]
pub evidence_collected: bool,
```
**Hash**: Add to `state/hash.rs` `GameObject` hash impl.
**Init sites**: Set to `false` in all `GameObject` construction sites:
- `state/mod.rs` `move_object_to_zone` (line ~359)
- `resolution.rs` (all `GameObject { ... }` struct literals -- search for `was_bargained: false` and add `evidence_collected: false` next to each)
- `rules/builder.rs` if applicable
- `effects/mod.rs` (token creation sites)

### Step 6: Add `Condition::EvidenceWasCollected`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `Condition` enum:
```rust
/// CR 701.59c: "if evidence was collected" -- true when
/// `evidence_collected` is set on the EffectContext or StackObject.
/// Linked ability check (CR 607).
EvidenceWasCollected,
```
**Hash**: Add to `state/hash.rs` `Condition` hash impl with discriminant 13 (next after OpponentHasPoisonCounters at 12).

### Step 7: Add `evidence_collected` to `EffectContext`

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add field to `EffectContext` struct:
```rust
/// CR 701.59c: If true, this spell was cast with collect evidence paid.
/// Used by `Condition::EvidenceWasCollected`.
pub evidence_collected: bool,
```
**Evaluation**: Add match arm in `evaluate_condition()`:
```rust
Condition::EvidenceWasCollected => ctx.evidence_collected,
```
**Pattern**: Follows `Condition::WasBargained => ctx.was_bargained` at effects/mod.rs line ~3198.

### Step 8: Rule Enforcement in `casting.rs`

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Two sections, following the Bargain pattern:

#### 8a. Validation (after bargain/casualty validation, ~line 2330)

```rust
// CR 701.59a / CR 601.2b,f: Collect Evidence -- validate the exiled cards.
// Collect Evidence is an additional cost: the player exiles cards from their
// graveyard with total mana value >= N.
let evidence_was_collected: bool = if !collect_evidence_cards.is_empty() {
    // 1. Validate the spell has CollectEvidence ability definition.
    let evidence_threshold = /* find AbilityDefinition::CollectEvidence { threshold, .. } */;
    // 2. Validate uniqueness (no duplicates).
    // 3. Validate each card is in the caster's graveyard.
    // 4. Sum mana values of all cards.
    // 5. Validate total >= threshold (CR 701.59a).
    // 6. If validation passes, return true.
    true
} else {
    // Check if evidence is mandatory.
    // If AbilityDefinition::CollectEvidence { mandatory: true, .. } exists
    // and collect_evidence_cards is empty, return error.
    false
};
```

**Pattern**: Follows Bargain validation at casting.rs lines 2286-2331.
**Key difference from Delve**: Delve reduces generic mana; collect evidence does NOT reduce mana at all. Delve validates `delve_cards.len() <= generic`; collect evidence validates `sum(mana_values) >= threshold`.

#### 8b. Payment (after bargain/casualty payment, ~line 2827)

```rust
// CR 701.59a: Pay the collect evidence additional cost -- exile cards.
let mut evidence_events: Vec<GameEvent> = Vec::new();
if evidence_was_collected {
    for &id in &collect_evidence_cards {
        let (new_exile_id, _) = state.move_object_to_zone(id, ZoneId::Exile)?;
        evidence_events.push(GameEvent::ObjectExiled {
            player,
            object_id: id,
            new_object_id: new_exile_id,
        });
    }
}
```

**Pattern**: Follows Delve payment at casting.rs lines 4271-4278.

#### 8c. StackObject creation (~line 3060)

Add `evidence_collected: evidence_was_collected,` to the `StackObject` struct literal, next to `was_bargained`.

#### 8d. `handle_cast_spell` function signature

Add `collect_evidence_cards: Vec<ObjectId>` parameter. This function already has many parameters.

### Step 9: Propagate `evidence_collected` Through Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: In every site where `ctx.was_bargained = stack_obj.was_bargained;` appears, add:
```rust
ctx.evidence_collected = stack_obj.evidence_collected;
```
There are 4 such sites (lines ~209, ~321, ~347, ~427). Search for `was_bargained` in resolution.rs and add the evidence_collected propagation next to each.

Also at the permanent-creation site (~line 427):
```rust
obj.evidence_collected = stack_obj.evidence_collected;
```

### Step 10: Wire `handle_cast_spell` Call Site

**File**: `crates/engine/src/rules/engine.rs` (or wherever `handle_cast_spell` is called)
**Action**: Pass the new `collect_evidence_cards` field from the `Command::CastSpell` destructure to `handle_cast_spell()`.
**Pattern**: Follow how `bargain_sacrifice` is passed through.

### Step 11: Replay Harness Action Type

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `cast_spell_collect_evidence` action type.

1. Add parameter to `build_cast_spell_command()` (or the equivalent internal function):
   - `collect_evidence_card_names: Vec<String>` -- names of cards in graveyard to exile
2. Add `"cast_spell_collect_evidence"` arm to `translate_player_action()`:
   - Reads `collect_evidence_cards` (array of card names) from the action JSON
   - Resolves each name to an ObjectId in the player's graveyard
   - Passes them via `collect_evidence_cards` on `Command::CastSpell`

**Pattern**: Follow `cast_spell_bargain` at replay_harness.rs line ~1275.

### Step 12: Replay Viewer Exhaustive Matches

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: No new `StackObjectKind` or `KeywordAbility` variants are being added, so no changes needed here. However, verify with `cargo build --workspace` after implementation.

### Step 13: Reset in `move_object_to_zone` and `state/mod.rs`

**File**: `crates/engine/src/state/mod.rs`
**Action**: In `move_object_to_zone`, reset `evidence_collected: false` in the new GameObject construction (next to `was_bargained: false`).

### Step 14: Unit Tests

**File**: `crates/engine/tests/collect_evidence.rs` (new file)
**Tests to write**:

1. `test_collect_evidence_basic_exile_from_graveyard` -- CR 701.59a
   - Setup: Player has a spell with CollectEvidence(6) and cards in graveyard with total MV >= 6.
   - Action: Cast spell with `collect_evidence_cards` pointing to graveyard cards.
   - Assert: Cards are exiled, `evidence_collected == true` on StackObject, spell resolves normally with full mana cost paid.

2. `test_collect_evidence_over_threshold` -- CR 701.59a "N or greater"
   - Setup: Evidence threshold is 4, player exiles cards with total MV = 7.
   - Assert: Valid -- over-exiling is allowed.

3. `test_collect_evidence_under_threshold_rejected` -- CR 701.59a / 701.59b
   - Setup: Evidence threshold is 6, player tries to exile cards with total MV = 4.
   - Assert: Error -- total MV insufficient.

4. `test_collect_evidence_no_cards_chosen_optional` -- CR 701.59a
   - Setup: Spell has optional CollectEvidence(6). Player passes empty `collect_evidence_cards`.
   - Assert: Spell resolves with `evidence_collected == false`. Conditional effect uses the non-evidence branch.

5. `test_collect_evidence_cannot_choose_insufficient_graveyard` -- CR 701.59b
   - Setup: Player's graveyard total MV < N. Player tries to collect evidence.
   - Assert: Error -- cannot choose to collect evidence.

6. `test_collect_evidence_condition_if_collected` -- CR 701.59c
   - Setup: Spell uses `Condition::EvidenceWasCollected` to branch effects.
   - Assert: When evidence collected, the "if evidence was collected" branch executes.
   - Assert: When evidence NOT collected, the base branch executes.

7. `test_collect_evidence_duplicate_card_rejected` -- Engine validation
   - Setup: Same ObjectId appears twice in `collect_evidence_cards`.
   - Assert: Error -- no duplicates allowed.

8. `test_collect_evidence_card_not_in_graveyard_rejected` -- Engine validation
   - Setup: ObjectId points to a card on battlefield, not graveyard.
   - Assert: Error -- cards must be in caster's graveyard.

9. `test_collect_evidence_opponents_graveyard_rejected` -- Engine validation
   - Setup: ObjectId points to a card in opponent's graveyard.
   - Assert: Error -- must be caster's own graveyard.

10. `test_collect_evidence_spell_without_ability_rejected` -- Engine validation
    - Setup: Spell without CollectEvidence ability, but `collect_evidence_cards` is non-empty.
    - Assert: Error -- spell does not have collect evidence.

11. `test_collect_evidence_mana_not_reduced` -- Key difference from Delve
    - Setup: Spell costs {3}{U} with CollectEvidence(4). Player exiles 2 cards (total MV 5).
    - Assert: Player still pays full {3}{U} mana cost. No generic mana reduction.

**Pattern**: Follow `crates/engine/tests/bargain.rs` for test structure, helpers, card definition setup.

### Step 15: Card Definition (later phase)

**Suggested card**: Crimestopper Sprite
- Mana cost: {2}{U}
- Type: Creature -- Faerie Detective
- Oracle: "As an additional cost to cast this spell, you may collect evidence 6. Flying. When this creature enters, tap target creature. If evidence was collected, put a stun counter on it."
- Uses: CollectEvidence(6), Flying, ETB trigger with `Condition::EvidenceWasCollected`
- **Note**: Stun counter effect may need a `CounterType::Stun` variant and stun counter replacement effect infrastructure. If not available, the card definition can omit the stun counter part and just model the tap + conditional branching.

**Alternative simpler card**: Deadly Cover-Up ({3}{B}{B} Sorcery, "As an additional cost to cast this spell, you may collect evidence 6. Destroy all creatures. If evidence was collected, [additional effect].") -- the "destroy all creatures" part uses existing `Effect::DestroyAll` but the evidence-collected bonus effect (exile from opponent's graveyard + search) is complex.

**Recommended**: Start with a **test-only card definition** in the test file (like bargain.rs does) for unit tests, then author a real card via the `card-definition-author` agent.

### Step 16: Game Script (later phase)

**Suggested scenario**: Player casts a spell with optional collect evidence 6.
- Step 1: Player has spell in hand, 2 cards in graveyard (total MV >= 6).
- Step 2: Player taps lands for mana, casts spell with `cast_spell_collect_evidence` action, specifying graveyard cards.
- Step 3: Assert graveyard cards are exiled, spell resolves with evidence-collected branch.
- Alternative path: Player casts same spell without collecting evidence, gets base effect.

**Subsystem directory**: `test-data/generated-scripts/stack/` (casting costs go in the stack subsystem)

## Interactions to Watch

- **Delve + Collect Evidence on same spell**: Extremely unlikely to appear on the same card, but if it did, both exile from graveyard independently. The same card cannot be exiled for both Delve and Collect Evidence. Validate no overlap in exiled cards.
- **Flashback + Collect Evidence**: If a card with collect evidence has flashback, when cast from graveyard via flashback, the card itself is on the stack (not in graveyard) and cannot be exiled for evidence. Other graveyard cards are still valid.
- **Escape + Collect Evidence**: Similar to flashback -- escape already exiles cards from graveyard. Cards exiled for escape cannot also be exiled for evidence. Validate no overlap between `escape_exile_cards` and `collect_evidence_cards`.
- **"Whenever you collect evidence" triggers**: Cards like Surveillance Monitor trigger whenever any collect evidence action happens. This would require a new `GameEvent::EvidenceCollected` event and `TriggerCondition::WheneverYouCollectEvidence`. **Defer this to a future batch** -- for now, focus on the cost mechanic and `Condition::EvidenceWasCollected` linked ability check.
- **Mana value of cards**: Use `characteristics.mana_cost` to compute mana value. Cards with no mana cost (like lands) have mana value 0. Cards with X in their mana cost have X=0 in the graveyard. Use the `mana_value()` method on the card.
