# Primitive Batch Plan: PB-J -- Copy/Redirect Spells

**Generated**: 2026-04-09
**Primitive**: Two new Effect variants (`CopySpellOnStack`, `ChangeTargets`) and one new TargetRequirement variant (`TargetSpellOrAbilityWithSingleTarget`)
**CR Rules**: 707.10, 707.10c, 115.7, 115.7a, 115.7d
**Cards affected**: 4 (4 existing fixes + 0 new)
**Dependencies**: None -- `copy_spell_on_stack()` in `rules/copy.rs` already exists (Storm/Cascade)
**Deferred items from prior PBs**: None directly relevant

## Primitive Specification

PB-J adds two DSL-level Effect variants that expose the existing spell-copy infrastructure
and new target-changing logic to card definitions:

1. **`Effect::CopySpellOnStack`** -- Copy a targeted spell on the stack N times. Wraps
   the existing `copy_spell_on_stack()` function in `rules/copy.rs`. The copy controller
   is the effect's controller. Targets are inherited from the original (choose-new-targets
   deferred to M10 interactive choice).

2. **`Effect::ChangeTargets`** -- Change the target(s) of a targeted spell or ability on
   the stack. Two sub-behaviors based on card text:
   - "Change the target" (Bolt Bend, Untimely Malfunction mode 2): CR 115.7a -- MUST
     change the target to another legal target. Single-target spells only.
   - "Choose new targets" (Deflecting Swat): CR 115.7d -- MAY change any/all targets.
     Multi-target spells are legal targets.

3. **`TargetRequirement::TargetSpellOrAbilityWithSingleTarget`** -- A new targeting
   requirement for spells like Bolt Bend that can only target a spell or ability on the
   stack that has exactly one target. Validated at cast time by checking
   `stack_object.targets.len() == 1`.

**Scope boundary**: Complete the Circuit's "When you next cast an instant or sorcery spell
this turn, copy that spell twice" requires a delayed spell-copy trigger mechanism that does
not exist in the engine. This is a separate primitive (delayed trigger on next spell cast)
that is OUT OF SCOPE for PB-J. Complete the Circuit will get its GrantFlash effect fixed
and a TODO for the delayed copy trigger.

## CR Rule Text

### CR 707.10 (Copying Spells)
> 707.10. To copy a spell, activated ability, or triggered ability means to put a copy of
> it onto the stack; a copy of a spell isn't cast and a copy of an activated ability isn't
> activated. A copy of a spell or ability copies both the characteristics of the spell or
> ability and all decisions made for it, including modes, targets, the value of X, and
> additional or alternative costs. (See rule 601, "Casting Spells.") Choices that are
> normally made on resolution are not copied. If an effect of the copy refers to objects
> used to pay its costs, it uses the objects used to pay the costs of the original spell or
> ability. A copy of a spell is owned by the player under whose control it was put on the
> stack. A copy of a spell or ability is controlled by the player under whose control it
> was put on the stack. A copy of a spell is itself a spell, even though it has no spell
> card associated with it. A copy of an ability is itself an ability.

### CR 707.10c (Choosing New Targets for Copies)
> 707.10c. Some effects copy a spell or ability and state that its controller may choose
> new targets for the copy. The player may leave any number of the targets unchanged, even
> if those targets would be illegal. If the player chooses to change some or all of the
> targets, the new targets must be legal. Once the player has decided what the copy's
> targets will be, the copy is put onto the stack with those targets.

### CR 115.7 (Changing Targets)
> 115.7. Some effects allow a player to change the target(s) of a spell or ability, and
> other effects allow a player to choose new targets for a spell or ability.

### CR 115.7a ("Change the target")
> 115.7a. If an effect allows a player to "change the target(s)" of a spell or ability,
> each target can be changed only to another legal target. If a target can't be changed to
> another legal target, the original target is unchanged, even if the original target is
> itself illegal by then. If all the targets aren't changed to other legal targets, none
> of them are changed.

