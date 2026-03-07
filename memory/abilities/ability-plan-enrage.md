# Ability Plan: Enrage

**Generated**: 2026-03-07
**CR**: Ability word (CR 207.2c) -- no individual CR entry; "Whenever this creature is dealt damage, [effect]"
**Priority**: P4
**Batch**: 12 (Ability Words)
**Similar abilities studied**: WhenDies trigger (replay_harness.rs:1951-1964), CombatDamageDealt handler (abilities.rs:4067-4166), WhenDealsCombatDamageToPlayer (replay_harness.rs:2016-2035)

## CR Rule Text

CR 207.2c: "An ability word appears in italics at the beginning of some abilities. Ability words are similar to keywords in that they tie together cards that have similar functionality, but they have no special rules meaning and no individual entries in the Comprehensive Rules."

Enrage is listed in the ability word enumeration at CR 207.2c.

The trigger pattern is: "Whenever this creature is dealt damage, [effect]."

Relevant damage rules:
- CR 120.3: Damage dealt to a creature is marked on that creature.
- CR 603.2g: An ability that triggers when damage is dealt does NOT trigger if the damage is prevented (reduced to 0).
- CR 510.2: All combat damage is dealt simultaneously in a single batch.

## Key Edge Cases

- **Multiple sources, single trigger**: Per Ripjaw Raptor ruling (2018-01-19): "If multiple sources deal damage to a creature with an enrage ability at the same time, most likely because multiple creatures blocked that creature, the enrage ability triggers only once." This is because combat damage is dealt simultaneously -- one damage event, one trigger.
- **Lethal damage still triggers**: Per ruling (2018-01-19): "If lethal damage is dealt to a creature with an enrage ability, that ability triggers. The creature with that enrage ability leaves the battlefield before that ability resolves, so it won't be affected by the resolving ability."
- **Damage prevention blocks trigger**: Per CR 603.2g, if ALL damage is prevented (final amount = 0), the trigger does not fire.
- **Non-combat damage triggers too**: Enrage says "is dealt damage" -- not "combat damage." A Lightning Bolt targeting a creature with Enrage triggers it.
- **Combat damage to creatures**: The `CombatDamageDealt` event contains per-assignment entries. Multiple assignments to the same creature (from multiple blockers) must deduplicate -- fire enrage only ONCE per creature per damage step.
- **Multiplayer**: No special multiplayer considerations beyond standard trigger ordering (APNAP).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant (TriggerCondition + TriggerEvent)
- [ ] Step 2: Rule enforcement (trigger wiring in check_triggers)
- [ ] Step 3: Trigger wiring (enrich_spec_from_def mapping)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Add TriggerCondition and TriggerEvent Variants

Enrage is an **ability word**, not a keyword. Per the gotcha in `memory/gotchas-infra.md`: "Keyword actions are Effects, NOT KeywordAbility enum variants." Similarly, ability words should NOT get a `KeywordAbility` variant. Enrage is implemented purely as a trigger condition that card definitions use.

**No `KeywordAbility::Enrage` variant needed.** No changes to `state/types.rs`.

#### Step 1a: TriggerCondition variant

**File**: `crates/engine/src/cards/card_definition.rs` (line ~1143, after `TributeNotPaid`)
**Action**: Add `WhenDealtDamage` variant to `TriggerCondition` enum

```rust
/// "Whenever this creature is dealt damage" -- ability word Enrage (CR 207.2c).
/// Fires when the source creature receives > 0 damage from any source (combat
/// or non-combat). If multiple sources deal damage simultaneously (e.g., combat),
/// triggers only once per damage event (CR 510.2).
WhenDealtDamage,
```

#### Step 1b: TriggerEvent variant

**File**: `crates/engine/src/state/game_object.rs` (line ~207, after `SelfAttacksWithGreaterPowerAlly`)
**Action**: Add `SelfIsDealtDamage` variant to `TriggerEvent` enum

```rust
/// CR 207.2c / CR 120.3: Triggers when this creature is dealt damage (> 0 after
/// prevention). Used by the Enrage ability word. Fires once per simultaneous
/// damage event, regardless of how many sources dealt damage at the same time.
SelfIsDealtDamage,
```

#### Step 1c: Hash for TriggerCondition

**File**: `crates/engine/src/state/hash.rs` (line ~3177, after `TributeNotPaid => 21u8`)
**Action**: Add hash discriminant for `WhenDealtDamage`

```rust
TriggerCondition::WhenDealtDamage => 22u8.hash_into(hasher),
```

#### Step 1d: Hash for TriggerEvent

**File**: `crates/engine/src/state/hash.rs` (line ~1407, after `SelfAttacksWithGreaterPowerAlly => 19u8`)
**Action**: Add hash discriminant for `SelfIsDealtDamage`

```rust
TriggerEvent::SelfIsDealtDamage => 20u8.hash_into(hasher),
```

#### Step 1e: helpers.rs export (if needed)

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: Check if `TriggerCondition` is already exported. If not, add it. (It likely is already since other card defs use it.)

### Step 2: Rule Enforcement -- Not Applicable

