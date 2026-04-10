# Primitive WIP: PB-S — GrantActivatedAbility

batch: PB-S
title: GrantActivatedAbility — Layer 6 AddManaAbility + AddActivatedAbility
cards_affected: 5 full (Cryptolith Rite, Chromatic Lantern, Citanul Hierophants, Paradise Mantle, Enduring Vitality) + 2 partial (Song of Freyalise, Umbral Mantle)
started: 2026-04-11
phase: implement
plan_file: memory/primitives/pb-plan-S.md

## Resolved Open Questions (oversight 2026-04-11)
1. **chars.abilities parallel population**: specialized vecs only. Do NOT populate the generic `abilities: Vector<AbilityInstance>` field. If implementation finds a contradiction (dispatch sites reading AbilityInstance), stop and flag.
2. **Face-down + grant interaction** (REVISED 2026-04-11 after runner stop-and-flag): Face-down creatures DO inherit Layer 6 grants — this is rules-correct per CR 708.2 (face-down only strips the card's own characteristics; external continuous effects still apply). Engine matches CR by design: override at layers.rs:216-237 runs before the layer loop, layer 6 then re-adds grants on top. Required test: `test_face_down_creature_inherits_granted_mana_ability` — asserts (a) face-down creature under Cryptolith Rite has the tap-for-mana ability, (b) ability list does NOT include front-face characteristics (name/other abilities), (c) after turning face-up, creature has BOTH its front-face abilities AND the granted ability (proves grant survives face-down→face-up cleanly). Subtlety check DONE by oversight 2026-04-11: override is clean reset (`Vector::new()`/`vec![]`), AddManaAbility is pure append (matches AddKeyword pattern), no intermediate dispatch between override and layer loop. Safe to proceed.
3. **Hash version bump**: yes, bump. Standard policy when adding LayerModification variants.
4. **mana_solver.rs calc-chars fix**: DEFER. Do NOT include in PB-S. Log as LOW follow-up if not already tracked.

## Scope Boundary (enforced)
- PB-S is **GrantActivatedAbility only**. NOT Marvin's reflection pattern.
- If Marvin comes up during implementation, **stop and flag** — do not expand scope.
- Target cards: Cryptolith Rite, Chromatic Lantern, Citanul Hierophants, Paradise Mantle, Enduring Vitality (5 full unblocks).
- Song of Freyalise and Umbral Mantle: leave TODOs with specific remaining-blocker notes (Saga / {Q}).

## Step Checklist (IMPLEMENT)
- [x] 1. Engine changes (plan Changes 1-8):
  - [x] Two new LayerModification variants added (continuous_effect.rs: AddActivatedAbility + AddManaAbility, +~25 LOC)
  - [x] apply_modification match arms added (layers.rs: two new arms after RemoveKeyword, +~10 LOC)
  - [x] hash.rs: discriminants 23 (AddActivatedAbility) + 24 (AddManaAbility) added; HashInto for ActivatedAbility/ManaAbility already existed (+~10 LOC)
  - [x] handle_tap_for_mana: uses calculate_characteristics for ability lookup (mana.rs, ~+6 LOC)
  - [x] legal_actions.rs: mana-ability loop uses calculated chars (+~4 LOC); activated-ability loop uses calculated chars (+~4 LOC)
  - [x] helpers.rs: added ActivatedAbility + ActivationCost exports (ManaAbility already present)
  - [x] Face-down override verified: layers.rs:216-237 runs before layer loop (246); face-down sets status.face_down=true; override clears mana_abilities, then layer 6 re-adds grant on top. CR 708.2 correct.
- [x] 2. Card def fixes (5 full + 2 partial TODO updates):
  - [x] cryptolith_rite.rs — fully authored with AddManaAbility + CreaturesYouControl
  - [x] chromatic_lantern.rs — self tap-for-any + AddManaAbility + LandsYouControl
  - [x] citanul_hierophants.rs — AddManaAbility(tap_for(Green)) + CreaturesYouControl
  - [x] paradise_mantle.rs — Equip + AddManaAbility + AttachedCreature
  - [x] enduring_vitality.rs — Vigilance + AddManaAbility + CreaturesYouControl; die-return TODO remains
  - [x] song_of_freyalise.rs — TODO text updated to reference PB-S LayerModification
  - [x] umbral_mantle.rs — TODO text updated to reference AddActivatedAbility + {Q} blocker
- [x] 3. Unit tests (10 tests, new file crates/engine/tests/grant_activated_ability.rs):
  - [x] test_cryptolith_rite_grants_mana_ability_to_creatures
  - [x] test_granted_mana_ability_taps_and_produces_mana (replaces implicit test; exercises TapForMana command)
  - [x] test_cryptolith_rite_grant_ends_when_source_leaves
  - [x] test_two_cryptolith_rites_grant_two_abilities_but_one_tap
  - [x] test_chromatic_lantern_grants_only_lands_not_creatures
  - [x] test_chromatic_lantern_lands_keep_existing_abilities
  - [x] test_paradise_mantle_grants_only_equipped_creature
  - [x] test_granted_mana_ability_respects_summoning_sickness
  - [x] test_humility_removes_granted_mana_ability
  - [x] test_face_down_creature_inherits_granted_mana_ability (CR 708.2; verifies face-up transition too)
- [x] 4. Build verification:
  - [x] `~/.cargo/bin/cargo test --all` green (all tests pass; 10 new tests)
  - [x] `~/.cargo/bin/cargo clippy -- -D warnings` clean (0 warnings)
  - [x] `~/.cargo/bin/cargo build --workspace` builds replay-viewer + TUI (no exhaustive-match regressions — LayerModification not matched in those tools)
  - [x] `~/.cargo/bin/cargo fmt --check` clean
- [x] 5. Hash version bump applied (discriminants 23 + 24 added to HashInto for LayerModification)
- [x] 6. No new TODOs introduced (5 card TODO blocks removed; 2 TODO texts updated with better info)
