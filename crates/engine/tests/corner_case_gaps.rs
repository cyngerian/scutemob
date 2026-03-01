//! Gap tests for corner cases with no prior test coverage (CC#23).
//!
//! These tests verify interactions that were previously untested ("GAP" status
//! in the corner-case audit). Each test documents the relevant CR citation
//! and the specific interaction it validates.

use mtg_engine::rules::{process_command, Command, GameEvent};
use mtg_engine::state::turn::Step;
use mtg_engine::state::{CardType, GameStateBuilder, ObjectSpec, PlayerId, Target, ZoneId};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

// ---------------------------------------------------------------------------
// CC#23: Flicker + object identity (CR 400.7 + CR 608.2b)
// ---------------------------------------------------------------------------

#[test]
/// CC#23 — CR 400.7, CR 608.2b:
/// A creature is "flickered" (exiled then returned to the battlefield) in response
/// to a kill spell targeting it.
///
/// When the kill spell resolves, its original target ObjectId no longer exists
/// (zone change produced a new object — CR 400.7). Since the single target is
/// illegal, the spell fizzles (CR 608.2b). No `CreatureDied` event fires because
/// the creature was never destroyed — it was exiled and returned. The returned
/// creature has a new ObjectId distinct from both the original and the exile copy.
fn test_cc23_flicker_kills_spell_fizzles_no_dies_trigger() {
    let p1 = p(1);
    let p2 = p(2);

    // p1's creature on the battlefield — the flicker target.
    let creature = ObjectSpec::creature(p1, "Grizzly Bears", 2, 2);
    // p2's kill spell in hand — targets the creature.
    let kill_spell = ObjectSpec::card(p2, "Terror")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    // Build state: p1 is active, but an instant can be cast any time.
    // We place it at Upkeep so p2 can respond with priority.
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(creature)
        .object(kill_spell)
        .build()
        .unwrap();

    // Find the original creature ObjectId (on the battlefield).
    let original_creature_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Find the kill spell's ObjectId (in p2's hand).
    let kill_spell_id = *state
        .zones
        .get(&ZoneId::Hand(p2))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // p1 has priority at Upkeep. p1 passes to p2.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    // p2 casts the kill spell targeting the creature.
    // CR 601.2c: target is validated at cast time (creature is on battlefield → legal).
    let (mut state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: kill_spell_id,
            targets: vec![Target::Object(original_creature_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    )
    .unwrap();
    assert_eq!(
        state.stack_objects.len(),
        1,
        "kill spell should be on the stack"
    );

    // ── Flicker: exile the creature then return it to the battlefield ──
    // This simulates an instant-speed "flicker" effect (e.g., Cloudshift, Momentary Blink)
    // that p1 would cast in response. We manipulate state directly here to isolate
    // the CR 400.7 + 608.2b interaction from the flicker mechanic itself.
    //
    // Step 1: exile the original creature.
    // CR 400.7: zone change produces a new object; original_creature_id is now dead.
    let (exile_id, _old_snapshot) = state
        .move_object_to_zone(original_creature_id, ZoneId::Exile)
        .unwrap();

    // CR 400.7: the exile object is a NEW object — different ID from original.
    assert_ne!(
        exile_id, original_creature_id,
        "CR 400.7: exile step must produce a new ObjectId"
    );

    // Step 2: return the creature from exile to the battlefield.
    // CR 400.7: another zone change produces yet another new object.
    let (returned_id, _) = state
        .move_object_to_zone(exile_id, ZoneId::Battlefield)
        .unwrap();

    // CR 400.7: the returned creature is a NEW object again.
    assert_ne!(
        returned_id, original_creature_id,
        "CR 400.7: returned creature must have a new ObjectId, distinct from the original"
    );
    assert_ne!(
        returned_id, exile_id,
        "CR 400.7: returned creature must have a new ObjectId, distinct from the exile copy"
    );

    // ── Resolution: pass priority until the stack resolves ──
    // The kill spell's target (original_creature_id) no longer exists in the game state.
    // CR 608.2b: all targets illegal → spell fizzles.
    let mut all_events = Vec::new();
    // p2 has priority after casting; pass until all 4 players pass.
    for _ in 0..8 {
        if state.stack_objects.is_empty() {
            break;
        }
        let holder = match state.turn.priority_holder {
            Some(h) => h,
            None => break,
        };
        let (ns, evs) = process_command(state, Command::PassPriority { player: holder }).unwrap();
        all_events.extend(evs);
        state = ns;
    }

    // ── Assertions ──

    // Stack should be empty after fizzle.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after spell fizzles"
    );

    // CR 608.2b: SpellFizzled must be emitted (not SpellResolved).
    assert!(
        all_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { player, .. } if *player == p2)),
        "CR 608.2b: SpellFizzled must be emitted when all targets are illegal"
    );

    // SpellResolved must NOT be emitted.
    assert!(
        !all_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellResolved { .. })),
        "SpellResolved must NOT be emitted — spell fizzled"
    );

    // CR 400.7 / no-dies: CreatureDied must NOT be emitted.
    // The creature was exiled (flickered), never destroyed. Exile is not death.
    assert!(
        !all_events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 400.7: CreatureDied must NOT fire — creature was exiled, not destroyed"
    );

    // CR 400.7: The returned creature is a new object on the battlefield.
    let bf_objects = state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(
        bf_objects.len(),
        1,
        "exactly one creature should be on the battlefield after flicker"
    );
    assert_eq!(
        bf_objects[0].id, returned_id,
        "the creature on the battlefield should be the returned (new) object"
    );
    assert_ne!(
        bf_objects[0].id, original_creature_id,
        "CR 400.7: the creature's current ObjectId must differ from the pre-flicker ObjectId"
    );
}