Enrage has no enforcement rules (no blocking restriction, no cost modification, no SBA). It is purely a triggered ability that fires when damage is dealt. All enforcement is in the trigger wiring (Step 3).

### Step 3: Trigger Wiring

#### Step 3a: enrich_spec_from_def mapping

**File**: `crates/engine/src/testing/replay_harness.rs` (line ~2116, after the `WhenDealsCombatDamageToPlayer` block)
**Action**: Add a block to map `TriggerCondition::WhenDealtDamage` to `TriggerEvent::SelfIsDealtDamage`
**Pattern**: Follow the `WhenDies` mapping at lines 1951-1964

```rust
// CR 207.2c: Convert "Whenever ~ is dealt damage" (Enrage ability word) triggers
// into runtime TriggeredAbilityDef entries.
for ability in &def.abilities {
    if let AbilityDefinition::Triggered {
        trigger_condition: TriggerCondition::WhenDealtDamage,
        effect,
        ..
    } = ability
    {
        spec = spec.with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfIsDealtDamage,
            intervening_if: None,
            description: "Enrage -- Whenever this creature is dealt damage (CR 207.2c)".to_string(),
            effect: Some(effect.clone()),
        });
    }
}
```

#### Step 3b: CombatDamageDealt handler -- fire SelfIsDealtDamage

**File**: `crates/engine/src/rules/abilities.rs` (after line ~4085, inside the `CombatDamageDealt` handler)
**Action**: After processing `SelfDealsCombatDamageToPlayer` and keyword triggers, add a section that collects unique creature ObjectIds that received > 0 damage, then calls `collect_triggers_for_event` for each with `SelfIsDealtDamage`.

**Important**: Per the ruling, multiple simultaneous damage sources trigger Enrage only once. The `CombatDamageDealt` event contains all assignments in one batch. We must deduplicate -- collect the set of creatures that were dealt damage, then fire `SelfIsDealtDamage` once per unique creature.

```rust
// Enrage / "Whenever this creature is dealt damage" (CR 207.2c):
// Collect unique creatures that received > 0 damage in this combat damage step.
// Multiple simultaneous sources trigger only once per creature (ruling 2018-01-19).
let mut damaged_creatures: Vec<ObjectId> = Vec::new();
for assignment in assignments {
    if assignment.amount == 0 {
        continue;
    }
    if let CombatDamageTarget::Creature(creature_id) = &assignment.target {
        if !damaged_creatures.contains(creature_id) {
            damaged_creatures.push(*creature_id);
        }
    }
}
for creature_id in damaged_creatures {
    collect_triggers_for_event(
        state,
        &mut triggers,
        TriggerEvent::SelfIsDealtDamage,
        Some(creature_id),
        None,
    );
}
```

#### Step 3c: DamageDealt handler -- fire SelfIsDealtDamage for non-combat damage

**File**: `crates/engine/src/rules/abilities.rs` (in the `check_triggers` match, add a new arm for `GameEvent::DamageDealt`)
**Action**: Add a handler for `GameEvent::DamageDealt` that fires `SelfIsDealtDamage` when the target is a creature with amount > 0.

```rust
GameEvent::DamageDealt { source: _, target, amount } => {
    if *amount == 0 {
        // CR 603.2g: prevented damage does not trigger.
        continue;
    }
    // Enrage / "Whenever this creature is dealt damage" (CR 207.2c):
    // Non-combat damage to a creature fires SelfIsDealtDamage on that creature.
    if let CombatDamageTarget::Creature(creature_id) = target {
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::SelfIsDealtDamage,
            Some(*creature_id),
            None,
        );
    }
}
```

**Note**: The `DamageDealt` event uses `CombatDamageTarget` for its target field (despite being non-combat damage). The enum name is somewhat misleading but this is the existing type.

### Step 4: Unit Tests

**File**: `crates/engine/tests/enrage.rs` (new file)
**Pattern**: Follow `crates/engine/tests/renown.rs` or `crates/engine/tests/ingest.rs` for combat damage trigger tests

**Tests to write**:

1. **`test_enrage_combat_damage_triggers`**
   - CR 207.2c / CR 120.3 -- creature with Enrage receives combat damage
   - Setup: P1 has a creature with `TriggerCondition::WhenDealtDamage` and effect `DrawCards { player: Controller, count: 1 }`. P2 has an attacker.
   - Attack P1, declare the enrage creature as blocker, deal combat damage
   - Assert: Enrage trigger fires, P1 draws a card

2. **`test_enrage_noncombat_damage_triggers`**
   - CR 207.2c -- creature with Enrage is dealt damage by a spell (DealDamage effect)
   - Setup: P1 has a creature with Enrage. P2 casts a damage spell targeting that creature.
   - Assert: Enrage trigger fires after spell damage resolves

3. **`test_enrage_prevented_damage_no_trigger`**
   - CR 603.2g -- if all damage is prevented, Enrage does NOT trigger
   - Setup: creature with Enrage has damage prevention (e.g., protection or prevention shield)
   - Deal damage that is fully prevented (final_dmg = 0)
   - Assert: No Enrage trigger fires

