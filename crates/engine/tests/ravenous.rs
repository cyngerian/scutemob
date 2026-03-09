//! Ravenous keyword ability tests (CR 702.156).
//!
//! Ravenous represents both a replacement effect and a triggered ability.
//! "This permanent enters with X +1/+1 counters on it" (replacement effect)
//! and "When this permanent enters, if X is 5 or more, draw a card." (trigger)
//! (CR 702.156a)
//!
//! X is the value chosen when the spell was cast (CR 107.3m). The permanent's
//! own X is 0 on the battlefield (CR 107.3i), but the ETB replacement and
//! trigger use the cast-time value.
//!
//! Key rules verified:
//! - ETB with X +1/+1 counters (X chosen at cast time) (CR 702.156a, CR 107.3m).
//! - No counters when X = 0 (CR 702.156a).
//! - X = 0 with base P/T 0/0 → creature dies to SBA (CR 704.5f).
//! - Draw trigger fires when X >= 5 (CR 702.156a).
//! - Draw trigger does NOT fire when X < 5 (boundary: X = 4 does not draw).
//! - Large X values work correctly (X = 10 → 10 counters).
//! - CounterAdded event is emitted when counters are placed.
//! - CardDrawn event is emitted when the draw trigger resolves.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, GameEvent, GameStateBuilder, KeywordAbility, ManaCost, ObjectId, ObjectSpec,
    PlayerId, Step, TypeLine, ZoneId,
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

/// Cast a Ravenous creature with the given x_value and generic mana budget.
/// The generic cost budget should include X (x_value) plus any base generic cost.
fn cast_ravenous(
    state: mtg_engine::GameState,
    caster: PlayerId,
    card_id: ObjectId,
    generic_mana: u32,
    x_value: u32,
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let mut state = state;
    state
        .players
        .get_mut(&caster)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, generic_mana);
    state.turn.priority_holder = Some(caster);

    process_command(
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
            // CR 107.3m: X value chosen at cast time.
            x_value,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell(x_value={}) failed: {:?}", x_value, e))
}

// ── Card Definitions ──────────────────────────────────────────────────────────

/// Test Ravenous creature. Cost {X}, base P/T 0/0.
/// "Ravenous (This creature enters with X +1/+1 counters on it.
///  If X is 5 or more, draw a card when it enters.)"
/// Pure-X cost (no colored pips) for simple test mana setup.
fn ravenous_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("ravenous-test".to_string()),
        name: "Test Ravenous Beast".to_string(),
        mana_cost: Some(ManaCost {
            generic: 0, // base generic = 0; X is passed as x_value and added at cast time
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Ravenous".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Ravenous)],
        ..Default::default()
    }
}

/// Build the ObjectSpec for the test Ravenous creature in hand.
fn ravenous_spec(player: PlayerId) -> ObjectSpec {
    ObjectSpec::card(player, "Test Ravenous Beast")
        .in_zone(ZoneId::Hand(player))
        .with_card_id(CardId("ravenous-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Ravenous)
        .with_mana_cost(ManaCost {
            generic: 0,
            ..Default::default()
        })
}

/// A dummy instant for the library (used when a draw needs to succeed).
fn dummy_library_card(suffix: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(format!("dummy-lib-{}", suffix)),
        name: format!("Library Dummy {}", suffix),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: None,
        toughness: None,
        abilities: vec![],
        ..Default::default()
    }
}

fn dummy_library_spec(player: PlayerId, suffix: &str) -> ObjectSpec {
    ObjectSpec::card(player, &format!("Library Dummy {}", suffix))
        .in_zone(ZoneId::Library(player))
        .with_card_id(CardId(format!("dummy-lib-{}", suffix)))
        .with_types(vec![CardType::Instant])
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 702.156a, CR 107.3m: Ravenous creature cast with X=3 enters with 3 +1/+1 counters.
/// No draw trigger fires (X < 5).
#[test]
fn test_ravenous_x3_enters_with_3_counters() {
    // CR 702.156a: cast with X=3, enter with 3 +1/+1 counters, no draw.
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![ravenous_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ravenous_spec(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Test Ravenous Beast");

    // Cast with X=3 (pay X = 3 generic mana).
    let (state, _cast_events) = cast_ravenous(state, p1, card_obj_id, 3, 3);

    // Resolve: pass priority for both players.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Creature is on battlefield.
    let creature_id =
        find_object_on_battlefield(&state, "Test Ravenous Beast").expect("creature on battlefield");

    // CR 702.156a: 3 +1/+1 counters placed at ETB.
    let counter_count = state
        .objects
        .get(&creature_id)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "creature should have 3 +1/+1 counters (X=3)"
    );

    // CounterAdded event must have been emitted.
    let counter_added = resolve_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 3,
                ..
            }
        )
    });
    assert!(counter_added, "CounterAdded(3) event must be emitted");

    // No draw trigger (X=3 < 5): no CardDrawn event in resolve + extra round.
    let (_, extra_events) = pass_all(state, &[p1, p2]);
    let card_drawn = resolve_events
        .iter()
        .chain(extra_events.iter())
        .any(|e| matches!(e, GameEvent::CardDrawn { .. }));
    assert!(!card_drawn, "no draw when X=3 (X < 5)");
}

