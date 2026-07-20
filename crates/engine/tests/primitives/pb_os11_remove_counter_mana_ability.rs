//! Tests for PB-OS11 Part A: `Cost::RemoveCounter` mana-ability lowering (OOS-LKI-3
//! reframed, CR 605.1a / CR 602.2c / CR 118.3).
//!
//! Workhorse prints "This creature enters with four +1/+1 counters on it. / Remove a
//! +1/+1 counter from this creature: Add {C}." The second ability is a mana ability
//! (CR 605.1a: no target, could add mana, not loyalty) with NO `{T}` component — only a
//! self-referential remove-counter cost. `mana_ability_lowering` now accepts
//! `Cost::RemoveCounter` (mirroring the PB-EF8 `exile_self_from_hand` no-tap
//! relaxation) and `handle_tap_for_mana` pays it, reusing the existing
//! `GameEvent::CounterRemoved`.
//!
//! Pattern follows `pb_ef8_exile_self_from_hand.rs` / `primitive_pb_ewc.rs`: build via
//! `all_cards()` + `enrich_spec_from_def`, activate/cast through real `Command`s, and
//! assert observable game state (mana pool, counters, zones) — never just that a
//! `ManaAbility` struct exists.
//!
//! Also backfills `gemstone_array` / `druids_repository`: their "Remove a charge
//! counter: Add one mana of any color" abilities newly lower into mana abilities too
//! (execution-verified per SR-34/36 "probe by execution" guardrail).

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, calculate_characteristics, card_name_to_id, check_and_apply_sbas,
    enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition, CardRegistry,
    Command, Cost, CounterType, Effect, GameEvent, GameState, GameStateBuilder, GameStateError,
    ManaColor, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ─────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn build_defs_and_registry() -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);
    (defs, registry)
}

fn enrich(
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

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn find_on_battlefield(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

fn pool_amount(state: &GameState, player: PlayerId, color: ManaColor) -> u32 {
    let pool = &state.player(player).expect("player exists").mana_pool;
    match color {
        ManaColor::White => pool.white,
        ManaColor::Blue => pool.blue,
        ManaColor::Black => pool.black,
        ManaColor::Red => pool.red,
        ManaColor::Green => pool.green,
        ManaColor::Colorless => pool.colorless,
    }
}

/// Cast a creature spell from hand, paying its cost with the supplied mana.
fn cast_creature(
    mut state: GameState,
    caster: PlayerId,
    card: ObjectId,
    mana: &[(ManaColor, u32)],
) -> (GameState, Vec<GameEvent>) {
    {
        let pool = &mut state.players_mut().get_mut(&caster).unwrap().mana_pool;
        for &(color, n) in mana {
            if n > 0 {
                pool.add(color, n);
            }
        }
    }
    state.turn_mut().priority_holder = Some(caster);
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: caster,
            card,
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
        })),
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {e:?}"))
}

