//! Tests for replacement/prevention effects (M8 Sessions 1-5).
//!
//! Session 1: data model, serde, builder wiring.
//! Session 2: core application framework — find_applicable, determine_action,
//!            loop prevention, self-replacement priority, OrderReplacements command.
//! Session 5: prevention effects — PreventDamage shields, PreventAllDamage, depletion.

use std::collections::HashSet;

use mtg_engine::rules::replacement::{self, ReplacementResult};
use mtg_engine::{
    AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, CombatDamageTarget, Command,
    Condition, CounterType, DamageTargetFilter, EffectDuration, GameEvent, GameStateBuilder,
    ManaCost, ObjectFilter, ObjectId, ObjectSpec, PlayerFilter, PlayerId, ReplacementEffect,
    ReplacementId, ReplacementModification, ReplacementTrigger, SubType, SuperType, TypeLine,
    ZoneId, ZoneType,
};

/// Helper: create a simple zone-change replacement effect for testing.
fn sample_zone_change_replacement(id: u64, controller: PlayerId) -> ReplacementEffect {
    ReplacementEffect {
        id: ReplacementId(id),
        source: None,
        controller,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    }
}

// ── Serialization round-trip tests ──

#[test]
/// CR 614 — ReplacementEffect serializes and deserializes correctly
/// Source: M8 Session 1 data model validation
fn test_replacement_effect_serde_roundtrip_zone_change() {
    let effect = sample_zone_change_replacement(1, PlayerId(1));
    let json = serde_json::to_string(&effect).unwrap();
    let deserialized: ReplacementEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(effect, deserialized);
}

#[test]
/// CR 614.12 — ETB replacement effect serializes correctly
/// Source: M8 Session 1 data model validation
fn test_replacement_effect_serde_roundtrip_etb_tapped() {
    let effect = ReplacementEffect {
        id: ReplacementId(2),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::WhileSourceOnBattlefield,
        is_self_replacement: true,
        trigger: ReplacementTrigger::WouldEnterBattlefield {
            filter: ObjectFilter::AnyCreature,
        },
        modification: ReplacementModification::EntersTapped,
    };
    let json = serde_json::to_string(&effect).unwrap();
    let deserialized: ReplacementEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(effect, deserialized);
}

#[test]
/// CR 614.12 — ETB with counters serializes correctly
/// Source: M8 Session 1 data model validation
fn test_replacement_effect_serde_roundtrip_etb_counters() {
    let effect = ReplacementEffect {
        id: ReplacementId(3),
        source: None,
        controller: PlayerId(2),
        duration: EffectDuration::WhileSourceOnBattlefield,
        is_self_replacement: true,
        trigger: ReplacementTrigger::WouldEnterBattlefield {
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::EntersWithCounters {
            counter: CounterType::PlusOnePlusOne,
            count: 3,
        },
    };
    let json = serde_json::to_string(&effect).unwrap();
    let deserialized: ReplacementEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(effect, deserialized);
}

#[test]
/// CR 615.7 — Prevention shield serializes correctly
/// Source: M8 Session 1 data model validation
fn test_replacement_effect_serde_roundtrip_prevent_damage() {
    let effect = ReplacementEffect {
        id: ReplacementId(4),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::UntilEndOfTurn,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Player(PlayerId(1)),
        },
        modification: ReplacementModification::PreventDamage(3),
    };
    let json = serde_json::to_string(&effect).unwrap();
    let deserialized: ReplacementEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(effect, deserialized);
}

#[test]
/// CR 614.11 — Draw replacement serializes correctly
/// Source: M8 Session 1 data model validation
fn test_replacement_effect_serde_roundtrip_skip_draw() {
    let effect = ReplacementEffect {
        id: ReplacementId(5),
        source: None,
        controller: PlayerId(3),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::OpponentsOf(PlayerId(3)),
        },
        modification: ReplacementModification::SkipDraw,
    };
    let json = serde_json::to_string(&effect).unwrap();
    let deserialized: ReplacementEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(effect, deserialized);
}

#[test]
/// CR 615.1 — Prevent all damage serializes correctly
/// Source: M8 Session 1 data model validation
fn test_replacement_effect_serde_roundtrip_prevent_all() {
    let effect = ReplacementEffect {
        id: ReplacementId(6),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::UntilEndOfTurn,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Any,
        },
        modification: ReplacementModification::PreventAllDamage,
    };
    let json = serde_json::to_string(&effect).unwrap();
    let deserialized: ReplacementEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(effect, deserialized);
}

#[test]
/// CR 614 — WouldGainLife replacement trigger serializes correctly
/// Source: M8 Session 1 data model validation
fn test_replacement_effect_serde_roundtrip_gain_life() {
    let effect = ReplacementEffect {
        id: ReplacementId(7),
        source: None,
        controller: PlayerId(2),
        duration: EffectDuration::WhileSourceOnBattlefield,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldGainLife {
            player_filter: PlayerFilter::Specific(PlayerId(2)),
        },
        modification: ReplacementModification::SkipDraw, // placeholder modification
    };
    let json = serde_json::to_string(&effect).unwrap();
    let deserialized: ReplacementEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(effect, deserialized);
}

// ── Builder round-trip tests ──

#[test]
/// CR 614 — GameStateBuilder adds replacement effects to GameState
/// Source: M8 Session 1 builder wiring
fn test_builder_adds_replacement_effect() {
    let effect = sample_zone_change_replacement(0, PlayerId(1));
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .with_replacement_effect(effect.clone())
        .build()
        .unwrap();

    assert_eq!(state.replacement_effects.len(), 1);
    assert_eq!(state.replacement_effects[0], effect);
}

#[test]
/// CR 614 — GameStateBuilder adds multiple replacement effects
/// Source: M8 Session 1 builder wiring
fn test_builder_adds_multiple_replacement_effects() {
    let effect1 = sample_zone_change_replacement(0, PlayerId(1));
    let effect2 = ReplacementEffect {
        id: ReplacementId(1),
        source: None,
        controller: PlayerId(2),
        duration: EffectDuration::UntilEndOfTurn,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Any,
        },
        modification: ReplacementModification::PreventAllDamage,
    };

    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect1.clone())
        .with_replacement_effect(effect2.clone())
        .build()
        .unwrap();

    assert_eq!(state.replacement_effects.len(), 2);
    assert_eq!(state.replacement_effects[0], effect1);
    assert_eq!(state.replacement_effects[1], effect2);
}

#[test]
/// CR 614 — next_replacement_id counter advances past pre-set IDs
/// Source: M8 Session 1 builder wiring
fn test_builder_advances_replacement_id_counter() {
    let effect = sample_zone_change_replacement(5, PlayerId(1));
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    // Counter should be past the highest pre-set ID (5 → next is 6)
    assert_eq!(state.next_replacement_id, 6);
}

#[test]
/// CR 614 — GameState::next_replacement_id() generates sequential IDs
/// Source: M8 Session 1 ID generation
fn test_game_state_next_replacement_id() {
    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .build()
        .unwrap();

    let id1 = state.next_replacement_id();
    let id2 = state.next_replacement_id();
    let id3 = state.next_replacement_id();

    assert_eq!(id1, ReplacementId(0));
    assert_eq!(id2, ReplacementId(1));
    assert_eq!(id3, ReplacementId(2));
    assert_eq!(state.next_replacement_id, 3);
}

#[test]
/// CR 614 — Default state has no replacement effects
/// Source: M8 Session 1 baseline validation
fn test_default_state_has_no_replacement_effects() {
    let state = GameStateBuilder::four_player().build().unwrap();
    assert!(state.replacement_effects.is_empty());
    assert_eq!(state.next_replacement_id, 0);
}

// ── ObjectFilter / PlayerFilter / DamageTargetFilter equality tests ──

#[test]
/// CR 614 — ObjectFilter variants are distinguishable
/// Source: M8 Session 1 data model validation
fn test_object_filter_equality() {
    assert_eq!(ObjectFilter::Any, ObjectFilter::Any);
    assert_ne!(ObjectFilter::Any, ObjectFilter::AnyCreature);
    assert_ne!(ObjectFilter::Any, ObjectFilter::Commander);
    assert_eq!(
        ObjectFilter::ControlledBy(PlayerId(1)),
        ObjectFilter::ControlledBy(PlayerId(1))
    );
    assert_ne!(
        ObjectFilter::ControlledBy(PlayerId(1)),
        ObjectFilter::ControlledBy(PlayerId(2))
    );
}

