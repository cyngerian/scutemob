//! Integration-test target `casting`: casting, mana, and cost payment.
//!
//! Each module below was its own `tests/*.rs` binary until SR-9a collapsed the
//! 297 of them into nine. Layout and the rule for where a new test file goes:
//! `docs/sr-9a-test-consolidation.md`. `tests/no_stray_test_binaries.rs` fails
//! the suite if a top-level `tests/*.rs` file reappears.

mod animated_creature_sacrifice_cost;
mod casting;
mod cost_primitives;
mod mana_and_lands;
mod mana_costs;
mod mana_filter;
mod mana_pool;
mod mana_restriction;
mod optional_cost_and_counter_tax;
mod spell_cost_modification;
mod x_cost_spells;
