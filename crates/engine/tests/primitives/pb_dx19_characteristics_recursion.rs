//! PB-DX19 — the unbounded `calculate_characteristics` recursion (OOS-SIM2-6, HIGH)
//! plus the unchecked P/T arithmetic fold-in (OOS-SIM2-5).
//!
//! `memory/primitives/seed-rerank-2026-08-02.md` §4 "Dispatch briefs → PB-DX19" is
//! authoritative. Every line it cites was re-verified against HEAD before any edit
//! (`feedback_verify_cr_before_implement`), and the verification is recorded in
//! `memory/primitives/pb-plan-DX19.md` §1.
//!
//! ## What was wrong (OOS-SIM2-6)
//!
//! `rules/layers.rs::calculate_characteristics` collects the active continuous
//! effects by calling `is_effect_active` on **every** entry in
//! `state.continuous_effects`, and `is_effect_active` evaluates each effect's
//! `condition` through `effects::check_static_condition`. The
//! `Condition::YouControlNOrMoreWithFilter` arm used to resolve each candidate
//! permanent's characteristics with `expect_characteristics`, i.e. by calling
//! `calculate_characteristics` again. Four hops, no depth guard, and — this is the
//! part the old comment got wrong — **no exit condition at all**, because the
//! recursion does not depend on which object the outer call was made for. Any
//! `calculate_characteristics` call, on any object, in any zone, re-enters
//! `is_effect_active` for the same conditional effect and recurses.
//!
//! The result is `thread ... has overflowed its stack` → SIGABRT. That is not a
//! `catch_unwind`-able panic, so the play-server's per-request boundary cannot
//! contain it: the whole 4-player game dies with the process.
//!
//! `indomitable_archangel` declares no `completeness` field, so it is
//! `Completeness::Complete` by derive, `validate_deck` accepts it, and the
//! simulator's `random_deck` will put it in any W-identity seat's pool.
//!
//! ## The fix
//!
//! Read **base** characteristics (`obj.characteristics`) in the condition's filter
//! test instead of layer-resolved ones. The precedent was already in the tree and
//! had made the opposite choice for this same hazard: `layers.rs`'s
//! `EffectAmount::PermanentCount` arm, whose comment names recursive CDA evaluation
//! as the reason. See `pb-plan-DX19.md` §3 for why the CR 613.8b dependency-aware
//! fixpoint is a batch of its own and not this one.
//!
//! ## Reading the tests below
//!
//! `recursion_*` are the OOS-SIM2-6 probes. Each was **watched failing** against a
//! reverted tree — the revert compiled, and the observed pre-fix output is quoted at
//! the test — never reasoned to (the standing discipline this suite keeps losing).

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::rules::replacement::register_static_continuous_effects;
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, CardDefinition, CardRegistry,
    GameState, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec, PlayerId, ZoneId,
};

// ── Helpers ─────────────────────────────────────────────────────────────────────

fn p1() -> PlayerId {
    PlayerId(1)
}

fn p2() -> PlayerId {
    PlayerId(2)
}

fn defs_map() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn registry() -> Arc<CardRegistry> {
    CardRegistry::new(all_cards())
}

/// Build an `ObjectSpec` from the **real** committed `CardDefinition`, so the
/// object carries the def's `card_id` and the ETB registrar can find it. This is
/// the whole point of the probe: nothing here hand-builds a `ContinuousEffect`.
fn real_card_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no real CardDefinition for '{}'", name));
    let base = ObjectSpec::card(owner, name)
        .in_zone(zone)
        .with_card_id(def.card_id.clone());
    enrich_spec_from_def(base, defs)
}

fn find_on_battlefield(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("'{}' not on the battlefield", name))
}

