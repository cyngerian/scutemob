# Ability Plan: Convoke

**Generated**: 2026-02-26
**CR**: 702.51
**Priority**: P2
**Similar abilities studied**: Cycling (`CycleCard` command, `abilities.rs:handle_cycle_card`), Flashback (cost alternative in `casting.rs:handle_cast_spell`), Delve (not yet implemented, but same cost-modification pattern as Convoke)

## CR Rule Text

**702.51.** Convoke

**702.51a** Convoke is a static ability that functions while the spell with convoke is on the stack. "Convoke" means "For each colored mana in this spell's total cost, you may tap an untapped creature of that color you control rather than pay that mana. For each generic mana in this spell's total cost, you may tap an untapped creature you control rather than pay that mana."

**702.51b** The convoke ability isn't an additional or alternative cost and applies only after the total cost of the spell with convoke is determined.

**702.51c** A creature tapped to pay for mana in a spell's total cost this way is said to have "convoked" that spell.

**702.51d** Multiple instances of convoke on the same spell are redundant.

### Related Rules

**601.2f** The player determines the total cost of the spell. Usually this is just the mana cost. Some spells have additional or alternative costs. Some effects may increase or reduce the cost to pay, or may provide other alternative costs. Costs may include paying mana, tapping permanents, sacrificing permanents, discarding cards, and so on. The total cost is the mana cost or alternative cost (as determined in rule 601.2b), plus all additional costs and cost increases, and minus all cost reductions. If multiple cost reductions apply, the player may apply them in any order. If the mana component of the total cost is reduced to nothing by cost reduction effects, it is considered to be {0}. It can't be reduced to less than {0}. Once the total cost is determined, any effects that directly affect the total cost are applied. Then the resulting total cost becomes "locked in." If effects would change the total cost after this time, they have no effect.

**601.2g** If the total cost includes a mana payment, the player then has a chance to activate mana abilities (see rule 605, "Mana Abilities"). Mana abilities must be activated before costs are paid.

**601.2h** The player pays the total cost. First, they pay all costs that don't involve random elements or moving objects from the library to a public zone, in any order. Then they pay all remaining costs in any order. Partial payments are not allowed. Unpayable costs can't be paid.

## Key Edge Cases

1. **Convoke applies after total cost is determined (CR 702.51b).** Commander tax, additional costs, and cost reductions are all applied before convoke. Convoke does not change the mana cost or mana value of the spell.

2. **Colored mana matching (CR 702.51a).** A tapped creature of a specific color pays for one colored mana of that color in the total cost. A creature of any color (or colorless) can pay for {1} of generic mana. A multicolored creature can pay for any one of its colors.

3. **Cannot tap more creatures than the total cost allows.** From Venerated Loxodon ruling: "You can't tap more creatures to convoke than it takes to pay for its total cost." For Siege Wurm ({5}{G}{G}), at most 7 creatures can convoke.

4. **Summoning sickness does NOT prevent convoke.** From Siege Wurm ruling: "You can tap any untapped creature you control to convoke a spell, even one you haven't controlled continuously since the beginning of your most recent turn." Convoke tapping is not an activated ability cost with {T} -- it is part of casting cost payment.

5. **Mana ability interaction.** From multiple card rulings: "If a creature you control has a mana ability with {T} in the cost, activating that ability while casting a spell with convoke will result in the creature being tapped before you pay the spell's costs. You won't be able to tap it again for convoke." The engine sequences: mana abilities (601.2g) happen BEFORE cost payment (601.2h), so a creature tapped for mana is already tapped when convoke happens.

6. **Tapping attacking/blocking creatures.** From ruling: "Tapping an untapped creature that's attacking or blocking to convoke a spell won't cause that creature to stop attacking or blocking." Convoke is legal during combat (e.g., casting an instant with convoke).

7. **Convoke is not an alternative cost (CR 702.51b).** It can be used alongside alternative costs (e.g., flashback) and additional costs (e.g., commander tax, kicker).

8. **Multiple instances are redundant (CR 702.51d).** No special handling needed -- the ability is checked as a boolean presence, not multiplied.

9. **Convoke with X spells.** From Chord of Calling ruling: "When using convoke to cast a spell with {X} in its mana cost, first choose the value for X. That choice, plus any cost increases or decreases, will determine the spell's total cost. Then you can tap creatures you control to help pay that cost."

