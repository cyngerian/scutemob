# Ability Plan: Squad

**Generated**: 2026-03-07
**CR**: 702.157
**Priority**: P4
**Similar abilities studied**: Replicate (CR 702.56) in `casting.rs:3562-3600`, `resolution.rs:1594-1610`, `tests/replicate.rs`; Myriad token-copy pattern in `resolution.rs:3566-3698`; Ravenous ETB trigger placement in `resolution.rs:1195-1237`

## CR Rule Text

702.157. Squad

702.157a Squad is a keyword that represents two linked abilities. The first is a static ability that functions while the creature spell with squad is on the stack. The second is a triggered ability that functions when the creature with squad enters the battlefield. "Squad [cost]" means "As an additional cost to cast this spell, you may pay [cost] any number of times" and "When this creature enters, if its squad cost was paid, create a token that's a copy of it for each time its squad cost was paid." Paying a spell's squad cost follows the rules for paying additional costs in rules 601.2b and 601.2f-h.

702.157b If a spell has multiple instances of squad, each is paid separately. If a permanent has multiple instances of squad, each triggers based on the payments made for that squad ability as it was cast, not based on payments for any other instance of squad.

## Key Edge Cases

- **Intervening-if**: "if its squad cost was paid" -- trigger only goes on the stack if squad_count > 0. Re-check at resolution is trivially true since count is immutable (same as Ravenous draw trigger).
- **Keyword must be present at trigger time (ruling 2022-10-07)**: "If, for some reason, the creature doesn't have the squad ability when it's on the battlefield, the ability won't trigger, even if you've paid the squad cost one or more times." Check `KeywordAbility::Squad` on the battlefield permanent using layer-resolved characteristics.
- **Spell countered = no trigger (ruling 2022-10-07)**: "If the spell is countered, the squad ability will not trigger, and no tokens will be created." This is naturally handled -- a countered spell never enters the battlefield.
- **Tokens are NOT cast (ruling 2022-10-07)**: "The tokens created by the squad ability aren't 'cast,' so any abilities that trigger when a spell is cast won't trigger for the copies." Tokens enter via `add_object` + `PermanentEnteredBattlefield` event, never through CastSpell.
- **Multiple instances (CR 702.157b)**: Each Squad instance triggers independently. V1: support single instance only (all Warhammer 40k cards have one Squad instance). Document as known limitation.
- **Token copies use copiable values (CR 707.2)**: Same as Myriad -- apply Layer 1 CopyOf continuous effect.
- **Multiplayer**: Token copies all enter under the caster's control. No defending-player considerations (unlike Myriad).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (CastSpell field, StackObject field, resolution ETB trigger)
- [ ] Step 3: Trigger wiring (SquadTrigger SOK + resolution arm)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Squad` variant
**Discriminant**: 137 (next after Discover = 136)
**Pattern**: Follow `KeywordAbility::Ravenous` -- Squad is a simple unit variant (the cost is on the CardDefinition, not the keyword itself)

**Hash**: `crates/engine/src/state/hash.rs`
- Add `KeywordAbility::Squad => 137u8.hash_into(hasher)` after the Discover arm

**Match arms to update** (grep for exhaustive `KeywordAbility` matches):
- `crates/engine/src/state/hash.rs` -- hash impl
- `tools/replay-viewer/src/view_model.rs` -- keyword display function (add `Squad => "Squad"` arm)

### Step 2: CastSpell Command Field

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `squad_count: u32` field to `Command::CastSpell`
**CR**: 702.157a -- "you may pay [cost] any number of times"
**Pattern**: Follow `replicate_count: u32` at line 222
**Doc comment**: `/// CR 702.157a: Number of times the squad cost was paid as an additional cost. 0 = not paid (no tokens). N = paid N times -> N token copies created on ETB.`
**Serde**: `#[serde(default)]`

### Step 3: StackObject Field

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `pub squad_count: u32` field to `StackObject` struct
**CR**: 702.157a -- carries squad count from cast to resolution
**Pattern**: Follow `x_value: u32` at line 292
**Serde**: `#[serde(default)]`
**Doc comment**: `/// CR 702.157a: Number of times the squad cost was paid. 0 = no squad. Propagated from CastSpell.squad_count at cast time; read at resolution for the ETB trigger.`

