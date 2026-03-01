# Ability Plan: Bushido

**Generated**: 2026-03-01
**CR**: 702.45
**Priority**: P4
**Similar abilities studied**: Exalted (CR 702.83, `crates/engine/tests/exalted.rs`, `crates/engine/src/state/builder.rs:398-416`), Battle Cry (CR 702.91, `crates/engine/tests/battle_cry.rs`, `crates/engine/src/state/builder.rs:439-463`)

## CR Rule Text

> 702.45. Bushido
>
> 702.45a Bushido is a triggered ability. "Bushido N" means "Whenever this creature blocks or becomes blocked, it gets +N/+N until end of turn." (See rule 509, "Declare Blockers Step.")
>
> 702.45b If a creature has multiple instances of bushido, each triggers separately.

## Key Edge Cases

- **Triggers on BOTH "blocks" and "becomes blocked"** (CR 702.45a): A creature with Bushido that blocks gets +N/+N. A creature with Bushido that becomes blocked (is attacking and has blockers declared against it) also gets +N/+N. These are two separate trigger conditions.
- **Does NOT double-trigger in the same combat**: If a Bushido creature blocks, it gets the bonus once (from the "blocks" condition). A Bushido creature that is blocked gets the bonus once (from the "becomes blocked" condition). A creature cannot both block AND become blocked in the same combat step -- it is either attacking (and may become blocked) or blocking (and blocks).
- **Multiple instances trigger separately** (CR 702.45b): A creature with Bushido 1 and Bushido 2 would trigger both, getting +1/+1 and +2/+2 (+3/+3 total).
- **Bonus is "until end of turn"**: Expires during cleanup step, same as Exalted.
- **N is variable**: Bushido N uses the number specified on the card (e.g., Bushido 2 = +2/+2). Must be stored as a parameter on the keyword.
- **Trigger fires at declare-blockers time** (CR 509.1i, CR 509.2a): The trigger is put on the stack before the active player gets priority in the declare blockers step.
- **Multiplayer**: Each defending player declares blockers independently. Each declaration can trigger Bushido on both the attackers (becoming blocked) and the blockers (blocking). The trigger fires per defending player's declaration.
- **"Becomes blocked" triggers once per attacker** (CR 509.3c): Even if multiple creatures block the same attacker, the attacker with Bushido triggers only once (it "becomes blocked" once).
- **Ruling (Fumiko)**: Variable Bushido calculates N at resolution time, not trigger time. For fixed-N Bushido, this distinction is irrelevant.
- **Ruling (Shape Stealer)**: Bushido acquired from copying resolves based on timing -- the bonus is calculated when the trigger resolves. For standard Bushido this is always the fixed N.
- **Ruling (Curtain of Light)**: Effects that cause a creature to become blocked do trigger Bushido.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (trigger wiring)
- [ ] Step 3: TriggerEvent variant for SelfBecomesBlocked
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Bushido(u32)` variant after `Ingest` (line ~638).
**Pattern**: Follow `KeywordAbility::Annihilator(u32)` at line ~291 -- parameterized keyword with u32.

```rust
/// CR 702.45: Bushido N -- triggered ability.
/// "Whenever this creature blocks or becomes blocked, it gets +N/+N
/// until end of turn."
/// CR 702.45b: Multiple instances trigger separately.
Bushido(u32),
```

**Hash**: Add to `crates/engine/src/state/hash.rs` in the `HashInto for KeywordAbility` impl (after line ~468, discriminant 75 for Ingest). Discriminant **77** (76 reserved for Flanking).

```rust
// Bushido (discriminant 77) -- CR 702.45
KeywordAbility::Bushido(n) => {
    77u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**View model**: Add to `tools/replay-viewer/src/view_model.rs` in the `keyword_to_string` function (after line ~687 for Ingest):

```rust
KeywordAbility::Bushido(n) => format!("Bushido {}", n),
```

**Match arms**: Grep for exhaustive `KeywordAbility` match expressions and add the new arm. Expected locations:
- `crates/engine/src/state/hash.rs` (HashInto impl) -- covered above
- `tools/replay-viewer/src/view_model.rs` (keyword_to_string) -- covered above
- `crates/engine/src/rules/combat.rs` -- no exhaustive match on keywords here
- Any future location that pattern-matches all keyword variants

### Step 2: TriggerEvent Variant for SelfBecomesBlocked

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `TriggerEvent::SelfBecomesBlocked` variant after `ControllerProliferates` (line ~198).
**CR**: 509.1h, 509.3c -- "An attacking creature with one or more creatures declared as blockers for it becomes a blocked creature."

```rust
/// CR 509.1h / CR 702.45a: Triggers when this attacking creature becomes blocked
/// (has one or more blockers declared against it). Used by the Bushido keyword.
/// The "becomes blocked" check is done at trigger-collection time in
/// `rules/abilities.rs` when processing `BlockersDeclared` events.
/// Triggers once per attacker regardless of how many creatures block it (CR 509.3c).
SelfBecomesBlocked,
```

**Hash**: Add to `crates/engine/src/state/hash.rs` in `HashInto for TriggerEvent` (after line ~1146, discriminant 17 for ControllerProliferates). Discriminant **18**.

```rust
// CR 509.1h / CR 702.45a: SelfBecomesBlocked trigger -- discriminant 18
TriggerEvent::SelfBecomesBlocked => 18u8.hash_into(hasher),
```

### Step 3: Trigger Generation in builder.rs

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add two `TriggeredAbilityDef` entries when `KeywordAbility::Bushido(n)` is encountered. Place after the Myriad block (around line ~690, after the last keyword trigger generation).
**CR**: 702.45a -- "Whenever this creature blocks or becomes blocked, it gets +N/+N until end of turn."
**Pattern**: Follow Exalted at lines 398-416 -- triggered ability with `ApplyContinuousEffect`, `CEFilter::Source`, `CEDuration::UntilEndOfTurn`, `ModifyBoth(N)`.

```rust
// CR 702.45a: Bushido N -- "Whenever this creature blocks or becomes
// blocked, it gets +N/+N until end of turn."
// Two TriggeredAbilityDefs per Bushido instance: one for SelfBlocks,
// one for SelfBecomesBlocked. Each triggers separately (CR 702.45b).
if let KeywordAbility::Bushido(n) = kw {
    let bushido_effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(*n as i32),
            filter: CEFilter::Source,
            duration: CEDuration::UntilEndOfTurn,
        }),
    };

    // Trigger 1: "Whenever this creature blocks"
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfBlocks,
        intervening_if: None,
        description: format!(
            "Bushido {} (CR 702.45a): Whenever this creature blocks, \
             it gets +{0}/+{0} until end of turn.", n
        ),
        effect: Some(bushido_effect.clone()),
    });

    // Trigger 2: "Whenever this creature becomes blocked"
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfBecomesBlocked,
        intervening_if: None,
        description: format!(
            "Bushido {} (CR 702.45a): Whenever this creature becomes blocked, \
             it gets +{0}/+{0} until end of turn.", n
        ),
        effect: Some(bushido_effect),
    });
}
```

**Note on Effect::clone()**: The `bushido_effect` needs to be cloned for the second push. `Effect` derives `Clone`, so this is safe.

### Step 4: Trigger Dispatch in abilities.rs

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `BlockersDeclared` event handler (line ~1444), add dispatch for `SelfBecomesBlocked` alongside the existing `SelfBlocks` dispatch.
**CR**: 509.1h -- the attacker becomes blocked when blockers are declared for it. 509.3c -- triggers once per attacker regardless of number of blockers.

Currently the code at line ~1444 looks like:
```rust
GameEvent::BlockersDeclared { blockers, .. } => {
    // SelfBlocks: fires on each creature that is blocking (CR 603.5).
    for (blocker_id, _) in blockers {
        collect_triggers_for_event(
            state, &mut triggers, TriggerEvent::SelfBlocks,
            Some(*blocker_id), None,
        );
    }
}
```

Add after the `SelfBlocks` loop:

```rust
// CR 509.1h / CR 702.45a: SelfBecomesBlocked -- fires on each
// ATTACKER that has at least one blocker declared against it.
// Collect unique attacker IDs to ensure each triggers only once
// (CR 509.3c: "generally triggers only once each combat").
let mut blocked_attackers: Vec<ObjectId> = blockers
    .iter()
    .map(|(_, attacker_id)| *attacker_id)
    .collect();
blocked_attackers.sort();
blocked_attackers.dedup();

for attacker_id in blocked_attackers {
    collect_triggers_for_event(
        state,
        &mut triggers,
        TriggerEvent::SelfBecomesBlocked,
        Some(attacker_id),
        None,
    );
}
```

**Key correctness detail**: The `dedup()` ensures an attacker blocked by multiple creatures triggers `SelfBecomesBlocked` only once (CR 509.3c). `ObjectId` must implement `Ord` for the sort (it does -- it's a `u64` newtype).

### Step 5: Unit Tests

**File**: `crates/engine/tests/bushido.rs` (new file)
**Tests to write**:

1. **`test_702_45a_bushido_blocker_gets_bonus`** -- CR 702.45a: A creature with Bushido 1 blocks. After the trigger resolves, it has +1/+1.
   - Setup: p1 attacks with a 3/3. p2 has a 2/2 with Bushido 1 and blocks.
   - Assert: After trigger resolves, the blocker is 3/3 (2+1/2+1).

2. **`test_702_45a_bushido_attacker_becomes_blocked`** -- CR 702.45a: A creature with Bushido 1 attacks and becomes blocked. After the trigger resolves, it has +1/+1.
   - Setup: p1 attacks with a 2/2 Bushido 1 creature. p2 blocks with a 3/3.
   - Assert: After trigger resolves, the attacker is 3/3 (2+1/2+1).

3. **`test_702_45a_bushido_does_not_double_trigger`** -- CR 702.45a: A single creature should get exactly one Bushido trigger per combat, not two. A blocker with Bushido blocks (SelfBlocks fires, SelfBecomesBlocked does NOT because this creature is blocking, not being blocked). An attacker with Bushido becomes blocked (SelfBecomesBlocked fires, SelfBlocks does NOT because this creature is attacking, not blocking).
   - Setup: p1 attacks with a 2/2 Bushido 1. p2 blocks with a 1/1 Bushido 1.
   - Assert: Exactly 2 triggers on the stack (one for attacker, one for blocker), not 4.

4. **`test_702_45b_bushido_multiple_instances`** -- CR 702.45b: Multiple Bushido instances on the same creature each trigger separately.
   - Setup: p1 attacks with a 1/1 that has Bushido 1 and Bushido 2. p2 blocks with a 5/5.
   - Assert: 2 triggers from the attacker's Bushido. After resolution, attacker is 4/4 (1+1+2/1+1+2).

5. **`test_702_45a_bushido_bonus_expires_eot`** -- CR 702.45a ("until end of turn"), CR 514.2: Bonus expires at cleanup.
   - Setup: Bushido creature blocks, trigger resolves.
   - Assert: After `expire_end_of_turn_effects`, creature returns to printed P/T.

6. **`test_702_45a_bushido_attacker_blocked_by_multiple`** -- CR 509.3c: Attacker with Bushido triggers only once even when blocked by two creatures.
   - Setup: p1 attacks with a 2/2 Bushido 1. p2 blocks with two creatures.
   - Assert: Only 1 Bushido trigger for the attacker (not 2).

7. **`test_702_45a_bushido_multiplayer`** -- Multiplayer: Bushido triggers from blockers declared by different defending players.
   - Setup: 4 players. p1 attacks p2 and p3 with Bushido creatures. Each defender blocks.
   - Assert: Bushido triggers fire for each blocking/blocked creature across defenders.

**Pattern**: Follow `crates/engine/tests/exalted.rs` for structure (helpers, import set, test naming).

**Imports needed**:
```rust
use mtg_engine::{
    calculate_characteristics, process_command, AttackTarget, CardRegistry, Command, GameEvent,
    GameState, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
```

### Step 6: Card Definition

**Suggested card**: Devoted Retainer
- Oracle text: "Bushido 1 (Whenever this creature blocks or becomes blocked, it gets +1/+1 until end of turn.)"
- Type: Creature -- Human Samurai
- Mana cost: {W}
- P/T: 1/1
- Only ability: Bushido 1

**File**: `crates/engine/src/cards/defs/devoted_retainer.rs`

```rust
// Devoted Retainer -- {W}, Creature -- Human Samurai 1/1; Bushido 1 (CR 702.45).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        name: "Devoted Retainer".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Samurai"]),
        oracle_text: "Bushido 1 (Whenever this creature blocks or becomes blocked, it gets +1/+1 until end of turn.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Bushido(1))],
    }
}
```

**Note**: The `build.rs` auto-discovers files in `defs/`, so no manual registration is needed.

### Step 7: Game Script (later phase)

**Suggested scenario**: Devoted Retainer (Bushido 1) blocks an attacker. The Bushido trigger fires, goes on the stack, resolves, and the blocker gets +1/+1 until end of turn.
**Subsystem directory**: `test-data/generated-scripts/combat/`
**Sequence number**: Check next available in `combat/` directory.

## Interactions to Watch

- **Combat step ordering**: Bushido triggers fire at `BlockersDeclared` time (CR 509.1i, 509.2a). The trigger goes on the stack and resolves BEFORE combat damage. This means a Bushido creature's bonus is in effect when combat damage is dealt.
- **Protection prevents blocking**: If a creature has protection from the attacker, it cannot block that attacker. Bushido never fires. No special handling needed -- the blocking legality check in `combat.rs` already prevents this.
- **First strike / double strike**: Bushido bonus applies to the combat damage step because it resolves in the declare-blockers step (before any damage step). Both first-strike and regular damage benefit from the Bushido bonus. No special handling needed.
- **"Put onto the battlefield blocking" (CR 509.4)**: A creature put onto the battlefield blocking is "blocking" but never "blocked" an attacking creature for trigger purposes. The current engine does not support this mechanic, so no Bushido interaction to worry about now.
- **Flanking interaction (Batch 2 sibling)**: Flanking gives -1/-1 to the blocker when it blocks a creature with Flanking. If the blocker also has Bushido, both triggers fire: the blocker gets -1/-1 from Flanking and +N/+N from Bushido. They are independent triggers that resolve in stack order.
- **Multiplayer declare-blockers**: In multiplayer, each defending player declares blockers independently (CR 509.1). The `handle_declare_blockers` function is called per defending player, emitting `BlockersDeclared` each time. Bushido triggers fire on each declaration independently. No special multiplayer handling needed -- the existing per-player declaration infrastructure handles it.

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `Bushido(u32)` variant |
| `crates/engine/src/state/hash.rs` | Add hash discriminant 77 for Bushido, discriminant 18 for SelfBecomesBlocked |
| `crates/engine/src/state/game_object.rs` | Add `TriggerEvent::SelfBecomesBlocked` variant |
| `crates/engine/src/state/builder.rs` | Add two `TriggeredAbilityDef` entries for Bushido |
| `crates/engine/src/rules/abilities.rs` | Add `SelfBecomesBlocked` dispatch in `BlockersDeclared` handler |
| `tools/replay-viewer/src/view_model.rs` | Add `Bushido(n)` to `keyword_to_string` |
| `crates/engine/tests/bushido.rs` | 7 tests (new file) |
| `crates/engine/src/cards/defs/devoted_retainer.rs` | Card definition (new file) |

## Infrastructure Note: SelfBecomesBlocked is Shared

The new `TriggerEvent::SelfBecomesBlocked` variant is NOT Bushido-specific. It implements the general "whenever this creature becomes blocked" trigger pattern from CR 509.3c. Other abilities that use this trigger condition include:

- Rampage (Batch 2.3: "Whenever this creature becomes blocked, it gets +N/+N for each creature blocking it beyond the first")
- Various card-specific abilities ("Whenever ~ becomes blocked, ...")

Adding it now as part of Bushido provides infrastructure for Rampage and other Batch 2 abilities that also need the "becomes blocked" trigger.
