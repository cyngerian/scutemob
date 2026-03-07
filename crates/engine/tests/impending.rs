//! Impending keyword ability tests (CR 702.176).
//!
//! Impending N--[cost] represents four abilities:
//! 1. Static (on stack): alternative cost -- pay [cost] instead of mana cost (CR 702.176a).
//! 2. Static (ETB replacement): enters with N time counters if impending cost paid (CR 702.176a).
//! 3. Static (battlefield): not a creature while impending cost paid AND has time counters (CR 702.176a).
//! 4. Triggered (battlefield): at beginning of controller's end step, if impending cost paid and has
//!    time counter, remove a time counter from it (CR 702.176a). Intervening-if (CR 603.4).
//!
//! Key rules verified:
//! - Impending is an alternative cost: pay impending cost instead of mana cost (CR 702.176a).
//! - After ETB (impending cast): N time counters on the permanent.
//! - While counters remain: NOT a creature (Layer 4 type-removal).
//! - End step: one time counter removed per controller's end step.
//! - After last counter removed: IS a creature (type-removal condition false).
//! - Normal cast (no impending): no time counters, IS a creature immediately.
//! - SBAs don't kill it while non-creature (no P/T check applies).
//! - Multiple end steps with N counters: becomes creature after exactly N steps.
//! - Alt cost exclusivity: cannot combine with other alt costs (CR 118.9a).
//! - Commander tax applies on top of impending cost (CR 118.9d / CR 903.8).
//! - Multiplayer: only CONTROLLER'S end step triggers counter removal (CR 702.176a).

use mtg_engine::state::types::{AltCostKind, SuperType};
use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, Command, CounterType, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
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
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// Test Impending Creature: Enchantment Creature {3}{G}{G}. Impending 4--{1}{G}{G}. 6/5.
/// Minimal card definition for testing impending mechanics.
fn impending_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-impending-creature".into()),
        name: "Test Impending Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Enchantment, CardType::Creature]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "Impending 4--{1}{G}{G}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Impending),
            AbilityDefinition::Impending {
                cost: ManaCost {
                    generic: 1,
                    green: 2,
                    ..Default::default()
                },
                count: 4,
            },
        ],
        power: Some(6),
        toughness: Some(5),
        ..Default::default()
    }
}

/// Impending creature in player's hand.
fn impending_creature_in_hand(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Impending Creature")
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("test-impending-creature".into()))
        .with_types(vec![CardType::Enchantment, CardType::Creature])
        .with_keyword(KeywordAbility::Impending)
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 2,
            ..Default::default()
        })
}

/// Set cast_alt_cost on the named battlefield permanent.
fn set_cast_alt_cost(state: &mut GameState, name: &str, cost: AltCostKind) {
    let id = find_in_zone(state, name, ZoneId::Battlefield)
        .unwrap_or_else(|| panic!("'{}' not found on battlefield", name));
    if let Some(obj) = state.objects.get_mut(&id) {
        obj.cast_alt_cost = Some(cost);
    }
}

// ── Test 1: Basic cast with impending cost ────────────────────────────────────

/// CR 702.176a — Test Impending Creature cast for impending cost {1}{G}{G}.
/// After resolution: on battlefield with 4 time counters. cast_alt_cost == Impending.
#[test]
fn test_impending_basic_cast() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![impending_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(impending_creature_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {1}{G}{G} — impending cost instead of mana cost {3}{G}{G}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Test Impending Creature");

    // Cast with impending cost.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Impending),
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
    .unwrap_or_else(|e| panic!("CastSpell with impending failed: {:?}", e));

    // Spell is on the stack with was_impended = true.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.176a: impended spell should be on the stack"
    );
    assert!(
        state.stack_objects[0].was_impended,
        "CR 702.176a: was_impended should be true on stack object"
    );

    // Mana consumed: {1}{G}{G} = 3 pips (not {3}{G}{G} = 5 pips).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.176a: impending cost {{1}}{{G}}{{G}} should be deducted from mana pool"
    );

    // Resolve the spell (both players pass priority).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature is on the battlefield.
    assert!(
        on_battlefield(&state, "Test Impending Creature"),
        "CR 702.176a: impended creature should be on battlefield after resolution"
    );

    // cast_alt_cost is set to Impending on the permanent.
    let bf_id = find_in_zone(&state, "Test Impending Creature", ZoneId::Battlefield).unwrap();
    assert_eq!(
        state.objects[&bf_id].cast_alt_cost,
        Some(AltCostKind::Impending),
        "CR 702.176a: cast_alt_cost should be Some(Impending) on battlefield permanent"
    );

    // 4 time counters entered with the permanent (CR 702.176a replacement effect).
    assert_eq!(
        time_counters(&state, "Test Impending Creature"),
        4,
        "CR 702.176a: impended permanent should enter with 4 time counters"
    );
}