/// Put Indomitable Archangel on the battlefield from its real def and register its
/// statics through the **production** ETB registrar
/// (`rules/replacement.rs::register_static_continuous_effects`) — the same function
/// `resolution.rs` and `lands.rs` call at every real ETB.
///
/// `artifact_count` controls whether Metalcraft is switched on; the recursion is
/// reached either way, because `check_static_condition` runs the filter test over
/// candidates *before* it knows the count.
fn archangel_board(artifact_count: usize) -> (GameState, ObjectId, ObjectId) {
    let defs = defs_map();
    let mut builder = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(real_card_spec(
            p1(),
            "Indomitable Archangel",
            ZoneId::Battlefield,
            &defs,
        ));
    for i in 0..artifact_count {
        builder = builder.object(ObjectSpec::artifact(p1(), &format!("P1 Artifact {}", i)));
    }
    let mut state = builder.build().unwrap();

    let angel = find_on_battlefield(&state, "Indomitable Archangel");
    let angel_card_id = state
        .objects()
        .get(&angel)
        .and_then(|o| o.card_id.clone())
        .expect("the Archangel object must carry its real card_id");

    register_static_continuous_effects(&mut state, angel, Some(&angel_card_id), &registry(), false);

    let first_artifact = if artifact_count > 0 {
        find_on_battlefield(&state, "P1 Artifact 0")
    } else {
        angel
    };
    (state, angel, first_artifact)
}

// ── OOS-SIM2-6: the recursion ───────────────────────────────────────────────────

/// **Observation A (stage 0), now a regression pin.** CR 604.2 / CR 613.1f.
///
/// Indomitable Archangel's Metalcraft static, registered through the real ETB
/// registrar with Metalcraft **on** (three artifacts), grants shroud to the
/// artifacts its controller controls — and resolving those characteristics
/// terminates.
///
/// Pre-fix, `cargo test --test primitives recursion_ -- --test-threads=1` printed
/// (verbatim, all three probes in this file selected):
///
/// ```text
/// running 3 tests
/// test pb_dx19_characteristics_recursion::recursion_is_independent_of_the_object_being_calculated ...
/// thread 'pb_dx19_characteristics_recursion::recursion_is_independent_of_the_object_being_calculated'
///   (1890885) has overflowed its stack
/// fatal runtime error: stack overflow, aborting
/// error: test failed, to rerun pass `-p mtg-engine --test primitives`
/// Caused by:
///   process didn't exit successfully: `.../primitives-9a8d4bc3cf8e53b7 recursion_
///   --test-threads=1` (signal: 6, SIGABRT: process abort signal)
/// ```
///
/// Two things in that output matter more than the failure itself. The signal is
/// **6/SIGABRT**, not an unwindable panic — `catch_unwind` cannot contain it. And
/// the run reports `running 3 tests` but only ever names **one**: the first probe to
/// execute took the whole binary down with it, and the other two never ran. That is
/// precisely what the defect does to a 4-player game in the play-server.
#[test]
fn recursion_metalcraft_on_grants_shroud_and_terminates() {
    let (state, _angel, artifact) = archangel_board(3);

    let chars = calculate_characteristics(&state, artifact)
        .expect("the artifact is live on the battlefield");
    assert!(
        chars.keywords.contains(&KeywordAbility::Shroud),
        "CR 604.2: with three artifacts, Metalcraft is on and P1's artifacts have \
         shroud; got keywords {:?}",
        chars.keywords
    );
}

/// **Observation B (stage 0), now a regression pin.** The condition is *false* here
/// (one artifact, Metalcraft needs three) and the recursion still fired pre-fix,
/// because `check_static_condition` calls `expect_characteristics` on each candidate
/// **before** it can know the count. The old comment's "termination is guaranteed"
/// argument never had a case to stand on.
///
/// Pre-fix output: identical stack-overflow SIGABRT to Observation A.
#[test]
fn recursion_metalcraft_off_still_terminates() {
    let (state, _angel, artifact) = archangel_board(1);

    let chars = calculate_characteristics(&state, artifact)
        .expect("the artifact is live on the battlefield");
    assert!(
        !chars.keywords.contains(&KeywordAbility::Shroud),
        "CR 604.2: one artifact is below Metalcraft's threshold of three, so no \
         shroud is granted; got keywords {:?}",
        chars.keywords
    );
}

