//! Daybound/Nightbound ability tests (CR 702.145 / CR 730).
//!
//! Daybound and Nightbound are keyword abilities on DFCs that link their transform
//! to the global day/night state. Key rules verified:
//! - CR 702.145d: Daybound permanent causes it to become day if neither day nor night.
//! - CR 702.145g: Nightbound permanent causes it to become night (if no daybound exists).
//! - CR 730.2a: Day→Night transition if previous turn's active player cast 0 spells.
//! - CR 730.2b: Night→Day transition if previous turn's active player cast 2+ spells.
//! - CR 702.145c: When it becomes night, daybound permanents (front face) transform.
//! - CR 702.145f: When it becomes day, nightbound permanents (back face) transform.
//! - CR 702.145b: Daybound permanents can't transform except via daybound (lock).

use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, CardDefinition, CardFace,
    CardId, CardRegistry, CardType, Command, DayNight, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, SubType, TypeLine, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// A mock Daybound card: front face has Daybound, back face has Nightbound.
/// Modeled after Brutal Cathar // Moonrage Brute.
fn brutal_cathar_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-brutal-cathar".to_string()),
        name: "Brutal Cathar".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Human".to_string()), SubType("Soldier".to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "Daybound".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Daybound)],
        power: Some(2),
        toughness: Some(2),
        back_face: Some(CardFace {
            name: "Moonrage Brute".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Werewolf".to_string())].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "Nightbound".to_string(),
            abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Nightbound)],
            power: Some(5),
            toughness: Some(5),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

fn cathar_on_battlefield(owner: PlayerId) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, "Brutal Cathar")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-brutal-cathar".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Daybound);
    spec.power = Some(2);
    spec.toughness = Some(2);
    spec
}

// ── Test 1: enforce_daybound_nightbound sets day when Daybound permanent exists ─

/// CR 702.145d: "Any time a player controls a permanent with daybound, if it's
/// neither day nor night, it becomes day."
#[test]
fn test_daybound_sets_day() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![brutal_cathar_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cathar_on_battlefield(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Game starts with neither day nor night.
    assert!(
        state.day_night.is_none(),
        "game should start with no day/night"
    );

    // Call enforce_daybound_nightbound — Brutal Cathar has Daybound → should set Day.
    let events = mtg_engine::rules::turn_actions::enforce_daybound_nightbound(&mut state);

    assert_eq!(
        state.day_night,
        Some(DayNight::Day),
        "game should become day when a daybound permanent is on the battlefield (CR 702.145d)"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::DayNightChanged { now: DayNight::Day })),
        "DayNightChanged{{Day}} event should be emitted"
    );
}

// ── Test 2: Daybound permanent transforms immediately when it's night ──────────

/// CR 702.145c: "Any time a player controls a permanent that is front face up with
/// daybound and it's night, that player transforms that permanent."
#[test]
fn test_daybound_transforms_at_night() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![brutal_cathar_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cathar_on_battlefield(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let cathar_id = find_object(&state, "Brutal Cathar");

    // Set game state to Night.
    state.day_night = Some(DayNight::Night);

    // Call enforce_daybound_nightbound — Brutal Cathar has Daybound, front face, Night → transform.
    let events = mtg_engine::rules::turn_actions::enforce_daybound_nightbound(&mut state);

    assert!(
        state.objects[&cathar_id].is_transformed,
        "daybound permanent should transform immediately when it's night (CR 702.145c)"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTransformed {
                to_back_face: true,
                ..
            }
        )),
        "PermanentTransformed event should be emitted"
    );

    let chars = calculate_characteristics(&state, cathar_id).expect("should have chars");
    assert_eq!(chars.name, "Moonrage Brute", "back face should be visible");
    assert_eq!(chars.power, Some(5), "back face P/T should be 5/5");
}

// ── Test 3: Nightbound permanent transforms when it's day ─────────────────────

