# Ability Review: Transform Mini-Milestone

**Date**: 2026-03-08
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.27, 712, 702.145, 702.146, 702.167, 730
**Files reviewed**: `cards/card_definition.rs`, `state/game_object.rs`, `state/types.rs`, `state/hash.rs`, `state/stack.rs`, `state/builder.rs`, `state/mod.rs`, `rules/layers.rs`, `rules/engine.rs`, `rules/casting.rs`, `rules/replacement.rs`, `rules/resolution.rs`, `rules/events.rs`, `rules/turn_actions.rs`, `rules/command.rs`, `tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/stack_view.rs`, `tests/transform.rs`, `tests/daybound.rs`, `tests/disturb.rs`, `tests/craft.rs`

## Verdict: needs-fix

The DFC data model (CardFace, is_transformed, DayNight), layer system integration, Transform command, Daybound/Nightbound enforcement, and Disturb casting/replacement are all well-implemented and match the CR rules correctly. The layer system face swap at line 87-128 of layers.rs is clean and correct, zone changes properly reset is_transformed (CR 400.7), and the disturb counter/fizzle exile path is handled.

However, there are two HIGH findings in the Craft implementation: (1) the craft mana cost is never validated or deducted from the player's mana pool, and (2) craft material type/count constraints are never validated against the `CraftMaterials` definition. There is also one MEDIUM finding: `state.timestamp_counter` is not incremented after use in several transform sites, violating the monotonic timestamp guarantee needed for CR 701.27f.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `engine.rs:1233` | **Craft mana cost never paid.** `handle_activate_craft` never looks up or deducts the mana cost from `AbilityDefinition::Craft { cost, .. }`. **Fix:** look up the Craft cost from card registry, call `can_pay_cost`/`pay_cost` before exiling. |
| 2 | **HIGH** | `engine.rs:1302` | **Craft material validation missing.** Material type and count from `CraftMaterials` are never checked -- any objects from battlefield/graveyard are accepted regardless of type or quantity. **Fix:** look up `CraftMaterials` from the card registry, validate material count and types. |
| 3 | MEDIUM | `engine.rs:1210` | **Timestamp counter not incremented after transform.** `last_transform_timestamp` is set to `state.timestamp_counter` but the counter is never incremented, breaking the monotonic guarantee for CR 701.27f. Same issue at `turn_actions.rs:1693`. **Fix:** add `state.timestamp_counter += 1;` after each use. |
| 4 | LOW | `replacement.rs:618` | **Disturb replacement comment says "from battlefield" but CR ruling says "from anywhere".** The implementation is actually correct for all practical scenarios (CR 400.7 resets `was_cast_disturbed` on zone changes; the fizzle path handles stack-to-graveyard separately). The comment should say "from anywhere, but only battlefield is reachable since CR 400.7 resets the flag on zone change" for clarity. **Fix:** update comment. |
| 5 | LOW | `engine.rs:1233` | **Craft resolves immediately without the stack.** CR 702.167a describes Craft as an activated ability, which per CR 602.2 should use the stack. The "return transformed" part should be the effect that resolves, with opponents having a response window after costs are paid. This is consistent with the engine's existing pattern for other activated abilities (Bloodrush, Scavenge, etc.) so it's not a new deviation. **Fix:** no fix needed now; address holistically when activated abilities are moved to the stack. |
| 6 | LOW | `tests/transform.rs:306` | **test_transform_once_guard does not exercise CR 701.27f.** The test transforms twice via commands, but CR 701.27f is about triggered/activated abilities on the stack failing to re-transform. The test just verifies two command transforms toggle back and forth. A true 701.27f test requires a TransformTrigger on the stack with a lower timestamp. **Fix:** rename to `test_transform_double_flip` and add a separate test for 701.27f if/when TransformTrigger is exercised from card definitions. |
| 7 | LOW | `tests/daybound.rs` | **Missing day-to-night and night-to-day transition tests via untap step.** Tests 1-6 call `enforce_daybound_nightbound` directly but never test `check_day_night_transition` (CR 730.2a/b) via the full untap step flow. The plan called for `test_day_to_night_no_spells` and `test_night_to_day_two_spells`. **Fix:** add tests that advance the turn and verify transitions in the untap step. |
| 8 | LOW | `layers.rs:87` | **No colors reset before applying back face color.** When the back face has a `color_indicator` (line 117-118) or a mana cost (line 119-121), colors are correctly set. But if the back face has NEITHER color_indicator NOR mana_cost, `chars.colors` retains the front face's colors. Most back faces have at least one, but this edge case should be handled. **Fix:** add `chars.colors = OrdSet::new();` before the color_indicator/mana_cost checks (line 116), so back faces with no color indicator and no mana cost are correctly colorless. |

### Finding Details

#### Finding 1: Craft mana cost never paid