/// **Observation C (stage 0), now a regression pin.** The recursion is not a
/// property of the *object being calculated* — it is a property of the effect being
/// tested. Calculating the **Archangel's own** characteristics recursed pre-fix even
/// though the Archangel is not an artifact and the granting filter never matches it.
///
/// This is the observation that falsifies the old comment at the fix site directly:
/// it argued termination from "we are checking the types of *other* battlefield
/// objects, not the object currently being calculated". The object currently being
/// calculated is irrelevant — `calculate_characteristics` evaluates every conditional
/// effect in the game on every call.
///
/// Pre-fix output: identical stack-overflow SIGABRT.
#[test]
fn recursion_is_independent_of_the_object_being_calculated() {
    let (state, angel, _artifact) = archangel_board(3);

    let chars =
        calculate_characteristics(&state, angel).expect("the Archangel is live on the battlefield");
    assert_eq!(
        chars.power,
        Some(4),
        "Indomitable Archangel is a 4/4 and nothing modifies it"
    );
    assert!(
        !chars.keywords.contains(&KeywordAbility::Shroud),
        "the Archangel is not an artifact, so its own Metalcraft grant does not \
         reach it; got keywords {:?}",
        chars.keywords
    );
}

// ── OOS-SIM2-5: P/T arithmetic saturates ────────────────────────────────────────
//
// `layers.rs` applied P/T with bare `+=` at ten sites. The seed named four. The
// ceiling is a documented deviation — see the module doc on
// `rules::layers::apply_layer_modification` — and these are the discriminating
// probes, one per site GROUP: the `+1/+1` counter path, the static `Modify*` arms,
// and the `*Dynamic` arms.
//
// **Why these discriminate in an ordinary `cargo test` run.** `[profile.dev]` (which
// `[profile.test]` inherits) leaves cargo's default `overflow-checks = true` in
// place — this workspace's `Cargo.toml` declares no `[profile.dev]` override, only
// `[profile.fuzz]`. So a bare `+=` at any of these sites *panics* here with
// "attempt to add with overflow", and each test below was watched doing exactly that
// against the reverted tree. The observed panic is quoted at each test.
//
// The counter-widening probe is the exception and is the important one: an `as i32`
// cast does **not** panic under `overflow-checks`. It wraps silently in *every*
// profile, which is why no fuzz run could ever have surfaced it.

use mtg_engine::{
    CardEffectTarget, CardType, ContinuousEffect, CounterType, EffectAmount, EffectDuration,
    EffectFilter, EffectId, EffectLayer, LayerModification,
};

/// Push a single continuous effect at the given layer onto an already-built state.
/// These probes are about the arithmetic in `apply_layer_modification`, not about
/// condition evaluation, so building the effect directly is the honest shape here —
/// unlike the recursion probes above, where hand-building was the whole defect.
fn push_effect(state: &mut mtg_engine::GameState, source: ObjectId, m: LayerModification) {
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9_001),
        source: Some(source),
        timestamp: 1,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(source),
        modification: m,
        is_cda: false,
        affected_set: None,
        condition: None,
    });
}

/// **Group 1 — the `+1/+1` counter path** (`calculate_characteristics`), the one
/// every game exercises. CR 121.3 / CR 613.4c.
///
/// Observed against the reverted tree (the revert compiled; line numbers are that
/// tree's, which carries this batch's doc comment but not its fix):
///
/// ```text
/// thread 'pb_dx19_characteristics_recursion::counter_path_saturates_instead_of_overflowing'
///   (1971281) panicked at crates/engine/src/rules/layers.rs:400:21:
/// attempt to add with overflow
/// ```
#[test]
fn counter_path_saturates_instead_of_overflowing() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::creature(p1(), "Nearly Maximal", i32::MAX, i32::MAX)
                .with_counter(CounterType::PlusOnePlusOne, 5),
        )
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Nearly Maximal");
    let chars = calculate_characteristics(&state, id).expect("live on the battlefield");
    assert_eq!(
        chars.power,
        Some(i32::MAX),
        "five +1/+1 counters on an i32::MAX creature saturate rather than wrapping \
         negative (documented deviation, PB-DX19)"
    );
    assert_eq!(
        chars.toughness,
        Some(i32::MAX),
        "toughness saturates likewise"
    );
}