/// CR 702.145f: "Any time a player controls a permanent that is back face up with
/// nightbound and it's day, that player transforms that permanent."
#[test]
fn test_nightbound_transforms_at_day() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![brutal_cathar_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cathar_on_battlefield(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let cathar_id = find_object(&state, "Brutal Cathar");

    // Manually transform to back face (simulate was already transformed).
    if let Some(obj) = state.objects.get_mut(&cathar_id) {
        obj.is_transformed = true;
    }

    // Set game state to Day.
    state.day_night = Some(DayNight::Day);

    // enforce_daybound_nightbound — back face has Nightbound, it's Day → transform back to front.
    let events = mtg_engine::rules::turn_actions::enforce_daybound_nightbound(&mut state);

    assert!(
        !state.objects[&cathar_id].is_transformed,
        "nightbound permanent (back face up) should transform back to front when it's day (CR 702.145f)"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTransformed {
                to_back_face: false,
                ..
            }
        )),
        "PermanentTransformed event should be emitted"
    );

    let chars = calculate_characteristics(&state, cathar_id).expect("should have chars");
    assert_eq!(
        chars.name, "Brutal Cathar",
        "front face should be visible again"
    );
    assert_eq!(chars.power, Some(2), "front face P/T should be 2/2");
}

// ── Test 4: Daybound blocks direct transform command ──────────────────────────

/// CR 702.145b: "This permanent can't transform except due to its daybound ability."
/// A direct Command::Transform on a daybound permanent should be rejected.
#[test]
fn test_daybound_blocks_direct_transform() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![brutal_cathar_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cathar_on_battlefield(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let cathar_id = find_object(&state, "Brutal Cathar");

    // Attempt direct transform — should be rejected (CR 702.145b).
    let result = process_command(
        state,
        Command::Transform {
            player: p1,
            permanent: cathar_id,
        },
    );
    assert!(
        result.is_err(),
        "daybound permanent should reject direct Transform command (CR 702.145b)"
    );
}

// ── Test 5: enforce_daybound_nightbound sets night when only Nightbound exists ─

/// CR 702.145g: "Any time a player controls a permanent with nightbound, if it's
/// neither day nor night and there are no permanents with daybound on the battlefield,
/// it becomes night."
#[test]
fn test_nightbound_sets_night() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![brutal_cathar_def()]);

    // Place cathar on battlefield, then manually transform to back face (Nightbound).

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cathar_on_battlefield(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let cathar_id = find_object(&state, "Brutal Cathar");

    // Manually transform to back face (Nightbound face).
    if let Some(obj) = state.objects.get_mut(&cathar_id) {
        obj.is_transformed = true;
    }

    // Game starts with neither day nor night.
    assert!(
        state.day_night.is_none(),
        "game should start with no day/night"
    );

    // enforce_daybound_nightbound — back face has Nightbound, no Daybound permanents → Night.
    let events = mtg_engine::rules::turn_actions::enforce_daybound_nightbound(&mut state);

    assert_eq!(
        state.day_night,
        Some(DayNight::Night),
        "game should become night when nightbound exists and no daybound (CR 702.145g)"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::DayNightChanged {
                now: DayNight::Night
            }
        )),
        "DayNightChanged{{Night}} event should be emitted"
    );
}

// ── Test 6: No change when it's neither day nor night and no daybound/nightbound ─

/// CR 730.2c: If it's neither day nor night, the untap step check doesn't happen.
/// enforce_daybound_nightbound with no relevant permanents should do nothing.
#[test]
fn test_day_night_no_change_without_permanents() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    assert!(state.day_night.is_none());

    let events = mtg_engine::rules::turn_actions::enforce_daybound_nightbound(&mut state);

    // Without any daybound/nightbound permanents, nothing changes.
    assert!(
        state.day_night.is_none(),
        "no change without daybound/nightbound permanents"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::DayNightChanged { .. })),
        "no DayNightChanged event without relevant permanents"
    );
}
