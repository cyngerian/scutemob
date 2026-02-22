//! Tests for stack resolution (CR 608).
//!
//! M3-C: all-pass → resolve top, LIFO order, graveyard/battlefield destination,
//! priority reset after resolution, and countering.

use mtg_engine::rules::resolution::counter_stack_object;
use mtg_engine::rules::{process_command, Command, GameEvent};
use mtg_engine::state::turn::Step;
use mtg_engine::state::{CardType, GameStateBuilder, KeywordAbility, ObjectSpec, PlayerId, ZoneId};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Helper: pass priority for all four players (p1 → p2 → p3 → p4) in order.
/// Returns the final (state, all_events) after the last pass.
/// Panics if any pass fails.
fn pass_all_four(
    state: mtg_engine::GameState,
    turn_order: [PlayerId; 4],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let mut s = state;
    let mut all_events = Vec::new();

    for player in &turn_order {
        let (ns, evs) = process_command(s, Command::PassPriority { player: *player }).unwrap();
        all_events.extend(evs);
        s = ns;
    }

    (s, all_events)
}

// ---------------------------------------------------------------------------
// CR 608.1 / CR 608.2n: instant/sorcery spells go to graveyard on resolution
// ---------------------------------------------------------------------------

#[test]
/// CR 608.1, CR 608.2n — when all players pass with a sorcery on the stack,
/// the sorcery moves to its owner's graveyard.
fn test_608_1_sorcery_resolves_to_graveyard() {
    let p1 = p(1);
    let sorcery = ObjectSpec::card(p1, "Mind Rot")
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(sorcery)
        .build();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Cast the sorcery.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();
    assert_eq!(state.stack_objects.len(), 1);

    // All four players pass — active player (p1) has priority after cast.
    let (final_state, events) = pass_all_four(state, [p(1), p(2), p(3), p(4)]);

    // Stack is now empty.
    assert!(final_state.stack_objects.is_empty());
    assert!(final_state.zones.get(&ZoneId::Stack).unwrap().is_empty());

    // Card is in the owner's graveyard (not in hand or stack).
    assert!(final_state.zones.get(&ZoneId::Hand(p1)).unwrap().is_empty());
    assert_eq!(
        final_state.zones.get(&ZoneId::Graveyard(p1)).unwrap().len(),
        1
    );

    // SpellResolved event emitted.
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellResolved { player, .. } if *player == p1)));
    // No PermanentEnteredBattlefield — this is a sorcery.
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. })));
}

#[test]
/// CR 608.1, CR 608.2n — instant spell resolves to graveyard at instant speed.
fn test_608_1_instant_resolves_to_graveyard() {
    let p1 = p(1);
    let p2 = p(2);
    // p2 casts an instant during p1's upkeep (instant speed — not active player).
    let instant = ObjectSpec::card(p2, "Brainstorm")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(instant)
        .build();

    // p1 has priority at start of upkeep — pass to p2.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    // p2 has priority; cast the instant.
    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p2))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // After CastSpell, active player (p1) gets priority. All four pass.
    let (final_state, events) = pass_all_four(state, [p(1), p(2), p(3), p(4)]);

    assert!(final_state.stack_objects.is_empty());
    assert_eq!(
        final_state.zones.get(&ZoneId::Graveyard(p2)).unwrap().len(),
        1
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellResolved { player, .. } if *player == p2)));
}

// ---------------------------------------------------------------------------
// CR 608.3a: permanent spells enter the battlefield on resolution
// ---------------------------------------------------------------------------

#[test]
/// CR 608.3a — when all players pass with a creature spell on the stack,
/// the creature enters the battlefield under the caster's control.
fn test_608_3a_creature_enters_battlefield() {
    let p1 = p(1);
    let creature = ObjectSpec::card(p1, "Grizzly Bears")
        .with_types(vec![CardType::Creature])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    let (final_state, events) = pass_all_four(state, [p(1), p(2), p(3), p(4)]);

    // Stack and Stack zone are empty.
    assert!(final_state.stack_objects.is_empty());
    assert!(final_state.zones.get(&ZoneId::Stack).unwrap().is_empty());

    // Creature is on the battlefield — not in graveyard.
    assert_eq!(
        final_state.zones.get(&ZoneId::Battlefield).unwrap().len(),
        1
    );
    assert!(final_state
        .zones
        .get(&ZoneId::Graveyard(p1))
        .unwrap()
        .is_empty());

    // PermanentEnteredBattlefield event emitted.
    assert!(events.iter().any(
        |e| matches!(e, GameEvent::PermanentEnteredBattlefield { player, .. } if *player == p1)
    ));
    // SpellResolved also emitted.
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellResolved { player, .. } if *player == p1)));

    // The permanent's controller is the caster.
    let new_id = final_state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()[0];
    let perm = final_state.objects.get(&new_id).unwrap();
    assert_eq!(perm.controller, p1);
    assert_eq!(perm.owner, p1);
}

