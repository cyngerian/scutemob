//! Proliferate keyword action tests (CR 701.34).
//!
//! Proliferate is a keyword action (like Scry and Surveil), not a keyword ability.
//! Cards say "Proliferate" as part of a spell or ability effect.
//!
//! Key rules verified:
//! - Proliferate adds one counter of each kind to each battlefield permanent that
//!   already has at least one counter (CR 701.34a).
//! - Proliferate adds one poison counter to each player that already has at least
//!   one poison counter (CR 701.34a).
//! - Permanents with no counters are NOT affected (CR 701.34a).
//! - Players with 0 poison counters are NOT affected (CR 701.34a).
//! - Permanents in non-battlefield zones are NOT affected (ruling 2023-02-04).
//! - Multiple counter types on one permanent are all proliferated (ruling 2023-02-04).
//! - GameEvent::Proliferated is always emitted, even with 0 eligible targets
//!   (ruling 2023-02-04: "triggers even if you chose no permanents or players").
//! - "Whenever you proliferate" triggers fire via TriggerEvent::ControllerProliferates.
//! - Multiplayer: all eligible permanents and players (including opponents) are affected.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, Effect, GameEvent, GameStateBuilder, ManaColor, ManaCost, ObjectId, ObjectSpec,
    PlayerId, Step, TriggerEvent, TriggeredAbilityDef, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Find an object by name in the game state (panics if not found).
fn find_by_name(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Execute Effect::Proliferate for a given player directly (no spell resolution).
///
/// Returns (state, events). Uses execute_effect directly.
fn run_proliferate(
    mut state: mtg_engine::GameState,
    controller: PlayerId,
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let effect = Effect::Proliferate;
    // ObjectId(0) is a dummy source. Proliferate does not reference ctx.source --
    // there is no "source object" concept in the CR 701.34a keyword action. If future
    // code adds source-based validation (e.g., replacement effects keyed on the source),
    // update this helper to use a real ObjectId from the game state.
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(controller, source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);
    (state, events)
}

/// Pass priority for all listed players once.
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

/// Build a "Proliferate" sorcery card definition.
fn proliferate_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("proliferate-spell".to_string()),
        name: "Proliferate Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Proliferate.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Proliferate,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// ── Test 1: Basic proliferate — +1/+1 counters increase ──────────────────────

#[test]
/// CR 701.34a — A permanent with +1/+1 counters gains 1 more after proliferate.
/// The CounterAdded event must be emitted and the Proliferated event must fire.
fn test_proliferate_basic_plus_one_counter() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Spike Feeder", 1, 1)
                .with_counter(CounterType::PlusOnePlusOne, 2)
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_proliferate(state, p1);

    // CR 701.34a: Spike Feeder should now have 3 +1/+1 counters (2 + 1).
    let spike = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Spike Feeder")
        .expect("Spike Feeder must be in state");

    let counter_count = spike
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);

    assert_eq!(
        counter_count, 3,
        "Spike Feeder must have 3 +1/+1 counters after proliferate; got {}",
        counter_count
    );

    // CounterAdded event must be emitted.
    let counter_added = events.iter().find(|e| {
        matches!(e, GameEvent::CounterAdded { counter, count, .. }
            if *counter == CounterType::PlusOnePlusOne && *count == 1)
    });
    assert!(
        counter_added.is_some(),
        "CounterAdded event must be emitted for +1/+1 counter"
    );

    // Proliferated event must be emitted.
    let proliferated = events.iter().find(|e| {
        matches!(e, GameEvent::Proliferated { controller, permanents_affected, .. }
            if *controller == p1 && *permanents_affected == 1)
    });
    assert!(
        proliferated.is_some(),
        "Proliferated event must be emitted with controller=p1, permanents_affected=1"
    );
}

// ── Test 2: Proliferate on -1/-1 counter ─────────────────────────────────────

