//! PB-DX42b (`OOS-ADJ-1` ≡ `OOS-DX19-2`) — the layer-bounded Metalcraft condition,
//! through the REAL channels.
//!
//! The engine-side probes live in `crates/engine/tests/primitives/
//! pb_dx42b_layer_bounded_conditions.rs` (a sibling agent's file, not this one).
//! This file exists because **existence is never sufficiency** (the
//! `kaito_shizuki` lesson, PB-DX43): a layer-bounded condition evaluator the
//! engine can now express correctly is not a repaired card until a real client can
//! actually drive the two cards this batch's supply census names and observe the
//! CR-correct outcome through a genuine cast/ETB, not a hand-built
//! `ContinuousEffect`.
//!
//! `plan §5` items 2 and 3: `indomitable_archangel`'s Metalcraft
//! ("Artifacts you control have shroud as long as you control three or more
//! artifacts", an ability word governed by CR 604.2 -- see this batch's engine
//! test file for why "CR 702.45a (Metalcraft)" is a wrong citation) is the
//! DEMAND side; `blinkmoth_nexus` (animated into an artifact via its own
//! activated ability, CR 613.1d Layer 4) and `eaten_by_piranhas` (an Aura that
//! REMOVES the Artifact card type via `LayerModification::SetCardTypes`, also
//! Layer 4) are the two directions of the SUPPLY side
//! (`pb-DX42b-stage0-census.md` §3b).
//!
//! # `OOS-DX43-6`, obeyed
//!
//! `GameStateBuilder::build()` registers NO static continuous effects, so a
//! conferring permanent placed straight on the battlefield confers nothing. Only
//! `indomitable_archangel` and `eaten_by_piranhas` carry a static ability whose
//! REGISTRATION matters here, so both are cast for real off `LocalGame`/
//! `HumanChoice` and let resolve through the ordinary ETB path
//! (`rules::resolution.rs`'s `register_static_continuous_effects` call), never
//! placed pre-built on the battlefield. `blinkmoth_nexus` and the three vanilla
//! artifact creatures (`Ornithopter`/`Memnite`/`Universal Automaton`) carry no
//! `AbilityDefinition::Static` of their own -- Nexus's animation is an
//! `Effect::ApplyContinuousEffect` produced at ACTIVATED-ABILITY RESOLUTION, which
//! works identically regardless of how Nexus reached the battlefield, and the
//! three vanilla creatures are just candidates whose printed card type is being
//! counted -- so builder placement is correct for all four.
//!
//! # Ordering, and why it is load-bearing in the OVER-count fixture
//!
//! `Eaten by Piranhas` is cast BEFORE `Indomitable Archangel` in the OVER-count
//! fixture. Casting the Archangel first would put three REAL artifact creatures
//! (Ornithopter/Memnite/Universal Automaton) on the board with Metalcraft already
//! ON, granting Shroud to all three -- including Ornithopter, the very creature
//! Eaten by Piranhas needs to enchant. CR 702.18a's Shroud blocks the controller's
//! OWN spells and abilities too, so that ordering would make the fixture
//! unbuildable, not merely awkward.
//!
//! # Outcome, not offer
//!
//! Both probes assert the RESOLUTION of an actual `game.submit(..)` call for a
//! REAL targeted spell (`Broken Bond`, "Destroy target artifact or enchantment",
//! oracle text verified via `lookup_card` before this file was written) --
//! `LocalGameError::Rejected(_)` under the shroud (item 2) or `Ok(_)` once the
//! Shroud correctly stops applying (item 3) -- never merely that an action was
//! listed among the offer's candidates.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, CardDefinition, GameState, GameStateBuilder,
    ObjectId, ObjectSpec, PlayerId, Target, ZoneId,
};
use mtg_simulator::params::{ActionParams, HumanChoice};
use mtg_simulator::{
    build_registry, AdvanceOutcome, Bot, LegalAction, LocalGame, LocalGameError, LocalGameLimits,
    PendingDecision, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SEED: u64 = 42_42_42;

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

/// A real card, from hand, via `enrich_spec_from_def` -- the object carries its
/// real `card_id` so the ETB registrar and the offer layer both see it correctly.
fn hand_card(owner: PlayerId, name: &str, defs: &HashMap<String, CardDefinition>) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(ZoneId::Hand(owner))
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

/// A real card, already on the battlefield, via `enrich_spec_from_def` -- for the
/// permanents this file's own module doc says do NOT need a real cast (no
/// `AbilityDefinition::Static` of their own).
fn battlefield_card(
    owner: PlayerId,
    name: &str,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

fn add_library_filler(mut builder: GameStateBuilder, players: &[PlayerId]) -> GameStateBuilder {
    for player in players {
        for i in 0..15 {
            builder = builder.object(
                ObjectSpec::card(
                    *player,
                    &format!("PB-DX42b Library Filler {i} ({player:?})"),
                )
                .in_zone(ZoneId::Library(*player)),
            );
        }
    }
    builder
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found in state"))
}

/// Every object on the battlefield named `name` -- for the two identically-named
/// `ObjectSpec::artifact` neighbours in the UNDER-count fixture.
fn find_all_named(state: &GameState, name: &str) -> Vec<ObjectId> {
    state
        .objects()
        .iter()
        .filter(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .collect()
}

fn start_human_game(fixture: GameState) -> LocalGame<StubProvider> {
    let bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    let human: BTreeSet<PlayerId> = [p(1), p(2)].into_iter().collect();
    let (game, _events) =
        LocalGame::start(fixture, SEED, StubProvider, bots, human, limits(), true)
            .expect("PB-DX42b channel game must start");
    game
}

/// Drive human seats, passing priority for whichever player is asked, until
/// `want` finds an action in the offered list. Player-agnostic on purpose
/// (`pb_dx52_stack_target_channel.rs`'s idiom): `want`'s own predicate is what
/// disambiguates which seat's turn it is to act. Panics rather than returning
/// `None` -- a probe that silently ends early asserts nothing.
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

/// Pass priority for whichever player is asked until the stack is empty.
fn resolve_stack_fully(game: &mut LocalGame<StubProvider>) {
    for _ in 0..40 {
        if game.state().stack_objects().is_empty() {
            return;
        }
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                let pass = d
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .unwrap_or_else(|| {
                        panic!("no PassPriority while resolving the stack: {:?}", d.actions)
                    });
                game.submit(
                    d.seq,
                    HumanChoice {
                        action_index: pass,
                        params: ActionParams::default(),
                    },
                )
                .expect("pass should be accepted");
            }
            other => panic!("expected AwaitingHuman while resolving the stack, got {other:?}"),
        }
    }
    panic!("stack did not resolve within 40 human decisions");
}

/// Cast `card_name` (already in p1's hand) with `targets`, auto-tapping the cost.
fn cast(game: &mut LocalGame<StubProvider>, card_name: &str, targets: Vec<Target>) {
    let card_id = find_obj(game.state(), card_name);
    let (decision, idx) = drive_until(
        game,
        &format!("CastSpell({card_name})"),
        |a| matches!(a, LegalAction::CastSpell { card, .. } if *card == card_id),
    );
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                targets,
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    )
    .unwrap_or_else(|e| panic!("casting {card_name} should be accepted: {e:?}"));
}

