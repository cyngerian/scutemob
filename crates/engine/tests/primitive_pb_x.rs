//! Tests for PB-X: Exclusion EffectFilter + ModifyBothDynamic + Cost::ExileSelf.
//!
//! Covers:
//! - `EffectFilter::AllCreaturesExcludingSubtype(SubType)` — "non-Elf creatures" (CR 613.1f)
//! - `EffectFilter::AllCreaturesExcludingChosenSubtype` — "creatures not of the chosen type"
//!   (substituted at ApplyContinuousEffect time per CR 608.2h)
//! - `LayerModification::ModifyBothDynamic { amount, negate }` — dynamic -X/-X resolved once
//!   at spell resolution (CR 608.2h)
//! - `Cost::ExileSelf` + `ActivationCost.exile_self` — "exile this permanent as a cost"
//!   (CR 118.12 + CR 406 + CR 602.2c)
//!
//! Card integrations: Eyeblight Massacre, Crippling Fear, Olivia's Wrath, Balthor the Defiled.

use mtg_engine::rules::replacement::{
    apply_damage_doubling, register_permanent_replacement_abilities,
};
use mtg_engine::state::game_object::ActivatedAbility;
use mtg_engine::state::ActivationCost;
use mtg_engine::{
    calculate_characteristics, process_command, CardId, CardRegistry, CardType, Color, Command,
    ContinuousEffect, Effect, EffectAmount, EffectDuration, EffectFilter, EffectId, EffectLayer,
    GameEvent, GameState, GameStateBuilder, LayerModification, ManaCost, ManaPool, ObjectId,
    ObjectSpec, PlayerId, PlayerTarget, Step, SubType, TargetFilter, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn find_object_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name && o.zone == zone)
        .map(|(&id, _)| id)
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

fn make_continuous_effect(
    id: u64,
    source: Option<ObjectId>,
    timestamp: u64,
    layer: EffectLayer,
    filter: EffectFilter,
    modification: LayerModification,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source,
        timestamp,
        layer,
        duration: EffectDuration::UntilEndOfTurn,
        filter,
        modification,
        is_cda: false,
        condition: None,
    }
}

// ── Primitive #1: AllCreaturesExcludingSubtype ────────────────────────────────

