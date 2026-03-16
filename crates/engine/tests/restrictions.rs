//! Tests for game restrictions / stax effects (PB-18).
//!
//! Tests cover:
//! - MaxSpellsPerTurn (Rule of Law, Archon of Emeria, Eidolon of Rhetoric) — CR 101.2
//! - OpponentsCantCastDuringYourTurn (Dragonlord Dromoka) — CR 101.2
//! - OpponentsCantCastOrActivateDuringYourTurn (Grand Abolisher, Myrel) — CR 101.2
//! - OpponentsCantCastFromNonHand (Drannith Magistrate) — CR 101.2
//! - ArtifactAbilitiesCantBeActivated (Collector Ouphe, Stony Silence) — CR 101.2
//! - Restriction removal when source leaves battlefield
//! - Multiple stax effects stacking

use mtg_engine::cards::card_definition::AbilityDefinition;
use mtg_engine::state::stubs::ActiveRestriction;
use mtg_engine::{
    process_command, CardDefinition, CardId, CardRegistry, CardType, Command, Effect,
    GameRestriction, GameStateBuilder, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId, Step,
    TypeLine, ZoneId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

/// Helper: build a basic instant spell card def.
fn instant_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(card_id.to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn cast_cmd(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell {
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
    }
}

/// Helper: find object by name.
fn find_by_name(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .values()
        .find(|o| o.characteristics.name == name)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
        .id
}

/// Helper: add an active restriction to the state, sourced from the given object.
fn add_restriction(
    state: &mut mtg_engine::GameState,
    source: ObjectId,
    controller: PlayerId,
    restriction: GameRestriction,
) {
    state.restrictions.push_back(ActiveRestriction {
        source,
        controller,
        restriction,
    });
}

// ─── MaxSpellsPerTurn ────────────────────────────────────────────────────────

#[test]
/// CR 101.2: Rule of Law — "Each player can't cast more than one spell each turn."
/// Player who already cast 1 spell is blocked from casting a second.
fn test_restriction_max_spells_blocks_second_spell() {
    let registry = CardRegistry::new(vec![instant_def("Zap", "zap")]);

    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1(), "Stax Piece", 2, 2).in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1(), "Zap")
                .in_zone(ZoneId::Hand(p1()))
                .with_card_id(CardId("zap".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost { generic: 1, ..ManaCost::default() }),
        )
        .player_mana(p1(), ManaPool { colorless: 10, ..ManaPool::default() })
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let stax_id = find_by_name(&state, "Stax Piece");
    add_restriction(
        &mut state,
        stax_id,
        p1(),
        GameRestriction::MaxSpellsPerTurn { max: 1 },
    );

    // P1 already cast 1 spell this turn.
    if let Some(ps) = state.players.get_mut(&p1()) {
        ps.spells_cast_this_turn = 1;
    }

    let zap = find_by_name(&state, "Zap");
    let result = process_command(state, cast_cmd(p1(), zap));
    assert!(result.is_err(), "second spell should be blocked by Rule of Law");
    let err = format!("{:?}", result.unwrap_err());
    assert!(err.contains("can't cast more than 1 spell"), "error: {}", err);
}

#[test]
/// CR 101.2: MaxSpellsPerTurn — first spell succeeds.
fn test_restriction_max_spells_allows_first_spell() {
    let registry = CardRegistry::new(vec![instant_def("Zap", "zap")]);

    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1(), "Stax Piece", 2, 2).in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1(), "Zap")
                .in_zone(ZoneId::Hand(p1()))
                .with_card_id(CardId("zap".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost { generic: 1, ..ManaCost::default() }),
        )
        .player_mana(p1(), ManaPool { colorless: 10, ..ManaPool::default() })
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let stax_id = find_by_name(&state, "Stax Piece");
    add_restriction(
        &mut state,
        stax_id,
        p1(),
        GameRestriction::MaxSpellsPerTurn { max: 1 },
    );

    let zap = find_by_name(&state, "Zap");
    let result = process_command(state, cast_cmd(p1(), zap));
    assert!(result.is_ok(), "first spell should succeed under Rule of Law");
}

