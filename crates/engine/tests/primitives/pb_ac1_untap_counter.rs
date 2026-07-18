//! PB-AC1: Counter / untap / once-per-turn primitives.
//!
//! Five additions, all exercised here:
//! - `Effect::UntapAll { filter }` (CR 701.26b)
//! - `TriggerCondition::WheneverPermanentUntaps { filter }` (CR 502.3 / 603.2e)
//! - `TriggerCondition::WhenCounterPlaced { counter, filter, on_self }` (CR 122.6 / 122.7)
//! - Generic `once_per_turn` limiter on triggered abilities (CR 603.2c / 603.2h)
//! - `KeywordAbility::DoesNotUntap` pseudo-keyword static (CR 502.3)
//!
//! CR Rules covered: 701.26 (tap/untap keyword action), 502.3 (untap step "effects can
//! keep one or more of a player's permanents from untapping"), 502.4 (no priority during
//! untap step; triggers held to upkeep), 603.2c/603.2e/603.2h (triggered abilities),
//! 122.6/122.7 (counters put on).

use mtg_engine::effects::{execute_effect, matches_filter, EffectContext};
use mtg_engine::rules::abilities::{check_triggers, flush_pending_triggers};
use mtg_engine::rules::layers::{calculate_characteristics, expire_until_next_turn_effects};
use mtg_engine::rules::turn_actions::untap_active_player_permanents;
use mtg_engine::state::game_object::TriggerEvent;
use mtg_engine::state::targeting::{SpellTarget, Target};
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition,
    CardEffectTarget as EffectTarget, CardRegistry, CardType, Command, CounterType, Effect,
    EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec,
    PlayerId, PlayerTarget, Step, TargetController, TargetFilter, TriggerCondition,
    TriggeredAbilityDef, ZoneId, HASH_SCHEMA_VERSION,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(player))
        .count()
}

fn graveyard_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Graveyard(player))
        .count()
}

fn ctx_for(_state: &GameState, controller: PlayerId, source: ObjectId) -> EffectContext {
    EffectContext::new(controller, source, vec![])
}

fn ctx_with_target(controller: PlayerId, source: ObjectId, target: ObjectId) -> EffectContext {
    EffectContext::new(
        controller,
        source,
        vec![SpellTarget {
            target: Target::Object(target),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    )
}

// ── A: Hash schema sentinel ───────────────────────────────────────────────────

/// PB-AC1 A-1: HASH_SCHEMA_VERSION sentinel.
/// PB-AC1 bumped 27→28: Effect::UntapAll (disc 87), TriggerCondition::
/// WheneverPermanentUntaps (disc 42) / WhenCounterPlaced (disc 43), runtime
/// TriggerEvent::AnyPermanentUntaps (disc 45) / CounterPlaced (disc 46),
/// KeywordAbility::DoesNotUntap (disc 162), plus new fields (once_per_turn,
/// counter_filter, counter_on_self, triggered_abilities_fired_this_turn).
#[test]
fn test_pb_ac1_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 52u8,
        "PB-AC1 bumped HASH_SCHEMA_VERSION 27->28. If you bumped again, update this test."
    );
}

// ── B: Effect::UntapAll (CR 701.26b) ────────────────────────────────────────────

/// CR 701.26b: `Effect::UntapAll` untaps tapped creatures matching the filter and
/// scoped to the caster's control ("you"), leaving an opponent's tapped creature
/// (out of scope) and a non-matching card type (a tapped land, filtered out by
/// card type) untouched.
#[test]
fn test_untap_all_untaps_matching_tapped_permanents() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "My Bear 1", 2, 2)
                .tapped()
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p1, "My Bear 2", 2, 2)
                .tapped()
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::land(p1, "My Tapped Land")
                .tapped()
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p2, "Opponent Bear", 2, 2)
                .tapped()
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let bear1 = find_object(&state, "My Bear 1");
    let bear2 = find_object(&state, "My Bear 2");
    let land = find_object(&state, "My Tapped Land");
    let opp_bear = find_object(&state, "Opponent Bear");

    let mut state = state;
    let mut ctx = ctx_for(&state, p1, bear1);
    let filter = TargetFilter {
        has_card_type: Some(CardType::Creature),
        controller: TargetController::You,
        ..Default::default()
    };
    let events = execute_effect(&mut state, &Effect::UntapAll { filter }, &mut ctx);

    assert!(
        !state.objects().get(&bear1).unwrap().status.tapped,
        "CR 701.26b: My Bear 1 (creature, you control, tapped) must be untapped by UntapAll"
    );
    assert!(
        !state.objects().get(&bear2).unwrap().status.tapped,
        "CR 701.26b: My Bear 2 (creature, you control, tapped) must be untapped by UntapAll"
    );
    assert!(
        state.objects().get(&land).unwrap().status.tapped,
        "UntapAll{{card_types:[Creature]}} must not untap a land (filter scoping)"
    );
    assert!(
        state.objects().get(&opp_bear).unwrap().status.tapped,
        "UntapAll{{controller:You}} must not untap an opponent's creature"
    );
    let untap_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentUntapped { .. }))
        .count();
    assert_eq!(
        untap_events, 2,
        "exactly 2 PermanentUntapped events for the 2 matching, tapped, you-controlled creatures"
    );
    assert_eq!(
        ctx.last_effect_count, 2,
        "ctx.last_effect_count mirrors ExileAll/DestroyAll"
    );
}

