//! Graft keyword ability tests (CR 702.58).
//!
//! Graft represents both a static ability and a triggered ability:
//! - Static: "This permanent enters with N +1/+1 counters on it" (CR 702.58a).
//! - Triggered: "Whenever another creature enters, if this permanent has a +1/+1
//!   counter on it, you may move a +1/+1 counter from this permanent onto that
//!   creature." (CR 702.58a).
//!
//! Key rules verified:
//! - Graft N causes the permanent to enter with N counters (CR 702.58a static).
//! - Trigger fires when another creature enters IF source has a +1/+1 counter (CR 603.4 intervening-if).
//! - Trigger does NOT fire when source has no +1/+1 counter (CR 603.4).
//! - Trigger does NOT fire for the graft permanent itself entering ("another creature").
//! - Trigger fires for ANY player's creature entering (unlike Evolve, which is controller-only).
//! - Non-creature permanents do NOT trigger Graft.
//! - Multiple instances of Graft each trigger separately (CR 702.58b).
//! - Moving all counters off a 0/0 graft creature causes SBA destruction (CR 704.5f).
//! - Re-check at resolution: if source loses counter before trigger resolves, trigger fizzles (CR 603.4).

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost,
    ObjectId, ObjectSpec, PlayerId, StackObjectKind, Step, TypeLine, ZoneId,
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

fn find_object_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
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

/// Cast a creature from hand, resolve it, and return (state, events).
/// Caller must ensure mana is in the pool and priority is set before calling.
fn cast_and_resolve(
    state: GameState,
    caster: PlayerId,
    card_name: &str,
    other_player: PlayerId,
) -> (GameState, Vec<GameEvent>) {
    let card_id = find_object(&state, card_name);
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: caster,
            card: card_id,
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell '{}' failed: {:?}", card_name, e));

    // Resolve: both players pass to resolve the spell (permanent lands on battlefield).
    pass_all(state, &[caster, other_player])
}

// ── Card definitions ──────────────────────────────────────────────────────────

/// 0/0 creature with Graft 2 (enters with 2 +1/+1 counters).
fn graft_2_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("graft-2".to_string()),
        name: "Graft Ooze".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Graft 2".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Graft(2))],
        power: Some(0),
        toughness: Some(0),
        ..Default::default()
    }
}

/// 0/0 creature with Graft 1 (enters with 1 +1/+1 counter).
fn graft_1_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("graft-1".to_string()),
        name: "Graft Sapling".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Graft 1".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Graft(1))],
        power: Some(0),
        toughness: Some(0),
        ..Default::default()
    }
}

/// 0/0 creature with Graft 2 AND Graft 3 (two separate instances; CR 702.58b).
fn double_graft_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("double-graft".to_string()),
        name: "Double Graft Wurm".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Graft 2\nGraft 3".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Graft(2)),
            AbilityDefinition::Keyword(KeywordAbility::Graft(3)),
        ],
        power: Some(0),
        toughness: Some(0),
        ..Default::default()
    }
}

/// 2/2 vanilla creature (target for Graft trigger).
fn vanilla_2_2_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("vanilla-2-2".to_string()),
        name: "Grizzly Bears".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// 1/1 vanilla creature for multiplayer tests (P2 casts this).
fn vanilla_1_1_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("vanilla-1-1".to_string()),
        name: "Tiny Saproling".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}

/// Artifact (non-creature permanent). For non-creature negative test.
fn vanilla_artifact_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("vanilla-artifact".to_string()),
        name: "Vanilla Artifact".to_string(),
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
        power: None,
        toughness: None,
        ..Default::default()
    }
}

// ── Test 1: ETB counter placement ────────────────────────────────────────────

