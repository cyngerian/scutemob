# Ability Plan: Vanishing

**Generated**: 2026-03-06
**CR**: 702.63
**Priority**: P4
**Similar abilities studied**: Suspend (CR 702.62, `crates/engine/src/rules/suspend.rs`), Impending (CR 702.176, `crates/engine/src/rules/resolution.rs:1395-1448`, `crates/engine/src/rules/turn_actions.rs:302-355`)

## CR Rule Text

702.63. Vanishing

702.63a Vanishing is a keyword that represents three abilities. "Vanishing N" means "This permanent enters with N time counters on it," "At the beginning of your upkeep, if this permanent has a time counter on it, remove a time counter from it," and "When the last time counter is removed from this permanent, sacrifice it."

702.63b Vanishing without a number means "At the beginning of your upkeep, if this permanent has a time counter on it, remove a time counter from it" and "When the last time counter is removed from this permanent, sacrifice it."

702.63c If a permanent has multiple instances of vanishing, each works separately.

## Key Edge Cases

- **Vanishing without N (702.63b)**: No ETB counter placement. Only the two triggered abilities (upkeep counter removal + last-counter sacrifice). Time counters must be placed by some other effect for the triggers to fire. Represented as `Vanishing(0)` with special handling: skip ETB counter placement when N=0.
- **Multiple instances (702.63c)**: Each works separately. Two instances of Vanishing 3 = two upkeep triggers removing one counter each (two counters removed per upkeep). Two sacrifice triggers when last counter removed (but permanent is already gone after the first).
- **Countered sacrifice trigger (Dreamtide Whale ruling)**: If the sacrifice trigger is countered (e.g., Stifle), the permanent stays on the battlefield with 0 time counters. Neither vanishing trigger can fire again (both have "if this permanent has a time counter on it" as intervening-if). The permanent is effectively immortal until counters are added.
- **0 toughness before sacrifice (Tidewalker ruling)**: If a creature's toughness is defined by time counters (e.g., Tidewalker is */1+*), when the last counter is removed, the SBA for 0 toughness happens simultaneously with the sacrifice trigger going on the stack. The creature dies to SBA before the sacrifice trigger resolves. Not a Vanishing-specific issue, but worth testing.
- **Copy entering with Vanishing (Dreamtide Whale ruling)**: A permanent entering as a copy of a permanent with Vanishing enters with the appropriate N time counters and vanishes normally. A permanent already on the battlefield gaining Vanishing via copy does NOT get counters (the ETB replacement only fires on entering).
- **Multiplayer**: Only the ACTIVE player's permanents tick down on their upkeep (CR 702.63a: "At the beginning of YOUR upkeep"). Same pattern as Suspend.
- **Difference from Suspend**: Vanishing is on permanents (battlefield), Suspend is on cards in exile. Vanishing triggers sacrifice; Suspend triggers casting. Vanishing's upkeep trigger fires for the permanent's controller; Suspend's fires for the card's owner.

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
**Action**: Add `KeywordAbility::Vanishing(u32)` variant. The `u32` is the N value (number of time counters). `Vanishing(0)` represents "Vanishing without a number" (CR 702.63b).
**Pattern**: Follow `KeywordAbility::Suspend` (marker keyword) but parameterized like `KeywordAbility::Modular(u32)` or `KeywordAbility::Bushido(u32)`.
**Discriminant**: 112 (next available after Escalate=111).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `KeywordAbility::Vanishing(n)` with discriminant 112u8, hashing n.
**Pattern**: Follow `KeywordAbility::Bushido(n)` at the hash.rs KeywordAbility match.

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Vanishing { count: u32 }` variant. This stores the N value for the ETB counter placement. Cards include both `AbilityDefinition::Keyword(KeywordAbility::Vanishing(N))` and `AbilityDefinition::Vanishing { count: N }`.
**Discriminant**: 41 (next available after Escalate=40).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `AbilityDefinition::Vanishing { count }` with discriminant 41u8.

**Match arms**: Grep for all `KeywordAbility` match expressions and add `Vanishing(n)` arm. Key locations:
- `state/hash.rs` (KeywordAbility + AbilityDefinition)
- `cards/builder.rs` (if keyword-to-trigger builder exists)
- Any exhaustive matches on KeywordAbility

### Step 2: Rule Enforcement — ETB Counter Placement

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: In the permanent-enters-battlefield block (near line 430 where Impending counters are placed), add a block that checks for `KeywordAbility::Vanishing(n)` in the permanent's keywords and places N time counters. This fires for ALL permanents with Vanishing, not just those cast with an alt cost (unlike Impending which only fires when `was_impended`).
**CR**: 702.63a -- "This permanent enters with N time counters on it" (ETB replacement effect).
**Pattern**: Follow the Impending counter-placement block at `resolution.rs:430-447`, but check `characteristics.keywords` for `Vanishing(n)` instead of `was_impended`. Skip if N=0 (CR 702.63b: Vanishing without a number has no ETB counter placement).

**File**: `crates/engine/src/rules/lands.rs`
**Action**: Add the same ETB counter placement for lands with Vanishing. Per ETB Site Gotchas, both `resolution.rs` and `lands.rs` must get any new ETB hook.
**Pattern**: Follow whatever ETB hooks already exist in `lands.rs`.

### Step 3: Trigger Wiring — Upkeep Counter Removal + Sacrifice

#### 3a: PendingTriggerKind

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add two new variants to `PendingTriggerKind`:
- `VanishingCounter` -- CR 702.63a upkeep counter-removal trigger
- `VanishingSacrifice` -- CR 702.63a last-counter sacrifice trigger

**Pattern**: Follow `SuspendCounter` and `ImpendingCounter`.

#### 3b: StackObjectKind

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add two new variants to `StackObjectKind`:
- `VanishingCounterTrigger { source_object: ObjectId, vanishing_permanent: ObjectId }` -- discriminant 37 (next after GravestormTrigger=36)
- `VanishingSacrificeTrigger { source_object: ObjectId, vanishing_permanent: ObjectId }` -- discriminant 38

**Pattern**: Follow `ImpendingCounterTrigger` and `SuspendCounterTrigger`.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms for both new StackObjectKind variants with discriminants 37 and 38.

#### 3c: Upkeep Trigger Queueing

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: In `upkeep_actions()`, after the Suspend counter loop (around line 98), add a new loop that scans all battlefield permanents controlled by the active player that have `KeywordAbility::Vanishing(_)` AND at least one time counter. Queue a `PendingTriggerKind::VanishingCounter` for each.
**CR**: 702.63a -- "At the beginning of your upkeep, if this permanent has a time counter on it, remove a time counter from it."
**Note**: Must use layer-resolved characteristics (`calculate_characteristics`) to read keywords, per gotchas-infra.md (parameterized keyword N-value extraction). However, Vanishing's upkeep trigger cares about keyword presence, not the N value. Still use layer-resolved chars for correctness under Humility/Dress Down.
**Note**: CR 702.63c (multiple instances): each instance triggers separately. If a permanent has two instances of Vanishing, queue two triggers. This requires iterating keywords and counting Vanishing instances.

#### 3d: Flush Pending Triggers

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers` (the big match on `PendingTriggerKind`), add:
- `PendingTriggerKind::VanishingCounter => StackObjectKind::VanishingCounterTrigger { source_object: trigger.source, vanishing_permanent: trigger.source }`
- `PendingTriggerKind::VanishingSacrifice => StackObjectKind::VanishingSacrificeTrigger { source_object: trigger.source, vanishing_permanent: trigger.source }`

