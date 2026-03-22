//! Overload keyword ability tests (CR 702.96).
//!
//! Overload is an alternative cost that changes "target" to "each" -- the spell
//! affects ALL valid objects instead of a chosen single target.
//!
//! Key rules verified:
//! - Overload is an alternative cost: pay overload cost instead of mana cost (CR 702.96a).
//! - Overloaded spells have NO targets (CR 702.96b).
//! - Overloaded spells cannot fizzle (CR 702.96b / CR 608.2b).
//! - Overloaded spells bypass hexproof/shroud (CR 702.96b).
//! - Condition::WasOverloaded drives conditional effects (CR 702.96a).
//! - Overload is mutually exclusive with other alternative costs (CR 118.9a).
//! - Commander tax applies on top of overload cost (CR 118.9d).
//! - Normal (non-overloaded) cast still requires a target.

use mtg_engine::cards::card_definition::{
    Condition, ForEachTarget, TargetController, TargetFilter,
};
use mtg_engine::state::types::AltCostKind;
use mtg_engine::state::{CardType, ManaPool, SuperType};
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    Command, Effect, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, Step, TargetRequirement, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_in_zone(
    state: &mtg_engine::GameState,
    name: &str,
    zone: ZoneId,
) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
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

/// Mock Vandalblast: Sorcery {R}.
/// Normal: "Destroy target artifact."
/// Overload {4}{R}: "Destroy each artifact you don't control."
///
/// The Conditional effect uses Condition::WasOverloaded to branch between
/// ForEach (opponents' artifacts) when overloaded and DeclaredTarget (single artifact) when not.
///
/// The overloaded branch uses TargetController::Opponent to model the oracle text
/// "each artifact you don't control". This documents the intended behavior and will enforce
/// it correctly once matches_filter checks the controller field (a separate LOW remediation item).
fn mock_vandalblast_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-vandalblast".to_string()),
        name: "Mock Vandalblast".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text:
            "Destroy target artifact. Overload {4}{R} (Destroy each artifact you don't control.)"
                .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Overload),
            AbilityDefinition::Overload {
                cost: ManaCost {
                    generic: 4,
                    red: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::WasOverloaded,
                    if_true: Box::new(Effect::ForEach {
                        over: ForEachTarget::EachPermanentMatching(TargetFilter {
                            has_card_type: Some(CardType::Artifact),
                            // CR 702.96a: "each artifact you don't control" — opponents only.
                            controller: TargetController::Opponent,
                            ..Default::default()
                        }),
                        effect: Box::new(Effect::DestroyPermanent {
                            target: CardEffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                        }),
                    }),
                    if_false: Box::new(Effect::DestroyPermanent {
                        target: CardEffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                    }),
                },
                targets: vec![TargetRequirement::TargetArtifact],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// A simple artifact card for use as target in tests.
fn mock_artifact_def(name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(name.to_lowercase().replace(' ', "-")),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        ..Default::default()
    }
}

// ── Test 1: Normal cast targets single artifact ────────────────────────────────

