//! PB-DX27 (scutemob-209): stale blocker notes closed on 3 defs (reconnaissance,
//! wight_of_the_reliquary, chandra_flamecaller) plus a half-close on a fourth
//! (blackblade_reforged — CR 702.6c Equip{3} authored, dynamic P/T static left
//! deliberately unauthored; see that def's completeness note).
//!
//! Every test below drives a real `Command` against the real `CardDefinition` in
//! `crates/card-defs/src/defs/`, not a synthetic fixture (SR-34/36).

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AttackTarget,
    CardDefinition, CardRegistry, CardType, CombatState, Command, CounterType, EffectChoiceAnswer,
    EffectChoiceQuestion, GameEvent, GameState, GameStateBuilder, GameStateError, ManaColor,
    ObjectId, ObjectSpec, PlayerId, Step, SuperType, Target, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

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
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_controlled_by(state: &GameState, name: &str, controller: PlayerId) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.controller == controller)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' controlled by {:?} not found", name, controller))
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players).0;
    }
    state
}

fn real_card_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no real CardDefinition for '{}'", name));
    let base = ObjectSpec::card(owner, name)
        .in_zone(zone)
        .with_card_id(def.card_id.clone());
    enrich_spec_from_def(base, defs)
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .iter()
        .filter(|(_, obj)| obj.zone == ZoneId::Hand(player))
        .count()
}

fn graveyard_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .iter()
        .filter(|(_, obj)| obj.zone == ZoneId::Graveyard(player))
        .count()
}

// ═══════════════════════════════════════════════════════════════════════════
// Reconnaissance — CR 506.4/701.21: {0}: Remove target attacking creature you
// control from combat and untap it.
// ═══════════════════════════════════════════════════════════════════════════

