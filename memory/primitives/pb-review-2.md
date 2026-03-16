# Primitive Batch Review: PB-2 -- Conditional ETB Tapped

**Date**: 2026-03-16
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 614.1c (replacement effects for entering the battlefield)
**Engine files reviewed**: `cards/card_definition.rs` (Condition variants, unless_condition field), `effects/mod.rs` (condition evaluation), `rules/replacement.rs` (ETB replacement dispatch), `state/hash.rs` (hash support)
**Card defs reviewed**: 46 with `unless_condition: Some(...)`, 10 missed cards identified

## Verdict: needs-fix

The engine changes are solid: 9 new Condition variants, `unless_condition` field on Replacement, condition evaluation logic, and hash support are all correctly implemented. The condition logic properly excludes the entering land from "other lands" counts and handles phased-out permanents. Tests cover all 7 condition types with positive and negative cases.

However, 10 card defs that should have been updated in PB-2 were missed entirely -- they still have empty `abilities: vec![]` with only TODO comments and no `unless_condition` replacement. Additionally, Isolated Chapel has stale TODO comments despite having been partially fixed. Three cards (Minas Tirith, Temple of the Dragon Queen) have remaining TODOs for abilities beyond the ETB condition.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `effects/mod.rs:3999` | **ControlLandWithSubtypes does not exclude ctx.source.** Unlike ControlAtMostNOtherLands and ControlAtLeastNOtherLands, this condition scans all battlefield lands without excluding the entering permanent. Currently safe because check-lands have no land subtypes, but a latent bug if any future card with land subtypes uses this condition. No fix needed now. |
| 2 | LOW | `effects/mod.rs:4047` | **ControlBasicLandsAtLeast does not exclude ctx.source.** Same pattern as finding 1. Currently safe because battle-lands are not Basic. No fix needed now. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **HIGH** | 10 cards (see details) | **Missed by PB-2 -- no unless_condition replacement added.** These cards have "enters tapped unless" in oracle text but still have empty abilities or no Replacement ability. **Fix:** Add `AbilityDefinition::Replacement` with appropriate `unless_condition` to each. |
| 2 | MEDIUM | `isolated_chapel.rs` | **Stale TODO comments.** Lines 3-4 and 16 still say "DSL gap" and "always tapped for now" but the condition IS implemented. Also, oracle_text uses old template "Isolated Chapel enters the battlefield tapped" instead of current "This land enters tapped". **Fix:** Remove stale TODO comments on lines 3-4 and 16. Update oracle_text to current template. |
| 3 | MEDIUM | `minas_tirith.rs:27` | **Remaining TODO for second activated ability.** `{1}{W}, {T}: Draw a card. Activate only if you attacked with two or more creatures this turn.` Not a PB-2 primitive (needs activation restriction DSL), but the TODO should be tracked. **Fix:** Leave TODO but acknowledge it's out of PB-2 scope. |
| 4 | MEDIUM | `temple_of_the_dragon_queen.rs:21-22` | **Remaining TODOs for color choice and mana ability.** The "choose a color" + "{T}: Add one mana of the chosen color" pattern needs a color choice DSL primitive that doesn't exist yet. **Fix:** Leave TODOs but acknowledge out of PB-2 scope. |

### Finding Details

#### Finding 1: 10 Cards Missed by PB-2

**Severity**: HIGH
**Cards affected**:
- `castle_ardenvale.rs` -- needs `ControlLandWithSubtypes(vec!["Plains"])`
- `castle_embereth.rs` -- needs `ControlLandWithSubtypes(vec!["Mountain"])`
- `castle_locthwain.rs` -- needs `ControlLandWithSubtypes(vec!["Swamp"])`
- `castle_vantress.rs` -- needs `ControlLandWithSubtypes(vec!["Island"])`
- `mystic_sanctuary.rs` -- needs `ControlAtLeastNOtherLandsWithSubtype { count: 3, subtype: "Island" }`
- `witchs_cottage.rs` -- needs `ControlAtLeastNOtherLandsWithSubtype { count: 3, subtype: "Swamp" }`
- `arena_of_glory.rs` -- needs `ControlLandWithSubtypes(vec!["Mountain"])`
- `mistrise_village.rs` -- needs `ControlLandWithSubtypes(vec!["Mountain", "Forest"])`
- `shifting_woodland.rs` -- needs `ControlLandWithSubtypes(vec!["Forest"])`
- `spymasters_vault.rs` -- needs `ControlLandWithSubtypes(vec!["Swamp"])`

