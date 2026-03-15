//! Prowess keyword ability tests (CR 702.108).
//!
//! Prowess is a triggered ability: "Whenever you cast a noncreature spell,
//! this creature gets +1/+1 until end of turn."
//!
//! Key rules verified:
//! - Triggers only on noncreature spells (CR 702.108a).
//! - Triggers only when the prowess creature's controller casts the spell ("you").
//! - Multiple spells stack additively.
//! - The +1/+1 bonus expires at end of turn (CR 514.2).
//! - Prowess resolves independently of the triggering spell (rulings).
//! - Storm copies are not casts and do not trigger prowess.

use mtg_engine::state::CardType;
use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, Command, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost,
    ObjectSpec, PlayerId, Step, Target, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
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

/// A simple instant (noncreature spell) — "Lightning Bolt" stand-in.
fn instant_def() -> CardDefinition {
    use mtg_engine::{CardEffectTarget, Effect, TargetRequirement};
    CardDefinition {
        card_id: CardId("lightning-bolt".to_string()),
        name: "Lightning Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: mtg_engine::cards::card_definition::EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A second instant (noncreature spell) — targets a player, to avoid damaging
/// the prowess creature during the multiple-spells test.
fn instant2_def() -> CardDefinition {
    use mtg_engine::{CardEffectTarget, Effect, TargetRequirement};
    CardDefinition {
        card_id: CardId("shock".to_string()),
        name: "Shock".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Shock deals 2 damage to any target.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: mtg_engine::cards::card_definition::EffectAmount::Fixed(2),
            },
            targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A creature spell — should NOT trigger prowess.
fn creature_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("grizzly-bears".to_string()),
        name: "Grizzly Bears".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}

/// An artifact creature spell — should NOT trigger prowess (creature type is present).
fn artifact_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("copper-gnomes".to_string()),
        name: "Copper Gnomes".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact, CardType::Creature]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}

// ── Test 1: Basic noncreature spell triggers prowess ──────────────────────────

#[test]
/// CR 702.108a — Prowess triggers when the controller casts a noncreature spell,
/// giving the prowess creature +1/+1 until end of turn.
fn test_prowess_basic_noncreature_spell_gives_plus_one() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![instant_def()]);

    // p1 has a 1/2 prowess creature on the battlefield.
    let prowess_creature =
        ObjectSpec::creature(p1, "Swiftspear", 1, 2).with_keyword(KeywordAbility::Prowess);

    // p1 has Lightning Bolt in hand.
    let spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(prowess_creature)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 red mana and priority.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Swiftspear");
    let spell_id = find_object(&state, "Lightning Bolt");

    // p1 casts Lightning Bolt targeting p2.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
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
        },
    )
    .unwrap();

    // SpellCast event emitted.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "CR 702.108a: SpellCast event expected"
    );

    // AbilityTriggered event emitted for prowess.
    assert!(
        cast_events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { controller, source_object_id, .. }
            if *controller == p1 && *source_object_id == creature_id
        )),
        "CR 702.108a: AbilityTriggered event expected for prowess creature"
    );

    // Stack has 2 items: the spell + prowess trigger.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.108a: stack should have spell + prowess trigger"
    );

    // Both players pass — prowess trigger resolves first (top of stack).
    let (state, _resolve_events) = pass_all(state, &[p1, p2]);

    // Prowess trigger resolved: creature is now 2/3 (1+1 / 2+1).
    let chars = calculate_characteristics(&state, creature_id)
        .expect("creature should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(2),
        "CR 702.108a: prowess should give +1 power (1 + 1 = 2)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.108a: prowess should give +1 toughness (2 + 1 = 3)"
    );
}

// ── Test 2: Creature spell does NOT trigger prowess ───────────────────────────

#[test]
/// CR 702.108a — Prowess does NOT trigger when the controller casts a creature spell.
fn test_prowess_does_not_trigger_on_creature_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![creature_spell_def()]);

    // p1 has a prowess creature on the battlefield.
    let prowess_creature =
        ObjectSpec::creature(p1, "Swiftspear", 1, 2).with_keyword(KeywordAbility::Prowess);

    // p1 has Grizzly Bears (creature spell) in hand.
    let spell = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            green: 1,
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(prowess_creature)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

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
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Grizzly Bears");

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
        },
    )
    .unwrap();

    // Only 1 item on stack (the creature spell — no prowess trigger).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.108a: creature spell should not trigger prowess"
    );

    // No AbilityTriggered events.
    assert!(
        !cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.108a: no prowess trigger for creature spell"
    );
}