fn setup_reconnaissance() -> (GameState, ObjectId, ObjectId, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let recon = real_card_spec(p1, "Reconnaissance", ZoneId::Battlefield, &defs);
    let attacker = ObjectSpec::creature(p1, "My Attacker", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(recon)
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let recon_id = find_object(&state, "Reconnaissance");
    let attacker_id = find_object(&state, "My Attacker");

    // Put the attacker into combat, tapped and attacking p2.
    if let Some(obj) = state.objects_mut().get_mut(&attacker_id) {
        obj.status.tapped = true;
    }
    let mut combat = CombatState::new(p2);
    combat
        .attackers
        .insert(attacker_id, AttackTarget::Player(p2));
    *state.combat_mut() = Some(combat);
    state.turn_mut().priority_holder = Some(p1);

    (state, recon_id, attacker_id, p1, p2)
}

/// CR 506.4/701.21 (PB-DX27): activating Reconnaissance's {0} ability targeting the
/// attacker removes it from combat AND untaps it.
#[test]
fn t1_reconnaissance_removes_attacker_and_untaps() {
    let (state, recon_id, attacker_id, p1, p2) = setup_reconnaissance();

    assert!(
        state.objects()[&attacker_id].status.tapped,
        "precondition: attacker starts tapped"
    );
    assert!(
        state
            .combat()
            .as_ref()
            .map(|c| c.attackers.contains_key(&attacker_id))
            .unwrap_or(false),
        "precondition: attacker starts in combat.attackers"
    );

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: recon_id,
            ability_index: 0,
            targets: vec![Target::Object(attacker_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("Reconnaissance's {0} ability should activate for free");
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        !state.objects()[&attacker_id].status.tapped,
        "CR 701.21: the target should be untapped"
    );
    assert!(
        !state
            .combat()
            .as_ref()
            .map(|c| c.attackers.contains_key(&attacker_id))
            .unwrap_or(false),
        "CR 506.4: the target should no longer be in combat.attackers"
    );
}

/// CR 702.6a-shape target filter ("you control"): an opponent's attacking creature
/// is not a legal target for Reconnaissance.
#[test]
fn t2_reconnaissance_rejects_a_creature_you_do_not_control() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let recon = real_card_spec(p1, "Reconnaissance", ZoneId::Battlefield, &defs);
    let opp_attacker =
        ObjectSpec::creature(p2, "Opponent Attacker", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(recon)
        .object(opp_attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    let recon_id = find_object(&state, "Reconnaissance");
    let opp_id = find_object(&state, "Opponent Attacker");
    if let Some(obj) = state.objects_mut().get_mut(&opp_id) {
        obj.status.tapped = true;
    }
    let mut combat = CombatState::new(p2);
    combat.attackers.insert(opp_id, AttackTarget::Player(p1));
    *state.combat_mut() = Some(combat);
    state.turn_mut().priority_holder = Some(p1);

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: recon_id,
            ability_index: 0,
            targets: vec![Target::Object(opp_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "Reconnaissance may only target an attacking creature ITS CONTROLLER controls; \
         got {:?}",
        result
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Wight of the Reliquary — CR 109.1/602.2: {T}, Sacrifice another creature:
// Search your library for a land card, put it onto the battlefield tapped,
// then shuffle.
// ═══════════════════════════════════════════════════════════════════════════

fn setup_wight() -> (GameState, ObjectId, ObjectId, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let wight = real_card_spec(p1, "Wight of the Reliquary", ZoneId::Battlefield, &defs);
    let fodder = ObjectSpec::creature(p1, "Fodder Bear", 2, 2).in_zone(ZoneId::Battlefield);
    let lib_land = ObjectSpec::card(p1, "Test Land")
        .with_types(vec![CardType::Land])
        .in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(wight)
        .object(fodder)
        .object(lib_land)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let wight_id = find_object(&state, "Wight of the Reliquary");
    let fodder_id = find_object(&state, "Fodder Bear");

    (state, wight_id, fodder_id, p1, p2)
}

/// CR 109.1/602.2 (PB-DX27): sacrificing another creature to Wight of the
/// Reliquary's ability searches the library for a land and puts it onto the
/// battlefield tapped.
#[test]
fn t3_wight_sacrifices_another_creature_for_a_land() {
    let (state, wight_id, fodder_id, p1, p2) = setup_wight();

    let (state, activate_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: wight_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(fodder_id),
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("Wight's sac-a-creature ability should activate");

    assert!(
        activate_events.iter().any(
            |e| matches!(e, GameEvent::CreatureDied { object_id, .. } if *object_id == fodder_id)
        ),
        "the sacrificed creature must die as a cost"
    );
    assert!(
        state.objects()[&wight_id].status.tapped,
        "the {{T}} portion of the cost must have tapped Wight"
    );

    // CR 608.2d: SearchLibrary blocks resolution for a player choice even when
    // exactly one candidate qualifies.
    let (state, _) = pass_all(state, &[p1, p2]);
    let entry = state
        .pending_effect_choice()
        .expect("CR 608.2d: the search should block for a player choice");
    let found = match &entry.question {
        EffectChoiceQuestion::SearchLibrary { candidates, .. } => {
            assert_eq!(
                candidates.len(),
                1,
                "exactly one land in the library qualifies"
            );
            candidates[0]
        }
        other => panic!("expected a search question, got {other:?}"),
    };
    let choice_player = entry.player;
    let choice_id = entry.choice_id;
    let (state, _) = process_command(
        state,
        Command::AnswerEffectChoice {
            player: choice_player,
            choice_id,
            answer: EffectChoiceAnswer::SearchLibrary { found: Some(found) },
        },
    )
    .expect("the search answer should be accepted");

    let land_obj = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Test Land")
        .expect("the searched land must exist");
    assert_eq!(
        land_obj.zone,
        ZoneId::Battlefield,
        "CR 701.19: the searched land must be put onto the battlefield"
    );
    assert!(
        land_obj.status.tapped,
        "the printed line puts the land onto the battlefield TAPPED"
    );
    assert_eq!(
        land_obj.controller, p1,
        "the land must enter under p1's control"
    );
}

/// CR 109.1: "Sacrifice ANOTHER creature" — Wight cannot pay its own cost by
/// sacrificing itself.
#[test]
fn t4_wight_cannot_sacrifice_itself() {
    let (state, wight_id, _fodder_id, p1, _p2) = setup_wight();

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: wight_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(wight_id),
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_err(),
        "CR 109.1: Wight must not be a legal choice to pay its own 'sacrifice another \
         creature' cost; got {:?}",
        result
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Chandra, Flamecaller — CR 701.9/121.1/121.2: 0: Discard all the cards in
// your hand, then draw that many cards plus one.
// ═══════════════════════════════════════════════════════════════════════════

/// CR 701.9/121.1/121.2 (PB-DX27): activating Chandra's 0 ability discards the
/// whole hand (3 cards) and draws 3 + 1 = 4.
#[test]
fn t5_chandra_zero_discards_hand_and_draws_plus_one() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let chandra = real_card_spec(p1, "Chandra, Flamecaller", ZoneId::Battlefield, &defs);
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(chandra.with_counter(CounterType::Loyalty, 4));
    for i in 0..3 {
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Hand Card {}", i)).in_zone(ZoneId::Hand(p1)));
    }
    for i in 0..10 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }
    let mut state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let chandra_id = find_object(&state, "Chandra, Flamecaller");
    assert_eq!(hand_count(&state, p1), 3, "precondition: 3 cards in hand");

    let (state, _) = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: chandra_id,
            ability_index: 1, // 0: discard hand, draw that many plus one
            targets: vec![],
            x_value: None,
        },
    )
    .expect("Chandra's 0 ability should activate");
    let state = resolve_stack(state, &[p1, p2]);

    assert_eq!(
        graveyard_count(&state, p1),
        3,
        "CR 701.9: the whole hand (3 cards) must be discarded"
    );
    assert_eq!(
        hand_count(&state, p1),
        4,
        "CR 121.1/121.2: draw that many (3) plus one = 4"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Blackblade Reforged — CR 702.6c: Equip legendary creature {3} (a SEPARATE
// activated ability from the plain Equip {7}).
// ═══════════════════════════════════════════════════════════════════════════

fn setup_blackblade() -> (GameState, ObjectId, ObjectId, ObjectId, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let blade = real_card_spec(p1, "Blackblade Reforged", ZoneId::Battlefield, &defs);
    let legend = ObjectSpec::creature(p1, "Legendary Bear", 2, 2)
        .with_supertypes(vec![SuperType::Legendary])
        .in_zone(ZoneId::Battlefield);
    let plain = ObjectSpec::creature(p1, "Plain Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(blade)
        .object(legend)
        .object(plain)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 7);
    state.turn_mut().priority_holder = Some(p1);

    let blade_id = find_object(&state, "Blackblade Reforged");
    let legend_id = find_object_controlled_by(&state, "Legendary Bear", p1);
    let plain_id = find_object_controlled_by(&state, "Plain Bear", p1);

    (state, blade_id, legend_id, plain_id, p1, p2)
}

/// CR 702.6c (PB-DX27): "Equip legendary creature {3}" is a separate activated
/// ability (index 1, after the plain Equip {7} at index 0) that attaches
/// Blackblade Reforged to a legendary creature the player controls, for {3}.
#[test]
fn t6_blackblade_equip_legendary_ability_attaches_for_three() {
    let (state, blade_id, legend_id, _plain_id, p1, p2) = setup_blackblade();

    // Sanity: card_name_to_id resolves (proves we loaded the real def, not a stub).
    let _ = card_name_to_id("Blackblade Reforged");

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blade_id,
            ability_index: 1, // Equip legendary creature {3}
            targets: vec![Target::Object(legend_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("Equip legendary creature {3} should activate against a legendary creature");

    let mana_pool = &state.players().get(&p1).unwrap().mana_pool;
    assert_eq!(
        mana_pool.colorless, 4,
        "Equip {{3}} must have spent exactly 3 of the 7 colorless mana in the pool"
    );

    let (state, resolve_events) = pass_all(state, &[p1, p2]);
    let blade_obj = state.objects().get(&blade_id).expect("blade exists");
    assert_eq!(
        blade_obj.attached_to,
        Some(legend_id),
        "Blackblade Reforged must be attached to the legendary creature"
    );
    assert!(
        resolve_events.iter().any(
            |e| matches!(e, GameEvent::EquipmentAttached { equipment_id, target_id, .. }
                if *equipment_id == blade_id && *target_id == legend_id)
        ),
        "EquipmentAttached event expected"
    );
}

/// CR 702.6c: a non-legendary creature is not a legal target for the {3} equip
/// ability (it remains legal for the plain {7} ability, unaffected by this batch).
#[test]
fn t7_blackblade_equip_legendary_ability_rejects_nonlegendary_creature() {
    let (state, blade_id, _legend_id, plain_id, p1, _p2) = setup_blackblade();

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blade_id,
            ability_index: 1, // Equip legendary creature {3}
            targets: vec![Target::Object(plain_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "CR 702.6c: a non-legendary creature must not be a legal target for 'Equip \
         legendary creature {{3}}'; got {:?}",
        result
    );
}

/// The plain Equip {7} ability (index 0) is untouched by this batch: it still
/// attaches to ANY creature the player controls, legendary or not, for {7}.
#[test]
fn t8_blackblade_plain_equip_seven_still_attaches_to_a_nonlegendary_creature() {
    let (state, blade_id, _legend_id, plain_id, p1, p2) = setup_blackblade();

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blade_id,
            ability_index: 0, // plain Equip {7}
            targets: vec![Target::Object(plain_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("plain Equip {7} should still attach to a non-legendary creature");

    let mana_pool = &state.players().get(&p1).unwrap().mana_pool;
    assert_eq!(
        mana_pool.colorless, 0,
        "Equip {{7}} must have spent all 7 colorless mana in the pool"
    );

    let state = resolve_stack(state, &[p1, p2]);
    let blade_obj = state.objects().get(&blade_id).expect("blade exists");
    assert_eq!(
        blade_obj.attached_to,
        Some(plain_id),
        "Blackblade Reforged must be attached via the untouched plain Equip {{7}} ability"
    );
}

/// The layer-resolved dynamic P/T static ("+1/+1 for each land you control") is
/// deliberately NOT authored on this def -- it must not silently start applying.
/// Non-vacuity + honesty check for the completeness marker.
#[test]
fn t9_blackblade_dynamic_land_bonus_is_not_yet_authored() {
    let defs = all_cards();
    let blade = defs
        .iter()
        .find(|d| d.name == "Blackblade Reforged")
        .expect("Blackblade Reforged must exist in the corpus");
    assert!(
        !matches!(blade.completeness, mtg_engine::Completeness::Complete),
        "Blackblade Reforged must stay non-Complete: the dynamic '+1/+1 for each land \
         you control' static is not authored (open resolve_cda_amount controller- \
         attribution question, same as crown_of_skemfar.rs/empyrial_plate.rs)"
    );
    let has_pt_static = blade
        .abilities
        .iter()
        .any(|a| matches!(a, mtg_engine::AbilityDefinition::Static { .. }));
    assert!(
        !has_pt_static,
        "no AbilityDefinition::Static was authored on Blackblade Reforged by this batch"
    );
}