**Hash**: `crates/engine/src/state/hash.rs`
- Add `squad_count.hash_into(hasher)` in the StackObject HashInto impl, after `x_value`

### Step 4: Casting Integration

**File**: `crates/engine/src/rules/casting.rs`
**Action**: In `handle_cast_spell`:
1. Read `squad_count` from the `Command::CastSpell` destructure
2. Validate: if `squad_count > 0`, the spell must have `KeywordAbility::Squad` in its definition
3. Add squad cost payment: multiply the squad cost by `squad_count` and add to total mana cost (same pattern as replicate_count mana addition)
4. Set `squad_count` on the `StackObject` being created

**CR**: 601.2b, 601.2f-h -- additional cost payment rules
**Pattern**: Follow replicate_count handling in casting.rs (around line 3560)

**Squad cost source**: The squad cost is stored on the CardDefinition. Need to check where it's stored -- it should be in `AbilityDefinition` or as a parameter on the keyword. Since `KeywordAbility::Squad` is a unit variant, the cost must come from an `AbilityDefinition`. Check if we need `AbilityDefinition::Squad { cost: ManaCost }` or if the cost is just a ManaCost on the keyword.

**Decision**: Add `AbilityDefinition::Squad { cost: ManaCost }` (discriminant 54, after CollectEvidence = 53). The keyword `KeywordAbility::Squad` is for presence-checking; the AbilityDefinition carries the cost data. This follows the Replicate pattern where `KeywordAbility::Replicate` is a unit variant and the replicate cost comes from... actually, let me check.

Actually, looking at the Replicate pattern: `KeywordAbility::Replicate` exists AND the replicate cost must come from somewhere. Let me verify.

**Update after research**: Looking at how `replicate_count` mana is charged in casting.rs -- the replicate cost needs to be looked up from the card definition. If Replicate uses the card's mana cost as the replicate cost (common for replicate), we need to check that pattern. For Squad, all printed cards use `Squad {2}` or similar fixed costs, so we need `AbilityDefinition::Squad { cost: ManaCost }`.

**AbilityDefinition discriminant**: 54 (next after CollectEvidence = 53)
**Hash**: Add AbilityDefinition::Squad arm in hash.rs

### Step 5: GameObject Field

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `pub squad_count: u32` field to `GameObject`
**CR**: 702.157a -- stored at resolution for the ETB trigger to read
**Pattern**: Follow `x_value: u32` at line 636
**Serde**: `#[serde(default)]`
**Init sites** (must add `squad_count: 0` to every `GameObject` construction):
- `crates/engine/src/state/builder.rs` -- GameStateBuilder
- `crates/engine/src/effects/mod.rs` -- token creation
- `crates/engine/src/rules/resolution.rs` -- all GameObject construction sites (Myriad tokens, Embalm tokens, etc.)

**Hash**: `crates/engine/src/state/hash.rs`
- Add `squad_count.hash_into(hasher)` in the GameObject HashInto impl

### Step 6: Resolution -- Propagate squad_count and Place ETB Trigger

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: In the permanent-enters-battlefield section of spell resolution (around line 430-440 where `x_value` is propagated):
1. Add `obj.squad_count = stack_obj.squad_count;` to propagate from stack to permanent
2. After the `PermanentEnteredBattlefield` event emission (around line 1195-1237, following Ravenous pattern):
   - Check if the permanent has `KeywordAbility::Squad` in its layer-resolved characteristics (CR ruling: "doesn't have the squad ability when it's on the battlefield, the ability won't trigger")
   - Check `squad_count > 0` (intervening-if)
   - If both true, push a `PendingTrigger` with `kind: PendingTriggerKind::SquadETB` and carry `squad_count` via a new field `squad_count: Option<u32>` on `PendingTrigger`

**CR**: 702.157a -- "When this creature enters, if its squad cost was paid, create a token that's a copy of it for each time its squad cost was paid."

### Step 7: PendingTrigger Additions

**File**: `crates/engine/src/state/stubs.rs`
**Action**:
1. Add `SquadETB` variant to `PendingTriggerKind` enum (after `RavenousDraw`)
2. Add `pub squad_count: Option<u32>` field to `PendingTrigger` struct (after `soulbond_pair_target`)
**Pattern**: Follow `RavenousDraw` / `champion_filter` pattern

