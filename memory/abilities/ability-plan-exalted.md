# Ability Plan: Exalted

**Generated**: 2026-02-26
**CR**: 702.83
**Priority**: P2
**Similar abilities studied**: Prowess (CR 702.108) -- triggered keyword, +1/+1 via continuous effect, `builder.rs` keyword-to-trigger translation, `abilities.rs` dispatch in `check_triggers`

## CR Rule Text

```
702.83. Exalted

702.83a Exalted is a triggered ability. "Exalted" means "Whenever a creature you
control attacks alone, that creature gets +1/+1 until end of turn."

702.83b A creature "attacks alone" if it's the only creature declared as an attacker
in a given combat phase. See rule 506.5.
```

Referenced rule:

```
506.5. A creature attacks alone if it's the only creature declared as an attacker
during the declare attackers step. A creature is attacking alone if it's attacking
but no other creatures are. A creature blocks alone if it's the only creature
declared as a blocker during the declare blockers step. A creature is blocking alone
if it's blocking but no other creatures are.
```

## Key Edge Cases

1. **Multiple instances stack separately (CR 702.83a, rulings).** If you control three
   permanents with exalted, each one triggers independently. The attacking creature gets
   +1/+1 for each instance. "If a creature has multiple instances of exalted, each
   triggers separately." A creature with 2 exalted counters fires 2 triggers.

2. **"Attacks alone" = exactly one creature declared as attacker (CR 702.83b, 506.5).**
   Attacking one player with one creature AND a planeswalker with another means 2 declared
   attackers -- exalted does NOT trigger. The count is based on the `DeclareAttackers`
   command's attackers list length, not "which player is attacked."

3. **Creatures put onto the battlefield attacking do NOT count (rulings).** They were never
   "declared as attackers." If one creature was declared and then tokens enter attacking,
   exalted still triggers (it already triggered at declaration). Those tokens do NOT prevent
   the trigger from resolving.

4. **If multiple creatures are declared but all but one are removed from combat, exalted
   does NOT trigger (rulings).** The trigger condition is checked at declaration time, not
   later. Multiple creatures declared = no exalted.

5. **The bonus goes to the attacking creature, not to the permanent with exalted (rulings).**
   This is a critical difference from Prowess where the bonus goes to the source creature.
   Exalted on an enchantment, land, or non-attacking creature still gives +1/+1 to the
   lone attacker.

6. **Bonuses last until end of turn (CR 702.83a).** Survives into additional combat phases.
   If a creature attacks alone in a second combat phase, all exalted abilities trigger again.

7. **Exalted triggers on ALL permanents you control, not just creatures (rulings).** An
   enchantment with exalted (e.g., Finest Hour) triggers too. The trigger source is the
   permanent with exalted; the effect target is the lone attacker.

8. **Multiplayer (Commander): each player's exalted triggers independently.** Only the
   attacking player's exalted abilities trigger (it says "a creature YOU control attacks
   alone"). Opponents' exalted abilities do not trigger when you attack alone.

9. **Keyword counters (CR 122.1b):** Exalted can appear as a keyword counter. Each counter
   is a separate instance. Already handled by the keyword counter system if present.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Exalted` variant after `SplitSecond` (line ~255).
**Pattern**: Follow the simple keyword pattern (no payload) like `KeywordAbility::Prowess` at line 170.

```rust
/// CR 702.83: Exalted -- "Whenever a creature you control attacks alone,
/// that creature gets +1/+1 until end of turn."
///
/// Implemented as a triggered ability. builder.rs auto-generates a
/// TriggeredAbilityDef from this keyword at object-construction time.
/// Multiple instances on different permanents each trigger separately.
Exalted,
```

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Exalted => 34u8.hash_into(hasher)` in the `HashInto for KeywordAbility` match (after `SplitSecond => 33u8` at line 348).

**Match arms**: Grep for `KeywordAbility::SplitSecond` to find all match expressions that
need a new arm. Most will be `_` wildcard arms and need no change. Verify no exhaustive
matches exist that would fail to compile.

### Step 2: New TriggerEvent Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add a new `TriggerEvent::CreatureControllerControlsAttacksAlone` variant.

Actually, a simpler approach: use a new trigger event `ControllerCreatureAttacksAlone` that
fires on every permanent controlled by the attacking player when exactly one creature is
declared as an attacker.

```rust
/// CR 702.83a: Triggers on each permanent controlled by the attacking player
/// when exactly one creature is declared as an attacker ("attacks alone").
/// Used by the Exalted keyword. The "attacks alone" check is done at
/// trigger-collection time in `rules/abilities.rs`. The +1/+1 effect targets
/// the lone attacker (not the source), resolved via DeclaredTarget { index: 0 }.
ControllerCreatureAttacksAlone,
```

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `TriggerEvent::ControllerCreatureAttacksAlone => 11u8.hash_into(hasher)` in
the `HashInto for TriggerEvent` match (after `OpponentCastsSpell => 10u8` at line 941).

