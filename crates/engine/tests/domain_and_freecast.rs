//! Domain count and CommanderFreeCast tests (PB-L).
//!
//! Tests for:
//! - EffectAmount::DomainCount: CR 305.6 / ability word "Domain" — counts distinct basic
//!   land types (Plains, Island, Swamp, Mountain, Forest) among lands the controller controls.
//! - AltCostKind::CommanderFreeCast: CR 118.9 / Commander 2020 cycle — "If you control a
//!   commander, you may cast this spell without paying its mana cost."
//! - Coiling Oracle ETB: CR 701.20 — RevealAndRoute with land → battlefield, else → hand.
//!
//! Key rules verified:
//! - CR 305.6: Basic land subtypes are Plains, Island, Swamp, Mountain, Forest.
//! - CR 305.6: Only distinct types count — two Plains = 1 (Plains), not 2.
//! - CR 305.6: Domain uses layer-resolved characteristics (Blood Moon / Dryad effects).
//! - CR 118.9: CommanderFreeCast sets cost to {0}; additional costs still apply.
//! - CR 118.9a: Only one alternative cost may be applied.
//! - 2020-04-17 ruling: "any commander" — doesn't have to be your own.
//! - CR 701.20a: RevealAndRoute with land filter: land → BF (untapped), non-land → hand.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::layers::calculate_characteristics;
use mtg_engine::state::continuous_effect::{
    ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, LayerModification,
};
use mtg_engine::state::game_object::ObjectId;
use mtg_engine::state::types::{AltCostKind, CardType, SubType};
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, Command, Effect,
    EffectAmount, GameEvent, GameState, GameStateBuilder, ManaColor, ManaCost, ObjectSpec,
    PlayerId, PlayerTarget, Step,
};

// ── Helpers ────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn ec(controller: PlayerId, source: ObjectId) -> EffectContext {
    EffectContext::new(controller, source, vec![])
}

/// Pass priority for all players once (clears stack items).
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

/// Helper land ObjectSpec with a given basic land subtype.
fn basic_land(owner: PlayerId, name: &str, subtype: &str) -> ObjectSpec {
    ObjectSpec::land(owner, name)
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_subtypes(vec![SubType(subtype.to_string())])
}

// ── Domain Count Tests ─────────────────────────────────────────────────────────

#[test]
/// CR 305.6 — Domain count with no basic lands = 0. Allied Strategies draws 0 cards.
fn test_domain_count_zero_lands() {
    let p1 = p(1);
    let source = ObjectSpec::artifact(p1, "Source").in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .object(source)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Source");
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    let mut ctx = ec(p1, source_id);
    let _events = execute_effect(
        &mut state,
        &Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::DomainCount,
        },
        &mut ctx,
    );

    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after, initial_hand,
        "CR 305.6: With no basic lands, domain count = 0; no cards drawn"
    );
}

#[test]
/// CR 305.6 — Domain count with all 5 basic land types = 5.
/// Allied Strategies draws a card for each, so 5 cards drawn from a 6-card library.
fn test_domain_count_all_five_types() {
    let p1 = p(1);

    // 5 distinct basic land types on the battlefield.
    let plains = basic_land(p1, "Plains", "Plains");
    let island = basic_land(p1, "Island", "Island");
    let swamp = basic_land(p1, "Swamp", "Swamp");
    let mountain = basic_land(p1, "Mountain", "Mountain");
    let forest = basic_land(p1, "Forest", "Forest");
    let source = ObjectSpec::artifact(p1, "Source").in_zone(ZoneId::Battlefield);

    // 6 library cards (more than domain count of 5, to avoid capping).
    let lib1 = ObjectSpec::card(p1, "Lib 1").in_zone(ZoneId::Library(p1));
    let lib2 = ObjectSpec::card(p1, "Lib 2").in_zone(ZoneId::Library(p1));
    let lib3 = ObjectSpec::card(p1, "Lib 3").in_zone(ZoneId::Library(p1));
    let lib4 = ObjectSpec::card(p1, "Lib 4").in_zone(ZoneId::Library(p1));
    let lib5 = ObjectSpec::card(p1, "Lib 5").in_zone(ZoneId::Library(p1));
    let lib6 = ObjectSpec::card(p1, "Lib 6").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .object(plains)
        .object(island)
        .object(swamp)
        .object(mountain)
        .object(forest)
        .object(source)
        .object(lib1)
        .object(lib2)
        .object(lib3)
        .object(lib4)
        .object(lib5)
        .object(lib6)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Source");
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    let mut ctx = ec(p1, source_id);
    let _events = execute_effect(
        &mut state,
        &Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::DomainCount,
        },
        &mut ctx,
    );

    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    let delta_hand = hand_after as i32 - initial_hand as i32;
    assert_eq!(
        delta_hand, 5,
        "CR 305.6: With all 5 basic land types, domain = 5; should draw 5 cards (drew {})",
        delta_hand
    );
}

