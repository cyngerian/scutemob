//! `mtg-fuzzer`'s pregame build path, lifted out of `src/bin/fuzzer.rs` so it can be
//! tested (PB-DX22 §B3).
//!
//! CR 103.3 / 103.5 / 903.6.
//!
//! # Why this is a library module and not a function in the binary
//!
//! Cargo compiles `src/bin/fuzzer.rs` as its own crate, so **no integration test can
//! `use` it**. The only way to "test" the fuzzer's state build was therefore to write a
//! second copy of it — which is exactly how `crates/simulator/tests/local_game.rs`'s
//! `build_state` came to exist ("Mirrors `mtg-fuzzer::run_single_game`'s builder logic")
//! and to carry the identical CR 903.6 registration defect. Both callers now share
//! [`place_registered_deck`], so a probe here is a probe on the binary, and
//! `pb_dx22_fuzz_instrument.rs`'s source gate machine-checks that no third copy appears.
//!
//! # Why NOT in `setup.rs`
//!
//! `setup.rs` is the `LocalGame`/play-server pregame path, and every play-server seed pin
//! is a function of it. Keeping the fuzzer's build in a separate file makes "this batch
//! cannot move a play-server pin" a property a reviewer can check from the diff's **file
//! list** rather than from its contents. The two paths differ deliberately — see
//! `setup.rs`'s module doc.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::{
    enrich_spec_from_def, register_commander_zone_replacements, CardDefinition, CardId,
    CardRegistry, GameState, GameStateBuilder, GameStateError, ObjectSpec, PlayerId, ZoneId,
};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::deck::{random_deck, DeckConfig};

/// The un-started `GameState` `mtg-fuzzer` plays, plus the decklists it was built from.
pub struct FuzzGameSetup {
    /// Not yet started: the caller passes this to `mtg_engine::start_game` (or
    /// `LocalGame::start`, which calls it).
    pub state: GameState,
    /// **Pre-shuffle** decks, ascending `PlayerId`, exactly as `random_deck` produced
    /// them — `deck.rs`'s structural order (≤60 non-lands, ≤5 non-basic lands, basics
    /// LAST). The shuffle probe compares the built libraries against this.
    pub decks: Vec<(PlayerId, DeckConfig)>,
}

/// Why a fuzz state could not be built.
#[derive(Debug)]
pub enum FuzzSetupError {
    /// `GameStateBuilder::build` refused the assembled state.
    Builder(GameStateError),
}

/// CR 903.6 — place one seat's commander **and register it**, then its main deck into
/// the library in the order given (the caller shuffles; see [`build_fuzz_state`]).
///
/// # The two commander steps are ONE operation here, on purpose
///
/// Placing an object in `ZoneId::Command(pid)` records *nothing*.
/// `builder.player_commander(pid, cid)` is what populates
/// `PlayerState::commander_ids`, and that field is what every commander rule keys off:
/// CR 903.8 tax (`rules/casting.rs`, mirrored by `legal_actions::effective_cast_cost`),
/// the CR 903.9a/704.6d command-zone-return SBA (`rules/commander.rs`), CR 903.10a
/// commander damage (`rules/combat.rs`), and the CR 903.9b hand/library redirects that
/// `register_commander_zone_replacements` derives *from* `commander_ids`. A game built
/// with the object but not the registration is not a Commander game — the commander is
/// recastable for free forever and deals no commander damage. `setup.rs:381-393` states
/// the same rule; `commander_cast.rs`'s module doc states it a third time.
///
/// Stating it was not enough: `bin/fuzzer.rs` and `tests/local_game.rs` both did half of
/// it for the life of the fuzzer (`OOS-SIM1-4`). Making the pair a single function is
/// the structural fix, and
/// `pb_dx22_fuzz_instrument::test_dx22_command_zone_placement_and_registration_are_one_operation`
/// machine-checks that no third copy re-splits it.
///
/// # The `if let Some(def)` skip is deliberate, and it is a divergence
///
/// A `CardId` with no matching `CardDefinition` is silently DROPPED here, producing a
/// short library with no error — inherited verbatim from `bin/fuzzer.rs` so the
/// extraction is behaviour-neutral. `setup.rs`'s equivalent refuses with
/// `SetupError::MissingCardDefinition`. Recorded as a divergence, not fixed
/// (`OOS-DX22-4`).
pub fn place_registered_deck(
    mut builder: GameStateBuilder,
    pid: PlayerId,
    deck: &DeckConfig,
    cards: &[CardDefinition],
    card_defs: &HashMap<String, CardDefinition>,
) -> GameStateBuilder {
    // CR 903.6: the commander goes to the command zone -- as an OBJECT and as a
    // REGISTRATION. See this function's doc for why doing only the first is not a
    // Commander game.
    if let Some(def) = cards.iter().find(|c| c.card_id == deck.commander) {
        let spec = ObjectSpec::card(pid, &def.name)
            .in_zone(ZoneId::Command(pid))
            .with_card_id(deck.commander.clone());
        let spec = enrich_spec_from_def(spec, card_defs);
        builder = builder.object(spec);
        builder = builder.player_commander(pid, deck.commander.clone());
    }

    // The remaining cards become the library (CR 903.6).
    for card_id in &deck.main_deck {
        if let Some(def) = cards.iter().find(|c| c.card_id == *card_id) {
            let spec = ObjectSpec::card(pid, &def.name)
                .in_zone(ZoneId::Library(pid))
                .with_card_id(card_id.clone());
            let spec = enrich_spec_from_def(spec, card_defs);
            builder = builder.object(spec);
        }
    }

    builder
}

