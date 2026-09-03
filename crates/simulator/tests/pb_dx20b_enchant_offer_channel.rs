//! PB-DX20b (`OOS-DX20-10`, HIGH; `OOS-DX20-5`) — CR 702.5a's printed Enchant line,
//! through the REAL channels.
//!
//! The engine-side probes live in
//! `crates/engine/tests/primitives/pb_dx20b_enchant_card_type_or.rs` and the corpus
//! roster in `crates/engine/tests/core/pb_dx20b_enchant_line_roster.rs`. This file
//! exists because **existence is never sufficiency** (the `kaito_shizuki` lesson,
//! PB-DX43): a restriction the engine enforces at cast time but the offer layer never
//! learns about is SR-38's defect wearing a fix's clothes — a clean offer followed by
//! a guaranteed refusal. Every probe below drives `StubProvider`'s offer layer,
//! `mtg_simulator::targeting::action_target_requirements` and
//! `LocalGame`/`HumanChoice` — the same three surfaces the browser and the bots go
//! through.
//!
//! * CR 702.5a — "Enchant [object or player]" restricts what an Aura may target.
//! * CR 303.4a — an Aura spell requires a target its Enchant ability can legally
//!   enchant.
//!
//! # The subject, and why the SET is the verdict
//!
//! `imprisoned_in_the_moon` (`Complete`, deck-legal) prints *"Enchant creature, land,
//! or planeswalker"* and declared `EnchantTarget::Permanent` until PB-DX20b — which
//! also admits artifacts, enchantments and battles. PB-DX20 made that widened offer
//! **human-reachable** in the browser, which is what turned a latent declaration bug
//! into `OOS-DX20-10`.
//!
//! So SR-38 has to hold in BOTH directions and a one-sided assertion proves neither
//! half:
//!
//! * an over-wide offer (the pre-batch defect) is caught only by asserting that the
//!   artifact and the enchantment are **absent**;
//! * an under-wide offer (the obvious over-correction — narrowing to `Creature`, which
//!   is what `kayas_ghostform` shipped for years) is caught only by asserting that all
//!   three printed-legal classes are **present**.
//!
//! Hence `c1` asserts the candidate SET, never a `>= 1` and never a `contains`.
//!
//! # Why the offer set is derived rather than read off `LegalAction`
//!
//! `LegalAction::CastSpell` carries no candidate list — it never has. Both real
//! clients derive one from the action:
//! `view.rs::action_option_view` (browser) and `targeting::plan_targets` (bots) each
//! call `action_target_requirements` and then `mtg_engine::legal_targets_per_slot`.
//! `c1` calls that same pair on the action `StubProvider` actually offered, so it
//! measures the channel rather than a re-implementation of it. The HTTP half of the
//! same claim — the wire-shaped `target_slots[0].candidates` a browser really
//! receives — is `test_dx20b_imprisoned_offer_excludes_the_artifact_over_http` in
//! `tools/play-server/src/main.rs`.
//!
//! # The board, and why each witness was chosen
//!
//! Five permanents, one per card type the pre-batch `EnchantTarget::Permanent` arm
//! admitted, all under `p1` (so `EnchantControllerConstraint::Any` — Imprisoned prints
//! no controller clause — is not silently doing the discriminating work here; the
//! controller axis is `kayas_ghostform`'s and `breath_of_fury`'s, covered by `t7`/`t8`
//! in the engine suite):
//!
//! | class | witness | why |
//! |---|---|---|
//! | Creature | `Dragonspeaker Shaman` | real, `Complete`, deck-legal; `abilities: vec![]` and its one `SpellCostModifier` is filtered to Dragon spells, so it cannot perturb the `{2}{U}` cast under test |
//! | Land | `Island` | real, `Complete`; also the `{U}` source |
//! | Planeswalker | `Chandra, Flamecaller` | real, `Complete`, deck-legal; given 4 Loyalty counters, because CR 704.5i destroys a 0-loyalty planeswalker at the first state-based check and a witness that dies before the offer is no witness |
//! | Artifact | `Sol Ring` | real, `Complete`, deck-legal — **the exclusion the HIGH is about**; also the `{C}{C}` source, so the class under exclusion is load-bearing in the fixture rather than decorative |
//! | Enchantment | `Anointed Procession` | real, `Complete`; a pure token-count replacement, inert on a board with no token creation |
//!
//! **Battle is the sixth class `EnchantTarget::Permanent` admitted and it is NOT
//! covered here, stated rather than dropped in silence**: an `all_cards()` walk at
//! HEAD returns **zero** `Complete` defs carrying `CardType::Battle`, so no real
//! deck-legal battle witness exists to put on this board. A synthetic one would test
//! `matches_filter`, which the engine suite already covers, not the channel.
//!
//! Mana: `Island` ({U}) + `Sol Ring` ({C}{C}) is exactly `{2}{U}`. `LocalGame`'s
//! `auto_tap_commands_for` finds it; nothing is poked.
//!
//! # Which probe discriminates which direction — disclosed here, not only in `memory/`
//!
//! The executed revert matrix is `scratchpad/pb-dx20b-channel-reverts.md`. Three
//! reverts were run: **R-A** (`imprisoned_in_the_moon` back to
//! `EnchantTarget::Permanent`, the pre-batch declaration), **R-B** (drop
//! `has_card_types` from `casting::enchant_filter_to_target_filter`, the engine
//! lowering) and **R-C** (`EnchantTarget::Creature`, the obvious over-correction).
//!
//! | probe | R-A | R-B | R-C |
//! |---|---|---|---|
//! | `c1` | RED | RED | RED |
//! | `c2` | green | green | RED |
//! | `c3` | green | green | RED |
//! | `c4` | RED | RED | RED |
//!
//! **`c2` and `c3` are green under R-A and R-B as STATED CONTROLS, not as gaps**, and
//! the reason is structural rather than incidental: `c2` submits only targets the
//! offer layer itself named, and `c3` submits a **Land**. Under an over-wide
//! declaration a Land is still legal and every printed-legal candidate is still
//! offered, so no over-wide revert can redden either of them whatever they assert.
//! That is exactly why R-C exists — and R-C reddens both. No row in the matrix is
//! honestly UNDISCRIMINATED.
//!
//! **`c4`'s RED is carried by assertion (1) in every column, and that is disclosed here rather
//! than left to the row to imply** (`/review` finding 6). `c4` makes three assertions; only the
//! candidate-set one moves under any revert in this table. Its assertion (2) was a tautology in
//! the first draft — Imprisoned's own Layer-4 effect makes the host a Land, so a
//! layer-resolved-types check held for any host under any declaration — and is now a real claim
//! read off the host's PRINTED types, which is a strengthening and still not a verdict: the
//! bot's host pick is unchanged by a widening (lowest `ObjectId` wins and the creature is
//! lowest), and the two plants that would move it make the bot stop casting the Aura entirely,
//! so `c4` reddens on its precondition rather than on (2). All of that was executed; `c4`'s own
//! docstring has the measurements.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, CardDefinition, CounterType, GameState,
    GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Target, ZoneId,
};
use mtg_simulator::params::{ActionParams, HumanChoice};
use mtg_simulator::targeting::action_target_requirements;
use mtg_simulator::{
    build_registry, AdvanceOutcome, Bot, HeuristicBot, LegalAction, LocalGame, LocalGameLimits,
    PendingDecision, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SEED: u64 = 20_20_20;

const AURA: &str = "Imprisoned in the Moon";
const CREATURE: &str = "Dragonspeaker Shaman";
const LAND: &str = "Island";
const PLANESWALKER: &str = "Chandra, Flamecaller";
const ARTIFACT: &str = "Sol Ring";
const ENCHANTMENT: &str = "Anointed Procession";

/// The three classes CR 702.5a's printed line admits, in board order.
const PRINTED_LEGAL: [&str; 3] = [CREATURE, LAND, PLANESWALKER];
/// The two classes the pre-PB-DX20b `EnchantTarget::Permanent` declaration wrongly
/// admitted and that a `Complete` deck-legal board can actually hold.
const PRINTED_ILLEGAL: [&str; 2] = [ARTIFACT, ENCHANTMENT];

fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 3,
        max_commands: 600,
        max_consecutive_passes: 500,
        record_journal: true,
    }
}

