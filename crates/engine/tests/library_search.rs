//! Tests for SearchLibrary TargetFilter extensions (CR 701.23, CR 202.3).
//!
//! PB-17: max_cmc, min_cmc, has_card_types (OR) filters on TargetFilter.
//! Validates that SearchLibrary correctly filters by mana value and card type
//! combinations for tutor effects.

use mtg_engine::cards::card_definition::{LibraryPosition, PlayerTarget, ZoneTarget};
use mtg_engine::state::turn::Step;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, GameEvent, GameState, GameStateBuilder, ManaCost, ManaPool, ObjectId, ObjectSpec,
    PlayerId, TargetFilter, TypeLine, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
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

/// Build a sorcery that searches caster's library with given filter and puts to destination.
fn tutor_spell(name: &str, card_id: &str, filter: TargetFilter, to: ZoneTarget) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(card_id.to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Sorcery],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter,
                    reveal: false,
                    destination: to,
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle {
                    player: PlayerTarget::Controller,
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn default_cast(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell {
        player,
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
    }
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("Object '{}' not found", name))
}

fn is_in_zone(state: &GameState, name: &str, zone_check: impl Fn(&ZoneId) -> bool) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && zone_check(&o.zone))
}

// ---------------------------------------------------------------------------
// CR 701.23 / CR 202.3 — max_cmc filter
// ---------------------------------------------------------------------------

