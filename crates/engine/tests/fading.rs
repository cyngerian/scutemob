//! Fading keyword ability tests (CR 702.32).
//!
//! Fading N represents two abilities (CR 702.32a):
//! 1. Replacement (ETB): enters with N fade counters on it.
//! 2. Triggered (upkeep): at beginning of controller's upkeep, remove a fade counter
//!    from this permanent. If you can't, sacrifice the permanent.
//!
//! Unlike Vanishing, Fading's upkeep trigger is a SINGLE trigger that handles both
//! counter removal AND sacrifice. There is no intervening-if condition -- the trigger
//! fires unconditionally. At resolution: if counters > 0, remove one; if 0, sacrifice.
//!
//! CR 702.32a: Fading always has a number (no "Fading without a number" like Vanishing).
//!
//! Lifecycle: Fading N survives N upkeeps of counter removal, then is sacrificed on the
//! (N+1)th upkeep when it can't remove a counter.
//!
//! Test setup note: Tests that verify the upkeep trigger start at Step::Untap with
//! priority manually set. Passing priority for all players at Untap causes the engine
//! to advance to Upkeep via handle_all_passed -> enter_step, which calls upkeep_actions
//! and queues the FadingTrigger. This avoids the need to pass through Draw
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

/// Count fade counters on the named object (wherever it is).
fn fade_counters(state: &GameState, name: &str) -> u32 {
    state
        .objects
        .values()
        .find(|o| o.characteristics.name == name)
        .and_then(|o| o.counters.get(&CounterType::Fade).copied())
        .unwrap_or(0)
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

/// Test Fading Creature: Creature {2}{G}. Fading 3. 5/5.
fn fading_3_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-fading-3".into()),
        name: "Test Fading 3 Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Fading 3".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Fading(3)),
            AbilityDefinition::Fading { count: 3 },
        ],
        power: Some(5),
        toughness: Some(5),
        ..Default::default()
    }
}

/// Test Fading Creature: Creature {1}{G}. Fading 2. 2/2.
fn fading_2_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-fading-2".into()),
        name: "Test Fading 2 Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Fading 2".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Fading(2)),
            AbilityDefinition::Fading { count: 2 },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Test Fading Enchantment: Enchantment {2}{W}. Fading 2.
fn fading_2_enchantment_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-fading-enchant".into()),
        name: "Test Fading Enchantment".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Fading 2".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Fading(2)),
            AbilityDefinition::Fading { count: 2 },
        ],
        power: None,
        toughness: None,
        ..Default::default()
    }
}

/// ObjectSpec for Fading 3 creature on the battlefield.
fn fading_3_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Fading 3 Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("test-fading-3".into()))
        .with_keyword(KeywordAbility::Fading(3))
        .with_types(vec![CardType::Creature])
}

/// ObjectSpec for Fading 2 creature on the battlefield.
fn fading_2_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Fading 2 Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("test-fading-2".into()))
        .with_keyword(KeywordAbility::Fading(2))
        .with_types(vec![CardType::Creature])
}

/// ObjectSpec for Fading 2 enchantment on the battlefield.
fn fading_enchantment_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Fading Enchantment")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("test-fading-enchant".into()))
        .with_keyword(KeywordAbility::Fading(2))
        .with_types(vec![CardType::Enchantment])
}

// ── Test 1: ETB places fade counters ─────────────────────────────────────────

#[test]
/// CR 702.32a — Permanent with Fading 3 enters the battlefield with 3 fade counters.
fn test_fading_etb_places_fade_counters() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![fading_3_creature_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(fading_3_on_battlefield(p1))
        .build()
        .unwrap();

    let obj_id = find_in_zone(&state, "Test Fading 3 Creature", ZoneId::Battlefield)
        .expect("creature should be on battlefield");

    // When built with ObjectSpec, counters are not automatically set.
    // The ETB placement in resolution.rs fires when permanents resolve from the stack.
    // Verify the keyword is present correctly for now.
    assert!(
        state.objects[&obj_id]
            .characteristics
            .keywords
            .contains(&KeywordAbility::Fading(3)),
        "CR 702.32a: permanent should have Fading(3) keyword"
    );
}

