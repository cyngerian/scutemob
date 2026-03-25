//! Characteristic-Defining Ability (CDA) P/T tests (CR 604.3, 613.4a).
//!
//! CDAs are static abilities that define an object's power and/or toughness
//! dynamically based on the game state. They apply in Layer 7a (CR 613.4a)
//! before other P/T effects.
//!
//! Key rules verified:
//! - CR 604.3, 613.4a: CDA P/T is evaluated dynamically against current game state.
//! - CR 613.4a/b: Layer 7a (CDA) applies before Layer 7b (P/T-setting like Humility).
//! - CR 613.4a/c: Layer 7a (CDA) applies before Layer 7c (P/T counters).
//! - CR 604.3a(5): CDAs are unconditional — no condition field.
//! - EffectAmount::Sum: P/T = battlefield count + graveyard count.

use mtg_engine::{
    calculate_characteristics, CardType, ContinuousEffect, CounterType, EffectAmount,
    EffectDuration, EffectFilter, EffectId, EffectLayer, GameStateBuilder, LayerModification,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, SubType, TargetFilter, ZoneId, ZoneTarget,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn subtype(s: &str) -> SubType {
    SubType(s.to_string())
}

fn find_bf_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("battlefield object '{}' not found", name))
}

/// Create a CDA continuous effect that sets P/T via creature count.
fn make_cda_effect(
    id: u64,
    source: ObjectId,
    filter: EffectFilter,
    count_filter: TargetFilter,
    controller: PlayerTarget,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: Some(source),
        timestamp: id,
        layer: EffectLayer::PtCda,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter,
        modification: LayerModification::SetPtDynamic {
            power: Box::new(EffectAmount::PermanentCount {
                filter: count_filter.clone(),
                controller: controller.clone(),
            }),
            toughness: Box::new(EffectAmount::PermanentCount {
                filter: count_filter,
                controller,
            }),
        },
        is_cda: true,
        condition: None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 604.3, 613.4a: CDA creature counts itself when evaluating P/T.
/// A "*/* = number of creatures you control" creature includes itself.
#[test]
fn test_cda_counts_self() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "CDA Creature", 0, 0)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let cda_id = find_bf_object(&state, "CDA Creature");

    // Add the CDA continuous effect (simulates what register_static_continuous_effects does).
    let cda_effect = make_cda_effect(
        100,
        cda_id,
        EffectFilter::SingleObject(cda_id),
        TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        PlayerTarget::Controller,
    );

    let mut state = state;
    state.continuous_effects.push_back(cda_effect);

    let chars = calculate_characteristics(&state, cda_id).unwrap();
    // The CDA creature counts itself — 1 creature → P/T = 1/1.
    assert_eq!(
        chars.power,
        Some(1),
        "CDA creature should count itself (power)"
    );
    assert_eq!(
        chars.toughness,
        Some(1),
        "CDA creature should count itself (toughness)"
    );
}

/// CR 604.3, 613.4a: With two creatures, P/T = 2.
#[test]
fn test_cda_power_toughness_basic() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "CDA Creature", 0, 0)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p(1), "Vanilla Soldier", 2, 2)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let cda_id = find_bf_object(&state, "CDA Creature");

    let cda_effect = make_cda_effect(
        100,
        cda_id,
        EffectFilter::SingleObject(cda_id),
        TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        PlayerTarget::Controller,
    );

    let mut state = state;
    state.continuous_effects.push_back(cda_effect);

    let chars = calculate_characteristics(&state, cda_id).unwrap();
    // Two creatures (self + Vanilla Soldier) → P/T = 2/2.
    assert_eq!(chars.power, Some(2), "CDA: 2 creatures → power 2");
    assert_eq!(chars.toughness, Some(2), "CDA: 2 creatures → toughness 2");
}