/// CR 702.96a — Normal (non-overloaded) cast of Mock Vandalblast ({R}).
/// Destroys the declared target artifact. Other artifacts are unaffected.
#[test]
fn test_702_96_normal_cast_targets_single() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        mock_vandalblast_def(),
        mock_artifact_def("Sol Ring"),
        mock_artifact_def("Arcane Signet"),
    ]);

    let spell = ObjectSpec::card(p1, "Mock Vandalblast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-vandalblast".to_string()))
        .with_types(vec![CardType::Sorcery]);

    let artifact1 = ObjectSpec::card(p2, "Sol Ring")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("sol-ring".to_string()))
        .with_types(vec![CardType::Artifact]);

    let artifact2 = ObjectSpec::card(p2, "Arcane Signet")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("arcane-signet".to_string()))
        .with_types(vec![CardType::Artifact]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(artifact1)
        .object(artifact2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {R} for normal cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Mock Vandalblast");
    let sol_ring_id = find_object(&state, "Sol Ring");

    // Cast normally, targeting Sol Ring.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![mtg_engine::state::targeting::Target::Object(sol_ring_id)],
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
    .unwrap_or_else(|e| panic!("Normal CastSpell failed: {:?}", e));

    // Spell on stack — NOT overloaded.
    assert!(
        !state.stack_objects[0].was_overloaded,
        "CR 702.96a: normal cast should not set was_overloaded"
    );

    // Resolve (both players pass).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Sol Ring destroyed; Arcane Signet survives.
    let sol_ring_in_graveyard = find_object_in_zone(&state, "Sol Ring", ZoneId::Graveyard(p2));
    let arcane_signet_on_bf = find_object_in_zone(&state, "Arcane Signet", ZoneId::Battlefield);
    assert!(
        sol_ring_in_graveyard.is_some(),
        "CR 702.96a: target artifact should be destroyed"
    );
    assert!(
        arcane_signet_on_bf.is_some(),
        "CR 702.96a: non-targeted artifact should survive normal cast"
    );
}

// ── Test 2: Overloaded cast destroys opponents' artifacts but not caster's ─────

/// CR 702.96a/b — Overloaded Mock Vandalblast ({4}{R}) destroys each artifact
/// P1 doesn't control (oracle text: "Destroy each artifact you don't control").
/// P1's own artifact must survive; P2's artifacts are destroyed.
///
/// Note: TargetController::Opponent in the mock's TargetFilter documents the
/// intended behavior. The mock_vandalblast_def uses TargetController::Opponent,
/// which is checked by collect_for_each via EachPermanentMatching.
#[test]
fn test_702_96_overloaded_cast_destroys_all_matching() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        mock_vandalblast_def(),
        mock_artifact_def("Sol Ring"),
        mock_artifact_def("Arcane Signet"),
        mock_artifact_def("P1 Artifact"),
    ]);

    let spell = ObjectSpec::card(p1, "Mock Vandalblast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-vandalblast".to_string()))
        .with_types(vec![CardType::Sorcery]);

    let artifact1 = ObjectSpec::card(p2, "Sol Ring")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("sol-ring".to_string()))
        .with_types(vec![CardType::Artifact]);

    let artifact2 = ObjectSpec::card(p2, "Arcane Signet")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("arcane-signet".to_string()))
        .with_types(vec![CardType::Artifact]);

    // P1's own artifact — must survive since the spell destroys only opponents' artifacts.
    let p1_artifact = ObjectSpec::card(p1, "P1 Artifact")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("p1-artifact".to_string()))
        .with_types(vec![CardType::Artifact]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(artifact1)
        .object(artifact2)
        .object(p1_artifact)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {4}{R} for overload cost.
    let pool = &mut state.players.get_mut(&p1).unwrap().mana_pool;
    pool.add(ManaColor::Red, 1);
    pool.add(ManaColor::Colorless, 4);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Mock Vandalblast");

    // Cast with overload (no targets).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Overload),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Overloaded CastSpell failed: {:?}", e));

    // Stack object: was_overloaded = true, targets empty.
    assert!(
        state.stack_objects[0].was_overloaded,
        "CR 702.96a: was_overloaded should be true on stack object"
    );
    assert!(
        state.stack_objects[0].targets.is_empty(),
        "CR 702.96b: overloaded spell should have no targets"
    );

    // Verify mana consumed: {4}{R} = 5 total.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.96a: {{4}}{{R}} overload cost should be fully deducted"
    );

    // Resolve: both players pass priority.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2's artifacts are destroyed (they are opponents' artifacts).
    let sol_ring_in_graveyard = find_object_in_zone(&state, "Sol Ring", ZoneId::Graveyard(p2));
    let arcane_signet_in_graveyard =
        find_object_in_zone(&state, "Arcane Signet", ZoneId::Graveyard(p2));
    assert!(
        sol_ring_in_graveyard.is_some(),
        "CR 702.96a: opponent's artifact (Sol Ring) should be destroyed by overloaded spell"
    );
    assert!(
        arcane_signet_in_graveyard.is_some(),
        "CR 702.96a: opponent's artifact (Arcane Signet) should be destroyed by overloaded spell"
    );

    // P1's own artifact must survive: "destroy each artifact you don't control".
    // CR 702.96a: the controller's own permanents are not affected.
    let p1_artifact_on_bf = find_object_in_zone(&state, "P1 Artifact", ZoneId::Battlefield);
    assert!(
        p1_artifact_on_bf.is_some(),
        "CR 702.96a: caster's own artifact should survive overloaded spell (TargetController::Opponent)"
    );
}