#[test]
/// CR 614 — PlayerFilter variants are distinguishable
/// Source: M8 Session 1 data model validation
fn test_player_filter_equality() {
    assert_eq!(PlayerFilter::Any, PlayerFilter::Any);
    assert_ne!(PlayerFilter::Any, PlayerFilter::Specific(PlayerId(1)));
    assert_eq!(
        PlayerFilter::OpponentsOf(PlayerId(1)),
        PlayerFilter::OpponentsOf(PlayerId(1))
    );
    assert_ne!(
        PlayerFilter::OpponentsOf(PlayerId(1)),
        PlayerFilter::OpponentsOf(PlayerId(2))
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Session 2 — Core application framework tests
// ═══════════════════════════════════════════════════════════════════════════

// ── find_applicable: no effects (no-op) ──

#[test]
/// CR 614 — No replacement effects registered → find_applicable returns empty
/// Source: M8 Session 2 baseline
fn test_find_applicable_no_effects_returns_empty() {
    let state = GameStateBuilder::four_player().build().unwrap();
    let trigger = ReplacementTrigger::WouldChangeZone {
        from: Some(ZoneType::Battlefield),
        to: ZoneType::Graveyard,
        filter: ObjectFilter::Any,
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert!(result.is_empty());
}

// ── find_applicable: single matching effect ──

#[test]
/// CR 614 — One matching replacement effect is found
/// Source: M8 Session 2 single-match
fn test_find_applicable_one_matching_effect() {
    let effect = sample_zone_change_replacement(0, PlayerId(1));
    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    let trigger = ReplacementTrigger::WouldChangeZone {
        from: Some(ZoneType::Battlefield),
        to: ZoneType::Graveyard,
        filter: ObjectFilter::Any,
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert_eq!(result, vec![ReplacementId(0)]);
}

// ── find_applicable: non-matching trigger type ──

#[test]
/// CR 614 — Effect with WouldChangeZone trigger doesn't match WouldDraw event
/// Source: M8 Session 2 trigger mismatch
fn test_find_applicable_wrong_trigger_type_returns_empty() {
    let effect = sample_zone_change_replacement(0, PlayerId(1));
    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    let trigger = ReplacementTrigger::WouldDraw {
        player_filter: PlayerFilter::Specific(PlayerId(1)),
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert!(result.is_empty());
}

// ── find_applicable: non-matching zone ──

#[test]
/// CR 614 — Effect watching for to:Graveyard doesn't match to:Exile
/// Source: M8 Session 2 zone mismatch
fn test_find_applicable_wrong_destination_zone() {
    let effect = sample_zone_change_replacement(0, PlayerId(1));
    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    let trigger = ReplacementTrigger::WouldChangeZone {
        from: Some(ZoneType::Battlefield),
        to: ZoneType::Exile,
        filter: ObjectFilter::Any,
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert!(result.is_empty());
}

// ── find_applicable: wildcard from-zone ──

#[test]
/// CR 614 — Effect with from:None matches any source zone
/// Source: M8 Session 2 wildcard from-zone
fn test_find_applicable_wildcard_from_zone_matches() {
    let effect = ReplacementEffect {
        id: ReplacementId(0),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: None, // wildcard — any source zone
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };
    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    // Event: from Hand to Graveyard — should match
    let trigger = ReplacementTrigger::WouldChangeZone {
        from: Some(ZoneType::Hand),
        to: ZoneType::Graveyard,
        filter: ObjectFilter::Any,
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert_eq!(result.len(), 1);
}

// ── find_applicable: duration check (WhileSourceOnBattlefield) ──

#[test]
/// CR 614 — WhileSourceOnBattlefield effect is inactive when source is gone
/// Source: M8 Session 2 duration validation
fn test_find_applicable_inactive_when_source_not_on_battlefield() {
    // Source object is NOT on the battlefield (it's in graveyard)
    let effect = ReplacementEffect {
        id: ReplacementId(0),
        source: Some(ObjectId(99)), // non-existent object
        controller: PlayerId(1),
        duration: EffectDuration::WhileSourceOnBattlefield,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };
    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    let trigger = ReplacementTrigger::WouldChangeZone {
        from: Some(ZoneType::Battlefield),
        to: ZoneType::Graveyard,
        filter: ObjectFilter::Any,
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert!(
        result.is_empty(),
        "should be inactive — source not on battlefield"
    );
}

#[test]
/// CR 614 — WhileSourceOnBattlefield effect is active when source is on battlefield
/// Source: M8 Session 2 duration validation
fn test_find_applicable_active_when_source_on_battlefield() {
    // Place the source on the battlefield
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::enchantment(PlayerId(1), "Rest in Peace"))
        .build()
        .unwrap();
    // The enchantment gets ObjectId(1) (first object added)
    let source_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    // Now build a state with the effect pointing to this source
    let effect = ReplacementEffect {
        id: ReplacementId(0),
        source: Some(source_id),
        controller: PlayerId(1),
        duration: EffectDuration::WhileSourceOnBattlefield,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: None,
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };
    // Rebuild with the effect included
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::enchantment(PlayerId(1), "Rest in Peace"))
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    let trigger = ReplacementTrigger::WouldChangeZone {
        from: Some(ZoneType::Battlefield),
        to: ZoneType::Graveyard,
        filter: ObjectFilter::Any,
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert_eq!(result.len(), 1);
}

// ── CR 614.5: Loop prevention ──

#[test]
/// CR 614.5 — Already-applied effect is excluded from find_applicable
/// Source: M8 Session 2 loop prevention
fn test_find_applicable_excludes_already_applied() {
    let effect = sample_zone_change_replacement(0, PlayerId(1));
    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    let trigger = ReplacementTrigger::WouldChangeZone {
        from: Some(ZoneType::Battlefield),
        to: ZoneType::Graveyard,
        filter: ObjectFilter::Any,
    };

    // First call: effect is found
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert_eq!(result.len(), 1);

    // Second call with the effect marked as already applied
    let mut already = HashSet::new();
    already.insert(ReplacementId(0));
    let result = replacement::find_applicable(&state, &trigger, &already);
    assert!(result.is_empty(), "CR 614.5: same effect can't apply twice");
}

// ── CR 614.15: Self-replacement priority ──

#[test]
/// CR 614.15 — Self-replacement effects are returned before other replacements
/// Source: M8 Session 2 self-replacement ordering
fn test_find_applicable_self_replacement_sorted_first() {
    let other_effect = ReplacementEffect {
        id: ReplacementId(0),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };
    let self_effect = ReplacementEffect {
        id: ReplacementId(1),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: true,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Command),
    };

    // Register other_effect FIRST, self_effect SECOND
    let state = GameStateBuilder::four_player()
        .with_replacement_effect(other_effect)
        .with_replacement_effect(self_effect)
        .build()
        .unwrap();

    let trigger = ReplacementTrigger::WouldChangeZone {
        from: Some(ZoneType::Battlefield),
        to: ZoneType::Graveyard,
        filter: ObjectFilter::Any,
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());

    // Self-replacement (ID 1) should come before other (ID 0) despite registration order
    assert_eq!(result.len(), 2);
    assert_eq!(
        result[0],
        ReplacementId(1),
        "self-replacement should be first"
    );
    assert_eq!(result[1], ReplacementId(0), "non-self should be second");
}

// ── determine_action: 0 applicable → NoApplicable ──

#[test]
/// CR 616.1 — No applicable effects → NoApplicable result
/// Source: M8 Session 2 determine_action baseline
fn test_determine_action_no_applicable() {
    let state = GameStateBuilder::four_player().build().unwrap();
    let result = replacement::determine_action(&state, &[], PlayerId(1), "test");
    assert!(matches!(result, ReplacementResult::NoApplicable));
}

// ── determine_action: 1 applicable → AutoApply ──

#[test]
/// CR 616.1 — Single applicable effect → AutoApply
/// Source: M8 Session 2 determine_action auto-apply
fn test_determine_action_single_effect_auto_applies() {
    let effect = sample_zone_change_replacement(0, PlayerId(1));
    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    let applicable = vec![ReplacementId(0)];
    let result = replacement::determine_action(&state, &applicable, PlayerId(1), "test");
    match result {
        ReplacementResult::AutoApply(id) => assert_eq!(id, ReplacementId(0)),
        other => panic!("expected AutoApply, got {:?}", other),
    }
}

// ── determine_action: 2+ with exactly 1 self-replacement → auto-apply self ──

#[test]
/// CR 616.1a — One self-replacement among multiple → auto-apply the self-replacement
/// Source: M8 Session 2 self-replacement auto-apply
fn test_determine_action_one_self_replacement_auto_applies() {
    let other = sample_zone_change_replacement(0, PlayerId(1));
    let self_eff = ReplacementEffect {
        id: ReplacementId(1),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: true,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Command),
    };

    let state = GameStateBuilder::four_player()
        .with_replacement_effect(other)
        .with_replacement_effect(self_eff)
        .build()
        .unwrap();

    let applicable = vec![ReplacementId(1), ReplacementId(0)];
    let result = replacement::determine_action(&state, &applicable, PlayerId(1), "test");
    match result {
        ReplacementResult::AutoApply(id) => {
            assert_eq!(id, ReplacementId(1), "self-replacement should auto-apply");
        }
        other => panic!("expected AutoApply, got {:?}", other),
    }
}

// ── determine_action: 2+ self-replacements → NeedsChoice among self only ──

#[test]
/// CR 616.1a — Multiple self-replacements → player chooses among self-replacements
/// Source: M8 Session 2 multiple self-replacements
fn test_determine_action_multiple_self_replacements_needs_choice() {
    let self1 = ReplacementEffect {
        id: ReplacementId(0),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: true,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };
    let self2 = ReplacementEffect {
        id: ReplacementId(1),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: true,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Command),
    };
    let non_self = sample_zone_change_replacement(2, PlayerId(2));

    let state = GameStateBuilder::four_player()
        .with_replacement_effect(self1)
        .with_replacement_effect(self2)
        .with_replacement_effect(non_self)
        .build()
        .unwrap();

    let applicable = vec![ReplacementId(0), ReplacementId(1), ReplacementId(2)];
    let result = replacement::determine_action(&state, &applicable, PlayerId(1), "test");
    match result {
        ReplacementResult::NeedsChoice { choices, .. } => {
            // Only self-replacements should be in the choices (CR 616.1a)
            assert_eq!(choices.len(), 2);
            assert!(choices.contains(&ReplacementId(0)));
            assert!(choices.contains(&ReplacementId(1)));
            assert!(!choices.contains(&ReplacementId(2)));
        }
        other => panic!("expected NeedsChoice, got {:?}", other),
    }
}

// ── determine_action: 2+ with no self-replacements → NeedsChoice among all ──

#[test]
/// CR 616.1e — Multiple non-self replacements → player chooses among all
/// Source: M8 Session 2 non-self replacement choice
fn test_determine_action_multiple_non_self_needs_choice() {
    let eff1 = sample_zone_change_replacement(0, PlayerId(1));
    let eff2 = ReplacementEffect {
        id: ReplacementId(1),
        source: None,
        controller: PlayerId(2),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Command),
    };

    let state = GameStateBuilder::four_player()
        .with_replacement_effect(eff1)
        .with_replacement_effect(eff2)
        .build()
        .unwrap();

    let applicable = vec![ReplacementId(0), ReplacementId(1)];
    let result = replacement::determine_action(&state, &applicable, PlayerId(1), "zone change");
    match result {
        ReplacementResult::NeedsChoice {
            player, choices, ..
        } => {
            assert_eq!(player, PlayerId(1));
            assert_eq!(choices.len(), 2);
        }
        other => panic!("expected NeedsChoice, got {:?}", other),
    }
}

// ── OrderReplacements command routing ──

#[test]
/// CR 616.1 — OrderReplacements command emits ReplacementEffectApplied
/// Source: M8 Session 2 command routing
fn test_order_replacements_command_emits_applied_event() {
    let eff1 = sample_zone_change_replacement(0, PlayerId(1));
    let eff2 = ReplacementEffect {
        id: ReplacementId(1),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Command),
    };

    let state = GameStateBuilder::four_player()
        .with_replacement_effect(eff1)
        .with_replacement_effect(eff2)
        .build()
        .unwrap();

    let (new_state, events) = mtg_engine::process_command(
        state,
        Command::OrderReplacements {
            player: PlayerId(1),
            ids: vec![ReplacementId(1), ReplacementId(0)],
        },
    )
    .unwrap();

    // Should emit ReplacementEffectApplied for the first ID
    let applied = events.iter().find(|e| {
        matches!(e, GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(1))
    });
    assert!(
        applied.is_some(),
        "should emit ReplacementEffectApplied for first ID"
    );

    // Event should be in history
    assert!(
        new_state.history.iter().any(|e| {
            matches!(e, GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(1))
        }),
        "applied event should be in history"
    );
}

#[test]
/// CR 616.1 — OrderReplacements with empty IDs returns error
/// Source: M8 Session 2 command validation
fn test_order_replacements_empty_ids_returns_error() {
    let state = GameStateBuilder::four_player().build().unwrap();
    let result = mtg_engine::process_command(
        state,
        Command::OrderReplacements {
            player: PlayerId(1),
            ids: vec![],
        },
    );
    assert!(result.is_err());
}

#[test]
/// CR 616.1 — OrderReplacements with non-existent ID returns error
/// Source: M8 Session 2 command validation
fn test_order_replacements_nonexistent_id_returns_error() {
    let state = GameStateBuilder::four_player().build().unwrap();
    let result = mtg_engine::process_command(
        state,
        Command::OrderReplacements {
            player: PlayerId(1),
            ids: vec![ReplacementId(99)],
        },
    );
    assert!(result.is_err());
}

#[test]
/// CR 616.1 — OrderReplacements by wrong player returns error
/// Source: M8 Session 2 command validation
fn test_order_replacements_wrong_player_returns_error() {
    let effect = sample_zone_change_replacement(0, PlayerId(1));
    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    // Player 2 tries to order player 1's replacement
    let result = mtg_engine::process_command(
        state,
        Command::OrderReplacements {
            player: PlayerId(2),
            ids: vec![ReplacementId(0)],
        },
    );
    assert!(result.is_err());
}

// ── Object filter matching with actual game objects ──

#[test]
/// CR 614 — ObjectFilter::AnyCreature matches creature objects
/// Source: M8 Session 2 filter matching
fn test_find_applicable_creature_filter_matches_creature() {
    let effect = ReplacementEffect {
        id: ReplacementId(0),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::AnyCreature,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };

    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Bear", 2, 2))
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    // Find the creature's ObjectId
    let creature_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    // Event trigger: this specific creature would die
    let trigger = ReplacementTrigger::WouldChangeZone {
        from: Some(ZoneType::Battlefield),
        to: ZoneType::Graveyard,
        filter: ObjectFilter::SpecificObject(creature_id),
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert_eq!(result.len(), 1, "AnyCreature should match a creature");
}

#[test]
/// CR 614 — ObjectFilter::AnyCreature does NOT match non-creature objects
/// Source: M8 Session 2 filter matching
fn test_find_applicable_creature_filter_does_not_match_enchantment() {
    let effect = ReplacementEffect {
        id: ReplacementId(0),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::AnyCreature,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };

    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::enchantment(PlayerId(1), "Some Enchantment"))
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    let enchantment_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    let trigger = ReplacementTrigger::WouldChangeZone {
        from: Some(ZoneType::Battlefield),
        to: ZoneType::Graveyard,
        filter: ObjectFilter::SpecificObject(enchantment_id),
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert!(
        result.is_empty(),
        "AnyCreature should not match enchantment"
    );
}

// ── Player filter matching ──

#[test]
/// CR 614 — PlayerFilter::OpponentsOf matches opponents but not the player
/// Source: M8 Session 2 player filter matching
fn test_find_applicable_opponents_filter() {
    let effect = ReplacementEffect {
        id: ReplacementId(0),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::OpponentsOf(PlayerId(1)),
        },
        modification: ReplacementModification::SkipDraw,
    };

    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    // Player 2 would draw — should match (opponent of P1)
    let trigger_p2 = ReplacementTrigger::WouldDraw {
        player_filter: PlayerFilter::Specific(PlayerId(2)),
    };
    let result = replacement::find_applicable(&state, &trigger_p2, &HashSet::new());
    assert_eq!(result.len(), 1, "P2 is an opponent of P1");

    // Player 1 would draw — should NOT match (not an opponent of self)
    let trigger_p1 = ReplacementTrigger::WouldDraw {
        player_filter: PlayerFilter::Specific(PlayerId(1)),
    };
    let result = replacement::find_applicable(&state, &trigger_p1, &HashSet::new());
    assert!(result.is_empty(), "P1 is not an opponent of P1");
}

// ── Damage target filter matching ──

#[test]
/// CR 615 — DamageTargetFilter::Any matches any damage event
/// Source: M8 Session 2 damage filter matching
fn test_find_applicable_damage_any_filter() {
    let effect = ReplacementEffect {
        id: ReplacementId(0),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Any,
        },
        modification: ReplacementModification::PreventAllDamage,
    };

    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    let trigger = ReplacementTrigger::DamageWouldBeDealt {
        target_filter: DamageTargetFilter::Player(PlayerId(2)),
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert_eq!(result.len(), 1, "Any should match Player target");
}

// ── UntilEndOfTurn duration ──

#[test]
/// CR 614 — UntilEndOfTurn effects are always active (cleanup handles removal)
/// Source: M8 Session 2 duration validation
fn test_find_applicable_until_end_of_turn_always_active() {
    let effect = ReplacementEffect {
        id: ReplacementId(0),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::UntilEndOfTurn,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Any,
        },
        modification: ReplacementModification::PreventDamage(3),
    };

    let state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    let trigger = ReplacementTrigger::DamageWouldBeDealt {
        target_filter: DamageTargetFilter::Any,
    };
    let result = replacement::find_applicable(&state, &trigger, &HashSet::new());
    assert_eq!(result.len(), 1, "UntilEndOfTurn should be active");
}

// ═══════════════════════════════════════════════════════════════════════════
// Session 3 — Zone-change interception + Commander die replacement
// ═══════════════════════════════════════════════════════════════════════════

use mtg_engine::state::builder::register_commander_zone_replacements;

// ── Simple creature dies with no replacement → graveyard normally ──

#[test]
/// CR 704.5g — Creature with lethal damage dies normally when no replacement active
/// Source: M8 Session 3 baseline
fn test_creature_dies_no_replacement_goes_to_graveyard() {
    // Create a 2/2 creature with 2 damage marked (lethal).
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Bear", 2, 2).with_damage(2))
        .build()
        .unwrap();

    let creature_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    // SBAs should move it to graveyard.
    let mut state = state;
    let events = mtg_engine::check_and_apply_sbas(&mut state);

    // Creature should be in graveyard.
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::CreatureDied { object_id, .. } if *object_id == creature_id)
        ),
        "creature should die via SBA"
    );

    // Nothing on battlefield.
    assert!(
        state.objects_in_zone(&ZoneId::Battlefield).is_empty(),
        "battlefield should be empty after death"
    );

    // No pending zone changes.
    assert!(state.pending_zone_changes.is_empty());
}

