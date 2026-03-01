# Ability Plan: Rampage

**Generated**: 2026-03-01
**CR**: 702.23
**Priority**: P4
**Batch**: 2 (Combat Triggers -- Blocking)
**Similar abilities studied**: Battle Cry (`builder.rs:444`, `abilities.rs:1329`, `tests/battle_cry.rs`), Ingest (`stack.rs:424`, `abilities.rs:2223`, `resolution.rs:1643`)

## CR Rule Text

```
702.23. Rampage

702.23a Rampage is a triggered ability. "Rampage N" means "Whenever this creature
becomes blocked, it gets +N/+N until end of turn for each creature blocking it
beyond the first." (See rule 509, "Declare Blockers Step.")

702.23b The rampage bonus is calculated only once per combat, when the triggered
ability resolves. Adding or removing blockers later in combat won't change the bonus.

702.23c If a creature has multiple instances of rampage, each triggers separately.
```

## Key Edge Cases

- **CR 702.23b**: Bonus is calculated at RESOLUTION time, not trigger time. The count of
  blockers is snapshotted once when the trigger resolves. Adding/removing blockers after
  resolution does not change the bonus.
- **CR 702.23c**: Multiple instances of Rampage each trigger separately. A creature with
  Rampage 2 and Rampage 3 that is blocked by 3 creatures gets two separate triggers:
  one for +2/+2 * 2 = +4/+4, one for +3/+3 * 2 = +6/+6, totaling +10/+10.
- **Blocked by exactly 1 creature**: No bonus (0 creatures beyond the first). The trigger
  still fires but resolves with zero bonus.
- **CR 509.3c**: "Whenever [a creature] becomes blocked" triggers only once per combat
  for that creature, even if blocked by multiple creatures. This is consistent with how
  Rampage works -- one trigger per becoming-blocked event, with the bonus multiplied by
  blocker count.
- **Multiplayer**: In Commander, multiple defending players may declare blockers sequentially.
  Each BlockersDeclared event should check if any attackers just became blocked (went from
  unblocked to blocked). The SelfBecomesBlocked trigger fires ONCE per attacker per combat
  (CR 509.3c) even if additional blockers are added by later defenders.
