# Ability Plan: Prowess

**Generated**: 2026-02-25
**CR**: 702.108
**Priority**: P1
**Similar abilities studied**: Ward (CR 702.21) -- triggered ability wired through `check_triggers` + `flush_pending_triggers` in `crates/engine/src/rules/abilities.rs`, enrichment at `crates/engine/src/state/builder.rs:L336-372`, tests in `crates/engine/tests/ward.rs`

## CR Rule Text

702.108. Prowess

702.108a Prowess is a triggered ability. "Prowess" means "Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn."

702.108b If a creature has multiple instances of prowess, each triggers separately.

## Key Edge Cases

- **Noncreature filter (Monastery Swiftspear ruling 2014-09-20)**: "Any spell you cast that doesn't have the type creature will cause prowess to trigger. If a spell has multiple types, and one of those types is creature (such as an artifact creature), casting it won't cause prowess to trigger. Playing a land also won't cause prowess to trigger." This means: check `card_types` of the cast spell; if `CardType::Creature` is present, prowess does NOT trigger regardless of other types.
- **Trigger independence (Monastery Swiftspear ruling 2014-09-20)**: "Once it triggers, prowess isn't connected to the spell that caused it to trigger. If that spell is countered, prowess will still resolve." The trigger goes on the stack on top of the spell; it resolves before the spell. If the spell is countered, the +1/+1 still applies.
- **Multiple instances (CR 702.108b)**: If a creature has multiple instances of prowess, each triggers separately. A creature with two prowess would get +2/+2. This is naturally handled because each `Prowess` keyword entry generates a separate `TriggeredAbilityDef` during enrichment.
- **Prowess resolves on the stack**: Like all triggered abilities, the prowess trigger goes on the stack above the spell that caused it. It resolves first (giving the +1/+1) before the triggering spell resolves. Prowess is NOT connected to the spell after triggering.
- **Controller check**: "Whenever **you** cast" means the prowess creature's controller must be the spell caster. Opponents casting noncreature spells do NOT trigger your prowess creatures.
- **Storm/cascade copies**: Storm copies are NOT cast (CR 702.40c), so they do NOT trigger prowess. The engine emits `SpellCopied` (not `SpellCast`) for storm copies in `create_storm_copies` (`crates/engine/src/rules/copy.rs`), so this is already correct. Cascade-cast spells ARE cast and emit `SpellCast` (`copy.rs:L347`), so they correctly DO trigger prowess.
- **Monastery Mentor interaction (ruling 2014-11-24)**: "The spell that causes Monastery Mentor's second ability to trigger will not cause the prowess ability of the Monk token that's created to trigger." The token enters AFTER the trigger was already checked. This is handled naturally because the token is not on the battlefield when `check_triggers` runs for the `SpellCast` event.
- **Multiplayer**: In Commander, each player's prowess creatures only trigger on their own noncreature spells. No special multiplayer considerations beyond the standard "you" = controller check.

## Current State (from ability-wip.md)

- [x] Step 1: Enum variant -- exists at `crates/engine/src/state/types.rs:L123` (`KeywordAbility::Prowess`)
- [x] Hash -- exists at `crates/engine/src/state/hash.rs:L282` (discriminant `16u8`)
- [ ] Step 2: Rule enforcement (trigger dispatch)
- [ ] Step 3: Trigger wiring (triggered ability effect)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant (DONE)

**File**: `crates/engine/src/state/types.rs`
**Status**: `KeywordAbility::Prowess` exists at line 123.
**Hash**: `16u8` discriminant at `crates/engine/src/state/hash.rs:L282`.
**No further action needed.**

### Step 2: Add `EffectFilter::Source` Variant

**File**: `crates/engine/src/state/continuous_effect.rs`
**Action**: Add a new `EffectFilter::Source` variant after `DeclaredTarget` (after line 97). This variant means "applies to the source object of the effect" and is resolved at `ApplyContinuousEffect` execution time to `SingleObject(ctx.source)`.

**Rationale**: Prowess's effect is "**this creature** gets +1/+1 until end of turn." The source of the triggered ability IS the prowess creature. At definition time (in the `TriggeredAbilityDef`), we don't know the creature's ObjectId, so we use a placeholder `EffectFilter::Source` that resolves dynamically.

**Pattern**: Follow `DeclaredTarget` at lines 88-97 of `continuous_effect.rs`.

Add after line 97:
```rust
/// Applies to the source object of the effect (e.g., "this creature gets +1/+1").
///
/// Resolved at `ApplyContinuousEffect` execution time to `SingleObject(ctx.source)`.
/// Used by keyword abilities like Prowess where the effect targets the source creature.
Source,
```

