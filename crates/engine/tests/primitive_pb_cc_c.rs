//! Tests for PB-CC-C: `LayerModification::ModifyPowerDynamic` and
//! `LayerModification::ModifyToughnessDynamic` (CR 613.4c, CR 608.2h).
//!
//! These variants are DSL placeholders, mirroring `ModifyBothDynamic`:
//! - They must be substituted into `ModifyPower(v)` / `ModifyToughness(v)` at
//!   `Effect::ApplyContinuousEffect` execution time (CR 608.2h).
//! - They must NEVER appear in a stored `ContinuousEffect` — if they reach
//!   layer-application code, `debug_assert!(false, …)` fires (CR 613.4c guard).
//! - `negate=true` produces `-v` from a non-negative amount.
//!
//! These variants implement CR 608.2h "X-locked-at-resolution" semantics for
//! one-shot spell pumps (e.g. "creatures you control get +X/+0 until end of turn
//! where X is the number of Vampires you control"). They are NOT suitable for
//! static abilities of permanents (CR 611.3a live re-evaluation) — that requires
//! a separate Layer-7c dynamic primitive (PB-CC-C-followup). See `CdaPowerToughness`
//! (Layer 7a) as the closest existing analogue.
//!
//! Tests:
//!   1. Substitution: `ModifyPowerDynamic` → `ModifyPower(v)` at execute time.
//!   2. Substitution: `ModifyToughnessDynamic` → `ModifyToughness(v)` at execute time.
//!   3. Panic guard: unsubstituted `ModifyPowerDynamic` reaching layer application panics.
//!   4. Panic guard: unsubstituted `ModifyToughnessDynamic` reaching layer application panics.
//!   5. Full-dispatch: CR 608.2h X-locked-at-resolution — post-substitute counter mutation
//!      does NOT change stored power (validates the lock-in semantic and full dispatch path).

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

/// CR 608.2h / CR 613.4c — `ModifyPowerDynamic` is resolved at
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

/// CR 608.2h / CR 613.4c — `ModifyToughnessDynamic` is resolved at
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

// ── Test 3: ModifyPowerDynamic stored with is_cda=false reaches live-eval ────────
//
// PB-CC-C-followup update: the previous `debug_assert!(false, …)` guard in
// `apply_layer_modification` was removed to enable CR 611.3a static-ability live
// re-evaluation. `ModifyPowerDynamic` now calls `resolve_cda_amount` in ALL cases
// (both `is_cda=true` static-ability path and any residual `is_cda=false` path).
// The only correct way to ensure a spell-effect dynamic is locked at resolution is
// to let the `Effect::ApplyContinuousEffect` substitution arm convert it to a
// concrete `ModifyPower(N)` before storage — which tests 1/2/5 verify.
//
// This test documents the new behavior: a directly-stored unsubstituted
// `ModifyPowerDynamic { Fixed(1), negate=false }` now produces power = base(2) + 1 = 3
// (Fixed(1) resolves deterministically via `resolve_cda_amount`).

/// CR 611.3a / CR 613.4c — `ModifyPowerDynamic` stored directly (bypassing
/// `Effect::ApplyContinuousEffect`) now resolves via `resolve_cda_amount`
/// rather than panicking. For `EffectAmount::Fixed(1)`: power = base(2) + 1 = 3.
///
/// Note: the *correct* spell-effect path substitutes this to `ModifyPower(1)`
/// before storage (tested by tests 1/2/5). This test exercises the residual
/// behavior (PB-CC-C-followup removed the `debug_assert!` guard).
#[test]
fn test_modify_power_dynamic_resolves_via_cda_amount_when_unsubstituted() {
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

    // Directly insert an unsubstituted ModifyPowerDynamic — simulates the residual case
    // where substitution was bypassed. PB-CC-C-followup: this now calls resolve_cda_amount
    // instead of debug_assert!-ing.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(9900),
        source: Some(stub_id),
        timestamp: 9900,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(stub_id),
        modification: LayerModification::ModifyPowerDynamic {
            amount: Box::new(EffectAmount::Fixed(1)),
            negate: false,
        },
        is_cda: false,
        condition: None,
    });

    // PB-CC-C-followup: no panic. Fixed(1) resolves to 1. Power = base(2) + 1 = 3.
    let chars = calculate_characteristics(&state, stub_id).unwrap();
    assert_eq!(chars.power, Some(3), "power = base(2) + Fixed(1) = 3");
    assert_eq!(chars.toughness, Some(2), "toughness unchanged at base(2)");
}

// ── Test 4: ModifyToughnessDynamic stored with is_cda=false reaches live-eval ───
//
// Same PB-CC-C-followup update as test 3 — see comment above for context.