#[test]
/// CR 701.23 / CR 202.3 — SearchLibrary with max_cmc filters by mana value
fn test_search_library_max_cmc_finds_matching_card() {
    let tutor = tutor_spell(
        "Low CMC Tutor",
        "low_cmc_tutor",
        TargetFilter {
            has_card_type: Some(CardType::Artifact),
            max_cmc: Some(1),
            ..Default::default()
        },
        ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
    );
    let target_card = CardDefinition {
        name: "Sol Ring".to_string(),
        card_id: CardId("sol_ring_test".to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Artifact],
            ..Default::default()
        },
        ..Default::default()
    };
    let decoy = CardDefinition {
        name: "Hedron Archive".to_string(),
        card_id: CardId("hedron_archive_test".to_string()),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Artifact],
            ..Default::default()
        },
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![tutor, target_card, decoy]);
    let players = [p(1), p(2), p(3), p(4)];

    let spell_spec = ObjectSpec::card(p(1), "Low CMC Tutor")
        .with_card_id(CardId("low_cmc_tutor".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..ManaCost::default()
        })
        .in_zone(ZoneId::Hand(p(1)));
    let target_spec = ObjectSpec::card(p(1), "Sol Ring")
        .with_card_id(CardId("sol_ring_test".to_string()))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..ManaCost::default()
        })
        .in_zone(ZoneId::Library(p(1)));
    let decoy_spec = ObjectSpec::card(p(1), "Hedron Archive")
        .with_card_id(CardId("hedron_archive_test".to_string()))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 4,
            ..ManaCost::default()
        })
        .in_zone(ZoneId::Library(p(1)));

    let state = GameStateBuilder::four_player()
        .with_registry(registry)
        .player_mana(
            p(1),
            ManaPool {
                colorless: 5,
                ..ManaPool::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(spell_spec)
        .object(target_spec)
        .object(decoy_spec)
        .build()
        .unwrap();

    let tutor_id = find_by_name(&state, "Low CMC Tutor");
    let (state, _) = process_command(state, default_cast(p(1), tutor_id)).unwrap();
    let (state, _) = pass_all(state, &players);

    assert!(
        is_in_zone(&state, "Sol Ring", |z| matches!(z, ZoneId::Hand(_))),
        "Sol Ring (CMC 1) should be found and put into hand"
    );
    assert!(
        is_in_zone(&state, "Hedron Archive", |z| matches!(
            z,
            ZoneId::Library(_)
        )),
        "Hedron Archive (CMC 4) should remain in library"
    );
}

// ---------------------------------------------------------------------------
// CR 701.23 / CR 202.3 — min_cmc filter
// ---------------------------------------------------------------------------

#[test]
/// CR 701.23 / CR 202.3 — SearchLibrary with min_cmc filters low-cost cards
fn test_search_library_min_cmc_filters_low_cost_cards() {
    let tutor = tutor_spell(
        "High CMC Tutor",
        "high_cmc_tutor",
        TargetFilter {
            min_cmc: Some(4),
            ..Default::default()
        },
        ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
    );
    let big_card = CardDefinition {
        name: "Colossal Dreadmaw".to_string(),
        card_id: CardId("colossal_dreadmaw_test".to_string()),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 2,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(6),
        toughness: Some(6),
        ..Default::default()
    };
    let small_card = CardDefinition {
        name: "Llanowar Elves".to_string(),
        card_id: CardId("llanowar_elves_test".to_string()),
        mana_cost: Some(ManaCost {
            green: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![tutor, big_card, small_card]);
    let players = [p(1), p(2), p(3), p(4)];

    let state = GameStateBuilder::four_player()
        .with_registry(registry)
        .player_mana(
            p(1),
            ManaPool {
                colorless: 5,
                ..ManaPool::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), "High CMC Tutor")
                .with_card_id(CardId("high_cmc_tutor".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "Colossal Dreadmaw")
                .with_card_id(CardId("colossal_dreadmaw_test".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 4,
                    green: 2,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Library(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "Llanowar Elves")
                .with_card_id(CardId("llanowar_elves_test".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    green: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Library(p(1))),
        )
        .build()
        .unwrap();

    let tutor_id = find_by_name(&state, "High CMC Tutor");
    let (state, _) = process_command(state, default_cast(p(1), tutor_id)).unwrap();
    let (state, _) = pass_all(state, &players);

    assert!(
        is_in_zone(&state, "Colossal Dreadmaw", |z| matches!(
            z,
            ZoneId::Hand(_)
        )),
        "Colossal Dreadmaw (CMC 6) should be found"
    );
    assert!(
        is_in_zone(&state, "Llanowar Elves", |z| matches!(
            z,
            ZoneId::Library(_)
        )),
        "Llanowar Elves (CMC 1) should remain in library"
    );
}

// ---------------------------------------------------------------------------
// CR 701.23 — has_card_types OR semantics
// ---------------------------------------------------------------------------

#[test]
/// CR 701.23 — SearchLibrary with has_card_types OR for "instant or sorcery"
fn test_search_library_has_card_types_or_semantics() {
    let tutor = tutor_spell(
        "Mystical Tutor Test",
        "mystical_tutor_test",
        TargetFilter {
            has_card_types: vec![CardType::Instant, CardType::Sorcery],
            ..Default::default()
        },
        ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Top,
        },
    );
    let instant = CardDefinition {
        name: "Lightning Bolt".to_string(),
        card_id: CardId("lightning_bolt_test".to_string()),
        mana_cost: Some(ManaCost {
            red: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        ..Default::default()
    };
    let creature = CardDefinition {
        name: "Grizzly Bears".to_string(),
        card_id: CardId("grizzly_bears_test".to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![tutor, instant, creature]);
    let players = [p(1), p(2), p(3), p(4)];

    let state = GameStateBuilder::four_player()
        .with_registry(registry)
        .player_mana(
            p(1),
            ManaPool {
                colorless: 5,
                ..ManaPool::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), "Mystical Tutor Test")
                .with_card_id(CardId("mystical_tutor_test".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "Lightning Bolt")
                .with_card_id(CardId("lightning_bolt_test".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    red: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Library(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "Grizzly Bears")
                .with_card_id(CardId("grizzly_bears_test".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    green: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Library(p(1))),
        )
        .build()
        .unwrap();

    let tutor_id = find_by_name(&state, "Mystical Tutor Test");
    let (state, _) = process_command(state, default_cast(p(1), tutor_id)).unwrap();
    let (state, _) = pass_all(state, &players);

    // Both stay in library (Bolt moved to top), creature stays
    assert!(
        is_in_zone(&state, "Lightning Bolt", |z| matches!(
            z,
            ZoneId::Library(_)
        )),
        "Lightning Bolt should be in library (on top)"
    );
    assert!(
        is_in_zone(&state, "Grizzly Bears", |z| matches!(z, ZoneId::Library(_))),
        "Grizzly Bears should remain in library (not matching instant/sorcery filter)"
    );
}

// ---------------------------------------------------------------------------
// CR 701.23 — empty filter (any card)
// ---------------------------------------------------------------------------

#[test]
/// CR 701.23 — Empty filter finds any card (Demonic Tutor pattern)
fn test_search_library_empty_filter_finds_any() {
    let tutor = tutor_spell(
        "Demonic Tutor Test",
        "demonic_tutor_test",
        TargetFilter::default(),
        ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
    );
    let target = CardDefinition {
        name: "Any Card".to_string(),
        card_id: CardId("any_card_test".to_string()),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Enchantment],
            ..Default::default()
        },
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![tutor, target]);
    let players = [p(1), p(2), p(3), p(4)];

    let state = GameStateBuilder::four_player()
        .with_registry(registry)
        .player_mana(
            p(1),
            ManaPool {
                colorless: 5,
                ..ManaPool::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), "Demonic Tutor Test")
                .with_card_id(CardId("demonic_tutor_test".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "Any Card")
                .with_card_id(CardId("any_card_test".to_string()))
                .with_types(vec![CardType::Enchantment])
                .with_mana_cost(ManaCost {
                    generic: 3,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Library(p(1))),
        )
        .build()
        .unwrap();

    let tutor_id = find_by_name(&state, "Demonic Tutor Test");
    let (state, _) = process_command(state, default_cast(p(1), tutor_id)).unwrap();
    let (state, _) = pass_all(state, &players);

    assert!(
        is_in_zone(&state, "Any Card", |z| matches!(z, ZoneId::Hand(_))),
        "Any Card should be found with empty filter"
    );
}

// ---------------------------------------------------------------------------
// CR 701.23 / CR 202.3 — combined filter: creature + max CMC
// ---------------------------------------------------------------------------

#[test]
/// CR 701.23 / CR 202.3 — Combined creature type + max CMC filter
fn test_search_library_combined_creature_max_cmc() {
    let tutor = tutor_spell(
        "Creature CMC Tutor",
        "creature_cmc_tutor",
        TargetFilter {
            has_card_type: Some(CardType::Creature),
            max_cmc: Some(3),
            ..Default::default()
        },
        ZoneTarget::Battlefield { tapped: false },
    );
    let small = CardDefinition {
        name: "Small Creature".to_string(),
        card_id: CardId("small_creature_test".to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };
    let big = CardDefinition {
        name: "Big Creature".to_string(),
        card_id: CardId("big_creature_test".to_string()),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 2,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(6),
        toughness: Some(6),
        ..Default::default()
    };
    let artifact = CardDefinition {
        name: "Low Artifact".to_string(),
        card_id: CardId("low_artifact_test".to_string()),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Artifact],
            ..Default::default()
        },
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![tutor, small, big, artifact]);
    let players = [p(1), p(2), p(3), p(4)];

    let state = GameStateBuilder::four_player()
        .with_registry(registry)
        .player_mana(
            p(1),
            ManaPool {
                colorless: 5,
                ..ManaPool::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), "Creature CMC Tutor")
                .with_card_id(CardId("creature_cmc_tutor".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "Small Creature")
                .with_card_id(CardId("small_creature_test".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    green: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Library(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "Big Creature")
                .with_card_id(CardId("big_creature_test".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 4,
                    green: 2,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Library(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "Low Artifact")
                .with_card_id(CardId("low_artifact_test".to_string()))
                .with_types(vec![CardType::Artifact])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Library(p(1))),
        )
        .build()
        .unwrap();

    let tutor_id = find_by_name(&state, "Creature CMC Tutor");
    let (state, _) = process_command(state, default_cast(p(1), tutor_id)).unwrap();
    let (state, _) = pass_all(state, &players);

    assert!(
        is_in_zone(&state, "Small Creature", |z| *z == ZoneId::Battlefield),
        "Small Creature (CMC 2, creature) should be on battlefield"
    );
    assert!(
        is_in_zone(&state, "Big Creature", |z| matches!(z, ZoneId::Library(_))),
        "Big Creature (CMC 6) should remain in library"
    );
    assert!(
        is_in_zone(&state, "Low Artifact", |z| matches!(z, ZoneId::Library(_))),
        "Low Artifact (not a creature) should remain in library"
    );
}

// ---------------------------------------------------------------------------
// CR 202.3 — No mana cost → MV 0
// ---------------------------------------------------------------------------

#[test]
/// CR 202.3 — Card with no mana cost has mana value 0
fn test_search_library_no_mana_cost_has_mv_zero() {
    let tutor = tutor_spell(
        "Zero CMC Tutor",
        "zero_cmc_tutor",
        TargetFilter {
            max_cmc: Some(0),
            ..Default::default()
        },
        ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
    );
    let land = CardDefinition {
        name: "Island Test".to_string(),
        card_id: CardId("island_test_search".to_string()),
        types: TypeLine {
            card_types: im::ordset![CardType::Land],
            ..Default::default()
        },
        ..Default::default()
    };
    let spell = CardDefinition {
        name: "One Drop".to_string(),
        card_id: CardId("one_drop_test".to_string()),
        mana_cost: Some(ManaCost {
            red: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![tutor, land, spell]);
    let players = [p(1), p(2), p(3), p(4)];

    let state = GameStateBuilder::four_player()
        .with_registry(registry)
        .player_mana(
            p(1),
            ManaPool {
                colorless: 5,
                ..ManaPool::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), "Zero CMC Tutor")
                .with_card_id(CardId("zero_cmc_tutor".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "Island Test")
                .with_card_id(CardId("island_test_search".to_string()))
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Library(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "One Drop")
                .with_card_id(CardId("one_drop_test".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    red: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Library(p(1))),
        )
        .build()
        .unwrap();

    let tutor_id = find_by_name(&state, "Zero CMC Tutor");
    let (state, _) = process_command(state, default_cast(p(1), tutor_id)).unwrap();
    let (state, _) = pass_all(state, &players);

    assert!(
        is_in_zone(&state, "Island Test", |z| matches!(z, ZoneId::Hand(_))),
        "Land (MV 0) should be found"
    );
    assert!(
        is_in_zone(&state, "One Drop", |z| matches!(z, ZoneId::Library(_))),
        "One Drop (MV 1) should remain in library"
    );
}

// ---------------------------------------------------------------------------
// CR 701.23 — No match scenario
// ---------------------------------------------------------------------------

#[test]
/// CR 701.23 — SearchLibrary with no matching card finds nothing
fn test_search_library_no_match_finds_nothing() {
    let tutor = tutor_spell(
        "Enchantment Tutor",
        "enchantment_tutor_test",
        TargetFilter {
            has_card_type: Some(CardType::Enchantment),
            ..Default::default()
        },
        ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
    );
    let creature = CardDefinition {
        name: "Bear".to_string(),
        card_id: CardId("bear_test_search".to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![tutor, creature]);
    let players = [p(1), p(2), p(3), p(4)];

    let state = GameStateBuilder::four_player()
        .with_registry(registry)
        .player_mana(
            p(1),
            ManaPool {
                colorless: 5,
                ..ManaPool::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), "Enchantment Tutor")
                .with_card_id(CardId("enchantment_tutor_test".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "Bear")
                .with_card_id(CardId("bear_test_search".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    green: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Library(p(1))),
        )
        .build()
        .unwrap();

    let tutor_id = find_by_name(&state, "Enchantment Tutor");
    let (state, _) = process_command(state, default_cast(p(1), tutor_id)).unwrap();
    let (state, _) = pass_all(state, &players);

    let hand_count = state
        .objects
        .values()
        .filter(|o| matches!(o.zone, ZoneId::Hand(pid) if pid == p(1)))
        .count();
    assert_eq!(hand_count, 0, "No enchantment found — hand should be empty");
    assert!(
        is_in_zone(&state, "Bear", |z| matches!(z, ZoneId::Library(_))),
        "Bear should remain in library"
    );
}

// ---------------------------------------------------------------------------
// CR 701.23 — Top of library destination (Vampiric Tutor pattern)
// ---------------------------------------------------------------------------

#[test]
/// CR 701.23 — SearchLibrary to top of library
fn test_search_library_to_top_of_library() {
    let tutor = tutor_spell(
        "Top Tutor",
        "top_tutor_test",
        TargetFilter::default(),
        ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Top,
        },
    );
    let target = CardDefinition {
        name: "Prize Card".to_string(),
        card_id: CardId("prize_card_test".to_string()),
        mana_cost: Some(ManaCost {
            generic: 5,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(5),
        toughness: Some(5),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![tutor, target]);
    let players = [p(1), p(2), p(3), p(4)];

    let state = GameStateBuilder::four_player()
        .with_registry(registry)
        .player_mana(
            p(1),
            ManaPool {
                colorless: 5,
                ..ManaPool::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), "Top Tutor")
                .with_card_id(CardId("top_tutor_test".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "Prize Card")
                .with_card_id(CardId("prize_card_test".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 5,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Library(p(1))),
        )
        .build()
        .unwrap();

    let tutor_id = find_by_name(&state, "Top Tutor");
    let (state, _) = process_command(state, default_cast(p(1), tutor_id)).unwrap();
    let (state, _) = pass_all(state, &players);

    assert!(
        is_in_zone(&state, "Prize Card", |z| matches!(z, ZoneId::Library(_))),
        "Prize Card should be in library (on top) after tutor"
    );
}
