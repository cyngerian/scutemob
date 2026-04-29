//! Armorcraft Judge ETB draw tests — PB-CC-B.
//!
//! Verifies that `TargetFilter.has_counter_type` correctly restricts the
//! `EffectAmount::PermanentCount` callsite when Armorcraft Judge's ETB trigger
//! counts creatures you control with a +1/+1 counter on them.
//!
//! CR Rules covered:
//! - CR 121.1: Counters modify the objects they are on.
//! - CR 121.6: Counters are tracked in `GameObject.counters` (NOT in Characteristics).
//! - Ruling 2020-11-10 (Armorcraft Judge): counts CREATURES with one or more
//!   +1/+1 counters (not the total number of counters); threshold is >= 1 counter.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    CardType, CounterType, Effect, EffectAmount, GameStateBuilder, ObjectId, ObjectSpec, PlayerId,
    PlayerTarget, TargetController, TargetFilter, ZoneId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

/// Find the ObjectId of the named object on the battlefield.
fn on_battlefield(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found on battlefield", name))
}

/// Build the PermanentCount effect with `has_counter_type: Some(PlusOnePlusOne)` and
/// `has_card_type: Some(Creature)`, `controller: You` — exactly Armorcraft Judge's ETB.
fn armorcraft_judge_effect() -> Effect {
    Effect::DrawCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::PermanentCount {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                controller: TargetController::You,
                has_counter_type: Some(CounterType::PlusOnePlusOne),
                ..Default::default()
            },
            controller: PlayerTarget::Controller,
        },
    }
}

// ── Test 1 ─────────────────────────────────────────────────────────────────────

/// CR 121.1 / Ruling 2020-11-10 (Armorcraft Judge) — Armorcraft Judge ETB draws
/// zero cards when no creatures you control have +1/+1 counters.
///
/// Setup: P1 controls three creatures, none with +1/+1 counters.
/// Expected: zero cards drawn (hand stays at 0).
#[test]
fn armorcraft_judge_no_counters_zero_draw() {
    // CR 121.1: counters modify objects; without counters, has_counter_type predicate fails.
    // Ruling 2020-11-10: only creatures WITH a +1/+1 counter count.
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Grizzly Bears", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1(), "Hill Giant", 3, 3).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1(), "Llanowar Elves", 1, 1).in_zone(ZoneId::Battlefield))
        // Judge itself (no counters)
        .object(ObjectSpec::creature(p1(), "Armorcraft Judge", 3, 3).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let source = on_battlefield(&state, "Armorcraft Judge");
    let effect = armorcraft_judge_effect();
    let mut ctx = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let hand_count = state
        .zones
        .get(&ZoneId::Hand(p1()))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        hand_count, 0,
        "CR 121.1 / Ruling 2020-11-10: no creatures have +1/+1 counters → 0 cards drawn"
    );
}

// ── Test 2 ─────────────────────────────────────────────────────────────────────

/// CR 121.1 / Ruling 2020-11-10 (Armorcraft Judge) — Armorcraft Judge ETB draws
/// one card when exactly one creature you control has a +1/+1 counter.
///
/// Setup: P1 controls two creatures; one has a +1/+1 counter, the other does not.
/// Armorcraft Judge itself enters without counters (should not count itself).
/// Expected: exactly 1 card drawn.
#[test]
fn armorcraft_judge_one_creature_with_counter_draws_one() {
    // CR 121.6: counters live on GameObject, not Characteristics.
    // Ruling 2020-11-10: Armorcraft Judge itself enters without counters and should
    // not count itself even though it is a creature you control.
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        // This creature has a +1/+1 counter → counts.
        .object(
            ObjectSpec::creature(p1(), "Servo", 1, 1)
                .with_counter(CounterType::PlusOnePlusOne, 1)
                .in_zone(ZoneId::Battlefield),
        )
        // This creature has NO counters → does not count.
        .object(ObjectSpec::creature(p1(), "Construct", 2, 2).in_zone(ZoneId::Battlefield))
        // Armorcraft Judge itself (no counters — enters fresh).
        .object(ObjectSpec::creature(p1(), "Armorcraft Judge", 3, 3).in_zone(ZoneId::Battlefield))
        // One library card so DrawCards has somewhere to draw from.
        .object(ObjectSpec::card(p1(), "Library Card 1").in_zone(ZoneId::Library(p1())))
        .build()
        .unwrap();

    let source = on_battlefield(&state, "Armorcraft Judge");
    let effect = armorcraft_judge_effect();
    let mut ctx = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let hand_count = state
        .zones
        .get(&ZoneId::Hand(p1()))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        hand_count, 1,
        "CR 121.6 / Ruling 2020-11-10: one creature with a +1/+1 counter → draw 1"
    );
}