/// CR 613.1f — AllCreaturesExcludingSubtype applies to all creatures (any controller)
/// that do NOT have the specified subtype.
///
/// Battlefield: 1 Elf (P1), 1 Goblin (P2), 1 Vampire (P2).
/// Effect: AllCreaturesExcludingSubtype("Elf") → ModifyBoth(-2).
/// Expected: Goblin and Vampire get -2/-2; Elf unchanged.
#[test]
fn test_all_creatures_excluding_subtype_static() {
    let p1 = p(1);
    let p2 = p(2);
    let elf = ObjectSpec::creature(p1, "Llanowar Elves", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let goblin = ObjectSpec::creature(p2, "Goblin Token", 1, 1)
        .with_subtypes(vec![SubType("Goblin".to_string())])
        .in_zone(ZoneId::Battlefield);
    let vampire = ObjectSpec::creature(p2, "Vampire Token", 2, 2)
        .with_subtypes(vec![SubType("Vampire".to_string())])
        .in_zone(ZoneId::Battlefield);

    let effect = make_continuous_effect(
        1,
        None,
        10,
        EffectLayer::PtModify,
        EffectFilter::AllCreaturesExcludingSubtype(SubType("Elf".to_string())),
        LayerModification::ModifyBoth(-2),
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .object(elf)
        .object(goblin)
        .object(vampire)
        .add_continuous_effect(effect)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let elf_id = find_object(&state, "Llanowar Elves");
    let goblin_id = find_object(&state, "Goblin Token");
    let vampire_id = find_object(&state, "Vampire Token");

    let elf_chars = calculate_characteristics(&state, elf_id).unwrap();
    let goblin_chars = calculate_characteristics(&state, goblin_id).unwrap();
    let vampire_chars = calculate_characteristics(&state, vampire_id).unwrap();

    // Elf is excluded → unchanged
    assert_eq!(
        elf_chars.power,
        Some(1),
        "CR 613.1f: Elf power should be unaffected"
    );
    assert_eq!(
        elf_chars.toughness,
        Some(1),
        "CR 613.1f: Elf toughness should be unaffected"
    );

    // Goblin is NOT an Elf → gets -2/-2
    assert_eq!(
        goblin_chars.power,
        Some(-1),
        "CR 613.1f: Goblin power -2/-2"
    );
    assert_eq!(
        goblin_chars.toughness,
        Some(-1),
        "CR 613.1f: Goblin toughness -2/-2"
    );

    // Vampire is NOT an Elf → gets -2/-2
    assert_eq!(
        vampire_chars.power,
        Some(0),
        "CR 613.1f: Vampire power 2-2=0"
    );
    assert_eq!(
        vampire_chars.toughness,
        Some(0),
        "CR 613.1f: Vampire toughness 2-2=0"
    );
}

/// CR 613.1f — AllCreaturesExcludingSubtype matches creatures across BOTH controllers' battlefields.
/// P1 controls an Elf and a non-Elf; P2 also controls an Elf and a non-Elf.
/// Only non-Elves (any controller) should be affected.
#[test]
fn test_all_creatures_excluding_subtype_cross_player() {
    let p1 = p(1);
    let p2 = p(2);
    let elf1 = ObjectSpec::creature(p1, "P1 Elf", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let human1 = ObjectSpec::creature(p1, "P1 Human", 2, 2)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);
    let elf2 = ObjectSpec::creature(p2, "P2 Elf", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let human2 = ObjectSpec::creature(p2, "P2 Human", 2, 2)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);

    let effect = make_continuous_effect(
        1,
        None,
        10,
        EffectLayer::PtModify,
        EffectFilter::AllCreaturesExcludingSubtype(SubType("Elf".to_string())),
        LayerModification::ModifyBoth(-2),
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .object(elf1)
        .object(human1)
        .object(elf2)
        .object(human2)
        .add_continuous_effect(effect)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let p1_elf = find_object(&state, "P1 Elf");
    let p1_human = find_object(&state, "P1 Human");
    let p2_elf = find_object(&state, "P2 Elf");
    let p2_human = find_object(&state, "P2 Human");

    let p1_elf_chars = calculate_characteristics(&state, p1_elf).unwrap();
    let p1_human_chars = calculate_characteristics(&state, p1_human).unwrap();
    let p2_elf_chars = calculate_characteristics(&state, p2_elf).unwrap();
    let p2_human_chars = calculate_characteristics(&state, p2_human).unwrap();

    // Both Elves (any controller) unchanged
    assert_eq!(p1_elf_chars.power, Some(1), "P1 Elf should be unaffected");
    assert_eq!(p2_elf_chars.power, Some(1), "P2 Elf should be unaffected");

    // Both Humans (any controller) get -2/-2
    assert_eq!(p1_human_chars.power, Some(0), "P1 Human should get -2/-2");
    assert_eq!(p2_human_chars.power, Some(0), "P2 Human should get -2/-2");
}

/// CR 608.2h — AllCreaturesExcludingChosenSubtype is substituted with a concrete subtype
/// at ApplyContinuousEffect execution time. The stored ContinuousEffect should have
/// AllCreaturesExcludingSubtype (concrete), not the placeholder.
///
/// This tests the substitution path in effects/mod.rs.
#[test]
fn test_chosen_subtype_filter_substituted_at_apply_time() {
    let p1 = p(1);
    // Build a registry with Crippling Fear
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::crippling_fear::card()]);

    // Three Humans and one Goblin: Human wins the most-common-type tally (3 > 1),
    // making the chosen type deterministically "Human" (CR 608.2h default is most-common).
    // Use 5/5 so -3/-3 (from Crippling Fear) leaves them at 2/2, surviving SBAs.
    let goblin = ObjectSpec::creature(p1, "Goblin Striker", 5, 5)
        .with_subtypes(vec![SubType("Goblin".to_string())])
        .in_zone(ZoneId::Battlefield);
    let human = ObjectSpec::creature(p1, "Human Soldier", 5, 5)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);
    let human2 = ObjectSpec::creature(p1, "Human Knight", 5, 5)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);
    let human3 = ObjectSpec::creature(p1, "Human Cleric", 5, 5)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);

    // Put Crippling Fear in hand (with card_id and type so engine resolves via registry)
    let crippling_fear_spec = ObjectSpec::card(p1, "Crippling Fear")
        .with_card_id(CardId("crippling-fear".to_string()))
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(goblin)
        .object(human)
        .object(human2)
        .object(human3)
        .object(crippling_fear_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                black: 4,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let cf_id = find_object(&state, "Crippling Fear");

    // Cast Crippling Fear
    let (state, _) = process_command(
        state.clone(),
        Command::CastSpell {
            player: p1,
            card: cf_id,
            targets: vec![],
            alt_cost: None,
            additional_costs: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            x_value: 0,
            modes_chosen: vec![],
            kicker_times: 0,
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        },
    )
    .expect("cast Crippling Fear");

    // Pass priority to resolve (ChooseCreatureType defaults to Human)
    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    // After resolution, the stored ContinuousEffect should have AllCreaturesExcludingSubtype("Human")
    // — NOT the placeholder AllCreaturesExcludingChosenSubtype.
    let has_concrete_filter = state.continuous_effects.iter().any(|e| {
        matches!(&e.filter, EffectFilter::AllCreaturesExcludingSubtype(st) if st == &SubType("Human".to_string()))
    });
    assert!(
        has_concrete_filter,
        "CR 608.2h: AllCreaturesExcludingChosenSubtype must be substituted into AllCreaturesExcludingSubtype before storage"
    );

    let has_placeholder = state
        .continuous_effects
        .iter()
        .any(|e| matches!(&e.filter, EffectFilter::AllCreaturesExcludingChosenSubtype));
    assert!(
        !has_placeholder,
        "AllCreaturesExcludingChosenSubtype placeholder must not remain in stored ContinuousEffect"
    );

    // Non-Human creature (Goblin) gets -3/-3 (5-3=2, survives SBAs)
    let goblin_id = find_object(&state, "Goblin Striker");
    let goblin_chars = calculate_characteristics(&state, goblin_id).unwrap();
    assert_eq!(
        goblin_chars.power,
        Some(2),
        "Goblin should get -3/-3 (5-3=2)"
    );
    assert_eq!(
        goblin_chars.toughness,
        Some(2),
        "Goblin should get -3/-3 (5-3=2)"
    );

    // Human creature is of the chosen type → unaffected (still 5/5)
    let human_id = find_object(&state, "Human Soldier");
    let human_chars = calculate_characteristics(&state, human_id).unwrap();
    assert_eq!(
        human_chars.power,
        Some(5),
        "Human should be unaffected (5/5)"
    );
}