#[test]
/// CR 702.32a — When a Fading 3 spell resolves, it enters with exactly 3 fade counters.
/// Uses CastSpell flow to exercise the ETB counter-placement code in resolution.rs.
fn test_fading_etb_counters_on_cast() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![fading_3_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Test Fading 3 Creature")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(CardId("test-fading-3".into()))
                .with_keyword(KeywordAbility::Fading(3))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    green: 1,
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
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Test Fading 3 Creature");

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
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .expect("CastSpell should succeed");

    // Resolve the spell: both players pass priority.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After resolution: permanent on battlefield with 3 fade counters.
    assert!(
        on_battlefield(&state, "Test Fading 3 Creature"),
        "CR 702.32a: creature should be on battlefield after resolving"
    );
    assert_eq!(
        fade_counters(&state, "Test Fading 3 Creature"),
        3,
        "CR 702.32a: Fading 3 creature should enter with exactly 3 fade counters"
    );
    // Must use fade counters, not time counters.
    assert_eq!(
        time_counters(&state, "Test Fading 3 Creature"),
        0,
        "CR 702.32a: Fading uses fade counters (CounterType::Fade), not time counters"
    );
}

// ── Test 2: Upkeep counter-removal trigger ────────────────────────────────────

#[test]
/// CR 702.32a — At beginning of controller's upkeep, one fade counter is removed.
/// After resolving: 3 counters -> 2.
///
/// Test setup: Start at Step::Untap with priority manually set. Passing priority for all
/// players at Untap advances to Upkeep via enter_step, which calls upkeep_actions and
/// queues the FadingTrigger.
fn test_fading_upkeep_removes_counter() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![fading_3_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(fading_3_on_battlefield(p1))
        .build()
        .unwrap();

    // Manually place 3 fade counters.
    let obj_id = find_object(&state, "Test Fading 3 Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.counters = obj.counters.update(CounterType::Fade, 3);
    }
    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep (enter_step queues FadingTrigger).
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::Upkeep,
        "should be at Upkeep after advancing from Untap"
    );

    // FadingTrigger should be on the stack.
    assert!(
        state.stack_objects.iter().any(|so| matches!(
            &so.kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Fading(_),
                data: mtg_engine::TriggerData::CounterRemoval { .. },
                ..
            }
        )),
        "CR 702.32a: FadingTrigger should be on the stack at upkeep"
    );

    // Both players pass -> trigger resolves -> 1 fade counter removed.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert_eq!(
        fade_counters(&state, "Test Fading 3 Creature"),
        2,
        "CR 702.32a: one fade counter should be removed, leaving 2"
    );
    assert!(
        on_battlefield(&state, "Test Fading 3 Creature"),
        "creature should still be on battlefield with 2 fade counters"
    );

    // CounterRemoved event should have been emitted.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CounterRemoved {
                counter: CounterType::Fade,
                count: 1,
                ..
            }
        )),
        "CR 702.32a: CounterRemoved (Fade) event should be emitted"
    );
}

// ── Test 3: Sacrifice when no counters ────────────────────────────────────────

