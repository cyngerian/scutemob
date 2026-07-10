//! Integration-test target `rules`: cross-cutting rules subsystems (layers, replacement, copy, triggers, targeting, protection, commander).
//!
//! Each module below was its own `tests/*.rs` binary until SR-9a collapsed the
//! 297 of them into nine. Layout and the rule for where a new test file goes:
//! `docs/sr-9a-test-consolidation.md`. `tests/no_stray_test_binaries.rs` fails
//! the suite if a top-level `tests/*.rs` file reappears.

mod abilities;
mod activation_condition;
mod commander;
mod commander_damage;
mod conditional_statics;
mod copy_effects;
mod copy_redirect;
mod counter_replacement;
mod creature_triggers;
mod damage_multiplier;
mod delayed_triggers;
mod effects;
mod etb_trigger_subtype_filter;
mod etb_trigger_suppression;
mod evasion_protection;
mod grant_activated_ability;
mod grant_flash;
mod layer_correctness;
mod layers;
mod loop_detection;
mod loyalty_target_validation;
mod mana_triggers;
mod modal;
mod modal_triggers;
mod monarch;
mod partner_variants;
mod partner_with;
mod phasing;
mod planeswalker;
mod protection;
mod replacement_effects;
mod restrictions;
mod split_second;
mod static_grants;
mod storm_copy;
mod targeted_abilities;
mod targeting;
mod trigger_doubling;
mod trigger_variants;