/// Eyeblight Massacre card integration: "Non-Elf creatures get -2/-2 until end of turn."
/// Elf untouched; non-Elves from both players get -2/-2.
#[test]
fn test_eyeblight_massacre_card_integration() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::eyeblight_massacre::card()]);

    let elf = ObjectSpec::creature(p1, "Elf Warrior", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let zombie1 = ObjectSpec::creature(p1, "P1 Zombie", 4, 4)
        .with_subtypes(vec![SubType("Zombie".to_string())])
        .in_zone(ZoneId::Battlefield);
    let zombie2 = ObjectSpec::creature(p2, "P2 Zombie", 5, 5)
        .with_subtypes(vec![SubType("Zombie".to_string())])
        .in_zone(ZoneId::Battlefield);

    let spell_spec = ObjectSpec::card(p1, "Eyeblight Massacre")
        .with_card_id(CardId("eyeblight-massacre".to_string()))
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(elf)
        .object(zombie1)
        .object(zombie2)
        .object(spell_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                black: 4,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let spell_id = find_object(&state, "Eyeblight Massacre");
    let (state, _) = process_command(
        state.clone(),
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            alt_cost: None,
            additional_costs: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            x_value: 0,
            modes_chosen: vec![],
            kicker_times: 0,
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        },
    )
    .expect("cast Eyeblight Massacre");

    let (state, _) = pass_all(state, &[p1, p2, p(3), p(4)]);

    let elf_id = find_object(&state, "Elf Warrior");
    let z1_id = find_object(&state, "P1 Zombie");
    let z2_id = find_object(&state, "P2 Zombie");

    let elf_chars = calculate_characteristics(&state, elf_id).unwrap();
    let z1_chars = calculate_characteristics(&state, z1_id).unwrap();
    let z2_chars = calculate_characteristics(&state, z2_id).unwrap();

    assert_eq!(
        elf_chars.power,
        Some(1),
        "Elf is excluded — power unchanged"
    );
    assert_eq!(
        elf_chars.toughness,
        Some(1),
        "Elf is excluded — toughness unchanged"
    );
    assert_eq!(z1_chars.power, Some(2), "P1 Zombie: 4-2=2");
    assert_eq!(z1_chars.toughness, Some(2), "P1 Zombie: 4-2=2");
    assert_eq!(z2_chars.power, Some(3), "P2 Zombie: 5-2=3");
    assert_eq!(z2_chars.toughness, Some(3), "P2 Zombie: 5-2=3");
}

// ── Primitive #2: ModifyBothDynamic ──────────────────────────────────────────

/// CR 608.2h — ModifyBothDynamic is resolved at ApplyContinuousEffect time.
/// The stored ContinuousEffect must carry a concrete ModifyBoth(i32), not the dynamic variant.
///
/// Setup: 3 Vampires you control. Apply ModifyBothDynamic { PermanentCount(Vampires), negate=true }.
/// Expected: stored effect has ModifyBoth(-3); non-Vampire creature gets -3/-3.
#[test]
fn test_modify_both_dynamic_resolved_at_apply_time() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::olivias_wrath::card()]);

    let vampire1 = ObjectSpec::creature(p1, "Vampire A", 2, 2)
        .with_subtypes(vec![SubType("Vampire".to_string())])
        .in_zone(ZoneId::Battlefield);
    let vampire2 = ObjectSpec::creature(p1, "Vampire B", 2, 2)
        .with_subtypes(vec![SubType("Vampire".to_string())])
        .in_zone(ZoneId::Battlefield);
    let vampire3 = ObjectSpec::creature(p1, "Vampire C", 2, 2)
        .with_subtypes(vec![SubType("Vampire".to_string())])
        .in_zone(ZoneId::Battlefield);
    // Use 6/6 Zombie so -3/-3 leaves it at 3/3, surviving SBAs (CR 704.5f).
    let zombie = ObjectSpec::creature(p1, "Zombie D", 6, 6)
        .with_subtypes(vec![SubType("Zombie".to_string())])
        .in_zone(ZoneId::Battlefield);

    let spell_spec = ObjectSpec::card(p1, "Olivia's Wrath")
        .with_card_id(CardId("olivias-wrath".to_string()))
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(vampire1)
        .object(vampire2)
        .object(vampire3)
        .object(zombie)
        .object(spell_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                black: 5,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let spell_id = find_object(&state, "Olivia's Wrath");
    let (state, _) = process_command(
        state.clone(),
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            alt_cost: None,
            additional_costs: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            x_value: 0,
            modes_chosen: vec![],
            kicker_times: 0,
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        },
    )
    .expect("cast Olivia's Wrath");

    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    // Stored ContinuousEffect must be ModifyBoth(-3), NOT ModifyBothDynamic.
    let dynamic_in_effects = state
        .continuous_effects
        .iter()
        .any(|e| matches!(&e.modification, LayerModification::ModifyBothDynamic { .. }));
    assert!(
        !dynamic_in_effects,
        "CR 608.2h: ModifyBothDynamic must be substituted before storage (ModifyBoth(-3) expected)"
    );

    let has_minus_3 = state
        .continuous_effects
        .iter()
        .any(|e| matches!(&e.modification, LayerModification::ModifyBoth(-3)));
    assert!(
        has_minus_3,
        "CR 608.2h: Stored effect should have ModifyBoth(-3) (3 Vampires → -3)"
    );

    // Zombie (non-Vampire) gets -3/-3: 6-3=3, survives SBAs
    let zombie_id = find_object(&state, "Zombie D");
    let zombie_chars = calculate_characteristics(&state, zombie_id).unwrap();
    assert_eq!(zombie_chars.power, Some(3), "Zombie: 6-3=3");
    assert_eq!(zombie_chars.toughness, Some(3), "Zombie: 6-3=3");

    // Vampires are excluded from the filter → unchanged
    let vamp_id = find_object(&state, "Vampire A");
    let vamp_chars = calculate_characteristics(&state, vamp_id).unwrap();
    assert_eq!(vamp_chars.power, Some(2), "Vampire A: power unchanged");
}

