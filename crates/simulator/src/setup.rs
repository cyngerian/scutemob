//! Deterministic pregame setup and mulligans for `LocalGame` (M11-local Session 2).
//!
//! `build_initial_state` assembles a full Commander pregame `GameState` — decks dealt,
//! shuffled, and admitted through the real `validate_deck` gate (Architecture Invariant
//! 9) — from a single seed, reproducibly. It is `tools/tui/src/play/app.rs`'s
//! pre-Session-2 `PlayApp::new` setup logic, lifted into `crates/simulator` and made
//! testable, so the play server (Session 5) and the TUI share one pregame path instead of
//! drifting copies. See `memory/m11-session-plan.md` §3-4 (Session 2).
//!
//! CR 103.5 / 402.1 (opening hand of seven), CR 103.5 / 103.5c (mulligans), CR 903.5a (100-card deck,
//! commander included), CR 903.6 (commander to the command zone, library shuffled),
//! CR 903.9b (the commander's hand/library-to-command-zone replacements).
//!
//! `crates/simulator/src/bin/fuzzer.rs` is deliberately **not** rewired onto this module:
//! its games start every player with an empty hand (session plan §1 fact 2), and every
//! recorded fuzz seed's behaviour is keyed to that starting condition. Giving it real
//! opening hands would silently change what every existing seed reproduces.

use std::collections::{BTreeSet, HashMap};

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use mtg_engine::{
    all_cards, enrich_spec_from_def, register_commander_zone_replacements, validate_deck,
    CardDefinition, CardId, CardRegistry, DeckViolation, GameState, GameStateBuilder,
    GameStateError, ObjectSpec, PlayerId, ZoneId,
};

use crate::deck::{random_deck, DeckConfig};
use crate::local_game::LocalGameLimits;

/// Which bot implementation fills a non-human seat. Mirrors the two `Bot` impls in this
/// crate (`RandomBot` for the fuzzer, `HeuristicBot` as the web client's default) — see
/// `docs/mtg-engine-simulator.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BotKind {
    Random,
    Heuristic,
}

/// Where each seat's deck comes from.
///
/// **A `RandomPerSeat` config is a *recipe*, not a decklist**: it names no cards, so every
/// rebuild of it from a different seed produces different decks — which is exactly what a
/// mulligan must not do (CR 103.5; see [`redeal`]). A caller that will rebuild the same
/// table more than once reads the dealt decklists back out of the first build with
/// [`dealt_decks`] and keeps the resulting `Fixed` list; `tools/play-server`'s
/// `session::new_game` does precisely that.
#[derive(Clone, Debug)]
pub enum DeckSource {
    /// Each seat gets an independently-built `random_deck`, drawn from the single
    /// `LocalGameConfig::seed`-seeded RNG in ascending `PlayerId` order — see
    /// `build_initial_state`'s determinism note.
    RandomPerSeat,
    /// A specific deck for one or more seats. A seat with no entry here is refused with
    /// `SetupError::NoDeckForSeat`, the same failure mode `RandomPerSeat` uses when
    /// `random_deck` cannot find a legendary creature to serve as commander.
    Fixed(Vec<(PlayerId, DeckConfig)>),
}

/// Configuration for a deterministic `LocalGame` pregame build. §3 of the session plan.
#[derive(Clone, Debug)]
pub struct LocalGameConfig {
    pub player_count: u32,
    /// Seats a human occupies (empty ⇒ a pure bot game).
    pub human_seats: BTreeSet<PlayerId>,
    pub bot_kind: BotKind,
    pub seed: u64,
    pub decks: DeckSource,
    pub limits: LocalGameLimits,
}

/// Errors from `build_initial_state` / `redeal`.
#[derive(Clone, Debug)]
pub enum SetupError {
    /// Architecture Invariant 9 (CR 903.5): a seat's deck failed `validate_deck` — wrong
    /// size, a duplicate, a color-identity violation, a banned card, or (the case this
    /// module exists to enforce) a non-`Complete` `CardDefinition`. Assembly is refused
    /// before a single object is placed in `GameStateBuilder`.
    InvalidDeck {
        seat: PlayerId,
        violations: Vec<DeckViolation>,
    },
    /// `DeckSource::RandomPerSeat` found no legendary creature among the `Complete` cards
    /// to serve as commander (`random_deck` returned `None`), or `DeckSource::Fixed` had
    /// no entry for this seat.
    NoDeckForSeat { seat: PlayerId },
    /// A `CardId` a deck names (commander or main-deck entry) has no `CardDefinition` in
    /// the card pool `build_initial_state` draws `ObjectSpec`s from. Distinct from
    /// `DeckViolation::UnknownCard`, which `validate_deck` already checks earlier in the
    /// same call — this is a defensive check at spec-build time, in case a
    /// `DeckSource::Fixed` deck was assembled against a different card pool.
    MissingCardDefinition { seat: PlayerId, card_id: CardId },
    /// `GameStateBuilder::build()` failed (e.g. no players).
    Builder(GameStateError),
}