#[test]
/// CR 701.34a — A permanent with -1/-1 counters gains 1 more after proliferate.
fn test_proliferate_minus_one_counter() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p2, "Hapless Zombie", 2, 2)
                .with_counter(CounterType::MinusOneMinusOne, 1)
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_proliferate(state, p1);

    // CR 701.34a: Hapless Zombie should now have 2 -1/-1 counters.
    let zombie = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Hapless Zombie")
        .expect("Hapless Zombie must be in state");

    let counter_count = zombie
        .counters
        .get(&CounterType::MinusOneMinusOne)
        .copied()
        .unwrap_or(0);

    assert_eq!(
        counter_count, 2,
        "Hapless Zombie must have 2 -1/-1 counters after proliferate; got {}",
        counter_count
    );

    // CounterAdded event must be emitted.
    let counter_added = events.iter().find(|e| {
        matches!(e, GameEvent::CounterAdded { counter, .. }
            if *counter == CounterType::MinusOneMinusOne)
    });
    assert!(
        counter_added.is_some(),
        "CounterAdded event must be emitted for -1/-1 counter"
    );
}

// ── Test 3: Proliferate charge counter on artifact ───────────────────────────

#[test]
/// CR 701.34a — An artifact with a charge counter gains 1 more after proliferate.
fn test_proliferate_charge_counter_on_artifact() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Contagion Clasp")
                .with_types(vec![CardType::Artifact])
                .with_counter(CounterType::Charge, 1)
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, _events) = run_proliferate(state, p1);

    // CR 701.34a: Contagion Clasp should now have 2 charge counters.
    let clasp = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Contagion Clasp")
        .expect("Contagion Clasp must be in state");

    let counter_count = clasp
        .counters
        .get(&CounterType::Charge)
        .copied()
        .unwrap_or(0);

    assert_eq!(
        counter_count, 2,
        "Contagion Clasp must have 2 charge counters after proliferate; got {}",
        counter_count
    );
}

// ── Test 3b: Proliferate loyalty counter on planeswalker ─────────────────────

#[test]
/// CR 701.34a + CR 122.1e — A planeswalker with loyalty counters receives 1 more
/// loyalty counter after proliferate. This is the most common real-world proliferate
/// interaction (Atraxa superfriends archetype). Verifies CounterType::Loyalty
/// is correctly incremented through the generic counter iteration path.
fn test_proliferate_loyalty_counter_on_planeswalker() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        // Planeswalker with 3 loyalty counters. Using the counter-based path
        // (CounterType::Loyalty) matching actual gameplay tracking (M7+ engine).
        .object(
            ObjectSpec::planeswalker(p1, "Sorin Markov", 3)
                .with_counter(CounterType::Loyalty, 3)
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let sorin_id = find_by_name(&state, "Sorin Markov");
    let (state, events) = run_proliferate(state, p1);

    // CR 701.34a + CR 122.1e: Sorin should now have 4 loyalty counters (3 + 1).
    let sorin = state
        .objects
        .get(&sorin_id)
        .expect("Sorin Markov must still be on battlefield");

    let loyalty_count = sorin
        .counters
        .get(&CounterType::Loyalty)
        .copied()
        .unwrap_or(0);

    assert_eq!(
        loyalty_count, 4,
        "Sorin Markov must have 4 loyalty counters after proliferate (was 3); got {}",
        loyalty_count
    );

    // CounterAdded event must be emitted for the loyalty counter.
    let counter_added = events.iter().find(|e| {
        matches!(e, GameEvent::CounterAdded {
            object_id,
            counter,
            count,
        } if *object_id == sorin_id && *counter == CounterType::Loyalty && *count == 1)
    });
    assert!(
        counter_added.is_some(),
        "CounterAdded event must be emitted for Sorin's loyalty counter (object_id={:?})",
        sorin_id
    );

    // Proliferated event must show 1 permanent affected.
    let proliferated = events.iter().find(|e| {
        matches!(e, GameEvent::Proliferated { controller, permanents_affected, .. }
            if *controller == p1 && *permanents_affected == 1)
    });
    assert!(
        proliferated.is_some(),
        "Proliferated event must show controller=p1, permanents_affected=1"
    );
}

// ── Test 4: Proliferate on player with poison counters ───────────────────────