10. **Multiplayer relevance.** Convoke is extremely common in Commander token decks (Emmara, Trostani, etc.). The ability must work correctly with commander tax (additional cost stacks with convoke, per CR 702.51b + CR 601.2f).

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

No partial work exists yet; starting from scratch.

## Design Decision: How to Pass Convoke Creatures

### Problem

Convoke requires the player to choose which creatures to tap as part of casting the spell. The current `CastSpell` command has signature:
```rust
CastSpell {
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
}
```

There is no field for convoke creature choices.

### Approach: Add `convoke_creatures: Vec<ObjectId>` to `CastSpell`

**Rationale:**
- Convoke is part of the cost payment during casting (CR 601.2h), not a separate action.
- Adding a field to `CastSpell` is the most natural fit -- the player makes the convoke choice at cast time.
- The field defaults to an empty `Vec` for non-convoke spells (no behavioral change for existing code).
- This matches how targets are already passed as a `Vec<Target>` at cast time.

**Alternative rejected: New `CastSpellWithConvoke` command.**
This would be inconsistent with how flashback works (flashback reuses `CastSpell` and detects the zone). Adding a separate command for every cost modification keyword (convoke, delve, improvise, emerge) would create an explosion of command variants. A single field on `CastSpell` is extensible.

**Alternative rejected: Automatic convoke (engine chooses creatures).**
The player MUST choose which creatures to tap -- different color matchings produce different results. Automatic selection violates CR 702.51a.

### Impact on Existing Code

- `CastSpell` gains one field (`convoke_creatures: Vec<ObjectId>`) -- all existing call sites pass `vec![]` or update the harness to pass the field.
- `handle_cast_spell` gains a validation block after total cost determination.
- The harness `translate_player_action` for `"cast_spell"` gains an optional `convoke` array field.

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Convoke` variant after `Dredge(u32)` (line ~231)
**Pattern**: Follow `KeywordAbility::Cycling` at line 226 -- a bare marker variant with no payload.

```rust
/// CR 702.51: Convoke — tap creatures to pay mana costs.
/// "For each colored mana in this spell's total cost, you may tap an untapped
/// creature of that color you control rather than pay that mana. For each generic
/// mana in this spell's total cost, you may tap an untapped creature you control
/// rather than pay that mana."
/// CR 702.51d: Multiple instances are redundant.
Convoke,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add discriminant 30 for `KeywordAbility::Convoke` in the `HashInto` impl (after `Dredge` at discriminant 29, line ~340).

