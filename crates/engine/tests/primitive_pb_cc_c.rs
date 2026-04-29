//! Tests for PB-CC-C: `LayerModification::ModifyPowerDynamic` and
//! `LayerModification::ModifyToughnessDynamic` (CR 613.1c, CR 608.2h).
//!
//! These variants are DSL placeholders, mirroring `ModifyBothDynamic`:
//! - They must be substituted into `ModifyPower(v)` / `ModifyToughness(v)` at
//!   `Effect::ApplyContinuousEffect` execution time (CR 608.2h).
//! - They must NEVER appear in a stored `ContinuousEffect` — if they reach
//!   layer-application code, `debug_assert!(false, …)` fires (CR 613.1c guard).
//! - `negate=true` produces `-v` from a non-negative amount.
//!
//! Tests:
//!   1. Substitution: `ModifyPowerDynamic` → `ModifyPower(v)` at execute time.
//!   2. Substitution: `ModifyToughnessDynamic` → `ModifyToughness(v)` at execute time.
//!   3. Panic guard: unsubstituted `ModifyPowerDynamic` reaching layer application panics.
//!   4. Panic guard: unsubstituted `ModifyToughnessDynamic` reaching layer application panics.
//!   5. Exuberant Fuseling power scaling: power = oil counter count, toughness fixed.

use mtg_engine::cards::card_definition::{ContinuousEffectDef, EffectTarget};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    calculate_characteristics, ContinuousEffect, CounterType, Effect, EffectAmount, EffectDuration,
    EffectFilter, EffectId, EffectLayer, GameStateBuilder, LayerModification, ObjectId, ObjectSpec,
    PlayerId, Step, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

// ── Test 1: ModifyPowerDynamic substitution ───────────────────────────────────

/// CR 608.2h / CR 613.1c — `ModifyPowerDynamic` is resolved at
/// `Effect::ApplyContinuousEffect` execution time. The stored `ContinuousEffect`
/// must carry a concrete `ModifyPower(i32)`, not the dynamic placeholder.
///
/// Setup: creature with 3 oil counters; apply `ModifyPowerDynamic { CounterCount(Oil),
/// negate=false }` via `execute_effect`.
/// Expected: stored effect is `ModifyPower(3)`, not `ModifyPowerDynamic`.
#[test]
fn test_modify_power_dynamic_substituted_at_apply_time() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .object(ObjectSpec::creature(p1, "Fuseling Stub", 0, 1).in_zone(ZoneId::Battlefield))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let fuseling_id = find_object(&state, "Fuseling Stub");

    // Pre-place 3 oil counters on the creature.
    {
        let obj = state.objects.get_mut(&fuseling_id).unwrap();
        obj.counters.insert(CounterType::Oil, 3);
    }

    // Execute ApplyContinuousEffect with ModifyPowerDynamic.
    // CR 608.2h: the substitution locks in the oil counter count (3) at execution time.
    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyPowerDynamic {
                amount: Box::new(EffectAmount::CounterCount {
                    target: EffectTarget::Source,
                    counter: CounterType::Oil,
                }),
                negate: false,
            },
            filter: EffectFilter::Source,
            duration: EffectDuration::WhileSourceOnBattlefield,
            condition: None,
        }),
    };

    let mut ctx = EffectContext::new(p1, fuseling_id, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    // Stored effect must be ModifyPower(3), NOT ModifyPowerDynamic.
    let dynamic_in_effects = state.continuous_effects.iter().any(|e| {
        matches!(
            &e.modification,
            LayerModification::ModifyPowerDynamic { .. }
        )
    });
    assert!(
        !dynamic_in_effects,
        "CR 608.2h: ModifyPowerDynamic must be substituted before storage \
         (ModifyPower(3) expected, found dynamic variant)"
    );

    let has_modify_power_3 = state
        .continuous_effects
        .iter()
        .any(|e| matches!(&e.modification, LayerModification::ModifyPower(3)));
    assert!(
        has_modify_power_3,
        "CR 608.2h: Stored effect should have ModifyPower(3) (3 oil counters → +3 power)"
    );

    // calculate_characteristics should yield power = 0 + 3 = 3, toughness = 1.
    let chars = calculate_characteristics(&state, fuseling_id).unwrap();
    assert_eq!(chars.power, Some(3), "power = base(0) + ModifyPower(3) = 3");
    assert_eq!(chars.toughness, Some(1), "toughness unchanged at 1");
}

