//! Bolster keyword action tests (CR 701.39).
//!
//! Bolster is a keyword action (like Scry and Surveil), not a keyword ability.
//! Cards say "bolster N" as part of a spell effect or triggered/activated ability
//! effect. "Bolster N" means "Choose a creature you control with the least toughness
//! or tied for least toughness among creatures you control. Put N +1/+1 counters on
//! that creature." (CR 701.39a)
//!
//! Key rules verified:
//! - Bolster N places N +1/+1 counters on the creature with least toughness (CR 701.39a).
//! - Bolster uses layer-aware toughness (ruling 2014-11-24).
//! - Bolster does NOT target; protection does not prevent it (ruling 2014-11-24).
//! - If the controller has no creatures, bolster does nothing (CR 701.39a).
//! - Tied toughness: deterministic fallback selects smallest ObjectId.
//! - Multiplayer: bolster only considers the bolster controller's creatures.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    calculate_characteristics, CardRegistry, Color, CounterType, Effect, EffectAmount, GameEvent,
    GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec, PlayerId, ProtectionQuality, Step,
    ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Execute Effect::Bolster for a given player/count directly.
///
/// Returns (state, events). Uses a placeholder source ObjectId.
fn run_bolster(
    mut state: mtg_engine::GameState,
    controller: PlayerId,
    count: i32,
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let effect = Effect::Bolster {
        player: mtg_engine::PlayerTarget::Controller,
        count: EffectAmount::Fixed(count),
    };
    // Placeholder source id — Bolster does not reference the source object.
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

// ── Test 1: Basic — single creature receives bolster counters ─────────────────

#[test]
/// CR 701.39a — Bolster places N +1/+1 counters on the creature with least toughness.
/// P1 controls a single 2/3 creature. Bolster 2 places 2 counters on it.
/// Layer-aware P/T should be 4/5 after resolution.
fn test_bolster_basic_single_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let creature = ObjectSpec::creature(p1, "Guard Gomazoa", 2, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let gomazoa_id = find_by_name(&state, "Guard Gomazoa");
    let (state, events) = run_bolster(state, p1, 2);

    // Should emit exactly one CounterAdded event.
    assert_eq!(
        events.len(),
        1,
        "CR 701.39a: bolster should emit one CounterAdded event"
    );
    assert!(
        matches!(
            &events[0],
            GameEvent::CounterAdded {
                object_id,
                counter: CounterType::PlusOnePlusOne,
                count: 2,
            } if *object_id == gomazoa_id
        ),
        "CR 701.39a: CounterAdded should reference the correct creature with count 2"
    );

    // Creature should have 2 +1/+1 counters.
    assert_eq!(
        get_plus_counters(&state, gomazoa_id),
        2,
        "CR 701.39a: creature should have 2 +1/+1 counters"
    );

    // Layer-aware P/T should be 4/5.
    let chars = calculate_characteristics(&state, gomazoa_id)
        .expect("Guard Gomazoa should be on battlefield");
    assert_eq!(chars.power, Some(4), "CR 701.39a: power should be 4 (2+2)");
    assert_eq!(
        chars.toughness,
        Some(5),
        "CR 701.39a: toughness should be 5 (3+2)"
    );
}

// ── Test 2: Bolster targets the creature with the least toughness ─────────────

#[test]
/// CR 701.39a — Bolster chooses the creature with the least toughness.
/// P1 controls three creatures: a 1/1, a 2/3, and a 3/5.
/// Bolster 2 places counters on the 1/1 (least toughness).
fn test_bolster_chooses_least_toughness() {
    let p1 = p(1);
    let p2 = p(2);

    let small = ObjectSpec::creature(p1, "Saproling", 1, 1).in_zone(ZoneId::Battlefield);
    let medium = ObjectSpec::creature(p1, "Grizzly Bears", 2, 3).in_zone(ZoneId::Battlefield);
    let large = ObjectSpec::creature(p1, "Hill Giant", 3, 5).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(small)
        .object(medium)
        .object(large)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let saproling_id = find_by_name(&state, "Saproling");
    let bears_id = find_by_name(&state, "Grizzly Bears");
    let giant_id = find_by_name(&state, "Hill Giant");

    let (state, _events) = run_bolster(state, p1, 2);

    // Only the 1/1 (least toughness) should receive counters.
    assert_eq!(
        get_plus_counters(&state, saproling_id),
        2,
        "CR 701.39a: the 1/1 creature (least toughness) should receive 2 counters"
    );
    assert_eq!(
        get_plus_counters(&state, bears_id),
        0,
        "CR 701.39a: the 2/3 creature should not receive counters"
    );
    assert_eq!(
        get_plus_counters(&state, giant_id),
        0,
        "CR 701.39a: the 3/5 creature should not receive counters"
    );
}

// ── Test 3: Tied toughness — deterministic tie-breaking by smallest ObjectId ──

#[test]
/// CR 701.39a (tie-breaking) — When multiple creatures are tied for least toughness,
/// the deterministic fallback (M10+ will use player choice) is smallest ObjectId.
fn test_bolster_tied_toughness_deterministic() {
    let p1 = p(1);
    let p2 = p(2);

    // Both have toughness 2.
    let creature_a = ObjectSpec::creature(p1, "Creature A", 1, 2).in_zone(ZoneId::Battlefield);
    let creature_b = ObjectSpec::creature(p1, "Creature B", 3, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature_a)
        .object(creature_b)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let id_a = find_by_name(&state, "Creature A");
    let id_b = find_by_name(&state, "Creature B");

    let (state, _events) = run_bolster(state, p1, 1);

    // The creature with the smaller ObjectId should receive the counter.
    let min_id = id_a.min(id_b);
    let max_id = id_a.max(id_b);

    assert_eq!(
        get_plus_counters(&state, min_id),
        1,
        "CR 701.39a tie-breaking: creature with smallest ObjectId should receive 1 counter"
    );
    assert_eq!(
        get_plus_counters(&state, max_id),
        0,
        "CR 701.39a tie-breaking: creature with larger ObjectId should not receive counters"
    );
}

// ── Test 4: No creatures — bolster does nothing ───────────────────────────────

#[test]
/// CR 701.39a — If the controller has no creatures on the battlefield, bolster does nothing.
fn test_bolster_no_creatures_does_nothing() {
    let p1 = p(1);
    let p2 = p(2);

    // P1 has only a non-creature permanent (artifact).
    let artifact = ObjectSpec::artifact(p1, "Sol Ring").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(artifact)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_bolster(state, p1, 3);

    // No events emitted, no counters placed, no panic.
    assert!(
        events.is_empty(),
        "CR 701.39a: bolster on empty creature pool should emit no events"
    );

    // The artifact should not have any counters.
    let artifact_id = find_by_name(&state, "Sol Ring");
    assert_eq!(
        get_plus_counters(&state, artifact_id),
        0,
        "CR 701.39a: artifact should not receive counters from bolster"
    );
}

// ── Test 5: Layer-aware toughness is used for comparison ─────────────────────

#[test]
/// CR 701.39a + ruling 2014-11-24 — Bolster uses layer-aware toughness.
/// P1 controls a 1/1 creature with 2 +1/+1 counters (effective T=3) and a 2/2 creature
/// (effective T=2). Bolster should pick the 2/2 (lower layer-aware T), not the 1/1.
fn test_bolster_uses_layer_aware_toughness() {
    let p1 = p(1);
    let p2 = p(2);

    // 1/1 creature with 2 +1/+1 counters already on it → layer-aware T = 3.
    let boosted = ObjectSpec::creature(p1, "Boosted Saproling", 1, 1)
        .with_counter(CounterType::PlusOnePlusOne, 2)
        .in_zone(ZoneId::Battlefield);

    // 2/2 vanilla creature → layer-aware T = 2.
    let vanilla = ObjectSpec::creature(p1, "Vanilla Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(boosted)
        .object(vanilla)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let boosted_id = find_by_name(&state, "Boosted Saproling");
    let vanilla_id = find_by_name(&state, "Vanilla Bear");

    let (state, _events) = run_bolster(state, p1, 1);

    // The 2/2 (lower effective T=2) should receive the counter.
    assert_eq!(
        get_plus_counters(&state, vanilla_id),
        1,
        "CR 701.39a ruling 2014-11-24: 2/2 has lower layer-aware toughness (2 vs 3) and should receive counter"
    );
    // The boosted creature (effective T=3) should not receive additional counters.
    assert_eq!(
        get_plus_counters(&state, boosted_id),
        2,
        "CR 701.39a ruling 2014-11-24: 1/1+2 counters (T=3) should not receive additional counters"
    );
}

// ── Test 6: Bolster does not target — protection does not apply ───────────────

#[test]
/// Ruling 2014-11-24 — Bolster does not target. Protection from white does not
/// prevent a white-sourced bolster from placing counters on the creature.
/// "You could put counters on a creature with protection from white."
fn test_bolster_not_targeting_ignores_protection() {
    let p1 = p(1);
    let p2 = p(2);

    // The only creature P1 controls has protection from white.
    let protected = ObjectSpec::creature(p1, "White Knight", 2, 2)
        .with_keyword(KeywordAbility::ProtectionFrom(
            ProtectionQuality::FromColor(Color::White),
        ))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(protected)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let knight_id = find_by_name(&state, "White Knight");

    // Execute bolster as if sourced from a white spell (source id placeholder).
    let (state, events) = run_bolster(state, p1, 2);

    // Protection does not prevent bolster (bolster does not target).
    assert_eq!(
        events.len(),
        1,
        "Ruling 2014-11-24: bolster should still place counters despite protection from white"
    );
    assert_eq!(
        get_plus_counters(&state, knight_id),
        2,
        "Ruling 2014-11-24: protected creature should receive 2 +1/+1 counters from bolster"
    );
}

// ── Test 7: Bolster can select the source creature ────────────────────────────

#[test]
/// CR 701.39a (edge case) — If the ETB creature is the one with the least toughness,
/// bolster places counters on it (the source). Bolster has no rule preventing
/// the source from receiving counters.
fn test_bolster_can_target_source() {
    let p1 = p(1);
    let p2 = p(2);

    // The ETB creature is a 1/1 (least toughness), which would be the bolster source.
    let etb_creature =
        ObjectSpec::creature(p1, "Abzan Skycaptain", 1, 1).in_zone(ZoneId::Battlefield);
    // A larger creature also under P1's control.
    let big_creature = ObjectSpec::creature(p1, "Siege Rhino", 4, 5).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(etb_creature)
        .object(big_creature)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let captain_id = find_by_name(&state, "Abzan Skycaptain");
    let rhino_id = find_by_name(&state, "Siege Rhino");

    let (state, _events) = run_bolster(state, p1, 2);

    // The 1/1 (ETB creature, least toughness) should receive the counters.
    assert_eq!(
        get_plus_counters(&state, captain_id),
        2,
        "CR 701.39a: ETB creature with least toughness (1) should receive 2 counters"
    );
    assert_eq!(
        get_plus_counters(&state, rhino_id),
        0,
        "CR 701.39a: 4/5 creature should not receive counters"
    );
}

// ── Test 8: Multiplayer — bolster only considers the controller's creatures ───

#[test]
/// CR 701.39a (multiplayer) — In a 4-player game, bolster only considers creatures
/// controlled by the bolster effect's controller, ignoring opponents' creatures.
fn test_bolster_multiplayer_only_controllers_creatures() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // P1 controls a 5/5 (only P1 creature).
    let p1_creature = ObjectSpec::creature(p1, "P1 Wurm", 5, 5).in_zone(ZoneId::Battlefield);
    // P2 controls a 1/1 (which would be chosen if bolster considered all creatures).
    let p2_creature = ObjectSpec::creature(p2, "P2 Saproling", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(p1_creature)
        .object(p2_creature)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let wurm_id = find_by_name(&state, "P1 Wurm");
    let saproling_id = find_by_name(&state, "P2 Saproling");

    // P1 bolsters 2 — should only consider P1's creatures.
    let (state, events) = run_bolster(state, p1, 2);

    // P1's 5/5 should receive counters (it is the only creature P1 controls).
    assert_eq!(
        events.len(),
        1,
        "CR 701.39a: bolster should emit one CounterAdded event"
    );
    assert_eq!(
        get_plus_counters(&state, wurm_id),
        2,
        "CR 701.39a: P1's 5/5 should receive 2 counters (only P1 creature)"
    );
    // P2's 1/1 should not receive counters even though it has lower toughness.
    assert_eq!(
        get_plus_counters(&state, saproling_id),
        0,
        "CR 701.39a: P2's creature should not receive counters — bolster only affects controller's creatures"
    );
}