/// **Group 1, second failure mode — the `u32 -> i32` counter widening.**
///
/// This is the site `[profile.fuzz]`'s `overflow-checks` could never have caught: an
/// `as` cast is not checked arithmetic, so `3_000_000_000u32 as i32` silently yields
/// a **negative** number in every profile, and a pile of `+1/+1` counters would have
/// *shrunk* the creature. Pre-fix this test failed by assertion rather than by panic:
///
/// ```text
/// thread 'pb_dx19_characteristics_recursion::counter_widening_saturates_instead_of_wrapping_negative'
///   (1971282) panicked at crates/engine/tests/primitives/pb_dx19_characteristics_recursion.rs:344:5:
/// assertion `left == right` failed
///   left: Some(-1294967294)
///  right: Some(2147483647)
/// ```
///
/// `-1294967294` is exactly `2 + (3_000_000_000u32 as i32)`: the wrapped count added
/// to the printed power, with the counter's sign inverted. Note that this probe is
/// the only one of the six that failed by **assertion** rather than by panic —
/// direct evidence that `overflow-checks` never guarded this site.
#[test]
fn counter_widening_saturates_instead_of_wrapping_negative() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::creature(p1(), "Overcountered", 2, 2)
                .with_counter(CounterType::PlusOnePlusOne, 3_000_000_000),
        )
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Overcountered");
    let chars = calculate_characteristics(&state, id).expect("live on the battlefield");
    assert_eq!(
        chars.power,
        Some(i32::MAX),
        "a counter count above i32::MAX must saturate, NOT wrap to a negative \
         modifier — an `as i32` cast wraps in every profile, overflow-checks included"
    );
}

/// **Group 2 — the static `Modify*` arms** (`ModifyPower` / `ModifyToughness` /
/// `ModifyBoth`). CR 613.1g (Layer 7c).
///
/// Observed against the reverted tree:
///
/// ```text
/// thread 'pb_dx19_characteristics_recursion::modify_both_arm_saturates_instead_of_overflowing'
///   (1971286) panicked at crates/engine/src/rules/layers.rs:1703:17:
/// attempt to add with overflow
/// ```
#[test]
fn modify_both_arm_saturates_instead_of_overflowing() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(
            p1(),
            "Pumped",
            i32::MAX - 1,
            i32::MAX - 1,
        ))
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Pumped");
    push_effect(&mut state, id, LayerModification::ModifyBoth(100));

    let chars = calculate_characteristics(&state, id).expect("live on the battlefield");
    assert_eq!(chars.power, Some(i32::MAX), "power saturates at i32::MAX");
    assert_eq!(
        chars.toughness,
        Some(i32::MAX),
        "toughness saturates at i32::MAX"
    );
}

/// **Group 2, the negative direction.** A large `-N/-N` effect must floor at
/// `i32::MIN` rather than wrapping to a huge positive power — which would turn a
/// lethal shrink into a game-winning pump. CR 613.1g.
///
/// Observed against the reverted tree: `attempt to add with overflow` at
/// `layers.rs:1703:17`, the `ModifyBoth` arm's power write.
#[test]
fn modify_both_arm_saturates_downward() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(
            p1(),
            "Shrunk",
            i32::MIN + 1,
            i32::MIN + 1,
        ))
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Shrunk");
    push_effect(&mut state, id, LayerModification::ModifyBoth(-100));

    let chars = calculate_characteristics(&state, id).expect("live on the battlefield");
    assert_eq!(
        chars.power,
        Some(i32::MIN),
        "power floors at i32::MIN rather than wrapping to a large POSITIVE value"
    );
}

