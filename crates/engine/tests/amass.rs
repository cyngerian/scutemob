//! Amass keyword action tests (CR 701.47).
//!
//! Amass is a keyword action (like Investigate, Proliferate), not a keyword ability.
//! Cards say "amass Zombies N" or "amass Orcs N" as part of a spell or triggered ability.
//!
//! CR 701.47a: "If you don't control an Army creature, create a 0/0 black [subtype] Army
//! creature token. Choose an Army creature you control. Put N +1/+1 counters on that
//! creature. If it isn't a [subtype], it becomes a [subtype] in addition to its other types."
//!
//! CR 701.47b: "A player 'amassed' after the process described in rule 701.47a is complete,
//! even if some or all of those actions were impossible." — Amassed event is always emitted.
//!
//! CR 701.47d: Older cards without a subtype were errata'd to "amass Zombies N".
//!
//! Key rules verified:
//! - Amass creates a 0/0 black [subtype] Army token when no Army exists (CR 701.47a).
//! - Amass adds counters to an existing Army (CR 701.47a).
//! - Amass adds the [subtype] to an existing Army that lacks it (CR 701.47a).
//! - Amass N=0 still creates the token (CR 701.47b: process always completes).
//! - Multiple Armies: deterministic fallback picks smallest ObjectId.
//! - Multiplayer: amass only looks at armies controlled by the effect's controller.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    calculate_characteristics, CardRegistry, CardType, CounterType, Effect, EffectAmount,
    GameEvent, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step, SubType, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Execute Effect::Amass for a given controller/subtype/count directly.
///
/// Returns (state, events). Uses a placeholder source ObjectId.
fn run_amass(
    mut state: mtg_engine::GameState,
    controller: PlayerId,
    subtype: &str,
    count: i32,
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let effect = Effect::Amass {
        subtype: subtype.to_string(),
        count: EffectAmount::Fixed(count),
    };
    // Placeholder source id — Amass does not reference the source object.
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(controller, source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);
    (state, events)
}

/// Find an object by name in the game state (panics if not found).
fn find_by_name(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Get the +1/+1 counter count on an object.
fn get_plus_counters(state: &mtg_engine::GameState, id: ObjectId) -> u32 {
    state
        .objects
        .get(&id)
        .and_then(|obj| obj.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0)
}

/// Check whether an object has a given creature subtype (by string name).
fn has_subtype(state: &mtg_engine::GameState, id: ObjectId, subtype: &str) -> bool {
    state
        .objects
        .get(&id)
        .map(|obj| {
            obj.characteristics
                .subtypes
                .contains(&SubType(subtype.to_string()))
        })
        .unwrap_or(false)
}

/// Count Army creature tokens (is_token == true, has Creature type, has Army subtype).
fn count_army_tokens(state: &mtg_engine::GameState, controller: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_token
                && obj.controller == controller
                && obj.characteristics.card_types.contains(&CardType::Creature)
                && obj
                    .characteristics
                    .subtypes
                    .contains(&SubType("Army".to_string()))
        })
        .count()
}

// ── Test 1: Creates Army token when no Army exists ────────────────────────────

