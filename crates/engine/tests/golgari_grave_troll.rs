//! Golgari Grave-Troll (OOS-EWC-2) — self-ETB EntersWithCounters using
//! `EffectAmount::CardCount { zone: Graveyard(Controller), player: Controller,
//! filter: TargetFilter { has_card_type: Creature } }`.
//!
//! CR 614.1c — "This creature enters with a +1/+1 counter on it for each
//! creature card in your graveyard." The replacement is a CR 614.15 self-ETB
//! replacement (`is_self: true`), so it applies to the entering permanent
//! itself before any other replacement effect. The count is evaluated at the
//! moment ETB is processed: the resolver builds an `EffectContext` pinned to
//! the entering object and calls `resolve_amount`, which counts cards matching
//! the filter currently in the controller's graveyard.
//!
//! Tests:
//!   (a) Cast from hand with N creature cards in graveyard → enters with N
//!       +1/+1 counters and P/T = N/N (0 base + N counters via Layer 7d).
//!   (b) Filter discrimination — instants and lands in the graveyard alongside
//!       creatures are not counted; only the creature cards contribute.
//!   (c) Empty graveyard → enters with zero counters (no `(PlusOnePlusOne, 0)`
//!       entry in the counters OrdMap), and the 0/0 creature dies to SBA after
//!       resolution. CR 704.5f.
//!   (d) Dredge 6 — engine machinery (`Command::ChooseDredge`) picks up
//!       Dredge from the auto-discovered card definition's keywords; replacing
//!       the draw step's draw mills 6 cards and returns Golgari Grave-Troll
//!       from graveyard to hand. CR 702.52a.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::state::types::CounterType;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    all_cards, calculate_characteristics, card_name_to_id, enrich_spec_from_def, process_command,
    CardDefinition, CardRegistry, Command, GameEvent, GameState, GameStateBuilder, ManaColor,
    ObjectId, ObjectSpec, PlayerId, Step,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_on_battlefield(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
}

fn count_in_zone(state: &GameState, zone: ZoneId) -> usize {
    state.objects.values().filter(|o| o.zone == zone).count()
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
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

/// Cast a non-X creature spell from hand, paying its full mana cost.
fn cast_creature(
    mut state: GameState,
    caster: PlayerId,
    card: ObjectId,
    mana: &[(ManaColor, u32)],
) -> (GameState, Vec<GameEvent>) {
    {
        let pool = &mut state.players.get_mut(&caster).unwrap().mana_pool;
        for &(color, n) in mana {
            if n > 0 {
                pool.add(color, n);
            }
        }
    }
    state.turn.priority_holder = Some(caster);
    process_command(
        state,
        Command::CastSpell {
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e))
}

/// Seed `n` creature cards by name into a player's graveyard, enriched from
/// the registry so their characteristics have `CardType::Creature`.
fn seed_graveyard_creatures(
    builder: GameStateBuilder,
    owner: PlayerId,
    names: &[&str],
    defs: &HashMap<String, CardDefinition>,
) -> GameStateBuilder {
    let mut b = builder;
    for name in names {
        b = b.object(enrich(owner, name, ZoneId::Graveyard(owner), defs));
    }
    b
}

// ── Test (a): Dynamic count — N creature cards in graveyard ──────────────────

/// CR 614.1c — Golgari Grave-Troll cast from hand while the controller's
/// graveyard contains exactly 3 creature cards must enter with 3 +1/+1
/// counters, and its layer-resolved P/T must be 3/3.
#[test]
fn test_golgari_grave_troll_enters_with_n_counters_for_n_creatures() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_defs_and_registry();

    let troll_in_hand = enrich(p1, "Golgari Grave-Troll", ZoneId::Hand(p1), &defs);

    let builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(troll_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    // Three real creature cards in p1's graveyard.
    let builder = seed_graveyard_creatures(
        builder,
        p1,
        &["Elvish Mystic", "Llanowar Elves", "Birds of Paradise"],
        &defs,
    );

    let state = builder.build().unwrap();

    let troll_hand_id = find_by_name(&state, "Golgari Grave-Troll");

    // Cast Golgari Grave-Troll for {4}{G}.
    let (state, _) = cast_creature(
        state,
        p1,
        troll_hand_id,
        &[(ManaColor::Colorless, 4), (ManaColor::Green, 1)],
    );
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_troll = find_on_battlefield(&state, "Golgari Grave-Troll")
        .expect("Golgari Grave-Troll must be on the battlefield after casting");

    let counter_count = state
        .objects
        .get(&bf_troll)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "CR 614.1c: Golgari Grave-Troll must enter with one +1/+1 counter for \
         each creature card in its controller's graveyard. Expected 3, got {}.",
        counter_count
    );

    // P/T = 0 base + 3 +1/+1 counters (Layer 7d, CR 613.1g).
    let chars = calculate_characteristics(&state, bf_troll)
        .expect("calculate_characteristics must return Some for the on-battlefield Troll");
    assert_eq!(
        (chars.power, chars.toughness),
        (Some(3), Some(3)),
        "CR 613.1g: layer-resolved P/T must be 3/3 (0 base + 3 from +1/+1 counters). \
         Got ({:?}/{:?}).",
        chars.power,
        chars.toughness
    );
}