### CR 115.7d ("Choose new targets")
> 115.7d. If an effect allows a player to "choose new targets" for a spell or ability,
> the player may leave any number of the targets unchanged, even if those targets would
> be illegal. If the player chooses to change some or all of the targets, the new targets
> must be legal and must not cause any unchanged targets to become illegal.

## Engine Changes

### Change 1: Add `TargetRequirement::TargetSpellOrAbilityWithSingleTarget`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to `TargetRequirement` enum (after `TargetCardInGraveyard`)
**Pattern**: Follow `TargetSpell` at line ~2212

```rust
/// "target spell or ability with a single target" (CR 115.7a)
/// Validates that the targeted stack object has exactly one target.
TargetSpellOrAbilityWithSingleTarget,
```

### Change 2: Validate `TargetSpellOrAbilityWithSingleTarget` in casting.rs

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add validation logic in `validate_target()` (near line ~5395)
**CR**: CR 115.7a -- "change the target of target spell or ability with a single target"

The new requirement must:
1. Check that the object is on the stack (same as `TargetSpell`)
2. Find the corresponding `StackObject` in `state.stack_objects`
3. Verify `stack_object.targets.len() == 1`
4. The spell/ability targeting itself with this requirement must NOT be a valid target
   (prevent self-targeting loops)

Also add the variant to the `match req` block that handles non-stack targets (~line 5429+)
to return `false`.

### Change 3: Add `Effect::CopySpellOnStack`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to `Effect` enum (after `LivingDeath`)

```rust
/// CR 707.10: Copy a spell on the stack. Creates `count` copies of the targeted
/// spell, each controlled by the effect's controller.
///
/// Uses the existing `copy_spell_on_stack()` from `rules/copy.rs`.
/// CR 707.10c: "choose new targets" is deterministic -- copies keep original targets.
/// Interactive target choice deferred to M10.
CopySpellOnStack {
    /// The spell to copy. Should be `EffectTarget::DeclaredTarget { index: N }`.
    target: EffectTarget,
    /// Number of copies to create (e.g., Fixed(2) for Complete the Circuit).
    count: EffectAmount,
},
```

### Change 4: Add `Effect::ChangeTargets`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to `Effect` enum (after `CopySpellOnStack`)

```rust
/// CR 115.7: Change or choose new targets for a spell or ability on the stack.
///
/// Two modes based on card text:
/// - `must_change: true` (CR 115.7a): "Change the target" -- MUST change to
///   another legal target. If no other legal target exists, target is unchanged.
///   Used by Bolt Bend, Untimely Malfunction.
/// - `must_change: false` (CR 115.7d): "Choose new targets" -- MAY change any
///   or all targets. Used by Deflecting Swat.
///
/// Deterministic fallback: retargets to the effect's controller (if legal).
/// If the controller is not a legal target, picks the first legal alternative
/// (smallest PlayerId/ObjectId). If no legal alternative exists, target unchanged.
ChangeTargets {
    /// The spell or ability whose targets to change.
    target: EffectTarget,
    /// CR 115.7a vs 115.7d: if true, the target MUST be changed to a different
    /// legal target (Bolt Bend). If false, the controller MAY choose new targets
    /// (Deflecting Swat).
    must_change: bool,
},
```

### Change 5: Dispatch `Effect::CopySpellOnStack` in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm after `Effect::LivingDeath` (around line ~4880)
**CR**: CR 707.10 -- spell copying

Logic:
1. Resolve `target` to get the stack object's ObjectId
2. Find the matching `StackObject` in `state.stack_objects`
3. Resolve `count` to a u32
4. Call `copy_spell_on_stack()` from `rules/copy.rs` N times
5. Collect and return all `GameEvent::SpellCopied` events