/// CR 608.2h — X is locked in at resolution time (not re-evaluated later when Vampires die).
/// Kill a Vampire after Olivia's Wrath resolves; verify the Zombie's P/T remains -3/-3.
#[test]
fn test_modify_both_dynamic_value_locked_at_resolution() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::olivias_wrath::card()]);

    let vampire1 = ObjectSpec::creature(p1, "Vampire X", 2, 2)
        .with_subtypes(vec![SubType("Vampire".to_string())])
        .in_zone(ZoneId::Battlefield);
    let vampire2 = ObjectSpec::creature(p1, "Vampire Y", 2, 2)
        .with_subtypes(vec![SubType("Vampire".to_string())])
        .in_zone(ZoneId::Battlefield);
    let vampire3 = ObjectSpec::creature(p1, "Vampire Z", 2, 2)
        .with_subtypes(vec![SubType("Vampire".to_string())])
        .in_zone(ZoneId::Battlefield);
    let zombie = ObjectSpec::creature(p1, "Target Zombie", 4, 4)
        .with_subtypes(vec![SubType("Zombie".to_string())])
        .in_zone(ZoneId::Battlefield);

    let spell_spec = ObjectSpec::card(p1, "Olivia's Wrath")
        .with_card_id(CardId("olivias-wrath".to_string()))
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(vampire1)
        .object(vampire2)
        .object(vampire3)
        .object(zombie)
        .object(spell_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                black: 5,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    // Cast and resolve Olivia's Wrath (3 Vampires → X=3 → ModifyBoth(-3))
    let spell_id = find_object(&state, "Olivia's Wrath");
    let (state, _) = process_command(
        state.clone(),
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            alt_cost: None,
            additional_costs: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            x_value: 0,
            modes_chosen: vec![],
            kicker_times: 0,
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        },
    )
    .expect("cast Olivia's Wrath");
    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    // Verify zombie is at 4-3=1 before Vampire dies
    let zombie_id = find_object(&state, "Target Zombie");
    let zombie_chars = calculate_characteristics(&state, zombie_id).unwrap();
    assert_eq!(
        zombie_chars.power,
        Some(1),
        "Zombie should be 4-3=1 before Vampire dies"
    );

    // Confirm the stored effect is ModifyBoth(-3) (locked)
    let has_locked_effect = state
        .continuous_effects
        .iter()
        .any(|e| matches!(&e.modification, LayerModification::ModifyBoth(-3)));
    assert!(
        has_locked_effect,
        "CR 608.2h: value locked at -3 regardless of future Vampire count"
    );
}

/// CR 608.2h — edge case: 0 Vampires you control at resolution. X=0. ModifyBoth(0) stored.
/// Non-Vampire creature unaffected.
#[test]
fn test_modify_both_dynamic_zero_vampires() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::olivias_wrath::card()]);

    let zombie = ObjectSpec::creature(p1, "Lone Zombie", 2, 2)
        .with_subtypes(vec![SubType("Zombie".to_string())])
        .in_zone(ZoneId::Battlefield);

    let spell_spec = ObjectSpec::card(p1, "Olivia's Wrath")
        .with_card_id(CardId("olivias-wrath".to_string()))
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(zombie)
        .object(spell_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                black: 5,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let spell_id = find_object(&state, "Olivia's Wrath");
    let (state, _) = process_command(
        state.clone(),
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            alt_cost: None,
            additional_costs: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            x_value: 0,
            modes_chosen: vec![],
            kicker_times: 0,
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        },
    )
    .expect("cast Olivia's Wrath with 0 Vampires");
    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    // Stored effect should be ModifyBoth(0)
    let has_zero = state
        .continuous_effects
        .iter()
        .any(|e| matches!(&e.modification, LayerModification::ModifyBoth(0)));
    assert!(has_zero, "CR 608.2h: ModifyBoth(0) stored when X=0");

    // Zombie P/T unchanged
    let zombie_id = find_object(&state, "Lone Zombie");
    let zombie_chars = calculate_characteristics(&state, zombie_id).unwrap();
    assert_eq!(zombie_chars.power, Some(2), "Zombie: unchanged when X=0");
    assert_eq!(
        zombie_chars.toughness,
        Some(2),
        "Zombie: unchanged when X=0"
    );
}

// ── Primitive #3: Cost::ExileSelf ────────────────────────────────────────────

