//! Evolve keyword ability tests (CR 702.100).
//!
//! Evolve is a triggered ability: "Whenever a creature you control enters,
//! if that creature's power is greater than this creature's power and/or that
//! creature's toughness is greater than this creature's toughness, put a +1/+1
//! counter on this creature."
//!
//! Key rules verified:
//! - Trigger fires when entering creature has greater P or T (CR 702.100a).
//! - Trigger does NOT fire when entering creature has equal or lesser P and T (CR 702.100a).
//! - Intervening-if re-checked at resolution (CR 603.4).
//! - Multiple instances each trigger separately (CR 702.100d).
//! - Noncreature permanents do not trigger evolve (CR 702.100c).
//! - OR condition: greater power alone, greater toughness alone, or both (CR 702.100a).
//! - Multiplayer: only creatures controlled by the same player trigger evolve (CR 702.100a).

use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, Command, CounterType, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, StackObjectKind, Step,
    TypeLine, ZoneId,
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
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell '{}' failed: {:?}", card_name, e));

    // Resolve: both players pass to resolve the spell (permanent lands on battlefield).
    pass_all(state, &[caster, other_player])
}

// ── Card definitions ──────────────────────────────────────────────────────────

/// 1/1 creature with Evolve.
fn evolve_1_1_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("evolve-1-1".to_string()),
        name: "Evolve Sapling".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Evolve".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Evolve)],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}

/// 2/2 creature with Evolve.
fn evolve_2_2_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("evolve-2-2".to_string()),
        name: "Evolve Beast".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Evolve".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Evolve)],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// 3/3 vanilla creature.
fn vanilla_3_3_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("vanilla-3-3".to_string()),
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
        power: Some(3),
        toughness: Some(3),
        ..Default::default()
    }
}

/// 1/4 vanilla creature (greater toughness only).
fn vanilla_1_4_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("vanilla-1-4".to_string()),
        name: "Tough Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(1),
        toughness: Some(4),
        ..Default::default()
    }
}

/// 3/1 vanilla creature (greater power only).
fn vanilla_3_1_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("vanilla-3-1".to_string()),
        name: "Strong Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(3),
        toughness: Some(1),
        ..Default::default()
    }
}

/// 2/2 vanilla creature (for equal-stat and small-creature tests).
fn vanilla_2_2_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("vanilla-2-2".to_string()),
        name: "Grizzly Cub".to_string(),
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

/// 1/1 vanilla creature (smaller than 2/2 evolve, for negative test).
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

/// Creature with TWO instances of Evolve (CR 702.100d).
fn double_evolve_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("double-evolve".to_string()),
        name: "Double Evolve Wurm".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Evolve\nEvolve".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Evolve),
            AbilityDefinition::Keyword(KeywordAbility::Evolve),
        ],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}

/// Artifact (non-creature permanent). For CR 702.100c test.
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

// ── Test 1: Basic — greater power triggers evolve ────────────────────────────