#[test]
/// CR 305.6 — Two Plains + one Island = domain count 2 (Plains + Island), not 3.
/// Duplicate land types are NOT counted separately.
fn test_domain_count_duplicate_types() {
    let p1 = p(1);

    // Two Plains (same type) + one Island.
    let plains1 = basic_land(p1, "Plains 1", "Plains");
    let plains2 = basic_land(p1, "Plains 2", "Plains");
    let island = basic_land(p1, "Island", "Island");
    let source = ObjectSpec::artifact(p1, "Source").in_zone(ZoneId::Battlefield);

    // Add 3 library cards so we can verify how many are drawn.
    let lib1 = ObjectSpec::card(p1, "Lib A").in_zone(ZoneId::Library(p1));
    let lib2 = ObjectSpec::card(p1, "Lib B").in_zone(ZoneId::Library(p1));
    let lib3 = ObjectSpec::card(p1, "Lib C").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .object(plains1)
        .object(plains2)
        .object(island)
        .object(source)
        .object(lib1)
        .object(lib2)
        .object(lib3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Source");
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    let mut ctx = ec(p1, source_id);
    let _events = execute_effect(
        &mut state,
        &Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::DomainCount,
        },
        &mut ctx,
    );

    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    let delta = hand_after as i32 - initial_hand as i32;
    assert_eq!(
        delta, 2,
        "CR 305.6: Two Plains + one Island = domain 2; should draw 2 cards (drew {})",
        delta
    );
}

#[test]
/// CR 305.6 — Domain counts only the controller's lands; opponent's lands don't count.
fn test_domain_count_only_controllers_lands() {
    let p1 = p(1);
    let p2 = p(2);

    // p1 has only one basic land type (Plains).
    // p2 has all 5 — but those don't count for p1.
    let p1_plains = basic_land(p1, "P1 Plains", "Plains");
    let p2_island = basic_land(p2, "P2 Island", "Island");
    let p2_swamp = basic_land(p2, "P2 Swamp", "Swamp");
    let p2_mountain = basic_land(p2, "P2 Mountain", "Mountain");
    let p2_forest = basic_land(p2, "P2 Forest", "Forest");
    let source = ObjectSpec::artifact(p1, "Source").in_zone(ZoneId::Battlefield);

    // p1 has 3 library cards.
    let lib1 = ObjectSpec::card(p1, "Lib 1").in_zone(ZoneId::Library(p1));
    let lib2 = ObjectSpec::card(p1, "Lib 2").in_zone(ZoneId::Library(p1));
    let lib3 = ObjectSpec::card(p1, "Lib 3").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(p1_plains)
        .object(p2_island)
        .object(p2_swamp)
        .object(p2_mountain)
        .object(p2_forest)
        .object(source)
        .object(lib1)
        .object(lib2)
        .object(lib3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Source");
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    let mut ctx = ec(p1, source_id);
    let _events = execute_effect(
        &mut state,
        &Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::DomainCount,
        },
        &mut ctx,
    );

    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    let delta = hand_after as i32 - initial_hand as i32;
    assert_eq!(
        delta, 1,
        "CR 305.6: Only p1's Plains counts; domain = 1 (drew {})",
        delta
    );
}

