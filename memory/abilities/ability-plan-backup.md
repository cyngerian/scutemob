# Ability Plan: Backup

**Generated**: 2026-03-06
**CR**: 702.165 (NOT 702.160 -- batch plan had wrong number; 702.160 is Prototype)
**Priority**: P4
**Similar abilities studied**: Graft (ETB trigger + counter placement, `abilities.rs:2423-2490`, `resolution.rs:2576-2630`, `stubs.rs:88-92`), Exploit (SelfEntersBattlefield trigger, `abilities.rs:2103-2115`, `abilities.rs:4026-4031`), ApplyContinuousEffect (Layer 6 ability granting, `effects/mod.rs:1413-1448`)

## CR Rule Text

702.165. Backup

702.165a Backup is a triggered ability. "Backup N" means "When this creature enters, put N +1/+1 counters on target creature. If that's another creature, it also gains the non-backup abilities of this creature printed below this one until end of turn." Cards with backup have one or more abilities printed after the backup ability. (Some cards with backup also have abilities printed before the backup ability.)

702.165b If a permanent enters the battlefield as a copy of a permanent with a backup ability or a token is created that is a copy of that permanent, the order of abilities printed on it is maintained.

702.165c Only abilities printed on the object with backup are granted by its backup ability. Any abilities gained by a permanent, whether due to a copy effect, an effect that grants an ability to a permanent, or an effect that creates a token with certain abilities, are not granted by a backup ability.

702.165d The abilities that a backup ability grants are determined as the ability is put on the stack. They won't change if the permanent with backup loses any abilities after the ability is put on the stack but before it resolves.

## Key Edge Cases