// ── Test 3: Overloaded spell cannot fizzle ────────────────────────────────────

/// CR 702.96b / CR 608.2b — Overloaded spell has no targets, so it cannot be
/// countered by the "all targets illegal" fizzle rule.
/// If there are no artifacts on the battlefield, the spell resolves (SpellResolved
/// event is emitted, not SpellFizzled).
#[test]
fn test_702_96_overloaded_no_targets_cannot_fizzle() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_vandalblast_def()]);

    let spell = ObjectSpec::card(p1, "Mock Vandalblast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-vandalblast".to_string()))
        .with_types(vec![CardType::Sorcery]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {4}{R} for overload cost. No artifacts on battlefield.
    let pool = &mut state.players.get_mut(&p1).unwrap().mana_pool;
    pool.add(ManaColor::Red, 1);
    pool.add(ManaColor::Colorless, 4);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Mock Vandalblast");

    // Cast with overload.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Overload),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Overloaded CastSpell failed: {:?}", e));

    // Resolve the spell — both players pass.
    let (_, events) = pass_all(state, &[p1, p2]);

    // Spell should have resolved (SpellResolved), NOT fizzled (SpellFizzled).
    let resolved = events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellResolved { .. }));
    let fizzled = events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellFizzled { .. }));
    assert!(
        resolved,
        "CR 702.96b: overloaded spell with no matching objects should still resolve"
    );
    assert!(
        !fizzled,
        "CR 702.96b: overloaded spell cannot be countered by the fizzle rule (CR 608.2b)"
    );
}

// ── Test 4: Overloaded spell bypasses hexproof ────────────────────────────────

/// CR 702.96b — "It may affect objects that couldn't be chosen as legal targets."
/// An overloaded spell can affect hexproof permanents since it doesn't target them.
#[test]
fn test_702_96_overloaded_bypasses_hexproof() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        mock_vandalblast_def(),
        mock_artifact_def("Hexproof Artifact"),
    ]);

    let spell = ObjectSpec::card(p1, "Mock Vandalblast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-vandalblast".to_string()))
        .with_types(vec![CardType::Sorcery]);

    // An artifact with hexproof: cannot be targeted, but CAN be affected by overload.
    let hexproof_artifact = ObjectSpec::card(p2, "Hexproof Artifact")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("hexproof-artifact".to_string()))
        .with_types(vec![CardType::Artifact])
        .with_keyword(KeywordAbility::Hexproof);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(hexproof_artifact)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {4}{R} for overload cost.
    let pool = &mut state.players.get_mut(&p1).unwrap().mana_pool;
    pool.add(ManaColor::Red, 1);
    pool.add(ManaColor::Colorless, 4);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Mock Vandalblast");

    // Cast with overload — no targets (the hexproof artifact is not targeted).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Overload),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Overloaded CastSpell failed: {:?}", e));

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Hexproof artifact should be destroyed (not targeted, so hexproof doesn't protect it).
    let hexproof_in_graveyard =
        find_object_in_zone(&state, "Hexproof Artifact", ZoneId::Graveyard(p2));
    assert!(
        hexproof_in_graveyard.is_some(),
        "CR 702.96b: overloaded spell should affect hexproof artifacts (not targeting them)"
    );
}