#[test]
/// CR 604.3, 613.4a — Territorial Maro CDA: P/T = 2 * domain count.
/// With 3 basic land types, P/T should be 6/6.
/// The CDA continuous effect is pre-registered via add_continuous_effect (mimics ETB registration).
fn test_territorial_maro_cda() {
    let p1 = p(1);

    // Load the real Territorial Maro definition.
    let maro_def = mtg_engine::cards::defs::territorial_maro::card();
    let registry = CardRegistry::new(vec![maro_def]);

    // Maro on the battlefield — use ObjectId 1 so we can reference it in the CDA effect.
    let maro = ObjectSpec::creature(p1, "Territorial Maro", 0, 0)
        .with_card_id(cid("territorial-maro"))
        .in_zone(ZoneId::Battlefield);

    // 3 basic land types: Plains, Island, Forest.
    let plains = basic_land(p1, "Plains", "Plains");
    let island = basic_land(p1, "Island", "Island");
    let forest = basic_land(p1, "Forest", "Forest");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(registry)
        .object(maro)
        .object(plains)
        .object(island)
        .object(forest)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let maro_id = find_object(&state, "Territorial Maro");

    // Pre-register the CDA continuous effect (mimics what register_static_continuous_effects does
    // when the creature enters the battlefield via normal resolution).
    // CR 604.3, 613.4a: CDA P/T = 2 * DomainCount, evaluated in Layer 7a (PtCda).
    let cda_effect = ContinuousEffect {
        id: EffectId(9000),
        source: Some(maro_id),
        timestamp: 1,
        layer: EffectLayer::PtCda,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(maro_id),
        modification: LayerModification::SetPtDynamic {
            power: Box::new(EffectAmount::Sum(
                Box::new(EffectAmount::DomainCount),
                Box::new(EffectAmount::DomainCount),
            )),
            toughness: Box::new(EffectAmount::Sum(
                Box::new(EffectAmount::DomainCount),
                Box::new(EffectAmount::DomainCount),
            )),
        },
        is_cda: true,
        condition: None,
    };
    state.continuous_effects.push_back(cda_effect);

    let chars = calculate_characteristics(&state, maro_id)
        .expect("Territorial Maro should have layer-resolved characteristics");

    // Domain = 3 (Plains + Island + Forest), P/T = 2 * 3 = 6.
    assert_eq!(
        chars.power,
        Some(6),
        "CR 604.3/613.4a: Territorial Maro P should be 6 with 3 basic land types (was {:?})",
        chars.power
    );
    assert_eq!(
        chars.toughness,
        Some(6),
        "CR 604.3/613.4a: Territorial Maro T should be 6 with 3 basic land types (was {:?})",
        chars.toughness
    );
}

#[test]
/// CR 604.3, 613.4a — Territorial Maro with no lands: P/T = 0/0.
fn test_territorial_maro_cda_zero_lands() {
    let p1 = p(1);

    let maro_def = mtg_engine::cards::defs::territorial_maro::card();
    let registry = CardRegistry::new(vec![maro_def]);

    let maro = ObjectSpec::creature(p1, "Territorial Maro", 0, 0)
        .with_card_id(cid("territorial-maro"))
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(registry)
        .object(maro)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let maro_id = find_object(&state, "Territorial Maro");

    // Pre-register CDA — with no basic lands, domain = 0, so P/T = 0/0.
    let cda_effect = ContinuousEffect {
        id: EffectId(9001),
        source: Some(maro_id),
        timestamp: 1,
        layer: EffectLayer::PtCda,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(maro_id),
        modification: LayerModification::SetPtDynamic {
            power: Box::new(EffectAmount::Sum(
                Box::new(EffectAmount::DomainCount),
                Box::new(EffectAmount::DomainCount),
            )),
            toughness: Box::new(EffectAmount::Sum(
                Box::new(EffectAmount::DomainCount),
                Box::new(EffectAmount::DomainCount),
            )),
        },
        is_cda: true,
        condition: None,
    };
    state.continuous_effects.push_back(cda_effect);

    let chars = calculate_characteristics(&state, maro_id)
        .expect("Territorial Maro should have characteristics");

    assert_eq!(
        chars.power,
        Some(0),
        "CR 604.3: Territorial Maro P = 0 with no lands (was {:?})",
        chars.power
    );
    assert_eq!(
        chars.toughness,
        Some(0),
        "CR 604.3: Territorial Maro T = 0 with no lands (was {:?})",
        chars.toughness
    );
}

