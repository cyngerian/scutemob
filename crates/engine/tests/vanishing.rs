//! Vanishing keyword ability tests (CR 702.63).
//!
//! Vanishing N represents three abilities (CR 702.63a):
//! 1. Replacement (ETB): enters with N time counters.
//! 2. Triggered (upkeep): at beginning of controller's upkeep, if this permanent
//!    has a time counter, remove a time counter from it. Intervening-if (CR 603.4).
//! 3. Triggered (last counter): when the last time counter is removed, sacrifice it.
//!
//! CR 702.63b: Vanishing without a number (Vanishing(0)) has no ETB counter placement.
//! CR 702.63c: Multiple instances work separately (each triggers independently).
//!
//! Test setup note: Tests that verify the upkeep trigger start at Step::Untap with
//! priority manually set. Passing priority for all players at Untap causes the engine
//! to advance to Upkeep via handle_all_passed -> enter_step, which calls upkeep_actions
//! and queues the VanishingCounterTrigger. This avoids the need to pass through Draw
//! (which would fail with an empty library).

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaCost, ObjectId,
    ObjectSpec, PlayerId, StackObjectKind, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
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
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == zone {
            Some(id)
        } else {
            None
        }
    })
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Graveyard(owner)).is_some()
}

/// Count time counters on the named object (wherever it is).
fn time_counters(state: &GameState, name: &str) -> u32 {
    state
        .objects
        .values()
        .find(|o| o.characteristics.name == name)
        .and_then(|o| o.counters.get(&CounterType::Time).copied())
        .unwrap_or(0)
}

/// Pass priority for all listed players once.
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

// ── Card definitions ───────────────────────────────────────────────────────────

/// Test Vanishing Creature: Creature {2}{U}. Vanishing 3. 2/2.
fn vanishing_3_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-vanishing-3".into()),
        name: "Test Vanishing 3 Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Vanishing 3".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vanishing(3)),
            AbilityDefinition::Vanishing { count: 3 },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Test Vanishing Creature: Creature {2}{U}. Vanishing 2. 2/2.
fn vanishing_2_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-vanishing-2".into()),
        name: "Test Vanishing 2 Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Vanishing 2".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vanishing(2)),
            AbilityDefinition::Vanishing { count: 2 },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Test Vanishing Creature: Creature {2}{U}. Vanishing (no number). 2/2.
/// Represented as Vanishing(0) per CR 702.63b.
fn vanishing_no_number_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-vanishing-0".into()),
        name: "Test Vanishing No Number".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Vanishing".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vanishing(0)),
            AbilityDefinition::Vanishing { count: 0 },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// ObjectSpec for Vanishing 3 creature on the battlefield with 3 time counters.
fn vanishing_3_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Vanishing 3 Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("test-vanishing-3".into()))
        .with_keyword(KeywordAbility::Vanishing(3))
        .with_types(vec![CardType::Creature])
}

/// ObjectSpec for Vanishing 2 creature on the battlefield.
fn vanishing_2_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Vanishing 2 Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("test-vanishing-2".into()))
        .with_keyword(KeywordAbility::Vanishing(2))
        .with_types(vec![CardType::Creature])
}

/// ObjectSpec for Vanishing (no number) creature on the battlefield.
fn vanishing_no_number_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Vanishing No Number")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("test-vanishing-0".into()))
        .with_keyword(KeywordAbility::Vanishing(0))
        .with_types(vec![CardType::Creature])
}

// ── Test 1: ETB places time counters ─────────────────────────────────────────

#[test]
/// CR 702.63a — Permanent with Vanishing 3 enters the battlefield with 3 time counters.
fn test_vanishing_etb_places_time_counters() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![vanishing_3_creature_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(vanishing_3_on_battlefield(p1))
        .build()
        .unwrap();

    let obj_id = find_in_zone(&state, "Test Vanishing 3 Creature", ZoneId::Battlefield)
        .expect("creature should be on battlefield");

    // When built with ObjectSpec, counters are not automatically set.
    // The ETB placement in resolution.rs fires when permanents resolve from the stack.
    // For direct-placement tests, we verify the keyword is present correctly.
    // The actual ETB counter placement is tested via the full cast flow.
    assert!(
        state.objects[&obj_id]
            .characteristics
            .keywords
            .contains(&KeywordAbility::Vanishing(3)),
        "CR 702.63a: permanent should have Vanishing(3) keyword"
    );
}

/// CR 702.63a — When a Vanishing 3 spell resolves, it enters with exactly 3 time counters.
/// Uses CastSpell flow to exercise the ETB counter-placement code in resolution.rs.
#[test]
fn test_vanishing_etb_counters_on_cast() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![vanishing_3_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Test Vanishing 3 Creature")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(CardId("test-vanishing-3".into()))
                .with_keyword(KeywordAbility::Vanishing(3))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    blue: 1,
                    ..Default::default()
                }),
        )
        .build()
        .unwrap();

    // Give p1 enough mana to cast.
    use mtg_engine::ManaColor;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Test Vanishing 3 Creature");

    // Cast the spell.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            kicker_times: 0,
            alt_cost: None,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
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
    .expect("CastSpell should succeed");

    // Resolve the spell: both players pass priority.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After resolution: permanent on battlefield with 3 time counters.
    assert!(
        on_battlefield(&state, "Test Vanishing 3 Creature"),
        "CR 702.63a: creature should be on battlefield after resolving"
    );
    assert_eq!(
        time_counters(&state, "Test Vanishing 3 Creature"),
        3,
        "CR 702.63a: Vanishing 3 creature should enter with exactly 3 time counters"
    );
}

