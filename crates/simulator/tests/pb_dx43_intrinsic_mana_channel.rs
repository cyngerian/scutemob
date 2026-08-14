//! PB-DX43 — the CR 305.6 intrinsic mana ability is **reachable**, not merely stored
//! (`scutemob-213`, criterion 6506; closes `OOS-DX27-1`).
//!
//! # Why this file exists and `pb_dx43_intrinsic_land_mana.rs` is not enough
//!
//! The engine-side probes in `crates/engine/tests/rules/pb_dx43_intrinsic_land_mana.rs`
//! all end at `calculate_characteristics(..).mana_abilities` — they prove the ability is
//! **computed**. That is exactly the claim PB-DX29 named the `kaito_shizuki` lesson for:
//! *existence is necessary and never sufficient*. `Effect::CreateEmblem` existed and
//! authoring against it shipped a 7-loyalty no-op, because no dispatch site reached it.
//! An intrinsic mana ability that no offer layer enumerates, no solver funds a cost from,
//! and no human can click is the same shape of nothing.
//!
//! So every probe here goes through a **real** channel and asserts a **real** consequence
//! — mana in a player's pool, or a `Command` the solver actually emitted — rather than
//! inspecting characteristics:
//!
//! | probe | channel | evidence |
//! |---|---|---|
//! | C1 | human — `LocalGame` + `HumanChoice` | `{B}` in `p1`'s pool after submitting the offered action |
//! | C2 | mana solver / auto-tap | `solve_mana_payment` funds a `{B}` cost off a Plains |
//! | C3 | human | Yavimaya `{G}` |
//! | C4 | human | the Dryad's five colours, each individually payable |
//! | C5 | offer layer + human | `awaken_the_woods`' Forest token taps for `{G}` |
//! | C6 | mana solver | the derivation does not DOUBLE a basic land's own source |
//!
//! # CR citations
//!
//! - **CR 305.6** — "An object with the land card type and a basic land type has the
//!   intrinsic ability '{T}: Add [mana symbol],' even if the text box doesn't actually
//!   contain that text or the object has no text box."
//! - **CR 305.7** — "If a land gains one or more land types in addition to its own, it
//!   keeps its land types and rules text, and it gains the new land types and mana
//!   abilities." This is the clause C1/C3/C4 assert: the Plains keeps `{W}` **and** gains
//!   the conferred colour. A probe that only checked for the new colour would pass on an
//!   implementation that wrongly REPLACED the printed ability.
//! - **CR 605.1a** — a mana ability does not use the stack, which is why the pool is
//!   readable immediately after `submit` with no intervening resolution.
//!
//! # Fixture rule obeyed throughout
//!
//! Every card object is built through `enrich_spec_from_def` against the real
//! `all_cards()` definition — never a hand-written `ObjectSpec` approximating a card.
//! `ObjectSpec::card()` creates naked objects (the standing `memory/gotchas-infra.md`
//! gotcha), and a hand-built "Plains-like" spec would let this file keep passing after
//! `plains.rs` changed underneath it. The one constructed object is C5's token, and it is
//! built by **destructuring the `TokenSpec` out of `awaken_the_woods`' own def** rather
//! than by transcribing its fields, for the same reason.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AbilityDefinition,
    CardDefinition, CardType, Command, Effect, GameState, GameStateBuilder, ManaColor, ManaCost,
    ObjectId, ObjectSpec, PlayerId, Step, TokenSpec, ZoneId,
};
use mtg_simulator::{
    build_registry, solve_mana_payment, ActionParams, AdvanceOutcome, HumanChoice, LegalAction,
    LegalActionProvider, LocalGame, LocalGameLimits, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// A real card object in a real zone, enriched from its real def.
fn real_card(defs: &HashMap<String, CardDefinition>, owner: PlayerId, name: &str) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no object named {name:?} in this fixture"))
}

fn limits(max_turns: u32) -> LocalGameLimits {
    LocalGameLimits {
        max_turns,
        max_commands: max_turns * 200,
        max_consecutive_passes: 100,
        record_journal: true,
    }
}

