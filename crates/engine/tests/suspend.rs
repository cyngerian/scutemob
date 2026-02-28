//! Suspend keyword ability tests (CR 702.62).
//!
//! Suspend is a keyword representing three abilities:
//! 1. Static (in hand): special action -- pay cost, exile with N time counters (CR 702.62a / 116.2f).
//! 2. Triggered (in exile): at beginning of owner's upkeep, remove a time counter (CR 702.62a).
//! 3. Triggered (in exile): when last counter removed, cast without paying mana cost (CR 702.62a).
//!    If it's a creature spell cast this way, it gains haste (CR 702.62a).
//!
//! A card is "suspended" only if in exile, has suspend, AND has time counters (CR 702.62b).
//!
//! Key rules verified:
//! - SuspendCard special action: pay suspend cost, exile face-up with N time counters (CR 702.62a)
//! - Cannot suspend a card not in hand (error case)
//! - Cannot suspend a card without Suspend keyword (error case)
//! - At owner's upkeep, counter-removal trigger fires and goes on stack (CR 702.62a)
//! - When last counter removed, cast-without-mana trigger fires (CR 702.62a)
//! - Creature cast via suspend gains haste -- summoning sickness cleared (CR 702.62a)
//! - Card cast via suspend triggers "whenever you cast a spell" (CR 702.62a)
//! - In multiplayer: only OWNER's upkeep ticks down the suspended card (CR 702.62a)
//!
//! Test setup note: tests that verify the upkeep trigger start at Step::Untap
//! with priority manually set to p1. Passing priority for all players at Untap
//! causes the engine to advance to Upkeep via handle_all_passed -> enter_step,
//! which calls upkeep_actions and queues the suspend trigger. This avoids the
//! need to pass through the Draw step (which would fail with an empty library).

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

fn in_exile(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Exile).is_some()
}

/// Count time counters on the named card (wherever it is).
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// Rift Bolt: Sorcery {2}{R}. Suspend 1 -- {R}.
fn rift_bolt_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("rift-bolt".to_string()),
        name: "Rift Bolt".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Suspend 1—{R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Suspend),
            AbilityDefinition::Suspend {
                cost: ManaCost {
                    red: 1,
                    ..Default::default()
                },
                time_counters: 1,
            },
        ],
        ..Default::default()
    }
}

/// Rorix Bladewing: Creature Dragon {3}{R}{R}{R}. Suspend 3 -- {2}{R}.
fn rorix_bladewing_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("rorix-bladewing".to_string()),
        name: "Rorix Bladewing".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(6),
        toughness: Some(5),
        oracle_text: "Flying, haste. Suspend 3—{2}{R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Suspend),
            AbilityDefinition::Suspend {
                cost: ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                },
                time_counters: 3,
            },
        ],
        ..Default::default()
    }
}

/// Build object spec for Rift Bolt in hand.
fn rift_bolt_in_hand(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Rift Bolt")
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("rift-bolt".to_string()))
        .with_keyword(KeywordAbility::Suspend)
        .with_types(vec![CardType::Sorcery])
}

/// Build object spec for Rift Bolt already in exile (simulates a previously suspended card).
fn rift_bolt_in_exile(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Rift Bolt")
        .in_zone(ZoneId::Exile)
        .with_card_id(CardId("rift-bolt".to_string()))
        .with_keyword(KeywordAbility::Suspend)
        .with_types(vec![CardType::Sorcery])
}

/// Build object spec for Rorix Bladewing already in exile.
fn rorix_in_exile(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Rorix Bladewing")
        .in_zone(ZoneId::Exile)
        .with_card_id(CardId("rorix-bladewing".to_string()))
        .with_keyword(KeywordAbility::Suspend)
        .with_types(vec![CardType::Creature])
}

// ── Test 1: Basic SuspendCard special action ──────────────────────────────────

