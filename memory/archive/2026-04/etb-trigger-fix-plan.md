# ETB Trigger Engine Correctness Fix Plan

**Generated**: 2026-03-08
**CR**: 603.3, 603.6a, 603.6d, 117.3b
**Severity**: HIGH (correctness bug affecting 20 card definitions)
**Related issues**: None previously filed; discovered via code analysis

## The Bug

Card-definition ETB triggers (`AbilityDefinition::Triggered { trigger_condition: WhenEntersBattlefield }`) execute **inline** during `resolve_top_of_stack()` via `fire_when_enters_triggered_effects()` in `replacement.rs`. Per CR 603.3, they should be **queued as pending triggers** and placed on the stack when a player next receives priority, allowing opponents to respond (e.g., with Stifle).

### What is correct (no change needed)

- **Keyword-derived ETB triggers** (LivingWeapon, Modular ETB counters, Ward, Prowess, etc.) are built by `builder.rs` into `characteristics.triggered_abilities` on the object. These are correctly dispatched by `collect_triggers_for_event()` in `abilities.rs`, which matches `PermanentEnteredBattlefield` events against `SelfEntersBattlefield`/`AnyPermanentEntersBattlefield` trigger events. They go through `PendingTrigger` -> `flush_pending_triggers` -> stack. **Correct per CR 603.3.**

- **Keyword ETB effects that are NOT triggers** (Fabricate counters/tokens, Devour sacrifices, Backup +1/+1 counters, Modular ETB counters, Ravenous draw, Squad/Offspring/Gift) are handled by dedicated `PendingTriggerKind` variants (e.g., `Backup`, `SquadETB`, `GiftETB`, `RavenousDraw`). These go on the stack as their own SOK types. **Correct per CR 603.3.** (Note: Fabricate is inline as a bot approximation -- documented as TODO.)

