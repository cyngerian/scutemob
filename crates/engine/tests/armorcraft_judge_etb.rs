//! Armorcraft Judge ETB draw tests — PB-CC-B.
//!
//! Verifies that `TargetFilter.has_counter_type` correctly restricts the
//! `EffectAmount::PermanentCount` callsite when Armorcraft Judge's ETB trigger
//! counts creatures you control with a +1/+1 counter on them.
//!
//! CR Rules covered:
//! - CR 122.1: Counters modify the objects they are on.
//! - CR 122.6: Counters are tracked in `GameObject.counters` (NOT in Characteristics).
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

/// CR 122.1 / Ruling 2020-11-10 (Armorcraft Judge) — Armorcraft Judge ETB draws
/// zero cards when no creatures you control have +1/+1 counters.
///
/// Setup: P1 controls three creatures (none with counters) + Armorcraft Judge.
/// Library: 4 cards available so a broken filter that counted all 4 creatures
/// would yield hand_count=4, distinguishable from the correct hand_count=0.
/// Expected: zero cards drawn (hand stays at 0).
#[test]
fn armorcraft_judge_no_counters_zero_draw() {
    // CR 122.1: counters modify objects; without counters, has_counter_type predicate fails.
    // Ruling 2020-11-10: only creatures WITH a +1/+1 counter count.
    // Library discriminator: 4 cards present so broken-filter (counts all 4 creatures) would
    // produce hand_count=4, but correct behavior yields hand_count=0.
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Grizzly Bears", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1(), "Hill Giant", 3, 3).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1(), "Llanowar Elves", 1, 1).in_zone(ZoneId::Battlefield))
        // Judge itself (no counters)
        .object(ObjectSpec::creature(p1(), "Armorcraft Judge", 3, 3).in_zone(ZoneId::Battlefield))
        // 4 library cards — enough to distinguish broken filter (draws 4) from correct (draws 0)
        .object(ObjectSpec::card(p1(), "Library Card 1").in_zone(ZoneId::Library(p1())))
        .object(ObjectSpec::card(p1(), "Library Card 2").in_zone(ZoneId::Library(p1())))
        .object(ObjectSpec::card(p1(), "Library Card 3").in_zone(ZoneId::Library(p1())))
        .object(ObjectSpec::card(p1(), "Library Card 4").in_zone(ZoneId::Library(p1())))
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
        "must be 0 not 4 (broken filter would count all 4 creatures)"
    );
}

// ── Test 2 ─────────────────────────────────────────────────────────────────────

/// CR 122.1 / Ruling 2020-11-10 (Armorcraft Judge) — Armorcraft Judge ETB draws
/// one card when exactly one creature you control has a +1/+1 counter.
///
/// Setup: P1 controls Servo (with counter) + Construct (no counter) + Armorcraft
/// Judge (no counter). Library: 4 cards so that a broken filter counting all 3
/// P1 creatures (incl. Judge) would yield hand_count=3, not 1.
/// Expected: exactly 1 card drawn.
#[test]
fn armorcraft_judge_one_creature_with_counter_draws_one() {
    // CR 122.6: counters live on GameObject, not Characteristics.
    // Ruling 2020-11-10: Armorcraft Judge itself enters without counters and should
    // not count itself even though it is a creature you control.
    // Library discriminator: 4 cards so broken filter (counts all 3 P1 creatures) gives
    // hand_count=3, distinguishable from correct hand_count=1.
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
        // 4 library cards so broken filter (n=3) would give hand_count=3 not 1.
        .object(ObjectSpec::card(p1(), "Library Card 1").in_zone(ZoneId::Library(p1())))
        .object(ObjectSpec::card(p1(), "Library Card 2").in_zone(ZoneId::Library(p1())))
        .object(ObjectSpec::card(p1(), "Library Card 3").in_zone(ZoneId::Library(p1())))
        .object(ObjectSpec::card(p1(), "Library Card 4").in_zone(ZoneId::Library(p1())))
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
        "must be 1 not 3 (broken filter would count all 3 P1 creatures)"
    );
}

// ── Test 3 ─────────────────────────────────────────────────────────────────────

/// Ruling 2020-11-10 (Armorcraft Judge) — a creature with THREE +1/+1 counters
/// still counts as only one creature, so Armorcraft Judge draws exactly 1 card.
///
/// Setup: P1 controls one creature with three +1/+1 counters + Armorcraft Judge.
/// Library: 4 cards so that a broken "sum counters" filter would yield hand_count=3,
/// distinguishable from the correct hand_count=1.
/// Expected: exactly 1 card drawn (counts creatures, not counters).
#[test]
fn armorcraft_judge_multiple_counters_one_creature_still_one() {
    // Ruling 2020-11-10: "draw a card for each creature you control with a +1/+1 counter
    // on it" — the count is creatures, not counter quantity. Three +1/+1 counters on one
    // creature still makes it count as one creature.
    // Library discriminator: 4 cards so broken "sum counters" filter (n=3) gives
    // hand_count=3, while correct behavior gives hand_count=1.
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
        // 4 library cards so broken sum-counters filter (n=3) gives hand_count=3 not 1.
        .object(ObjectSpec::card(p1(), "Library Card 1").in_zone(ZoneId::Library(p1())))
        .object(ObjectSpec::card(p1(), "Library Card 2").in_zone(ZoneId::Library(p1())))
        .object(ObjectSpec::card(p1(), "Library Card 3").in_zone(ZoneId::Library(p1())))
        .object(ObjectSpec::card(p1(), "Library Card 4").in_zone(ZoneId::Library(p1())))
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
        "Ruling 2020-11-10: counts CREATURES not counters; broken sum-counters would give 3"
    );
}

// ── Test 4 ─────────────────────────────────────────────────────────────────────

/// CR 122.6 + Armorcraft Judge — `controller: TargetController::You` filter ensures
/// that an opponent's creature with a +1/+1 counter does NOT count for the draw.
///
/// Setup: P2 controls a creature with a +1/+1 counter. P1 controls no other
/// counter-bearing creatures (Judge itself has none). Library: 2 cards so a broken
/// filter ignoring controller would yield hand_count=1, distinguishable from 0.
/// Expected: P1 draws 0 cards — the opponent's countered creature is excluded.
#[test]
fn armorcraft_judge_filters_other_players_creatures() {
    // CR 122.6: counter check uses has_counter_type against GameObject.counters.
    // The TargetController::You filter (pre-existing in PermanentCount) ensures
    // only permanents controlled by the ability's controller are counted.
    // Library discriminator: 2 cards so broken filter (ignores controller, n=1) gives
    // hand_count=1, distinguishable from correct hand_count=0.
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
        // 2 library cards so broken controller-blind filter (n=1) gives hand_count=1 not 0.
        .object(ObjectSpec::card(p1(), "Library Card 1").in_zone(ZoneId::Library(p1())))
        .object(ObjectSpec::card(p1(), "Library Card 2").in_zone(ZoneId::Library(p1())))
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
        hand_count,
        0,
        "controller filter rules out opponent's counter-bearing creature; broken filter would give 1"
    );
}