4. **`test_enrage_multiple_blockers_triggers_once`**
   - Ruling 2018-01-19 -- multiple simultaneous combat damage sources trigger only once
   - Setup: creature with Enrage attacks, is blocked by two creatures
   - Both blockers deal damage simultaneously in combat damage step
   - Assert: Exactly one Enrage trigger fires (not two)

5. **`test_enrage_lethal_damage_still_triggers`**
   - Ruling 2018-01-19 -- lethal damage triggers Enrage; creature dies before resolution
   - Setup: creature with Enrage (e.g., 2/3) receives 5 damage
   - Assert: Trigger fires; creature is in graveyard when trigger resolves; effect still happens (draws a card -- the creature leaving doesn't prevent the draw)

### Step 5: Card Definition

**Suggested card**: Ripjaw Raptor
- Simpler than Ranging Raptors (draw vs. search library)
- Oracle text: "Enrage -- Whenever this creature is dealt damage, draw a card."
- {2}{G}{G}, Creature -- Dinosaur, 4/5
- Green color identity

**File**: `crates/engine/src/cards/defs/ripjaw_raptor.rs`

**Translation to DSL**:
```rust
CardDefinition {
    name: "Ripjaw Raptor".to_string(),
    cost: ManaCost { generic: 2, green: 2, ..Default::default() },
    card_types: vec![CardType::Creature],
    sub_types: vec![SubType::Dinosaur],
    power: Some(4),
    toughness: Some(5),
    abilities: vec![
        AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDealtDamage,
            effect: Effect::DrawCards {
                player: EffectTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            optional: false,
        },
    ],
    ..Default::default()
}
```

### Step 6: Game Script

**Suggested scenario**: `test-data/generated-scripts/combat/174_enrage_combat_damage.json`
**Description**: P1 controls Ripjaw Raptor (4/5). P2 controls a 3/3 creature. P2 attacks with the 3/3. P1 blocks with Ripjaw Raptor. Combat damage resolves -- Ripjaw Raptor takes 3 damage, Enrage triggers, P1 draws a card. Raptor survives (3 damage on a 5-toughness creature).

**Subsystem directory**: `test-data/generated-scripts/combat/`
**Sequence number**: Check next available -- likely 174+

### Step 7: Coverage Doc Update

**File**: `docs/mtg-engine-ability-coverage.md`
**Action**: Update Enrage row from `none` to `validated` after tests pass.
**Status fields**: enum: n/a (ability word), enforcement: n/a, tests: yes, script: yes, card: Ripjaw Raptor

## Interactions to Watch

- **Wither/Infect damage to creatures**: Wither and Infect deal damage as -1/-1 counters (CR 702.80a, CR 702.90c). This IS still damage -- Enrage should trigger. The `DamageDealt` event is emitted even for wither/infect damage in `effects/mod.rs` (line 309), so this should work automatically.
- **Deathtouch + damage**: Deathtouch with 1 damage to an Enrage creature should trigger Enrage and kill the creature. The trigger still fires because damage > 0.
- **Prevention effects**: Only relevant if ALL damage is prevented. Partial prevention (e.g., 3 damage reduced to 1) still triggers Enrage because final_dmg > 0.
- **Combat damage and SBAs**: Combat damage is dealt, then SBAs are checked. Creatures that die from lethal damage will have their Enrage trigger already queued before they leave the battlefield. The trigger resolves after the creature is gone (ruling 2018-01-19).
- **No KeywordAbility match arms needed**: Since Enrage is not a `KeywordAbility` variant, there are no match arms to update in `types.rs`, `hash.rs` (keyword section), `view_model.rs`, `stack_view.rs`, or `builder.rs` keyword loops.

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/engine/src/cards/card_definition.rs` | Add `TriggerCondition::WhenDealtDamage` |
| `crates/engine/src/state/game_object.rs` | Add `TriggerEvent::SelfIsDealtDamage` |
| `crates/engine/src/state/hash.rs` | Add hash discriminants for both new variants |
| `crates/engine/src/testing/replay_harness.rs` | Add `enrich_spec_from_def` mapping for `WhenDealtDamage` -> `SelfIsDealtDamage` |
| `crates/engine/src/rules/abilities.rs` | Add `SelfIsDealtDamage` dispatch in `CombatDamageDealt` handler + new `DamageDealt` handler |
| `crates/engine/tests/enrage.rs` | New file: 5 unit tests |
| `crates/engine/src/cards/defs/ripjaw_raptor.rs` | New card definition (auto-discovered by build.rs) |

## Risk Assessment

**Low risk**. This ability:
- Adds no new `KeywordAbility` variant (no exhaustive match cascade)
- Uses existing trigger infrastructure (`collect_triggers_for_event`)
- Uses existing damage events (`CombatDamageDealt`, `DamageDealt`)
- Requires no new commands, effects, or state fields
- The only new infrastructure is the `DamageDealt` handler in `check_triggers`, which is also useful for future "when dealt damage" triggers beyond Enrage