/// CR 701.26b: "Only tapped permanents can be untapped." An already-untapped
/// matching permanent emits no `PermanentUntapped` event and is unaffected.
#[test]
fn test_untap_all_only_tapped() {
    let p1 = p(1);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Already Untapped Bear", 2, 2).in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let bear = find_object(&state, "Already Untapped Bear");
    assert!(!state.objects().get(&bear).unwrap().status.tapped);

    let mut state = state;
    let mut ctx = ctx_for(&state, p1, bear);
    let events = execute_effect(
        &mut state,
        &Effect::UntapAll {
            filter: TargetFilter::default(),
        },
        &mut ctx,
    );

    assert!(
        events
            .iter()
            .all(|e| !matches!(e, GameEvent::PermanentUntapped { .. })),
        "CR 701.26b: an already-untapped permanent must not emit a spurious PermanentUntapped event"
    );
    assert_eq!(ctx.last_effect_count, 0);
}

/// CR 502.3: `UntapAll { controller: You }` in a 4-player game untaps only the
/// caster's permanents, not any other player's.
#[test]
fn test_untap_all_multiplayer_controller_scope() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let state = GameStateBuilder::four_player()
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "P1 Bear", 2, 2)
                .tapped()
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p2, "P2 Bear", 2, 2)
                .tapped()
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p3, "P3 Bear", 2, 2)
                .tapped()
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p4, "P4 Bear", 2, 2)
                .tapped()
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let p1_bear = find_object(&state, "P1 Bear");
    let p2_bear = find_object(&state, "P2 Bear");
    let p3_bear = find_object(&state, "P3 Bear");
    let p4_bear = find_object(&state, "P4 Bear");

    let mut state = state;
    let mut ctx = ctx_for(&state, p1, p1_bear);
    let filter = TargetFilter {
        controller: TargetController::You,
        ..Default::default()
    };
    execute_effect(&mut state, &Effect::UntapAll { filter }, &mut ctx);

    assert!(!state.objects().get(&p1_bear).unwrap().status.tapped);
    assert!(state.objects().get(&p2_bear).unwrap().status.tapped);
    assert!(state.objects().get(&p3_bear).unwrap().status.tapped);
    assert!(state.objects().get(&p4_bear).unwrap().status.tapped);
}

// ── C: TriggerCondition::WheneverPermanentUntaps (CR 502.3 / 603.2e) ──────────

/// CR 502.3 / 603.2e: Mesmeric Orb card integration. A permanent untapped by an
/// effect makes ITS CONTROLLER mill 1 -- even when that controller is not
/// Mesmeric Orb's controller (proving PlayerTarget::ControllerOf(TriggeringCreature)
/// resolves against the untapped permanent, not the trigger source).
#[test]
fn test_wheneverpermanentuntaps_fires_on_effect_untap_mesmeric_orb() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let orb_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Mesmeric Orb").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let p2_creature = ObjectSpec::creature(p2, "P2 Tapped Creature", 2, 2)
        .tapped()
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(orb_spec)
        .object(p2_creature);
    for i in 0..3 {
        builder = builder.object(
            ObjectSpec::card(p2, &format!("P2 Library Card {i}")).in_zone(ZoneId::Library(p2)),
        );
    }
    let state = builder.build().unwrap();

    let creature_id = find_object(&state, "P2 Tapped Creature");
    let pre_gy = graveyard_count(&state, p2);

    let mut state = state;
    let mut ctx = ctx_for(&state, p2, creature_id);
    let events = execute_effect(
        &mut state,
        &Effect::UntapPermanent {
            target: EffectTarget::Source,
        },
        &mut ctx,
    );
    let triggers = check_triggers(&state, &events);
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }
    let flush_events = flush_pending_triggers(&mut state);
    assert!(
        flush_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 502.3/603.2e: Mesmeric Orb must trigger when a permanent becomes untapped"
    );
    // Resolve the stack (single trigger).
    while !state.stack_objects().is_empty() {
        let (s, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
        let (s, _) = process_command(s, Command::PassPriority { player: p2 }).unwrap();
        state = s;
    }
    assert_eq!(
        graveyard_count(&state, p2),
        pre_gy + 1,
        "the untapped creature's CONTROLLER (p2) must mill, not Mesmeric Orb's controller (p1)"
    );
}