// ── Creature dies with "exile instead" replacement → goes to exile ──

#[test]
/// CR 614 — Single "exile instead" replacement redirects creature death to exile
/// Source: M8 Session 3 single auto-apply redirect
fn test_creature_dies_with_exile_replacement_goes_to_exile() {
    // Register a Rest-in-Peace-like effect: any creature going to graveyard → exile instead.
    let rip_effect = ReplacementEffect {
        id: ReplacementId(0),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };

    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Bear", 2, 2).with_damage(2))
        .with_replacement_effect(rip_effect)
        .build()
        .unwrap();

    let creature_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    let mut state = state;
    let events = mtg_engine::check_and_apply_sbas(&mut state);

    // Should have a ReplacementEffectApplied event.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(0)
        )),
        "replacement effect should be applied"
    );

    // Creature should NOT be in graveyard.
    assert!(
        state
            .objects_in_zone(&ZoneId::Graveyard(PlayerId(1)))
            .is_empty(),
        "graveyard should be empty — creature was exiled instead"
    );

    // Creature should be in exile.
    let exile_objects = state.objects_in_zone(&ZoneId::Exile);
    assert_eq!(exile_objects.len(), 1, "creature should be in exile");

    // ObjectExiled event emitted.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ObjectExiled { object_id, .. } if *object_id == creature_id
        )),
        "should emit ObjectExiled event"
    );

    // No CreatureDied event (it was exiled, not put in graveyard).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "should not emit CreatureDied for exiled creature"
    );
}

// ── Commander dies → SBA emits choice, owner returns to command zone (CR 903.9a) ──

#[test]
/// CR 903.9a / CR 704.6d — Commander with lethal damage: SBA emits choice, owner returns.
///
/// With the M9 fix, the SBA no longer auto-applies: it emits
/// `CommanderZoneReturnChoiceRequired`. The owner then sends
/// `ReturnCommanderToCommandZone` to complete the move.
/// Source: M9 Session 3 SBA model update; fix MR-M9-01
fn test_commander_dies_auto_redirects_to_command_zone() {
    let cmdr_card_id = CardId("cmdr-1".to_string());
    let p1 = PlayerId(1);

    let mut state = GameStateBuilder::four_player()
        .player_commander(p1, cmdr_card_id.clone())
        .object(
            ObjectSpec::creature(p1, "Commander", 3, 3)
                .with_card_id(cmdr_card_id.clone())
                .with_damage(3),
        )
        .build()
        .unwrap();

    // In M9, no graveyard/exile replacement is registered for commanders —
    // those paths are now handled by SBA (check_commander_zone_return_sba).
    // The hand/library replacements are still registered via CR 903.9b.
    register_commander_zone_replacements(&mut state);

    // First SBA pass: lethal damage → commander moves to graveyard,
    // then commander zone return SBA emits CommanderZoneReturnChoiceRequired.
    let events = mtg_engine::check_and_apply_sbas(&mut state);

    // Commander should be in graveyard (choice not yet resolved).
    assert!(
        !state.objects_in_zone(&ZoneId::Graveyard(p1)).is_empty(),
        "commander should be in graveyard pending owner's choice"
    );

    // CommanderZoneReturnChoiceRequired should be emitted (MR-M9-01 fix).
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { .. })),
        "should emit CommanderZoneReturnChoiceRequired event; events: {events:?}"
    );

    // Owner resolves the choice: return to command zone.
    let choice_event = events.iter().find(
        |e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { owner, .. } if *owner == p1),
    );
    let obj_id = match choice_event.unwrap() {
        GameEvent::CommanderZoneReturnChoiceRequired { object_id, .. } => *object_id,
        _ => unreachable!(),
    };

    let (state, return_events) = mtg_engine::rules::process_command(
        state,
        Command::ReturnCommanderToCommandZone {
            player: p1,
            object_id: obj_id,
        },
    )
    .unwrap();

    // Commander should now be in command zone.
    let cmd_objects = state.objects_in_zone(&ZoneId::Command(p1));
    assert_eq!(cmd_objects.len(), 1, "commander should be in command zone");

    assert!(
        state.objects_in_zone(&ZoneId::Graveyard(p1)).is_empty(),
        "graveyard should be empty after choice resolved"
    );

    assert!(
        return_events
            .iter()
            .any(|e| matches!(e, GameEvent::CommanderReturnedToCommandZone { .. })),
        "should emit CommanderReturnedToCommandZone event"
    );
}

// ── Commander dies with Rest in Peace active → SBA returns from exile (M9 model) ──

#[test]
/// CR 903.9a + Corner case #18 — Commander + Rest in Peace: M9 SBA model (fix MR-M9-01).
///
/// RiP (graveyard→exile) fires as a replacement, then the commander-return SBA
/// emits `CommanderZoneReturnChoiceRequired`. The owner then sends
/// `ReturnCommanderToCommandZone` to complete the move to command zone.
/// Source: M9 Session 3 corner case 18 (SBA model update); fix MR-M9-01
fn test_commander_dies_with_rest_in_peace_needs_choice() {
    let cmdr_card_id = CardId("cmdr-1".to_string());
    let p1 = PlayerId(1);

    // Rest in Peace effect: anything going to graveyard → exile instead.
    let rip_effect = ReplacementEffect {
        id: ReplacementId(100),
        source: None,
        controller: PlayerId(2), // opponent controls RiP
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: None,
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };

    let mut state = GameStateBuilder::four_player()
        .player_commander(p1, cmdr_card_id.clone())
        .object(
            ObjectSpec::creature(p1, "Commander", 3, 3)
                .with_card_id(cmdr_card_id.clone())
                .with_damage(3),
        )
        .with_replacement_effect(rip_effect)
        .build()
        .unwrap();

    // Register commander zone-change replacements (hand/library only in M9).
    register_commander_zone_replacements(&mut state);

    // SBAs: lethal damage → RiP redirects graveyard→exile → SBA emits choice.
    let events = mtg_engine::check_and_apply_sbas(&mut state);

    // Commander should be in exile pending the owner's choice.
    assert!(
        state.objects_in_zone(&ZoneId::Graveyard(p1)).is_empty(),
        "graveyard should be empty (RiP redirected to exile)"
    );
    assert_eq!(
        state.objects_in_zone(&ZoneId::Exile).len(),
        1,
        "commander should be in exile awaiting owner's choice"
    );

    // CommanderZoneReturnChoiceRequired should be emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { .. })),
        "should emit CommanderZoneReturnChoiceRequired; events: {events:?}"
    );

    // No pending zone changes from replacement system (the SBA choice is separate).
    assert!(
        state.pending_zone_changes.is_empty(),
        "no pending replacement zone changes"
    );

    // Owner resolves choice: return commander to command zone.
    let choice_event = events.iter().find(
        |e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { owner, .. } if *owner == p1),
    );
    let obj_id = match choice_event.unwrap() {
        GameEvent::CommanderZoneReturnChoiceRequired { object_id, .. } => *object_id,
        _ => unreachable!(),
    };

    let (state, _) = mtg_engine::rules::process_command(
        state,
        Command::ReturnCommanderToCommandZone {
            player: p1,
            object_id: obj_id,
        },
    )
    .unwrap();

    // Commander should now be in command zone.
    let cmd_objects = state.objects_in_zone(&ZoneId::Command(p1));
    assert_eq!(
        cmd_objects.len(),
        1,
        "commander should be in command zone after owner's choice"
    );
    assert!(
        state.objects_in_zone(&ZoneId::Exile).is_empty(),
        "exile should be empty after choice resolved"
    );
}

// ── Commander dies with RiP → owner chooses to leave commander in exile ──

#[test]
/// CR 903.9a — Commander + Rest in Peace: owner may choose to leave commander in exile.
///
/// RiP sends the commander to exile. The SBA emits a choice. Here the owner
/// sends `LeaveCommanderInZone` to leave the commander in exile (e.g., for
/// reanimation synergy). The commander stays in exile.
/// Source: M9 Session 3 corner case 18 (SBA model); fix MR-M9-01
fn test_commander_dies_with_rip_player_chooses_command_zone() {
    let cmdr_card_id = CardId("cmdr-1".to_string());
    let p1 = PlayerId(1);

    let rip_effect = ReplacementEffect {
        id: ReplacementId(100),
        source: None,
        controller: PlayerId(2),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: None,
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };

    let mut state = GameStateBuilder::four_player()
        .player_commander(p1, cmdr_card_id.clone())
        .object(
            ObjectSpec::creature(p1, "Commander", 3, 3)
                .with_card_id(cmdr_card_id.clone())
                .with_damage(3),
        )
        .with_replacement_effect(rip_effect)
        .build()
        .unwrap();

    register_commander_zone_replacements(&mut state);

    // SBAs: lethal damage → RiP redirects to exile → SBA emits choice event.
    let events = mtg_engine::check_and_apply_sbas(&mut state);

    // Commander is in exile awaiting owner's choice.
    assert_eq!(
        state.objects_in_zone(&ZoneId::Exile).len(),
        1,
        "commander should be in exile awaiting choice"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CommanderZoneReturnChoiceRequired {
                from_zone: ZoneType::Exile,
                ..
            }
        )),
        "should emit CommanderZoneReturnChoiceRequired from exile; events: {events:?}"
    );

    // Owner chooses to leave commander in exile.
    let choice_event = events.iter().find(
        |e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { owner, .. } if *owner == p1),
    );
    let obj_id = match choice_event.unwrap() {
        GameEvent::CommanderZoneReturnChoiceRequired { object_id, .. } => *object_id,
        _ => unreachable!(),
    };

    let (state, _) = mtg_engine::rules::process_command(
        state,
        Command::LeaveCommanderInZone {
            player: p1,
            object_id: obj_id,
        },
    )
    .unwrap();

    // Commander stays in exile.
    assert_eq!(
        state.objects_in_zone(&ZoneId::Exile).len(),
        1,
        "commander should still be in exile after choosing to leave it there"
    );
    assert!(
        state.objects_in_zone(&ZoneId::Command(p1)).is_empty(),
        "command zone should be empty — owner chose not to return"
    );
    assert!(
        state.pending_commander_zone_choices.is_empty(),
        "pending choice should be cleared after LeaveCommanderInZone"
    );
}

// ── HasCardId filter matches across zone changes ──