/// A real, `Complete` def in `zone`, built through `enrich_spec_from_def` +
/// `card_name_to_id` so it carries its printed types/abilities — never an
/// `ObjectSpec::card()` naked object (`OOS-DX47-4`).
fn real(
    defs: &HashMap<String, CardDefinition>,
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

/// `p1` holds the Aura and controls one permanent of each of the five card types
/// `EnchantTarget::Permanent` used to admit. See the module doc's table.
fn fixture() -> GameState {
    let defs = card_defs_by_name();
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(build_registry())
        .active_player(p(1))
        .object(real(&defs, p(1), AURA, ZoneId::Hand(p(1))))
        .object(real(&defs, p(1), CREATURE, ZoneId::Battlefield))
        .object(real(&defs, p(1), LAND, ZoneId::Battlefield))
        .object(
            // CR 704.5i: a planeswalker with 0 loyalty is destroyed by the first
            // state-based check. `enrich_spec_from_def` sets
            // `characteristics.loyalty`, which is NOT what CR 704.5i reads — the
            // counter is (PB-DX29's `pb_dx29_loyalty_target_surface.rs` records the
            // same distinction).
            real(&defs, p(1), PLANESWALKER, ZoneId::Battlefield)
                .with_counter(CounterType::Loyalty, 4),
        )
        .object(real(&defs, p(1), ARTIFACT, ZoneId::Battlefield))
        .object(real(&defs, p(1), ENCHANTMENT, ZoneId::Battlefield));
    for player in [p(1), p(2)] {
        for i in 0..30 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Library Filler {i}"))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    builder
        .build()
        .expect("PB-DX20b channel fixture must build")
}

fn object_named(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("'{name}' must be on the battlefield in this fixture"))
}

fn name_of(state: &GameState, id: ObjectId) -> String {
    state
        .objects()
        .get(&id)
        .map(|o| o.characteristics.name.clone())
        .unwrap_or_else(|| format!("<unknown {id:?}>"))
}

fn start_human_game() -> LocalGame<StubProvider> {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(p(2), Box::new(HeuristicBot::new(SEED, "p2".to_string())));
    let human: BTreeSet<PlayerId> = [p(1)].into_iter().collect();
    let (game, _events) =
        LocalGame::start(fixture(), SEED, StubProvider, bots, human, limits(), true)
            .expect("PB-DX20b channel game must start");
    game
}

/// Drive the human seat, passing priority, until `want` finds an action in the
/// offered list. **Panics rather than returning `None`** — a probe that silently ends
/// early is a probe that asserts nothing.
fn drive_until(
    game: &mut LocalGame<StubProvider>,
    label: &str,
    want: impl Fn(&LegalAction) -> bool,
) -> (PendingDecision, usize) {
    for _ in 0..120 {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                if let Some(i) = d.actions.iter().position(&want) {
                    return (d, i);
                }
                let pass = d
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .unwrap_or_else(|| {
                        panic!(
                            "no {label} offer and no PassPriority either: {:?}",
                            d.actions
                        )
                    });
                game.submit(
                    d.seq,
                    HumanChoice {
                        action_index: pass,
                        params: ActionParams::default(),
                    },
                )
                .expect("passing priority should be accepted");
            }
            other => panic!("expected AwaitingHuman while hunting {label}, got {other:?}"),
        }
    }
    panic!("no {label} offer within 120 human decisions");
}