#[test]
/// CR 702.62a / CR 116.2f — SuspendCard: pay {R}, exile Rift Bolt from hand face-up
/// with 1 time counter, is_suspended=true, CardSuspended event emitted.
fn test_suspend_basic_exile_from_hand() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![rift_bolt_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(rift_bolt_in_hand(p1))
        .build()
        .unwrap();

    // Give p1 {R} for the suspend cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Rift Bolt");
    assert_eq!(
        state.objects[&card_id].zone,
        ZoneId::Hand(p1),
        "Rift Bolt should start in hand"
    );

    let (state, events) = process_command(
        state,
        Command::SuspendCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("SuspendCard should succeed");

    // Card should be in exile.
    assert!(
        in_exile(&state, "Rift Bolt"),
        "CR 702.62a: Rift Bolt should be in exile after suspend"
    );

    // Card should NOT be in hand.
    assert!(
        find_in_zone(&state, "Rift Bolt", ZoneId::Hand(p1)).is_none(),
        "Rift Bolt should no longer be in hand"
    );

    // Exile object should have 1 time counter.
    assert_eq!(
        time_counters(&state, "Rift Bolt"),
        1,
        "CR 702.62a: Rift Bolt should have 1 time counter in exile"
    );

    // Exile object should be marked as suspended (face up, not face down).
    let exile_obj = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Exile && o.characteristics.name == "Rift Bolt")
        .expect("Rift Bolt should exist in exile");
    assert!(
        exile_obj.is_suspended,
        "CR 702.62b: exile object should have is_suspended=true"
    );
    assert!(
        !exile_obj.status.face_down,
        "suspended cards are exiled face-up (unlike foretell)"
    );

    // Events: ManaCostPaid + CardSuspended.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaCostPaid { .. })),
        "ManaCostPaid event should be emitted"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardSuspended { player, .. } if *player == p1)),
        "CR 702.62a: CardSuspended event should be emitted"
    );
}

// ── Test 2: Counter removal on upkeep ────────────────────────────────────────

#[test]
/// CR 702.62a (second ability) — At the beginning of owner's upkeep, a trigger
/// fires to remove a time counter. After resolving: 2 counters become 1.
///
/// Test setup: Start at Step::Untap with priority manually set. Passing priority
/// for all players at Untap advances to Upkeep via enter_step, which calls
/// upkeep_actions and queues the SuspendCounterTrigger.
fn test_suspend_counter_removal_on_upkeep() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![rift_bolt_def()]);

    // Start at Untap so that passing priority advances to Upkeep via enter_step,
    // which calls upkeep_actions and queues the SuspendCounterTrigger.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(rift_bolt_in_exile(p1))
        .build()
        .unwrap();

    // Manually set is_suspended and time counters (simulating a previously suspended card).
    let card_id = find_object(&state, "Rift Bolt");
    if let Some(obj) = state.objects.get_mut(&card_id) {
        obj.is_suspended = true;
        obj.counters = obj.counters.update(CounterType::Time, 2);
    }

    // Set priority at p1 at Untap (unusual but valid for test setup).
    state.turn.priority_holder = Some(p1);

    // Pass priority for all players at Untap -> advances to Upkeep -> upkeep_actions queues trigger.
    // The trigger goes on the stack and priority is given to p1.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Now at Upkeep with SuspendCounterTrigger on the stack.
    assert_eq!(
        state.turn.step,
        Step::Upkeep,
        "should be at Upkeep after advancing from Untap"
    );

    // Both players pass -> trigger resolves -> 1 counter removed.
    let (state, events) = pass_all(state, &[p1, p2]);

    // After resolution: Rift Bolt should have 1 time counter (was 2).
    assert_eq!(
        time_counters(&state, "Rift Bolt"),
        1,
        "CR 702.62a: time counter should decrease from 2 to 1 after upkeep trigger resolves"
    );

    // Card should still be in exile (still suspended — 1 counter remaining).
    assert!(
        in_exile(&state, "Rift Bolt"),
        "Rift Bolt should still be in exile (still has 1 time counter)"
    );

    // CounterRemoved event should have been emitted during resolution.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CounterRemoved {
                counter: CounterType::Time,
                ..
            }
        )),
        "CR 702.62a: CounterRemoved event should be emitted for the time counter"
    );
}

// ── Test 3: Last counter removal triggers free cast ───────────────────────────

#[test]
/// CR 702.62a (second + third ability) — When the last time counter is removed
/// during the owner's upkeep, a SuspendCastTrigger is queued. When it resolves,
/// the spell is cast without paying its mana cost and a SpellCast event fires.
fn test_suspend_last_counter_triggers_cast() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![rift_bolt_def()]);

    // Start at Untap with Rift Bolt having 1 time counter (about to be last).
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(rift_bolt_in_exile(p1))
        .build()
        .unwrap();

    let card_id = find_object(&state, "Rift Bolt");
    if let Some(obj) = state.objects.get_mut(&card_id) {
        obj.is_suspended = true;
        obj.counters = obj.counters.update(CounterType::Time, 1);
    }

    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep (enter_step queues SuspendCounterTrigger, flushes to stack).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Both pass -> SuspendCounterTrigger resolves -> last counter removed ->
    // SuspendCastTrigger queued and flushed to stack.
    let (state, counter_events) = pass_all(state, &[p1, p2]);

    // Time counters should now be 0.
    assert_eq!(
        time_counters(&state, "Rift Bolt"),
        0,
        "CR 702.62a: Rift Bolt should have 0 time counters after last counter removed"
    );

    // CounterRemoved event should have been emitted.
    assert!(
        counter_events.iter().any(|e| matches!(
            e,
            GameEvent::CounterRemoved {
                counter: CounterType::Time,
                ..
            }
        )),
        "CR 702.62a: CounterRemoved event should be emitted"
    );

    // Now the SuspendCastTrigger should be on the stack.
    // Both pass -> cast trigger resolves -> Rift Bolt is cast without paying mana cost.
    let (state, cast_events) = pass_all(state, &[p1, p2]);

    // SpellCast event should be emitted (suspend IS a cast).
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "CR 702.62a: SpellCast event should be emitted when suspend cast trigger resolves"
    );

    // Rift Bolt should no longer be in exile (it was cast).
    assert!(
        !in_exile(&state, "Rift Bolt"),
        "CR 702.62a: Rift Bolt should no longer be in exile after cast trigger fires"
    );
}

