# Ability Plan: Ravenous

**Generated**: 2026-03-07
**CR**: 702.156
**Priority**: P4
**Similar abilities studied**: Bloodthirst (702.54) ETB counter placement in `resolution.rs:854-920` and `lands.rs:299-350`; Impending (702.176) ETB counter placement in `resolution.rs:505-522`

## CR Rule Text

> **702.156.** Ravenous
>
> **702.156a** Ravenous is a keyword found on some creature cards with {X} in their
> mana cost. Ravenous represents both a replacement effect and a triggered ability.
> "Ravenous" means "This permanent enters with X +1/+1 counters on it" and "When
> this permanent enters, if X is 5 or more, draw a card." See rule 107.3m.

Supporting rule:

> **107.3m** If an object's enters-the-battlefield triggered ability or replacement
> effect refers to X, and the spell that became that object as it resolved had a value
> of X chosen for any of its costs, the value of X for that ability is the same as the
> value of X for that spell, although the value of X for that permanent is 0. This is
> an exception to rule 107.3i.

## Key Edge Cases

From CR 702.156a and card rulings (2022-10-07):

1. **X refers to the value chosen at cast time, NOT the number of counters that ended up
   on the permanent** (ruling 2022-10-07). If a replacement effect modifies the number of
   counters (e.g., Doubling Season), the "if X is 5 or more" check still uses the original
   X, not the doubled amount.
2. **Copied spells retain the same X value** (ruling 2022-10-07). If a permanent spell with
   Ravenous is copied, the copy has the same X and enters with X counters.
3. **A copy of a creature with Ravenous that enters as a copy (e.g., Clone) does NOT get
   counters** (ruling 2022-10-07). Ravenous only fires when the spell had an X cost.
4. **Counters are placed as it enters (replacement effect), not after** (ruling 2022-10-07).
   Triggered abilities that check P/T on ETB see the counters.
5. **The "draw a card" part is a triggered ability** (intervening-if: "if X is 5 or more").
   It uses the stack and can be responded to / countered.
6. **X is 0 for permanents on the battlefield** (CR 107.3m). Only the ETB replacement and
   trigger use the cast-time X value.
7. **Multiplayer**: No special multiplayer interaction. The draw trigger has no "opponent"
   component.

## Critical Infrastructure Gap: X Value Tracking

**The engine currently has NO support for tracking X from CastSpell through to ETB.**

- `CastSpell` has no `x_value` field
- `StackObject` has no `x_value` field
- `GameObject` has no `x_value` field (for ETB trigger reference)
- `EffectAmount::XValue` exists but `resolve_amount()` returns `0` (placeholder at `effects/mod.rs:2594`)
- `EffectContext` has no `x_value` field

This is a **prerequisite** that must be added before Ravenous can work. The same
infrastructure will unblock `EffectAmount::XValue` for all X-cost spells (e.g.,
Pull from Tomorrow, which currently uses `XValue` but silently draws 0 cards).

### How X flows through the system

1. **CastSpell** command: player declares `x_value: u32` (new field)
2. **Casting** (`casting.rs`): validate X >= 0; add X to the generic mana cost of the spell;
   store X on the `StackObject` (new field: `x_value: u32`)
3. **Resolution** (`resolution.rs`): when a permanent spell resolves, copy `x_value` from
   `StackObject` to `GameObject` (new field: `x_value: u32`) per CR 107.3m
4. **Effect resolution** (`effects/mod.rs`): `resolve_amount(XValue)` reads `ctx.x_value`
   instead of returning 0
5. **EffectContext**: add `x_value: u32` field, populated from StackObject at resolution

## Current State (from ability-wip.md)

- [ ] 1. Enum variant (KeywordAbility::Ravenous)
- [ ] 2. Rule enforcement (X value infra + ETB counter placement + draw trigger)
- [ ] 3. Trigger wiring (draw-a-card ETB trigger when X >= 5)
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: X Value Infrastructure (PREREQUISITE)

