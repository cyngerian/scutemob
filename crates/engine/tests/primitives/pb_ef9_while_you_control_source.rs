//! Tests for PB-EF9: `EffectDuration::WhileYouControlSource` (EF-W-PB2-5).
//!
//! CR 611.2b/c: "for as long as you control [source]" is a continuous-effect duration
//! that differs from `WhileSourceOnBattlefield` ONLY when control of the source changes
//! away from the effect's creator. The borrowed permanent returns to its owner the
//! moment the creator stops controlling the source, and the effect never resumes even
//! if the creator later regains control (CR 611.2c: the set of affected objects is
//! fixed when the effect begins).
//!
//! Correctness is split across three pieces, each with its own test:
//! - `is_effect_active` always reports `true` for this duration (never a live control
//!   check) -- termination is owned solely by `expire_while_you_control_source_effects`.
//! - `expire_while_you_control_source_effects` (layers.rs) computes "ended" from the
//!   source's current controller/zone and PERMANENTLY removes the effect -- the "never
//!   resumes" guarantee.
//! - `recompute_object_controller` (layers.rs) reverts the borrowed object's controller,
//!   respecting any other still-active `SetController` effect (stacked control).
//!
//! Decoy discipline (SR-36 sense): the DECOY test uses a Layer-7c P/T modifier sourced
//! by the SAME object, under `WhileSourceOnBattlefield`, to prove the two durations
//! differ *only* on control change (not "we removed all continuous effects on SBA").
//! A `SetController` decoy was deliberately NOT used here -- `WhileSourceOnBattlefield`
//! gain-control has its own pre-existing never-reverts gap (OOS-EF9-1), and reusing it
//! as a decoy would conflate that gap with this test.

