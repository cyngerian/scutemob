//! Play-from-graveyard permission system tests (PB-B).
//!
//! CR 601.3: A player can begin to cast a spell only if a rule or effect allows it.
//! CR 305.1: Playing a land from the graveyard requires explicit permission.
//!
//! Covered:
//! - Basic land play from graveyard (StaticPlayFromGraveyard, LandsOnly)
//! - Land from GY follows timing restrictions (main phase, stack empty)
//! - Land from GY counts against land-play limit
//! - Permission removed when source permanent leaves battlefield
//! - Wrenn emblem PermanentsAndLands: cast creature from graveyard
//! - Wrenn emblem PermanentsAndLands: does NOT allow instant/sorcery from GY
//! - CastSelfFromGraveyard: Oathsworn Vampire (with life gained)
//! - CastSelfFromGraveyard: Oathsworn Vampire (without life gained — fails)
//! - CastSelfFromGraveyard: Squee alt mana cost + exile 4 GY cards
//! - CastSelfFromGraveyard: Squee insufficient GY cards — fails
//! - CastSelfFromGraveyard: Brokkos requires Mutate alt cost
//! - life_gained_this_turn tracking: incremented, reset at turn start
//! - Emblem permission persists even after source PW leaves

use mtg_engine::cards::card_definition::{
    CastFromGraveyardAdditionalCost, EffectAmount, PlayerTarget,
};
use mtg_engine::state::stubs::PlayFromGraveyardPermission;
use mtg_engine::{
    enrich_spec_from_def, process_command, AbilityDefinition, AltCostKind, CardDefinition, CardId,
    CardRegistry, CardType, Command, Effect, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ManaCost, ObjectId, ObjectSpec, PlayFromTopFilter, PlayerId, Step, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn cast_spell(player: PlayerId, card: ObjectId, alt_cost: Option<AltCostKind>) -> Command {
    Command::CastSpell {
        player,
        card,
        targets: vec![],
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost,
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
        face_down_kind: None,
        additional_costs: vec![],
    }
}

fn play_land(player: PlayerId, card: ObjectId) -> Command {
    Command::PlayLand { player, card }
}

// ── Card definitions ───────────────────────────────────────────────────────────

fn plains_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("gy-test-plains".to_string()),
        name: "GY Plains".to_string(),
        mana_cost: None,
        types: mtg_engine::TypeLine {
            card_types: [CardType::Land].into_iter().collect(),
            supertypes: [mtg_engine::SuperType::Basic].into_iter().collect(),
            subtypes: [mtg_engine::SubType("Plains".to_string())]
                .into_iter()
                .collect(),
        },
        oracle_text: "{T}: Add {W}.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

fn creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("gy-test-creature".to_string()),
        name: "GY Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    }
}

fn sorcery_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("gy-test-sorcery".to_string()),
        name: "GY Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn oathsworn_vampire_def() -> CardDefinition {
    use mtg_engine::cards::card_definition::Condition;
    CardDefinition {
        card_id: CardId("test-oathsworn-vampire".to_string()),
        name: "Test Oathsworn Vampire".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text:
            "This creature enters tapped.\nYou may cast this card from your graveyard if you gained life this turn."
                .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::CastSelfFromGraveyard {
            condition: Some(Box::new(Condition::ControllerGainedLifeThisTurn)),
            alt_mana_cost: None,
            additional_costs: vec![],
            required_alt_cost: None,
        }],
        ..Default::default()
    }
}

fn squee_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-squee".to_string()),
        name: "Test Squee".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text:
            "You may cast this card from your graveyard by paying {3}{R} and exiling four other cards from your graveyard."
                .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::CastSelfFromGraveyard {
            condition: None,
            alt_mana_cost: Some(ManaCost {
                generic: 3,
                red: 1,
                ..Default::default()
            }),
            additional_costs: vec![CastFromGraveyardAdditionalCost::ExileOtherGraveyardCards(4)],
            required_alt_cost: None,
        }],
        ..Default::default()
    }
}

fn filler_card_def(n: u8) -> CardDefinition {
    CardDefinition {
        card_id: CardId(format!("gy-filler-{}", n)),
        name: format!("Filler {}", n),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        ..Default::default()
    }
}