#[test]
/// CR 702.32a — When upkeep trigger fires with 0 fade counters, permanent is sacrificed.
/// With 1 counter: first upkeep removes it (1->0, no sacrifice yet). Second upkeep:
/// trigger fires, can't remove (0 counters), sacrifices.
fn test_fading_sacrifice_when_no_counters() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![fading_3_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(fading_3_on_battlefield(p1))
        .build()
        .unwrap();

    // Place exactly 1 fade counter (will be removed on first upkeep, then sacrifice on second).
    let obj_id = find_object(&state, "Test Fading 3 Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.counters = obj.counters.update(CounterType::Fade, 1);
    }
    state.turn.priority_holder = Some(p1);

    // === First upkeep: trigger fires, removes the 1 fade counter ===
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Resolve FadingTrigger -> removes the last counter (1->0). No sacrifice yet.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        fade_counters(&state, "Test Fading 3 Creature"),
        0,
        "CR 702.32a: last counter removed, should have 0 fade counters"
    );
    // Still on battlefield -- sacrifice happens on NEXT upkeep.
    assert!(
        on_battlefield(&state, "Test Fading 3 Creature"),
        "CR 702.32a: creature should still be on battlefield after first counter removal (0 counters)"
    );

    // Advance to next upkeep manually.
    let mut state = state;
    state.turn.step = Step::Untap;
    state.turn.priority_holder = Some(p1);

    // === Second upkeep: trigger fires, can't remove (0 counters) -> sacrifice ===
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // FadingTrigger on stack.
    assert!(
        state.stack_objects.iter().any(|so| matches!(
            &so.kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Fading(_),
                data: mtg_engine::TriggerData::CounterRemoval { .. },
                ..
            }
        )),
        "CR 702.32a: FadingTrigger should be on the stack at second upkeep"
    );

    // Resolve -> sacrifice because 0 fade counters.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Test Fading 3 Creature"),
        "CR 702.32a: creature should be sacrificed when trigger can't remove a fade counter"
    );
    assert!(
        in_graveyard(&state, "Test Fading 3 Creature", p1),
        "CR 702.32a: sacrificed creature should be in owner's graveyard"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.32a: CreatureDied event should be emitted on Fading sacrifice"
    );
}

// ── Test 4: Full lifecycle ────────────────────────────────────────────────────

#[test]
/// CR 702.32a — Fading 2 permanent enters with 2 counters. Full lifecycle:
/// Upkeep 1: 2->1 (stays on battlefield). Upkeep 2: 1->0 (stays). Upkeep 3: 0->sacrifice.
/// Fading N gives N upkeeps of counter removal + 1 sacrifice upkeep = N+1 total upkeeps.
fn test_fading_full_lifecycle() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![fading_2_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(fading_2_on_battlefield(p1))
        .build()
        .unwrap();

    // Place 2 fade counters (simulating ETB placement).
    let obj_id = find_object(&state, "Test Fading 2 Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.counters = obj.counters.update(CounterType::Fade, 2);
    }
    state.turn.priority_holder = Some(p1);

    // === Upkeep 1: advance to Upkeep, trigger fires, 2->1 counter ===
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        fade_counters(&state, "Test Fading 2 Creature"),
        1,
        "CR 702.32a: should have 1 fade counter after first upkeep"
    );
    assert!(
        on_battlefield(&state, "Test Fading 2 Creature"),
        "CR 702.32a: should still be on battlefield after first upkeep"
    );

    // Reset to Untap for upkeep 2.
    let mut state = state;
    state.turn.step = Step::Untap;
    state.turn.priority_holder = Some(p1);

    // === Upkeep 2: 1->0 counter (no sacrifice yet) ===
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        fade_counters(&state, "Test Fading 2 Creature"),
        0,
        "CR 702.32a: should have 0 fade counters after second upkeep"
    );
    assert!(
        on_battlefield(&state, "Test Fading 2 Creature"),
        "CR 702.32a: should still be on battlefield after second upkeep (0 counters, sacrifice on next)"
    );

    // Reset to Untap for upkeep 3.
    let mut state = state;
    state.turn.step = Step::Untap;
    state.turn.priority_holder = Some(p1);

    // === Upkeep 3: can't remove (0 counters) -> sacrifice ===
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Test Fading 2 Creature"),
        "CR 702.32a: Fading 2 creature sacrificed on 3rd upkeep (N+1)"
    );
    assert!(
        in_graveyard(&state, "Test Fading 2 Creature", p1),
        "CR 702.32a: sacrificed Fading creature should be in owner's graveyard"
    );
}

// ── Test 5: Multiplayer — only active player's permanents trigger ──────────────