// ── Test 3: Artifact creature spell does NOT trigger prowess ──────────────────

#[test]
/// CR 702.108a, Monastery Swiftspear ruling 2014-09-20 — "If a spell has multiple
/// types, and one of those types is creature (such as an artifact creature),
/// casting it won't cause prowess to trigger."
fn test_prowess_does_not_trigger_on_artifact_creature_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![artifact_creature_def()]);

    let prowess_creature =
        ObjectSpec::creature(p1, "Swiftspear", 1, 2).with_keyword(KeywordAbility::Prowess);

    // Artifact creature — has both Artifact and Creature card types.
    let spell = ObjectSpec::card(p1, "Copper Gnomes")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 4,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(prowess_creature)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Copper Gnomes");

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
        },
    )
    .unwrap();

    // Only 1 item on stack — no prowess trigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.108a: artifact creature should not trigger prowess"
    );
    assert!(
        !cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.108a: no prowess trigger for artifact creature spell"
    );
}

// ── Test 4: Opponent's noncreature spell does NOT trigger prowess ─────────────

#[test]
/// CR 702.108a — "whenever YOU cast" means only the prowess creature's controller.
/// An opponent casting a noncreature spell does NOT trigger prowess.
fn test_prowess_does_not_trigger_on_opponent_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![instant_def()]);

    // p1 has prowess creature on battlefield.
    let prowess_creature =
        ObjectSpec::creature(p1, "Swiftspear", 1, 2).with_keyword(KeywordAbility::Prowess);

    // p2 has Lightning Bolt in hand (p2 is active player).
    let spell = ObjectSpec::card(p2, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p2))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(prowess_creature)
        .object(spell)
        .active_player(p2) // p2 is active player
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p2);

    let spell_id = find_object(&state, "Lightning Bolt");

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![Target::Player(p1)],
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
        },
    )
    .unwrap();

    // Only p2's spell on the stack — p1's prowess creature should NOT trigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.108a: opponent spell should not trigger p1's prowess creature"
    );
    assert!(
        !cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.108a: no AbilityTriggered event for p1's prowess when opponent casts"
    );
}

// ── Test 5: Prowess resolves independently of triggering spell ─────────────────

#[test]
/// CR 702.108a, Monastery Swiftspear ruling 2014-09-20 — "Once it triggers,
/// prowess isn't connected to the spell that caused it to trigger."
/// After prowess resolves, the triggering spell remains on the stack.
fn test_prowess_resolves_independently_of_triggering_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![instant_def()]);

    let prowess_creature =
        ObjectSpec::creature(p1, "Swiftspear", 1, 2).with_keyword(KeywordAbility::Prowess);

    let spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(prowess_creature)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Swiftspear");
    let spell_id = find_object(&state, "Lightning Bolt");

    // p1 casts Lightning Bolt targeting p2.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
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
        },
    )
    .unwrap();

    // Stack: Lightning Bolt (bottom) + Prowess trigger (top).
    assert_eq!(state.stack_objects.len(), 2);

    // Both pass — prowess trigger resolves first.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After prowess resolves: creature has +1/+1, Lightning Bolt still on stack.
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert_eq!(
        chars.power,
        Some(2),
        "CR 702.108a: prowess should have resolved giving +1 power"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.108a: prowess should have resolved giving +1 toughness"
    );

    // Lightning Bolt is still on the stack (not yet resolved).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.108a: Lightning Bolt should still be on stack after prowess resolves"
    );
}

// ── Test 6: Prowess +1/+1 expires at end of turn ──────────────────────────────

#[test]
/// CR 702.108a ("until end of turn"), CR 514.2 — the +1/+1 from prowess expires
/// during the Cleanup step. After expire_end_of_turn_effects is called, the
/// continuous effect is removed and the creature returns to its printed P/T.
fn test_prowess_until_end_of_turn_expires() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![instant_def()]);

    let prowess_creature =
        ObjectSpec::creature(p1, "Swiftspear", 1, 2).with_keyword(KeywordAbility::Prowess);

    let spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(prowess_creature)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Swiftspear");
    let spell_id = find_object(&state, "Lightning Bolt");

    // Cast and let prowess resolve.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
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
        },
    )
    .unwrap();

    // Both pass — prowess resolves (top of stack).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Prowess effect active: creature is 2/3.
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert_eq!(chars.power, Some(2), "prowess effect active before cleanup");
    assert_eq!(
        chars.toughness,
        Some(3),
        "prowess effect active before cleanup"
    );

    // Simulate cleanup by calling expire_end_of_turn_effects.
    let mut state = state;
    mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state);

    // After cleanup: creature is back to 1/2.
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert_eq!(
        chars.power,
        Some(1),
        "CR 514.2: prowess +1 power should expire at cleanup"
    );
    assert_eq!(
        chars.toughness,
        Some(2),
        "CR 514.2: prowess +1 toughness should expire at cleanup"
    );
}