// ── Test (b): Filter discrimination — only creature cards count ──────────────

/// CR 614.1c + filter — Non-creature cards in the graveyard (instants,
/// sorceries, lands) MUST NOT contribute to the count. Only `CardType::Creature`
/// cards under the controller's graveyard pass the TargetFilter check.
#[test]
fn test_golgari_grave_troll_filter_excludes_non_creature_cards() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_defs_and_registry();

    let troll_in_hand = enrich(p1, "Golgari Grave-Troll", ZoneId::Hand(p1), &defs);

    let builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(troll_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        // Two creature cards.
        .object(enrich(p1, "Elvish Mystic", ZoneId::Graveyard(p1), &defs))
        .object(enrich(p1, "Llanowar Elves", ZoneId::Graveyard(p1), &defs))
        // Two non-creature cards (lands).
        .object(enrich(p1, "Forest", ZoneId::Graveyard(p1), &defs))
        .object(enrich(p1, "Swamp", ZoneId::Graveyard(p1), &defs));

    let state = builder.build().unwrap();

    let troll_hand_id = find_by_name(&state, "Golgari Grave-Troll");

    let (state, _) = cast_creature(
        state,
        p1,
        troll_hand_id,
        &[(ManaColor::Colorless, 4), (ManaColor::Green, 1)],
    );
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_troll = find_on_battlefield(&state, "Golgari Grave-Troll")
        .expect("Golgari Grave-Troll must be on the battlefield after casting");

    let counter_count = state
        .objects
        .get(&bf_troll)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "TargetFilter.has_card_type = Some(Creature) must exclude the two lands \
         (Forest + Swamp). Expected 2 creature cards, got {} counter(s).",
        counter_count
    );
}

// ── Test (c): Empty graveyard — 0/0 dies to SBA ──────────────────────────────

/// CR 614.1c edge: when the controller's graveyard contains zero creature
/// cards, `EffectAmount::CardCount` resolves to 0. The +1/+1 counter
/// modification's `> 0` guard means no counter entry is inserted (no zombie
/// `(PlusOnePlusOne, 0)` entry). The 0/0 creature then dies to SBA
/// (CR 704.5f).
#[test]
fn test_golgari_grave_troll_empty_graveyard_dies_to_sba() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_defs_and_registry();

    let troll_in_hand = enrich(p1, "Golgari Grave-Troll", ZoneId::Hand(p1), &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(troll_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let troll_hand_id = find_by_name(&state, "Golgari Grave-Troll");

    let (state, _) = cast_creature(
        state,
        p1,
        troll_hand_id,
        &[(ManaColor::Colorless, 4), (ManaColor::Green, 1)],
    );
    let (state, events) = pass_all(state, &[p1, p2]);

    // The Troll must NOT remain on the battlefield — 0/0 dies to SBA.
    assert!(
        find_on_battlefield(&state, "Golgari Grave-Troll").is_none(),
        "CR 704.5f: a 0/0 creature with no counters and no other P/T grants \
         must die to SBA. Found the Troll still on the battlefield."
    );

    // The Troll is now in its owner's graveyard.
    let troll_in_grave = state.objects.values().any(|o| {
        o.characteristics.name == "Golgari Grave-Troll" && o.zone == ZoneId::Graveyard(p1)
    });
    assert!(
        troll_in_grave,
        "Golgari Grave-Troll must end up in its owner's graveyard after dying to SBA."
    );

    // Suppression check: no CounterAdded { count: 0 } event was emitted by the
    // resolver. emit_etb_modification suppresses zero counts; a regression
    // here would mean we'd emit a no-op CounterAdded that the network layer
    // would broadcast.
    let zero_counter_event = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 0,
                ..
            }
        )
    });
    assert!(
        !zero_counter_event,
        "Resolver must suppress CounterAdded when CardCount resolves to 0 (no-op)."
    );
}