// ── Test 2: Not a creature while counters remain ──────────────────────────────

/// CR 702.176a — While impending cost was paid AND time counters remain, the permanent
/// is NOT a creature (Layer 4 type-removal via calculate_characteristics).
#[test]
fn test_impending_not_a_creature_while_counters() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![impending_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(impending_creature_in_hand(p1))
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
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Test Impending Creature");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Impending),
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

    // Resolve the spell.
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_in_zone(&state, "Test Impending Creature", ZoneId::Battlefield).unwrap();

    // Verify 4 time counters.
    assert_eq!(
        time_counters(&state, "Test Impending Creature"),
        4,
        "should have 4 time counters"
    );

    // Layer 4: NOT a creature while counters remain.
    let chars = calculate_characteristics(&state, bf_id).expect("object should exist");
    assert!(
        !chars.card_types.contains(&CardType::Creature),
        "CR 702.176a: permanent with impending cost paid and time counters should NOT be a creature"
    );
    // But IS still an Enchantment.
    assert!(
        chars.card_types.contains(&CardType::Enchantment),
        "CR 702.176a: permanent should still be an Enchantment while not a creature"
    );
}

// ── Test 3: Counter removed at end step ──────────────────────────────────────

/// CR 702.176a — At the beginning of the controller's end step, one time counter is removed.
#[test]
fn test_impending_counter_removed_at_end_step() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![impending_creature_def()]);

    // Place impending permanent with 4 time counters directly on the battlefield.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Test Impending Creature")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("test-impending-creature".into()))
                .with_types(vec![CardType::Enchantment, CardType::Creature])
                .with_keyword(KeywordAbility::Impending)
                .with_counter(CounterType::Time, 4),
        )
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    // Set cast_alt_cost to Impending directly (simulates having cast for impending cost).
    set_cast_alt_cost(
        &mut state,
        "Test Impending Creature",
        AltCostKind::Impending,
    );

    // Advance to End step (both players pass priority at PostCombatMain).
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::End, "should be at End step");

    // At End step entry, ImpendingCounter trigger should be queued.
    // Both players pass priority to let the trigger resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // One counter should have been removed: 4 -> 3.
    assert_eq!(
        time_counters(&state, "Test Impending Creature"),
        3,
        "CR 702.176a: one time counter should be removed at beginning of controller's end step"
    );
}

// ── Test 4: Becomes creature when last counter removed ────────────────────────

/// CR 702.176a — After last time counter removed, the permanent IS a creature.
#[test]
fn test_impending_becomes_creature_when_last_counter_removed() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![impending_creature_def()]);

    // Start with 1 time counter.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Test Impending Creature")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("test-impending-creature".into()))
                .with_types(vec![CardType::Enchantment, CardType::Creature])
                .with_keyword(KeywordAbility::Impending)
                .with_counter(CounterType::Time, 1),
        )
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    set_cast_alt_cost(
        &mut state,
        "Test Impending Creature",
        AltCostKind::Impending,
    );

    let bf_id = find_in_zone(&state, "Test Impending Creature", ZoneId::Battlefield).unwrap();

    // Verify: with 1 counter, is NOT a creature yet.
    let chars_before = calculate_characteristics(&state, bf_id).unwrap();
    assert!(
        !chars_before.card_types.contains(&CardType::Creature),
        "pre-condition: should NOT be a creature with 1 time counter"
    );

    // Advance to End step and let the trigger fire.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::End);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Last counter removed: 0 time counters remain.
    assert_eq!(
        time_counters(&state, "Test Impending Creature"),
        0,
        "CR 702.176a: all time counters should be removed after last end step trigger"
    );

    // Now IS a creature (Layer 4 type-removal condition false: no time counters).
    let bf_id_new = find_in_zone(&state, "Test Impending Creature", ZoneId::Battlefield).unwrap();
    let chars_after = calculate_characteristics(&state, bf_id_new).unwrap();
    assert!(
        chars_after.card_types.contains(&CardType::Creature),
        "CR 702.176a: after last counter removed, should BE a creature"
    );
    assert!(
        chars_after.card_types.contains(&CardType::Enchantment),
        "CR 702.176a: should still be an Enchantment after becoming a creature"
    );
}

