# Mutate Mini-Milestone Session Plan

**Generated**: 2026-03-08
**Milestone**: Mutate Mini-Milestone (post-B15, pre-M10)
**Sessions**: 3
**Estimated new tests**: 18-22
**Workstream**: W1 (abilities)
**Commit prefix**: `W1-Mutate:`

---

## What This Delivers

- `KeywordAbility::Mutate` (discriminant 144) for keyword presence-checking
- `AltCostKind::Mutate` for the alternative casting cost
- `merged_components: Vector<MergedComponent>` on `GameObject` for the merged-permanent model
- Mutate casting path: targets a non-Human creature you own (CR 702.140a)
- Mutate resolution: merge spell with target, controller chooses over/under (CR 702.140c)
- Zone-change splitting: all components move together; each becomes a separate object in the destination zone (CR 729.3)
- Layer 1 (Copy): merged permanent uses topmost component's characteristics as a copiable effect (CR 729.2a)
- Layer 6 (Ability): merged permanent has ALL abilities from ALL components (CR 702.140e)
- `TriggerEvent::SelfMutates` for "whenever this creature mutates" triggers (CR 702.140d)
- `GameEvent::CreatureMutated` for trigger dispatch
- `StackObjectKind::MutatingCreatureSpell` (discriminant 59) for stack representation
- 3 card definitions: Gemrazer, Nethroi Apex of Death, Brokkos Apex of Forever
- Corner case #32 (Mutate Stack Ordering) covered

## Architecture Summary

### Core Design Decision: MergedComponent Model

A merged permanent is a single `GameObject` that is "represented by" multiple cards/tokens.
We model this with a `merged_components: Vector<MergedComponent>` field on `GameObject`.

    pub struct MergedComponent {
        /// The CardId of this component (for looking up definitions).
        pub card_id: Option<CardId>,
        /// The base characteristics of this component (frozen at merge time).
        pub characteristics: Characteristics,
        /// Whether this component is a token.
        pub is_token: bool,
    }

**Key invariants:**
- `merged_components[0]` is always the topmost component
- An unmerged permanent has `merged_components` empty (not a vec of one)
- The topmost component's characteristics become the base for the layer system (CR 729.2a)
- ALL components contribute abilities to the layer system (CR 702.140e)
- When the merged permanent changes zones, each component becomes a separate `GameObject` in the destination (CR 729.3)

**Why not `Vec<ObjectId>`?** The batch plan suggested `Vec<ObjectId>`, but merged components
are NOT separate objects on the battlefield. They are part of one object. Using ObjectIds
would require maintaining phantom objects that are "part of" another object -- a zone
that doesn't exist. Instead, we store the component data inline. When the merged permanent
leaves the battlefield, we create new GameObjects from the stored component data.

### New Files
- None. All changes go into existing files.

### Modified Files
- `crates/engine/src/state/game_object.rs` -- `MergedComponent` struct, `merged_components` field on `GameObject`
- `crates/engine/src/state/types.rs` -- `KeywordAbility::Mutate` (disc 144), `AltCostKind::Mutate`
- `crates/engine/src/state/stack.rs` -- `StackObjectKind::MutatingCreatureSpell` (disc 59)
- `crates/engine/src/state/hash.rs` -- hash `MergedComponent` and `merged_components`
- `crates/engine/src/state/mod.rs` -- `move_object_to_zone` updated for component splitting; new `merged_components: Vector::new()` in new-object initialization
- `crates/engine/src/state/builder.rs` -- `merged_components: Vector::new()` in object construction
- `crates/engine/src/effects/mod.rs` -- `merged_components: Vector::new()` in token creation
- `crates/engine/src/rules/resolution.rs` -- mutate resolution path (merge instead of ETB)
- `crates/engine/src/rules/casting.rs` -- mutate cast validation (target non-Human you own)
- `crates/engine/src/rules/layers.rs` -- Layer 1 topmost-component characteristics, Layer 6 all-component abilities
- `crates/engine/src/rules/abilities.rs` -- `SelfMutates` trigger dispatch on `CreatureMutated`
- `crates/engine/src/rules/events.rs` -- `GameEvent::CreatureMutated`
- `crates/engine/src/rules/command.rs` -- `mutate_target: Option<ObjectId>` on `CastSpell`
- `tools/replay-viewer/src/view_model.rs` -- exhaustive match arm for `MutatingCreatureSpell` SOK + `Mutate` KW
- `tools/tui/src/play/panels/stack_view.rs` -- exhaustive match arm for `MutatingCreatureSpell`
- `crates/engine/src/cards/helpers.rs` -- export `MergedComponent` if needed by card defs
- `crates/engine/src/cards/defs/gemrazer.rs`, `nethroi_apex_of_death.rs`, `brokkos_apex_of_forever.rs`
- `crates/engine/src/testing/replay_harness.rs` -- `cast_spell_mutate` action type