- **Ruling (Varchild's War-Riders, 2007-09-16)**: "The rampage bonus is calculated only
  once per combat, when the triggered ability resolves. Adding or removing blockers later
  in combat won't change the bonus." -- confirms CR 702.23b.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- not started
- [ ] Step 2: TriggerEvent variant -- not started
- [ ] Step 3: StackObjectKind variant -- not started
- [ ] Step 4: Trigger dispatch in abilities.rs -- not started
- [ ] Step 5: Trigger flush in abilities.rs -- not started
- [ ] Step 6: Resolution in resolution.rs -- not started
- [ ] Step 7: builder.rs wiring -- not started
- [ ] Step 8: Unit tests -- not started

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Rampage(u32)` variant after `Ingest` (line ~638).
The `u32` parameter is the N value (e.g., Rampage 2 has N=2).

```rust
/// CR 702.23: Rampage N -- triggered ability.
/// "Whenever this creature becomes blocked, it gets +N/+N until end of turn
/// for each creature blocking it beyond the first."
/// CR 702.23b: Bonus calculated once at resolution time.
/// CR 702.23c: Multiple instances trigger separately.
Rampage(u32),
```

**Pattern**: Follow `KeywordAbility::Annihilator(u32)` at types.rs (parameterized keyword).
**Discriminant**: 78 (Flanking=76, Bushido=77, Rampage=78 per batch plan).

**Hash file**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm after `Ingest` (line ~468), before the closing `}`:

```rust
// Rampage (discriminant 78) -- CR 702.23
KeywordAbility::Rampage(n) => {
    78u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**Match arms to update** (grep for exhaustive matches on `KeywordAbility`):
- `hash.rs` KeywordAbility impl (covered above)
- Any display/debug formatting that matches on KeywordAbility variants

### Step 2: TriggerEvent Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `SelfBecomesBlocked` variant to `TriggerEvent` enum (after `SelfBlocks` at line ~141).

```rust
/// CR 509.3c: Triggers when this attacking creature becomes blocked.
/// Fires once per combat when at least one creature is declared as a
/// blocker for this attacker. Used by Rampage (CR 702.23a) and
/// Bushido (CR 702.45a) -- both are "becomes blocked" triggers.
SelfBecomesBlocked,
```

**Hash file**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm to `TriggerEvent` impl after `ControllerProliferates` (line ~1146):

```rust
// CR 509.3c / CR 702.23a: SelfBecomesBlocked trigger -- discriminant 18
TriggerEvent::SelfBecomesBlocked => 18u8.hash_into(hasher),
```

**Note**: This trigger event is shared infrastructure for Batch 2. Both Rampage and Bushido
will use `SelfBecomesBlocked`. Adding it here benefits Bushido's later implementation.

### Step 3: StackObjectKind Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `RampageTrigger` variant after `IngestTrigger` (line ~427):

```rust
/// CR 702.23a: Rampage N triggered ability on the stack.
///
/// When this trigger resolves:
/// 1. Count blockers for the source attacker from `state.combat`.
/// 2. Compute bonus = (blocker_count - 1) * rampage_n.
/// 3. If bonus > 0, apply +bonus/+bonus as a continuous effect
///    (UntilEndOfTurn) to the source creature.
///
/// CR 702.23b: Bonus calculated once at resolution. Later blocker
/// changes do not affect it.
/// CR 603.10: Source need not be on battlefield at resolution time.
RampageTrigger {
    source_object: ObjectId,
    rampage_n: u32,
},
```

**Hash file**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm in `StackObjectKind` impl after `IngestTrigger` (line ~1381):

```rust
// RampageTrigger (discriminant 19) -- CR 702.23a
StackObjectKind::RampageTrigger {
    source_object,
    rampage_n,
} => {
    19u8.hash_into(hasher);
    source_object.hash_into(hasher);
    rampage_n.hash_into(hasher);
}
```

**View model file**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add arm after `IngestTrigger` (line ~475):

```rust
StackObjectKind::RampageTrigger { source_object, .. } => {
    ("rampage_trigger", Some(*source_object))
}
```

**TUI stack_view file**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`
**Action**: Add arm after `IngestTrigger` (line ~82):

```rust
StackObjectKind::RampageTrigger { source_object, .. } => {
    ("Rampage: ".to_string(), Some(*source_object))
}
```

### Step 4: PendingTrigger Fields

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add two fields to `PendingTrigger` after `ingest_target_player` (line ~242):

```rust
/// CR 702.23a: If true, this pending trigger is a Rampage trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::RampageTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. The
/// `rampage_n` field carries the N parameter from the keyword.
#[serde(default)]
pub is_rampage_trigger: bool,
/// CR 702.23a: The N value of the Rampage keyword (e.g., 2 for Rampage 2).
///
/// Only meaningful when `is_rampage_trigger` is true.
#[serde(default)]
pub rampage_n: Option<u32>,
```

**Hash file**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add the two new fields to the `PendingTrigger` `hash_into` implementation
(after the `ingest_target_player` line, around line ~1060):

```rust
// CR 702.23a: rampage trigger fields
self.is_rampage_trigger.hash_into(hasher);
self.rampage_n.hash_into(hasher);
```

### Step 5: Trigger Dispatch in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Extend the `BlockersDeclared` handler (line ~1444) to dispatch
`SelfBecomesBlocked` triggers on each attacker that just became blocked.

After the existing `SelfBlocks` loop (lines 1446-1454), add:

```rust
// SelfBecomesBlocked: fires on each attacker that becomes blocked (CR 509.3c).
// Collect unique attacker IDs from blockers (an attacker may have multiple blockers).
// CR 509.3c: fires only once per attacker per combat, even if blocked by multiple creatures.
{
    let mut blocked_attackers: Vec<ObjectId> = blockers
        .iter()
        .map(|(_, attacker_id)| *attacker_id)
        .collect();
    blocked_attackers.sort();
    blocked_attackers.dedup();

    for attacker_id in blocked_attackers {
        let pre_len = triggers.len();
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::SelfBecomesBlocked,
            Some(attacker_id),
            None,
        );

        // CR 702.23a: Tag Rampage triggers with is_rampage_trigger and rampage_n.
        // Identify rampage triggers by checking the source object's keywords.
        if let Some(obj) = state.objects.get(&attacker_id) {
            for t in &mut triggers[pre_len..] {
                // Check if the triggered ability's description starts with "Rampage"
                // (set by builder.rs). If so, tag the PendingTrigger with rampage data.
                if let Some(ability_def) = obj
                    .characteristics
                    .triggered_abilities
                    .get(t.ability_index)
                {
                    if ability_def.description.starts_with("Rampage") {
                        t.is_rampage_trigger = true;
                        // Extract the N value from the keyword.
                        // The ability_index maps to a triggered ability generated
                        // from a Rampage(n) keyword in builder.rs.
                        // We find the matching Rampage keyword on the object.
                        for kw in &obj.characteristics.keywords {
                            if let KeywordAbility::Rampage(n) = kw {
                                // If multiple Rampage instances exist, match by
                                // checking if this trigger's description contains the N value.
                                // Each Rampage(n) generates its own trigger, so we check
                                // if the description contains the formatted N.
                                if ability_def.description.contains(&format!("Rampage {n}")) {
                                    t.rampage_n = Some(*n);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

**Important**: The `SelfBecomesBlocked` dispatch must only fire once per attacker per combat.
Since `BlockersDeclared` fires per defending player, and in multiplayer multiple defenders
can block the same attacker (if it attacks via Myriad or other means), we need to ensure
the trigger only fires the first time the attacker transitions from unblocked to blocked.

However, in practice: each attacker attacks one target (one defending player), so only that
defending player declares blockers for it. Multiple `BlockersDeclared` events won't include
the same attacker ID from different defenders. The dedup above handles the case where
multiple blockers block the same attacker within a single declaration.

### Step 6: Trigger Flush in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add a rampage trigger branch in `flush_pending_triggers` (the function that
converts `PendingTrigger` to `StackObject`). This goes in the chain of `is_*_trigger`
checks, after the `is_ingest_trigger` check (around line ~2223).

```rust
} else if trigger.is_rampage_trigger {
    // CR 702.23a: Rampage N "becomes blocked" trigger.
    // The rampage_n parameter was tagged by the BlockersDeclared handler.
    StackObjectKind::RampageTrigger {
        source_object: trigger.source,
        rampage_n: trigger.rampage_n.unwrap_or(1),
    }
}
```

### Step 7: Resolution in resolution.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add a resolution arm for `RampageTrigger` after the `IngestTrigger` arm
(around line ~1660). Also add `RampageTrigger` to the counter-spell passthrough match
(around line ~1779).

Resolution arm:

```rust
// CR 702.23a: Rampage N -- "Whenever this creature becomes blocked, it gets
// +N/+N until end of turn for each creature blocking it beyond the first."
// CR 702.23b: Bonus calculated once at resolution time.
StackObjectKind::RampageTrigger {
    source_object,
    rampage_n,
} => {
    // Count blockers for this attacker from combat state.
    let blocker_count = state
        .combat
        .as_ref()
        .map(|c| c.blockers_for(source_object).len())
        .unwrap_or(0);

    // CR 702.23a: "for each creature blocking it beyond the first"
    let beyond_first = blocker_count.saturating_sub(1);
    let bonus = (beyond_first as i32) * (rampage_n as i32);

    if bonus > 0 {
        // Check source is still on the battlefield.
        if state.objects.get(&source_object).map_or(false, |obj| {
            obj.zone == ZoneId::Battlefield
        }) {
            // Apply +bonus/+bonus as continuous effects (UntilEndOfTurn).
            let ts = state.timestamp_counter;
            state.timestamp_counter += 1;

            // Power bonus
            state.continuous_effects.push_back(ContinuousEffect {
                id: EffectId(ts),
                source: source_object,
                layer: EffectLayer::PtModify,
                modification: LayerModification::ModifyPower(bonus),
                filter: EffectFilter::SingleObject(source_object),
                duration: EffectDuration::UntilEndOfTurn,
                timestamp: ts,
            });

            let ts2 = state.timestamp_counter;
            state.timestamp_counter += 1;

            // Toughness bonus
            state.continuous_effects.push_back(ContinuousEffect {
                id: EffectId(ts2),
                source: source_object,
                layer: EffectLayer::PtModify,
                modification: LayerModification::ModifyToughness(bonus),
                filter: EffectFilter::SingleObject(source_object),
                duration: EffectDuration::UntilEndOfTurn,
                timestamp: ts2,
            });
        }
    }

    events.push(GameEvent::AbilityResolved {
        controller: stack_obj.controller,
        stack_object_id: stack_obj.id,
    });
}
```

Counter-spell passthrough (add `RampageTrigger` to the `|` list around line ~1779):

```rust
| StackObjectKind::RampageTrigger { .. }
```

### Step 8: builder.rs Wiring

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Add a Rampage block in the keyword-to-triggered-ability loop (after the
`BattleCry` block around line ~463, or in the appropriate alphabetical location).

```rust
// CR 702.23a: Rampage N -- "Whenever this creature becomes blocked, it
// gets +N/+N until end of turn for each creature blocking it beyond
// the first."
// Each keyword instance generates one TriggeredAbilityDef (CR 702.23c).
// The trigger uses SelfBecomesBlocked. The effect is `None` because
// resolution is handled by the custom RampageTrigger StackObjectKind
// (bonus is computed at resolution time from combat state, per CR 702.23b).
if let KeywordAbility::Rampage(n) = kw {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfBecomesBlocked,
        intervening_if: None,
        description: format!(
            "Rampage {n} (CR 702.23a): Whenever this creature becomes blocked, \
             it gets +{n}/+{n} until end of turn for each creature blocking \
             it beyond the first."
        ),
        effect: None, // Custom resolution via RampageTrigger
    });
}
```

### Step 9: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/rampage.rs`
**Pattern**: Follow `tests/battle_cry.rs` (same combat trigger test pattern).

**Tests to write**:

1. **`test_702_23a_rampage_blocked_by_two_gets_bonus`** -- Creature with Rampage 2 attacked
   and blocked by 2 creatures. After trigger resolves, creature has +2/+2 (1 beyond first * 2).
   Start at DeclareBlockers step, declare attackers first, then blockers.

2. **`test_702_23a_rampage_blocked_by_one_no_bonus`** -- Creature with Rampage 2 blocked by
   exactly 1 creature. Trigger fires (AbilityTriggered event) but resolves with no P/T change
   (0 beyond first). Verify P/T unchanged.

3. **`test_702_23a_rampage_blocked_by_three_scaled_bonus`** -- Creature with Rampage 2 blocked
   by 3 creatures. Bonus = 2 * 2 = +4/+4. Verify both power and toughness increased.

4. **`test_702_23c_multiple_rampage_instances`** -- Creature with Rampage 2 added twice,
   blocked by 3 creatures. Two triggers, each resolving to +4/+4. Total = +8/+8.

5. **`test_702_23b_bonus_calculated_at_resolution`** -- Verify that the blocker count is
   determined at resolution time (from combat state). This is implicitly tested by all
   the above tests since the trigger resolves after blockers are declared.

6. **`test_702_23a_rampage_not_blocked_no_trigger`** -- Creature with Rampage attacks but
   is not blocked. No trigger fires. Verify stack is empty after blockers declared, P/T
   unchanged.

7. **`test_702_23a_rampage_bonus_expires_at_end_of_turn`** -- After Rampage trigger resolves,
   call `expire_end_of_turn_effects` and verify the creature returns to printed P/T.

**Test setup pattern** (from battle_cry.rs):
- 2-player game, P1 active at DeclareAttackers step
- P1 has creature with `KeywordAbility::Rampage(N)` on battlefield
- P2 has blocker creatures on battlefield
- P1 declares attacker -> P2 declares blockers -> pass priority -> trigger resolves
- Assert P/T via `calculate_characteristics`

**Important test detail**: The test must first `DeclareAttackers`, then transition to
DeclareBlockers step (pass priority for all players), then `DeclareBlockers`. The
`BlockersDeclared` event triggers Rampage, and the trigger goes on the stack for
resolution during the DeclareBlockers priority window.

### Step 10: Card Definition (later phase)

**Suggested card**: Wolverine Pack
- Name: Wolverine Pack
- Cost: {2}{G}{G}
- Type: Creature -- Wolverine
- P/T: 2/4
- Abilities: Rampage 2
- Oracle: "Rampage 2 (Whenever this creature becomes blocked, it gets +2/+2 until end of turn for each creature blocking it beyond the first.)"
- File: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/wolverine_pack.rs`

### Step 11: Game Script (later phase)

**Suggested scenario**: Wolverine Pack (2/4, Rampage 2) attacks P2. P2 blocks with 3
creatures (e.g., three 1/1 tokens). Rampage trigger resolves, Wolverine Pack becomes 6/8.
Combat damage step assigns damage.
**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

- **Combat state access at resolution**: The `state.combat` must exist and contain
  blocker data when the RampageTrigger resolves. The trigger fires during the
  DeclareBlockers step (CR 509.2a) and resolves while still in combat, so
  `state.combat` should be present. But verify the test flow ensures combat state
  is populated.

- **UntilEndOfTurn cleanup**: The +N/+N bonus uses `EffectDuration::UntilEndOfTurn`,
  which is already handled by `expire_end_of_turn_effects()` in `layers.rs`. No new
  infrastructure needed.

- **SelfBecomesBlocked as shared infrastructure**: This trigger event will also be used
  by Bushido (same batch). The dispatch code in `abilities.rs` should be generic enough
  that both Rampage and Bushido can hook into it. Rampage tags triggers with
  `is_rampage_trigger`; Bushido will tag with `is_bushido_trigger`.

- **Multiplayer**: In 4-player Commander, if P1 attacks P2 and P3 with different
  creatures, each defending player declares blockers independently. The
  `BlockersDeclared` event fires per defending player. The `SelfBecomesBlocked`
  dispatch already deduplicates attacker IDs within a single declaration, and each
  attacker can only be in one player's blocker declaration (since each attacker targets
  one defending player).

- **Layer system**: The +N/+N is applied as two `ContinuousEffect` entries in
  `EffectLayer::PtModify` with `EffectFilter::SingleObject`. This matches the
  existing pattern for Battle Cry's `ModifyPower` effect.

- **Protection**: If the attacker has protection from a blocker's color, the blocker
  is already prevented from blocking by the protection system. If blocking is somehow
  forced (unlikely), protection prevents damage but doesn't prevent the "becomes
  blocked" trigger.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `Rampage(u32)` to `KeywordAbility` |
| `crates/engine/src/state/game_object.rs` | Add `SelfBecomesBlocked` to `TriggerEvent` |
| `crates/engine/src/state/stack.rs` | Add `RampageTrigger` to `StackObjectKind` |
| `crates/engine/src/state/stubs.rs` | Add `is_rampage_trigger`, `rampage_n` to `PendingTrigger` |
| `crates/engine/src/state/hash.rs` | Add hash arms for all new variants + fields |
| `crates/engine/src/state/builder.rs` | Add Rampage -> TriggeredAbilityDef wiring |
| `crates/engine/src/rules/abilities.rs` | Add SelfBecomesBlocked dispatch + RampageTrigger flush |
| `crates/engine/src/rules/resolution.rs` | Add RampageTrigger resolution arm + counter passthrough |
| `tools/replay-viewer/src/view_model.rs` | Add RampageTrigger arm |
| `tools/tui/src/play/panels/stack_view.rs` | Add RampageTrigger arm |
| `crates/engine/tests/rampage.rs` | 7 unit tests |
