# Primitive Batch Review: PB-32 -- Static/Effect Primitives (Lands, Prevention, Control, Animation)

**Date**: 2026-03-26
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 305.2, 305.2a, 305.2b, 615.1, 613.1b, 701.12a, 701.12b, 604.3, 604.3a, 614.1c
**Engine files reviewed**: `cards/card_definition.rs`, `effects/mod.rs`, `state/mod.rs`, `state/stubs.rs`, `state/builder.rs`, `state/hash.rs`, `state/continuous_effect.rs`, `rules/turn_actions.rs`, `rules/combat.rs`, `rules/replacement.rs`, `rules/resolution.rs`, `rules/layers.rs`
**Card defs reviewed**: 22 (explore, urban_evolution, aesi_tyrant_of_gyre_strait, mina_and_denn_wildborn, wayward_swordtooth, druid_class, spike_weaver, maze_of_ith, kor_haven, zealous_conscripts, connive, dragonlord_silumgar, sarkhan_vol, olivia_voldaren, thieving_skydiver, alexios_deimos_of_kosmos, den_of_the_bugbear, creeping_tar_pit, destiny_spinner, tatyova_steward_of_tides, wrenn_and_realmbreaker, imprisoned_in_the_moon, oko_thief_of_crowns)

## Verdict: needs-fix

PB-32 is a large batch covering four DSL gaps (G-18 through G-21) with 5 new Effect variants, 1 new AbilityDefinition variant, 4 new GameState fields, and 1 new EffectFilter variant. Engine changes are mostly correct and well-structured. Hash discriminants are unique and sequential. GameState fields are properly defaulted in builder.rs and reset in turn_actions.rs. Combat damage prevention enforcement in combat.rs is correct. The card definitions for G-18, G-19, and G-20 are generally well-done. Two findings require fixes: one MEDIUM (wrong layer for dynamic P/T in Destiny Spinner) and one MEDIUM (missing Kicker keyword on Thieving Skydiver). Several LOW findings exist for approximations and pre-existing issues.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `effects/mod.rs:4003` | **GainControl does not emit GameEvent.** No GameEvent is emitted when control changes, unlike other major state changes. Not blocking but reduces event-driven observability. **Fix:** Consider adding a GameEvent::ControlChanged in a future batch; no action needed now. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 2 | **MEDIUM** | `destiny_spinner.rs:50` | **Wrong layer for dynamic P/T.** Uses `EffectLayer::PtCda` (Layer 7a) for `SetPtDynamic`, but this is an externally-granted effect from Destiny Spinner's activated ability, not a CDA intrinsic to the land (CR 604.3a criterion 2 fails). Should be `EffectLayer::PtSet` (Layer 7b). **Fix:** Change `EffectLayer::PtCda` to `EffectLayer::PtSet` on line 50. |
| 3 | **MEDIUM** | `thieving_skydiver.rs` | **Missing Kicker keyword.** Oracle text: "Kicker {X}." All other Kicker cards (goblin_bushwhacker, burst_lightning, etc.) include `AbilityDefinition::Keyword(KeywordAbility::Kicker)`. This card omits it. Without the keyword, the engine won't offer the Kicker cost option during casting. **Fix:** Add `AbilityDefinition::Keyword(KeywordAbility::Kicker),` before the Flying keyword on line 20. |
| 4 | LOW | `druid_class.rs:26` | **Landfall trigger uses wrong trigger condition (pre-existing).** Level 1 uses `WhenEntersBattlefield` (self-ETB) instead of `WheneverPermanentEntersBattlefield` with land filter. Documented in file TODOs. Not a PB-32 regression. **Fix:** None for this batch; pre-existing DSL gap in trigger condition. |
| 5 | LOW | `sarkhan_vol.rs:25` | **+1 ability not implemented (pre-existing TODO).** "+1: Creatures you control get +1/+1 and gain haste until end of turn" uses `Effect::Nothing`. The plan did not scope this ability. **Fix:** None for this batch; pre-existing DSL gap for blanket pump effects. |
| 6 | LOW | `alexios_deimos_of_kosmos.rs:30` | **Upkeep trigger with GainControl not implemented (expected).** Plan noted this as partially fixable but the runner left it as TODO due to "each player's upkeep" trigger gap. Acceptable. **Fix:** None for this batch. |
| 7 | LOW | `wrenn_and_realmbreaker.rs:31-85` | **+1 duration approximation.** Oracle says "until your next turn" but def uses `UntilEndOfTurn`. Plan acknowledged this gap. Documented in the file. **Fix:** None for this batch; requires new `EffectDuration::UntilYourNextTurn` variant. |
| 8 | LOW | `imprisoned_in_the_moon.rs:18` | **EnchantTarget too broad.** Oracle says "Enchant creature, land, or planeswalker" but def uses `EnchantTarget::Permanent` (also matches artifacts, enchantments). No `EnchantTarget::CreatureLandOrPlaneswalker` variant exists. **Fix:** None for this batch; pre-existing DSL gap in EnchantTarget. |
| 9 | LOW | `dragonlord_silumgar.rs:36` | **Target too broad.** Oracle says "target creature or planeswalker" but def uses `TargetPermanent`. No `TargetCreatureOrPlaneswalker` variant exists. **Fix:** None for this batch; pre-existing DSL gap in TargetRequirement. |