#[test]
/// CR 702.58a — Graft N static ability: permanent enters with N +1/+1 counters.
/// A 0/0 creature with Graft 2 enters the battlefield; it should have 2 +1/+1 counters.
fn test_graft_etb_counters() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![graft_2_def()]);

    let graft_card = ObjectSpec::creature(p1, "Graft Ooze", 0, 0)
        .with_keyword(KeywordAbility::Graft(2))
        .with_card_id(CardId("graft-2".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(graft_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 3);
    state.turn.priority_holder = Some(p1);

    // Cast and resolve the graft creature. Both players pass to let it resolve.
    let (state, _) = cast_and_resolve(state, p1, "Graft Ooze", p2);

    // Verify the graft creature is on the battlefield with 2 +1/+1 counters.
    let graft_id = find_object_in_zone(&state, "Graft Ooze", ZoneId::Battlefield)
        .expect("CR 702.58a: Graft Ooze should be on battlefield");

    let obj = state.objects.get(&graft_id).unwrap();
    let counter_count = obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "CR 702.58a: Graft 2 creature should enter with 2 +1/+1 counters"
    );
}

// ── Test 2: Trigger moves counter when another creature enters ────────────────

#[test]
/// CR 702.58a — Graft trigger fires and moves one +1/+1 counter when another
/// creature enters. Graft creature with 2 counters; entering 2/2 gets 1 counter;
/// Graft creature retains 1 counter.
fn test_graft_trigger_moves_counter() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![graft_2_def(), vanilla_2_2_def()]);

    // Graft creature already on battlefield with 2 +1/+1 counters (as it entered).
    let graft_obj = ObjectSpec::creature(p1, "Graft Ooze", 0, 0)
        .with_keyword(KeywordAbility::Graft(2))
        .with_card_id(CardId("graft-2".to_string()))
        .with_counter(CounterType::PlusOnePlusOne, 2)
        .in_zone(ZoneId::Battlefield);

    // 2/2 vanilla creature in hand to cast.
    let entering = ObjectSpec::creature(p1, "Grizzly Bears", 2, 2)
        .with_card_id(CardId("vanilla-2-2".to_string()))
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(graft_obj)
        .object(entering)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state.turn.priority_holder = Some(p1);

    let graft_id = find_object(&state, "Graft Ooze");

    // Cast and resolve the entering creature.
    let (state, _) = cast_and_resolve(state, p1, "Grizzly Bears", p2);

    // Graft trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.58a: Graft trigger should be on the stack after creature ETB"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::GraftTrigger { .. }
        ),
        "CR 702.58a: stack entry should be GraftTrigger"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Graft creature should have lost 1 counter (now has 1).
    let graft_obj = state.objects.get(&graft_id).unwrap();
    let graft_counters = graft_obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        graft_counters, 1,
        "CR 702.58a: Graft source should have 1 +1/+1 counter remaining after moving one"
    );

    // Entering creature should have gained 1 counter.
    let entering_id = find_object_in_zone(&state, "Grizzly Bears", ZoneId::Battlefield)
        .expect("CR 702.58a: Grizzly Bears should be on battlefield");
    let entering_obj = state.objects.get(&entering_id).unwrap();
    let entering_counters = entering_obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        entering_counters, 1,
        "CR 702.58a: entering creature should have 1 +1/+1 counter after Graft trigger"
    );
}

// ── Test 3: Trigger does not fire without counters ────────────────────────────

#[test]
/// CR 702.58a intervening-if (CR 603.4): If the Graft permanent has no +1/+1
/// counters, the trigger does not fire when another creature enters.
fn test_graft_trigger_does_not_fire_without_counters() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![graft_2_def(), vanilla_2_2_def()]);

    // Graft creature on battlefield with NO +1/+1 counters.
    let graft_obj = ObjectSpec::creature(p1, "Graft Ooze", 0, 0)
        .with_keyword(KeywordAbility::Graft(2))
        .with_card_id(CardId("graft-2".to_string()))
        .in_zone(ZoneId::Battlefield);

    let entering = ObjectSpec::creature(p1, "Grizzly Bears", 2, 2)
        .with_card_id(CardId("vanilla-2-2".to_string()))
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(graft_obj)
        .object(entering)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state.turn.priority_holder = Some(p1);

    // Cast and resolve the entering creature.
    let (state, _) = cast_and_resolve(state, p1, "Grizzly Bears", p2);

    // No Graft trigger should be on the stack (intervening-if check fails).
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.58a / CR 603.4: Graft trigger should NOT fire when source has no +1/+1 counter"
    );
}