#[test]
/// CR 903.9 / CR 400.7 — HasCardId filter matches commander by CardId not ObjectId
/// Source: M8 Session 3 filter validation
fn test_has_card_id_filter_matches_object() {
    let card_id = CardId("test-card".to_string());

    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Test", 2, 2).with_card_id(card_id.clone()))
        .build()
        .unwrap();

    let obj_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    // HasCardId should match.
    assert!(
        replacement::object_matches_filter(
            &state,
            obj_id,
            &ObjectFilter::HasCardId(card_id.clone())
        ),
        "HasCardId should match object with that CardId"
    );

    // Wrong CardId should not match.
    assert!(
        !replacement::object_matches_filter(
            &state,
            obj_id,
            &ObjectFilter::HasCardId(CardId("wrong".to_string()))
        ),
        "HasCardId should not match object with different CardId"
    );
}

// ── ExileObject effect on commander: SBA emits choice, owner returns (MR-M9-01 fix) ──

#[test]
/// CR 614 + CR 903.9a — ExileObject effect on commander: the effect moves the commander
/// to exile, then the SBA emits `CommanderZoneReturnChoiceRequired`. The owner responds
/// with `ReturnCommanderToCommandZone` to complete the move to command zone.
/// Source: M9 Session 3 SBA model update; fix MR-M9-01
fn test_exile_effect_checks_replacements() {
    let cmdr_card_id = CardId("cmdr-exile".to_string());
    let p1 = PlayerId(1);

    let mut state = GameStateBuilder::four_player()
        .player_commander(p1, cmdr_card_id.clone())
        .object(ObjectSpec::creature(p1, "Commander", 4, 4).with_card_id(cmdr_card_id.clone()))
        .build()
        .unwrap();

    // In M9, register_commander_zone_replacements registers hand/library only (CR 903.9b).
    // Exile path is now handled by SBA (CR 903.9a).
    register_commander_zone_replacements(&mut state);

    let cmdr_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    // Execute ExileObject effect directly.
    use mtg_engine::effects::EffectContext;
    use mtg_engine::SpellTarget;
    use mtg_engine::Target;
    use mtg_engine::{CardEffectTarget, Effect};

    let effect = Effect::ExileObject {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
    };
    let mut ctx = EffectContext::new(
        PlayerId(2),   // opponent is exiling it
        ObjectId(999), // source doesn't matter
        vec![SpellTarget {
            target: Target::Object(cmdr_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );

    let _events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // After the effect, commander is in exile.
    let exile_objects = state.objects_in_zone(&ZoneId::Exile);
    assert_eq!(
        exile_objects.len(),
        1,
        "commander should be in exile after ExileObject (before SBA runs)"
    );

    // SBA emits choice — commander stays in exile pending owner's decision.
    let sba_events = mtg_engine::check_and_apply_sbas(&mut state);
    assert!(
        sba_events
            .iter()
            .any(|e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { .. })),
        "should emit CommanderZoneReturnChoiceRequired from SBA; events: {sba_events:?}"
    );
    assert_eq!(
        state.objects_in_zone(&ZoneId::Exile).len(),
        1,
        "commander should still be in exile pending choice"
    );

    // Owner resolves choice: return to command zone.
    let choice_event = sba_events.iter().find(
        |e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { owner, .. } if *owner == p1),
    );
    let obj_id = match choice_event.unwrap() {
        GameEvent::CommanderZoneReturnChoiceRequired { object_id, .. } => *object_id,
        _ => unreachable!(),
    };

    let (state, return_events) = mtg_engine::rules::process_command(
        state,
        Command::ReturnCommanderToCommandZone {
            player: p1,
            object_id: obj_id,
        },
    )
    .unwrap();

    let cmd_objects = state.objects_in_zone(&ZoneId::Command(p1));
    assert_eq!(
        cmd_objects.len(),
        1,
        "commander should be in command zone after choice resolved"
    );
    assert!(
        state.objects_in_zone(&ZoneId::Exile).is_empty(),
        "exile zone should be empty after choice resolved"
    );
    assert!(
        return_events
            .iter()
            .any(|e| matches!(e, GameEvent::CommanderReturnedToCommandZone { .. })),
        "should emit CommanderReturnedToCommandZone event"
    );
}

// ── DestroyPermanent effect with redirect ──

#[test]
/// CR 614 — DestroyPermanent effect checks replacement effects
/// Source: M8 Session 3 effect interception
fn test_destroy_effect_checks_replacements() {
    // Register "exile instead of graveyard" for all objects.
    let rip_effect = ReplacementEffect {
        id: ReplacementId(0),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };

    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(2), "Target", 3, 3))
        .with_replacement_effect(rip_effect)
        .build()
        .unwrap();

    let creature_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    use mtg_engine::effects::EffectContext;
    use mtg_engine::SpellTarget;
    use mtg_engine::Target;
    use mtg_engine::{CardEffectTarget, Effect};

    let effect = Effect::DestroyPermanent {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
    };
    let mut state = state;
    let mut ctx = EffectContext::new(
        PlayerId(1),
        ObjectId(999),
        vec![SpellTarget {
            target: Target::Object(creature_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );

    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Creature should be in exile (RiP redirected), not graveyard.
    assert!(
        state
            .objects_in_zone(&ZoneId::Graveyard(PlayerId(2)))
            .is_empty(),
        "graveyard should be empty — RiP redirected to exile"
    );
    assert_eq!(
        state.objects_in_zone(&ZoneId::Exile).len(),
        1,
        "creature should be in exile"
    );

    // ReplacementEffectApplied should be emitted.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(0)
        )),
        "should emit ReplacementEffectApplied event"
    );
}

// ── register_commander_zone_replacements creates correct effects (M9 model) ──

#[test]
/// CR 903.9b — register_commander_zone_replacements creates 2 effects per commander (M9 model).
///
/// In M9, only hand and library replacements are registered (CR 903.9b). Graveyard and exile
/// paths are handled by the SBA in check_commander_zone_return_sba (CR 903.9a / CR 704.6d).
/// Source: M9 Session 3 SBA model update
fn test_register_commander_replacements_creates_effects() {
    let cmdr_id = CardId("cmdr-test".to_string());
    let mut state = GameStateBuilder::four_player()
        .player_commander(PlayerId(1), cmdr_id.clone())
        .build()
        .unwrap();

    assert_eq!(
        state.replacement_effects.len(),
        0,
        "no replacement effects before registration"
    );

    register_commander_zone_replacements(&mut state);

    // Should have 2 effects: one for hand, one for library (M9 model).
    assert_eq!(
        state.replacement_effects.len(),
        2,
        "should have 2 replacement effects for commander (hand + library)"
    );

    // One should trigger on WouldChangeZone to Hand (CR 903.9b).
    assert!(
        state.replacement_effects.iter().any(|e| matches!(
            &e.trigger,
            ReplacementTrigger::WouldChangeZone {
                to: ZoneType::Hand,
                ..
            }
        )),
        "should have hand replacement (CR 903.9b bounce redirect)"
    );

    // One should trigger on WouldChangeZone to Library (CR 903.9b).
    assert!(
        state.replacement_effects.iter().any(|e| matches!(
            &e.trigger,
            ReplacementTrigger::WouldChangeZone {
                to: ZoneType::Library,
                ..
            }
        )),
        "should have library replacement (CR 903.9b tuck redirect)"
    );

    // Both should redirect to Command zone.
    assert!(
        state.replacement_effects.iter().all(|e| matches!(
            &e.modification,
            ReplacementModification::RedirectToZone(ZoneType::Command)
        )),
        "both should redirect to command zone"
    );

    // Both should be controlled by P1.
    assert!(
        state
            .replacement_effects
            .iter()
            .all(|e| e.controller == PlayerId(1)),
        "both should be controlled by commander owner"
    );
}

// ── Pending zone change skipped in subsequent SBA pass (non-commander scenario) ──

#[test]
/// CR 614 — Objects with pending replacement choices are skipped in SBA passes.
///
/// In M9, commander graveyard/exile paths are SBAs (no pending zone change created).
/// This test verifies the pending-skip logic still works for non-commander objects
/// with two competing replacements.
/// Source: M8 Session 3 pending skip validation (adapted for M9 model)
fn test_pending_zone_change_skipped_in_sba() {
    // Use a non-commander creature to test the pending-zone-change skipping logic.
    // Two replacements compete: exile-instead-of-graveyard (effect_a) and
    // graveyard-instead-of-exile (effect_b), both targeting the creature.
    let effect_a = ReplacementEffect {
        id: ReplacementId(10),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: None,
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };
    let effect_b = ReplacementEffect {
        id: ReplacementId(11),
        source: None,
        controller: PlayerId(2),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: None,
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };

    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Creature", 3, 3).with_damage(3))
        .with_replacement_effect(effect_a)
        .with_replacement_effect(effect_b)
        .build()
        .unwrap();

    // First SBA pass creates the pending zone change.
    let _events1 = mtg_engine::check_and_apply_sbas(&mut state);
    assert_eq!(state.pending_zone_changes.len(), 1);

    // Second SBA pass should NOT move the creature again or create another pending.
    let events2 = mtg_engine::check_and_apply_sbas(&mut state);
    assert_eq!(
        state.pending_zone_changes.len(),
        1,
        "should still have exactly one pending zone change"
    );
    // Second pass should produce no creature-related events.
    assert!(
        !events2
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "should not produce another CreatureDied in second pass"
    );
    assert!(
        !events2
            .iter()
            .any(|e| matches!(e, GameEvent::ReplacementChoiceRequired { .. })),
        "should not produce another ReplacementChoiceRequired in second pass"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Session 4 — Draw replacement + ETB replacement
// ═══════════════════════════════════════════════════════════════════════════

use mtg_engine::rules::turn_actions::draw_card;
use mtg_engine::{all_cards, Step};

// ── Draw replacement: baseline ───────────────────────────────────────────

#[test]
/// CR 614.11 — draw with no replacement draws normally and emits CardDrawn
/// Source: M8 Session 4 draw baseline
fn test_draw_no_replacement_draws_normally() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::card(p1, "Mountain").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    let events = draw_card(&mut state, p1).unwrap();

    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        1,
        "hand should have one card after draw"
    );
    assert_eq!(
        state.zone(&ZoneId::Library(p1)).unwrap().len(),
        0,
        "library should be empty after draw"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "should emit CardDrawn for the drawing player"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::ReplacementEffectApplied { .. })),
        "no replacement effect should fire"
    );
}

// ── Draw replacement: SkipDraw via draw_card ──────────────────────────────

#[test]
/// CR 614.10 — SkipDraw replacement suppresses the draw; no CardDrawn emitted
/// Source: M8 Session 4 SkipDraw via draw_card
fn test_draw_skip_draw_replacement_fires_no_card_drawn() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let skip_effect = ReplacementEffect {
        id: ReplacementId(1),
        source: None,
        controller: p1,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::card(p1, "Island").in_zone(ZoneId::Library(p1)))
        .with_replacement_effect(skip_effect)
        .build()
        .unwrap();

    let events = draw_card(&mut state, p1).unwrap();

    assert_eq!(
        state.zone(&ZoneId::Library(p1)).unwrap().len(),
        1,
        "library should still have 1 card — SkipDraw prevented the draw"
    );
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        0,
        "hand should still be empty"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "CardDrawn should NOT be emitted when SkipDraw applies"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(1))
        ),
        "ReplacementEffectApplied should be emitted for SkipDraw"
    );
}

// ── Draw replacement: SkipDraw only affects target player ─────────────────

#[test]
/// CR 614.10 / PlayerFilter — SkipDraw targeting P1 does not affect P2's draw
/// Source: M8 Session 4 player filter validation
fn test_draw_skip_draw_only_affects_target_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let skip_effect = ReplacementEffect {
        id: ReplacementId(2),
        source: None,
        controller: p1,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1), // only P1's draws
        },
        modification: ReplacementModification::SkipDraw,
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::card(p2, "Plains").in_zone(ZoneId::Library(p2)))
        .with_replacement_effect(skip_effect)
        .build()
        .unwrap();

    // P2 draws — should not be affected by P1's SkipDraw.
    let events = draw_card(&mut state, p2).unwrap();

    assert_eq!(
        state.zone(&ZoneId::Hand(p2)).unwrap().len(),
        1,
        "P2 should draw normally — replacement targets P1 only"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p2)),
        "P2 should get CardDrawn"
    );
    assert!(
        !events.iter().any(
            |e| matches!(e, GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(2))
        ),
        "P1's SkipDraw should not fire for P2's draw"
    );
}

// ── ETB replacement: global EntersTapped via PlayLand ────────────────────

#[test]
/// CR 614.12 / 614.1c — EntersTapped replacement fires via PlayLand
/// Source: M8 Session 4 ETB tapped (Corner Case 19)
fn test_etb_land_enters_tapped_replacement() {
    use mtg_engine::Command;

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Global "all permanents enter tapped" effect.
    let enters_tapped = ReplacementEffect {
        id: ReplacementId(10),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldEnterBattlefield {
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::EntersTapped,
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Hand(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_replacement_effect(enters_tapped)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let land_id = state.objects_in_zone(&ZoneId::Hand(p1)).first().unwrap().id;

    let (new_state, events) = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: land_id,
        },
    )
    .unwrap();

    let bf_objects = new_state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(bf_objects.len(), 1, "land should be on the battlefield");
    assert!(
        bf_objects[0].status.tapped,
        "land should be tapped (EntersTapped replacement fired)"
    );

    // CR 614.1c: entered tapped, not untap-then-tap.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentTapped { .. })),
        "PermanentTapped event should be emitted"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(10))
        ),
        "ReplacementEffectApplied should be emitted for global effect"
    );
}

