# Primitive Batch Review: PB-15 -- Saga & Class Mechanics

**Date**: 2026-03-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 714 (Saga Cards), 714.2b (chapter triggers), 714.3a (ETB lore counter), 714.3b (precombat main lore counter), 714.4 (sacrifice after final chapter), 716 (Class Cards), 716.2a (level-up activated ability), 716.2b (level designation), 716.2d (default level 1)
**Engine files reviewed**: `card_definition.rs` (SagaChapter/ClassLevel variants), `game_object.rs` (class_level field), `hash.rs` (discriminants 67-68), `sba.rs` (check_saga_sbas), `turn_actions.rs` (precombat_main_actions), `replacement.rs` (ETB lore counter + class_level init + fire_saga_chapter_triggers), `engine.rs` (handle_level_up_class), `command.rs` (LevelUpClass), `resolution.rs` (SagaChapter fallback)
**Card defs reviewed**: urzas_saga.rs, druid_class.rs (2 cards)

## Verdict: needs-fix

The Saga framework is solid: lore counter placement on ETB and precombat main, chapter trigger firing via threshold crossing (CR 714.2b), and SBA sacrifice with pending-chapter-on-stack protection (CR 714.4) are all correctly implemented. The Class framework has one HIGH finding: `LevelUpClass` resolves immediately without going through the stack, contradicting CR 716.2a which defines it as an activated ability. The Druid Class rulings explicitly confirm "Gaining a level is a normal activated ability. It uses the stack and can be responded to." Both card defs have multiple remaining TODOs with placeholder effects that produce wrong game state.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `engine.rs:2548-2684` | **Class level-up bypasses the stack.** CR 716.2a defines level-up as an activated ability. Druid Class rulings: "Gaining a level is a normal activated ability. It uses the stack and can be responded to." Current implementation resolves immediately in `handle_level_up_class` without creating a stack object. **Fix:** Create a `StackObjectKind` variant (e.g., `ClassLevelAbility { source_object, target_level }`) and push it onto the stack. Move the level-setting and static-effect registration to resolution.rs. |
| 2 | MEDIUM | `turn_actions.rs:609-612` | **Saga precombat main TBA missing phased-in check.** The filter checks `zone == Battlefield` and `card_id.is_some()` but does not check `is_phased_in()`. A phased-out Saga would incorrectly receive a lore counter. The SBA function correctly checks `is_phased_in()`. **Fix:** Add `&& obj.is_phased_in()` to the filter predicate at line 611. |
| 3 | LOW | `engine.rs:2674-2678` | **LevelUpClass emits AbilityActivated with stack_object_id == source.** Comment says "No stack object -- level-up doesn't use the stack" but this is a consequence of finding 1. When finding 1 is fixed, this event emission should be moved to resolution and use the actual stack object id. **Fix:** Addressed by finding 1 fix. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 4 | MEDIUM | `urzas_saga.rs` | **Chapters I and II are GainLife(0) placeholders.** Oracle says "I -- This Saga gains '{T}: Add {C}.'" and "II -- This Saga gains '{2}, {T}: Create a 0/0 colorless Construct...'" The card has TODO comments acknowledging this. Both chapters produce wrong game state (gain 0 life instead of granting abilities). **Fix:** Implement as continuous effects that grant activated abilities, or document as a known DSL gap if ability-granting continuous effects are not yet expressible. |
| 5 | MEDIUM | `druid_class.rs` | **Three TODOs remaining with placeholder/empty abilities.** Level 1 Landfall uses `WhenEntersBattlefield` (any permanent, not land-specific). Level 2 has empty `abilities: vec![]` (should grant additional land play). Level 3 has empty `abilities: vec![]` (should animate a land). **Fix:** Level 1: use a land-entering trigger condition if available, or document as DSL gap. Levels 2 and 3: implement when "additional land play" modifier and "land animation" continuous effect are available in DSL, or document as known gaps. |
| 6 | LOW | `urzas_saga.rs` | **Chapter III search uses mana value instead of actual mana cost.** Urza's Saga ruling (2021-06-18): "you can find only a card with actual mana cost {0} or {1}, not mana value 0 or 1." The `max_cmc: Some(1)` field checks `ManaCost::mana_value()` which would incorrectly match e.g. an artifact with mana cost {U}. **Fix:** When a `literal_mana_cost` filter is added to `TargetFilter`, update this card def. Document as known approximation until then. |

### Finding Details

#### Finding 1: Class level-up bypasses the stack