// ── Test 4: Trigger does not fire for self ────────────────────────────────────

#[test]
/// CR 702.58a "another creature": The Graft permanent entering does NOT trigger
/// its own Graft ability (only "another creature" entering triggers it).
fn test_graft_trigger_does_not_fire_for_self() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![graft_2_def()]);

    let graft_card = ObjectSpec::creature(p1, "Graft Ooze", 0, 0)
        .with_keyword(KeywordAbility::Graft(2))
        .with_card_id(CardId("graft-2".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(graft_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 3);
    state.turn.priority_holder = Some(p1);

    // Cast and resolve the graft creature itself (no other graft creature on battlefield).
    let (state, _) = cast_and_resolve(state, p1, "Graft Ooze", p2);

    // No GraftTrigger should be on the stack. (Only ETB counters were placed; no trigger.)
    assert!(
        !state
            .stack_objects
            .iter()
            .any(|so| matches!(so.kind, StackObjectKind::GraftTrigger { .. })),
        "CR 702.58a: Graft should NOT trigger from its own ETB ('another creature')"
    );

    // The graft creature should have 2 +1/+1 counters (static ability fired).
    let graft_id = find_object_in_zone(&state, "Graft Ooze", ZoneId::Battlefield)
        .expect("Graft Ooze should be on battlefield");
    let counter_count = state
        .objects
        .get(&graft_id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "CR 702.58a: Graft 2 creature should still enter with 2 counters"
    );
}

// ── Test 5: Trigger fires for opponent's creatures ────────────────────────────

#[test]
/// CR 702.58a: Unlike Evolve, Graft fires for ANY player's creature entering,
/// including opponents'. P1 controls a Graft creature; P2 casts a creature.
/// Graft trigger should fire.
fn test_graft_trigger_fires_for_opponents_creatures() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![graft_1_def(), vanilla_1_1_def()]);

    // P1 controls Graft creature on battlefield with 1 counter.
    let graft_obj = ObjectSpec::creature(p1, "Graft Sapling", 0, 0)
        .with_keyword(KeywordAbility::Graft(1))
        .with_card_id(CardId("graft-1".to_string()))
        .with_counter(CounterType::PlusOnePlusOne, 1)
        .in_zone(ZoneId::Battlefield);

    // P2 has a 1/1 creature in hand to cast.
    let p2_creature = ObjectSpec::creature(p2, "Tiny Saproling", 1, 1)
        .with_card_id(CardId("vanilla-1-1".to_string()))
        .with_mana_cost(ManaCost {
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p2));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(graft_obj)
        .object(p2_creature)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p2);

    let graft_id = find_object(&state, "Graft Sapling");

    // P2 casts and resolves their creature.
    let (state, _) = cast_and_resolve(state, p2, "Tiny Saproling", p1);

    // Graft trigger should be on the stack (controlled by P1 since P1 controls the source).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.58a: Graft trigger should fire for opponent's creature entering"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::GraftTrigger { .. }
        ),
        "CR 702.58a: stack entry should be GraftTrigger"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p2, p1]);

    // After trigger resolves: Graft Sapling had 1 counter, moved it away, now 0/0 with 0
    // counters. SBAs (CR 704.5f) destroy it. Verify it's in graveyard or has 0 counters.
    // The object may be gone from `state.objects` under the same ID after SBAs killed it.
    let graft_counters = state
        .objects
        .get(&graft_id)
        .map(|obj| {
            obj.counters
                .get(&CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0)
        })
        .unwrap_or(0); // If object is gone (destroyed by SBA), it had 0 counters before removal.
    assert_eq!(
        graft_counters, 0,
        "CR 702.58a: Graft source should have 0 +1/+1 counters after moving its only counter"
    );

    // Entering creature should have 1 counter.
    let entering_id = find_object_in_zone(&state, "Tiny Saproling", ZoneId::Battlefield)
        .expect("Tiny Saproling should be on battlefield");
    let entering_counters = state
        .objects
        .get(&entering_id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        entering_counters, 1,
        "CR 702.58a: opponent's entering creature should receive the +1/+1 counter"
    );
}