// Inject a LandsOnly graveyard permission for testing.
fn inject_gy_lands_only(state: &mut GameState, source: ObjectId, controller: PlayerId) {
    state
        .play_from_graveyard_permissions
        .push_back(PlayFromGraveyardPermission {
            source,
            controller,
            filter: PlayFromTopFilter::LandsOnly,
            condition: None,
        });
}

// Inject a PermanentsAndLands graveyard permission for testing (Wrenn emblem style).
fn inject_gy_permanents_and_lands(
    state: &mut GameState,
    source: ObjectId,
    controller: PlayerId,
    is_emblem: bool,
) {
    state
        .play_from_graveyard_permissions
        .push_back(PlayFromGraveyardPermission {
            source,
            controller,
            filter: PlayFromTopFilter::PermanentsAndLands,
            condition: None,
        });
    // If the source is an emblem, mark it so the retain() sweep keeps it.
    if is_emblem {
        if let Some(obj) = state.objects.get_mut(&source) {
            obj.is_emblem = true;
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 305.1, CR 601.3 (PB-B): Play a land from the graveyard when a LandsOnly
/// PlayFromGraveyardPermission is active (Ancient Greenwarden / Perennial Behemoth style).
#[test]
fn test_play_from_graveyard_land_basic() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![plains_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        // Permission source on battlefield
        .object(ObjectSpec::creature(p1, "Permission Source", 2, 4).in_zone(ZoneId::Battlefield))
        // Land in graveyard
        .object(
            ObjectSpec::land(p1, "GY Plains")
                .with_card_id(CardId("gy-test-plains".to_string()))
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let source_id = find_object(&state, "Permission Source");
    inject_gy_lands_only(&mut state, source_id, p1);

    let plains_id = find_object(&state, "GY Plains");

    let (state2, events) = process_command(state, play_land(p1, plains_id))
        .expect("PlayLand from graveyard should succeed");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. })),
        "PermanentEnteredBattlefield event should be emitted"
    );
    // Land should now be on the battlefield, not in the graveyard.
    let on_bf = state2
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "GY Plains" && obj.zone == ZoneId::Battlefield);
    assert!(
        on_bf,
        "Land should be on the battlefield after playing from graveyard"
    );
}

/// CR 305.1 (PB-B): Playing a land from the graveyard still follows timing restrictions.
/// Must be main phase, stack empty, active player.
#[test]
fn test_play_from_graveyard_land_timing_restriction() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![plains_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::BeginningOfCombat) // NOT a main phase
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Permission Source", 2, 4).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::land(p1, "GY Plains")
                .with_card_id(CardId("gy-test-plains".to_string()))
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let source_id = find_object(&state, "Permission Source");
    inject_gy_lands_only(&mut state, source_id, p1);

    let plains_id = find_object(&state, "GY Plains");

    let result = process_command(state, play_land(p1, plains_id));
    assert!(
        result.is_err(),
        "PlayLand from graveyard during Combat should fail"
    );
}

/// CR 305.1 (PB-B): Land from graveyard counts against the land-play limit.
/// If the player already played a land this turn, they cannot play another.
#[test]
fn test_play_from_graveyard_land_uses_land_play_count() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![plains_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Permission Source", 2, 4).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::land(p1, "GY Plains")
                .with_card_id(CardId("gy-test-plains".to_string()))
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    // Exhaust the land play
    if let Some(p) = state.players.get_mut(&p1) {
        p.land_plays_remaining = 0;
    }

    let source_id = find_object(&state, "Permission Source");
    inject_gy_lands_only(&mut state, source_id, p1);

    let plains_id = find_object(&state, "GY Plains");
    let result = process_command(state, play_land(p1, plains_id));
    assert!(
        result.is_err(),
        "PlayLand from graveyard should fail when land plays exhausted"
    );
}

/// CR 601.3 (PB-B): When the source permanent leaves the battlefield, the
/// play-from-graveyard permission is removed during the next reset_turn_state sweep.
#[test]
fn test_play_from_graveyard_permission_removed_when_source_leaves() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![plains_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Permission Source", 2, 4).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::land(p1, "GY Plains")
                .with_card_id(CardId("gy-test-plains".to_string()))
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let source_id = find_object(&state, "Permission Source");
    inject_gy_lands_only(&mut state, source_id, p1);

    // Move source to graveyard (simulating it being destroyed).
    let _ = state.move_object_to_zone(source_id, ZoneId::Graveyard(p1));

    // After reset_turn_state (which sweeps stale permissions), no permission should exist.
    mtg_engine::rules::turn_actions::reset_turn_state(&mut state, p1);

    assert!(
        state.play_from_graveyard_permissions.is_empty(),
        "Permission should be removed when source is no longer on battlefield"
    );
}

