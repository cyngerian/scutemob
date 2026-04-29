//! Tests for Mossborn Hydra (PB-CC-W: stale-TODO wire-up).
//!
//! Mossborn Hydra is {2}{G}, 0/0 Trample, "This creature enters with a +1/+1
//! counter on it. Landfall — Whenever a land you control enters, double the
//! number of +1/+1 counters on this creature."
//!
//! The Landfall doubling is expressed in the existing DSL by combining
//! `Effect::AddCounterAmount` with `EffectAmount::CounterCount` reading
//! `EffectTarget::Source`: read N counters from self, add N more → 2N total.
//! Ruling 2024-11-08: "put a number of +1/+1 counters on it equal to the
//! number it already has."
//!
//! These tests verify:
//! 1. Trigger registration: a land you control entering triggers Mossborn
//!    Hydra's Landfall ability (CR 207.2c + CR 603.2 + TargetController::You).
//! 2. Doubling math: `Effect::AddCounterAmount { count: CounterCount(Source) }`
//!    on a creature with N +1/+1 counters yields 2N counters total
//!    (CR 121.2 — counters added by an effect are added to the existing
//!    pool, not replacing them).
//! 3. Non-you-control negative: an opponent's land does NOT trigger Mossborn
//!    Hydra's Landfall (TargetController::You filter).
//!
//! CR references:
//!   - CR 121.2  — counters are placed on permanents and players; effects can
//!     add or remove them
//!   - CR 207.2c — "Landfall" is an ability word
//!   - CR 603.2  — triggered abilities check once per event
//!   - CR 603.6  — zone-change triggers (for "enters the battlefield")

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::abilities::check_triggers;
use mtg_engine::rules::events::GameEvent;
use mtg_engine::state::game_object::TriggerEvent;
use mtg_engine::{
    all_cards, enrich_spec_from_def, CardDefinition, CardEffectTarget, CardRegistry, CounterType,
    Effect, EffectAmount, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step,
    ZoneId,
};
use std::collections::HashMap;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn load_defs() -> HashMap<String, CardDefinition> {
    let cards = all_cards();
    cards.iter().map(|d| (d.name.clone(), d.clone())).collect()
}

fn triggers_for_land_entering(
    state: &GameState,
    controller: PlayerId,
    land_id: ObjectId,
) -> Vec<mtg_engine::state::stubs::PendingTrigger> {
    let events = vec![GameEvent::PermanentEnteredBattlefield {
        object_id: land_id,
        player: controller,
    }];
    check_triggers(state, &events)
}

// ── 1. Trigger registration: Mossborn Hydra fires on its controller's land ────

/// CR 603.2 + CR 207.2c — Mossborn Hydra's Landfall triggers when a land its
/// controller controls enters the battlefield.
#[test]
fn test_mossborn_hydra_landfall_triggers_on_own_land() {
    let p1 = p(1);
    let defs = load_defs();

    let hydra_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Mossborn Hydra").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let p1_land = ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(hydra_spec)
        .object(p1_land)
        .build()
        .unwrap();

    let hydra_id = find_object(&state, "Mossborn Hydra");
    let land_id = find_object(&state, "Forest");

    let triggers = triggers_for_land_entering(&state, p1, land_id);
    let hydra_triggers: Vec<_> = triggers.iter().filter(|t| t.source == hydra_id).collect();
    assert_eq!(
        hydra_triggers.len(),
        1,
        "CR 603.2 + CR 207.2c: Mossborn Hydra's Landfall must trigger exactly \
         once when a land its controller controls enters. Got {} triggers.",
        hydra_triggers.len()
    );
    assert_eq!(
        hydra_triggers[0].triggering_event,
        Some(TriggerEvent::AnyPermanentEntersBattlefield),
        "CR 603.2: Landfall dispatches through AnyPermanentEntersBattlefield"
    );
}

// ── 2. Non-you-control negative: opponent's land does NOT trigger Landfall ────

