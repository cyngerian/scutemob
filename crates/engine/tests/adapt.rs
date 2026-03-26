//! Adapt keyword ability tests (CR 701.46).
//!
//! Adapt is a keyword action (CR 701.46), not a keyword ability (CR 702.xx).
//! Cards with Adapt present it as an activated ability: "{cost}: Adapt N."
//! "Adapt N" means "If this permanent has no +1/+1 counters on it, put N +1/+1
//! counters on it." (CR 701.46a)
//!
//! Key rules verified:
//! - Adapt N places N +1/+1 counters when the source has no +1/+1 counters (CR 701.46a).
//! - Adapt does NOT place counters when the source already has +1/+1 counters (CR 701.46a).
//! - Activation is ALWAYS legal even if the creature has counters; check happens at
//!   resolution time, not activation time (ruling 2019-01-25).
//! - After losing all +1/+1 counters, the creature can adapt again (ruling 2019-01-25).
//! - Mana cost is paid at activation time (CR 602.2).
//! - CounterAdded event is emitted when adapt successfully places counters.

use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    calculate_characteristics, process_command, CardEffectTarget, Command, Condition, CounterType,
    Effect, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ManaPool, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
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

fn find_object_on_battlefield(state: &mtg_engine::GameState, name: &str) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
}

/// Get the +1/+1 counter count on an object.
fn get_plus_counters(state: &mtg_engine::GameState, id: ObjectId) -> u32 {
    state
        .objects
        .get(&id)
        .and_then(|obj| obj.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0)
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

/// Build an Adapt N activated ability with the given mana cost.
///
/// Effect: Conditional — if source has no +1/+1 counters, add N +1/+1 counters.
/// The condition check happens at resolution time, not activation time (ruling 2019-01-25).
fn adapt_ability(adapt_n: u32, mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        targets: vec![],
        cost: ActivationCost {
            requires_tap: false,
            mana_cost: Some(mana),
            sacrifice_self: false,
            discard_card: false,
            discard_self: false,
            forage: false,
            sacrifice_filter: None,
            remove_counter_cost: None,
        },
        description: format!("Adapt {adapt_n} (CR 701.46a)"),
        effect: Some(Effect::Conditional {
            condition: Condition::SourceHasNoCountersOfType {
                counter: CounterType::PlusOnePlusOne,
            },
            if_true: Box::new(Effect::AddCounter {
                target: CardEffectTarget::Source,
                counter: CounterType::PlusOnePlusOne,
                count: adapt_n,
            }),
            if_false: Box::new(Effect::Nothing),
        }),
        sorcery_speed: false,
        activation_condition: None,
    }
}

/// Build a creature with an Adapt N activated ability and enough mana in the pool.
fn build_adapt_state_with_mana(
    owner: PlayerId,
    name: &str,
    power: i32,
    toughness: i32,
    adapt_n: u32,
    mana: ManaCost,
    mana_pool: ManaPool,
) -> mtg_engine::GameState {
    let creature = ObjectSpec::creature(owner, name, power, toughness)
        .with_keyword(KeywordAbility::Adapt(adapt_n))
        .with_activated_ability(adapt_ability(adapt_n, mana))
        .in_zone(ZoneId::Battlefield);

    GameStateBuilder::four_player()
        .active_player(owner)
        .add_player_with(owner, |pb| pb.mana(mana_pool))
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap()
}

// ── Test 1: Basic adapt adds counters ────────────────────────────────────────

#[test]
/// CR 701.46a — Adapt N places N +1/+1 counters when the source has no +1/+1 counters.
/// A creature with Adapt 2 and no +1/+1 counters activates. After resolution,
/// the creature has exactly 2 +1/+1 counters.
fn test_adapt_basic_adds_counters() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let cost = ManaCost {
        generic: 2,
        green: 1,
        ..Default::default()
    };
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Green, 1);
    pool.add(ManaColor::Colorless, 2);

    let state = build_adapt_state_with_mana(p1, "Adapt Crawler", 2, 2, 2, cost, pool);

    let source_id = find_object(&state, "Adapt Crawler");

    // Activate adapt — ability goes on the stack, mana is paid.
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
    .unwrap();

    // All four players pass priority → ability resolves.
    let (state, events) = pass_all(state, &[p1, p2, p3, p4]);

    // CR 701.46a: creature should have 2 +1/+1 counters.
    let source_id = find_object_on_battlefield(&state, "Adapt Crawler")
        .expect("Adapt Crawler should still be on the battlefield");
    assert_eq!(
        get_plus_counters(&state, source_id),
        2,
        "CR 701.46a: Adapt 2 should place 2 +1/+1 counters when no counters are present"
    );

    // CounterAdded event should have been emitted.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 2,
                ..
            }
        )),
        "CR 701.46a: CounterAdded event should be emitted with count=2"
    );

    // Layer-aware P/T should be 4/4 (2+2 / 2+2).
    let chars = calculate_characteristics(&state, source_id)
        .expect("Adapt Crawler should have characteristics");
    assert_eq!(
        chars.power,
        Some(4),
        "CR 701.46a: power should be 4 (2 base + 2 counters)"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "CR 701.46a: toughness should be 4 (2 base + 2 counters)"
    );
}