#[test]
/// CR 701.47a — Amass creates a 0/0 black Zombie Army token when controller has no Army.
/// Put N +1/+1 counters on it. Final state: N/N Zombie Army token.
fn test_amass_creates_army_token_when_none_exists() {
    let p1 = p(1);
    let p2 = p(2);

    // P1 has no creatures at all.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_amass(state, p1, "Zombie", 2);

    // Should emit: TokenCreated, PermanentEnteredBattlefield, CounterAdded, Amassed.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TokenCreated { player, .. } if *player == p1)),
        "CR 701.47a: TokenCreated event should be emitted for the new Army token"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::PermanentEnteredBattlefield { player, .. } if *player == p1)
        ),
        "CR 701.47a: PermanentEnteredBattlefield should be emitted for the Army token"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 2,
                ..
            }
        )),
        "CR 701.47a: CounterAdded event with count=2 should be emitted"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Amassed { player, count: 2, .. } if *player == p1)),
        "CR 701.47b: Amassed event should always be emitted"
    );

    // One Army token should now exist on the battlefield.
    assert_eq!(
        count_army_tokens(&state, p1),
        1,
        "CR 701.47a: exactly one Army token should be on the battlefield"
    );

    // Find the token and verify it has 2 +1/+1 counters (making it a 2/2).
    let token_id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.is_token
                && obj.controller == p1
                && obj
                    .characteristics
                    .subtypes
                    .contains(&SubType("Army".to_string()))
        })
        .map(|(id, _)| *id)
        .expect("Army token should exist");

    assert_eq!(
        get_plus_counters(&state, token_id),
        2,
        "CR 701.47a: Army token should have 2 +1/+1 counters"
    );

    // Layer-aware P/T should be 2/2 (0/0 base + 2/2 from counters).
    let chars =
        calculate_characteristics(&state, token_id).expect("Army token should be on battlefield");
    assert_eq!(
        chars.power,
        Some(2),
        "CR 701.47a: token should be 2/2 (power)"
    );
    assert_eq!(
        chars.toughness,
        Some(2),
        "CR 701.47a: token should be 2/2 (toughness)"
    );
}

// ── Test 2: Adds counters to an existing Army ─────────────────────────────────

#[test]
/// CR 701.47a — Amass adds N +1/+1 counters to an existing Army creature.
/// The Army is not a token, just a creature with the Army subtype.
fn test_amass_adds_counters_to_existing_army() {
    let p1 = p(1);
    let p2 = p(2);

    // P1 controls a Zombie Army creature already with 1 counter (making it a 1/1).
    let army = ObjectSpec::creature(p1, "Zombie Army", 0, 0)
        .with_subtypes(vec![
            SubType("Zombie".to_string()),
            SubType("Army".to_string()),
        ])
        .with_counter(CounterType::PlusOnePlusOne, 1)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(army)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let army_id = find_by_name(&state, "Zombie Army");

    let (state, events) = run_amass(state, p1, "Zombie", 3);

    // No new token should be created — P1 already has an Army.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::TokenCreated { .. })),
        "CR 701.47a: no token should be created when an Army already exists"
    );

    // Should emit CounterAdded with count=3.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 3,
                object_id,
            } if *object_id == army_id
        )),
        "CR 701.47a: CounterAdded should reference the existing Army with count=3"
    );

    // The Army should now have 4 total +1/+1 counters (1 existing + 3 new).
    assert_eq!(
        get_plus_counters(&state, army_id),
        4,
        "CR 701.47a: existing Army should have 4 total +1/+1 counters (1 + 3)"
    );

    // Layer-aware P/T should be 4/4 (0/0 base + 4/4 from counters).
    let chars =
        calculate_characteristics(&state, army_id).expect("Zombie Army should be on battlefield");
    assert_eq!(
        chars.power,
        Some(4),
        "CR 701.47a: Zombie Army should be 4/4 (power)"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "CR 701.47a: Zombie Army should be 4/4 (toughness)"
    );

    // Amassed event should still be emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Amassed { player, count: 3, .. } if *player == p1)),
        "CR 701.47b: Amassed event should be emitted"
    );
}

// ── Test 3: Adds subtype to existing Army that lacks it ───────────────────────

#[test]
/// CR 701.47a — "If it isn't a [subtype], it becomes a [subtype] in addition to its other types."
/// Zombie Army amasses Orcs 1: the Army should gain the Orc subtype.
fn test_amass_adds_subtype_to_existing_army() {
    let p1 = p(1);
    let p2 = p(2);

    // P1 controls a Zombie Army (already has Zombie and Army subtypes).
    let army = ObjectSpec::creature(p1, "Zombie Army", 0, 0)
        .with_subtypes(vec![
            SubType("Zombie".to_string()),
            SubType("Army".to_string()),
        ])
        .with_counter(CounterType::PlusOnePlusOne, 2)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(army)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let army_id = find_by_name(&state, "Zombie Army");

    // Amass Orcs 1 — the Army doesn't have Orc subtype yet.
    let (state, _events) = run_amass(state, p1, "Orc", 1);

    // The Army should now have all three subtypes: Zombie, Army, Orc.
    assert!(
        has_subtype(&state, army_id, "Army"),
        "CR 701.47a: Army subtype should still be present"
    );
    assert!(
        has_subtype(&state, army_id, "Zombie"),
        "CR 701.47a: existing Zombie subtype should be preserved"
    );
    assert!(
        has_subtype(&state, army_id, "Orc"),
        "CR 701.47a: Orc subtype should be added by amass Orcs"
    );

    // Should have 3 total counters (2 existing + 1 new).
    assert_eq!(
        get_plus_counters(&state, army_id),
        3,
        "CR 701.47a: Army should have 3 counters after amass Orcs 1"
    );
}