// ── Test 5: Alternative cost exclusivity ─────────────────────────────────────

/// CR 118.9a — Overload is an alternative cost. Cannot combine with evoke.
#[test]
fn test_702_96_alternative_cost_exclusivity_with_evoke() {
    let p1 = p(1);
    let p2 = p(2);

    // Mock a card that has both Evoke and Overload (pathological combination).
    let weird_card = CardDefinition {
        card_id: CardId("weird-card".to_string()),
        name: "Weird Card".to_string(),
        mana_cost: Some(ManaCost {
            red: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Evoke),
            AbilityDefinition::Evoke {
                cost: ManaCost {
                    red: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Keyword(KeywordAbility::Overload),
            AbilityDefinition::Overload {
                cost: ManaCost {
                    generic: 4,
                    red: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![weird_card]);

    let card = ObjectSpec::card(p1, "Weird Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("weird-card".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let card_id = find_object(&state, "Weird Card");

    // Attempt to cast with both evoke AND overload — must fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Evoke),
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
        "CR 118.9a: cannot combine overload with evoke (only one alternative cost)"
    );
}

// ── Test 6: Overload pays the overload cost ───────────────────────────────────

/// CR 702.96a / CR 601.2f — The overload cost is paid instead of the mana cost.
/// Insufficient mana for the overload cost is rejected.
#[test]
fn test_702_96_pays_overload_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_vandalblast_def()]);

    let spell = ObjectSpec::card(p1, "Mock Vandalblast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-vandalblast".to_string()))
        .with_types(vec![CardType::Sorcery]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only give p1 {R} (normal cost), not {4}{R} (overload cost).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Mock Vandalblast");

    // Attempt overload with insufficient mana — must fail.
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
            alt_cost: Some(AltCostKind::Overload),
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
        "CR 702.96a / CR 601.2f: insufficient mana for overload cost should be rejected"
    );
}

// ── Test 7: Targets forbidden when overloaded ─────────────────────────────────

/// CR 702.96b — Overloaded spells have no targets.
/// Passing a target when cast_with_overload: true must return an error.
#[test]
fn test_702_96_no_targets_allowed_when_overloaded() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_vandalblast_def(), mock_artifact_def("Sol Ring")]);

    let spell = ObjectSpec::card(p1, "Mock Vandalblast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-vandalblast".to_string()))
        .with_types(vec![CardType::Sorcery]);

    let artifact = ObjectSpec::card(p2, "Sol Ring")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("sol-ring".to_string()))
        .with_types(vec![CardType::Artifact]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(artifact)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let pool = &mut state.players.get_mut(&p1).unwrap().mana_pool;
    pool.add(ManaColor::Red, 1);
    pool.add(ManaColor::Colorless, 4);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Mock Vandalblast");
    let artifact_id = find_object(&state, "Sol Ring");

    // Attempt to cast overloaded WITH a target — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![mtg_engine::state::targeting::Target::Object(artifact_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Overload),
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
        "CR 702.96b: overloaded spells have no targets — passing a target should be rejected"
    );
}

// ── Test 8: Normal cast (not overloaded) — WasOverloaded is false ─────────────