```rust
Effect::CopySpellOnStack { target, count } => {
    let targets = resolve_effect_target_list(state, target, ctx);
    let n = resolve_amount(state, count, ctx);
    for resolved in targets {
        if let ResolvedTarget::Object(stack_obj_id) = resolved {
            // Verify the object is on the stack
            let is_on_stack = state.stack_objects.iter().any(|s| s.id == stack_obj_id);
            if is_on_stack {
                for _ in 0..n {
                    match crate::rules::copy::copy_spell_on_stack(
                        state, stack_obj_id, ctx.controller, false,
                    ) {
                        Ok((_, evt)) => events.push(evt),
                        Err(_) => break,
                    }
                }
            }
        }
    }
}
```

### Change 6: Dispatch `Effect::ChangeTargets` in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm after `Effect::CopySpellOnStack`
**CR**: CR 115.7a / 115.7d -- changing targets

Logic:
1. Resolve `target` to get the stack object's ObjectId
2. Find the matching `StackObject` in `state.stack_objects`
3. If `must_change` (CR 115.7a): for each target in the spell's `targets`, find an
   alternative legal target. If no alternative exists, leave unchanged. Must change
   to a DIFFERENT target than the current one.
4. If `!must_change` (CR 115.7d): deterministic fallback -- leave all targets unchanged
   (interactive choice deferred to M10). This is legal per CR 115.7d ("may leave any
   number unchanged").
5. For the deterministic `must_change` case: pick the controller as the new target if
   legal; otherwise pick the first legal alternative (smallest ObjectId/PlayerId).
   If no legal alternative exists, target remains unchanged.
6. Emit `GameEvent::TargetsChanged` (new event, see Change 8).

The implementation needs access to target validation logic from `casting.rs`. Extract
or call the relevant `validate_target()` function. The `TargetRequirement` used to
validate new targets should be inferred from the original spell's target requirements
(stored on the `StackObject` or looked up from the card registry).

**Simplification for M9.4**: Since we lack interactive player choice, the deterministic
fallback for `must_change: true` should:
- Get the current target from `stack_object.targets[0]`
- Find all legal targets for that spell's `TargetRequirement`
- Filter out the current target
- Pick the smallest ObjectId/PlayerId as the new target
- If no valid alternative, leave unchanged

For `must_change: false`, the deterministic fallback leaves targets unchanged (the
player "chose" to keep them).

### Change 7: Hash new Effect variants in hash.rs

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms after `Effect::AddManaMatchingType` (discriminant 81)

```rust
// PB-J: CopySpellOnStack (discriminant 82)
Effect::CopySpellOnStack { target, count } => {
    82u8.hash_into(hasher);
    target.hash_into(hasher);
    count.hash_into(hasher);
}
// PB-J: ChangeTargets (discriminant 83)
Effect::ChangeTargets { target, must_change } => {
    83u8.hash_into(hasher);
    target.hash_into(hasher);
    must_change.hash_into(hasher);
}
```

### Change 8: Hash new TargetRequirement variant in hash.rs

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm after `TargetCardInGraveyard` (discriminant 14)

```rust
// PB-J: TargetSpellOrAbilityWithSingleTarget (discriminant 15)
TargetRequirement::TargetSpellOrAbilityWithSingleTarget => 15u8.hash_into(hasher),
```

### Change 9: Add `GameEvent::TargetsChanged` event

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add new event variant

```rust
/// CR 115.7: Targets of a spell or ability were changed.
TargetsChanged {
    stack_object_id: ObjectId,
    old_targets: Vec<SpellTarget>,
    new_targets: Vec<SpellTarget>,
},
```

### Change 10: Hash new GameEvent variant in hash.rs

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `GameEvent::TargetsChanged` (check current max discriminant
for GameEvent and add +1).

### Exhaustive Match Updates

Files requiring new match arms:

| File | Match expression | Action |
|------|-----------------|--------|
| `crates/engine/src/effects/mod.rs` | `match effect` (L222) | Add dispatch for `CopySpellOnStack` + `ChangeTargets` |
| `crates/engine/src/state/hash.rs` | `Effect` hash (L4674+) | Add discriminants 82, 83 |
| `crates/engine/src/state/hash.rs` | `TargetRequirement` hash (L4038+) | Add discriminant 15 |
| `crates/engine/src/state/hash.rs` | `GameEvent` hash | Add discriminant for `TargetsChanged` |
| `crates/engine/src/rules/casting.rs` | `validate_target()` (L5369+) | Add arm for `TargetSpellOrAbilityWithSingleTarget` |

**Note**: `Effect` is NOT exhaustively matched in `view_model.rs`, `stack_view.rs`, or
other tool files -- those only reference specific variants. No tool updates needed.

## Card Definition Fixes

### bolt_bend.rs
**Oracle text**: "This spell costs {3} less to cast if you control a creature with power 4 or greater. Change the target of target spell or ability with a single target."
**Current state**: TODO -- cost reduction works, abilities vec is empty
**Fix**: Add `AbilityDefinition::Spell` with `Effect::ChangeTargets { target: DeclaredTarget { index: 0 }, must_change: true }` and `targets: vec![TargetRequirement::TargetSpellOrAbilityWithSingleTarget]`.

### deflecting_swat.rs
**Oracle text**: "If you control a commander, you may cast this spell without paying its mana cost. You may choose new targets for target spell or ability."
**Current state**: TODO -- has Spell ability with `Effect::Nothing` and `TargetSpell` target
**Fix**:
1. Add `AbilityDefinition::AltCastAbility { kind: AltCostKind::CommanderFreeCast, cost: ManaCost::default(), details: None }` (same pattern as Deadly Rollick)
2. Change effect to `Effect::ChangeTargets { target: DeclaredTarget { index: 0 }, must_change: false }`
3. Keep `targets: vec![TargetRequirement::TargetSpell]` (Deflecting Swat can target ANY spell or ability, not just single-target ones)

### untimely_malfunction.rs
**Oracle text**: "Choose one -- Destroy target artifact. / Change the target of target spell or ability with a single target. / One or two target creatures can't block this turn."
**Current state**: Mode 1 is `Effect::Sequence(vec![])` with TODO
**Fix**:
1. Add `TargetRequirement::TargetSpellOrAbilityWithSingleTarget` to the targets vec for mode 1
2. Change mode 1 effect to `Effect::ChangeTargets { target: DeclaredTarget { index: 1 }, must_change: true }`
3. The target index for mode 1 should be 1 (mode 0 uses index 0 for the artifact target). Verify the modal target assignment convention.

### complete_the_circuit.rs
**Oracle text**: "Convoke. You may cast sorcery spells this turn as though they had flash. When you next cast an instant or sorcery spell this turn, copy that spell twice. You may choose new targets for the copies."
**Current state**: Has Convoke keyword and GrantFlash spell effect. Missing the delayed copy trigger.
**Fix**: The GrantFlash effect is correct. The "When you next cast an instant or sorcery spell this turn, copy that spell twice" requires a delayed trigger on next spell cast, which is a separate engine primitive not yet built. **Leave the current implementation as-is and update the TODO comment to reference PB-J partial completion + the missing delayed spell-copy trigger primitive.**

## New Card Definitions

None. All affected cards already have card def files.

## Unit Tests

**File**: `crates/engine/tests/copy_redirect.rs` (new file)
**Tests to write**:

- `test_copy_spell_on_stack_basic` -- CR 707.10: Cast a Lightning Bolt, then resolve a
  CopySpellOnStack effect targeting it. Verify two Lightning Bolts on the stack (original
  + copy). Verify the copy has `is_copy: true` and same targets as original.

- `test_copy_spell_on_stack_twice` -- CR 707.10: Copy a spell 2 times. Verify 3 total
  stack objects (original + 2 copies). Copies resolve before original (LIFO).

- `test_change_targets_must_change` -- CR 115.7a: Cast Lightning Bolt targeting Player A.
  Cast Bolt Bend targeting the Lightning Bolt. Resolve Bolt Bend. Verify Lightning Bolt's
  target changed to a different legal target. Bolt Bend ruling: "You must change the target
  if possible."

- `test_change_targets_no_alternative` -- CR 115.7a: Set up a scenario where the current
  target is the only legal target. Resolve ChangeTargets with `must_change: true`. Verify
  target is unchanged (ruling: "If there are no legal targets to choose from, the target
  isn't changed").

- `test_change_targets_may_choose_new` -- CR 115.7d: Cast a spell targeting Player A.
  Cast Deflecting Swat targeting it. Resolve. Since deterministic fallback leaves targets
  unchanged for `must_change: false`, verify targets are unchanged.

- `test_target_spell_or_ability_single_target_validation` -- Verify that
  `TargetSpellOrAbilityWithSingleTarget` rejects spells with 0 or 2+ targets at cast time.

- `test_target_spell_or_ability_single_target_accepts` -- Verify that a spell with exactly
  one target is a valid target for `TargetSpellOrAbilityWithSingleTarget`.

- `test_bolt_bend_integration` -- Set up a game with Bolt Bend and Lightning Bolt. Cast
  Lightning Bolt targeting an opponent. Cast Bolt Bend targeting the Lightning Bolt. Verify
  the Lightning Bolt's target changes.

**Pattern**: Follow tests in `crates/engine/tests/mass_reanimate.rs` for effect test structure.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (3 of 4; Complete the Circuit partial)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in Bolt Bend, Deflecting Swat, Untimely Malfunction
- [ ] Complete the Circuit TODO updated to reference remaining gap (delayed copy trigger)

## Risks & Edge Cases

- **ChangeTargets deterministic fallback for `must_change: true`**: The engine must pick a
  new target automatically. Strategy: retarget to the effect controller if legal, else
  smallest ObjectId/PlayerId among legal alternatives. This is imperfect but deterministic.
  Interactive choice (M10) will replace this.

- **Untimely Malfunction modal target indexing**: Mode 1 needs its own target slot. The
  modal target assignment convention must be verified -- each mode may use different
  declared target indices. Check how other modal spells (Abzan Charm, etc.) assign target
  indices per mode.

- **TargetRequirement validation for abilities on the stack**: The new
  `TargetSpellOrAbilityWithSingleTarget` must handle both spells AND abilities (triggered
  and activated) on the stack. Abilities are also StackObjects with `targets` vecs.

- **Complete the Circuit scope**: The delayed "when you next cast" trigger is a significant
  engine feature (requires a new `DelayedTrigger` variant or a per-player `pending_spell_copy`
  state field). This is explicitly deferred. The card will have a partial implementation
  (GrantFlash works, copy does not).

- **Self-targeting prevention**: A spell using `TargetSpellOrAbilityWithSingleTarget`
  should not be able to target itself (Bolt Bend targeting its own stack entry). The
  validation in casting.rs should exclude the casting spell's own stack object ID.

- **Target legality for ChangeTargets**: The implementation needs to know the original
  spell's `TargetRequirement` to validate that the new target is legal. This information
  is NOT stored on `StackObject` -- it's on the `CardDefinition`. The implementation must
  look up the card registry to find the requirement, or use a simplified approach (any
  legal target of the same kind: player targets stay players, object targets stay objects
  on the battlefield). The simplified approach is safer for M9.4.

- **Interaction with `cant_be_countered`**: ChangeTargets does NOT counter the spell --
  it redirects it. This is orthogonal to counter protection.

- **Triggered abilities as targets**: Stack objects with `StackObjectKind::TriggeredAbility`
  also have targets and can be targeted by Bolt Bend. The validation must handle all SOK
  variants, not just `Spell`.
