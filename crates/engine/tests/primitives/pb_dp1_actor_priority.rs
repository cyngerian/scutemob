//! PB-DP1 probes: priority after casting a spell, activating an ability, or
//! taking a special action goes to the ACTOR, not the active player.
//!
//! CR 117.3c: "If a player has priority when they cast a spell, activate an
//! ability, or take a special action, that player receives priority afterward."
//! CR 116.3: "If a player takes a special action, that player receives
//! priority afterward."
//! CR 117.4: an action taken between passes restarts the pass-round
//! (`players_passed` must be reset).
//!
//! Written FIRST and run RED before any engine line moves (AC 5512). P1-P5 and
//! P8 must fail against the pre-fix engine; P6 and P7 are green-both-sides
//! control probes by design (see `memory/primitives/pb-plan-DP1.md` §3).

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    process_command, AbilityDefinition, ActivatedAbility, ActivationCost, CardDefinition, CardId,
    CardRegistry, CardType, Command, FaceDownKind, GameState, GameStateBuilder,
    KeywordAbility, ManaAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step,
    SubType, TurnFaceUpMethod, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Tap-only activated ability, instant speed (no sorcery-speed restriction).
fn tap_ability(description: &str) -> ActivatedAbility {
    ActivatedAbility {
        targets: vec![],
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: None,
            sacrifice_self: false,
            discard_card: false,
            discard_self: false,
            forage: false,
            sacrifice_filter: None,
            remove_counter_cost: None,
            exile_self: false,
            exert: false,
            life_cost: 0,
            sacrifice_exclude_self: false,
            exile_self_from_hand: false,
        },
        description: description.to_string(),
        effect: None,
        sorcery_speed: false,
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        modes: None,
    }
}

/// A minimal `CastSpellData` with every alt-cost/mode field at its default,
/// so each probe only needs to set `player` and `card`.
fn cast(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
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
    }))
}

/// Build a Vehicle ObjectSpec (mirrors `mechanics_a_d/crew.rs::vehicle_spec`).
fn vehicle_spec(owner: PlayerId, name: &str, power: i32, toughness: i32, crew_n: u32) -> ObjectSpec {
    let mut spec = ObjectSpec::artifact(owner, name)
        .with_subtypes(vec![SubType("Vehicle".to_string())])
        .with_keyword(KeywordAbility::Crew(crew_n));
    spec.power = Some(power);
    spec.toughness = Some(toughness);
    spec
}

// ── P1 — non-active player casting an instant retains priority ───────────────

#[test]
/// CR 117.3c, CR 601.2i — a non-active player who holds priority casts an
/// instant and retains priority afterward; it does not revert to the active
/// player.
fn test_dp1_non_active_player_casting_instant_retains_priority() {
    let p1 = p(1);
    let p2 = p(2);

    let instant = ObjectSpec::card(p2, "Probe Bolt")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(instant)
        .build()
        .unwrap();
    // Simulate p1 having passed priority to p2 (CR 117.3d).
    state.turn_mut().priority_holder = Some(p2);

    let card_id = find_object(&state, "Probe Bolt");
    let (new_state, _events) = process_command(state, cast(p2, card_id))
        .expect("CR 117.3c: p2 held priority and should be able to cast an instant");

    assert_eq!(
        new_state.turn().priority_holder,
        Some(p2),
        "CR 117.3c: the caster, not the active player, receives priority after casting"
    );
}

// ── P2 — actor can respond to their own spell with no intervening pass ───────

#[test]
/// CR 117.3c — after casting, the actor retains priority and may immediately
/// cast a second spell in response to their own, with no intervening
/// `PassPriority`.
fn test_dp1_actor_can_respond_to_own_spell() {
    let p1 = p(1);
    let p2 = p(2);

    let instant_a = ObjectSpec::card(p2, "Probe Bolt A")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));
    let instant_b = ObjectSpec::card(p2, "Probe Bolt B")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(instant_a)
        .object(instant_b)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p2);

    let card_a = find_object(&state, "Probe Bolt A");
    let (state, _) =
        process_command(state, cast(p2, card_a)).expect("first cast by p2 should succeed");

    let card_b = find_object(&state, "Probe Bolt B");
    // No intervening PassPriority: p2 responds to their own spell.
    let (state, _) = process_command(state, cast(p2, card_b)).expect(
        "CR 117.3c: p2 should retain priority and be able to cast a second spell without passing",
    );

    assert_eq!(
        state.stack_objects().len(),
        2,
        "both spells should be on the stack"
    );
    assert_eq!(state.turn().priority_holder, Some(p2));
}