This step adds X value tracking across the entire spell lifecycle. It is NOT
Ravenous-specific; it unblocks all X-cost spells.

#### 1a. CastSpell command — add `x_value` field

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `x_value: u32` field to `CastSpell` variant (with `#[serde(default)]`)
**Location**: After `fuse: bool` field (~line 282), before the closing `}`
**Doc comment**: `/// CR 107.3m: The value chosen for X in the spell's mana cost. 0 for non-X spells.`

#### 1b. StackObject — add `x_value` field

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `pub x_value: u32` field to `StackObject` struct (with `#[serde(default)]`)
**Location**: After `was_fused: bool` (~line 278)
**Doc comment**: `/// CR 107.3m: The value of X chosen when this spell was cast. 0 for non-X spells.`

#### 1c. GameObject — add `x_value` field

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `pub x_value: u32` field to `GameObject` struct (with `#[serde(default)]`)
**Doc comment**: `/// CR 107.3m: The value of X chosen when the spell that became this permanent was cast. Used by ETB replacement effects and triggers that reference X (e.g., Ravenous). The permanent's own X is 0 (CR 107.3i), but ETB abilities use this stored value.`

#### 1d. EffectContext — add `x_value` field

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add `pub x_value: u32` field to `EffectContext` struct (~line 77)
**Doc comment**: `/// CR 107.3m: The value of X for this spell or ability. Set from StackObject.x_value at resolution.`

#### 1e. Casting — validate and propagate X

**File**: `crates/engine/src/rules/casting.rs`
**Action**: In `handle_cast_spell`, after building the `StackObject`:
- Copy `cmd.x_value` to `stack_obj.x_value`
- Add X to the mana cost: `total_cost.generic += cmd.x_value` (X counts as generic mana; for spells like `{X}{2}{R}`, the X adds to the generic portion)
- Validate: if the spell's ManaCost does not indicate X-cost capability (future: add `has_x: bool` to ManaCost or CardDefinition), skip. For now, trust the command.

**Note**: The engine's `ManaCost` struct has no `has_x` field. This should be added to
`ManaCost` or `CardDefinition` so the engine can validate that X was legally declared.
For the initial implementation, accept any `x_value` on any spell (non-X spells will
have `x_value: 0` by default).

#### 1f. Resolution — propagate X to GameObject

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: In the permanent-resolution block (around line 430-440 where `cast_alt_cost` is set),
add: `obj.x_value = stack_obj.x_value;`

#### 1g. EffectContext — populate X at resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Where `EffectContext` is constructed for spell resolution, set `x_value: stack_obj.x_value`

#### 1h. resolve_amount — use ctx.x_value

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Change `EffectAmount::XValue => 0` (line 2594) to `EffectAmount::XValue => ctx.x_value as i32`

#### 1i. Hash updates

**File**: `crates/engine/src/state/hash.rs`
**Action**:
- Add `x_value.hash_into(hasher)` to StackObject's HashInto impl
- Add `x_value.hash_into(hasher)` to GameObject's HashInto impl
- EffectContext does not need hashing (transient)

#### 1j. Replay harness — add x_value to cast_spell action

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In `translate_player_action`, for the `cast_spell` action type, read
`x_value` from the action JSON (with default 0) and pass it to the `CastSpell` command.
Pattern: follow `fuse` or `modes_chosen` deserialization.

#### 1k. Replay viewer — no StackObjectKind change needed

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: No new SOK variant. But verify `cargo build --workspace` compiles.

#### 1l. Default propagation sites

**File**: Multiple files where `StackObject` is constructed (copy system, trigger system)
**Action**: Ensure `x_value: 0` is set in all existing construction sites. Since the field
has `#[serde(default)]`, existing serialized data will default to 0. But explicit
construction in Rust code (e.g., `StackObject { ... }`) will fail to compile without
the new field. Grep for `StackObject {` and add `x_value: 0` (or `x_value: stack_obj.x_value`
for copies).