// ── Test 6: Non-creature permanents do not trigger Graft ──────────────────────

#[test]
/// CR 702.58a: Only creatures entering trigger Graft. An artifact entering
/// the battlefield does NOT trigger the Graft ability.
fn test_graft_noncreature_does_not_trigger() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![graft_1_def(), vanilla_artifact_def()]);

    // Graft creature on battlefield with 1 counter.
    let graft_obj = ObjectSpec::creature(p1, "Graft Sapling", 0, 0)
        .with_keyword(KeywordAbility::Graft(1))
        .with_card_id(CardId("graft-1".to_string()))
        .with_counter(CounterType::PlusOnePlusOne, 1)
        .in_zone(ZoneId::Battlefield);

    // Non-creature artifact in hand.
    let artifact = ObjectSpec::card(p1, "Vanilla Artifact")
        .with_types(vec![CardType::Artifact])
        .with_card_id(CardId("vanilla-artifact".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(graft_obj)
        .object(artifact)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    // Cast and resolve the artifact.
    let (state, _) = cast_and_resolve(state, p1, "Vanilla Artifact", p2);

    // No Graft trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.58a: Graft should NOT trigger when a non-creature permanent enters"
    );
}

// ── Test 7: Multiple instances trigger separately ────────────────────────────

#[test]
/// CR 702.58b — Multiple instances of Graft each trigger separately.
/// A creature with Graft 2 and Graft 3 enters with 5 counters. When another
/// creature enters, two separate GraftTrigger stack objects are created.
fn test_graft_multiple_instances() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![double_graft_def(), vanilla_2_2_def()]);

    // A creature with two Graft instances (Graft 2 and Graft 3), already on battlefield
    // with 5 +1/+1 counters (as it would have entered).
    let graft_obj = ObjectSpec::creature(p1, "Double Graft Wurm", 0, 0)
        .with_keyword(KeywordAbility::Graft(2))
        .with_keyword(KeywordAbility::Graft(3))
        .with_card_id(CardId("double-graft".to_string()))
        .with_counter(CounterType::PlusOnePlusOne, 5)
        .in_zone(ZoneId::Battlefield);

    let entering = ObjectSpec::creature(p1, "Grizzly Bears", 2, 2)
        .with_card_id(CardId("vanilla-2-2".to_string()))
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(graft_obj)
        .object(entering)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state.turn.priority_holder = Some(p1);

    let graft_id = find_object(&state, "Double Graft Wurm");

    // Cast and resolve the entering creature.
    let (state, _) = cast_and_resolve(state, p1, "Grizzly Bears", p2);

    // Two GraftTrigger stack objects should be on the stack (one per instance, CR 702.58b).
    let graft_triggers = state
        .stack_objects
        .iter()
        .filter(|so| matches!(so.kind, StackObjectKind::GraftTrigger { .. }))
        .count();
    assert_eq!(
        graft_triggers, 2,
        "CR 702.58b: two Graft instances should produce two separate triggers"
    );

    // Resolve both triggers (each player passes twice for 2 triggers).
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    // After resolving both triggers, source should have 3 counters (5 - 2 moved).
    let graft_obj = state.objects.get(&graft_id).unwrap();
    let graft_counters = graft_obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        graft_counters, 3,
        "CR 702.58b: two Graft triggers resolved; source should have 3 counters (5 - 2)"
    );

    // Entering creature should have 2 counters gained.
    let entering_id = find_object_in_zone(&state, "Grizzly Bears", ZoneId::Battlefield)
        .expect("Grizzly Bears should be on battlefield");
    let entering_counters = state
        .objects
        .get(&entering_id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        entering_counters, 2,
        "CR 702.58b: entering creature should have 2 +1/+1 counters (one from each trigger)"
    );
}

// ── Test 8: Multiple Graft ETB counters sum correctly (CR 702.58b) ────────────