/// Cast Workhorse (paying {6} generic) and resolve the stack. Returns the resulting
/// state and Workhorse's battlefield ObjectId.
fn cast_and_resolve_workhorse(p1: PlayerId, p2: PlayerId) -> (GameState, ObjectId) {
    let (defs, registry) = build_defs_and_registry();
    let spec = enrich(p1, "Workhorse", ZoneId::Hand(p1), &defs);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let in_hand = find_by_name(&state, "Workhorse");
    let (state, _) = cast_creature(state, p1, in_hand, &[(ManaColor::Colorless, 6)]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let id =
        find_on_battlefield(&state, "Workhorse").expect("Workhorse must resolve to battlefield");
    (state, id)
}

fn counters_on(state: &GameState, id: ObjectId, counter: CounterType) -> u32 {
    state
        .objects()
        .get(&id)
        .and_then(|o| o.counters.get(&counter).copied())
        .unwrap_or(0)
}

// ── T1 — CR 614.1c: enters with four +1/+1 counters ───────────────────────────────

/// CR 614.1c: Workhorse's self-replacement puts 4 +1/+1 counters on it as it enters,
/// giving it layer-resolved P/T 4/4 (base 0/0 + Layer 7d).
#[test]
fn test_workhorse_enters_with_four_counters() {
    let (state, id) = cast_and_resolve_workhorse(p(1), p(2));

    assert_eq!(
        counters_on(&state, id, CounterType::PlusOnePlusOne),
        4,
        "CR 614.1c: Workhorse enters with four +1/+1 counters"
    );
    let chars = calculate_characteristics(&state, id).expect("Workhorse must be on battlefield");
    assert_eq!(
        chars.power,
        Some(4),
        "0/0 base + four +1/+1 counters = 4 power"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "0/0 base + four +1/+1 counters = 4 toughness"
    );
}

// ── T2 — CR 605.1a: the lowered no-tap remove-counter mana ability ────────────────

/// CR 605.1a/605.3b: activating Workhorse's mana ability removes one +1/+1 counter
/// and adds {C} through the LOWERED mana-ability path (`TapForMana`), not the stack.
#[test]
fn test_workhorse_remove_counter_adds_colorless() {
    let (state, id) = cast_and_resolve_workhorse(p(1), p(2));

    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: id,
            ability_index: 0,
            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    )
    .expect("Workhorse's remove-counter mana ability should activate");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Colorless),
        1,
        "Remove a +1/+1 counter: Add {{C}}"
    );
    assert_eq!(
        counters_on(&state, id, CounterType::PlusOnePlusOne),
        3,
        "one +1/+1 counter must be removed as the cost"
    );
    assert!(
        !state.objects()[&id].status.tapped,
        "the ability has no {{T}} component — Workhorse must NOT become tapped"
    );
    assert!(
        state.stack_objects().is_empty(),
        "CR 605.3b: a mana ability must not use the stack"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CounterRemoved { object_id, counter, count }
                if *object_id == id && *counter == CounterType::PlusOnePlusOne && *count == 1
        )),
        "a CounterRemoved event must be emitted for the paid cost"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                color: ManaColor::Colorless,
                amount: 1,
                ..
            }
        )),
        "a ManaAdded{{Colorless, 1}} event must be emitted"
    );
}

// ── T3 — the ability registers as a mana ability, not an activated ability ───────

/// CR 605.1a: Workhorse's remove-counter ability must be lowered into
/// `mana_abilities`, never registered in `activated_abilities` (SF-6-style exclusion
/// — the two lists must never disagree).
#[test]
fn test_workhorse_lowered_as_mana_ability_not_activated() {
    let (defs, _registry) = build_defs_and_registry();
    let spec = enrich(p(1), "Workhorse", ZoneId::Battlefield, &defs);

    assert_eq!(
        spec.mana_abilities.len(),
        1,
        "Workhorse's remove-counter ability must lower into exactly one ManaAbility"
    );
    assert!(
        spec.activated_abilities.is_empty(),
        "the same ability must not ALSO appear in activated_abilities"
    );
    let ma = &spec.mana_abilities[0];
    assert_eq!(
        ma.remove_counter,
        Some((CounterType::PlusOnePlusOne, 1)),
        "ManaAbility.remove_counter must carry (PlusOnePlusOne, 1)"
    );
    assert!(
        !ma.requires_tap,
        "Workhorse's mana ability has no {{T}} component"
    );
    assert!(
        !ma.any_color,
        "Workhorse produces a FIXED {{C}}, not any-color — it must not touch the \
         AddManaAnyColor color bug that keeps gemstone_array/druids_repository \
         known_wrong"
    );
}

// ── T4 — removing the last counter: mana still produced, then SBA death ─────────