/// Drive to the human's `CastSpell` offer for the Aura and return the decision, the
/// action index, and the Aura's `ObjectId`.
fn drive_to_aura_cast(game: &mut LocalGame<StubProvider>) -> (PendingDecision, usize, ObjectId) {
    // Snapshot the state BEFORE `drive_until` borrows the game mutably; the Aura's id
    // in hand does not move while priority is being passed.
    let aura = *game
        .state()
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == AURA)
        .map(|(id, _)| id)
        .expect("the Aura must be in p1's hand");
    let (decision, index) = drive_until(
        game,
        "CastSpell(Imprisoned in the Moon)",
        |a| matches!(a, LegalAction::CastSpell { card, .. } if *card == aura),
    );
    (decision, index, aura)
}

/// The candidate names the real channel offers for the Aura's single target slot.
///
/// This is the exact pair `view.rs::action_option_view` (browser) and
/// `targeting::plan_targets` (bots) call, in that order — not a re-derivation.
fn offered_candidate_names(state: &GameState, action: &LegalAction) -> Vec<String> {
    let requirements = action_target_requirements(state, action);
    assert_eq!(
        requirements.len(),
        1,
        "CR 303.4a: an Aura spell has exactly one target slot; got {requirements:?}"
    );
    let source = match action {
        LegalAction::CastSpell { card, .. } => *card,
        other => panic!("expected a CastSpell action, got {other:?}"),
    };
    let per_slot = mtg_engine::legal_targets_per_slot(state, p(1), source, &requirements);
    assert_eq!(per_slot.len(), 1, "one slot in, one slot out");
    per_slot[0]
        .iter()
        .map(|t| match t {
            Target::Object(id) => name_of(state, *id),
            Target::Player(pl) => format!("<player {}>", pl.0),
        })
        .collect()
}

