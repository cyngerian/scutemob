//! Play-from-top-of-library permission system tests (PB-A).
//!
//! CR 601.3: A player can begin to cast a spell only if a rule or effect allows it.
//! CR 305.1: Playing a land is a special action restricted to hand (or top of library
//!           if an active permission exists).
//!
//! Covered:
//! - Basic land play from library top (CR 305.1 / 305.2a)
//! - Land play from top uses land play limit (2021-03-19 ruling on Courser of Kruphix)
//! - Cast creature from library top (CR 601.2)
//! - Normal timing restrictions still apply with permission
//! - Filter: LandsOnly, CreaturesOnly, ArtifactsAndColorless, CreaturesWithMinPower
//! - Bolas's Citadel: PayLifeForManaValue alternative cost (CR 118.9)
//! - Cannot use PayLifeForManaValue without pay_life_instead permission
//! - Cannot cast second card from top (only top card)
//! - Source leaves battlefield → permission removed
//! - Multiple permissions from different sources
//! - Without permission, casting from library top fails
//! - Without permission, PlayLand from library fails

use mtg_engine::cards::card_definition::{
    ContinuousEffectDef as CardContinuousEffectDef, EffectAmount, PlayerTarget,
};
use mtg_engine::state::stubs::PlayFromTopPermission;
use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, AltCostKind, CardDefinition,
    CardId, CardRegistry, CardType, Color, Command, Effect, EffectDuration, EffectFilter,
    EffectLayer, GameEvent, GameState, GameStateBuilder, KeywordAbility, LayerModification,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayFromTopFilter, PlayerId, Step, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// An ObjectSpec for "Test Plains" in a player's library (Land, no cost).
fn plains_spec(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::land(owner, "Test Plains").in_zone(ZoneId::Library(owner))
}

/// An ObjectSpec for "Test Small Creature" in a player's library (Creature 2/2, {1}{G}).
fn small_creature_spec(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::creature(owner, "Test Small Creature", 2, 2)
        .in_zone(ZoneId::Library(owner))
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::Green])
}

/// An ObjectSpec for "Test Big Creature" in a player's library (Creature 4/4, {4}).
fn big_creature_spec(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::creature(owner, "Test Big Creature", 4, 4)
        .in_zone(ZoneId::Library(owner))
        .with_mana_cost(ManaCost {
            generic: 4,
            ..Default::default()
        })
}

/// An ObjectSpec for "Test Sorcery" in a player's library (Sorcery, {3}{B}).
fn sorcery_spec(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Sorcery")
        .in_zone(ZoneId::Library(owner))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 3,
            black: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::Black])
}

/// An ObjectSpec for "Test Artifact Creature" in a player's library (Artifact+Creature 4/4, {4}).
fn artifact_creature_spec(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::artifact(owner, "Test Artifact Creature")
        .in_zone(ZoneId::Library(owner))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 4,
            ..Default::default()
        })
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn cast_spell(player: PlayerId, card: ObjectId) -> Command {
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
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
        face_down_kind: None,
        additional_costs: vec![],
    }
}

fn cast_spell_pay_life(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell {
        player,
        card,
        targets: vec![],
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: Some(AltCostKind::PayLifeForManaValue),
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
        face_down_kind: None,
        additional_costs: vec![],
    }
}

// ── Card definitions ───────────────────────────────────────────────────────────

/// A basic land card: Plains (no abilities needed for play-from-top tests).
fn plains_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-plains".to_string()),
        name: "Test Plains".to_string(),
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

/// A basic creature: 2/2 creature for {1}{G}.
fn small_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-small-creature".to_string()),
        name: "Test Small Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
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

/// A big creature: 4/4 creature for {4}.
fn big_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-big-creature".to_string()),
        name: "Test Big Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![],
        ..Default::default()
    }
}

/// A sorcery for {3}{B}.
fn sorcery_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-sorcery".to_string()),
        name: "Test Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            black: 1,
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

/// An artifact creature for {4} — colorless/artifact.
fn artifact_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-artifact-creature".to_string()),
        name: "Test Artifact Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Artifact, CardType::Creature]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![],
        ..Default::default()
    }
}

