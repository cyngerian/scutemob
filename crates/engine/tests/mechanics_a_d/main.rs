//! Integration-test target `mechanics_a_d`: per-keyword and per-mechanic tests, names a-d.
//!
//! Each module below was its own `tests/*.rs` binary until SR-9a collapsed the
//! 297 of them into nine. Layout and the rule for where a new test file goes:
//! `docs/sr-9a-test-consolidation.md`. `tests/no_stray_test_binaries.rs` fails
//! the suite if a top-level `tests/*.rs` file reappears.

mod adapt;
mod adventure_tests;
mod affinity;
mod afflict;
mod afterlife;
mod aftermath;
mod alliance;
mod amass;
mod amplify;
mod annihilator;
mod armorcraft_judge_etb;
mod ascend;
mod assist;
mod backup;
mod bargain;
mod battle_cry;
mod bestow;
mod blitz;
mod blood_tokens;
mod bloodrush;
mod bloodthirst;
mod bolster;
mod bushido;
mod buyback;
mod cascade;
mod casualty;
mod champion;
mod changeling;
mod channel;
mod chosen_creature_type;
mod cipher;
mod cleave;
mod clue_tokens;
mod coin_flip_dice;
mod collect_evidence;
mod connive;
mod convoke;
mod corrupted;
mod count_based_scaling;
mod craft;
mod crew;
mod cumulative_upkeep;
mod cycling;
mod dash;
mod daybound;
mod decayed;
mod delve;
mod destroy_and_reanimate;
mod dethrone;
mod devoid;
mod devour;
mod discover;
mod disturb;
mod domain_and_freecast;
mod dredge;
mod dungeon_cards;
mod dungeon_data_model;
mod dungeon_resolution;
mod dungeon_venture;