#[test]
/// **c1** — CR 702.5a / CR 303.4a: the offer set is EXACTLY the printed line.
///
/// The verdict is set equality, and both directions are load-bearing:
///
/// * **over-wide** (the pre-PB-DX20b defect, `OOS-DX20-10`): `EnchantTarget::Permanent`
///   put `Sol Ring` and `Anointed Procession` in this list, and PB-DX20 made that list
///   clickable in a browser. Reverting `imprisoned_in_the_moon.rs` to `Permanent`
///   reddens this test with those two names in the diff.
/// * **under-wide**: narrowing to `EnchantTarget::Creature` (what `kayas_ghostform`
///   shipped for years — `OOS-DX20-5`) drops `Island` and `Chandra, Flamecaller`, and
///   reddens it too.
///
/// A `>= 1`, a `contains`, or a "no artifact offered" assertion would each pass on one
/// of those two broken engines. Only the SET refuses both.
fn c1_the_offer_set_is_exactly_the_printed_enchant_line() {
    let mut game = start_human_game();
    let (decision, index, _aura) = drive_to_aura_cast(&mut game);
    let names: BTreeSet<String> = offered_candidate_names(game.state(), &decision.actions[index])
        .into_iter()
        .collect();

    let expected: BTreeSet<String> = PRINTED_LEGAL.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        names, expected,
        "CR 702.5a — 'Enchant creature, land, or planeswalker'. The offer layer must \
         name exactly those three permanents on this board and nothing else. Missing \
         means the declaration is too NARROW (the OOS-DX20-5 shape); extra means it is \
         too WIDE (OOS-DX20-10, the HIGH this batch closes)."
    );

    // Stated separately rather than left implicit in the set equality, because these
    // two names ARE the HIGH and a failure message that says so is worth the line.
    for illegal in PRINTED_ILLEGAL {
        assert!(
            !names.contains(illegal),
            "CR 702.5a: '{illegal}' is not a creature, land or planeswalker, so \
             Imprisoned in the Moon may not target it — yet the offer layer named it. \
             This is OOS-DX20-10 verbatim: PB-DX20 made this offer clickable."
        );
    }

    // Non-vacuity floor: the two excluded permanents must actually BE on the
    // battlefield. Without this, deleting them from the fixture would make the
    // exclusion assertions above pass while measuring nothing.
    for illegal in PRINTED_ILLEGAL {
        let _ = object_named(game.state(), illegal);
    }
}

