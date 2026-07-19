//! PB-OS8: `Effect::LookAtTopThenPlace` (closes OOS-EF10-1) + `TargetFilter.min_cmc_amount`
//! rider (closes PB-OS6 deferred sub-primitive (d)).
//!
//! `Effect::LookAtTopThenPlace` is the put-AT-MOST-ONE sibling of the already-shipped
//! `Effect::RevealAndRoute` (which puts ALL matches): look at the top `count` cards of a
//! library, optionally pay an interposed `place_cost` (CR 118.12), place at most one card
//! matching `filter` (honoring runtime `max_cmc_amount`/`min_cmc_amount`, CR 202.3/608.2h)
//! to `destination`, and send the rest to `rest_to` (CR 401, ObjectId-ascending deterministic
//! placement, NOT `rand`). Modeled directly on `Effect::RevealAndRoute`'s executor.
//!
//! `TargetFilter.min_cmc_amount` is the runtime LOWER-bound mirror of the existing
//! `max_cmc_amount`; honored by both `Effect::SearchLibrary` and `Effect::LookAtTopThenPlace`.
//!
//! CR Rules covered: 120/121 (look/draw), 202.3/608.2h (runtime mana value), 118.12
//! (optional cost), 601.2 (choose at most one), 401 (library order), 400.7 (new object on
//! zone change), 603.3/603.4 (ETB triggers + intervening-if).
//!
//! Cards affected: `birthing_ritual` (inert -> Complete), `growing_rites_of_itlimoc`
//! (partial -> Complete).
//!
//! Patterns mirrored: `tests/primitives/pb_ef10_sacrifice_driven_amounts.rs` (runtime cmc
//! cap + sacrifice LKI + sentinels, direct-executor probing), the `RevealAndRoute` usage in
//! narset/bounty (library top-N setup), `tests/primitives/pb_os6_dfc_flip_conditions.rs`
//! (end-step-trigger card-integration harness + sentinel layout).

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, CardDefinition, CardId, CardRegistry,
    CardType, Command, Cost, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder,
    LibraryPosition, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step,
    TargetFilter, ZoneId, ZoneTarget, HASH_SCHEMA_VERSION, PROTOCOL_VERSION,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn in_zone(state: &GameState, name: &str, zone: ZoneId) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == zone)
}

fn in_hand(state: &GameState, name: &str, owner: PlayerId) -> bool {
    in_zone(state, name, ZoneId::Hand(owner))
}

fn in_library(state: &GameState, name: &str, owner: PlayerId) -> bool {
    in_zone(state, name, ZoneId::Library(owner))
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    in_zone(state, name, ZoneId::Graveyard(owner))
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    in_zone(state, name, ZoneId::Battlefield)
}

fn count_in_zone(state: &GameState, zone: ZoneId) -> usize {
    state.objects().values().filter(|o| o.zone == zone).count()
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

/// Pass priority repeatedly until `target` step is reached.
fn advance_to_step(mut state: GameState, target: Step) -> GameState {
    let mut guard = 0;
    loop {
        if state.turn().step == target {
            return state;
        }
        guard += 1;
        assert!(
            guard < 500,
            "advance_to_step exceeded safety guard (infinite loop?)"
        );
        let holder = state.turn().priority_holder.expect("no priority holder");
        let (new_state, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = new_state;
    }
}

/// Resolve everything currently on the stack by passing priority in turn order.
fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players).0;
    }
    state
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn real_card_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no real CardDefinition for '{}'", name));
    let base = ObjectSpec::card(owner, name)
        .in_zone(zone)
        .with_card_id(def.card_id.clone());
    enrich_spec_from_def(base, defs)
}

fn creature_with_mv(owner: PlayerId, name: &str, mv: u32, zone: ZoneId) -> ObjectSpec {
    ObjectSpec::creature(owner, name, mv as i32, mv as i32)
        .with_card_id(CardId(format!(
            "os8-{}",
            name.to_lowercase().replace(' ', "-")
        )))
        .with_mana_cost(ManaCost {
            generic: mv,
            ..Default::default()
        })
        .in_zone(zone)
}

