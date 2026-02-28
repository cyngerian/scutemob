# Ability Plan: Regenerate

**Generated**: 2026-02-28
**CR**: 701.19
**Priority**: P3
**Similar abilities studied**: Prevention effects (`PreventDamage`/`PreventAllDamage` in `replacement.rs`), Indestructible check in `effects/mod.rs:521-533`, `Effect::Goad` as keyword action pattern

## CR Rule Text

### 701.19. Regenerate

> **701.19a** If the effect of a resolving spell or ability regenerates a permanent, it creates a replacement effect that protects the permanent the next time it would be destroyed this turn. In this case, "Regenerate [permanent]" means "The next time [permanent] would be destroyed this turn, instead remove all damage marked on it and its controller taps it. If it's an attacking or blocking creature, remove it from combat."
>
> **701.19b** If the effect of a static ability regenerates a permanent, it replaces destruction with an alternate effect each time that permanent would be destroyed. In this case, "Regenerate [permanent]" means "Instead remove all damage marked on [permanent] and its controller taps it. If it's an attacking or blocking creature, remove it from combat."
>
> **701.19c** Neither activating an ability that creates a regeneration shield nor casting a spell that creates a regeneration shield is the same as regenerating a permanent. Effects that say that a permanent can't be regenerated don't preclude such abilities from being activated or such spells from being cast; rather, they cause regeneration shields to not be applied.

### Related Rules

> **614.8** Regeneration is a destruction-replacement effect. The word "instead" doesn't appear on the card but is implicit in the definition of regeneration. "Regenerate [permanent]" means "The next time [permanent] would be destroyed this turn, instead remove all damage marked on it and its controller taps it. If it's an attacking or blocking creature, remove it from combat." Abilities that trigger from damage being dealt still trigger even if the permanent regenerates. See rule 701.19.
>
> **701.8c** A regeneration effect replaces a destruction event. See rule 701.19, "Regenerate."
>
> **120.6** [...] All damage marked on a permanent is removed when it regenerates (see rule 701.19, "Regenerate") and during the cleanup step (see rule 514.2).

## Key Edge Cases

1. **One-shot shield (CR 701.19a)**: A resolving spell/ability creates a regeneration shield that lasts until it intercepts one destruction OR until end of turn, whichever comes first. This is analogous to the existing `PreventDamage(n)` one-shot pattern.
2. **Multiple shields stack (CR 701.19a)**: Activating regeneration multiple times creates multiple independent shields. After one intercepts destruction, the others remain until used or end of turn.
3. **"Can't be regenerated" (CR 701.19c)**: Some effects say "destroy target creature. It can't be regenerated." (e.g., Wrath of God). The shield still exists but is not applied. The permanent is destroyed despite having a regeneration shield.
4. **Combat removal**: When regeneration replaces destruction during combat, the creature is removed from combat (removed from `combat.attackers` or `combat.blockers`). The creature remains on the battlefield, tapped.
5. **Damage triggers still fire (CR 614.8)**: Abilities that trigger from damage being dealt still trigger even if the permanent regenerates. Regeneration does NOT prevent the damage; it replaces the subsequent destruction.
6. **Not the same as "being regenerated" (CR 701.19c)**: Activating "{B}: Regenerate ~" is not regenerating the creature -- it creates a shield. The creature is "regenerated" only when the shield actually intercepts destruction.
7. **Static regeneration (CR 701.19b)**: A static ability that says "Regenerate [permanent]" replaces destruction every time, not just once. This is a continuous replacement, not a one-shot shield. (Uncommon pattern -- most regeneration is activated.)
8. **Indestructible interaction**: Indestructible prevents destruction outright (CR 702.12a). If a creature has both indestructible and a regeneration shield, indestructible applies first and the shield is not consumed.
9. **704.5f (zero toughness)**: A creature with toughness 0 or less is put into its owner's graveyard -- this is NOT destruction. Regeneration does NOT prevent this. The SBA for 704.5f is distinct from 704.5g/h.
10. **Multiplayer**: No special multiplayer considerations; regeneration works identically in all player counts.

## Current State (from ability-wip.md)

- [ ] Step 1: Keyword action / Effect variant
- [ ] Step 2: Rule enforcement (replacement effect registration + interception)
- [ ] Step 3: Trigger wiring (N/A for the shield itself)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Design Decision: Architecture