impl std::fmt::Display for SetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetupError::InvalidDeck { seat, violations } => {
                write!(
                    f,
                    "seat {:?}'s deck failed validation ({} violation(s))",
                    seat,
                    violations.len()
                )?;
                for v in violations {
                    write!(f, "; {v}")?;
                }
                Ok(())
            }
            SetupError::NoDeckForSeat { seat } => {
                write!(f, "no deck could be built for seat {seat:?}")
            }
            SetupError::MissingCardDefinition { seat, card_id } => {
                write!(
                    f,
                    "seat {seat:?}'s deck references {card_id:?}, which has no CardDefinition"
                )
            }
            SetupError::Builder(e) => write!(f, "failed to build the initial game state: {e}"),
        }
    }
}

impl std::error::Error for SetupError {}

/// Mix `base` with `seat` and `mulligan_count` into a new seed for `redeal`.
///
/// Deliberately not a plain `base ^ seat.0 ^ mulligan_count` — that collapses back to
/// `base` itself whenever the two perturbation terms are equal (e.g. seat 1 taking their
/// very first mulligan: `mulligan_count == 1 == seat.0`), which would re-deal the
/// mulliganing player the identical hand they just rejected. Each term is instead run
/// through a distinct odd multiplier (splitmix64-style) before combining. Both multipliers
/// are odd and therefore invertible mod 2^64, so each `mulligan_count` has exactly one
/// colliding `seat`, and for `mulligan_count` in 1..8 those seat numbers are all above
/// 2×10^18 — unreachable at any real table. The one exception is the identity `(seat 0,
/// mulligan 0)`, which returns `base` unchanged; `PlayerId(0)` is never allocated (seats
/// start at 1, see `build_initial_state`) and mulligan 0 is not a redeal, so it is
/// unreachable rather than handled.
fn redeal_seed(base: u64, seat: PlayerId, mulligan_count: u32) -> u64 {
    let seat_term = seat.0.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
    let mulligan_term = u64::from(mulligan_count)
        .wrapping_mul(0xBF58_476D_1CE4_E5B9)
        .wrapping_add(1);
    base ^ seat_term ^ mulligan_term
}

/// `"Human-<n>"` for a human seat, `"Bot-<n>"` otherwise — mirrors the
/// `format!("Bot-{}", i)` the TUI already used for its bot names.
fn seat_name(pid: PlayerId, human_seats: &BTreeSet<PlayerId>) -> String {
    if human_seats.contains(&pid) {
        format!("Human-{}", pid.0)
    } else {
        format!("Bot-{}", pid.0)
    }
}

/// Look up `card_id` in a prebuilt `CardId → CardDefinition` index, or fail with
/// `SetupError::MissingCardDefinition`.
///
/// Indexed rather than a linear scan of `all_cards()`: this is called once per card
/// placed, so a 4-seat table does 400 lookups over ~1,800 defs — 720k `CardId` string
/// comparisons per build, and `redeal` pays it again on every mulligan.
fn find_def<'a>(
    by_card_id: &HashMap<&CardId, &'a CardDefinition>,
    seat: PlayerId,
    card_id: &CardId,
) -> Result<&'a CardDefinition, SetupError> {
    by_card_id
        .get(card_id)
        .copied()
        .ok_or_else(|| SetupError::MissingCardDefinition {
            seat,
            card_id: card_id.clone(),
        })
}

/// The seats a config describes, in ascending order — `PlayerId(1)..=player_count`.
///
/// Ascending, not `HashMap` iteration order: every random draw keyed off `cfg.seed` is
/// taken in this sequence, and that is what makes the seed reproduce a table.
fn seat_ids(cfg: &LocalGameConfig) -> Vec<PlayerId> {
    (1..=cfg.player_count)
        .map(|i| PlayerId(u64::from(i)))
        .collect()
}