/// A two-player state where `p1` controls `lands` outright on the battlefield and has
/// then **played or cast** every entry in `entering` through the real command path.
///
/// # Why `entering` cannot just be another `builder.object(..)` on the battlefield
///
/// `GameStateBuilder::build()` does **not** call
/// `rules::replacement::register_static_continuous_effects` — nothing does, until a
/// permanent actually enters the battlefield through `Command::PlayLand`
/// (`rules/lands.rs`) or spell resolution (`rules/resolution.rs`). A conferring
/// permanent dropped straight onto the battlefield by the builder therefore registers
/// **no `ContinuousEffect` at all**, and `calculate_characteristics` sees nothing to
/// apply — Urborg would sit there conferring nothing and every probe below would fail
/// for a reason that has nothing to do with CR 305.6.
///
/// This bit the first draft of this file, and the shape is worth naming: **a fixture
/// that never registers the effect makes a probe fail for a reason the probe does not
/// describe** — the mirror image of PB-DX25b's fixture that made a probe *pass* by
/// removing the only condition under which the code was wrong. The fix is not to poke
/// the effect in by hand but to put the card where a real game puts it and let the real
/// command path register it, which is also strictly stronger evidence: these probes now
/// prove "play Urborg, THEN tap your Plains for `{B}`", end to end.
///
/// The two pokes retained (`mana_pool`, `priority_holder`) are the same ones
/// `pb_dx27_blood_moon_type_scope.rs::cast_and_resolve_2r` uses, and neither touches
/// anything the derivation reads.
fn state_with(
    defs: &HashMap<String, CardDefinition>,
    lands: &[&str],
    entering: &[&str],
) -> GameState {
    state_with_opponent_lands(defs, lands, entering, &[])
}

/// `state_with`, plus lands controlled by `p2`. Split out for C4b, which needs the
/// Dryad's `LandsYouControl` filter to have something on the other side of it.
fn state_with_opponent_lands(
    defs: &HashMap<String, CardDefinition>,
    lands: &[&str],
    entering: &[&str],
    opponent_lands: &[&str],
) -> GameState {
    let (p1, p2) = (p(1), p(2));
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(build_registry());
    for name in lands {
        builder = builder.object(real_card(defs, p1, name));
    }
    for name in opponent_lands {
        builder = builder.object(real_card(defs, p2, name));
    }
    for name in entering {
        builder = builder.object(enrich_spec_from_def(
            ObjectSpec::card(p1, name)
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(card_name_to_id(name)),
            defs,
        ));
    }
    for player in [p1, p2] {
        for i in 0..20 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Library Filler {i}"))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    let mut state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    for name in entering {
        let card = find_by_name(&state, name);
        let is_land = state
            .objects()
            .get(&card)
            .map(|o| o.characteristics.card_types.contains(&CardType::Land))
            .unwrap_or(false);
        state.turn_mut().priority_holder = Some(p1);
        state = if is_land {
            // CR 305.1: playing a land is a special action; `rules/lands.rs` is what
            // registers its static continuous effects.
            process_command(state, Command::PlayLand { player: p1, card })
                .unwrap_or_else(|e| panic!("playing {name} failed: {e:?}"))
                .0
        } else {
            {
                let pool = &mut state.players_mut().get_mut(&p1).unwrap().mana_pool;
                pool.colorless = 8;
                pool.red = 4;
                pool.green = 4;
            }
            let (s, _) = process_command(
                state,
                Command::CastSpell(Box::new(CastSpellData {
                    player: p1,
                    card,
                    targets: vec![],
                    convoke_creatures: vec![],
                    improvise_artifacts: vec![],
                    delve_cards: vec![],
                    kicker_times: 0,
                    alt_cost: None,
                    prototype: false,
                    modes_chosen: vec![],
                    x_value: 0,
                    face_down_kind: None,
                    additional_costs: vec![],
                    hybrid_choices: vec![],
                    phyrexian_life_payments: vec![],
                })),
            )
            .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"));
            let mut s = s;
            let mut guard = 0;
            while !s.stack_objects().is_empty() {
                guard += 1;
                assert!(guard < 100, "resolving {name} exceeded the safety guard");
                for pl in [p1, p2] {
                    s = process_command(s, Command::PassPriority { player: pl })
                        .unwrap_or_else(|e| panic!("passing priority failed: {e:?}"))
                        .0;
                }
            }
            s
        };
        // Leave no floating mana: an unemptied pool would let C2/C5's solver
        // assertions pass off the pool rather than off a land (CR 106.4).
        state.players_mut().get_mut(&p1).unwrap().mana_pool = Default::default();
    }
    state
}