**Severity**: HIGH
**File**: `crates/engine/src/rules/engine.rs:1233`
**CR Rule**: 702.167a -- "[Cost], Exile this permanent, Exile [materials] from among permanents you control and/or cards in your graveyard: Return this card to the battlefield transformed under its owner's control."
**Issue**: `handle_activate_craft` validates the source is on the battlefield, has a Craft ability, timing is sorcery-speed, and materials are in valid zones. However, it never looks up the `ManaCost` from `AbilityDefinition::Craft { cost, .. }`, never calls `can_pay_cost` to verify the player can afford it, and never calls `pay_cost` to deduct it. The test (`test_craft_basic_exile_and_transform`) adds mana to the pool but never verifies it was deducted. The craft ability activates for free.
**Fix**: After the `has_craft` check (line 1292), look up the `AbilityDefinition::Craft { cost, .. }` to extract the mana cost. Call `casting::can_pay_cost(pool, &cost)` to validate. Call `casting::pay_cost(&mut p.mana_pool, &cost)` to deduct. Update tests to verify mana is deducted after activation.

#### Finding 2: Craft material validation missing

**Severity**: HIGH
**File**: `crates/engine/src/rules/engine.rs:1302`
**CR Rule**: 702.167b -- "If an object in the [materials] of a craft ability is described using only a card type or subtype without the word 'card,' it refers to either a permanent on the battlefield that is that type or subtype or a card in a graveyard that is that type or subtype."
**Issue**: The handler only checks that materials are in battlefield or graveyard zones. It does not validate: (a) that the number of materials matches what `CraftMaterials` requires (e.g., `Artifacts(2)` requires exactly 2), or (b) that each material is of the correct type (e.g., must be an artifact). Any objects from valid zones are accepted regardless of type or count.
**Fix**: Look up `CraftMaterials` from the card registry alongside the cost. Match on the `CraftMaterials` variant to validate both count and type. For `CraftMaterials::Artifacts(n)`, verify `material_ids.len() == n` and each material has `CardType::Artifact` in its characteristics (use `calculate_characteristics` for battlefield permanents, or check `obj.characteristics.card_types` for graveyard cards).

#### Finding 3: Timestamp counter not incremented

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/engine.rs:1210`, `crates/engine/src/rules/turn_actions.rs:1693`
**CR Rule**: 701.27f -- The transform-once guard compares `last_transform_timestamp` against an ability's creation timestamp. Both must be monotonically increasing.
**Issue**: In `handle_transform` (engine.rs:1210), `obj.last_transform_timestamp = state.timestamp_counter` is set but `state.timestamp_counter` is not incremented afterward. Same pattern in `enforce_daybound_nightbound` (turn_actions.rs:1693). In contrast, resolution.rs:6866-6867 correctly does `state.timestamp_counter += 1;`. Without incrementing, multiple transforms in the same command processing cycle get the same timestamp, potentially breaking the CR 701.27f guard.
**Fix**: Add `state.timestamp_counter += 1;` after setting `last_transform_timestamp` in both `handle_transform` (engine.rs) and `enforce_daybound_nightbound` (turn_actions.rs).

#### Finding 4: Disturb replacement comment inaccuracy

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:618`
**CR Rule**: 702.146 ruling -- "If this permanent would be put into a graveyard from anywhere, exile it instead."
**Issue**: The comment on line 618-623 says "Only applies when moving from battlefield to graveyard (not other zones)" which contradicts the oracle text's "from anywhere." The code is correct because `was_cast_disturbed` is only set on battlefield permanents and is reset by CR 400.7 on zone changes, so battlefield-to-graveyard is the only reachable case. The fizzle path (resolution.rs:96) handles the stack-to-graveyard case separately via `is_cast_transformed`. The comment should explain why the "from battlefield" guard is sufficient.
**Fix**: Update the comment to: "Checks `from == Battlefield` because `was_cast_disturbed` is only set on resolved permanents and is reset by CR 400.7 on zone change. The stack-to-graveyard case (countered spells) is handled separately via `is_cast_transformed` in the fizzle path (resolution.rs)."

#### Finding 5: Craft resolves without the stack

**Severity**: LOW
**File**: `crates/engine/src/rules/engine.rs:1233`
**CR Rule**: 702.167a -- Craft is an activated ability; CR 602.2 -- activated abilities use the stack.
**Issue**: The current implementation resolves the entire craft ability (exile costs + return transformed) in a single `handle_activate_craft` call with no stack interaction. Per CR, opponents should have a priority window after costs are paid (exile self + materials) but before the return-to-battlefield effect resolves. This is consistent with the engine's existing pattern for Bloodrush, Scavenge, Saddle, and other activated abilities that also resolve immediately.
**Fix**: No immediate fix needed. Document as a known engine limitation. When the engine supports generic stack-based activated ability resolution, Craft should be migrated to that system.

#### Finding 6: test_transform_once_guard misnomer

**Severity**: LOW
**File**: `crates/engine/tests/transform.rs:306`
**CR Rule**: 701.27f
**Issue**: The test is named `test_transform_once_guard` and cites CR 701.27f, but it just transforms twice via direct commands. CR 701.27f is specifically about triggered/activated abilities failing to re-transform a permanent that already transformed since the ability was put on the stack. The test doesn't exercise this guard at all -- it just verifies that two transform commands toggle the permanent back to front face.
**Fix**: Rename to `test_transform_double_toggle` or similar. The actual CR 701.27f test requires a TransformTrigger on the stack, which depends on card definition-driven triggers not yet implemented.