- **Keyword ETB self-effects that are genuinely inline** (Exploit's "you may sacrifice", PartnerWith's "search", Hideaway's "look at top N") go through their own `PendingTriggerKind` variants. **Correct.**

- **Static "enters with/as" effects** (CR 603.6d: enters tapped, enters with counters) are replacement effects, not triggers. They correctly execute inline via `apply_self_etb_from_definition` and `apply_etb_replacements`. **Correct per CR 603.6d.**

### What is broken

`fire_when_enters_triggered_effects()` in `replacement.rs` (lines 928-1091) iterates the CardDef's `abilities` vec looking for `AbilityDefinition::Triggered { trigger_condition: WhenEntersBattlefield }` and calls `execute_effect()` inline. This bypasses the stack entirely.

**Affected card definitions** (20 cards):
- Avalanche Riders, Aven Riftwatcher, Blind Hunter, Crimestopper Sprite, Frantic Scapegoat
- Geological Appraiser, Kitchen Finks, Mulldrifter, Overlord of the Hauntwoods, Ox of Agonas
- Prosperous Innkeeper, Raffine's Informant, Rest in Peace, Sky Hussar, Solemn Simulacrum
- Thraben Inspector, Torch Slinger, Vivisection Evangelist, Voldaren Epicure, Wall of Omens

Also broken: `TriggerCondition::TributeNotPaid` (1 card: Fanatic of Xenagos) -- same inline pattern.

### CR Rules

**CR 603.3**: "Once an ability has triggered, its controller puts it on the stack as an object that's not a card the next time a player would receive priority."

**CR 603.6a**: "Enters-the-battlefield abilities trigger when a permanent enters the battlefield. These are written, 'When [this object] enters, ...' Each time an event puts one or more permanents onto the battlefield, all permanents on the battlefield (including the newcomers) are checked for any enters-the-battlefield triggers that match the event."

**CR 603.6d**: "[This permanent] enters with/as ... is a static ability -- not a triggered ability -- whose effect occurs as part of the event that puts the permanent onto the battlefield." (This is what distinguishes inline-correct vs stack-required.)

**CR 117.3b**: "The active player receives priority after a spell or ability (other than a mana ability) resolves."

## Architecture Analysis

### Current call chain (broken)

```
resolve_top_of_stack()
  -> [at each ETB site] fire_when_enters_triggered_effects(state, new_id, ...)
     -> iterates CardDef.abilities for WhenEntersBattlefield
     -> execute_effect() INLINE  <-- BUG: bypasses stack
  -> [at bottom, line 7057] check_triggers(state, &events)
     -> collect_triggers_for_event() matches SelfEntersBattlefield against
        characteristics.triggered_abilities (keyword-derived only)
     -> queues PendingTrigger
  -> [line 7065] check_and_apply_sbas()
  -> [line 7069] flush_pending_triggers()  <-- keyword triggers go on stack here
  -> [line 7075] grant priority
```

### Target call chain (fixed)

```
resolve_top_of_stack()
  -> [at each ETB site] queue_carddef_etb_triggers(state, new_id, ...)  <-- NEW
     -> iterates CardDef.abilities for WhenEntersBattlefield/TributeNotPaid
     -> pushes PendingTrigger { kind: Normal, ability_index: idx }
     -> returns Vec<GameEvent> (empty -- no inline effects)
  -> [at bottom, line 7057] check_triggers(state, &events)
     -> collect_triggers_for_event() matches keyword-derived triggers
     -> queues those PendingTrigger
  -> [line 7065] check_and_apply_sbas()
  -> [line 7069] flush_pending_triggers()  <-- ALL triggers go on stack here
     -> CardDef ETB triggers become TriggeredAbility SOK
  -> [line 7075] grant priority
```

### Resolution path for CardDef triggers (already exists)

The `TriggeredAbility` SOK resolution at `resolution.rs:1837-1907` already has a card registry fallback path (B14 fix). When `ability_index` doesn't match a `characteristics.triggered_abilities` entry, it falls back to `state.card_registry.get(card_id).abilities[ability_index]` and executes the `AbilityDefinition::Triggered { effect }` directly. This means the resolution side is already correct -- only the queueing side needs fixing.

## Modification Surface

| File | Function/Location | Line | What to change |
|------|-------------------|------|----------------|
| `rules/replacement.rs` | `fire_when_enters_triggered_effects` | 928-1091 | Rename to `queue_carddef_etb_triggers`; replace inline `execute_effect` with `state.pending_triggers.push_back(PendingTrigger { kind: Normal, ... })` |
| `rules/resolution.rs` | 7 call sites | 1675, 2846, 3668, 5893, 6110, 6328, 6563 | Update function name; handle return type change (now returns no events for CardDef triggers) |
| `rules/lands.rs` | ETB site | 423 | Same update as resolution.rs |
| `rules/resolution.rs` | `TriggeredAbility` SOK resolution | 1837-1907 | No change needed (B14 fallback handles it) |
| `rules/abilities.rs` | `flush_pending_triggers` | 5724+ | No change needed (Normal -> TriggeredAbility SOK already handled) |

## Implementation Steps

### Step 1: Refactor `fire_when_enters_triggered_effects` into `queue_carddef_etb_triggers`

**File**: `crates/engine/src/rules/replacement.rs`
**Lines**: 928-1091

Replace the inline `execute_effect()` calls with `state.pending_triggers.push_back()` for `WhenEntersBattlefield` and `TributeNotPaid` triggers.

The function signature changes:
```rust
// BEFORE:
pub fn fire_when_enters_triggered_effects(
    state: &mut GameState,
    new_id: ObjectId,
    controller: PlayerId,
    card_id: Option<&CardId>,
    registry: &CardRegistry,
) -> Vec<GameEvent>

// AFTER:
pub fn queue_carddef_etb_triggers(
    state: &mut GameState,
    new_id: ObjectId,
    controller: PlayerId,
    card_id: Option<&CardId>,
    registry: &CardRegistry,
)
```

Key changes in the body:
1. For `WhenEntersBattlefield` triggers: instead of calling `execute_effect`, push a `PendingTrigger { source: new_id, ability_index: idx, controller, kind: PendingTriggerKind::Normal, ... }` where `idx` is the index into `def.abilities`.
2. For `TributeNotPaid` triggers: same pattern, but with an intervening-if check at trigger time (tribute_was_paid == false). Push the trigger with ability_index pointing to the TributeNotPaid entry in def.abilities.
3. For Fabricate: leave inline for now (it's a keyword ETB with a bot-play inline approximation, documented as TODO). It already has the correct annotation in the code. Alternatively, it could go through the Fabricate-specific trigger path, but that's a separate fix.
4. The function no longer returns `Vec<GameEvent>` (CardDef triggers produce no inline events).

**CR 603.4 (intervening-if at trigger time)**: The intervening-if check currently in the inline path (e.g., `Condition::OpponentHasPoisonCounters`) must be preserved. Check the condition BEFORE pushing the `PendingTrigger`. If false, skip the trigger (same as `collect_triggers_for_event` does for keyword triggers).

**Kicker context**: The current inline path creates `EffectContext::new_with_kicker(controller, new_id, vec![], kicker_times_paid)`. The kicker state is on the permanent (`obj.kicker_times_paid`) and will be available at resolution time when the `TriggeredAbility` SOK resolves. The resolution path at lines 1894-1901 creates `EffectContext::new(controller, source_object, ...)` -- this needs kicker_times_paid propagation. **This is a sub-fix**: add `kicker_times_paid` lookup in the CardDef fallback resolution path.

### Step 2: Update all 8 call sites

**Files**: `resolution.rs` (7 sites), `lands.rs` (1 site)

At each call site:
```rust
// BEFORE:
let etb_trigger_evts = super::replacement::fire_when_enters_triggered_effects(
    state, new_id, controller, card_id.as_ref(), &registry,
);
events.extend(etb_trigger_evts);

// AFTER:
super::replacement::queue_carddef_etb_triggers(
    state, new_id, controller, card_id.as_ref(), &registry,
);
// No events to extend -- triggers will be flushed to stack later.
```

Call sites (resolution.rs line numbers):
1. **L1675**: Creature/permanent spell resolution (main path)
2. **L2846**: ChampionLTB trigger resolution (exiled card returns)
3. **L3668**: Another ChampionLTB path
4. **L5893**: NinjutsuTrigger resolution (ninja enters)
5. **L6110**: EmbalmTrigger resolution (token enters)
6. **L6328**: EternalizeTrigger resolution (token enters)
7. **L6563**: EncoreTrigger resolution (token enters, inside loop)

Call site (lands.rs):
8. **L423**: `handle_play_land` (land enters battlefield)

### Step 3: Fix kicker context in CardDef trigger resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Lines**: 1894-1901 (CardDef fallback path in TriggeredAbility SOK resolution)

Currently:
```rust
let mut ctx = EffectContext::new(
    stack_obj.controller,
    source_object,
    stack_obj.targets.clone(),
);
```

Need to add kicker context:
```rust
let kicker_times_paid = state.objects.get(&source_object)
    .map(|o| o.kicker_times_paid)
    .unwrap_or(0);
let mut ctx = EffectContext::new_with_kicker(
    stack_obj.controller,
    source_object,
    stack_obj.targets.clone(),
    kicker_times_paid,
);
```

### Step 4: Handle face-down suppression

**File**: `crates/engine/src/rules/replacement.rs` (in the new `queue_carddef_etb_triggers`)

The current code at resolution.rs L1671-1674 suppresses `fire_when_enters_triggered_effects` for face-down permanents:
```rust
let etb_trigger_evts = if is_face_down_entering {
    vec![]  // CR 708.3: No ETB abilities from the card itself fire face-down.
} else {
    fire_when_enters_triggered_effects(...)
};
```

The face-down check should move INTO `queue_carddef_etb_triggers`:
```rust
pub fn queue_carddef_etb_triggers(state: &mut GameState, new_id: ObjectId, ...) {
    // CR 708.3: Face-down permanents have no triggered abilities.
    if let Some(obj) = state.objects.get(&new_id) {
        if obj.status.face_down && obj.face_down_as.is_some() {
            return;
        }
    }
    // ... rest of the function
}
```

Then the call sites no longer need the `if is_face_down_entering` guard.

### Step 5: Update tests

Tests that verify inline ETB behavior need to account for triggers now going on the stack.

**Tests that will break (and how to fix)**:

1. **Evoke tests** (`tests/evoke.rs`): These test Mulldrifter (which has `WhenEntersBattlefield` draw trigger in its CardDef). Currently the draw happens inline during resolution. After the fix, the draw trigger goes on the stack alongside the evoke sacrifice trigger. Tests need to pass priority to resolve both triggers.

2. **Fabricate tests** (`tests/fabricate.rs`): Fabricate stays inline (it's a keyword bot approximation, not a CardDef trigger). No change.

3. **Corrupted ETB tests** (`tests/corrupted.rs`): These test `intervening_if: Some(Condition::OpponentHasPoisonCounters(3))` on `WhenEntersBattlefield`. After the fix, the trigger goes on the stack (intervening-if is checked at trigger time to decide whether to queue, then re-checked at resolution time). Tests need to pass priority to resolve the trigger.

4. **Kicker ETB tests** (`tests/kicker.rs`): If these test kicker-conditional ETB effects from CardDefs, they'll need priority passes. Check whether these use CardDef triggers or ObjectSpec triggers.

5. **Connive ETB tests** (`tests/connive.rs`): If `WhenEntersBattlefield` triggers connive from a CardDef.

6. **Discover tests** (`tests/discover.rs`): Geological Appraiser has `WhenEntersBattlefield` discover trigger.

7. **Gift ETB tests** (`tests/gift.rs`): Gift has its own PendingTriggerKind; no change expected.

8. **Tribute tests** (`tests/tribute.rs`): Fanatic of Xenagos's `TributeNotPaid` trigger goes on the stack now.

**Test pattern after fix**: Instead of asserting the effect happened during resolution, assert the trigger is on the stack, then pass priority to resolve it, then assert the effect.

```rust
// BEFORE (inline): after casting + pass_all, effect already happened
let (state, _) = pass_all_four(state, ...);
assert_eq!(hand_count(&state, p1), initial + 1); // draw already happened

// AFTER (stack): after casting + pass_all, trigger is on stack
let (state, _) = pass_all_four(state, ...);  // resolves creature spell
assert_eq!(state.stack_objects.len(), 1);      // ETB trigger on stack
let (state, _) = pass_all_four(state, ...);  // resolves ETB trigger
assert_eq!(hand_count(&state, p1), initial + 1); // NOW the draw happened
```

### Step 6: Update game scripts

Game scripts that test cards with `WhenEntersBattlefield` CardDef triggers will need additional priority-pass steps between permanent resolution and effect assertion. Check scripts in `test-data/generated-scripts/` that reference the 20 affected cards.

## Interactions to Watch

### 1. Evoke ordering (CR 702.74a)
When a creature is evoked, both the evoke sacrifice trigger AND the card's ETB trigger go on the stack. The controller orders them (APNAP, CR 603.3b). If the draw trigger is on top, it resolves first (player draws before sacrifice). If the sacrifice is on top, the creature is sacrificed before the draw. After this fix, this ordering becomes correct -- currently the draw happens inline before the sacrifice trigger.

### 2. Stifle/Counterspell interaction
Currently, inline ETB effects cannot be countered. After this fix, a player can Stifle Wall of Omens' draw trigger, which is CR-correct.

### 3. Panharmonicon doubling
Panharmonicon doubles `AnyPermanentEntersBattlefield` triggers. CardDef `WhenEntersBattlefield` triggers going through `PendingTriggerKind::Normal` will NOT be doubled by the current `doubler_applies_to_trigger` function (which only matches specific `PendingTriggerKind` variants and `TriggerEvent` types). This is a pre-existing gap that should be addressed separately, but the fix should NOT make it worse.

### 4. Rest in Peace special case
Rest in Peace has a `WhenEntersBattlefield` trigger that exiles all cards from all graveyards. Currently this fires inline, which means it happens atomically during resolution. After the fix, it goes on the stack, meaning opponents can respond (e.g., flash in a creature from the graveyard before it's exiled). This is CR-correct but changes observable behavior. The `rest_in_peace.rs` card def comment specifically says "Executed inline (non-interactively)" -- this comment should be removed.

### 5. Multiple ETB triggers from same permanent
If a card has multiple `WhenEntersBattlefield` entries (no current cards do, but possible), each gets its own `PendingTrigger`. The controller orders them. This is correct per CR 603.3b.

### 6. Token copies with ETB triggers
Embalm/Eternalize/Encore create token copies. If the original card's CardDef has a `WhenEntersBattlefield` trigger, the token should also get the trigger. The current code passes `source_card_id` to `fire_when_enters_triggered_effects` for tokens -- the fixed version needs to do the same (pass the card_id of the copied card to `queue_carddef_etb_triggers`).

## Risk Assessment

### Low risk
- **Resolution path**: The `TriggeredAbility` SOK resolution already has CardDef registry fallback (B14 fix). No new resolution code needed.
- **Flush infrastructure**: `flush_pending_triggers` already handles `PendingTriggerKind::Normal` -> `TriggeredAbility` SOK.
- **Pattern precedent**: Upkeep/end-step CardDef trigger sweep (B14) uses the exact same pattern.

### Medium risk
- **Test breakage scope**: 20 card definitions affected; tests for any of these cards will need extra priority passes. The test fixes are mechanical but numerous.
- **Kicker context**: The resolution fallback path doesn't pass kicker_times_paid to EffectContext. Cards like Torch Slinger (kicker changes the ETB target) will behave incorrectly unless Step 3 is completed.
- **Evoke interaction**: Mulldrifter evoke tests are sensitive to trigger ordering. The test currently assumes draw happens before sacrifice -- after the fix, both triggers are on the stack and the controller orders them. Bot play (deterministic) will put them in a specific order that may differ from the current inline behavior.

### Potential issues
- **Fabricate inline**: Fabricate remains inline (bot approximation). If someone adds Fabricate to a card that ALSO has a `WhenEntersBattlefield` CardDef trigger, the Fabricate will fire before the trigger hits the stack. This is a pre-existing design debt.
- **Tribute intervening-if at resolution**: The `TributeNotPaid` condition needs to be re-checked at resolution time (CR 603.4). The current CardDef fallback path at resolution.rs:1892 says "CardDef intervening_if evaluated at trigger time" and sets `condition_holds = true`. This is wrong for `TributeNotPaid` where the condition is based on a game state that could theoretically change (though in practice `tribute_was_paid` is locked on the permanent). For correctness, the resolution path should re-check the `intervening_if` condition. This is a pre-existing gap, not introduced by this fix, but it becomes newly exercised.

## File Checklist

- [ ] `crates/engine/src/rules/replacement.rs` -- refactor function
- [ ] `crates/engine/src/rules/resolution.rs` -- 7 call sites + kicker context fix
- [ ] `crates/engine/src/rules/lands.rs` -- 1 call site
- [ ] `crates/engine/src/cards/defs/rest_in_peace.rs` -- remove "inline" comment
- [ ] `crates/engine/tests/evoke.rs` -- extra priority passes
- [ ] `crates/engine/tests/corrupted.rs` -- extra priority passes
- [ ] `crates/engine/tests/tribute.rs` -- extra priority passes
- [ ] `crates/engine/tests/discover.rs` -- check if affected
- [ ] `crates/engine/tests/kicker.rs` -- check if affected
- [ ] `crates/engine/tests/connive.rs` -- check if affected
- [ ] Game scripts using affected cards -- add priority steps
- [ ] `memory/gotchas-infra.md` -- update ETB site documentation
- [ ] `memory/gotchas-rules.md` -- remove/update any references to inline ETB