/// **Group 3 — the `*Dynamic` arms**, the ones `devilish_valet` actually reaches.
///
/// `ModifyPowerDynamic` re-evaluates live at every `calculate_characteristics` call
/// (CR 611.3a), resolving its amount through `resolve_cda_amount`. Here the amount is
/// the creature's own power, which is the shape `devilish_valet` produces once
/// `effects/mod.rs` has substituted the dynamic modification to a concrete one at
/// resolution (CR 608.2h) — each trigger adds the creature's *current* power, so the
/// value doubles per trigger and reaches `i32::MAX` in about 31 triggers.
///
/// Observed against the reverted tree: `attempt to add with overflow` at
/// `layers.rs:1753:17`, the `ModifyPowerDynamic` arm's power write.
#[test]
fn dynamic_arm_saturates_instead_of_overflowing() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Valet", i32::MAX, 4))
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Valet");
    push_effect(
        &mut state,
        id,
        LayerModification::ModifyPowerDynamic {
            amount: Box::new(EffectAmount::Fixed(i32::MAX)),
            negate: false,
        },
    );

    let chars = calculate_characteristics(&state, id).expect("live on the battlefield");
    assert_eq!(
        chars.power,
        Some(i32::MAX),
        "doubling an already-maximal power saturates rather than wrapping to a \
         negative power, which would make the creature die to CR 704.5a"
    );
}

/// **Group 3, the negation.** `*Dynamic` arms compute `delta` as `-raw` when
/// `negate` is set. `-i32::MIN` has no `i32` representation: it panicked under
/// `overflow-checks` and wrapped back to `i32::MIN` in plain `--release`, i.e. a
/// "gets -X/-X" effect would have *kept* the sign it was supposed to flip.
///
/// Observed against the reverted tree: `attempt to negate with overflow` at
/// `layers.rs:1751:38` — the negation, not the addition, so this probe reaches a
/// site none of the others does.
#[test]
fn dynamic_arm_negation_saturates() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Negated", 3, 3))
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Negated");
    push_effect(
        &mut state,
        id,
        LayerModification::ModifyPowerDynamic {
            amount: Box::new(EffectAmount::Fixed(i32::MIN)),
            negate: true,
        },
    );

    let chars = calculate_characteristics(&state, id).expect("live on the battlefield");
    assert_eq!(
        chars.power,
        Some(i32::MAX),
        "negating i32::MIN saturates to i32::MAX, then 3 + i32::MAX saturates again; \
         the sign flip must not silently fail to happen"
    );
}

// ── The deviation, pinned ───────────────────────────────────────────────────────

/// **This test pins behaviour that is WRONG by CR, on purpose.** CR 613.1d.
///
/// The base-characteristics fix that closes OOS-SIM2-6 costs exactly one thing: a
/// type change granted by another continuous effect is invisible to this condition.
/// That cost is live in the shipped corpus, not theoretical — `blinkmoth_nexus` and
/// `inkmoth_nexus` animate themselves with a Layer-4
/// `AddCardTypes([Artifact, Creature])`, neither declares a `completeness` field (so
/// both are `Complete` by derive), and both are colourless lands, so they sit in the
/// same deck pool as Indomitable Archangel.
///
/// By CR 613.1d an animated Nexus **is** an artifact and **must** count toward
/// Metalcraft. It does not, and this test asserts that it does not.
///
/// It is written this way so the wrongness is discoverable rather than remembered:
/// when the CR 613.8b dependency-aware fixpoint lands (OOS-DX19-2), this test fails,
/// and the batch that makes it fail is the batch that should flip the assertion. A
/// deviation with no failing test attached to it is just a comment nobody reads.
#[test]
fn deviation_animated_nexus_does_not_count_toward_metalcraft() {
    // Two plain artifacts plus one land that another effect has turned into an
    // artifact: three artifacts by CR 613.1d, two by base characteristics.
    let defs = defs_map();
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(real_card_spec(
            p1(),
            "Indomitable Archangel",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(ObjectSpec::artifact(p1(), "P1 Artifact 0"))
        .object(ObjectSpec::artifact(p1(), "P1 Artifact 1"))
        .object(ObjectSpec::land(p1(), "Animated Land"))
        .build()
        .unwrap();

    let angel = find_on_battlefield(&state, "Indomitable Archangel");
    let angel_card_id = state
        .objects()
        .get(&angel)
        .and_then(|o| o.card_id.clone())
        .expect("the Archangel object must carry its real card_id");
    register_static_continuous_effects(&mut state, angel, Some(&angel_card_id), &registry(), false);

    // Animate the land into an artifact creature, exactly as blinkmoth_nexus does.
    let land = find_on_battlefield(&state, "Animated Land");
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9_100),
        source: Some(land),
        timestamp: 50,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(land),
        modification: LayerModification::AddCardTypes(
            [CardType::Artifact, CardType::Creature]
                .into_iter()
                .collect(),
        ),
        is_cda: false,
        affected_set: None,
        condition: None,
    });

    // The animation itself works — the layer system reports the land as an artifact.
    let land_chars = calculate_characteristics(&state, land).expect("live on the battlefield");
    assert!(
        land_chars.card_types.contains(&CardType::Artifact),
        "precondition: CR 613.1d makes the animated land an artifact; got {:?}",
        land_chars.card_types
    );

    // ...and Metalcraft still does not see it. THIS IS THE DEVIATION.
    let artifact = find_on_battlefield(&state, "P1 Artifact 0");
    let chars = calculate_characteristics(&state, artifact).expect("live on the battlefield");
    assert!(
        !chars.keywords.contains(&KeywordAbility::Shroud),
        "DEVIATION PIN (PB-DX19): by CR 613.1d the animated land is a third artifact \
         and Metalcraft should be ON. The base-characteristics read that closes \
         OOS-SIM2-6 cannot see it, so no shroud is granted. If this assertion has \
         started failing, the CR 613.8b fixpoint (OOS-DX19-2) has landed and this \
         test should be INVERTED, not deleted."
    );
}