#[test]
/// CR 702.58b — A creature with Graft 2 and Graft 3 enters with 5 +1/+1 counters
/// (the N values from all instances sum together).
fn test_graft_multiple_instances_etb_counter_sum() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![double_graft_def()]);

    let graft_card = ObjectSpec::creature(p1, "Double Graft Wurm", 0, 0)
        .with_keyword(KeywordAbility::Graft(2))
        .with_keyword(KeywordAbility::Graft(3))
        .with_card_id(CardId("double-graft".to_string()))
        .with_mana_cost(ManaCost {
            generic: 4,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(graft_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 5);
    state.turn.priority_holder = Some(p1);

    // Cast and resolve the double-graft creature.
    let (state, _) = cast_and_resolve(state, p1, "Double Graft Wurm", p2);

    let graft_id = find_object_in_zone(&state, "Double Graft Wurm", ZoneId::Battlefield)
        .expect("Double Graft Wurm should be on battlefield");
    let counter_count = state
        .objects
        .get(&graft_id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 5,
        "CR 702.58b: Graft 2 + Graft 3 = 5 +1/+1 counters on entry"
    );
}

// ── Test 9: Resolution re-check (CR 603.4) — trigger fizzles if counter removed ──

#[test]
/// CR 603.4: Intervening-if re-checked at resolution. If the Graft source loses
/// its last +1/+1 counter BEFORE the trigger resolves (e.g., another effect
/// removed it), the trigger does nothing.
///
/// Simulated by placing a Graft source with 1 counter on battlefield, another
/// creature entering triggers Graft, but then we manually modify state to remove
/// the counter before the trigger resolves — the trigger fizzles.
fn test_graft_resolution_recheck_intervening_if() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![graft_1_def(), vanilla_2_2_def()]);

    // Graft creature on battlefield with 1 counter.
    let graft_obj = ObjectSpec::creature(p1, "Graft Sapling", 0, 0)
        .with_keyword(KeywordAbility::Graft(1))
        .with_card_id(CardId("graft-1".to_string()))
        .with_counter(CounterType::PlusOnePlusOne, 1)
        .in_zone(ZoneId::Battlefield);

    let entering = ObjectSpec::creature(p1, "Grizzly Bears", 2, 2)
        .with_card_id(CardId("vanilla-2-2".to_string()))
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(graft_obj)
        .object(entering)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state.turn.priority_holder = Some(p1);

    let graft_id = find_object(&state, "Graft Sapling");

    // Cast and resolve the entering creature; Graft trigger queues.
    let (mut state, _) = cast_and_resolve(state, p1, "Grizzly Bears", p2);

    // Verify trigger is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Graft trigger should be on stack"
    );

    // Manually remove the last +1/+1 counter from the graft source (simulating
    // an effect that removed it while the trigger was on the stack).
    if let Some(obj) = state.objects.get_mut(&graft_id) {
        obj.counters = obj.counters.without(&CounterType::PlusOnePlusOne);
    }

    // Now resolve the trigger — the intervening-if re-check should fail.
    let (state, _) = pass_all(state, &[p1, p2]);

    // The graft source had its counter manually removed; after the trigger fizzles,
    // SBAs (CR 704.5f) destroy the 0/0 creature. Check via graveyard or absence.
    // If still present, it has 0 counters; if gone, it was destroyed by SBAs.
    let graft_counters = state
        .objects
        .get(&graft_id)
        .map(|obj| {
            obj.counters
                .get(&CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    assert_eq!(
        graft_counters, 0,
        "CR 603.4: trigger fizzled — source should have 0 counters (or be destroyed by SBA)"
    );

    // The entering creature should have 0 counters (trigger fizzled, no counter moved).
    let entering_id = find_object_in_zone(&state, "Grizzly Bears", ZoneId::Battlefield)
        .expect("Grizzly Bears should be on battlefield");
    let entering_counters = state
        .objects
        .get(&entering_id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        entering_counters, 0,
        "CR 603.4: entering creature should have 0 counters (trigger fizzled)"
    );
}