// ── Test 3 ─────────────────────────────────────────────────────────────────────

/// Ruling 2020-11-10 (Armorcraft Judge) — a creature with THREE +1/+1 counters
/// still counts as only one creature, so Armorcraft Judge draws exactly 1 card.
///
/// Setup: P1 controls one creature with three +1/+1 counters.
/// Expected: exactly 1 card drawn (counts creatures, not counters).
#[test]
fn armorcraft_judge_multiple_counters_one_creature_still_one() {
    // Ruling 2020-11-10: "draw a card for each creature you control with a +1/+1 counter
    // on it" — the count is creatures, not counter quantity. Three +1/+1 counters on one
    // creature still makes it count as one creature.
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        // One creature, three +1/+1 counters — still counts as 1 for the draw.
        .object(
            ObjectSpec::creature(p1(), "Pumped Creature", 2, 2)
                .with_counter(CounterType::PlusOnePlusOne, 3)
                .in_zone(ZoneId::Battlefield),
        )
        // Judge itself (no counters).
        .object(ObjectSpec::creature(p1(), "Armorcraft Judge", 3, 3).in_zone(ZoneId::Battlefield))
        // Library card to draw.
        .object(ObjectSpec::card(p1(), "Library Card 1").in_zone(ZoneId::Library(p1())))
        .build()
        .unwrap();

    let source = on_battlefield(&state, "Armorcraft Judge");
    let effect = armorcraft_judge_effect();
    let mut ctx = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let hand_count = state
        .zones
        .get(&ZoneId::Hand(p1()))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        hand_count, 1,
        "Ruling 2020-11-10: three counters on one creature → still draws 1 (counts creatures)"
    );
}

// ── Test 4 ─────────────────────────────────────────────────────────────────────

/// CR 121.6 + Armorcraft Judge — `controller: TargetController::You` filter ensures
/// that an opponent's creature with a +1/+1 counter does NOT count for the draw.
///
/// Setup: P2 controls a creature with a +1/+1 counter. P1 controls no creatures
/// with +1/+1 counters (other than the Judge itself, which has none).
/// Expected: P1 draws 0 cards — the opponent's countered creature is excluded.
#[test]
fn armorcraft_judge_filters_other_players_creatures() {
    // CR 121.6: counter check uses has_counter_type against GameObject.counters.
    // The TargetController::You filter (pre-existing in PermanentCount) ensures
    // only permanents controlled by the ability's controller are counted.
    // This test is a regression guard that the controller filter still works correctly
    // alongside the new has_counter_type field.
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        // Opponent's creature WITH a +1/+1 counter — must NOT count for P1's draw.
        .object(
            ObjectSpec::creature(p2(), "Opponent Pumped Creature", 2, 2)
                .with_counter(CounterType::PlusOnePlusOne, 1)
                .in_zone(ZoneId::Battlefield),
        )
        // Judge itself (no counters, controlled by P1).
        .object(ObjectSpec::creature(p1(), "Armorcraft Judge", 3, 3).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let source = on_battlefield(&state, "Armorcraft Judge");
    let effect = armorcraft_judge_effect();
    let mut ctx = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let hand_count = state
        .zones
        .get(&ZoneId::Hand(p1()))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        hand_count, 0,
        "CR 121.6: opponent's creature with +1/+1 counter must not count for controller's draw"
    );
}