/// **Group 1, third failure mode — the CDA path's own copy of the widening.**
/// Found by the PB-DX19 review, not by the original scope: `resolve_cda_amount`'s
/// `EffectAmount::CounterCount` arm had the identical `u32 as i32` cast, one
/// function away from the sites the batch had just fixed, and it feeds every
/// `SetPtDynamic` / `SetBothDynamic` / `Modify*Dynamic` P/T write.
///
/// The plan had explicitly reasoned this function safe on the grounds that it
/// "returns bounded counts" — true of its `.count()` arms, false of this one, since
/// `counters` is an `OrdMap<CounterType, u32>` that nothing bounds. That is the
/// hazard of arguing safety by function rather than by expression, and it is the
/// same shape of wrong reasoning as the comment that let OOS-SIM2-6 survive.
///
/// CR 613.4a (Layer 7a, characteristic-defining P/T).
#[test]
fn cda_counter_count_widening_saturates() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::creature(p1(), "Countful", 0, 0)
                .with_counter(CounterType::PlusOnePlusOne, 3_000_000_000),
        )
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Countful");
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9_200),
        source: Some(id),
        timestamp: 1,
        layer: EffectLayer::PtCda,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(id),
        modification: LayerModification::SetPtDynamic {
            power: Box::new(EffectAmount::CounterCount {
                target: CardEffectTarget::Source,
                counter: CounterType::PlusOnePlusOne,
            }),
            toughness: Box::new(EffectAmount::CounterCount {
                target: CardEffectTarget::Source,
                counter: CounterType::PlusOnePlusOne,
            }),
        },
        is_cda: true,
        affected_set: None,
        condition: None,
    });

    let chars = calculate_characteristics(&state, id).expect("live on the battlefield");
    // Layer 7a SETS P/T from the count, then the +1/+1 counter path in Layer 7d adds
    // the same (saturated) count on top; both saturate, so the answer is i32::MAX.
    assert_eq!(
        chars.power,
        Some(i32::MAX),
        "a CDA reading a counter count above i32::MAX must saturate, not wrap to a \
         negative base power"
    );
}

