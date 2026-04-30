//! Tests for PB-CC-A: `EffectAmount::PlayerCounterCount` (CR 122.1).
//!
//! This new variant counts counters (`Poison` today; future `Energy`/`Experience`/etc.)
//! on a player or sums across players. Oracle text "for each poison counter your
//! opponents have" is unambiguously a SUM (Vishgraz ruling 2023-02-04 + CR 122.1
//! "the number of [counters] on this player"): every poison counter on any
//! opponent counts once.
//!
//! Engine surface:
//! - `effects/mod.rs::resolve_amount` arm — used by ApplyContinuousEffect
//!   substitution (e.g. ModifyBothDynamic) and by other resolve_amount callers.
//! - `rules/layers.rs::resolve_cda_amount` arm — used by Layer 7a
//!   `LayerModification::SetPtDynamic` (i.e. `AbilityDefinition::CdaPowerToughness`).
//!   No layer recursion: `PlayerState::poison_counters` is NOT a layer-derived
//!   characteristic.
//! - `state/hash.rs` arm — discriminant 16; HASH_SCHEMA_VERSION bumped 11→12.
//!
//! Tests:
//!   1. Schema-version sentinel (catches unintended hash changes).
//!   2. `Controller` reads the controller's poison count.
//!   3. `EachOpponent` sums every opponent's poison (the Vishgraz semantic).
//!   4. `EachPlayer` sums every player including the controller.
//!   5. `DeclaredTarget { index }` reads the spell's chosen player.
//!   6. Non-`Poison` counter kinds return 0 (no panic — defensive contract).
//!   7. CDA path via `CdaPowerToughness` + `SetPtDynamic` flows through
//!      `resolve_cda_amount` and `calculate_characteristics`.
//!   8. CDA sum path: 4-player game with 0 / 3 / 8 distributions yields
//!      P/T scaled by the sum across opponents (the Vishgraz scenario).

use mtg_engine::cards::card_definition::{ContinuousEffectDef, EffectAmount as EA, PlayerTarget};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    calculate_characteristics, CardId, CounterType, Effect, EffectDuration, EffectFilter,
    EffectLayer, GameStateBuilder, LayerModification, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
    HASH_SCHEMA_VERSION,
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

/// Resolve `EffectAmount::PlayerCounterCount` indirectly via the
/// `ApplyContinuousEffect` substitution arm. Substitution at execute time turns
/// `ModifyBothDynamic { amount: PlayerCounterCount(...) }` into
/// `ModifyBoth(N)`; we read N out of the stored ContinuousEffect.
fn resolve_via_apply(
    state: &mut mtg_engine::GameState,
    source_id: ObjectId,
    controller: PlayerId,
    player: PlayerTarget,
    counter: CounterType,
) -> i32 {
    let pre_len = state.continuous_effects.len();
    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBothDynamic {
                amount: Box::new(EA::PlayerCounterCount { player, counter }),
                negate: false,
            },
            filter: EffectFilter::Source,
            duration: EffectDuration::WhileSourceOnBattlefield,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(controller, source_id, vec![]);
    execute_effect(state, &effect, &mut ctx);

    // Last pushed effect carries the substituted ModifyBoth(N).
    let post = state
        .continuous_effects
        .iter()
        .nth(pre_len)
        .expect("ApplyContinuousEffect must push exactly one effect");
    match &post.modification {
        LayerModification::ModifyBoth(n) => *n,
        other => panic!(
            "expected substituted ModifyBoth(_), got {:?}; the substitution arm \
             in effects/mod.rs lost the dynamic value",
            other
        ),
    }
}

// ── Test 1: HASH_SCHEMA_VERSION sentinel ─────────────────────────────────────

/// Catch any uncommitted bump after PB-CC-C-followup. The next primitive that touches
/// `AbilityDefinition` shape must update both the constant and this sentinel.
#[test]
fn test_hash_schema_version_after_pb_cc_c_followup() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 13,
        "PB-CC-C-followup bumped HASH_SCHEMA_VERSION 12→13; if you intentionally bumped it \
         again, update this test together with state/hash.rs history."
    );
}

// ── Test 2: PlayerTarget::Controller reads controller's poison ───────────────

#[test]
fn test_player_counter_count_controller() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Source", 1, 1).in_zone(ZoneId::Battlefield))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().poison_counters = 4;
    state.players.get_mut(&p2).unwrap().poison_counters = 7;

    let src = find_object(&state, "Source");
    let n = resolve_via_apply(
        &mut state,
        src,
        p1,
        PlayerTarget::Controller,
        CounterType::Poison,
    );
    assert_eq!(n, 4, "Controller (p1) has 4 poison counters");
}

// ── Test 3: PlayerTarget::EachOpponent sums opponents (Vishgraz semantic) ────