### Step 2: KeywordAbility::Ravenous Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Ravenous` variant to `KeywordAbility` enum
**Location**: After `Spree` (~line 1240)
**Doc comment**: `/// CR 702.156: Ravenous -- "This permanent enters with X +1/+1 counters on it" and "When this permanent enters, if X is 5 or more, draw a card."`
**Pattern**: Follow `KeywordAbility::Fuse` (no parameter — Ravenous itself carries no N; the N comes from X at cast time)

#### 2a. Hash

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Ravenous => 135u8.hash_into(hasher)` to the KW HashInto impl
**Location**: After `Spree => 134u8` (~line 659)

#### 2b. Replay viewer keyword display

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::Ravenous => "Ravenous".to_string()` to the keyword display match
**Location**: After `KeywordAbility::Spree =>` (~line 854)

### Step 3: ETB Counter Placement (Replacement Effect)

**CR**: 702.156a — "This permanent enters with X +1/+1 counters on it"
**CR**: 107.3m — X for the ETB replacement is the X from the spell's cost

#### 3a. resolution.rs — place X +1/+1 counters at ETB

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: After the Bloodthirst block (~line 920), add a Ravenous block:
```
// CR 702.156a: Ravenous -- "This permanent enters with X +1/+1 counters on it."
// X is the value from the spell's cost (stack_obj.x_value), per CR 107.3m.
{
    let has_ravenous = obj.characteristics.keywords.contains(&KeywordAbility::Ravenous);
    if has_ravenous && stack_obj.x_value > 0 {
        let current = obj.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        obj.counters = obj.counters.update(CounterType::PlusOnePlusOne, current + stack_obj.x_value);
        events.push(GameEvent::CounterAdded {
            object_id: new_id,
            counter: CounterType::PlusOnePlusOne,
            count: stack_obj.x_value,
        });
    }
}
```
**Pattern**: Follow Bloodthirst block at line 854-920 (same counter-placement pattern)
**Note**: Unlike Bloodthirst which reads N from the keyword, Ravenous reads X from `stack_obj.x_value`

#### 3b. lands.rs — place X +1/+1 counters for land ETB (completeness)

