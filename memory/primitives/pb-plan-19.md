# Primitive Batch Plan: PB-19 -- Mass Destroy / Board Wipes

**Generated**: 2026-03-19
**Primitive**: `Effect::DestroyAll`, `Effect::ExileAll`, `EffectAmount::LastEffectCount`
**CR Rules**: 701.8 (Destroy), 702.12 (Indestructible), 701.19 (Regenerate), 406.2 (Exile)
**Cards affected**: 12 (7 existing fixes + 5 new)
**Dependencies**: None
**Deferred items from prior PBs**: None

## Primitive Specification

The engine can destroy/exile individual permanents via `Effect::DestroyPermanent` and
`Effect::ExileObject`, and both support `EffectTarget::AllCreatures` / `AllPermanentsMatching`
for mass targeting. However:

1. **No destroyed-count tracking**: Cards like Fumigate ("gain 1 life for each creature
   destroyed this way") and Bane of Progress ("put a +1/+1 counter for each permanent
   destroyed this way") need the count of permanents actually destroyed (after indestructible
   and regeneration checks). The current ForEach pattern cannot track this.

2. **No regeneration prevention**: Wrath of God and Damnation say "They can't be regenerated"
   but the existing `DestroyPermanent` has no mechanism to skip regeneration checks.

3. **Controller filter missing on `AllPermanentsMatching`**: The `resolve_effect_target_list`
   function for `AllPermanentsMatching` does NOT apply `filter.controller` (line 3223-3232 of
   `effects/mod.rs`). Cards like Ruinous Ultimatum ("destroy all nonland permanents your
   opponents control") would incorrectly hit the caster's permanents too.

This batch adds:
- `Effect::DestroyAll { filter, cant_be_regenerated }` -- destroys all battlefield permanents
  matching the filter, respects indestructible/regeneration/umbra armor, and stores the count
  of actually-destroyed permanents in `ctx.last_effect_count`.
- `Effect::ExileAll { filter }` -- exiles all battlefield permanents matching the filter,
  stores count in `ctx.last_effect_count`.
- `EffectAmount::LastEffectCount` -- reads `ctx.last_effect_count` for follow-up effects.
- `last_effect_count: u32` field on `EffectContext`.
- Fix: `AllPermanentsMatching` controller filter enforcement.

## CR Rule Text

### CR 701.8 -- Destroy
> 701.8a To destroy a permanent, move it from the battlefield to its owner's graveyard.
>
> 701.8b The only ways a permanent can be destroyed are as a result of an effect that
> uses the word "destroy" or as a result of the state-based actions that check for lethal
> damage (see rule 704.5g) or damage from a source with deathtouch (see rule 704.5h). If
> a permanent is put into its owner's graveyard for any other reason, it hasn't been
> "destroyed."
>
> 701.8c A regeneration effect replaces a destruction event. See rule 701.19, "Regenerate."

### CR 702.12 -- Indestructible
> 702.12a Indestructible is a static ability.
>
> 702.12b A permanent with indestructible can't be destroyed. Such permanents aren't
> destroyed by lethal damage, and they ignore the state-based action that checks for
> lethal damage (see rule 704.5g).

### CR 701.19 -- Regenerate
> 701.19c Neither activating an ability that creates a regeneration shield nor casting a
> spell that creates a regeneration shield is the same as regenerating a permanent. Effects
> that say that a permanent can't be regenerated don't preclude such abilities from being
> activated or such spells from being cast; rather, they cause regeneration shields to not
> be applied.

### CR 406.2 -- Exile
> 406.2 To exile an object is to put it into the exile zone from whatever zone it's
> currently in.

## Engine Changes

### Change 1: Add `last_effect_count` to `EffectContext`

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add `pub last_effect_count: u32` field to `EffectContext` struct (line ~95).
Initialize to 0 in `EffectContext::new()` (line ~100) and `new_with_kicker()`.

### Change 2: Add `EffectAmount::LastEffectCount` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `LastEffectCount` variant to `enum EffectAmount` (after `CounterCount`, line ~1480).
Doc comment: `/// The count of permanents affected by the preceding DestroyAll/ExileAll (stored in ctx).`

### Change 3: Wire `EffectAmount::LastEffectCount` in resolver

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm in `resolve_amount()` (line ~3369):
```rust
EffectAmount::LastEffectCount => ctx.last_effect_count as i32,
```

### Change 4: Add `Effect::DestroyAll` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `enum Effect` (after `DestroyPermanent`, around line ~1006):
```rust
/// CR 701.8: Destroy all permanents on the battlefield matching the filter.
/// Respects indestructible (CR 702.12), regeneration (CR 701.19), and umbra armor.
/// Stores the count of actually-destroyed permanents in ctx.last_effect_count
/// for use by EffectAmount::LastEffectCount (e.g. Fumigate, Bane of Progress).
DestroyAll {
    filter: TargetFilter,
    /// CR 701.19c: If true, regeneration shields are not applied.
    /// Used by Wrath of God, Damnation, etc.
    cant_be_regenerated: bool,
},
```

### Change 5: Add `Effect::ExileAll` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `enum Effect` (after `DestroyAll`):
```rust
/// CR 406.2: Exile all permanents on the battlefield matching the filter.
/// Stores the count of actually-exiled permanents in ctx.last_effect_count.
ExileAll {
    filter: TargetFilter,
},
```

### Change 6: Dispatch `Effect::DestroyAll` in execute_effect

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm after the `Effect::DestroyPermanent` block (after line ~776).

The implementation iterates all battlefield permanents matching the filter (respecting
controller via `filter.controller` against `ctx.controller`, phased-in check). For each:
1. Check indestructible (CR 702.12) -- skip if true
2. If NOT `cant_be_regenerated`, check regeneration shield (CR 701.19) -- apply if present
3. Check umbra armor (CR 702.89a) -- apply if present
4. Check zone-change replacement effects (CR 614)
5. Move to graveyard, emit CreatureDied/PermanentDestroyed events
6. Increment a local `destroyed_count`

After the loop, set `ctx.last_effect_count = destroyed_count`.

**Pattern**: Reuse the per-permanent destruction logic from `Effect::DestroyPermanent` (lines
615-776). Extract the indestructible/regeneration/umbra/replacement/move logic into a helper
function `destroy_single_permanent(state, id, cant_be_regenerated, events) -> bool` that
returns true if the permanent was actually destroyed. Both `DestroyPermanent` and `DestroyAll`
call this helper.

**CR**: 701.8a (destroy = move to graveyard), 702.12b (indestructible), 701.19c (regeneration
prevention)

### Change 7: Dispatch `Effect::ExileAll` in execute_effect

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm after the `Effect::DestroyAll` block.

Iterates all battlefield permanents matching the filter (with controller check). For each:
1. Check zone-change replacement effects (CR 614)
2. Move to exile zone
3. Emit ObjectExiled events
4. Increment `exiled_count`

After the loop, set `ctx.last_effect_count = exiled_count`.

**Pattern**: Reuse logic from `Effect::ExileObject` (lines 778-870).

### Change 8: Fix `AllPermanentsMatching` controller filter

**File**: `crates/engine/src/effects/mod.rs`
**Action**: In `resolve_effect_target_list_indexed()`, the `AllPermanentsMatching` arm
(line ~3223) does NOT check `filter.controller`. Add controller check:
```rust
EffectTarget::AllPermanentsMatching(filter) => state
    .objects
    .iter()
    .filter(|(_, obj)| {
        obj.zone == ZoneId::Battlefield
            && obj.is_phased_in()
            && matches_filter(&obj.characteristics, filter)
            && match filter.controller {
                TargetController::Any => true,
                TargetController::You => obj.controller == ctx.controller,
                TargetController::Opponent => obj.controller != ctx.controller,
            }
    })
    .map(|(&id, _)| (None, ResolvedTarget::Object(id)))
    .collect(),
```

Note: This fix also benefits existing uses of `AllPermanentsMatching` beyond PB-19.
Verify no existing card defs rely on the current (broken) behavior.

### Change 9: Exhaustive match updates

Files requiring new match arms for the new variants:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/state/hash.rs` | `Effect` HashInto | L4102 | Add `DestroyAll` (54u8) and `ExileAll` (55u8) arms |
| `crates/engine/src/state/hash.rs` | `EffectAmount` HashInto | L3782 | Add `LastEffectCount` (9u8) arm |
| `crates/engine/src/effects/mod.rs` | `execute_effect` | L175 | Add dispatch for `DestroyAll` and `ExileAll` |
| `crates/engine/src/effects/mod.rs` | `resolve_amount` | L3369 | Add `LastEffectCount` arm |

No exhaustive matches in `tools/replay-viewer/` or `tools/tui/` for `Effect` or `EffectAmount`.

### Change 10: Export new types from helpers.rs

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: No new types need exporting. `TargetFilter`, `Effect`, `EffectAmount` are already
available in card defs via `use crate::cards::helpers::*`.

Verify `EffectAmount` is re-exported:
```
Grep pattern="EffectAmount" path="crates/engine/src/cards/helpers.rs"
```

## Card Definition Fixes

### 1. wrath_of_god.rs
**Oracle text**: "Destroy all creatures. They can't be regenerated."
**Current state**: Uses `ForEach { EachCreature, DestroyPermanent }` -- works but no regeneration prevention.
**Fix**: Replace with:
```rust
effect: Effect::DestroyAll {
    filter: TargetFilter {
        has_card_type: Some(CardType::Creature),
        ..Default::default()
    },
    cant_be_regenerated: true,
},
```
Remove the `ForEach` wrapper.

### 2. damnation.rs
**Oracle text**: "Destroy all creatures. They can't be regenerated."
**Current state**: Same as Wrath of God.
**Fix**: Same as Wrath of God -- `DestroyAll` with `cant_be_regenerated: true`.

### 3. supreme_verdict.rs
**Oracle text**: "This spell can't be countered. Destroy all creatures."
**Current state**: Uses `ForEach { EachCreature, DestroyPermanent }`.
**Fix**: Replace with `DestroyAll { filter: creature, cant_be_regenerated: false }`.
Keep `cant_be_countered: true`.

### 4. path_of_peril.rs
**Oracle text**: "Cleave {4}{W}{B}. Destroy all creatures [with mana value 2 or less]."
**Current state**: Stale TODO -- claims max_mana_value is missing, but `TargetFilter.max_cmc`
exists since PB-17.
**Fix**: In the `if_false` branch, replace the current filter with:
```rust
if_false: Box::new(Effect::DestroyAll {
    filter: TargetFilter {
        has_card_type: Some(CardType::Creature),
        max_cmc: Some(2),
        ..Default::default()
    },
    cant_be_regenerated: false,
}),
```
Also update the `if_true` branch to use `DestroyAll`. Remove TODO comment.

### 5. sublime_exhalation.rs
**Oracle text**: "Undaunted. Destroy all creatures."
**Current state**: Uses `DestroyPermanent { target: AllCreatures }`.
**Fix**: Replace with `DestroyAll { filter: creature, cant_be_regenerated: false }`.

### 6. final_showdown.rs
**Oracle text**: Mode 2: "Destroy all creatures."
**Current state**: Uses `DestroyPermanent { target: AllCreatures }` in mode 2.
**Fix**: Replace mode 2 with `DestroyAll { filter: creature, cant_be_regenerated: false }`.

### 7. scavenger_grounds.rs
**Oracle text**: "{2}, {T}, Sacrifice a Desert: Exile all cards from all graveyards."
**Current state**: TODO -- says "blocked on exile-all-graveyards effect".
**Fix**: The effect IS expressible with existing ForEach + ExileObject (same pattern as
Rest in Peace). Add the activated ability:
```rust
AbilityDefinition::Activated {
    cost: Cost::Sequence(vec![
        Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
        Cost::Tap,
        Cost::Sacrifice(TargetFilter {
            has_subtype: Some(SubType::from("Desert")),
            ..Default::default()
        }),
    ]),
    effect: Effect::ForEach {
        over: ForEachTarget::EachCardInAllGraveyards,
        effect: Box::new(Effect::ExileObject {
            target: EffectTarget::DeclaredTarget { index: 0 },
        }),
    },
    timing_restriction: None,
    targets: vec![],
},
```
Remove TODO comment. Note: "Sacrifice a Desert" can sacrifice itself since it has
subtype Desert.

## New Card Definitions

### 8. vanquish_the_horde.rs
**Oracle text**: "This spell costs {1} less to cast for each creature on the battlefield. Destroy all creatures."
**CardDefinition sketch**:
- mana_cost: {6}{W}{W}
- types: Sorcery
- abilities: cost reduction static (PermanentCount creatures), DestroyAll { creature filter, cant_be_regenerated: false }
- Cost reduction: `AbilityDefinition::CostReduction` with `PermanentCount { filter: creature, controller: EachPlayer }`

### 9. fumigate.rs
**Oracle text**: "Destroy all creatures. You gain 1 life for each creature destroyed this way."
**CardDefinition sketch**:
- mana_cost: {3}{W}{W}
- types: Sorcery
- abilities: Spell with `Sequence([DestroyAll { creature, cant_be_regenerated: false }, GainLife { Controller, LastEffectCount }])`

### 10. bane_of_progress.rs
**Oracle text**: "When this creature enters, destroy all artifacts and enchantments. Put a +1/+1 counter on this creature for each permanent destroyed this way."
**CardDefinition sketch**:
- mana_cost: {4}{G}{G}
- types: Creature -- Elemental 2/2
- abilities: ETB Triggered with `Sequence([DestroyAll { filter: has_card_types: [Artifact, Enchantment], cant_be_regenerated: false }, AddCounter { Source, PlusOnePlusOne, LastEffectCount }])`
- Note: `has_card_types` uses OR semantics -- must match Artifact OR Enchantment. The filter
  needs `has_card_types: vec![CardType::Artifact, CardType::Enchantment]` with the existing
  OR-semantics behavior of `has_card_types`. Verify this works (line 1567-1571): "must have at
  least one of these types" -- correct for "artifacts and enchantments".

### 11. ruinous_ultimatum.rs
**Oracle text**: "Destroy all nonland permanents your opponents control."
**CardDefinition sketch**:
- mana_cost: {R}{R}{W}{W}{W}{B}{B}
- types: Sorcery
- abilities: Spell with `DestroyAll { filter: { non_land: true, controller: Opponent }, cant_be_regenerated: false }`

### 12. cyclonic_rift.rs
**Oracle text**: "Return target nonland permanent you don't control to its owner's hand. Overload {6}{U}."
**CardDefinition sketch**:
- mana_cost: {1}{U}
- types: Instant
- abilities:
  - Keyword: Overload
  - AltCostKind: Overload { cost: {6}{U} }
  - Spell with Conditional on WasOverloaded:
    - if_true: ForEach { EachPermanentMatching({ non_land: true, controller: Opponent }), MoveZone { DeclaredTarget{0}, Hand } }
    - if_false: MoveZone { DeclaredTarget{0}, Hand }
  - targets: [TargetPermanentWithFilter({ non_land: true, controller: Opponent })]
- Note: Overload changes "target" to "each" (CR 702.96). Existing overload infra handles
  the conditional target-vs-each pattern.

## Unit Tests

**File**: `crates/engine/tests/mass_destroy.rs` (new file)
**Tests to write**:
- `test_destroy_all_creatures_basic` -- CR 701.8a: DestroyAll with creature filter destroys all creatures, non-creatures survive
- `test_destroy_all_respects_indestructible` -- CR 702.12b: indestructible creature survives DestroyAll
- `test_destroy_all_cant_be_regenerated` -- CR 701.19c: cant_be_regenerated=true bypasses regen shields
- `test_destroy_all_allows_regeneration` -- CR 701.19: cant_be_regenerated=false allows regen shield to save
- `test_destroy_all_count_tracking` -- DestroyAll sets last_effect_count, GainLife reads it via LastEffectCount (Fumigate pattern)
- `test_destroy_all_filtered_by_cmc` -- Austere Command pattern: creatures with max_cmc <= 3
- `test_destroy_all_nonland_opponents` -- Ruinous Ultimatum: non_land + controller: Opponent
- `test_exile_all_basic` -- CR 406.2: ExileAll exiles matching permanents
- `test_exile_all_count_tracking` -- ExileAll sets last_effect_count
- `test_all_permanents_matching_controller_filter` -- Fix verification: AllPermanentsMatching respects controller field
- `test_destroy_all_triggers_dies` -- Destroyed creatures trigger "when dies" abilities
- `test_destroy_all_multiplayer` -- 4-player game: all players' creatures destroyed simultaneously

**Pattern**: Follow tests in `crates/engine/tests/card_def_fixes.rs` and `crates/engine/tests/effects_basic.rs` for builder setup.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved
- [ ] New card defs authored (5 new: Vanquish, Fumigate, Bane, Ruinous, Cyclonic Rift)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs

## Risks & Edge Cases

1. **Helper extraction from DestroyPermanent**: The per-permanent destruction logic (indestructible check, regeneration, umbra armor, replacement effects, zone move, event emission) spans ~160 lines. Extracting it into a shared helper is the right approach but requires careful parameter threading (state, events, ctx references). If the helper signature becomes unwieldy, consider a struct to hold the shared context.

2. **Controller filter fix on AllPermanentsMatching may break existing card defs**: Verify that no existing card def relies on `AllPermanentsMatching` with a non-Any controller field but expects the filter to be ignored. Grep for `AllPermanentsMatching` usage in card defs to check. Current uses: Path of Peril (controller: Any/default), Camellia (controller: You). Camellia's `controller: You` was presumably intended to work but was silently a no-op. The fix makes it work correctly.

3. **Simultaneous destruction vs sequential**: CR does not define "destroy all" as a special action. Each permanent is destroyed individually (each checks indestructible, regeneration independently). However, all destructions happen as part of a single spell resolution, so all resulting "when dies" triggers should be collected and put on the stack together (APNAP order). The existing trigger infrastructure handles this -- `check_triggers` is called after all effects resolve.

4. **Bane of Progress self-reference**: "Put a +1/+1 counter on this creature for each permanent destroyed this way." If Bane of Progress is also an artifact or enchantment, it would be targeted for destruction but is indestructible by default? No -- Bane is a Creature, not an Artifact/Enchantment, so the filter won't match it. No self-reference issue.

5. **Cyclonic Rift overloaded timing**: The overloaded version has no targets (CR 702.96b) and resolves as "each nonland permanent you don't control." This means it can't be countered by single-target counterspells (like Negate targeting it as a spell is fine, but protection from blue doesn't prevent the bounce). Ensure the overload conditional correctly removes the targeting requirement.

6. **Toxic Deluge is NOT in this batch**: Despite being a board wipe in the card universe, Toxic Deluge ("all creatures get -X/-X until end of turn") is a mass continuous effect, not a DestroyAll. It requires a different primitive (mass temporary P/T modification as a spell effect). It will need to be authored separately, potentially as part of Phase 2 authoring when continuous-effect-from-spell support is more mature.

7. **EffectContext is passed as `&mut`**: The `last_effect_count` field must be set within the `DestroyAll`/`ExileAll` arms where `ctx` is already `&mut EffectContext`. Verify the `execute_effect` signature passes `ctx` as mutable (it does -- line 172: `ctx: &mut EffectContext`).