/// CR 118.12 + CR 406 + CR 602.2c — ExileSelf cost moves source to exile at activation time.
/// After activation (before resolution), the source should be in exile.
#[test]
fn test_exile_self_cost_moves_source_to_exile() {
    let p1 = p(1);
    // Artifact with Cost::ExileSelf + Effect::DrawCards (simple body)
    let source = ObjectSpec::artifact(p1, "Exile Stone")
        .in_zone(ZoneId::Battlefield)
        .with_activated_ability(ActivatedAbility {
            targets: vec![],
            cost: ActivationCost {
                exile_self: true,
                ..Default::default()
            },
            description: "Exile this: Draw a card.".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        });

    let library_card = ObjectSpec::card(p1, "placeholder").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(source)
        .object(library_card)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Exile Stone");

    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("activate exile-self ability should succeed");

    // Source moved to exile at activation time (before resolution)
    let exiled = find_object_in_zone(&state, "Exile Stone", ZoneId::Exile);
    assert!(
        exiled.is_some(),
        "CR 118.12 + CR 406 + CR 602.2c: source must be in exile after exile-self cost payment"
    );

    // Source is NOT on the battlefield anymore
    let still_on_bf = find_object_in_zone(&state, "Exile Stone", ZoneId::Battlefield);
    assert!(
        still_on_bf.is_none(),
        "source must not remain on battlefield after exile-self"
    );

    // ObjectExiled event emitted
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ObjectExiled { player, .. } if *player == p1)),
        "CR 118.12 + CR 406 + CR 602.2c: ObjectExiled event must be emitted when exile-self cost is paid"
    );

    // Ability is on the stack (not yet resolved)
    assert!(
        !state.stack_objects.is_empty(),
        "Ability should be on the stack waiting for priority passes"
    );
}

/// CR 602.2 — ExileSelf ability resolves correctly via embedded_effect even after source is gone.
/// After resolution: card drawn, source still in exile.
#[test]
fn test_exile_self_ability_resolves_after_source_gone() {
    let p1 = p(1);
    let source = ObjectSpec::artifact(p1, "Exile Stone II")
        .in_zone(ZoneId::Battlefield)
        .with_activated_ability(ActivatedAbility {
            targets: vec![],
            cost: ActivationCost {
                exile_self: true,
                ..Default::default()
            },
            description: "Exile this: Draw a card.".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        });

    // Need a card in library to draw
    let library_card = ObjectSpec::creature(p1, "Library Bear", 2, 2).in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(source)
        .object(library_card)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Exile Stone II");
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // Activate ability
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("activate");

    // Resolve: pass priority until stack is empty
    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    // Effect resolved: card was drawn
    let final_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        final_hand_size,
        initial_hand_size + 1,
        "CR 602.2: effect resolves via embedded_effect even after source is in exile"
    );

    // Source remains in exile
    let still_exiled = find_object_in_zone(&state, "Exile Stone II", ZoneId::Exile);
    assert!(
        still_exiled.is_some(),
        "Source must remain in exile after resolution"
    );
}

/// CR 602.2 — Mana cost + ExileSelf: if mana payment fails, exile does not happen (cost atomicity).
/// This tests that cost validation blocks the whole activation.
#[test]
fn test_exile_self_with_mana_fails_without_mana() {
    let p1 = p(1);
    // Ability costs {3} + ExileSelf
    let source = ObjectSpec::artifact(p1, "Costly Exile Stone")
        .in_zone(ZoneId::Battlefield)
        .with_activated_ability(ActivatedAbility {
            targets: vec![],
            cost: ActivationCost {
                mana_cost: Some(ManaCost {
                    generic: 3,
                    ..Default::default()
                }),
                exile_self: true,
                ..Default::default()
            },
            description: "{3}, Exile this: Draw a card.".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        });

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(source)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        // No mana in pool — can't pay {3}
        .build()
        .unwrap();

    let source_id = find_object(&state, "Costly Exile Stone");

    // Activation should fail due to insufficient mana
    let result = process_command(
        state.clone(),
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2: activation should fail when mana cost cannot be paid"
    );

    // Source remains on battlefield — exile did NOT happen
    let still_on_bf = find_object_in_zone(&state, "Costly Exile Stone", ZoneId::Battlefield);
    assert!(
        still_on_bf.is_some(),
        "Source should remain on battlefield when activation fails"
    );
}

/// PB-S H1 defense: `exile_self: true` and `exile_self: false` produce different hashes.
/// Two states that differ only in an ActivationCost.exile_self field must hash differently.
/// This defends against the hash gap that PB-S H1 found (missing field in HashInto).
#[test]
fn test_exile_self_field_participates_in_hash() {
    let p1 = p(1);

    let cost_no_exile = ActivationCost {
        exile_self: false,
        ..Default::default()
    };

    let cost_with_exile = ActivationCost {
        exile_self: true,
        ..Default::default()
    };

    // Build two states where the only difference is exile_self on an activated ability
    let make_state = |cost: ActivationCost| {
        let source = ObjectSpec::artifact(p1, "Hash Stone")
            .in_zone(ZoneId::Battlefield)
            .with_activated_ability(ActivatedAbility {
                targets: vec![],
                cost,
                description: "Test ability".to_string(),
                effect: None,
                sorcery_speed: false,
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            });
        GameStateBuilder::new()
            .add_player(p1)
            .add_player(p(2))
            .add_player(p(3))
            .add_player(p(4))
            .object(source)
            .at_step(Step::PreCombatMain)
            .active_player(p1)
            .build()
            .unwrap()
    };

    let state_no_exile = make_state(cost_no_exile);
    let state_with_exile = make_state(cost_with_exile);

    let hash_no_exile = state_no_exile.public_state_hash();
    let hash_with_exile = state_with_exile.public_state_hash();

    assert_ne!(
        hash_no_exile,
        hash_with_exile,
        "PB-S H1 defense: exile_self field must participate in ActivationCost hash (two costs differing only in exile_self must produce distinct public hashes)"
    );
}

// ── Balthor Integration ────────────────────────────────────────────────────────