// ── Test 2: Adapt does nothing when source already has +1/+1 counters ─────────

#[test]
/// CR 701.46a (ruling 2019-01-25) — "If the creature has a +1/+1 counter on it
/// for any reason, you simply won't put any +1/+1 counters on it."
/// A creature with 1 existing +1/+1 counter activates adapt. After resolution,
/// the counter count remains 1 (no additional counters placed).
fn test_adapt_does_nothing_with_existing_counters() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let cost = ManaCost {
        generic: 2,
        green: 1,
        ..Default::default()
    };
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Green, 1);
    pool.add(ManaColor::Colorless, 2);

    // Pre-load the creature with 1 +1/+1 counter.
    let creature = ObjectSpec::creature(p1, "Already Adapted", 2, 2)
        .with_keyword(KeywordAbility::Adapt(2))
        .with_activated_ability(adapt_ability(2, cost))
        .with_counter(CounterType::PlusOnePlusOne, 1)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .add_player_with(p1, |pb| pb.mana(pool))
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Already Adapted");
    assert_eq!(
        get_plus_counters(&state, source_id),
        1,
        "precondition: creature starts with 1 +1/+1 counter"
    );

    // Activate adapt (legal even with counters — ruling 2019-01-25).
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
    .unwrap();

    // All four players pass priority → ability resolves.
    let (state, events) = pass_all(state, &[p1, p2, p3, p4]);

    // CR 701.46a: counter count should still be 1 — no additional counters placed.
    let source_id = find_object_on_battlefield(&state, "Already Adapted")
        .expect("Already Adapted should still be on the battlefield");
    assert_eq!(
        get_plus_counters(&state, source_id),
        1,
        "CR 701.46a: adapt should not place additional counters when creature already has +1/+1 counters"
    );

    // No CounterAdded event should have been emitted.
    assert!(
        !events.iter().any(|e| matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                ..
            }
        )),
        "CR 701.46a: CounterAdded should NOT be emitted when condition is false"
    );
}

// ── Test 3: Activation is always legal (ruling 2019-01-25) ───────────────────

#[test]
/// CR 701.46a (ruling 2019-01-25) — "You can always activate an ability that will
/// cause a creature to adapt." Activation is legal even when the creature already
/// has +1/+1 counters. The ability goes on the stack and the mana cost is paid.
fn test_adapt_activation_always_legal() {
    let p1 = p(1);

    let cost = ManaCost {
        generic: 1,
        green: 1,
        ..Default::default()
    };
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Green, 1);
    pool.add(ManaColor::Colorless, 1);

    // Creature already has a +1/+1 counter.
    let creature = ObjectSpec::creature(p1, "Stubborn Adaptor", 3, 3)
        .with_keyword(KeywordAbility::Adapt(1))
        .with_activated_ability(adapt_ability(1, cost))
        .with_counter(CounterType::PlusOnePlusOne, 1)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .add_player_with(p1, |pb| pb.mana(pool))
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Stubborn Adaptor");

    // Activation should succeed even though the creature has +1/+1 counters.
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
    .unwrap();

    // The ability should be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "ruling 2019-01-25: ability should be on the stack even when creature has +1/+1 counters"
    );

    // Mana was consumed (both generic and green).
    assert_eq!(
        state.players.get(&p1).unwrap().mana_pool.total(),
        0,
        "ruling 2019-01-25: mana cost should be paid at activation time"
    );

    // ManaCostPaid event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaCostPaid { .. })),
        "CR 602.2: ManaCostPaid event should be emitted"
    );
}

// ── Test 4: Can adapt again after losing all counters ────────────────────────