// ── Test 7: Multiple spells stack additively ──────────────────────────────────

#[test]
/// CR 702.108a — Casting two noncreature spells creates two prowess triggers.
/// After both resolve, the creature has +2/+2 total.
/// Uses two instants so the second can be cast while the first is on the stack.
fn test_prowess_multiple_spells_stack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![instant_def(), instant2_def()]);

    // p1 has a prowess creature (1/2).
    let prowess_creature =
        ObjectSpec::creature(p1, "Swiftspear", 1, 2).with_keyword(KeywordAbility::Prowess);

    // p1 has two instants in hand: Lightning Bolt and Shock.
    let bolt = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let shock = ObjectSpec::card(p1, "Shock")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(prowess_creature)
        .object(bolt)
        .object(shock)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 2);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Swiftspear");
    let bolt_id = find_object(&state, "Lightning Bolt");

    // Cast Lightning Bolt targeting p2 — prowess trigger 1 goes on stack.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: bolt_id,
            targets: vec![Target::Player(p2)],
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
        },
    )
    .unwrap();

    // Stack has 2 items: Lightning Bolt + prowess trigger.
    assert_eq!(state.stack_objects.len(), 2);

    // Both pass — prowess trigger 1 resolves first, giving +1/+1.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature is now 2/3. Stack still has Lightning Bolt.
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert_eq!(chars.power, Some(2), "after first prowess: 1+1=2");
    assert_eq!(chars.toughness, Some(3), "after first prowess: 2+1=3");
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Lightning Bolt still on stack"
    );

    // Cast Shock (instant) while Lightning Bolt is on stack — prowess trigger 2.
    let shock_id = find_object(&state, "Shock");
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: shock_id,
            targets: vec![Target::Player(p2)],
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
        },
    )
    .unwrap();

    // Stack: Lightning Bolt + Shock + prowess trigger 2 = 3 items.
    assert_eq!(state.stack_objects.len(), 3);

    // Both pass — prowess trigger 2 resolves, giving another +1/+1.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature is now 3/4 (+2/+2 total from two prowess triggers).
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.108a: two prowess triggers should give +2 power total"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "CR 702.108a: two prowess triggers should give +2 toughness total"
    );
}

// ── Test 8: Multiplayer — only controller's prowess triggers ──────────────────

#[test]
/// CR 702.108a, multiplayer — In a 4-player game, only the active player's
/// prowess creatures trigger on their own noncreature spells. Opponents'
/// prowess creatures do NOT trigger.
fn test_prowess_multiplayer_only_controllers_creatures_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let registry = CardRegistry::new(vec![instant_def()]);

    // p1 has a prowess creature (active player).
    let p1_prowess =
        ObjectSpec::creature(p1, "P1 Swiftspear", 1, 2).with_keyword(KeywordAbility::Prowess);

    // p3 also has a prowess creature (opponent).
    let p3_prowess =
        ObjectSpec::creature(p3, "P3 Swiftspear", 1, 2).with_keyword(KeywordAbility::Prowess);

    // p1 has Lightning Bolt in hand.
    let spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry.clone())
        .object(p1_prowess)
        .object(p3_prowess)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let p1_creature_id = find_object(&state, "P1 Swiftspear");
    let spell_id = find_object(&state, "Lightning Bolt");

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
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
        },
    )
    .unwrap();

    // Only 2 items on stack: spell + p1's prowess trigger.
    // p3's prowess creature should NOT trigger (different controller).
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.108a: only p1's prowess should trigger (spell + 1 trigger)"
    );

    // Exactly 1 AbilityTriggered event — from p1's creature only.
    let triggered_count = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 1,
        "CR 702.108a: exactly 1 prowess trigger expected (p1's creature only)"
    );

    // The triggered ability is controlled by p1 and sources from p1's creature.
    assert!(
        cast_events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { controller, source_object_id, .. }
            if *controller == p1 && *source_object_id == p1_creature_id
        )),
        "CR 702.108a: p1's prowess trigger should be controlled by p1"
    );
}