fn land_card(owner: PlayerId, name: &str, zone: ZoneId) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .with_card_id(CardId(format!(
            "os8-{}",
            name.to_lowercase().replace(' ', "-")
        )))
        .with_types(vec![CardType::Land])
        .in_zone(zone)
}

// ═══════════════════════════════════════════════════════════════════════════
// Effect::LookAtTopThenPlace — direct-executor probes
// ═══════════════════════════════════════════════════════════════════════════

/// CR 120/601.2 -- basic: top 4 has exactly 1 creature among 3 lands; the creature is
/// placed into hand, the other 3 are bottomed (still in the library, ObjectId-ascending).
#[test]
fn test_look_place_creature_to_hand_growing_rites() {
    let p1 = p(1);
    let p2 = p(2);

    let creature = creature_with_mv(p1, "Only Creature", 2, ZoneId::Library(p1));
    let land_a = land_card(p1, "Land A", ZoneId::Library(p1));
    let land_b = land_card(p1, "Land B", ZoneId::Library(p1));
    let land_c = land_card(p1, "Land C", ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .object(land_a)
        .object(land_b)
        .object(land_c)
        .build()
        .unwrap();

    let effect = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(4),
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        place_cost: None,
        destination: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        in_hand(&state, "Only Creature", p1),
        "the single matching creature should be placed into hand"
    );
    assert!(
        in_library(&state, "Land A", p1)
            && in_library(&state, "Land B", p1)
            && in_library(&state, "Land C", p1),
        "the three non-matching lands should be bottomed (still in the library)"
    );
    assert_eq!(
        count_in_zone(&state, ZoneId::Library(p1)),
        3,
        "exactly 3 cards should remain in the library after the creature is placed"
    );
}

/// DECOY: top 4 has NO creature. Nothing is placed; all 4 are bottomed; hand unchanged.
#[test]
fn test_look_place_no_match_leaves_all_bottomed() {
    let p1 = p(1);
    let p2 = p(2);

    let land_a = land_card(p1, "No Match Land A", ZoneId::Library(p1));
    let land_b = land_card(p1, "No Match Land B", ZoneId::Library(p1));
    let land_c = land_card(p1, "No Match Land C", ZoneId::Library(p1));
    let land_d = land_card(p1, "No Match Land D", ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(land_a)
        .object(land_b)
        .object(land_c)
        .object(land_d)
        .build()
        .unwrap();

    let effect = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(4),
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        place_cost: None,
        destination: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        count_in_zone(&state, ZoneId::Hand(p1)),
        0,
        "DECOY: with no matching card, nothing should be placed into hand"
    );
    assert_eq!(
        count_in_zone(&state, ZoneId::Library(p1)),
        4,
        "all 4 looked-at cards should be bottomed (still in the library) when nothing matches"
    );
}

/// DECOY vs `RevealAndRoute`: top N has TWO matching creatures. Exactly ONE (the
/// lowest ObjectId, i.e. the first added) is placed; the other is bottomed. Fails if the
/// executor were accidentally modeled on `RevealAndRoute`'s put-ALL-matches semantics.
#[test]
fn test_look_place_at_most_one_even_when_two_match() {
    let p1 = p(1);
    let p2 = p(2);

    // Added first -> lower ObjectId -> the deterministic winner.
    let creature_a = creature_with_mv(p1, "Creature A", 2, ZoneId::Library(p1));
    let creature_b = creature_with_mv(p1, "Creature B", 3, ZoneId::Library(p1));
    let land_x = land_card(p1, "Land X", ZoneId::Library(p1));
    let land_y = land_card(p1, "Land Y", ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature_a)
        .object(creature_b)
        .object(land_x)
        .object(land_y)
        .build()
        .unwrap();

    let effect = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(4),
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        place_cost: None,
        destination: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        in_hand(&state, "Creature A", p1),
        "the deterministic (lowest ObjectId) matching creature should be placed"
    );
    assert!(
        in_library(&state, "Creature B", p1),
        "DECOY: the second matching creature must be bottomed, NOT also placed \
         (RevealAndRoute would put both -- LookAtTopThenPlace puts at most one)"
    );
    assert_eq!(
        count_in_zone(&state, ZoneId::Hand(p1)),
        1,
        "exactly one card should be in hand -- never two"
    );
}