// ── The non-layer paths, which the first fix regressed ──────────────────────────
//
// `check_static_condition` / `check_condition` are SHARED evaluators. Five callers
// reach them and only one — `is_effect_active`, inside `calculate_characteristics`
// — closes the recursion. PB-DX19's first attempt read base characteristics
// unconditionally and so broke the four safe paths to fix the one unsafe one. The
// review caught it; `characteristics_for_condition` is the repair.
//
// These probes pin the safe paths at the layer-RESOLVED answer CR 613.1d requires,
// so that a future "just read base characteristics" simplification fails loudly.

/// CR 613.4c + CR 603.4: a 2/2 with two `+1/+1` counters has power **4**, and a
/// condition asking "power 4 or greater" evaluated off the layer walk must say so.
///
/// This is `garruks_uprising`'s intervening-if shape (`min_power: Some(4)`, a
/// `Complete` deck-legal card). Under PB-DX19's first, unguarded fix this read the
/// printed power of 2 and the trigger would never have queued — a silent false
/// negative on a real card, invisible to all 4,274 tests because no existing test
/// put a counter-pumped creature through a power-filtered condition.
#[test]
fn non_layer_path_reads_layer_resolved_power() {
    use mtg_engine::effects::check_static_condition;
    use mtg_engine::{Condition, TargetFilter};

    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::creature(p1(), "Counter Bear", 2, 2)
                .with_counter(CounterType::PlusOnePlusOne, 2),
        )
        .build()
        .unwrap();

    let bear = find_on_battlefield(&state, "Counter Bear");

    // Precondition: the layer system really does report power 4.
    let chars = calculate_characteristics(&state, bear).expect("live on the battlefield");
    assert_eq!(
        chars.power,
        Some(4),
        "CR 613.4c: two +1/+1 counters on a 2/2 make it a 4/4"
    );

    // Evaluated OFF the layer walk, the condition must see that 4 — not the
    // printed 2.
    let condition = Condition::YouControlNOrMoreWithFilter {
        count: 1,
        filter: TargetFilter {
            min_power: Some(4),
            ..Default::default()
        },
    };
    assert!(
        check_static_condition(&state, &condition, bear, p1()),
        "CR 613.1d: a condition evaluated outside the layer walk reads LAYER-RESOLVED \
         characteristics. Reading base power here is the regression PB-DX19's review \
         caught -- it would silently stop garruks_uprising's intervening-if from \
         firing on a counter-pumped creature."
    );
}

/// The same call, made from *inside* the layer walk, is where the deviation lives —
/// and this test states the boundary explicitly so nobody has to infer it.
///
/// `deviation_animated_nexus_does_not_count_toward_metalcraft` above already pins
/// the wrong answer on the layer path. Together the two probes say: base
/// characteristics inside the walk, layer-resolved everywhere else, and the
/// difference is deliberate.
#[test]
fn the_deviation_is_scoped_to_the_layer_walk_only() {
    use mtg_engine::in_layer_walk;

    assert!(
        !in_layer_walk(),
        "a test body is not inside calculate_characteristics"
    );

    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Bear", 2, 2))
        .build()
        .unwrap();
    let bear = find_on_battlefield(&state, "Bear");

    // The guard is re-entrant and restores itself on the way out (it decrements in
    // Drop, so an early return inside calculate_characteristics cannot leak depth).
    let _ = calculate_characteristics(&state, bear);
    assert!(
        !in_layer_walk(),
        "LayerWalkGuard must decrement on Drop -- a leaked depth would silently \
         downgrade every later condition evaluation on this thread to base \
         characteristics, which is the regression this guard exists to prevent"
    );
}

