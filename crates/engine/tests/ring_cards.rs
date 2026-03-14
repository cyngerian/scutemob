//! Card definition tests for The Ring Tempts You mechanic — Call of the Ring (CR 701.54).
//!
//! Tests cover:
//! - Call of the Ring ETB triggers the Ring tempts the controller (CR 701.54a, CR 603.3)
//! - ring_tempts_you harness action translates to Command::TheRingTemptsYou (CR 701.54a)

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, GameEvent, GameStateBuilder, ObjectSpec, PlayerId, Step,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Build the card-definition map and registry, returned as a tuple.
fn build_defs_and_registry() -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);
    (defs, registry)
}

/// Build an ObjectSpec enriched from its card definition.
fn make_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

// ── Call of the Ring ──────────────────────────────────────────────────────────

/// CR 701.54a / CR 603.3: When Call of the Ring enters the battlefield, its ETB
/// triggered ability "the Ring tempts you" fires. The controller's ring level should
/// advance from 0 to 1, and a creature should be chosen as ring-bearer.
///
/// Setup: p1 has a Bog Raiders on the battlefield (to serve as ring-bearer) and
/// Call of the Ring in hand with enough mana ({3}{B}).
///
/// Expected: After Call of the Ring resolves and the ETB trigger resolves,
/// RingTempted event fires with new_level=1, and RingBearerChosen event fires
/// pointing to Bog Raiders.
///
/// Source: Call of the Ring oracle text; CR 701.54a (ring temptation keyword action);
/// CR 603.3 (ETB triggers go on the stack).
#[test]
fn test_call_of_the_ring_etb_tempts() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    // Bog Raiders is a simple 2/2 creature — serves as ring-bearer.
    let bog_raiders = make_spec(p1, "Bog Raiders", ZoneId::Battlefield, &defs);
    let call_of_the_ring = make_spec(p1, "Call of the Ring", ZoneId::Hand(p1), &defs);

    let mut state = GameStateBuilder::new()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bog_raiders)
        .object(call_of_the_ring)
        .build()
        .unwrap();

    // Give p1 mana for Call of the Ring ({3}{B}).
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool.colorless += 3;
        ps.mana_pool.black += 1;
    }

    // Find Call of the Ring in p1's hand.
    let cotr_id = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Call of the Ring" && obj.zone == ZoneId::Hand(p1) {
                Some(id)
            } else {
                None
            }
        })
        .expect("Call of the Ring should be in p1 hand");

    // Cast Call of the Ring.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: cotr_id,
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
        },
    )
    .expect("casting Call of the Ring should succeed");

    // Both players pass priority → Call of the Ring resolves → enters battlefield → ETB trigger queued.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).expect("pass p1");
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).expect("pass p2");

    // Call of the Ring should now be on the battlefield.
    let cotr_on_battlefield = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Call of the Ring" && o.zone == ZoneId::Battlefield);
    assert!(
        cotr_on_battlefield,
        "Call of the Ring should be on the battlefield after resolving"
    );

    // ETB trigger (the Ring tempts you) should be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "ETB 'the Ring tempts you' trigger should be on the stack"
    );

    // Both players pass priority → ETB trigger resolves → Effect::TheRingTemptsYou fires.
    let (state, etb_events) =
        process_command(state, Command::PassPriority { player: p1 }).expect("pass p1");
    let (state, etb_events2) =
        process_command(state, Command::PassPriority { player: p2 }).expect("pass p2");

    let all_events: Vec<_> = etb_events.into_iter().chain(etb_events2).collect();

    // CR 701.54a: RingTempted event should fire with new_level = 1.
    let ring_tempted = all_events
        .iter()
        .any(|e| matches!(e, GameEvent::RingTempted { player, new_level: 1 } if *player == p1));
    assert!(
        ring_tempted,
        "RingTempted event with new_level=1 should fire when Call of the Ring ETB trigger resolves"
    );

    // CR 701.54b: p1's ring level should be 1.
    let ring_level = state.players.get(&p1).map(|ps| ps.ring_level).unwrap_or(0);
    assert_eq!(
        ring_level, 1,
        "p1 ring level should be 1 after Call of the Ring ETB trigger resolves"
    );

    // CR 701.54b: RingBearerChosen event should fire (Bog Raiders chosen as ring-bearer).
    let bearer_chosen = all_events
        .iter()
        .any(|e| matches!(e, GameEvent::RingBearerChosen { player, .. } if *player == p1));
    assert!(
        bearer_chosen,
        "RingBearerChosen event should be emitted when ring tempts p1 (Bog Raiders is the bearer)"
    );

    // p1's ring_bearer_id should be set to Bog Raiders.
    let ring_bearer_id = state.players.get(&p1).and_then(|ps| ps.ring_bearer_id);
    assert!(
        ring_bearer_id.is_some(),
        "p1 should have a ring-bearer after the Ring tempts them"
    );

    // The ring-bearer should be Bog Raiders.
    let bearer_is_bog_raiders = ring_bearer_id.is_some_and(|id| {
        state
            .objects
            .get(&id)
            .is_some_and(|o| o.characteristics.name == "Bog Raiders")
    });
    assert!(
        bearer_is_bog_raiders,
        "Bog Raiders should be the ring-bearer (only creature controlled by p1)"
    );

    // Stack should be empty after both trigger resolutions.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after ETB trigger resolved"
    );
}

