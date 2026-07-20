//! Integration-test target `mechanics_m_z`: per-keyword and per-mechanic tests, names m-z.
//!
//! Each module below was its own `tests/*.rs` binary until SR-9a collapsed the
//! 297 of them into nine. Layout and the rule for where a new test file goes:
//! `docs/sr-9a-test-consolidation.md`. `tests/no_stray_test_binaries.rs` fails
//! the suite if a top-level `tests/*.rs` file reappears.

mod library_ordering;
mod madness;
mod mass_bounce;
mod mass_destroy;
mod mass_reanimate;
mod meld;
mod melee;
mod miracle;
mod modular;
mod morph;
mod mossborn_hydra;
mod mutate;
mod mutate_data_model;
mod myriad;
mod ninjutsu;
mod offspring;
mod outlast;
mod overload;
mod pain_lands;
mod pb_ef5_transform_self;
mod pb_os4_return_transformed;
mod pb_os4b_face_aware_abilities;
mod persist;
mod play_from_graveyard;
mod play_from_top;
mod plot;
mod poisonous;
mod prevent_next_untap;
mod proliferate;
mod prototype;
mod provoke;
mod prowess;
mod rampage;
mod ravenous;
mod reconfigure;
mod recover;
mod regenerate;
mod renown;
mod replicate;
mod retrace;
mod reveal_and_route;
mod ring_cards;
mod ring_tempts_you;
mod riot;
mod saddle;
mod saga_class;
mod scavenge;
mod shadow;
mod skulk;
mod soulbond;
mod spectacle;
mod splice;
mod spree;
mod squad;
mod surge;
mod surveil;
mod suspect;
mod suspend;
mod token_damage_search_replacement;
mod toxic;
mod training;
mod transform;
mod treasure_tokens;
mod tribute;
mod umbra_armor;
mod undaunted;
mod undying;
mod unearth;
mod vanishing;
mod ward;