/// CR 611.3a / CR 613.4c — `ModifyToughnessDynamic` stored directly resolves via
/// `resolve_cda_amount` for `EffectAmount::Fixed(1)`: toughness = base(2) + 1 = 3.
#[test]
fn test_modify_toughness_dynamic_resolves_via_cda_amount_when_unsubstituted() {
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

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(9901),
        source: Some(stub_id),
        timestamp: 9901,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(stub_id),
        modification: LayerModification::ModifyToughnessDynamic {
            amount: Box::new(EffectAmount::Fixed(1)),
            negate: false,
        },
        is_cda: false,
        condition: None,
    });

    // PB-CC-C-followup: no panic. Fixed(1) resolves to 1. Toughness = base(2) + 1 = 3.
    let chars = calculate_characteristics(&state, stub_id).unwrap();
    assert_eq!(chars.power, Some(2), "power unchanged at base(2)");
    assert_eq!(
        chars.toughness,
        Some(3),
        "toughness = base(2) + Fixed(1) = 3"
    );
}

// ── Test 5: Full-dispatch — CR 608.2h X-locked-at-resolution semantic ────────

/// CR 608.2h / CR 613.4c — Validates the full dispatch path for one-shot spell
/// use cases (the valid use case for `ModifyPowerDynamic`/`ModifyToughnessDynamic`).
///
/// Scenario:
///   1. Creature is on the battlefield with 2 oil counters.
///   2. A resolving spell applies `Effect::ApplyContinuousEffect` with
///      `ModifyPowerDynamic { CounterCount(Oil), negate=false }`.
///   3. Substitution captures the counter count (2) at resolution time.
///      Stored effect becomes `ModifyPower(2)`.
///   4. `calculate_characteristics` after resolution: power = base(0) + 2 = 2.
///   5. Source counter count is mutated (now 5 oil counters).
///   6. `calculate_characteristics` again: power is STILL 2, not 5.
///      This validates CR 608.2h "X is locked in at resolution" for one-shot pumps.
///
/// NOTE: This is the correct semantic for one-shot spell effects (CR 608.2h).
/// Static abilities of permanents (CR 611.3a) must re-evaluate continuously —
/// that requires the deferred Layer-7c dynamic primitive (PB-CC-C-followup).
/// `CdaPowerToughness` (Layer 7a, `resolve_cda_amount`) is the closest existing
/// dynamic-static analogue.
#[test]
fn test_modify_power_dynamic_x_locked_at_resolution() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .object(ObjectSpec::creature(p1, "Spell Target", 0, 1).in_zone(ZoneId::Battlefield))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let target_id = find_object(&state, "Spell Target");

    // Step 1: Place 2 oil counters on the creature (state at spell resolution time).
    {
        let obj = state.objects.get_mut(&target_id).unwrap();
        obj.counters.insert(CounterType::Oil, 2);
    }

    // Step 2: Resolving spell executes ApplyContinuousEffect with ModifyPowerDynamic.
    // CR 608.2h: substitution locks the oil counter count (2) at execution time.
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

    let mut ctx = EffectContext::new(p1, target_id, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    // Step 3: Verify substitution stored ModifyPower(2), not the dynamic placeholder.
    let dynamic_in_effects = state.continuous_effects.iter().any(|e| {
        matches!(
            &e.modification,
            LayerModification::ModifyPowerDynamic { .. }
        )
    });
    assert!(
        !dynamic_in_effects,
        "CR 608.2h: ModifyPowerDynamic must be substituted before storage"
    );

    // Step 4: calculate_characteristics with 2 oil counters → power = 0 + 2 = 2.
    let chars_before = calculate_characteristics(&state, target_id).unwrap();
    assert_eq!(
        chars_before.power,
        Some(2),
        "CR 608.2h: power = base(0) + ModifyPower(2) = 2 (locked at resolution)"
    );
    assert_eq!(chars_before.toughness, Some(1), "toughness unchanged");

    // Step 5: Mutate source counter count (simulate post-resolution counter additions).
    {
        let obj = state.objects.get_mut(&target_id).unwrap();
        obj.counters.insert(CounterType::Oil, 5);
    }

    // Step 6: calculate_characteristics again — power must still be 2, not 5.
    // CR 608.2h: the one-shot spell effect locked in X=2 at resolution; subsequent
    // counter changes do NOT retroactively change the stored ModifyPower value.
    // (Contrast CR 611.3a static-ability live re-evaluation, which is out of scope here.)
    let chars_after = calculate_characteristics(&state, target_id).unwrap();
    assert_eq!(
        chars_after.power,
        Some(2),
        "CR 608.2h: power must remain 2 after counter mutation — X locked at resolution \
         (stored ModifyPower(2) does not change when source has 5 oil counters now)"
    );
    assert_eq!(
        chars_after.toughness,
        Some(1),
        "toughness unchanged after mutation"
    );
}