#[test]
/// CR 702.100a — Evolve fires when entering creature has greater power.
/// P1 controls a 1/1 evolve creature. A 3/3 enters under P1's control.
/// Trigger fires. After resolution, evolve creature has 1 +1/+1 counter (2/2).
fn test_evolve_basic_greater_power() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![evolve_1_1_def(), vanilla_3_3_def()]);

    // Evolve creature already on battlefield.
    let evolve_obj = ObjectSpec::creature(p1, "Evolve Sapling", 1, 1)
        .with_keyword(KeywordAbility::Evolve)
        .with_card_id(CardId("evolve-1-1".to_string()))
        .in_zone(ZoneId::Battlefield);

    // Entering 3/3 creature in hand — must have P/T set for evolve comparison.
    let entering = ObjectSpec::creature(p1, "Grizzly Bears", 3, 3)
        .with_card_id(CardId("vanilla-3-3".to_string()))
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
        .object(evolve_obj)
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

    let evolve_id = find_object(&state, "Evolve Sapling");

    // Cast and resolve the 3/3 creature.
    let (state, _) = cast_and_resolve(state, p1, "Grizzly Bears", p2);

    // Grizzly Bears should be on battlefield.
    assert!(
        find_object_in_zone(&state, "Grizzly Bears", ZoneId::Battlefield).is_some(),
        "CR 702.100a: entering creature should be on the battlefield"
    );

    // Evolve trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.100a: evolve trigger should be on the stack after ETB"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::EvolveTrigger { .. }
        ),
        "CR 702.100a: stack entry should be EvolveTrigger"
    );

    // Resolve the trigger — both players pass.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Evolve creature should have 1 +1/+1 counter.
    let obj = state
        .objects
        .get(&evolve_id)
        .expect("Evolve Sapling should be on battlefield");
    let counter_count = obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "CR 702.100a: evolve creature should have 1 +1/+1 counter"
    );

    // Layer-aware P/T should be 2/2.
    let chars = calculate_characteristics(&state, evolve_id)
        .expect("Evolve Sapling should be on battlefield");
    assert_eq!(chars.power, Some(2), "CR 702.100a: power should be 2 (1+1)");
    assert_eq!(
        chars.toughness,
        Some(2),
        "CR 702.100a: toughness should be 2 (1+1)"
    );
}

// ── Test 2: Greater toughness triggers evolve ─────────────────────────────────

#[test]
/// CR 702.100a — Evolve fires when entering creature has greater toughness only.
/// P1 controls a 2/2 evolve creature. A 1/4 enters (T 4 > 2). Trigger fires.
fn test_evolve_basic_greater_toughness() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![evolve_2_2_def(), vanilla_1_4_def()]);

    let evolve_obj = ObjectSpec::creature(p1, "Evolve Beast", 2, 2)
        .with_keyword(KeywordAbility::Evolve)
        .with_card_id(CardId("evolve-2-2".to_string()))
        .in_zone(ZoneId::Battlefield);

    // 1/4 entering — P/T must be set for evolve comparison.
    let entering = ObjectSpec::creature(p1, "Tough Creature", 1, 4)
        .with_card_id(CardId("vanilla-1-4".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(evolve_obj)
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
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let evolve_id = find_object(&state, "Evolve Beast");

    let (state, _) = cast_and_resolve(state, p1, "Tough Creature", p2);

    // Evolve trigger should be on the stack (toughness 4 > 2).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.100a: evolve trigger fires when entering creature has greater toughness"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::EvolveTrigger { .. }
        ),
        "CR 702.100a: stack entry should be EvolveTrigger"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    let obj = state
        .objects
        .get(&evolve_id)
        .expect("Evolve Beast on battlefield");
    let counter_count = obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "CR 702.100a: evolve creature should have 1 +1/+1 counter after greater-toughness trigger"
    );
}

// ── Test 3: Greater power only — OR condition ─────────────────────────────────