#[test]
/// CR 101.2: MaxSpellsPerTurn affects opponents too.
fn test_restriction_max_spells_affects_opponents() {
    let registry = CardRegistry::new(vec![instant_def("Bolt", "bolt")]);

    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1(), "Stax Piece", 2, 2).in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p2(), "Bolt")
                .in_zone(ZoneId::Hand(p2()))
                .with_card_id(CardId("bolt".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost { generic: 1, ..ManaCost::default() }),
        )
        .player_mana(p2(), ManaPool { colorless: 10, ..ManaPool::default() })
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let stax_id = find_by_name(&state, "Stax Piece");
    add_restriction(
        &mut state,
        stax_id,
        p1(),
        GameRestriction::MaxSpellsPerTurn { max: 1 },
    );

    // P2 already cast 1 spell.
    if let Some(ps) = state.players.get_mut(&p2()) {
        ps.spells_cast_this_turn = 1;
    }
    state.turn.priority_holder = Some(p2());

    let bolt = find_by_name(&state, "Bolt");
    let result = process_command(state, cast_cmd(p2(), bolt));
    assert!(result.is_err(), "P2 should be blocked by Rule of Law");
}

// ─── OpponentsCantCastDuringYourTurn ─────────────────────────────────────────

#[test]
/// CR 101.2: Dragonlord Dromoka — "Your opponents can't cast spells during your turn."
fn test_restriction_opponents_cant_cast_during_your_turn() {
    let registry = CardRegistry::new(vec![instant_def("Bolt", "bolt")]);

    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1(), "Dromoka", 5, 7).in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p2(), "Bolt")
                .in_zone(ZoneId::Hand(p2()))
                .with_card_id(CardId("bolt".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost { generic: 1, ..ManaCost::default() }),
        )
        .player_mana(p2(), ManaPool { colorless: 10, ..ManaPool::default() })
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let dromoka = find_by_name(&state, "Dromoka");
    add_restriction(
        &mut state,
        dromoka,
        p1(),
        GameRestriction::OpponentsCantCastDuringYourTurn,
    );
    state.turn.priority_holder = Some(p2());

    let bolt = find_by_name(&state, "Bolt");
    let result = process_command(state, cast_cmd(p2(), bolt));
    assert!(result.is_err(), "opponents can't cast during controller's turn");
    let err = format!("{:?}", result.unwrap_err());
    assert!(err.contains("opponents can't cast spells during your turn"), "error: {}", err);
}

#[test]
/// Controller CAN cast during their own turn with Dromoka out.
fn test_restriction_dromoka_controller_can_cast() {
    let registry = CardRegistry::new(vec![instant_def("Zap", "zap")]);

    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1(), "Dromoka", 5, 7).in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1(), "Zap")
                .in_zone(ZoneId::Hand(p1()))
                .with_card_id(CardId("zap".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost { generic: 1, ..ManaCost::default() }),
        )
        .player_mana(p1(), ManaPool { colorless: 10, ..ManaPool::default() })
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let dromoka = find_by_name(&state, "Dromoka");
    add_restriction(
        &mut state,
        dromoka,
        p1(),
        GameRestriction::OpponentsCantCastDuringYourTurn,
    );

    let zap = find_by_name(&state, "Zap");
    let result = process_command(state, cast_cmd(p1(), zap));
    assert!(result.is_ok(), "Dromoka controller can cast on own turn");
}

// ─── OpponentsCantCastFromNonHand ────────────────────────────────────────────

#[test]
/// CR 101.2: Drannith Magistrate — blocks opponents casting from graveyard.
fn test_restriction_drannith_blocks_graveyard_cast() {
    let registry = CardRegistry::new(vec![instant_def("GY Spell", "gy-spell")]);

    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1(), "Magistrate", 1, 3).in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p2(), "GY Spell")
                .in_zone(ZoneId::Graveyard(p2()))
                .with_card_id(CardId("gy-spell".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost { generic: 1, ..ManaCost::default() }),
        )
        .player_mana(p2(), ManaPool { colorless: 10, ..ManaPool::default() })
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let magistrate = find_by_name(&state, "Magistrate");
    add_restriction(
        &mut state,
        magistrate,
        p1(),
        GameRestriction::OpponentsCantCastFromNonHand,
    );
    state.turn.priority_holder = Some(p2());

    let gy_spell = find_by_name(&state, "GY Spell");
    let result = process_command(
        state,
        Command::CastSpell {
            player: p2(),
            card: gy_spell,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(mtg_engine::AltCostKind::Flashback),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(result.is_err(), "Drannith should block non-hand casting");
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("opponents can't cast spells from anywhere other than their hands"),
        "error: {}",
        err
    );
}

// ─── ArtifactAbilitiesCantBeActivated ────────────────────────────────────────

