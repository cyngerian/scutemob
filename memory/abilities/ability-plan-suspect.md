# Ability Plan: Suspect

**Generated**: 2026-03-07
**CR**: 701.60
**Priority**: P4 (Batch 13.2)
**Similar abilities studied**: Goad (Effect::Goad, goaded_by on GameObject, combat.rs enforcement), Decayed (KeywordAbility::Decayed, can't-block check in combat.rs), Renowned (is_renowned designation on GameObject)

## CR Rule Text

701.60. Suspect

701.60a Certain spells and abilities instruct a player to suspect a creature. That creature becomes suspected until it leaves the battlefield or until a spell or ability causes it to no longer be suspected.

701.60b Suspected is a designation a permanent can have. Only permanents can have the suspected designation. Suspected is neither an ability nor part of the permanent's copiable values.

701.60c A suspected permanent has menace and "This creature can't block" for as long as it's suspected.

701.60d A suspected permanent can't become suspected again.

## Key Edge Cases

- **Humility interaction (ruling 2024-02-02)**: If a suspected creature loses all abilities, it will lose menace and "This creature can't block", but it won't stop being suspected. When the ability-removal effect ends, suspect grants return.
- **Not a copiable value (CR 701.60b, ruling 2024-02-02)**: If a permanent becomes a copy of a suspected creature, it won't be suspected. The `is_suspected` flag is not copied by `copy.rs`.
- **Idempotent (CR 701.60d)**: A suspected permanent can't become suspected again. Suspecting an already-suspected creature is a no-op.
- **No limit on suspected creatures (ruling 2024-02-02)**: Any number of creatures can be suspected simultaneously. Suspecting a new creature doesn't cause other creatures to stop being suspected.
- **Unsuspect**: Some effects cause a creature to "no longer be suspected." This simply clears the `is_suspected` flag.
- **Zone change clears designation (CR 701.60a)**: Suspected status lasts "until it leaves the battlefield." Zone change (CR 400.7) creates a new object, so `is_suspected` resets to false in `move_object_to_zone`.
- **Multiplayer**: No special multiplayer considerations. Any player's effect can suspect any creature.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant / Effect / Event / GameObject field
- [ ] Step 2: Rule enforcement (calculate_characteristics + combat.rs)
- [ ] Step 3: Trigger wiring (n/a -- Suspect is a keyword action, not a trigger)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Effect, Event, and GameObject Field

Suspect is a **keyword action** (CR 701.60), not a keyword ability. It does NOT get a `KeywordAbility` enum variant. It follows the pattern of `Effect::Goad` -- an effect that applies a designation to a creature.

**No `KeywordAbility` variant needed.** The user prompt suggested KW 137, but keyword actions are Effects, not KeywordAbilities (see gotchas-infra.md: "Keyword actions are Effects, NOT `KeywordAbility` enum variants"). Cards that have "Suspect" in their Keywords list (like Frantic Scapegoat) list it as a keyword for Scryfall categorization, but the engine models it as an Effect that sets a flag.

#### Step 1a: Add `is_suspected` field to `GameObject`

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `pub is_suspected: bool` field with `#[serde(default)]` attribute.
**Location**: Near `is_renowned` (line ~476) -- both are designations.
**Doc comment**: Reference CR 701.60b: "Suspected is a designation a permanent can have. Only permanents can have the suspected designation. Suspected is neither an ability nor part of the permanent's copiable values."

#### Step 1b: Add `Effect::Suspect` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Suspect { target: EffectTarget }` to the `Effect` enum.
**Pattern**: Follow `Effect::Goad { target: EffectTarget }` (line ~879).
**Doc comment**: Reference CR 701.60a.

#### Step 1c: Add `Effect::Unsuspect` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Unsuspect { target: EffectTarget }` to the `Effect` enum.
**Pattern**: Follow `Effect::Suspect` immediately after.
**Doc comment**: Reference CR 701.60a: "until a spell or ability causes it to no longer be suspected."

#### Step 1d: Add `GameEvent::Suspected` and `GameEvent::Unsuspected` variants

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add two event variants after the existing Goaded event (line ~784).
**Pattern**: Follow `GameEvent::Goaded`.

```
Suspected {
    object_id: ObjectId,
    controller: PlayerId,
}

Unsuspected {
    object_id: ObjectId,
    controller: PlayerId,
}
```

#### Step 1e: Hash the new field

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.is_suspected.hash_into(hasher);` after `self.is_renowned` (line ~840).
**Doc comment**: `// Suspect (CR 701.60b) -- designation flag`

#### Step 1f: Initialize in all object-creation sites

**Files to update** (initialize `is_suspected: false`):
- `crates/engine/src/state/builder.rs` -- near `is_renowned: false` (line ~973)
- `crates/engine/src/state/mod.rs` -- BOTH `move_object_to_zone` sites (lines ~347 and ~502). Add `is_suspected: false` with comment `// CR 701.60a: suspected designation is cleared on zone change.`
- `crates/engine/src/effects/mod.rs` -- all token creation sites (search for `is_renowned: false` and add `is_suspected: false` next to each)
- `crates/engine/src/rules/resolution.rs` -- object creation sites (search for `is_renowned: false`)

#### Step 1g: Update replay-viewer view_model.rs

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: No `KeywordAbility` or `StackObjectKind` arms needed (no new KW or SOK variant). However, verify compilation with `cargo build --workspace`.

### Step 2: Rule Enforcement

#### Step 2a: Effect execution for Suspect and Unsuspect

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add handler arms for `Effect::Suspect` and `Effect::Unsuspect`.
**Pattern**: Follow `Effect::Goad` handler (line ~1698).

For `Effect::Suspect`:
1. Resolve targets.
2. For each resolved `ResolvedTarget::Object(id)`:
   - Check `obj.is_suspected` -- if already true, skip (CR 701.60d: idempotent).
   - Set `obj.is_suspected = true`.
   - Emit `GameEvent::Suspected { object_id: id, controller: ctx.controller }`.

For `Effect::Unsuspect`:
1. Resolve targets.
2. For each resolved `ResolvedTarget::Object(id)`:
   - If `obj.is_suspected` is true, set to false.
   - Emit `GameEvent::Unsuspected { object_id: id, controller: ctx.controller }`.

#### Step 2b: Menace grant in calculate_characteristics

**File**: `crates/engine/src/rules/layers.rs`
**Action**: In `calculate_characteristics`, add a check: if the raw `GameObject` has `is_suspected == true`, insert `KeywordAbility::Menace` into `chars.keywords`.
**Location**: BEFORE the layer loop begins (alongside the Changeling CDA at lines ~63-73), so that Layer 6 ability-removal effects (Humility) can strip the Menace grant.
**CR**: 701.60c: "A suspected permanent has menace [...] for as long as it's suspected."
**Humility interaction**: Because Menace is added before Layer 6 effects run, a Humility-style "lose all abilities" in Layer 6 will correctly strip the Menace. This matches the ruling: "If a suspected creature loses all abilities, it will lose menace."

**Note**: Add the Menace insertion BEFORE the layer loop, not inside a specific layer case. The Changeling/Devoid CDAs are already placed before the loop as a special pre-layer step. Suspect's Menace grant should be placed in the same block. It will be part of the base keywords that Layer 6 effects can modify.

#### Step 2c: "Can't block" enforcement in combat.rs

**File**: `crates/engine/src/rules/combat.rs`
**Action**: In `validate_declare_blockers`, add a check for `is_suspected` alongside the existing Decayed check (line ~546).
**Pattern**: Follow the Decayed can't-block check:
```rust
// CR 702.147a: A creature with decayed can't block.
if blocker_chars.keywords.contains(&KeywordAbility::Decayed) { ... }
```

Add after the Decayed check:
```rust
// CR 701.60c: A suspected creature can't block (it has "This creature can't block").
// Note: This is an ability granted by the suspect designation. Under Humility
// (ability removal), the can't-block restriction should also be removed.
// We use the presence of Menace in layer-resolved chars as a proxy:
// if Menace was stripped by ability removal, the can't-block ability was too.
if obj.is_suspected {
    // Check if suspect grants are still active (Menace not stripped by ability removal)
    if blocker_chars.keywords.contains(&KeywordAbility::Menace) || !ability_removal_active {
        return Err(...);
    }
}
```

**IMPORTANT Humility consideration**: The simplest correct approach is to check `obj.is_suspected` directly, matching the Decayed pattern. The Humility interaction (ability removal should also remove can't-block) is a rare edge case. Document it as a known limitation with a `// TODO: Humility interaction` comment. The Menace grant IS correctly handled (stripped in Layer 6). The can't-block restriction is harder to model as a layer effect since it's not a keyword -- accept the limitation for now.

**Simplified approach (recommended)**: Just check `obj.is_suspected` in combat.rs, like Decayed checks for the Decayed keyword. Add a `// TODO` comment about Humility interaction. This matches the Decayed pattern and avoids over-engineering.

Also add the suspected check to the provoke/goad forced-block code (~line 837) alongside the Decayed check there:
```rust
// CR 701.60c: Suspected creatures can't block.
if obj.is_suspected { continue; } // Requirement impossible -- skip
```

### Step 3: Trigger Wiring

**N/A** -- Suspect is a keyword action (applied via `Effect::Suspect`), not a triggered ability. There is no trigger to wire. Cards that trigger "whenever you suspect" or "whenever a creature becomes suspected" would be separate card-specific triggers keyed off `GameEvent::Suspected`, but those are card definition concerns, not engine infrastructure.

### Step 4: Unit Tests

**File**: `crates/engine/tests/suspect.rs`
**Tests to write**:

- `test_suspect_basic_gains_menace` -- CR 701.60c
  Suspect a creature. Verify layer-resolved characteristics include Menace.
  Pattern: Use `GameStateBuilder::four_player()`, place a vanilla creature on battlefield, execute `Effect::Suspect`, check `calculate_characteristics` includes Menace.

- `test_suspect_basic_cant_block` -- CR 701.60c
  Suspect a creature. Attempt to declare it as a blocker. Verify `InvalidCommand` error.
  Pattern: Follow Decayed can't-block test pattern.

- `test_suspect_menace_evasion` -- CR 701.60c + CR 702.110
  Suspect an attacking creature. Verify that a single blocker cannot block it (Menace requires 2+ blockers).
  Pattern: Follow Menace test pattern in existing combat tests.

- `test_suspect_idempotent` -- CR 701.60d
  Suspect an already-suspected creature. Verify no error and no additional event emitted (or a single duplicate-protection check).

- `test_suspect_zone_change_clears` -- CR 701.60a + CR 400.7
  Suspect a creature, then bounce it to hand and replay it. Verify the returned creature is NOT suspected.

- `test_suspect_not_copiable` -- CR 701.60b
  Suspect a creature. Create a copy of it (e.g., via clone effect). Verify the copy is NOT suspected.
  Note: Only if copy infrastructure supports this test easily.

- `test_unsuspect_removes_menace_and_blocking_restriction` -- CR 701.60a
  Suspect a creature, then unsuspect it. Verify Menace is gone and it can block.

- `test_suspect_negative_can_attack` -- Verify suspected creatures CAN still attack (suspect restricts blocking, not attacking).

**Pattern**: Follow tests for Decayed in `crates/engine/tests/` and Goad tests.

### Step 5: Card Definition (later phase)

**Suggested card**: Frantic Scapegoat
- Mana cost: {R}
- Type: Creature -- Goat
- P/T: 1/1
- Oracle: Haste. When this creature enters, suspect it.
- **Simplification for card def**: Implement the first two abilities (Haste + ETB self-suspect). The third ability (transfer suspect to another creature) is complex and can be deferred.
- The ETB trigger uses `Effect::Suspect { target: EffectTarget::Source }`.

**Card lookup**: use `card-definition-author` agent.

### Step 6: Game Script (later phase)

**Suggested scenario**: "Suspected creature has menace and can't block"
1. Player A controls Frantic Scapegoat (enters, becomes suspected via ETB trigger).
2. Verify it has Menace in characteristics.
3. Player B attacks with a creature. Player A attempts to block with Frantic Scapegoat -- fails (suspected, can't block).
4. Next combat: Player A attacks with Frantic Scapegoat. Player B must assign 2+ blockers (Menace).

**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

- **Menace enforcement in combat.rs**: Menace is already enforced (line ~769). Suspected creatures will naturally have Menace in their layer-resolved chars, so existing Menace validation covers the evasion.
- **Layer system**: The Menace grant is added as a pre-layer-loop keyword insertion (like Changeling). This means it appears in base chars and can be stripped by Layer 6 effects. If the engine later implements full Humility, suspect's Menace grant will be correctly handled.
- **Can't block vs. Humility**: The can't-block check uses `is_suspected` directly (like Decayed uses its keyword). Under Humility, the keyword removal strips Menace but does NOT clear `is_suspected`, so the can't-block restriction persists. This is a known minor inaccuracy. Document with a TODO.
- **Copy effects**: `is_suspected` must NOT be copied when `copy.rs` copies an object. Since copy.rs copies characteristics (copiable values) and `is_suspected` is explicitly not a copiable value (CR 701.60b), it should be excluded. Verify that `copy.rs` does not blindly copy all GameObject fields.
- **Goad interaction**: A suspected creature that is also goaded must attack (goad) but can't block (suspect). No conflict -- both are attack/block restrictions, not contradictions.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/game_object.rs` | Add `is_suspected: bool` field |
| `crates/engine/src/cards/card_definition.rs` | Add `Effect::Suspect` and `Effect::Unsuspect` |
| `crates/engine/src/rules/events.rs` | Add `GameEvent::Suspected` and `GameEvent::Unsuspected` |
| `crates/engine/src/state/hash.rs` | Hash `is_suspected` |
| `crates/engine/src/state/builder.rs` | Initialize `is_suspected: false` |
| `crates/engine/src/state/mod.rs` | Initialize `is_suspected: false` in both `move_object_to_zone` sites |
| `crates/engine/src/effects/mod.rs` | Handle `Effect::Suspect` and `Effect::Unsuspect`; init in token creation |
| `crates/engine/src/rules/resolution.rs` | Initialize `is_suspected: false` in object creation |
| `crates/engine/src/rules/layers.rs` | Add Menace keyword grant when `is_suspected` |
| `crates/engine/src/rules/combat.rs` | Add suspected can't-block check (2 sites) |
| `crates/engine/tests/suspect.rs` | New test file with 7-8 tests |