/// CR 502.3 / 502.4: an untap-STEP batch untap (not an effect) also queues the
/// trigger; per CR 502.4, the trigger is held until the next priority window
/// (upkeep) rather than firing during the untap step itself.
#[test]
fn test_wheneverpermanentuntaps_fires_at_untap_step_held_to_upkeep() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let orb_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Mesmeric Orb").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::Untap)
        .object(orb_spec)
        .object(
            ObjectSpec::creature(p1, "P1 Tapped Attacker", 2, 2)
                .tapped()
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::card(p1, "P1 Library Card").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    // CR 502.2/502.3: run the untap-step turn-based action directly (mirrors
    // `enter_step`'s TBA dispatch for Step::Untap).
    let untap_events = untap_active_player_permanents(&mut state);
    assert!(
        untap_events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentsUntapped { .. })),
        "sanity: the tapped attacker must generate a PermanentsUntapped batch event"
    );
    let triggers = check_triggers(&state, &untap_events);
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }
    // CR 502.4: no player receives priority during the untap step. The trigger is
    // QUEUED but must NOT yet be on the stack.
    assert!(
        !state.pending_triggers().is_empty(),
        "CR 502.4: the WheneverPermanentUntaps trigger must be queued (held) during Untap"
    );
    assert!(
        state.stack_objects().is_empty(),
        "CR 502.4: the trigger must NOT be on the stack yet -- no priority during Untap"
    );

    // The next time a player would receive priority (usually Upkeep), the held
    // trigger is flushed to the stack.
    let flush_events = flush_pending_triggers(&mut state);
    assert!(flush_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })));
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 502.4: the untap-step WheneverPermanentUntaps trigger is held and appears \
         on the stack once priority is granted (simulated: the next flush_pending_triggers)"
    );
}

/// CR 603.2e: a permanent entering the battlefield already untapped does NOT fire
/// WheneverPermanentUntaps (no untap event occurs -- ETB is not "becomes untapped").
#[test]
fn test_wheneverpermanentuntaps_not_on_enters_untapped() {
    let p1 = p(1);
    let defs = load_defs();

    let orb_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Mesmeric Orb").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(CardRegistry::new(vec![]))
        .object(orb_spec)
        .build()
        .unwrap();

    // Simulate a permanent entering the battlefield (untapped by default) --
    // PermanentEnteredBattlefield only, no PermanentUntapped/PermanentsUntapped event.
    let orb_id = find_object(&state, "Mesmeric Orb");
    let events = vec![GameEvent::PermanentEnteredBattlefield {
        object_id: orb_id,
        player: p1,
    }];
    let triggers = check_triggers(&state, &events);
    assert!(
        triggers
            .iter()
            .all(|t| t.triggering_event != Some(TriggerEvent::AnyPermanentUntaps)),
        "CR 603.2e: entering the battlefield untapped must NOT fire WheneverPermanentUntaps"
    );
}

// ── D: TriggerCondition::WhenCounterPlaced (CR 122.6 / 122.7) ─────────────────

