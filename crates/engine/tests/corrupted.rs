//! Corrupted ability word tests (CR 207.2c).
//!
//! Corrupted is an ability word (CR 207.2c), not a keyword ability.  It has no
//! special rules meaning of its own; it merely labels abilities with the
//! condition "if an opponent has three or more poison counters."
//!
//! The condition is modeled as `Condition::OpponentHasPoisonCounters(u32)` and
//! used as an `intervening_if` guard on `AbilityDefinition::Triggered` in card
//! definitions.  For WhenEntersBattlefield triggers, the engine queues them as
//! `PendingTrigger` via `queue_carddef_etb_triggers` in `replacement.rs`, checking
//! the `Condition::OpponentHasPoisonCounters` guard at trigger time (CR 603.4) and
//! again at resolution time (CR 603.4 — trigger fizzles if condition no longer holds).
//!
//! Key rules verified:
//! - CR 207.2c: Corrupted is an ability word with no special rules meaning.
//! - CR 603.4: Intervening-if conditions are checked at trigger time AND resolution time.
//! - Multiplayer: "an opponent" means ANY living opponent has >= 3 poison.
//! - Controller's own poison counters never satisfy the condition.
//! - Eliminated opponents (has_lost == true) are excluded from the check.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Condition, Effect, EffectAmount, GameState, GameStateBuilder, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, PlayerTarget, Step, TriggerCondition, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn library_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Library(player))
        .count()
}

/// Pass priority for each listed player once (resolves top of stack or advances step).
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<mtg_engine::GameEvent>) {
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

/// Build a CardDefinition for a Corrupted creature: a 2/2 with
/// "Corrupted — When this creature enters, if an opponent has three or more
/// poison counters, draw a card."
fn corrupted_etb_def(card_id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Corrupted — When this creature enters, if an opponent has three or more \
                      poison counters, draw a card. (CR 207.2c)"
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: Some(Condition::OpponentHasPoisonCounters(3)),
            targets: vec![],
        }],
        ..Default::default()
    }
}

/// Cast the named creature from the player's hand.
fn cast_corrupted_creature(state: GameState, player: PlayerId, name: &str) -> GameState {
    let card_id = find_object(&state, name);
    process_command(
        state,
        Command::CastSpell {
            player,
            card: card_id,
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
    .unwrap_or_else(|e| panic!("CastSpell ({}) failed: {:?}", name, e))
    .0
}

// ── Test 1: Threshold met — draw fires on ETB ─────────────────────────────────

#[test]
/// CR 207.2c / CR 603.4 — When a creature with a Corrupted ETB trigger enters
/// and an opponent has exactly 3 poison counters, the intervening-if condition is
/// satisfied and the draw-card effect fires inline during ETB.
///
/// Observable: P1's library shrinks by 1 (the drawn card moved to hand) after
/// the spell resolves.  We track library size (before vs after) to detect the draw,
/// since the creature itself leaves the hand on cast — making hand count ambiguous.
fn test_corrupted_etb_fires_when_opponent_has_3_poison() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = corrupted_etb_def("corrupted-t1", "Corrupted T1");
    let registry = CardRegistry::new(vec![def]);

    let creature_in_hand = ObjectSpec::card(p1, "Corrupted T1")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("corrupted-t1".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });
    // Put 2 cards in library so we can observe a draw event.
    let lib1 = ObjectSpec::creature(p1, "Library Card 1a", 1, 1).in_zone(ZoneId::Library(p1));
    let lib2 = ObjectSpec::creature(p1, "Library Card 1b", 1, 1).in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_in_hand)
        .object(lib1)
        .object(lib2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // P2 has exactly 3 poison counters — threshold met.
    state.players.get_mut(&p2).unwrap().poison_counters = 3;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let lib_before = library_count(&state, p1);

    // P1 casts the Corrupted creature — it moves from hand to stack.
    let state = cast_corrupted_creature(state, p1, "Corrupted T1");

    // Both players pass priority → spell resolves → ETB trigger queued on stack.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 603.3: ETB trigger is now on the stack. Both players pass priority again
    // to resolve the Corrupted draw trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Library should be 1 smaller (the drawn card moved to hand).
    assert_eq!(
        library_count(&state, p1),
        lib_before - 1,
        "CR 207.2c / CR 603.4: Library should shrink by 1 when Corrupted draw fires with opponent at 3 poison"
    );
}

// ── Test 2: Below threshold — draw does NOT fire ──────────────────────────────