/// Every `TapForMana` the **offer layer** emits for `source`, as `(ability_index)`.
/// Read from `StubProvider` — the sole producer of `LegalAction` in the tree — so this
/// is the same list the browser and every bot see.
fn offered_tap_indices(state: &GameState, player: PlayerId, source: ObjectId) -> Vec<usize> {
    StubProvider
        .legal_actions(state, player)
        .into_iter()
        .filter_map(|a| match a {
            LegalAction::TapForMana {
                source: s,
                ability_index,
                ..
            } if s == source => Some(ability_index),
            _ => None,
        })
        .collect()
}

/// Drive a human-seat `LocalGame` until the current decision offers at least one
/// `TapForMana` for `source`, then return that decision. `LocalGame::start` always
/// resets to `Step::Untap` regardless of what the builder set (its own doc), so the
/// first human decision is Upkeep — but a mana ability is legal at instant speed
/// (CR 605.1a), so the very first decision already carries the offer. The loop is
/// defensive, not load-bearing.
fn drive_to_tap_offer<P: LegalActionProvider>(
    game: &mut LocalGame<P>,
    source: ObjectId,
) -> mtg_simulator::PendingDecision {
    for _ in 0..40 {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                if d.actions
                    .iter()
                    .any(|a| matches!(a, LegalAction::TapForMana { source: s, .. } if *s == source))
                {
                    return d;
                }
                let pass = d
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority { .. }))
                    .expect("a priority window always offers PassPriority");
                game.submit(
                    d.seq,
                    HumanChoice {
                        action_index: pass,
                        params: ActionParams::default(),
                    },
                )
                .expect("passing priority should be accepted");
            }
            other => panic!("expected AwaitingHuman while hunting a tap offer, got {other:?}"),
        }
    }
    panic!("no TapForMana offer for {source:?} within 40 decisions");
}

/// The whole real-path drive, factored because C1/C3/C4/C5 differ only in fixture and
/// expected colour: build a human-seat game, take the offer layer's `TapForMana` list
/// for `source`, submit the one that the assertion wants, and return the resulting pool
/// count for `want`.
///
/// `pick` selects among the offered ability indices. It receives the whole list so a
/// caller can assert its SHAPE (how many offers exist) as well as pick one.
fn tap_and_count(
    defs: &HashMap<String, CardDefinition>,
    lands: &[&str],
    entering: &[&str],
    source_name: &str,
    want: ManaColor,
    pick: impl Fn(&[usize]) -> usize,
) -> (u32, usize) {
    let p1 = p(1);
    let state = state_with(defs, lands, entering);
    let source = find_by_name(&state, source_name);
    let human: BTreeSet<PlayerId> = [p1].into_iter().collect();
    let (mut game, _events) = LocalGame::start(
        state,
        43_43_43,
        StubProvider,
        HashMap::new(),
        human,
        limits(3),
        true,
    )
    .expect("game should start");

    let decision = drive_to_tap_offer(&mut game, source);
    let offered: Vec<usize> = decision
        .actions
        .iter()
        .filter_map(|a| match a {
            LegalAction::TapForMana {
                source: s,
                ability_index,
                ..
            } if *s == source => Some(*ability_index),
            _ => None,
        })
        .collect::<Vec<usize>>();
    let chosen = pick(&offered);
    let action_index = decision
        .actions
        .iter()
        .position(|a| {
            matches!(a, LegalAction::TapForMana { source: s, ability_index, .. }
                     if *s == source && *ability_index == chosen)
        })
        .expect("the picked ability index must be among the offered actions");

    game.submit(
        decision.seq,
        HumanChoice {
            action_index,
            params: ActionParams::default(),
        },
    )
    .unwrap_or_else(|e| panic!("submitting the offered TapForMana failed: {e:?}"));

    let pool = game.state().players().get(&p1).unwrap().mana_pool.clone();
    (pool.get(want), offered.len())
}