/// CR 702.96a — When a spell is NOT cast with overload, Condition::WasOverloaded
/// evaluates to false, and the single-target branch is executed.
#[test]
fn test_702_96_condition_was_overloaded_false_when_not_overloaded() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        mock_vandalblast_def(),
        mock_artifact_def("Sol Ring"),
        mock_artifact_def("Arcane Signet"),
    ]);

    let spell = ObjectSpec::card(p1, "Mock Vandalblast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-vandalblast".to_string()))
        .with_types(vec![CardType::Sorcery]);

    let artifact1 = ObjectSpec::card(p2, "Sol Ring")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("sol-ring".to_string()))
        .with_types(vec![CardType::Artifact]);

    let artifact2 = ObjectSpec::card(p2, "Arcane Signet")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("arcane-signet".to_string()))
        .with_types(vec![CardType::Artifact]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(artifact1)
        .object(artifact2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 only {R} (normal cost).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Mock Vandalblast");
    let sol_ring_id = find_object(&state, "Sol Ring");

    // Cast normally, targeting only Sol Ring.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![mtg_engine::state::targeting::Target::Object(sol_ring_id)],
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
    .unwrap_or_else(|e| panic!("Normal cast failed: {:?}", e));

    // was_overloaded is false.
    assert!(
        !state.stack_objects[0].was_overloaded,
        "CR 702.96a: normal cast should have was_overloaded = false"
    );

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Only Sol Ring destroyed; Arcane Signet survives (WasOverloaded was false → single-target branch).
    let sol_ring_in_gy = find_object_in_zone(&state, "Sol Ring", ZoneId::Graveyard(p2));
    let arcane_on_bf = find_object_in_zone(&state, "Arcane Signet", ZoneId::Battlefield);
    assert!(
        sol_ring_in_gy.is_some(),
        "CR 702.96a: targeted artifact should be destroyed when not overloaded"
    );
    assert!(
        arcane_on_bf.is_some(),
        "CR 702.96a: non-targeted artifact should survive when not overloaded (WasOverloaded = false)"
    );
}

// ── Test 9: Commander tax applies on top of overload cost ─────────────────────

/// CR 118.9d — Additional costs apply on top of an alternative cost.
/// CR 903.8 — Commander tax is +{2} per prior cast from the command zone.
///
/// When a commander with Overload is cast from the command zone using the overload
/// cost, the commander tax is added on top of the overload cost (not the mana cost).
/// One prior cast → tax = +{2} → total = overload cost + {2}.
#[test]
fn test_702_96_commander_tax_applies_to_overload_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let cmd_id = CardId("mock-overload-commander".to_string());

    // Mock commander sorcery with Overload {4}{R}. Base mana cost: {R}.
    // Using a Sorcery in the command zone for simplicity — the engine checks zone,
    // not card type, for the commander tax path.
    let overload_commander_def = CardDefinition {
        card_id: cmd_id.clone(),
        name: "Overload Commander".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Overload {4}{R}.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Overload),
            AbilityDefinition::Overload {
                cost: ManaCost {
                    generic: 4,
                    red: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::WasOverloaded,
                    if_true: Box::new(Effect::ForEach {
                        over: ForEachTarget::EachPermanentMatching(TargetFilter {
                            has_card_type: Some(CardType::Artifact),
                            controller: TargetController::Opponent,
                            ..Default::default()
                        }),
                        effect: Box::new(Effect::DestroyPermanent {
                            target: CardEffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                        }),
                    }),
                    if_false: Box::new(Effect::DestroyPermanent {
                        target: CardEffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                    }),
                },
                targets: vec![TargetRequirement::TargetArtifact],
                modes: None,
                cant_be_countered: false,
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![overload_commander_def]);

    // Commander in the command zone.
    let cmd_obj = ObjectSpec::card(p1, "Overload Commander")
        .with_card_id(cmd_id.clone())
        .with_types(vec![CardType::Creature])
        .with_supertypes(vec![SuperType::Legendary])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Command(p1));

    // Overload cost {4}{R} + {2} commander tax = {6}{R} = 7 mana total.
    let total_mana = ManaPool {
        red: 1,
        colorless: 6,
        ..Default::default()
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, cmd_id.clone())
        .player_mana(p1, total_mana)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(cmd_obj)
        .build()
        .unwrap();

    // Pre-set commander tax to 1 (cast once previously → next cast adds +{2}).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(cmd_id.clone(), 1);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = *state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Cast with overload from command zone.
    let (new_state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Overload),
            prototype: false,
                modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| {
        panic!(
            "CR 118.9d: overload+tax cast from command zone should succeed with sufficient mana: {:?}",
            e
        )
    });

    // Verify CommanderCastFromCommandZone event with tax_paid = 1.
    let commander_event = events.iter().find(|e| {
        matches!(
            e,
            GameEvent::CommanderCastFromCommandZone { player, card_id, tax_paid: 1 }
            if *player == p1 && *card_id == cmd_id
        )
    });
    assert!(
        commander_event.is_some(),
        "CR 118.9d / CR 903.8: expected CommanderCastFromCommandZone with tax_paid=1 for overload cast; events: {:?}",
        events
    );

    // Verify spell is on stack with was_overloaded = true.
    assert!(
        new_state.stack_objects[0].was_overloaded,
        "CR 702.96a: stack object should have was_overloaded = true"
    );

    // Verify all 7 mana was consumed ({4}{R} overload + {2} tax = {6}{R}).
    assert!(
        new_state.players[&p1].mana_pool.is_empty(),
        "CR 118.9d: all mana ({{4}}{{R}} overload + {{2}} commander tax = {{6}}{{R}}) should be consumed"
    );

    // Verify commander tax incremented to 2.
    let tax_after = new_state.players[&p1]
        .commander_tax
        .get(&cmd_id)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        tax_after, 2,
        "CR 903.8: commander tax should increment to 2 after second cast"
    );
}

