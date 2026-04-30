//! Tests for PB-CC-C-followup: `AbilityDefinition::CdaModifyPowerToughness` —
//! Layer-7c dynamic CDA modification with continuous re-evaluation (CR 611.3a).
//!
//! Contrast with PB-CC-C's substitution path (CR 608.2h): spell-effect dynamic
//! modifications are locked at resolution. This primitive enables static abilities
//! of permanents to continuously re-evaluate their Layer-7c P/T modifiers.
//!
//! Engine surface:
//! - `card_definition.rs`: `AbilityDefinition::CdaModifyPowerToughness` (disc 76)
//! - `replacement.rs`: `register_static_continuous_effects` arm for the new variant.
//!   Stores `ModifyPowerDynamic` / `ModifyToughnessDynamic` / `ModifyBothDynamic`
//!   with `is_cda: true` and `condition: None` per CR 604.3a(5).
//! - `layers.rs`: `apply_layer_modification` — three Dynamic arms now call
//!   `resolve_cda_amount` live instead of debug_assert!-ing
//! - `hash.rs`: disc 76 arm for `CdaModifyPowerToughness`; HASH_SCHEMA_VERSION = 13
//!
//! Card defs updated: Vishgraz the Doomhive + Exuberant Fuseling (both were
//! Option-B deferrals from PB-CC-C and PB-CC-A).
//!
//! Out of scope: Fuseling's `WheneverCreatureOrArtifactDies` death trigger
//! (separate primitive).
//!
//! Tests:
//!   (a) PB-CC-C T5 regression — `test_modify_power_dynamic_x_locked_at_resolution`
//!       passes UNMODIFIED after PB-CC-C-followup engine changes.
//!       (Confirmed by `cargo test --test primitive_pb_cc_c`; no code here.)
//!   (b) `test_cda_modify_power_toughness_re_evaluates_after_counter_mutation`
//!   (c) `test_vishgraz_scales_with_opponent_poison_counters`
//!   (d) `test_exuberant_fuseling_power_scales_with_oil_counters`
//!   (e) `test_hash_schema_version_after_pb_cc_c_followup` + hash-determinism checks

use mtg_engine::{
    all_cards, calculate_characteristics, card_name_to_id, enrich_spec_from_def, CardDefinition,
    CardId, CardRegistry, ContinuousEffect, CounterType, EffectAmount, EffectDuration,
    EffectFilter, EffectId, EffectLayer, GameStateBuilder, LayerModification, ObjectId, ObjectSpec,
    PlayerId, Step, ZoneId, HASH_SCHEMA_VERSION,
};
use std::collections::HashMap;

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

// ── Test (b): Static-ability re-evaluates after counter mutation ──────────────

/// CR 611.3a — static-ability continuous effect is not locked in; it re-evaluates
/// at every `calculate_characteristics` call.
/// CR 613.4c — Layer 7c modifies (does not set) P/T.
/// CR 122.1 — counter counts are game state, not characteristics.
///
/// Discriminating assertion: power changes between two `calculate_characteristics`
/// calls separated by a counter mutation. If the substitution path were accidentally
/// taken, the value would be locked at the first snapshot.
#[test]
fn test_cda_modify_power_toughness_re_evaluates_after_counter_mutation() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p1, "Oil Creature", 0, 1)
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("synthetic-oil-cda".to_string())),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Oil Creature");

    // Register a ContinuousEffect with ModifyPowerDynamic + is_cda=true at Layer 7c.
    // This mirrors what register_static_continuous_effects does for
    // AbilityDefinition::CdaModifyPowerToughness { power: Some(CounterCount{Source, Oil}), .. }.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(8800),
        source: Some(creature_id),
        timestamp: 8800,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(creature_id),
        modification: LayerModification::ModifyPowerDynamic {
            amount: Box::new(EffectAmount::CounterCount {
                target: mtg_engine::CardEffectTarget::Source,
                counter: CounterType::Oil,
            }),
            negate: false,
        },
        is_cda: true,
        condition: None,
    });

    // Step 1: 0 oil counters. Power = base(0) + 0 = 0.
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert_eq!(chars.power, Some(0), "0 oil counters: power = 0");
    assert_eq!(chars.toughness, Some(1), "toughness unchanged at 1");

    // Step 2: mutate to 2 oil counters. Re-read — power must change.
    // CR 611.3a: "isn't locked in; applies at any given moment to whatever its text indicates."
    {
        let obj = state.objects.get_mut(&creature_id).unwrap();
        obj.counters.insert(CounterType::Oil, 2);
    }
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert_eq!(
        chars.power,
        Some(2),
        "CR 611.3a: 2 oil counters → power = 2 (re-evaluated, not locked)"
    );
    assert_eq!(chars.toughness, Some(1), "toughness unchanged");

    // Step 3: mutate to 5 oil counters. Power must follow.
    {
        let obj = state.objects.get_mut(&creature_id).unwrap();
        obj.counters.insert(CounterType::Oil, 5);
    }
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert_eq!(
        chars.power,
        Some(5),
        "CR 611.3a: 5 oil counters → power = 5 (re-evaluated again)"
    );

    // Step 4: remove all counters. Power returns to 0.
    {
        let obj = state.objects.get_mut(&creature_id).unwrap();
        obj.counters.remove(&CounterType::Oil);
    }
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert_eq!(
        chars.power,
        Some(0),
        "CR 611.3a: 0 oil counters → power = 0 (down-scaling works)"
    );
}