#[test]
/// CR 702.32a "your upkeep" — In multiplayer, Fading only ticks down on the
/// controlling player's upkeep, not on other players' upkeeps.
fn test_fading_multiplayer_only_active_player() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![fading_3_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p2) // p2 is active, p1 controls the creature
        .at_step(Step::Untap)
        .object(fading_3_on_battlefield(p1)) // p1's creature
        .build()
        .unwrap();

    // Place 3 fade counters on p1's creature.
    let obj_id = find_object(&state, "Test Fading 3 Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.counters = obj.counters.update(CounterType::Fade, 3);
    }
    state.turn.priority_holder = Some(p2);

    // Advance p2's Untap -> p2's Upkeep.
    let (state, _) = pass_all(state, &[p2, p1]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // No FadingTrigger for p1's permanent during p2's upkeep.
    assert!(
        !state.stack_objects.iter().any(|so| matches!(
            &so.kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Fading(_),
                data: mtg_engine::TriggerData::CounterRemoval { .. },
                ..
            }
        )),
        "CR 702.32a: FadingTrigger must not fire on a non-controller's upkeep"
    );
    assert_eq!(
        fade_counters(&state, "Test Fading 3 Creature"),
        3,
        "CR 702.32a: fade counters must not be removed on non-controller's upkeep"
    );
}

// ── Test 6: Non-creature sacrifice ────────────────────────────────────────────

#[test]
/// CR 702.32a — Fading on an enchantment (like Parallax Wave) sacrifices correctly.
/// The sacrifice path in FadingTrigger must handle non-creature permanents.
fn test_fading_non_creature_sacrifice() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![fading_2_enchantment_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(fading_enchantment_on_battlefield(p1))
        .build()
        .unwrap();

    // Place 0 fade counters (simulates a state where all were removed externally).
    // The trigger fires unconditionally and will sacrifice.
    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep -> FadingTrigger fires.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Trigger on stack.
    assert!(
        state.stack_objects.iter().any(|so| matches!(
            &so.kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Fading(_),
                data: mtg_engine::TriggerData::CounterRemoval { .. },
                ..
            }
        )),
        "CR 702.32a: FadingTrigger should be on stack for enchantment"
    );

    // Resolve -> sacrifice the enchantment (0 counters).
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Test Fading Enchantment"),
        "CR 702.32a: Fading enchantment should be sacrificed when trigger can't remove counter"
    );
    // Enchantment goes to graveyard (via CreatureDied path which handles any permanent).
    assert!(
        find_in_zone(&state, "Test Fading Enchantment", ZoneId::Graveyard(p1)).is_some(),
        "CR 702.32a: sacrificed Fading enchantment should be in owner's graveyard"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.32a: CreatureDied event should be emitted for non-creature Fading sacrifice"
    );
}

// ── Test 7: Fade counters are distinct from time counters ─────────────────────

#[test]
/// CR 702.32a — Fading uses CounterType::Fade, not CounterType::Time.
/// A permanent with both Fading and time counters should only decrement fade counters.
fn test_fading_uses_fade_counters_not_time() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![fading_3_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(fading_3_on_battlefield(p1))
        .build()
        .unwrap();

    // Give the permanent both fade counters (for Fading) and time counters (for something else).
    let obj_id = find_object(&state, "Test Fading 3 Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.counters = obj.counters.update(CounterType::Fade, 3);
        obj.counters = obj.counters.update(CounterType::Time, 5);
    }
    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep -> FadingTrigger queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Resolve FadingTrigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Fade counter decremented, time counter untouched.
    assert_eq!(
        fade_counters(&state, "Test Fading 3 Creature"),
        2,
        "CR 702.32a: Fading trigger should remove only a fade counter (3->2)"
    );
    assert_eq!(
        time_counters(&state, "Test Fading 3 Creature"),
        5,
        "CR 702.32a: time counters must not be affected by Fading trigger"
    );
    assert!(
        on_battlefield(&state, "Test Fading 3 Creature"),
        "creature should still be on battlefield (2 fade counters remaining)"
    );
}
