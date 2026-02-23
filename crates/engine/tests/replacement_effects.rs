//! Tests for replacement/prevention effects (M8 Sessions 1-2).
//!
//! Session 1: data model, serde, builder wiring.
//! Session 2: core application framework — find_applicable, determine_action,
//!            loop prevention, self-replacement priority, OrderReplacements command.

use std::collections::HashSet;

use mtg_engine::rules::replacement::{self, ReplacementResult};
use mtg_engine::{
    Command, CounterType, DamageTargetFilter, EffectDuration, GameEvent, GameStateBuilder,
    ObjectFilter, ObjectId, ObjectSpec, PlayerFilter, PlayerId, ReplacementEffect, ReplacementId,
    ReplacementModification, ReplacementTrigger, ZoneId, ZoneType,
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
use mtg_engine::CardId;

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

// ── Commander dies → single replacement auto-applies → command zone ──

#[test]
/// CR 903.9 — Commander with lethal damage: zone-change replacement redirects to command zone
/// Source: M8 Session 3 commander auto-apply
fn test_commander_dies_auto_redirects_to_command_zone() {
    let cmdr_card_id = CardId("cmdr-1".to_string());

    let mut state = GameStateBuilder::four_player()
        .player_commander(PlayerId(1), cmdr_card_id.clone())
        .object(
            ObjectSpec::creature(PlayerId(1), "Commander", 3, 3)
                .with_card_id(cmdr_card_id.clone())
                .with_damage(3),
        )
        .build()
        .unwrap();

    // Register commander zone-change replacements.
    register_commander_zone_replacements(&mut state);

    let _cmdr_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    let events = mtg_engine::check_and_apply_sbas(&mut state);

    // Commander should NOT be in graveyard.
    assert!(
        state
            .objects_in_zone(&ZoneId::Graveyard(PlayerId(1)))
            .is_empty(),
        "graveyard should be empty — commander redirected to command zone"
    );

    // Commander should be in command zone.
    let cmd_objects = state.objects_in_zone(&ZoneId::Command(PlayerId(1)));
    assert_eq!(cmd_objects.len(), 1, "commander should be in command zone");

    // ReplacementEffectApplied should be emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ReplacementEffectApplied { .. })),
        "should emit ReplacementEffectApplied event"
    );

    // No CreatureDied event (it went to command zone, not graveyard).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "should not emit CreatureDied — commander went to command zone"
    );
}

// ── Commander dies with Rest in Peace active → NeedsChoice ──

#[test]
/// CR 903.9 + CR 616.1 — Commander + Rest in Peace: two replacements compete, player chooses
/// Source: M8 Session 3 corner case 18
fn test_commander_dies_with_rest_in_peace_needs_choice() {
    let cmdr_card_id = CardId("cmdr-1".to_string());

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
        .player_commander(PlayerId(1), cmdr_card_id.clone())
        .object(
            ObjectSpec::creature(PlayerId(1), "Commander", 3, 3)
                .with_card_id(cmdr_card_id.clone())
                .with_damage(3),
        )
        .with_replacement_effect(rip_effect)
        .build()
        .unwrap();

    // Register commander zone-change replacements (adds 2 effects per commander).
    register_commander_zone_replacements(&mut state);

    let cmdr_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;

    let events = mtg_engine::check_and_apply_sbas(&mut state);

    // Two replacements compete: RiP (exile instead) and commander (command zone instead).
    // The affected player (commander owner, P1) must choose.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ReplacementChoiceRequired { player, choices, .. }
            if *player == PlayerId(1) && choices.len() == 2
        )),
        "should emit ReplacementChoiceRequired with 2 choices"
    );

    // Commander should still be on the battlefield (pending choice).
    assert!(
        state
            .objects
            .get(&cmdr_id)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false),
        "commander should remain on battlefield while choice is pending"
    );

    // Should have a pending zone change.
    assert_eq!(
        state.pending_zone_changes.len(),
        1,
        "should have one pending zone change"
    );
    assert_eq!(
        state.pending_zone_changes[0].object_id, cmdr_id,
        "pending zone change should be for the commander"
    );
    assert_eq!(
        state.pending_zone_changes[0].affected_player,
        PlayerId(1),
        "affected player should be the commander's owner"
    );
}

// ── Commander dies with Rest in Peace → player chooses command zone ──