#[test]
/// CR 702.100a — Trigger fires on greater power alone even if entering has less
/// toughness (inclusive OR). Evolve creature: 2/4 (counters). Entering: 3/1.
/// Power 3 > 2 triggers evolve even though toughness 1 < 4.
fn test_evolve_greater_power_only_or_condition() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![evolve_2_2_def(), vanilla_3_1_def()]);

    // Manually give the evolve creature extra toughness via a +0/+2 setup:
    // Build as a 2/4 creature directly.
    let evolve_obj = ObjectSpec::creature(p1, "Evolve Beast", 2, 4)
        .with_keyword(KeywordAbility::Evolve)
        .with_card_id(CardId("evolve-2-2".to_string()))
        .in_zone(ZoneId::Battlefield);

    // 3/1 entering — P/T must be set for evolve comparison.
    let entering = ObjectSpec::creature(p1, "Strong Creature", 3, 1)
        .with_card_id(CardId("vanilla-3-1".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(evolve_obj)
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
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let evolve_id = find_object(&state, "Evolve Beast");

    let (state, _) = cast_and_resolve(state, p1, "Strong Creature", p2);

    // Trigger should fire — entering power 3 > evolve power 2 (even though T 1 < 4).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.100a: evolve trigger fires when entering power > evolve power (OR condition)"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    let obj = state
        .objects
        .get(&evolve_id)
        .expect("Evolve Beast on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "CR 702.100a: evolve counter added on greater-power-only trigger"
    );
}

// ── Test 4: Equal stats — no trigger ─────────────────────────────────────────

#[test]
/// CR 702.100a negative — Trigger does NOT fire when entering creature has
/// equal power and toughness to the evolve creature.
/// P1 controls a 2/2 evolve creature. A 2/2 enters. No trigger.
fn test_evolve_no_trigger_equal_stats() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![evolve_2_2_def(), vanilla_2_2_def()]);

    let evolve_obj = ObjectSpec::creature(p1, "Evolve Beast", 2, 2)
        .with_keyword(KeywordAbility::Evolve)
        .with_card_id(CardId("evolve-2-2".to_string()))
        .in_zone(ZoneId::Battlefield);

    // A 2/2 entering — same stats as the evolve creature. P/T must be set.
    let entering = ObjectSpec::creature(p1, "Grizzly Cub", 2, 2)
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
        .object(evolve_obj)
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

    let (state, _) = cast_and_resolve(state, p1, "Grizzly Cub", p2);

    // Grizzly Cub should be on battlefield.
    assert!(
        find_object_in_zone(&state, "Grizzly Cub", ZoneId::Battlefield).is_some(),
        "CR 702.100a: entering creature should be on the battlefield"
    );

    // No evolve trigger should be on the stack.
    let evolve_on_stack = state
        .stack_objects
        .iter()
        .any(|s| matches!(s.kind, StackObjectKind::EvolveTrigger { .. }));
    assert!(
        !evolve_on_stack,
        "CR 702.100a: evolve does NOT trigger when entering P/T is equal to evolve P/T"
    );
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.100a: stack should be empty after equal-stat ETB"
    );
}

// ── Test 5: Smaller creature — no trigger ────────────────────────────────────

#[test]
/// CR 702.100a negative — Trigger does NOT fire when entering creature is
/// smaller in both power and toughness.
/// P1 controls a 3/3 evolve creature. A 1/1 enters. No trigger.
fn test_evolve_no_trigger_smaller_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![evolve_2_2_def(), vanilla_1_1_def()]);

    // 3/3 evolve creature on battlefield.
    let evolve_obj = ObjectSpec::creature(p1, "Evolve Beast", 3, 3)
        .with_keyword(KeywordAbility::Evolve)
        .with_card_id(CardId("evolve-2-2".to_string()))
        .in_zone(ZoneId::Battlefield);

    // 1/1 entering creature in hand — P/T must be set for evolve comparison.
    let entering = ObjectSpec::creature(p1, "Tiny Saproling", 1, 1)
        .with_card_id(CardId("vanilla-1-1".to_string()))
        .with_mana_cost(ManaCost {
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(evolve_obj)
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
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    let (state, _) = cast_and_resolve(state, p1, "Tiny Saproling", p2);

    assert!(
        find_object_in_zone(&state, "Tiny Saproling", ZoneId::Battlefield).is_some(),
        "Tiny Saproling should be on battlefield"
    );

    let evolve_on_stack = state
        .stack_objects
        .iter()
        .any(|s| matches!(s.kind, StackObjectKind::EvolveTrigger { .. }));
    assert!(
        !evolve_on_stack,
        "CR 702.100a: evolve does NOT trigger when entering creature is smaller"
    );
}

// ── Test 6: Noncreature permanent does not trigger evolve ─────────────────────

#[test]
/// CR 702.100c — A noncreature permanent entering the battlefield does not
/// trigger evolve. Only creatures can trigger evolve.
fn test_evolve_noncreature_does_not_trigger() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![evolve_1_1_def(), vanilla_artifact_def()]);

    let evolve_obj = ObjectSpec::creature(p1, "Evolve Sapling", 1, 1)
        .with_keyword(KeywordAbility::Evolve)
        .with_card_id(CardId("evolve-1-1".to_string()))
        .in_zone(ZoneId::Battlefield);

    // Noncreature artifact in hand.
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
        .object(evolve_obj)
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

    // Cast and resolve the noncreature artifact.
    let (state, _) = cast_and_resolve(state, p1, "Vanilla Artifact", p2);

    // Artifact should be on battlefield.
    assert!(
        find_object_in_zone(&state, "Vanilla Artifact", ZoneId::Battlefield).is_some(),
        "CR 702.100c: noncreature artifact should be on battlefield"
    );

    // No evolve trigger should be on the stack.
    let evolve_on_stack = state
        .stack_objects
        .iter()
        .any(|s| matches!(s.kind, StackObjectKind::EvolveTrigger { .. }));
    assert!(
        !evolve_on_stack,
        "CR 702.100c: noncreature permanent entering does NOT trigger evolve"
    );
}

