//! Integration-test target `primitives`: PB-* primitive batches (`pb_*` / `primitive_*`).
//!
//! Each module below was its own `tests/*.rs` binary until SR-9a collapsed the
//! 297 of them into nine. Layout and the rule for where a new test file goes:
//! `docs/sr-9a-test-consolidation.md`. `tests/no_stray_test_binaries.rs` fails
//! the suite if a top-level `tests/*.rs` file reappears.

mod counter_replacement_pb_cd;
mod pb_ac1_untap_counter;
mod pb_ac2_card_integration;
mod pb_ac3_dynamic_pt_counts;
mod pb_ac4_card_integration;
mod pb_ac4_per_mode_targeting;
mod pb_ac5_alt_costs;
mod pb_ac6_card_integration;
mod pb_ac6_phase_action_conditions;
mod pb_ac7_ability_index_desync;
mod pb_ac7_card_integration;
mod pb_ac7_type_change_ability_removal;
mod pb_ac8_restrictions_and_wingame;
mod pb_ac9_wheel_and_misc;
mod pb_k_land_drops;
mod pb_l_landfall;
mod pbd_damaged_player_filter;
mod pbn_subtype_filtered_triggers;
mod pbp_power_of_sacrificed_creature;
mod pbt_up_to_n_targets;
mod primitive_pb32;
mod primitive_pb37;
mod primitive_pb_cc_a;
mod primitive_pb_cc_c;
mod primitive_pb_cc_c_followup;
mod primitive_pb_eat;
mod primitive_pb_ewc;
mod primitive_pb_ewcd;
mod primitive_pb_lki_cc;
mod primitive_pb_lki_power;
mod primitive_pb_oos_lki_power_3;
mod primitive_pb_q;
mod primitive_pb_ts;
mod primitive_pb_x;
mod primitive_pb_xa;
mod primitive_pb_xa2;
mod primitive_pb_xs;
mod primitive_pb_xs_e;
mod primitive_sr34_composite_mana_costs;
mod sr13_lki_damage_source;
