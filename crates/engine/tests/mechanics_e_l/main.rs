//! Integration-test target `mechanics_e_l`: per-keyword and per-mechanic tests, names e-l.
//!
//! Each module below was its own `tests/*.rs` binary until SR-9a collapsed the
//! 297 of them into nine. Layout and the rule for where a new test file goes:
//! `docs/sr-9a-test-consolidation.md`. `tests/no_stray_test_binaries.rs` fails
//! the suite if a top-level `tests/*.rs` file reappears.

mod echo;
mod effect_sacrifice_permanents_filter;
mod embalm;
mod emerge;
mod enchant;
mod encore;
mod enlist;
mod enrage;
mod entwine;
mod equip;
mod escalate;
mod escape;
mod eternalize;
mod evoke;
mod evolve;
mod exalted;
mod exploit;
mod extort;
mod extra_turns;
mod fabricate;
mod fading;
mod flanking;
mod flashback;
mod food_tokens;
mod forage;
mod forecast;
mod foretell;
mod fortify;
mod fuse;
mod gift;
mod golgari_grave_troll;
mod graft;
mod gravestorm;
mod graveyard_abilities;
mod graveyard_targeting;
mod haunt;
mod hideaway;
mod horsemanship;
mod impending;
mod improvise;
mod ingest;
mod investigate;
mod jump_start;
mod keywords;
mod kicker;
mod land_animation;
mod library_search;
mod living_metal;
mod living_weapon;