**File**: `crates/engine/src/rules/lands.rs`
**Action**: After the Bloodthirst block (~line 350), add Ravenous block.
Since lands with Ravenous don't exist in printed Magic, and lands can't have X in
their mana cost (they aren't cast), this block is for completeness only. It should
read X from the `GameObject.x_value` field (which would be 0 for lands since lands
aren't cast as spells). Can be a no-op or omitted with a comment.
**Recommendation**: Add a comment-only placeholder: `// Ravenous: lands cannot have X cost; no ETB counters possible.`

### Step 4: ETB Triggered Ability (Draw if X >= 5)

**CR**: 702.156a — "When this permanent enters, if X is 5 or more, draw a card."
**CR**: 107.3m — X for the trigger is the X from the spell's cost
**CR**: 603.4 — Intervening-if: checked both at trigger time and resolution

This is a triggered ability with an intervening-if condition. It fires when the
permanent enters the battlefield, and the condition "X >= 5" is checked both when
the trigger would go on the stack AND when it resolves.

#### 4a. Add RavenousDraw to PendingTriggerKind

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `RavenousDraw` variant to `PendingTriggerKind`
**Doc comment**: `/// CR 702.156a: Ravenous draw trigger -- "if X is 5 or more, draw a card."`

#### 4b. Add RavenousDrawTrigger to StackObjectKind

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add variant to `StackObjectKind`:
```
RavenousDrawTrigger {
    source_object: ObjectId,
    ravenous_permanent: ObjectId,
    x_value: u32,
}
```
**Discriminant**: Next available SOK discriminant (check current chain)
**Hash**: Add to `state/hash.rs` SOK HashInto impl

#### 4c. Fire the trigger in resolution.rs

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: In the ETB trigger section (after counter placement in step 3a), check
if X >= 5 and if so, queue a `RavenousDraw` pending trigger:
```
if has_ravenous && stack_obj.x_value >= 5 {
    state.pending_triggers.push_back(PendingTrigger {
        source: new_id,
        owner: controller,
        kind: PendingTriggerKind::RavenousDraw,
        // Store x_value for intervening-if re-check at resolution
    });
}
```
**Note**: The x_value needs to be available at resolution time for the intervening-if
re-check. Store it on the PendingTrigger or on the StackObjectKind.

#### 4d. Convert pending trigger to stack object in abilities.rs

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers` or `convert_pending_trigger`, add arm for
`PendingTriggerKind::RavenousDraw`:
```
PendingTriggerKind::RavenousDraw => {
    StackObjectKind::RavenousDrawTrigger {
        source_object: trigger.source,
        ravenous_permanent: trigger.source,
        x_value: /* needs to come from somewhere -- stored on PendingTrigger or on GameObject */,
    }
}
```
**Design decision**: The simplest approach is to read `x_value` from `state.objects.get(trigger.source).x_value` since we already stored it on the GameObject in step 1f. The intervening-if re-check at resolution also reads from the GameObject.

#### 4e. Resolve the trigger in resolution.rs

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution arm for `StackObjectKind::RavenousDrawTrigger`:
```
StackObjectKind::RavenousDrawTrigger { ravenous_permanent, x_value, .. } => {
    // Intervening-if re-check (CR 603.4): x_value must still be >= 5
    // x_value is fixed from cast time, so this always passes if it triggered.
    // But check the permanent is still on the battlefield (CR 603.4 also
    // requires the source to still exist for triggered abilities).
    if x_value >= 5 {
        if let Some(obj) = state.objects.get(&ravenous_permanent) {
            if matches!(obj.zone, ZoneId::Battlefield) {
                // Draw a card for the controller
                let drawn = crate::rules::turn_actions::draw_card(state, controller);
                events.extend(drawn);
            }
        }
    }
    events.push(GameEvent::AbilityResolved { controller, stack_object_id: stack_obj.id });
}
```
**Pattern**: Follow `ImpendingCounterTrigger` resolution at line 2474-2510

#### 4f. Fizzle handling

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::RavenousDrawTrigger { .. }` to the match arms that
handle trigger fizzle/counter. Follow the pattern of other triggered abilities (line 5227+).

### Step 5: Unit Tests

**File**: `crates/engine/tests/ravenous.rs`
**Tests to write**:

1. `test_ravenous_x3_enters_with_3_counters` -- CR 702.156a: Cast a Ravenous creature with X=3; verify 3 +1/+1 counters on ETB; verify no draw trigger fires
2. `test_ravenous_x5_enters_with_5_counters_and_draws` -- CR 702.156a: Cast with X=5; verify 5 counters AND controller draws a card
3. `test_ravenous_x0_enters_with_no_counters_no_draw` -- CR 702.156a: Cast with X=0; verify 0 counters, no draw, creature is 0/0 (base P/T) and dies to SBA if base is 0/0
4. `test_ravenous_x5_boundary_draw_fires` -- CR 702.156a: X=5 draws; X=4 does not. Boundary test.
5. `test_ravenous_x10_enters_with_10_counters` -- Large X value works correctly

**Pattern**: Follow tests for Bloodthirst in `crates/engine/tests/bloodthirst.rs`

**Test setup**: Use `GameStateBuilder::four_player()`, create a Ravenous creature
card definition inline or use the test card from Step 6. The creature should have
a mana cost like `{X}{G}` and base P/T 0/0 (like most Ravenous creatures). Use the
`cast_spell` command with the new `x_value` field.

### Step 6: Card Definition (later phase)

**Suggested card**: Tyrant Guard
- Name: Tyrant Guard
- Cost: {X}{2}{G}
- Type: Creature -- Tyranid
- P/T: 3/3
- Oracle: "Ravenous (This creature enters with X +1/+1 counters on it. If X is 5 or more, draw a card when it enters.) Shieldwall -- Sacrifice this creature: Creatures you control with counters on them gain hexproof and indestructible until end of turn."
- Abilities: `KeywordAbility::Ravenous`, activated ability (sacrifice: grant hexproof+indestructible to creatures with counters)
- The activated ability is complex; for the initial card, only Ravenous needs to work. The sacrifice ability can be `AbilityDefinition::Activated` with appropriate effects.

**Alternative simpler card**: A test-only card with just Ravenous and no other abilities.
For unit tests, define an inline card `"Test Ravenous Beast"` with cost `{X}{G}`, P/T 0/0,
abilities: `[AbilityDefinition::Keyword(KeywordAbility::Ravenous)]`.

**Card lookup**: use `card-definition-author` agent for Tyrant Guard

### Step 7: Game Script (later phase)

**Suggested scenario**: Cast Tyrant Guard (or test card) with X=3 (gets 3 counters, no draw),
then cast another with X=6 (gets 6 counters, draws a card). Verify counter counts and
hand sizes.
**Subsystem directory**: `test-data/generated-scripts/etb-triggers/`

## Interactions to Watch

1. **EffectAmount::XValue resolution**: Step 1h fixes the global `XValue => 0` bug. This
   will immediately affect Pull from Tomorrow (`cards/defs/pull_from_tomorrow.rs`) which
   uses `EffectAmount::XValue` for DrawCards. After the fix, Pull from Tomorrow will
   correctly draw X cards -- but only if the CastSpell command includes `x_value`. Existing
   tests for Pull from Tomorrow (if any) may need updating.

2. **Copy spells**: When a spell with X is copied (e.g., via Storm, Replicate), the copy
   should inherit `x_value` from the original `StackObject`. Verify that the copy system
   propagates `x_value`. Check `crates/engine/src/rules/copy.rs` for StackObject copy logic.

3. **Doubling Season / counter doublers**: If Doubling Season is in play, the replacement
   effect doubles the +1/+1 counters, but the "if X is 5 or more" check uses the original
   X value (ruling 2022-10-07). The implementation correctly separates these: counters come
   from `stack_obj.x_value` (which Doubling Season could modify), while the trigger check
   uses the stored `x_value` directly.

4. **Two ETB sites**: Per `memory/gotchas-infra.md`, both `resolution.rs` AND `lands.rs`
   are ETB sites. Ravenous on lands is impossible (lands aren't cast with X), so `lands.rs`
   only needs a comment placeholder.

5. **SBA for 0/0 creature**: A Ravenous creature with base P/T 0/0 cast with X=0 enters
   with 0 counters and has toughness 0, dying immediately to SBA 704.5f. This is correct
   behavior.

6. **ManaCost.has_x**: The engine's `ManaCost` struct has no concept of "this cost includes X."
   The X value is purely determined by the `CastSpell.x_value` field. Long-term, a `has_x: bool`
   or `x_count: u32` field on `ManaCost` or `CardDefinition` would enable validation (reject
   non-zero X on non-X spells). For now, trust the command. This is a LOW issue to address
   later.

## Discriminant Chain

- **KeywordAbility**: next = 135 (Ravenous)
- **StackObjectKind**: check current chain before assigning RavenousDrawTrigger
- **AbilityDefinition**: no new variant needed (Ravenous is just a KW, not a parameterized AbilDef)
- **PendingTriggerKind**: add RavenousDraw (check current chain)

## Effort Estimate

**Medium** (not Low as originally estimated). The X value infrastructure is a
cross-cutting change touching 8+ files. The Ravenous-specific logic is simple,
but the prerequisite work is substantial. Recommend splitting into two commits:
1. X value infrastructure (Step 1)
2. Ravenous keyword + enforcement + tests (Steps 2-5)