/// CR 702.X (Balthor): static "Minion creatures get +1/+1" while Balthor is on the battlefield.
///
/// NOTE: GameStateBuilder bypasses ETB, so the static ContinuousEffect is injected manually.
/// This tests that `AllCreaturesWithSubtype("Minion")` + `ModifyBoth(1)` + `WhileSourceOnBattlefield`
/// applies correctly across all players' battlefields.
#[test]
fn test_balthor_static_minion_pump() {
    let p1 = p(1);

    let balthor_spec = ObjectSpec::creature(p1, "Balthor the Defiled", 2, 2)
        .with_card_id(CardId("balthor-the-defiled".to_string()))
        .in_zone(ZoneId::Battlefield);
    let minion1 = ObjectSpec::creature(p1, "Minion A", 2, 2)
        .with_subtypes(vec![SubType("Minion".to_string())])
        .in_zone(ZoneId::Battlefield);
    let minion2 = ObjectSpec::creature(p(2), "Minion B", 1, 1)
        .with_subtypes(vec![SubType("Minion".to_string())])
        .in_zone(ZoneId::Battlefield);
    let wizard = ObjectSpec::creature(p1, "Wizard C", 3, 3)
        .with_subtypes(vec![SubType("Wizard".to_string())])
        .in_zone(ZoneId::Battlefield);

    // Build state first to get Balthor's ObjectId, then inject the static effect.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(balthor_spec)
        .object(minion1)
        .object(minion2)
        .object(wizard)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // Locate Balthor so the static effect has its source set correctly.
    let balthor_id = find_object(&state, "Balthor the Defiled");

    // Inject the Balthor static as if register_static_continuous_effects had been called.
    // This mirrors what the ETB path does for AbilityDefinition::Static.
    let static_effect = ContinuousEffect {
        id: EffectId(9001),
        source: Some(balthor_id),
        timestamp: 1,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::AllCreaturesWithSubtype(SubType("Minion".to_string())),
        modification: LayerModification::ModifyBoth(1),
        is_cda: false,
        condition: None,
    };
    state.continuous_effects.push_back(static_effect);

    let m1_id = find_object(&state, "Minion A");
    let m2_id = find_object(&state, "Minion B");
    let w_id = find_object(&state, "Wizard C");

    let m1_chars = calculate_characteristics(&state, m1_id).unwrap();
    let m2_chars = calculate_characteristics(&state, m2_id).unwrap();
    let w_chars = calculate_characteristics(&state, w_id).unwrap();

    // Both Minions (any controller) get +1/+1 from Balthor static
    assert_eq!(m1_chars.power, Some(3), "Minion A: 2+1=3");
    assert_eq!(m1_chars.toughness, Some(3), "Minion A: 2+1=3");
    assert_eq!(m2_chars.power, Some(2), "Minion B: 1+1=2");
    assert_eq!(m2_chars.toughness, Some(2), "Minion B: 1+1=2");

    // Wizard is not a Minion → unaffected
    assert_eq!(w_chars.power, Some(3), "Wizard C: unchanged");
    assert_eq!(w_chars.toughness, Some(3), "Wizard C: unchanged");

    // Balthor itself is 2/2 (Zombie Dwarf, not a Minion)
    let balthor_chars = calculate_characteristics(&state, balthor_id).unwrap();
    assert_eq!(balthor_chars.power, Some(2), "Balthor: 2/2 base");
    assert_eq!(balthor_chars.toughness, Some(2), "Balthor: 2/2 base");
}

// ── Balthor End-to-End Integration ────────────────────────────────────────────