#[test]
fn test_player_counter_count_each_opponent_sums() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(ObjectSpec::creature(p1, "Source", 1, 1).in_zone(ZoneId::Battlefield))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // Controller p1 has 100 poison — must NOT be summed.
    state.players.get_mut(&p1).unwrap().poison_counters = 100;
    state.players.get_mut(&p2).unwrap().poison_counters = 1;
    state.players.get_mut(&p3).unwrap().poison_counters = 2;
    state.players.get_mut(&p4).unwrap().poison_counters = 5;

    let src = find_object(&state, "Source");
    let n = resolve_via_apply(
        &mut state,
        src,
        p1,
        PlayerTarget::EachOpponent,
        CounterType::Poison,
    );
    assert_eq!(
        n, 8,
        "EachOpponent sums opponents: 1+2+5=8 (controller's 100 excluded)"
    );
}

// ── Test 4: PlayerTarget::EachPlayer sums everyone ──────────────────────────

#[test]
fn test_player_counter_count_each_player_sums() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(ObjectSpec::creature(p1, "Source", 1, 1).in_zone(ZoneId::Battlefield))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().poison_counters = 3;
    state.players.get_mut(&p2).unwrap().poison_counters = 4;
    state.players.get_mut(&p3).unwrap().poison_counters = 5;

    let src = find_object(&state, "Source");
    let n = resolve_via_apply(
        &mut state,
        src,
        p1,
        PlayerTarget::EachPlayer,
        CounterType::Poison,
    );
    assert_eq!(n, 12, "EachPlayer sums everyone: 3+4+5=12");
}

// ── Test 5: PlayerTarget::DeclaredTarget reads the chosen player ────────────

#[test]
fn test_player_counter_count_declared_target() {
    use mtg_engine::SpellTarget;
    use mtg_engine::Target;

    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(ObjectSpec::creature(p1, "Source", 1, 1).in_zone(ZoneId::Battlefield))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().poison_counters = 1;
    state.players.get_mut(&p2).unwrap().poison_counters = 9;
    state.players.get_mut(&p3).unwrap().poison_counters = 2;

    let src = find_object(&state, "Source");

    // Build an EffectContext with player target index 0 = p2.
    let ctx_targets = vec![SpellTarget {
        target: Target::Player(p2),
        zone_at_cast: None,
    }];
    let mut ctx = EffectContext::new(p1, src, ctx_targets);

    let pre_len = state.continuous_effects.len();
    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBothDynamic {
                amount: Box::new(EA::PlayerCounterCount {
                    player: PlayerTarget::DeclaredTarget { index: 0 },
                    counter: CounterType::Poison,
                }),
                negate: false,
            },
            filter: EffectFilter::Source,
            duration: EffectDuration::WhileSourceOnBattlefield,
            condition: None,
        }),
    };
    execute_effect(&mut state, &effect, &mut ctx);

    let post = state.continuous_effects.iter().nth(pre_len).unwrap();
    match &post.modification {
        LayerModification::ModifyBoth(n) => assert_eq!(
            *n, 9,
            "DeclaredTarget index 0 = p2 with 9 poison counters; sum-of-one is the value"
        ),
        other => panic!("expected ModifyBoth, got {:?}", other),
    }
}

// ── Test 6: Non-Poison counter kinds return 0 (no panic) ────────────────────

#[test]
fn test_player_counter_count_unsupported_kind_returns_zero() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Source", 1, 1).in_zone(ZoneId::Battlefield))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().poison_counters = 12;
    state.players.get_mut(&p2).unwrap().poison_counters = 12;

    let src = find_object(&state, "Source");

    // Try a non-Poison counter type; should resolve to 0 without panicking.
    // We use Energy (a real player-counter kind in MTG, but not yet wired to
    // a flat field on PlayerState) AND a Custom kind to exercise both the
    // enum-variant branch and the wildcard fallback.
    let n_energy = resolve_via_apply(
        &mut state,
        src,
        p1,
        PlayerTarget::EachOpponent,
        CounterType::Energy,
    );
    assert_eq!(
        n_energy, 0,
        "Energy resolves to 0 (no PlayerState::energy_counters today; defensive future-proof contract)"
    );

    let n_custom = resolve_via_apply(
        &mut state,
        src,
        p1,
        PlayerTarget::EachOpponent,
        CounterType::Custom("rad".to_string()),
    );
    assert_eq!(
        n_custom, 0,
        "Custom counter kinds resolve to 0 (no panic for unknown kinds)"
    );
}

// ── Test 7: CDA path — resolve_cda_amount via SetPtDynamic ──────────────────