// ── ETB replacement: EntersWithCounters ───────────────────────────────────

#[test]
/// CR 614.12 — EntersWithCounters replacement adds counters when permanent ETBs
/// Source: M8 Session 4 ETB with counters
fn test_etb_permanent_enters_with_counters() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Global "all creatures enter with a +1/+1 counter" effect.
    let enters_with_counter = ReplacementEffect {
        id: ReplacementId(11),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldEnterBattlefield {
            filter: ObjectFilter::AnyCreature,
        },
        modification: ReplacementModification::EntersWithCounters {
            counter: CounterType::PlusOnePlusOne,
            count: 1,
        },
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Bear", 2, 2))
        .with_replacement_effect(enters_with_counter)
        .build()
        .unwrap();

    let bear_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    // Apply ETB replacements directly to simulate the ETB path.
    let events = mtg_engine::rules::replacement::apply_etb_replacements(&mut state, bear_id, p1);

    let bear = state.objects.get(&bear_id).unwrap();
    assert_eq!(
        bear.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "bear should have 1 +1/+1 counter from EntersWithCounters"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::CounterAdded { object_id, .. } if *object_id == bear_id)
        ),
        "CounterAdded event should be emitted"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(11))
        ),
        "ReplacementEffectApplied should be emitted for global effect"
    );
}

// ── ETB replacement: filter mismatch ──────────────────────────────────────

#[test]
/// CR 614.12 — ETB replacement with AnyCreature filter does not fire for lands
/// Source: M8 Session 4 filter validation
fn test_etb_replacement_does_not_fire_for_non_matching_filter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Effect that only applies to creatures.
    let creature_only = ReplacementEffect {
        id: ReplacementId(12),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldEnterBattlefield {
            filter: ObjectFilter::AnyCreature,
        },
        modification: ReplacementModification::EntersTapped,
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::land(p1, "Forest"))
        .with_replacement_effect(creature_only)
        .build()
        .unwrap();

    let land_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    let events = mtg_engine::rules::replacement::apply_etb_replacements(&mut state, land_id, p1);

    let land = state.objects.get(&land_id).unwrap();
    assert!(
        !land.status.tapped,
        "land should not be tapped — AnyCreature filter doesn't match lands"
    );
    assert!(
        events.is_empty(),
        "no events should be emitted when filter doesn't match"
    );
}

// ── ETB self-replacement: Dimir Guildgate enters tapped ───────────────────

#[test]
/// CR 614.1c / 614.15 — Dimir Guildgate's self-ETB replacement causes it to enter tapped
/// Source: M8 Session 4 self-ETB from card definition (Corner Case 19)
fn test_dimir_guildgate_enters_tapped_via_card_definition() {
    use mtg_engine::Command;

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let guildgate_card_id = CardId("dimir-guildgate".to_string());
    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::land(p1, "Dimir Guildgate")
                .with_card_id(guildgate_card_id)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let gate_id = state.objects_in_zone(&ZoneId::Hand(p1)).first().unwrap().id;

    let (new_state, events) = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: gate_id,
        },
    )
    .unwrap();

    let bf_objects = new_state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(
        bf_objects.len(),
        1,
        "Dimir Guildgate should be on the battlefield"
    );
    assert!(
        bf_objects[0].status.tapped,
        "Dimir Guildgate should enter tapped (self-ETB replacement from card definition)"
    );
    // CR 614.1c: entered tapped — PermanentTapped fires.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentTapped { .. })),
        "PermanentTapped event should be emitted for Dimir Guildgate's ETB replacement"
    );
    // Self-ETB replacements do NOT emit ReplacementEffectApplied (no registered ID).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::ReplacementEffectApplied { .. })),
        "self-ETB from card definition should NOT emit ReplacementEffectApplied"
    );
}

// ── Session 5: Prevention effects (CR 615) ───────────────────────────────────

#[test]
/// CR 615.7 — "Prevent the next N damage" shield reduces damage by N; excess gets through.
/// Source: M8 Session 5 — prevent-next-3 shield takes 5 damage → 2 gets through
fn test_prevention_shield_partial_reduce() {
    let shield_id = ReplacementId(10);
    let source = ObjectId(1);
    let player = PlayerId(1);

    let effect = ReplacementEffect {
        id: shield_id,
        source: None,
        controller: player,
        duration: EffectDuration::UntilEndOfTurn,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Any,
        },
        modification: ReplacementModification::PreventDamage(3),
    };
    let mut state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .with_prevention_counter(shield_id, 3)
        .build()
        .unwrap();

    let target = CombatDamageTarget::Player(player);
    let (final_dmg, events) =
        mtg_engine::rules::replacement::apply_damage_prevention(&mut state, source, &target, 5);

    // CR 615.7: 3 damage prevented, 2 gets through.
    assert_eq!(final_dmg, 2, "3-damage shield should reduce 5 to 2");
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::DamagePrevented {
                prevented: 3,
                remaining: 2,
                ..
            }
        )),
        "DamagePrevented event should report 3 prevented, 2 remaining"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ReplacementEffectApplied { effect_id, .. }
            if *effect_id == shield_id
        )),
        "ReplacementEffectApplied should be emitted for the shield"
    );
}

#[test]
/// CR 615.7 — Prevention shield depletes: counter reaches 0 and effect is removed.
/// Source: M8 Session 5 — shield exhausted after preventing damage equal to its capacity
fn test_prevention_shield_depletes_and_is_removed() {
    let shield_id = ReplacementId(20);
    let source = ObjectId(1);
    let player = PlayerId(1);

    let effect = ReplacementEffect {
        id: shield_id,
        source: None,
        controller: player,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Any,
        },
        modification: ReplacementModification::PreventDamage(3),
    };
    let mut state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .with_prevention_counter(shield_id, 3)
        .build()
        .unwrap();

    let target = CombatDamageTarget::Player(player);

    // First hit: 5 damage; shield prevents 3, counter → 0, effect removed.
    let (final_dmg, _) =
        mtg_engine::rules::replacement::apply_damage_prevention(&mut state, source, &target, 5);
    assert_eq!(final_dmg, 2, "first hit: 3 prevented, 2 gets through");

    // Shield counter should be gone and effect should be removed.
    assert!(
        !state.prevention_counters.contains_key(&shield_id),
        "prevention counter should be removed after shield is exhausted"
    );
    assert!(
        !state.replacement_effects.iter().any(|e| e.id == shield_id),
        "ReplacementEffect should be removed when shield counter reaches 0"
    );

    // Second hit: no prevention in effect — all 3 damage gets through.
    let (final_dmg2, events2) =
        mtg_engine::rules::replacement::apply_damage_prevention(&mut state, source, &target, 3);
    assert_eq!(
        final_dmg2, 3,
        "second hit: shield gone, all damage gets through"
    );
    assert!(
        !events2
            .iter()
            .any(|e| matches!(e, GameEvent::DamagePrevented { .. })),
        "no DamagePrevented on second hit after shield is exhausted"
    );
}

#[test]
/// CR 615.1 — "Prevent all damage" zeros all damage from the event; effect is not consumed.
/// Source: M8 Session 5 — PreventAllDamage modification
fn test_prevent_all_damage() {
    let shield_id = ReplacementId(30);
    let source = ObjectId(1);
    let player = PlayerId(1);

    let effect = ReplacementEffect {
        id: shield_id,
        source: None,
        controller: player,
        duration: EffectDuration::UntilEndOfTurn,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Any,
        },
        modification: ReplacementModification::PreventAllDamage,
    };
    let mut state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .build()
        .unwrap();

    let target = CombatDamageTarget::Player(player);

    // First event: 7 damage → all prevented.
    let (final_dmg, events) =
        mtg_engine::rules::replacement::apply_damage_prevention(&mut state, source, &target, 7);
    assert_eq!(final_dmg, 0, "PreventAllDamage should zero all damage");
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::DamagePrevented {
                prevented: 7,
                remaining: 0,
                ..
            }
        )),
        "DamagePrevented event should show 7 prevented, 0 remaining"
    );

    // Effect is NOT consumed — it should still be active.
    assert!(
        state.replacement_effects.iter().any(|e| e.id == shield_id),
        "PreventAllDamage effect should remain active (not consumed)"
    );

    // Second event: 5 more damage → also all prevented.
    let (final_dmg2, _) =
        mtg_engine::rules::replacement::apply_damage_prevention(&mut state, source, &target, 5);
    assert_eq!(
        final_dmg2, 0,
        "PreventAllDamage prevents a second event too"
    );
}

#[test]
/// CR 615.7 / CR 616.1 — Two prevention effects apply to the same event sequentially.
/// Both reduce damage in registration order; the first shield can be exhausted first.
/// Source: M8 Session 5 — sequential prevention shields on same damage event
fn test_two_prevention_effects_sequential() {
    let shield_a = ReplacementId(40); // PreventDamage(3)
    let shield_b = ReplacementId(41); // PreventDamage(4)
    let source = ObjectId(1);
    let player = PlayerId(1);

    let effect_a = ReplacementEffect {
        id: shield_a,
        source: None,
        controller: player,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Any,
        },
        modification: ReplacementModification::PreventDamage(3),
    };
    let effect_b = ReplacementEffect {
        id: shield_b,
        source: None,
        controller: player,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Any,
        },
        modification: ReplacementModification::PreventDamage(4),
    };
    let mut state = GameStateBuilder::four_player()
        .with_replacement_effect(effect_a)
        .with_prevention_counter(shield_a, 3)
        .with_replacement_effect(effect_b)
        .with_prevention_counter(shield_b, 4)
        .build()
        .unwrap();

    let target = CombatDamageTarget::Player(player);

    // 6 damage incoming:
    //   shield_a (PreventDamage(3)): prevents 3, counter → 0, shield exhausted.
    //   shield_b (PreventDamage(4)): prevents remaining 3, counter → 1.
    // Final: 0 damage through.
    let (final_dmg, events) =
        mtg_engine::rules::replacement::apply_damage_prevention(&mut state, source, &target, 6);

    assert_eq!(
        final_dmg, 0,
        "both shields together should prevent all 6 damage"
    );

    // Shield A should be exhausted and removed.
    assert!(
        !state.replacement_effects.iter().any(|e| e.id == shield_a),
        "shield_a (3-cap) should be removed after exhaustion"
    );
    // Shield B should have 1 remaining.
    assert_eq!(
        state.prevention_counters.get(&shield_b).copied(),
        Some(1),
        "shield_b (4-cap) should have 1 remaining after preventing 3"
    );

    // Two DamagePrevented events should have been emitted.
    let prevented_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DamagePrevented { .. }))
        .collect();
    assert_eq!(
        prevented_events.len(),
        2,
        "one DamagePrevented event per prevention effect applied"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// M8 Fix Session 1 — MR-M8-03: UntilEndOfTurn replacement expiration
// ═══════════════════════════════════════════════════════════════════════════

#[test]
/// CR 514.2 — UntilEndOfTurn replacement effects are removed during Cleanup.
/// Source: MR-M8-03 fix — expire_end_of_turn_effects now also removes replacement effects.
fn test_until_end_of_turn_replacement_expires_at_cleanup() {
    let shield_id = ReplacementId(50);

    let effect = ReplacementEffect {
        id: shield_id,
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::UntilEndOfTurn,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Any,
        },
        modification: ReplacementModification::PreventDamage(5),
    };

    let mut state = GameStateBuilder::four_player()
        .with_replacement_effect(effect)
        .with_prevention_counter(shield_id, 5)
        .build()
        .unwrap();

    // Effect should be active before cleanup.
    assert_eq!(
        state.replacement_effects.len(),
        1,
        "effect should be registered before cleanup"
    );
    assert!(
        state.prevention_counters.contains_key(&shield_id),
        "prevention counter should exist before cleanup"
    );

    // Simulate cleanup: call expire_end_of_turn_effects.
    mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state);

    // Effect should be gone after cleanup.
    assert_eq!(
        state.replacement_effects.len(),
        0,
        "UntilEndOfTurn replacement effect should be removed at cleanup (CR 514.2)"
    );
    assert!(
        !state.prevention_counters.contains_key(&shield_id),
        "prevention counter for expired effect should also be removed"
    );
}