// ── Test 2: ModifyToughnessDynamic substitution ───────────────────────────────

/// CR 608.2h / CR 613.1c — `ModifyToughnessDynamic` is resolved at
/// `Effect::ApplyContinuousEffect` execution time. The stored `ContinuousEffect`
/// must carry a concrete `ModifyToughness(i32)`, not the dynamic placeholder.
///
/// Setup: creature with 2 oil counters; apply `ModifyToughnessDynamic { CounterCount(Oil),
/// negate=false }` via `execute_effect`.
/// Expected: stored effect is `ModifyToughness(2)`, not `ModifyToughnessDynamic`.
#[test]
fn test_modify_toughness_dynamic_substituted_at_apply_time() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .object(ObjectSpec::creature(p1, "Dynamic Stub", 1, 1).in_zone(ZoneId::Battlefield))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let stub_id = find_object(&state, "Dynamic Stub");

    // Pre-place 2 oil counters on the creature.
    {
        let obj = state.objects.get_mut(&stub_id).unwrap();
        obj.counters.insert(CounterType::Oil, 2);
    }

    // Execute ApplyContinuousEffect with ModifyToughnessDynamic.
    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyToughnessDynamic {
                amount: Box::new(EffectAmount::CounterCount {
                    target: EffectTarget::Source,
                    counter: CounterType::Oil,
                }),
                negate: false,
            },
            filter: EffectFilter::Source,
            duration: EffectDuration::WhileSourceOnBattlefield,
            condition: None,
        }),
    };

    let mut ctx = EffectContext::new(p1, stub_id, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    // Stored effect must be ModifyToughness(2), NOT ModifyToughnessDynamic.
    let dynamic_in_effects = state.continuous_effects.iter().any(|e| {
        matches!(
            &e.modification,
            LayerModification::ModifyToughnessDynamic { .. }
        )
    });
    assert!(
        !dynamic_in_effects,
        "CR 608.2h: ModifyToughnessDynamic must be substituted before storage \
         (ModifyToughness(2) expected, found dynamic variant)"
    );

    let has_modify_toughness_2 = state
        .continuous_effects
        .iter()
        .any(|e| matches!(&e.modification, LayerModification::ModifyToughness(2)));
    assert!(
        has_modify_toughness_2,
        "CR 608.2h: Stored effect should have ModifyToughness(2) (2 oil counters → +2 toughness)"
    );

    // calculate_characteristics should yield power = 1, toughness = 1 + 2 = 3.
    let chars = calculate_characteristics(&state, stub_id).unwrap();
    assert_eq!(chars.power, Some(1), "power unchanged at 1");
    assert_eq!(
        chars.toughness,
        Some(3),
        "toughness = base(1) + ModifyToughness(2) = 3"
    );
}

// ── Test 3: Panic guard — ModifyPowerDynamic must never reach layer application ──

/// CR 608.2h / CR 613.1c — If `ModifyPowerDynamic` reaches layer-application code
/// without having been substituted, `debug_assert!(false, …)` panics. This indicates
/// a bug in the calling code (substitution was skipped).
///
/// This test directly stores an unsubstituted `ModifyPowerDynamic` effect in game state
/// and calls `calculate_characteristics`, which must panic.
#[test]
#[should_panic]
fn test_modify_power_dynamic_panics_if_not_substituted() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .object(ObjectSpec::creature(p1, "Bug Stub", 2, 2).in_zone(ZoneId::Battlefield))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let stub_id = find_object(&state, "Bug Stub");

    // Directly insert an unsubstituted ModifyPowerDynamic into continuous_effects.
    // This simulates a bug where the substitution arm in effects/mod.rs was skipped.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(9900),
        source: Some(stub_id),
        timestamp: 9900,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(stub_id),
        // CR 608.2h: this must never reach layer application.
        modification: LayerModification::ModifyPowerDynamic {
            amount: Box::new(EffectAmount::Fixed(1)),
            negate: false,
        },
        is_cda: false,
        condition: None,
    });

    // calculate_characteristics triggers layer application, which must panic on this variant.
    let _ = calculate_characteristics(&state, stub_id);
}