**Hash**: Update `crates/engine/src/state/hash.rs` at the `impl HashInto for EffectFilter` block (line 521-549). Add after the `DeclaredTarget` arm (line 544-547):
```rust
EffectFilter::Source => 12u8.hash_into(hasher),
```

**ApplyContinuousEffect resolution**: Update `crates/engine/src/effects/mod.rs` in the `Effect::ApplyContinuousEffect` handler (line 881). In the `resolved_filter` match block (lines 884-897), add a new arm before the `other` catch-all:
```rust
CEFilter::Source => CEFilter::SingleObject(ctx.source),
```

### Step 3: Add `TriggerEvent::ControllerCastsNoncreatureSpell`

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add a new variant to `TriggerEvent` enum (after `SelfBecomesTargetByOpponent` at line 117):
```rust
/// CR 702.108a: Triggers when the controller of this permanent casts a
/// noncreature spell. Used by the Prowess keyword. The noncreature check
/// and controller-match are verified at trigger-collection time in
/// `rules/abilities.rs`.
ControllerCastsNoncreatureSpell,
```

**Hash**: Update `crates/engine/src/state/hash.rs` at the `impl HashInto for TriggerEvent` block (lines 868-877). Add after `SelfBecomesTargetByOpponent` (line 876):
```rust
// CR 702.108a: Prowess trigger -- discriminant 7
TriggerEvent::ControllerCastsNoncreatureSpell => 7u8.hash_into(hasher),
```

### Step 4: Wire Prowess Trigger Dispatch in `check_triggers`

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `check_triggers` function, expand the `GameEvent::SpellCast` match arm (lines 314-323) to also dispatch `ControllerCastsNoncreatureSpell`.

**CR**: 702.108a -- "Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn."

The current code at lines 314-323:
```rust
GameEvent::SpellCast { .. } => {
    collect_triggers_for_event(
        state,
        &mut triggers,
        TriggerEvent::AnySpellCast,
        None,
        None,
    );
}
```

Replace with:
```rust
GameEvent::SpellCast { player, source_object_id, .. } => {
    // AnySpellCast: fires on all permanents that watch for spell casts.
    collect_triggers_for_event(
        state,
        &mut triggers,
        TriggerEvent::AnySpellCast,
        None,
        None,
    );

    // CR 702.108a: Prowess -- "Whenever you cast a noncreature spell."
    // Check if the cast spell is noncreature by inspecting the source object's
    // card types. Only fire if the spell lacks CardType::Creature.
    let is_noncreature = state
        .objects
        .get(source_object_id)
        .map(|obj| {
            !obj.characteristics
                .card_types
                .contains(&crate::state::types::CardType::Creature)
        })
        .unwrap_or(false);

    if is_noncreature {
        // Collect triggers only for permanents controlled by the caster.
        // Prowess says "whenever YOU cast" -- only the controller's creatures trigger.
        let prowess_sources: Vec<ObjectId> = state
            .objects
            .values()
            .filter(|obj| {
                obj.zone == ZoneId::Battlefield && obj.controller == *player
            })
            .map(|obj| obj.id)
            .collect();

        for obj_id in prowess_sources {
            collect_triggers_for_event(
                state,
                &mut triggers,
                TriggerEvent::ControllerCastsNoncreatureSpell,
                Some(obj_id),
                None,
            );
        }
    }
}
```

**Note on imports**: Verify `CardType` is accessible. If not, add `use crate::state::types::CardType;` at the top of `abilities.rs`. The code above uses the fully-qualified path to be safe.

**Design choice**: Pre-filter by controller rather than passing a player filter into `collect_triggers_for_event`. This keeps the existing function signature unchanged and localizes the change to the `SpellCast` branch.

### Step 5: Auto-Generate Prowess `TriggeredAbilityDef` from Keyword