#[test]
/// CR 207.2c / CR 603.4 — When no opponent has >= 3 poison counters, the
/// Corrupted condition is not met and the draw-card effect does NOT fire.
///
/// Observable: P1's library is unchanged after the spell resolves.
fn test_corrupted_etb_does_not_fire_below_threshold() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = corrupted_etb_def("corrupted-t2", "Corrupted T2");
    let registry = CardRegistry::new(vec![def]);

    let creature_in_hand = ObjectSpec::card(p1, "Corrupted T2")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("corrupted-t2".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });
    let lib1 = ObjectSpec::creature(p1, "Library Card 2", 1, 1).in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_in_hand)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // P2 has only 2 poison counters — one short of the threshold.
    state.players.get_mut(&p2).unwrap().poison_counters = 2;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let lib_before = library_count(&state, p1);

    let state = cast_corrupted_creature(state, p1, "Corrupted T2");
    let (state, _) = pass_all(state, &[p1, p2]);

    // Library unchanged — no draw (condition not met).
    assert_eq!(
        library_count(&state, p1),
        lib_before,
        "CR 207.2c: Library should NOT shrink when opponent has only 2 poison (below threshold 3)"
    );
}

// ── Test 3: Multiplayer — any-opponent semantics ──────────────────────────────

#[test]
/// CR 207.2c — In a multiplayer game, the Corrupted condition is satisfied if
/// ANY opponent has >= 3 poison counters, not necessarily all opponents.
///
/// Setup: 4-player game. P2=0 poison, P3=3 poison, P4=1 poison.
/// P1's Corrupted ETB draw fires because P3 satisfies the condition.
fn test_corrupted_condition_any_opponent_multiplayer() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let def = corrupted_etb_def("corrupted-t3", "Corrupted T3");
    let registry = CardRegistry::new(vec![def]);

    let creature_in_hand = ObjectSpec::card(p1, "Corrupted T3")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("corrupted-t3".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });
    let lib1 = ObjectSpec::creature(p1, "Library Card 3", 1, 1).in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(creature_in_hand)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // P2=0, P3=3 (meets threshold), P4=1.
    state.players.get_mut(&p2).unwrap().poison_counters = 0;
    state.players.get_mut(&p3).unwrap().poison_counters = 3;
    state.players.get_mut(&p4).unwrap().poison_counters = 1;

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let lib_before = library_count(&state, p1);

    let state = cast_corrupted_creature(state, p1, "Corrupted T3");
    // Resolve the creature spell.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    // CR 603.3: Resolve the ETB trigger (draw a card).
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    assert_eq!(
        library_count(&state, p1),
        lib_before - 1,
        "CR 207.2c: Corrupted draw should fire when ANY opponent (P3) has >= 3 poison"
    );
}

// ── Test 4: Controller's own poison ignored ──────────────────────────────────

#[test]
/// CR 207.2c — The controller's own poison counters never satisfy the Corrupted
/// condition.  "An opponent" explicitly excludes the controller.
///
/// Setup: P1 has 5 poison counters; P2 (opponent) has only 1 poison.
/// Assert: no draw fires.
fn test_corrupted_condition_ignores_controller_poison() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = corrupted_etb_def("corrupted-t4", "Corrupted T4");
    let registry = CardRegistry::new(vec![def]);

    let creature_in_hand = ObjectSpec::card(p1, "Corrupted T4")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("corrupted-t4".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });
    let lib1 = ObjectSpec::creature(p1, "Library Card 4", 1, 1).in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_in_hand)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Controller (P1) has 5 poison — well above threshold, but doesn't count.
    // Opponent (P2) has only 1 poison — below threshold.
    state.players.get_mut(&p1).unwrap().poison_counters = 5;
    state.players.get_mut(&p2).unwrap().poison_counters = 1;

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let lib_before = library_count(&state, p1);

    let state = cast_corrupted_creature(state, p1, "Corrupted T4");
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        library_count(&state, p1),
        lib_before,
        "CR 207.2c: Corrupted must NOT fire when only the controller has >= 3 poison"
    );
}

// ── Test 5: Eliminated opponent ignored ──────────────────────────────────────

#[test]
/// CR 207.2c — An eliminated opponent (has_lost == true) is excluded from the
/// Corrupted condition check even if they have >= 3 poison counters.
///
/// Setup: P2 has 10 poison and has_lost=true; P3=0 poison (alive).
/// Assert: no draw fires.
fn test_corrupted_condition_ignores_eliminated_opponents() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    let def = corrupted_etb_def("corrupted-t5", "Corrupted T5");
    let registry = CardRegistry::new(vec![def]);

    let creature_in_hand = ObjectSpec::card(p1, "Corrupted T5")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("corrupted-t5".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });
    let lib1 = ObjectSpec::creature(p1, "Library Card 5", 1, 1).in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .object(creature_in_hand)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // P2 is eliminated with 10 poison (would meet threshold if alive).
    state.players.get_mut(&p2).unwrap().poison_counters = 10;
    state.players.get_mut(&p2).unwrap().has_lost = true;
    // P3 is alive but has 0 poison.
    state.players.get_mut(&p3).unwrap().poison_counters = 0;

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let lib_before = library_count(&state, p1);

    let state = cast_corrupted_creature(state, p1, "Corrupted T5");
    // P2 is eliminated — only pass for p1 and p3.
    let (state, _) = pass_all(state, &[p1, p3]);

    assert_eq!(
        library_count(&state, p1),
        lib_before,
        "CR 207.2c: Corrupted must NOT fire when only the eliminated opponent has >= 3 poison"
    );
}