/// CR 613.4a — `AbilityDefinition::CdaPowerToughness` registers a Layer 7a
/// `SetPtDynamic` continuous effect. The layer system calls
/// `resolve_cda_amount` to compute power and toughness on every
/// `calculate_characteristics`. A `PlayerCounterCount` CDA must flow through
/// that path correctly.
///
/// Note: this test exercises the CDA-evaluation path. It does NOT ship a
/// Layer-7c CDA-style modify (which is the deferred PB-CC-C-followup primitive
/// — Vishgraz's card def). It uses `CdaPowerToughness` as a synthetic harness
/// because that is the only Layer-7a dynamic-CDA registration today; the
/// PB-CC-A engine variant must produce correct values regardless of which
/// CDA-shape calls into it.
#[test]
fn test_player_counter_count_cda_path_layer7a() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    // Build a synthetic source object whose card definition is a CDA reading
    // EachOpponent poison.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(
            ObjectSpec::creature(p1, "Test CDA", 0, 0)
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("test-pcc-cda".to_string())),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.players.get_mut(&p2).unwrap().poison_counters = 3;
    state.players.get_mut(&p3).unwrap().poison_counters = 5;
    // controller p1 has 0 poison

    let src = find_object(&state, "Test CDA");

    // Manually register a SetPtDynamic continuous effect mirroring what
    // `register_static_continuous_effects` does for CdaPowerToughness.
    let amount = EA::PlayerCounterCount {
        player: PlayerTarget::EachOpponent,
        counter: CounterType::Poison,
    };
    state
        .continuous_effects
        .push_back(mtg_engine::ContinuousEffect {
            id: mtg_engine::EffectId(7700),
            source: Some(src),
            timestamp: 7700,
            layer: EffectLayer::PtCda,
            duration: EffectDuration::WhileSourceOnBattlefield,
            filter: EffectFilter::SingleObject(src),
            modification: LayerModification::SetPtDynamic {
                power: Box::new(amount.clone()),
                toughness: Box::new(amount),
            },
            is_cda: true,
            condition: None,
        });

    let chars = calculate_characteristics(&state, src).unwrap();
    assert_eq!(
        chars.power,
        Some(8),
        "Layer 7a CDA SetPtDynamic should sum opponent poison: 3+5=8"
    );
    assert_eq!(chars.toughness, Some(8), "toughness mirrors power");
}

// ── Test 8: CDA sum semantic across distributions (Vishgraz scenario) ───────

/// 4-player game scaling test: opponents' poison goes through 0 / 3 / 8
/// distributions. Validates the SUM semantic at the layer level — single
/// scenario, three game states, three measurements.
#[test]
fn test_player_counter_count_cda_scaling_4p_sum() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(
            ObjectSpec::creature(p1, "Synthetic Vishgraz", 0, 0)
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("synthetic-vishgraz".to_string())),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let src = find_object(&state, "Synthetic Vishgraz");
    let amount = EA::PlayerCounterCount {
        player: PlayerTarget::EachOpponent,
        counter: CounterType::Poison,
    };
    state
        .continuous_effects
        .push_back(mtg_engine::ContinuousEffect {
            id: mtg_engine::EffectId(7800),
            source: Some(src),
            timestamp: 7800,
            layer: EffectLayer::PtCda,
            duration: EffectDuration::WhileSourceOnBattlefield,
            filter: EffectFilter::SingleObject(src),
            modification: LayerModification::SetPtDynamic {
                power: Box::new(amount.clone()),
                toughness: Box::new(amount),
            },
            is_cda: true,
            condition: None,
        });

    // (a) All zero — power/toughness = 0.
    let chars = calculate_characteristics(&state, src).unwrap();
    assert_eq!(chars.power, Some(0), "0+0+0 = 0");
    assert_eq!(chars.toughness, Some(0));

    // (b) Distribute 3 across opponents (p2=1, p3=1, p4=1). Sum = 3.
    state.players.get_mut(&p2).unwrap().poison_counters = 1;
    state.players.get_mut(&p3).unwrap().poison_counters = 1;
    state.players.get_mut(&p4).unwrap().poison_counters = 1;
    let chars = calculate_characteristics(&state, src).unwrap();
    assert_eq!(chars.power, Some(3), "1+1+1 = 3");
    assert_eq!(chars.toughness, Some(3));

    // (c) Distribute 8 unevenly (p2=5, p3=2, p4=1). Sum = 8 (NOT max=5,
    // NOT count-of-poisoned=3).
    state.players.get_mut(&p2).unwrap().poison_counters = 5;
    state.players.get_mut(&p3).unwrap().poison_counters = 2;
    state.players.get_mut(&p4).unwrap().poison_counters = 1;
    let chars = calculate_characteristics(&state, src).unwrap();
    assert_eq!(
        chars.power,
        Some(8),
        "Sum-semantic: 5+2+1 = 8 (NOT max(5)=5, NOT count(>0)=3)"
    );
    assert_eq!(chars.toughness, Some(8));

    // (d) Controller's own poison is irrelevant for EachOpponent.
    state.players.get_mut(&p1).unwrap().poison_counters = 999;
    let chars = calculate_characteristics(&state, src).unwrap();
    assert_eq!(
        chars.power,
        Some(8),
        "controller's 999 poison MUST NOT contribute to EachOpponent"
    );
}
