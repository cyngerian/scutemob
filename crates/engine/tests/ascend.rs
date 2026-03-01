//! Ascend keyword ability tests (CR 702.131).
//!
//! Ascend grants the "city's blessing" designation to a player who controls
//! ten or more permanents and has a source with ascend.
//!
//! CR 702.131a: Ascend on an instant or sorcery is a spell ability. Grants the
//! city's blessing at resolution if the controller has 10+ permanents.
//!
//! CR 702.131b: Ascend on a permanent is a static ability. Grants the city's
//! blessing any time the controller has 10+ permanents and a permanent with ascend.
//!
//! CR 702.131c: The city's blessing is permanent — once gained, it cannot be lost.
//!
//! Key rules verified:
//! - 10+ permanents with an ascend source grants the city's blessing (CR 702.131b).
//! - Below threshold (9 permanents) does not grant the blessing.
//! - The blessing persists even after permanents drop below 10 (CR 702.131c).
//! - 10+ permanents WITHOUT an ascend source does NOT grant the blessing (key ruling).
//! - Multiple players can have the city's blessing simultaneously (CR 702.131c).
//! - Ascend on a spell grants the blessing at resolution, not cast time (CR 702.131a).
//! - Token permanents count toward the 10-permanent threshold.

use mtg_engine::{
    process_command, CardRegistry, CardType, Command, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ManaCost, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ────────────────────────────────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all listed players once (one round).
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

// ── Test 1: Basic — 10 permanents with ascend source grants the blessing ───────

#[test]
/// CR 702.131b — Ascend on a permanent: player controls 10 permanents including
/// one with Ascend. After SBA check the player gains the city's blessing.
///
/// Setup: P1 controls 9 basic lands + 1 creature with Ascend keyword (total = 10).
/// Action: Pass priority (SBA check fires).
/// Assert: P1.has_citys_blessing == true. CitysBlessingGained event emitted.
fn test_ascend_basic_permanent_grants_blessing() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let ascend_creature = ObjectSpec::creature(p1, "Wayward Swordtooth", 3, 3)
        .with_keyword(KeywordAbility::Ascend)
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ascend_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    // Add 9 lands to reach the 10-permanent threshold.
    for i in 1..=9 {
        builder = builder
            .object(ObjectSpec::land(p1, &format!("Forest {}", i)).in_zone(ZoneId::Battlefield));
    }

    let state = builder.build().unwrap();

    // Verify the count before SBA check.
    let count_before = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == p1)
        .count();
    assert_eq!(count_before, 10, "should control exactly 10 permanents");

    // Pass priority — SBA check runs.
    let (state, events) = pass_all(state, &[p1, p2]);

    // CitysBlessingGained event must be emitted for P1.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CitysBlessingGained { player } if *player == p1)),
        "CR 702.131b: CitysBlessingGained event expected for P1"
    );

    // P1 now has the city's blessing.
    assert!(
        state
            .players
            .get(&p1)
            .map(|p| p.has_citys_blessing)
            .unwrap_or(false),
        "CR 702.131b: P1 should have the city's blessing after controlling 10 permanents with ascend"
    );
}

// ── Test 2: Below threshold — 9 permanents, no blessing ───────────────────────

#[test]
/// CR 702.131b (negative case) — player controls an ascend permanent but only
/// 9 permanents total. The city's blessing is not granted.
///
/// Setup: P1 controls 8 basic lands + 1 creature with Ascend keyword (total = 9).
/// Action: Pass priority.
/// Assert: P1.has_citys_blessing == false. No CitysBlessingGained event.
fn test_ascend_below_threshold_no_blessing() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let ascend_creature = ObjectSpec::creature(p1, "Ascend Wannabe", 3, 3)
        .with_keyword(KeywordAbility::Ascend)
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ascend_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    // Add only 8 lands: creature + 8 lands = 9 total (below threshold).
    for i in 1..=8 {
        builder = builder
            .object(ObjectSpec::land(p1, &format!("Swamp {}", i)).in_zone(ZoneId::Battlefield));
    }

    let state = builder.build().unwrap();

    let count_before = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == p1)
        .count();
    assert_eq!(count_before, 9, "should control exactly 9 permanents");

    let (state, events) = pass_all(state, &[p1, p2]);

    // No CitysBlessingGained event.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CitysBlessingGained { player } if *player == p1)),
        "CR 702.131b: CitysBlessingGained must NOT fire when controlling only 9 permanents"
    );

    // P1 does not have the city's blessing.
    assert!(
        !state
            .players
            .get(&p1)
            .map(|p| p.has_citys_blessing)
            .unwrap_or(true),
        "CR 702.131b: P1 should NOT have the city's blessing with only 9 permanents"
    );
}

// ── Test 3: Blessing is permanent once gained ──────────────────────────────────

