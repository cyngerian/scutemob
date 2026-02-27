//! Undying keyword ability tests (CR 702.93).
//!
//! Undying is a triggered ability: "When this permanent is put into a graveyard from
//! the battlefield, if it had no +1/+1 counters on it, return it to the battlefield
//! under its owner's control with a +1/+1 counter on it."
//!
//! Key rules verified:
//! - Trigger fires on SBA death when creature has no +1/+1 counters (CR 702.93a).
//! - Intervening-if: does NOT trigger when creature already has +1/+1 counters (CR 702.93a).
//! - After returning with +1/+1 counter, second death does NOT trigger undying (CR 702.93a).
//! - Token with undying: trigger fires but token ceases to exist in graveyard (CR 704.5d).
//! - Multiplayer APNAP ordering: multiple simultaneous undying deaths (CR 603.3).
//! - Counter annihilation (CR 704.5q): -1/-1 cancels +1/+1, creature can undying again.

use mtg_engine::{
    check_and_apply_sbas, process_command, CardRegistry, Command, CounterType, GameEvent,
    GameStateBuilder, KeywordAbility, ObjectSpec, PlayerId, StackObjectKind, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_by_name(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_by_name_in_zone(
    state: &mtg_engine::GameState,
    name: &str,
    zone: ZoneId,
) -> Option<mtg_engine::ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Count objects with a given name on the battlefield.
fn count_on_battlefield(state: &mtg_engine::GameState, name: &str) -> usize {
    state
        .objects
        .values()
        .filter(|obj| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .count()
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

// ── Test 1: Basic undying — creature returns with +1/+1 counter ───────────────

#[test]
/// CR 702.93a — Creature with Undying and no +1/+1 counters dies via SBA (lethal damage);
/// undying trigger fires; creature returns to battlefield with one +1/+1 counter.
fn test_undying_basic_returns_with_plus_counter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // 2/2 creature with Undying keyword and lethal damage (2 damage marked on a 2/2).
    let undying_creature = ObjectSpec::creature(p1, "Undying Bear", 2, 2)
        .with_keyword(KeywordAbility::Undying)
        .with_damage(2) // lethal damage → SBA will kill it
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(undying_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pass priority for both players → SBA fires → creature dies → undying trigger queued.
    let (state, events) = pass_all(state, &[p1, p2]);

    // CreatureDied event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.93a: CreatureDied event expected when undying creature dies"
    );

    // Undying trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.93a: undying trigger should be on the stack after creature dies"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::TriggeredAbility { .. }
        ),
        "stack object should be a triggered ability (undying)"
    );

    // Creature is in the graveyard (not on battlefield yet).
    assert!(
        find_by_name_in_zone(&state, "Undying Bear", ZoneId::Graveyard(p1)).is_some(),
        "creature should be in graveyard before trigger resolves"
    );
    assert_eq!(
        count_on_battlefield(&state, "Undying Bear"),
        0,
        "creature should not be on battlefield yet"
    );

    // Both players pass → undying trigger resolves → creature moves from graveyard to battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature should be back on the battlefield with one +1/+1 counter.
    assert_eq!(
        count_on_battlefield(&state, "Undying Bear"),
        1,
        "CR 702.93a: undying creature should be back on the battlefield"
    );
    assert!(
        find_by_name_in_zone(&state, "Undying Bear", ZoneId::Graveyard(p1)).is_none(),
        "creature should NOT be in graveyard after undying resolves"
    );

    // The returned creature has exactly one +1/+1 counter.
    let returned_id = find_by_name(&state, "Undying Bear");
    let returned_obj = state.objects.get(&returned_id).unwrap();
    let plus_counter = returned_obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        plus_counter, 1,
        "CR 702.93a: returned undying creature must have exactly 1 +1/+1 counter"
    );
}

// ── Test 2: Undying does NOT trigger when creature has +1/+1 counter ──────────

#[test]
/// CR 702.93a (intervening-if) — A creature with Undying that already has a +1/+1 counter
/// dies; undying does NOT trigger (intervening-if condition false at trigger time).
fn test_undying_does_not_trigger_with_plus_counter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // 2/2 with Undying and one +1/+1 counter (effective 3/3). Mark 4 damage (lethal for 3 toughness).
    let undying_creature = ObjectSpec::creature(p1, "Spent Wolf", 2, 2)
        .with_keyword(KeywordAbility::Undying)
        .with_counter(CounterType::PlusOnePlusOne, 1)
        .with_damage(4) // lethal for effective 3 toughness
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(undying_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pass priority → SBA fires → creature dies (has +1/+1 counter → undying does NOT trigger).
    let (state, events) = pass_all(state, &[p1, p2]);

    // CreatureDied event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CreatureDied event should be emitted"
    );

    // No undying trigger on the stack.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.93a: undying must NOT trigger when creature had +1/+1 counters"
    );

    // Creature remains in graveyard — NOT returned to battlefield.
    assert!(
        find_by_name_in_zone(&state, "Spent Wolf", ZoneId::Graveyard(p1)).is_some(),
        "creature should be in graveyard (undying did not trigger)"
    );
    assert_eq!(
        count_on_battlefield(&state, "Spent Wolf"),
        0,
        "creature should NOT be on battlefield (undying did not trigger)"
    );
}