// ── ring_tempts_you harness action ────────────────────────────────────────────

/// CR 701.54a: The `ring_tempts_you` harness action translates to
/// `Command::TheRingTemptsYou` for the given player.
///
/// This test verifies that the Command works directly (not via card ETB). It mirrors
/// the unit tests in ring_tempts_you.rs but uses the Command API rather than calling
/// handle_ring_tempts_you() directly.
///
/// Source: CR 701.54a (keyword action "the Ring tempts you").
#[test]
fn test_ring_tempts_you_harness_action() {
    let (_, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    // Add a creature so a ring-bearer can be chosen.
    let creature = ObjectSpec::creature(p1, "Bearer Creature", 2, 2);

    let state = GameStateBuilder::new()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature)
        .build()
        .unwrap();

    // Verify initial ring level is 0.
    let initial_level = state.players.get(&p1).map(|ps| ps.ring_level).unwrap_or(0);
    assert_eq!(initial_level, 0, "ring level should start at 0");

    // Issue Command::TheRingTemptsYou directly (this is what the harness action translates to).
    let (state, events) = process_command(state, Command::TheRingTemptsYou { player: p1 })
        .expect("TheRingTemptsYou command should succeed");

    // CR 701.54a: Ring level should advance to 1.
    let ring_level = state.players.get(&p1).map(|ps| ps.ring_level).unwrap_or(0);
    assert_eq!(
        ring_level, 1,
        "ring level should advance to 1 after Command::TheRingTemptsYou"
    );

    // CR 701.54b: RingTempted event with new_level=1 should be emitted.
    let ring_tempted = events
        .iter()
        .any(|e| matches!(e, GameEvent::RingTempted { player, new_level: 1 } if *player == p1));
    assert!(
        ring_tempted,
        "RingTempted event with new_level=1 should be emitted by Command::TheRingTemptsYou"
    );

    // CR 701.54b: RingBearerChosen event should be emitted (Bearer Creature chosen).
    let bearer_chosen = events
        .iter()
        .any(|e| matches!(e, GameEvent::RingBearerChosen { player, .. } if *player == p1));
    assert!(
        bearer_chosen,
        "RingBearerChosen event should be emitted when Command::TheRingTemptsYou fires"
    );

    // p1's ring_bearer_id should be set.
    let ring_bearer_id = state.players.get(&p1).and_then(|ps| ps.ring_bearer_id);
    assert!(
        ring_bearer_id.is_some(),
        "p1 should have a ring-bearer after Command::TheRingTemptsYou"
    );

    // p2's ring level should be unaffected.
    let p2_ring_level = state.players.get(&p2).map(|ps| ps.ring_level).unwrap_or(0);
    assert_eq!(
        p2_ring_level, 0,
        "p2 ring level should remain 0 (only p1 was tempted)"
    );
}