### Step 3: PendingTrigger -- New Field for Exalted Target

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add a new field `exalted_attacker_id: Option<ObjectId>` to `PendingTrigger`.
This carries the lone attacker's ObjectId so `flush_pending_triggers` can set it as
`Target::Object(attacker_id)` in the trigger's targets list.

```rust
/// CR 702.83a: The lone attacker's ObjectId for Exalted triggers.
///
/// Populated when a `ControllerCreatureAttacksAlone` trigger fires. At flush
/// time, this ID is set as `Target::Object(attacker_id)` at index 0 so the
/// effect's `CEFilter::DeclaredTarget { index: 0 }` resolves to the correct
/// creature (the lone attacker, not the exalted source).
/// `None` for all other trigger types.
#[serde(default)]
pub exalted_attacker_id: Option<ObjectId>,
```

**Hash**: This field is already implicitly covered because `PendingTrigger` is not part of
the state hash (it's transient queue state). No hash.rs update needed for this field.

### Step 4: Trigger Generation in builder.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Add an Exalted keyword-to-trigger translation block after the Prowess block
(line ~395).

**Pattern**: Follow the Prowess pattern at lines 378-395. The key difference:
- Trigger event: `ControllerCreatureAttacksAlone` (not `ControllerCastsNoncreatureSpell`)
- Effect: `ApplyContinuousEffect` with `CEFilter::DeclaredTarget { index: 0 }` (not `CEFilter::Source`)
- The `DeclaredTarget { index: 0 }` resolves to the lone attacker at effect execution time

```rust
// CR 702.83a: Exalted -- "Whenever a creature you control attacks alone,
// that creature gets +1/+1 until end of turn."
// Each keyword instance generates one TriggeredAbilityDef.
// The +1/+1 targets the lone attacker (DeclaredTarget), not the source.
if matches!(kw, KeywordAbility::Exalted) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::ControllerCreatureAttacksAlone,
        intervening_if: None,
        description: "Exalted (CR 702.83a): Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn.".to_string(),
        effect: Some(Effect::ApplyContinuousEffect {
            effect_def: Box::new(ContinuousEffectDef {
                layer: EffectLayer::PtModify,
                modification: LayerModification::ModifyBoth(1),
                filter: CEFilter::DeclaredTarget { index: 0 },
                duration: CEDuration::UntilEndOfTurn,
            }),
        }),
    });
}
```

### Step 5: Trigger Dispatch in abilities.rs (check_triggers)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add Exalted trigger dispatch inside the `AttackersDeclared` event handler
(after the SelfAttacks loop, around line 662).

**CR**: 702.83b -- "attacks alone" = exactly one creature declared as attacker.
**CR**: 702.83a -- triggers on ALL permanents the attacking player controls that have exalted.

```rust
// CR 702.83a/b: Exalted -- "Whenever a creature you control attacks alone."
// If exactly one creature is declared as an attacker, fire exalted triggers
// on ALL permanents controlled by the attacking player (not just the attacker).
if attackers.len() == 1 {
    let (lone_attacker_id, _) = &attackers[0];
    let exalted_sources: Vec<ObjectId> = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == *attacking_player)
        .map(|obj| obj.id)
        .collect();

    let pre_len = triggers.len();
    for obj_id in exalted_sources {
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::ControllerCreatureAttacksAlone,
            Some(obj_id),
            None,
        );
    }
    // Tag exalted triggers with the lone attacker's ObjectId so
    // flush_pending_triggers can set Target::Object(attacker_id) at index 0.
    for t in &mut triggers[pre_len..] {
        t.exalted_attacker_id = Some(*lone_attacker_id);
    }
}
```

The `attacking_player` is available from the `AttackersDeclared` event destructuring:
```rust
GameEvent::AttackersDeclared { attacking_player, attackers, .. } => {
```

### Step 6: Flush Trigger Wiring in abilities.rs (flush_pending_triggers)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add a branch in `flush_pending_triggers` for `exalted_attacker_id` so that the
trigger's target list includes `Target::Object(attacker_id)`.

At line ~946 (inside the `trigger_targets` computation), add after the
`triggering_player` branch:

```rust
} else if let Some(attacker_id) = trigger.exalted_attacker_id {
    vec![SpellTarget {
        target: Target::Object(attacker_id),
        zone_at_cast: None,
    }]
}
```

This ensures `DeclaredTarget { index: 0 }` in the effect resolves to the lone attacker.

### Step 7: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/exalted.rs`
**Pattern**: Follow the prowess test file at `/home/airbaggie/scutemob/crates/engine/tests/prowess.rs`.

**Tests to write**:

1. **`test_exalted_basic_attacks_alone_gives_plus_one`** -- CR 702.83a
   - Setup: p1 has a 2/2 creature with exalted on battlefield. p1 has another 1/1 creature.
   - p1 declares only the 1/1 as attacker.
   - Exalted triggers. After resolution, the 1/1 is now 2/2.
   - Verifies: the bonus goes to the attacker, not the exalted source.

2. **`test_exalted_does_not_trigger_with_multiple_attackers`** -- CR 702.83b
   - Setup: p1 has two creatures, one with exalted.
   - p1 declares both as attackers.
   - No exalted trigger fires. Stack has 0 triggered abilities.

3. **`test_exalted_multiple_instances_stack`** -- Rulings
   - Setup: p1 has three permanents with exalted (e.g., 2 creatures + 1 enchantment sim).
   - p1 declares one creature as attacker.
   - Three exalted triggers fire. After all resolve, the attacker has +3/+3.

4. **`test_exalted_on_non_attacker_permanent_targets_attacker`** -- CR 702.83a
   - Setup: p1 has creature A (no exalted) and creature B (with exalted, not attacking).
   - p1 declares creature A as the lone attacker.
   - Creature B's exalted triggers and gives +1/+1 to creature A (not B).

5. **`test_exalted_does_not_trigger_on_opponent_attack`** -- CR 702.83a ("you")
   - Setup: p1 has an exalted creature. p2 attacks alone.
   - p1's exalted does NOT trigger.

6. **`test_exalted_bonus_expires_at_end_of_turn`** -- CR 702.83a ("until end of turn")
   - Setup: Trigger exalted, resolve it, then call `expire_end_of_turn_effects`.
   - The creature returns to its printed P/T.

7. **`test_exalted_multiplayer_only_attacker_controller_triggers`** -- 4-player
   - Setup: p1 and p3 each have an exalted creature. p1 attacks alone.
   - Only p1's exalted triggers. p3's does not.

8. **`test_exalted_with_zero_attackers_does_not_trigger`** -- Edge case
   - Setup: p1 has an exalted creature. p1 declares no attackers (empty vec).
   - No exalted trigger fires.

### Step 8: Card Definition (later phase)

**Suggested card**: Qasali Pridemage -- simple creature with Exalted + sacrifice ability.
Also: Akrasan Squire (simplest possible exalted creature, 1/1 with exalted only).
**Card lookup**: use `card-definition-author` agent.

For the initial implementation, Akrasan Squire is the simplest test card:
- {W}, Creature -- Human Soldier 1/1, Exalted.
- No other abilities. Perfect for basic validation.

Noble Hierarch is the most Commander-relevant but requires mana abilities ({T}: Add {G},
{W}, or {U}) which adds complexity. Use as a second card.

### Step 9: Game Script (later phase)

**Suggested scenario**: "Exalted Voltron Attack"
- p1 controls two creatures with exalted + one creature attacking alone.
- Declare one attacker, both exalted triggers fire, resolve, attacker has +2/+2.
- Proceed to combat damage step, verify damage dealt includes exalted bonus.
**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

1. **Combat trigger ordering**: Exalted triggers go on the stack alongside SelfAttacks
   triggers. In APNAP order, the active player's triggers are ordered. Multiple exalted
   triggers from the same player can be ordered by that player. Since all exalted triggers
   have the same effect (+1/+1 to the same creature), the order doesn't matter functionally.

2. **Prowess + Exalted on same creature**: Both are independent triggered abilities. If a
   creature has both and attacks alone while the controller also cast a noncreature spell,
   both trigger independently. No interaction issues.

3. **Creatures entering attacking (e.g., Myriad, token copies)**: These creatures are NOT
   declared as attackers. They do not change the "attacks alone" determination. If one
   creature was declared and tokens enter attacking, exalted already triggered and resolves
   normally.

4. **Defender + Exalted**: A creature with both Defender and Exalted can't attack, but its
   Exalted still triggers when another creature you control attacks alone. No issue -- the
   trigger is on the permanent with exalted, the effect targets the attacker.

5. **Layer system**: The +1/+1 is a `PtModify` layer effect (`EffectLayer::PtModify`),
   same as Prowess. No dependency complications.

6. **Additional combat phases**: Exalted triggers again in each combat phase where a
   creature attacks alone. The `UntilEndOfTurn` duration ensures bonuses from the first
   combat survive into the second. This is already handled by the existing
   `CEDuration::UntilEndOfTurn` infrastructure.

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Exalted` variant |
| `crates/engine/src/state/game_object.rs` | Add `TriggerEvent::ControllerCreatureAttacksAlone` |
| `crates/engine/src/state/stubs.rs` | Add `exalted_attacker_id: Option<ObjectId>` to `PendingTrigger` |
| `crates/engine/src/state/hash.rs` | Add hash discriminants for new keyword (34) and trigger event (11) |
| `crates/engine/src/state/builder.rs` | Add Exalted keyword-to-trigger translation |
| `crates/engine/src/rules/abilities.rs` | Add exalted dispatch in `check_triggers` + target wiring in `flush_pending_triggers` |
| `crates/engine/tests/exalted.rs` | 8 unit tests |