#[test]
/// CR 701.34a — A player with poison counters receives 1 more after proliferate.
/// PoisonCountersGiven event must be emitted.
fn test_proliferate_poison_counter_on_player() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_poison(p2, 5)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_proliferate(state, p1);

    // CR 701.34a: p2 should now have 6 poison counters.
    let p2_state = state.players.get(&p2).expect("p2 must exist");
    assert_eq!(
        p2_state.poison_counters, 6,
        "p2 must have 6 poison counters after proliferate; got {}",
        p2_state.poison_counters
    );

    // PoisonCountersGiven event must be emitted.
    let poison_event = events.iter().find(|e| {
        matches!(e, GameEvent::PoisonCountersGiven { player, amount, .. }
            if *player == p2 && *amount == 1)
    });
    assert!(
        poison_event.is_some(),
        "PoisonCountersGiven event must be emitted for p2 with amount=1"
    );

    // Proliferated event must reflect 1 player affected.
    let proliferated = events.iter().find(
        |e| matches!(e, GameEvent::Proliferated { players_affected, .. } if *players_affected == 1),
    );
    assert!(
        proliferated.is_some(),
        "Proliferated event must show players_affected=1"
    );
}

// ── Test 5: Multiple counter types on one permanent ──────────────────────────

#[test]
/// CR ruling 2023-02-04 — A permanent with multiple counter types gets one of each.
/// Both +1/+1 and charge counters must be incremented individually.
fn test_proliferate_multiple_counter_types() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Multi Counter Artifact")
                .with_types(vec![CardType::Artifact])
                .with_counter(CounterType::PlusOnePlusOne, 2)
                .with_counter(CounterType::Charge, 1)
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_proliferate(state, p1);

    let artifact = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Multi Counter Artifact")
        .expect("Multi Counter Artifact must be in state");

    // +1/+1 counters: 2 → 3.
    let plus_count = artifact
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        plus_count, 3,
        "+1/+1 counters must increase from 2 to 3; got {}",
        plus_count
    );

    // Charge counters: 1 → 2.
    let charge_count = artifact
        .counters
        .get(&CounterType::Charge)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        charge_count, 2,
        "Charge counters must increase from 1 to 2; got {}",
        charge_count
    );

    // Two CounterAdded events should have been emitted (one per counter type).
    let counter_added_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CounterAdded { .. }))
        .count();
    assert_eq!(
        counter_added_count, 2,
        "Must emit 2 CounterAdded events (one per counter type); got {}",
        counter_added_count
    );
}

// ── Test 6: Multiple targets — creatures and player ──────────────────────────

#[test]
/// CR 701.34a — Multiple permanents with counters and a player with poison
/// are all proliferated in a single action.
fn test_proliferate_multiple_targets() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .player_poison(p3, 3)
        .object(
            ObjectSpec::creature(p1, "Spike A", 1, 1)
                .with_counter(CounterType::PlusOnePlusOne, 2)
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p2, "Spike B", 1, 1)
                .with_counter(CounterType::PlusOnePlusOne, 1)
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_proliferate(state, p1);

    // Spike A: 2 → 3.
    let spike_a = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Spike A")
        .expect("Spike A must be in state");
    let a_count = spike_a
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        a_count, 3,
        "Spike A must have 3 +1/+1 counters; got {}",
        a_count
    );

    // Spike B: 1 → 2.
    let spike_b = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Spike B")
        .expect("Spike B must be in state");
    let b_count = spike_b
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        b_count, 2,
        "Spike B must have 2 +1/+1 counters; got {}",
        b_count
    );

    // p3 poison: 3 → 4.
    let p3_state = state.players.get(&p3).expect("p3 must exist");
    assert_eq!(
        p3_state.poison_counters, 4,
        "p3 must have 4 poison counters; got {}",
        p3_state.poison_counters
    );

    // Proliferated event: 2 permanents, 1 player.
    let proliferated = events.iter().find(|e| {
        matches!(e, GameEvent::Proliferated {
            controller,
            permanents_affected,
            players_affected,
        } if *controller == p1 && *permanents_affected == 2 && *players_affected == 1)
    });
    assert!(
        proliferated.is_some(),
        "Proliferated event must show 2 permanents and 1 player affected"
    );
}

// ── Test 7: Permanent with no counters is not affected ───────────────────────