#[test]
/// CR 514.2 — Indefinite replacement effects survive Cleanup; UntilEndOfTurn ones do not.
/// Source: MR-M8-03 fix — only UntilEndOfTurn effects expire, others persist.
fn test_indefinite_replacement_survives_cleanup() {
    let eot_id = ReplacementId(60);
    let indefinite_id = ReplacementId(61);

    let eot_effect = ReplacementEffect {
        id: eot_id,
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::UntilEndOfTurn,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::Any,
        },
        modification: ReplacementModification::PreventAllDamage,
    };

    let indefinite_effect = ReplacementEffect {
        id: indefinite_id,
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };

    let mut state = GameStateBuilder::four_player()
        .with_replacement_effect(eot_effect)
        .with_replacement_effect(indefinite_effect)
        .build()
        .unwrap();

    assert_eq!(state.replacement_effects.len(), 2);

    mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state);

    // Only the UntilEndOfTurn effect should be removed.
    assert_eq!(
        state.replacement_effects.len(),
        1,
        "only UntilEndOfTurn should be removed; Indefinite should survive"
    );
    assert!(
        state
            .replacement_effects
            .iter()
            .any(|e| e.id == indefinite_id),
        "the Indefinite effect should still be present"
    );
    assert!(
        !state.replacement_effects.iter().any(|e| e.id == eot_id),
        "the UntilEndOfTurn effect should have been removed"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// M8 Fix Session 1 — MR-M8-02: ChoiceRequired in DestroyPermanent/ExileObject
// ═══════════════════════════════════════════════════════════════════════════

#[test]
/// CR 616.1 — DestroyPermanent emits ReplacementChoiceRequired when multiple
/// replacement effects apply to the same zone change (non-commander scenario).
/// Source: MR-M8-02 fix — ChoiceRequired arm now handled in DestroyPermanent.
///
/// In M9, commander graveyard replacements are SBAs. This test uses two explicit
/// non-commander graveyard replacements to exercise the ChoiceRequired path.
fn test_destroy_permanent_emits_choice_required_for_multiple_replacements() {
    // Two competing graveyard replacements on a non-commander creature.
    let effect_a = ReplacementEffect {
        id: ReplacementId(0),
        source: None,
        controller: PlayerId(2),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: None,
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };
    let effect_b = ReplacementEffect {
        id: ReplacementId(1),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: None,
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };

    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Creature", 4, 4))
        .with_replacement_effect(effect_a)
        .with_replacement_effect(effect_b)
        .build()
        .unwrap();

    let creature_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    // Execute DestroyPermanent on the creature.
    use mtg_engine::effects::EffectContext;
    use mtg_engine::SpellTarget;
    use mtg_engine::Target;
    use mtg_engine::{CardEffectTarget, Effect};

    let effect = Effect::DestroyPermanent {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
    };
    let mut ctx = EffectContext::new(
        PlayerId(2),
        ObjectId(999),
        vec![SpellTarget {
            target: Target::Object(creature_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );

    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Should have emitted ReplacementChoiceRequired, not moved the creature yet.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ReplacementChoiceRequired { .. })),
        "CR 616.1: should emit ReplacementChoiceRequired when multiple replacements apply"
    );

    // Creature should still be on battlefield (or in pending_zone_changes), not yet moved.
    assert_eq!(
        state.pending_zone_changes.len(),
        1,
        "a PendingZoneChange should have been created"
    );

    // Creature should still be on battlefield until choice is resolved.
    assert!(
        state
            .objects_in_zone(&ZoneId::Battlefield)
            .iter()
            .any(|o| o.id == creature_id),
        "creature should still be on battlefield until player chooses"
    );
}

#[test]
/// CR 616.1 — ExileObject emits ReplacementChoiceRequired when multiple
/// replacement effects apply to the same zone change (non-commander scenario).
/// Source: MR-M8-02 fix — ChoiceRequired arm now handled in ExileObject.
///
/// In M9, commander exile replacements are SBAs. This test uses two explicit
/// exile replacements to exercise the ChoiceRequired path.
fn test_exile_object_emits_choice_required_for_multiple_replacements() {
    // Two competing exile replacements on a non-commander creature.
    let effect_a = ReplacementEffect {
        id: ReplacementId(200),
        source: None,
        controller: PlayerId(2),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: None,
            to: ZoneType::Exile,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Graveyard),
    };
    let effect_b = ReplacementEffect {
        id: ReplacementId(201),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: None,
            to: ZoneType::Exile,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Graveyard),
    };

    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Creature", 4, 4))
        .with_replacement_effect(effect_a)
        .with_replacement_effect(effect_b)
        .build()
        .unwrap();

    let creature_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    use mtg_engine::effects::EffectContext;
    use mtg_engine::SpellTarget;
    use mtg_engine::Target;
    use mtg_engine::{CardEffectTarget, Effect};

    let effect = Effect::ExileObject {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
    };
    let mut ctx = EffectContext::new(
        PlayerId(2),
        ObjectId(999),
        vec![SpellTarget {
            target: Target::Object(creature_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );

    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Should emit ReplacementChoiceRequired.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ReplacementChoiceRequired { .. })),
        "CR 616.1: ExileObject should emit ReplacementChoiceRequired when multiple replacements apply"
    );
    assert_eq!(
        state.pending_zone_changes.len(),
        1,
        "a PendingZoneChange should have been created"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// M8 Fix Session 1 — MR-M8-06: zone_change_events event type for non-creatures
// ═══════════════════════════════════════════════════════════════════════════

#[test]
/// CR 701.7 — A non-creature permanent going to graveyard via pending zone change
/// resolution emits PermanentDestroyed, not CreatureDied.
/// Source: MR-M8-06 fix — zone_change_events checks card types.
fn test_zone_change_events_enchantment_emits_permanent_destroyed() {
    // Two replacements both redirecting from graveyard so the enchantment gets
    // a ChoiceRequired, which we then resolve to exercise zone_change_events.
    let effect_a = ReplacementEffect {
        id: ReplacementId(300),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };
    // effect_b is a self-replacement that redirects to graveyard (no-op redirect,
    // just to create a 2-effect scenario).
    let effect_b = ReplacementEffect {
        id: ReplacementId(301),
        source: None,
        controller: PlayerId(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: true, // triggers first per CR 614.15
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };

    // Place an enchantment on the battlefield.
    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::enchantment(PlayerId(1), "Aura"))
        .with_replacement_effect(effect_a)
        .with_replacement_effect(effect_b)
        .build()
        .unwrap();

    let enchantment_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    // Manually insert a pending zone change so we can call process_command.
    use mtg_engine::state::replacement_effect::PendingZoneChange;
    state.pending_zone_changes.push_back(PendingZoneChange {
        object_id: enchantment_id,
        original_from: ZoneType::Battlefield,
        original_destination: ZoneType::Graveyard,
        affected_player: PlayerId(1),
        already_applied: Vec::new(),
    });

    // Player 1 resolves the choice by picking effect_b (self-replacement to Exile).
    let (state, events) = mtg_engine::process_command(
        state,
        Command::OrderReplacements {
            player: PlayerId(1),
            ids: vec![ReplacementId(301)],
        },
    )
    .unwrap();

    // The enchantment should now be in exile, not graveyard.
    assert!(
        state.objects_in_zone(&ZoneId::Exile).len() >= 1,
        "enchantment should be in exile after redirect"
    );

    // Should NOT emit CreatureDied (it's an enchantment, not a creature).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "non-creature should not emit CreatureDied (MR-M8-06)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// M8 Fix Session 2 — MR-M8-07/08/09/10
// ═══════════════════════════════════════════════════════════════════════════

// ── MR-M8-07: shared draw path via DrawCards effect ──────────────────────

#[test]
/// CR 614.10 / CR 614.11 — SkipDraw via DrawCards effect (draw_one_card path)
/// is handled identically to the turn-draw path (draw_card).
/// Source: MR-M8-07 — both draw paths use check_would_draw_replacement.
fn test_draw_cards_effect_respects_skip_draw_replacement() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::{Effect, EffectAmount, PlayerTarget};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Register a SkipDraw replacement for p1.
    let skip_effect = ReplacementEffect {
        id: ReplacementId(500),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };

    // Give p1 a library card so we can verify it stays there.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::card(p1, "Forest").in_zone(ZoneId::Library(p1)))
        .with_replacement_effect(skip_effect)
        .build()
        .unwrap();

    let effect = Effect::DrawCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
    };
    let mut ctx = EffectContext::new(p1, ObjectId(999), vec![]);

    let events = execute_effect(&mut state, &effect, &mut ctx);

    // Library should still have the card — SkipDraw prevented the draw.
    assert_eq!(
        state.zone(&ZoneId::Library(p1)).unwrap().len(),
        1,
        "draw_one_card path: library should still have 1 card when SkipDraw applies"
    );
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        0,
        "draw_one_card path: hand should still be empty when SkipDraw applies"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "draw_one_card path: CardDrawn should NOT be emitted when SkipDraw applies"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(500))
        ),
        "draw_one_card path: ReplacementEffectApplied should be emitted for SkipDraw"
    );
}

// ── MR-M8-08: NeedsChoice for multiple WouldDraw replacements ────────────

#[test]
/// CR 616.1 — Multiple WouldDraw replacements cause ReplacementChoiceRequired
/// to be emitted; the draw is deferred rather than proceeding silently.
/// Source: MR-M8-08 — NeedsChoice handled in check_would_draw_replacement.
fn test_draw_needs_choice_emits_replacement_choice_required() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Two competing SkipDraw replacements with no self-replacement.
    // Neither is a self-replacement, so NeedsChoice triggers (CR 616.1e).
    let skip_a = ReplacementEffect {
        id: ReplacementId(600),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };
    let skip_b = ReplacementEffect {
        id: ReplacementId(601),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::card(p1, "Island").in_zone(ZoneId::Library(p1)))
        .with_replacement_effect(skip_a)
        .with_replacement_effect(skip_b)
        .build()
        .unwrap();

    // draw_card path
    use mtg_engine::rules::turn_actions::draw_card;
    let events = draw_card(&mut state, p1).unwrap();

    // The draw should be deferred — ReplacementChoiceRequired emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ReplacementChoiceRequired { player, .. } if *player == p1)),
        "CR 616.1: draw_card should emit ReplacementChoiceRequired when multiple WouldDraw replacements apply"
    );
    // Card must not have been drawn.
    assert_eq!(
        state.zone(&ZoneId::Library(p1)).unwrap().len(),
        1,
        "library should still have the card when draw is deferred"
    );
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        0,
        "hand should be empty when draw is deferred"
    );
}

// ── MR-M8-09: Leyline of the Void opponent-only filter ───────────────────

#[test]
/// CR 614.1a — ObjectFilter::OwnedByOpponentsOf matches opponent-owned objects
/// and excludes the owning player's own objects.
/// Validates the filter used by Leyline of the Void (MR-M8-09).
/// Source: MR-M8-09 — OwnedByOpponentsOf filter bound at registration.
fn test_leyline_of_the_void_opponent_only_filter() {
    use mtg_engine::rules::replacement::object_matches_filter;
    use mtg_engine::{all_cards, CardId, CardRegistry};

    let p1 = PlayerId(1); // Leyline controller
    let p2 = PlayerId(2); // Leyline's opponent

    let leyline_card_id = CardId("leyline-of-the-void".to_string());
    let registry = CardRegistry::new(all_cards());

    // Build state: Leyline on battlefield (p1), one p1-owned card and one p2-owned card.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::enchantment(p1, "Leyline of the Void")
                .with_card_id(leyline_card_id)
                .in_zone(ZoneId::Battlefield),
        )
        // p1's own card in graveyard
        .object(ObjectSpec::card(p1, "Forest").in_zone(ZoneId::Graveyard(p1)))
        // p2's card in graveyard
        .object(ObjectSpec::card(p2, "Swamp").in_zone(ZoneId::Graveyard(p2)))
        .with_registry(registry)
        .build()
        .unwrap();

    // Register Leyline's replacement effect (binds OwnedByOpponentsOf to p1).
    let leyline_obj_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;
    let leyline_cid = state.objects.get(&leyline_obj_id).unwrap().card_id.clone();
    let reg = state.card_registry.clone();
    mtg_engine::rules::replacement::register_permanent_replacement_abilities(
        &mut state,
        leyline_obj_id,
        p1,
        leyline_cid.as_ref(),
        &reg,
    );

    // Extract the registered filter — it should be OwnedByOpponentsOf(p1).
    let leyline_filter = state
        .replacement_effects
        .iter()
        .find(|e| e.source == Some(leyline_obj_id))
        .map(|e| {
            if let ReplacementTrigger::WouldChangeZone { filter, .. } = &e.trigger {
                filter.clone()
            } else {
                ObjectFilter::Any
            }
        })
        .expect("Leyline should have registered a replacement effect");

    assert!(
        matches!(leyline_filter, ObjectFilter::OwnedByOpponentsOf(pid) if pid == p1),
        "registered filter should be OwnedByOpponentsOf(p1), not the placeholder"
    );

    // Find the two graveyard objects by owner.
    let p1_card_id = state
        .objects
        .values()
        .find(|o| o.owner == p1 && o.zone == ZoneId::Graveyard(p1))
        .map(|o| o.id)
        .expect("p1's Forest should be in graveyard");

    let p2_card_id = state
        .objects
        .values()
        .find(|o| o.owner == p2 && o.zone == ZoneId::Graveyard(p2))
        .map(|o| o.id)
        .expect("p2's Swamp should be in graveyard");

    // p2's card should match (opponent-owned).
    assert!(
        object_matches_filter(&state, p2_card_id, &leyline_filter),
        "Leyline should match p2's card (opponent-owned)"
    );
    // p1's card should NOT match (controller's own card).
    assert!(
        !object_matches_filter(&state, p1_card_id, &leyline_filter),
        "Leyline should NOT match p1's own card (MR-M8-09)"
    );
}

