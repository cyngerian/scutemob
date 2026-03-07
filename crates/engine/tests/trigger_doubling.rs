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

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameStateBuilder, ManaCost, ObjectSpec, PlayerId, StackObjectKind, Step,
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
    }
}

/// Build a triggered ability: "Whenever any permanent enters the battlefield..."
fn any_etb_trigger(description: &str) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
        intervening_if: None,
        description: description.to_string(),
        effect: None,
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
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Test Panharmonicon")
        .map(|(id, _)| *id)
        .unwrap();

    state
        .trigger_doublers
        .push_back(mtg_engine::TriggerDoubler {
            source: panharmonicon_obj_id,
            controller: p1,
            filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
            additional_triggers: 1,
        });

    // Give p1 enough mana to cast the entering creature (MV=2).
    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 2;

    let entering_hand_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Entering Creature")
        .map(|(id, _)| *id)
        .unwrap();

    // p1 casts the entering creature.
    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: entering_hand_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
        },
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
        .stack_objects
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
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Panharmonicon Alpha")
        .map(|(id, _)| *id)
        .unwrap();
    let beta_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Panharmonicon Beta")
        .map(|(id, _)| *id)
        .unwrap();

    state
        .trigger_doublers
        .push_back(mtg_engine::TriggerDoubler {
            source: alpha_id,
            controller: p1,
            filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
            additional_triggers: 1,
        });
    state
        .trigger_doublers
        .push_back(mtg_engine::TriggerDoubler {
            source: beta_id,
            controller: p1,
            filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
            additional_triggers: 1,
        });

    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 1;

    let entering_hand_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Entering Creature 2")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: entering_hand_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
        },
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
        .stack_objects
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
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Panharmonicon Removal Test")
        .map(|(id, _)| *id)
        .unwrap();

    state
        .trigger_doublers
        .push_back(mtg_engine::TriggerDoubler {
            source: panharmonicon_obj_id,
            controller: p1,
            filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
            additional_triggers: 1,
        });

    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 1;

    let entering_hand_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Entering Creature 3")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: entering_hand_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
        },
    )
    .unwrap();

    // All four players pass → creature resolves, enters battlefield, triggers fire (2x).
    let (state, _resolution_events) = pass_all_four(state, [p1, p2, p3, p4]);

    // Verify the stack has 2 triggered abilities from Panharmonicon doubling.
    let trigger_stack_count_before = state
        .stack_objects
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
        .trigger_doublers
        .retain(|d| d.source != panharmonicon_obj_id);

    // The stack should STILL have 2 triggered abilities — they don't disappear when
    // Panharmonicon is removed. Triggers already on the stack are independent.
    let trigger_stack_count_after = state
        .stack_objects
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
/// Previous tests inject TriggerDoubler directly into state.trigger_doublers.
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
    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 5;

    let pharm_hand_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Reg-Test Panharmonicon")
        .map(|(id, _)| *id)
        .unwrap();

    // Step 1: Cast Panharmonicon.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: pharm_hand_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
        },
    )
    .unwrap();

    // Step 2: All four pass → Panharmonicon resolves and enters the battlefield.
    // After resolution, register_static_continuous_effects fires and registers the
    // TriggerDoubler from Panharmonicon's AbilityDefinition::TriggerDoubling entry.
    // The Watcher's ETB trigger (from Panharmonicon entering) also goes onto the stack.
    let (state, _pharm_resolution_events) = pass_all_four(state, [p1, p2, p3, p4]);

    // The TriggerDoubler must now be registered (no manual injection).
    assert_eq!(
        state.trigger_doublers.len(),
        1,
        "After Panharmonicon resolves via casting, trigger_doublers should have 1 entry \
         (registered by register_static_continuous_effects); got {}",
        state.trigger_doublers.len()
    );

    // Step 2b: Drain all Watcher triggers from Panharmonicon's own ETB off the stack.
    // Panharmonicon is an artifact, so its own entry may trigger (and double) the Watcher.
    // One call to pass_all_four resolves exactly one stack item; loop until empty.
    let mut state = state;
    while !state.stack_objects.is_empty() {
        let (s, _) = pass_all_four(state, [p1, p2, p3, p4]);
        state = s;
    }

    // Step 3: Cast the entering creature (with 1 remaining mana).
    // Stack must be empty before casting a sorcery-speed spell (CR 307.1).
    let entering_hand_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Reg-Test Entering")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: entering_hand_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
        },
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