/// CR 903.5 — read the decklists **that were actually dealt** back out of a pregame
/// `GameState`, as one concrete [`DeckConfig`] per seat.
///
/// # Why this exists, and why it reads the state rather than the config (G2, `scutemob-187`)
///
/// `DeckSource::RandomPerSeat` is a *recipe*, not a decklist: the commander and all 99
/// main-deck cards of every seat are a function of `cfg.seed` (`crate::deck::random_deck`).
/// [`redeal`] rebuilds the whole table from a **perturbed** seed, so a caller still holding
/// the recipe re-rolls every seat's decklist *and commander* on every mulligan — and the
/// command zone is public (CR 903.6), so the other three players watch their commanders
/// change. CR 103.5 makes a mulligan a permutation of a **fixed** library-plus-hand
/// multiset; it may not replace the multiset. Storing `DeckSource::Fixed(dealt_decks(..))`
/// once, right after the first build, is what makes the perturbed seed reach only the
/// shuffle — the CR-correct behaviour. `tools/play-server`'s `session::new_game` does this
/// for every browser game.
///
/// **The state is the source of truth, deliberately.** The obvious alternative — factor the
/// `RandomPerSeat` draw out of [`build_initial_state`] and resolve straight from the config
/// — was implemented first and reverted, with the reason measured rather than argued: it
/// cannot be done without moving the RNG stream (today the per-seat deck draw and that
/// seat's shuffle interleave, so seat 2's *deck* depends on seat 1's *shuffle*), and moving
/// it re-rolls every table every existing seed builds. That reddened seven tests —
/// six `tools/play-server` probes that pin card names at `SEED = 0`, and
/// `local_game_playthrough` seed 1, which landed on a deck that exposes a pre-existing
/// engine defect ("Aura spells require exactly one target"). Reading the dealt state
/// instead moves **nothing**: the game is built exactly as before, and this only records
/// what it was built with. It is also the stronger guarantee — the multiset a mulligan
/// permutes is the one the player was literally dealt, not one re-derived from a config
/// that is merely believed to agree.
///
/// # Contract
///
/// Pure. Every seat in `cfg` must be present in `state` with exactly one registered
/// commander (`PlayerState::commander_ids`) and a hand plus library of `CardId`-carrying
/// objects — which is precisely what [`build_initial_state`] produces. Any other state
/// (a seat missing, no commander, an object with no `card_id`) is refused with
/// `SetupError::NoDeckForSeat`, because a partially-readable table is not a decklist —
/// as is a seat whose hand ∪ library is empty, or whose commander is sitting in one of
/// them rather than in the command zone. Cards are taken from hand ∪ library, so it does
/// not matter whether this is called before or after the opening hand is dealt; it does
/// matter that it is called before the game is played into.
///
/// A seat with **two** commanders (CR 903.3 partner / background) is refused for the same
/// reason: `DeckConfig` has one `commander` field and cannot express that pairing, and
/// this module never builds one. The refusal is deliberate — silently keeping the first
/// would drop a commander from the rebuilt table.
pub fn dealt_decks(
    state: &GameState,
    cfg: &LocalGameConfig,
) -> Result<Vec<(PlayerId, DeckConfig)>, SetupError> {
    let mut resolved = Vec::with_capacity(cfg.player_count as usize);
    for pid in seat_ids(cfg) {
        let player = state
            .players()
            .get(&pid)
            .ok_or(SetupError::NoDeckForSeat { seat: pid })?;
        // CR 903.5a treats the commander as part of the 100; `DeckConfig` carries it
        // separately, and `build_initial_state` re-adds it to the validated list.
        let commanders: Vec<CardId> = player.commander_ids.iter().cloned().collect();
        // Exactly one: CR 903.3's partner/background variants are not built by this module,
        // and a seat with none is not a Commander deck at all.
        let [commander] = &commanders[..] else {
            return Err(SetupError::NoDeckForSeat { seat: pid });
        };
        let commander = commander.clone();

        let mut main_deck = Vec::with_capacity(99);
        for zone in [ZoneId::Hand(pid), ZoneId::Library(pid)] {
            for obj in state.objects_in_zone(&zone) {
                let card_id = obj
                    .card_id
                    .clone()
                    .ok_or(SetupError::NoDeckForSeat { seat: pid })?;
                main_deck.push(card_id);
            }
        }
        // Two local shape floors, so a wrong-phase call is refused *here* rather than
        // degrading into a decklist that only fails much later, in `validate_deck` on the
        // next rebuild, with an error naming the wrong cause (review LOW 5):
        //
        // * an empty hand+library is not a deck at all (a mid-game state whose cards have
        //   moved to the battlefield/graveyard would read this way);
        // * the commander appearing in the main deck means it is not in the command zone
        //   where CR 903.6 put it, and would make the rebuilt deck 101 cards.
        if main_deck.is_empty() || main_deck.contains(&commander) {
            return Err(SetupError::NoDeckForSeat { seat: pid });
        }

        resolved.push((
            pid,
            DeckConfig {
                commander,
                main_deck,
            },
        ));
    }
    Ok(resolved)
}