#[test]
/// CR 608.3a — artifact spell resolves to battlefield.
fn test_608_3a_artifact_enters_battlefield() {
    let p1 = p(1);
    let artifact = ObjectSpec::card(p1, "Sol Ring")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(artifact)
        .build();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    let (final_state, _events) = pass_all_four(state, [p(1), p(2), p(3), p(4)]);

    assert!(final_state.stack_objects.is_empty());
    assert_eq!(
        final_state.zones.get(&ZoneId::Battlefield).unwrap().len(),
        1
    );
}

// ---------------------------------------------------------------------------
// CR 116.3b: priority resets to active player after resolution
// ---------------------------------------------------------------------------

#[test]
/// CR 116.3b — after a spell resolves, the active player receives priority.
fn test_608_1_priority_goes_to_active_player_after_resolution() {
    let p1 = p(1);
    let p2 = p(2);
    // p2 casts an instant on p1's turn.
    let instant = ObjectSpec::card(p2, "Lightning Bolt")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(instant)
        .build();

    // p1 passes → p2 has priority.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p2))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // After cast, p1 (active player) gets priority. All four pass to resolve.
    let (final_state, _events) = pass_all_four(state, [p(1), p(2), p(3), p(4)]);

    // After resolution, p1 (active player) should hold priority.
    assert_eq!(final_state.turn.priority_holder, Some(p1));
    // players_passed has been reset.
    assert!(final_state.turn.players_passed.is_empty());
}

// ---------------------------------------------------------------------------
// CR 608.1: LIFO ordering — second all-pass resolves next item on stack
// ---------------------------------------------------------------------------

#[test]
/// CR 608.1 — with two spells on the stack, all-pass resolves the top one
/// (LIFO). A second all-pass resolves the next one.
fn test_608_1_lifo_resolves_top_first() {
    let p1 = p(1);
    let sorcery1 = ObjectSpec::card(p1, "Divination")
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));
    let instant2 = ObjectSpec::card(p1, "Brainstorm")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(sorcery1)
        .object(instant2)
        .build();

    // Find both cards in hand.
    let hand_ids: Vec<_> = state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .to_vec();
    assert_eq!(hand_ids.len(), 2);

    // Cast the sorcery first (empty stack required for sorcery speed).
    let (sorcery_id, instant_id) = {
        let objs = &state.objects;
        let mut sorcery = None;
        let mut instant = None;
        for id in &hand_ids {
            if objs[id]
                .characteristics
                .card_types
                .contains(&CardType::Sorcery)
            {
                sorcery = Some(*id);
            } else {
                instant = Some(*id);
            }
        }
        (sorcery.unwrap(), instant.unwrap())
    };

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: sorcery_id,
            targets: vec![],
        },
    )
    .unwrap();
    assert_eq!(state.stack_objects.len(), 1);

    // Now cast the instant on top of the sorcery.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: instant_id,
            targets: vec![],
        },
    )
    .unwrap();
    assert_eq!(state.stack_objects.len(), 2);

    // The instant is on top (pushed last).
    let top = state.stack_objects.last().unwrap();
    let top_source = match &top.kind {
        mtg_engine::StackObjectKind::Spell { source_object } => *source_object,
        _ => panic!("expected spell"),
    };
    let top_card_types = state.objects[&top_source]
        .characteristics
        .card_types
        .clone();
    assert!(top_card_types.contains(&CardType::Instant));

    // First all-pass: instant resolves (→ graveyard).
    let (state, events1) = pass_all_four(state, [p(1), p(2), p(3), p(4)]);
    assert_eq!(state.stack_objects.len(), 1, "one spell still on stack");
    assert!(events1
        .iter()
        .any(|e| matches!(e, GameEvent::SpellResolved { .. })));

    // Second all-pass: sorcery resolves (→ graveyard).
    let (final_state, events2) = pass_all_four(state, [p(1), p(2), p(3), p(4)]);
    assert!(
        final_state.stack_objects.is_empty(),
        "stack empty after second resolution"
    );
    assert!(events2
        .iter()
        .any(|e| matches!(e, GameEvent::SpellResolved { .. })));

    // Both cards now in graveyard.
    assert_eq!(
        final_state.zones.get(&ZoneId::Graveyard(p1)).unwrap().len(),
        2
    );
}

