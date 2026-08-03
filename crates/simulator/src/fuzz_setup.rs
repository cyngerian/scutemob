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
    enrich_spec_from_def, CardDefinition, CardId, CardRegistry, GameState, GameStateBuilder,
    GameStateError, ObjectSpec, PlayerId, ZoneId,
};
use rand::rngs::StdRng;
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

/// CR 903.6 — place one seat's commander, then its main deck into the library in the
/// order given (the caller shuffles; see [`build_fuzz_state`]).
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
    // CR 903.6: the commander goes to the command zone.
    if let Some(def) = cards.iter().find(|c| c.card_id == deck.commander) {
        let spec = ObjectSpec::card(pid, &def.name)
            .in_zone(ZoneId::Command(pid))
            .with_card_id(deck.commander.clone());
        let spec = enrich_spec_from_def(spec, card_defs);
        builder = builder.object(spec);
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
/// `PlayerId` order.
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

    let mut decks: Vec<(PlayerId, DeckConfig)> = Vec::new();
    for &pid in &player_ids {
        if let Some(deck) = random_deck(&mut rng, cards) {
            decks.push((pid, deck));
        } else {
            // Fallback: just basic lands
            let fallback = DeckConfig {
                commander: CardId("teysa-karlov".to_string()),
                main_deck: (0..99).map(|_| CardId("plains".to_string())).collect(),
            };
            decks.push((pid, fallback));
        }
    }

    // Build initial state using GameStateBuilder, populating libraries from decks
    let mut builder = GameStateBuilder::new().with_registry(registry.clone());

    for &pid in &player_ids {
        builder = builder.add_player(pid);
    }

    // Build a name→def lookup for enriching card specs
    let card_defs: HashMap<String, CardDefinition> =
        cards.iter().map(|c| (c.name.clone(), c.clone())).collect();

    for (pid, deck) in &decks {
        builder = place_registered_deck(builder, *pid, deck, cards, &card_defs);
    }

    builder = builder.first_turn_of_game();

    let state = builder.build().map_err(FuzzSetupError::Builder)?;

    Ok(FuzzGameSetup { state, decks })
}