// ── Test 4: Panic guard — ModifyToughnessDynamic must never reach layer application ─

/// CR 608.2h / CR 613.1c — Same guard as test 3, but for `ModifyToughnessDynamic`.
#[test]
#[should_panic]
fn test_modify_toughness_dynamic_panics_if_not_substituted() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .object(ObjectSpec::creature(p1, "Bug Stub T", 2, 2).in_zone(ZoneId::Battlefield))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let stub_id = find_object(&state, "Bug Stub T");

    // Directly insert an unsubstituted ModifyToughnessDynamic into continuous_effects.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(9901),
        source: Some(stub_id),
        timestamp: 9901,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(stub_id),
        // CR 608.2h: this must never reach layer application.
        modification: LayerModification::ModifyToughnessDynamic {
            amount: Box::new(EffectAmount::Fixed(1)),
            negate: false,
        },
        is_cda: false,
        condition: None,
    });

    // calculate_characteristics triggers layer application, which must panic on this variant.
    let _ = calculate_characteristics(&state, stub_id);
}

// ── Test 5: Exuberant Fuseling power scaling ──────────────────────────────────

/// CR 613.1c / CR 608.2h — Exuberant Fuseling CDA: "this creature gets +1/+0
/// for each oil counter on it." Tests that `execute_effect` with
/// `ApplyContinuousEffect { ModifyPowerDynamic { CounterCount(Oil), negate=false } }`
/// substitutes to `ModifyPower(N)` where N is the oil counter count, and that
/// `calculate_characteristics` yields the correct power.
///
/// Cases: N ∈ {0, 1, 3}. Toughness is 1 throughout (no toughness modification).
#[test]
fn test_exuberant_fuseling_power_scales_with_oil_counters() {
    let cases: &[(u32, i32)] = &[(0, 0), (1, 1), (3, 3)];

    for &(oil_count, expected_power) in cases {
        let p1 = p(1);

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p(2))
            .object(ObjectSpec::creature(p1, "Fuseling", 0, 1).in_zone(ZoneId::Battlefield))
            .at_step(Step::PreCombatMain)
            .active_player(p1)
            .build()
            .unwrap();

        let fuseling_id = find_object(&state, "Fuseling");

        // Pre-place N oil counters on the creature.
        if oil_count > 0 {
            let obj = state.objects.get_mut(&fuseling_id).unwrap();
            obj.counters.insert(CounterType::Oil, oil_count);
        }

        // Execute the CDA effect expression from exuberant_fuseling.rs.
        // CR 608.2h: the oil counter count is locked at execution time.
        let effect = Effect::ApplyContinuousEffect {
            effect_def: Box::new(ContinuousEffectDef {
                layer: EffectLayer::PtModify,
                modification: LayerModification::ModifyPowerDynamic {
                    amount: Box::new(EffectAmount::CounterCount {
                        target: EffectTarget::Source,
                        counter: CounterType::Oil,
                    }),
                    negate: false,
                },
                filter: EffectFilter::Source,
                duration: EffectDuration::WhileSourceOnBattlefield,
                condition: None,
            }),
        };

        let mut ctx = EffectContext::new(p1, fuseling_id, vec![]);
        execute_effect(&mut state, &effect, &mut ctx);

        let chars = calculate_characteristics(&state, fuseling_id).unwrap();
        assert_eq!(
            chars.power,
            Some(expected_power),
            "Fuseling with {oil_count} oil counters: expected power {expected_power}, \
             got {:?} (CR 613.1c, PB-CC-C)",
            chars.power
        );
        assert_eq!(
            chars.toughness,
            Some(1),
            "Fuseling toughness must be fixed at 1 regardless of oil counters \
             (CR 613.1c: no toughness modification in this CDA)"
        );
    }
}