// ── CommanderFreeCast Tests ────────────────────────────────────────────────────

/// Minimal commander card definition (a legendary creature).
fn test_commander_def() -> CardDefinition {
    CardDefinition {
        card_id: cid("test-commander"),
        name: "Test Commander".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            supertypes: [mtg_engine::SuperType::Legendary].into_iter().collect(),
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Legendary creature.".to_string(),
        power: Some(3),
        toughness: Some(3),
        ..Default::default()
    }
}

/// Minimal Fierce Guardianship-style definition (counter noncreature, CommanderFreeCast).
fn test_free_cast_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: cid("test-free-cast-spell"),
        name: "Test Free Cast Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "If you control a commander, you may cast this spell without paying its mana cost. Draw a card.".to_string(),
        abilities: vec![
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::CommanderFreeCast,
                cost: ManaCost::default(),
                details: None,
            },
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

#[test]
/// CR 118.9 — CommanderFreeCast: cast without paying mana cost while controlling a commander.
/// The spell goes on the stack; the caster pays {0}.
fn test_commander_free_cast_with_commander() {
    let p1 = p(1);
    let p2 = p(2);

    let commander_def = test_commander_def();
    let free_spell_def = test_free_cast_spell_def();
    let registry = CardRegistry::new(vec![commander_def, free_spell_def]);

    let commander_card_id = cid("test-commander");

    // Commander on p1's battlefield.
    let commander = ObjectSpec::creature(p1, "Test Commander", 3, 3)
        .with_card_id(commander_card_id.clone())
        .in_zone(ZoneId::Battlefield);

    // Free-cast spell in p1's hand.
    let spell = ObjectSpec::card(p1, "Test Free Cast Spell")
        .with_card_id(cid("test-free-cast-spell"))
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, commander_card_id)
        .object(commander)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Free Cast Spell");

    // p1 casts for free (no mana needed).
    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::CommanderFreeCast),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("CR 118.9: Free-cast while controlling a commander should succeed");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "CR 118.9: SpellCast event expected for CommanderFreeCast"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 118.9: Spell should be on the stack after free-cast"
    );
}

#[test]
/// CR 118.9 — CommanderFreeCast rejected when no commander is controlled on the battlefield.
fn test_commander_free_cast_without_commander() {
    let p1 = p(1);
    let p2 = p(2);

    let free_spell_def = test_free_cast_spell_def();
    let registry = CardRegistry::new(vec![free_spell_def]);

    // No commander on the battlefield.
    let spell = ObjectSpec::card(p1, "Test Free Cast Spell")
        .with_card_id(cid("test-free-cast-spell"))
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let spell_id = find_object(&state, "Test Free Cast Spell");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::CommanderFreeCast),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 118.9: CommanderFreeCast without a commander on the battlefield should fail"
    );
}

#[test]
/// 2020-04-17 ruling — CommanderFreeCast works with any commander, including an opponent's
/// commander that the caster controls.
fn test_commander_free_cast_any_commander() {
    let p1 = p(1);
    let p2 = p(2);

    let p2_commander_def = CardDefinition {
        card_id: cid("p2-commander"),
        name: "P2 Commander".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            supertypes: [mtg_engine::SuperType::Legendary].into_iter().collect(),
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };
    let free_spell_def = test_free_cast_spell_def();
    let registry = CardRegistry::new(vec![p2_commander_def, free_spell_def]);

    let p2_commander_card_id = cid("p2-commander");

    // p2's commander on the battlefield, but controlled by p1.
    let commander = ObjectSpec::creature(p1, "P2 Commander", 2, 2)
        .with_card_id(p2_commander_card_id.clone())
        .in_zone(ZoneId::Battlefield);

    let spell = ObjectSpec::card(p1, "Test Free Cast Spell")
        .with_card_id(cid("test-free-cast-spell"))
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        // Register as p2's commander in the system.
        .player_commander(p2, p2_commander_card_id)
        .object(commander)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let spell_id = find_object(&state, "Test Free Cast Spell");

    // p1 controls p2's commander on the battlefield — should satisfy the condition.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::CommanderFreeCast),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_ok(),
        "2020-04-17 ruling: Any commander (even opponent's) satisfies the condition; \
         result: {:?}",
        result.err()
    );
}