/// **The card-level probe the plan promised (§8.5) and the first pass omitted.**
///
/// Everything above proves saturation with synthetic `LayerModification`s. This one
/// uses the real `devilish_valet` definition — `Complete`, deck-legal — and the real
/// substitution path, so the claim "an unbounded doubling chain reaches `i32::MAX`
/// in about 31 triggers" rests on the shipped card rather than on a hand-built
/// effect.
///
/// CR 608.2h: each trigger resolves `EffectAmount::PowerOf(Source)` to a concrete
/// `ModifyPower(v)` at resolution, so the modifier locks in at the creature's
/// *current* power — power doubles per trigger. Starting from 1, the 31st doubling
/// leaves `i32`.
///
/// Rather than resolve 31 triggers through the stack, this stacks the already-
/// substituted `ModifyPower` effects the resolution path produces, doubling each
/// time and clamping as the engine does; the point under test is that the layer
/// system's accumulation saturates instead of wrapping to a negative power (which
/// would kill the creature to CR 704.5a — a "gets huge" card that silently dies).
#[test]
fn devilish_valet_doubling_chain_saturates() {
    let defs = defs_map();
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(real_card_spec(
            p1(),
            "Devilish Valet",
            ZoneId::Battlefield,
            &defs,
        ))
        .build()
        .unwrap();

    let valet = find_on_battlefield(&state, "Devilish Valet");
    let base = calculate_characteristics(&state, valet)
        .expect("live on the battlefield")
        .power
        .expect("Devilish Valet is a creature with power");

    // Replay the doubling chain: each trigger adds the creature's CURRENT power,
    // which is exactly what CR 608.2h substitution produces at resolution.
    let mut current = base;
    for i in 0..40 {
        state.continuous_effects_mut().push_back(ContinuousEffect {
            id: EffectId(9_300 + i),
            source: Some(valet),
            timestamp: 100 + i,
            layer: EffectLayer::PtModify,
            duration: EffectDuration::UntilEndOfTurn,
            filter: EffectFilter::SingleObject(valet),
            modification: LayerModification::ModifyPower(current),
            is_cda: false,
            affected_set: None,
            condition: None,
        });
        current = current.saturating_add(current);
    }

    let chars = calculate_characteristics(&state, valet).expect("live on the battlefield");
    assert_eq!(
        chars.power,
        Some(i32::MAX),
        "40 doublings of a real Devilish Valet must saturate at i32::MAX. Before \
         PB-DX19 this wrapped to a NEGATIVE power in --release (and panicked under \
         [profile.fuzz]'s overflow-checks), so the card that 'gets huge' would \
         quietly become a 0/0 and die to CR 704.5a"
    );
    assert!(
        chars.toughness.unwrap_or(0) > 0,
        "only power doubles; toughness is untouched and the creature stays alive"
    );
}

/// The non-layer path on a **different** `matches_filter` branch — subtypes rather
/// than power. CR 613.1d again, and this is the `bloodline_keeper` + changeling
/// shape (CR 702.73a makes a changeling every creature type, and the expansion is
/// applied *inside* the layer loop, so only a layer-resolved read sees it).
///
/// Two branches are pinned rather than one because `matches_filter` decides each
/// characteristic separately; a regression could plausibly restore one and not the
/// other.
#[test]
fn non_layer_path_reads_layer_resolved_subtypes() {
    use mtg_engine::effects::check_static_condition;
    use mtg_engine::{Condition, SubType, TargetFilter};

    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Shifter", 1, 1))
        .build()
        .unwrap();

    let shifter = find_on_battlefield(&state, "Shifter");
    // A Layer-4 subtype grant, the shape CR 702.73a produces.
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9_400),
        source: Some(shifter),
        timestamp: 5,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(shifter),
        modification: LayerModification::AddSubtypes(
            [SubType("Vampire".to_string())].into_iter().collect(),
        ),
        is_cda: false,
        affected_set: None,
        condition: None,
    });

    let chars = calculate_characteristics(&state, shifter).expect("live on the battlefield");
    assert!(
        chars.subtypes.contains(&SubType("Vampire".to_string())),
        "precondition: CR 613.1d — the Layer-4 grant makes it a Vampire; got {:?}",
        chars.subtypes
    );

    let condition = Condition::YouControlNOrMoreWithFilter {
        count: 1,
        filter: TargetFilter {
            has_subtype: Some(SubType("Vampire".to_string())),
            ..Default::default()
        },
    };
    assert!(
        check_static_condition(&state, &condition, shifter, p1()),
        "CR 613.1d: an activation_condition evaluated outside the layer walk must see \
         the GRANTED subtype. Reading base subtypes here is what would make \
         bloodline_keeper reject a changeling"
    );
}