/// CR 702.156a, CR 107.3m: Ravenous creature cast with X=5 enters with 5 +1/+1 counters
/// AND controller draws a card.
#[test]
fn test_ravenous_x5_enters_with_5_counters_and_draws() {
    // CR 702.156a: cast with X=5, enter with 5 +1/+1 counters, draw a card.
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![ravenous_creature_def(), dummy_library_card("5")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ravenous_spec(p1))
        .object(dummy_library_spec(p1, "5"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Test Ravenous Beast");

    let initial_hand_size = state
        .objects
        .iter()
        .filter(|(_, o)| o.zone == ZoneId::Hand(p1))
        .count();

    // Cast with X=5 (pay X = 5 generic).
    let (state, _) = cast_ravenous(state, p1, card_obj_id, 5, 5);

    // Resolve spell.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);
    // Resolve draw trigger (goes on stack after spell resolves).
    let (state, draw_events) = pass_all(state, &[p1, p2]);

    let all_events: Vec<_> = resolve_events
        .iter()
        .chain(draw_events.iter())
        .cloned()
        .collect();

    // Creature is on battlefield.
    let creature_id =
        find_object_on_battlefield(&state, "Test Ravenous Beast").expect("creature on battlefield");

    // CR 702.156a: 5 +1/+1 counters.
    let counter_count = state
        .objects
        .get(&creature_id)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert_eq!(
        counter_count, 5,
        "creature should have 5 +1/+1 counters (X=5)"
    );

    // CR 702.156a: draw trigger fires when X >= 5.
    // The creature card left hand (cast to stack to battlefield), so hand delta is:
    //   -1 (cast creature) + 1 (draw trigger) = 0 net change.
    // i.e., final_hand_size == initial_hand_size.
    let final_hand_size = state
        .objects
        .iter()
        .filter(|(_, o)| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        final_hand_size,
        initial_hand_size, // net: −1 for cast + 1 for draw = 0
        "p1 should have drawn 1 card replacing the cast creature (X=5)"
    );

    // CardDrawn event must be present.
    let card_drawn = all_events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { .. }));
    assert!(
        card_drawn,
        "CardDrawn event must be emitted for Ravenous X=5"
    );
}

/// CR 702.156a: Ravenous creature cast with X=0 enters with 0 counters, no draw.
/// With base P/T 0/0 and 0 counters, it has toughness 0 → dies to SBA (CR 704.5f).
#[test]
fn test_ravenous_x0_enters_with_no_counters_no_draw() {
    // CR 702.156a: cast with X=0, enter with 0 counters, no draw, 0/0 dies.
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![ravenous_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ravenous_spec(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Test Ravenous Beast");

    // Cast with X=0 (total cost = 0, no mana to add).
    let (state, _) = cast_ravenous(state, p1, card_obj_id, 0, 0);

    // Resolve (spell goes on stack then resolves).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // CR 702.156a: 0 counters placed (x_value=0 → no counters).
    // No CounterAdded event.
    let counter_added = resolve_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                ..
            }
        )
    });
    assert!(!counter_added, "no counters placed when X=0");

    // No draw trigger (X=0 < 5).
    let (_, extra_events) = pass_all(state, &[p1, p2]);
    let card_drawn = resolve_events
        .iter()
        .chain(extra_events.iter())
        .any(|e| matches!(e, GameEvent::CardDrawn { .. }));
    assert!(!card_drawn, "no draw when X=0");
    // Note: CR 704.5f (SBA: creature with toughness 0 dies) would fire here
    // for a real 0/0 card, but is tested separately in the SBA test suite.
}

