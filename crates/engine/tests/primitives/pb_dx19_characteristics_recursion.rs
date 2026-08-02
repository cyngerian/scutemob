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