#[test]
/// **c2** — SR-38: no clean offer followed by a refusal. Every candidate `c1`
/// measured is submitted for real, through `LocalGame::submit`/`ActionParams`, and the
/// engine ACCEPTS it.
///
/// One fresh game per candidate, because a cast consumes the card. The offer set is
/// re-derived inside each game rather than carried across from `c1` — an id from a
/// different `LocalGame` is a different object.
fn c2_every_offered_target_is_accepted_by_the_engine() {
    for want in PRINTED_LEGAL {
        let mut game = start_human_game();
        let (decision, index, _aura) = drive_to_aura_cast(&mut game);
        let action = decision.actions[index].clone();
        let requirements = action_target_requirements(game.state(), &action);
        let source = match &action {
            LegalAction::CastSpell { card, .. } => *card,
            other => panic!("expected a CastSpell action, got {other:?}"),
        };
        let candidates =
            mtg_engine::legal_targets_per_slot(game.state(), p(1), source, &requirements);
        let target = candidates[0]
            .iter()
            .find(|t| matches!(t, Target::Object(id) if name_of(game.state(), *id) == want))
            .unwrap_or_else(|| panic!("'{want}' must be an offered candidate (see c1)"))
            .clone();

        game.submit(
            decision.seq,
            HumanChoice {
                action_index: index,
                params: ActionParams {
                    targets: vec![target],
                    // The browser sets this on every cast; without it `submit` never
                    // taps, and an `InsufficientMana` refusal would masquerade as a
                    // targeting refusal — which is the exact confusion this probe
                    // exists to rule out.
                    auto_tap: true,
                    ..ActionParams::default()
                },
            },
        )
        .unwrap_or_else(|e| {
            panic!(
                "SR-38: the offer layer named '{want}' as a legal target and the engine \
                 then REFUSED the cast: {e:?}. An offer the engine will not honour is \
                 the defect, not the fix."
            )
        });
    }
}

#[test]
/// **c3** — CR 303.4a / CR 702.5a: no printed-legal class is refused, asserted by
/// RESOLUTION EFFECT.
///
/// The LAND is the class this drives end to end, and it is the interesting one: a
/// `Land` is what the pre-batch `EnchantTarget::Permanent` and the obvious
/// over-correction `EnchantTarget::Creature` disagree about, so an under-wide fix that
/// still passes `c1`'s exclusion half fails HERE.
///
/// The verdict is `attached_to`, read off the resolved Aura permanent — never
/// "`submit` returned `Ok`". `pb_dx45_optional_cost_channel.rs` sets that standard and
/// it discriminates a real second failure mode: a cast can be accepted and the Aura
/// can still fail to attach (CR 704.5m would then bin it), which is exactly the
/// silent-fizzle shape `OOS-CARDS1-2` was filed for on Reconfigure.
fn c3_a_land_target_resolves_and_the_aura_is_attached_to_it() {
    let mut game = start_human_game();
    let (decision, index, _aura) = drive_to_aura_cast(&mut game);
    let land = object_named(game.state(), LAND);

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: index,
            params: ActionParams {
                targets: vec![Target::Object(land)],
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    )
    .expect(
        "CR 702.5a: a Land is a printed-legal target for 'Enchant creature, land, or planeswalker'",
    );

    // Pass priority until the Aura has left the stack and is a permanent.
    let mut resolved = None;
    for _ in 0..120 {
        if let Some((id, obj)) = game
            .state()
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == AURA && o.zone == ZoneId::Battlefield)
        {
            resolved = Some((*id, obj.attached_to));
            break;
        }
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                let pass = d
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .unwrap_or_else(|| panic!("no PassPriority while resolving: {:?}", d.actions));
                game.submit(
                    d.seq,
                    HumanChoice {
                        action_index: pass,
                        params: ActionParams::default(),
                    },
                )
                .expect("passing priority should be accepted");
            }
            other => panic!("unexpected outcome while resolving the Aura: {other:?}"),
        }
    }

    let (_aura_permanent, attached_to) =
        resolved.expect("the Aura must resolve onto the battlefield within 120 decisions");
    assert_eq!(
        attached_to,
        Some(land),
        "CR 303.4a/CR 702.5a: the resolved Aura must be attached to the LAND it targeted. \
         `Ok` from `submit` alone would not have caught a cast that is accepted and then \
         fails to attach."
    );
}