/// Build the fuzzer's un-started `GameState` for `player_count` seats.
///
/// Deterministic in `seed` alone: one `StdRng::seed_from_u64(seed)`, drawn in ascending
/// `PlayerId` order — per seat `random_deck` **then** `shuffle`.
///
/// # Where the shuffle sits in the RNG stream, and why (PB-DX22 §2 Fix A)
///
/// The two draws **interleave per seat** — `deck₁, shuffle₁, deck₂, shuffle₂, …` — which
/// is byte-for-byte the pattern `setup::build_initial_state` uses. There, the
/// interleaving is load-bearing (seat 2's decklist depends on seat 1's shuffle, and
/// splitting the loop re-rolls every seeded table — measured, `scutemob-187`). Here
/// nothing is preserved either way: `bin/fuzzer.rs`'s module doc already declares
/// recorded fuzz seeds non-portable across engine changes, and PB-DX22 is another such
/// boundary. So the choice is free, and the free choice is the one that keeps the two
/// build paths the same shape — a future reader should not have to work out why they
/// differ.
///
/// The `random_deck`-returned-`None` fallback branch shuffles too. `SliceRandom::shuffle`
/// consumes RNG on a 99-element slice regardless of element distinctness, so both
/// branches advance the stream identically per seat; only the deck-construction draws
/// differ, a pre-existing asymmetry this does not touch.
///
/// **Not a `setup::build_initial_state` replacement.** This path deals no opening hand
/// (CR 103.5) and never runs `validate_deck` — see `setup.rs`'s module doc for why the
/// two are kept apart, and `OOS-DX22-1` / `OOS-DX22-5` for the consequences.
pub fn build_fuzz_state(
    seed: u64,
    player_count: u32,
    cards: &[CardDefinition],
    registry: &Arc<CardRegistry>,
) -> Result<FuzzGameSetup, FuzzSetupError> {
    let mut rng = StdRng::seed_from_u64(seed);

    // Build random decks for each player
    let player_ids: Vec<PlayerId> = (1..=player_count).map(|i| PlayerId(i as u64)).collect();

    // Two lists, deliberately: `decks` is the PRE-shuffle decklist (what `random_deck`
    // produced, `deck.rs`'s structural order) and is what `FuzzGameSetup` returns, so
    // the shuffle probe has something to compare the built library against. `dealt` is
    // the shuffled order actually placed into the libraries.
    let mut decks: Vec<(PlayerId, DeckConfig)> = Vec::new();
    let mut dealt: Vec<(PlayerId, DeckConfig)> = Vec::new();
    for &pid in &player_ids {
        let mut deck = match random_deck(&mut rng, cards) {
            Some(deck) => deck,
            // Fallback: just basic lands
            None => DeckConfig {
                commander: CardId("teysa-karlov".to_string()),
                main_deck: (0..99).map(|_| CardId("plains".to_string())).collect(),
            },
        };
        decks.push((pid, deck.clone()));
        // CR 103.3 / CR 903.6: "each player shuffles the remaining cards of their deck so
        // that the cards are in a random order. Those cards become the player's library."
        // Before PB-DX22 the fuzzer never did this, so ~34 basics sat on top of every
        // library and the first non-land arrived around personal draw 35-40 — game turn
        // ~136-156 at four seats. `OOS-UI2-1`'s "the fuzzer has never cast a spell" and
        // `OOS-SIM3-1`'s "earliest cast turn 143" are that one fact read at two
        // `--max-turns` values.
        deck.main_deck.shuffle(&mut rng);
        dealt.push((pid, deck));
    }

    // Build initial state using GameStateBuilder, populating libraries from decks
    let mut builder = GameStateBuilder::new().with_registry(registry.clone());

    for &pid in &player_ids {
        builder = builder.add_player(pid);
    }

    // Build a name→def lookup for enriching card specs
    let card_defs: HashMap<String, CardDefinition> =
        cards.iter().map(|c| (c.name.clone(), c.clone())).collect();

    for (pid, deck) in &dealt {
        builder = place_registered_deck(builder, *pid, deck, cards, &card_defs);
    }

    builder = builder.first_turn_of_game();

    let mut state = builder.build().map_err(FuzzSetupError::Builder)?;

    // CR 903.9b: a commander that would go to its owner's hand or library may go to the
    // command zone instead. These are REPLACEMENT effects, not triggers, so they must
    // exist in `state.replacement_effects` before the game starts, and
    // `GameStateBuilder` does not derive them from `commander_ids` itself
    // (`state/builder.rs`'s own doc says so). Omitting this call does NOT break
    // CR 903.9a — the graveyard/exile return is an SBA keyed on `commander_ids` and
    // would work without it — it makes CR 903.9b silently *not exist*: a bounced or
    // shuffled-away commander stays where it went, no `CommanderZoneRedirect` is ever
    // emitted, and any measurement counting that event reads zero VACUOUSLY. Same call
    // `setup.rs` and `testing/replay_harness.rs`'s script path make.
    register_commander_zone_replacements(&mut state);

    Ok(FuzzGameSetup { state, decks })
}