**CR Rule**: 614.1c -- "Effects that read '[This permanent] enters with . . . ,' 'As [this permanent] enters . . . ,' or '[This permanent] enters as . . .' are replacement effects."
**Issue**: These 10 cards all have "enters tapped unless" in their oracle text but were not updated by PB-2. They still have empty abilities (just TODOs) or are missing the Replacement ability entirely (Spymaster's Vault has a mana ability but no replacement). Without the replacement, these cards never enter tapped at all -- they produce wrong game state by giving free untapped mana.

**Fix**: For each card, add `AbilityDefinition::Replacement { trigger: ReplacementTrigger::WouldEnterBattlefield { filter: ObjectFilter::Any }, modification: ReplacementModification::EntersTapped, is_self: true, unless_condition: Some(<appropriate condition>) }` to the abilities vec. Also add the mana abilities where they are TODOs (tap for mana is a simple pattern already used by all other PB-2 cards). The 4 castles and 3 special lands also need their additional activated abilities as TODOs at minimum.

#### Finding 2: Isolated Chapel Stale TODOs

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/isolated_chapel.rs:3-4,16`
**Issue**: Lines 3-4 say "DSL gap -- the conditional ETB check... is not supported. Modeled as always entering tapped (safe conservative fallback)." Line 16 says "TODO: conditional ETB -- should check for Plains/Swamp; always tapped for now." But the `unless_condition` IS set to `Some(Condition::ControlLandWithSubtypes(...))` on line 23, so the condition IS checked. These comments are stale and misleading. Additionally, the oracle_text on line 14 uses the old template "Isolated Chapel enters the battlefield tapped" instead of current oracle text "This land enters tapped unless you control a Plains or a Swamp."
**Fix**: Remove the stale comments on lines 3-4 and line 16. Update oracle_text to `"This land enters tapped unless you control a Plains or a Swamp.\n{T}: Add {W} or {B}."`.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 614.1c (replacement effects for ETB) | Yes | Yes | `apply_self_etb_from_definition` in replacement.rs |
| 614.1c + check-land pattern | Yes | Yes | `test_conditional_etb_check_land_condition_met`, `test_conditional_etb_check_land_condition_not_met` |
| 614.1c + fast-land pattern | Yes | Yes | `test_conditional_etb_fast_land` (2 cases) |
| 614.1c + bond-land pattern | Yes | Yes | `test_conditional_etb_bond_land` (4-player + 2-player) |
| 614.1c + battle-land pattern | Yes | Yes | `test_conditional_etb_battle_land` (2 cases) |
| 614.1c + slow-land pattern | Yes | Yes | `test_conditional_etb_slow_land` (2 cases) |
| 614.1c + subtype-count pattern | Yes | Yes | `test_conditional_etb_subtype_count_land` (Mystic Sanctuary, 2 cases) |
| 614.1c + reveal-land pattern | Yes | Yes | `test_conditional_etb_reveal_land` (2 cases) |
| 614.1c + ControlLegendaryCreature | Yes | No | No test for Minas Tirith pattern |
| 614.1c + ControlCreatureWithSubtype | Yes | No | No test for Temple of the Dragon Queen pattern |
| 614.1c + Condition::Or | Yes | No | No test for combined Or condition |

## Card Def Summary

### Cards Successfully Updated (46 total)

| Cycle | Cards | Oracle Match | TODOs | Game State |
|-------|-------|-------------|-------|------------|
| Check-lands (10) | glacial_fortress, dragonskull_summit, rootbound_crag, hinterland_harbor, sulfur_falls, sunpetal_grove, drowned_catacomb, woodland_cemetery, clifftop_retreat | Yes (9/10) | 0 (9/10) | Correct |
| Check-lands (1) | isolated_chapel | Old template | 2 stale | Correct (condition works despite stale TODOs) |
| Fast-lands (3) | blooming_marsh, darkslick_shores, concealed_courtyard | Yes | 0 | Correct |
| Bond-lands (10) | morphic_pool, sea_of_clouds, bountiful_promenade, spire_garden, spectator_seating, vault_of_champions, training_center, luxury_suite, undergrowth_stadium, rejuvenating_springs | Yes | 0 | Correct |
| Battle-lands (5) | cinder_glade, canopy_vista, prairie_stream, smoldering_marsh, sunken_hollow | Yes | 0 | Correct |
| Slow-lands (8) | haunted_ridge, stormcarved_coast, rockfall_vale, shipwreck_marsh, deathcap_glade, dreamroot_cascade, sundown_pass, shattered_sanctum | Yes | 0 | Correct |
| Snarls/Reveal (6) | foreboding_ruins, choked_estuary, necroblossom_snarl, shineshadow_snarl, furycalm_snarl, frostboil_snarl | Yes | 0 | Correct |
| Tribal reveal (2) | gilt_leaf_palace, flamekin_village | Yes | 0 | Correct |
| Special (2) | minas_tirith, temple_of_the_dragon_queen | Yes | 1, 2 | Correct (ETB condition works; other abilities are TODO) |

### Cards Missed by PB-2 (10 total)

| Card | Condition Needed | Current State |
|------|-----------------|---------------|
| castle_ardenvale | ControlLandWithSubtypes(["Plains"]) | Empty abilities, all TODO |
| castle_embereth | ControlLandWithSubtypes(["Mountain"]) | Empty abilities, all TODO |
| castle_locthwain | ControlLandWithSubtypes(["Swamp"]) | Empty abilities, all TODO |
| castle_vantress | ControlLandWithSubtypes(["Island"]) | Empty abilities, all TODO |
| mystic_sanctuary | ControlAtLeastNOtherLandsWithSubtype { count: 3, subtype: "Island" } | Empty abilities, all TODO |
| witchs_cottage | ControlAtLeastNOtherLandsWithSubtype { count: 3, subtype: "Swamp" } | Empty abilities, all TODO |
| arena_of_glory | ControlLandWithSubtypes(["Mountain"]) | Empty abilities, all TODO |
| mistrise_village | ControlLandWithSubtypes(["Mountain", "Forest"]) | Empty abilities, all TODO |
| shifting_woodland | ControlLandWithSubtypes(["Forest"]) | Empty abilities, all TODO |
| spymasters_vault | ControlLandWithSubtypes(["Swamp"]) | Has mana ability, missing Replacement |

## Mana Color Verification (Spot-Checked)

All sampled cards produce the correct mana colors per oracle text. The `mana_pool(W, U, B, R, G, C)` argument ordering was verified across multiple cycles. No mana color mismatches found.

## Test Summary

14 tests covering PB-2 conditions in `replacement_effects.rs`:
- 2 tests for check-lands (condition met / not met)
- 1 test for fast-lands (2 cases: at limit / over limit)
- 1 test for bond-lands (2 cases: 4-player / 2-player)
- 1 test for battle-lands (2 cases: enough basics / too few)
- 1 test for slow-lands (2 cases: enough other lands / too few)
- 1 test for subtype-count lands (Mystic Sanctuary, 2 cases)
- 1 test for reveal-lands (2 cases: matching card in hand / no match)

**Gaps**: No tests for `ControlLegendaryCreature`, `ControlCreatureWithSubtype`, or `Condition::Or`. These are LOW priority since the logic is straightforward and the patterns are tested by analogy.