#[test]
/// Ruling 2019-01-25 — "If a creature somehow loses all of its +1/+1 counters,
/// it can adapt again and get more +1/+1 counters."
/// A creature adapts (gets N counters), then all counters are removed, then adapts again.
fn test_adapt_after_losing_counters() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let cost = ManaCost {
        generic: 2,
        green: 1,
        ..Default::default()
    };

    let creature = ObjectSpec::creature(p1, "Resilient Crawler", 2, 2)
        .with_keyword(KeywordAbility::Adapt(2))
        .with_activated_ability(adapt_ability(2, cost.clone()))
        .in_zone(ZoneId::Battlefield);

    let mut pool = ManaPool::default();
    pool.add(ManaColor::Green, 1);
    pool.add(ManaColor::Colorless, 2);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .add_player_with(p1, |pb| pb.mana(pool))
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Resilient Crawler");

    // First adapt: activate → resolve → creature gets 2 +1/+1 counters.
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
    .unwrap();
    let (mut state, _) = pass_all(state, &[p1, p2, p3, p4]);

    let source_id = find_object_on_battlefield(&state, "Resilient Crawler").unwrap();
    assert_eq!(
        get_plus_counters(&state, source_id),
        2,
        "precondition: creature should have 2 +1/+1 counters after first adapt"
    );

    // Remove all +1/+1 counters via direct effect execution.
    let remove_effect = Effect::RemoveCounter {
        target: CardEffectTarget::Source,
        counter: CounterType::PlusOnePlusOne,
        count: 2,
    };
    let mut ctx = EffectContext::new(p1, source_id, vec![]);
    execute_effect(&mut state, &remove_effect, &mut ctx);

    let source_id = find_object_on_battlefield(&state, "Resilient Crawler").unwrap();
    assert_eq!(
        get_plus_counters(&state, source_id),
        0,
        "precondition: creature should have 0 +1/+1 counters after removal"
    );

    // Replenish mana for second activation.
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
        .add(ManaColor::Colorless, 2);

    // Second adapt: activate → resolve → creature gets 2 +1/+1 counters again.
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
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    let source_id = find_object_on_battlefield(&state, "Resilient Crawler").unwrap();
    assert_eq!(
        get_plus_counters(&state, source_id),
        2,
        "ruling 2019-01-25: creature should have 2 +1/+1 counters after adapting again"
    );
}

// ── Test 5: Mana cost is paid at activation time ─────────────────────────────

#[test]
/// CR 602.2 — Mana cost is paid when the activated ability is placed on the stack.
/// P1 has exactly enough mana for Adapt 1 ({1}{G}). After activation, the mana pool
/// is empty. If P1 had insufficient mana, activation would fail.
fn test_adapt_pays_mana_cost() {
    let p1 = p(1);

    let cost = ManaCost {
        generic: 1,
        green: 1,
        ..Default::default()
    };

    // Exactly enough mana: 1 green + 1 generic (colorless).
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Green, 1);
    pool.add(ManaColor::Colorless, 1);

    let creature = ObjectSpec::creature(p1, "Cost Checker", 2, 2)
        .with_keyword(KeywordAbility::Adapt(1))
        .with_activated_ability(adapt_ability(1, cost))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .add_player_with(p1, |pb| pb.mana(pool))
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Cost Checker");

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
    .unwrap();

    // CR 602.2: mana pool should be completely drained.
    assert_eq!(
        state.players.get(&p1).unwrap().mana_pool.total(),
        0,
        "CR 602.2: mana pool should be drained after paying activation cost"
    );

    // ManaCostPaid event should be emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaCostPaid { .. })),
        "CR 602.2: ManaCostPaid event should be emitted"
    );

    // Ability is on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "CR 602.2: ability should be on the stack after activation"
    );
}

// ── Test 6: CounterAdded event emitted on successful adapt ────────────────────

#[test]
/// CR 701.46a — When adapt successfully places +1/+1 counters, a CounterAdded event
/// is emitted. This is the event that card-specific triggers (like Sharktocrab's
/// "whenever one or more +1/+1 counters are put on this creature") listen to.
fn test_adapt_counter_added_event_emitted() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let cost = ManaCost {
        generic: 2,
        blue: 1,
        ..Default::default()
    };
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Blue, 1);
    pool.add(ManaColor::Colorless, 2);

    let state = build_adapt_state_with_mana(p1, "Sharktocrab Clone", 3, 3, 1, cost, pool);

    let source_id = find_object(&state, "Sharktocrab Clone");

    // Activate adapt.
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
    .unwrap();

    // All players pass → ability resolves.
    let (state, events) = pass_all(state, &[p1, p2, p3, p4]);

    // CR 701.46a: CounterAdded event should be emitted with correct fields.
    let counter_added = events.iter().find(|e| {
        matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 1,
                ..
            }
        )
    });
    assert!(
        counter_added.is_some(),
        "CR 701.46a: CounterAdded event should be emitted when adapt successfully places counters"
    );

    // The creature should have 1 +1/+1 counter.
    let source_id = find_object_on_battlefield(&state, "Sharktocrab Clone")
        .expect("Sharktocrab Clone should still be on the battlefield");
    assert_eq!(
        get_plus_counters(&state, source_id),
        1,
        "CR 701.46a: creature should have 1 +1/+1 counter after adapt resolves"
    );
}