/// CR 603.3: placing onto the battlefield fires `PermanentEnteredBattlefield` (ETB path).
#[test]
fn test_look_place_onto_battlefield_fires_etb() {
    let p1 = p(1);
    let p2 = p(2);

    let creature = creature_with_mv(p1, "Etb Creature", 2, ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    let effect = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        place_cost: None,
        destination: ZoneTarget::Battlefield { tapped: false },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        on_battlefield(&state, "Etb Creature"),
        "the matching creature should have entered the battlefield"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. })),
        "CR 603.3: entering the battlefield must emit PermanentEnteredBattlefield (ETB path)"
    );
}

/// Birthing Ritual core: `place_cost: Sacrifice(creature)` gates AND parameterizes the
/// placement. A MV-2 creature is sacrificed (cap = 1+2 = 3); the top has a MV-3 creature
/// (placed) and a MV-5 creature (over cap, bottomed).
#[test]
fn test_look_place_cost_sacrifice_gates_and_parameterizes() {
    let p1 = p(1);
    let p2 = p(2);

    let sac_fodder = creature_with_mv(p1, "Sac Fodder", 2, ZoneId::Battlefield);
    let good_target = creature_with_mv(p1, "Good Target", 3, ZoneId::Library(p1));
    let too_expensive = creature_with_mv(p1, "Too Expensive", 5, ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(sac_fodder)
        .object(good_target)
        .object(too_expensive)
        .build()
        .unwrap();

    let effect = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(7),
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            max_cmc_amount: Some(Box::new(EffectAmount::Sum(
                Box::new(EffectAmount::Fixed(1)),
                Box::new(EffectAmount::ManaValueOfSacrificedCreature),
            ))),
            ..Default::default()
        },
        place_cost: Some(Box::new(Cost::Sacrifice(TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        }))),
        destination: ZoneTarget::Battlefield { tapped: false },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        in_graveyard(&state, "Sac Fodder", p1),
        "CR 118.12: the sacrifice cost must have been paid"
    );
    assert!(
        on_battlefield(&state, "Good Target"),
        "MV-3 creature (cap = 1 + 2 = 3) should be placed onto the battlefield"
    );
    assert!(
        !on_battlefield(&state, "Too Expensive"),
        "MV-5 creature exceeds the runtime cap (3) and must NOT be placed"
    );
    assert!(
        in_library(&state, "Too Expensive", p1),
        "the over-cap creature should be bottomed, not left floating"
    );
    assert!(
        ctx.sacrifice_fired,
        "ctx.sacrifice_fired should be latched true after the sacrifice"
    );
}

/// DECOY: `place_cost: Sacrifice(creature)` but the controller has NO creature to
/// sacrifice. Placement is skipped entirely -- even though a matching card was in the
/// top N -- and ALL looked-at cards are bottomed.
#[test]
fn test_look_place_cost_declined_when_unpayable_skips_placement() {
    let p1 = p(1);
    let p2 = p(2);

    // No creature on the battlefield to sacrifice.
    let would_have_matched = creature_with_mv(p1, "Would Have Matched", 1, ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(would_have_matched)
        .build()
        .unwrap();

    let effect = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(7),
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        place_cost: Some(Box::new(Cost::Sacrifice(TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        }))),
        destination: ZoneTarget::Battlefield { tapped: false },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        !on_battlefield(&state, "Would Have Matched"),
        "DECOY: with no creature to sacrifice, placement must be skipped even though a \
         matching card was in the top N"
    );
    assert!(
        in_library(&state, "Would Have Matched", p1),
        "the matching-but-unplaced card should be bottomed alongside everything else"
    );
}