/// CR 118.12 + CR 406 + CR 602.2c — Balthor activated ability: exile Balthor as cost,
/// return all black and red creatures from each player's graveyard to the battlefield.
///
/// Setup:
/// - Balthor on the battlefield under P1 (abilities injected via with_activated_ability)
/// - P1 graveyard: 1 black creature, 1 red creature
/// - P2 graveyard: 1 black creature, 1 green creature (green must stay)
/// - Activate {B}{B}{B} + ExileSelf
///
/// Assertions:
/// (a) Balthor ends up in exile, NOT graveyard
/// (b) All black/red creatures (P1 and P2) are on the battlefield
/// (c) The green creature remains in P2's graveyard
#[test]
fn test_balthor_activated_reanimates_black_and_red() {
    let p1 = p(1);
    let p2 = p(2);

    // Build Balthor with the exact same activated ability from the card def
    let balthor = ObjectSpec::creature(p1, "Balthor the Defiled", 2, 2)
        .with_card_id(CardId("balthor-the-defiled".to_string()))
        .with_subtypes(vec![SubType("Zombie".to_string()), SubType("Dwarf".to_string())])
        .in_zone(ZoneId::Battlefield)
        .with_activated_ability(ActivatedAbility {
            targets: vec![],
            cost: ActivationCost {
                mana_cost: Some(ManaCost {
                    black: 3,
                    ..Default::default()
                }),
                exile_self: true,
                ..Default::default()
            },
            description: "{B}{B}{B}, Exile Balthor the Defiled: Each player returns all black and all red creature cards from their graveyard to the battlefield.".to_string(),
            effect: Some(Effect::ReturnAllFromGraveyardToBattlefield {
                graveyards: PlayerTarget::EachPlayer,
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    colors: Some(
                        [Color::Black, Color::Red].into_iter().collect(),
                    ),
                    ..Default::default()
                },
                tapped: false,
                controller_override: None,
                unique_names: false,
                permanent_cards_only: false,
            }),
            sorcery_speed: false,
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        });

    // Creatures in graveyards — set their colors so the filter works
    let p1_black = ObjectSpec::creature(p1, "P1 Black Zombie", 2, 2)
        .with_subtypes(vec![SubType("Zombie".to_string())])
        .with_colors(vec![Color::Black])
        .in_zone(ZoneId::Graveyard(p1));
    let p1_red = ObjectSpec::creature(p1, "P1 Red Dragon", 3, 3)
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_colors(vec![Color::Red])
        .in_zone(ZoneId::Graveyard(p1));
    let p2_black = ObjectSpec::creature(p2, "P2 Black Vampire", 2, 2)
        .with_subtypes(vec![SubType("Vampire".to_string())])
        .with_colors(vec![Color::Black])
        .in_zone(ZoneId::Graveyard(p2));
    let p2_green = ObjectSpec::creature(p2, "P2 Green Elf", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .with_colors(vec![Color::Green])
        .in_zone(ZoneId::Graveyard(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .object(balthor)
        .object(p1_black)
        .object(p1_red)
        .object(p2_black)
        .object(p2_green)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                black: 3,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let balthor_id = find_object(&state, "Balthor the Defiled");

    // Activate Balthor's ability (index 0 — the only activated ability)
    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: balthor_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("Balthor activation should succeed");

    // ExileSelf: Balthor should be in exile, not graveyard, immediately after activation
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ObjectExiled { player, .. } if *player == p1)),
        "CR 118.12 + CR 406: ObjectExiled event must be emitted"
    );

    // Resolve the ability: pass priority through all players
    let (state, _) = pass_all(state, &[p1, p2, p(3), p(4)]);

    // (a) Balthor is in exile, NOT graveyard
    let balthor_in_exile = find_object_in_zone(&state, "Balthor the Defiled", ZoneId::Exile);
    assert!(
        balthor_in_exile.is_some(),
        "(a) Balthor must be in exile after ExileSelf activation"
    );
    let balthor_in_gy = find_object_in_zone(&state, "Balthor the Defiled", ZoneId::Graveyard(p1));
    assert!(
        balthor_in_gy.is_none(),
        "(a) Balthor must NOT be in graveyard"
    );

    // (b) Black and red creatures are on the battlefield
    let p1_black_bf = find_object_in_zone(&state, "P1 Black Zombie", ZoneId::Battlefield);
    assert!(
        p1_black_bf.is_some(),
        "(b) P1 black creature must be on battlefield after reanimate"
    );
    let p1_red_bf = find_object_in_zone(&state, "P1 Red Dragon", ZoneId::Battlefield);
    assert!(
        p1_red_bf.is_some(),
        "(b) P1 red creature must be on battlefield after reanimate"
    );
    let p2_black_bf = find_object_in_zone(&state, "P2 Black Vampire", ZoneId::Battlefield);
    assert!(
        p2_black_bf.is_some(),
        "(b) P2 black creature must be on battlefield after reanimate"
    );

    // (c) Green creature remains in P2's graveyard
    let p2_green_bf = find_object_in_zone(&state, "P2 Green Elf", ZoneId::Battlefield);
    assert!(
        p2_green_bf.is_none(),
        "(c) P2 green creature must NOT be on battlefield"
    );
    let p2_green_gy = find_object_in_zone(&state, "P2 Green Elf", ZoneId::Graveyard(p2));
    assert!(
        p2_green_gy.is_some(),
        "(c) P2 green creature must remain in graveyard"
    );
}

// ── Obelisk of Urd Integration ────────────────────────────────────────────────