```rust
// Convoke (discriminant 30) -- CR 702.51
KeywordAbility::Convoke => 30u8.hash_into(hasher),
```

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::Convoke => "Convoke".to_string()` to `format_keyword` match (after `Dredge` at line ~593).

**Match arm sweep**: Grep for all `match` on `KeywordAbility` to find exhaustive patterns that need the new variant. Known locations:
- `state/hash.rs` (HashInto impl) -- covered above
- `tools/replay-viewer/src/view_model.rs` (format_keyword) -- covered above
- Any other exhaustive match patterns found by the runner

### Step 2: CastSpell Command Extension

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `convoke_creatures: Vec<ObjectId>` field to the `CastSpell` variant.

```rust
CastSpell {
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
    /// CR 702.51: Creatures to tap for convoke cost reduction.
    /// Empty vec for non-convoke spells. Each creature must be:
    /// - Untapped, on the battlefield, controlled by the caster
    /// - A creature (by current characteristics)
    /// - Not already listed (no duplicates)
    /// Colored creatures pay for one colored mana of their color;
    /// any creature pays for {1} generic. Validated in handle_cast_spell.
    convoke_creatures: Vec<ObjectId>,
}
```

**Impact**: Every existing `Command::CastSpell { player, card, targets }` pattern match must be updated to `Command::CastSpell { player, card, targets, convoke_creatures }` (or use `..` rest pattern where the field is not needed). The runner must grep for all `CastSpell` pattern matches and update them.

**Key files that construct or match `CastSpell`**:
- `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs` (match arm in `process_command`)
- `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs` (constructs `CastSpell` in `translate_player_action`)
- `/home/airbaggie/scutemob/crates/engine/tests/` (many test files construct `CastSpell`)
- `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` (if `Command` is hashed)

### Step 3: Rule Enforcement in casting.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Function**: `handle_cast_spell`
**Action**: After total cost determination (line ~159, after the `mana_cost` variable is set) and BEFORE mana payment (line ~253), add convoke validation and cost reduction logic.

**CR 702.51a + 702.51b enforcement:**

1. **Check spell has Convoke keyword**: If `convoke_creatures` is non-empty but the spell lacks `KeywordAbility::Convoke`, return error.

2. **Validate each creature in `convoke_creatures`**:
   - Must exist in `state.objects`
   - Must be on the battlefield (`zone == ZoneId::Battlefield`)
   - Must be controlled by the caster (`controller == player`)
   - Must be a creature (via `calculate_characteristics`, check `card_types.contains(&CardType::Creature)`)
   - Must be untapped (`status.tapped == false`)
   - No duplicates (check uniqueness)

3. **Compute convoke cost reduction** (CR 702.51a):
   - Start with the `mana_cost` as computed (including commander tax, flashback alt cost, etc.)
   - For each creature in `convoke_creatures`:
     - Get the creature's colors from `calculate_characteristics`
     - If the creature has a color that matches a remaining colored cost component (W/U/B/R/G) and that component is > 0, reduce that colored component by 1. For multicolored creatures, the player's choice of which color to pay is implicit in the ordering -- the engine can pick the first matching color, but ideally the player chooses. **Simplification for V1**: For multicolored creatures, try to match colored costs in WUBRG order; if no colored match, reduce generic by 1. This is deterministic and correct for the vast majority of cases. A future enhancement could allow explicit color choice per creature.
     - If the creature has no matching colored cost remaining, reduce generic cost by 1.
     - If neither colored nor generic cost can be reduced, return an error (too many creatures).

4. **Apply reduced cost**: Replace `mana_cost` with the convoke-reduced cost for the existing `can_pay_cost` / `pay_cost` flow.

5. **Tap the convoke creatures** (CR 601.2h): After successful cost validation, tap each creature and emit `PermanentTapped` events. Tapping happens at cost payment time. Note: the creatures might already be tapped by mana abilities (601.2g precedes 601.2h), so the untapped check at step 2 is important.

**Pseudocode location** (inserted into `handle_cast_spell` between cost determination and `can_pay_cost`):

```rust
// CR 702.51: Apply convoke cost reduction.
let mana_cost = if !convoke_creatures.is_empty() {
    // Validate spell has Convoke
    if !chars.keywords.contains(&KeywordAbility::Convoke) {
        return Err(GameStateError::InvalidCommand(
            "spell does not have convoke".into(),
        ));
    }
    // Validate and reduce cost
    apply_convoke_reduction(state, player, &convoke_creatures, mana_cost)?
} else {
    mana_cost
};
```

**New helper function** in `casting.rs`:

```rust
/// CR 702.51a: Validate convoke creatures and reduce the mana cost accordingly.
///
/// Each tapped creature pays for {1} generic or one colored mana matching
/// the creature's color. Returns the reduced ManaCost and taps the creatures.
/// Emits PermanentTapped events.
fn apply_convoke_reduction(
    state: &mut GameState,
    player: PlayerId,
    convoke_creatures: &[ObjectId],
    cost: Option<ManaCost>,
    events: &mut Vec<GameEvent>,
) -> Result<Option<ManaCost>, GameStateError> { ... }
```

### Step 3b: Trigger Wiring

**N/A.** Convoke is a static ability that functions during casting (CR 702.51a). It is not a triggered ability -- it modifies cost payment. No trigger wiring is needed.

However, note that `PermanentTapped` events will be emitted for each convoked creature. These could fire `SelfBecomesTapped` triggers on those creatures. The existing trigger infrastructure in `check_triggers` already handles `PermanentTapped` events, so this works automatically.

### Step 4: Harness Update

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Function**: `translate_player_action`
**Action**: Update the `"cast_spell"` arm to optionally read a `convoke` array from the action JSON and pass it to `Command::CastSpell`.

The action JSON would look like:
```json
{
    "action": "cast_spell",
    "player": "p1",
    "card": "Siege Wurm",
    "convoke": ["Llanowar Elves", "Elvish Mystic", "Saproling Token"]
}
```

The harness resolves each creature name to an ObjectId on the battlefield (using `find_on_battlefield`).

Also update `"cast_spell_flashback"` to pass `convoke_creatures: vec![]`.

**New struct field** in the action deserialization (if using structured JSON):
```rust
// In the PlayerAction or wherever actions are deserialized:
convoke: Option<Vec<String>>,  // creature names to tap for convoke
```

### Step 5: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/convoke.rs` (new file)

