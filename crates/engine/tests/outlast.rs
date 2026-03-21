//! Outlast keyword ability tests (CR 702.107).
//!
//! Outlast is an activated ability that functions while the creature is on the battlefield.
//! "Outlast [cost]" means "[Cost], {T}: Put a +1/+1 counter on this creature.
//! Activate only as a sorcery." (CR 702.107a)
//!
//! Key rules verified:
//! - Basic: activating Outlast adds a +1/+1 counter on the source (CR 702.107a).
//! - Sorcery-speed restriction: active player only, main phase, empty stack (CR 702.107a / CR 602.5d).
//! - Summoning sickness: requires {T}, so a creature with summoning sickness cannot activate
//!   Outlast unless it has haste (Ruling 2014-09-20 / CR 302.6).
//! - Requires mana payment; error on insufficient mana (CR 602.2b).
//! - Cannot activate if source is already tapped (CR 602.2).
//! - Stacks: activating Outlast across turns accumulates counters (CR 702.107a).
//! - Not a cast: spells_cast_this_turn is unchanged after Outlast activation (CR 602.2).

use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    process_command, CardEffectTarget, Command, CounterType, Effect, GameEvent, GameStateBuilder,
    KeywordAbility, ManaColor, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
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

/// Build an Outlast activated ability with the given mana cost.
///
/// CR 702.107a: "[Cost], {T}: Put a +1/+1 counter on this creature. Activate only as a sorcery."
fn outlast_ability(mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        targets: vec![],
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: Some(mana),
            sacrifice_self: false,
            discard_card: false,
            discard_self: false,
            forage: false,
            sacrifice_filter: None,
        },
        description: "Outlast (CR 702.107a)".to_string(),
        effect: Some(Effect::AddCounter {
            target: CardEffectTarget::Source,
            counter: CounterType::PlusOnePlusOne,
            count: 1,
        }),
        sorcery_speed: true,
            activation_condition: None,
    }
}

/// Build a creature with Outlast on the battlefield with the given mana pool.
/// The creature has `has_summoning_sickness = false` (simulating entering a prior turn).
fn build_outlast_state(
    owner: PlayerId,
    name: &str,
    outlast_cost: ManaCost,
    mana_pool: ManaPool,
) -> mtg_engine::GameState {
    let creature = ObjectSpec::creature(owner, name, 2, 2)
        .with_keyword(KeywordAbility::Outlast)
        .with_activated_ability(outlast_ability(outlast_cost));

    let mut state = GameStateBuilder::four_player()
        .active_player(owner)
        .add_player_with(owner, |pb| pb.mana(mana_pool))
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    // Clear summoning sickness so tap cost is payable.
    let ids: Vec<_> = state
        .objects
        .iter()
        .filter(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .collect();
    for id in ids {
        if let Some(obj) = state.objects.get_mut(&id) {
            obj.has_summoning_sickness = false;
        }
    }
    state
}

// ── Test 1: Basic Outlast adds a +1/+1 counter ───────────────────────────────

#[test]
/// CR 702.107a — Creature with Outlast {1} on the battlefield; player pays {1} + taps.
/// After resolution, the creature has 1 +1/+1 counter.
fn test_outlast_basic_adds_counter() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let outlast_cost = ManaCost {
        generic: 1,
        ..Default::default()
    };
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Colorless, 1);

    let state = build_outlast_state(p1, "Outlast Warrior", outlast_cost, pool);
    let source_id = find_object(&state, "Outlast Warrior");

    // p1 activates Outlast.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .unwrap();

    // All four players pass priority → ability resolves.
    let (state, events) = pass_all(state, &[p1, p2, p3, p4]);

    // CR 702.107a: creature should have 1 +1/+1 counter.
    let source_id = find_object_on_battlefield(&state, "Outlast Warrior")
        .expect("Outlast Warrior should still be on the battlefield");
    assert_eq!(
        get_plus_counters(&state, source_id),
        1,
        "CR 702.107a: Outlast should place 1 +1/+1 counter on the source"
    );

    // CounterAdded event should have been emitted.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 1,
                ..
            }
        )),
        "CR 702.107a: CounterAdded event should be emitted with count=1"
    );

    // AbilityActivated was emitted.
    // (collected earlier — check the global events across both phases is not tracked here;
    // the activation event is sufficient for correctness.)
}

// ── Test 2: Sorcery-speed restriction ────────────────────────────────────────

#[test]
/// CR 702.107a / CR 602.5d — Outlast can only be activated as a sorcery (main phase,
/// active player, empty stack). Attempting in an end step should return an error.
fn test_outlast_sorcery_speed_restriction() {
    let p1 = p(1);

    let outlast_cost = ManaCost {
        generic: 1,
        ..Default::default()
    };
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Colorless, 1);

    // Construct state in end step instead of main phase.
    let creature = ObjectSpec::creature(p1, "Outlast Warrior", 2, 2)
        .with_keyword(KeywordAbility::Outlast)
        .with_activated_ability(outlast_ability(outlast_cost));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .add_player_with(p1, |pb| pb.mana(pool))
        .at_step(Step::End)
        .object(creature)
        .build()
        .unwrap();

    // Clear summoning sickness.
    let ids: Vec<_> = state
        .objects
        .iter()
        .filter(|(_, obj)| obj.characteristics.name == "Outlast Warrior")
        .map(|(id, _)| *id)
        .collect();
    for id in ids {
        if let Some(obj) = state.objects.get_mut(&id) {
            obj.has_summoning_sickness = false;
        }
    }

    let source_id = find_object(&state, "Outlast Warrior");

    // Activating in end step should fail (sorcery-speed restriction).
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.5d: Outlast must fail when not in a main phase with empty stack"
    );
}

