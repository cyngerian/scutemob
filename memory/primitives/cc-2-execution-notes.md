# CC-2 execution notes — delete scattered HASH/PROTOCOL version sentinels (`scutemob-238`, 2026-09-05)

Source: `docs/course-correction-2026-09.md` §3.1 item 5. Tests and docs only; A5 gates untouched
(`git diff` over `crates/engine/src` is EMPTY; `hash_schema.rs` / `protocol_schema.rs` untouched).

## Census (by script over every `assert*!` whose two args are a version symbol and an integer literal)

53 live literal assertions (43 × `HASH_SCHEMA_VERSION == 85`, 10 × `PROTOCOL_VERSION == 44`) in 43
files, outside the two canonical gates. Disposition:
- **33 sentinel-only `#[test]` fns deleted whole** (42 of the 53 sites; some fns held both symbols).
- **11 assertions deleted in place** inside tests that also assert other things.
- **4 tests renamed** because their names still said `hash_schema_version` after the assertion left:
  `pbt_up_to_n_targets::test_pbt_hash_schema_version_live_sentinel` → `test_pbt_up_to_n_variants_hash_distinctly`;
  `primitive_pb_cc_c_followup::test_hash_schema_version_after_pb_lki_cc` → `test_cda_modify_hash_determinism_and_discrimination`;
  `primitive_pb_lki_power::test_pb_lki_power_hash_schema_version_and_determinism` → `test_pb_lki_power_hash_determinism_and_variant_discrimination`;
  `primitive_pb_ts::test_pb_ts_hash_schema_version_and_token_spec_hash_determinism` → `test_pb_ts_token_spec_hash_determinism`.
- 40 now-unused imports stripped; 30 orphaned `// ──` banners and ~20 stale prose lines repaired.

## Suite delta (byte-exact name set difference of the two run logs, regex not end-anchored)

Baseline (main at `4815467c`, CC-1's log): **5,363 / 0 / 6**. Final: **5,330 / 0 / 6**, 72 targets.
Count delta **−33 == 33 deleted tests**. Name diff: **37 leavers = 33 deletions + 4 renames; 4 additions = the 4 renames**;
duplicate-name scan EMPTY on both runs. Gates green on both sides: `hash_schema::{hash_schema_version_sentinel,
declaration_fingerprint_is_pinned, stream_fingerprint_is_pinned, history_is_append_only, frozen_prefix_is_pinned}`,
`protocol_schema::{protocol_version_sentinel, protocol_schema_fingerprint_is_pinned, history_is_append_only, frozen_prefix_is_pinned}`.

## Deleted tests (33), by target::name
  - effect_sacrifice_permanents_filter::test_sft_hash_schema_version_live_sentinel
  - loyalty_target_validation::test_pb_ls6_hash_schema_version_is_26
  - optional_cost_and_counter_tax::test_hash_schema_version_is_29
  - pb_ac1_untap_counter::test_pb_ac1_hash_schema_version_live_sentinel
  - pb_ac5_alt_costs::test_hash_schema_version_is_32
  - pb_ac6_phase_action_conditions::test_hash_schema_version_is_33
  - pb_ac7_type_change_ability_removal::test_hash_schema_version_is_34
  - pb_ac8_restrictions_and_wingame::test_pb_ac8_hash_schema_version_live_sentinel
  - pb_ac9_wheel_and_misc::test_pb_ac9_hash_schema_version_live_sentinel
  - pb_dx5_affected_set_snapshot::test_dx5_hash_schema_version_is_70
  - pb_dx6_unflattened_payment_sites::pb_dx6_wire_versions
  - pb_ef10_sacrifice_driven_amounts::test_pb_ef10_version_sentinels
  - pb_ef11_wheel_greatest_discarded::test_pb_ef11_hash_schema_version_live_sentinel
  - pb_ef12_any_color_choice::test_ef12_protocol_version_sentinel
  - pb_ef1_exclude_self_enforcement::test_ef1_hash_schema_version_live_sentinel
  - pb_ef2_create_token_recipient::test_pb_ef2_hash_schema_version_live_sentinel
  - pb_ef7_modal_activated::test_ef7_hash_and_protocol_versions
  - pb_os10_singleton_cleanup::test_pb_os10_version_sentinel
  - pb_os5_relative_attacker_count::test_os5_version_sentinels
  - pb_os6_dfc_flip_conditions::test_os6_version_sentinels
  - pb_os7_defending_player_continuous_filter::test_os7_version_sentinels
  - pb_os8_look_at_top_then_place::test_pb_os8_version_sentinels
  - pb_os9_lieutenant_commander_control::test_pb_os9_version_sentinels
  - pbt_up_to_n_targets::test_pbt_hash_schema_version_sentinel_regression
  - primitive_pb_cc_a::test_hash_schema_version_after_pb_lki_cc
  - primitive_pb_eat::test_pb_eat_hash_schema_version_live_sentinel
  - primitive_pb_ewc::test_pb_ewc_hash_schema_version_live_sentinel
  - primitive_pb_ewcd::test_pb_ewcd_hash_schema_version_live_sentinel
  - primitive_pb_lki_cc::test_pb_lki_cc_hash_schema_version_live_sentinel
  - primitive_pb_xa2::test_pb_hash_schema_version_live_sentinel
  - primitive_pb_xa::test_pb_hash_schema_version_live_sentinel
  - primitive_pb_xs::test_pbxs_hash_schema_version_matches_live_sentinel
  - primitive_pb_xs_e::test_pbxse_hash_schema_version_live_sentinel