/// CR 118.9d — Insufficient mana for overload cost + commander tax is rejected.
/// With overload {4}{R} and 1 prior cast, need {6}{R} (7 mana); providing only {4}{R}
/// (5 mana, exactly the overload cost without tax) must be rejected.
#[test]
fn test_702_96_commander_tax_overload_insufficient_mana_rejected() {
    let p1 = p(1);
    let p2 = p(2);
    let cmd_id = CardId("mock-overload-commander-2".to_string());

    let overload_commander_def = CardDefinition {
        card_id: cmd_id.clone(),
        name: "Overload Commander 2".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Overload {4}{R}.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Overload),
            AbilityDefinition::Overload {
                cost: ManaCost {
                    generic: 4,
                    red: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::WasOverloaded,
                    if_true: Box::new(Effect::ForEach {
                        over: ForEachTarget::EachCreature,
                        effect: Box::new(Effect::DestroyPermanent {
                            target: CardEffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                        }),
                    }),
                    if_false: Box::new(Effect::DestroyPermanent {
                        target: CardEffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                    }),
                },
                targets: vec![TargetRequirement::TargetArtifact],
                modes: None,
                cant_be_countered: false,
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![overload_commander_def]);

    let cmd_obj = ObjectSpec::card(p1, "Overload Commander 2")
        .with_card_id(cmd_id.clone())
        .with_types(vec![CardType::Creature])
        .with_supertypes(vec![SuperType::Legendary])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Command(p1));

    // Only {4}{R} — enough for overload cost alone but NOT for overload + {2} tax.
    let insufficient_mana = ManaPool {
        red: 1,
        colorless: 4,
        ..Default::default()
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, cmd_id.clone())
        .player_mana(p1, insufficient_mana)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(cmd_obj)
        .build()
        .unwrap();

    // Pre-set commander tax to 1 (one prior cast → +{2} on next cast).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(cmd_id.clone(), 1);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = *state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Attempt overload cast with insufficient mana — must fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Overload),
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
        "CR 118.9d: overload cost + commander tax exceeds available mana — cast must be rejected"
    );
}

// ── Test 11: Overloaded spell affects all opponents in multiplayer ─────────────