/// CR 613.4a/b: CDA applies in Layer 7a; Layer 7b ("Humility"-style) applies after and wins.
/// CDA sets 2/2 in 7a, then 7b overwrites to 1/1.
#[test]
fn test_cda_layer_7a_before_7b() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "CDA Layer Test", 0, 0)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p(1), "Other Creature", 1, 1)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let cda_id = find_bf_object(&state, "CDA Layer Test");

    let cda_effect = make_cda_effect(
        100,
        cda_id,
        EffectFilter::SingleObject(cda_id),
        TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        PlayerTarget::Controller,
    );
    // Layer 7b: "base P/T becomes 1/1" (like Humility).
    let humility_effect = ContinuousEffect {
        id: EffectId(200),
        source: None,
        timestamp: 1000, // later timestamp
        layer: EffectLayer::PtSet,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::SingleObject(cda_id),
        modification: LayerModification::SetPowerToughness {
            power: 1,
            toughness: 1,
        },
        is_cda: false,
        condition: None,
    };

    let mut state = state;
    state.continuous_effects.push_back(cda_effect);
    state.continuous_effects.push_back(humility_effect);

    // CDA sets 2/2 (2 creatures) in 7a, then Humility overwrites to 1/1 in 7b.
    let chars = calculate_characteristics(&state, cda_id).unwrap();
    assert_eq!(
        chars.power,
        Some(1),
        "Layer 7b overrides CDA (Layer 7a): power should be 1"
    );
    assert_eq!(
        chars.toughness,
        Some(1),
        "Layer 7b overrides CDA (Layer 7a): toughness should be 1"
    );
}

/// CR 613.4a/c: CDA applies in Layer 7a; counters apply in Layer 7c and stack on top.
/// CDA sets 1/1, +1/+1 counter adds to get 2/2.
#[test]
fn test_cda_layer_7a_before_7c() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "CDA Counter Test", 0, 0)
                .with_types(vec![CardType::Creature])
                .with_counter(CounterType::PlusOnePlusOne, 1)
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let cda_id = find_bf_object(&state, "CDA Counter Test");

    let cda_effect = make_cda_effect(
        100,
        cda_id,
        EffectFilter::SingleObject(cda_id),
        TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        PlayerTarget::Controller,
    );

    let mut state = state;
    state.continuous_effects.push_back(cda_effect);

    // CDA: 1 creature (self) → 1/1. +1/+1 counter adds in Layer 7c → 2/2.
    let chars = calculate_characteristics(&state, cda_id).unwrap();
    assert_eq!(chars.power, Some(2), "CDA (1/1) + counter (+1) = 2 power");
    assert_eq!(
        chars.toughness,
        Some(2),
        "CDA (1/1) + counter (+1) = 2 toughness"
    );
}

/// CR 604.3a(5): CDA is always unconditional and dynamically re-evaluated.
/// Verify that removing a creature decreases the CDA value.
#[test]
fn test_cda_updates_dynamically() {
    // State A: 3 creatures → P/T = 3.
    let state_a = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "CDA Dyn", 0, 0)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p(1), "Friend A", 1, 1)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p(1), "Friend B", 1, 1)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();
    let cda_id_a = find_bf_object(&state_a, "CDA Dyn");
    let mut state_a = state_a;
    state_a.continuous_effects.push_back(make_cda_effect(
        100,
        cda_id_a,
        EffectFilter::SingleObject(cda_id_a),
        TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        PlayerTarget::Controller,
    ));
    let chars_a = calculate_characteristics(&state_a, cda_id_a).unwrap();
    assert_eq!(chars_a.power, Some(3), "With 3 creatures: power = 3");

    // State B: only 1 creature (the CDA creature itself) → P/T = 1.
    let state_b = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "CDA Dyn", 0, 0)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();
    let cda_id_b = find_bf_object(&state_b, "CDA Dyn");
    let mut state_b = state_b;
    state_b.continuous_effects.push_back(make_cda_effect(
        100,
        cda_id_b,
        EffectFilter::SingleObject(cda_id_b),
        TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        PlayerTarget::Controller,
    ));
    let chars_b = calculate_characteristics(&state_b, cda_id_b).unwrap();
    assert_eq!(
        chars_b.power,
        Some(1),
        "With 1 creature (self only): power = 1"
    );
}