#[test]
/// CR 701.34a — A permanent with no counters is NOT affected by proliferate.
/// A player with 0 poison is NOT affected. Only Proliferated event is emitted.
fn test_proliferate_no_counters_noop() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Vanilla Creature", 2, 2).in_zone(ZoneId::Battlefield))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_proliferate(state, p1);

    // No CounterAdded events (vanilla creature has no counters).
    let counter_added = events
        .iter()
        .find(|e| matches!(e, GameEvent::CounterAdded { .. }));
    assert!(
        counter_added.is_none(),
        "CounterAdded must NOT be emitted when no permanents have counters"
    );

    // No PoisonCountersGiven events (no player has poison).
    let poison_event = events
        .iter()
        .find(|e| matches!(e, GameEvent::PoisonCountersGiven { .. }));
    assert!(
        poison_event.is_none(),
        "PoisonCountersGiven must NOT be emitted when no player has poison"
    );

    // Proliferated event still emitted (with 0 for both).
    let proliferated = events.iter().find(|e| {
        matches!(e, GameEvent::Proliferated {
            permanents_affected,
            players_affected,
            ..
        } if *permanents_affected == 0 && *players_affected == 0)
    });
    assert!(
        proliferated.is_some(),
        "Proliferated event must still be emitted when no targets are eligible"
    );

    // Vanilla creature must still have no counters.
    let creature = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Vanilla Creature")
        .expect("Vanilla Creature must be in state");
    assert!(
        creature.counters.is_empty(),
        "Vanilla Creature must still have no counters after proliferate"
    );
}

// ── Test 8: Proliferated event always emitted (no eligible targets) ──────────

#[test]
/// Ruling 2023-02-04 — GameEvent::Proliferated is always emitted, even when
/// there are zero eligible permanents and zero players with poison counters.
/// This ensures "whenever you proliferate" triggers still fire.
fn test_proliferate_event_always_emitted() {
    let p1 = p(1);
    let p2 = p(2);

    // No permanents with counters, no players with poison.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (_state, events) = run_proliferate(state, p1);

    let proliferated = events
        .iter()
        .find(|e| matches!(e, GameEvent::Proliferated { .. }));
    assert!(
        proliferated.is_some(),
        "Proliferated event must be emitted even with no eligible targets"
    );

    if let Some(GameEvent::Proliferated {
        controller,
        permanents_affected,
        players_affected,
    }) = proliferated
    {
        assert_eq!(*controller, p1, "Controller must be p1");
        assert_eq!(
            *permanents_affected, 0,
            "permanents_affected must be 0; got {}",
            permanents_affected
        );
        assert_eq!(
            *players_affected, 0,
            "players_affected must be 0; got {}",
            players_affected
        );
    }
}

// ── Test 9: Non-battlefield cards with counters are ignored ──────────────────

#[test]
/// Ruling 2023-02-04 — "You can't choose cards in any zone other than the
/// battlefield, even if they have counters on them." A card in the graveyard
/// with counters is NOT proliferated.
fn test_proliferate_ignores_non_battlefield() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            // A creature in the graveyard with +1/+1 counters — should be ignored.
            ObjectSpec::creature(p1, "Graveyard Spike", 1, 1)
                .with_counter(CounterType::PlusOnePlusOne, 3)
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .object(
            // A creature on the battlefield with no counters — should be unaffected.
            ObjectSpec::creature(p1, "Live Creature", 2, 2).in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_proliferate(state, p1);

    // No CounterAdded events — the only permanent with counters is in the graveyard.
    let counter_added = events
        .iter()
        .find(|e| matches!(e, GameEvent::CounterAdded { .. }));
    assert!(
        counter_added.is_none(),
        "CounterAdded must NOT be emitted for graveyard card with counters"
    );

    // Graveyard Spike must still have 3 counters (unchanged).
    let grave_spike = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Graveyard Spike")
        .expect("Graveyard Spike must be in state");
    let grave_count = grave_spike
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        grave_count, 3,
        "Graveyard Spike must still have 3 +1/+1 counters (not proliferated); got {}",
        grave_count
    );

    // Proliferated event must still fire (with 0 permanents affected).
    let proliferated = events.iter().find(|e| {
        matches!(e, GameEvent::Proliferated { permanents_affected, .. }
            if *permanents_affected == 0)
    });
    assert!(
        proliferated.is_some(),
        "Proliferated event must fire with 0 permanents (graveyard card skipped)"
    );
}