// ── Test 2: Upkeep counter-removal trigger ────────────────────────────────────

#[test]
/// CR 702.63a (second ability) — At beginning of controller's upkeep, if this permanent
/// has a time counter, remove a time counter from it. After resolving: 3 counters -> 2.
///
/// Test setup: Start at Step::Untap with priority manually set. Passing priority for all
/// players at Untap advances to Upkeep via enter_step, which calls upkeep_actions and
/// queues the VanishingCounterTrigger.
fn test_vanishing_upkeep_removes_counter() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![vanishing_3_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(vanishing_3_on_battlefield(p1))
        .build()
        .unwrap();

    // Manually place 3 time counters.
    let obj_id = find_object(&state, "Test Vanishing 3 Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.counters = obj.counters.update(CounterType::Time, 3);
    }
    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep (enter_step queues VanishingCounterTrigger).
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::Upkeep,
        "should be at Upkeep after advancing from Untap"
    );

    // Both players pass -> trigger resolves -> 1 counter removed.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert_eq!(
        time_counters(&state, "Test Vanishing 3 Creature"),
        2,
        "CR 702.63a: one time counter should be removed, leaving 2"
    );
    assert!(
        on_battlefield(&state, "Test Vanishing 3 Creature"),
        "creature should still be on battlefield with 2 counters"
    );

    // CounterRemoved event should have been emitted.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CounterRemoved {
                counter: CounterType::Time,
                count: 1,
                ..
            }
        )),
        "CR 702.63a: CounterRemoved event should be emitted"
    );
}

// ── Test 3: VanishingSacrificeTrigger on last counter ─────────────────────────

#[test]
/// CR 702.63a (third ability) — When the last time counter is removed from this
/// permanent, sacrifice it. With 1 time counter, after upkeep: creature sacrificed.
fn test_vanishing_sacrifice_on_last_counter() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![vanishing_3_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(vanishing_3_on_battlefield(p1))
        .build()
        .unwrap();

    // Place exactly 1 time counter (last one).
    let obj_id = find_object(&state, "Test Vanishing 3 Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.counters = obj.counters.update(CounterType::Time, 1);
    }
    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep (queues VanishingCounterTrigger).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Resolve VanishingCounterTrigger -> removes last counter -> queues VanishingSacrificeTrigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After counter removal: 0 counters, VanishingSacrificeTrigger on stack.
    assert_eq!(
        time_counters(&state, "Test Vanishing 3 Creature"),
        0,
        "CR 702.63a: last counter removed, should have 0 counters"
    );
    assert!(
        state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::VanishingSacrificeTrigger { .. })),
        "CR 702.63a: VanishingSacrificeTrigger should be on the stack"
    );

    // Resolve the sacrifice trigger -> creature sacrificed.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Test Vanishing 3 Creature"),
        "CR 702.63a: creature should no longer be on battlefield after sacrifice"
    );
    assert!(
        in_graveyard(&state, "Test Vanishing 3 Creature", p1),
        "CR 702.63a: sacrificed creature should be in owner's graveyard"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.63a: CreatureDied event should be emitted on sacrifice"
    );
}

// ── Test 4: Full lifecycle over multiple upkeeps ──────────────────────────────

#[test]
/// CR 702.63a — Vanishing 2 creature: enters with 2 counters, ticks down over 2 upkeeps,
/// sacrificed on the upkeep when last counter is removed.
fn test_vanishing_full_lifecycle() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![vanishing_2_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(vanishing_2_on_battlefield(p1))
        .build()
        .unwrap();

    // Place 2 time counters (simulating ETB placement).
    let obj_id = find_object(&state, "Test Vanishing 2 Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.counters = obj.counters.update(CounterType::Time, 2);
    }
    state.turn.priority_holder = Some(p1);

    // === Upkeep 1: advance to Upkeep, counter trigger fires ===
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    let (state, _) = pass_all(state, &[p1, p2]);

    // After upkeep 1: 1 counter remaining, still on battlefield.
    assert_eq!(
        time_counters(&state, "Test Vanishing 2 Creature"),
        1,
        "CR 702.63a: should have 1 counter after first upkeep"
    );
    assert!(
        on_battlefield(&state, "Test Vanishing 2 Creature"),
        "CR 702.63a: should still be on battlefield after first upkeep"
    );
    // No sacrifice trigger yet.
    assert!(
        !state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::VanishingSacrificeTrigger { .. })),
        "No sacrifice trigger yet -- creature has 1 counter remaining"
    );

    // Advance to next upkeep: we need to get back to Untap -> Upkeep.
    // The state is now at Upkeep. We'll skip to PreCombatMain and advance turns.
    // Since advancing through draw step requires empty library handling, manually
    // set the active player's turn to p1 and step to Untap.
    let mut state = state;
    state.turn.step = Step::Untap;
    state.turn.priority_holder = Some(p1);

    // === Upkeep 2: advance to Upkeep, last counter trigger fires ===
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Resolve VanishingCounterTrigger -> removes last counter -> queues VanishingSacrificeTrigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        time_counters(&state, "Test Vanishing 2 Creature"),
        0,
        "CR 702.63a: no counters after second upkeep"
    );

    // Resolve VanishingSacrificeTrigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Test Vanishing 2 Creature"),
        "CR 702.63a: creature sacrificed after last counter removed in second upkeep"
    );
    assert!(
        in_graveyard(&state, "Test Vanishing 2 Creature", p1),
        "CR 702.63a: sacrificed creature should be in owner's graveyard"
    );
}