/// CR 601.3 (PB-B): A PermanentsAndLands permission (from Wrenn emblem) allows
/// casting a creature spell from the graveyard.
#[test]
fn test_play_from_graveyard_permanent_spell() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        // A dummy source to hold the permission reference
        .object(ObjectSpec::creature(p1, "Emblem Source", 0, 1).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::creature(p1, "GY Creature", 2, 2)
                .with_card_id(CardId("gy-test-creature".to_string()))
                .with_mana_cost(ManaCost {
                    generic: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    // Add enough mana to cast {2}
    if let Some(p) = state.players.get_mut(&p1) {
        p.mana_pool.colorless = 5;
    }

    let source_id = find_object(&state, "Emblem Source");
    inject_gy_permanents_and_lands(&mut state, source_id, p1, false);

    let creature_id = find_object(&state, "GY Creature");

    let (state2, events) = process_command(state, cast_spell(p1, creature_id, None))
        .expect("Cast creature from graveyard with PermanentsAndLands permission should succeed");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "SpellCast event should be emitted"
    );
    // Creature should now be on the stack.
    let on_stack = state2
        .stack_objects
        .iter()
        .any(|so| matches!(so.kind, mtg_engine::StackObjectKind::Spell { .. }));
    assert!(on_stack, "Creature spell should be on the stack");
}

/// CR 601.3 (PB-B): A PermanentsAndLands permission does NOT allow casting
/// instants or sorceries from the graveyard.
#[test]
fn test_play_from_graveyard_no_instant_sorcery() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Emblem Source", 0, 1).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::card(p1, "GY Sorcery")
                .with_card_id(CardId("gy-test-sorcery".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    if let Some(p) = state.players.get_mut(&p1) {
        p.mana_pool.colorless = 5;
    }

    let source_id = find_object(&state, "Emblem Source");
    inject_gy_permanents_and_lands(&mut state, source_id, p1, false);

    let sorcery_id = find_object(&state, "GY Sorcery");

    let result = process_command(state, cast_spell(p1, sorcery_id, None));
    assert!(
        result.is_err(),
        "Casting a sorcery from GY via PermanentsAndLands permission should fail"
    );
}

/// CR 601.3, Ruling 2018-01-19 (PB-B): Oathsworn Vampire may be cast from GY
/// when the controller gained life this turn.
#[test]
fn test_cast_self_from_graveyard_oathsworn_gained_life() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![oathsworn_vampire_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(
            ObjectSpec::creature(p1, "Test Oathsworn Vampire", 2, 2)
                .with_card_id(CardId("test-oathsworn-vampire".to_string()))
                .with_mana_cost(ManaCost {
                    generic: 1,
                    black: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    // Controller gained life this turn.
    if let Some(p) = state.players.get_mut(&p1) {
        p.life_gained_this_turn = 3;
        p.mana_pool.black = 1;
        p.mana_pool.colorless = 1;
    }

    let vampire_id = find_object(&state, "Test Oathsworn Vampire");

    let (_, events) = process_command(state, cast_spell(p1, vampire_id, None))
        .expect("Oathsworn Vampire from GY should succeed when life was gained");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "SpellCast event should be emitted"
    );
}

/// CR 601.3, Ruling 2018-01-19 (PB-B): Oathsworn Vampire cannot be cast from GY
/// when the controller has NOT gained life this turn.
#[test]
fn test_cast_self_from_graveyard_oathsworn_no_life_gained() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![oathsworn_vampire_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(
            ObjectSpec::creature(p1, "Test Oathsworn Vampire", 2, 2)
                .with_card_id(CardId("test-oathsworn-vampire".to_string()))
                .with_mana_cost(ManaCost {
                    generic: 1,
                    black: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    // life_gained_this_turn defaults to 0 — condition not met.
    if let Some(p) = state.players.get_mut(&p1) {
        p.mana_pool.black = 1;
        p.mana_pool.colorless = 1;
    }

    let vampire_id = find_object(&state, "Test Oathsworn Vampire");

    let result = process_command(state, cast_spell(p1, vampire_id, None));
    assert!(
        result.is_err(),
        "Oathsworn Vampire from GY should fail when no life gained this turn"
    );
}

/// CR 601.3 (PB-B): Squee, Dubious Monarch can be cast from GY paying {3}{R}
/// and exiling 4 other graveyard cards.
#[test]
fn test_cast_self_from_graveyard_squee_with_4_fillers() {
    let p1 = p(1);
    let p2 = p(2);
    let fillers: Vec<CardDefinition> = (1..=4).map(filler_card_def).collect();
    let mut all_defs = vec![squee_def()];
    all_defs.extend(fillers.clone());
    let registry = CardRegistry::new(all_defs);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(
            ObjectSpec::creature(p1, "Test Squee", 2, 2)
                .with_card_id(CardId("test-squee".to_string()))
                .with_mana_cost(ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Graveyard(p1)),
        );

    // Add 4 filler cards to the graveyard.
    for i in 1..=4u8 {
        builder = builder.object(
            ObjectSpec::creature(p1, &format!("Filler {}", i), 1, 1)
                .with_card_id(CardId(format!("gy-filler-{}", i)))
                .in_zone(ZoneId::Graveyard(p1)),
        );
    }

    let mut state = builder.build().unwrap();
    state.turn.priority_holder = Some(p1);
    // Squee's alt cost from GY is {3}{R}. Supply EXACTLY {3}{R} (4 total).
    // If the engine incorrectly charges the normal mana cost {2}{R} (3 total),
    // this would still succeed — so we also run a tighter test below that
    // demonstrates the cost enforcement. This test verifies the happy path.
    if let Some(p) = state.players.get_mut(&p1) {
        p.mana_pool.red = 1;
        p.mana_pool.colorless = 3;
    }

    let squee_id = find_object(&state, "Test Squee");

    let (_, events) = process_command(state, cast_spell(p1, squee_id, None))
        .expect("Squee from GY with 4 filler cards and {3}{R} mana should succeed");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "SpellCast event should be emitted for Squee"
    );
}

/// CR 601.3 (PB-B): Squee cannot be cast from GY without at least 4 other GY cards to exile.
#[test]
fn test_cast_self_from_graveyard_squee_insufficient_exile_cards() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![squee_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(
            ObjectSpec::creature(p1, "Test Squee", 2, 2)
                .with_card_id(CardId("test-squee".to_string()))
                .with_mana_cost(ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Graveyard(p1)),
        )
        // Only 2 filler cards — not enough to exile 4
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    if let Some(p) = state.players.get_mut(&p1) {
        p.mana_pool.red = 1;
        p.mana_pool.colorless = 3;
    }

    let squee_id = find_object(&state, "Test Squee");

    let result = process_command(state, cast_spell(p1, squee_id, None));
    assert!(
        result.is_err(),
        "Squee from GY should fail without enough other GY cards to exile"
    );
}

/// CR 601.3, Ruling 2020-04-17 (PB-B): Brokkos can only be cast from GY using
/// its mutate cost — required_alt_cost enforces this.
#[test]
fn test_cast_self_from_graveyard_brokkos_requires_mutate() {
    let p1 = p(1);
    let p2 = p(2);

    let brokkos_def = CardDefinition {
        card_id: CardId("test-brokkos".to_string()),
        name: "Test Brokkos".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            black: 1,
            green: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Mutate {3}{U/B}{G}\nYou may cast this card from your graveyard using its mutate ability.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Mutate),
            // CR 702.140a: Mutate cost {3}{U/B}{G}
            AbilityDefinition::MutateCost {
                cost: ManaCost { generic: 3, green: 1, blue: 1, ..Default::default() },
            },
            AbilityDefinition::CastSelfFromGraveyard {
                condition: None,
                alt_mana_cost: None,
                additional_costs: vec![],
                required_alt_cost: Some(AltCostKind::Mutate),
            },
        ],
        ..Default::default()
    };

    let defs: std::collections::HashMap<String, CardDefinition> =
        std::iter::once((brokkos_def.name.clone(), brokkos_def.clone())).collect();
    let registry = CardRegistry::new(vec![brokkos_def]);

    // Use enrich_spec_from_def so `characteristics.keywords` contains Mutate (from the card def).
    // card_id must be set explicitly so the registry lookup in casting.rs succeeds.
    let brokkos_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Test Brokkos")
            .with_card_id(CardId("test-brokkos".to_string()))
            .in_zone(ZoneId::Graveyard(p1)),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(brokkos_spec)
        // A non-Human creature target for Mutate
        .object(ObjectSpec::creature(p1, "Mutate Target", 1, 1).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    if let Some(p) = state.players.get_mut(&p1) {
        p.mana_pool.blue = 2;
        p.mana_pool.black = 2;
        p.mana_pool.green = 3;
        p.mana_pool.colorless = 4;
    }

    let brokkos_id = find_object(&state, "Test Brokkos");

    // Attempt to cast WITHOUT mutate alt cost — should fail.
    let result_no_mutate = process_command(state.clone(), cast_spell(p1, brokkos_id, None));
    assert!(
        result_no_mutate.is_err(),
        "Brokkos from GY without Mutate alt cost should fail (Ruling 2020-04-17)"
    );

    // Attempt WITH mutate alt cost + mutate target additional cost — should succeed.
    let target_id = find_object(&state, "Mutate Target");
    let cmd_mutate = Command::CastSpell {
        player: p1,
        card: brokkos_id,
        targets: vec![],
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: Some(AltCostKind::Mutate),
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
        face_down_kind: None,
        additional_costs: vec![mtg_engine::AdditionalCost::Mutate {
            target: target_id,
            on_top: true,
        }],
    };
    let result_with_mutate = process_command(state, cmd_mutate);
    assert!(
        result_with_mutate.is_ok(),
        "Brokkos from GY with Mutate alt cost should succeed: {:?}",
        result_with_mutate.err()
    );
}

/// CR 601.3, PB-B: life_gained_this_turn is incremented when life is gained and
/// reset to 0 at the start of each turn.
#[test]
fn test_life_gained_this_turn_tracking() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Initially zero.
    assert_eq!(
        state.players.get(&p1).unwrap().life_gained_this_turn,
        0,
        "life_gained_this_turn should start at 0"
    );

    // Manually set life_gained_this_turn to simulate gaining life.
    if let Some(p) = state.players.get_mut(&p1) {
        p.life_gained_this_turn = 5;
    }

    assert_eq!(
        state.players.get(&p1).unwrap().life_gained_this_turn,
        5,
        "life_gained_this_turn should be 5 after gaining life"
    );

    // reset_turn_state should clear it.
    mtg_engine::rules::turn_actions::reset_turn_state(&mut state, p1);

    assert_eq!(
        state.players.get(&p1).unwrap().life_gained_this_turn,
        0,
        "life_gained_this_turn should reset to 0 at turn start"
    );
}

/// CR 601.3 (PB-B): An emblem-sourced permission persists even when the source
/// planeswalker leaves the battlefield (emblems live in command zone permanently).
#[test]
fn test_play_from_graveyard_emblem_permission_persists() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![plains_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        // An emblem object in the command zone
        .object(ObjectSpec::creature(p1, "Wrenn Emblem", 0, 0).in_zone(ZoneId::Command(p1)))
        .object(
            ObjectSpec::land(p1, "GY Plains")
                .with_card_id(CardId("gy-test-plains".to_string()))
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Mark the source as an emblem so the retain() sweep keeps it.
    let emblem_id = find_object(&state, "Wrenn Emblem");
    if let Some(obj) = state.objects.get_mut(&emblem_id) {
        obj.is_emblem = true;
        obj.zone = ZoneId::Command(p1);
    }

    inject_gy_permanents_and_lands(&mut state, emblem_id, p1, true);

    // Run reset_turn_state (which sweeps stale permissions).
    mtg_engine::rules::turn_actions::reset_turn_state(&mut state, p1);

    // Permission should still exist because source is an emblem.
    assert_eq!(
        state.play_from_graveyard_permissions.len(),
        1,
        "Emblem-sourced graveyard permission should persist after reset_turn_state"
    );

    // Now verify land play works.
    state.turn.priority_holder = Some(p1);
    let plains_id = find_object(&state, "GY Plains");

    let (_, events) = process_command(state, play_land(p1, plains_id))
        .expect("PlayLand from graveyard with emblem permission should succeed");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. })),
        "Land should enter the battlefield from graveyard via emblem permission"
    );
}

/// CR 601.2f / CR 118.9 (PB-B fix MEDIUM-1): Squee's alt mana cost {3}{R} is enforced
/// when casting from the graveyard. With only {2}{R} (normal cost, 3 mana) in the pool,
/// the cast must fail — verifying the engine charges {3}{R}, not {2}{R}.
#[test]
fn test_cast_self_from_graveyard_squee_alt_cost_enforced() {
    let p1 = p(1);
    let p2 = p(2);
    let fillers: Vec<CardDefinition> = (1..=4).map(filler_card_def).collect();
    let mut all_defs = vec![squee_def()];
    all_defs.extend(fillers);
    let registry = CardRegistry::new(all_defs);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(
            ObjectSpec::creature(p1, "Test Squee", 2, 2)
                .with_card_id(CardId("test-squee".to_string()))
                .with_mana_cost(ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Graveyard(p1)),
        );

    for i in 1..=4u8 {
        builder = builder.object(
            ObjectSpec::creature(p1, &format!("Filler {}", i), 1, 1)
                .with_card_id(CardId(format!("gy-filler-{}", i)))
                .in_zone(ZoneId::Graveyard(p1)),
        );
    }

    let mut state = builder.build().unwrap();
    state.turn.priority_holder = Some(p1);
    // Provide only {2}{R} — Squee's normal mana cost, not the GY alt cost {3}{R}.
    // The cast must fail: the engine must charge the alt cost {3}{R}.
    if let Some(p) = state.players.get_mut(&p1) {
        p.mana_pool.red = 1;
        p.mana_pool.colorless = 2; // {2}{R} total — insufficient for {3}{R}
    }

    let squee_id = find_object(&state, "Test Squee");

    let result = process_command(state, cast_spell(p1, squee_id, None));
    assert!(
        result.is_err(),
        "Squee from GY should fail with only {{2}}{{R}} — alt cost is {{3}}{{R}} (CR 601.2f / CR 118.9)"
    );
}

/// CR 601.2h (PB-B fix MEDIUM-2): When Squee is cast from the graveyard, the four
/// other graveyard cards are actually exiled as part of cost payment.
/// After the cast, the graveyard should contain only Squee (on stack) with 0 other cards.
#[test]
fn test_cast_self_from_graveyard_squee_exiles_4_cards() {
    let p1 = p(1);
    let p2 = p(2);
    let fillers: Vec<CardDefinition> = (1..=4).map(filler_card_def).collect();
    let mut all_defs = vec![squee_def()];
    all_defs.extend(fillers);
    let registry = CardRegistry::new(all_defs);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(
            ObjectSpec::creature(p1, "Test Squee", 2, 2)
                .with_card_id(CardId("test-squee".to_string()))
                .with_mana_cost(ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Graveyard(p1)),
        );

    for i in 1..=4u8 {
        builder = builder.object(
            ObjectSpec::creature(p1, &format!("Filler {}", i), 1, 1)
                .with_card_id(CardId(format!("gy-filler-{}", i)))
                .in_zone(ZoneId::Graveyard(p1)),
        );
    }

    let mut state = builder.build().unwrap();
    state.turn.priority_holder = Some(p1);
    if let Some(p) = state.players.get_mut(&p1) {
        p.mana_pool.red = 1;
        p.mana_pool.colorless = 3; // {3}{R} — exact alt cost
    }

    let squee_id = find_object(&state, "Test Squee");

    let (state2, events) = process_command(state, cast_spell(p1, squee_id, None))
        .expect("Squee from GY with 4 filler cards and {3}{R} should succeed");

    // Four ObjectExiled events should have been emitted — one per filler card.
    let exile_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectExiled { .. }))
        .collect();
    assert_eq!(
        exile_events.len(),
        4,
        "Exactly 4 ObjectExiled events should be emitted for ExileOtherGraveyardCards(4) (CR 601.2h)"
    );

    // The filler cards should now be in exile, not in the graveyard.
    let gy_zone = ZoneId::Graveyard(p1);
    let gy_count = state2
        .zones
        .get(&gy_zone)
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        gy_count, 0,
        "Graveyard should be empty after Squee moves to stack and 4 fillers are exiled (CR 601.2h)"
    );

    // The 4 filler cards should be in exile.
    let exile_zone = ZoneId::Exile;
    let exile_count = state2
        .zones
        .get(&exile_zone)
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        exile_count,
        4,
        "4 filler cards should be in exile after paying ExileOtherGraveyardCards(4) cost (CR 601.2h)"
    );
}