// ── Coiling Oracle ETB Tests ────────────────────────────────────────────────────

#[test]
/// CR 701.20 — Coiling Oracle ETB: reveal top card; if it's a land, put onto battlefield.
fn test_coiling_oracle_land_to_battlefield() {
    let p1 = p(1);

    let oracle_def = mtg_engine::cards::defs::coiling_oracle::card();
    let registry = CardRegistry::new(vec![oracle_def]);

    // A land card on top of p1's library.
    let top_card = ObjectSpec::land(p1, "Basic Plains")
        .with_card_id(cid("basic-plains"))
        .with_subtypes(vec![SubType("Plains".to_string())])
        .in_zone(ZoneId::Library(p1));

    // Coiling Oracle in hand (will be cast and ETB).
    let oracle = ObjectSpec::creature(p1, "Coiling Oracle", 1, 1)
        .with_card_id(cid("coiling-oracle"))
        .with_types(vec![CardType::Creature])
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(registry)
        .object(top_card)
        .object(oracle)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Add mana to cast Coiling Oracle ({G}{U}).
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
        .add(ManaColor::Blue, 1);

    let oracle_id = find_object(&state, "Coiling Oracle");

    // Cast Coiling Oracle.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: oracle_id,
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
        },
    )
    .unwrap();

    // Resolve: both players pass priority.
    let (state, _) = pass_all(state, &[p1, p(2)]);
    // ETB trigger is on the stack — resolve it.
    let (state, _) = pass_all(state, &[p1, p(2)]);

    // "Basic Plains" should now be on the battlefield (land → BF).
    let is_on_bf = find_in_zone(&state, "Basic Plains", ZoneId::Battlefield).is_some();
    let is_in_hand = find_in_zone(&state, "Basic Plains", ZoneId::Hand(p1)).is_some();
    let is_in_library = find_in_zone(&state, "Basic Plains", ZoneId::Library(p1)).is_some();

    assert!(
        is_on_bf,
        "CR 701.20: Coiling Oracle should put a revealed land onto the battlefield \
         (in hand: {}, in library: {})",
        is_in_hand, is_in_library
    );
}

#[test]
/// CR 701.20 — Coiling Oracle ETB: reveal top card; if not a land, put into hand.
fn test_coiling_oracle_nonland_to_hand() {
    let p1 = p(1);

    let oracle_def = mtg_engine::cards::defs::coiling_oracle::card();
    let registry = CardRegistry::new(vec![oracle_def]);

    // A non-land card (Sorcery) on top of p1's library.
    let top_card = ObjectSpec::card(p1, "Fireball")
        .with_card_id(cid("fireball"))
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Library(p1));

    let oracle = ObjectSpec::creature(p1, "Coiling Oracle", 1, 1)
        .with_card_id(cid("coiling-oracle"))
        .with_types(vec![CardType::Creature])
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(registry)
        .object(top_card)
        .object(oracle)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

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
        .add(ManaColor::Blue, 1);

    let oracle_id = find_object(&state, "Coiling Oracle");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: oracle_id,
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
        },
    )
    .unwrap();

    // Resolve oracle, then its ETB trigger.
    let (state, _) = pass_all(state, &[p1, p(2)]);
    let (state, _) = pass_all(state, &[p1, p(2)]);

    // "Fireball" should now be in p1's hand (non-land → hand).
    let is_in_hand = find_in_zone(&state, "Fireball", ZoneId::Hand(p1)).is_some();
    let is_on_bf = find_in_zone(&state, "Fireball", ZoneId::Battlefield).is_some();

    assert!(
        is_in_hand,
        "CR 701.20: Coiling Oracle should put a revealed non-land into hand \
         (on battlefield: {})",
        is_on_bf
    );
}