#[test]
/// CR 903.9 + CR 616.1 — Player resolves choice by sending commander to command zone
/// Source: M8 Session 3 corner case 18 resolution
fn test_commander_dies_with_rip_player_chooses_command_zone() {
    let cmdr_card_id = CardId("cmdr-1".to_string());

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
        .player_commander(PlayerId(1), cmdr_card_id.clone())
        .object(
            ObjectSpec::creature(PlayerId(1), "Commander", 3, 3)
                .with_card_id(cmdr_card_id.clone())
                .with_damage(3),
        )
        .with_replacement_effect(rip_effect)
        .build()
        .unwrap();

    register_commander_zone_replacements(&mut state);

    // Run SBAs to get the pending choice.
    let _sba_events = mtg_engine::check_and_apply_sbas(&mut state);

    // Find the commander replacement ID (the one that redirects to command zone).
    let cmdr_repl_id = state
        .replacement_effects
        .iter()
        .find(|e| {
            matches!(
                &e.modification,
                ReplacementModification::RedirectToZone(ZoneType::Command)
            ) && matches!(
                &e.trigger,
                ReplacementTrigger::WouldChangeZone {
                    to: ZoneType::Graveyard,
                    ..
                }
            )
        })
        .map(|e| e.id)
        .expect("commander graveyard replacement should exist");

    // Player 1 chooses commander replacement first.
    let (state, _order_events) = mtg_engine::process_command(
        state,
        Command::OrderReplacements {
            player: PlayerId(1),
            ids: vec![cmdr_repl_id],
        },
    )
    .expect("OrderReplacements should succeed");

    // Commander should now be in the command zone.
    let cmd_objects = state.objects_in_zone(&ZoneId::Command(PlayerId(1)));
    assert_eq!(
        cmd_objects.len(),
        1,
        "commander should be in command zone after player chose"
    );

    // Graveyard should be empty.
    assert!(
        state
            .objects_in_zone(&ZoneId::Graveyard(PlayerId(1)))
            .is_empty(),
        "graveyard should be empty"
    );

    // Exile should be empty (RiP didn't get to fire first).
    assert!(
        state.objects_in_zone(&ZoneId::Exile).is_empty(),
        "exile should be empty — commander went to command zone"
    );

    // Pending zone changes should be cleared.
    assert!(
        state.pending_zone_changes.is_empty(),
        "pending zone changes should be cleared"
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

// ── Creature in ExileObject effect with redirect ──

#[test]
/// CR 614 — ExileObject effect checks replacement effects (auto-apply redirect)
/// Source: M8 Session 3 effect interception
fn test_exile_effect_checks_replacements() {
    // A commander being exiled should be redirectable to command zone.
    let cmdr_card_id = CardId("cmdr-exile".to_string());

    let mut state = GameStateBuilder::four_player()
        .player_commander(PlayerId(1), cmdr_card_id.clone())
        .object(
            ObjectSpec::creature(PlayerId(1), "Commander", 4, 4).with_card_id(cmdr_card_id.clone()),
        )
        .build()
        .unwrap();

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

    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Commander should be in command zone, not exile.
    let cmd_objects = state.objects_in_zone(&ZoneId::Command(PlayerId(1)));
    assert_eq!(
        cmd_objects.len(),
        1,
        "commander should be redirected to command zone when exiled"
    );
    assert!(
        state.objects_in_zone(&ZoneId::Exile).is_empty(),
        "exile zone should be empty — commander was redirected"
    );

    // ReplacementEffectApplied should be emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ReplacementEffectApplied { .. })),
        "should emit ReplacementEffectApplied event"
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

// ── register_commander_zone_replacements creates correct effects ──

#[test]
/// CR 903.9 — register_commander_zone_replacements creates 2 effects per commander
/// Source: M8 Session 3 commander registration validation
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

    // Should have 2 effects: one for graveyard, one for exile.
    assert_eq!(
        state.replacement_effects.len(),
        2,
        "should have 2 replacement effects for commander"
    );

    // One should trigger on WouldChangeZone to Graveyard.
    assert!(
        state.replacement_effects.iter().any(|e| matches!(
            &e.trigger,
            ReplacementTrigger::WouldChangeZone {
                to: ZoneType::Graveyard,
                ..
            }
        )),
        "should have graveyard replacement"
    );

    // One should trigger on WouldChangeZone to Exile.
    assert!(
        state.replacement_effects.iter().any(|e| matches!(
            &e.trigger,
            ReplacementTrigger::WouldChangeZone {
                to: ZoneType::Exile,
                ..
            }
        )),
        "should have exile replacement"
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

// ── Pending zone change skipped in subsequent SBA pass ──

#[test]
/// CR 614 — Objects with pending replacement choices are skipped in SBA passes
/// Source: M8 Session 3 pending skip validation
fn test_pending_zone_change_skipped_in_sba() {
    let cmdr_card_id = CardId("cmdr-skip".to_string());

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
        .player_commander(PlayerId(1), cmdr_card_id.clone())
        .object(
            ObjectSpec::creature(PlayerId(1), "Commander", 3, 3)
                .with_card_id(cmdr_card_id.clone())
                .with_damage(3),
        )
        .with_replacement_effect(rip_effect)
        .build()
        .unwrap();

    register_commander_zone_replacements(&mut state);

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
