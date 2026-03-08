# Corner Case Correctness Audit

> **Living document.** Re-audit after each milestone that adds mechanics or cards.
> Use `cr-coverage-auditor` agent for bulk re-checks.
>
> **Last audited**: 2026-02-23 (post-M9.4)

---

## Summary

| Status | Count | % |
|--------|-------|---|
| Covered | 29 | 81% |
| Partial | 0 | 0% |
| Gap | 4 | 11% |
| Deferred | 3 | 8% |

**Card definition gaps**: 0 (all 12 fixed in M9.4)

---

## Corner Case Coverage Table

| # | Name | CR Refs | Status | Evidence | Addressed In |
|---|------|---------|--------|----------|-------------|
| 1 | Humility + Opalescence | 613.8 | **COVERED** | `layers.rs:878` | M4 |
| 2 | Blood Moon + Urborg (BM newer) | 613.8 | **COVERED** | `layers.rs:1010` | M4 |
| 3 | Blood Moon + Urborg (BM older, dependency wins) | 613.8 | **COVERED** | `layers.rs:1072` | M4 |
| 4 | Yixlid Jailer + Anger | 613.1f, 112.6 | **COVERED** | `test_cc4_yixlid_jailer_removes_anger_graveyard_ability` in `layers.rs` | M9.4 S4 |
| 5 | Clone copying a Clone | 707.2, 707.3 | **COVERED** | `test_clone_copies_clone_chain` in `copy_effects.rs` | M9.4 S7 |
| 6 | Humility + Magus of the Moon | 613.8 | **COVERED** | `test_cc6_humility_magus_of_moon_nondependency` in `layers.rs` | M9.4 S3 |
| 7 | Opalescence + Parallax Wave | 400.7, 613 | **COVERED** | `test_cc7_opalescence_parallax_wave_zone_change` in `layers.rs` | M9.4 S4 |
| 8 | Deathtouch + Trample | 702.2, 702.19 | **COVERED** | `combat.rs:541` | M6 |
| 9 | Indestructible + Deathtouch | 702.12, 704.5h | **COVERED** | `test_cc9_indestructible_survives_deathtouch` in `sba.rs` | M9.4 S3 |
| 10 | Legendary Rule (Simultaneous Entry) | 704.5j, 603.6a | **COVERED** | `test_cc10_legendary_rule_simultaneous_etb_triggers` in `sba.rs` | M9.4 S3 |
| 11 | +1/+1 and -1/-1 Counter Annihilation | 704.5q | **COVERED** | `sba.rs:558, 591` | M3 |
| 12 | Spell Fizzle (All Targets Illegal) | 608.2b | **COVERED** | `targeting.rs:297` | M5 |
| 13 | Partial Fizzle (Some Targets Illegal) | 608.2b | **COVERED** | `targeting.rs:386` | M5 |
| 14 | APNAP Trigger Ordering | 603.3b | **COVERED** | `abilities.rs:616`, `six_player.rs:222` | M7 |
| 15 | Panharmonicon Trigger Doubling | 603.2d | **COVERED** | `test_panharmonicon_doubles_etb_trigger` in `trigger_doubling.rs` | M9.4 S9 |
| 16 | Multiple Replacement Effects (Player Choice) | 614.5 | **COVERED** | `replacement_effects.rs:745, 2568` | M8 |
| 17 | Self-Replacement Effects Apply First | 614.15 | **COVERED** | `replacement_effects.rs:525, 613` | M8 |
| 18 | Commander Zone-Change + Rest in Peace | 903.9a, 614 | **COVERED** | `replacement_effects.rs:1240, 1341` | M8–M9 |
| 19 | "Enters Tapped" Replacement | 614 | **COVERED** | `replacement_effects.rs:1952, 2134` | M8 |
| 20 | First Strike + Double Strike (Combined) | 702.4, 702.7 | **COVERED** | `test_cc20_first_strike_blocks_double_strike` in `combat.rs` | M9.4 S4 |
| 21 | Protection from X (DEBT) | 702.16 | **COVERED** | `protection.rs` — 9 tests covering DEBT: damage, enchanting, blocking, targeting | M9.4 S5–S6 |
| 22 | Hexproof vs. Non-Targeted Effects | 702.11 | **COVERED** | `test_cc22_hexproof_does_not_block_global_effects` in `keywords.rs` | M9.4 S3 |
| 23 | "When This Dies" + Flicker | 400.7, 608.2b | **COVERED** | `test_cc23_flicker_kills_spell_fizzles_no_dies_trigger` in `corner_case_gaps.rs` | M9.4 S4 |
| 24 | Tokens Briefly in Non-Battlefield Zones | 111.8, 704.5d | **COVERED** | `test_cc24_token_dies_trigger_fires_before_sba_cleanup` in `sba.rs` | M9.4 S4 |
| 25 | Phasing and Auras/Equipment | 702.26 | **DEFERRED** | Phasing not implemented | Deferred |
| 26 | Commander Damage from a Copy | 903 | **COVERED** | `commander_damage.rs:221` | M9 |
| 27 | Commander Tax with Partners | 903 | **COVERED** | `commander.rs:905` | M9 |
| 28 | Commander Dies with Exile Replacement | 903.9a, 614 | **COVERED** | `replacement_effects.rs:1240, 1341` | M8–M9 |
| 29 | Cascade into a Split Card | 702.84, 708.4 | **COVERED** | `test_cascade_combined_mana_value_skip` in `cascade.rs` — documents split-card MV behavior (CR 708.4); cascade implemented via `copy::resolve_cascade` | M9.4 S9 |
| 30 | Morph/Manifest Face-Down | 708 | **DEFERRED** | Face-down mechanics not implemented | Deferred |
| 31 | Aura on Illegal Permanent After Type Change | 704.5m | **COVERED** | `test_cc31_aura_falls_off_after_type_change_ends` in `sba.rs` | M9.4 S4 |
| 32 | Mutate Stack Ordering | 725 | **COVERED** | `test_mutate_*` in `mutate.rs` — merged_cards model, over/under choice, zone-change splitting (CR 729.5), mutate trigger; game script 192 | B15+Mutate |
| 33 | Sylvan Library + Draw Replacement | 614 | **COVERED** | `test_cc33_sylvan_library_draw_tracking` in `replacement_effects.rs` — `cards_drawn_this_turn` tracking verified | M9.4 S4 |
| 34 | Reveillark + Karmic Guide Loop | 726, 104.4b | **COVERED** | `test_loop_detection_threshold_is_three` in `loop_detection.rs` — detection algorithm in `rules/loop_detection.rs` | M9.4 S10 |
| 35 | Storm + Copying | 702.40, 707.10 | **COVERED** | `test_storm_creates_copies`, `test_spell_copy_is_not_cast` in `storm_copy.rs` | M9.4 S8 |
| 36 | Blood Moon + Urza's Saga (Layer 4 vs 6, Saga SBA) | 714.4, 613.1d, 613.1f, 613.7 | **GAP** | No test — requires Saga gained-ability tracking + updated CR 714.4 SBA logic (June 2025 rules change) | — |