/// CR 603.2 + TargetController::You — Mossborn Hydra's Landfall must NOT
/// trigger when an opponent's land enters the battlefield.
#[test]
fn test_mossborn_hydra_landfall_does_not_trigger_on_opponent_land() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let hydra_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Mossborn Hydra").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let p2_land = ObjectSpec::land(p2, "Island").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(hydra_spec)
        .object(p2_land)
        .build()
        .unwrap();

    let hydra_id = find_object(&state, "Mossborn Hydra");
    let p2_land_id = find_object(&state, "Island");

    let triggers = triggers_for_land_entering(&state, p2, p2_land_id);
    let hydra_triggers: Vec<_> = triggers.iter().filter(|t| t.source == hydra_id).collect();
    assert!(
        hydra_triggers.is_empty(),
        "CR 603.2 + TargetController::You: Mossborn Hydra's Landfall must NOT \
         trigger when an opponent's land enters. Got {} triggers.",
        hydra_triggers.len()
    );
}

// ── 3. Doubling math: N → 2N via AddCounterAmount + CounterCount(Source) ─────

/// CR 121.2 — when an effect adds counters, those counters are added to any
/// existing counters on the permanent, not replacing them. Mossborn Hydra's
/// Landfall reads its current N +1/+1 counters via `EffectAmount::CounterCount`
/// and adds N more via `Effect::AddCounterAmount`, yielding 2N counters total.
///
/// This test executes the exact effect expression embedded in
/// `mossborn_hydra.rs` against a Hydra placeholder pre-loaded with N counters
/// for N ∈ {1, 2, 3, 5, 10}.
#[test]
fn test_mossborn_hydra_landfall_doubles_counters() {
    let cases: &[(u32, u32)] = &[(1, 2), (2, 4), (3, 6), (5, 10), (10, 20)];

    for &(initial, expected) in cases {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .object(ObjectSpec::creature(p(1), "Mossborn Hydra Stub", 0, 0))
            .build()
            .unwrap();

        let hydra = find_object(&state, "Mossborn Hydra Stub");

        // Pre-place N +1/+1 counters on the stub creature.
        {
            let obj = state.objects.get_mut(&hydra).unwrap();
            obj.counters.insert(CounterType::PlusOnePlusOne, initial);
        }

        // The exact effect expression from mossborn_hydra.rs Landfall trigger:
        let effect = Effect::AddCounterAmount {
            target: CardEffectTarget::Source,
            counter: CounterType::PlusOnePlusOne,
            count: EffectAmount::CounterCount {
                target: CardEffectTarget::Source,
                counter: CounterType::PlusOnePlusOne,
            },
        };

        let mut ctx = EffectContext::new(p(1), hydra, vec![]);
        execute_effect(&mut state, &effect, &mut ctx);

        let actual = state
            .objects
            .get(&hydra)
            .unwrap()
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0);

        assert_eq!(
            actual, expected,
            "CR 121.2 + ruling 2024-11-08: Mossborn Hydra Landfall must double \
             N={} +1/+1 counters to {}; got {}",
            initial, expected, actual
        );
    }
}

/// Edge case: 0 counters → 0 counters (Mossborn Hydra ETB places 1 before
/// Landfall ever fires, so N=0 should not occur in practice — but the engine
/// must not crash and must not place phantom counters when there are none to
/// double).
#[test]
fn test_mossborn_hydra_landfall_with_zero_counters_is_noop() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Mossborn Hydra Stub", 0, 0))
        .build()
        .unwrap();

    let hydra = find_object(&state, "Mossborn Hydra Stub");
    // No counters placed.

    let effect = Effect::AddCounterAmount {
        target: CardEffectTarget::Source,
        counter: CounterType::PlusOnePlusOne,
        count: EffectAmount::CounterCount {
            target: CardEffectTarget::Source,
            counter: CounterType::PlusOnePlusOne,
        },
    };

    let mut ctx = EffectContext::new(p(1), hydra, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let counters = state
        .objects
        .get(&hydra)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counters, 0,
        "0 counters doubled is still 0; effect must be a no-op when there is \
         nothing to double. Got {}",
        counters
    );
}