// ── Test (c): Vishgraz scales across multi-opponent poison counters ────────────

/// CR 122.1 — "each poison counter your opponents have" sums across ALL opponents.
/// Vishgraz 2023-02-04 ruling: sum (NOT count-of-poisoned, NOT max).
/// CR 611.3a — static ability, not locked-in.
/// CR 613.4c — Layer 7c modify.
///
/// Multi-opponent game: p2, p3, p4 are opponents of p1 (Vishgraz's controller).
/// Step 1: 0 poison everywhere → P/T = base 3/3.
/// Step 2: p2=1, p3=0, p4=2 → sum=3 → P/T = 3+3 = 6/6.
/// Step 3: p2=5, p3=2, p4=1 → sum=8 → P/T = 3+8 = 11/11.
/// Step 4: p1=4 (controller) → does NOT count → P/T stays 11/11.
#[test]
fn test_vishgraz_scales_with_opponent_poison_counters() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // Load Vishgraz card def.
    let defs: HashMap<String, CardDefinition> = all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect();

    let vishgraz_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Vishgraz, the Doomhive")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Vishgraz, the Doomhive")),
        &defs,
    );

    let registry = CardRegistry::new(defs.values().cloned().collect::<Vec<_>>());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(std::sync::Arc::clone(&registry))
        .object(vishgraz_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let vishgraz_id = find_object(&state, "Vishgraz, the Doomhive");

    // The builder places objects directly on the battlefield, bypassing the ETB code path.
    // We must call register_static_continuous_effects manually to simulate ETB registration.
    let card_id = state
        .objects
        .get(&vishgraz_id)
        .and_then(|o| o.card_id.clone());
    mtg_engine::rules::replacement::register_static_continuous_effects(
        &mut state,
        vishgraz_id,
        card_id.as_ref(),
        &registry,
    );

    // Step 1: 0 poison on all players. Vishgraz's CDA contributes +0/+0.
    // Base P/T = 3/3. Expected 3/3.
    let chars = calculate_characteristics(&state, vishgraz_id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "Step 1: 0 opponent poison → power = base 3"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "Step 1: 0 opponent poison → toughness = base 3"
    );

    // Step 2: p2=1, p3=0, p4=2 → sum=3. Expected P/T = 3+3 = 6/6.
    let mut state = state;
    {
        let ps = state.players.get_mut(&p2).unwrap();
        ps.poison_counters = 1;
    }
    {
        let ps = state.players.get_mut(&p4).unwrap();
        ps.poison_counters = 2;
    }
    let chars = calculate_characteristics(&state, vishgraz_id).unwrap();
    assert_eq!(
        chars.power,
        Some(6),
        "Step 2: p2=1,p3=0,p4=2 → sum=3 → power=3+3=6 (SUM semantic, not max=2 or count=2)"
    );
    assert_eq!(chars.toughness, Some(6), "Step 2: toughness=6");

    // Step 3: p2=5, p3=2, p4=1 → sum=8. Expected P/T = 3+8 = 11/11.
    // Discriminating choice: sum=8, count-of-poisoned=3, max=5.
    {
        let ps = state.players.get_mut(&p2).unwrap();
        ps.poison_counters = 5;
    }
    {
        let ps = state.players.get_mut(&p3).unwrap();
        ps.poison_counters = 2;
    }
    {
        let ps = state.players.get_mut(&p4).unwrap();
        ps.poison_counters = 1;
    }
    let chars = calculate_characteristics(&state, vishgraz_id).unwrap();
    assert_eq!(
        chars.power,
        Some(11),
        "Step 3: p2=5,p3=2,p4=1 → sum=8 → power=3+8=11 \
         (Vishgraz ruling 2023-02-04: SUM not max=5 or count=3)"
    );
    assert_eq!(chars.toughness, Some(11), "Step 3: toughness=11");

    // Step 4: give p1 (controller) 4 poison counters. EachOpponent excludes controller.
    // Expected: P/T still 11/11 (p1's own poison ignored).
    {
        let ps = state.players.get_mut(&p1).unwrap();
        ps.poison_counters = 4;
    }
    let chars = calculate_characteristics(&state, vishgraz_id).unwrap();
    assert_eq!(
        chars.power,
        Some(11),
        "Step 4: controller's own poison (4) must NOT count — EachOpponent excludes p1"
    );
    assert_eq!(
        chars.toughness,
        Some(11),
        "Step 4: toughness unchanged at 11"
    );
}