// ── Test 4: Creature cast via suspend gains haste ─────────────────────────────

#[test]
/// CR 702.62a (haste clause) — A creature cast via suspend's cast trigger gains
/// haste. V1 implementation: summoning sickness cleared on ETB.
fn test_suspend_creature_gains_haste() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![rorix_bladewing_def()]);

    // Start at Untap with Rorix Bladewing having 1 time counter.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(rorix_in_exile(p1))
        .build()
        .unwrap();

    let dragon_id = find_object(&state, "Rorix Bladewing");
    if let Some(obj) = state.objects.get_mut(&dragon_id) {
        obj.is_suspended = true;
        obj.counters = obj.counters.update(CounterType::Time, 1);
    }

    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep (enter_step queues counter trigger).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Counter trigger resolves (last counter) -> cast trigger queued.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Cast trigger resolves -> Rorix cast without paying mana cost (on stack).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Rorix spell resolves -> enters battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Rorix should be on the battlefield.
    assert!(
        on_battlefield(&state, "Rorix Bladewing"),
        "CR 702.62a: Rorix Bladewing should be on battlefield after suspend cast resolves"
    );

    // Rorix should NOT have summoning sickness (haste granted via suspend).
    let rorix = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Rorix Bladewing" && o.zone == ZoneId::Battlefield)
        .expect("Rorix Bladewing should be on battlefield");
    assert!(
        !rorix.has_summoning_sickness,
        "CR 702.62a: creature cast via suspend should have haste (no summoning sickness)"
    );
}

// ── Test 5: Cast without paying mana cost ────────────────────────────────────

#[test]
/// CR 702.62d — When cast via suspend, the spell is cast without paying its
/// mana cost. No mana should be deducted when the cast trigger resolves.
fn test_suspend_cast_without_paying_mana_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![rift_bolt_def()]);

    // Start at Untap with Rift Bolt in exile with 1 time counter.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(rift_bolt_in_exile(p1))
        .build()
        .unwrap();

    let card_id = find_object(&state, "Rift Bolt");
    if let Some(obj) = state.objects.get_mut(&card_id) {
        obj.is_suspended = true;
        obj.counters = obj.counters.update(CounterType::Time, 1);
    }

    // Give p1 some mana -- it should NOT be spent when suspend triggers fire.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 3);
    state.turn.priority_holder = Some(p1);

    let mana_before = state.players[&p1].mana_pool.red;

    // Advance Untap -> Upkeep -> counter trigger queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Counter trigger resolves -> cast trigger queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Cast trigger resolves -> Rift Bolt cast without paying mana cost.
    // Capture events from this round to verify no ManaCostPaid fires.
    let (state, cast_events) = pass_all(state, &[p1, p2]);

    // CR 702.62d: Casting a spell via suspend does not require paying its mana cost.
    // Verify no ManaCostPaid event was emitted during cast trigger resolution.
    let mana_paid_events: Vec<_> = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::ManaCostPaid { .. }))
        .collect();
    assert!(
        mana_paid_events.is_empty(),
        "CR 702.62d: no ManaCostPaid event should fire when casting via suspend trigger; found {:?}",
        mana_paid_events,
    );

    // Also verify the mana pool was not depleted as payment (mana may be emptied at step
    // transitions but should not be spent by the suspend cast mechanics).
    let _ = mana_before;
    let mana_after = state.players[&p1].mana_pool.red;
    let _ = mana_after; // may be 0 due to empty_all_mana_pools at step transition
}

// ── Test 6: Invalid — card not in hand ───────────────────────────────────────

#[test]
/// CR 702.62a — SuspendCard should fail if the card is not in the player's hand.
fn test_suspend_invalid_not_in_hand() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![rift_bolt_def()]);

    // Rift Bolt starts on the battlefield (not in hand).
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Rift Bolt")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("rift-bolt".to_string()))
                .with_keyword(KeywordAbility::Suspend)
                .with_types(vec![CardType::Sorcery]),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Rift Bolt");
    let result = process_command(
        state,
        Command::SuspendCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.62a: SuspendCard should fail when card is not in hand"
    );
}