#### Finding 7: Missing day/night transition tests

**Severity**: LOW
**File**: `crates/engine/tests/daybound.rs`
**CR Rule**: 730.2a/b
**Issue**: The test file tests `enforce_daybound_nightbound` directly (which is good for unit testing the enforcement logic), but the plan called for integration tests covering `check_day_night_transition`: specifically `test_day_to_night_no_spells` (CR 730.2a) and `test_night_to_day_two_spells` (CR 730.2b). These would verify the full untap step flow including spell counting from the previous turn.
**Fix**: Add two tests: (1) Set `state.day_night = Some(Day)`, `state.previous_turn_spells_cast = 0`, run `check_day_night_transition`, verify it becomes Night. (2) Set `state.day_night = Some(Night)`, `state.previous_turn_spells_cast = 2`, run `check_day_night_transition`, verify it becomes Day.

#### Finding 8: Back face colors not reset

**Severity**: LOW
**File**: `crates/engine/src/rules/layers.rs:87`
**CR Rule**: 712.8e -- "it has only the characteristics of its back face"
**Issue**: When resolving back face characteristics, `chars.colors` is only overwritten if the back face has a `color_indicator` (line 117-118) or a `mana_cost` (line 119-121). If neither is present, `chars.colors` retains the front face's colors, which contradicts CR 712.8e ("only the characteristics of its back face"). A back face with no color indicator and no mana cost should be colorless.
**Fix**: Add `chars.colors = OrdSet::new();` on the line before `if let Some(ref color_indicator) = back_face.color_indicator` (before line 117). This ensures the default is colorless, with color_indicator or mana_cost overriding as needed.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.27a (transform action) | Yes | Yes | test_transform_basic_flip |
| 701.27b (distinct from face-down) | N/A | N/A | Morph not yet implemented |
| 701.27c (non-DFC = nothing) | Yes | Yes | test_transform_non_dfc_does_nothing |
| 701.27d (instant/sorcery back face) | Yes | No | Guard present in engine.rs:1186-1204, no test |
| 701.27e (transforms-into triggers) | Partial | No | GameEvent::PermanentTransformed emitted, but no trigger dispatch for "transforms into" |
| 701.27f (transform-once guard) | Yes | No | last_transform_timestamp tracked; test is misnamed and doesn't exercise it |
| 701.27g (transformed permanent) | N/A | N/A | Informational; no direct enforcement needed |
| 712.8a (non-battlefield = front) | Yes | Yes | test_transform_dfc_graveyard_uses_front_face |
| 712.8c (disturb stack mana value) | Partial | Partial | is_cast_transformed handled; mana value on stack not fully tested |
| 712.8d (front face up = front chars) | Yes | Yes | Layer system default |
| 712.8e (back face up = back chars, MV from front) | Yes | Yes | test_transform_dfc_mana_value_uses_front_face |
| 712.9 (only DFCs can transform) | Yes | Yes | Via 701.27c check |
| 712.10 (instant/sorcery back = nothing) | Yes | No | Same as 701.27d |
| 712.13 (spell enters same face as stack) | Yes | Yes | test_disturb_enters_transformed |
| 712.14 (non-stack ETB = front face) | Yes | Yes | Via move_object_to_zone reset |
| 712.18 (transform = not new object) | Yes | Yes | test_transform_preserves_counters |
| 702.145b (daybound: enters transformed, can't transform except) | Yes | Yes | test_daybound_blocks_direct_transform |
| 702.145c (front+daybound+night = transform) | Yes | Yes | test_daybound_transforms_at_night |
| 702.145d (daybound = becomes day) | Yes | Yes | test_daybound_sets_day |
| 702.145e (nightbound: can't transform except) | Yes | Yes | Via daybound lock (same code path) |
| 702.145f (back+nightbound+day = transform) | Yes | Yes | test_nightbound_transforms_at_day |
| 702.145g (nightbound + no daybound = night) | Yes | Yes | test_nightbound_sets_night |
| 730.1 (game starts with neither) | Yes | Yes | test_daybound_sets_day (asserts None) |
| 730.2a (day + 0 spells = night) | Yes | No | Missing integration test |
| 730.2b (night + 2+ spells = day) | Yes | No | Missing integration test |
| 730.2c (neither = skip) | Yes | Yes | test_day_night_no_change_without_permanents |
| 702.146a (disturb from graveyard) | Yes | Yes | test_disturb_cast_from_graveyard |
| 702.146b (enters transformed) | Yes | Yes | test_disturb_enters_transformed |
| 702.146 ruling (exile replacement) | Yes | Yes | test_disturb_exile_replacement_check |
| 702.167a (craft: cost + exile + return) | Partial | Yes | Mana cost not paid (Finding 1) |
| 702.167b (material types) | No | No | Material validation missing (Finding 2) |
| 702.167c (exiled cards reference) | Yes | Yes | test_craft_tracks_exiled_materials |
| 702.167a ruling (non-DFC stays exiled) | Yes | Yes | test_craft_non_dfc_stays_exiled |

## Previous Findings (first review)

N/A -- this is the initial review.