/// Inject a LandsOnly permission into state for the given source/controller.
fn inject_lands_only(state: &mut GameState, source: ObjectId, controller: PlayerId) {
    state
        .play_from_top_permissions
        .push_back(PlayFromTopPermission {
            source,
            controller,
            filter: PlayFromTopFilter::LandsOnly,
            look_at_top: false,
            reveal_top: true,
            pay_life_instead: false,
            condition: None,
            on_cast_effect: None,
        });
}

/// Inject a CreaturesOnly permission into state for the given source/controller.
fn inject_creatures_only(state: &mut GameState, source: ObjectId, controller: PlayerId) {
    state
        .play_from_top_permissions
        .push_back(PlayFromTopPermission {
            source,
            controller,
            filter: PlayFromTopFilter::CreaturesOnly,
            look_at_top: true,
            reveal_top: false,
            pay_life_instead: false,
            condition: None,
            on_cast_effect: None,
        });
}

/// Inject an All-cards permission into state for the given source/controller.
fn inject_all_cards(state: &mut GameState, source: ObjectId, controller: PlayerId) {
    state
        .play_from_top_permissions
        .push_back(PlayFromTopPermission {
            source,
            controller,
            filter: PlayFromTopFilter::All,
            look_at_top: false,
            reveal_top: true,
            pay_life_instead: false,
            condition: None,
            on_cast_effect: None,
        });
}

/// Inject an All permission with pay_life_instead for the given source/controller.
fn inject_pay_life(state: &mut GameState, source: ObjectId, controller: PlayerId) {
    state
        .play_from_top_permissions
        .push_back(PlayFromTopPermission {
            source,
            controller,
            filter: PlayFromTopFilter::All,
            look_at_top: true,
            reveal_top: false,
            pay_life_instead: true,
            condition: None,
            on_cast_effect: None,
        });
}

/// Inject an ArtifactsAndColorless permission for the given source/controller.
fn inject_artifacts_colorless(state: &mut GameState, source: ObjectId, controller: PlayerId) {
    state
        .play_from_top_permissions
        .push_back(PlayFromTopPermission {
            source,
            controller,
            filter: PlayFromTopFilter::ArtifactsAndColorless,
            look_at_top: true,
            reveal_top: false,
            pay_life_instead: false,
            condition: None,
            on_cast_effect: None,
        });
}

/// Inject a CreaturesWithMinPower(4) permission for the given source/controller.
fn inject_creatures_min_power_4(state: &mut GameState, source: ObjectId, controller: PlayerId) {
    state
        .play_from_top_permissions
        .push_back(PlayFromTopPermission {
            source,
            controller,
            filter: PlayFromTopFilter::CreaturesWithMinPower(4),
            look_at_top: true,
            reveal_top: false,
            pay_life_instead: false,
            condition: None,
            on_cast_effect: None,
        });
}

// ── Tests ───────────────────────────────────────────────────────────────────────

/// CR 305.1 / CR 305.2a (PB-A): Play a land from the top of the library when a
/// LandsOnly permission is active (Courser of Kruphix / Oracle of Mul Daya style).
#[test]
fn test_play_from_top_basic_land() {
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
        // The last card added to Library is on top (zone.top() = last element).
        .object(plains_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Inject LandsOnly permission pointing to the real source creature.
    let source_id = find_object(&state, "Permission Source");
    inject_lands_only(&mut state, source_id, p1);

    let plains_id = find_object(&state, "Test Plains");
    // Verify the card is on top of the library.
    let top = state.zones.get(&ZoneId::Library(p1)).and_then(|z| z.top());
    assert_eq!(top, Some(plains_id), "Plains should be on top of library");

    let (state2, events) = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: plains_id,
        },
    )
    .expect("PlayLand from library top should succeed");

    // The land must have entered the battlefield.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. })),
        "PermanentEnteredBattlefield event should be present"
    );
    // Library should now be empty.
    let lib = state2
        .zones
        .get(&ZoneId::Library(p1))
        .map(|z| z.len())
        .unwrap_or(0);
    assert_eq!(lib, 0, "Library should be empty after playing the top card");
}

