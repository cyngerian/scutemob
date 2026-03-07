# Ability Plan: Tribute

**Generated**: 2026-03-07
**CR**: 702.104
**Priority**: P4
**Similar abilities studied**: Bloodthirst (702.54, `resolution.rs:791-850`), Amplify (702.38, `resolution.rs:691-790`), Devour (702.82, `resolution.rs:852-1030`)

## CR Rule Text

> **702.104. Tribute**
>
> **702.104a** Tribute is a static ability that functions as the creature with tribute is entering the battlefield. "Tribute N" means "As this creature enters, choose an opponent. That player may put an additional N +1/+1 counters on it as it enters."
>
> **702.104b** Objects with tribute have triggered abilities that check "if tribute wasn't paid." This condition is true if the opponent chosen as a result of the tribute ability didn't have the creature enter the battlefield with +1/+1 counters as specified by the creature's tribute ability.

## Key Edge Cases

- **Opponent choice in multiplayer**: The controller chooses which opponent is given the tribute decision (CR 702.104a: "choose an opponent"). This is NOT the left-opponent or a fixed rule -- the controller picks.
- **No response window before ETB**: Players cannot respond to the tribute decision before the creature enters the battlefield (ruling 2014-02-01). The choice happens as part of ETB, not on the stack.
- **Counters enter with the creature**: If the opponent pays tribute, the creature enters WITH the counters already on it (ruling 2014-02-01). It does not enter and then get counters placed on it. This is a replacement-like static ability (CR 614.1c analog).
- **"Tribute wasn't paid" is an intervening-if condition**: The triggered ability checks the condition both when triggering and when resolving (CR 603.4). If tribute was paid, the trigger never fires.
- **Triggered ability resolves even if creature leaves**: The triggered ability resolves even if the creature with tribute is no longer on the battlefield (ruling 2014-02-01).
- **Target not known during casting**: If the triggered ability has a target, that target is not known while the creature spell is on the stack (ruling 2014-02-01). Opponents cannot respond to the tribute choice when deciding whether to counter.
- **Bot/deterministic play**: In the engine's current bot-play model, the opponent's choice is deterministic. The engine should auto-choose: opponent does NOT pay tribute (so the "tribute wasn't paid" trigger fires), since in most game scenarios the controller built the card to benefit from the non-payment. This is consistent with how Amplify auto-reveals all matching cards.
- **Counter attribution**: For effects that check which player put counters on the entering creature, the chosen opponent put those counters on it, not the controller (ruling 2017-04-18). Not critical for initial implementation but noted.

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
**Action**: Add `KeywordAbility::Tribute(u32)` variant after `Fortify` (line ~1201)
**Discriminant**: 131
**Pattern**: Follow `KeywordAbility::Bloodthirst(u32)` at line 1134