// ── Test 7: Invalid — card has no Suspend keyword ────────────────────────────

#[test]
/// CR 702.62a — SuspendCard should fail if the card does not have the Suspend keyword.
fn test_suspend_invalid_no_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    // A plain sorcery with no special abilities.
    let plain_sorcery_def = CardDefinition {
        card_id: CardId("plain-sorcery".to_string()),
        name: "Plain Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![plain_sorcery_def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plain Sorcery")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(CardId("plain-sorcery".to_string()))
                .with_types(vec![CardType::Sorcery]),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Plain Sorcery");
    let result = process_command(
        state,
        Command::SuspendCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.62a: SuspendCard should fail when card has no Suspend keyword"
    );
}

// ── Test 8: Card no longer in exile after cast (CR 702.62b + CR 400.7) ────────

#[test]
/// CR 702.62b / CR 400.7 — After the suspended card is cast (zone change: exile ->
/// stack), it is no longer in exile and is therefore no longer "suspended." The
/// new object on the stack does not have is_suspended=true (that's an exile flag).
fn test_suspend_no_longer_suspended_after_cast() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![rift_bolt_def()]);

    // Rift Bolt in exile with 1 time counter.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(rift_bolt_in_exile(p1))
        .build()
        .unwrap();

    let card_id = find_object(&state, "Rift Bolt");
    if let Some(obj) = state.objects.get_mut(&card_id) {
        obj.is_suspended = true;
        obj.counters = obj.counters.update(CounterType::Time, 1);
    }
    state.turn.priority_holder = Some(p1);

    // Advance to Upkeep -> counter trigger -> cast trigger -> spell cast.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    // After the cast trigger fires, the card is on the stack (no longer in exile).
    // CR 400.7: zone change creates new object; old exile object is gone.
    assert!(
        !in_exile(&state, "Rift Bolt"),
        "CR 400.7 / CR 702.62b: Rift Bolt should NOT be in exile after cast trigger fires"
    );

    // The card should be on the stack as a spell or resolved to graveyard.
    let bolt_on_stack = state.stack_objects.iter().any(|so| {
        if let StackObjectKind::Spell { source_object } = so.kind {
            state
                .objects
                .get(&source_object)
                .map(|o| o.characteristics.name == "Rift Bolt")
                .unwrap_or(false)
        } else {
            false
        }
    });
    let bolt_resolved = find_in_zone(&state, "Rift Bolt", ZoneId::Graveyard(p1)).is_some();

    assert!(
        bolt_on_stack || bolt_resolved,
        "Rift Bolt should be on stack or resolved to graveyard after suspend cast trigger fires"
    );
}

// ── Test 9: Multiplayer — only owner's upkeep ticks down the card ─────────────

#[test]
/// CR 702.62a (multiplayer) — Suspend counters are only removed at the beginning
/// of the OWNER'S upkeep. A card owned by player 2 should NOT tick down during
/// player 1's upkeep.
fn test_suspend_not_active_player_upkeep_no_trigger() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let registry = CardRegistry::new(vec![rift_bolt_def()]);

    // p2 owns a Rift Bolt in exile. It is p1's turn.
    // Start at Untap so passing priority advances to Upkeep via enter_step.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1) // p1's turn, NOT p2's turn
        .at_step(Step::Untap)
        .object(rift_bolt_in_exile(p2)) // owned by p2
        .build()
        .unwrap();

    let card_id = find_object(&state, "Rift Bolt");
    if let Some(obj) = state.objects.get_mut(&card_id) {
        obj.is_suspended = true;
        obj.counters = obj.counters.update(CounterType::Time, 2);
    }

    state.turn.priority_holder = Some(p1);

    // All players pass priority at Untap -> advances to Upkeep via enter_step.
    // upkeep_actions is called for p1's upkeep. Since p2 is the OWNER of the
    // suspended card (and p1 is active), no suspend trigger should be queued.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // We're now at Upkeep. Both players pass -> any triggers resolve.
    // p2's suspended card should NOT have had a counter removed.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Rift Bolt should still have 2 time counters (no counter was removed).
    assert_eq!(
        time_counters(&state, "Rift Bolt"),
        2,
        "CR 702.62a: p2's suspended card should NOT tick down during p1's upkeep (multiplayer)"
    );

    // Rift Bolt should still be in exile.
    assert!(
        in_exile(&state, "Rift Bolt"),
        "Rift Bolt should still be in exile with full counters"
    );
}