// ── Test 3: Summoning sickness prevents activation ────────────────────────────

#[test]
/// Ruling 2014-09-20 / CR 302.6 — Outlast requires {T}. A creature that entered
/// this turn (has_summoning_sickness = true) cannot activate Outlast.
fn test_outlast_summoning_sickness_prevents_activation() {
    let p1 = p(1);

    let outlast_cost = ManaCost {
        generic: 1,
        ..Default::default()
    };
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Colorless, 1);

    // Build with summoning sickness still active (don't clear it).
    let creature = ObjectSpec::creature(p1, "Fresh Outlaster", 2, 2)
        .with_keyword(KeywordAbility::Outlast)
        .with_activated_ability(outlast_ability(outlast_cost));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .add_player_with(p1, |pb| pb.mana(pool))
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    // The builder places permanents without summoning sickness by default (they are treated
    // as having been on the battlefield since turn start). Manually set sickness to simulate
    // a creature that entered this turn (CR 302.6 / Ruling 2014-09-20).
    let source_id = find_object(&state, "Fresh Outlaster");
    if let Some(obj) = state.objects.get_mut(&source_id) {
        obj.has_summoning_sickness = true;
    }

    // Activation should fail: tap cost + summoning sickness.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 302.6: Outlast should fail when source has summoning sickness"
    );
}

// ── Test 4: Insufficient mana returns error ───────────────────────────────────

#[test]
/// CR 602.2b — Outlast cost includes mana. If the player cannot pay, the activation fails.
fn test_outlast_requires_mana() {
    let p1 = p(1);

    let outlast_cost = ManaCost {
        generic: 2,
        ..Default::default()
    };
    // Give the player only 1 generic mana — not enough for {2}.
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Colorless, 1);

    let state = build_outlast_state(p1, "Outlast Warrior", outlast_cost, pool);
    let source_id = find_object(&state, "Outlast Warrior");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2b: Outlast should fail when player cannot pay the mana cost"
    );
}

// ── Test 5: Cannot activate if source is already tapped ──────────────────────

#[test]
/// CR 602.2 — Outlast requires {T} in its cost. A creature that is already tapped
/// cannot activate Outlast.
fn test_outlast_already_tapped() {
    let p1 = p(1);

    let outlast_cost = ManaCost {
        generic: 1,
        ..Default::default()
    };
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Colorless, 1);

    let mut state = build_outlast_state(p1, "Outlast Warrior", outlast_cost, pool);

    // Tap the creature manually.
    let source_id = find_object(&state, "Outlast Warrior");
    if let Some(obj) = state.objects.get_mut(&source_id) {
        obj.status.tapped = true;
    }

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2: Outlast should fail when source is already tapped"
    );
}

// ── Test 6: Counters stack across multiple activations ────────────────────────

#[test]
/// CR 702.107a — Outlast can be activated once per turn (once each turn the creature
/// untaps). Simulated by activating, resolving, then manually resetting tap state
/// and refilling mana, then activating again. The creature accumulates 2 counters.
fn test_outlast_stacks_counters() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let outlast_cost = ManaCost {
        generic: 1,
        ..Default::default()
    };
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Colorless, 1);

    let state = build_outlast_state(p1, "Outlast Veteran", outlast_cost.clone(), pool);
    let source_id = find_object(&state, "Outlast Veteran");

    // ── First activation ──
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .unwrap();

    let (mut state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Verify 1 counter.
    let source_id = find_object_on_battlefield(&state, "Outlast Veteran")
        .expect("Outlast Veteran should be on battlefield after first activation");
    assert_eq!(
        get_plus_counters(&state, source_id),
        1,
        "After first Outlast: 1 counter"
    );

    // Reset for second activation: untap and refill mana.
    if let Some(obj) = state.objects.get_mut(&source_id) {
        obj.status.tapped = false;
    }
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    // ── Second activation ──
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Verify 2 cumulative counters.
    let source_id = find_object_on_battlefield(&state, "Outlast Veteran")
        .expect("Outlast Veteran should be on battlefield after second activation");
    assert_eq!(
        get_plus_counters(&state, source_id),
        2,
        "CR 702.107a: After two Outlast activations, creature should have 2 +1/+1 counters"
    );
}

// ── Test 7: Outlast is not a cast ─────────────────────────────────────────────

#[test]
/// CR 602.2 — Outlast is an activated ability, not a spell. Activating it does not
/// increment spells_cast_this_turn and does not emit a SpellCast event.
fn test_outlast_not_a_cast() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let outlast_cost = ManaCost {
        generic: 1,
        ..Default::default()
    };
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Colorless, 1);

    let state = build_outlast_state(p1, "Outlast Monk", outlast_cost, pool);
    let source_id = find_object(&state, "Outlast Monk");

    let spells_before = state
        .players
        .get(&p1)
        .map(|ps| ps.spells_cast_this_turn)
        .unwrap_or(0);

    let (state, activate_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .unwrap();

    let (state, resolve_events) = pass_all(state, &[p1, p2, p3, p4]);

    let all_events: Vec<_> = activate_events
        .iter()
        .chain(resolve_events.iter())
        .collect();

    // No SpellCast event should be emitted.
    assert!(
        !all_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "CR 602.2: Outlast is an activated ability; no SpellCast event expected"
    );

    // spells_cast_this_turn should be unchanged.
    let spells_after = state
        .players
        .get(&p1)
        .map(|ps| ps.spells_cast_this_turn)
        .unwrap_or(0);
    assert_eq!(
        spells_after, spells_before,
        "CR 602.2: spells_cast_this_turn should be unchanged after Outlast activation"
    );
}