#[test]
/// CR 702.131c — "The city's blessing is a designation... that other rules and
/// effects can identify." Once gained, it persists even if permanents drop below 10.
///
/// Setup: Manually set P1.has_citys_blessing = true (simulates having gained it).
///   Then P1 controls only 5 permanents (none with Ascend).
/// Action: Pass priority.
/// Assert: P1 still has the city's blessing (it cannot be lost).
fn test_ascend_blessing_permanent_once_gained() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    // Only 5 permanents — well below threshold.
    for i in 1..=5 {
        builder = builder
            .object(ObjectSpec::land(p1, &format!("Island {}", i)).in_zone(ZoneId::Battlefield));
    }

    let mut state = builder.build().unwrap();

    // Simulate the player having previously gained the blessing.
    if let Some(p) = state.players.get_mut(&p1) {
        p.has_citys_blessing = true;
    }

    // Pass priority — SBA check runs. Even with < 10 permanents, blessing is kept.
    let (state, events) = pass_all(state, &[p1, p2]);

    // No NEW CitysBlessingGained event (blessing was already set).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CitysBlessingGained { player } if *player == p1)),
        "CR 702.131c: CitysBlessingGained must NOT fire again for a player who already has it"
    );

    // Blessing is still true.
    assert!(
        state
            .players
            .get(&p1)
            .map(|p| p.has_citys_blessing)
            .unwrap_or(false),
        "CR 702.131c: the city's blessing must persist — it cannot be lost once gained"
    );
}

// ── Test 4: No ascend source — 10+ permanents, no blessing ────────────────────

#[test]
/// Key ruling on Wayward Swordtooth: "If you control ten permanents but don't
/// control a permanent or resolving spell with ascend, you don't get the city's
/// blessing."
///
/// Setup: P1 controls 10 basic lands — all without Ascend keyword.
/// Action: Pass priority.
/// Assert: P1.has_citys_blessing == false (no ascend source).
fn test_ascend_no_ascend_source_no_blessing() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    // 10 lands with no Ascend keyword.
    for i in 1..=10 {
        builder = builder
            .object(ObjectSpec::land(p1, &format!("Mountain {}", i)).in_zone(ZoneId::Battlefield));
    }

    let state = builder.build().unwrap();

    let count = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == p1)
        .count();
    assert_eq!(
        count, 10,
        "should control exactly 10 permanents (no ascend)"
    );

    let (state, events) = pass_all(state, &[p1, p2]);

    // No CitysBlessingGained — no ascend source.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CitysBlessingGained { player } if *player == p1)),
        "Key ruling: CitysBlessingGained must NOT fire without an ascend source"
    );

    assert!(
        !state
            .players
            .get(&p1)
            .map(|p| p.has_citys_blessing)
            .unwrap_or(true),
        "Key ruling: P1 must NOT gain the city's blessing without an ascend permanent"
    );
}

// ── Test 5: Multiple players can have the blessing simultaneously ──────────────

#[test]
/// CR 702.131c — "Any number of players may have the city's blessing at the
/// same time." P1 and P2 each independently gain the blessing.
///
/// Setup: P1 controls 9 lands + 1 Ascend creature (10 total).
///   P2 controls 9 lands + 1 Ascend creature (10 total).
/// Action: Pass priority.
/// Assert: Both P1 and P2 have has_citys_blessing == true.
fn test_ascend_multiple_players_independent() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let ascend_p1 = ObjectSpec::creature(p1, "P1 Ascend Creature", 3, 3)
        .with_keyword(KeywordAbility::Ascend)
        .in_zone(ZoneId::Battlefield);

    let ascend_p2 = ObjectSpec::creature(p2, "P2 Ascend Creature", 3, 3)
        .with_keyword(KeywordAbility::Ascend)
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ascend_p1)
        .object(ascend_p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    // 9 lands each — combined with the ascend creature, each has 10 permanents.
    for i in 1..=9 {
        builder = builder
            .object(ObjectSpec::land(p1, &format!("P1 Plains {}", i)).in_zone(ZoneId::Battlefield));
        builder = builder
            .object(ObjectSpec::land(p2, &format!("P2 Plains {}", i)).in_zone(ZoneId::Battlefield));
    }

    let state = builder.build().unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // Both players gain the city's blessing.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CitysBlessingGained { player } if *player == p1)),
        "CR 702.131c: CitysBlessingGained expected for P1"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CitysBlessingGained { player } if *player == p2)),
        "CR 702.131c: CitysBlessingGained expected for P2"
    );

    assert!(
        state
            .players
            .get(&p1)
            .map(|p| p.has_citys_blessing)
            .unwrap_or(false),
        "CR 702.131c: P1 should have the city's blessing"
    );
    assert!(
        state
            .players
            .get(&p2)
            .map(|p| p.has_citys_blessing)
            .unwrap_or(false),
        "CR 702.131c: P2 should have the city's blessing"
    );
}