/// CR 118.3 / CR 704.5f: removing Workhorse's last +1/+1 counter still produces the
/// mana (the cost is paid and the ability resolves), but Workhorse is now 0/0 and
/// dies to state-based actions on the next SBA check.
#[test]
fn test_workhorse_last_counter_removal_then_dies() {
    let (mut state, id) = cast_and_resolve_workhorse(p(1), p(2));

    // Remove three of the four counters first.
    for _ in 0..3 {
        let (s, _) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: id,
                ability_index: 0,
                chosen_color: None,
                        hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
},
        )
        .expect("removing a counter while >=1 remain must succeed");
        state = s;
    }
    assert_eq!(counters_on(&state, id, CounterType::PlusOnePlusOne), 1);

    // Remove the fourth (last) counter.
    let (mut state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: id,
            ability_index: 0,
            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    )
    .expect("removing the last counter must still succeed and produce mana");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Colorless),
        4,
        "all four activations must have produced {{C}} each"
    );
    assert_eq!(
        counters_on(&state, id, CounterType::PlusOnePlusOne),
        0,
        "the last +1/+1 counter must be removed"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CounterRemoved { object_id, .. } if *object_id == id
        )),
        "the final removal must still emit CounterRemoved"
    );

    // Workhorse is now a 0/0 creature: CR 704.5f moves it to the graveyard as an SBA.
    let sba_events = check_and_apply_sbas(&mut state);
    assert!(
        state.objects().get(&id).is_none(),
        "CR 400.7: the pre-death ObjectId is dead after the SBA move"
    );
    let in_graveyard = state.objects().iter().any(|(_, obj)| {
        obj.characteristics.name == "Workhorse" && matches!(obj.zone, ZoneId::Graveyard(_))
    });
    assert!(
        in_graveyard,
        "a 0/0 Workhorse must be moved to the graveyard by CR 704.5f"
    );
    assert!(
        sba_events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { object_id, .. } if *object_id == id)),
        "a CreatureDied event must be emitted for the 0-toughness SBA"
    );
}

// ── T5 — insufficient counters is rejected (CR 118.3) ────────────────────────────

/// CR 118.3: activating the ability with zero +1/+1 counters present must be
/// rejected before any mutation — no mana produced, no counter removed.
#[test]
fn test_workhorse_insufficient_counters_rejected() {
    let (mut state, id) = cast_and_resolve_workhorse(p(1), p(2));
    // Exhaust all four counters first.
    for _ in 0..4 {
        let (s, _) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: id,
                ability_index: 0,
                chosen_color: None,
                        hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
},
        )
        .expect("first four activations must succeed");
        state = s;
    }
    assert_eq!(counters_on(&state, id, CounterType::PlusOnePlusOne), 0);
    let probe_pool_before = pool_amount(&state, p(1), ManaColor::Colorless);

    let result = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: id,
            ability_index: 0,
            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    );

    assert!(
        matches!(result, Err(GameStateError::InvalidCommand(ref msg)) if msg.contains("118.3")),
        "activating with zero counters present must be rejected (CR 118.3): {result:?}"
    );
    let _ = probe_pool_before;
}

// ── T6 — self-referential accounting: another permanent's counters are untouched ──