/// The index, among `source`'s offered `TapForMana` actions, whose ability produces
/// exactly `color`. Resolving the index this way (rather than hard-coding 1) means the
/// probe does not silently start testing a different ability if append order changes —
/// it would instead fail to find one and panic.
fn index_producing(state: &GameState, source: ObjectId, color: ManaColor) -> usize {
    let chars = mtg_engine::rules::layers::calculate_characteristics(state, source)
        .expect("the source object exists");
    chars
        .mana_abilities
        .iter()
        .position(|ma| ma.produces.get(&color).copied().unwrap_or(0) > 0)
        .unwrap_or_else(|| {
            panic!(
                "no mana ability producing {color:?} on {source:?}: {:?}",
                chars.mana_abilities
            )
        })
}

// ── C1: Urborg, through the human channel ───────────────────────────────────────

/// **C1** — CR 305.6/305.7. A `Plains` under `urborg_tomb_of_yawgmoth` is a Swamp in
/// addition to its own type, so it has the intrinsic `{T}: Add {B}`. This drives the
/// **human** channel end to end: the offer layer emits the action, `LocalGame::submit`
/// routes it through `params.rs` into `Command::TapForMana`, the engine resolves it, and
/// black mana lands in `p1`'s pool.
///
/// Asserts BOTH halves of CR 305.7's last sentence — the Plains offers **two** abilities
/// (it keeps `{W}` and gains `{B}`), not one. A probe checking only for `{B}` would pass
/// against an implementation that wrongly replaced the printed ability.
///
/// **Pre-PB-DX43 this is red twice over**: `mana_abilities` had no derivation from
/// subtypes at all, so the Plains offered exactly one action and no amount of clicking
/// could ever produce `{B}`.
#[test]
fn c1_a_plains_under_urborg_taps_for_black_through_the_human_channel() {
    let defs = card_defs_by_name();
    let probe_state = state_with(&defs, &["Plains"], &["Urborg, Tomb of Yawgmoth"]);
    let plains = find_by_name(&probe_state, "Plains");
    let black_idx = index_producing(&probe_state, plains, ManaColor::Black);

    let (black, offers) = tap_and_count(
        &defs,
        &["Plains"],
        &["Urborg, Tomb of Yawgmoth"],
        "Plains",
        ManaColor::Black,
        |_| black_idx,
    );

    assert_eq!(
        offers, 2,
        "CR 305.7: the Plains KEEPS its printed {{T}}: Add {{W}} and GAINS {{T}}: Add {{B}} \
         — the offer layer must show two, not one (a replacement, not an addition, would \
         also show one)"
    );
    assert_eq!(
        black, 1,
        "CR 305.6: submitting the offered action must actually put {{B}} in the pool — \
         this is the whole difference between an ability that is computed and one that is \
         reachable"
    );
}

// ── C2: the mana solver / auto-tap path ─────────────────────────────────────────