/// CR 604.3, 613.4a: Partial CDA — only power is dynamic, toughness is Fixed(4).
/// (Adeline, Resplendent Cathar pattern: power = creature count, toughness = 4.)
#[test]
fn test_cda_partial_power_only() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "Partial CDA", 0, 4)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p(1), "Other Creature", 1, 1)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let cda_id = find_bf_object(&state, "Partial CDA");

    // CDA only sets power dynamically; toughness is Fixed(4).
    let cda_effect = ContinuousEffect {
        id: EffectId(100),
        source: Some(cda_id),
        timestamp: 100,
        layer: EffectLayer::PtCda,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(cda_id),
        modification: LayerModification::SetPtDynamic {
            power: Box::new(EffectAmount::PermanentCount {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                },
                controller: PlayerTarget::Controller,
            }),
            toughness: Box::new(EffectAmount::Fixed(4)),
        },
        is_cda: true,
        condition: None,
    };

    let mut state = state;
    state.continuous_effects.push_back(cda_effect);

    let chars = calculate_characteristics(&state, cda_id).unwrap();
    // 2 creatures (self + Other Creature): power = 2, toughness = 4.
    assert_eq!(chars.power, Some(2), "Partial CDA: power = 2 (2 creatures)");
    assert_eq!(
        chars.toughness,
        Some(4),
        "Partial CDA: toughness = 4 (Fixed)"
    );
}

/// CR 604.3, 613.4a: Multiplayer CDA — "Goblins on the battlefield" counts all players'.
/// (Reckless One pattern: P/T = total Goblin count across all players.)
#[test]
fn test_cda_multiplayer_all_players() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        // Player 1 controls the CDA creature (itself is a Goblin) + another Goblin.
        .object(
            ObjectSpec::creature(p(1), "Reckless One", 0, 0)
                .with_types(vec![CardType::Creature])
                .with_subtypes(vec![subtype("Goblin"), subtype("Avatar")])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p(1), "P1 Goblin", 1, 1)
                .with_types(vec![CardType::Creature])
                .with_subtypes(vec![subtype("Goblin")])
                .in_zone(ZoneId::Battlefield),
        )
        // Player 2 controls a Goblin.
        .object(
            ObjectSpec::creature(p(2), "P2 Goblin", 1, 1)
                .with_types(vec![CardType::Creature])
                .with_subtypes(vec![subtype("Goblin")])
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let cda_id = find_bf_object(&state, "Reckless One");

    // CDA: P/T = number of Goblins on the battlefield (all players).
    let cda_effect = ContinuousEffect {
        id: EffectId(100),
        source: Some(cda_id),
        timestamp: 100,
        layer: EffectLayer::PtCda,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(cda_id),
        modification: LayerModification::SetPtDynamic {
            power: Box::new(EffectAmount::PermanentCount {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    has_subtype: Some(subtype("Goblin")),
                    ..Default::default()
                },
                controller: PlayerTarget::EachPlayer,
            }),
            toughness: Box::new(EffectAmount::PermanentCount {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    has_subtype: Some(subtype("Goblin")),
                    ..Default::default()
                },
                controller: PlayerTarget::EachPlayer,
            }),
        },
        is_cda: true,
        condition: None,
    };

    let mut state = state;
    state.continuous_effects.push_back(cda_effect);

    let chars = calculate_characteristics(&state, cda_id).unwrap();
    // 3 Goblins total: Reckless One (self), P1's other Goblin, P2's Goblin.
    assert_eq!(
        chars.power,
        Some(3),
        "Multiplayer CDA: counts all Goblins (3) → power 3"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "Multiplayer CDA: toughness also 3"
    );
}