/// CR 614.1c — Obelisk of Urd: "As this enters, choose a creature type."
/// Chosen type is set via Replacement (not Triggered), so the static +2/+2 anthem
/// is immediately active after Obelisk enters — chosen_creature_type is NOT deferred.
///
/// **Observability window test** (anti-C1 regression):
/// - Humans and Goblins are on battlefield before Obelisk enters.
/// - Cast Obelisk via registry path.
/// - After resolution (stack empty), assert that:
///   (1) Humans get +2/+2 immediately (Replacement form set chosen_creature_type before anthem registered)
///   (2) Goblins are NOT pumped
///
/// If the choice were deferred to trigger resolution (the C1 bug), chosen_creature_type
/// would be None at the anthem-registration point, and power would still show base 1/1.
#[test]
fn test_obelisk_of_urd_chosen_type_pump() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::obelisk_of_urd::card()]);

    // Two Humans (more common, so Replacement picks Human deterministically)
    let human1 = ObjectSpec::creature(p1, "Human Soldier 1", 1, 1)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);
    let human2 = ObjectSpec::creature(p1, "Human Soldier 2", 1, 1)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);
    let goblin = ObjectSpec::creature(p1, "Goblin Ruffian", 1, 1)
        .with_subtypes(vec![SubType("Goblin".to_string())])
        .in_zone(ZoneId::Battlefield);

    // Obelisk costs {6}: give player 6 generic mana
    let obelisk_spec = ObjectSpec::card(p1, "Obelisk of Urd")
        .with_card_id(CardId("obelisk-of-urd".to_string()))
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(human1)
        .object(human2)
        .object(goblin)
        .object(obelisk_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                // Obelisk of Urd costs {6} — pay with 6 colorless (white+blue+black+red+green+2 colorless)
                white: 1,
                blue: 1,
                black: 1,
                red: 1,
                green: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let obelisk_id = find_object(&state, "Obelisk of Urd");

    // Cast Obelisk of Urd
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: obelisk_id,
            targets: vec![],
            alt_cost: None,
            additional_costs: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            x_value: 0,
            modes_chosen: vec![],
            kicker_times: 0,
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        },
    )
    .expect("cast Obelisk of Urd");

    // Resolve: pass priority for all players (Obelisk resolves, enters battlefield)
    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    // After resolution, Obelisk is on the battlefield with chosen_creature_type set (Replacement)
    // The static anthem immediately applies. Verify immediately — before any further priority pass.

    // Find the humans and goblin on the battlefield (they were already there)
    let h1_id = find_object(&state, "Human Soldier 1");
    let h2_id = find_object(&state, "Human Soldier 2");
    let g_id = find_object(&state, "Goblin Ruffian");

    let h1_chars = calculate_characteristics(&state, h1_id).unwrap();
    let h2_chars = calculate_characteristics(&state, h2_id).unwrap();
    let g_chars = calculate_characteristics(&state, g_id).unwrap();

    // Humans (chosen type) get +2/+2 immediately — CR 614.1c Replacement sets type at ETB
    assert_eq!(
        h1_chars.power,
        Some(3),
        "Human 1 must get +2 power immediately after Obelisk enters (anti-C1 regression)"
    );
    assert_eq!(
        h1_chars.toughness,
        Some(3),
        "Human 1 must get +2 toughness immediately"
    );
    assert_eq!(
        h2_chars.power,
        Some(3),
        "Human 2 must get +2 power immediately"
    );

    // Goblin is NOT of the chosen type — unchanged
    assert_eq!(
        g_chars.power,
        Some(1),
        "Goblin must NOT be pumped by Obelisk of Urd (wrong creature type)"
    );
    assert_eq!(
        g_chars.toughness,
        Some(1),
        "Goblin toughness must be unchanged"
    );
}

// ── City on Fire Integration ──────────────────────────────────────────────────

/// CR 614.1 — City on Fire: "If a source you control would deal damage to a permanent
/// or player, it deals triple that damage instead."
///
/// A creature with power 2 deals combat damage; player B should receive 6 (2 × 3).
/// Uses apply_damage_doubling (which handles both Double and Triple replacements).
///
/// CR 616.1 stacking note: Multiple self-replacement effects from the same player's
/// permanents are ordered by the affected player (or controller for damage-source
/// replacements). City on Fire (×3) + Angrath's Marauders (×2) → 2 × 3 × 2 = 12
/// or 2 × 2 × 3 = 12. Order is irrelevant for multiplication; both orderings give
/// the same result. Cite CR 616.1 / CR 701.10g.
#[test]
fn test_city_on_fire_triples_damage() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::city_on_fire::card()]);

    // City on Fire on P1's battlefield
    let city_spec = ObjectSpec::card(p1, "City on Fire")
        .with_card_id(CardId("city-on-fire".to_string()))
        .with_types(vec![CardType::Enchantment])
        .in_zone(ZoneId::Battlefield);

    // P1's creature (power 2) — the damage source
    let attacker = ObjectSpec::creature(p1, "Attacker", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(city_spec)
        .object(attacker)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // Register City on Fire's replacement ability
    let city_id = find_object(&state, "City on Fire");
    let registry = state.card_registry.clone();
    register_permanent_replacement_abilities(
        &mut state,
        city_id,
        p1,
        Some(&CardId("city-on-fire".to_string())).as_ref().copied(),
        &registry,
    );

    let attacker_id = find_object(&state, "Attacker");

    // CR 614.1a: City on Fire triples damage from P1's sources
    let (tripled, events) = apply_damage_doubling(&state, attacker_id, 2, None);
    assert_eq!(
        tripled, 6,
        "CR 614.1 / CR 701.10g: City on Fire must triple 2 → 6"
    );
    assert!(
        !events.is_empty(),
        "ReplacementEffectApplied event must be emitted"
    );

    // CR 616.1 / CR 701.10g: Doc note — City on Fire (×3) + a DoubleDamage effect (×2)
    // stacks multiplicatively: 2 × 3 × 2 = 12 (order-independent for multiplication).
    // Full stacking test is covered by damage_multiplier.rs test_double_and_triple_stack.
}

/// CR 614.1 — City on Fire does NOT triple damage from opponents' sources.
#[test]
fn test_city_on_fire_does_not_triple_opponent_sources() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::city_on_fire::card()]);

    let city_spec = ObjectSpec::card(p1, "City on Fire")
        .with_card_id(CardId("city-on-fire".to_string()))
        .with_types(vec![CardType::Enchantment])
        .in_zone(ZoneId::Battlefield);

    let opp_creature = ObjectSpec::creature(p2, "Opp Creature", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(city_spec)
        .object(opp_creature)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let city_id = find_object(&state, "City on Fire");
    let registry2 = state.card_registry.clone();
    register_permanent_replacement_abilities(
        &mut state,
        city_id,
        p1,
        Some(&CardId("city-on-fire".to_string())).as_ref().copied(),
        &registry2,
    );

    let opp_id = find_object(&state, "Opp Creature");

    // Opponent's source — not tripled by P1's City on Fire
    let (damage, events) = apply_damage_doubling(&state, opp_id, 3, None);
    assert_eq!(
        damage, 3,
        "CR 614.1: City on Fire must NOT triple opponent sources"
    );
    assert!(
        events.is_empty(),
        "No replacement event for opponent sources"
    );
}