### State Changes
- `GameObject` gains `merged_components: Vector<MergedComponent>` (empty for unmerged permanents)
- `CastSpell` command gains `mutate_target: Option<ObjectId>` (the non-Human creature you own to mutate onto)
- `StackObject` gains `mutate_target: Option<ObjectId>` (preserved from CastSpell for resolution)
- `StackObject` gains `mutate_on_top: bool` (controller's over/under choice; true = spell on top)

### New Events
- `GameEvent::CreatureMutated { object_id: ObjectId, player: PlayerId }` -- emitted when a mutating creature spell successfully merges with its target

### New Enum Variants
- `KeywordAbility::Mutate` (discriminant 144)
- `AltCostKind::Mutate`
- `StackObjectKind::MutatingCreatureSpell { source_object: ObjectId, target: ObjectId }` (discriminant 59)
- `TriggerEvent::SelfMutates`
- `GameEvent::CreatureMutated { object_id, player }`

### Interception Sites
- `resolution.rs:resolve_top_of_stack` -- new match arm for `MutatingCreatureSpell` SOK; also handles the `StackObjectKind::Spell` path for illegal-target fallback (CR 702.140b)
- `casting.rs:handle_cast_spell` -- validate `mutate_target` when `alt_cost == Some(Mutate)`: target must be non-Human creature you own on battlefield
- `layers.rs:calculate_characteristics` -- Layer 1: if `merged_components` is non-empty, use `merged_components[0].characteristics` as the base; Layer 6: add abilities from `merged_components[1..]`
- `state/mod.rs:move_object_to_zone` -- if the source has non-empty `merged_components`, create a separate `GameObject` for each component in the destination zone (CR 729.3)
- `abilities.rs:check_triggers` -- dispatch `SelfMutates` on `CreatureMutated` event

---

## Session Breakdown

### Session 1: Data Model + Casting Validation (8 items)

**Files**: `state/game_object.rs`, `state/types.rs`, `state/stack.rs`, `state/hash.rs`, `state/mod.rs`, `state/builder.rs`, `effects/mod.rs`, `rules/command.rs`, `rules/casting.rs`, `tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/stack_view.rs`

**Goal**: All new types exist, compile, and hash correctly. Mutate can be announced as an alternative cost with a valid target. No resolution yet.

1. [x] **Add `MergedComponent` struct** to `state/game_object.rs` (CR 729.2):
   - Fields: `card_id: Option<CardId>`, `characteristics: Characteristics`, `is_token: bool`
   - Derive: `Clone, Debug, PartialEq, Eq, Serialize, Deserialize`

2. [x] **Add `merged_components: Vector<MergedComponent>`** to `GameObject` (CR 729.2):
   - Default: `Vector::new()` (empty = not merged)
   - Initialize to `Vector::new()` in: `state/mod.rs:move_object_to_zone` (new object), `state/builder.rs` (test objects), `effects/mod.rs` (token creation)
   - **CR 400.7**: cleared on zone change (new object starts empty)

3. [x] **Add enum variants** to `state/types.rs`:
   - `KeywordAbility::Mutate` with discriminant 147 (B15 inserted FriendsForever=144, ChooseABackground=145, DoctorsCompanion=146 before this)
   - `AltCostKind::Mutate` (after Cleave)

4. [x] **Add `StackObjectKind::MutatingCreatureSpell`** to `state/stack.rs` (discriminant 59):
   - Fields: `source_object: ObjectId, target: ObjectId`
   - Add match arm in `tools/replay-viewer/src/view_model.rs` (stack_kind_info)
   - Add match arm in `tools/tui/src/play/panels/stack_view.rs`
   - Add `KeywordAbility::Mutate` arm in replay-viewer keyword display function
   - Also added `mutate_target: None, mutate_on_top: false` to `tools/tui/src/play/input.rs` CastSpell

5. [x] **Add `mutate_target: Option<ObjectId>`** to `CastSpell` command in `rules/command.rs`:
   - `#[serde(default)]` for backward compatibility
   - Doc comment: CR 702.140a target non-Human creature you own

6. [x] **Add `mutate_target` and `mutate_on_top`** to `StackObject` in `state/stack.rs`:
   - `mutate_target: Option<ObjectId>` -- propagated from CastSpell
   - `mutate_on_top: bool` -- controller's over/under choice (true = spell on top)
   - Initialize both in all existing `StackObject` construction sites (search for `StackObject {`)
   - Also patched 95 test files via Python script + 3 manual fixes (backup.rs, gift.rs, zone_integrity.rs, snapshot_perf.rs, commander_damage.rs)

7. [x] **Hash new types** in `state/hash.rs`:
   - Implement `HashInto` for `MergedComponent`
   - Add `merged_components` to `GameObject` hasher (after `is_reconfigured`)
   - Add `mutate_target` and `mutate_on_top` to `StackObject` hasher

8. [x] **Tests**: `test_mutate_data_model_compiles`, `test_merged_component_default_empty`, `test_mutate_keyword_in_ordset`
   - Verify `GameObject` with empty `merged_components` works normally
   - Verify `KeywordAbility::Mutate` can be inserted/checked in OrdSet
   - `cargo build --workspace` passes (replay-viewer + TUI compile)
   - Also exported `MergedComponent` from `state/mod.rs` and `lib.rs`

### Session 2: Resolution Merge + Zone-Change Splitting + Trigger (8 items)

**Files**: `rules/resolution.rs`, `rules/casting.rs`, `rules/layers.rs`, `rules/abilities.rs`, `rules/events.rs`, `state/mod.rs`

**Goal**: Mutate spells can be cast, resolve (merging with the target), and the merged permanent has correct characteristics. Zone changes split the components. Mutate triggers fire.

1. [x] **Casting validation** in `rules/casting.rs` `handle_cast_spell` (CR 702.140a):
   - When `mutate_target` is `Some(target_id)` AND `alt_cost == Some(Mutate)`:
     - Validate target is on the battlefield
     - Validate target is a creature (by layer-resolved characteristics)
     - Validate target is NOT Human (check subtypes for `SubType("Human")`)
     - Validate target has the same owner as the spell (CR 702.140a: "same owner as this spell")
     - Set `stack_obj.kind = StackObjectKind::MutatingCreatureSpell { source_object, target }`
     - Populate `stack_obj.mutate_target` and move card to stack as normal
   - When `mutate_target` is `Some` but `alt_cost` is not `Mutate`, reject

2. [x] **Add `GameEvent::CreatureMutated`** to `rules/events.rs` (CR 702.140d):
   - Fields: `object_id: ObjectId, player: PlayerId`
   - Hash it in `hash.rs` if GameEvent is hashed (check existing pattern)

3. [x] **Resolution merge** in `rules/resolution.rs` (CR 702.140b, 702.140c, 729.2):
   - New match arm in `resolve_top_of_stack` for `StackObjectKind::MutatingCreatureSpell { source_object, target }`:
     - **Illegal target check (CR 702.140b)**: if target is no longer legal (left battlefield, no longer creature, became Human, etc.), fall through to normal creature spell resolution (enters battlefield normally as if not mutating)
     - **Legal target (CR 702.140c)**: spell does NOT enter the battlefield. Instead:
       a. Read the spell's characteristics from `state.objects.get(&source_object)`
       b. Build a `MergedComponent` from the spell's characteristics and card_id
       c. Read the target permanent's existing `merged_components`
       d. If `stack_obj.mutate_on_top` is true, insert new component at index 0; else append at end
       e. If target had no merged_components yet, first create a component from the target's own characteristics and card_id, then add the spell's component
       f. Remove the spell object from the stack/objects (it no longer exists as a separate entity -- CR 729.2b: "leaves its previous zone and becomes part of an object")
       g. Update the target permanent's base characteristics to match the topmost component (CR 729.2a)
          NOTE: Actually synced obj.characteristics directly from merged_components[0] at Step 5 for trigger scanning correctness
       h. Emit `GameEvent::CreatureMutated { object_id: target_id, player: controller }`
       i. Emit `GameEvent::SpellResolved` for the mutating spell
       j. CR 729.2c: The merged permanent is NOT considered to have just entered the battlefield -- no ETB triggers
     - **Important**: the spell object on the stack must be removed from `state.objects` after merging. Use `state.objects.remove(&source_object)` and remove from the stack zone.

4. [x] **Layer 1 (Copy) integration** in `rules/layers.rs` (CR 729.2a):
   - In `calculate_characteristics`, at the Copy layer:
     - If `obj.merged_components` is non-empty, replace `chars` with `merged_components[0].characteristics.clone()`
     - This is a "copiable effect whose timestamp is the time the objects merged" (CR 729.2a) -- the merge timestamp is the permanent's existing timestamp (already set)

5. [x] **Layer 6 (Ability) integration** in `rules/layers.rs` (CR 702.140e):
   - After the normal Layer 6 processing:
     - If `obj.merged_components.len() > 1`, collect abilities from `merged_components[1..]` (non-topmost components) and add them to `chars.keywords`, `chars.activated_abilities`, `chars.triggered_abilities`, `chars.mana_abilities`
     - These are ADDITIONAL abilities, not replacing. Topmost characteristics already have topmost abilities from Layer 1.

6. [x] **Zone-change splitting** in `state/mod.rs:move_object_to_zone` (CR 729.3):
   - Before the existing zone-move logic, check if `old_object.merged_components` is non-empty
   - If so, split: create a new `GameObject` for each component EXCEPT the first one (which becomes the primary new object)
   - Each component gets: fresh ObjectId, component's card_id, component's characteristics, owner from old_object, destination zone, `is_token` from component
   - The primary new object (component[0] = topmost) follows the existing zone-move path
   - Additional component objects are inserted into `state.objects` and the destination zone
   - CR 729.3a: for graveyard/library, the player may arrange order -- for now, use component order (top to bottom)
   - Return the primary ObjectId as before (callers expect a single ObjectId)
   - **Important**: each split component starts with empty `merged_components` in the new zone
   - NOTE: Also added override of new_object.characteristics from merged_components[0] before inserting into zone

7. [x] **Add `TriggerEvent::SelfMutates`** and dispatch in `rules/abilities.rs`:
   - Add variant to `TriggerEvent` enum in `state/game_object.rs`
   - In `check_triggers` in `abilities.rs`, add arm for `GameEvent::CreatureMutated`:
     - Fire `SelfMutates` triggers on the mutated permanent itself
     - Check `triggered_abilities` for `TriggerEvent::SelfMutates` on the merged permanent (which now has abilities from ALL components)

8. [x] **Tests**: `test_mutate_resolution_basic_merge`, `test_mutate_validation_rejects_human_target`, `test_mutate_resolution_illegal_target_fallback`, `test_mutate_zone_change_splits_components`, `test_mutate_trigger_fires`, `test_mutate_under_uses_target_characteristics`
   - Basic: merge verifies merged_components structure, beast P/T visible via layer system, no ETB triggers
   - Human validation: casting rejected when target is Human
   - Illegal target: target leaves before resolution; beast enters battlefield normally
   - Zone split: merged permanent moves to graveyard; both component cards appear as separate objects
   - Trigger: "whenever this creature mutates" fires and stack contains TriggeredAbility from wolf_id
   - Under: mutate_on_top=false; wolf stays topmost, beast is bottom; wolf P/T from layer system

### Session 3: Card Definitions + Integration Tests + Harness (7 items)

**Files**: `cards/defs/gemrazer.rs`, `cards/defs/nethroi_apex_of_death.rs`, `cards/defs/brokkos_apex_of_forever.rs`, `testing/replay_harness.rs`, `cards/helpers.rs`

**Goal**: Full end-to-end validation with real cards. Game scripts for mutate scenarios.

1. [x] **Card definition: Gemrazer** (`cards/defs/gemrazer.rs`):
   - 4/4 Beast, {3}{G}, Reach + Trample + Mutate
   - Mutate cost {1}{G}{G}
   - "Whenever this creature mutates, destroy target artifact or enchantment an opponent controls"
   - Use `AbilityDefinition::Keyword(KeywordAbility::Mutate)` for the keyword marker
   - Use `AbilityDefinition::Triggered` with `TriggerEvent::SelfMutates` for the mutate trigger
   - Store the mutate cost in a new `AbilityDefinition::MutateCost { cost: ManaCost }` variant (disc 59) OR inline it in the existing pattern. Decision: use a dedicated `AbilityDefinition::MutateCost { cost: ManaCost }` to keep the DSL explicit.

2. [x] **Add `AbilityDefinition::MutateCost`** to `cards/card_definition.rs` (disc 59):
   - Fields: `cost: ManaCost` (the alternative cost to pay)
   - Read by `casting.rs` to determine the mutate mana cost at cast time

3. [x] **Card definition: Nethroi, Apex of Death** (`cards/defs/nethroi_apex_of_death.rs`):
   - 5/5 Legendary Cat Nightmare Beast, {2}{W}{B}{G}, Deathtouch + Lifelink + Mutate
   - Mutate cost {4}{G/W}{B}{B} (simplify hybrid as {4}{G}{B}{B} for now, or model hybrid if supported)
   - "Whenever this creature mutates, return any number of target creature cards with total power 10 or less from your graveyard to the battlefield"
   - **DSL gap note**: the "total power 10 or less" multi-target constraint is complex. Implement the trigger stub with a placeholder effect or simplified version (e.g., return one creature card).

4. [x] **Card definition: Brokkos, Apex of Forever** (`cards/defs/brokkos_apex_of_forever.rs`):
   - 6/6 Legendary Nightmare Beast Elemental, {2}{B}{G}{U}, Trample + Mutate
   - Mutate cost {2}{U/B}{G}{G} (simplify hybrid)
   - "You may cast this card from your graveyard using its mutate ability"
   - This requires: `AbilityDefinition::Static` or a special `cast_from_graveyard_with_mutate` marker. For now, model as a static ability that the casting system recognizes.
   - **Deferred**: full "cast from graveyard" integration for Brokkos. Mark with TODO.

5. [x] **Harness action: `cast_spell_mutate`** in `testing/replay_harness.rs`:
   - New action type in `PlayerAction` / `translate_player_action`
   - Fields: `card_name`, `mutate_target_name`, `on_top: bool`, `mana` (for the mutate cost)
   - Translates to `Command::CastSpell` with `alt_cost: Some(AltCostKind::Mutate)` and `mutate_target: Some(target_id)`
   - Also set `stack_obj.mutate_on_top` via a new field on CastSpell or handle in casting.rs

6. [x] **Integration tests** (3 new tests + 6 from S2):
   - `test_mutate_gemrazer_destroys_artifact`: cast Gemrazer for mutate onto a creature; verify mutate trigger fires and destroys target artifact
   - `test_mutate_over_under_choice`: verify topmost characteristics change based on `mutate_on_top`
   - `test_mutate_stacking_three_deep`: mutate A onto B, then C onto the merged AB; verify three components, topmost C's characteristics
   - `test_mutate_bounce_returns_all_cards`: bounce merged permanent; all component cards go to hand
   - `test_mutate_non_human_validation`: attempt to mutate onto a Human creature; casting fails
   - `test_mutate_merged_has_all_abilities`: verify layer system gives merged permanent abilities from ALL components
   - `test_mutate_aura_equipment_survive_merge`: Aura/Equipment on target permanent survives the merge (CR 729.2c: same object, continuous effects continue)

7. [x] **Game script**: `192_mutate_gemrazer.json` — Gemrazer mutate onto Canopy Crawler, trigger fires (resolves without targets due to DSL gap — see disputes in script). Script passes. Zone-change split script deferred (separate session or future work).
   - Script: basic Gemrazer mutate onto a creature token
   - Script: mutate zone-change split (merged permanent dies, verify graveyard state)

---

## Acceptance Criteria Checklist

- [ ] `KeywordAbility::Mutate` exists and is recognized by the layer system
- [ ] Mutate casting validates: non-Human creature, same owner, on battlefield
- [ ] Mutate resolution merges spell with target (over/under choice)
- [ ] Merged permanent has topmost component's characteristics (name, P/T, types, colors)
- [ ] Merged permanent has ALL abilities from ALL components
- [ ] Illegal mutate target: spell resolves as normal creature (enters battlefield)
- [ ] Zone change splits merged permanent into individual components
- [ ] "Whenever this creature mutates" trigger fires on successful merge
- [ ] 3 card definitions authored (Gemrazer, Nethroi, Brokkos)
- [ ] All tests pass: `~/.cargo/bin/cargo test --all`
- [ ] Zero clippy warnings: `~/.cargo/bin/cargo clippy -- -D warnings`
- [ ] Formatted: `~/.cargo/bin/cargo fmt --check`
- [ ] `cargo build --workspace` passes (replay-viewer + TUI compile)
- [ ] Corner case #32 (Mutate Stack Ordering) covered by tests

## Key CR References

| CR Section | Summary | Session |
|------------|---------|---------|
| 702.140a | Mutate is an alternative cost targeting non-Human creature you own | 1, 2 |
| 702.140b | Illegal target: ceases to be mutating, resolves as normal creature | 2 |
| 702.140c | Legal target: merges, controller chooses over/under | 2 |
| 702.140d | "Whenever this creature mutates" trigger timing | 2 |
| 702.140e | Merged permanent has all abilities from all components | 2 |
| 702.140f | Effects referring to mutating spell refer to merged permanent | 2 |
| 729.1 | Mutate is the only merge keyword | 1 |
| 729.2 | Merge mechanics: place on top/under, becomes merged permanent | 2 |
| 729.2a | Topmost component's characteristics only (copiable effect at Layer 1) | 2 |
| 729.2b | Merging object leaves previous zone, becomes part of battlefield object | 2 |
| 729.2c | Merged permanent is same object: not new ETB, continuous effects persist | 2 |
| 729.2d | Token status determined by topmost component | 2 |
| 729.3 | Leaving battlefield: one permanent leaves, components go individually | 2 |
| 729.3a | Graveyard/library: owner arranges order | 2 |
| 729.3c | "Finds the new object" finds ALL component objects | 2 |
| 729.3d | Replacement effects on leaving apply to all components | 2 |
| 400.7 | Zone change = new object (merged_components cleared) | 1 |

## Corner Cases Addressed

| Corner Case # | Description | Session |
|---------------|-------------|---------|
| #32 | Mutate Stack Ordering — over/under choice, topmost characteristics | 2, 3 |

## Deferred Complexity

These are explicitly OUT OF SCOPE for this mini-milestone (CR 729 subsections):

- **Mutate + copy effects** (CR 729.8): what happens when you copy a merged permanent
- **Mutate + face-down** (CR 729.2e-g): face-down merged components, turning face up
- **Mutate + double-faced** (CR 729.2i-j): transforming merged permanents with DFC components
- **Mutate token ownership**: edge cases with tokens from different controllers in the merge stack
- **CR 729.3d commander exception**: commander in a merged permanent; replacement effects on zone change. The existing commander SBA should handle the commander card individually when the merged permanent splits.
- **Hybrid mana in mutate costs**: Nethroi and Brokkos have hybrid costs; simplify for now
- **Brokkos graveyard casting**: "You may cast this from your graveyard using its mutate ability" requires a cast-from-zone permission system

## Discriminant Chain

After this mini-milestone:
- `KeywordAbility`: 144 (Mutate) -- next available: 145
- `StackObjectKind`: 59 (MutatingCreatureSpell) -- next available: 60
- `AbilityDefinition`: 59 (MutateCost) -- next available: 60

## Gotchas to Watch

1. **`move_object_to_zone` returns `(ObjectId, GameObject)` for ONE object.** The splitting logic must handle creating additional objects for components while still returning the "primary" object. Callers that use the returned ObjectId (e.g., for "when this dies" triggers) will get the topmost component's new ObjectId. Additional components are "extra" returns that callers don't track -- this matches CR 729.3c which says effects "find ALL of those objects."

2. **The spell object on the stack must be REMOVED, not moved.** In normal resolution, the spell card moves from Stack to Battlefield. In mutate resolution, the spell card ceases to exist as a separate entity -- its data is absorbed into the target permanent's `merged_components`. Use `state.objects.remove()` after capturing its data.

3. **Layer 1 override must happen BEFORE Layer 4+ effects.** The topmost component's characteristics replace the base characteristics at Layer 1 (Copy). This means Blood Moon, Humility, etc. all apply AFTER the merge characteristics are set. This is correct per CR 729.2a ("copiable effect whose timestamp is the time the objects merged").

4. **`merged_components` is empty for normal permanents, NOT a vec of one.** Checking `merged_components.is_empty()` distinguishes merged from unmerged. This avoids wasting memory on every permanent in the game.

5. **ETB triggers must NOT fire on merge** (CR 729.2c). The merged permanent is the same object -- it didn't just enter the battlefield. Any code that fires triggers on `PermanentEnteredBattlefield` must not be invoked during mutate resolution.

6. **Auras and Equipment survive merge** (CR 729.2c). Since the permanent is the same object with the same ObjectId, all `attached_to` references remain valid. No special handling needed.

7. **`mutate_on_top: bool` on CastSpell vs StackObject.** The over/under choice is made at cast time (when the spell is put on the stack), not at resolution. It must be stored on the StackObject and read during resolution. Add it to CastSpell command and propagate to StackObject in casting.rs.