// ── Test (d): Exuberant Fuseling power scales with oil counters ───────────────

/// CR 611.3a — static ability, not locked-in.
/// CR 613.4c — Layer 7c modify (power only; toughness unchanged).
///
/// Fuseling oracle: "This creature gets +1/+0 for each oil counter on it."
/// Toughness must always stay at base 1 — single-axis power-only modifier.
///
/// Note: Fuseling's ETB AddCounter trigger is NOT exercised here. We zero the
/// counter after construction to make the test deterministic.
#[test]
fn test_exuberant_fuseling_power_scales_with_oil_counters() {
    let p1 = p(1);

    let defs: HashMap<String, CardDefinition> = all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect();

    let fuseling_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Exuberant Fuseling")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Exuberant Fuseling")),
        &defs,
    );

    let registry = CardRegistry::new(defs.values().cloned().collect::<Vec<_>>());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(std::sync::Arc::clone(&registry))
        .object(fuseling_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let fuseling_id = find_object(&state, "Exuberant Fuseling");

    // The builder places objects directly on the battlefield, bypassing the ETB code path.
    // We must call register_static_continuous_effects manually to simulate ETB registration.
    let card_id = state
        .objects
        .get(&fuseling_id)
        .and_then(|o| o.card_id.clone());
    mtg_engine::rules::replacement::register_static_continuous_effects(
        &mut state,
        fuseling_id,
        card_id.as_ref(),
        &registry,
    );

    // Zero oil counters explicitly to make this test deterministic
    // (ETB trigger may have fired depending on build path).
    {
        let obj = state.objects.get_mut(&fuseling_id).unwrap();
        obj.counters.remove(&CounterType::Oil);
    }

    // Step 1: 0 oil counters. Power = base(0) + 0 = 0. Toughness = 1.
    let chars = calculate_characteristics(&state, fuseling_id).unwrap();
    assert_eq!(chars.power, Some(0), "0 oil counters: power = 0");
    assert_eq!(
        chars.toughness,
        Some(1),
        "toughness always 1 (single-axis power-only)"
    );

    // Step 2: 1 oil counter. Power = 0 + 1 = 1.
    {
        let obj = state.objects.get_mut(&fuseling_id).unwrap();
        obj.counters.insert(CounterType::Oil, 1);
    }
    let chars = calculate_characteristics(&state, fuseling_id).unwrap();
    assert_eq!(chars.power, Some(1), "1 oil counter: power = 1");
    assert_eq!(chars.toughness, Some(1), "toughness unchanged");

    // Step 3: 3 oil counters. Power = 0 + 3 = 3.
    {
        let obj = state.objects.get_mut(&fuseling_id).unwrap();
        obj.counters.insert(CounterType::Oil, 3);
    }
    let chars = calculate_characteristics(&state, fuseling_id).unwrap();
    assert_eq!(chars.power, Some(3), "3 oil counters: power = 3");
    assert_eq!(chars.toughness, Some(1), "toughness unchanged at 1");

    // Step 4: set counter to 1 (down-scaling from 3 → 1 via overwrite). Power = 0 + 1 = 1.
    {
        let obj = state.objects.get_mut(&fuseling_id).unwrap();
        obj.counters.insert(CounterType::Oil, 1);
    }
    let chars = calculate_characteristics(&state, fuseling_id).unwrap();
    assert_eq!(
        chars.power,
        Some(1),
        "CR 611.3a: down-scaling — 1 oil counter → power = 1"
    );
    assert_eq!(chars.toughness, Some(1), "toughness unchanged");
}

// ── Test (e): Hash determinism and HASH_SCHEMA_VERSION sentinel ───────────────