// ── Test (d): Dredge 6 returns the Troll to hand ─────────────────────────────

/// CR 702.52a — Dredge 6: while Golgari Grave-Troll is in its owner's
/// graveyard and the owner has at least 6 cards in library, replacing a draw
/// with dredge mills 6 cards and moves the Troll to hand. The keyword is
/// auto-detected from the card definition by `enrich_spec_from_def`
/// (replay_harness.rs), and the existing engine machinery
/// (Command::ChooseDredge / GameEvent::DredgeChoiceRequired) handles the
/// flow with zero new engine work.
#[test]
fn test_golgari_grave_troll_dredge_six_returns_to_hand() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_defs_and_registry();

    let troll_in_grave = enrich(p1, "Golgari Grave-Troll", ZoneId::Graveyard(p1), &defs);

    let builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(troll_in_grave)
        .active_player(p1)
        .at_step(Step::Upkeep);

    // 8 filler cards in p1's library: enough to dredge 6 (need >= 6) and have
    // some left over so the test does not bump into the empty-library edge.
    let mut builder = builder;
    for i in 0..8 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Filler {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }

    let mut state = builder.build().unwrap();
    // CR 103.8: mark as NOT first turn so the draw step actually draws.
    state.turn.is_first_turn_of_game = false;
    state.turn.priority_holder = Some(p1);

    // Pass through Upkeep — both players pass priority — engine advances to
    // Draw and fires the draw turn-based action, which checks dredge first.
    let (state, events) = pass_all(state, &[p1, p2]);

    // DredgeChoiceRequired must be present and offer Golgari Grave-Troll.
    // `options` is `Vec<(ObjectId, u32)>` (card id, dredge amount).
    let (dredge_card_id, dredge_amount) = events
        .iter()
        .find_map(|e| {
            if let GameEvent::DredgeChoiceRequired { player, options } = e {
                if *player == p1 {
                    options.first().copied()
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("DredgeChoiceRequired expected with Golgari Grave-Troll as an option");
    assert_eq!(
        dredge_amount, 6,
        "Dredge amount surfaced in the choice event must be 6 (the keyword's N). \
         Got {}.",
        dredge_amount
    );

    let lib_before = count_in_zone(&state, ZoneId::Library(p1));
    let grave_before = count_in_zone(&state, ZoneId::Graveyard(p1));
    let hand_before = count_in_zone(&state, ZoneId::Hand(p1));

    // Choose to dredge — mills 6 cards and moves the Troll to hand.
    let (state, dredge_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: Some(dredge_card_id),
        },
    )
    .unwrap_or_else(|e| panic!("Command::ChooseDredge failed: {:?}", e));

    // Exactly 6 CardMilled events.
    let mill_count = dredge_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardMilled { player, .. } if *player == p1))
        .count();
    assert_eq!(
        mill_count, 6,
        "CR 702.52a: Dredge 6 must mill exactly 6 cards; got {}.",
        mill_count
    );

    // Library lost 6.
    let lib_after = count_in_zone(&state, ZoneId::Library(p1));
    assert_eq!(
        lib_after,
        lib_before - 6,
        "Library should have 6 fewer cards after Dredge 6."
    );

    // Graveyard net: +6 milled, -1 (Troll moved to hand) = +5.
    let grave_after = count_in_zone(&state, ZoneId::Graveyard(p1));
    assert_eq!(
        grave_after,
        grave_before + 6 - 1,
        "Graveyard net change must be +5 (6 milled, Troll moves to hand)."
    );

    // Hand gained the Troll.
    let hand_after = count_in_zone(&state, ZoneId::Hand(p1));
    assert_eq!(
        hand_after,
        hand_before + 1,
        "Hand must have +1 card (the dredged Troll)."
    );
    assert!(
        state.objects.values().any(|o| {
            o.characteristics.name == "Golgari Grave-Troll" && o.zone == ZoneId::Hand(p1)
        }),
        "Golgari Grave-Troll must be in hand after dredging."
    );

    // CR 702.52a — dredge is NOT a draw; CardDrawn must not be emitted.
    assert!(
        !dredge_events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "CR 702.52a: dredge replaces the draw; CardDrawn must not fire."
    );
}