Regenerate is fundamentally different from existing replacement effects in the engine. Current zone-change replacements (`WouldChangeZone`) redirect to a different zone. Regeneration does NOT redirect -- it **prevents the zone change entirely** and performs side effects instead (remove damage, tap, remove from combat).

### Option chosen: New `ReplacementTrigger::WouldBeDestroyed` + `ReplacementModification::Regenerate`

The design adds:
1. A new `ReplacementTrigger::WouldBeDestroyed` variant that matches destruction events specifically (not all zone changes).
2. A new `ReplacementModification::Regenerate` variant that tells the interception site to perform the regeneration replacement (remove damage, tap, remove from combat) instead of destroying.
3. The `Effect::Regenerate { target }` variant registers a one-shot `ReplacementEffect` with `UntilEndOfTurn` duration and the target permanent as a `SpecificObject` filter.
4. Two interception sites need to check for regeneration shields: (a) SBA `check_creature_sbas` for lethal damage / deathtouch destruction (704.5g/h), and (b) `Effect::DestroyPermanent` for spell/ability-based destruction.

### Why not reuse `WouldChangeZone`?

`WouldChangeZone` from Battlefield to Graveyard would match, but the modification behavior is fundamentally different -- it does NOT redirect to another zone. It keeps the object on the battlefield and performs side effects. Adding a "stay in place" variant to `ZoneChangeAction` would be confusing since the current pipeline assumes a zone change always happens. A dedicated trigger type is cleaner.

## Implementation Steps

### Step 1: Effect Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Effect::Regenerate { target: EffectTarget }` variant to the `Effect` enum.
**Location**: After `Effect::Goad` (around line 479), in the "Permanents" section.
**CR**: 701.19a -- "If the effect of a resolving spell or ability regenerates a permanent, it creates a replacement effect..."

```rust
/// CR 701.19a: Regenerate -- create a one-shot regeneration shield on the target
/// permanent. The next time that permanent would be destroyed this turn, instead
/// remove all damage marked on it, tap it, and remove it from combat (if in combat).
/// The shield lasts until used or until end of turn (cleanup step).
Regenerate { target: EffectTarget },
```

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` in the `Effect` `HashInto` impl.
Use **discriminant 37** (next available after Investigate=36).

```rust
// CR 701.19a: Regenerate (discriminant 37)
Effect::Regenerate { target } => {
    37u8.hash_into(hasher);
    target.hash_into(hasher);
}
```

### Step 2a: Replacement Effect Types

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/replacement_effect.rs`

**Action 1**: Add `ReplacementTrigger::WouldBeDestroyed` variant.
**Location**: After `DamageWouldBeDealt` (around line 46).

```rust
/// A permanent would be destroyed (CR 701.8, 614.8).
/// Matches both SBA-based destruction (704.5g/h) and effect-based destruction
/// (Effect::DestroyPermanent). Does NOT match 704.5f (zero toughness -- not destruction).
WouldBeDestroyed { filter: ObjectFilter },
```

**Action 2**: Add `ReplacementModification::Regenerate` variant.
**Location**: After `ShuffleIntoOwnerLibrary` (around line 73).

```rust
/// CR 701.19a/614.8: Regeneration -- instead of being destroyed, remove all damage
/// marked on the permanent, tap it, and remove it from combat (if attacking/blocking).
/// One-shot: consumed after one use (tracked via one-shot removal, not prevention_counters).
Regenerate,
```

**Hash updates** in `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`:

For `ReplacementTrigger::WouldBeDestroyed`:
```rust
ReplacementTrigger::WouldBeDestroyed { filter } => {
    5u8.hash_into(hasher); // next after DamageWouldBeDealt=4
    filter.hash_into(hasher);
}
```

For `ReplacementModification::Regenerate`:
```rust
ReplacementModification::Regenerate => 7u8.hash_into(hasher), // next after ShuffleIntoOwnerLibrary=6
```

### Step 2b: Trigger Matching in replacement.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/replacement.rs`

**Action**: Add a match arm to `trigger_matches()` (around line 248) for `WouldBeDestroyed`.