**Pattern**: Follow `PendingTriggerKind::ImpendingCounter` at `abilities.rs:3775-3783`.

#### 3e: Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution handlers for both new StackObjectKind variants:

**VanishingCounterTrigger**:
1. Intervening-if (CR 603.4): permanent must still be on battlefield AND have at least one time counter.
2. Remove one time counter.
3. Emit `CounterRemoved` event.
4. If that was the last counter (count was 1), queue a `VanishingSacrifice` trigger.
5. Emit `AbilityResolved`.

**Pattern**: Follow `SuspendCounterTrigger` resolution at `resolution.rs:1765-1847`, but adapted: sacrifice trigger instead of cast trigger.

**VanishingSacrificeTrigger**:
1. Intervening-if (CR 603.4): permanent must still be on battlefield. (No counter check needed -- the trigger fires because the last counter was removed.)
2. Sacrifice the permanent (move to graveyard, emit `CreatureDied` or `PermanentSacrificed`).
3. Emit `AbilityResolved`.

**Pattern**: Follow `BlitzSacrificeTrigger` or `EvokeSacrificeTrigger` resolution patterns for the sacrifice logic.

#### 3f: Add to exhaustive match arms

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `| StackObjectKind::VanishingCounterTrigger { .. } | StackObjectKind::VanishingSacrificeTrigger { .. }` to the no-op match arm at ~line 3473.

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arms for both new variants:
- `StackObjectKind::VanishingCounterTrigger { vanishing_permanent, .. } => ("Vanishing tick: ".to_string(), Some(*vanishing_permanent))`
- `StackObjectKind::VanishingSacrificeTrigger { vanishing_permanent, .. } => ("Vanishing sacrifice: ".to_string(), Some(*vanishing_permanent))`