/// Synthetic card def with a `WhenCounterPlaced { counter: PlusOnePlusOne, on_self: true }`
/// trigger that draws a card, mirroring Fathom Mage / Dusk Legion Duelist's shape.
fn counter_trigger_creature_def(
    name: &'static str,
    on_self: bool,
    filter: Option<TargetFilter>,
) -> CardDefinition {
    use mtg_engine::TypeLine;
    CardDefinition {
        card_id: mtg_engine::state::player::CardId(format!(
            "test-{}",
            name.to_lowercase().replace(' ', "-")
        )),
        name: name.to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenCounterPlaced {
                counter: Some(CounterType::PlusOnePlusOne),
                filter,
                on_self,
            },
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

/// CR 122.6: a `WhenCounterPlaced { on_self: true }` trigger fires when a +1/+1
/// counter is placed on the trigger source itself, and does NOT fire for a counter
/// placed on a different creature.
#[test]
fn test_whencounterplaced_self_plus1plus1() {
    let p1 = p(1);
    let def = counter_trigger_creature_def("Test Fathom Mage", true, None);
    let defs: HashMap<String, CardDefinition> =
        std::iter::once((def.name.clone(), def.clone())).collect();
    let registry = CardRegistry::new(vec![def]);

    let fathom_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Test Fathom Mage").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(registry)
        .object(fathom_spec)
        .object(ObjectSpec::creature(p1, "Bystander Creature", 1, 1).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let fathom_id = find_object(&state, "Test Fathom Mage");
    let bystander_id = find_object(&state, "Bystander Creature");

    // Counter on a DIFFERENT creature must not fire an on_self trigger.
    let events_other = vec![GameEvent::CounterAdded {
        object_id: bystander_id,
        counter: CounterType::PlusOnePlusOne,
        count: 1,
    }];
    let triggers_other = check_triggers(&state, &events_other);
    assert!(
        triggers_other.iter().all(|t| t.source != fathom_id),
        "CR 122.6: on_self trigger must not fire for a counter placed on another creature"
    );

    // Counter on the source ITSELF must fire.
    let events_self = vec![GameEvent::CounterAdded {
        object_id: fathom_id,
        counter: CounterType::PlusOnePlusOne,
        count: 1,
    }];
    let triggers_self = check_triggers(&state, &events_self);
    let fathom_triggers: Vec<_> = triggers_self
        .iter()
        .filter(|t| t.source == fathom_id)
        .collect();
    assert_eq!(
        fathom_triggers.len(),
        1,
        "CR 122.6: on_self trigger must fire exactly once when a +1/+1 counter is put on the source"
    );
    assert_eq!(
        fathom_triggers[0].triggering_event,
        Some(TriggerEvent::CounterPlaced)
    );
}

/// CR 122.6: an `on_self: false` + `filter: {controller: You}` trigger (Simic
/// Ascendancy shape) fires when a +1/+1 counter lands on ANOTHER creature you
/// control, but not on an opponent's creature.
#[test]
fn test_whencounterplaced_filtered_you_control() {
    let p1 = p(1);
    let p2 = p(2);
    let filter = TargetFilter {
        controller: TargetController::You,
        ..Default::default()
    };
    let def = counter_trigger_creature_def("Test Ascendancy Watcher", false, Some(filter));
    let defs: HashMap<String, CardDefinition> =
        std::iter::once((def.name.clone(), def.clone())).collect();
    let registry = CardRegistry::new(vec![def]);

    let watcher_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Test Ascendancy Watcher").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(watcher_spec)
        .object(ObjectSpec::creature(p1, "My Other Creature", 1, 1).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p2, "Opponent Creature", 1, 1).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let watcher_id = find_object(&state, "Test Ascendancy Watcher");
    let my_other_id = find_object(&state, "My Other Creature");
    let opp_id = find_object(&state, "Opponent Creature");

    // Counter on MY other creature -> fires.
    let events_mine = vec![GameEvent::CounterAdded {
        object_id: my_other_id,
        counter: CounterType::PlusOnePlusOne,
        count: 1,
    }];
    let triggers_mine = check_triggers(&state, &events_mine);
    assert!(
        triggers_mine.iter().any(|t| t.source == watcher_id),
        "CR 122.6: filtered (you control) trigger must fire for a counter on another creature you control"
    );

    // Counter on an OPPONENT's creature -> does not fire.
    let events_opp = vec![GameEvent::CounterAdded {
        object_id: opp_id,
        counter: CounterType::PlusOnePlusOne,
        count: 1,
    }];
    let triggers_opp = check_triggers(&state, &events_opp);
    assert!(
        triggers_opp.iter().all(|t| t.source != watcher_id),
        "CR 122.6: filtered (you control) trigger must NOT fire for a counter on an opponent's creature"
    );
}

/// CR 122.7: a trigger filtered to `counter: Some(PlusOnePlusOne)` does NOT fire
/// when a different counter kind (e.g. -1/-1) is placed.
#[test]
fn test_whencounterplaced_wrong_counter_kind_no_fire() {
    let p1 = p(1);
    let def = counter_trigger_creature_def("Test Kind Filter Creature", true, None);
    let defs: HashMap<String, CardDefinition> =
        std::iter::once((def.name.clone(), def.clone())).collect();
    let registry = CardRegistry::new(vec![def]);

    let spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Test Kind Filter Creature").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(registry)
        .object(spec)
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Test Kind Filter Creature");

    // Positive control (per gotchas-rules.md #39: the wedge must be independently
    // observable): the RIGHT counter kind (+1/+1) DOES fire.
    let events_right = vec![GameEvent::CounterAdded {
        object_id: obj_id,
        counter: CounterType::PlusOnePlusOne,
        count: 1,
    }];
    assert!(
        check_triggers(&state, &events_right)
            .iter()
            .any(|t| t.source == obj_id),
        "positive control: +1/+1 counter placement must fire this trigger (counter_filter: Some(PlusOnePlusOne))"
    );

    let events = vec![GameEvent::CounterAdded {
        object_id: obj_id,
        counter: CounterType::MinusOneMinusOne,
        count: 1,
    }];
    let triggers = check_triggers(&state, &events);
    assert!(
        triggers.iter().all(|t| t.source != obj_id),
        "CR 122.7: a trigger filtered to +1/+1 counters must not fire for a -1/-1 counter placement"
    );
}

/// CR 122.6: Sharktocrab card integration -- Adapt puts a +1/+1 counter on itself,
/// which must fire the WhenCounterPlaced trigger (proving the CardDef -> runtime
/// conversion loop wired counter_filter/counter_on_self correctly).
#[test]
fn test_sharktocrab_counter_trigger_fires_via_carddef() {
    let p1 = p(1);
    let defs = load_defs();

    let crab_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Sharktocrab").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(CardRegistry::new(vec![]))
        .object(crab_spec)
        .build()
        .unwrap();

    let crab_id = find_object(&state, "Sharktocrab");
    let events = vec![GameEvent::CounterAdded {
        object_id: crab_id,
        counter: CounterType::PlusOnePlusOne,
        count: 1,
    }];
    let triggers = check_triggers(&state, &events);
    let crab_triggers: Vec<_> = triggers.iter().filter(|t| t.source == crab_id).collect();
    assert_eq!(
        crab_triggers.len(),
        1,
        "CR 122.6: Sharktocrab's WhenCounterPlaced (on_self:true) trigger must fire when it \
         receives a +1/+1 counter"
    );
}

// ── E: once_per_turn limiter (CR 603.2c / 603.2h) ──────────────────────────────

/// Kill `count` creatures (owned by `p1`, distinct from `exclude`) in a single
/// `Effect::DestroyAll` call, returning the resulting events.
/// Destroy all creatures controlled by an OPPONENT of `controller` (so the trigger
/// source itself, controlled by `controller`, survives) in a single batch.
fn kill_opponent_creatures(
    state: &mut GameState,
    controller: PlayerId,
    source: ObjectId,
) -> Vec<GameEvent> {
    let mut ctx = ctx_for(state, controller, source);
    execute_effect(
        state,
        &Effect::DestroyAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                controller: TargetController::Opponent,
                ..Default::default()
            },
            cant_be_regenerated: true,
        },
        &mut ctx,
    )
}

/// CR 603.2c / 603.2h: Morbid Opportunist card integration. Two SEPARATE creature
/// deaths in one turn draw exactly ONE card (once-per-turn limiter); a full turn
/// cycle resets the limiter so the next death draws again.
#[test]
fn test_once_per_turn_trigger_fires_once_across_turn() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let mo_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Morbid Opportunist").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(mo_spec)
        .object(ObjectSpec::creature(p2, "Fodder 1", 1, 1).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p2, "Fodder 2", 1, 1).in_zone(ZoneId::Battlefield));
    for i in 0..4 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("MO Library Card {i}")).in_zone(ZoneId::Library(p1)),
        );
    }
    let mut state = builder.build().unwrap();

    let mo_id = find_object(&state, "Morbid Opportunist");
    let fodder1 = find_object(&state, "Fodder 1");
    let fodder2 = find_object(&state, "Fodder 2");

    let pre_hand = hand_count(&state, p1);

    // First death: draw exactly 1.
    let mut ctx = ctx_with_target(p2, fodder1, fodder1);
    let events1 = execute_effect(
        &mut state,
        &Effect::DestroyPermanent {
            target: EffectTarget::DeclaredTarget { index: 0 },
            cant_be_regenerated: true,
        },
        &mut ctx,
    );
    let triggers1 = check_triggers(&state, &events1);
    for t in triggers1 {
        state.pending_triggers_mut().push_back(t);
    }
    flush_pending_triggers(&mut state);
    while !state.stack_objects().is_empty() {
        let (s, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
        let (s, _) = process_command(s, Command::PassPriority { player: p2 }).unwrap();
        state = s;
    }
    assert_eq!(
        hand_count(&state, p1),
        pre_hand + 1,
        "CR 603.10a: first creature death draws exactly 1 card"
    );

    // Second, SEPARATE death (same turn): must NOT draw again (once-per-turn gate).
    let mut ctx = ctx_with_target(p2, fodder2, fodder2);
    let events2 = execute_effect(
        &mut state,
        &Effect::DestroyPermanent {
            target: EffectTarget::DeclaredTarget { index: 0 },
            cant_be_regenerated: true,
        },
        &mut ctx,
    );
    let triggers2 = check_triggers(&state, &events2);
    for t in triggers2 {
        state.pending_triggers_mut().push_back(t);
    }
    flush_pending_triggers(&mut state);
    while !state.stack_objects().is_empty() {
        let (s, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
        let (s, _) = process_command(s, Command::PassPriority { player: p2 }).unwrap();
        state = s;
    }
    assert_eq!(
        hand_count(&state, p1),
        pre_hand + 1,
        "CR 603.2c/603.2h: a SECOND creature death in the same turn must NOT draw a second card \
         ('This ability triggers only once each turn')"
    );

    // Reset for next turn (mirrors the untap-step sweep).
    assert!(
        state
            .objects()
            .get(&mo_id)
            .unwrap()
            .triggered_abilities_fired_this_turn
            .contains(&0),
        "the fired-this-turn set must record ability index 0 as fired"
    );
    expire_until_next_turn_effects(&mut state, p1);
    assert!(
        state
            .objects()
            .get(&mo_id)
            .unwrap()
            .triggered_abilities_fired_this_turn
            .is_empty(),
        "CR 603.2h: the once-per-turn fired set resets at the next untap-step sweep"
    );
}

/// CR 603.2c: three SIMULTANEOUS creature deaths (one batch event set) produce
/// exactly ONE trigger instance (flush-time within-batch dedup), not three.
#[test]
fn test_once_per_turn_trigger_batched_deaths() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let mo_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Morbid Opportunist").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(mo_spec)
        .object(ObjectSpec::creature(p2, "Batch Fodder 1", 1, 1).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p2, "Batch Fodder 2", 1, 1).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p2, "Batch Fodder 3", 1, 1).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::card(p1, "MO Library Card").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    let source_id = find_object(&state, "Morbid Opportunist");
    let pre_hand = hand_count(&state, p1);

    let all_events = kill_opponent_creatures(&mut state, p1, source_id);
    let died_count = all_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(
        died_count, 3,
        "sanity: all 3 creatures died in this single batch"
    );

    let triggers = check_triggers(&state, &all_events);
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }
    let flushed = flush_pending_triggers(&mut state);
    let ability_triggered_count = flushed
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        ability_triggered_count, 1,
        "CR 603.2c/603.2h: 3 simultaneous creature deaths must put Morbid Opportunist's \
         ability on the stack exactly ONCE (once-per-turn dedup within the same batch)"
    );

    while !state.stack_objects().is_empty() {
        let (s, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
        let (s, _) = process_command(s, Command::PassPriority { player: p2 }).unwrap();
        state = s;
    }
    assert_eq!(hand_count(&state, p1), pre_hand + 1);
}