/// The remove-counter cost is self-referential (source = the `TapForMana` source
/// ObjectId) — activating one Workhorse must not touch a second Workhorse's counters.
#[test]
fn test_remove_counter_mana_ability_reads_source_counters_only() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_defs_and_registry();
    let spec_a = enrich(p1, "Workhorse", ZoneId::Hand(p1), &defs);
    let spec_b = ObjectSpec::card(p1, "Workhorse")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(card_name_to_id("Workhorse"));
    let spec_b = enrich_spec_from_def(spec_b, &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec_a)
        .object(spec_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Cast both Workhorses.
    let hand_ids: Vec<ObjectId> = state
        .objects()
        .iter()
        .filter(|(_, o)| o.characteristics.name == "Workhorse")
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(hand_ids.len(), 2, "two Workhorse cards must be in hand");

    let (state, _) = cast_creature(state, p1, hand_ids[0], &[(ManaColor::Colorless, 6)]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = cast_creature(state, p1, hand_ids[1], &[(ManaColor::Colorless, 6)]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let battlefield_ids: Vec<ObjectId> = state
        .objects()
        .iter()
        .filter(|(_, o)| o.characteristics.name == "Workhorse" && o.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(battlefield_ids.len(), 2, "both Workhorses must resolve");
    let (source_a, source_b) = (battlefield_ids[0], battlefield_ids[1]);

    let (state, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: source_a,
            ability_index: 0,
            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    )
    .expect("activating Workhorse A's mana ability must succeed");

    assert_eq!(
        counters_on(&state, source_a, CounterType::PlusOnePlusOne),
        3,
        "Workhorse A must lose one counter"
    );
    assert_eq!(
        counters_on(&state, source_b, CounterType::PlusOnePlusOne),
        4,
        "Workhorse B's counters must be untouched — the cost is self-referential"
    );
}

// ── T7 — the lowering gate is scoped to RemoveCounter, not a blanket relaxation ──

/// (a) A synthetic def with a bare `Cost::RemoveCounter` + `Effect::AddMana{C}` lowers
/// to a `ManaAbility` (`remove_counter == Some(..)`, `requires_tap == false`) and is
/// excluded from `activated_abilities`. (b) A synthetic def with
/// `Cost::Sequence([DiscardCard, RemoveCounter])` does NOT lower — `DiscardCard` needs
/// a caller-supplied card, so the whole sequence declines. This proves the no-tap-guard
/// relaxation added for `remove_counter` is scoped to that flag alone, exactly like the
/// PB-EF8 `exile_self_from_hand` scoping.
#[test]
fn pb_os11_remove_counter_lowering_gate_is_not_vacuous() {
    let (defs, _registry) = build_defs_and_registry();

    let remove_counter_only_def = CardDefinition {
        card_id: mtg_engine::CardId("pb-os11-vacuity-remove-counter-only".to_string()),
        name: "PB-OS11 Vacuity Probe Remove Counter Only".to_string(),
        types: mtg_engine::cards::card_definition::TypeLine {
            card_types: vec![mtg_engine::CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::RemoveCounter {
                counter: CounterType::PlusOnePlusOne,
                count: 1,
            },
            effect: Effect::AddMana {
                player: mtg_engine::cards::card_definition::PlayerTarget::Controller,
                mana: mtg_engine::ManaPool {
                    colorless: 1,
                    ..Default::default()
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    };
    let mut defs_pos = defs.clone();
    defs_pos.insert(
        remove_counter_only_def.name.clone(),
        remove_counter_only_def.clone(),
    );
    let spec_pos = enrich_spec_from_def(
        ObjectSpec::card(p(1), &remove_counter_only_def.name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(remove_counter_only_def.card_id.clone()),
        &defs_pos,
    );
    assert_eq!(
        spec_pos.mana_abilities.len(),
        1,
        "a bare Cost::RemoveCounter ability must lower into a ManaAbility (positive control)"
    );
    assert!(
        spec_pos.activated_abilities.is_empty(),
        "the same ability must not ALSO appear in activated_abilities"
    );
    assert!(
        !spec_pos.mana_abilities[0].requires_tap,
        "a remove-counter cost has no {{T}} component"
    );

    // Negative: Cost::Sequence([DiscardCard, RemoveCounter]) must NOT lower —
    // DiscardCard needs a caller-supplied card, which TapForMana has no payload for.
    let discard_remove_counter_def = CardDefinition {
        card_id: mtg_engine::CardId("pb-os11-vacuity-discard-remove-counter".to_string()),
        name: "PB-OS11 Vacuity Probe Discard Remove Counter".to_string(),
        types: mtg_engine::cards::card_definition::TypeLine {
            card_types: vec![mtg_engine::CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![
                Cost::DiscardCard,
                Cost::RemoveCounter {
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
            ]),
            effect: Effect::AddMana {
                player: mtg_engine::cards::card_definition::PlayerTarget::Controller,
                mana: mtg_engine::ManaPool {
                    colorless: 1,
                    ..Default::default()
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    };
    let mut defs_neg = defs;
    defs_neg.insert(
        discard_remove_counter_def.name.clone(),
        discard_remove_counter_def.clone(),
    );
    let spec_neg = enrich_spec_from_def(
        ObjectSpec::card(p(1), &discard_remove_counter_def.name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(discard_remove_counter_def.card_id.clone()),
        &defs_neg,
    );
    assert_eq!(
        spec_neg.mana_abilities.len(),
        0,
        "Cost::Sequence([DiscardCard, RemoveCounter]) must NOT lower into a ManaAbility \
         (negative control — DiscardCard needs a caller-supplied card)"
    );
    assert_eq!(
        spec_neg.activated_abilities.len(),
        1,
        "it must instead register as a stack-using activated ability"
    );
}

// ── T8 / T9 — backfill: Gemstone Array / Druids' Repository any-color lowering ───

/// PB-OS11 backfill (execution-verify per SR-34/36): Gemstone Array's "Remove a charge
/// counter: Add one mana of any color" now lowers into a mana ability and produces the
/// CHOSEN color (not the documented colorless bug), closing its `known_wrong` note.
#[test]
fn test_gemstone_array_any_color_lowered() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_defs_and_registry();
    let spec = enrich(p1, "Gemstone Array", ZoneId::Battlefield, &defs)
        .with_counter(CounterType::Charge, 5);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    let id = find_by_name(&state, "Gemstone Array");

    // Sanity: the def-level lowering is as expected before activating.
    let spec_check = enrich(p1, "Gemstone Array", ZoneId::Battlefield, &defs);
    assert_eq!(spec_check.mana_abilities.len(), 1);
    assert_eq!(spec_check.activated_abilities.len(), 1);

    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: id,
            ability_index: 0,
            chosen_color: Some(ManaColor::Green),
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    )
    .expect("Gemstone Array's remove-counter mana ability should activate");

    assert_eq!(
        pool_amount(&state, p1, ManaColor::Green),
        1,
        "the lowered any-color mana ability must produce the CHOSEN color, not colorless"
    );
    assert_eq!(
        pool_amount(&state, p1, ManaColor::Colorless),
        0,
        "no colorless mana should be produced — that was the known_wrong bug"
    );
    assert_eq!(
        counters_on(&state, id, CounterType::Charge),
        4,
        "one charge counter must be removed as the cost"
    );
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::CounterRemoved { object_id, counter, .. }
            if *object_id == id && *counter == CounterType::Charge
    )));
}

/// PB-OS11 backfill (execution-verify): Druids' Repository's "Remove a charge counter:
/// Add one mana of any color" lowers the same way.
#[test]
fn test_druids_repository_any_color_lowered() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_defs_and_registry();
    let spec = enrich(p1, "Druids' Repository", ZoneId::Battlefield, &defs)
        .with_counter(CounterType::Charge, 5);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    let id = find_by_name(&state, "Druids' Repository");

    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: id,
            ability_index: 0,
            chosen_color: Some(ManaColor::Blue),
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    )
    .expect("Druids' Repository's remove-counter mana ability should activate");

    assert_eq!(
        pool_amount(&state, p1, ManaColor::Blue),
        1,
        "the lowered any-color mana ability must produce the CHOSEN color, not colorless"
    );
    assert_eq!(
        pool_amount(&state, p1, ManaColor::Colorless),
        0,
        "no colorless mana should be produced — that was the known_wrong bug"
    );
    assert_eq!(
        counters_on(&state, id, CounterType::Charge),
        4,
        "one charge counter must be removed as the cost"
    );
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::CounterRemoved { object_id, counter, .. }
            if *object_id == id && *counter == CounterType::Charge
    )));
}