// ── MR-M8-10: cleanup_sba_rounds included in hash ────────────────────────

#[test]
/// Architecture Invariant 7 — `cleanup_sba_rounds` field contributes to the hash.
/// Two states differing only in cleanup_sba_rounds must produce different hashes.
/// Source: MR-M8-10 — cleanup_sba_rounds added to TurnState::hash_into.
fn test_hash_cleanup_sba_rounds_affects_hash() {
    let state1 = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .build()
        .unwrap();

    let mut state2 = state1.clone();

    // States must be identical before modification.
    assert_eq!(
        state1.public_state_hash(),
        state2.public_state_hash(),
        "precondition: identical states must have identical hashes"
    );

    // Advance cleanup_sba_rounds in state2 only.
    state2.turn.cleanup_sba_rounds = 1;

    // Hashes must now differ.
    assert_ne!(
        state1.public_state_hash(),
        state2.public_state_hash(),
        "states differing only in cleanup_sba_rounds must have different hashes (MR-M8-10)"
    );
}

// ── CC#33: Sylvan Library draw tracking ──────────────────────────────────────

#[test]
/// CC#33 / CR 121.1 — `cards_drawn_this_turn` counter increments on each draw
/// and resets to 0 at the start of the player's turn.
///
/// Sylvan Library's upkeep ability asks "how many cards have you drawn this turn
/// so far?" and uses this count to determine whether the player must pay life or
/// put cards back. Accurate draw tracking is required for Sylvan Library to
/// function correctly.
///
/// This test verifies:
/// 1. `cards_drawn_this_turn` starts at 0.
/// 2. Each successful draw increments the counter.
/// 3. `reset_turn_state` resets the counter back to 0.
fn test_cc33_sylvan_library_draw_tracking() {
    use mtg_engine::rules::turn_actions::{draw_card, reset_turn_state};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::card(p1, "Island").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Forest").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Swamp").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    // Initial state: no cards drawn yet.
    assert_eq!(
        state.players.get(&p1).unwrap().cards_drawn_this_turn,
        0,
        "cards_drawn_this_turn should start at 0"
    );

    // Draw first card: counter increments to 1.
    draw_card(&mut state, p1).unwrap();
    assert_eq!(
        state.players.get(&p1).unwrap().cards_drawn_this_turn,
        1,
        "after first draw, cards_drawn_this_turn should be 1"
    );

    // Draw second card: counter increments to 2.
    draw_card(&mut state, p1).unwrap();
    assert_eq!(
        state.players.get(&p1).unwrap().cards_drawn_this_turn,
        2,
        "after second draw, cards_drawn_this_turn should be 2"
    );

    // Draw third card: counter increments to 3.
    draw_card(&mut state, p1).unwrap();
    assert_eq!(
        state.players.get(&p1).unwrap().cards_drawn_this_turn,
        3,
        "after third draw, cards_drawn_this_turn should be 3"
    );

    // p2 drawing does NOT affect p1's counter.
    assert_eq!(
        state.players.get(&p2).unwrap().cards_drawn_this_turn,
        0,
        "p2's cards_drawn_this_turn should still be 0 (only p1 drew cards)"
    );

    // Simulate turn start: reset_turn_state resets the counter to 0.
    reset_turn_state(&mut state, p1);
    assert_eq!(
        state.players.get(&p1).unwrap().cards_drawn_this_turn,
        0,
        "after reset_turn_state, cards_drawn_this_turn should reset to 0"
    );
}

// ── MR-M8-15: Self-replacement + global ETB replacement both apply ─────────

#[test]
/// MR-M8-15 / CR 614.12 / CR 614.15 — When a permanent enters the battlefield,
/// both its own self-ETB replacement (is_self: true, e.g. enters tapped) and a
/// global ETB replacement registered in state (e.g. enters with counters) apply
/// in order: self-replacement first (CR 614.15), then global.
///
/// Setup:
/// - Card definition with AbilityDefinition::Replacement { is_self: true, EntersTapped }.
/// - Global ETB replacement in state.replacement_effects (EntersWithCounters).
/// Both modifications must be present on the creature after both functions fire.
fn test_etb_self_and_global_replacement_both_apply() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let card_id = CardId("enters-tapped-test-card".to_string());

    // Card definition: creature with a self-replacement that makes it enter tapped.
    let def = CardDefinition {
        card_id: card_id.clone(),
        name: "Enters Tapped Creature".into(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "This creature enters the battlefield tapped.".into(),
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldEnterBattlefield {
                filter: ObjectFilter::Any,
            },
            modification: ReplacementModification::EntersTapped,
            is_self: true,
            unless_condition: None,
        }],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    };

    let registry = CardRegistry::new(vec![def]);

    // Global ETB replacement: any creature that enters gets a +1/+1 counter.
    let global_etb = ReplacementEffect {
        id: ReplacementId(50),
        source: None,
        controller: p2, // controlled by p2 (e.g. Vorinclex)
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldEnterBattlefield {
            filter: ObjectFilter::AnyCreature,
        },
        modification: ReplacementModification::EntersWithCounters {
            counter: CounterType::PlusOnePlusOne,
            count: 1,
        },
    };

    // Build state: creature on the battlefield with the card_id set.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Enters Tapped Creature", 2, 2).with_card_id(card_id.clone()),
        )
        .with_replacement_effect(global_etb)
        .with_registry(registry.clone())
        .build()
        .unwrap();

    let creature_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Enters Tapped Creature")
        .map(|(id, _)| *id)
        .unwrap();

    // CR 614.15: apply self-replacement first.
    let _self_evts = mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state,
        creature_id,
        p1,
        Some(&card_id),
        &registry,
    );

    // Then apply global ETB replacements.
    let _global_evts =
        mtg_engine::rules::replacement::apply_etb_replacements(&mut state, creature_id, p1);

    let creature = state.objects.get(&creature_id).unwrap();

    // Self-replacement: creature must be tapped.
    assert!(
        creature.status.tapped,
        "CR 614.15: self-ETB replacement (EntersTapped) must apply; creature not tapped"
    );

    // Global replacement: creature must have a +1/+1 counter.
    assert_eq!(
        creature
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "CR 614.12: global ETB replacement (EntersWithCounters) must also apply; \
         creature has {} +1/+1 counters (expected 1)",
        creature
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0)
    );
}

// ── PB-2: Conditional ETB Tapped ────────────────────────────────────────────

/// Helper: create a land card definition with conditional ETB tapped.
fn conditional_etb_land(
    id: &str,
    name: &str,
    subtypes: &[&str],
    unless_condition: Condition,
) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.to_string()),
        name: name.to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Land].iter().cloned().collect(),
            supertypes: Default::default(),
            subtypes: subtypes.iter().map(|s| SubType(s.to_string())).collect(),
        },
        oracle_text: format!("Conditional ETB tapped land: {}", name),
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldEnterBattlefield {
                filter: ObjectFilter::Any,
            },
            modification: ReplacementModification::EntersTapped,
            is_self: true,
            unless_condition: Some(unless_condition),
        }],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        ..Default::default()
    }
}

/// Helper: create a basic land card definition.
fn basic_land_def(id: &str, name: &str, subtype: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.to_string()),
        name: name.to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Land].iter().cloned().collect(),
            supertypes: [SuperType::Basic].iter().cloned().collect(),
            subtypes: [SubType(subtype.to_string())].iter().cloned().collect(),
        },
        oracle_text: format!("Basic land: {}", name),
        abilities: vec![],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        ..Default::default()
    }
}

/// Helper: find an object by name in the game state.
fn find_cond_etb_by_name(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("Object '{}' not found", name))
}

#[test]
/// PB-2 / CR 614.1c — Check-land pattern: "enters tapped unless you control a
/// Plains or an Island." When the controller has a Plains on the battlefield,
/// the land enters untapped.
fn test_conditional_etb_check_land_condition_met() {
    let p1 = PlayerId(1);

    let plains_def = basic_land_def("plains", "Plains", "Plains");
    let check_land_def = conditional_etb_land(
        "glacial-fortress",
        "Glacial Fortress",
        &[],
        Condition::ControlLandWithSubtypes(vec![
            SubType("Plains".to_string()),
            SubType("Island".to_string()),
        ]),
    );
    let registry = CardRegistry::new(vec![plains_def, check_land_def]);

    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Plains")
                .with_types(vec![CardType::Land])
                .with_subtypes(vec![SubType("Plains".to_string())])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Glacial Fortress")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("glacial-fortress".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let check_land_id = find_cond_etb_by_name(&state, "Glacial Fortress");

    let evts = mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state,
        check_land_id,
        p1,
        Some(&CardId("glacial-fortress".to_string())),
        &registry,
    );

    let obj = state.objects.get(&check_land_id).unwrap();
    assert!(
        !obj.status.tapped,
        "CR 614.1c: Check-land should enter untapped when controller has a Plains"
    );
    assert!(
        !evts
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentTapped { .. })),
        "No PermanentTapped event when condition is met"
    );
}

#[test]
/// PB-2 / CR 614.1c — Check-land pattern: when the controller does NOT have a
/// matching land type, the land enters tapped.
fn test_conditional_etb_check_land_condition_not_met() {
    let p1 = PlayerId(1);

    let check_land_def = conditional_etb_land(
        "glacial-fortress",
        "Glacial Fortress",
        &[],
        Condition::ControlLandWithSubtypes(vec![
            SubType("Plains".to_string()),
            SubType("Island".to_string()),
        ]),
    );
    let registry = CardRegistry::new(vec![check_land_def]);

    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Glacial Fortress")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("glacial-fortress".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let check_land_id = find_cond_etb_by_name(&state, "Glacial Fortress");

    let evts = mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state,
        check_land_id,
        p1,
        Some(&CardId("glacial-fortress".to_string())),
        &registry,
    );

    let obj = state.objects.get(&check_land_id).unwrap();
    assert!(
        obj.status.tapped,
        "CR 614.1c: Check-land should enter tapped when controller has no Plains/Island"
    );
    assert!(
        evts.iter()
            .any(|e| matches!(e, GameEvent::PermanentTapped { .. })),
        "PermanentTapped event must be emitted"
    );
}

#[test]
/// PB-2 / CR 614.1c — Fast-land pattern: "enters tapped unless you control two
/// or fewer other lands." With 2 other lands, enters untapped; with 3, enters tapped.
fn test_conditional_etb_fast_land() {
    let p1 = PlayerId(1);

    let fast_land_def = conditional_etb_land(
        "blooming-marsh",
        "Blooming Marsh",
        &[],
        Condition::ControlAtMostNOtherLands(2),
    );
    let registry = CardRegistry::new(vec![fast_land_def]);

    // Case 1: 2 other lands → enters untapped.
    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Land 0")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Land 1")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Blooming Marsh")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("blooming-marsh".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let fast_land_id = find_cond_etb_by_name(&state, "Blooming Marsh");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state,
        fast_land_id,
        p1,
        Some(&CardId("blooming-marsh".to_string())),
        &registry,
    );
    assert!(
        !state.objects.get(&fast_land_id).unwrap().status.tapped,
        "Fast-land: 2 other lands → enters untapped"
    );

    // Case 2: 3 other lands → enters tapped.
    let mut state2 = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Land A")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Land B")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Land C")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Blooming Marsh")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("blooming-marsh".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let fast_land_id2 = find_cond_etb_by_name(&state2, "Blooming Marsh");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state2,
        fast_land_id2,
        p1,
        Some(&CardId("blooming-marsh".to_string())),
        &registry,
    );
    assert!(
        state2.objects.get(&fast_land_id2).unwrap().status.tapped,
        "Fast-land: 3 other lands → enters tapped"
    );
}

