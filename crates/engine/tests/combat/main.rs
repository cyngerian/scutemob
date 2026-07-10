//! Integration-test target `combat`: the combat system.
//!
//! Each module below was its own `tests/*.rs` binary until SR-9a collapsed the
//! 297 of them into nine. Layout and the rule for where a new test file goes:
//! `docs/sr-9a-test-consolidation.md`. `tests/no_stray_test_binaries.rs` fails
//! the suite if a top-level `tests/*.rs` file reappears.

mod additional_combat;
mod combat;
mod combat_damage_triggers;
mod combat_harness;
mod fight_bite;
mod tapped_and_attacking;