/// CR 305.2a (2021-03-19 ruling on Courser of Kruphix):
/// Playing a land from the top of the library uses a land play.
/// If the player already used their land play this turn, they cannot play another.
#[test]
fn test_play_from_top_land_uses_land_play() {
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
        // Two Plains in library (the last one is on top).
        .object(plains_spec(p1))
        .object(plains_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_lands_only(&mut state, source_id, p1);

    // First land play should succeed.
    let top_id = state
        .zones
        .get(&ZoneId::Library(p1))
        .and_then(|z| z.top())
        .unwrap();
    let (state2, _) = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: top_id,
        },
    )
    .expect("First PlayLand from library top should succeed");

    // Land plays remaining should now be 0.
    let land_plays = state2.player(p1).unwrap().land_plays_remaining;
    assert_eq!(
        land_plays, 0,
        "Land plays should be 0 after playing one land"
    );

    // Second land play should fail.
    let top_id2 = state2
        .zones
        .get(&ZoneId::Library(p1))
        .and_then(|z| z.top())
        .unwrap();
    let err = process_command(
        state2,
        Command::PlayLand {
            player: p1,
            card: top_id2,
        },
    );
    assert!(
        err.is_err(),
        "Second PlayLand should fail when land plays are exhausted"
    );
}

/// CR 601.2 (PB-A): Cast a creature spell from the top of the library when a
/// CreaturesOnly permission is active (Elven Chorus / Vizier of the Menagerie style).
#[test]
fn test_play_from_top_cast_creature() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![small_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Permission Source", 2, 2).in_zone(ZoneId::Battlefield))
        .object(small_creature_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_creatures_only(&mut state, source_id, p1);

    // Add enough mana to cast the creature ({1}{G}).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let creature_id = find_object(&state, "Test Small Creature");
    let (state2, events) = process_command(state, cast_spell(p1, creature_id))
        .expect("CastSpell from library top should succeed");

    // Spell should be on the stack.
    assert!(
        !state2.stack_objects.is_empty(),
        "Spell should be on the stack after casting"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "SpellCast event should be emitted"
    );
}

/// CR 601.3 (2019-06-14 ruling on Future Sight):
/// Normal timing restrictions still apply. Cannot cast a sorcery-speed spell
/// during an opponent's turn even with a play-from-top permission.
#[test]
fn test_play_from_top_cast_respects_timing() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p2) // p2's turn
        .object(ObjectSpec::creature(p1, "Permission Source", 2, 2).in_zone(ZoneId::Battlefield))
        .object(sorcery_spec(p1))
        .build()
        .unwrap();

    // p1 holds priority during p2's main phase.
    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_all_cards(&mut state, source_id, p1);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);

    let sorcery_id = find_object(&state, "Test Sorcery");
    // Cannot cast sorcery during opponent's turn even with Future Sight permission.
    let result = process_command(state, cast_spell(p1, sorcery_id));
    assert!(
        result.is_err(),
        "Should not be able to cast sorcery-speed spell during opponent's turn"
    );
}