/// CR 103.5 / 402.1 (opening hand), CR 903.5a / 903.6 (commander to the command zone, deck
/// admission, library shuffle) — build a full pregame `GameState`.
///
/// **Name caution:** `mtg_engine` exports an unrelated `build_initial_state` (the
/// replay-harness one, which builds a state from a *script's* initial-state block and
/// takes entirely different arguments). Both are crate-root re-exports, so a consumer
/// that imports both crates must qualify the call — `mtg_simulator::build_initial_state`
/// — rather than bare-`use` either. `tools/tui/src/play/app.rs` does exactly that.
///
/// **Not yet started**:
/// callers pass the result to `mtg_engine::start_game` (or `LocalGame::start`, which
/// calls it), which runs `check_all_defs_complete` as the independent second line of
/// defence Architecture Invariant 9 requires. Deck admission here does not replace that
/// check — it prevents nearly every game from ever reaching it rejected.
///
/// Deterministic: every random draw (deck construction, per-seat shuffle) is taken from a
/// single `StdRng` seeded with `cfg.seed`, consumed in ascending `PlayerId` order — the
/// same `cfg.seed` always reproduces the same `GameState` (pinned by
/// `test_setup_same_seed_same_state_hash`).
///
/// **The two draws interleave per seat, and that is load-bearing**: a `RandomPerSeat`
/// seat's deck is drawn, then that seat's library is shuffled, then the next seat's deck is
/// drawn — so seat 2's *decklist* depends on seat 1's *shuffle*. Splitting the loop into
/// "all decks, then all shuffles" would therefore re-roll every table every existing seed
/// builds (measured: seven tests, `scutemob-187` — see [`dealt_decks`], which exists
/// because of that). Under `DeckSource::Fixed` no deck draw happens at all, so seat 1's
/// shuffle is the first draw off the stream, which is the property `tools/play-server`'s
/// `UI1_SEED`/`UI2_SEED`/`SIM1_SEED` fixtures pin their opening hands on.
pub fn build_initial_state(
    cfg: &LocalGameConfig,
) -> Result<(GameState, HashMap<PlayerId, String>), SetupError> {
    let cards = all_cards();
    let registry = CardRegistry::new(cards.clone());
    let card_defs: HashMap<String, CardDefinition> =
        cards.iter().map(|c| (c.name.clone(), c.clone())).collect();
    // Keyed by `CardId`, unlike `card_defs` above, which `enrich_spec_from_def` requires
    // to be keyed by name. Built once; see `find_def`.
    let by_card_id: HashMap<&CardId, &CardDefinition> =
        cards.iter().map(|c| (&c.card_id, c)).collect();

    let mut rng = StdRng::seed_from_u64(cfg.seed);

    let fixed_decks: HashMap<PlayerId, DeckConfig> = match &cfg.decks {
        DeckSource::RandomPerSeat => HashMap::new(),
        DeckSource::Fixed(pairs) => pairs.iter().cloned().collect(),
    };

    // Ascending order, not `HashMap` iteration order — every random draw below must be
    // taken in a fixed sequence for `cfg.seed` to reproduce the same state.
    let player_ids: Vec<PlayerId> = seat_ids(cfg);

    let mut builder = GameStateBuilder::new().with_registry(registry.clone());
    for &pid in &player_ids {
        builder = builder.add_player(pid);
    }

    let mut names = HashMap::new();

    for &pid in &player_ids {
        let mut deck = match &cfg.decks {
            DeckSource::RandomPerSeat => {
                let deck =
                    random_deck(&mut rng, &cards).ok_or(SetupError::NoDeckForSeat { seat: pid })?;
                // The 99+1 contract `random_deck` promises (CR 903.5a): 99 main-deck
                // cards plus the commander is exactly 100.
                debug_assert_eq!(
                    deck.main_deck.len(),
                    99,
                    "random_deck must produce exactly 99 main-deck cards"
                );
                deck
            }
            DeckSource::Fixed(_) => fixed_decks
                .get(&pid)
                .cloned()
                .ok_or(SetupError::NoDeckForSeat { seat: pid })?,
        };

        // Architecture Invariant 9, through the real gate — not re-derived. CR 903.5a's
        // 100-card check is included, since `deck_card_ids` is main_deck + commander.
        let mut deck_card_ids = deck.main_deck.clone();
        deck_card_ids.push(deck.commander.clone());
        let result = validate_deck(&[deck.commander.clone()], &deck_card_ids, &registry, &[]);
        if !result.valid {
            return Err(SetupError::InvalidDeck {
                seat: pid,
                violations: result.violations,
            });
        }

        // CR 903.6: commander to the command zone.
        //
        // Two separate steps, and BOTH are required. Placing the object in
        // `ZoneId::Command` only puts a card there; `player_commander` is what records it
        // in `PlayerState::commander_ids`, which is the field every commander rule keys
        // off — commander tax (`rules/casting.rs`), the CR 903.9a/704.6d command-zone
        // return SBA and CR 903.10a commander damage (`rules/commander.rs`,
        // `rules/combat.rs`), and the CR 903.9b hand/library redirects registered below.
        // A game built with the object but not the registration is not a Commander game:
        // the commander is recastable for free forever and deals no commander damage.
        // (The pre-Session-2 TUI setup this module lifts had exactly that gap; it is
        // fixed here rather than carried forward. Same pairing as
        // `testing/replay_harness.rs`'s script path.)
        let commander_def = find_def(&by_card_id, pid, &deck.commander)?;
        let spec = ObjectSpec::card(pid, &commander_def.name)
            .in_zone(ZoneId::Command(pid))
            .with_card_id(deck.commander.clone());
        builder = builder.object(enrich_spec_from_def(spec, &card_defs));
        builder = builder.player_commander(pid, deck.commander.clone());

        // CR 903.6: shuffle the remaining deck; CR 103.5: the first 7 become the opening
        // hand, and the rest form the library.
        deck.main_deck.shuffle(&mut rng);
        let split_at = deck.main_deck.len().min(7);
        let (hand_cards, library_cards) = deck.main_deck.split_at(split_at);

        for card_id in hand_cards {
            let def = find_def(&by_card_id, pid, card_id)?;
            let spec = ObjectSpec::card(pid, &def.name)
                .in_zone(ZoneId::Hand(pid))
                .with_card_id(card_id.clone());
            builder = builder.object(enrich_spec_from_def(spec, &card_defs));
        }
        for card_id in library_cards {
            let def = find_def(&by_card_id, pid, card_id)?;
            let spec = ObjectSpec::card(pid, &def.name)
                .in_zone(ZoneId::Library(pid))
                .with_card_id(card_id.clone());
            builder = builder.object(enrich_spec_from_def(spec, &card_defs));
        }

        names.insert(pid, seat_name(pid, &cfg.human_seats));
    }

    builder = builder.first_turn_of_game();
    let mut state = builder.build().map_err(SetupError::Builder)?;

    // CR 903.9b: a commander that would go to its owner's hand or library may go to the
    // command zone instead. These are replacement effects, not triggers, so they must
    // exist in `state.replacement_effects` before the game starts — `GameStateBuilder`
    // does not derive them from `commander_ids` itself. Same call the script path makes
    // (`testing/replay_harness.rs`).
    register_commander_zone_replacements(&mut state);

    Ok((state, names))
}

