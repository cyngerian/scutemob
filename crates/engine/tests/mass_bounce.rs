//! Mass Bounce tests — PB-G.
//!
//! Verifies:
//! - Effect::BounceAll: returns matching permanents to owners' hands
//! - TargetFilter.exclude_subtypes: skips permanents with listed subtypes
//! - TargetFilter.max_toughness: only bounces creatures with toughness <= N
//! - max_toughness_amount: dynamic toughness threshold via EffectAmount
//! - Controller filter: Opponent-only bounce
//! - CR 400.7: bounced objects become new objects in hand

use mtg_engine::cards::card_definition::PlayerTarget;
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::state::types::SubType;
use mtg_engine::{
    CardType, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec,
    PlayerId, Step, TargetController, TargetFilter, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn count_in_hand(state: &GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(player))
        .count()
}

fn count_on_battlefield(state: &GameState) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield)
        .count()
}

fn run_effect(
    mut state: GameState,
    controller: PlayerId,
    effect: Effect,
) -> (GameState, Vec<GameEvent>, u32) {
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(controller, source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);
    let last_count = ctx.last_effect_count;
    (state, events, last_count)
}

// ── CR 400.7: BounceAll basic ────────────────────────────────────────────────

#[test]
/// CR 400.7 — BounceAll with creature filter returns all creatures to owners' hands;
/// non-creatures stay on battlefield.
fn test_bounce_all_creatures_basic() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Bear", 2, 2))
        .object(ObjectSpec::creature(p(2), "Goblin", 1, 1))
        .object(
            ObjectSpec::card(p(1), "Sol Ring")
                .with_types(vec![CardType::Artifact])
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    assert_eq!(count_on_battlefield(&state), 3);

    let (state, _events, bounced) = run_effect(
        state,
        p(1),
        Effect::BounceAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
            max_toughness_amount: None,
        },
    );

    // Both creatures bounced, artifact stays.
    assert_eq!(bounced, 2);
    assert_eq!(count_on_battlefield(&state), 1); // Sol Ring
    assert_eq!(count_in_hand(&state, p(1)), 1); // Bear
    assert_eq!(count_in_hand(&state, p(2)), 1); // Goblin
}

// ── TargetFilter.exclude_subtypes ────────────────────────────────────────────

#[test]
/// BounceAll with exclude_subtypes skips creatures with those subtypes.
/// (Whelming Wave pattern: "except for Krakens, Leviathans, Octopuses, and Serpents")
fn test_bounce_all_exclude_subtypes() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Bear", 2, 2))
        .object(
            ObjectSpec::creature(p(2), "Big Kraken", 6, 6)
                .with_subtypes(vec![SubType("Kraken".to_string())]),
        )
        .object(ObjectSpec::creature(p(2), "Elf", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, _, bounced) = run_effect(
        state,
        p(1),
        Effect::BounceAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                exclude_subtypes: vec![
                    SubType("Kraken".to_string()),
                    SubType("Serpent".to_string()),
                ],
                ..Default::default()
            },
            max_toughness_amount: None,
        },
    );

    // Bear and Elf bounced; Kraken stays.
    assert_eq!(bounced, 2);
    assert_eq!(count_on_battlefield(&state), 1);
    let kraken = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Big Kraken");
    assert!(kraken.is_some(), "Kraken should remain on battlefield");
}

// ── TargetFilter non_creature + non_land ─────────────────────────────────────

#[test]
/// BounceAll with non_creature + non_land bounces artifacts/enchantments/planeswalkers,
/// keeps creatures and lands. (Filter Out pattern)
fn test_bounce_all_noncreature_nonland() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Bear", 2, 2))
        .object(
            ObjectSpec::card(p(1), "Sol Ring")
                .with_types(vec![CardType::Artifact])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p(2), "Some Enchantment")
                .with_types(vec![CardType::Enchantment])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p(1), "Forest")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, _, bounced) = run_effect(
        state,
        p(1),
        Effect::BounceAll {
            filter: TargetFilter {
                non_creature: true,
                non_land: true,
                ..Default::default()
            },
            max_toughness_amount: None,
        },
    );

    // Sol Ring and enchantment bounced; creature and land stay.
    assert_eq!(bounced, 2);
    assert_eq!(count_on_battlefield(&state), 2); // Bear + Forest
}

// ── max_toughness_amount (dynamic) ───────────────────────────────────────────