- **Self-targeting**: Backup can target the creature itself. If it does, it gets N +1/+1 counters but does NOT gain additional abilities (CR 702.165a: "if that's another creature").
- **Abilities determined at trigger time**: CR 702.165d -- the abilities to grant are locked in when the trigger goes on the stack, not at resolution. This means if the creature loses abilities (e.g., Dress Down) after the trigger is placed but before resolution, the target still gains the original abilities.
- **Only printed abilities, not gained ones**: CR 702.165c -- only abilities from the card definition are granted, not abilities from continuous effects or copy effects. This simplifies implementation: read abilities from `CardDefinition`, not from layer-resolved characteristics.
- **"Printed below" ordering**: CR 702.165a -- only abilities listed AFTER the Backup ability in oracle text. In our DSL, this maps to abilities after the `AbilityDefinition::Keyword(KeywordAbility::Backup(N))` entry in the `abilities` Vec.
- **Non-backup abilities only**: The granted abilities exclude the Backup keyword itself (CR 702.165a: "non-backup abilities").
- **Multiple Backup instances**: Each triggers separately (standard triggered ability rule). Each grants its own N counters and potentially different abilities (though in practice cards have one Backup instance).
- **Multiplayer**: Target creature can be any creature on the battlefield (not limited to controller's creatures). Standard targeting rules apply.
- **Fizzle**: If the target creature is no longer a legal target at resolution (left battlefield, gained hexproof from opponent, etc.), the entire trigger fizzles -- no counters, no abilities.
- **Interaction with Panharmonicon**: This is a SelfEntersBattlefield trigger. Per MEMORY.md, `doubler_applies_to_trigger` only matches `AnyPermanentEntersBattlefield`, so Backup triggers are NOT doubled by Panharmonicon in the current implementation. This is actually a known gap (SelfEntersBattlefield triggers aren't doubled), but it's consistent with other ETB self-triggers (PartnerWith, Hideaway, Exploit).

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

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Backup(u32)` variant after `Devour(u32)` (line ~1143).
**Pattern**: Follow `KeywordAbility::Graft(u32)` at line 1105 -- parameterized keyword with N value.
**Discriminant**: 125.
**Doc comment**:
```
/// CR 702.165: Backup N -- "When this creature enters, put N +1/+1 counters
/// on target creature. If that's another creature, it also gains the non-backup
/// abilities of this creature printed below this one until end of turn."
///
/// Triggered ability (CR 702.165a). The N value is the number of +1/+1 counters.
/// Multiple instances each trigger separately.
///
/// Discriminant 125.
Backup(u32),
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `KeywordAbility::Backup(n)` after the `Devour` arm.
**Pattern**: Follow `Graft(n)` at line 605-608 -- hash discriminant then N value.
```
KeywordAbility::Backup(n) => {
    125u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**Match arms to update** (grep for `KeywordAbility` exhaustive matches):
- `tools/replay-viewer/src/view_model.rs`: keyword display function -- add `KeywordAbility::Backup(n) => format!("Backup {n}")`
- Any other exhaustive matches on `KeywordAbility` found via grep.

### Step 2: PendingTriggerKind + StackObjectKind

**File**: `crates/engine/src/state/stubs.rs`
**Action 2a**: Add `PendingTriggerKind::Backup` variant after `Graft` (line ~92).
**Doc comment**: `/// CR 702.165a: Backup trigger -- fired when this creature enters the battlefield.`

**Action 2b**: Add field `backup_abilities` to `PendingTrigger` struct (after `graft_entering_creature`, line ~300).
**Type**: `Option<Vec<KeywordAbility>>`
**Purpose**: CR 702.165d -- abilities are determined at trigger time and locked in. Store the resolved keyword abilities to grant (already filtered to non-Backup, already filtered to "printed below" from the card definition).
**Doc comment**:
```
/// CR 702.165d: The keyword abilities to grant to the target creature.
///
/// Only meaningful when `kind == PendingTriggerKind::Backup`. Determined at
/// trigger time from the card definition's abilities printed below the Backup
/// keyword (CR 702.165a). Non-Backup keywords only (CR 702.165c).
#[serde(default)]
pub backup_abilities: Option<Vec<KeywordAbility>>,
```

**Action 2c**: Add field `backup_n` to `PendingTrigger` struct.
**Type**: `Option<u32>`
**Purpose**: The N value for how many +1/+1 counters to place.
```
/// CR 702.165a: The N value from Backup N -- how many +1/+1 counters to place.
///
/// Only meaningful when `kind == PendingTriggerKind::Backup`.
#[serde(default)]
pub backup_n: Option<u32>,
```

**File**: `crates/engine/src/state/stack.rs`
**Action 2d**: Add `StackObjectKind::BackupTrigger` variant. **Discriminant 46.**
```
/// CR 702.165a: Backup triggered ability on the stack.
///
/// "When this creature enters, put N +1/+1 counters on target creature.
/// If that's another creature, it also gains the non-backup abilities of
/// this creature printed below this one until end of turn."
///
/// At resolution: place N +1/+1 counters on the target creature. If the
/// target is a different creature from the source, register a Layer 6
/// UntilEndOfTurn continuous effect granting the stored keyword abilities.
///
/// Discriminant 46.
BackupTrigger {
    source_object: ObjectId,
    target_creature: ObjectId,
    counter_count: u32,
    /// Keyword abilities to grant (determined at trigger time per CR 702.165d).
    /// Empty if targeting self (CR 702.165a: "if that's another creature").
    abilities_to_grant: Vec<KeywordAbility>,
},
```

**File**: `crates/engine/src/state/hash.rs`
**Action 2e**: Add hash arm for `StackObjectKind::BackupTrigger` after `ScavengeAbility`.
```
StackObjectKind::BackupTrigger {
    source_object,
    target_creature,
    counter_count,
    abilities_to_grant,
} => {
    46u8.hash_into(hasher);
    source_object.hash_into(hasher);
    target_creature.hash_into(hasher);
    counter_count.hash_into(hasher);
    for kw in abilities_to_grant {
        kw.hash_into(hasher);
    }
}
```

**Match arms to update** (grep for `StackObjectKind` exhaustive matches):
- `tools/replay-viewer/src/view_model.rs`: `stack_kind_info()` -- add arm for `BackupTrigger`
- `tools/tui/src/play/panels/stack_view.rs`: add arm for `BackupTrigger`
- `crates/engine/src/rules/resolution.rs`: fizzle/counter handling (line ~4647-4651 area)

### Step 3: Trigger Wiring

**File**: `crates/engine/src/rules/abilities.rs`

**Action 3a -- Trigger detection in `check_triggers`** (in the `PermanentEnteredBattlefield` handler, near the SelfEntersBattlefield block, ~line 2004):

After the existing SelfEntersBattlefield trigger collection block (which handles Exploit, Hideaway, PartnerWith, etc.), add a Backup trigger block:

```
// CR 702.165a: Backup -- "When this creature enters, put N +1/+1 counters
// on target creature."
// Fires as a SelfEntersBattlefield trigger. Each Backup instance triggers
// separately.
{
    let entering_obj = state.objects.get(object_id);
    if let Some(obj) = entering_obj {
        let card_id = obj.card_id.clone();
        if let Some(cid) = card_id {
            if let Some(def) = state.card_registry.get(&cid) {
                // Find all Backup(N) instances and their positions in the abilities vec.
                for (idx, ability) in def.abilities.iter().enumerate() {
                    if let AbilityDefinition::Keyword(KeywordAbility::Backup(n)) = ability {
                        // CR 702.165d: Determine abilities at trigger time.
                        // CR 702.165a: "non-backup abilities printed below this one"
                        // CR 702.165c: Only printed abilities (from card def), not gained.
                        let abilities_below: Vec<KeywordAbility> = def.abilities[idx+1..]
                            .iter()
                            .filter_map(|a| match a {
                                AbilityDefinition::Keyword(kw)
                                    if !matches!(kw, KeywordAbility::Backup(_)) => Some(kw.clone()),
                                _ => None,
                            })
                            .collect();

                        triggers.push(PendingTrigger {
                            source: *object_id,
                            ability_index: idx,
                            controller: obj.controller,
                            kind: PendingTriggerKind::Backup,
                            triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                            entering_object_id: Some(*object_id),
                            backup_abilities: Some(abilities_below),
                            backup_n: Some(*n),
                            // All other fields default
                            ..Default::default() // or manually set all to None/default
                        });
                    }
                }
            }
        }
    }
}
```

**Important note on PendingTrigger default**: PendingTrigger does not derive `Default`. The runner must either manually set all fields to their defaults (None/0) or add a `Default` impl. Follow the pattern of other trigger pushes in this file (manually setting every field).

**Action 3b -- flush_pending_triggers** (in the `PendingTriggerKind` match in `flush_pending_triggers`, ~line 4026):

```
PendingTriggerKind::Backup => {
    // CR 702.165a: Backup ETB trigger.
    // Deterministic target selection: target self (gets counters but no abilities).
    // In a real game, the player would choose; for deterministic bot play,
    // targeting self is the simplest default.
    let target = trigger.source; // self-target as default
    let n = trigger.backup_n.unwrap_or(1);
    let abilities = trigger.backup_abilities.clone().unwrap_or_default();

    StackObjectKind::BackupTrigger {
        source_object: trigger.source,
        target_creature: target,
        counter_count: n,
        // Self-targeting: no abilities granted (CR 702.165a).
        abilities_to_grant: if target == trigger.source {
            vec![]
        } else {
            abilities
        },
    }
}
```

### Step 4: Resolution Logic

**File**: `crates/engine/src/rules/resolution.rs`

**Action 4a -- Add resolution arm** for `StackObjectKind::BackupTrigger` (near the `GraftTrigger` resolution at ~line 2576):

```
// CR 702.165a: Backup trigger resolves.
// 1. Put N +1/+1 counters on target creature.
// 2. If target is another creature, grant keyword abilities until EOT via
//    Layer 6 continuous effect.
StackObjectKind::BackupTrigger {
    source_object,
    target_creature,
    counter_count,
    abilities_to_grant,
} => {
    // Verify target is still on the battlefield (fizzle check).
    let target_exists = state.objects.get(&target_creature)
        .map(|o| matches!(o.zone, ZoneId::Battlefield))
        .unwrap_or(false);

    if !target_exists {
        // Target is gone; trigger fizzles. No counters, no abilities.
        events.push(GameEvent::SpellOrAbilityFizzled {
            stack_object_id: stack_obj_id,
        });
    } else {
        // 1. Place N +1/+1 counters on target.
        if let Some(obj) = state.objects.get_mut(&target_creature) {
            let current = obj.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
            obj.counters.insert(CounterType::PlusOnePlusOne, current + counter_count);
        }
        events.push(GameEvent::CountersAdded {
            object_id: target_creature,
            counter_type: CounterType::PlusOnePlusOne,
            count: counter_count,
        });

        // 2. If target is another creature and there are abilities to grant,
        //    register Layer 6 continuous effects (UntilEndOfTurn).
        if target_creature != source_object && !abilities_to_grant.is_empty() {
            let kw_set: im::OrdSet<KeywordAbility> = abilities_to_grant.into_iter().collect();
            let ts = state.timestamp_counter;
            state.timestamp_counter += 1;
            let id_inner = state.next_object_id().0;
            let eff = crate::state::continuous_effect::ContinuousEffect {
                id: crate::state::continuous_effect::EffectId(id_inner),
                source: Some(source_object),
                layer: crate::state::continuous_effect::Layer::AbilityAddingRemoving, // Layer 6
                modification: LayerModification::AddKeywords(kw_set),
                filter: crate::state::continuous_effect::EffectFilter::SingleObject(target_creature),
                duration: crate::state::continuous_effect::Duration::UntilEndOfTurn,
                is_cda: false,
                timestamp: ts,
            };
            state.continuous_effects.push_back(eff);
        }
    }
}
```

**Action 4b -- Add BackupTrigger to the fizzle/counter handling** (in the match arm that handles countering abilities, ~line 4647):

Add `| StackObjectKind::BackupTrigger { .. }` to the existing arm that handles `GraftTrigger`, `ScavengeAbility`, etc.

### Step 5: Unit Tests

**File**: `crates/engine/tests/backup.rs` (new file)

**Tests to write**:

1. `test_backup_self_target_gets_counters` -- CR 702.165a: Backup creature enters, triggers, resolves targeting itself. Verify N +1/+1 counters added. Verify NO additional abilities gained (self-target rule).

2. `test_backup_another_creature_gets_counters_and_abilities` -- CR 702.165a: Backup creature enters, trigger targets another creature. Verify N +1/+1 counters on target. Verify target gains the backup creature's keyword abilities (e.g., Flying, First Strike, Lifelink for Boon-Bringer Valkyrie). Verify abilities expire at end of turn (check after cleanup step).

3. `test_backup_abilities_expire_at_end_of_turn` -- Verify the granted abilities are removed during cleanup step (UntilEndOfTurn duration). Use 2+ player setup per testing gotcha.

4. `test_backup_does_not_grant_backup_keyword` -- CR 702.165a: The granted abilities must exclude Backup itself. Even if the card has Backup listed as a keyword, it should not appear in the granted set.

5. `test_backup_only_grants_abilities_below` -- CR 702.165a: Only abilities printed BELOW (after in Vec) the Backup entry are granted. Abilities before Backup in the definition are not granted.

6. `test_backup_target_leaves_battlefield_fizzle` -- If the target creature leaves the battlefield before resolution, the trigger fizzles: no counters, no abilities.

7. `test_backup_enum_variant_exists` -- Verify `KeywordAbility::Backup(1)` can be constructed and is distinct from other variants. Pattern: simple construction + match test.

**Pattern**: Follow Graft tests in `crates/engine/tests/graft.rs` for setup structure (GameStateBuilder, card definitions with the keyword, ETB trigger flow).

**Note**: Tests 2, 3, and 5 require a card definition with abilities listed after Backup (e.g., a test-only card with Backup 1 + Flying + Lifelink). The test file should define inline `CardDefinition` structs for this purpose rather than depending on specific card definitions in `defs/`.

### Step 6: Card Definition (later phase)

**Suggested card**: Boon-Bringer Valkyrie ({3}{W}{W}, Creature -- Angel Warrior, 4/4, Backup 1, Flying, First Strike, Lifelink). Good test card because it has three keyword abilities below Backup.

**Alternative simpler card**: Backup Agent ({1}{W}, Creature -- Human Citizen, 1/1). However, Backup Agent's oracle text doesn't actually have the Backup keyword -- it just says "put a +1/+1 counter on target creature" without the keyword. **Use Boon-Bringer Valkyrie instead.**

**Card lookup**: use `card-definition-author` agent.

### Step 7: Game Script (later phase)

**Suggested scenario**: Boon-Bringer Valkyrie enters targeting another creature. Verify counters placed and abilities granted. Second turn passes to verify abilities expire.

**Subsystem directory**: `test-data/generated-scripts/baseline/` (or a new `abilities/` directory if one exists).

## Interactions to Watch

- **Layer 6 timing with Humility**: If Humility is on the battlefield, it removes all abilities in Layer 6. The Backup continuous effect also operates in Layer 6. Timestamp ordering determines which wins. If Humility has a later timestamp, the granted abilities are removed. If the Backup CE has a later timestamp, the abilities survive. The implementation using `ContinuousEffect` with proper timestamps handles this correctly.

- **Stifle / counterspell interaction**: BackupTrigger is a triggered ability on the stack. It can be countered by Stifle or similar. If countered, no counters and no abilities. The fizzle/counter arm in resolution.rs (Step 4b) handles this.

- **Panharmonicon**: SelfEntersBattlefield triggers are currently NOT doubled by Panharmonicon (known gap). This is consistent with other self-ETB triggers but technically incorrect for Backup -- Panharmonicon should double it if the creature is an artifact or creature entering. This is a pre-existing gap, not new to Backup.

- **CR 702.165c -- only printed abilities**: The implementation reads from `CardDefinition.abilities`, which is the printed card text. This naturally excludes gained abilities per CR 702.165c. If the source is a copy (Copy effect in Layer 1), the copy's copiable abilities would be the "printed" ones -- but since we read from CardDefinition (the original), we might miss copy effects. This is acceptable for now since copy-of-Backup is an edge case, and CR 702.165b says the order is maintained for copies.

- **CR 702.165d -- abilities locked at trigger time**: The implementation stores `backup_abilities` on `PendingTrigger` at trigger-check time (Step 3a), which correctly locks them in before resolution. If the source loses abilities between trigger and resolution, the stored list is unaffected.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| `KeywordAbility` | `Backup(u32)` | 125 |
| `StackObjectKind` | `BackupTrigger` | 46 |
| `PendingTriggerKind` | `Backup` | (enum, no explicit discriminant) |

No new `AbilityDefinition` variant needed -- Backup uses `AbilityDefinition::Keyword(KeywordAbility::Backup(N))`.