/// Review Finding 3 (MEDIUM, AC 5077) -- the core `LookAtTopThenPlace`-vs-`SearchLibrary`
/// distinction: candidates are scoped strictly to the top-`count` window, NOT the whole
/// library. Library has `count + 2` cards; a matching creature sits INSIDE the window
/// (index 0) and a second matching creature sits at index `count` -- one position PAST the
/// window (`take(count)` never reaches it). Assert the in-window match is placed, and the
/// out-of-window match is completely UNTOUCHED: still in the library under its ORIGINAL
/// ObjectId (not re-inserted as a new object via a bottoming move, CR 400.7), and no zone-move
/// event (`ObjectPutOnLibrary`/`ObjectReturnedToHand`) was ever emitted referencing it. This
/// would fail if the executor accidentally scanned the whole library (or `count + 1`/more)
/// instead of `object_ids().take(count)`.
#[test]
fn test_look_place_truncates_at_top_n_leaves_out_of_window_match_untouched() {
    let p1 = p(1);
    let p2 = p(2);

    // count = 3; library has count + 2 = 5 cards.
    let in_window = creature_with_mv(p1, "In Window Creature", 2, ZoneId::Library(p1)); // index 0
    let filler_a = land_card(p1, "Truncation Filler A", ZoneId::Library(p1)); // index 1
    let filler_b = land_card(p1, "Truncation Filler B", ZoneId::Library(p1)); // index 2
    let out_of_window = creature_with_mv(p1, "Out Of Window Creature", 2, ZoneId::Library(p1)); // index 3 == count
    let filler_c = land_card(p1, "Truncation Filler C", ZoneId::Library(p1)); // index 4

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(in_window)
        .object(filler_a)
        .object(filler_b)
        .object(out_of_window)
        .object(filler_c)
        .build()
        .unwrap();

    // Capture the out-of-window creature's ORIGINAL ObjectId before the effect runs -- the
    // load-bearing check is that this exact id is untouched afterward, not merely that a
    // same-named card exists somewhere in the library (a bottomed card would also satisfy
    // that weaker check, but under a NEW ObjectId per CR 400.7).
    let out_of_window_original_id = find_obj(&state, "Out Of Window Creature");

    let effect = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(3),
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        place_cost: None,
        destination: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        in_hand(&state, "In Window Creature", p1),
        "the in-window matching creature (index 0, inside the top-3 window) should be placed"
    );

    assert!(
        !in_hand(&state, "Out Of Window Creature", p1)
            && !on_battlefield(&state, "Out Of Window Creature"),
        "the out-of-window matching creature (index 3, past the top-3 window) must NOT be \
         placed -- it was never a candidate"
    );
    assert_eq!(
        state
            .objects()
            .get(&out_of_window_original_id)
            .map(|o| o.zone),
        Some(ZoneId::Library(p1)),
        "the out-of-window creature's ORIGINAL ObjectId must still be live in the library -- \
         a bottoming move would have retired it and created a new object (CR 400.7)"
    );
    assert!(
        !events.iter().any(|e| matches!(
            e,
            GameEvent::ObjectPutOnLibrary { object_id, .. }
                | GameEvent::ObjectReturnedToHand { object_id, .. }
            if *object_id == out_of_window_original_id
        )),
        "no zone-move event should ever reference the out-of-window creature's original id -- \
         it was structurally unreachable, not merely un-selected"
    );
}