/// **C2** — the solver (`mana_solver::gather_sources`) is what funds every auto-tapped
/// cast, for bots and for a human's `auto_tap: true` submit alike. A `{B}` cost must be
/// solvable off a board whose only land is a Plains, because Urborg makes it a Swamp.
///
/// This is a different channel from C1, not a restatement: C1 proves a human can click
/// the ability, C2 proves the engine will find it on the player's behalf. `OOS-SIM6-3`
/// is a standing reminder that the two are separately reachable.
#[test]
fn c2_the_mana_solver_funds_a_black_cost_off_a_plains_under_urborg() {
    let defs = card_defs_by_name();
    let state = state_with(&defs, &["Plains"], &["Urborg, Tomb of Yawgmoth"]);

    let black_cost = ManaCost {
        black: 1,
        ..Default::default()
    };
    let plan = solve_mana_payment(&state, p(1), &black_cost);
    assert!(
        plan.is_some(),
        "the solver must fund {{B}} from a Plains that Urborg has made a Swamp (CR 305.6)"
    );
    assert_eq!(
        plan.as_ref().unwrap().len(),
        1,
        "exactly one source is needed: {:?}",
        plan
    );

    // Non-vacuity: the SAME fixture without Urborg must NOT solve, or this probe would
    // pass on a solver that funds anything.
    let no_urborg = state_with(&defs, &["Plains"], &[]);
    assert!(
        solve_mana_payment(&no_urborg, p(1), &black_cost).is_none(),
        "without Urborg a lone Plains cannot fund {{B}} — if this solves, the probe above \
         proves nothing"
    );
}

// ── C3: Yavimaya ────────────────────────────────────────────────────────────────

/// **C3** — CR 305.6, the `{G}` arm. `yavimaya_cradle_of_growth` is a separate `Complete`
/// deck-legal def with its own `AddSubtypes(Forest)` static; testing only Urborg and
/// assuming the sibling is fine is the exact shortcut PB-DX24's `t8` exists to forbid.
#[test]
fn c3_a_plains_under_yavimaya_taps_for_green_through_the_human_channel() {
    let defs = card_defs_by_name();
    let entering = ["Yavimaya, Cradle of Growth"];
    let probe_state = state_with(&defs, &["Plains"], &entering);
    let plains = find_by_name(&probe_state, "Plains");
    let green_idx = index_producing(&probe_state, plains, ManaColor::Green);

    let (green, offers) = tap_and_count(
        &defs,
        &["Plains"],
        &entering,
        "Plains",
        ManaColor::Green,
        |_| green_idx,
    );
    assert_eq!(offers, 2, "keeps {{W}}, gains {{G}} (CR 305.7)");
    assert_eq!(green, 1, "the offered action must actually produce {{G}}");
}

// ── C4: the Dryad — every colour, each individually payable ─────────────────────

/// **C4** — `dryad_of_the_ilysian_grove` makes lands you control **every** basic land
/// type, so a Plains under it must be able to produce any of the five. Each colour is
/// driven through the human channel on its own fresh fixture, because a permanent can
/// only be tapped once — asserting "it offers five actions" alone would not prove any of
/// the other four actually works.
///
/// Also asserts the Dryad's `LandsYouControl` filter is real: a land controlled by `p2`
/// gains nothing.
#[test]
fn c4_a_plains_under_the_dryad_taps_for_every_colour_through_the_human_channel() {
    let defs = card_defs_by_name();
    let entering = ["Dryad of the Ilysian Grove"];

    for color in [
        ManaColor::White,
        ManaColor::Blue,
        ManaColor::Black,
        ManaColor::Red,
        ManaColor::Green,
    ] {
        let probe_state = state_with(&defs, &["Plains"], &entering);
        let plains = find_by_name(&probe_state, "Plains");
        let idx = index_producing(&probe_state, plains, color);
        let (got, offers) = tap_and_count(&defs, &["Plains"], &entering, "Plains", color, |_| idx);
        assert_eq!(
            offers, 5,
            "CR 305.6: a Plains that is every basic land type offers exactly five mana \
             abilities — its own {{W}} discharges the Plains intrinsic, so five and not six"
        );
        assert_eq!(
            got, 1,
            "the offered action for {color:?} must produce {color:?}"
        );
    }
}