```rust
(
    ReplacementTrigger::WouldBeDestroyed { filter: eff_filter },
    ReplacementTrigger::WouldBeDestroyed { filter: evt_filter },
) => event_object_matches_filter(state, evt_filter, eff_filter),
```

### Step 2c: Effect Execution -- Register the Shield

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`

**Action**: Add handler for `Effect::Regenerate { target }` in `execute_effect()`.
**Location**: After the `Effect::Goad` handler (around line 1440).
**Pattern**: Follow `Effect::Goad` -- resolve target, then modify state.

```rust
// CR 701.19a: Regenerate -- create a one-shot regeneration shield on the
// target permanent. The shield is a UntilEndOfTurn replacement effect that
// intercepts the next WouldBeDestroyed event for this specific permanent.
Effect::Regenerate { target } => {
    let targets = resolve_effect_target_list(state, target, ctx);
    for resolved in targets {
        if let ResolvedTarget::Object(id) = resolved {
            // Verify the target is on the battlefield
            let on_battlefield = state
                .objects
                .get(&id)
                .map(|o| o.zone == ZoneId::Battlefield)
                .unwrap_or(false);
            if !on_battlefield {
                continue;
            }

            let regen_id = state.next_replacement_id();
            state.replacement_effects.push_back(ReplacementEffect {
                id: regen_id,
                source: Some(id), // The permanent being protected
                controller: ctx.controller,
                duration: EffectDuration::UntilEndOfTurn,
                is_self_replacement: true, // CR 614.15: self-replacement
                trigger: ReplacementTrigger::WouldBeDestroyed {
                    filter: ObjectFilter::SpecificObject(id),
                },
                modification: ReplacementModification::Regenerate,
            });

            events.push(GameEvent::RegenerationShieldCreated {
                object_id: id,
                shield_id: regen_id,
                controller: ctx.controller,
            });
        }
    }
}
```

### Step 2d: Interception Site 1 -- SBA Destruction (704.5g/h)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/sba.rs`

**Action**: Before adding a creature to the `dying` list in `check_creature_sbas()` for 704.5g/h cases (NOT 704.5f), check for regeneration shields.

The current code at lines 372-385 checks for lethal damage and deathtouch damage. When either condition is true AND the creature is NOT indestructible, the creature is marked for destruction. We need to intercept BEFORE moving to graveyard and check for regeneration shields.

**Modification point**: Inside the `for id in dying` loop (around line 392), before calling `check_zone_change_replacement`, check for `WouldBeDestroyed` replacement effects. If a regeneration shield is found, apply it instead of destroying.

The specific logic:
1. Determine if the creature is dying due to 704.5f (zero toughness) vs 704.5g/h (lethal damage / deathtouch). Only 704.5g/h are "destruction" and can be regenerated.
2. For 704.5g/h creatures, check `find_applicable()` with `WouldBeDestroyed { filter: SpecificObject(id) }`.
3. If a regeneration shield matches, apply it: remove all damage, tap the permanent, remove from combat, consume (remove) the one-shot replacement effect, emit `Regenerated` event. Do NOT move to graveyard.
4. If no regeneration shield matches, proceed with existing destruction logic.

**Key change**: The `dying` vec collection needs to be split into two categories:
- `dying_zero_toughness`: creatures dying from 704.5f (cannot be regenerated)
- `dying_destruction`: creatures dying from 704.5g/h (can be regenerated)

Or, more simply, add a boolean flag alongside each dying ObjectId indicating whether the cause is destruction (regenerable) vs zero toughness (not regenerable).

**Suggested approach**: Change `dying: Vec<ObjectId>` to `dying: Vec<(ObjectId, bool)>` where the bool is `is_destruction` (true for 704.5g/h, false for 704.5f). Then in the loop, only check regeneration for `is_destruction == true`.

**Helper function** (add to `replacement.rs`):