/// Review Finding 4 (LOW) -- edge paths: `count` resolving to an empty top-N window (via an
/// empty library) place nothing, bottom nothing, and -- critically -- do NOT pay an
/// interposed `place_cost` (the `top_ids.is_empty()` guard `continue`s before the cost
/// block is ever reached; see the executor comment at the `is_empty()` check).
#[test]
fn test_look_place_empty_library_places_nothing_and_skips_cost() {
    let p1 = p(1);
    let p2 = p(2);

    // A creature on the battlefield that COULD be sacrificed if the cost were ever reached.
    let would_be_sacrificed = creature_with_mv(p1, "Never Sacrificed", 2, ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(would_be_sacrificed)
        .build()
        .unwrap();
    // No library cards at all -- top_ids is empty regardless of `count`.

    let effect = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(7),
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        place_cost: Some(Box::new(Cost::Sacrifice(TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        }))),
        destination: ZoneTarget::Battlefield { tapped: false },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        on_battlefield(&state, "Never Sacrificed"),
        "with an empty library, the interposed sacrifice cost must NOT be paid -- the \
         top_ids.is_empty() guard continues before the cost block is ever reached"
    );
    assert_eq!(
        count_in_zone(&state, ZoneId::Library(p1)),
        0,
        "nothing should be bottomed -- there was nothing to bottom"
    );
    assert!(
        events.is_empty(),
        "an empty library should produce no events at all (no placement, no bottoming, no \
         cost payment)"
    );
    assert!(
        !ctx.sacrifice_fired,
        "ctx.sacrifice_fired must remain false -- the cost was never reached, let alone paid"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TargetFilter.min_cmc_amount — runtime lower-bound cap
// ═══════════════════════════════════════════════════════════════════════════

/// CR 202.3/608.2h -- direct `SearchLibrary` with a runtime min floor. Library has a
/// MV-2 and a MV-4 creature; only the MV-4 is found (mirrors PB-EF10's
/// `test_search_max_cmc_amount_caps_by_runtime_value`, but for the floor).
#[test]
fn test_min_cmc_amount_caps_search_by_runtime_floor() {
    let p1 = p(1);
    let p2 = p(2);

    let mv2 = creature_with_mv(p1, "Below Floor", 2, ZoneId::Library(p1));
    let mv4 = creature_with_mv(p1, "Above Floor", 4, ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(mv2)
        .object(mv4)
        .build()
        .unwrap();

    let effect = Effect::SearchLibrary {
        player: PlayerTarget::Controller,
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            min_cmc_amount: Some(Box::new(EffectAmount::Fixed(3))),
            ..Default::default()
        },
        reveal: false,
        destination: ZoneTarget::Battlefield { tapped: false },
        shuffle_before_placing: false,
        also_search_graveyard: false,
    };
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        on_battlefield(&state, "Above Floor"),
        "MV-4 creature (floor = 3) should be found and placed"
    );
    assert!(
        !on_battlefield(&state, "Below Floor"),
        "MV-2 creature is below the runtime floor (3) and should NOT be found"
    );
}