// ── Test 3: Second death with +1/+1 counter does not trigger undying ──────────

#[test]
/// CR 702.93a — After an undying creature returns with a +1/+1 counter, if it dies again,
/// undying does NOT trigger (it now has a +1/+1 counter from the first return).
fn test_undying_second_death_no_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // 2/2 with Undying, no counters, lethal damage.
    let undying_creature = ObjectSpec::creature(p1, "Young Wolf", 2, 2)
        .with_keyword(KeywordAbility::Undying)
        .with_damage(2) // lethal
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(undying_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // --- First death ---
    // Pass priority → SBA fires → creature dies → undying trigger queued.
    let (state, events) = pass_all(state, &[p1, p2]);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "First death: CreatureDied should be emitted"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "First death: undying trigger should be on stack"
    );

    // Both players pass → undying trigger resolves → creature returns with +1/+1 counter.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        count_on_battlefield(&state, "Young Wolf"),
        1,
        "First undying: creature should be back on battlefield"
    );

    // Verify the returned creature has a +1/+1 counter.
    let returned_id = find_by_name(&state, "Young Wolf");
    let returned_obj = state.objects.get(&returned_id).unwrap();
    assert_eq!(
        returned_obj
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "Returned creature must have 1 +1/+1 counter"
    );

    // --- Mark lethal damage on the returned 3/3 (2/2 base + 1/+1 counter = 3/3) ---
    // Manually apply 3 damage (lethal for effective 3 toughness).
    let mut state = state;
    let creature_id = find_by_name(&state, "Young Wolf");
    state.objects.get_mut(&creature_id).unwrap().damage_marked = 3;

    // --- Second death ---
    // Pass priority → SBA fires → creature dies (has +1/+1 counter → undying does NOT trigger).
    let (state, events) = pass_all(state, &[p1, p2]);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "Second death: CreatureDied should be emitted"
    );

    // No undying trigger on stack.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.93a: undying must NOT trigger on second death (creature has +1/+1 counter)"
    );

    // Creature stays in graveyard.
    assert!(
        find_by_name_in_zone(&state, "Young Wolf", ZoneId::Graveyard(p1)).is_some(),
        "Creature should be in graveyard after second death"
    );
    assert_eq!(
        count_on_battlefield(&state, "Young Wolf"),
        0,
        "Creature should NOT be on battlefield after second death"
    );
}

// ── Test 4: Token with undying — trigger fires but token ceases to exist ──────

#[test]
/// CR 702.93a + CR 704.5d — A token creature with Undying triggers the ability when it dies,
/// but the token ceases to exist in the graveyard (SBA CR 704.5d) before the trigger resolves.
/// At resolution, MoveZone finds no source in the graveyard → trigger has no effect.
fn test_undying_token_trigger_but_no_return() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Token creature with Undying and lethal damage.
    // .token() marks is_token=true; with_damage makes SBA kill it.
    let undying_token = ObjectSpec::creature(p1, "Wolf Token", 1, 1)
        .token()
        .with_keyword(KeywordAbility::Undying)
        .with_damage(1) // lethal
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(undying_token)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pass priority → SBA1: token dies → CreatureDied → undying trigger queued.
    // SBA2: token ceases to exist in graveyard (CR 704.5d) — checked in second SBA pass.
    let (state, events) = pass_all(state, &[p1, p2]);

    // CreatureDied event should have been emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.93a: CreatureDied should be emitted even for token death"
    );

    // Resolve any triggers that made it to the stack.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    // No Wolf Token should be on the battlefield.
    assert_eq!(
        count_on_battlefield(&state, "Wolf Token"),
        0,
        "CR 704.5d + CR 702.93a: token must not be on battlefield (MoveZone no-op for missing source)"
    );
    // Token also should not be in the graveyard (tokens cease to exist there).
    let in_graveyard = state.objects.values().any(|obj| {
        obj.characteristics.name == "Wolf Token" && matches!(obj.zone, ZoneId::Graveyard(_))
    });
    assert!(
        !in_graveyard,
        "CR 704.5d: token must not persist in graveyard"
    );
}

// ── Test 5: Multiplayer APNAP — two undying creatures die simultaneously ──────