// ── F: KeywordAbility::DoesNotUntap (CR 502.3) ─────────────────────────────────

/// CR 502.3: a permanent with `KeywordAbility::DoesNotUntap`, tapped, remains
/// tapped after its controller's untap step runs.
#[test]
fn test_does_not_untap_static_keeps_permanent_tapped() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .active_player(p1)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::card(p1, "Test Mana Vault")
                .with_types(vec![CardType::Artifact])
                .tapped()
                .with_keyword(KeywordAbility::DoesNotUntap)
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let vault_id = find_object(&state, "Test Mana Vault");
    untap_active_player_permanents(&mut state);

    assert!(
        state.objects().get(&vault_id).unwrap().status.tapped,
        "CR 502.3: a DoesNotUntap permanent must remain tapped through the untap step"
    );
}

/// CR 502.3 + CR 613.1f (Humility-style ability removal): a creature with
/// `DoesNotUntap` under a `RemoveAllAbilities` layer-6 effect DOES untap, proving
/// the untap-step check is layer-resolved (calculate_characteristics), not
/// base-characteristics.
#[test]
fn test_does_not_untap_removed_by_humility() {
    use mtg_engine::{
        ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, LayerModification,
    };

    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .active_player(p1)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Sharpshooter-like Creature", 1, 1)
                .tapped()
                .with_keyword(KeywordAbility::DoesNotUntap)
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Sharpshooter-like Creature");

    // Sanity: without ability removal, the keyword is present and blocks untapping.
    let chars_before = calculate_characteristics(&state, creature_id).unwrap();
    assert!(
        chars_before
            .keywords
            .contains(&KeywordAbility::DoesNotUntap),
        "wedge property: DoesNotUntap must be present pre-Humility"
    );

    // Apply a Humility-style RemoveAllAbilities Layer 6 effect on this creature.
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9001),
        source: Some(creature_id),
        layer: EffectLayer::Ability,
        filter: EffectFilter::SingleObject(creature_id),
        modification: LayerModification::RemoveAllAbilities,
        duration: EffectDuration::WhileSourceOnBattlefield,
        timestamp: 1,
        is_cda: false,
        condition: None,
    });

    let chars_after = calculate_characteristics(&state, creature_id).unwrap();
    assert!(
        !chars_after.keywords.contains(&KeywordAbility::DoesNotUntap),
        "wedge property: DoesNotUntap must be REMOVED once RemoveAllAbilities applies"
    );

    untap_active_player_permanents(&mut state);
    assert!(
        !state.objects().get(&creature_id).unwrap().status.tapped,
        "CR 502.3 + 613.1f: with DoesNotUntap removed by Humility-style ability removal, \
         the creature DOES untap normally"
    );
}