use std::collections::HashMap;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::rules::layers::{calculate_characteristics, is_effect_active};
use mtg_engine::state::continuous_effect::{
    ContinuousEffect as CE, EffectDuration, EffectFilter, EffectId, EffectLayer, LayerModification,
};
use mtg_engine::state::player::ManaPool;
use mtg_engine::state::test_util;
use mtg_engine::{
    all_cards, card_name_to_id, check_and_apply_sbas, enrich_spec_from_def, process_command,
    CardDefinition, CardRegistry, Command, GameState, GameStateBuilder, ObjectId, ObjectSpec,
    PlayerId, Step, Target, ZoneId,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn defs_map() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn real_card_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

/// Pass priority once for every player in the list (resolves the top stack item or
/// advances the turn).
fn pass_all(state: GameState, players: &[PlayerId]) -> GameState {
    let mut current = state;
    for &pl in players {
        let (s, _) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
    }
    current
}

/// Register a `WhileYouControlSource(borrower)` Layer-2 SetController effect sourced
/// by `source_id`, applying to `target_id`, and imperatively set the target's
/// controller (mirrors what `Effect::GainControl` does at resolution).
fn borrow_via_source(
    state: &mut GameState,
    effect_id: u64,
    source_id: ObjectId,
    target_id: ObjectId,
    borrower: PlayerId,
    timestamp: u64,
) {
    let eff = CE {
        id: EffectId(effect_id),
        source: Some(source_id),
        layer: EffectLayer::Control,
        modification: LayerModification::SetController(borrower),
        filter: EffectFilter::SingleObject(target_id),
        duration: EffectDuration::WhileYouControlSource(borrower),
        is_cda: false,
        timestamp,
        condition: None,
    };
    state.continuous_effects_mut().push_back(eff);
    state.objects_mut().get_mut(&target_id).unwrap().controller = borrower;
}

/// Simulate an opponent stealing `object_id` (an unrelated `Indefinite` SetController
/// effect, as a generic "some other spell/ability gained control of this" steal).
fn steal_indefinitely(
    state: &mut GameState,
    effect_id: u64,
    object_id: ObjectId,
    thief: PlayerId,
    timestamp: u64,
) {
    let eff = CE {
        id: EffectId(effect_id),
        source: None,
        layer: EffectLayer::Control,
        modification: LayerModification::SetController(thief),
        filter: EffectFilter::SingleObject(object_id),
        duration: EffectDuration::Indefinite,
        is_cda: false,
        timestamp,
        condition: None,
    };
    state.continuous_effects_mut().push_back(eff);
    state.objects_mut().get_mut(&object_id).unwrap().controller = thief;
}

fn has_control_effect(state: &GameState, effect_id: u64) -> bool {
    state
        .continuous_effects()
        .iter()
        .any(|e| e.id == EffectId(effect_id))
}

// ── Engine-mechanism tests ───────────────────────────────────────────────────

/// CR 611.2c -- when the effect's creator no longer controls the source (here:
/// because an opponent's separate steal effect took the source), the borrowed
/// permanent reverts to its owner and the WhileYouControlSource effect is
/// permanently removed.
#[test]
fn test_while_you_control_source_ends_when_opponent_gains_source() {
    let p1 = p(1);
    let p2 = p(2);
    let source = ObjectSpec::creature(p1, "Source", 2, 2).in_zone(ZoneId::Battlefield);
    let borrowed = ObjectSpec::creature(p2, "Borrowed", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .object(borrowed)
        .build()
        .unwrap();

    let source_id = find_obj(&state, "Source");
    let borrowed_id = find_obj(&state, "Borrowed");

    borrow_via_source(&mut state, 900, source_id, borrowed_id, p1, 900);
    assert_eq!(
        state.objects().get(&borrowed_id).unwrap().controller,
        p1,
        "p1 should control the borrowed creature"
    );

    // Opponent gains control of the SOURCE (not the borrowed creature).
    steal_indefinitely(&mut state, 901, source_id, p2, 901);
    assert_eq!(
        state.objects().get(&source_id).unwrap().controller,
        p2,
        "test invariant: p2 must now control the source"
    );

    check_and_apply_sbas(&mut state);

    assert_eq!(
        state.objects().get(&borrowed_id).unwrap().controller,
        p2,
        "CR 611.2c: the borrowed creature must revert to its owner (p2) once p1 no \
         longer controls the source"
    );
    assert!(
        !has_control_effect(&state, 900),
        "CR 611.2c: the WhileYouControlSource effect must be permanently removed"
    );
}

/// Sibling of the test above: a `WhileSourceOnBattlefield` Layer-7c effect sourced by
/// the SAME object, applying to a DIFFERENT permanent, must NOT end when the source's
/// controller changes -- only its own duration (source leaves the battlefield) governs
/// it. This proves the two durations diverge only on control change, not on any
/// broader "control changed, wipe continuous effects" behavior. A P/T decoy is used
/// (not a SetController decoy) to avoid entangling with the pre-existing
/// WhileSourceOnBattlefield gain-control never-reverts gap (OOS-EF9-1).
#[test]
fn test_while_you_control_source_decoy_whilesourceonbattlefield_does_not_end() {
    let p1 = p(1);
    let p2 = p(2);
    let source = ObjectSpec::creature(p1, "Source", 2, 2).in_zone(ZoneId::Battlefield);
    let borrowed = ObjectSpec::creature(p2, "Borrowed", 3, 3).in_zone(ZoneId::Battlefield);
    let decoy_target = ObjectSpec::creature(p1, "DecoyTarget", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .object(borrowed)
        .object(decoy_target)
        .build()
        .unwrap();

    let source_id = find_obj(&state, "Source");
    let borrowed_id = find_obj(&state, "Borrowed");
    let decoy_id = find_obj(&state, "DecoyTarget");

    borrow_via_source(&mut state, 910, source_id, borrowed_id, p1, 910);

    // DECOY: a WhileSourceOnBattlefield +1/+1 static effect sourced by the same object,
    // applying to a third, unrelated creature.
    let decoy_eff = CE {
        id: EffectId(911),
        source: Some(source_id),
        layer: EffectLayer::PtModify,
        modification: LayerModification::ModifyBoth(1),
        filter: EffectFilter::SingleObject(decoy_id),
        duration: EffectDuration::WhileSourceOnBattlefield,
        is_cda: false,
        timestamp: 911,
        condition: None,
    };
    state.continuous_effects_mut().push_back(decoy_eff);

    let pre_chars = calculate_characteristics(&state, decoy_id).unwrap();
    assert_eq!(
        pre_chars.power,
        Some(3),
        "test invariant: the decoy +1/+1 must be observably applied before the steal"
    );

    // Opponent gains control of the SOURCE. Source stays on the battlefield.
    steal_indefinitely(&mut state, 912, source_id, p2, 912);

    check_and_apply_sbas(&mut state);

    assert_eq!(
        state.objects().get(&borrowed_id).unwrap().controller,
        p2,
        "sanity: the WhileYouControlSource borrow DID end in this same scenario"
    );
    assert!(
        has_control_effect(&state, 911),
        "the WhileSourceOnBattlefield decoy effect must still be present -- it is \
         governed by source presence on the battlefield, not source control"
    );
    assert!(
        is_effect_active(
            &state,
            state
                .continuous_effects()
                .iter()
                .find(|e| e.id == EffectId(911))
                .unwrap()
        ),
        "the WhileSourceOnBattlefield decoy effect must still be active"
    );
    let post_chars = calculate_characteristics(&state, decoy_id).unwrap();
    assert_eq!(
        post_chars.power,
        Some(3),
        "the decoy's +1/+1 modification must still apply after the source's control changed"
    );
}

/// CR 611.2c -- "never resumes": once the effect has ended (source no longer
/// controlled by the borrower), regaining control of the source must NOT revive it.
#[test]
fn test_while_you_control_source_does_not_resume() {
    let p1 = p(1);
    let p2 = p(2);
    let source = ObjectSpec::creature(p1, "Source", 2, 2).in_zone(ZoneId::Battlefield);
    let borrowed = ObjectSpec::creature(p2, "Borrowed", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .object(borrowed)
        .build()
        .unwrap();

    let source_id = find_obj(&state, "Source");
    let borrowed_id = find_obj(&state, "Borrowed");

    borrow_via_source(&mut state, 920, source_id, borrowed_id, p1, 920);
    steal_indefinitely(&mut state, 921, source_id, p2, 921);
    check_and_apply_sbas(&mut state);
    assert_eq!(
        state.objects().get(&borrowed_id).unwrap().controller,
        p2,
        "setup: the borrow must have ended before testing non-resumption"
    );
    assert!(!has_control_effect(&state, 920));

    // p1 regains control of the source (remove the thief's steal, hand it back).
    state
        .continuous_effects_mut()
        .retain(|e| e.id != EffectId(921));
    state.objects_mut().get_mut(&source_id).unwrap().controller = p1;
    assert_eq!(
        state.objects().get(&source_id).unwrap().controller,
        p1,
        "test invariant: p1 must control the source again"
    );

    check_and_apply_sbas(&mut state);

    assert_eq!(
        state.objects().get(&borrowed_id).unwrap().controller,
        p2,
        "CR 611.2c: the borrowed creature must NOT resume under p1 even though p1 \
         regained control of the source"
    );
    assert!(
        !has_control_effect(&state, 920),
        "the ended WhileYouControlSource effect must not reappear"
    );
}

/// CR 611.2b -- the source leaving the battlefield (a new ObjectId per CR 400.7) also
/// ends the effect and reverts control.
#[test]
fn test_while_you_control_source_ends_when_source_leaves() {
    let p1 = p(1);
    let p2 = p(2);
    let source = ObjectSpec::creature(p1, "Source", 2, 2).in_zone(ZoneId::Battlefield);
    let borrowed = ObjectSpec::creature(p2, "Borrowed", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .object(borrowed)
        .build()
        .unwrap();

    let source_id = find_obj(&state, "Source");
    let borrowed_id = find_obj(&state, "Borrowed");

    borrow_via_source(&mut state, 930, source_id, borrowed_id, p1, 930);

    // The source leaves the battlefield -- CR 400.7: it becomes a new object.
    test_util::move_object_to_zone(&mut state, source_id, ZoneId::Graveyard(p1))
        .expect("source should move to the graveyard");

    check_and_apply_sbas(&mut state);

    assert_eq!(
        state.objects().get(&borrowed_id).unwrap().controller,
        p2,
        "CR 611.2b: the borrowed creature must revert to its owner when the source \
         leaves the battlefield"
    );
    assert!(
        !has_control_effect(&state, 930),
        "the WhileYouControlSource effect must be removed once the source is gone"
    );
}

/// CR 702.26e -- a phased-out source is STILL controlled by its controller. The
/// borrowed permanent must NOT revert while the source is merely phased out.
#[test]
fn test_while_you_control_source_survives_source_phase_out() {
    let p1 = p(1);
    let p2 = p(2);
    let source = ObjectSpec::creature(p1, "Source", 2, 2).in_zone(ZoneId::Battlefield);
    let borrowed = ObjectSpec::creature(p2, "Borrowed", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .object(borrowed)
        .build()
        .unwrap();

    let source_id = find_obj(&state, "Source");
    let borrowed_id = find_obj(&state, "Borrowed");

    borrow_via_source(&mut state, 940, source_id, borrowed_id, p1, 940);

    // Phase the source out. Controller is unchanged; zone stays Battlefield
    // (CR 702.26e -- phased-out permanents are still on the battlefield for control
    // purposes, they are simply treated as though they don't exist for most rules).
    state
        .objects_mut()
        .get_mut(&source_id)
        .unwrap()
        .status
        .phased_out = true;
    assert_eq!(
        state.objects().get(&source_id).unwrap().controller,
        p1,
        "test invariant: phasing out does not change controller"
    );

    check_and_apply_sbas(&mut state);

    assert_eq!(
        state.objects().get(&borrowed_id).unwrap().controller,
        p1,
        "CR 702.26e: a phased-out source is still controlled -- the borrow must survive"
    );
    assert!(
        has_control_effect(&state, 940),
        "the WhileYouControlSource effect must still be present while the source is \
         merely phased out"
    );
}

/// CR 611.2c, 4-player -- owner p3, borrower p1, thief p2. When p2 steals the SOURCE
/// (which p1 controls), the borrowed creature must return to its OWNER (p3), not to
/// the thief (p2) and not stay with the borrower (p1).
#[test]
fn test_while_you_control_source_multiplayer() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let source = ObjectSpec::creature(p1, "Source", 2, 2).in_zone(ZoneId::Battlefield);
    let borrowed = ObjectSpec::creature(p3, "Borrowed", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .object(borrowed)
        .build()
        .unwrap();

    let source_id = find_obj(&state, "Source");
    let borrowed_id = find_obj(&state, "Borrowed");

    borrow_via_source(&mut state, 950, source_id, borrowed_id, p1, 950);
    assert_eq!(
        state.objects().get(&borrowed_id).unwrap().controller,
        p1,
        "p1 (borrower) should control the borrowed creature"
    );

    // p2 (thief) steals the SOURCE from p1.
    steal_indefinitely(&mut state, 951, source_id, p2, 951);

    check_and_apply_sbas(&mut state);

    assert_eq!(
        state.objects().get(&borrowed_id).unwrap().controller,
        p3,
        "CR 611.2c: the borrowed creature must return to its OWNER (p3), not the \
         thief (p2) and not stay with the original borrower (p1)"
    );
    assert!(!has_control_effect(&state, 950));
}

// ── Card integration tests ───────────────────────────────────────────────────

/// Olivia Voldaren's `{3}{B}{B}: Gain control of target Vampire for as long as you
/// control Olivia Voldaren.` -- the Vampire returns to its owner when Olivia is
/// stolen (this is the case that was wrong under the old
/// `WhileSourceOnBattlefield` approximation: Olivia stays on the battlefield, just
/// under a new controller).
#[test]
fn test_olivia_voldaren_steal_returns_when_olivia_stolen() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let olivia = real_card_spec(p1, "Olivia Voldaren", ZoneId::Battlefield, &defs);
    let vampire = ObjectSpec::creature(p2, "Test Vampire", 2, 2)
        .with_types(vec![mtg_engine::CardType::Creature])
        .with_subtypes(vec![mtg_engine::SubType("Vampire".to_string())])
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(olivia)
        .object(vampire)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    state.players_mut().get_mut(&p1).unwrap().mana_pool = ManaPool {
        colorless: 3,
        black: 2,
        ..Default::default()
    };

    let olivia_id = find_obj(&state, "Olivia Voldaren");
    let vampire_id = find_obj(&state, "Test Vampire");

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: olivia_id,
            ability_index: 1,
            targets: vec![Target::Object(vampire_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
        },
    )
    .expect("activate Olivia's {3}{B}{B} ability");
    let state = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.objects().get(&vampire_id).unwrap().controller,
        p1,
        "p1 should now control the Vampire"
    );

    // Steal Olivia (she stays on the battlefield, just under p2's control).
    let mut state = state;
    steal_indefinitely(&mut state, 990, olivia_id, p2, 990);

    check_and_apply_sbas(&mut state);

    assert_eq!(
        state.objects().get(&vampire_id).unwrap().controller,
        p2,
        "the Vampire must return to its owner (p2) once Olivia is stolen -- this is \
         the case WhileSourceOnBattlefield got wrong (Olivia never left the \
         battlefield, she just changed controller)"
    );
}

/// Companion to the test above: Olivia dying (leaving the battlefield entirely) also
/// returns the Vampire to its owner.
#[test]
fn test_olivia_voldaren_steal_returns_when_olivia_dies() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let olivia = real_card_spec(p1, "Olivia Voldaren", ZoneId::Battlefield, &defs);
    let vampire = ObjectSpec::creature(p2, "Test Vampire", 2, 2)
        .with_types(vec![mtg_engine::CardType::Creature])
        .with_subtypes(vec![mtg_engine::SubType("Vampire".to_string())])
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(olivia)
        .object(vampire)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    state.players_mut().get_mut(&p1).unwrap().mana_pool = ManaPool {
        colorless: 3,
        black: 2,
        ..Default::default()
    };

    let olivia_id = find_obj(&state, "Olivia Voldaren");
    let vampire_id = find_obj(&state, "Test Vampire");

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: olivia_id,
            ability_index: 1,
            targets: vec![Target::Object(vampire_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
        },
    )
    .expect("activate Olivia's {3}{B}{B} ability");
    let mut state = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.objects().get(&vampire_id).unwrap().controller,
        p1,
        "p1 should now control the Vampire"
    );

    // Olivia leaves the battlefield entirely.
    test_util::move_object_to_zone(&mut state, olivia_id, ZoneId::Graveyard(p1))
        .expect("Olivia should move to the graveyard");

    check_and_apply_sbas(&mut state);

    assert_eq!(
        state.objects().get(&vampire_id).unwrap().controller,
        p2,
        "the Vampire must return to its owner (p2) once Olivia leaves the battlefield"
    );
}

/// Dragonlord Silumgar's ETB `gain control of target creature or planeswalker for as
/// long as you control Dragonlord Silumgar.` -- driven end-to-end via `Command::CastSpell`
/// so the real `WhenEntersBattlefield` CardDef trigger path fires. Reversion is tested
/// on Silumgar leaving the battlefield.
#[test]
fn test_dragonlord_silumgar_etb_steal_reverts_on_leave() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let silumgar_in_hand = real_card_spec(p1, "Dragonlord Silumgar", ZoneId::Hand(p1), &defs);
    // The only eligible creature/planeswalker on the battlefield -- makes the ETB
    // trigger's auto-target-picker deterministic.
    let target_creature =
        ObjectSpec::creature(p2, "Steal Target", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(silumgar_in_hand)
        .object(target_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    state.players_mut().get_mut(&p1).unwrap().mana_pool = ManaPool {
        colorless: 4,
        blue: 1,
        black: 1,
        ..Default::default()
    };

    let card_id = find_obj(&state, "Dragonlord Silumgar");
    let target_id = find_obj(&state, "Steal Target");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: card_id,
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
    .expect("cast Dragonlord Silumgar");
    // Resolve the creature spell onto the battlefield (queues the real
    // WhenEntersBattlefield CardDef trigger).
    let state = pass_all(state, &[p1, p2]);
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Dragonlord Silumgar"
                && o.zone == ZoneId::Battlefield),
        "Dragonlord Silumgar should have resolved onto the battlefield"
    );
    // Resolve the ETB trigger (auto-picks the sole eligible target).
    let mut state = pass_all(state, &[p1, p2]);

    let silumgar_id = state
        .objects()
        .iter()
        .find(|(_, o)| {
            o.characteristics.name == "Dragonlord Silumgar" && o.zone == ZoneId::Battlefield
        })
        .map(|(id, _)| *id)
        .expect("Silumgar should still be on the battlefield");

    assert_eq!(
        state.objects().get(&target_id).unwrap().controller,
        p1,
        "CR 613.1b: p1 should now control the stolen creature"
    );

    // Silumgar leaves the battlefield.
    test_util::move_object_to_zone(&mut state, silumgar_id, ZoneId::Graveyard(p1))
        .expect("Silumgar should move to the graveyard");

    check_and_apply_sbas(&mut state);

    assert_eq!(
        state.objects().get(&target_id).unwrap().controller,
        p2,
        "CR 611.2b: the stolen creature must revert to its owner (p2) once Silumgar \
         leaves the battlefield"
    );
}