/// `LookAtTopThenPlace` with `min_cmc_amount == max_cmc_amount == Fixed(3)` expresses
/// "mana value EQUAL TO 3" (the Birthing Pod shape). Top has MV-2/MV-3/MV-4 creatures;
/// only the MV-3 is placeable.
#[test]
fn test_look_place_min_and_max_equal_exact_mv() {
    let p1 = p(1);
    let p2 = p(2);

    let mv2 = creature_with_mv(p1, "Exact MV Two", 2, ZoneId::Library(p1));
    let mv3 = creature_with_mv(p1, "Exact MV Three", 3, ZoneId::Library(p1));
    let mv4 = creature_with_mv(p1, "Exact MV Four", 4, ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(mv2)
        .object(mv3)
        .object(mv4)
        .build()
        .unwrap();

    let effect = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(3),
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            min_cmc_amount: Some(Box::new(EffectAmount::Fixed(3))),
            max_cmc_amount: Some(Box::new(EffectAmount::Fixed(3))),
            ..Default::default()
        },
        place_cost: None,
        destination: ZoneTarget::Battlefield { tapped: false },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        on_battlefield(&state, "Exact MV Three"),
        "the MV-3 creature (min=max=3) should be the only placeable card"
    );
    assert!(
        !on_battlefield(&state, "Exact MV Two") && !on_battlefield(&state, "Exact MV Four"),
        "MV-2 and MV-4 both fall outside the exact-3 window and must not be placed"
    );
    assert!(
        in_library(&state, "Exact MV Two", p1) && in_library(&state, "Exact MV Four", p1),
        "the excluded creatures should be bottomed"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Card-integration
// ═══════════════════════════════════════════════════════════════════════════

/// CR 603.3/603.4/118.12 -- Birthing Ritual's end-step trigger: control a creature
/// (intervening-if), the trigger fires, sacrifices the controlled creature (MV 2, cap
/// = 1+2=3), and places the MV-3 candidate onto the battlefield while the MV-5 candidate
/// is bottomed.
#[test]
fn test_birthing_ritual_end_step_flip() {
    let p1 = p(1);
    let p2 = p(2);
    let players = [p1, p2];
    let defs = load_defs();

    let ritual = real_card_spec(p1, "Birthing Ritual", ZoneId::Battlefield, &defs);
    let controlled_beast = creature_with_mv(p1, "Controlled Beast", 2, ZoneId::Battlefield);
    let cheap_target = creature_with_mv(p1, "Cheap Target", 3, ZoneId::Library(p1));
    let pricey_target = creature_with_mv(p1, "Pricey Target", 5, ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(ritual)
        .object(controlled_beast)
        .object(cheap_target)
        .object(pricey_target)
        .build()
        .unwrap();

    let state = advance_to_step(state, Step::End);
    let state = resolve_stack(state, &players);

    assert!(
        in_graveyard(&state, "Controlled Beast", p1),
        "CR 118.12: the controlled creature should have been sacrificed"
    );
    assert!(
        on_battlefield(&state, "Cheap Target"),
        "MV-3 creature (cap = 1 + 2 = 3) should be placed onto the battlefield"
    );
    assert!(
        !on_battlefield(&state, "Pricey Target"),
        "MV-5 creature exceeds the runtime cap (3) and must not be placed"
    );
    assert!(
        in_library(&state, "Pricey Target", p1),
        "the over-cap creature should be bottomed"
    );
}

/// CR 603.3 -- Growing Rites of Itlimoc's real ETB: cast the enchantment, resolve, and
/// the ETB trigger looks at the top 4, puts the matching creature into hand, bottoms
/// the rest.
#[test]
fn test_growing_rites_etb_look_four() {
    let p1 = p(1);
    let p2 = p(2);
    let players = [p1, p2];
    let defs = load_defs();

    let growing_rites = real_card_spec(p1, "Growing Rites of Itlimoc", ZoneId::Hand(p1), &defs);
    let lib_creature = creature_with_mv(p1, "Rites Target Creature", 2, ZoneId::Library(p1));
    let lib_land_a = land_card(p1, "Rites Land A", ZoneId::Library(p1));
    let lib_land_b = land_card(p1, "Rites Land B", ZoneId::Library(p1));
    let lib_land_c = land_card(p1, "Rites Land C", ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(growing_rites)
        .object(lib_creature)
        .object(lib_land_a)
        .object(lib_land_b)
        .object(lib_land_c)
        .build()
        .unwrap();

    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool = ManaPool {
            colorless: 2,
            green: 1,
            ..Default::default()
        };
    }

    let rites_id = find_obj(&state, "Growing Rites of Itlimoc");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: rites_id,
            targets: vec![],
            modes_chosen: vec![],
            x_value: 0,
            kicker_times: 0,
            additional_costs: vec![],
            alt_cost: None,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        })),
    )
    .expect("Cast Growing Rites of Itlimoc should succeed");

    let state = resolve_stack(state, &players);

    assert!(
        on_battlefield(&state, "Growing Rites of Itlimoc"),
        "Growing Rites should have resolved onto the battlefield"
    );
    assert!(
        in_hand(&state, "Rites Target Creature", p1),
        "CR 603.3: the matching creature should be placed into hand"
    );
    assert!(
        in_library(&state, "Rites Land A", p1)
            && in_library(&state, "Rites Land B", p1)
            && in_library(&state, "Rites Land C", p1),
        "the non-matching lands should be bottomed"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Hash soundness
// ═══════════════════════════════════════════════════════════════════════════

/// `Effect::LookAtTopThenPlace` hashes distinctly from `Effect::RevealAndRoute` (same-
/// shaped fields) and from a variant of itself with min/max_cmc_amount swapped.
#[test]
fn test_lookattopthenplace_hashes_distinctly() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;
    let hash_effect = |effect: &Effect| -> [u8; 32] {
        let mut hasher = Hasher::new();
        effect.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    let base_filter = TargetFilter {
        has_card_type: Some(CardType::Creature),
        ..Default::default()
    };

    let ltp = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(4),
        filter: base_filter.clone(),
        place_cost: None,
        destination: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };

    let rar = Effect::RevealAndRoute {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(4),
        filter: base_filter.clone(),
        matched_dest: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        unmatched_dest: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
    };

    assert_ne!(
        hash_effect(&ltp),
        hash_effect(&rar),
        "LookAtTopThenPlace and RevealAndRoute must hash distinctly (different discriminants)"
    );

    let ltp_max = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(4),
        filter: TargetFilter {
            max_cmc_amount: Some(Box::new(EffectAmount::Fixed(3))),
            ..base_filter.clone()
        },
        place_cost: None,
        destination: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };
    let ltp_min = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(4),
        filter: TargetFilter {
            min_cmc_amount: Some(Box::new(EffectAmount::Fixed(3))),
            ..base_filter
        },
        place_cost: None,
        destination: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
        optional: true,
    };
    assert_ne!(
        hash_effect(&ltp_max),
        hash_effect(&ltp_min),
        "min_cmc_amount and max_cmc_amount set to the same value must hash distinctly"
    );
}

