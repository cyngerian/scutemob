//! PB-DX55 Half 3 (`OOS-SIM5-5`) — CR 700.2a, through the REAL bot channel:
//! `StubProvider::legal_actions` (the offer layer) -> `targeting::plan_targets`
//! (per-mode target announcement) -> `params::action_to_command_with_params` (the
//! `Command` a bot actually submits) -> `process_command` -> real resolution.
//!
//! The engine-side per-mode query correctness lives in
//! `crates/engine/tests/primitives/pb_dx55_modal_activated_targets.rs`; the corpus
//! roster and mechanism gate are `core::pb_dx55_modal_activated_roster` and
//! `core::pb_dx55_per_mode_slicer_ratchet`. This file exists because **existence is
//! never sufficiency** (the `kaito_shizuki` lesson, PB-DX43): before this batch,
//! `queries::ability_target_requirements` returned `vec![]` for all three corpus
//! modal activated abilities on every board, so `targeting::plan_targets` returned
//! `TargetPlan::NotTargeted` and `params.rs` still filled `modes_chosen: [0]`
//! (`legal_actions::ability_default_modes`'s pre-PB-DX55 declared-order default) --
//! the engine then refused with *"modal spell with per-mode targets requires exactly
//! 1 target(s) for the chosen mode(s) but got 0 (CR 700.2c)"*, the largest single
//! refusal class measured at HEAD (22 of 70 on the seed-0/7/42 A/B, `sim5_bot_cast_
//! discipline`).
//!
//! Every assertion below is on the RESOLUTION EFFECT (an object destroyed, or a
//! creature's layer-resolved power/toughness changed), never on the command
//! returning `Ok(_)` alone, per `pb_dx48_ward_channel.rs`'s standard.
use std::collections::HashMap;

use mtg_engine::{
    all_cards, calculate_characteristics, card_name_to_id, enrich_spec_from_def, process_command,
    ActivatedAbility, ActivationCost, CardDefinition, CardRegistry, Command, Effect, GameEvent,
    GameState, GameStateBuilder, ManaColor, ModeSelection, ObjectId, ObjectSpec, PlayerId, Step,
    Target, TargetFilter, TargetRequirement, ZoneId,
};
use mtg_simulator::legal_actions::LegalActionProvider;
use mtg_simulator::params::{action_to_command_with_params, ActionParams};
use mtg_simulator::targeting::{plan_targets, TargetPlan};
use mtg_simulator::{LegalAction, StubProvider};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found on the battlefield"))
}

fn is_on_battlefield(state: &GameState, id: ObjectId) -> bool {
    state
        .objects()
        .get(&id)
        .is_some_and(|o| o.zone == ZoneId::Battlefield)
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

fn modal_ability_index(state: &GameState, id: ObjectId) -> usize {
    calculate_characteristics(state, id)
        .expect("layer-resolved characteristics")
        .activated_abilities
        .iter()
        .position(|a| a.modes.is_some())
        .unwrap_or_else(|| panic!("object {id:?} has no modal activated ability"))
}

fn activate_action_for(actions: &[LegalAction], source: ObjectId) -> LegalAction {
    actions
        .iter()
        .find(|a| matches!(a, LegalAction::ActivateAbility { source: s, .. } if *s == source))
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "no ActivateAbility offer for source {source:?} -- SR-38: the offer \
                 and the engine must agree, and neither offered nor accepted is the \
                 exact failure this file exists to close. Full offer list: {actions:?}"
            )
        })
}