**Tests to write:**

1. **`test_convoke_basic_tap_creatures_reduce_cost`** (CR 702.51a)
   - Setup: Player has Siege Wurm ({5}{G}{G}) in hand, 5 green creature tokens on battlefield, 2 green mana in pool.
   - Cast Siege Wurm with 5 creatures convoked (5 pay generic), 2G from pool pays {G}{G}.
   - Assert: spell on stack, 5 creatures tapped, mana pool empty.

2. **`test_convoke_colored_mana_match`** (CR 702.51a)
   - Setup: Player has a {2}{W}{W} spell in hand, 2 white creature tokens on battlefield, 2 colorless mana in pool.
   - Cast with 2 white creatures convoked (pay {W}{W}), 2 colorless pays {2}.
   - Assert: spell on stack, creatures tapped, correct mana spent.

3. **`test_convoke_generic_mana_any_creature`** (CR 702.51a)
   - Setup: Player has a {3}{G} spell, 3 red creatures on battlefield, 1 green mana.
   - Cast with 3 red creatures convoked (pay {3} generic), 1G from pool pays {G}.
   - Assert: spell on stack, 3 red creatures tapped.

4. **`test_convoke_reject_no_keyword`** (CR 702.51a)
   - Setup: Player has a spell without Convoke in hand, creature on battlefield.
   - Cast with convoke_creatures non-empty.
   - Assert: error returned ("spell does not have convoke").

5. **`test_convoke_reject_tapped_creature`** (CR 702.51a)
   - Setup: Creature is already tapped.
   - Cast with that creature in convoke_creatures.
   - Assert: error returned.

6. **`test_convoke_reject_not_creature`** (CR 702.51a)
   - Setup: Artifact (non-creature) on battlefield.
   - Cast with artifact in convoke_creatures.
   - Assert: error returned.

7. **`test_convoke_reject_not_controlled`** (CR 702.51a)
   - Setup: Creature controlled by opponent.
   - Cast with opponent's creature.
   - Assert: error returned.

8. **`test_convoke_reject_too_many_creatures`** (CR 702.51a / ruling)
   - Setup: Spell costs {2}{G}. Player tries to convoke 4 creatures (only 3 mana in cost).
   - Assert: error returned.

9. **`test_convoke_with_commander_tax`** (CR 702.51b + CR 903.8)
   - Setup: Commander with Convoke costs {3}{G}{G}, has been cast once (tax = {2}).
   - Total cost = {5}{G}{G}. Convoke 5 creatures, pay {G}{G} from pool.
   - Assert: spell on stack, correct total cost paid.

10. **`test_convoke_no_summoning_sickness`** (ruling)
    - Setup: Creature entered this turn (no haste).
    - Cast with that creature convoked.
    - Assert: succeeds (convoke ignores summoning sickness).

11. **`test_convoke_zero_creatures`** (normal cast)
    - Setup: Spell with Convoke, cast with empty convoke_creatures.
    - Assert: normal mana payment, spell on stack.

12. **`test_convoke_multicolored_creature_pays_colored`** (ruling)
    - Setup: Spell costs {1}{W}{G}. One Selesnya (W/G) creature on battlefield.
    - Convoke with 1 creature paying {W} (or {G}).
    - Assert: creature tapped, one colored pip reduced.

**Pattern**: Follow `/home/airbaggie/scutemob/crates/engine/tests/cycling.rs` structure.

### Step 6: Card Definition

**Suggested card**: **Siege Wurm**

