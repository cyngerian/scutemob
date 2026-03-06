# Ability Plan: Fading

**Generated**: 2026-03-06
**CR**: 702.32
**Priority**: P4
**Similar abilities studied**: Vanishing (CR 702.63, `crates/engine/src/rules/turn_actions.rs:100-155`, `crates/engine/src/rules/resolution.rs:983-1102`, `crates/engine/tests/vanishing.rs`)

## CR Rule Text

702.32. Fading

702.32a Fading is a keyword that represents two abilities. "Fading N" means "This permanent enters with N fade counters on it" and "At the beginning of your upkeep, remove a fade counter from this permanent. If you can't, sacrifice the permanent."

## Key Edge Cases

- **Single trigger handles both decrement and sacrifice (CR 702.32a)**: Unlike Vanishing (which has separate counter-removal and sacrifice triggers), Fading has ONE upkeep trigger. It tries to remove a fade counter; if it can't (0 counters), it sacrifices the permanent instead. This means the sacrifice happens during trigger resolution, not as a separate triggered ability.
- **No "Fading without a number" variant**: Unlike Vanishing (702.63b), Fading always has N. Every Fading card specifies a number.
- **Fade counters, not time counters**: Fading uses `CounterType::Fade` (new), not `CounterType::Time`. Parallax Wave and Saproling Burst use "fade counters" in their activated abilities ("Remove a fade counter from this enchantment: ..."), so the counter type must be correct.
- **Stifle interaction**: If the upkeep trigger is countered (e.g., Stifle), the permanent keeps its current fade counter count. On the next upkeep, the trigger fires again normally. This differs from Vanishing where Stifle on the sacrifice trigger leaves it at 0 counters with no future triggers.
- **Proliferate / Vampire Hexmage**: Adding fade counters extends lifespan. Removing all fade counters does NOT immediately sacrifice (sacrifice only happens during the upkeep trigger resolution when removal fails).
- **Multiplayer**: Only the ACTIVE player's permanents trigger (CR 702.32a: "your upkeep"). Same pattern as Vanishing/Suspend.
- **Multiple instances**: No explicit rule for multiple instances (unlike Vanishing 702.63c), but standard keyword rules apply -- each instance triggers separately. Two instances = two counter-removal triggers per upkeep. Second trigger would try to remove when count may already be 0, causing sacrifice.
- **Enchantments with Fading**: Parallax Wave and Saproling Burst are enchantments (not creatures). The sacrifice path must handle non-creature permanents (use `PermanentSacrificed` event, not just `CreatureDied`).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (ETB counter placement)
- [ ] Step 3: Trigger wiring (upkeep trigger)
- [ ] Step 4: Unit tests

## Implementation Steps

### Step 1: Enum Variant + Counter Type

**File**: `crates/engine/src/state/types.rs`
**Action 1a**: Add `Fade` to the `CounterType` enum, after `Time` (line 141):
```rust
Fade,
```
**Action 1b**: Add `KeywordAbility::Fading(u32)` variant after `Vanishing(u32)` (line 1013):
```rust
/// CR 702.32a: Fading N -- "This permanent enters with N fade counters on it"
/// and "At the beginning of your upkeep, remove a fade counter from this
/// permanent. If you can't, sacrifice the permanent."
///
/// Unlike Vanishing, Fading always has a number (no "Fading without a number").
/// The upkeep trigger is a SINGLE trigger that handles both counter removal and
/// sacrifice (if removal fails).
Fading(u32),
```
**Discriminant**: 113 (next available after Vanishing=112).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `KeywordAbility::Fading(n)` with discriminant 113u8, hashing n.
**Pattern**: Follow `KeywordAbility::Vanishing(n)` at hash.rs line 562-565.

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Fading { count: u32 }` variant after `Vanishing { count: u32 }` (line 471):
```rust
/// CR 702.32a: Fading N -- "This permanent enters with N fade counters on it."
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Fading(N))` for quick
/// presence-checking without scanning all abilities.
///
/// `count` is N (the number of fade counters placed on ETB).
Fading { count: u32 },
```
**Discriminant**: 42 (next available after Vanishing=41).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `AbilityDefinition::Fading { count }` with discriminant 42u8.
**Pattern**: Follow `AbilityDefinition::Vanishing { count }` at hash.rs line 3382-3385.

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: Verify `CounterType` is exported. If not, add it. (It is likely already exported since Vanishing tests use it.)

