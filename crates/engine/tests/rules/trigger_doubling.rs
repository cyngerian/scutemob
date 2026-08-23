//! Trigger doubling tests (CR 603.2d) — Panharmonicon-style effects.
//!
//! Session 9 of M9.4 implements:
//! - `TriggerDoubler` data model in `state/stubs.rs`
//! - `GameState::trigger_doublers` field
//! - `AbilityDefinition::TriggerDoubling` variant in `cards/card_definition.rs`
//! - Registration in `rules/replacement.rs::register_static_continuous_effects`
//! - Doubling logic in `rules/abilities.rs::flush_pending_triggers`
//!
//! CC#15: Panharmonicon doubles ETB triggers.
//! Two Panharmonicons triple a trigger.
//! Removing Panharmonicon after triggers are already on the stack doesn't cancel them.

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Completeness, Effect, EffectAmount, GameEvent, GameStateBuilder, ManaCost, ObjectSpec,
    PlayerId, PlayerTarget, StackObjectKind, Step, TriggerCondition, TriggerDoubler,
    TriggerDoublerFilter, TriggerEvent, TriggeredAbilityDef, TypeLine, ZoneId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}
fn p3() -> PlayerId {
    PlayerId(3)
}
fn p4() -> PlayerId {
    PlayerId(4)
}

/// Pass priority for all four players in order, collecting all events.
fn pass_all_four(
    state: mtg_engine::GameState,
    order: [PlayerId; 4],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for p in order {
        let (s, ev) = process_command(current, Command::PassPriority { player: p })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", p, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Build a CardDefinition for a Panharmonicon-like artifact.
///
/// CR 603.2d: "Whenever an artifact or creature enters the battlefield under your control,
/// if a triggered ability of a permanent you control would trigger from that event,
/// that ability triggers an additional time."
fn panharmonicon_def(id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.into()),
        name: name.into(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "Whenever an artifact or creature enters the battlefield under your control, if a triggered ability of a permanent you control would trigger from that event, that ability triggers an additional time.".into(),
        abilities: vec![AbilityDefinition::TriggerDoubling {
            filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
            additional_triggers: 1,
        }],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    }
}

/// Build a triggered ability: "Whenever any permanent enters the battlefield..."
fn any_etb_trigger(description: &str) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
        intervening_if: None,
        description: description.to_string(),
        effect: None,
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
    }
}

// ── CC#15: Panharmonicon doubles ETB triggers ─────────────────────────────────