```
/// CR 702.104: Tribute N -- "As this creature enters, choose an opponent.
/// That player may put an additional N +1/+1 counters on it as it enters."
///
/// Static ability that functions at ETB time (CR 702.104a). The creature's
/// controller chooses an opponent, who may pay tribute (place N counters)
/// or decline (triggering the "tribute wasn't paid" ability).
///
/// Discriminant 131.
Tribute(u32),
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `KeywordAbility::Tribute(n)` after `Fortify` arm (~line 630)
**Pattern**: Follow `Bloodthirst(n)` hash pattern at line 620-624

```
// Tribute (discriminant 131) -- CR 702.104
KeywordAbility::Tribute(n) => {
    131u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `tribute_was_paid: bool` field to `GameObject` after `paired_with`
**Purpose**: Track whether tribute was paid for this permanent so the intervening-if trigger condition can check it. Reset to `false` on zone changes (CR 400.7).

```
/// CR 702.104b: Whether tribute was paid for this permanent. When true,
/// the chosen opponent placed N +1/+1 counters on it as it entered.
/// Used by "if tribute wasn't paid" intervening-if trigger conditions.
/// Reset to false on zone changes (CR 400.7).
#[serde(default)]
pub tribute_was_paid: bool,
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.tribute_was_paid.hash_into(hasher);` to `GameObject`'s `HashInto` impl

**Match arms to update** (grep for exhaustive `KeywordAbility` matches):
1. `tools/replay-viewer/src/view_model.rs` -- keyword display function (add `Tribute(n)` arm)
2. `tools/tui/src/play/panels/stack_view.rs` -- if SOK variant is added (Step 3)

**Initialization sites for `tribute_was_paid: false`**:
1. `crates/engine/src/state/mod.rs` -- both `move_object_to_zone` sites (~lines 365, 516)
2. `crates/engine/src/state/builder.rs` -- `GameStateBuilder` object construction (~line 963)
3. `crates/engine/src/rules/resolution.rs` -- all token-creation `GameObject` literals (grep `creatures_devoured: 0` to find all ~5 sites)

### Step 2: Rule Enforcement (ETB Counter Placement)

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add Tribute N handling block after the Bloodthirst block (~after line 850)
**CR**: 702.104a -- "As this creature enters, choose an opponent. That player may put an additional N +1/+1 counters on it as it enters."
**Pattern**: Follow Bloodthirst at lines 791-850

Implementation:
1. Collect all `Tribute(n)` instances from the card definition (same pattern as Bloodthirst/Amplify)
2. If any Tribute instances exist:
   a. Choose an opponent (in bot-play: pick the first non-eliminated opponent by turn order, or the opponent with highest life total -- keep it simple, use first valid opponent)
   b. Determine whether the opponent pays: **auto-decline** (opponent does NOT pay tribute). This is the deterministic bot behavior -- the engine does not yet have interactive opponent choices during ETB resolution. The tribute creature is designed so the controller benefits from non-payment.
   c. If tribute is NOT paid: set `tribute_was_paid = false` on the `GameObject` (default)
   d. If tribute IS paid: add N +1/+1 counters to the entering permanent, set `tribute_was_paid = true`, emit `CounterAdded` event

```rust
// CR 702.104a: Tribute N -- "As this creature enters, choose an opponent.
// That player may put an additional N +1/+1 counters on it as it enters."
// CR 702.104b: Objects with tribute have triggered abilities that check
// "if tribute wasn't paid."
//
// Implementation: Deterministic bot play -- opponent always declines tribute.
// The creature enters without extra counters, and the "tribute wasn't paid"
// triggered ability fires.
{
    let tribute_instances: Vec<u32> = card_id
        .as_ref()
        .and_then(|cid| registry.get(cid.clone()))
        .map(|def| {
            def.abilities
                .iter()
                .filter_map(|a| match a {
                    crate::cards::card_definition::AbilityDefinition::Keyword(
                        KeywordAbility::Tribute(n),
                    ) => Some(*n),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();

    if !tribute_instances.is_empty() {
        // Bot play: opponent does not pay tribute.
        // tribute_was_paid remains false (default).
        // The "if tribute wasn't paid" triggered ability will fire.
        //
        // Future: when interactive opponent choices are implemented,
        // this block should prompt the chosen opponent and conditionally
        // add counters + set tribute_was_paid = true.
    }
}
```

**File**: `crates/engine/src/rules/lands.rs`
**Action**: Add a parallel Tribute hook for consistency (lands never have Tribute, but follow the dual-site pattern from gotchas-infra.md)
**Pattern**: Follow Bloodthirst block in `lands.rs` at lines 299-340

### Step 3: Trigger Wiring

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `StackObjectKind::TributeTrigger` variant
**Discriminant**: SOK 50

```
/// CR 702.104b: Tribute "wasn't paid" triggered ability.
/// Fires when a creature with tribute enters and the chosen opponent
/// declined to place counters.
///
/// Discriminant 50.
TributeTrigger {
    /// The permanent that entered with tribute.
    tribute_permanent_id: ObjectId,
},
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `StackObjectKind::TributeTrigger`

**File**: `crates/engine/src/rules/abilities.rs` (or `resolution.rs` inline)
**Action**: After the Tribute enforcement block (Step 2) and the `PermanentEnteredBattlefield` event, check if `tribute_was_paid == false` on the entering permanent. If so, and the card definition has a triggered ability with condition `TriggerCondition::WhenEntersBattlefield` and an intervening-if that checks tribute, push a `TributeTrigger` to `pending_triggers`.

**Design decision**: The "if tribute wasn't paid" trigger is card-specific (each Tribute card has a different effect). Two approaches:
1. **Card-definition driven** (preferred): The card definition's `AbilityDefinition::Triggered` with `trigger_condition: WhenEntersBattlefield` carries the effect. The tribute check is an intervening-if condition. At ETB, check `tribute_was_paid == false` and fire the card's triggered effect.
2. **SOK-driven**: A `TributeTrigger` SOK variant that, when resolved, looks up the card definition's tribute-triggered effect.

**Recommended approach**: Use the existing `fire_when_enters_triggered_effects` mechanism in `replacement.rs`. Add a new `TriggerCondition::WhenEntersBattlefieldIfTributeNotPaid` variant (or use the existing `WhenEntersBattlefield` with an `InterveningIf::TributeNotPaid` check). The trigger fires inline at ETB time if `tribute_was_paid == false`.

Actually, the simpler approach (matching Devour/Champion pattern): use a `StackObjectKind::TributeTrigger` that goes on the stack. When it resolves, look up the card definition for the tribute creature and execute its tribute-triggered effect. This allows opponents to respond to the trigger.

**File**: `crates/engine/src/state/types.rs`
**Action**: No new `AbilityDefinition` variant needed. The trigger effect is carried inline in the card definition's existing `AbilityDefinition::Triggered` with `trigger_condition: WhenEntersBattlefield`. The `TributeTrigger` SOK variant's resolution looks it up from the card def.

**Alternatively (simpler, no new SOK)**: Use the existing `fire_when_enters_triggered_effects` path. Add an `InterveningIf` check: only fire the card's `WhenEntersBattlefield` triggered ability if `tribute_was_paid == false` on the entering permanent. This avoids a new SOK variant entirely.

**Final recommendation**: Add a new `TriggerCondition` variant or `InterveningIf` variant. The cleanest is:

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `TriggerCondition::WhenEntersBattlefieldTributeNotPaid` variant (no new AbilDef discriminant needed -- this is a TriggerCondition enum value within `AbilityDefinition::Triggered`). Or simpler: use existing `WhenEntersBattlefield` and add an `intervening_if` field.

**Simplest viable approach** (recommended by the runner):
1. Keep `TriggerCondition::WhenEntersBattlefield` as-is
2. In `fire_when_enters_triggered_effects`, after matching `WhenEntersBattlefield`, check if the entering permanent has `tribute_was_paid == false` AND the card has a `KeywordAbility::Tribute(_)`. If both true, fire the triggered effect. If tribute was paid, skip.
3. No new SOK variant, no new TriggerCondition variant. The card definition uses `AbilityDefinition::Triggered { trigger_condition: WhenEntersBattlefield, effect: <tribute-not-paid effect> }`.
4. The `Tribute(n)` keyword on the card tells the engine this is a tribute creature. The `WhenEntersBattlefield` triggered ability is the tribute-not-paid effect.

**HOWEVER**: This conflates all `WhenEntersBattlefield` triggers with the tribute condition. A card with BOTH a normal ETB trigger AND a tribute-not-paid trigger would break. None of the printed Tribute cards have this issue, but it's fragile.

**Best approach**: Use a dedicated `TriggerCondition::TributeNotPaid` variant.

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `TriggerCondition` enum:
```
/// CR 702.104b: "When ~ enters, if tribute wasn't paid, ..."
TributeNotPaid,
```

**File**: `crates/engine/src/rules/replacement.rs` (in `fire_when_enters_triggered_effects`)
**Action**: Add a match arm for `TriggerCondition::TributeNotPaid`:
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::TributeNotPaid,
    effect,
    ..
} => {
    // CR 702.104b: Only fire if tribute was not paid.
    let tribute_not_paid = state
        .objects
        .get(&new_id)
        .map(|o| !o.tribute_was_paid)
        .unwrap_or(false);
    if tribute_not_paid {
        let ctx = EffectContext { source: new_id, controller, .. };
        let (new_state, new_events) = execute_effect(state, effect, &ctx);
        *state = new_state;
        evts.extend(new_events);
    }
}
```

**No new SOK variant needed** (SOK 50 reserved but not used). The trigger fires inline via `fire_when_enters_triggered_effects`, same as other ETB triggered effects.

**No new AbilityDefinition discriminant needed** -- `TriggerCondition` is a field within `AbilityDefinition::Triggered`, not a top-level variant.

### Step 4: Unit Tests

**File**: `crates/engine/tests/tribute.rs`
**Tests to write**:

1. **`test_tribute_basic_not_paid`** -- CR 702.104a/b
   - Creature with Tribute N enters the battlefield
   - Bot opponent declines tribute (default behavior)
   - Creature enters without extra +1/+1 counters
   - The "tribute wasn't paid" triggered effect fires
   - Verify: creature at base P/T, triggered effect applied

2. **`test_tribute_paid_counters_placed`** -- CR 702.104a
   - Set `tribute_was_paid = true` manually on the entering creature (simulating opponent paying)
   - Creature enters with N +1/+1 counters
   - The "tribute wasn't paid" trigger does NOT fire
   - Verify: creature has N extra +1/+1 counters, no triggered effect

3. **`test_tribute_n_value`** -- CR 702.104a
   - Tribute 6 creature (Nessian Wilds Ravager): when not paid, enters at base 6/6
   - Tribute 1 creature (Fanatic of Xenagos): when not paid, enters at base 3/3 and trigger fires

4. **`test_tribute_multiplayer_opponent_choice`** -- CR 702.104a
   - 4-player game, controller enters tribute creature
   - The "choose an opponent" is deterministic (first valid opponent)
   - Verify the opponent choice is made (opponent is not the controller, not eliminated)

5. **`test_tribute_trigger_resolves_after_creature_leaves`** -- Ruling 2014-02-01
   - Tribute creature enters, trigger goes on stack
   - Remove the creature before trigger resolves
   - Trigger still resolves (the effect may fizzle if it targets the creature, but the trigger itself fires)

6. **`test_tribute_keyword_on_card`** -- Basic enum/type test
   - Card definition with `KeywordAbility::Tribute(3)` compiles and is present in card's abilities

**Pattern**: Follow tests in `crates/engine/tests/bloodthirst.rs` (setup, cast spell, verify counters/effects)

### Step 5: Card Definition (later phase)

**Suggested card**: Fanatic of Xenagos
- **Mana cost**: {1}{R}{G}
- **Type**: Creature -- Centaur Warrior 3/3
- **Abilities**: Trample, Tribute 1
- **Tribute-not-paid effect**: Gets +1/+1 and gains haste until end of turn
- **Rationale**: Simple card, no targeting required, tests both counter-placement (if paid) and buff effect (if not paid). Trample is already implemented.

**Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: Fanatic of Xenagos enters the battlefield. Opponent declines tribute. Fanatic gets +1/+1 and haste until end of turn, becoming a 4/4 trampler with haste. Attacks for 4 damage.

**Subsystem directory**: `test-data/generated-scripts/baseline/`

## Interactions to Watch

- **Enters-the-battlefield replacement effects** (CR 614.1c): Tribute is a static ability that functions "as the creature enters." It interacts with other ETB replacements (enters tapped, etc.) -- the controller chooses the order of self-replacement effects (CR 614.15).
- **Panharmonicon / Doubling Season**: Panharmonicon doubles ETB triggered abilities. The "if tribute wasn't paid" trigger IS a triggered ability and SHOULD be doubled by Panharmonicon. However, note gotcha: `SelfEntersBattlefield` triggers are NOT doubled by the current `doubler_applies_to_trigger` implementation (only matches `AnyPermanentEntersBattlefield`). This is a known limitation.
- **Humility / Dress Down**: If the Tribute keyword is removed before ETB processing (Layer 6), the tribute static ability should not function. However, since Tribute functions "as the creature enters," and layers are recalculated after ETB, this is a timing edge case. The engine should check for Tribute from the card definition (like Bloodthirst/Amplify), not from layer-resolved characteristics, since the ability functions during ETB before layers are fully established.
- **Multiplayer implications**: Controller chooses which opponent to offer tribute to. This is strategic -- in a 4-player game, the controller might offer tribute to the opponent least likely to pay. Current bot play auto-declines, making the choice moot.

## Discriminant Assignments

| Type | Name | Discriminant |
|------|------|-------------|
| KeywordAbility | `Tribute(u32)` | 131 |
| StackObjectKind | (not used -- trigger fires inline) | -- |
| AbilityDefinition | (not used -- uses existing `Triggered` variant) | -- |
| TriggerCondition | `TributeNotPaid` | (enum variant, no explicit discriminant tracking) |
| GameObject field | `tribute_was_paid: bool` | (not an enum) |