/// p1 controls Cankerbloom + `mana` floating generic mana; p2 controls whichever of
/// Sol Ring (artifact) / Anointed Procession (enchantment) `with_artifact` /
/// `with_enchantment` requests -- both real, `Complete`, deck-legal corpus defs.
fn setup_cankerbloom(
    mana: u32,
    with_artifact: bool,
    with_enchantment: bool,
) -> (GameState, ObjectId, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let subject = enrich_spec_from_def(
        ObjectSpec::card(p1, "Cankerbloom")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Cankerbloom")),
        &defs,
    );

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(subject)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    if with_artifact {
        builder = builder.object(enrich_spec_from_def(
            ObjectSpec::card(p2, "Sol Ring")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(card_name_to_id("Sol Ring")),
            &defs,
        ));
    }
    if with_enchantment {
        builder = builder.object(enrich_spec_from_def(
            ObjectSpec::card(p2, "Anointed Procession")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(card_name_to_id("Anointed Procession")),
            &defs,
        ));
    }

    let mut state = builder.build().expect("state builds");
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, mana);
    state.turn_mut().priority_holder = Some(p1);

    let source = find_object(&state, "Cankerbloom");
    (state, source, p1, p2)
}

/// c1 (the headline AC) -- a bot announces and the engine ACCEPTS a per-mode target
/// on Cankerbloom's real modal activated ability, reached ENTIRELY through the offer
/// layer, `targeting::plan_targets` and `action_to_command_with_params` -- never a
/// hand-built `Command`. Board carries an artifact only, so mode 0 (destroy target
/// artifact) is the legal default (checked first, in declared order).
#[test]
fn c1_bot_channel_announces_and_the_engine_accepts_a_per_mode_target() {
    let (state, source, p1, p2) = setup_cankerbloom(1, true, false);
    let sol_ring = find_object(&state, "Sol Ring");

    let actions = StubProvider.legal_actions(&state, p1);
    let action = activate_action_for(&actions, source);

    let plan = plan_targets(&state, p1, &action);
    let TargetPlan::Announce(targets) = &plan else {
        panic!(
            "CR 700.2c/700.2f: the bot's targeting plan must be a real announcement, \
             not {plan:?} -- this is `OOS-SIM5-5` verbatim: the pre-PB-DX55 query \
             returned `vec![]` for every mode of this ability, so this arm would have \
             been `NotTargeted`"
        );
    };
    assert_eq!(
        targets,
        &vec![Target::Object(sol_ring)],
        "the bot must announce the Sol Ring for mode 0's TargetArtifact requirement"
    );

    let params = ActionParams {
        targets: plan.announced(),
        ..ActionParams::default()
    };
    let command = action_to_command_with_params(&state, p1, &action, &params)
        .expect("building the Command from the bot's own announced targets must succeed");
    let Command::ActivateAbility { modes_chosen, .. } = &command else {
        panic!("expected an ActivateAbility command, got {command:?}");
    };
    assert_eq!(
        modes_chosen,
        &vec![0],
        "CR 700.2a: mode 0 is the first legal mode (an artifact exists) and must be \
         the bot's default -- `legal_actions::ability_default_modes`"
    );

    let (state, _) =
        process_command(state, command).expect("the engine must accept the bot's own announcement");
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        !is_on_battlefield(&state, sol_ring),
        "CR 700.2c: mode 0's DestroyPermanent effect must actually have resolved"
    );
    assert!(events.iter().any(
        |e| matches!(e, GameEvent::PermanentDestroyed { object_id, .. } if *object_id == sol_ring)
    ));
}