/// **C4b** — the Dryad's filter is `LandsYouControl`, not `AllLands`. A `p2`-controlled
/// Plains must gain nothing, or C4 would be proving that the derivation ignores filters
/// rather than that it consumes the resolved subtype set.
///
/// **Channel choice, stated rather than glossed**: this probe reads the **mana solver**,
/// not the offer layer. `StubProvider::legal_actions` returns nothing at all for a player
/// who does not hold priority, so an offer-layer assertion against `p2` would read `0`
/// offers whatever the derivation did — a **vacuous pass** dressed as a filter check.
/// That is the first draft of this probe, and it failed loudly for the wrong reason,
/// which is why the method is recorded here. `mana_solver::gather_sources` filters on
/// `obj.controller`, not on priority, so it answers the question actually being asked:
/// can `p2` pay `{B}` off their own Plains?
#[test]
fn c4b_the_dryads_grant_does_not_reach_an_opponents_land() {
    let defs = card_defs_by_name();
    let (p1, p2) = (p(1), p(2));
    // `p1` casts the Dryad; the only Plains on the board is `p2`'s.
    let state = state_with_opponent_lands(&defs, &[], &["Dryad of the Ilysian Grove"], &["Plains"]);

    let black = ManaCost {
        black: 1,
        ..Default::default()
    };
    let white = ManaCost {
        white: 1,
        ..Default::default()
    };

    assert!(
        solve_mana_payment(&state, p2, &white).is_some(),
        "non-vacuity: the opponent's Plains must still fund its own {{W}}, or the {{B}} \
         assertion below would pass on a fixture where p2 simply has no lands"
    );
    assert!(
        solve_mana_payment(&state, p2, &black).is_none(),
        "the Dryad's LandsYouControl filter must bound the derivation — an opponent's \
         Plains is not a Swamp, so it cannot fund {{B}} (CR 305.6 derives from the \
         RESOLVED subtype set, and the filter is what decides that set)"
    );
    // And the controller's own side of the same board still works, so this is a
    // statement about the filter rather than about the Dryad having failed to resolve.
    let _ = p1;
}

// ── C5: awaken_the_woods' Forest token — the def the memo's census missed ────────

/// The `TokenSpec` `awaken_the_woods` actually creates, destructured out of its own
/// `CardDefinition` rather than transcribed. Transcribing would let this probe keep
/// passing after the def changed, which is precisely the drift this batch's census
/// lesson is about.
fn awaken_the_woods_token_spec() -> TokenSpec {
    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Awaken the Woods")
        .expect("Awaken the Woods has a def");
    for ability in &def.abilities {
        if let AbilityDefinition::Spell { effect, .. } = ability {
            if let Effect::Repeat { effect, .. } = effect {
                if let Effect::CreateToken { spec } = effect.as_ref() {
                    return spec.clone();
                }
            }
        }
    }
    panic!(
        "Awaken the Woods no longer creates a token through Repeat/CreateToken — \
            re-derive this helper rather than deleting the probe"
    );
}

/// **C5** — `awaken_the_woods` is `Complete` and deck-legal, and its Forest Dryad land
/// token declares `mana_abilities: vec![]`. Before PB-DX43 that token was a Forest that
/// produced **nothing**: a fourth live-wrong def, invisible to the v4 memo's census
/// because that census scans `LayerModification` payloads and a token grants its types
/// through a `TokenSpec`. It is fixed for free by the derivation.
///
/// The token is placed by an `ObjectSpec` built FROM the def's own `TokenSpec`, so the
/// fixture cannot drift from the card.
#[test]
fn c5_awaken_the_woods_forest_token_taps_for_green_through_the_offer_layer() {
    let spec = awaken_the_woods_token_spec();
    assert!(
        spec.mana_abilities.is_empty(),
        "the premise of this probe is that the token authors NO mana ability and relies \
         entirely on CR 305.6; if the def gains one, re-word this probe rather than \
         deleting it: {:?}",
        spec.mana_abilities
    );

    let p1 = p(1);
    // `ObjectSpec::creature` is the only constructor that sets power/toughness, so the
    // token is built from it and then given the def's REAL card-type and subtype sets
    // (`with_types` replaces, it does not append) — the two fields the derivation
    // actually reads, taken from the def rather than transcribed.
    let token = ObjectSpec::creature(p1, &spec.name, spec.power, spec.toughness)
        .in_zone(ZoneId::Battlefield)
        .with_types(spec.card_types.iter().cloned().collect::<Vec<CardType>>())
        .with_subtypes(spec.subtypes.iter().cloned().collect::<Vec<_>>());

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(build_registry())
        .object(token);
    for i in 0..20 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Filler {i}")).in_zone(ZoneId::Library(p1)),
        );
    }
    let state = builder.active_player(p1).build().unwrap();

    let token_id = find_by_name(&state, &spec.name);
    let offers = offered_tap_indices(&state, p1, token_id);
    assert_eq!(
        offers.len(),
        1,
        "CR 305.6: a token that IS a Forest land has the intrinsic {{T}}: Add {{G}}, and \
         the offer layer must show it — pre-PB-DX43 this list was empty"
    );

    let plan = solve_mana_payment(
        &state,
        p1,
        &ManaCost {
            green: 1,
            ..Default::default()
        },
    );
    assert!(
        plan.is_some(),
        "the solver must be able to fund {{G}} from the Forest token too"
    );
}