/// CR 604.3, 613.4a: EffectAmount::Sum in CDA.
/// P/T = Elves you control (battlefield) + Elf cards in graveyard.
/// (Abomination of Llanowar pattern.)
#[test]
fn test_cda_with_effect_amount_sum() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        // Abomination itself (is an Elf on battlefield).
        .object(
            ObjectSpec::creature(p(1), "Abomination", 0, 0)
                .with_types(vec![CardType::Creature])
                .with_subtypes(vec![subtype("Elf"), subtype("Horror")])
                .in_zone(ZoneId::Battlefield),
        )
        // Another Elf on battlefield.
        .object(
            ObjectSpec::creature(p(1), "Elf Friend", 1, 1)
                .with_types(vec![CardType::Creature])
                .with_subtypes(vec![subtype("Elf")])
                .in_zone(ZoneId::Battlefield),
        )
        // An Elf card in the graveyard.
        .object(
            ObjectSpec::card(p(1), "Dead Elf")
                .in_zone(ZoneId::Graveyard(p(1)))
                .with_types(vec![CardType::Creature])
                .with_subtypes(vec![subtype("Elf")]),
        )
        .build()
        .unwrap();

    let abom_id = find_bf_object(&state, "Abomination");

    let elf_filter = TargetFilter {
        has_card_type: Some(CardType::Creature),
        has_subtype: Some(subtype("Elf")),
        ..Default::default()
    };

    // CDA: P/T = Sum(PermanentCount(Elves), CardCount(Graveyard Elves)).
    let cda_effect = ContinuousEffect {
        id: EffectId(100),
        source: Some(abom_id),
        timestamp: 100,
        layer: EffectLayer::PtCda,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(abom_id),
        modification: LayerModification::SetPtDynamic {
            power: Box::new(EffectAmount::Sum(
                Box::new(EffectAmount::PermanentCount {
                    filter: elf_filter.clone(),
                    controller: PlayerTarget::Controller,
                }),
                Box::new(EffectAmount::CardCount {
                    zone: ZoneTarget::Graveyard {
                        owner: PlayerTarget::Controller,
                    },
                    player: PlayerTarget::Controller,
                    filter: Some(elf_filter.clone()),
                }),
            )),
            toughness: Box::new(EffectAmount::Sum(
                Box::new(EffectAmount::PermanentCount {
                    filter: elf_filter.clone(),
                    controller: PlayerTarget::Controller,
                }),
                Box::new(EffectAmount::CardCount {
                    zone: ZoneTarget::Graveyard {
                        owner: PlayerTarget::Controller,
                    },
                    player: PlayerTarget::Controller,
                    filter: Some(elf_filter),
                }),
            )),
        },
        is_cda: true,
        condition: None,
    };

    let mut state = state;
    state.continuous_effects.push_back(cda_effect);

    let chars = calculate_characteristics(&state, abom_id).unwrap();
    // Battlefield Elves: Abomination (1) + Elf Friend (1) = 2.
    // Graveyard Elves: Dead Elf (1) = 1.
    // Total Sum: 3.
    assert_eq!(
        chars.power,
        Some(3),
        "Sum CDA: 2 battlefield + 1 graveyard Elves = 3 power"
    );
    assert_eq!(chars.toughness, Some(3), "Sum CDA: toughness also 3");
}

/// CR 604.3, 613.4a: CDA with CardCount (cards in hand).
/// (Psychosis Crawler pattern: P/T = cards in hand.)
#[test]
fn test_cda_card_count_in_hand() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "Hand Crawler", 0, 0)
                .with_types(vec![CardType::Creature, CardType::Artifact])
                .in_zone(ZoneId::Battlefield),
        )
        // 3 cards in hand.
        .object(ObjectSpec::card(p(1), "Hand Card 1").in_zone(ZoneId::Hand(p(1))))
        .object(ObjectSpec::card(p(1), "Hand Card 2").in_zone(ZoneId::Hand(p(1))))
        .object(ObjectSpec::card(p(1), "Hand Card 3").in_zone(ZoneId::Hand(p(1))))
        .build()
        .unwrap();

    let cda_id = find_bf_object(&state, "Hand Crawler");

    let cda_effect = ContinuousEffect {
        id: EffectId(100),
        source: Some(cda_id),
        timestamp: 100,
        layer: EffectLayer::PtCda,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(cda_id),
        modification: LayerModification::SetPtDynamic {
            power: Box::new(EffectAmount::CardCount {
                zone: ZoneTarget::Hand {
                    owner: PlayerTarget::Controller,
                },
                player: PlayerTarget::Controller,
                filter: None,
            }),
            toughness: Box::new(EffectAmount::CardCount {
                zone: ZoneTarget::Hand {
                    owner: PlayerTarget::Controller,
                },
                player: PlayerTarget::Controller,
                filter: None,
            }),
        },
        is_cda: true,
        condition: None,
    };

    let mut state = state;
    state.continuous_effects.push_back(cda_effect);

    let chars = calculate_characteristics(&state, cda_id).unwrap();
    // 3 cards in hand → P/T = 3/3.
    assert_eq!(chars.power, Some(3), "Hand CDA: 3 cards in hand → power 3");
    assert_eq!(chars.toughness, Some(3), "Hand CDA: toughness 3");
}