#[test]
/// **c4** — the BOT path, MEASURED. Two seats, both `HeuristicBot`, no human
/// anywhere, one fixed seed, the same fixture.
///
/// Three things are asserted, and the docstring says which of them move under the
/// revert and which do not, because "the bot path is unchanged" is only a useful
/// sentence if it names what was compared.
///
/// 1. **The bot's own candidate set** for the Aura cast is the same three names `c1`
///    measured. `targeting::plan_targets` calls exactly the pair this assertion
///    calls, so this is the bot's real answer space. **This MOVES under the revert**
///    (5 names instead of 3) — see `scratchpad/pb-dx20b-channel-reverts.md`.
/// 2. **CR 704.5m through the SBA, on the bot path**: the bot really casts the Aura
///    here (measured, not assumed), and the resolved Aura is still on the battlefield
///    and still attached at the halt. That is the `sba::matches_enchant_target` half
///    of the batch — PB-DX20b rewrote it to call the same lowering the cast path uses,
///    and if the two disagreed the CR 704.5m SBA would have binned this Aura to the
///    graveyard some steps after it resolved. **This is CORROBORATION, not the verdict**
///    — see the paragraph below, which is `/review` finding 6.
/// 3. **The rejection census, by class.** `rejection_count() == 0` is NOT asserted and
///    that is deliberate: this fixture reproduces a **pre-existing** SR-38 defect that
///    has nothing to do with PB-DX20b, and pinning zero would have meant either
///    deleting the witness that finds it or asserting something false. See below.
///
/// # Assertion (2) is corroboration, and the first draft of it was a TAUTOLOGY
///
/// `/review` finding 6. The first draft asserted that the host's **layer-resolved** card types
/// intersect `{Creature, Land, Planeswalker}`. Imprisoned's own Layer-4 effect makes the
/// enchanted permanent *"a colorless land"* — which is one of the three — so that assertion held
/// for **any** host under **any** declaration, including the pre-batch `EnchantTarget::Permanent`
/// this probe exists to catch. It could not fail. Worse, the docstring above narrated exactly
/// that mechanism and presented it as the reason the assertion is sound, when it is the reason
/// the assertion is empty.
///
/// It is now split into two claims that can each be false: **(2a)** the host's **printed** card
/// types are in the printed-legal set (a Sol Ring or an Anointed Procession host fails it), and
/// **(2b)** the host's layer-resolved types contain `Land`, which asserts Imprisoned's Layer 4
/// actually applied rather than leaving it to prose.
///
/// **And that is still not enough to make (2) the verdict, which is stated rather than implied.**
/// Measured, not argued — every line below was executed:
///
/// * under **R-A** (`has_card_types` emptied, i.e. the pre-batch "any permanent" reach) `c4`
///   panics at assertion **(1)**, the candidate set;
/// * with assertion (1) temporarily deleted, `c4` under R-A is **GREEN** — the bot's host pick
///   does not move, because `plan_targets` takes the first candidate, `legal_targets_per_slot`
///   walks in ascending `ObjectId`, and the creature has the lowest id on this board. A widening
///   therefore cannot change the pick at all;
/// * two further plants that *do* move the pick — declaring `[Artifact]` (host would be the
///   Sol Ring) and `[Enchantment]` (host would be the Anointed Procession) — redden `c4` on its
///   **precondition** instead: with those declarations the bot never casts the Aura, so (2a) is
///   not reached. "All rows RED" produced by the wrong assertion is PB-DX48's own finding, so
///   the panic LINE was read in each case rather than the pass/fail bit.
///
/// So the module-doc revert table's `c4: RED` is carried by **(1)** in every column, and (2a)/(2b)
/// are corroboration whose failure would be a finding but whose success is not a proof. Making
/// them the verdict needs a fixture whose lowest-`ObjectId` battlefield permanent is *not*
/// printed-legal, which this board cannot be — the artifact is also its `{C}{C}` mana source.
///
/// # The pre-existing defect this fixture reproduces (`legal_actions.rs:1276`)
///
/// Once the Aura resolves onto the Dragonspeaker Shaman, that permanent is a Land and
/// is no longer a creature. The `DeclareAttackers` offer loop nevertheless keeps
/// offering it, because it filters on `obj.characteristics.card_types` — the **raw,
/// printed** characteristics — and never on `calculate_characteristics`, so a Layer-4
/// type change is invisible to it. The bot attacks with it and the engine refuses with
/// `InvalidCommand("Object ObjectId(N) is not a creature")`. `status.tapped`,
/// `KeywordAbility::Defender` and `KeywordAbility::Haste` are read from the same raw
/// struct three lines away, so a granted Defender is equally invisible.
///
/// This is a *pre-PB-DX20b* defect on both halves of the pair: the widened
/// `EnchantTarget::Permanent` also admitted creatures, and the bot picks the first
/// candidate either way. It is **reported, not fixed and not asserted away** — the
/// probe pins the class and the count so a *different* class of refusal, or more of
/// this one, reddens rather than passing in silence.
fn c4_the_bot_path_sees_the_same_offer_set_and_its_aura_stays_attached() {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(p(1), Box::new(HeuristicBot::new(SEED, "p1".to_string())));
    bots.insert(
        p(2),
        Box::new(HeuristicBot::new(SEED + 1, "p2".to_string())),
    );
    let (mut game, _events) = LocalGame::start(
        fixture(),
        SEED,
        StubProvider,
        bots,
        BTreeSet::new(),
        limits(),
        true,
    )
    .expect("PB-DX20b bot game must start");

    // (1) The bot's own view of the Aura cast, taken from the real offer layer before
    // the game is driven — the same two calls `plan_targets` makes internally.
    let aura_in_hand = *game
        .state()
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == AURA)
        .map(|(id, _)| id)
        .expect("the Aura must be in p1's hand");
    let action = LegalAction::CastSpell {
        card: aura_in_hand,
        from_zone: ZoneId::Hand(p(1)),
        additional_costs: Default::default(),
        alt_cost: None,
    };
    let bot_candidates: BTreeSet<String> = offered_candidate_names(game.state(), &action)
        .into_iter()
        .collect();
    let expected: BTreeSet<String> = PRINTED_LEGAL.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        bot_candidates, expected,
        "CR 702.5a: the bot path derives its announcement from the same \
         `legal_targets_per_slot` the browser does, so it must see the same three names"
    );

    // Drive the whole game to its halt. `advance()` with no human seat runs everything
    // internally, which is what makes the census below a whole-game measurement.
    let outcome = game.advance();
    assert!(
        matches!(
            outcome,
            AdvanceOutcome::Halted(_) | AdvanceOutcome::GameOver(_)
        ),
        "a game with no human seat must not stop on a human decision: {outcome:?}"
    );

    // (2) CR 704.5m on the bot path, asserted by RESOLUTION EFFECT.
    let (_, aura_permanent) = game
        .state()
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == AURA && o.zone == ZoneId::Battlefield)
        .map(|(id, o)| (*id, o.clone()))
        .expect(
            "precondition: the bot must actually have CAST the Aura for this probe to \
             measure anything. If this fires, the bot's scoring changed and the whole \
             census below is vacuous — re-measure, do not re-tune.",
        );
    let host = aura_permanent
        .attached_to
        .expect("CR 704.5m: an Aura on the battlefield attached to nothing is binned by the SBA");
    // (2a) The Enchant restriction, asserted on the host's PRINTED card types.
    //
    // `/review` finding 6: the first draft asserted that the host's LAYER-RESOLVED types
    // intersect {Creature, Land, Planeswalker}, and that assertion is near-tautological —
    // Imprisoned's own Layer-4 effect makes the enchanted permanent a **Land**, which is one of
    // the three, so it held for ANY host under ANY declaration, including the pre-batch
    // `EnchantTarget::Permanent`. Read off the base `Characteristics` instead: the layer walk
    // never writes them back, so this is what the card is printed as, and it is a claim that can
    // be false — a Sol Ring or an Anointed Procession host fails it.
    let printed_types: Vec<String> = game
        .state()
        .objects()
        .get(&host)
        .expect("the enchanted permanent must still exist")
        .characteristics
        .card_types
        .iter()
        .map(|t| format!("{t:?}"))
        .collect();
    assert!(
        printed_types
            .iter()
            .any(|t| t == "Creature" || t == "Land" || t == "Planeswalker"),
        "CR 702.5a/704.5m: the bot-cast Aura is attached to '{}', whose PRINTED card types are \
         {printed_types:?} — none of which its printed Enchant line \"Enchant creature, land, \
         or planeswalker\" admits. `sba::enchant_filter_matches` and \
         `casting::enchant_filter_to_target_filter` have diverged.",
        name_of(game.state(), host)
    );

    // (2b) …and the reason the attachment SURVIVES the CR 704.5m sweep, asserted rather than
    // narrated: Imprisoned's own Layer-4 effect really did make the host a Land (CR 613.1d).
    // Without this the pair reads as "the SBA happened not to fire"; with it, the CR subtlety
    // that makes a correct engine keep this attachment is measured on the bot path.
    let host_types: Vec<String> = mtg_engine::calculate_characteristics(game.state(), host)
        .expect("the enchanted permanent must still exist")
        .card_types
        .iter()
        .map(|t| format!("{t:?}"))
        .collect();
    assert!(
        host_types.iter().any(|t| t == "Land"),
        "CR 613.1d: Imprisoned in the Moon makes the enchanted permanent \"a colorless land\", \
         so the host's LAYER-RESOLVED types must contain Land — that is why CR 704.5m leaves \
         the Aura attached to a permanent whose printed type is Creature. Got {host_types:?} \
         for '{}'.",
        name_of(game.state(), host)
    );

    // (3) The rejection census, by class. See the docstring.
    let classes: BTreeSet<String> = game
        .rejections()
        .iter()
        .map(|r| {
            r.error
                .split_once('(')
                .map(|(head, _)| head.to_string())
                .unwrap_or_else(|| r.error.clone())
        })
        .collect();
    assert!(
        game.rejections()
            .iter()
            .all(|r| format!("{:?}", r.command).contains("DeclareAttackers")),
        "every refusal on this board is expected to be the pre-existing \
         `legal_actions.rs:1276` raw-characteristics DeclareAttackers defect (see this \
         test's docstring). A refusal of any OTHER command shape on a board whose only \
         non-land action is casting this Aura would be PB-DX20b's own: {:?}",
        game.rejections()
    );
    assert_eq!(
        classes.len(),
        1,
        "exactly one refusal CLASS is expected here: {classes:?} / {:?}",
        game.rejections()
    );
    assert_eq!(
        game.rejection_count(),
        2,
        "measured at HEAD: two `DeclareAttackers` refusals (turns 1 and 3), both the \
         pre-existing raw-characteristics defect. A move here is a finding to report, \
         not a number to re-tune: {:?}",
        game.rejections()
    );
}