---

## Card Definition Gaps

All 12 card definition gaps addressed in M9.4. No remaining simplifications or no-op placeholders.

| Card(s) | Gap | Severity | CR Ref | Status |
|---------|-----|----------|--------|--------|
| Lightning Greaves, Swiftfoot Boots | Equipment doesn't grant keywords via continuous effects | HIGH | 702.11, 702.10 | **FIXED** M9.4 S2 |
| Alela, Cunning Conqueror | Goad is `DrawCards(0)` no-op placeholder | HIGH | 701.38 | **FIXED** M9.4 S1 |
| Read the Bones | Scry simplified to nothing | MEDIUM | 701.18 | **FIXED** M9.4 S1 |
| Dimir Guildgate | Modal color choice simplified to colorless | MEDIUM | 106.6 | **FIXED** M9.4 S1 |
| Rogue's Passage | "Can't be blocked" effect not implemented | MEDIUM | 509.1a | **FIXED** M9.4 S2 |
| Thought Vessel, Reliquary Tower | "No maximum hand size" not implemented | MEDIUM | 402.2 | **FIXED** M9.4 S1 |
| Path to Exile | Optional search is unconditional | MEDIUM | 701.19 | **FIXED** M9.4 S1 |
| Rhystic Study | Payment choice deferred — draw always fires | MEDIUM | — | **FIXED** M9.4 S2 |
| Rest in Peace | ETB "exile all graveyards" not implemented | MEDIUM | — | **FIXED** M9.4 S2 |
| Leyline of the Void | Opening hand rule not modeled | LOW | 113.6b | **FIXED** M9.4 S3 |
| Darksteel Colossus | Shuffle-into-library is just redirect | LOW | 701.20 | **FIXED** M9.4 S3 |
| Alela (trigger scoping) | Fires on all turns, not just opponent turns | LOW | 603.2 | **FIXED** M9.4 S1 (data model) |

---

## Deferred Items

These require entire unimplemented subsystems. They will be addressed when those
subsystems are built, likely as part of M12+ card pipeline expansion.

| # | Name | Mechanic | Rationale |
|---|------|----------|-----------|
| 25 | Phasing + Auras/Equipment | Phasing (CR 702.26) | Rare in Commander; entire subsystem needed |
| 30 | Morph/Manifest Face-Down | Face-down (CR 708) | Entire subsystem needed |

---

## Audit Process

1. Read `docs/mtg-engine-corner-cases.md` (35 cases)
2. Grep `crates/engine/tests/` for test functions covering each case
3. Grep `test-data/generated-scripts/` for scripts covering each case
4. Mark: COVERED / PARTIAL / GAP
5. For existing card definitions: scan `crates/engine/src/cards/definitions.rs` for TODO comments, simplifications, and no-op placeholders
6. Update this document with findings

**Trigger**: Run after every milestone that adds mechanics, keywords, or card definitions.