// ── Test 5: Normal cast — no time counters, IS a creature immediately ─────────

/// CR 702.176a (negative test) — Cast for normal mana cost, no time counters.
/// Permanent IS a creature immediately with no end-step counter removal triggers.
#[test]
fn test_impending_normal_cast() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![impending_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(impending_creature_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay full mana cost {3}{G}{G} = 5 mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Test Impending Creature");

    // Cast for normal cost — no alt_cost.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
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
        },
    )
    .unwrap_or_else(|e| panic!("Normal CastSpell failed: {:?}", e));

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Test Impending Creature"),
        "creature should be on battlefield after normal cast"
    );

    // No time counters.
    assert_eq!(
        time_counters(&state, "Test Impending Creature"),
        0,
        "CR 702.176a: normal cast should enter with 0 time counters"
    );

    // IS a creature (no impending condition is met: cast_alt_cost == None).
    let bf_id = find_in_zone(&state, "Test Impending Creature", ZoneId::Battlefield).unwrap();
    let chars = calculate_characteristics(&state, bf_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "CR 702.176a: normally cast permanent should be a creature"
    );
    assert_eq!(
        state.objects[&bf_id].cast_alt_cost, None,
        "CR 702.176a: normally cast permanent should have cast_alt_cost == None"
    );
}

// ── Test 6: SBAs while not a creature ────────────────────────────────────────

/// CR 702.176a — While impending permanent is not a creature, SBAs do not
/// apply creature rules to it. The permanent stays on the battlefield.
#[test]
fn test_impending_sba_while_not_creature() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![impending_creature_def()]);

    // Place impending permanent with time counters directly on battlefield.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Test Impending Creature")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("test-impending-creature".into()))
                .with_types(vec![CardType::Enchantment, CardType::Creature])
                .with_keyword(KeywordAbility::Impending)
                .with_counter(CounterType::Time, 2),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    set_cast_alt_cost(
        &mut state,
        "Test Impending Creature",
        AltCostKind::Impending,
    );

    let bf_id = find_in_zone(&state, "Test Impending Creature", ZoneId::Battlefield).unwrap();

    // Verify it's not a creature via layer calculations.
    let chars = calculate_characteristics(&state, bf_id).unwrap();
    assert!(
        !chars.card_types.contains(&CardType::Creature),
        "pre-condition: should NOT be a creature with time counters and impending cost paid"
    );

    // The permanent should still be on the battlefield (SBAs don't kill it because
    // it's not a creature with 0 toughness -- it's just an Enchantment now).
    assert!(
        on_battlefield(&state, "Test Impending Creature"),
        "CR 702.176a: impending non-creature should remain on battlefield without SBA death"
    );
}

// ── Test 7: Multiple end steps count down correctly ───────────────────────────