// ── Test 10: Player with 0 poison is not affected ────────────────────────────

#[test]
/// CR 701.34a — A player with 0 poison counters is not affected by proliferate
/// ("players that have a counter"). Only players with ≥1 poison are chosen.
fn test_proliferate_zero_poison_player_unaffected() {
    let p1 = p(1);
    let p2 = p(2);

    // p1 has poison, p2 has none.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_poison(p1, 3)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, _events) = run_proliferate(state, p1);

    // p1 poison: 3 → 4.
    let p1_state = state.players.get(&p1).expect("p1 must exist");
    assert_eq!(
        p1_state.poison_counters, 4,
        "p1 must have 4 poison counters after proliferate; got {}",
        p1_state.poison_counters
    );

    // p2 poison: still 0.
    let p2_state = state.players.get(&p2).expect("p2 must exist");
    assert_eq!(
        p2_state.poison_counters, 0,
        "p2 must still have 0 poison counters (was not eligible); got {}",
        p2_state.poison_counters
    );
}

// ── Test 11: "Whenever you proliferate" trigger fires ────────────────────────

#[test]
/// CR 701.34 — "Whenever you proliferate" trigger fires when a player proliferates.
/// Validated via a creature that gains a +1/+1 counter on proliferate.
/// Uses CastSpell + PassPriority to go through the full trigger pipeline.
fn test_whenever_you_proliferate_trigger_fires() {
    let p1 = p(1);
    let p2 = p(2);

    let def = proliferate_spell_def();
    let registry = CardRegistry::new(vec![def]);

    // A creature with "Whenever you proliferate, put a +1/+1 counter on this creature."
    let proliferate_watcher = ObjectSpec::creature(p1, "Core Prowler", 2, 2)
        .with_triggered_ability(TriggeredAbilityDef {
            etb_filter: None,
            targets: vec![],
            trigger_on: TriggerEvent::ControllerProliferates,
            intervening_if: None,
            description:
                "Whenever you proliferate, put a +1/+1 counter on this creature. (CR 701.34)"
                    .to_string(),
            effect: Some(Effect::AddCounter {
                target: mtg_engine::CardEffectTarget::Source,
                counter: CounterType::PlusOnePlusOne,
                count: 1,
            }),
        })
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(proliferate_watcher)
        .object(
            ObjectSpec::card(p1, "Proliferate Spell")
                .with_card_id(CardId("proliferate-spell".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let spell_id = find_by_name(&state, "Proliferate Spell");

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

    // First pass_all: resolves proliferate spell, queues trigger.
    let (state, _) = pass_all(state.clone(), &[p1, p2]);
    // Second pass_all: resolves "whenever you proliferate" trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 701.34: Core Prowler should have 1 +1/+1 counter from the trigger.
    let prowler = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Core Prowler" && obj.zone == ZoneId::Battlefield)
        .expect("Core Prowler must be on the battlefield");

    let counter_count = prowler
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);

    assert_eq!(
        counter_count, 1,
        "Core Prowler must have 1 +1/+1 counter after 'whenever you proliferate' trigger; got {}",
        counter_count
    );
}

// ── Test 12: Multiplayer — all players with poison are affected ──────────────

#[test]
/// CR 701.34a multiplayer — In a 4-player game, all players with poison counters
/// and all permanents with counters receive +1, regardless of controller.
fn test_proliferate_multiplayer_affects_all_players() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        // p2 has 2 poison, p3 has 5 poison, p4 has 0 poison (not affected).
        .player_poison(p2, 2)
        .player_poison(p3, 5)
        // p2 controls a creature with a +1/+1 counter.
        .object(
            ObjectSpec::creature(p2, "Opponent Spike", 2, 2)
                .with_counter(CounterType::PlusOnePlusOne, 1)
                .in_zone(ZoneId::Battlefield),
        )
        // p3 controls an artifact with a charge counter.
        .object(
            ObjectSpec::card(p3, "Opponent Artifact")
                .with_types(vec![CardType::Artifact])
                .with_counter(CounterType::Charge, 2)
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_proliferate(state, p1);

    // p2 poison: 2 → 3.
    let p2_state = state.players.get(&p2).expect("p2 must exist");
    assert_eq!(
        p2_state.poison_counters, 3,
        "p2 must have 3 poison after proliferate; got {}",
        p2_state.poison_counters
    );

    // p3 poison: 5 → 6.
    let p3_state = state.players.get(&p3).expect("p3 must exist");
    assert_eq!(
        p3_state.poison_counters, 6,
        "p3 must have 6 poison after proliferate; got {}",
        p3_state.poison_counters
    );

    // p4 poison: still 0.
    let p4_state = state.players.get(&p4).expect("p4 must exist");
    assert_eq!(
        p4_state.poison_counters, 0,
        "p4 must still have 0 poison (was not eligible); got {}",
        p4_state.poison_counters
    );

    // Opponent Spike: 1 → 2 +1/+1 counters.
    let spike = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Opponent Spike")
        .expect("Opponent Spike must be in state");
    let spike_count = spike
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        spike_count, 2,
        "Opponent Spike must have 2 +1/+1 counters; got {}",
        spike_count
    );

    // Opponent Artifact: 2 → 3 charge counters.
    let artifact = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Opponent Artifact")
        .expect("Opponent Artifact must be in state");
    let artifact_count = artifact
        .counters
        .get(&CounterType::Charge)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        artifact_count, 3,
        "Opponent Artifact must have 3 charge counters; got {}",
        artifact_count
    );

    // Proliferated event: 2 permanents affected, 2 players affected.
    let proliferated = events.iter().find(|e| {
        matches!(e, GameEvent::Proliferated {
            controller,
            permanents_affected,
            players_affected,
        } if *controller == p1 && *permanents_affected == 2 && *players_affected == 2)
    });
    assert!(
        proliferated.is_some(),
        "Proliferated event must show controller=p1, 2 permanents, 2 players affected"
    );
}