// ── Test 4: Amass 0 still creates token (CR 701.47b) ─────────────────────────

#[test]
/// CR 701.47b — Amass 0 with no Army still creates a 0/0 token and emits Amassed.
/// "A player 'amassed' after the process described in rule 701.47a is complete,
/// even if some or all of those actions were impossible."
///
/// Note: The 0/0 token will be killed by SBAs after resolution. But it was created.
fn test_amass_zero_still_creates_token() {
    let p1 = p(1);
    let p2 = p(2);

    // P1 has no creatures.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_amass(state, p1, "Zombie", 0);

    // Token should be created (0/0, will die to SBAs but was created).
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TokenCreated { player, .. } if *player == p1)),
        "CR 701.47b: TokenCreated should be emitted even for amass 0"
    );

    // No CounterAdded since N=0.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CounterAdded { .. })),
        "CR 701.47a: no counters should be placed for amass 0"
    );

    // Amassed event should still be emitted (CR 701.47b).
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Amassed { player, count: 0, .. } if *player == p1)),
        "CR 701.47b: Amassed event should be emitted even when N=0"
    );

    // One Army token exists (before SBAs would remove it).
    assert_eq!(
        count_army_tokens(&state, p1),
        1,
        "CR 701.47b: the 0/0 Army token was created before SBAs"
    );

    // The token has 0 +1/+1 counters and is a true 0/0.
    let token_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.is_token && obj.controller == p1)
        .map(|(id, _)| *id)
        .expect("Army token should exist before SBAs");

    assert_eq!(
        get_plus_counters(&state, token_id),
        0,
        "CR 701.47a: 0/0 Army token should have no +1/+1 counters"
    );
    let chars = calculate_characteristics(&state, token_id)
        .expect("token is still on battlefield (before SBAs)");
    assert_eq!(
        chars.power,
        Some(0),
        "CR 701.47b: token is a true 0/0 (power)"
    );
    assert_eq!(
        chars.toughness,
        Some(0),
        "CR 701.47b: token is a true 0/0 (toughness)"
    );
}

// ── Test 5: Multiple Armies — deterministic picks smallest ObjectId ────────────

#[test]
/// CR 701.47a ruling 2023-06-16 — When multiple Army creatures exist (e.g., via
/// Changeling), choose which to put the +1/+1 counters on.
/// Deterministic fallback: smallest ObjectId (consistent with Bolster pattern).
fn test_amass_multiple_armies_chooses_smallest_object_id() {
    let p1 = p(1);
    let p2 = p(2);

    // Two Army creatures — use explicit subtypes to mark them as Army.
    let army_a = ObjectSpec::creature(p1, "Army Alpha", 1, 1)
        .with_subtypes(vec![
            SubType("Zombie".to_string()),
            SubType("Army".to_string()),
        ])
        .in_zone(ZoneId::Battlefield);
    let army_b = ObjectSpec::creature(p1, "Army Beta", 2, 2)
        .with_subtypes(vec![
            SubType("Orc".to_string()),
            SubType("Army".to_string()),
        ])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(army_a)
        .object(army_b)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let id_a = find_by_name(&state, "Army Alpha");
    let id_b = find_by_name(&state, "Army Beta");
    let min_id = id_a.min(id_b);
    let max_id = id_a.max(id_b);

    let (state, events) = run_amass(state, p1, "Zombie", 2);

    // No token should be created — both are already Armies.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::TokenCreated { .. })),
        "CR 701.47a: no token created when an Army already exists"
    );

    // The Army with the smallest ObjectId should receive the counters.
    assert_eq!(
        get_plus_counters(&state, min_id),
        2,
        "CR 701.47a: Army with smallest ObjectId should receive 2 counters"
    );
    assert_eq!(
        get_plus_counters(&state, max_id),
        0,
        "CR 701.47a: Army with larger ObjectId should not receive counters"
    );

    // Amassed event references the chosen army (min_id).
    assert!(
        events.iter().any(|e| matches!(e, GameEvent::Amassed { player, army_id, count: 2 } if *player == p1 && *army_id == min_id)),
        "CR 701.47b: Amassed event should reference the chosen Army (smallest ObjectId)"
    );
}