// ---------------------------------------------------------------------------
// Regression: empty stack all-pass still advances step
// ---------------------------------------------------------------------------

#[test]
/// CR 500.4 — with an empty stack, all players passing advances the step.
/// (Regression: ensure the stack-resolution branch doesn't break step advancement.)
fn test_608_1_empty_stack_all_pass_advances_step() {
    let state = GameStateBuilder::four_player()
        .active_player(p(1))
        .at_step(Step::Upkeep)
        .build();

    // All four pass priority — should advance past Upkeep.
    let (final_state, events) = pass_all_four(state, [p(1), p(2), p(3), p(4)]);

    // Should have advanced to a new step.
    assert_ne!(final_state.turn.step, Step::Upkeep);
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::StepChanged { .. })));
    // Stack stays empty.
    assert!(final_state.stack_objects.is_empty());
}

// ---------------------------------------------------------------------------
// counter_stack_object: spell countered → goes to graveyard without resolving
// ---------------------------------------------------------------------------

#[test]
/// CR 608.2b, CR 701.5 — counter_stack_object removes a spell from the stack
/// and puts it in the owner's graveyard without it resolving.
fn test_counter_stack_object_spell_to_graveyard() {
    let p1 = p(1);
    let sorcery = ObjectSpec::card(p1, "Divination")
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(sorcery)
        .build();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (mut state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // Grab the stack_object_id from the SpellCast event.
    let stack_object_id = events
        .iter()
        .find_map(|e| {
            if let GameEvent::SpellCast {
                stack_object_id, ..
            } = e
            {
                Some(*stack_object_id)
            } else {
                None
            }
        })
        .unwrap();

    assert_eq!(state.stack_objects.len(), 1);

    // Counter the spell directly (bypassing command dispatch — M3-D will hook this up properly).
    let counter_events = counter_stack_object(&mut state, stack_object_id).unwrap();

    // Stack is empty.
    assert!(state.stack_objects.is_empty());
    assert!(state.zones.get(&ZoneId::Stack).unwrap().is_empty());

    // Card is in graveyard, not on battlefield.
    assert_eq!(state.zones.get(&ZoneId::Graveyard(p1)).unwrap().len(), 1);
    assert!(state.zones.get(&ZoneId::Battlefield).unwrap().is_empty());

    // SpellCountered event emitted, NOT SpellResolved.
    assert!(counter_events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellCountered { player, .. } if *player == p1)));
    assert!(!counter_events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellResolved { .. })));

    // Active player (p1) gets priority after countering.
    assert_eq!(state.turn.priority_holder, Some(p1));
}

#[test]
/// CR 701.5 — a countered permanent spell goes to graveyard, not battlefield.
fn test_counter_stack_object_permanent_to_graveyard_not_battlefield() {
    let p1 = p(1);
    let creature = ObjectSpec::card(p1, "Serra Angel")
        .with_types(vec![CardType::Creature])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (mut state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    let stack_object_id = events
        .iter()
        .find_map(|e| {
            if let GameEvent::SpellCast {
                stack_object_id, ..
            } = e
            {
                Some(*stack_object_id)
            } else {
                None
            }
        })
        .unwrap();

    counter_stack_object(&mut state, stack_object_id).unwrap();

    // Creature must NOT be on battlefield.
    assert!(state.zones.get(&ZoneId::Battlefield).unwrap().is_empty());
    // Creature IS in graveyard.
    assert_eq!(state.zones.get(&ZoneId::Graveyard(p1)).unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// Flash keyword: instant-speed casting still resolves correctly
// ---------------------------------------------------------------------------

#[test]
/// CR 702.36, CR 608.1 — a creature with Flash cast at instant speed resolves
/// to the battlefield.
fn test_608_flash_creature_resolves_to_battlefield() {
    let p1 = p(1);
    let flash_creature = ObjectSpec::card(p1, "Briarhorn")
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Flash)
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep) // Not main phase — valid only due to Flash
        .object(flash_creature)
        .build();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // p1 can cast at instant speed (Flash).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    let (final_state, events) = pass_all_four(state, [p(1), p(2), p(3), p(4)]);

    assert!(final_state.stack_objects.is_empty());
    assert_eq!(
        final_state.zones.get(&ZoneId::Battlefield).unwrap().len(),
        1
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. })));
}
