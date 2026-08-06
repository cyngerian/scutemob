//! Integration-test target `core`: engine foundations and the machine-checked invariant gates (state, turn, priority, resolution, SBAs, hashing, protocol, registry).
//!
//! Each module below was its own `tests/*.rs` binary until SR-9a collapsed the
//! 297 of them into nine. Layout and the rule for where a new test file goes:
//! `docs/sr-9a-test-consolidation.md`. `tests/no_stray_test_binaries.rs` fails
//! the suite if a top-level `tests/*.rs` file reappears.

mod ability_definition_registry;
mod authoring_report;
mod bare_lookup_ratchet;
mod builder_tests;
mod card_def_fixes;
mod card_defs_fmt;
mod card_registry_gate;
mod cards1_equip_target_roster;
mod cards2_printed_field_fidelity;
mod cda_tests;
mod completeness_deviation_scan;
mod concede;
mod corner_case_gaps;
mod decision_gate;
mod decision_site_walk;
mod deck_validation;
mod effect_choose_gate;
mod emblem_tests;
mod face_dereg_parity;
mod hash_schema;
mod invariants;
mod keyword_registry;
mod lki_diagnostics_scan;
mod object_identity;
mod pb_dx24_trigger_zone_roster;
mod pb_dx25_stack_registry_roster;
mod pb_dx25b_announced_target_roster;
mod pb_dx5_continuous_effect_roster;
mod pb_dx6_turn_face_up_and_attack_tax_roster;
mod pb_rs1_roster_sweep;
mod pb_rs2_hybrid_phyrexian_activation_roster;
mod pb_rs3_combat_trigger_roster;
mod pending_trigger_shape;
mod priority;
mod protocol_roundtrip;
mod protocol_schema;
mod resolution;
mod sba;
mod six_player;
mod snapshot_perf;
mod state_foundation;
mod state_hashing;
mod state_invariants;
mod turn_actions;
mod turn_invariants;
mod turn_structure;
mod ui2_additional_cost_roster;
mod zone_integrity;