### Step 4: Unit Tests

**File**: `crates/engine/tests/vanishing.rs`
**Tests to write**:
- `test_vanishing_etb_places_time_counters` -- CR 702.63a: Permanent with Vanishing 3 enters with 3 time counters.
- `test_vanishing_upkeep_removes_counter` -- CR 702.63a: At beginning of upkeep, one time counter is removed.
- `test_vanishing_sacrifice_on_last_counter` -- CR 702.63a: When last counter removed, sacrifice trigger queued and resolves to sacrifice.
- `test_vanishing_full_lifecycle` -- CR 702.63a: Vanishing 2 permanent enters, ticks down over 2 upkeeps, sacrificed on 3rd upkeep.
- `test_vanishing_without_number_no_etb_counters` -- CR 702.63b: Vanishing(0) does not place counters at ETB; triggers don't fire without external counter placement.
- `test_vanishing_multiplayer_only_active_player` -- CR 702.63a "your upkeep": Only active player's permanents tick down.
- `test_vanishing_multiple_instances` -- CR 702.63c: Two instances of Vanishing → two counter-removal triggers per upkeep.

**Pattern**: Follow Suspend tests in `crates/engine/tests/suspend.rs` and Impending tests in `crates/engine/tests/impending.rs` (if they exist), or general upkeep trigger tests.

### Step 5: Card Definition (later phase)

**Suggested card**: Aven Riftwatcher (2W, 2/3 Bird Rebel Soldier, Flying, Vanishing 3, ETB/LTB: gain 2 life)
**Alternative**: Keldon Marauders (1R, 3/3 Human Warrior, Vanishing 2, ETB/LTB: deals 1 damage to target player or planeswalker)
**Rationale**: Aven Riftwatcher is a good first card -- it has Flying (already validated), Vanishing 3, and a straightforward ETB/LTB trigger. Keldon Marauders is simpler (no Flying) but has targeted damage on ETB/LTB.
**Card lookup**: use `card-definition-author` agent.

### Step 6: Game Script (later phase)

**Suggested scenario**: Aven Riftwatcher enters the battlefield with 3 time counters. Over 3 upkeeps, counters tick down. On the upkeep when the last counter is removed, the sacrifice trigger fires. Assert: life gained on ETB, 3 counters on entry, counter removed each upkeep, sacrifice on last counter removal, life gained on LTB.
**Subsystem directory**: `test-data/generated-scripts/stack/` (trigger resolution is stack-centric).

### Step 7: Coverage Doc Update

**File**: `docs/mtg-engine-ability-coverage.md`
**Action**: Update Vanishing row from `none` to `validated` with card and script references.

## Interactions to Watch

- **CounterType::Time is shared with Suspend and Impending.** Vanishing uses the same counter type. If a permanent has both Vanishing and Impending (unlikely but possible via copy effects), both counter-removal triggers fire independently. This should work correctly because they have separate PendingTriggerKind variants.
- **Humility / Dress Down (Layer 6 ability removal)**: If Vanishing is removed from a permanent that already has time counters, the upkeep trigger should NOT fire (keyword check uses layer-resolved characteristics). But if Humility is later removed, the triggers resume. This is automatic if we check `calculate_characteristics` at trigger queue time.
- **Sacrifice vs. "put into graveyard"**: The sacrifice trigger specifically says "sacrifice it." Sacrifice is a specific action (CR 701.17) distinct from "destroy" or "put into graveyard." Cards like Sigarda, Host of Herons ("Spells and abilities your opponents control can't cause you to sacrifice permanents") don't apply because it's your own ability causing the sacrifice. But Tajuru Preserver ("Spells and abilities your opponents control can't cause you to sacrifice permanents") also doesn't apply. However, if a replacement effect prevents sacrifice (rare), the permanent stays.
- **Proliferate**: Adding time counters to a Vanishing permanent extends its lifespan. Removing time counters (e.g., Vampire Hexmage) accelerates sacrifice. Both are correct by design since the trigger checks counter presence.
