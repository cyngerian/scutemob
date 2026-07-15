//! Integration-test target `scripts`: the JSON game-script corpus and its replay harness.
//!
//! Each module below was its own `tests/*.rs` binary until SR-9a collapsed the
//! 297 of them into nine. Layout and the rule for where a new test file goes:
//! `docs/sr-9a-test-consolidation.md`. `tests/no_stray_test_binaries.rs` fails
//! the suite if a top-level `tests/*.rs` file reappears.

mod completeness_gate;
mod harness_equivalence;
mod run_all_scripts;
mod script_replay;
mod script_schema;
mod unread_init_fields;