```rust
/// CR 701.19a/614.8: Check if a regeneration shield can replace destruction.
///
/// Returns `Some(shield_id)` if a regeneration shield exists for this permanent,
/// or `None` if no shield applies.
pub fn check_regeneration_shield(
    state: &GameState,
    object_id: ObjectId,
) -> Option<ReplacementId> {
    let trigger = ReplacementTrigger::WouldBeDestroyed {
        filter: ObjectFilter::SpecificObject(object_id),
    };
    let applicable = find_applicable(state, &trigger, &std::collections::HashSet::new());
    // Find the first applicable regeneration modification
    applicable.into_iter().find(|id| {
        state
            .replacement_effects
            .iter()
            .any(|e| e.id == *id && e.modification == ReplacementModification::Regenerate)
    })
}

/// CR 701.19a: Apply a regeneration shield to a permanent that would be destroyed.
///
/// Performs the regeneration replacement:
/// 1. Remove all damage marked on the permanent (CR 701.19a).
/// 2. Tap the permanent (CR 701.19a).
/// 3. If it's an attacking or blocking creature, remove it from combat (CR 701.19a).
/// 4. Remove the one-shot regeneration shield (consumed).
///
/// Returns the events to emit.
pub fn apply_regeneration(
    state: &mut GameState,
    object_id: ObjectId,
    shield_id: ReplacementId,
) -> Vec<GameEvent> {
    let mut events = Vec::new();

    // 1. Remove all damage
    if let Some(obj) = state.objects.get_mut(&object_id) {
        obj.damage_marked = 0;
        obj.deathtouch_damage = false;
    }

    // 2. Tap the permanent
    if let Some(obj) = state.objects.get_mut(&object_id) {
        obj.status.tapped = true;
    }

    // 3. Remove from combat (if attacking or blocking)
    if let Some(combat) = &mut state.combat {
        combat.attackers.remove(&object_id);
        combat.blockers.remove(&object_id);
        // Also remove from damage_assignment_order
        combat.damage_assignment_order.remove(&object_id);
        // Remove as a blocker from all damage assignment orders
        for (_attacker_id, order) in combat.damage_assignment_order.iter_mut() {
            order.retain(|&blocker| blocker != object_id);
        }
    }

    // 4. Remove the one-shot shield (consumed)
    let keep: im::Vector<_> = state
        .replacement_effects
        .iter()
        .filter(|e| e.id != shield_id)
        .cloned()
        .collect();
    state.replacement_effects = keep;

    events.push(GameEvent::Regenerated {
        object_id,
        shield_id,
    });

    events
}
```

### Step 2e: Interception Site 2 -- Effect-based Destruction

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`

**Action**: In the `Effect::DestroyPermanent` handler (around line 517), before the indestructible check AND before `check_zone_change_replacement`, check for regeneration shields.

The check should go AFTER the indestructible check (if indestructible, skip entirely -- regeneration is not needed). Then check for regeneration shields BEFORE the zone-change replacement check.

```rust
// CR 701.19a/614.8: Check regeneration shields before destruction.
if let Some(shield_id) = crate::rules::replacement::check_regeneration_shield(state, id) {
    let regen_events = crate::rules::replacement::apply_regeneration(state, id, shield_id);
    events.extend(regen_events);
    continue; // Skip destruction -- permanent stays on battlefield
}
```

### Step 2f: GameEvent Variants

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`

**Action**: Add two new event variants.

```rust
/// A regeneration shield was created on a permanent (CR 701.19a).
///
/// Emitted when `Effect::Regenerate` resolves, creating a one-shot
/// replacement effect that will intercept the next destruction event.
RegenerationShieldCreated {
    /// The permanent the shield protects.
    object_id: ObjectId,
    /// The ReplacementId of the shield (for tracking consumption).
    shield_id: ReplacementId,
    /// The controller who created the shield.
    controller: PlayerId,
},

/// A permanent was regenerated -- destruction was replaced (CR 701.19a/614.8).
///
/// Emitted when a regeneration shield intercepts destruction. The permanent
/// remains on the battlefield with all damage removed, tapped, and removed
/// from combat (if applicable).
Regenerated {
    /// The permanent that was regenerated (still on the battlefield).
    object_id: ObjectId,
    /// The shield that was consumed.
    shield_id: ReplacementId,
},
```

**Hash**: Add to `GameEvent` `HashInto` impl in `hash.rs`:

```rust
// CR 701.19a: RegenerationShieldCreated (discriminant 83)
GameEvent::RegenerationShieldCreated { object_id, shield_id, controller } => {
    83u8.hash_into(hasher);
    object_id.hash_into(hasher);
    shield_id.hash_into(hasher);
    controller.hash_into(hasher);
}
// CR 701.19a/614.8: Regenerated (discriminant 84)
GameEvent::Regenerated { object_id, shield_id } => {
    84u8.hash_into(hasher);
    object_id.hash_into(hasher);
    shield_id.hash_into(hasher);
}
```