#[test]
/// CR 603.3 — Multiple undying creatures die simultaneously; triggers ordered by APNAP.
/// Both creatures should return to the battlefield after all triggers resolve.
fn test_undying_multiplayer_apnap_ordering() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    // P1 has an undying creature with lethal damage.
    let p1_creature = ObjectSpec::creature(p1, "P1 Undying", 2, 2)
        .with_keyword(KeywordAbility::Undying)
        .with_damage(2)
        .in_zone(ZoneId::Battlefield);

    // P3 has an undying creature with lethal damage.
    let p3_creature = ObjectSpec::creature(p3, "P3 Undying", 2, 2)
        .with_keyword(KeywordAbility::Undying)
        .with_damage(2)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(p1_creature)
        .object(p3_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // All four players pass priority → SBA fires → both creatures die → two undying triggers.
    let (state, events) = pass_all(state, &[p1, p2, p3, p4]);

    // Both CreatureDied events emitted.
    let died_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(
        died_count, 2,
        "CR 702.93a: two CreatureDied events expected (one per undying creature)"
    );

    // Two undying triggers on the stack (APNAP ordered: P1's first, P3's second → P3 resolves first).
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 603.3: two undying triggers should be on the stack"
    );

    // Resolve all triggers: all 4 players pass twice (once per trigger).
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Both creatures should be back on the battlefield.
    assert_eq!(
        count_on_battlefield(&state, "P1 Undying"),
        1,
        "CR 702.93a: P1's undying creature should be back on battlefield"
    );
    assert_eq!(
        count_on_battlefield(&state, "P3 Undying"),
        1,
        "CR 702.93a: P3's undying creature should be back on battlefield"
    );

    // Both returned creatures should have exactly one +1/+1 counter.
    for name in &["P1 Undying", "P3 Undying"] {
        let id = find_by_name(&state, name);
        let obj = state.objects.get(&id).unwrap();
        let counter = obj
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0);
        assert_eq!(
            counter, 1,
            "CR 702.93a: {name} should have exactly 1 +1/+1 counter after undying"
        );
    }
}

// ── Test 6: Counter annihilation — undying creature can loop ──────────────────

#[test]
/// CR 704.5q + CR 702.93a — An undying creature returns with a +1/+1 counter. A -1/-1 counter
/// is added (via manual state mutation to simulate a counter-adding effect). SBA annihilates
/// both counters. Now the creature has no +1/+1 counter and can undying again on next death.
fn test_undying_minus_one_cancellation_enables_second_undying() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // 2/2 with Undying, lethal damage.
    let undying_creature = ObjectSpec::creature(p1, "Geralf-like Wolf", 2, 2)
        .with_keyword(KeywordAbility::Undying)
        .with_damage(2)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(undying_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // --- First death: undying triggers, creature returns with +1/+1 counter ---
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.stack_objects.len(), 1, "Undying trigger on stack");

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        count_on_battlefield(&state, "Geralf-like Wolf"),
        1,
        "Creature back on battlefield after first undying"
    );

    // Verify +1/+1 counter is present.
    let creature_id = find_by_name(&state, "Geralf-like Wolf");
    let obj = state.objects.get(&creature_id).unwrap();
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "Creature has 1 +1/+1 counter after undying"
    );

    // --- Simulate adding a -1/-1 counter (as from an external effect like Graft or Festering Newt) ---
    // Manually add the -1/-1 counter; SBA 704.5q will annihilate both.
    let mut state = state;
    let creature_id = find_by_name(&state, "Geralf-like Wolf");
    {
        let obj = state.objects.get_mut(&creature_id).unwrap();
        let cur = obj
            .counters
            .get(&CounterType::MinusOneMinusOne)
            .copied()
            .unwrap_or(0);
        obj.counters.insert(CounterType::MinusOneMinusOne, cur + 1);
    }

    // Apply SBAs manually to trigger counter annihilation (CR 704.5q).
    let _sba_events = check_and_apply_sbas(&mut state);

    // After SBA, both counters should be gone (annihilated).
    let creature_id = find_by_name(&state, "Geralf-like Wolf");
    let obj = state.objects.get(&creature_id).unwrap();
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        0,
        "CR 704.5q: +1/+1 counter should be annihilated by -1/-1"
    );
    assert_eq!(
        obj.counters
            .get(&CounterType::MinusOneMinusOne)
            .copied()
            .unwrap_or(0),
        0,
        "CR 704.5q: -1/-1 counter should be annihilated by +1/+1"
    );

    // --- Now mark lethal damage again; creature has no +1/+1 counter → undying triggers ---
    // After annihilation, creature is 2/2 again. Mark 2 lethal damage.
    let creature_id = find_by_name(&state, "Geralf-like Wolf");
    state.objects.get_mut(&creature_id).unwrap().damage_marked = 2;

    let (state, events) = pass_all(state, &[p1, p2]);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "Second death: CreatureDied should be emitted"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.93a + CR 704.5q: undying should trigger again after counter annihilation"
    );

    // Resolve the second undying trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        count_on_battlefield(&state, "Geralf-like Wolf"),
        1,
        "Creature returns on second undying"
    );

    // Has +1/+1 counter again.
    let creature_id = find_by_name(&state, "Geralf-like Wolf");
    let obj = state.objects.get(&creature_id).unwrap();
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "Second undying: creature has 1 +1/+1 counter"
    );
}