// ── P3 — actor can respond to their own activated ability ────────────────────

#[test]
/// CR 117.3c, CR 602.2b — a non-active player who activates an instant-speed
/// activated ability retains priority afterward and can immediately activate a
/// second one, with no intervening pass.
fn test_dp1_actor_can_respond_to_own_activated_ability() {
    let p1 = p(1);
    let p2 = p(2);

    let creature_a = ObjectSpec::creature(p2, "Probe Creature A", 1, 1)
        .with_activated_ability(tap_ability("{T}: Probe effect A"))
        .in_zone(ZoneId::Battlefield);
    let creature_b = ObjectSpec::creature(p2, "Probe Creature B", 1, 1)
        .with_activated_ability(tap_ability("{T}: Probe effect B"))
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature_a)
        .object(creature_b)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p2);

    let source_a = find_object(&state, "Probe Creature A");
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: source_a,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("first activation by p2 should succeed");

    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 117.3c: p2 retains priority after activating their own ability"
    );

    let source_b = find_object(&state, "Probe Creature B");
    // No intervening PassPriority.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: source_b,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect(
        "CR 117.3c: p2 should retain priority and be able to activate a second ability without passing",
    );

    assert_eq!(state.stack_objects().len(), 2);
    assert_eq!(state.turn().priority_holder, Some(p2));
}

// ── P4 — cycling by a non-active player retains priority ─────────────────────

#[test]
/// CR 702.29a, CR 602.2b, CR 117.3c — cycling is instant speed with no
/// active-player gate; the cycling player retains priority afterward.
fn test_dp1_non_active_player_cycling_retains_priority() {
    let p1 = p(1);
    let p2 = p(2);

    let cycling_def = CardDefinition {
        card_id: CardId("dp1-cycling-card".to_string()),
        name: "DP1 Cycling Card".to_string(),
        mana_cost: None,
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Cycling {1}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![cycling_def]);

    let cycling_card = ObjectSpec::card(p2, "DP1 Cycling Card")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("dp1-cycling-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);
    let library_card = ObjectSpec::card(p2, "Library Filler").in_zone(ZoneId::Library(p2));

    let mut state = GameStateBuilder::four_player()
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(cycling_card)
        .object(library_card)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p2);

    let card_id = find_object(&state, "DP1 Cycling Card");
    let (state, _) = process_command(
        state,
        Command::CycleCard {
            player: p2,
            card: card_id,
        },
    )
    .expect("CR 702.29a: p2 should be able to cycle during p1's turn");

    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 117.3c: the cycling player retains priority"
    );
}

// ── P5 — crewing a vehicle by a non-active player retains priority ───────────

#[test]
/// CR 702.122a, CR 117.3c — crewing has no sorcery-speed gate; the crewing
/// player retains priority afterward.
fn test_dp1_non_active_player_crewing_retains_priority() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p2, "Probe Copter", 3, 3, 1);
    let crew_member = ObjectSpec::creature(p2, "Probe Pilot", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(crew_member)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p2);

    let vehicle_id = find_object(&state, "Probe Copter");
    let pilot_id = find_object(&state, "Probe Pilot");

    let (state, _) = process_command(
        state,
        Command::CrewVehicle {
            player: p2,
            vehicle: vehicle_id,
            crew_creatures: vec![pilot_id],
        },
    )
    .expect("CR 702.122a: p2 should be able to crew during p1's turn");

    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 117.3c: the crewing player retains priority"
    );
}

// ── P6 — control probe: active player casting still holds priority ──────────

#[test]
/// CR 117.3c — control probe: when the active player casts a spell they hold
/// priority, they still hold priority afterward. Guards against a mis-targeted
/// edit that writes some other player into `priority_holder`. Green both
/// before and after the fix, by design.
fn test_dp1_active_player_casting_still_holds_priority() {
    let p1 = p(1);

    let instant = ObjectSpec::card(p1, "Probe Bolt")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(instant)
        .build()
        .unwrap();
    // Default priority_holder from the builder is already Some(p1) (the active
    // player), matching "if a player has priority when they cast" (CR 117.3c).

    let card_id = find_object(&state, "Probe Bolt");
    let (state, _) =
        process_command(state, cast(p1, card_id)).expect("active player cast should succeed");

    assert_eq!(state.turn().priority_holder, Some(p1));
}

// ── P7 — PRESERVE: mana ability does not reset players_passed or disturb holder ──