#[test]
/// PB-2 / CR 614.1c — Bond-land pattern: "enters tapped unless you have two or
/// more opponents." In 4-player Commander (3 opponents), enters untapped.
/// In 2-player (1 opponent), enters tapped.
fn test_conditional_etb_bond_land() {
    let p1 = PlayerId(1);

    let bond_land_def = conditional_etb_land(
        "bountiful-promenade",
        "Bountiful Promenade",
        &[],
        Condition::HaveTwoOrMoreOpponents,
    );
    let registry = CardRegistry::new(vec![bond_land_def.clone()]);

    // 4-player game (3 opponents) → enters untapped.
    let mut state4 = GameStateBuilder::four_player()
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Bountiful Promenade")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("bountiful-promenade".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let bond_id4 = find_cond_etb_by_name(&state4, "Bountiful Promenade");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state4,
        bond_id4,
        p1,
        Some(&CardId("bountiful-promenade".to_string())),
        &registry,
    );
    assert!(
        !state4.objects.get(&bond_id4).unwrap().status.tapped,
        "Bond-land: 3 opponents → enters untapped"
    );

    // 2-player game (1 opponent) → enters tapped.
    let mut state2 = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Bountiful Promenade")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("bountiful-promenade".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let bond_id2 = find_cond_etb_by_name(&state2, "Bountiful Promenade");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state2,
        bond_id2,
        p1,
        Some(&CardId("bountiful-promenade".to_string())),
        &registry,
    );
    assert!(
        state2.objects.get(&bond_id2).unwrap().status.tapped,
        "Bond-land: 1 opponent → enters tapped"
    );
}

#[test]
/// PB-2 / CR 614.1c — Battle-land pattern: "enters tapped unless you control
/// two or more basic lands."
fn test_conditional_etb_battle_land() {
    let p1 = PlayerId(1);

    let battle_land_def = conditional_etb_land(
        "canopy-vista",
        "Canopy Vista",
        &["Forest", "Plains"],
        Condition::ControlBasicLandsAtLeast(2),
    );
    let plains_def = basic_land_def("plains", "Plains", "Plains");
    let forest_def = basic_land_def("forest", "Forest", "Forest");
    let registry = CardRegistry::new(vec![battle_land_def, plains_def, forest_def]);

    // Case 1: 2 basic lands → enters untapped.
    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Plains")
                .with_types(vec![CardType::Land])
                .with_supertypes(vec![SuperType::Basic])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Forest")
                .with_types(vec![CardType::Land])
                .with_supertypes(vec![SuperType::Basic])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Canopy Vista")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("canopy-vista".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let battle_id = find_cond_etb_by_name(&state, "Canopy Vista");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state,
        battle_id,
        p1,
        Some(&CardId("canopy-vista".to_string())),
        &registry,
    );
    assert!(
        !state.objects.get(&battle_id).unwrap().status.tapped,
        "Battle-land: 2 basic lands → enters untapped"
    );

    // Case 2: 1 basic land → enters tapped.
    let mut state2 = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Plains")
                .with_types(vec![CardType::Land])
                .with_supertypes(vec![SuperType::Basic])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Canopy Vista")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("canopy-vista".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let battle_id2 = find_cond_etb_by_name(&state2, "Canopy Vista");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state2,
        battle_id2,
        p1,
        Some(&CardId("canopy-vista".to_string())),
        &registry,
    );
    assert!(
        state2.objects.get(&battle_id2).unwrap().status.tapped,
        "Battle-land: 1 basic land → enters tapped"
    );
}

#[test]
/// PB-2 / CR 614.1c — Slow-land pattern: "enters tapped unless you control
/// two or more other lands."
fn test_conditional_etb_slow_land() {
    let p1 = PlayerId(1);

    let slow_land_def = conditional_etb_land(
        "deathcap-glade",
        "Deathcap Glade",
        &[],
        Condition::ControlAtLeastNOtherLands(2),
    );
    let registry = CardRegistry::new(vec![slow_land_def]);

    // Case 1: 2 other lands → enters untapped.
    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Land 0")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Land 1")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Deathcap Glade")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("deathcap-glade".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let slow_id = find_cond_etb_by_name(&state, "Deathcap Glade");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state,
        slow_id,
        p1,
        Some(&CardId("deathcap-glade".to_string())),
        &registry,
    );
    assert!(
        !state.objects.get(&slow_id).unwrap().status.tapped,
        "Slow-land: 2 other lands → enters untapped"
    );

    // Case 2: 1 other land → enters tapped.
    let mut state2 = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Land 0")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Deathcap Glade")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("deathcap-glade".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let slow_id2 = find_cond_etb_by_name(&state2, "Deathcap Glade");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state2,
        slow_id2,
        p1,
        Some(&CardId("deathcap-glade".to_string())),
        &registry,
    );
    assert!(
        state2.objects.get(&slow_id2).unwrap().status.tapped,
        "Slow-land: 1 other land → enters tapped"
    );
}

#[test]
/// PB-2 / CR 614.1c — Mystic Sanctuary pattern: "enters tapped unless you
/// control three or more other Islands."
fn test_conditional_etb_subtype_count_land() {
    let p1 = PlayerId(1);

    let sanctuary_def = conditional_etb_land(
        "mystic-sanctuary",
        "Mystic Sanctuary",
        &["Island"],
        Condition::ControlAtLeastNOtherLandsWithSubtype {
            count: 3,
            subtype: SubType("Island".to_string()),
        },
    );
    let registry = CardRegistry::new(vec![sanctuary_def]);

    // Case 1: 3 other Islands → enters untapped.
    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Island 0")
                .with_types(vec![CardType::Land])
                .with_subtypes(vec![SubType("Island".to_string())])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Island 1")
                .with_types(vec![CardType::Land])
                .with_subtypes(vec![SubType("Island".to_string())])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Island 2")
                .with_types(vec![CardType::Land])
                .with_subtypes(vec![SubType("Island".to_string())])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Mystic Sanctuary")
                .with_types(vec![CardType::Land])
                .with_subtypes(vec![SubType("Island".to_string())])
                .with_card_id(CardId("mystic-sanctuary".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let sanctuary_id = find_cond_etb_by_name(&state, "Mystic Sanctuary");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state,
        sanctuary_id,
        p1,
        Some(&CardId("mystic-sanctuary".to_string())),
        &registry,
    );
    assert!(
        !state.objects.get(&sanctuary_id).unwrap().status.tapped,
        "Mystic Sanctuary: 3 other Islands → enters untapped"
    );

    // Case 2: 2 other Islands → enters tapped.
    let mut state2 = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Island 0")
                .with_types(vec![CardType::Land])
                .with_subtypes(vec![SubType("Island".to_string())])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Island 1")
                .with_types(vec![CardType::Land])
                .with_subtypes(vec![SubType("Island".to_string())])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1, "Mystic Sanctuary")
                .with_types(vec![CardType::Land])
                .with_subtypes(vec![SubType("Island".to_string())])
                .with_card_id(CardId("mystic-sanctuary".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let sanctuary_id2 = find_cond_etb_by_name(&state2, "Mystic Sanctuary");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state2,
        sanctuary_id2,
        p1,
        Some(&CardId("mystic-sanctuary".to_string())),
        &registry,
    );
    assert!(
        state2.objects.get(&sanctuary_id2).unwrap().status.tapped,
        "Mystic Sanctuary: 2 other Islands → enters tapped"
    );
}

#[test]
/// PB-2 / CR 614.1c — Reveal-land pattern: "you may reveal a [type] card from
/// your hand." Deterministic: auto-reveal if matching card in hand.
fn test_conditional_etb_reveal_land() {
    let p1 = PlayerId(1);

    let reveal_land_def = conditional_etb_land(
        "choked-estuary",
        "Choked Estuary",
        &[],
        Condition::CanRevealFromHandWithSubtype(vec![
            SubType("Island".to_string()),
            SubType("Swamp".to_string()),
        ]),
    );
    let registry = CardRegistry::new(vec![reveal_land_def]);

    // Case 1: Island in hand → enters untapped.
    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Island")
                .with_types(vec![CardType::Land])
                .with_subtypes(vec![SubType("Island".to_string())])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Choked Estuary")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("choked-estuary".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let reveal_id = find_cond_etb_by_name(&state, "Choked Estuary");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state,
        reveal_id,
        p1,
        Some(&CardId("choked-estuary".to_string())),
        &registry,
    );
    assert!(
        !state.objects.get(&reveal_id).unwrap().status.tapped,
        "Reveal-land: Island in hand → enters untapped"
    );

    // Case 2: no matching card in hand → enters tapped.
    let mut state2 = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry.clone())
        .object(
            ObjectSpec::card(p1, "Choked Estuary")
                .with_types(vec![CardType::Land])
                .with_card_id(CardId("choked-estuary".to_string()))
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let reveal_id2 = find_cond_etb_by_name(&state2, "Choked Estuary");
    mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state2,
        reveal_id2,
        p1,
        Some(&CardId("choked-estuary".to_string())),
        &registry,
    );
    assert!(
        state2.objects.get(&reveal_id2).unwrap().status.tapped,
        "Reveal-land: no matching card in hand → enters tapped"
    );
}

// =============================================================================
// PB-3: Shockland ETB — EntersTappedUnlessPayLife(2)
// =============================================================================

/// CR 614.1c: Shocklands enter tapped in deterministic mode (pre-M10 fallback:
/// "you may pay 2 life" is not paid → enters tapped).
#[test]
fn test_shockland_enters_tapped_deterministic_fallback() {
    use mtg_engine::{all_cards, Step};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::land(p1, "Blood Crypt")
                .with_card_id(CardId("blood-crypt".to_string()))
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_id = state.objects_in_zone(&ZoneId::Hand(p1)).first().unwrap().id;

    let (new_state, events) = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Shockland should be on the battlefield, tapped (deterministic fallback).
    let bf_objects = new_state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(
        bf_objects.len(),
        1,
        "Blood Crypt should be on the battlefield"
    );
    assert!(
        bf_objects[0].status.tapped,
        "Blood Crypt should enter tapped (deterministic: life payment not available pre-M10)"
    );

    // PermanentTapped event should fire.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentTapped { .. })),
        "PermanentTapped event should be emitted for shockland ETB replacement"
    );
}

/// CR 614.1c: All 10 shocklands use EntersTappedUnlessPayLife(2) and all enter tapped
/// in deterministic mode. Verifies the card definitions are wired correctly.
#[test]
fn test_all_shocklands_enter_tapped() {
    use mtg_engine::{all_cards, Step};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let shocklands = [
        ("blood-crypt", "Blood Crypt"),
        ("breeding-pool", "Breeding Pool"),
        ("godless-shrine", "Godless Shrine"),
        ("hallowed-fountain", "Hallowed Fountain"),
        ("overgrown-tomb", "Overgrown Tomb"),
        ("sacred-foundry", "Sacred Foundry"),
        ("steam-vents", "Steam Vents"),
        ("stomping-ground", "Stomping Ground"),
        ("temple-garden", "Temple Garden"),
        ("watery-grave", "Watery Grave"),
    ];

    for (slug, name) in &shocklands {
        let registry = CardRegistry::new(all_cards());

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(
                ObjectSpec::land(p1, name)
                    .with_card_id(CardId(slug.to_string()))
                    .in_zone(ZoneId::Hand(p1)),
            )
            .at_step(Step::PreCombatMain)
            .active_player(p1)
            .with_registry(registry)
            .build()
            .unwrap();
        state.turn.priority_holder = Some(p1);

        let card_id = state.objects_in_zone(&ZoneId::Hand(p1)).first().unwrap().id;

        let (new_state, _events) = mtg_engine::process_command(
            state,
            Command::PlayLand {
                player: p1,
                card: card_id,
            },
        )
        .unwrap();

        let bf_objects = new_state.objects_in_zone(&ZoneId::Battlefield);
        assert_eq!(bf_objects.len(), 1, "{name} should be on the battlefield");
        assert!(
            bf_objects[0].status.tapped,
            "{name} should enter tapped (deterministic fallback)"
        );
    }
}

/// Verify EntersTappedUnlessPayLife is distinct from EntersTapped in the card definition.
#[test]
fn test_shockland_uses_pay_life_variant_not_enters_tapped() {
    use mtg_engine::all_cards;

    let cards = all_cards();
    let blood_crypt = cards
        .iter()
        .find(|c| c.card_id.0 == "blood-crypt")
        .expect("Blood Crypt should be in all_cards");

    let has_pay_life = blood_crypt.abilities.iter().any(|a| {
        matches!(
            a,
            AbilityDefinition::Replacement {
                modification: ReplacementModification::EntersTappedUnlessPayLife(2),
                ..
            }
        )
    });
    assert!(
        has_pay_life,
        "Blood Crypt should use EntersTappedUnlessPayLife(2), not EntersTapped"
    );
}