/// CC#15 / CR 603.2d — Panharmonicon doubles ETB-watching triggered abilities.
///
/// Setup:
/// - Panharmonicon already on battlefield (its TriggerDoubler registered).
/// - "Watcher" creature on battlefield with "Whenever any permanent ETB, do X".
/// - Cast a third creature (the entering permanent).
///
/// When the third creature resolves and enters the battlefield, Watcher's
/// ETB trigger should fire TWICE (once baseline + once from Panharmonicon).
/// This means 2 `AbilityTriggered` events and 2 `TriggeredAbility` stack objects.
#[test]
fn test_panharmonicon_doubles_etb_trigger() {
    let p1 = p1();
    let p2 = p2();
    let p3 = p3();
    let p4 = p4();

    let panharmonicon_def = panharmonicon_def("panharmonicon-test", "Test Panharmonicon");
    let panharmonicon_card_id = panharmonicon_def.card_id.clone();

    // A plain creature card definition for the entering creature (no special abilities).
    let entering_def = CardDefinition {
        card_id: CardId("entering-creature-test".into()),
        name: "Entering Creature".into(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "".into(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    };
    let entering_card_id = entering_def.card_id.clone();

    let registry = CardRegistry::new(vec![panharmonicon_def, entering_def]);

    // Build state:
    // - p1 is active player (main phase)
    // - Panharmonicon is ALREADY on the battlefield (its TriggerDoubler is pre-registered
    //   by directly adding to trigger_doublers — the registration path is tested separately)
    // - "Watcher" creature is on battlefield with AnyPermanentEntersBattlefield trigger
    // - "Entering Creature" is in p1's hand, ready to cast
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        // Panharmonicon on battlefield (with card_id for static effect registration).
        .object(
            ObjectSpec::artifact(p1, "Test Panharmonicon")
                .with_card_id(panharmonicon_card_id.clone())
                .in_zone(ZoneId::Battlefield),
        )
        // Watcher creature on battlefield — watches for any ETB.
        .object(
            ObjectSpec::creature(p1, "Watcher", 1, 1)
                .with_triggered_ability(any_etb_trigger(
                    "Whenever a permanent ETB, do something (Panharmonicon test)",
                ))
                .in_zone(ZoneId::Battlefield),
        )
        // Entering creature in hand.
        .object(
            ObjectSpec::creature(p1, "Entering Creature", 2, 2)
                .with_card_id(entering_card_id)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Manually register the TriggerDoubler for the already-on-battlefield Panharmonicon.
    // (In a full game, this would be registered when Panharmonicon itself resolved.
    //  Here we're testing the trigger doubling mechanism, not the registration pathway.)
    let mut state = state;
    let panharmonicon_obj_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Test Panharmonicon")
        .map(|(id, _)| *id)
        .unwrap();

    state
        .trigger_doublers_mut()
        .push_back(mtg_engine::TriggerDoubler {
            source: panharmonicon_obj_id,
            controller: p1,
            filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
            additional_triggers: 1,
        });

    // Give p1 enough mana to cast the entering creature (MV=2).
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 2;

    let entering_hand_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Entering Creature")
        .map(|(id, _)| *id)
        .unwrap();

    // p1 casts the entering creature.
    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: entering_hand_id,
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
    .unwrap();

    // All four players pass priority → entering creature resolves and enters battlefield.
    // After entering, Watcher's AnyPermanentEntersBattlefield trigger fires.
    // Panharmonicon doubles it → 2 triggers on the stack.
    let (state, resolution_events) = pass_all_four(state, [p1, p2, p3, p4]);

    // Count AbilityTriggered events in the resolution batch.
    let triggered_count = resolution_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();

    assert_eq!(
        triggered_count, 2,
        "Panharmonicon should double the ETB trigger: expected 2 AbilityTriggered events, got {}; events: {:?}",
        triggered_count, resolution_events
    );

    // The stack should have exactly 2 triggered abilities.
    let trigger_stack_count = state
        .stack_objects()
        .iter()
        .filter(|s| matches!(s.kind, StackObjectKind::TriggeredAbility { .. }))
        .count();

    assert_eq!(
        trigger_stack_count, 2,
        "CC#15: Stack should have 2 triggered abilities (doubled by Panharmonicon); got {}",
        trigger_stack_count
    );
}

// ── Two Panharmonicons triple the trigger ─────────────────────────────────────

/// CR 603.2d — Two Panharmonicons: each adds 1 additional trigger.
///
/// With two Panharmonicons, an ETB-watching trigger fires 3 times:
/// 1 baseline + 1 from first Panharmonicon + 1 from second Panharmonicon = 3.
///
/// Rulings confirm that each Panharmonicon adds independently, they don't multiply.
#[test]
fn test_two_panharmonicons_triple_triggers() {
    let p1 = p1();
    let p2 = p2();
    let p3 = p3();
    let p4 = p4();

    let panharmonicon1_def = panharmonicon_def("panharmonicon-test-1", "Panharmonicon Alpha");
    let panharmonicon2_def = panharmonicon_def("panharmonicon-test-2", "Panharmonicon Beta");
    let entering_def = CardDefinition {
        card_id: CardId("entering-creature-test-2".into()),
        name: "Entering Creature 2".into(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "".into(),
        abilities: vec![],
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    };
    let entering_card_id = entering_def.card_id.clone();

    let registry = CardRegistry::new(vec![panharmonicon1_def, panharmonicon2_def, entering_def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        // Two Panharmonicons on battlefield.
        .object(ObjectSpec::artifact(p1, "Panharmonicon Alpha").in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::artifact(p1, "Panharmonicon Beta").in_zone(ZoneId::Battlefield))
        // Watcher with AnyPermanentEntersBattlefield.
        .object(
            ObjectSpec::creature(p1, "Watcher 2", 1, 1)
                .with_triggered_ability(any_etb_trigger(
                    "Whenever a permanent ETB (two-panharmonicon test)",
                ))
                .in_zone(ZoneId::Battlefield),
        )
        // Entering creature in hand.
        .object(
            ObjectSpec::creature(p1, "Entering Creature 2", 1, 1)
                .with_card_id(entering_card_id)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Register two TriggerDoublers (one per Panharmonicon).
    let mut state = state;

    let alpha_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Panharmonicon Alpha")
        .map(|(id, _)| *id)
        .unwrap();
    let beta_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Panharmonicon Beta")
        .map(|(id, _)| *id)
        .unwrap();

    state
        .trigger_doublers_mut()
        .push_back(mtg_engine::TriggerDoubler {
            source: alpha_id,
            controller: p1,
            filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
            additional_triggers: 1,
        });
    state
        .trigger_doublers_mut()
        .push_back(mtg_engine::TriggerDoubler {
            source: beta_id,
            controller: p1,
            filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
            additional_triggers: 1,
        });

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 1;

    let entering_hand_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Entering Creature 2")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: entering_hand_id,
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
    .unwrap();

    // All four players pass priority → creature resolves → 3 triggers fire (1 + 1 + 1).
    let (state, resolution_events) = pass_all_four(state, [p1, p2, p3, p4]);

    let triggered_count = resolution_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();

    assert_eq!(
        triggered_count, 3,
        "Two Panharmonicons should triple the ETB trigger: expected 3, got {}; events: {:?}",
        triggered_count, resolution_events
    );

    let trigger_stack_count = state
        .stack_objects()
        .iter()
        .filter(|s| matches!(s.kind, StackObjectKind::TriggeredAbility { .. }))
        .count();

    assert_eq!(
        trigger_stack_count, 3,
        "Stack should have 3 triggered abilities (tripled by two Panharmonicons); got {}",
        trigger_stack_count
    );
}

// ── Panharmonicon removal doesn't cancel already-triggered abilities ───────────

/// CR 603.2d — Removing Panharmonicon AFTER triggers are already on the stack
/// does not cancel those triggers. Triggers that were pushed to the stack
/// remain there independently of the permanent that generated the doubling.
///
/// This test:
/// 1. Sets up Panharmonicon + Watcher on battlefield.
/// 2. Causes a creature to enter — Watcher fires twice (2 triggers on stack).
/// 3. Verifies the stack has 2 triggers before any resolution.
/// 4. Manually removes the TriggerDoubler to simulate Panharmonicon leaving.
/// 5. Verifies the stack STILL has 2 triggers.
///
/// (The triggers are already on the stack; the doubler's removal only prevents
///  future ETB events from being doubled, not already-queued triggers.)
#[test]
fn test_panharmonicon_removal_doesnt_cancel_already_triggered() {
    let p1 = p1();
    let p2 = p2();
    let p3 = p3();
    let p4 = p4();

    let panharmonicon_def =
        panharmonicon_def("panharmonicon-test-removal", "Panharmonicon Removal Test");
    let entering_def = CardDefinition {
        card_id: CardId("entering-creature-test-3".into()),
        name: "Entering Creature 3".into(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "".into(),
        abilities: vec![],
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    };
    let entering_card_id = entering_def.card_id.clone();

    let registry = CardRegistry::new(vec![panharmonicon_def, entering_def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(ObjectSpec::artifact(p1, "Panharmonicon Removal Test").in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::creature(p1, "Watcher 3", 1, 1)
                .with_triggered_ability(any_etb_trigger("Whenever a permanent ETB (removal test)"))
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p1, "Entering Creature 3", 1, 1)
                .with_card_id(entering_card_id)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;

    let panharmonicon_obj_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Panharmonicon Removal Test")
        .map(|(id, _)| *id)
        .unwrap();

    state
        .trigger_doublers_mut()
        .push_back(mtg_engine::TriggerDoubler {
            source: panharmonicon_obj_id,
            controller: p1,
            filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
            additional_triggers: 1,
        });

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 1;

    let entering_hand_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Entering Creature 3")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: entering_hand_id,
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
    .unwrap();

    // All four players pass → creature resolves, enters battlefield, triggers fire (2x).
    let (state, _resolution_events) = pass_all_four(state, [p1, p2, p3, p4]);

    // Verify the stack has 2 triggered abilities from Panharmonicon doubling.
    let trigger_stack_count_before = state
        .stack_objects()
        .iter()
        .filter(|s| matches!(s.kind, StackObjectKind::TriggeredAbility { .. }))
        .count();

    assert_eq!(
        trigger_stack_count_before, 2,
        "Before Panharmonicon removal: expected 2 triggers on stack, got {}",
        trigger_stack_count_before
    );

    // Simulate Panharmonicon leaving the battlefield: remove its TriggerDoubler entry.
    // (In a real game this happens via SBA or destruction; here we directly remove it.)
    let mut state = state;
    state
        .trigger_doublers_mut()
        .retain(|d| d.source != panharmonicon_obj_id);

    // The stack should STILL have 2 triggered abilities — they don't disappear when
    // Panharmonicon is removed. Triggers already on the stack are independent.
    let trigger_stack_count_after = state
        .stack_objects()
        .iter()
        .filter(|s| matches!(s.kind, StackObjectKind::TriggeredAbility { .. }))
        .count();

    assert_eq!(
        trigger_stack_count_after, 2,
        "After Panharmonicon removal: stack triggers should be unchanged (still 2); got {}",
        trigger_stack_count_after
    );
}

// ── MR-M9.4-14: Full ETB-to-doubler registration pipeline ────────────────────

/// MR-M9.4-14 / CR 603.2d — Casting and resolving a Panharmonicon auto-registers
/// its TriggerDoubler via `register_static_continuous_effects` (no manual injection).
///
/// Verifies the complete path:
///   CardDefinition::TriggerDoubling → cast → resolve → ETB → static ability
///   registration → trigger_doublers populated → doubling works for next ETB.
///
/// Previous tests inject TriggerDoubler directly into state.trigger_doublers().
/// This test exercises the real registration pathway.
#[test]
fn test_panharmonicon_registration_via_resolution() {
    let p1 = p1();
    let p2 = p2();
    let p3 = p3();
    let p4 = p4();

    let pharm_def = panharmonicon_def("panharmonicon-reg-test", "Reg-Test Panharmonicon");
    let pharm_card_id = pharm_def.card_id.clone();

    let entering_def = CardDefinition {
        card_id: CardId("entering-creature-reg-test".into()),
        name: "Reg-Test Entering".into(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "".into(),
        abilities: vec![],
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    };
    let entering_card_id = entering_def.card_id.clone();

    let registry = CardRegistry::new(vec![pharm_def, entering_def]);

    // Panharmonicon starts in hand; Watcher is on battlefield; entering creature in hand.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(
            ObjectSpec::artifact(p1, "Reg-Test Panharmonicon")
                .with_card_id(pharm_card_id.clone())
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::creature(p1, "Reg-Test Watcher", 1, 1)
                .with_triggered_ability(any_etb_trigger("ETB watcher for reg test"))
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p1, "Reg-Test Entering", 1, 1)
                .with_card_id(entering_card_id)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Give p1 enough mana: 4 for Panharmonicon + 1 for entering creature.
    let mut state = state;
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 5;

    let pharm_hand_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Reg-Test Panharmonicon")
        .map(|(id, _)| *id)
        .unwrap();

    // Step 1: Cast Panharmonicon.
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: pharm_hand_id,
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
    .unwrap();

    // Step 2: All four pass → Panharmonicon resolves and enters the battlefield.
    // After resolution, register_static_continuous_effects fires and registers the
    // TriggerDoubler from Panharmonicon's AbilityDefinition::TriggerDoubling entry.
    // The Watcher's ETB trigger (from Panharmonicon entering) also goes onto the stack.
    let (state, _pharm_resolution_events) = pass_all_four(state, [p1, p2, p3, p4]);

    // The TriggerDoubler must now be registered (no manual injection).
    assert_eq!(
        state.trigger_doublers().len(),
        1,
        "After Panharmonicon resolves via casting, trigger_doublers should have 1 entry \
         (registered by register_static_continuous_effects); got {}",
        state.trigger_doublers().len()
    );

    // Step 2b: Drain all Watcher triggers from Panharmonicon's own ETB off the stack.
    // Panharmonicon is an artifact, so its own entry may trigger (and double) the Watcher.
    // One call to pass_all_four resolves exactly one stack item; loop until empty.
    let mut state = state;
    while !state.stack_objects().is_empty() {
        let (s, _) = pass_all_four(state, [p1, p2, p3, p4]);
        state = s;
    }

    // Step 3: Cast the entering creature (with 1 remaining mana).
    // Stack must be empty before casting a sorcery-speed spell (CR 307.1).
    let entering_hand_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Reg-Test Entering")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: entering_hand_id,
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
    .unwrap();

    // Step 4: All four pass → entering creature resolves.
    // Watcher's ETB trigger fires TWICE (doubled by the registered Panharmonicon).
    let (_state, resolution_events) = pass_all_four(state, [p1, p2, p3, p4]);

    let triggered_count = resolution_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();

    assert_eq!(
        triggered_count, 2,
        "CR 603.2d: Panharmonicon registered via resolution should double ETB trigger: \
         expected 2 AbilityTriggered events, got {}; events: {:?}",
        triggered_count, resolution_events
    );
}

// ── PB-M: SelfEntersBattlefield fix (Bug 1) ──────────────────────────────────

/// CR 603.2d + Panharmonicon ruling 2021-03-19 — Panharmonicon doubles a
/// creature's own "when this enters" self-ETB triggered ability.
///
/// Previously, `doubler_applies_to_trigger` only matched
/// `TriggerEvent::AnyPermanentEntersBattlefield`, so self-ETB triggers (using
/// `TriggerEvent::SelfEntersBattlefield`) were never doubled. PB-M fixes this by
/// also matching `SelfEntersBattlefield` in the `ArtifactOrCreatureETB` arm.
///
/// Setup: Panharmonicon + a creature with its own self-ETB trigger. When the
/// creature enters (via a second creature resolving), the self-ETB trigger must
/// fire twice.
#[test]
fn test_panharmonicon_doubles_self_etb_trigger() {
    let p1 = p1();
    let p2 = p2();
    let p3 = p3();
    let p4 = p4();

    let pharm_def = panharmonicon_def("pharm-self-etb", "Panharmonicon SelfETB Test");
    let pharm_card_id = pharm_def.card_id.clone();

    // Creature with its own self-ETB trigger (SelfEntersBattlefield).
    let self_etb_def = CardDefinition {
        card_id: CardId("self-etb-creature".into()),
        name: "Self-ETB Creature".into(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "When this creature enters, do something.".into(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    };
    let self_etb_card_id = self_etb_def.card_id.clone();

    let registry = CardRegistry::new(vec![pharm_def, self_etb_def]);

    // Panharmonicon on battlefield; creature with self-ETB trigger (SelfEntersBattlefield)
    // is in hand via ObjectSpec triggered_ability.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(
            ObjectSpec::artifact(p1, "Panharmonicon SelfETB Test")
                .with_card_id(pharm_card_id)
                .in_zone(ZoneId::Battlefield),
        )
        // Creature in hand with a self-ETB trigger (SelfEntersBattlefield event).
        .object(
            ObjectSpec::creature(p1, "Self-ETB Creature", 2, 2)
                .with_card_id(self_etb_card_id)
                .with_triggered_ability(TriggeredAbilityDef {
                    counter_filter: None,
                    counter_on_self: false,
                    once_per_turn: false,
                    trigger_on: TriggerEvent::SelfEntersBattlefield,
                    intervening_if: None,
                    description: "When this enters, do something (self-ETB test)".to_string(),
                    effect: None,
                    etb_filter: None,
                    death_filter: None,
                    combat_damage_filter: None,
                    triggering_creature_filter: None,
                    targets: vec![],
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;

    // Register Panharmonicon's TriggerDoubler manually (already on battlefield).
    let pharm_obj_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Panharmonicon SelfETB Test")
        .map(|(id, _)| *id)
        .unwrap();

    state.trigger_doublers_mut().push_back(TriggerDoubler {
        source: pharm_obj_id,
        controller: p1,
        filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
        additional_triggers: 1,
    });

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 2;

    let creature_hand_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Self-ETB Creature")
        .map(|(id, _)| *id)
        .unwrap();

    // Cast the creature.
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: creature_hand_id,
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
    .unwrap();

    // All four pass → creature resolves, self-ETB trigger fires.
    // Panharmonicon's ArtifactOrCreatureETB filter now also matches SelfEntersBattlefield.
    // Expect 2 AbilityTriggered events (1 base + 1 doubled).
    let (_state, resolution_events) = pass_all_four(state, [p1, p2, p3, p4]);

    let triggered_count = resolution_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();

    assert_eq!(
        triggered_count, 2,
        "CR 603.2d (PB-M Bug 1 fix): Panharmonicon must double self-ETB (SelfEntersBattlefield) \
         triggers; expected 2 AbilityTriggered events, got {}; events: {:?}",
        triggered_count, resolution_events
    );
}

// ── PB-M: Negative test — enchantment ETB not doubled by Panharmonicon ───────

/// CR 603.2d — Panharmonicon only doubles ETBs caused by artifacts or creatures.
/// An enchantment entering the battlefield must NOT cause doubling.
///
/// This is a negative test verifying the type check in `doubler_applies_to_trigger`.
#[test]
fn test_panharmonicon_does_not_double_enchantment_etb() {
    let p1 = p1();
    let p2 = p2();
    let p3 = p3();
    let p4 = p4();

    let pharm_def = panharmonicon_def("pharm-no-enchantment", "Panharmonicon No-Enchantment");
    let pharm_card_id = pharm_def.card_id.clone();

    // Enchantment card — not an artifact or creature.
    let enchantment_def = CardDefinition {
        card_id: CardId("test-enchantment-entering".into()),
        name: "Test Enchantment".into(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Enchantment].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "".into(),
        abilities: vec![],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    };
    let enchantment_card_id = enchantment_def.card_id.clone();

    let registry = CardRegistry::new(vec![pharm_def, enchantment_def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(
            ObjectSpec::artifact(p1, "Panharmonicon No-Enchantment")
                .with_card_id(pharm_card_id)
                .in_zone(ZoneId::Battlefield),
        )
        // Watcher with AnyPermanentEntersBattlefield trigger.
        .object(
            ObjectSpec::creature(p1, "Watcher Enchantment Test", 1, 1)
                .with_triggered_ability(any_etb_trigger("Watcher for enchantment ETB test"))
                .in_zone(ZoneId::Battlefield),
        )
        // Enchantment in hand (use enchantment spec so CardType::Enchantment is set).
        .object(
            ObjectSpec::enchantment(p1, "Test Enchantment")
                .with_card_id(enchantment_card_id)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;

    let pharm_obj_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Panharmonicon No-Enchantment")
        .map(|(id, _)| *id)
        .unwrap();

    state.trigger_doublers_mut().push_back(TriggerDoubler {
        source: pharm_obj_id,
        controller: p1,
        filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
        additional_triggers: 1,
    });

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 2;

    let enchantment_hand_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Test Enchantment")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: enchantment_hand_id,
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
    .unwrap();

    // Enchantment resolves. Watcher's AnyPermanentEntersBattlefield trigger fires ONCE only.
    // Panharmonicon must NOT double it (the entering permanent is an enchantment, not an
    // artifact or creature).
    let (_state, resolution_events) = pass_all_four(state, [p1, p2, p3, p4]);

    let triggered_count = resolution_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();

    assert_eq!(
        triggered_count, 1,
        "CR 603.2d: Panharmonicon must NOT double enchantment ETBs; \
         expected 1 AbilityTriggered event, got {}; events: {:?}",
        triggered_count, resolution_events
    );
}

// ── PB-M: AnyPermanentETB filter doubles enchantment ETBs ────────────────────

/// CR 603.2d — `AnyPermanentETB` filter (Yarok / Elesh Norn pattern) doubles
/// ETB triggers from any permanent entering, including enchantments — which
/// `ArtifactOrCreatureETB` does NOT double.
///
/// Verifies the new `AnyPermanentETB` variant added in PB-M.
#[test]
fn test_any_permanent_etb_doubler_doubles_enchantment() {
    let p1 = p1();
    let p2 = p2();
    let p3 = p3();
    let p4 = p4();

    // AnyPermanentETB doubler (Yarok / Elesh Norn pattern).
    let any_perm_doubler_def = CardDefinition {
        card_id: CardId("any-perm-doubler-test".into()),
        name: "Any-Perm Doubler".into(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "If a permanent entering causes a triggered ability to trigger, that ability triggers an additional time.".into(),
        abilities: vec![AbilityDefinition::TriggerDoubling {
            filter: TriggerDoublerFilter::AnyPermanentETB,
            additional_triggers: 1,
        }],
        power: Some(5),
        toughness: Some(4),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    };

    let enchantment_entering = CardDefinition {
        card_id: CardId("enchantment-for-any-perm-test".into()),
        name: "Enchantment For AnyPerm Test".into(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Enchantment].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "".into(),
        abilities: vec![],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    };
    let ench_card_id = enchantment_entering.card_id.clone();

    let registry = CardRegistry::new(vec![any_perm_doubler_def, enchantment_entering]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        // AnyPermanentETB doubler on battlefield.
        .object(ObjectSpec::creature(p1, "Any-Perm Doubler", 5, 4).in_zone(ZoneId::Battlefield))
        // Watcher with AnyPermanentEntersBattlefield trigger.
        .object(
            ObjectSpec::creature(p1, "Watcher AnyPerm", 1, 1)
                .with_triggered_ability(any_etb_trigger("Watcher for AnyPerm test"))
                .in_zone(ZoneId::Battlefield),
        )
        // Enchantment in hand (use enchantment spec so CardType::Enchantment is set).
        .object(
            ObjectSpec::enchantment(p1, "Enchantment For AnyPerm Test")
                .with_card_id(ench_card_id)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;

    let doubler_obj_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Any-Perm Doubler")
        .map(|(id, _)| *id)
        .unwrap();

    state.trigger_doublers_mut().push_back(TriggerDoubler {
        source: doubler_obj_id,
        controller: p1,
        filter: TriggerDoublerFilter::AnyPermanentETB,
        additional_triggers: 1,
    });

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 2;

    let ench_hand_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Enchantment For AnyPerm Test")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: ench_hand_id,
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
    .unwrap();

    // Enchantment resolves. AnyPermanentETB filter doubles the watcher's trigger.
    // Expect 2 AbilityTriggered events (enchantment ETB IS doubled by AnyPermanentETB).
    let (_state, resolution_events) = pass_all_four(state, [p1, p2, p3, p4]);

    let triggered_count = resolution_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();

    assert_eq!(
        triggered_count, 2,
        "CR 603.2d: AnyPermanentETB filter must double enchantment ETB triggers; \
         expected 2 AbilityTriggered events, got {}; events: {:?}",
        triggered_count, resolution_events
    );
}

// ── PB-M: LandETB filter doubles landfall triggers ───────────────────────────

/// CR 603.2d — `LandETB` filter (Ancient Greenwarden pattern) doubles ETB
/// triggers when a land enters but NOT when a creature enters.
///
/// Part 1: Land entering → 2 triggers (doubled).
/// Part 2 (negative): Creature entering → 1 trigger (NOT doubled).
///
/// Verifies the new `LandETB` variant added in PB-M.
#[test]
fn test_land_etb_doubler_doubles_landfall_not_creature() {
    let p1 = p1();
    let p2 = p2();
    let p3 = p3();
    let p4 = p4();

    // Land card with proper card_id for the registry (so enrich populates land type).
    let land_def = CardDefinition {
        card_id: CardId("test-basic-land-landetb".into()),
        name: "Test Basic Land LandETB".into(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Land].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "{T}: Add {G}.".into(),
        abilities: vec![],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    };
    let land_card_id = land_def.card_id.clone();

    let creature_def = CardDefinition {
        card_id: CardId("test-creature-landetb".into()),
        name: "Test Creature LandETB".into(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "".into(),
        abilities: vec![],
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    };
    let creature_card_id = creature_def.card_id.clone();

    let registry = CardRegistry::new(vec![land_def, creature_def]);

    // ── Part 1: Land entering → doubled ──────────────────────────────────────

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        // LandETB doubler (Ancient Greenwarden pattern) on battlefield.
        .object(ObjectSpec::creature(p1, "LandETB Doubler", 5, 7).in_zone(ZoneId::Battlefield))
        // Watcher with AnyPermanentEntersBattlefield trigger (watches all ETBs).
        .object(
            ObjectSpec::creature(p1, "LandETB Watcher", 1, 1)
                .with_triggered_ability(any_etb_trigger("LandETB watcher"))
                .in_zone(ZoneId::Battlefield),
        )
        // Land in hand.
        .object(
            ObjectSpec::land(p1, "Test Basic Land LandETB")
                .with_card_id(land_card_id)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry.clone())
        .build()
        .unwrap();

    let mut state = state;

    let doubler_obj_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "LandETB Doubler")
        .map(|(id, _)| *id)
        .unwrap();

    state.trigger_doublers_mut().push_back(TriggerDoubler {
        source: doubler_obj_id,
        controller: p1,
        filter: TriggerDoublerFilter::LandETB,
        additional_triggers: 1,
    });

    let land_hand_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Test Basic Land LandETB")
        .map(|(id, _)| *id)
        .unwrap();

    // Play the land. AbilityTriggered events are emitted immediately by check_triggers
    // (called inside Command::PlayLand handling), before priority is given out.
    let (state, play_events) = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: land_hand_id,
        },
    )
    .unwrap();

    // Drain remaining stack items (watcher trigger resolves).
    let (_state, _drain_events) = pass_all_four(state, [p1, p2, p3, p4]);

    // LandETB filter matches → watcher trigger doubles → expect 2 AbilityTriggered
    // events in the PlayLand response (triggers queued and emitted during land ETB).
    let triggered_count = play_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();

    assert_eq!(
        triggered_count, 2,
        "CR 603.2d: LandETB filter must double triggers when a land enters; \
         expected 2 AbilityTriggered events, got {}; events: {:?}",
        triggered_count, play_events
    );

    // ── Part 2: Creature entering → NOT doubled ───────────────────────────────

    // Fresh state for Part 2: same setup but creature enters instead of a land.
    let state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(ObjectSpec::creature(p1, "LandETB Doubler2", 5, 7).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::creature(p1, "LandETB Watcher2", 1, 1)
                .with_triggered_ability(any_etb_trigger("LandETB watcher 2"))
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p1, "Test Creature LandETB", 1, 1)
                .with_card_id(creature_card_id)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state2 = state2;

    let doubler2_obj_id = state2
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "LandETB Doubler2")
        .map(|(id, _)| *id)
        .unwrap();

    state2.trigger_doublers_mut().push_back(TriggerDoubler {
        source: doubler2_obj_id,
        controller: p1,
        filter: TriggerDoublerFilter::LandETB,
        additional_triggers: 1,
    });

    state2
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 1;

    let creature_hand_id = state2
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Test Creature LandETB")
        .map(|(id, _)| *id)
        .unwrap();

    let (state2, _) = process_command(
        state2,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: creature_hand_id,
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
    .unwrap();

    // Creature resolves. LandETB filter must NOT match → expect 1 AbilityTriggered only.
    let (_state2, resolution_events2) = pass_all_four(state2, [p1, p2, p3, p4]);

    let triggered_count2 = resolution_events2
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();

    assert_eq!(
        triggered_count2, 1,
        "CR 603.2d: LandETB filter must NOT double triggers when a creature enters; \
         expected 1 AbilityTriggered event, got {}; events: {:?}",
        triggered_count2, resolution_events2
    );
}

// ── PB-M Fix: CardDef ETB path — queue_carddef_etb_triggers doubling ──────────

/// CR 603.2d — Bug 2 regression: `queue_carddef_etb_triggers` sets
/// `entering_object_id: Some(new_id)` so `doubler_applies_to_trigger` can inspect
/// the entering permanent's card types and apply the doubler.
///
/// Unlike all other PB-M tests (which use `ObjectSpec::with_triggered_ability` and
/// go through `check_triggers`), this test uses a `CardDefinition` with
/// `AbilityDefinition::Triggered { trigger_condition: WhenEntersBattlefield, .. }`,
/// which routes through `queue_carddef_etb_triggers` — the code path fixed in Bug 2.
///
/// Setup:
/// - Panharmonicon TriggerDoubler registered for `ArtifactOrCreatureETB`.
/// - Creature with a CardDef-based ETB trigger (AbilityDefinition::Triggered).
/// - Creature resolves and enters the battlefield.
///
/// Expected: 2 `AbilityTriggered` events (1 base + 1 doubled by Panharmonicon).
#[test]
fn test_panharmonicon_doubles_carddef_etb_trigger() {
    let p1 = p1();
    let p2 = p2();
    let p3 = p3();
    let p4 = p4();

    let pharm_def = panharmonicon_def("pharm-carddef-etb", "Panharmonicon CardDef ETB");
    let pharm_card_id = pharm_def.card_id.clone();

    // Creature with a CardDef-based ETB trigger (NOT ObjectSpec runtime trigger).
    // This is the path exercised by queue_carddef_etb_triggers.
    let creature_with_carddef_etb = CardDefinition {
        card_id: CardId("carddef-etb-creature".into()),
        name: "CardDef ETB Creature".into(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "When this creature enters the battlefield, draw a card.".into(),
        // AbilityDefinition::Triggered routes through queue_carddef_etb_triggers
        // (not ObjectSpec runtime triggers / check_triggers).
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    };
    let creature_card_id = creature_with_carddef_etb.card_id.clone();

    let registry = CardRegistry::new(vec![pharm_def, creature_with_carddef_etb]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        // Panharmonicon already on battlefield.
        .object(
            ObjectSpec::artifact(p1, "Panharmonicon CardDef ETB")
                .with_card_id(pharm_card_id)
                .in_zone(ZoneId::Battlefield),
        )
        // Creature in hand — has a CardDef ETB trigger (AbilityDefinition::Triggered).
        .object(
            ObjectSpec::creature(p1, "CardDef ETB Creature", 2, 2)
                .with_card_id(creature_card_id)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Manually register Panharmonicon's TriggerDoubler (already on battlefield).
    let mut state = state;

    let pharm_obj_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Panharmonicon CardDef ETB")
        .map(|(id, _)| *id)
        .unwrap();

    state.trigger_doublers_mut().push_back(TriggerDoubler {
        source: pharm_obj_id,
        controller: p1,
        filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
        additional_triggers: 1,
    });

    // Give p1 enough mana to cast the creature (MV=2).
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 2;

    let creature_hand_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "CardDef ETB Creature")
        .map(|(id, _)| *id)
        .unwrap();

    // Cast the creature.
    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: creature_hand_id,
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
    .unwrap();

    // All four players pass priority → creature resolves and enters the battlefield via
    // queue_carddef_etb_triggers. The CardDef ETB trigger fires, and Panharmonicon's
    // ArtifactOrCreatureETB doubler (Bug 2 fix: entering_object_id: Some(new_id)) doubles it.
    // Expect 2 AbilityTriggered events (1 baseline + 1 doubled).
    let (_state, resolution_events) = pass_all_four(state, [p1, p2, p3, p4]);

    let triggered_count = resolution_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();

    assert_eq!(
        triggered_count, 2,
        "CR 603.2d (PB-M Bug 2 fix): Panharmonicon must double CardDef-based ETB triggers \
         routed through queue_carddef_etb_triggers; expected 2 AbilityTriggered events, \
         got {}; events: {:?}",
        triggered_count, resolution_events
    );
}

// ── PB-DX15a: the `CreatureDeath` doubler arm (rider `OOS-DX24-1`) ──────────────

/// A "when this dies" triggered ability (CR 603.6c / CR 700.4 / CR 603.10a).
fn dx15a_dies_trigger(description: &str) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        trigger_on: TriggerEvent::SelfDies,
        intervening_if: None,
        description: description.to_string(),
        effect: None,
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
    }
}

#[test]
/// CR 603.2d / CR 603.6c / CR 603.10a — a `TriggerDoublerFilter::CreatureDeath` doubler
/// (Teysa Karlov, Drivnod) **doubles a dying creature's own "when this dies" trigger**.
///
/// # Why this test exists — it is the evidence behind `OOS-DX24-1`'s DEFERRAL
///
/// This file had **nine** tests before PB-DX15a and **not one** of them exercised the
/// `CreatureDeath` arm of `doubler_applies_to_trigger` — every one is an ETB arm. So the
/// most common real interaction that arm exists to serve had zero coverage, and a change
/// to that function could break it with the whole workspace green.
///
/// `OOS-DX24-1` asks for exactly such a change. Its prescribed fix is *"one source-zone
/// conjunct at the top of `doubler_applies_to_trigger`, before the `match`, covering all
/// four arms at once"*, on the CR 110.1 reasoning that every printed doubler says "a
/// triggered ability of a **permanent** you control" and a card in a graveyard is not a
/// permanent.
///
/// **That conjunct, written as prescribed, breaks this test.** A CR 603.6c/603.10a
/// look-back dies trigger is constructed as
/// `PendingTrigger::blank(*new_grave_id, *death_controller, kind)` — the dying creature's
/// **graveyard** object is the trigger's `source`, because the battlefield object is
/// already gone by trigger-check time (`move_object_to_zone` removes it from
/// `state.objects`). So at doubling time BOTH the legitimate case (this test: Teysa
/// doubling a dies trigger, which CR 603.10a says WAS a permanent's ability) and the
/// illegitimate case the seed describes (Nether Traitor's `trigger_zone: Graveyard`
/// ability, which was never a permanent's) present a source sitting in a graveyard.
/// **Zone alone cannot separate them.**
///
/// The correct discriminator is *why* the source is in the graveyard — whether it
/// arrived there as part of this very event — and that information exists only in
/// `check_triggers`' `arrived_in_graveyard_this_batch` set, which is not in scope at
/// `compute_trigger_doubling`'s call site, or in a construction-time marker on
/// `PendingTrigger`, which is a hashed, serialized type and therefore a HASH/PROTOCOL
/// bump this PB cannot take (one wire bump per PB, and this batch predicted and measured
/// NONE). Full reasoning: `memory/primitives/pb-DX15a-execution-notes.md` §4.4.
///
/// So the rider is deferred, and this test is what makes the deferral checkable: whoever
/// takes `OOS-DX24-1` must keep this green.
fn test_dx15a_creature_death_doubler_doubles_a_look_back_dies_trigger() {
    let p1 = p1();
    let p2 = p2();
    let p3 = p3();
    let p4 = p4();

    // A 2/2 with 2 damage marked (lethal, CR 704.5g) carrying its own dies trigger.
    let dying = ObjectSpec::creature(p1, "Dying Bear", 2, 2)
        .with_damage(2)
        .with_triggered_ability(dx15a_dies_trigger(
            "When Dying Bear dies, do something (CR 700.4)",
        ))
        .in_zone(ZoneId::Battlefield);

    // The doubler's own body. It needs to be a battlefield permanent, because
    // `doubler_applies_to_trigger`'s FIRST guard is the DOUBLER's battlefield presence.
    let doubler_body = ObjectSpec::creature(p1, "Death Doubler", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(dying)
        .object(doubler_body)
        .build()
        .unwrap();

    let mut state = state;
    let doubler_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Death Doubler")
        .map(|(id, _)| *id)
        .expect("doubler body should be on the battlefield");
    state.trigger_doublers_mut().push_back(TriggerDoubler {
        source: doubler_id,
        controller: p1,
        filter: TriggerDoublerFilter::CreatureDeath,
        additional_triggers: 1,
    });

    // Pass priority around the table so SBAs run and the lethal-damage creature dies.
    let mut state = state;
    for player in [p1, p2, p3, p4] {
        let (s, _) = process_command(state, Command::PassPriority { player }).unwrap();
        state = s;
    }

    // CR 603.2d: base 1 + 1 additional = the dies trigger goes on the stack TWICE.
    let dies_triggers_on_stack = state
        .stack_objects()
        .iter()
        .filter(|so| matches!(so.kind, StackObjectKind::TriggeredAbility { .. }))
        .count();
    assert_eq!(
        dies_triggers_on_stack, 2,
        "CR 603.2d: a CreatureDeath doubler must double the dying creature's own \
         look-back 'when this dies' trigger (CR 603.10a). Getting 1 here means the \
         doubler stopped seeing it -- which is exactly what OOS-DX24-1's prescribed \
         source-zone conjunct does, because a look-back dies trigger's source IS a \
         graveyard object."
    );
}

#[test]
/// CR 110.1 / CR 603.2d / CR 113.6m — a `trigger_zone: Graveyard` ability is **not** a
/// permanent's ability, so a `CreatureDeath` doubler must **not** double it.
///
/// This is the defect `OOS-DX24-1` was filed for (Nether Traitor in your graveyard,
/// Teysa Karlov on your battlefield, another creature dies → **two** `{B}` offers instead
/// of one), and it is the negative half of
/// [`test_dx15a_creature_death_doubler_doubles_a_look_back_dies_trigger`].
///
/// **The pair is the point.** Both trigger sources sit in a graveyard at doubling time,
/// so a source-ZONE conjunct — which is what the row prescribed — cannot separate them
/// and gets this one right only by getting the other one wrong. The shipped discriminator
/// is the triggering EVENT: `SelfDies`/`SelfEntersBattlefield` are CR 603.6c/603.10a
/// look-backs on something that *was* a permanent, while `AnyCreatureDies` /
/// `AnyPermanentEntersBattlefield` on a graveyard source can only have come from
/// `collect_graveyard_carddef_triggers`, i.e. a card that never was one.
///
/// Either test alone is satisfiable by a wrong implementation. Together they are not.
fn test_dx15a_creature_death_doubler_does_not_double_a_graveyard_zone_ability() {
    let p1 = p1();
    let p2 = p2();
    let p3 = p3();
    let p4 = p4();

    // A plain 2/2 with lethal damage marked — it dies to SBAs and carries no ability of
    // its own, so every trigger observed below belongs to the graveyard watcher.
    let dying = ObjectSpec::creature(p1, "Ordinary Bear", 2, 2)
        .with_damage(2)
        .in_zone(ZoneId::Battlefield);

    // The graveyard-functioning watcher: Nether Traitor's shape. Its ability watches
    // creature deaths and functions FROM the graveyard, so it was never a permanent's
    // ability and CR 110.1 puts it outside every printed doubler's scope.
    let watcher_def = CardDefinition {
        card_id: CardId("dx15a-graveyard-death-watcher".into()),
        name: "Graveyard Death Watcher".into(),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "Whenever another creature is put into your graveyard from the battlefield, \
                      draw a card."
            .into(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverCreatureDies {
                filter: None,
                controller: None,
                owner: None,
                exclude_self: true,
                nontoken_only: false,
            },
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: Some(mtg_engine::cards::card_definition::TriggerZone::Graveyard),
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    };
    let watcher_card_id = watcher_def.card_id.clone();

    let doubler_body = ObjectSpec::creature(p1, "Death Doubler", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![watcher_def]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(dying)
        .object(doubler_body)
        .object(
            ObjectSpec::creature(p1, "Graveyard Death Watcher", 1, 1)
                .with_card_id(watcher_card_id)
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .build()
        .unwrap();

    let mut state = state;
    let doubler_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Death Doubler")
        .map(|(id, _)| *id)
        .expect("doubler body should be on the battlefield");
    state.trigger_doublers_mut().push_back(TriggerDoubler {
        source: doubler_id,
        controller: p1,
        filter: TriggerDoublerFilter::CreatureDeath,
        additional_triggers: 1,
    });

    for player in [p1, p2, p3, p4] {
        let (s, _) = process_command(state, Command::PassPriority { player }).unwrap();
        state = s;
    }

    let triggers_on_stack = state
        .stack_objects()
        .iter()
        .filter(|so| matches!(so.kind, StackObjectKind::TriggeredAbility { .. }))
        .count();

    // Non-vacuity first: the watcher's ability must actually have fired, or "not doubled"
    // would be satisfied by an ability that never triggers at all.
    assert_eq!(
        triggers_on_stack, 1,
        "CR 110.1 / CR 603.2d: the graveyard-zone ability triggers ONCE and is NOT \
         doubled -- it is not a permanent's ability. Getting 2 here is OOS-DX24-1's \
         filed defect; getting 0 means the ability did not fire at all and this test is \
         vacuous rather than passing."
    );
}