// ── Test 5: Vanishing without number does not place ETB counters ──────────────

#[test]
/// CR 702.63b — Vanishing without a number (Vanishing(0)) does not place time counters
/// at ETB. The upkeep/sacrifice triggers only fire if counters are placed externally.
fn test_vanishing_without_number_no_etb_counters() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![vanishing_no_number_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(vanishing_no_number_on_battlefield(p1))
        .build()
        .unwrap();

    // No counters placed externally.
    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // CR 702.63b: No upkeep trigger should fire (intervening-if: no time counter present).
    assert!(
        !state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::VanishingCounterTrigger { .. })),
        "CR 702.63b: No VanishingCounterTrigger without time counters"
    );

    // Permanent still on battlefield.
    assert!(
        on_battlefield(&state, "Test Vanishing No Number"),
        "CR 702.63b: creature should remain on battlefield (no counters to remove)"
    );
    assert_eq!(
        time_counters(&state, "Test Vanishing No Number"),
        0,
        "CR 702.63b: no time counters should have been placed"
    );
}

// ── Test 6: Multiplayer — only active player's permanents tick down ───────────

#[test]
/// CR 702.63a "your upkeep" — In multiplayer, Vanishing only ticks down on the
/// controlling player's upkeep, not on other players' upkeeps.
fn test_vanishing_multiplayer_only_active_player() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![vanishing_3_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p2) // p2 is active, p1 controls the creature
        .at_step(Step::Untap)
        .object(vanishing_3_on_battlefield(p1)) // p1's creature
        .build()
        .unwrap();

    // Place 3 time counters on p1's creature.
    let obj_id = find_object(&state, "Test Vanishing 3 Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.counters = obj.counters.update(CounterType::Time, 3);
    }
    state.turn.priority_holder = Some(p2);

    // Advance p2's Untap -> p2's Upkeep.
    let (state, _) = pass_all(state, &[p2, p1]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // No VanishingCounterTrigger for p1's permanent during p2's upkeep.
    assert!(
        !state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::VanishingCounterTrigger { .. })),
        "CR 702.63a: VanishingCounterTrigger must not fire on a non-controller's upkeep"
    );
    assert_eq!(
        time_counters(&state, "Test Vanishing 3 Creature"),
        3,
        "CR 702.63a: time counters must not be removed on non-controller's upkeep"
    );
}

// ── Test 7: Multiple instances trigger separately ─────────────────────────────

#[test]
/// CR 702.63c — If a permanent has two instances of Vanishing, each works separately:
/// two counter-removal triggers fire each upkeep (two counters removed per upkeep).
fn test_vanishing_multiple_instances() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![vanishing_3_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(
            // Two Vanishing instances: Vanishing(3) and Vanishing(1) so they
            // are distinct values in im::OrdSet (which deduplicates equal values).
            ObjectSpec::card(p1, "Test Vanishing 3 Creature")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("test-vanishing-3".into()))
                .with_keyword(KeywordAbility::Vanishing(3))
                .with_keyword(KeywordAbility::Vanishing(1))
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    // Place 4 time counters.
    let obj_id = find_object(&state, "Test Vanishing 3 Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.counters = obj.counters.update(CounterType::Time, 4);
    }
    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep -> two VanishingCounterTriggers queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Two triggers on the stack (one per Vanishing instance).
    let vanishing_counter_triggers = state
        .stack_objects
        .iter()
        .filter(|so| matches!(&so.kind, StackObjectKind::VanishingCounterTrigger { .. }))
        .count();
    assert_eq!(
        vanishing_counter_triggers, 2,
        "CR 702.63c: two Vanishing instances should produce two counter-removal triggers"
    );

    // Resolve first trigger: 4 -> 3 counters.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        time_counters(&state, "Test Vanishing 3 Creature"),
        3,
        "CR 702.63c: first trigger removes one counter (4 -> 3)"
    );

    // Resolve second trigger: 3 -> 2 counters.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        time_counters(&state, "Test Vanishing 3 Creature"),
        2,
        "CR 702.63c: second trigger removes another counter (3 -> 2)"
    );
}