**File**: `crates/engine/src/state/builder.rs`
**Location**: Lines 336-372 (the Ward keyword-to-trigger expansion block)
**Action**: Add a Prowess case in the `for kw in spec.keywords.iter()` loop, immediately after the Ward block (after line 371, before line 372's closing brace for the loop).

Add:
```rust
if matches!(kw, KeywordAbility::Prowess) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::ControllerCastsNoncreatureSpell,
        intervening_if: None,
        description: "Prowess (CR 702.108a): Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.".to_string(),
        effect: Some(Effect::ApplyContinuousEffect {
            effect_def: Box::new(ContinuousEffectDef {
                layer: EffectLayer::PtModify,
                modification: LayerModification::ModifyBoth(1),
                filter: EffectFilter::Source,
                duration: EffectDuration::UntilEndOfTurn,
            }),
        }),
    });
}
```

**Imports needed in `builder.rs`**: Verify these types are in scope. Some may already be imported; add any that are missing:
- `ContinuousEffectDef` from `crate::cards::card_definition`
- `EffectLayer`, `EffectDuration`, `EffectFilter`, `LayerModification` from `crate::state::continuous_effect`

**CR 702.108b handling**: Multiple instances of prowess trigger separately. Since `OrdSet<KeywordAbility>` deduplicates, a creature with two `Prowess` entries only has one in the set. This is fine for standard cards (all have exactly one prowess). If a creature needs two prowess instances (e.g., from an ability grant), that's handled by adding the triggered ability twice at the grant site, not here. For now, the 1:1 keyword-to-trigger mapping is correct.

**Note**: The `for kw in spec.keywords.iter()` loop iterates each keyword once. Since `Prowess` has no payload (unlike `Ward(u32)`), a simple `matches!` check works. The Ward block uses `if let KeywordAbility::Ward(cost_n) = kw` to extract the cost; Prowess needs no extraction.

### Step 6: Unit Tests

**File**: `crates/engine/tests/prowess.rs` (new file)
**Pattern**: Follow the Ward test structure in `crates/engine/tests/ward.rs`

**Helpers to define** (same pattern as ward.rs):
- `find_object(state, name) -> ObjectId`
- `pass_all(state, players) -> (GameState, Vec<GameEvent>)`
- `noncreature_spell_def() -> CardDefinition` -- a simple instant (e.g., Lightning Bolt)

**Tests to write**:

#### 1. `test_prowess_basic_noncreature_spell_gives_plus_one`
- **CR**: 702.108a
- **Setup**: p1 has a 1/2 creature with `KeywordAbility::Prowess` on battlefield. p1 has a noncreature spell (instant) in hand. p1 is active player with priority.
- **Actions**: p1 casts the instant. Check stack has 2 items (spell + prowess trigger). Both players pass -- prowess trigger resolves first.
- **Assert**: Use `mtg_engine::rules::layers::calculate_characteristics(state, creature_id)` to verify power = 2, toughness = 3. Also verify `AbilityTriggered` event was emitted.

#### 2. `test_prowess_does_not_trigger_on_creature_spell`
- **CR**: 702.108a, Monastery Swiftspear ruling
- **Setup**: p1 has prowess creature. p1 casts a creature spell (card with `CardType::Creature`).
- **Assert**: Stack has only 1 item (the creature spell). No prowess trigger.

#### 3. `test_prowess_does_not_trigger_on_artifact_creature_spell`
- **CR**: 702.108a, ruling: "If a spell has multiple types, and one of those types is creature (such as an artifact creature), casting it won't cause prowess to trigger."
- **Setup**: p1 has prowess creature. p1 casts spell with `card_types: [Artifact, Creature]`.
- **Assert**: Stack has only 1 item. No prowess trigger.

#### 4. `test_prowess_does_not_trigger_on_opponent_spell`
- **CR**: 702.108a ("you cast")
- **Setup**: p1 has prowess creature on battlefield. p2 is active player. p2 casts a noncreature instant.
- **Assert**: Stack has only 1 item (p2's spell). No prowess trigger for p1's creature. No `AbilityTriggered` events in the cast output.

#### 5. `test_prowess_resolves_independently_of_triggering_spell`
- **CR**: 702.108a, ruling: "Once it triggers, prowess isn't connected to the spell."
- **Setup**: p1 has prowess creature. p1 casts instant. Both on stack. Both pass -- prowess resolves.
- **Assert**: After prowess resolves, creature has +1/+1. The triggering spell is still on the stack unreserolved. Stack has 1 item remaining.

#### 6. `test_prowess_until_end_of_turn_expires`
- **CR**: 702.108a ("until end of turn"), CR 514.2
- **Setup**: p1 has prowess creature. p1 casts instant. Prowess resolves, +1/+1 applied. Advance turn to cleanup.
- **Assert**: After cleanup step, `calculate_characteristics` shows original P/T (no +1/+1). The `UntilEndOfTurn` continuous effect was removed.
- **Note**: Use 2+ players. Advance through full turn with `pass_all` or manual phase advancement.

#### 7. `test_prowess_multiple_spells_stack`
- **CR**: 702.108a
- **Setup**: p1 has prowess creature. p1 casts two instants sequentially (with mana for both). Let prowess triggers resolve after each.
- **Assert**: After both prowess triggers resolve, `calculate_characteristics` shows +2/+2 total.

#### 8. `test_prowess_multiplayer_only_controllers_creatures_trigger`
- **CR**: 702.108a, multiplayer
- **Setup**: 4-player Commander game. p1 has prowess creature. p3 has prowess creature. p1 (active) casts noncreature spell.
- **Assert**: p1's prowess creature triggers (1 prowess trigger on stack). p3's creature does NOT trigger. Stack has 2 items (spell + 1 trigger). Verify the triggered ability's controller is p1.

### Step 7: Card Definition (later phase)

**Suggested card**: Monastery Swiftspear
- Card ID: `monastery-swiftspear`
- Mana cost: {R}
- Type: Creature -- Human Monk
- Keywords: Haste, Prowess
- P/T: 1/2
- Simple, iconic, no additional abilities beyond Haste + Prowess
- Use `card-definition-author` agent

### Step 8: Game Script (later phase)

**Suggested scenario**: "Monastery Swiftspear prowess trigger from Lightning Bolt, then combat"
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Script steps**:
1. Initial state: p1 controls Monastery Swiftspear (1/2, haste, prowess) on battlefield. p1 has Lightning Bolt in hand with sufficient red mana. p2 has 40 life (Commander).
2. p1 casts Lightning Bolt targeting p2 (or a creature).
3. Prowess trigger goes on stack above Lightning Bolt.
4. All pass -- prowess resolves. Swiftspear P/T is now 2/3.
5. All pass -- Lightning Bolt resolves. p2 takes 3 damage.
6. Move to combat. Declare Swiftspear attacking p2.
7. No blockers. Combat damage: p2 takes 2 (Swiftspear's boosted power).
8. Assert p2 life = 40 - 3 - 2 = 35.

## Interactions to Watch

- **Layer system**: The +1/+1 from prowess is a Layer 7c (`PtModify`) continuous effect with `UntilEndOfTurn` duration. It correctly interacts with `SetPowerToughness` (Layer 7b, e.g., Humility setting 1/1 base) and `SwitchPowerToughness` (Layer 7d) because of strict layer ordering. Prowess's +1/+1 applies AFTER base-setting and BEFORE switching.
- **Humility interaction**: If Humility is on the battlefield ("All creatures lose all abilities and have base 1/1"), prowess is removed in Layer 6. The creature's `triggered_abilities` are cleared by `RemoveAllAbilities`, so `collect_triggers_for_event` won't find any prowess trigger. This should work naturally with no special handling.
- **Cleanup step expiry**: `expire_end_of_turn_effects` in `crates/engine/src/rules/layers.rs:L542` removes all `UntilEndOfTurn` continuous effects. The prowess +1/+1 effect will be cleaned up automatically.
- **Storm copies**: Storm copies emit `SpellCopied` events (not `SpellCast`) in `create_storm_copies` at `crates/engine/src/rules/copy.rs`. This means storm copies correctly do NOT trigger prowess. Cascade-cast spells emit `SpellCast` at `copy.rs:L347` and DO trigger prowess (correct per CR 702.85c -- cascade casts ARE real casts).
- **Enrichment dependency**: The prowess `TriggeredAbilityDef` is added in `builder.rs` during `GameStateBuilder::build()`. This means any `ObjectSpec::creature(...).with_keyword(KeywordAbility::Prowess)` automatically gets the triggered ability. Card definitions that include `Prowess` in their keywords will also get it through `enrich_spec_from_def` (verify this path handles keywords the same way).
- **`enrich_spec_from_def` site**: Check `crates/engine/src/state/stubs.rs` or wherever `enrich_spec_from_def` lives. If it has its own keyword-to-trigger expansion (separate from `builder.rs`), the Prowess case must be added there too. This is critical: the two ETB sites pattern (builder.rs for tests, enrich for card defs) means BOTH must be updated.

## File Change Summary

| File | Line(s) | Action |
|------|---------|--------|
| `crates/engine/src/state/continuous_effect.rs` | After L97 | Add `EffectFilter::Source` variant |
| `crates/engine/src/state/game_object.rs` | After L117 | Add `TriggerEvent::ControllerCastsNoncreatureSpell` variant |
| `crates/engine/src/state/hash.rs` | L544-547 | Add `EffectFilter::Source => 12u8` hash arm |
| `crates/engine/src/state/hash.rs` | L875-876 | Add `TriggerEvent::ControllerCastsNoncreatureSpell => 7u8` hash arm |
| `crates/engine/src/effects/mod.rs` | L884-897 | Add `CEFilter::Source => CEFilter::SingleObject(ctx.source)` resolution |
| `crates/engine/src/rules/abilities.rs` | L314-323 | Expand `SpellCast` branch with noncreature check + `ControllerCastsNoncreatureSpell` dispatch |
| `crates/engine/src/state/builder.rs` | After L371 | Add Prowess keyword-to-`TriggeredAbilityDef` expansion (same loop as Ward) |
| `crates/engine/tests/prowess.rs` | New file | 8 tests covering basic, negative, edge cases, multiplayer |