/// Hash infrastructure — PB-CC-C-followup bumped HASH_SCHEMA_VERSION 12→13.
///
/// (e-1) Schema-version sentinel (catches uncommitted hash changes).
/// (e-2) Determinism: two states with identical CdaModify ContinuousEffect → same hash.
/// (e-3) Distinct inner amounts hash distinctly.
/// (e-4) is_cda=true vs is_cda=false produce different hashes (is_cda IS hashed).
/// (e-5) CdaModifyPowerToughness AbilityDefinition arms — power/toughness fields
///       contribute to the hash independently.
#[test]
fn test_hash_schema_version_after_pb_cc_c_followup() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    // (e-1) Sentinel: must be exactly 13.
    assert_eq!(
        HASH_SCHEMA_VERSION, 13u8,
        "PB-CC-C-followup bumped HASH_SCHEMA_VERSION 12→13 (AbilityDefinition::CdaModifyPowerToughness \
         disc 76, CR 611.3a). If you bumped again, update this test and state/hash.rs history."
    );

    let hash_effect = |eff: &ContinuousEffect| -> [u8; 32] {
        let mut h = Hasher::new();
        eff.hash_into(&mut h);
        *h.finalize().as_bytes()
    };

    // (e-2) Determinism: two effects with identical fields → same hash.
    let make_oil_effect = |id: u64, src: ObjectId| -> ContinuousEffect {
        ContinuousEffect {
            id: EffectId(id),
            source: Some(src),
            timestamp: id,
            layer: EffectLayer::PtModify,
            duration: EffectDuration::WhileSourceOnBattlefield,
            filter: EffectFilter::SingleObject(src),
            modification: LayerModification::ModifyPowerDynamic {
                amount: Box::new(EffectAmount::CounterCount {
                    target: mtg_engine::CardEffectTarget::Source,
                    counter: CounterType::Oil,
                }),
                negate: false,
            },
            is_cda: true,
            condition: None,
        }
    };
    let src = ObjectId(42);
    let eff_a = make_oil_effect(100, src);
    let eff_b = make_oil_effect(100, src);
    assert_eq!(
        hash_effect(&eff_a),
        hash_effect(&eff_b),
        "(e-2) Determinism: identical ContinuousEffect structs must hash equally"
    );

    // (e-3) Distinct inner amounts hash distinctly.
    let eff_oil = make_oil_effect(100, src);
    let eff_p1p1 = ContinuousEffect {
        modification: LayerModification::ModifyPowerDynamic {
            amount: Box::new(EffectAmount::CounterCount {
                target: mtg_engine::CardEffectTarget::Source,
                counter: CounterType::PlusOnePlusOne,
            }),
            negate: false,
        },
        ..make_oil_effect(100, src)
    };
    let eff_poison = ContinuousEffect {
        modification: LayerModification::ModifyBothDynamic {
            amount: Box::new(EffectAmount::PlayerCounterCount {
                player: mtg_engine::PlayerTarget::EachOpponent,
                counter: CounterType::Poison,
            }),
            negate: false,
        },
        ..make_oil_effect(100, src)
    };
    let h_oil = hash_effect(&eff_oil);
    let h_p1p1 = hash_effect(&eff_p1p1);
    let h_poison = hash_effect(&eff_poison);
    assert_ne!(h_oil, h_p1p1, "(e-3) Oil != P1P1 counter amounts");
    assert_ne!(h_oil, h_poison, "(e-3) Oil != PlayerCounterCount(Poison)");
    assert_ne!(h_p1p1, h_poison, "(e-3) P1P1 != Poison");

    // (e-4) is_cda=true vs is_cda=false produce different hashes.
    let eff_cda_true = make_oil_effect(100, src);
    let eff_cda_false = ContinuousEffect {
        is_cda: false,
        ..make_oil_effect(100, src)
    };
    assert_ne!(
        hash_effect(&eff_cda_true),
        hash_effect(&eff_cda_false),
        "(e-4) is_cda=true must hash differently from is_cda=false (is_cda IS hashed at line ~1515)"
    );

    // (e-5) AbilityDefinition::CdaModifyPowerToughness — power vs toughness fields distinguish.
    let hash_ability = |ab: &mtg_engine::AbilityDefinition| -> [u8; 32] {
        let mut h = Hasher::new();
        ab.hash_into(&mut h);
        *h.finalize().as_bytes()
    };
    let ab_power_only = mtg_engine::AbilityDefinition::CdaModifyPowerToughness {
        power: Some(EffectAmount::Fixed(3)),
        toughness: None,
    };
    let ab_toughness_only = mtg_engine::AbilityDefinition::CdaModifyPowerToughness {
        power: None,
        toughness: Some(EffectAmount::Fixed(3)),
    };
    let ab_both = mtg_engine::AbilityDefinition::CdaModifyPowerToughness {
        power: Some(EffectAmount::Fixed(3)),
        toughness: Some(EffectAmount::Fixed(3)),
    };
    let h_po = hash_ability(&ab_power_only);
    let h_to = hash_ability(&ab_toughness_only);
    let h_both = hash_ability(&ab_both);
    assert_ne!(
        h_po, h_to,
        "(e-5) power-only and toughness-only CdaModifyPowerToughness must hash distinctly"
    );
    assert_ne!(h_po, h_both, "(e-5) power-only != both-fields");
    assert_ne!(h_to, h_both, "(e-5) toughness-only != both-fields");
}