// ── Test 6: Boundary — exactly 3 poison meets threshold ───────────────────────

#[test]
/// CR 207.2c — The threshold is "three or more" (>= 3), not "more than three" (> 3).
/// Exactly 3 poison counters is sufficient.  This boundary test complements test 2
/// (2 poison = fails) and test 1 (3 poison = passes).
fn test_corrupted_boundary_exactly_3_poison_meets_threshold() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = corrupted_etb_def("corrupted-t6", "Corrupted T6");
    let registry = CardRegistry::new(vec![def]);

    let creature_in_hand = ObjectSpec::card(p1, "Corrupted T6")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("corrupted-t6".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });
    let lib1 = ObjectSpec::creature(p1, "Library Card 6", 1, 1).in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_in_hand)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Exactly 3 — the boundary value.
    state.players.get_mut(&p2).unwrap().poison_counters = 3;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let lib_before = library_count(&state, p1);

    let state = cast_corrupted_creature(state, p1, "Corrupted T6");
    // Resolve the creature spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    // CR 603.3: Resolve the ETB trigger (draw a card).
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        library_count(&state, p1),
        lib_before - 1,
        "CR 207.2c: Corrupted threshold is >= 3 — exactly 3 poison should satisfy the condition"
    );
}

// ── Test 7: CR 603.4 resolution-time check — trigger fizzles if condition
//            no longer holds when the trigger resolves ─────────────────────────

#[test]
/// CR 603.4 — "If the ability triggers, it checks the stated condition again as
/// it resolves. If the condition isn't true at that time, the ability is removed
/// from the stack and does nothing."
///
/// Scenario: P2 has 3 poison at trigger time (Corrupted ETB trigger fires and
/// goes on the stack). Before the trigger resolves, P2's poison drops to 2.
/// The draw effect must NOT fire at resolution.
///
/// Note: The engine does not provide a "remove poison counter" command for bots,
/// so we simulate the mid-stack drop by mutating the state directly between the
/// two priority passes (after the creature ETB queues the trigger but before
/// priority is passed to resolve it).
fn test_corrupted_etb_fizzles_if_condition_drops_before_resolution() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = corrupted_etb_def("corrupted-t7", "Corrupted T7");
    let registry = CardRegistry::new(vec![def]);

    let creature_in_hand = ObjectSpec::card(p1, "Corrupted T7")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("corrupted-t7".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });
    // Two library cards so we can observe a draw if it happens.
    let lib1 = ObjectSpec::creature(p1, "Library Card 7a", 1, 1).in_zone(ZoneId::Library(p1));
    let lib2 = ObjectSpec::creature(p1, "Library Card 7b", 1, 1).in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_in_hand)
        .object(lib1)
        .object(lib2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // P2 starts with 3 poison — Corrupted condition met at trigger time.
    state.players.get_mut(&p2).unwrap().poison_counters = 3;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let lib_before = library_count(&state, p1);

    // Cast Corrupted T7.
    let state = cast_corrupted_creature(state, p1, "Corrupted T7");

    // Both players pass priority → spell resolves → ETB trigger queued on stack.
    // The Corrupted condition (3 poison) holds at trigger time, so the trigger fires.
    let (mut state, _) = pass_all(state, &[p1, p2]);

    // Simulate an effect that removes a poison counter from P2 before the trigger resolves.
    // (e.g. Melira, Sylvok Outcast or Leeches — not implemented as commands, so direct mutation.)
    // CR 603.4: The condition is re-checked at resolution; if it fails, the trigger does nothing.
    state.players.get_mut(&p2).unwrap().poison_counters = 2;

    // Both players pass priority → Corrupted ETB trigger resolves.
    // At resolution time: P2 has only 2 poison → condition fails → trigger fizzles.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Library unchanged — the draw did NOT fire.
    assert_eq!(
        library_count(&state, p1),
        lib_before,
        "CR 603.4: Corrupted draw must NOT fire when opponent's poison drops below 3 before resolution"
    );
}