#[test]
/// BounceAll with max_toughness_amount bounces only opponents' creatures with toughness
/// at or below the resolved amount. (Scourge of Fleets pattern)
fn test_bounce_all_opponent_creatures_dynamic_toughness() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        // p(1) controls 2 Islands — threshold is 2
        .object(
            ObjectSpec::card(p(1), "Island A")
                .with_types(vec![CardType::Land])
                .with_subtypes(vec![SubType("Island".to_string())])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p(1), "Island B")
                .with_types(vec![CardType::Land])
                .with_subtypes(vec![SubType("Island".to_string())])
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::creature(p(2), "Small Fish", 1, 1))
        .object(ObjectSpec::creature(p(2), "Medium Fish", 2, 2))
        .object(ObjectSpec::creature(p(2), "Big Fish", 3, 5))
        .object(ObjectSpec::creature(p(1), "Own Bear", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, _, bounced) = run_effect(
        state,
        p(1),
        Effect::BounceAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                controller: TargetController::Opponent,
                ..Default::default()
            },
            max_toughness_amount: Some(EffectAmount::PermanentCount {
                filter: TargetFilter {
                    has_subtype: Some(SubType("Island".to_string())),
                    ..Default::default()
                },
                controller: PlayerTarget::Controller,
            }),
        },
    );

    // 2 Islands → threshold 2. Small Fish (T=1) and Medium Fish (T=2) bounced.
    // Big Fish (T=5) stays. Own Bear not affected (controller filter).
    assert_eq!(bounced, 2);
    // Remaining: 2 Islands + Big Fish + Own Bear = 4
    assert_eq!(count_on_battlefield(&state), 4);
    assert_eq!(count_in_hand(&state, p(2)), 2);
}

// ── TargetFilter.max_toughness (static) ──────────────────────────────────────

#[test]
/// max_toughness on TargetFilter correctly filters by toughness.
fn test_bounce_all_max_toughness_static() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Small", 1, 1))
        .object(ObjectSpec::creature(p(1), "Medium", 2, 3))
        .object(ObjectSpec::creature(p(2), "Big", 4, 5))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, _, bounced) = run_effect(
        state,
        p(1),
        Effect::BounceAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                max_toughness: Some(3),
                ..Default::default()
            },
            max_toughness_amount: None,
        },
    );

    // Small (T=1) and Medium (T=3) bounced; Big (T=5) stays.
    assert_eq!(bounced, 2);
    assert_eq!(count_on_battlefield(&state), 1);
}

// ── Count tracking ───────────────────────────────────────────────────────────

#[test]
/// BounceAll correctly sets ctx.last_effect_count to the number of bounced permanents.
fn test_bounce_all_count_tracking() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(ObjectSpec::creature(p(1), "A", 1, 1))
        .object(ObjectSpec::creature(p(1), "B", 2, 2))
        .object(ObjectSpec::creature(p(1), "C", 3, 3))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (_state, _events, bounced) = run_effect(
        state,
        p(1),
        Effect::BounceAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
            max_toughness_amount: None,
        },
    );

    assert_eq!(bounced, 3);
}

// ── Multiplayer ──────────────────────────────────────────────────────────────

#[test]
/// BounceAll with controller Opponent bounces all 3 opponents' creatures in a 4-player game.
fn test_bounce_all_multiplayer_opponent_only() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(ObjectSpec::creature(p(1), "My Bear", 2, 2))
        .object(ObjectSpec::creature(p(2), "Opp2 Goblin", 1, 1))
        .object(ObjectSpec::creature(p(3), "Opp3 Elf", 1, 1))
        .object(ObjectSpec::creature(p(4), "Opp4 Soldier", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, _, bounced) = run_effect(
        state,
        p(1),
        Effect::BounceAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                controller: TargetController::Opponent,
                ..Default::default()
            },
            max_toughness_amount: None,
        },
    );

    // 3 opponents' creatures bounced, controller's stays.
    assert_eq!(bounced, 3);
    assert_eq!(count_on_battlefield(&state), 1); // My Bear
    assert_eq!(count_in_hand(&state, p(2)), 1);
    assert_eq!(count_in_hand(&state, p(3)), 1);
    assert_eq!(count_in_hand(&state, p(4)), 1);
}

// ── exclude_subtypes on DestroyAll (Crux of Fate fix verification) ───────────

#[test]
/// exclude_subtypes works on DestroyAll too (Crux of Fate: destroy all non-Dragon creatures).
fn test_destroy_all_exclude_subtypes() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Bear", 2, 2))
        .object(
            ObjectSpec::creature(p(2), "Shivan Dragon", 5, 5)
                .with_subtypes(vec![SubType("Dragon".to_string())]),
        )
        .object(ObjectSpec::creature(p(1), "Elf", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, _, destroyed) = run_effect(
        state,
        p(1),
        Effect::DestroyAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                exclude_subtypes: vec![SubType("Dragon".to_string())],
                ..Default::default()
            },
            cant_be_regenerated: false,
        },
    );

    // Bear and Elf destroyed; Dragon survives.
    assert_eq!(destroyed, 2);
    assert_eq!(count_on_battlefield(&state), 1);
    let dragon = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Shivan Dragon");
    assert!(dragon.is_some(), "Dragon should survive");
}