#[test]
/// CR 605.3a/b, CR 117.3b parenthetical — a mana ability does not use the
/// stack, does not reset `players_passed`, and does not disturb the current
/// priority holder. This is the PRESERVE regression pin: it must stay green
/// both before and after the fix. Never weaken this test to make another
/// probe pass.
fn test_dp1_mana_ability_does_not_reset_players_passed() {
    let p1 = p(1);
    let p2 = p(2);

    let forest = ObjectSpec::land(p2, "Probe Forest").with_mana_ability(ManaAbility::tap_for(
        ManaColor::Green,
    ));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(forest)
        .build()
        .unwrap();
    // p1 has already passed; p2 currently holds priority.
    state.turn_mut().players_passed.insert(p1);
    state.turn_mut().priority_holder = Some(p2);

    let land_id = find_object(&state, "Probe Forest");
    let (state, _) = process_command(
        state,
        Command::TapForMana {
            player: p2,
            source: land_id,
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("tapping for mana should succeed");

    assert!(
        state.turn().players_passed.contains(&p1),
        "CR 117.3b parenthetical: a mana ability must not reset players_passed"
    );
    assert_eq!(
        state.turn().players_passed.len(),
        1,
        "players_passed must contain exactly the pre-existing entry, nothing added or removed"
    );
    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 605.3a/b: a mana ability does not touch the priority holder"
    );
}

// ── P8 — foretell resets players_passed (CR 117.4) ───────────────────────────

#[test]
/// CR 116.2h, CR 116.3, CR 117.4 — foretelling is a special action; the
/// acting player receives priority afterward (already true by construction,
/// since the priority guard requires the actor to already hold it) and the
/// pass-round restarts because an action was taken between passes.
fn test_dp1_foretell_resets_players_passed() {
    let p1 = p(1);

    let foretell_def = CardDefinition {
        card_id: CardId("dp1-foretell-card".to_string()),
        name: "DP1 Foretell Card".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Foretell),
            AbilityDefinition::Foretell {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![foretell_def]);

    let card = ObjectSpec::card(p1, "DP1 Foretell Card")
        .with_card_id(CardId("dp1-foretell-card".to_string()))
        .with_keyword(KeywordAbility::Foretell)
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::four_player()
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(card)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    // p1 holds priority; simulate p2 and p3 (or a stack object cycle) having
    // already passed earlier in this round.
    state.turn_mut().priority_holder = Some(p1);
    state.turn_mut().players_passed.insert(p(2));
    state.turn_mut().players_passed.insert(p(3));

    let card_id = find_object(&state, "DP1 Foretell Card");
    let (state, _) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("foretell should succeed");

    assert!(
        state.turn().players_passed.is_empty(),
        "CR 117.4: an action was taken between passes, so the pass-round must restart"
    );
    assert_eq!(
        state.turn().priority_holder,
        Some(p1),
        "CR 116.3: the acting player still holds priority"
    );
}

// ── P9 — special action actor holds priority after turning a permanent face up ──

#[test]
/// CR 116.2b, CR 116.3 — turning a face-down permanent face up is a special
/// action; the player who took it receives priority afterward, and
/// `players_passed` is empty (already true today; this pins the invariant
/// against Step 8's fix).
fn test_dp1_special_action_actor_holds_priority_after_turn_face_up() {
    let p1 = p(1);
    let p2 = p(2);

    let manifest_def = CardDefinition {
        card_id: CardId("dp1-manifested-creature".to_string()),
        name: "DP1 Manifested Creature".to_string(),
        mana_cost: Some(ManaCost::default()),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![manifest_def]);

    let spec = ObjectSpec::card(p2, "DP1 Manifested Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("dp1-manifested-creature".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let face_down_id = find_object(&state, "DP1 Manifested Creature");
    if let Some(obj) = state.objects_mut().get_mut(&face_down_id) {
        obj.status.face_down = true;
        obj.face_down_as = Some(FaceDownKind::Manifest);
    }
    state.turn_mut().priority_holder = Some(p2);

    let (state, _) = process_command(
        state,
        Command::TurnFaceUp {
            player: p2,
            permanent: face_down_id,
            method: TurnFaceUpMethod::ManaCost,
        },
    )
    .expect("CR 116.2b: p2 should be able to turn their manifested creature face up");

    assert!(
        state.turn().players_passed.is_empty(),
        "players_passed was already empty and should remain so"
    );
    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 116.3: the player who took the special action receives priority afterward"
    );
}