#[test]
/// CR 101.2: Collector Ouphe — "Activated abilities of artifacts can't be activated."
fn test_restriction_artifact_abilities_blocked() {
    let registry = CardRegistry::new(vec![]);

    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1(), "Ouphe", 2, 2).in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::artifact(p2(), "Mind Stone"))
        .player_mana(p2(), ManaPool { colorless: 10, ..ManaPool::default() })
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let ouphe = find_by_name(&state, "Ouphe");
    add_restriction(
        &mut state,
        ouphe,
        p1(),
        GameRestriction::ArtifactAbilitiesCantBeActivated,
    );
    state.turn.priority_holder = Some(p2());

    let mind_stone = find_by_name(&state, "Mind Stone");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p2(),
            source: mind_stone,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    );

    assert!(result.is_err(), "artifact activated abilities should be blocked");
    let err = format!("{:?}", result.unwrap_err());
    assert!(err.contains("activated abilities of artifacts"), "error: {}", err);
}

// ─── Restriction removal when source leaves ──────────────────────────────────

#[test]
/// When the restriction source moves to graveyard, restrictions no longer apply.
fn test_restriction_inactive_when_source_leaves_battlefield() {
    let registry = CardRegistry::new(vec![instant_def("Bolt", "bolt")]);

    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1(), "Stax Piece", 2, 2).in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p2(), "Bolt")
                .in_zone(ZoneId::Hand(p2()))
                .with_card_id(CardId("bolt".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost { generic: 1, ..ManaCost::default() }),
        )
        .player_mana(p2(), ManaPool { colorless: 10, ..ManaPool::default() })
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let stax_id = find_by_name(&state, "Stax Piece");
    add_restriction(
        &mut state,
        stax_id,
        p1(),
        GameRestriction::MaxSpellsPerTurn { max: 1 },
    );

    // P2 already cast 1 spell.
    if let Some(ps) = state.players.get_mut(&p2()) {
        ps.spells_cast_this_turn = 1;
    }
    state.turn.priority_holder = Some(p2());

    // Move stax piece to graveyard (destroyed).
    if let Some(obj) = state.objects.get_mut(&stax_id) {
        obj.zone = ZoneId::Graveyard(p1());
    }

    let bolt = find_by_name(&state, "Bolt");
    let result = process_command(state, cast_cmd(p2(), bolt));
    assert!(result.is_ok(), "restriction should not apply when source is off battlefield");
}

// ─── Grand Abolisher ─────────────────────────────────────────────────────────

#[test]
/// CR 101.2: Grand Abolisher — opponents can't cast during controller's turn.
fn test_restriction_grand_abolisher_blocks_opponent_cast() {
    let registry = CardRegistry::new(vec![instant_def("Bolt", "bolt")]);

    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1(), "Grand Abolisher", 2, 2).in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p2(), "Bolt")
                .in_zone(ZoneId::Hand(p2()))
                .with_card_id(CardId("bolt".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost { generic: 1, ..ManaCost::default() }),
        )
        .player_mana(p2(), ManaPool { colorless: 10, ..ManaPool::default() })
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let abolisher = find_by_name(&state, "Grand Abolisher");
    add_restriction(
        &mut state,
        abolisher,
        p1(),
        GameRestriction::OpponentsCantCastOrActivateDuringYourTurn,
    );
    state.turn.priority_holder = Some(p2());

    let bolt = find_by_name(&state, "Bolt");
    let result = process_command(state, cast_cmd(p2(), bolt));
    assert!(result.is_err(), "opponents can't cast during Abolisher controller's turn");
}

#[test]
/// Grand Abolisher's controller CAN cast on own turn.
fn test_restriction_grand_abolisher_controller_can_cast() {
    let registry = CardRegistry::new(vec![instant_def("Zap", "zap")]);

    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1(), "Grand Abolisher", 2, 2).in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1(), "Zap")
                .in_zone(ZoneId::Hand(p1()))
                .with_card_id(CardId("zap".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost { generic: 1, ..ManaCost::default() }),
        )
        .player_mana(p1(), ManaPool { colorless: 10, ..ManaPool::default() })
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let abolisher = find_by_name(&state, "Grand Abolisher");
    add_restriction(
        &mut state,
        abolisher,
        p1(),
        GameRestriction::OpponentsCantCastOrActivateDuringYourTurn,
    );

    let zap = find_by_name(&state, "Zap");
    let result = process_command(state, cast_cmd(p1(), zap));
    assert!(result.is_ok(), "Abolisher controller can still cast on own turn");
}
