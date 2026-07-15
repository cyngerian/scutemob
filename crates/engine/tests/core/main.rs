//! Integration-test target `core`: engine foundations and the machine-checked invariant gates (state, turn, priority, resolution, SBAs, hashing, protocol, registry).
//!
//! Each module below was its own `tests/*.rs` binary until SR-9a collapsed the
//! 297 of them into nine. Layout and the rule for where a new test file goes:
//! `docs/sr-9a-test-consolidation.md`. `tests/no_stray_test_binaries.rs` fails
//! the suite if a top-level `tests/*.rs` file reappears.

mod ability_definition_registry;
mod builder_tests;
mod card_def_fixes;
mod card_registry_gate;
mod cda_tests;
mod completeness_deviation_scan;
mod concede;
mod corner_case_gaps;
mod deck_validation;
mod emblem_tests;
mod hash_schema;
mod invariants;
mod keyword_registry;
mod object_identity;
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
mod zone_integrity;