### Step 8: SquadTrigger StackObjectKind

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `SquadTrigger` variant to `StackObjectKind`
**Discriminant**: 52 (next after BloodrushAbility = 51)
**Fields**: `source_object: ObjectId, squad_count: u32`
**Pattern**: Follow `ReplicateTrigger` structure but simpler (no `original_stack_id` -- Squad creates tokens, not spell copies)

**Hash**: `crates/engine/src/state/hash.rs`
- Add SquadTrigger hash arm: `52u8.hash_into(hasher); source_object.hash_into(hasher); squad_count.hash_into(hasher);`

**Match arms to update** (exhaustive StackObjectKind matches):
- `crates/engine/src/rules/resolution.rs` -- the "counterable abilities" list (around line 5494-5515)
- `tools/replay-viewer/src/view_model.rs` -- stack_kind_info display
- `tools/tui/src/play/panels/stack_view.rs` -- TUI stack display

### Step 9: Flush PendingTrigger -> StackObject

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers` (the match on `PendingTriggerKind`), add:
```
PendingTriggerKind::SquadETB => {
    StackObjectKind::SquadTrigger {
        source_object: trigger.source,
        squad_count: trigger.squad_count.unwrap_or(0),
    }
}
```
**Pattern**: Follow `PendingTriggerKind::RavenousDraw` arm

### Step 10: SquadTrigger Resolution -- Create Token Copies

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution arm for `StackObjectKind::SquadTrigger { source_object, squad_count }`:
1. Check source_object is still on the battlefield (CR 400.7 -- if it left, no tokens)
2. Loop `squad_count` times, creating a token copy each iteration:
   a. Build `GameObject` with characteristics cloned from the source (same as Myriad at line 3601)
   b. Set `is_token: true`, `squad_count: 0` (tokens don't have squad payments)
   c. Call `state.add_object(token_obj, ZoneId::Battlefield)`
   d. Apply Layer 1 `CopyOf` continuous effect via `copy::create_copy_effect(state, token_id, source_object, controller)`
   e. Emit `TokenCreated` + `PermanentEnteredBattlefield` events
3. Emit `AbilityResolved` event

**CR**: 702.157a -- "create a token that's a copy of it for each time its squad cost was paid"
**Pattern**: Follow Myriad token creation loop at `resolution.rs:3585-3698`

**Important difference from Myriad**:
- Tokens are NOT tapped (enter normally)
- Tokens are NOT attacking (enter normally)
- No `myriad_exile_at_eoc` flag
- Tokens have summoning sickness (normal ETB behavior)

### Step 11: Counterable Abilities List

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `| StackObjectKind::SquadTrigger { .. }` to the counterable-abilities match arm (around line 5494-5515)
**Note**: If Squad trigger is countered (e.g., by Stifle), no tokens are created but the permanent stays on the battlefield.

### Step 12: Replay Harness

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**:
1. Add `cast_spell_squad` action type that sets `squad_count` on the CastSpell command
2. Ensure default `squad_count: 0` is set in all existing `cast_spell` paths
**Pattern**: Follow `cast_spell_replicate` at line 1567

### Step 13: Unit Tests

**File**: `crates/engine/tests/squad.rs`
**Tests to write**:

1. `test_squad_basic_one_payment` -- Cast a creature with Squad paying once; resolve spell; resolve SquadTrigger; assert 2 creatures on battlefield (original + 1 token copy)
   - CR 702.157a

2. `test_squad_multiple_payments` -- Cast with squad_count=3; resolve; assert 4 creatures (original + 3 tokens)
   - CR 702.157a

3. `test_squad_zero_payments` -- Cast with squad_count=0; resolve; assert no SquadTrigger on stack, 1 creature on battlefield
   - CR 702.157a intervening-if

4. `test_squad_tokens_are_copies` -- Verify token copies have same name, P/T, abilities as original
   - CR 707.2

5. `test_squad_rejected_without_keyword` -- Cast with squad_count > 0 on a creature without Squad keyword; assert rejection
   - Validation

6. `test_squad_trigger_requires_keyword_on_battlefield` -- Cast creature with Squad, squad_count=1; before resolution, remove Squad from the permanent (e.g., via Humility effect removing abilities); assert no SquadTrigger fires
   - Ruling 2022-10-07

7. `test_squad_tokens_not_cast` -- Verify tokens don't increment `spells_cast_this_turn`
   - Ruling 2022-10-07

**Pattern**: Follow `tests/replicate.rs` structure (helpers, card definition setup, pass_all pattern)

### Step 14: Card Definition (later phase)

**Suggested card**: Ultramarines Honour Guard
- Mana cost: {3}{W}
- Type: Creature -- Astartes Warrior
- P/T: 2/2
- Squad {2}
- Other creatures you control get +1/+1.
- Good test card because the lord effect lets us verify tokens get the buff from the original

**Alternative**: Arco-Flagellant ({2}{B}, 3/1, Squad {2}, can't block, has activated ability)

**Card lookup**: use `card-definition-author` agent

### Step 15: Game Script (later phase)

**Suggested scenario**: "Squad ETB creates token copies"
- P1 casts Ultramarines Honour Guard with squad_count=2 ({3}{W} + {2} + {2} = {7}{W})
- Spell resolves, permanent enters with Squad
- SquadTrigger fires, resolves
- Assert: 3 creatures on battlefield (original + 2 token copies)
- Assert: Each creature is 3/3 (2/2 base + 1/1 from lord effect on original, and each copy is also a lord so they all buff each other)

**Subsystem directory**: `test-data/generated-scripts/etb-triggers/`

## Interactions to Watch

- **Panharmonicon**: Squad's ETB trigger uses `SelfEntersBattlefield` event. Per MEMORY.md: `doubler_applies_to_trigger` only matches `AnyPermanentEntersBattlefield`, so Squad is NOT doubled by Panharmonicon. This is the same known limitation as Champion, Backup, etc. Document but do not fix (fix holistically later).
- **Humility / Dress Down**: If the permanent loses Squad before the trigger fires, no tokens (ruling 2022-10-07). The check must use layer-resolved characteristics at trigger-collection time.
- **Flickering**: If the creature is flickered before the SquadTrigger resolves, the trigger finds `source_object` gone from battlefield (CR 400.7) and creates no tokens. The re-entered creature is a new object with `squad_count: 0`.
- **Copy effects on the original**: Token copies use copiable values at trigger resolution time, not cast time. If an effect changes the original's characteristics between ETB and trigger resolution, tokens copy the modified version.
- **Commander tax**: Squad cost is an additional cost, orthogonal to commander tax. Both apply.
- **Convoke/Delve/Improvise**: These reduce mana costs, which are separate from squad additional costs. Squad costs are added after the base cost is determined.

## Files Modified (Summary)

1. `crates/engine/src/state/types.rs` -- KeywordAbility::Squad
2. `crates/engine/src/state/hash.rs` -- hash arms for KW, SO, GO, SOK, AbilDef
3. `crates/engine/src/rules/command.rs` -- squad_count on CastSpell
4. `crates/engine/src/state/stack.rs` -- squad_count on StackObject, SquadTrigger SOK
5. `crates/engine/src/state/game_object.rs` -- squad_count on GameObject
6. `crates/engine/src/state/stubs.rs` -- PendingTriggerKind::SquadETB, squad_count on PendingTrigger
7. `crates/engine/src/state/builder.rs` -- squad_count: 0 init
8. `crates/engine/src/effects/mod.rs` -- squad_count: 0 in token creation
9. `crates/engine/src/rules/casting.rs` -- validate + charge squad cost + propagate
10. `crates/engine/src/rules/resolution.rs` -- propagate to GO, place ETB trigger, resolve SquadTrigger
11. `crates/engine/src/rules/abilities.rs` -- flush SquadETB -> SquadTrigger SOK
12. `crates/engine/src/testing/replay_harness.rs` -- cast_spell_squad action
13. `crates/engine/src/cards/card_definition.rs` -- AbilityDefinition::Squad { cost }
14. `tools/replay-viewer/src/view_model.rs` -- KW + SOK display arms
15. `tools/tui/src/play/panels/stack_view.rs` -- SOK display arm
16. `crates/engine/tests/squad.rs` -- unit tests

## Discriminant Chain

| Type | Variant | Discriminant |
|------|---------|-------------|
| KeywordAbility | Squad | 137 |
| StackObjectKind | SquadTrigger | 52 |
| AbilityDefinition | Squad | 54 |