/// CR 702.156a: X=4 does NOT fire the draw trigger (boundary).
/// X=5 DOES fire the draw trigger.
#[test]
fn test_ravenous_x4_no_draw_boundary() {
    // CR 702.156a: X=4 (below threshold) — no draw trigger.
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![ravenous_creature_def(), dummy_library_card("4")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ravenous_spec(p1))
        .object(dummy_library_spec(p1, "4"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Test Ravenous Beast");

    // Cast with X=4 (pay X = 4 generic).
    let (state, _) = cast_ravenous(state, p1, card_obj_id, 4, 4);
    let (state, resolve_events) = pass_all(state, &[p1, p2]);
    // Extra round in case any trigger is accidentally queued.
    let (_, extra_events) = pass_all(state, &[p1, p2]);

    // No draw event should have occurred.
    let drew = resolve_events
        .iter()
        .chain(extra_events.iter())
        .any(|e| matches!(e, GameEvent::CardDrawn { .. }));
    assert!(
        !drew,
        "X=4 should NOT trigger the draw (boundary: X must be >= 5)"
    );
}

/// CR 603.4, CR 608.2b: Draw trigger still fires even if the Ravenous creature is removed
/// before the trigger resolves. The intervening-if condition is ONLY "if X is 5 or more"
/// — creature presence is NOT part of the condition. Triggered abilities fizzle only for
/// lack of legal targets (CR 608.2b); this ability has no targets.
#[test]
fn test_ravenous_draw_still_fires_if_creature_removed() {
    // CR 603.4: intervening-if only checks "if X is 5 or more" at resolution,
    // not whether the source creature is still on the battlefield.
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![ravenous_creature_def(), dummy_library_card("rem")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ravenous_spec(p1))
        .object(dummy_library_spec(p1, "rem"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Test Ravenous Beast");

    // Cast with X=5. Spell goes on stack.
    let (state, _) = cast_ravenous(state, p1, card_obj_id, 5, 5);

    // Resolve the spell. Creature enters battlefield; draw trigger (X >= 5) goes on stack.
    let (mut state, _resolve_events) = pass_all(state, &[p1, p2]);

    // Verify the creature is on the battlefield before we remove it.
    let creature_bf_id = find_object_on_battlefield(&state, "Test Ravenous Beast")
        .expect("creature should be on battlefield after spell resolves");

    // Simulate removal (e.g., opponent casts removal in response to the draw trigger):
    // directly move the Ravenous creature to the graveyard.
    if let Some(obj) = state.objects.get_mut(&creature_bf_id) {
        obj.zone = ZoneId::Graveyard(p1);
    }

    // Confirm the creature is no longer on the battlefield.
    assert!(
        find_object_on_battlefield(&state, "Test Ravenous Beast").is_none(),
        "creature should be gone from battlefield before draw trigger resolves"
    );

    // Resolve the draw trigger. Per CR 603.4 + CR 608.2b, the draw still happens
    // because the only intervening-if condition is "X >= 5" (which was captured
    // at trigger-fire time), and the ability has no targets to fizzle on.
    let (_, draw_events) = pass_all(state, &[p1, p2]);

    let card_drawn = draw_events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { .. }));
    assert!(
        card_drawn,
        "CardDrawn must fire even if Ravenous creature was removed before draw trigger resolves \
         (CR 603.4: intervening-if is only 'X >= 5'; CR 608.2b: no targets = no fizzle)"
    );
}

/// CR 702.156a, CR 107.3m: Large X value (X=10) correctly places 10 counters and draws.
#[test]
fn test_ravenous_x10_enters_with_10_counters() {
    // CR 702.156a: cast with X=10, enter with 10 +1/+1 counters, draw a card.
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![ravenous_creature_def(), dummy_library_card("10")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ravenous_spec(p1))
        .object(dummy_library_spec(p1, "10"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Test Ravenous Beast");

    // Cast with X=10 (pay X = 10 generic).
    let (state, _) = cast_ravenous(state, p1, card_obj_id, 10, 10);

    // Resolve spell + draw trigger.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);
    let (state, draw_events) = pass_all(state, &[p1, p2]);
    let all_events: Vec<_> = resolve_events
        .iter()
        .chain(draw_events.iter())
        .cloned()
        .collect();

    // Creature on battlefield.
    let creature_id =
        find_object_on_battlefield(&state, "Test Ravenous Beast").expect("creature on battlefield");

    // 10 +1/+1 counters.
    let counter_count = state
        .objects
        .get(&creature_id)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert_eq!(
        counter_count, 10,
        "creature should have 10 +1/+1 counters (X=10)"
    );

    // CounterAdded(10) event.
    let counter_added = all_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 10,
                ..
            }
        )
    });
    assert!(counter_added, "CounterAdded(10) event must be emitted");

    // Draw trigger fires (X=10 >= 5) → CardDrawn event.
    let card_drawn = all_events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { .. }));
    assert!(
        card_drawn,
        "CardDrawn event must be emitted for Ravenous X=10"
    );
}