// ── Test 13: Auto-select-all adds harmful counters to self ───────────────────

#[test]
/// CR 701.34a — "choose any number" means the controller may choose NOT to include
/// themselves, but the current auto-select-all model always includes every eligible
/// permanent and player, including the proliferating player's own counters.
///
/// This test documents the known limitation: if p1 has 9 poison counters and
/// proliferates, their own poison goes to 10, which triggers the SBA loss condition
/// (CR 704.5c). A real player would never make this choice. The auto-select-all
/// simplification produces a game-losing state that is impossible under correct CR play.
///
/// When interactive choice is added in M10+, the controller must be able to exclude
/// themselves and their own permanents from the proliferate selection.
// TODO(M10+): Replace this test with one that verifies the player CAN choose not to
// proliferate their own poison. This test then becomes a regression guard for the
// "controller excluded self" path.
fn test_proliferate_auto_select_adds_own_poison() {
    let p1 = p(1);
    let p2 = p(2);

    // p1 has 9 poison (one away from losing), p2 has 0 poison.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_poison(p1, 9)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_proliferate(state, p1);

    // Auto-select-all: p1's own poison is incremented to 10 (game-losing).
    // In real MTG, p1 would choose to exclude themselves.
    let p1_state = state.players.get(&p1).expect("p1 must exist");
    assert_eq!(
        p1_state.poison_counters, 10,
        // TODO(M10+): This behavior is incorrect; interactive choice must allow opting out.
        "Auto-select-all increments controller's own poison to 10; got {}",
        p1_state.poison_counters
    );

    // PoisonCountersGiven event is emitted even for the proliferating player themselves.
    let self_poison_event = events.iter().find(|e| {
        matches!(e, GameEvent::PoisonCountersGiven { player, amount, .. }
            if *player == p1 && *amount == 1)
    });
    assert!(
        self_poison_event.is_some(),
        "PoisonCountersGiven must be emitted for the proliferating player's own poison (auto-select-all limitation)"
    );

    // p2 is unaffected (no poison counters to proliferate).
    let p2_state = state.players.get(&p2).expect("p2 must exist");
    assert_eq!(
        p2_state.poison_counters, 0,
        "p2 must still have 0 poison counters; got {}",
        p2_state.poison_counters
    );
}