### Step 3: Trigger Wiring

**N/A for the shield itself.** Regeneration shields are replacement effects, not triggers.

However, note that `Effect::Regenerate` can be the effect of a triggered or activated ability (e.g., "{B}: Regenerate this creature" is an activated ability whose effect is `Effect::Regenerate { target: EffectTarget::Source }`). The trigger/activation wiring is already handled by the existing `AbilityDefinition::Activated` and `AbilityDefinition::Triggered` infrastructure.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/regenerate.rs`
**Pattern**: Follow tests in `crates/engine/tests/prevention.rs` and `crates/engine/tests/indestructible.rs` (or similar SBA-interaction test files).

**Tests to write**:

1. **`test_regenerate_shield_prevents_destruction_by_spell`**
   - CR 701.19a, 614.8
   - Setup: Creature on battlefield with a regeneration shield (via activated ability). Cast Doom Blade targeting the creature.
   - Assert: Creature stays on battlefield. Damage removed. Creature is tapped. `Regenerated` event emitted. No `CreatureDied` event.

2. **`test_regenerate_shield_prevents_sba_lethal_damage`**
   - CR 701.19a, 704.5g
   - Setup: 2/2 creature with regeneration shield. Deal 3 damage (marked on creature). Run SBAs.
   - Assert: Creature stays on battlefield. `damage_marked` reset to 0. Creature is tapped. `Regenerated` event emitted.

3. **`test_regenerate_shield_prevents_sba_deathtouch_damage`**
   - CR 701.19a, 704.5h
   - Setup: 4/4 creature with regeneration shield. Deal 1 deathtouch damage. Run SBAs.
   - Assert: Creature stays on battlefield. `damage_marked` reset to 0. `deathtouch_damage` reset to false. Creature is tapped.

4. **`test_regenerate_shield_is_one_shot`**
   - CR 701.19a
   - Setup: Creature with one regeneration shield. First destruction is replaced. Then deal lethal damage again.
   - Assert: First destruction: creature survives. Second destruction: creature dies (no shield remaining).

5. **`test_regenerate_multiple_shields`**
   - CR 701.19a
   - Setup: Creature with two regeneration shields. Two sequential destruction events.
   - Assert: First destruction: creature survives, one shield consumed. Second destruction: creature survives, second shield consumed. Third destruction would kill it.

6. **`test_regenerate_removes_from_combat_attacker`**
   - CR 701.19a
   - Setup: Creature declared as attacker. During combat, creature would be destroyed. Regeneration shield intercepts.
   - Assert: Creature removed from `combat.attackers`. Creature tapped. Creature still on battlefield.

7. **`test_regenerate_removes_from_combat_blocker`**
   - CR 701.19a
   - Setup: Creature declared as blocker. During combat, creature would be destroyed. Regeneration shield intercepts.
   - Assert: Creature removed from `combat.blockers`. Creature tapped. Creature still on battlefield.

8. **`test_regenerate_does_not_prevent_zero_toughness`**
   - CR 704.5f, 701.19a
   - Setup: Creature with toughness reduced to 0 (e.g., by a -X/-X effect). Creature has regeneration shield.
   - Assert: Creature dies via 704.5f (not destruction). Regeneration shield is NOT consumed.

9. **`test_regenerate_shield_expires_at_end_of_turn`**
   - CR 701.19a, 514.2
   - Setup: Create regeneration shield. Advance to cleanup step.
   - Assert: Shield removed from `state.replacement_effects`. (Relies on existing `expire_end_of_turn_effects` in `layers.rs`.)

10. **`test_regenerate_not_applied_when_indestructible`**
    - CR 702.12a, 701.19a
    - Setup: Indestructible creature with regeneration shield. Cast destroy spell.
    - Assert: Creature stays on battlefield (indestructible prevents destruction). Regeneration shield is NOT consumed (indestructible handled first).

### Step 5: Card Definition

**Suggested card**: Drudge Skeletons
- **Oracle text**: "{B}: Regenerate this creature."
- **Type**: Creature -- Skeleton, 1/1, {1}{B}
- **Rationale**: Simplest regeneration card. Single activated ability with mana cost, effect is `Regenerate { target: Source }`.

**Card definition structure**:
```rust
CardDefinition {
    card_id: cid("drudge-skeletons"),
    name: "Drudge Skeletons".to_string(),
    mana_cost: Some(ManaCost { black: 1, generic: 1, ..Default::default() }),
    types: creature_types(&["Skeleton"]),
    oracle_text: "{B}: Regenerate this creature.".to_string(),
    power: Some(1),
    toughness: Some(1),
    abilities: vec![
        AbilityDefinition::Activated {
            cost: Cost::Mana(ManaCost { black: 1, ..Default::default() }),
            effect: Effect::Regenerate { target: EffectTarget::Source },
            timing_restriction: None, // Can be activated any time
        },
    ],
    ..Default::default()
}
```

**Alternative card**: Sedge Troll ({2}{R}, Creature -- Troll, 2/2; gets +1/+1 with Swamp; {B}: Regenerate ~). More complex but tests color identity interaction.

### Step 6: Game Script

**Suggested scenario**: "Drudge Skeletons survives Doom Blade via regeneration"
**Subsystem directory**: `test-data/generated-scripts/replacement/`

**Script outline**:
1. Player 1 has Drudge Skeletons on the battlefield.
2. Player 1 activates "{B}: Regenerate this creature" (pay B).
3. Player 2 casts Doom Blade targeting Drudge Skeletons.
4. Doom Blade resolves -- regeneration shield intercepts destruction.
5. Assert: Drudge Skeletons still on battlefield, tapped, damage_marked = 0.

## Interactions to Watch

### Regeneration vs Zone-Change Replacements
If both a regeneration shield AND a zone-change replacement (e.g., Rest in Peace "exile instead") apply to the same destruction event, regeneration should apply first as a self-replacement (CR 614.15) since it's on the permanent itself. If regeneration succeeds, no zone change happens, so the zone-change replacement never fires.

### Regeneration vs "Can't be regenerated"
The engine does not yet have a "can't be regenerated" mechanism. Cards like Wrath of God say "They can't be regenerated." This requires a per-destruction-event flag that suppresses regeneration shield checking. This is a future enhancement -- for now, the Wrath of God card definition already uses `Effect::DestroyPermanent` without the flag. A `cant_regenerate: bool` field on `Effect::DestroyPermanent` or a separate mechanism should be planned but is NOT part of this implementation.

### Regeneration during SBA batch processing
SBAs are checked as a batch (CLAUDE.md invariant). Multiple creatures dying simultaneously each get their own regeneration check. A single creature with multiple shields only consumes one per destruction event.

### im-rs OrdMap mutation for combat state
The `combat.damage_assignment_order` uses `OrdMap<ObjectId, Vec<ObjectId>>`. When removing a regenerated blocker from damage assignment orders, iterate over all entries and filter. Since `im-rs` `OrdMap` has no `iter_mut`, collect and rebuild.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/cards/card_definition.rs` | Add `Effect::Regenerate { target }` variant |
| `crates/engine/src/state/replacement_effect.rs` | Add `ReplacementTrigger::WouldBeDestroyed`, `ReplacementModification::Regenerate` |
| `crates/engine/src/state/hash.rs` | Hash arms for new Effect, ReplacementTrigger, ReplacementModification, GameEvent variants |
| `crates/engine/src/rules/replacement.rs` | `trigger_matches()` arm for `WouldBeDestroyed`; `check_regeneration_shield()` + `apply_regeneration()` helpers |
| `crates/engine/src/rules/sba.rs` | Check regeneration shields before destroying creatures (704.5g/h only, not 704.5f) |
| `crates/engine/src/effects/mod.rs` | `Effect::Regenerate` execution handler; regeneration check in `Effect::DestroyPermanent` handler |
| `crates/engine/src/rules/events.rs` | `GameEvent::RegenerationShieldCreated`, `GameEvent::Regenerated` variants |
| `crates/engine/tests/regenerate.rs` | 10 unit tests |
| `crates/engine/src/cards/definitions.rs` | Drudge Skeletons card definition (Step 5) |