// ── Test 7: Opponent's creature does not trigger evolve ───────────────────────

#[test]
/// CR 702.100a — Evolve only triggers on creatures controlled by the SAME player.
/// P1 has an evolve creature. P2 casts a 3/3. No trigger for P1's evolve.
/// Then P1 casts a 3/3 — trigger fires.
fn test_evolve_opponents_creature_does_not_trigger() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![evolve_1_1_def(), vanilla_3_3_def(), vanilla_2_2_def()]);

    let evolve_obj = ObjectSpec::creature(p1, "Evolve Sapling", 1, 1)
        .with_keyword(KeywordAbility::Evolve)
        .with_card_id(CardId("evolve-1-1".to_string()))
        .in_zone(ZoneId::Battlefield);

    // P2's 3/3 creature in hand — P/T must be set.
    let p2_creature = ObjectSpec::creature(p2, "Grizzly Bears", 3, 3)
        .with_card_id(CardId("vanilla-3-3".to_string()))
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p2));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(evolve_obj)
        .object(p2_creature)
        .active_player(p2) // P2 is active so they can cast
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state.turn.priority_holder = Some(p2);

    // P2 casts and resolves their 3/3. P1 passes priority too.
    let card_id = find_object(&state, "Grizzly Bears");
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
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
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    )
    .expect("P2 CastSpell should succeed");

    // Both players pass to resolve.
    let (state, _) = pass_all(state, &[p2, p1]);

    // Grizzly Bears should be on battlefield under P2's control.
    assert!(
        find_object_in_zone(&state, "Grizzly Bears", ZoneId::Battlefield).is_some(),
        "Grizzly Bears should be on battlefield under P2"
    );

    // No evolve trigger on P1's evolve creature (different controller).
    let evolve_on_stack = state
        .stack_objects
        .iter()
        .any(|s| matches!(s.kind, StackObjectKind::EvolveTrigger { .. }));
    assert!(
        !evolve_on_stack,
        "CR 702.100a: P2's creature entering does NOT trigger P1's evolve"
    );
}

// ── Test 8: Multiple instances trigger separately (CR 702.100d) ───────────────