**Severity**: HIGH
**File**: `crates/engine/src/rules/engine.rs:2548-2684`
**CR Rule**: 716.2a -- "[Cost]: Level N -- [Abilities]" means "[Cost]: This Class's level becomes N. Activate only if this Class is level N-1 and only as a sorcery"
**Oracle (Druid Class rulings)**: "Gaining a level is a normal activated ability. It uses the stack and can be responded to."
**Issue**: `handle_level_up_class` validates the cost, pays mana, sets `class_level`, and registers continuous effects all in one synchronous function call. There is no stack object created, so opponents cannot respond to the level-up with instant-speed interaction (e.g., destroying the Class in response). This is a fundamental rules violation for CR 716.2a.
**Fix**: Create a new `StackObjectKind` variant (e.g., `ClassLevelAbility { source_object: ObjectId, target_level: u32 }`). In `handle_level_up_class`, only validate legality and pay the cost, then push a stack object. Move the level-setting (`obj.class_level = target_level`) and continuous effect registration to `resolution.rs` in a new match arm for this SOK variant. Add the SOK to hash.rs, TUI stack_view.rs, and replay-viewer view_model.rs.

#### Finding 2: Saga precombat main TBA missing phased-in check

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/turn_actions.rs:609-612`
**CR Rule**: 714.3b -- "that player puts a lore counter on each Saga they control with one or more chapter abilities"
**Issue**: The filter at lines 609-612 checks `obj.controller == active` and `obj.zone == Battlefield` but omits `obj.is_phased_in()`. Per CR 702.26d, a phased-out permanent is treated as though it does not exist; it should not receive lore counters. The SBA function `check_saga_sbas` correctly includes the `is_phased_in()` check.
**Fix**: Add `&& obj.is_phased_in()` to the filter chain at line 611.

#### Finding 4: Urza's Saga chapters I and II are placeholders

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/urzas_saga.rs:18-35`
**Oracle**: "I -- This Saga gains '{T}: Add {C}.' II -- This Saga gains '{2}, {T}: Create a 0/0 colorless Construct artifact creature token with 'This token gets +1/+1 for each artifact you control.'"
**Issue**: Both chapters use `Effect::GainLife { amount: Fixed(0) }` as placeholders. This produces wrong game state -- the Saga should gain activated abilities, not gain 0 life. The TODOs correctly identify this as needing "gains activated ability" continuous effects.
**Fix**: If the DSL supports granting activated abilities via continuous effects, implement chapters I and II correctly. If not, this is a DSL gap that should be tracked for a future primitive batch. The card currently produces wrong game state.

#### Finding 5: Druid Class has three TODOs

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/druid_class.rs:25-55`
**Oracle**: Level 1: "Landfall -- Whenever a land you control enters, you gain 1 life." Level 2: "You may play an additional land on each of your turns." Level 3: "When this Class becomes level 3, target land you control becomes a creature..."
**Issue**: (a) Level 1 trigger uses `WhenEntersBattlefield` which triggers on any permanent entering, not just lands. (b) Level 2 `abilities: vec![]` is empty -- should grant additional land plays. (c) Level 3 `abilities: vec![]` is empty -- should animate a land on level-up.
**Fix**: (a) Use a land-specific trigger condition when available. (b-c) Implement when DSL supports these patterns, or document as DSL gaps.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 714.1 | N/A | N/A | Layout rule, not engine-relevant |
| 714.2a | Yes | No | Roman numeral mapping -- implicit in chapter: u32 |
| 714.2b | Yes | Yes | saga_chapter_trigger_fires_at_threshold_cr714_2b |
| 714.2c | Yes | Yes | saga_multiple_chapters_can_trigger_cr714_2c (lore jump) |
| 714.2d | Yes | Yes | Implicit in check_saga_sbas (max of chapters) |
| 714.3a | Yes | Partial | ETB lore counter in replacement.rs; test saga_etb_places_lore_counter_cr714_3a tests via TBA not ETB |
| 714.3b | Yes (bug) | Yes | precombat_main_actions -- missing is_phased_in check |
| 714.4 | Yes | Yes | saga_sacrifice_sba_after_final_chapter_cr714_4, saga_not_sacrificed_while_chapter_on_stack_cr714_4 |
| 716.2a | Partial | Yes | Level-up implemented but bypasses stack (Finding 1) |
| 716.2b | Yes | No | class_level field on GameObject |
| 716.2d | Yes | No | Default level 1 on ETB (replacement.rs:1038) |
| 716.3 | Yes | Partial | Test class def has non-ClassLevel triggered ability |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| urzas_saga | Partial | 2 | No | Chapters I/II are GainLife(0) placeholders; Ch III approximates mana cost filter |
| druid_class | Partial | 3 | No | Level 1 trigger too broad, Levels 2/3 empty |

## Test Summary

The test file `crates/engine/tests/saga_class.rs` has 10 tests (5 Saga, 5 Class):
- **Saga positive**: ETB lore counter (via TBA), precombat main increment, chapter trigger threshold, multi-chapter trigger, SBA sacrifice
- **Saga negative**: Not sacrificed while chapter on stack
- **Class positive**: Level 1->2, sequential 1->2->3
- **Class negative**: Wrong level rejection, sorcery speed enforcement, insufficient mana
- **Missing**: No test for Saga ETB via actual CastSpell resolution, no test for Class level-up with opponent response (relevant once Finding 1 is fixed), no test for phased-out Saga