// ── Test 6: Tokens count as permanents ────────────────────────────────────────

#[test]
/// Ruling: "A permanent is any object on the battlefield, including tokens and
/// lands. Spells and emblems aren't permanents."
///
/// Setup: P1 controls 5 lands + 1 Ascend creature + 4 token creatures (total = 10).
/// Action: Pass priority.
/// Assert: P1 has the city's blessing (tokens counted).
fn test_ascend_tokens_count_as_permanents() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let ascend_creature = ObjectSpec::creature(p1, "Jungle Creeper", 3, 3)
        .with_keyword(KeywordAbility::Ascend)
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ascend_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    // 5 lands.
    for i in 1..=5 {
        builder = builder.object(
            ObjectSpec::land(p1, &format!("Jungle Land {}", i)).in_zone(ZoneId::Battlefield),
        );
    }

    // 4 token creatures (tokens are permanents per ruling).
    for i in 1..=4 {
        builder = builder.object(
            ObjectSpec::creature(p1, &format!("Saproling Token {}", i), 1, 1)
                .token()
                .in_zone(ZoneId::Battlefield),
        );
    }

    let state = builder.build().unwrap();

    // Verify: 1 (ascend) + 5 (lands) + 4 (tokens) = 10.
    let count = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == p1)
        .count();
    assert_eq!(
        count, 10,
        "should control exactly 10 permanents (including tokens)"
    );

    let (state, events) = pass_all(state, &[p1, p2]);

    // CitysBlessingGained emitted because tokens count.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CitysBlessingGained { player } if *player == p1)),
        "Ruling: CitysBlessingGained expected — tokens count toward the 10-permanent threshold"
    );

    assert!(
        state
            .players
            .get(&p1)
            .map(|p| p.has_citys_blessing)
            .unwrap_or(false),
        "Ruling: P1 should have the city's blessing when tokens bring the count to 10"
    );
}

// ── Test 7: Ascend on instant/sorcery — blessing granted at resolution ─────────

#[test]
/// CR 702.131a — Ascend on an instant or sorcery is a spell ability.
/// "If you control ten or more permanents and you don't have the city's blessing,
/// you get the city's blessing for the rest of the game."
///
/// This means: the blessing is NOT granted when the spell is cast (the spell is
/// on the stack), only when it resolves.
///
/// Setup: P1 controls 10 permanents. P1 casts a sorcery with Ascend (zero mana cost).
/// Assert after cast (before resolve): has_citys_blessing == false.
/// Resolve the spell. Assert after resolve: has_citys_blessing == true.
fn test_ascend_instant_sorcery_on_resolution() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Sorcery with Ascend keyword, zero mana cost so it's free to cast.
    let ascend_sorcery = ObjectSpec::card(p1, "Verdant Confluence")
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 0,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Ascend)
        .in_zone(ZoneId::Hand(p1));

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ascend_sorcery)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    // 10 permanents on battlefield (no Ascend source there — only the spell has it).
    for i in 1..=10 {
        builder = builder.object(
            ObjectSpec::land(p1, &format!("Verdant Land {}", i)).in_zone(ZoneId::Battlefield),
        );
    }

    let state = builder.build().unwrap();

    // Verify 10 permanents (lands only — spell is in hand, not a permanent).
    let land_count = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == p1)
        .count();
    assert_eq!(land_count, 10, "should have 10 permanents before casting");

    // ASSERT: no blessing yet before casting.
    assert!(
        !state
            .players
            .get(&p1)
            .map(|p| p.has_citys_blessing)
            .unwrap_or(true),
        "CR 702.131a: P1 must NOT have the city's blessing before the spell is cast"
    );

    // Find the sorcery card in hand.
    let card_id = find_object(&state, "Verdant Confluence");

    // Cast the sorcery.
    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
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
        },
    )
    .expect("CastSpell should succeed");

    // ASSERT: no blessing after cast, before resolution.
    // CR 702.131a ruling: "If you cast a spell with ascend, you don't get the
    // city's blessing until it resolves."
    assert!(
        !state
            .players
            .get(&p1)
            .map(|p| p.has_citys_blessing)
            .unwrap_or(true),
        "CR 702.131a: blessing must NOT be granted at cast time, only at resolution"
    );

    // Both players pass priority to resolve the spell.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // ASSERT: CitysBlessingGained event in the resolve batch.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::CitysBlessingGained { player } if *player == p1)),
        "CR 702.131a: CitysBlessingGained must be emitted when the ascend spell resolves"
    );

    // ASSERT: P1 has the blessing after resolution.
    assert!(
        state
            .players
            .get(&p1)
            .map(|p| p.has_citys_blessing)
            .unwrap_or(false),
        "CR 702.131a: P1 should have the city's blessing after the ascend sorcery resolves"
    );
}