/// c2 -- CR 700.2a's own example clause, through the SAME channel: *"If one of the
/// modes would be illegal (due to an inability to choose legal targets, for
/// example), that mode can't be chosen."* Board has an enchantment and NO artifact,
/// so mode 0 (destroy target artifact) has zero candidates and
/// `ability_default_modes` must fall through to mode 1 (destroy target
/// enchantment) -- the first mode that DOES have a legal candidate -- rather than
/// defaulting to mode 0 and leaving the bot unable to announce anything.
#[test]
fn c2_bot_channel_skips_an_illegal_mode_and_defaults_to_the_next_legal_one() {
    let (state, source, p1, p2) = setup_cankerbloom(1, false, true);
    let procession = find_object(&state, "Anointed Procession");

    let actions = StubProvider.legal_actions(&state, p1);
    let action = activate_action_for(&actions, source);

    let plan = plan_targets(&state, p1, &action);
    let TargetPlan::Announce(targets) = &plan else {
        panic!("expected a real announcement for mode 1, got {plan:?}");
    };
    assert_eq!(targets, &vec![Target::Object(procession)]);

    let params = ActionParams {
        targets: plan.announced(),
        ..ActionParams::default()
    };
    let command = action_to_command_with_params(&state, p1, &action, &params)
        .expect("building the Command must succeed");
    let Command::ActivateAbility { modes_chosen, .. } = &command else {
        panic!("expected an ActivateAbility command, got {command:?}");
    };
    assert_eq!(
        modes_chosen,
        &vec![1],
        "CR 700.2a: mode 0 has no legal target (no artifact on this board) and \
         cannot be chosen; mode 1 must be the default instead"
    );

    let (state, _) = process_command(state, command)
        .expect("the engine must accept the fallback-mode announcement");
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        !is_on_battlefield(&state, procession),
        "mode 1's DestroyPermanent effect must actually have resolved"
    );
    assert!(events.iter().any(
        |e| matches!(e, GameEvent::PermanentDestroyed { object_id, .. } if *object_id == procession)
    ));
}

/// c3 -- CR 700.2a, the SUPPRESSION half, on a SYNTHETIC ability (zero corpus members
/// have two modes that are BOTH always illegal -- all three real modal activated
/// abilities have at least one target-free or always-satisfiable mode, so this shape
/// needs a constructed fixture to exercise at all): both modes require a creature
/// with `min_power: Some(999)`, and the only creature on the board is a 2/2. NO mode
/// is legal, so (SR-38) the activation must not be offered at all -- and, on the
/// SAME fixture, an explicit attempt naming the only creature that exists is refused
/// by the engine as an illegal target, proving the suppression is not merely
/// cosmetic (the offer and the engine agree that nothing legal exists here).
#[test]
fn c3_bot_channel_never_offers_a_modal_activation_with_no_legal_mode() {
    let p1 = p(1);
    let p2 = p(2);

    let impossible_target = vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
        min_power: Some(999),
        ..Default::default()
    })];
    let gadget_ability = ActivatedAbility {
        cost: ActivationCost::default(),
        description: "Choose one -- do nothing to a huge creature".to_string(),
        effect: Some(Effect::Nothing),
        modes: Some(ModeSelection {
            min_modes: 1,
            max_modes: 1,
            allow_duplicate_modes: false,
            mode_costs: None,
            modes: vec![Effect::Nothing, Effect::Nothing],
            mode_targets: Some(vec![impossible_target.clone(), impossible_target]),
        }),
        ..Default::default()
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::artifact(p1, "Impossible Gadget")
                .in_zone(ZoneId::Battlefield)
                .with_activated_ability(gadget_ability),
        )
        .object(ObjectSpec::creature(p2, "Weak Bear", 2, 2))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state builds");
    state.turn_mut().priority_holder = Some(p1);

    let gadget = find_object(&state, "Impossible Gadget");
    let weak_bear = find_object(&state, "Weak Bear");
    let idx = modal_ability_index(&state, gadget);

    let actions = StubProvider.legal_actions(&state, p1);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, LegalAction::ActivateAbility { source, .. } if *source == gadget)),
        "CR 700.2a (SR-38): with no legal mode this activation must not be offered at \
         all -- got it in the offer list: {actions:?}"
    );

    // The engine independently agrees: naming the ONLY creature that exists for
    // mode 0 is refused as an illegal target (it fails `min_power: 999`).
    let err = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: gadget,
            ability_index: idx,
            targets: vec![Target::Object(weak_bear)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![0],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect_err("a 2-power creature cannot satisfy a min_power: 999 filter");
    let msg = format!("{err:?}").to_lowercase();
    assert!(
        msg.contains("target") || msg.contains("invalid"),
        "expected a target-legality refusal naming the illegal candidate, got {msg}"
    );
}