**Match arms**: Grep for all exhaustive `KeywordAbility` match expressions and add `Fading(n)` arm (typically `Fading(_) => { /* no special builder behavior */ }`). Same for `AbilityDefinition` exhaustive matches. Key locations:
- `state/hash.rs` (KeywordAbility + AbilityDefinition)
- Any exhaustive matches on KeywordAbility or AbilityDefinition

### Step 2: Rule Enforcement -- ETB Counter Placement

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: After the Vanishing ETB counter placement block (around line 466), add a parallel block for Fading that checks for `KeywordAbility::Fading(n)` and places N fade counters:
```
// CR 702.32a: "This permanent enters with N fade counters on it."
// Fading N places N fade counters on the permanent as it enters.
// Unlike Vanishing, Fading always has N > 0.
```
**CR**: 702.32a -- "This permanent enters with N fade counters on it."
**Pattern**: Identical to the Vanishing block at resolution.rs:448-470, but:
- Filter for `KeywordAbility::Fading(n)` instead of `Vanishing(n)`
- Use `CounterType::Fade` instead of `CounterType::Time`
- No special N=0 handling needed (Fading always has N)

**File**: `crates/engine/src/rules/lands.rs`
**Action**: After the Vanishing ETB counter placement block (around line 115-140), add the same block for Fading. Per ETB Site Gotchas, both `resolution.rs` and `lands.rs` must get any new ETB hook.
**Pattern**: Identical to Vanishing's lands.rs block but with `Fading(n)` and `CounterType::Fade`.

### Step 3: Trigger Wiring -- Single Upkeep Trigger