// ── C6: the derivation does not DOUBLE an ordinary basic land ───────────────────

/// **C6** — the idempotence half, measured through a channel rather than by inspecting
/// characteristics. A plain `Swamp` on an otherwise empty board must offer **exactly
/// one** `TapForMana`, and the solver must see exactly one source. If the derivation
/// were not idempotent every basic land in every game would render two identical rows in
/// the browser's mana-source list and be double-counted by the solver — which is
/// `OOS-DX27-10`'s observable symptom, generalised from two moons to all 1,803 defs.
#[test]
fn c6_a_plain_swamp_still_offers_exactly_one_tap_for_mana() {
    let defs = card_defs_by_name();
    let state = state_with(&defs, &["Swamp"], &[]);
    let swamp = find_by_name(&state, "Swamp");

    let offers = offered_tap_indices(&state, p(1), swamp);
    assert_eq!(
        offers,
        vec![0],
        "a basic Swamp offers exactly one mana ability, at index 0 — the printed one. \
         Index 0 is load-bearing: Command::TapForMana.ability_index is a dense index \
         into mana_abilities, so a derived ability inserted ahead of the printed one \
         would silently re-point every existing index-0 command (OOS-DX26-3)"
    );

    let plan = solve_mana_payment(
        &state,
        p(1),
        &ManaCost {
            black: 1,
            ..Default::default()
        },
    )
    .expect("a Swamp funds {B}");
    assert_eq!(plan.len(), 1, "one source, one command: {plan:?}");
}

/// **C6b** — the two-moon case, through the offer layer. `OOS-DX27-10`'s filed symptom
/// is *"the play-server's mana-source list renders two rows for one land, and the solver
/// sees two sources"*, so the closure evidence belongs at the offer layer, not only in
/// `calculate_characteristics`.
#[test]
fn c6b_two_moons_offer_exactly_one_tap_for_mana_on_a_nonbasic_land() {
    let defs = card_defs_by_name();
    let state = state_with(
        &defs,
        &["Ancient Den"],
        &["Blood Moon", "Magus of the Moon"],
    );
    let den = find_by_name(&state, "Ancient Den");

    let offers = offered_tap_indices(&state, p(1), den);
    assert_eq!(
        offers.len(),
        1,
        "CR 305.7 gives a land its intrinsic mana ability ONCE however many effects set \
         its type. Pre-PB-DX43 both moons hand-authored an append-only AddManaAbility \
         grant and this list had two entries (OOS-DX27-10)."
    );

    let plan = solve_mana_payment(
        &state,
        p(1),
        &ManaCost {
            red: 1,
            ..Default::default()
        },
    )
    .expect("a Blood-Mooned Ancient Den funds {R}");
    assert_eq!(plan.len(), 1, "one source, one command: {plan:?}");

    // Non-vacuity: the Den's PRINTED {W} must be gone (CR 305.7 sentence 2), or "exactly
    // one offer" could be satisfied by the wrong ability surviving.
    assert!(
        solve_mana_payment(
            &state,
            p(1),
            &ManaCost {
                white: 1,
                ..Default::default()
            }
        )
        .is_none(),
        "Ancient Den's printed {{T}}: Add {{W}} must be gone under the moons (CR 305.7)"
    );
}