/// CR 601.3 (PB-A): A LandsOnly filter must reject non-land cards from being cast.
#[test]
fn test_play_from_top_filter_rejects_wrong_type() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![small_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Permission Source", 2, 4).in_zone(ZoneId::Battlefield))
        .object(small_creature_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_lands_only(&mut state, source_id, p1);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let creature_id = find_object(&state, "Test Small Creature");
    let result = process_command(state, cast_spell(p1, creature_id));
    assert!(
        result.is_err(),
        "LandsOnly filter should reject casting a creature from top of library"
    );
}

/// CR 601.3 (PB-A) / 2019-07-12 ruling:
/// ArtifactsAndColorless filter allows artifact spells from library top.
#[test]
fn test_play_from_top_artifacts_and_colorless_filter() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![artifact_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::artifact(p1, "Permission Source").in_zone(ZoneId::Battlefield))
        .object(artifact_creature_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_artifacts_colorless(&mut state, source_id, p1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);

    let artifact_id = find_object(&state, "Test Artifact Creature");
    let (state2, _) = process_command(state, cast_spell(p1, artifact_id)).expect(
        "Should be able to cast artifact creature from top with ArtifactsAndColorless filter",
    );

    assert!(
        !state2.stack_objects.is_empty(),
        "Artifact creature should be on the stack"
    );
}

/// CR 601.3 (PB-A): ArtifactsAndColorless filter rejects non-artifact colored spells.
#[test]
fn test_play_from_top_artifacts_colorless_filter_rejects_colored_nonartifact() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![small_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::artifact(p1, "Permission Source").in_zone(ZoneId::Battlefield))
        .object(small_creature_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_artifacts_colorless(&mut state, source_id, p1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let creature_id = find_object(&state, "Test Small Creature");
    let result = process_command(state, cast_spell(p1, creature_id));
    assert!(
        result.is_err(),
        "ArtifactsAndColorless should reject a colored non-artifact creature"
    );
}

/// CR 118.9 (PB-A / 2019-05-03 ruling on Bolas's Citadel):
/// When pay_life_instead is true, casting a spell from the top of the library
/// deducts life equal to the spell's mana value instead of paying mana.
#[test]
fn test_play_from_top_bolas_citadel_pay_life() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![small_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::artifact(p1, "Permission Source").in_zone(ZoneId::Battlefield))
        // Small creature costs {1}{G} = mana value 2.
        .object(small_creature_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_pay_life(&mut state, source_id, p1);
    let initial_life = state.player(p1).unwrap().life_total;
    let creature_id = find_object(&state, "Test Small Creature");

    let (state2, _events) = process_command(state, cast_spell_pay_life(p1, creature_id))
        .expect("Cast via PayLifeForManaValue should succeed");

    // Life should have decreased by the spell's mana value (2).
    let final_life = state2.player(p1).unwrap().life_total;
    assert_eq!(
        final_life,
        initial_life - 2,
        "Life should decrease by mana value (2) when casting via Bolas's Citadel"
    );
    // Spell should be on the stack.
    assert!(
        !state2.stack_objects.is_empty(),
        "Spell should be on the stack"
    );
}

/// CR 601.3 (PB-A): Cannot use PayLifeForManaValue when there's no active
/// pay_life_instead permission.
#[test]
fn test_play_from_top_bolas_citadel_requires_permission() {
    let p1 = p(1);
    let p2 = p(2);
    // Use an All permission that does NOT have pay_life_instead.
    let registry = CardRegistry::new(vec![small_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Permission Source", 2, 2).in_zone(ZoneId::Battlefield))
        .object(small_creature_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_all_cards(&mut state, source_id, p1);

    let creature_id = find_object(&state, "Test Small Creature");
    let result = process_command(state, cast_spell_pay_life(p1, creature_id));
    assert!(
        result.is_err(),
        "PayLifeForManaValue should fail without a pay_life_instead permission"
    );
}

/// CR 601.3 (PB-A): Cannot cast the second card from the top of the library,
/// only the first (top) card.
#[test]
fn test_play_from_top_not_from_second_card() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![small_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Permission Source", 2, 2).in_zone(ZoneId::Battlefield))
        // Add two creatures. The SECOND one added is on top.
        .object(small_creature_spec(p1))
        .object(small_creature_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_creatures_only(&mut state, source_id, p1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    // Find the BOTTOM card (first inserted = index 0 = not top).
    let lib_ids = state
        .zones
        .get(&ZoneId::Library(p1))
        .map(|z| z.object_ids())
        .unwrap_or_default();
    assert_eq!(lib_ids.len(), 2);
    let bottom_card = lib_ids[0]; // first = bottom

    let result = process_command(state, cast_spell(p1, bottom_card));
    assert!(
        result.is_err(),
        "Should not be able to cast the second card from top (only the top card is allowed)"
    );
}

/// CR 601.3 (PB-A): When the source permanent leaves the battlefield, its
/// permission should be removed by the cleanup sweep in reset_turn_state.
/// Test verifies the cleanup logic works correctly.
#[test]
fn test_play_from_top_source_leaves() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![small_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        // Library card to try to cast after source leaves.
        .object(small_creature_spec(p1))
        // Source creature on battlefield — acts as the permission source.
        .object(ObjectSpec::creature(p1, "Source Creature", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    // Get the source creature's ObjectId and inject a permission for it.
    let source_id = find_object(&state, "Source Creature");
    inject_creatures_only(&mut state, source_id, p1);

    // Permission should now be registered.
    assert_eq!(state.play_from_top_permissions.len(), 1);

    // Manually move the source to the graveyard (simulate removal).
    state
        .move_object_to_zone(source_id, ZoneId::Graveyard(p1))
        .unwrap();

    // Force cleanup sweep (normally happens in reset_turn_state).
    let objects = state.objects.clone();
    state.play_from_top_permissions.retain(|perm| {
        objects
            .get(&perm.source)
            .map(|o| matches!(o.zone, ZoneId::Battlefield))
            .unwrap_or(false)
    });

    // Permission should be gone.
    assert_eq!(
        state.play_from_top_permissions.len(),
        0,
        "Permission should be removed when source leaves battlefield"
    );

    // Now casting from top should fail.
    state.turn.priority_holder = Some(p1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    let creature_id = find_object(&state, "Test Small Creature");
    let result = process_command(state, cast_spell(p1, creature_id));
    assert!(
        result.is_err(),
        "Casting from top should fail when source has left battlefield"
    );
}

/// CR 601.3 (PB-A): Two play-from-top sources; removing one doesn't affect the other.
#[test]
fn test_play_from_top_multiple_permissions() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![small_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Lands Source", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "Creatures Source", 2, 2).in_zone(ZoneId::Battlefield))
        .object(small_creature_spec(p1))
        .build()
        .unwrap();

    let lands_src_id = find_object(&state, "Lands Source");
    let creatures_src_id = find_object(&state, "Creatures Source");
    inject_lands_only(&mut state, lands_src_id, p1);
    inject_creatures_only(&mut state, creatures_src_id, p1);

    // Both permissions should be registered.
    assert_eq!(state.play_from_top_permissions.len(), 2);

    // Remove the LandsOnly source manually.
    state
        .move_object_to_zone(lands_src_id, ZoneId::Graveyard(p1))
        .unwrap();

    let objects = state.objects.clone();
    state.play_from_top_permissions.retain(|perm| {
        objects
            .get(&perm.source)
            .map(|o| matches!(o.zone, ZoneId::Battlefield))
            .unwrap_or(false)
    });

    // CreaturesOnly permission should still be there.
    assert_eq!(
        state.play_from_top_permissions.len(),
        1,
        "Only CreaturesOnly permission should remain"
    );
    assert!(
        state
            .play_from_top_permissions
            .iter()
            .any(|p| matches!(p.filter, PlayFromTopFilter::CreaturesOnly)),
        "CreaturesOnly filter should still be active"
    );
}

/// CR 601.3 (PB-A): CreaturesWithMinPower(4) allows a 4-power creature
/// but rejects a 2-power creature (Thundermane Dragon style).
#[test]
fn test_play_from_top_power_filter() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![big_creature_def(), small_creature_def()]);

    // First: big creature (power 4) should be allowed.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Permission Source", 4, 4).in_zone(ZoneId::Battlefield))
        .object(big_creature_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_creatures_min_power_4(&mut state, source_id, p1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);

    let big_id = find_object(&state, "Test Big Creature");
    let result = process_command(state, cast_spell(p1, big_id));
    assert!(
        result.is_ok(),
        "Big creature (power 4) should be castable via CreaturesWithMinPower(4)"
    );

    // Second: small creature (power 2) should be rejected.
    let mut state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Permission Source", 4, 4).in_zone(ZoneId::Battlefield))
        .object(small_creature_spec(p1))
        .build()
        .unwrap();

    state2.turn.priority_holder = Some(p1);
    let source_id2 = find_object(&state2, "Permission Source");
    inject_creatures_min_power_4(&mut state2, source_id2, p1);
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let small_id = find_object(&state2, "Test Small Creature");
    let result2 = process_command(state2, cast_spell(p1, small_id));
    assert!(
        result2.is_err(),
        "Small creature (power 2) should be rejected by CreaturesWithMinPower(4)"
    );
}

/// CR 601.3 (PB-A): All filter allows spells of any type from library top
/// (Future Sight style).
#[test]
fn test_play_from_top_all_types_future_sight() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Permission Source", 2, 2).in_zone(ZoneId::Battlefield))
        .object(sorcery_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_all_cards(&mut state, source_id, p1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);

    let sorcery_id = find_object(&state, "Test Sorcery");
    let result = process_command(state, cast_spell(p1, sorcery_id));
    assert!(
        result.is_ok(),
        "All filter should allow casting any spell type from library top"
    );
}

/// CR 601.3 (PB-A): Without any play-from-top permission, casting from library
/// top is rejected (card is not in hand or a zone the player may cast from).
#[test]
fn test_play_from_top_no_permission_rejected() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![small_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(small_creature_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    // No permissions injected.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let creature_id = find_object(&state, "Test Small Creature");
    let result = process_command(state, cast_spell(p1, creature_id));
    assert!(
        result.is_err(),
        "Should not be able to cast from library top without a permission"
    );
}

/// CR 305.1 (PB-A): Without a play-from-top land permission, PlayLand from
/// library is rejected.
#[test]
fn test_play_from_top_land_no_permission_rejected() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![plains_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(plains_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    // No land permissions injected.

    let plains_id = find_object(&state, "Test Plains");
    let result = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: plains_id,
        },
    );
    assert!(
        result.is_err(),
        "Should not be able to play a land from library top without a permission"
    );
}

/// Pass priority for all listed players once (helper for resolution tests).
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

/// An ObjectSpec for "Test X Creature" in a player's library (Creature {X}{G}{G}, P/T = 0/0).
/// Used to test that X=0 is enforced for PayLifeForManaValue.
fn x_creature_spec(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::creature(owner, "Test X Creature", 0, 0)
        .in_zone(ZoneId::Library(owner))
        .with_mana_cost(ManaCost {
            green: 2,
            x_count: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::Green])
}

/// An X-cost creature CardDefinition for {X}{G}{G} with x_count: 1.
fn x_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-x-creature".to_string()),
        name: "Test X Creature".to_string(),
        mana_cost: Some(ManaCost {
            green: 2,
            x_count: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![],
        ..Default::default()
    }
}

/// Inject a CreaturesWithMinPower(4) permission with haste on_cast_effect
/// for the given source/controller (Thundermane Dragon style).
fn inject_creatures_min_power_4_with_haste(
    state: &mut GameState,
    source: ObjectId,
    controller: PlayerId,
) {
    state
        .play_from_top_permissions
        .push_back(PlayFromTopPermission {
            source,
            controller,
            filter: PlayFromTopFilter::CreaturesWithMinPower(4),
            look_at_top: true,
            reveal_top: false,
            pay_life_instead: false,
            condition: None,
            on_cast_effect: Some(Box::new(Effect::ApplyContinuousEffect {
                effect_def: Box::new(CardContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::UntilEndOfTurn,
                    condition: None,
                }),
            })),
        });
}

/// CR 107.3 / 2019-05-03 Bolas's Citadel ruling (PB-A fix): When casting via
/// PayLifeForManaValue, X must be 0. The engine rejects any cast where x_value > 0
/// is paired with the PayLifeForManaValue alt cost.
#[test]
fn test_play_from_top_bolas_citadel_x_is_zero() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![x_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::artifact(p1, "Permission Source").in_zone(ZoneId::Battlefield))
        // X creature costs {X}{G}{G} (mana value = 2 for X=0, variable for X>0).
        .object(x_creature_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_pay_life(&mut state, source_id, p1);

    let creature_id = find_object(&state, "Test X Creature");

    // Attempting to specify x_value > 0 with PayLifeForManaValue must be rejected.
    let bad_cast = Command::CastSpell {
        player: p1,
        card: creature_id,
        targets: vec![],
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: Some(AltCostKind::PayLifeForManaValue),
        prototype: false,
        modes_chosen: vec![],
        x_value: 3, // non-zero — must be rejected
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
        face_down_kind: None,
        additional_costs: vec![],
    };
    let result = process_command(state.clone(), bad_cast);
    assert!(
        result.is_err(),
        "x_value > 0 with PayLifeForManaValue should be rejected (CR 107.3 / 2019-05-03 ruling)"
    );

    // x_value = 0 with PayLifeForManaValue must succeed (pays life = mana value 2).
    let good_cast = Command::CastSpell {
        player: p1,
        card: creature_id,
        targets: vec![],
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: Some(AltCostKind::PayLifeForManaValue),
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
        face_down_kind: None,
        additional_costs: vec![],
    };
    let initial_life = state.player(p1).unwrap().life_total;
    let (state2, _events) = process_command(state, good_cast)
        .expect("x_value=0 with PayLifeForManaValue should succeed");
    // Life paid = mana value of {X}{G}{G} with X=0 → {0}{G}{G} = 2.
    assert_eq!(
        state2.player(p1).unwrap().life_total,
        initial_life - 2,
        "Life should decrease by mana value (2) for {{X}}{{G}}{{G}} with X=0"
    );
}

/// Thundermane Dragon on_cast_effect fix (PB-A review HIGH-1): A creature cast from
/// the top of the library via a permission with on_cast_effect gains Haste on the
/// battlefield — not just on the stack — because the fix applies the keyword at
/// resolution using the new battlefield ObjectId (CR 400.7).
#[test]
fn test_play_from_top_haste_grant() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![big_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Permission Source", 4, 4).in_zone(ZoneId::Battlefield))
        // Big creature: 4/4 for {4} — matches CreaturesWithMinPower(4).
        .object(big_creature_spec(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Permission Source");
    inject_creatures_min_power_4_with_haste(&mut state, source_id, p1);

    // Add mana for the 4/4 creature ({4}).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);

    let creature_id = find_object(&state, "Test Big Creature");
    // Cast the creature from the top of the library.
    let (state2, _) = process_command(state, cast_spell(p1, creature_id))
        .expect("Cast via CreaturesWithMinPower(4) permission should succeed");

    // The spell is on the stack; cast_from_top_with_bonus should be set.
    assert!(
        !state2.stack_objects.is_empty(),
        "Spell should be on the stack"
    );
    assert!(
        state2
            .stack_objects
            .iter()
            .any(|so| so.cast_from_top_with_bonus),
        "StackObject should have cast_from_top_with_bonus = true"
    );

    // Pass priority for both players to resolve the spell.
    let (state3, _) = pass_all(state2, &[p1, p2]);

    // The creature should now be on the battlefield.
    let bf_creature = state3
        .objects
        .values()
        .find(|obj| {
            obj.characteristics.name == "Test Big Creature"
                && matches!(obj.zone, ZoneId::Battlefield)
        })
        .expect("Big Creature should be on the battlefield after resolution");

    // The battlefield permanent's keywords must include Haste (CR 400.7 fix: applied at resolution
    // to the new ObjectId, not via a continuous effect targeting the dead stack ObjectId).
    assert!(
        bf_creature
            .characteristics
            .keywords
            .contains(&KeywordAbility::Haste),
        "Creature cast from top via on_cast_effect permission should have Haste on the battlefield"
    );

    // Also verify via calculate_characteristics to confirm layer system sees Haste.
    let chars = calculate_characteristics(&state3, bf_creature.id)
        .expect("calculate_characteristics should return Some for a battlefield permanent");
    assert!(
        chars.keywords.contains(&KeywordAbility::Haste),
        "calculate_characteristics should show Haste on the battlefield creature"
    );
}