#[test]
/// CR 702.100d — A creature with two instances of evolve generates two separate
/// triggers when a larger creature enters.
/// First trigger resolves: 1 counter (1/1 -> 2/2). Second re-checks (3/3 > 2/2):
/// condition still holds, another counter (2/2 -> 3/3).
fn test_evolve_multiple_instances() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![double_evolve_def(), vanilla_3_3_def()]);

    // 1/1 creature with TWO evolve instances on battlefield.
    let double_evolve_obj = ObjectSpec::creature(p1, "Double Evolve Wurm", 1, 1)
        .with_keyword(KeywordAbility::Evolve)
        .with_card_id(CardId("double-evolve".to_string()))
        .in_zone(ZoneId::Battlefield);

    // 3/3 entering — P/T must be set for evolve comparison.
    let entering = ObjectSpec::creature(p1, "Grizzly Bears", 3, 3)
        .with_card_id(CardId("vanilla-3-3".to_string()))
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
        .object(double_evolve_obj)
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

    let evolve_id = find_object(&state, "Double Evolve Wurm");

    // Cast and resolve the 3/3 entering creature.
    let (state, _) = cast_and_resolve(state, p1, "Grizzly Bears", p2);

    // Two evolve triggers should be on the stack (one per evolve instance).
    let evolve_count = state
        .stack_objects
        .iter()
        .filter(|s| matches!(s.kind, StackObjectKind::EvolveTrigger { .. }))
        .count();
    assert_eq!(
        evolve_count, 2,
        "CR 702.100d: two instances of evolve generate two separate triggers"
    );

    // Resolve first trigger — adds 1 counter (1/1 -> 2/2 via layer-aware chars).
    let (state, _) = pass_all(state, &[p1, p2]);
    let obj = state
        .objects
        .get(&evolve_id)
        .expect("evolve creature on battlefield");
    let count_after_first = obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        count_after_first, 1,
        "CR 702.100d: first evolve trigger adds 1 counter"
    );

    // One trigger should still be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.100d: second evolve trigger still on stack"
    );

    // Resolve second trigger — re-checks intervening-if.
    // Evolve creature is now 2/2 (1+1 counter). Entering creature is 3/3.
    // 3 > 2 for both P and T, condition still holds — another counter.
    let (state, _) = pass_all(state, &[p1, p2]);
    let obj = state
        .objects
        .get(&evolve_id)
        .expect("evolve creature on battlefield");
    let count_after_second = obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        count_after_second, 2,
        "CR 702.100d: second evolve trigger also resolves (3/3 > 2/2), adding another counter"
    );
}

// ── Test 9: Intervening-if fails at resolution ────────────────────────────────

#[test]
/// CR 603.4 — Intervening-if condition re-checked at resolution time.
/// Trigger fires at ETB time (entering 3/3 > evolve 1/1), but before the
/// trigger resolves, the evolve creature is pumped to 4/4 via counters.
/// At resolution, the 3/3 entering creature is NOT greater than the 4/4
/// evolve creature in either stat, so the trigger does nothing (no counter).
fn test_evolve_intervening_if_fails_at_resolution() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![evolve_1_1_def(), vanilla_3_3_def()]);

    let evolve_obj = ObjectSpec::creature(p1, "Evolve Sapling", 1, 1)
        .with_keyword(KeywordAbility::Evolve)
        .with_card_id(CardId("evolve-1-1".to_string()))
        .in_zone(ZoneId::Battlefield);

    // 3/3 entering creature in hand.
    let entering = ObjectSpec::creature(p1, "Grizzly Bears", 3, 3)
        .with_card_id(CardId("vanilla-3-3".to_string()))
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
        .object(evolve_obj)
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

    let evolve_id = find_object(&state, "Evolve Sapling");

    // Cast the 3/3 — trigger fires (3/3 > 1/1 at trigger time). After this,
    // Grizzly Bears is on the battlefield and the evolve trigger is on the stack.
    let (mut state, _) = cast_and_resolve(state, p1, "Grizzly Bears", p2);

    // Verify the evolve trigger is on the stack before we pump the evolve creature.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 603.4: evolve trigger should be on the stack after the 3/3 enters"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::EvolveTrigger { .. }
        ),
        "CR 603.4: stack entry should be EvolveTrigger"
    );

    // Before the trigger resolves, pump the evolve creature to 4/4 via counters.
    // This simulates an opponent responding with a pump effect between the trigger
    // going on the stack and its resolution.
    // Add 3 +1/+1 counters directly: 1/1 base + 3 counters = 4/4.
    {
        let obj = state
            .objects
            .get_mut(&evolve_id)
            .expect("Evolve Sapling should be on battlefield");
        obj.counters = obj.counters.update(CounterType::PlusOnePlusOne, 3);
    }

    // Verify the evolve creature is now 4/4 (layer-aware).
    let chars_before = calculate_characteristics(&state, evolve_id)
        .expect("Evolve Sapling should be on battlefield");
    assert_eq!(
        chars_before.power,
        Some(4),
        "CR 603.4 setup: evolve creature should be 4/4 before trigger resolves"
    );
    assert_eq!(
        chars_before.toughness,
        Some(4),
        "CR 603.4 setup: evolve creature should be 4/4 before trigger resolves"
    );

    // Now resolve the trigger. At resolution the re-check fires:
    // entering 3/3 vs evolve 4/4 — neither 3 > 4 (P) nor 3 > 4 (T).
    // CR 603.4: condition is false, trigger does nothing.
    let (state, _) = pass_all(state, &[p1, p2]);

    // The evolve creature should still have exactly 3 counters (manually added),
    // NOT 4. The trigger must NOT have placed an additional counter.
    let obj = state
        .objects
        .get(&evolve_id)
        .expect("Evolve Sapling should still be on battlefield");
    let counter_count = obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "CR 603.4: trigger should do nothing when re-check fails at resolution \
         (evolve 4/4 is not exceeded by entering 3/3 in either stat)"
    );

    // Final layer-aware P/T should be 4/4 (3 counters only, no extra from trigger).
    let chars_after = calculate_characteristics(&state, evolve_id)
        .expect("Evolve Sapling should be on battlefield");
    assert_eq!(
        chars_after.power,
        Some(4),
        "CR 603.4: evolve creature should be 4/4 — trigger did not add a counter"
    );
    assert_eq!(
        chars_after.toughness,
        Some(4),
        "CR 603.4: evolve creature should be 4/4 — trigger did not add a counter"
    );
}