// ── item 2: UNDER-count, real channel ────────────────────────────────────────

/// `Indomitable Archangel` (cast for real) + two plain artifacts + `Blinkmoth
/// Nexus` (animated for real, via its OWN `{1}` activated ability) = three real
/// CR 613.1d artifacts. The old ambient-depth-counter deviation would have read
/// the Nexus's BASE characteristics (a bare Land) for the Metalcraft candidate
/// check, undercounting to two and leaving Metalcraft OFF.
fn under_count_fixture() -> GameState {
    let defs = card_defs_by_name();
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(build_registry())
        .active_player(p(1))
        .object(hand_card(p(1), "Indomitable Archangel", &defs))
        .object(hand_card(p(1), "Broken Bond", &defs))
        .object(battlefield_card(p(1), "Blinkmoth Nexus", &defs))
        .object(ObjectSpec::artifact(p(1), "PB-DX42b Under Artifact"))
        .object(ObjectSpec::artifact(p(1), "PB-DX42b Under Artifact"));
    for _ in 0..6 {
        builder = builder.object(battlefield_card(p(1), "Plains", &defs));
    }
    for _ in 0..2 {
        builder = builder.object(battlefield_card(p(1), "Forest", &defs));
    }
    builder = add_library_filler(builder, &[p(1), p(2)]);
    builder
        .build()
        .expect("PB-DX42b UNDER-count fixture must build")
}