// ── Test 6: Multiplayer — only considers controller's armies ──────────────────

#[test]
/// CR 701.47a (multiplayer) — Amass only looks at Army creatures controlled by
/// the effect's controller. Opponent's Army creatures are not eligible.
fn test_amass_multiplayer_only_own_armies() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // P2 controls a large Zombie Army — should NOT be chosen by P1's amass.
    let p2_army = ObjectSpec::creature(p2, "P2 Zombie Army", 5, 5)
        .with_subtypes(vec![
            SubType("Zombie".to_string()),
            SubType("Army".to_string()),
        ])
        .with_counter(CounterType::PlusOnePlusOne, 5)
        .in_zone(ZoneId::Battlefield);

    // P1 has no Army creatures — should create a fresh token.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(p2_army)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let p2_army_id = find_by_name(&state, "P2 Zombie Army");

    let (state, events) = run_amass(state, p1, "Zombie", 2);

    // P1 should have created a new token (P2's Army was ineligible).
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TokenCreated { player, .. } if *player == p1)),
        "CR 701.47a: P1 should create a new Army token (P2's Army is not P1's)"
    );

    // P2's Army should be untouched (no counters added to it).
    assert_eq!(
        get_plus_counters(&state, p2_army_id),
        5,
        "CR 701.47a: P2's Army should not receive additional counters from P1's amass"
    );

    // P1 now has an Army token.
    assert_eq!(
        count_army_tokens(&state, p1),
        1,
        "CR 701.47a: P1 should have exactly 1 Army token after amass"
    );

    // The Amassed event is for P1.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Amassed { player, count: 2, .. } if *player == p1)),
        "CR 701.47b: Amassed event should be emitted for P1"
    );
}

// ── Test 7: Subtype not re-added if already present ──────────────────────────

#[test]
/// CR 701.47a — "If it isn't a [subtype], it becomes a [subtype] in addition to its other types."
/// If the Army already has the target subtype, no duplicate is added.
fn test_amass_subtype_not_duplicated_if_already_present() {
    let p1 = p(1);
    let p2 = p(2);

    // P1 controls a Zombie Army (already has the Zombie subtype).
    let army = ObjectSpec::creature(p1, "Zombie Army", 0, 0)
        .with_subtypes(vec![
            SubType("Zombie".to_string()),
            SubType("Army".to_string()),
        ])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(army)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let army_id = find_by_name(&state, "Zombie Army");

    // Amass Zombies 1 — Army already IS a Zombie.
    let (state, _events) = run_amass(state, p1, "Zombie", 1);

    // Zombie subtype should still be present.
    assert!(
        has_subtype(&state, army_id, "Zombie"),
        "CR 701.47a: Zombie subtype should still be present (was already there)"
    );
    assert!(
        has_subtype(&state, army_id, "Army"),
        "CR 701.47a: Army subtype should still be present"
    );

    // Subtypes should still be a minimal set: Zombie + Army (no duplicate Zombie).
    let obj = state.objects.get(&army_id).unwrap();
    let zombie_count = obj
        .characteristics
        .subtypes
        .iter()
        .filter(|st| st.0 == "Zombie")
        .count();
    assert_eq!(
        zombie_count, 1,
        "CR 701.47a: Zombie subtype should appear exactly once (OrdSet deduplicates)"
    );
}