/// CR 502.3: a permanent with BOTH `DoesNotUntap` AND a `skip_untap_steps` freeze
/// does not consume the freeze counter -- DoesNotUntap short-circuits before the
/// skip_untap_steps branch.
#[test]
fn test_does_not_untap_does_not_consume_skip_untap_counter() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .active_player(p1)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::card(p1, "Double-Frozen Permanent")
                .with_types(vec![CardType::Artifact])
                .tapped()
                .with_keyword(KeywordAbility::DoesNotUntap)
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Double-Frozen Permanent");
    if let Some(obj) = state.objects_mut().get_mut(&obj_id) {
        obj.skip_untap_steps = 1;
    }

    untap_active_player_permanents(&mut state);

    let obj = state.objects().get(&obj_id).unwrap();
    assert!(obj.status.tapped, "still tapped: DoesNotUntap wins");
    assert_eq!(
        obj.skip_untap_steps, 1,
        "CR 502.3: DoesNotUntap must short-circuit BEFORE the skip_untap_steps decrement -- \
         the freeze counter is untouched"
    );
}

/// CR 502.3: Goblin Sharpshooter card integration -- the keyword is present via
/// the CardDef -> layer-resolved conversion, and blocks untapping.
#[test]
fn test_goblin_sharpshooter_does_not_untap_via_carddef() {
    let p1 = p(1);
    let defs = load_defs();

    let gs_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Goblin Sharpshooter")
            .tapped()
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .active_player(p1)
        .with_registry(CardRegistry::new(vec![]))
        .object(gs_spec)
        .build()
        .unwrap();

    let gs_id = find_object(&state, "Goblin Sharpshooter");
    let chars = calculate_characteristics(&state, gs_id).unwrap();
    assert!(
        chars.keywords.contains(&KeywordAbility::DoesNotUntap),
        "Goblin Sharpshooter's CardDef must carry Keyword(DoesNotUntap)"
    );

    untap_active_player_permanents(&mut state);
    assert!(
        state.objects().get(&gs_id).unwrap().status.tapped,
        "CR 502.3: Goblin Sharpshooter doesn't untap during its controller's untap step"
    );
}