/// `TargetFilter.min_cmc_amount` hashes distinctly from `max_cmc_amount` (same value)
/// and from a different `min_cmc_amount` value.
#[test]
fn test_min_cmc_amount_hashes_distinctly() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    let hash_filter = |f: &TargetFilter| -> [u8; 32] {
        let mut hasher = Hasher::new();
        f.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    let f_min3 = TargetFilter {
        min_cmc_amount: Some(Box::new(EffectAmount::Fixed(3))),
        ..Default::default()
    };
    let f_max3 = TargetFilter {
        max_cmc_amount: Some(Box::new(EffectAmount::Fixed(3))),
        ..Default::default()
    };
    let f_min0 = TargetFilter {
        min_cmc_amount: Some(Box::new(EffectAmount::Fixed(0))),
        ..Default::default()
    };

    assert_ne!(
        hash_filter(&f_min3),
        hash_filter(&f_max3),
        "min_cmc_amount and max_cmc_amount (same Fixed(3) value) must hash distinctly"
    );
    assert_ne!(
        hash_filter(&f_min3),
        hash_filter(&f_min0),
        "min_cmc_amount(3) and min_cmc_amount(0) must hash distinctly"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Version sentinels
// ═══════════════════════════════════════════════════════════════════════════

/// PROTOCOL_VERSION and HASH_SCHEMA_VERSION are the machine-forced values for this
/// batch: `Effect` gained `LookAtTopThenPlace` and `TargetFilter` gained
/// `min_cmc_amount` (both already in the closure -- type COUNT unchanged, declared
/// shape moves). See crates/engine/src/rules/protocol.rs and
/// crates/engine/src/state/hash.rs for the authoritative bump.
#[test]
fn test_pb_os8_version_sentinels() {
    assert_eq!(
        PROTOCOL_VERSION, 25,
        "PROTOCOL_VERSION should be 23 after PB-OS8 (Effect::LookAtTopThenPlace + \
         TargetFilter.min_cmc_amount)"
    );
    assert_eq!(
        HASH_SCHEMA_VERSION, 62u8,
        "HASH_SCHEMA_VERSION should be 60 after PB-OS8"
    );
}