### Finding Details

#### Finding 2: Wrong Layer for Dynamic P/T in Destiny Spinner

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/destiny_spinner.rs:50`
**CR Rule**: 604.3a -- "A static ability is a characteristic-defining ability if it meets the following criteria: ... (2) it is printed on the card it affects, it was granted to the token it affects by the effect that created the token, or it was acquired by the object it affects as the result of a copy effect or text-changing effect"
**Issue**: The animated land's P/T is set by Destiny Spinner's activated ability, not by text printed on the land card itself. This fails criterion (2) of CR 604.3a, so it is NOT a CDA. CDAs are processed in Layer 7a; non-CDA P/T-setting effects go in Layer 7b. Using `EffectLayer::PtCda` causes the dynamic P/T to be applied before Layer 7b static P/T-setting effects, which could produce incorrect results if another effect sets P/T at Layer 7b (it would override the animated P/T instead of the other way around).
**Fix**: Change `EffectLayer::PtCda` to `EffectLayer::PtSet` on line 50 of `destiny_spinner.rs`. Verify that `LayerModification::SetPtDynamic` is handled in the Layer 7b code path in layers.rs (the match arm at line 1008 will fire regardless of which layer the effect is assigned to, so this change should work without engine modifications).

#### Finding 3: Missing Kicker Keyword on Thieving Skydiver

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/thieving_skydiver.rs`
**Oracle**: "Kicker {X}. X can't be 0."
**Issue**: The card definition does not include `AbilityDefinition::Keyword(KeywordAbility::Kicker)`. All other Kicker cards in the codebase include this keyword marker. The intervening-if `Condition::WasKicked` on the ETB trigger depends on the Kicker keyword being present for the casting system to offer the Kicker cost and set the `was_kicked` flag on the spell. Without the keyword, `WasKicked` will always evaluate to false and the GainControl effect will never fire.
**Fix**: Add `AbilityDefinition::Keyword(KeywordAbility::Kicker),` as the first ability in the abilities vec (before Flying), matching the pattern used by other Kicker cards.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 305.2 (additional land plays) | Yes | Yes | test_additional_land_play_spell, test_additional_land_play_static_applied_at_turn_start |
| 305.2a (stacking) | Yes | Yes | test_additional_land_play_stacks |
| 305.2b (can't exceed) | Yes (implicit) | No | Enforcement is in legal_actions.rs land play check, not explicitly tested here |
| 615.1 (prevention effects) | Yes | Yes | test_prevent_all_combat_damage_flag, combat.rs enforcement |
| 615.1 (per-creature) | Yes | Yes | test_prevent_combat_damage_from_target |
| 613.1b (Layer 2 control) | Yes | Yes | test_gain_control_creates_continuous_effect |
| 701.12a (exchange all-or-nothing) | Partial | No | ExchangeControl checks both targets exist but doesn't explicitly test partial failure |
| 701.12b (exchange same controller) | Yes | Yes | test_exchange_control_same_controller_noop |
| 701.12b (exchange different) | Yes | Yes | test_exchange_control_different_controllers |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| explore.rs | Yes | 0 | Yes | Clean |
| urban_evolution.rs | Yes | 0 | Yes | Clean |
| aesi_tyrant_of_gyre_strait.rs | Yes | 0 | Yes | "may draw" approximated as mandatory |
| mina_and_denn_wildborn.rs | Partial | 1 | Partial | Activated ability TODO (pre-existing DSL gap) |
| wayward_swordtooth.rs | Partial | 1 | Partial | City's blessing restriction TODO (pre-existing) |
| druid_class.rs | Partial | 2 | Partial | Level 1 Landfall + Level 3 animation TODOs (pre-existing) |
| spike_weaver.rs | Yes | 0 | Yes | Clean; ETB is trigger not replacement (pre-existing) |
| maze_of_ith.rs | Yes | 0 | Yes | "attacking creature" approximated as TargetCreature |
| kor_haven.rs | Yes | 0 | Yes | "attacking creature" approximated as TargetCreature |
| zealous_conscripts.rs | Yes | 0 | Yes | Clean |
| connive.rs | Yes | 0 | Yes | Clean |
| dragonlord_silumgar.rs | Partial | 0 | Yes | TargetPermanent broader than oracle; WhileSourceOnBattlefield approx |
| sarkhan_vol.rs | Partial | 1 | Partial | +1 ability Effect::Nothing (pre-existing) |
| olivia_voldaren.rs | Partial | 1 | Partial | Ability 1 TODO (pre-existing DSL gap) |
| thieving_skydiver.rs | **No** | 0 | **No** | **Missing Kicker keyword (Finding 3)** |
| alexios_deimos_of_kosmos.rs | Partial | 3 | Partial | Multiple TODOs (pre-existing + partially in scope) |
| den_of_the_bugbear.rs | Partial | 0 | Yes | Attack token trigger omitted (documented) |
| creeping_tar_pit.rs | Yes | 0 | Yes | Clean |
| destiny_spinner.rs | Partial | 2 | **No** | **Wrong layer (Finding 2)** + can't-be-countered static TODO |
| tatyova_steward_of_tides.rs | Partial | 1 | Partial | Flying grant to land-creatures TODO (pre-existing) |
| wrenn_and_realmbreaker.rs | Partial | 4 | Partial | Duration approx + multiple TODOs (pre-existing) |
| imprisoned_in_the_moon.rs | Partial | 0 | Partial | EnchantTarget too broad; {T}:Add{C} grant omitted |
| oko_thief_of_crowns.rs | Yes | 0 | Yes | Clean |

## Test Assessment

Tests are structured well with CR citations and good coverage of state manipulation. However, most tests simulate effects by directly manipulating state fields rather than executing through `execute_effect_inner`. This means the dispatch arms in `effects/mod.rs` are not directly tested by these unit tests. The tests verify that `reset_turn_state` and `expire_end_of_turn_effects` handle the new fields correctly, which is valuable. Missing: no integration test that casts a real card def through the full spell resolution pipeline.

| Category | Count | Notes |
|----------|-------|-------|
| G-18 tests | 4 | Spell, static, stacking, stale cleanup |
| G-19 tests | 4 | Flag set, flag reset, per-creature from, per-creature reset |
| G-20 tests | 5 | Create CE, expire EOT, exchange different, exchange same, multiplayer |
| G-21 tests | 0 | No dedicated tests; animation uses existing ApplyContinuousEffect which is tested elsewhere |
| Integration tests | 0 | No card-level integration tests casting through full pipeline |