// ── G: CR 122.6 enters-with-counters edge (current-behavior assertion) ────────

/// CR 122.6: "counters being put on an object ... refers to ... also to an object
/// that's given counters as it enters the battlefield." Current engine behavior:
/// PermanentEnteredBattlefield with counters already set does NOT emit a
/// `GameEvent::CounterAdded`, so `WhenCounterPlaced` does not fire for
/// enters-with-counters. This test documents CURRENT (WRONG) behavior -- a
/// known fidelity gap tracked as MR-AC1-01 in docs/mtg-engine-milestone-reviews.md,
/// not fixed in this PB (see plan risk notes and PB-AC1 review Finding 3).
#[test]
fn test_whencounterplaced_enters_with_counters_current_behavior() {
    let p1 = p(1);
    let def = counter_trigger_creature_def("Test Enters With Counters", true, None);
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Test Enters With Counters")
                .with_card_id(card_id)
                .with_types(vec![CardType::Creature])
                .with_counter(CounterType::PlusOnePlusOne, 1)
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Test Enters With Counters");
    // Only a PermanentEnteredBattlefield event is emitted for entering with counters
    // (no synthetic CounterAdded) -- this mirrors real ETB-with-counters resolution.
    let events = vec![GameEvent::PermanentEnteredBattlefield {
        object_id: obj_id,
        player: p1,
    }];
    let triggers = check_triggers(&state, &events);
    assert!(
        triggers
            .iter()
            .all(|t| t.source != obj_id || t.triggering_event != Some(TriggerEvent::CounterPlaced)),
        "CURRENT BEHAVIOR (documented, not expanded in this PB): entering with counters \
         already present does not fire WhenCounterPlaced, because no CounterAdded event \
         is emitted for the ETB-with-counters path."
    );
}