/// **c1** -- CR 613.1d / CR 604.2 / CR 702.18a: with the real, layer-resolved
/// count of artifacts at three (via a real activated-ability animation, not a
/// hand-built `ContinuousEffect`), Metalcraft is ON and a targeted spell aimed at
/// one of p1's OWN plain artifacts is REFUSED by the engine.
#[test]
fn c1_animated_nexus_completes_metalcraft_and_the_engine_refuses_the_target() {
    let mut game = start_human_game(under_count_fixture());

    cast(&mut game, "Indomitable Archangel", vec![]);
    resolve_stack_fully(&mut game);

    let nexus_id = find_obj(game.state(), "Blinkmoth Nexus");

    // Fund the Nexus's `{1}` animate ability from the mana pool: tap ONE land
    // (any land, since the cost is fully generic) via the ordinary human
    // `TapForMana` channel.
    let (decision, tap_idx) = drive_until(&mut game, "TapForMana", |a| {
        matches!(a, LegalAction::TapForMana { .. })
    });
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: tap_idx,
            params: ActionParams::default(),
        },
    )
    .expect("tapping a land for the Nexus's activation should be accepted");

    // `Characteristics.mana_abilities` and `Characteristics.activated_abilities`
    // are TWO SEPARATE arrays -- the Nexus's `{T}: Add {C}` mana ability lives in
    // the former (offered as `TapForMana`, already handled above) and is NOT a
    // member of `activated_abilities` at all. `activated_abilities[0]` is
    // therefore the animate ability itself ("{1}: Becomes a 1/1 Blinkmoth
    // artifact creature ..."), verified directly against the def's cost
    // (`generic: 1`, no tap) before this file was finalized; `activated_
    // abilities[1]` is the target-requiring pump ability.
    let (decision, act_idx) = drive_until(&mut game, "ActivateAbility(Blinkmoth Nexus, 0)", |a| {
        matches!(a, LegalAction::ActivateAbility { source, ability_index, .. }
            if *source == nexus_id && *ability_index == 0)
    });
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: act_idx,
            params: ActionParams::default(),
        },
    )
    .expect("activating the Nexus's animate ability should be accepted");
    resolve_stack_fully(&mut game);

    // Precondition, asserted before the discriminating step: the Nexus really is
    // a layer-resolved artifact now (CR 613.1d), so this probe's premise holds.
    let nexus_chars = mtg_engine::calculate_characteristics(game.state(), nexus_id)
        .expect("the Nexus is live on the battlefield");
    assert!(
        nexus_chars
            .card_types
            .contains(&mtg_engine::CardType::Artifact),
        "precondition: the Nexus's own animate ability must make it an artifact; \
         got {:?}",
        nexus_chars.card_types
    );

    let plain_artifacts = find_all_named(game.state(), "PB-DX42b Under Artifact");
    assert_eq!(
        plain_artifacts.len(),
        2,
        "test bug: the UNDER-count fixture must place exactly two plain artifacts"
    );
    let target = plain_artifacts[0];

    // The discriminating step: cast Broken Bond ("Destroy target artifact or
    // enchantment") at one of p1's OWN plain artifacts. Two plain artifacts plus
    // the animated Nexus is three real CR 613.1d artifacts -- Metalcraft is ON
    // and Shroud must refuse this target.
    let broken_bond_id = find_obj(game.state(), "Broken Bond");
    let (decision, cast_idx) = drive_until(
        &mut game,
        "CastSpell(Broken Bond)",
        |a| matches!(a, LegalAction::CastSpell { card, .. } if *card == broken_bond_id),
    );
    let result = game.submit(
        decision.seq,
        HumanChoice {
            action_index: cast_idx,
            params: ActionParams {
                targets: vec![Target::Object(target)],
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    );
    match result {
        Err(LocalGameError::Rejected(err)) => {
            eprintln!("PB-DX42b c1: engine correctly rejected the shrouded target: {err:?}");
        }
        Err(other) => panic!(
            "expected LocalGameError::Rejected (CR 702.18a shroud), got a \
             different error: {other:?}"
        ),
        Ok(events) => panic!(
            "CR 613.1d/CR 604.2/CR 702.18a: with THREE real artifacts (two plain \
             plus the animated Nexus), Metalcraft must be ON and casting Broken \
             Bond at a shrouded artifact must be REFUSED. It was accepted \
             instead -- events: {events:?}"
        ),
    }
}

// ── item 3: OVER-count, same channel ────────────────────────────────────────

/// `Eaten by Piranhas` removes the Artifact card type from `Ornithopter`, so the
/// real artifact count among p1's three printed artifact creatures drops to two
/// (`Memnite`, `Universal Automaton`) -- Metalcraft OFF. The old ambient-depth-
/// counter deviation would have read Ornithopter's BASE characteristics (still
/// Artifact + Creature, ignoring the Aura's Layer-4 removal) for the Metalcraft
/// candidate check, overcounting to three and wrongly turning Metalcraft ON.
fn over_count_fixture() -> GameState {
    let defs = card_defs_by_name();
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(build_registry())
        .active_player(p(1))
        .object(hand_card(p(1), "Indomitable Archangel", &defs))
        .object(hand_card(p(1), "Eaten by Piranhas", &defs))
        .object(hand_card(p(1), "Broken Bond", &defs))
        .object(battlefield_card(p(1), "Ornithopter", &defs))
        .object(battlefield_card(p(1), "Memnite", &defs))
        .object(battlefield_card(p(1), "Universal Automaton", &defs));
    for _ in 0..5 {
        builder = builder.object(battlefield_card(p(1), "Plains", &defs));
    }
    for _ in 0..3 {
        builder = builder.object(battlefield_card(p(1), "Island", &defs));
    }
    for _ in 0..3 {
        builder = builder.object(battlefield_card(p(1), "Forest", &defs));
    }
    builder = add_library_filler(builder, &[p(1), p(2)]);
    builder
        .build()
        .expect("PB-DX42b OVER-count fixture must build")
}

/// **c2** -- CR 613.1d / CR 604.2 / CR 702.18a: with a real artifact creature's
/// printed card type genuinely removed by a resolved Aura (not a hand-built
/// `ContinuousEffect`), the real artifact count is two, Metalcraft is OFF, and a
/// targeted spell aimed at one of p1's REMAINING plain artifacts is ACCEPTED.
#[test]
fn c2_eaten_by_piranhas_breaks_metalcraft_and_the_engine_accepts_the_target() {
    let mut game = start_human_game(over_count_fixture());

    // Cast Eaten by Piranhas FIRST -- see this file's module doc for why: casting
    // the Archangel first would grant Shroud to all three real artifact
    // creatures on the board (including Ornithopter), and CR 702.18a's Shroud
    // blocks the controller's OWN spells too, making Ornithopter unenchantable.
    let ornithopter_id = find_obj(game.state(), "Ornithopter");
    cast(
        &mut game,
        "Eaten by Piranhas",
        vec![Target::Object(ornithopter_id)],
    );
    resolve_stack_fully(&mut game);

    // Precondition: the Aura really did strip the Artifact type (CR 613.1d,
    // Layer 4) via a REAL resolution, not a hand-built ContinuousEffect.
    let ornithopter_chars = mtg_engine::calculate_characteristics(game.state(), ornithopter_id)
        .expect("Ornithopter is live on the battlefield");
    assert!(
        !ornithopter_chars
            .card_types
            .contains(&mtg_engine::CardType::Artifact),
        "precondition: Eaten by Piranhas must remove the Artifact card type; got \
         {:?}",
        ornithopter_chars.card_types
    );
    assert!(
        ornithopter_chars
            .card_types
            .contains(&mtg_engine::CardType::Creature),
        "precondition: Eaten by Piranhas leaves the enchanted permanent a \
         creature; got {:?}",
        ornithopter_chars.card_types
    );

    cast(&mut game, "Indomitable Archangel", vec![]);
    resolve_stack_fully(&mut game);

    let memnite_id = find_obj(game.state(), "Memnite");

    // The discriminating step: cast Broken Bond at Memnite, one of the two REAL
    // remaining artifacts. Two real artifacts is below Metalcraft's threshold of
    // three -- Metalcraft is OFF and the cast must be ACCEPTED.
    let broken_bond_id = find_obj(game.state(), "Broken Bond");
    let (decision, cast_idx) = drive_until(
        &mut game,
        "CastSpell(Broken Bond)",
        |a| matches!(a, LegalAction::CastSpell { card, .. } if *card == broken_bond_id),
    );
    let events = game
        .submit(
            decision.seq,
            HumanChoice {
                action_index: cast_idx,
                params: ActionParams {
                    targets: vec![Target::Object(memnite_id)],
                    auto_tap: true,
                    ..ActionParams::default()
                },
            },
        )
        .unwrap_or_else(|e| {
            panic!(
                "CR 613.1d/CR 604.2/CR 702.18a: with only TWO real artifacts \
                 (Ornithopter no longer counts), Metalcraft must be OFF and \
                 casting Broken Bond at Memnite must be ACCEPTED. It was refused \
                 instead: {e:?}"
            )
        });
    assert!(
        events.iter().any(
            |e| matches!(e, mtg_engine::GameEvent::SpellCast { player, .. } if *player == p(1))
        ),
        "the cast must have actually been announced, not merely accepted with no \
         effect -- events: {events:?}"
    );

    // The RESOLUTION effect, not merely the acceptance: pass the cast through to
    // resolution and confirm Memnite is actually destroyed.
    resolve_stack_fully(&mut game);
    assert!(
        game.state().objects().get(&memnite_id).is_none()
            || game
                .state()
                .objects()
                .get(&memnite_id)
                .map(|o| o.zone != ZoneId::Battlefield)
                .unwrap_or(true),
        "Broken Bond's resolution effect must actually destroy Memnite (CR \
         701.8), proving the cast was not merely accepted but fully carried \
         through"
    );
}