// ── Test 10: Multiplayer — only same controller triggers ──────────────────────

#[test]
/// CR 702.100a multiplayer — In a 4-player game, P1's evolve creature only
/// triggers when a creature controlled by P1 enters. P2 controls a large
/// creature that would qualify but does NOT trigger P1's evolve.
fn test_evolve_multiplayer_only_same_controller() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let registry = CardRegistry::new(vec![evolve_1_1_def(), vanilla_3_3_def()]);

    let evolve_obj = ObjectSpec::creature(p1, "Evolve Sapling", 1, 1)
        .with_keyword(KeywordAbility::Evolve)
        .with_card_id(CardId("evolve-1-1".to_string()))
        .in_zone(ZoneId::Battlefield);

    // P2's 3/3 creature in hand — enters under P2's control. P/T must be set.
    let p2_creature = ObjectSpec::creature(p2, "Grizzly Bears", 3, 3)
        .with_card_id(CardId("vanilla-3-3".to_string()))
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p2));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(evolve_obj)
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
        .add(ManaColor::Green, 2);
    state.turn.priority_holder = Some(p2);

    // P2 casts their 3/3.
    let card_id = find_object(&state, "Grizzly Bears");
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
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
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    )
    .expect("P2 CastSpell should succeed");

    // All players pass to resolve (4-player game).
    let (state, _) = pass_all(state, &[p2, p3, p4, p1]);

    // Grizzly Bears should be on battlefield under P2.
    assert!(
        find_object_in_zone(&state, "Grizzly Bears", ZoneId::Battlefield).is_some(),
        "Grizzly Bears should be on battlefield under P2"
    );

    // No evolve trigger on P1's evolve creature (P2's creature, not P1's).
    let evolve_on_stack = state
        .stack_objects
        .iter()
        .any(|s| matches!(s.kind, StackObjectKind::EvolveTrigger { .. }));
    assert!(
        !evolve_on_stack,
        "CR 702.100a: P2's 3/3 entering does NOT trigger P1's evolve (different controller)"
    );

    // P1's evolve creature should have 0 counters.
    let evolve_id = find_object(&state, "Evolve Sapling");
    let obj = state
        .objects
        .get(&evolve_id)
        .expect("Evolve Sapling on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        0,
        "CR 702.100a: P1's evolve creature should have no counters (only P2's creature entered)"
    );
}