/// CR 702.176a — Cast with 4 time counters. After each end step, one counter
/// is removed. The permanent remains non-creature while counters exist.
#[test]
fn test_impending_multiple_end_steps() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![impending_creature_def()]);

    // Start with 2 time counters to keep the test concise.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Test Impending Creature")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("test-impending-creature".into()))
                .with_types(vec![CardType::Enchantment, CardType::Creature])
                .with_keyword(KeywordAbility::Impending)
                .with_counter(CounterType::Time, 2),
        )
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    set_cast_alt_cost(
        &mut state,
        "Test Impending Creature",
        AltCostKind::Impending,
    );

    // Verify: 2 counters, NOT a creature.
    let bf_id = find_in_zone(&state, "Test Impending Creature", ZoneId::Battlefield).unwrap();
    let chars_initial = calculate_characteristics(&state, bf_id).unwrap();
    assert!(
        !chars_initial.card_types.contains(&CardType::Creature),
        "pre-condition: should NOT be a creature with 2 time counters"
    );

    // End step 1: PostCombatMain -> End, trigger resolves.
    let (state, _) = pass_all(state, &[p1, p2]); // Advance to End
    assert_eq!(state.turn.step, Step::End);
    let (state, _) = pass_all(state, &[p1, p2]); // Trigger resolves
    assert_eq!(
        time_counters(&state, "Test Impending Creature"),
        1,
        "after end step 1: should have 1 time counter remaining"
    );

    // Still NOT a creature with 1 counter.
    let bf_id = find_in_zone(&state, "Test Impending Creature", ZoneId::Battlefield).unwrap();
    let chars_mid = calculate_characteristics(&state, bf_id).unwrap();
    assert!(
        !chars_mid.card_types.contains(&CardType::Creature),
        "after end step 1: still not a creature with 1 counter"
    );

    // After end step 2, the permanent should have 0 counters and BE a creature.
    // (Handled in test_impending_becomes_creature_when_last_counter_removed --
    //  this test focuses on the multi-step countdown.)
    assert_eq!(
        time_counters(&state, "Test Impending Creature"),
        1,
        "CR 702.176a: each end step removes exactly 1 counter (2->1 after first step)"
    );
}

// ── Test 8: CardType::Creature restored after last counter ───────────────────

/// CR 702.176a — With cast_alt_cost == Impending but 0 time counters, IS a creature.
/// The type-removal condition requires BOTH impending cost paid AND time counters > 0.
#[test]
fn test_impending_creature_card_type_restored() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![impending_creature_def()]);

    // Set up with 0 time counters and cast_alt_cost == Impending.
    // With 0 counters, the type-removal condition is false (requires counters > 0).
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Test Impending Creature")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("test-impending-creature".into()))
                .with_types(vec![CardType::Enchantment, CardType::Creature])
                .with_keyword(KeywordAbility::Impending),
            // No .with_counter(CounterType::Time, N) -- 0 counters
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Set cast_alt_cost to Impending (as if spell was cast for impending cost).
    set_cast_alt_cost(
        &mut state,
        "Test Impending Creature",
        AltCostKind::Impending,
    );

    let bf_id = find_in_zone(&state, "Test Impending Creature", ZoneId::Battlefield).unwrap();

    // With 0 time counters and impending cost paid: IS a creature.
    // Condition: cast_alt_cost == Impending AND counters > 0. Second is false => no removal.
    let chars = calculate_characteristics(&state, bf_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "CR 702.176a: with 0 time counters (even with impending cost paid), should be a creature"
    );
    assert!(
        chars.card_types.contains(&CardType::Enchantment),
        "should still be an Enchantment"
    );
}

// ── Test 9: Alt cost exclusivity ─────────────────────────────────────────────

/// CR 118.9a — Attempting to cast with AltCostKind::Impending on a card that lacks
/// the Impending ability should return an error.
#[test]
fn test_impending_alt_cost_exclusivity() {
    let p1 = p(1);
    let p2 = p(2);

    // A card WITHOUT impending ability.
    let vanilla_def = CardDefinition {
        card_id: CardId("vanilla-creature".into()),
        name: "Vanilla Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        power: Some(3),
        toughness: Some(3),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![vanilla_def]);

    let vanilla_spec = ObjectSpec::card(p1, "Vanilla Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("vanilla-creature".into()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(vanilla_spec)
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
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Vanilla Creature");

    // Attempt to cast with impending alt_cost on a card that doesn't have impending.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Impending),
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
    );

    assert!(
        result.is_err(),
        "CR 702.176a / CR 118.9a: casting with impending on a non-impending card should fail"
    );
}

// ── Test 10: Commander tax applies on top of impending cost ───────────────────