// ── H: matches_filter sanity (used by the new dispatch blocks) ────────────────

/// Sanity check that the TargetFilter used in the WheneverPermanentUntaps /
/// WhenCounterPlaced dispatch blocks correctly matches creature card type.
#[test]
fn test_matches_filter_creature_type_sanity() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Filter Sanity Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();
    let bear = find_object(&state, "Filter Sanity Bear");
    let chars = calculate_characteristics(&state, bear).unwrap();
    let filter = TargetFilter {
        has_card_type: Some(CardType::Creature),
        ..Default::default()
    };
    assert!(matches_filter(&chars, &filter));
    let land_filter = TargetFilter {
        has_card_type: Some(CardType::Land),
        ..Default::default()
    };
    assert!(!matches_filter(&chars, &land_filter));
}

// ── I: exercise TriggeredAbilityDef with_triggered_ability directly ───────────

/// Direct-construction sanity test proving `ObjectSpec::with_triggered_ability`
/// correctly wires the new `counter_filter` / `counter_on_self` / `once_per_turn`
/// fields onto a runtime `TriggeredAbilityDef` (bypassing the CardDef conversion
/// loop entirely), and that `collect_triggers_for_event` reads them.
#[test]
fn test_triggered_ability_def_direct_construction_wiring() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Direct Wire Creature", 2, 2)
                .in_zone(ZoneId::Battlefield)
                .with_triggered_ability(TriggeredAbilityDef {
                    trigger_on: TriggerEvent::CounterPlaced,
                    intervening_if: None,
                    description: "test".to_string(),
                    effect: None,
                    etb_filter: None,
                    death_filter: None,
                    combat_damage_filter: None,
                    triggering_creature_filter: None,
                    targets: vec![],
                    once_per_turn: true,
                    counter_filter: Some(CounterType::Loyalty),
                    counter_on_self: true,
                }),
        )
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Direct Wire Creature");
    let chars = calculate_characteristics(&state, obj_id).unwrap();
    let def = chars
        .triggered_abilities
        .first()
        .expect("triggered ability present");
    assert!(def.once_per_turn);
    assert_eq!(def.counter_filter, Some(CounterType::Loyalty));
    assert!(def.counter_on_self);

    // Wrong counter kind -> no fire.
    let events_wrong = vec![GameEvent::CounterAdded {
        object_id: obj_id,
        counter: CounterType::PlusOnePlusOne,
        count: 1,
    }];
    assert!(check_triggers(&state, &events_wrong)
        .iter()
        .all(|t| t.source != obj_id));

    // Right counter kind, on self -> fires.
    let events_right = vec![GameEvent::CounterAdded {
        object_id: obj_id,
        counter: CounterType::Loyalty,
        count: 1,
    }];
    assert!(check_triggers(&state, &events_right)
        .iter()
        .any(|t| t.source == obj_id));
}