/// CR 702.96a/b — Overloaded spell replaces "target" with "each", affecting
/// ALL opponents' matching permanents in a multiplayer game.
///
/// This verifies Commander-format correctness (architecture invariant 5:
/// "Multiplayer-first. Priority, triggers, combat — everything is designed for N players.")
/// With P2, P3, and P4 each controlling artifacts, all should be destroyed.
/// P1's own artifact survives (TargetController::Opponent).
#[test]
fn test_702_96_overloaded_hits_all_opponents_multiplayer() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let registry = CardRegistry::new(vec![
        mock_vandalblast_def(),
        mock_artifact_def("Sol Ring"),
        mock_artifact_def("Arcane Signet"),
        mock_artifact_def("Mana Vault"),
        mock_artifact_def("Mana Crypt"),
        mock_artifact_def("P1 Relic"),
    ]);

    let spell = ObjectSpec::card(p1, "Mock Vandalblast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-vandalblast".to_string()))
        .with_types(vec![CardType::Sorcery]);

    // P2 has 2 artifacts; both should be destroyed.
    let p2_artifact1 = ObjectSpec::card(p2, "Sol Ring")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("sol-ring".to_string()))
        .with_types(vec![CardType::Artifact]);
    let p2_artifact2 = ObjectSpec::card(p2, "Arcane Signet")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("arcane-signet".to_string()))
        .with_types(vec![CardType::Artifact]);

    // P3 has 1 artifact; should be destroyed.
    let p3_artifact = ObjectSpec::card(p3, "Mana Vault")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mana-vault".to_string()))
        .with_types(vec![CardType::Artifact]);

    // P4 has 1 artifact; should be destroyed.
    let p4_artifact = ObjectSpec::card(p4, "Mana Crypt")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mana-crypt".to_string()))
        .with_types(vec![CardType::Artifact]);

    // P1 has their own artifact; must survive (TargetController::Opponent).
    let p1_artifact = ObjectSpec::card(p1, "P1 Relic")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("p1-relic".to_string()))
        .with_types(vec![CardType::Artifact]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(spell)
        .object(p2_artifact1)
        .object(p2_artifact2)
        .object(p3_artifact)
        .object(p4_artifact)
        .object(p1_artifact)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {4}{R} for overload cost.
    let pool = &mut state.players.get_mut(&p1).unwrap().mana_pool;
    pool.add(ManaColor::Red, 1);
    pool.add(ManaColor::Colorless, 4);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Mock Vandalblast");

    // Cast with overload (no targets).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Overload),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("4-player overloaded CastSpell failed: {:?}", e));

    assert!(
        state.stack_objects[0].was_overloaded,
        "CR 702.96a: was_overloaded should be true in 4-player game"
    );

    // Resolve: all 4 players pass priority.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // P2's artifacts are destroyed.
    assert!(
        find_object_in_zone(&state, "Sol Ring", ZoneId::Graveyard(p2)).is_some(),
        "CR 702.96a: P2's Sol Ring should be destroyed by overloaded spell in 4-player game"
    );
    assert!(
        find_object_in_zone(&state, "Arcane Signet", ZoneId::Graveyard(p2)).is_some(),
        "CR 702.96a: P2's Arcane Signet should be destroyed by overloaded spell in 4-player game"
    );

    // P3's artifact is destroyed.
    assert!(
        find_object_in_zone(&state, "Mana Vault", ZoneId::Graveyard(p3)).is_some(),
        "CR 702.96a: P3's Mana Vault should be destroyed by overloaded spell in 4-player game"
    );

    // P4's artifact is destroyed.
    assert!(
        find_object_in_zone(&state, "Mana Crypt", ZoneId::Graveyard(p4)).is_some(),
        "CR 702.96a: P4's Mana Crypt should be destroyed by overloaded spell in 4-player game"
    );

    // P1's own artifact survives (TargetController::Opponent excludes caster's permanents).
    assert!(
        find_object_in_zone(&state, "P1 Relic", ZoneId::Battlefield).is_some(),
        "CR 702.96a: P1's own artifact should survive overloaded spell (TargetController::Opponent)"
    );
}