/// CR 103.5 (mulligan) / CR 103.5c (free first mulligan in multiplayer) — a pregame
/// re-deal.
///
/// `handle_take_mulligan` (`rules/commander.rs`) is CR 103.5-faithful as of PB-DP2
/// (`scutemob-150`): it runs a real seeded `Zone::shuffle` and draws a genuinely new
/// hand. This function is **not** a workaround for that. It exists because M11-local
/// offers mulligans *before* `start_game` is ever called — no command has been issued
/// yet, so a full pregame rebuild invalidates no history — which is simpler than routing
/// an in-game `TakeMulligan` command through a game that has not started.
///
/// This performs only the "shuffle and draw a fresh 7" half of CR 103.5. Per CR 103.5,
/// after the `mulligan_count`-th mulligan the player puts `mulligan_count - 1` cards (the
/// first mulligan is free per CR 103.5c) on the bottom of their library, in any order
/// they choose. Expressing that choice needs `ActionParams` (Session 3), so it is left to
/// the caller once that lands; this function only re-deals.
///
/// Rebuilds the **whole table** — every seat, not just `seat` — perturbing the seed by
/// both `seat` and `mulligan_count` so two different seats mulliganing (or the same seat
/// mulliganing twice) never collide on an identical redeal.
///
/// # The caller must hand this a `DeckSource::Fixed` config (CR 103.5)
///
/// **A perturbed seed re-runs whatever the config describes.** With
/// `DeckSource::RandomPerSeat` that includes the deck draw itself, so every seat gets
/// a brand-new 99 *and a new commander* — the command zone is public (CR 903.6), so the
/// other three players watch their commanders change, and the mulliganing player's own
/// decklist is not the one they mulliganed. CR 103.5 makes a mulligan a permutation of a
/// **fixed** library-plus-hand multiset; replacing the multiset is not a mulligan at all.
/// (Shipped that way and found in the first human playtest — G2 of
/// `memory/playtest-triage-2026-08-02b.md`, fixed in `scutemob-187`.)
///
/// With `DeckSource::Fixed` the perturbed seed reaches only the shuffle, which *is* the
/// CR-correct behaviour: same cards, new order, new hand. So record the dealt decklists
/// once ([`dealt_decks`]) and keep them; `session::new_game` in `tools/play-server`
/// does that for every browser game, and
/// `test_redeal_preserves_every_seats_deck_and_commander` is the gate.
///
/// This function does not (and cannot) reject a `RandomPerSeat` config: `redeal` is also
/// the pregame-rebuild primitive for callers that have not started a game yet, where
/// re-rolling is harmless. The obligation is the caller's, and it is pinned by that test.
///
/// **Two remaining limitations of the whole-table shortcut**, both acceptable for the
/// M11-local v1 UX path and neither of them "safe" in the way a first draft of this
/// comment claimed:
///
/// 1. Even with `Fixed` decks it is not invisible to the other seats: every seat's library
///    is reshuffled and every seat's opening hand is redrawn, not just `seat`'s. Hidden
///    zones (Architecture Invariant 7) are unobserved, so no other player can *see* it,
///    but a bot that had already been dealt a hand is dealt a different one.
/// 2. It cannot represent a partially-decided table. CR 103.5: "Once a player chooses not
///    to take a mulligan, the remaining cards become that player's opening hand", and per
///    CR 103.5c each player has their own mulligan count. A single
///    `(seat, mulligan_count)` signature has nowhere to record that seat 2 already kept,
///    so rebuilding discards that kept hand.
///
/// Both fall out of the same simplification — one seed reproduces one whole table. A
/// per-seat mulligan state (each seat holding its own count and a `kept` flag, rebuilt
/// independently) is the shape that fixes them, and belongs with the play-server pregame
/// flow that will actually offer mulligans seat by seat. **Per-seat RNG streams alone do
/// not get there** and are deliberately not implemented here (`scutemob-187`): keying each
/// seat's shuffle on `(cfg.seed, pid)` still moves every seat when `redeal` perturbs
/// `cfg.seed`, so isolating the mulliganing seat needs the per-seat mulligan *counts* to
/// live in the config, not just a per-seat stream — and re-deriving the shuffle seed at
/// all would move the opening hands that `tools/play-server`'s `UI1_SEED`/`UI2_SEED`/
/// `SIM1_SEED` fixtures pin by original index.
pub fn redeal(
    cfg: &LocalGameConfig,
    seat: PlayerId,
    mulligan_count: u32,
) -> Result<(GameState, HashMap<PlayerId, String>), SetupError> {
    let redeal_cfg = LocalGameConfig {
        seed: redeal_seed(cfg.seed, seat, mulligan_count),
        ..cfg.clone()
    };
    build_initial_state(&redeal_cfg)
}