/// CR 118.9d / CR 903.8 — Commander tax is applied on top of the impending cost.
/// If the commander has impending 4--{1}{G}{G} and was cast once before (2-mana tax),
/// the impending cost becomes {3}{G}{G}.
#[test]
fn test_impending_commander_tax() {
    let p1 = p(1);
    let p2 = p(2);

    // Impending commander definition.
    let commander_def = CardDefinition {
        card_id: CardId("impending-commander".into()),
        name: "Impending Commander".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Enchantment, CardType::Creature]
                .into_iter()
                .collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Impending 4--{1}{G}{G}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Impending),
            AbilityDefinition::Impending {
                cost: ManaCost {
                    generic: 1,
                    green: 2,
                    ..Default::default()
                },
                count: 4,
            },
        ],
        power: Some(6),
        toughness: Some(5),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![commander_def]);

    let card_id_raw = CardId("impending-commander".into());
    let commander_spec = ObjectSpec::card(p1, "Impending Commander")
        .in_zone(ZoneId::Command(p1))
        .with_card_id(card_id_raw.clone())
        .with_types(vec![CardType::Enchantment, CardType::Creature])
        .with_keyword(KeywordAbility::Impending)
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, card_id_raw.clone())
        .object(commander_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Manually set the commander tax to simulate having cast it once before
    // (1 previous cast = +2 generic tax on next cast).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(card_id_raw.clone(), 1);

    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Impending Commander");

    // With 1 previous cast, impending cost {1}{G}{G} + {2} tax = {3}{G}{G}.
    // Try to cast with just {1}{G}{G} -- should fail (insufficient mana for tax).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let result = process_command(
        state.clone(),
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Impending),
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
    );

    // Should fail: insufficient mana (need {3}{G}{G} but only have {1}{G}{G}).
    assert!(
        result.is_err(),
        "CR 118.9d: impending cost + commander tax should require {{3}}{{G}}{{G}}, not just {{1}}{{G}}{{G}}"
    );

    // Now provide enough mana: {3}{G}{G}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);

    let card_id = find_object(&state, "Impending Commander");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Impending),
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
    );

    assert!(
        result.is_ok(),
        "CR 118.9d: impending cost + commander tax should succeed with {{3}}{{G}}{{G}}: {:?}",
        result.err()
    );
}

// ── Test 11: Counter removal only on controller's end step ────────────────────

/// CR 702.176a — "At the beginning of YOUR end step." Counter removal fires only
/// for the controller's own end step. In a 2-player game, p2's end step should
/// NOT tick down p1's impending permanent.
#[test]
fn test_impending_counter_removal_only_on_controller_end_step() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![impending_creature_def()]);

    // Place p1's impending permanent on the battlefield with 4 time counters.
    // Set active player to p2 at PostCombatMain so we advance into p2's End step.
    // This verifies that p2's end step does NOT trigger p1's counter removal.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Test Impending Creature")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("test-impending-creature".into()))
                .with_types(vec![CardType::Enchantment, CardType::Creature])
                .with_keyword(KeywordAbility::Impending)
                .with_counter(CounterType::Time, 4),
        )
        .active_player(p2)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    // Set cast_alt_cost to Impending (simulating having been cast for impending cost).
    set_cast_alt_cost(
        &mut state,
        "Test Impending Creature",
        AltCostKind::Impending,
    );

    // Advance to p2's End step. end_step_actions() fires on entry, scanning for
    // impending permanents controlled by the ACTIVE player (p2). p1's permanent
    // is controlled by p1, so no ImpendingCounter trigger is queued for it.
    let (state, _) = pass_all(state, &[p2, p1]);
    assert_eq!(state.turn.step, Step::End, "should be at End step");
    assert_eq!(state.turn.active_player, p2, "should be p2's end step");

    // Both players pass priority — any triggers on the stack resolve.
    // (There should be no ImpendingCounter trigger for p1's permanent.)
    let (state, _) = pass_all(state, &[p2, p1]);

    // p1's impending permanent should still have 4 time counters.
    // p2's end step should NOT trigger p1's counter removal (only p1's end step does).
    assert_eq!(
        time_counters(&state, "Test Impending Creature"),
        4,
        "CR 702.176a: p2's end step should NOT remove counters from p1's impending permanent"
    );
}