#### 3a: PendingTriggerKind

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add one new variant to `PendingTriggerKind` after `VanishingSacrifice` (line 77):
```rust
/// CR 702.32a: Fading upkeep trigger -- remove a fade counter or sacrifice.
FadingUpkeep,
```
**Note**: Only ONE trigger kind needed (unlike Vanishing's two). The single trigger handles both counter removal and sacrifice.

#### 3b: StackObjectKind

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add one new variant to `StackObjectKind` after `VanishingSacrificeTrigger` (line 905):
```rust
/// CR 702.32a: "At the beginning of your upkeep, remove a fade counter from
/// this permanent. If you can't, sacrifice the permanent." Discriminant 39.
FadingTrigger {
    source_object: ObjectId,
    fading_permanent: ObjectId,
},
```
**Discriminant**: 39 (next available after VanishingSacrificeTrigger=38).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `StackObjectKind::FadingTrigger` with discriminant 39u8.
**Pattern**: Follow `VanishingCounterTrigger` hash arm at hash.rs line 1656-1663.

#### 3c: Upkeep Trigger Queueing

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: After the Vanishing counter loop (around line 155), add a new loop for Fading:

Scan all battlefield permanents controlled by the active player that have `KeywordAbility::Fading(_)`. Queue a `PendingTriggerKind::FadingUpkeep` for each instance.

**CR**: 702.32a -- "At the beginning of your upkeep, remove a fade counter from this permanent. If you can't, sacrifice the permanent."
**Pattern**: Follow the Vanishing loop at turn_actions.rs:108-155, but:
- Filter for `KeywordAbility::Fading(_)` instead of `Vanishing(_)`
- NO intervening-if counter check at queueing time. The Fading trigger fires regardless of whether there are fade counters -- the "if you can't" check happens at resolution. (This differs from Vanishing which has an intervening-if condition.)
- Count instances for multiple Fading (same as Vanishing 702.63c pattern)
- Use `PendingTriggerKind::FadingUpkeep`

**Important difference from Vanishing**: Vanishing's upkeep trigger has "if this permanent has a time counter on it" as an intervening-if condition (only fires if counters exist). Fading has NO such condition -- the trigger always fires. It tries to remove a counter; if it can't (none exist), it sacrifices. This means Fading triggers queue even when the permanent has 0 fade counters.

#### 3d: Flush Pending Triggers

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers` (after the `VanishingSacrifice` arm around line 3801), add:
```rust
PendingTriggerKind::FadingUpkeep => {
    // CR 702.32a: Fading upkeep trigger.
    // "At the beginning of your upkeep, remove a fade counter from
    // this permanent. If you can't, sacrifice the permanent."
    StackObjectKind::FadingTrigger {
        source_object: trigger.source,
        fading_permanent: trigger.source,
    }
}
```

#### 3e: Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution handler for `StackObjectKind::FadingTrigger` after the `VanishingSacrificeTrigger` block (around line 1102):

**FadingTrigger resolution logic**:
1. Check if permanent is still on the battlefield (CR 400.7). If not, trigger does nothing.
2. Check fade counter count on the permanent.
3. **If fade counters > 0**: Remove one fade counter. Emit `CounterRemoved` event.
4. **If fade counters == 0 (or None)**: Sacrifice the permanent (same sacrifice logic as `VanishingSacrificeTrigger` -- zone-change replacement check, move to graveyard, emit `CreatureDied`/`PermanentSacrificed`).
5. Emit `AbilityResolved`.

**CR**: 702.32a -- "remove a fade counter from this permanent. If you can't, sacrifice the permanent."
**Pattern**: Combine the counter-removal logic from `VanishingCounterTrigger` (resolution.rs:990-1070) with the sacrifice logic from `VanishingSacrificeTrigger` (resolution.rs:1080-1102+), but in a single block:
- Use `CounterType::Fade` instead of `CounterType::Time`
- If count > 0: remove counter (like Vanishing counter trigger)
- If count == 0 or None: sacrifice (like Vanishing sacrifice trigger)
- No separate sacrifice trigger queued -- sacrifice happens inline

#### 3f: Add to exhaustive match arms

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `| StackObjectKind::FadingTrigger { .. }` to the no-op/countered match arm at ~line 3726 alongside the Vanishing variants.

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arm for the new variant:
```rust
StackObjectKind::FadingTrigger { fading_permanent, .. } => {
    ("Fading: ".to_string(), Some(*fading_permanent))
}
```
**Pattern**: Follow `VanishingCounterTrigger` arm at stack_view.rs:141-144.

### Step 4: Unit Tests

**File**: `crates/engine/tests/fading.rs`
**Tests to write**:

- `test_fading_etb_places_fade_counters` -- CR 702.32a: Permanent with Fading 3 enters with 3 fade counters (not time counters). Verify via CastSpell flow.
- `test_fading_upkeep_removes_counter` -- CR 702.32a: At beginning of upkeep, one fade counter is removed. 3 -> 2 counters.
- `test_fading_sacrifice_when_no_counters` -- CR 702.32a: When upkeep trigger fires with 0 fade counters, permanent is sacrificed. Start with 1 counter, first upkeep removes it (0 remaining), second upkeep trigger can't remove -> sacrifice.
- `test_fading_full_lifecycle` -- CR 702.32a: Fading 2 permanent enters with 2 counters. First upkeep: 2->1. Second upkeep: 1->0. Third upkeep: can't remove -> sacrificed. (3 upkeeps total, unlike Vanishing 2 which takes 2 upkeeps to sacrifice.)
- `test_fading_multiplayer_only_active_player` -- CR 702.32a "your upkeep": Only active player's permanents trigger.
- `test_fading_non_creature_sacrifice` -- CR 702.32a: Fading on an enchantment (like Parallax Wave) sacrifices correctly.
- `test_fading_uses_fade_counters_not_time` -- CR 702.32a: Verify that Fading uses `CounterType::Fade`, not `CounterType::Time`. A permanent with both Fading and time counters should only decrement fade counters.

**Pattern**: Follow `crates/engine/tests/vanishing.rs` structure exactly. Same helpers (`p()`, `find_object()`, `find_in_zone()`, `on_battlefield()`, `in_graveyard()`, `pass_all()`). Replace:
- `CounterType::Time` -> `CounterType::Fade`
- `KeywordAbility::Vanishing(N)` -> `KeywordAbility::Fading(N)`
- `AbilityDefinition::Vanishing { count: N }` -> `AbilityDefinition::Fading { count: N }`
- `StackObjectKind::VanishingCounterTrigger` -> `StackObjectKind::FadingTrigger`
- No `VanishingSacrificeTrigger` equivalent -- sacrifice happens inside `FadingTrigger` resolution
- Counter helper renamed: `fade_counters()` instead of `time_counters()`
- Card IDs: `CardId("test-fading-3".into())` etc.

**Key lifecycle difference from Vanishing**: Fading N survives N upkeeps (not N-1). Fading 3 -> 3 counters -> upkeep 1 (2 counters) -> upkeep 2 (1 counter) -> upkeep 3 (0 counters) -> upkeep 4 (can't remove, sacrifice). Vanishing 3 sacrifices on the 3rd upkeep (when last counter is removed, sacrifice trigger fires immediately). Fading 3 sacrifices on the 4th upkeep (counter is removed to 0 successfully; next upkeep can't remove, then sacrifice).

**Wait -- re-reading CR 702.32a carefully**: "remove a fade counter from this permanent. If you can't, sacrifice the permanent." When count is 1, the trigger removes the counter (1->0). That upkeep does NOT sacrifice. The next upkeep, it can't remove (0 counters), so it sacrifices. This means Fading N gives N+1 upkeeps of life (N counter-removal upkeeps + 1 sacrifice upkeep). This is a critical difference from Vanishing where Vanishing N gives N upkeeps.

### Step 5: Card Definition (later phase)

**Suggested card**: Blastoderm ({2}{G}{G}, 5/5 Beast, Shroud, Fading 3)
**Rationale**: Iconic Nemesis card. Shroud is already validated. Simple creature with no other abilities beyond Fading and Shroud.
**Alternative**: Parallax Wave ({2}{W}{W}, Enchantment, Fading 5, activated ability to exile creatures) -- more complex, tests non-creature Fading but requires exile tracking infrastructure.
**Card lookup**: use `card-definition-author` agent.

### Step 6: Game Script (later phase)

**Suggested scenario**: Blastoderm enters with 3 fade counters. Over 4 upkeeps: counters tick down 3->2->1->0, then on the 4th upkeep (0 counters), Blastoderm is sacrificed. Assert: 3 fade counters on entry, counter removed each of first 3 upkeeps, sacrifice on 4th upkeep when removal fails.
**Subsystem directory**: `test-data/generated-scripts/stack/` (trigger resolution is stack-centric).

### Step 7: Coverage Doc Update

**File**: `docs/mtg-engine-ability-coverage.md`
**Action**: Update Fading row from `none` to `validated` with card and script references.

## Interactions to Watch

- **CounterType::Fade is distinct from CounterType::Time.** Fading and Vanishing use different counter types. A permanent with both Fading and Vanishing (unlikely but possible via copy) would have both fade and time counters, and each ability only interacts with its own counter type.
- **Humility / Dress Down (Layer 6 ability removal)**: If Fading is removed from a permanent that already has fade counters, the upkeep trigger should NOT fire (keyword check uses layer-resolved characteristics). Counters remain but are inert until the keyword is restored.
- **Proliferate**: Adding fade counters to a Fading permanent extends its lifespan (more removals before 0). Proliferate can target fade counters.
- **Vampire Hexmage**: Removing all fade counters does NOT immediately sacrifice. The permanent survives until the next upkeep trigger fires and finds 0 counters.
- **Non-creature permanents**: Parallax Wave and Saproling Burst are enchantments with Fading. The sacrifice path in FadingTrigger resolution must handle non-creatures. The existing `VanishingSacrificeTrigger` sacrifice code uses `check_zone_change_replacement` + `move_object_to_zone` which already handles any permanent type. Follow the same pattern.
- **Sacrifice prevention**: Effects like "you can't sacrifice permanents" (extremely rare) would prevent the sacrifice. The standard sacrifice path through `check_zone_change_replacement` handles this.