**Rationale**: Simple creature with Convoke + Trample, no other complex abilities. Cost {5}{G}{G} provides good test coverage of both generic and colored mana convoke. Both keywords are already implemented in the engine (Trample is validated). No ETB triggers, no additional effects to distract from the Convoke test.

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs`
**Card details** (from oracle):
- Name: Siege Wurm
- Mana Cost: {5}{G}{G}
- Type: Creature -- Wurm
- Oracle Text: Convoke, Trample
- P/T: 5/5
- Color Identity: G
- Keywords: Convoke, Trample

**Definition pattern**:
```rust
CardDefinition {
    card_id: cid("siege-wurm"),
    name: "Siege Wurm".to_string(),
    mana_cost: Some(ManaCost { green: 2, generic: 5, ..Default::default() }),
    types: types_sub(&[CardType::Creature], &["Wurm"]),
    oracle_text: "Convoke\nTrample".to_string(),
    keywords: vec![KeywordAbility::Convoke, KeywordAbility::Trample],
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Convoke),
        AbilityDefinition::Keyword(KeywordAbility::Trample),
    ],
    power: Some(5),
    toughness: Some(5),
    ..Default::default()
}
```

### Step 7: Game Script

**Suggested scenario**: "Player casts Siege Wurm using convoke with three green creatures and mana from lands."

**File**: `/home/airbaggie/scutemob/test-data/generated-scripts/stack/` (casting is a stack operation)
**Script name**: `0XX_siege_wurm_convoke_basic.json` (next available number in the directory)

**Scenario outline**:
1. Player 1 (active) starts in PreCombatMain with:
   - Siege Wurm in hand
   - 3 Llanowar Elves on battlefield (untapped, green creatures)
   - 2 Forests on battlefield (for {G}{G} mana)
   - 2 other lands on battlefield (for {2} generic mana)
2. Player 1 taps 2 Forests for {G}{G}.
3. Player 1 taps 2 other lands for {2} generic.
4. Player 1 casts Siege Wurm with convoke: taps 3 Llanowar Elves (each pays {1} generic from the {5} generic component). Remaining cost after convoke: {2}{G}{G} paid from pool.
5. All players pass priority.
6. Siege Wurm resolves, enters battlefield as 5/5 with Trample.
7. Assert: Siege Wurm on battlefield, 3 Llanowar Elves tapped, mana pool empty.

**Subsystem directory**: `/home/airbaggie/scutemob/test-data/generated-scripts/stack/`

## Interactions to Watch

1. **Convoke + Commander Tax (CR 702.51b + CR 903.8)**: Commander tax is an additional cost added to the total cost at step 601.2f. Convoke applies after the total cost is determined (702.51b), so it correctly helps pay the increased cost including tax. The engine's flow: `base_mana_cost` -> `apply_commander_tax` -> `apply_convoke_reduction` -> `pay_cost`.

2. **Convoke + Flashback (CR 702.51b)**: Flashback is an alternative cost (CR 702.34). Convoke is not an additional or alternative cost (CR 702.51b). They can be used together. The engine determines flashback cost first, then convoke reduces that total.

3. **Convoke + Ward/Hexproof**: If creatures are being tapped for convoke and a Ward trigger fires during the cast, the creatures are already tapped (cost paid at 601.2h). The Ward payment would be a separate cost. This is handled naturally by the stack-based priority flow.

4. **Convoke creatures and "tapped" triggers**: Tapping creatures for convoke emits `PermanentTapped` events. Cards with "whenever a creature becomes tapped" triggers (e.g., Emmara, Soul of the Accord) will see these. The engine's `check_triggers` already processes `PermanentTapped` events, so this works automatically.

5. **Convoke and `is_tapped` status for SBA checks**: After convoke creatures are tapped, they are correctly in `status.tapped = true` state. This affects blocking eligibility in the same turn if convoke was used during combat (e.g., casting an instant with convoke in the declare blockers step).

6. **Performance**: Convoke validation iterates over `convoke_creatures` (typically 0-7 creatures). This adds negligible overhead to `handle_cast_spell` -- well within the existing 205us full-turn-4p benchmark budget.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Convoke` variant |
| `crates/engine/src/state/hash.rs` | Add discriminant 30 to `HashInto` for `KeywordAbility` |
| `crates/engine/src/rules/command.rs` | Add `convoke_creatures: Vec<ObjectId>` to `CastSpell` |
| `crates/engine/src/rules/casting.rs` | Add `apply_convoke_reduction` helper; integrate into `handle_cast_spell` |
| `crates/engine/src/rules/engine.rs` | Update `CastSpell` match arm to pass `convoke_creatures` |
| `crates/engine/src/testing/replay_harness.rs` | Update `cast_spell` action to read `convoke` array |
| `tools/replay-viewer/src/view_model.rs` | Add `Convoke` arm to `format_keyword` |
| `crates/engine/tests/convoke.rs` | New test file: 12 tests |
| `crates/engine/src/cards/definitions.rs` | Add Siege Wurm card definition |
| All files matching `CastSpell { player, card, targets }` | Add `convoke_creatures` field (vec![] for non-convoke) |
