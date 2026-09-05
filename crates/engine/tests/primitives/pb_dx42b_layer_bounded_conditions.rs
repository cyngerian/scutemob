//! PB-DX42b — layer-bounded condition evaluation (`OOS-ADJ-1` ≡ `OOS-DX19-2`, plus
//! `OOS-DX19-1`'s residue).
//!
//! Authority: `docs/audits/mtg-characteristics-recursion-adjudication.md` §3.2(iii),
//! §3.3, §5.2; `memory/primitives/pb-plan-DX42b.md` §5 items 2-6 and 9 (this file
//! covers the ENGINE half of that list — items 4, 5, 6, plus the wrong-way-round pin
//! for §3's labelled deviation and the retired-assert successor probe; items 2/3, the
//! CHANNEL probes, live in `crates/simulator/tests/pb_dx42b_metalcraft_channel.rs`;
//! item 9, the `OOS-ADJ-2` rider, lives in
//! `crates/engine/tests/core/pb_dx42a_continuous_condition_roster.rs`). Items 1, 7
//! and 8 shipped with the engine half and are NOT repeated here — see
//! `pb_dx19_characteristics_recursion.rs`'s "PB-DX42b addendum" module-doc section.
//!
//! ## The defect in one paragraph
//!
//! `characteristics_for_condition` used to return **printed** characteristics for
//! ANY condition evaluated anywhere inside a `calculate_characteristics` walk,
//! because the only thing it could consult was an ambient `thread_local!` depth
//! counter saying "somewhere inside the layer system" and nothing more — a CR 613.1d
//! deviation on every OTHER conditional effect that happened to be resolving at the
//! same time, not just the one self-referential effect the original fix needed to
//! guard against. `CharacteristicEvalContext` (`rules::layers.rs`) replaces the
//! counter with an explicit, per-`EffectId` `in_flight` set plus a `bound: Option
//! <EffectLayer>` marking how far a nested query may resolve, so suppression is
//! scoped to the ONE effect whose OWN condition is being evaluated, and a nested
//! query resolves through the SPECIFIC layer its filter needs rather than through
//! whatever the outer walk happens to be bounded at.
//!
//! ## CR citations (verified against the MCP rules server before this file was
//! written; two corrections against citations already standing in `layers.rs` and
//! this batch's own plan, both flagged in the close-out report rather than
//! propagated here)
//!
//! - **CR 613.1d** (Layer 4, type-changing effects): "Layer 4: Type-changing effects
//!   are applied. These include effects that change an object's card type, subtype,
//!   and/or supertype."
//! - **CR 604.2** (conditional continuous effects, the "as long as" mechanism a
//!   static ability like Metalcraft relies on): "Static abilities create continuous
//!   effects ... These effects are active as long as the permanent with the ability
//!   remains on the battlefield and has the ability ..."
//! - **"Metalcraft" has no CR-numbered definition at all** — it is an *ability word*
//!   (CR 207.2c: ability words "have no special rules meaning and no individual
//!   entries in the Comprehensive Rules"), governed generically by CR 604.2 above.
//!   `crates/card-defs/src/defs/indomitable_archangel.rs`'s own comment cites
//!   "CR 702.45a (Metalcraft)" — **wrong**; CR 702.45a is Bushido. Not fixed here
//!   (out of this file's scope, a card-def comment), reported at close-out.
//! - **CR 702.18a** (Shroud): "'Shroud' means 'This permanent or player can't be the
//!   target of spells or abilities.'"
//! - **CR 712.8d/712.8e** (double-faced permanent characteristics): "712.8d: While a
//!   double-faced permanent has its front face up, it has only the characteristics
//!   of its front face." / "712.8e: While a nonmodal double-faced permanent has its
//!   back face up, it has only the characteristics of its back face."
//! - **CR 613.8a / CR 613.8b** (the dependency/timestamp tiebreak that does NOT
//!   govern the labelled deviation this file pins wrong-way-round): CR 613.8a's own
//!   text is a single rule with an internal (a)/(b)/(c) list — **"CR 613.8a(a)" is
//!   not a real citation form** (both `layers.rs` and this batch's own plan use it;
//!   flagged at close-out, not fixed here). Cited as plain **CR 613.8a** below.
//!   613.8a's list item (a) is the "same layer" confinement clause; 613.8b is the
//!   timestamp tiebreak for effects that DO depend on each other.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::rules::replacement::register_static_continuous_effects;
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, CardDefinition, CardRegistry,
    CardType, Condition, ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer,
    GameState, GameStateBuilder, KeywordAbility, LayerModification, ObjectId, ObjectSpec, PlayerId,
    TargetFilter, ZoneId,
};

// ── Helpers (mirrors `pb_dx19_characteristics_recursion.rs`'s shape) ────────────

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
/// object carries the def's `card_id` and the ETB registrar can find it.
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

// ── Item 4: `thaumatic_compass`'s DFC face swap is a PRE-LOOP base rewrite ──────
//
// CR 712.8d/712.8e — `layers.rs`'s `if obj.is_transformed { ... }` block runs BEFORE
// the layer loop and BEFORE the activity sweep this batch bounds, so it is a
// completely different code path from the Layer-4 `ContinuousEffectDef` supply
// sources the channel probes (item 2/3) exercise
// (`pb-DX42b-stage0-census.md` §3c states this explicitly). Front face is an
// Artifact; back face ("Spires of Orazca") is a Land.
//
// **Correction, found by executing the depth-counter revert rather than assumed:**
// the TRANSFORMED probe below is NOT independent of this batch's fix the way the
// module doc above first suggested. Reverting `characteristics_for_condition_ctx`
// to the retired ambient-depth-counter shape (base characteristics for ANY object
// queried while inside a layer walk) reddens
// `thaumatic_compass_transformed_stops_feeding_metalcraft` too: under that revert,
// the CANDIDATE resolution never calls `calculate_characteristics_through` at all
// (it just clones `obj.characteristics`), so the PRE-LOOP DFC rewrite -- which only
// runs INSIDE that function -- never executes for the Compass candidate, and a
// transformed Compass is still read as its printed FRONT face (Artifact). The
// UNTRANSFORMED probe does NOT discriminate the revert (base characteristics
// happens to equal the front face already), which is why both directions are
// pinned rather than one.

/// `Indomitable Archangel` + two plain artifacts + `Thaumatic Compass`
/// (front-face-up = an Artifact, feeding Metalcraft as the third artifact;
/// transformed = a Land, no longer feeding it). Only the Archangel needs a REAL ETB
/// registration (`OOS-DX43-6`) — the Compass and the plain artifacts are just
/// candidates whose CARD TYPE is being counted, not permanents whose own static
/// ability matters, so builder placement is correct for them (the same shape
/// `nexus_animated_by_a_continuous_effect_now_counts_toward_metalcraft`'s "Animated
/// Land" object uses).
fn archangel_compass_board(transformed: bool) -> (GameState, ObjectId, ObjectId) {
    let defs = defs_map();
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .with_registry(registry())
        .object(real_card_spec(
            p1(),
            "Indomitable Archangel",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(ObjectSpec::artifact(p1(), "PB-DX42b Compass Neighbour A"))
        .object(ObjectSpec::artifact(p1(), "PB-DX42b Compass Neighbour B"))
        .object(real_card_spec(
            p1(),
            "Thaumatic Compass",
            ZoneId::Battlefield,
            &defs,
        ))
        .build()
        .unwrap();

    let angel = find_on_battlefield(&state, "Indomitable Archangel");
    let angel_card_id = state
        .objects()
        .get(&angel)
        .and_then(|o| o.card_id.clone())
        .expect("the Archangel object must carry its real card_id");
    register_static_continuous_effects(&mut state, angel, Some(&angel_card_id), &registry(), false);

    let compass = find_on_battlefield(&state, "Thaumatic Compass");
    if transformed {
        state
            .objects_mut()
            .get_mut(&compass)
            .expect("compass exists")
            .is_transformed = true;
    }
    let neighbour = find_on_battlefield(&state, "PB-DX42b Compass Neighbour A");
    (state, compass, neighbour)
}

/// CR 712.8d: front face up, Thaumatic Compass has only its FRONT face's
/// characteristics (Artifact) — the third artifact, so Metalcraft is ON.
#[test]
fn thaumatic_compass_front_face_feeds_metalcraft_as_the_third_artifact() {
    let (state, compass, neighbour) = archangel_compass_board(false);

    let compass_chars =
        calculate_characteristics(&state, compass).expect("live on the battlefield");
    assert!(
        compass_chars.card_types.contains(&CardType::Artifact),
        "precondition: CR 712.8d — front face up, Thaumatic Compass has only its \
         front face's characteristics (Artifact); got {:?}",
        compass_chars.card_types
    );
    assert!(
        !compass_chars.card_types.contains(&CardType::Land),
        "precondition: the front face is not a Land; got {:?}",
        compass_chars.card_types
    );

    let chars = calculate_characteristics(&state, neighbour).expect("live on the battlefield");
    assert!(
        chars.keywords.contains(&KeywordAbility::Shroud),
        "with the untransformed Compass (Artifact) plus two plain artifacts = 3 \
         artifacts, Metalcraft must be ON; got keywords {:?}",
        chars.keywords
    );
}

/// CR 712.8d/712.8e: back face up, Thaumatic Compass has only Spires of Orazca's
/// characteristics (Land, no mana cost) — no longer an artifact, so the count drops
/// to 2 and Metalcraft is OFF. This is the negative case; the DFC pre-loop rewrite,
/// not the Layer-4 supply mechanism items 2/3 exercise, is what changes here.
#[test]
fn thaumatic_compass_transformed_stops_feeding_metalcraft() {
    let (state, compass, neighbour) = archangel_compass_board(true);

    let compass_chars =
        calculate_characteristics(&state, compass).expect("live on the battlefield");
    assert!(
        compass_chars.card_types.contains(&CardType::Land),
        "precondition: CR 712.8d — back face up, Thaumatic Compass has only Spires \
         of Orazca's characteristics (Land); got {:?}",
        compass_chars.card_types
    );
    assert!(
        !compass_chars.card_types.contains(&CardType::Artifact),
        "precondition: CR 712.8d — the back face is not an Artifact; got {:?}",
        compass_chars.card_types
    );

    let chars = calculate_characteristics(&state, neighbour).expect("live on the battlefield");
    assert!(
        !chars.keywords.contains(&KeywordAbility::Shroud),
        "with the Compass now a Land (not an Artifact), only 2 real artifacts \
         remain; Metalcraft must be OFF; got keywords {:?}",
        chars.keywords
    );
}

// ── Item 5: two distinct conditional effects nest without mutual suppression ────
//
// `nexus_animated_by_a_continuous_effect_now_counts_toward_metalcraft`
// (`pb_dx19_characteristics_recursion.rs`) already proves a NESTED query resolves
// an UNCONDITIONAL sibling effect correctly. It is structurally incapable of
// catching a "suppress ANY nested condition evaluation while ANOTHER effect's
// condition is mid-evaluation" bug (a scalar/boolean `in_flight` instead of a
// per-`EffectId` set), because `is_effect_condition_satisfied`'s very first line
// for an UNCONDITIONAL effect is `let Some(ref condition) = effect.condition else
// { return true; }` — it returns before ever consulting `eval.in_flight` at all.
// This probe's second effect (Y) is ALSO conditional, so its evaluation reaches the
// `eval.in_flight.contains(&effect.id)` check, which is the only place such a bug
// could manifest.

/// `X` (Metalcraft-shaped, source = Warden, layer = Ability(6), condition requires
/// TypeChange(4)) and `Y` (source = Land L, layer = TypeChange(4), condition
/// requires Text(3)) are two DISTINCT `ContinuousEffect`s with two DISTINCT
/// `EffectId`s. Evaluating X's condition enters a nested walk bounded at
/// TypeChange; that walk's activity sweep includes Y (Y.layer <= TypeChange); Y's
/// OWN condition evaluation is what a scalar `in_flight` would wrongly suppress.
///
/// `y_condition_satisfied` toggles Y's own count threshold between trivially true
/// (every object this fixture builds has `mana_cost: None`, i.e. mana value 0, so
/// "you control 1+ object with mana value 0 or less" is satisfied by the Warden
/// alone) and unreachably high — the non-vacuity floor proving the positive case
/// isn't "L always becomes an artifact no matter what Y says".
fn nesting_board(y_condition_satisfied: bool) -> (GameState, ObjectId, ObjectId) {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "PB-DX42b Warden", 2, 2))
        .object(ObjectSpec::artifact(p1(), "PB-DX42b Plain Artifact A"))
        .object(ObjectSpec::artifact(p1(), "PB-DX42b Plain Artifact B"))
        .object(ObjectSpec::land(p1(), "PB-DX42b Land L"))
        .build()
        .unwrap();

    let warden = find_on_battlefield(&state, "PB-DX42b Warden");
    let land = find_on_battlefield(&state, "PB-DX42b Land L");
    let artifact_a = find_on_battlefield(&state, "PB-DX42b Plain Artifact A");

    // Effect X: CR 604.2/613.1f (ability word "Metalcraft"'s generic mechanism) --
    // "artifacts you control have shroud as long as you control 3+ artifacts".
    // Condition requires TypeChange (CR 613.1d), strictly earlier than X's own
    // Ability(6) layer.
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9_800),
        source: Some(warden),
        timestamp: 1,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::ArtifactsYouControl,
        modification: LayerModification::AddKeyword(KeywordAbility::Shroud),
        is_cda: false,
        affected_set: None,
        condition: Some(Condition::YouControlNOrMoreWithFilter {
            count: 3,
            filter: TargetFilter {
                has_card_type: Some(CardType::Artifact),
                ..Default::default()
            },
        }),
    });

    // Effect Y: a SEPARATE, ALSO-conditional effect on a DIFFERENT object (the
    // Land). Condition requires Text(3), strictly earlier than Y's own
    // TypeChange(4) layer.
    let y_count = if y_condition_satisfied { 1 } else { 100 };
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9_801),
        source: Some(land),
        timestamp: 2,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(land),
        modification: LayerModification::AddCardTypes([CardType::Artifact].into_iter().collect()),
        is_cda: false,
        affected_set: None,
        condition: Some(Condition::YouControlNOrMoreWithFilter {
            count: y_count,
            filter: TargetFilter {
                max_cmc: Some(0),
                ..Default::default()
            },
        }),
    });

    (state, artifact_a, land)
}

/// **The discriminating probe.** Must be RED under a revert to the retired ambient
/// depth-counter design: that design's `characteristics_for_condition` returned
/// BASE characteristics for ANY object queried while inside a layer walk, so the
/// candidate resolution for Land L (inside X's nested walk) would see L's base type
/// (Land only, ignoring Y's Layer-4 grant entirely) regardless of whether Y's OWN
/// condition holds — count stays at 2, Metalcraft stays OFF, no Shroud.
#[test]
fn two_distinct_conditional_effects_nest_without_mutual_suppression() {
    let (state, artifact_a, land) = nesting_board(true);

    // Precondition: Y's own condition is satisfied and the Land really is
    // layer-resolved as an artifact when queried directly.
    let land_chars = calculate_characteristics(&state, land).expect("live on the battlefield");
    assert!(
        land_chars.card_types.contains(&CardType::Artifact),
        "precondition: Y's condition is satisfied (count=1, trivially true via any \
         mana-cost-0 object), so CR 613.1d must make the Land an artifact; got {:?}",
        land_chars.card_types
    );

    // The DISCRIMINATING assertion: X's condition, evaluated as part of resolving a
    // THIRD object's (artifact_a's) characteristics, must see the correct,
    // layer-resolved count of 3 artifacts (2 plain + the Land via Y) and grant
    // Shroud -- NOT be suppressed because a DIFFERENT effect's (Y's) condition
    // happened to be evaluated while X's own condition was mid-evaluation.
    // `eval.in_flight` is keyed on `EffectId`, so only X's OWN id is ever in flight
    // during X's evaluation; Y's distinct id is never in that set, and Y's
    // condition is evaluated on its own merits.
    let chars = calculate_characteristics(&state, artifact_a).expect("live on the battlefield");
    assert!(
        chars.keywords.contains(&KeywordAbility::Shroud),
        "CR 613.1d/604.2: with Y correctly resolving the Land into a THIRD \
         artifact, Metalcraft (X) must be ON. A design that suppresses ANY nested \
         condition evaluation merely because ANOTHER effect's condition is \
         mid-evaluation (rather than keying suppression on the SPECIFIC EffectId) \
         would treat Y as inactive here too, leaving the Land a bare Land, the \
         count at 2, and no Shroud -- got keywords {:?}.",
        chars.keywords
    );
}

/// Non-vacuity floor for the probe above: with Y's OWN condition genuinely
/// unsatisfied, the Land must stay a Land and Metalcraft must stay OFF -- proving
/// the positive test isn't "L always becomes an artifact no matter what Y's
/// condition says".
#[test]
fn the_nested_effect_correctly_staying_inactive_is_not_suppression() {
    let (state, artifact_a, land) = nesting_board(false);

    let land_chars = calculate_characteristics(&state, land).expect("live on the battlefield");
    assert!(
        !land_chars.card_types.contains(&CardType::Artifact),
        "precondition: Y's condition is genuinely false (count threshold \
         unreachably high), so CR 613.1d must leave the Land as a Land; got {:?}",
        land_chars.card_types
    );

    let chars = calculate_characteristics(&state, artifact_a).expect("live on the battlefield");
    assert!(
        !chars.keywords.contains(&KeywordAbility::Shroud),
        "with only 2 real artifacts (Y correctly inactive, not suppressed), \
         Metalcraft must be OFF; got keywords {:?}",
        chars.keywords
    );
}

// ── Item 6 + the wrong-way-round pin: a same-layer self-referential effect ──────
//
// Both probes below share ONE fixture on purpose: a continuous effect whose
// condition asks about a characteristic ITS OWN modification grants, at the SAME
// layer. `required_characteristic_layer()` for this condition is
// `Some(EffectLayer::Ability)`, equal to the effect's own `Ability` layer, so
// `required < effect.layer` is FALSE -- the exact class `is_effect_condition_
// satisfied`'s `debug_assert!` exists to flag. **The two outcomes are mutually
// exclusive within a single build profile**: in a DEBUG build the assert panics on
// the FIRST (outermost) evaluation, before the function ever reaches the
// `InFlightGuard`/recursion machinery that would produce a SUPPRESSED boolean
// answer; the suppression path is therefore observable only in a build where
// `debug_assertions` is off (`is_effect_condition_satisfied`'s own doc: "the
// debug_assert! below never lets a same-or-later-layer condition reach here in a
// debug build at all"). This is proven structurally, not merely asserted here --
// see the close-out report for the general argument (the strictly-decreasing-bound
// termination proof extends to ANY chain of distinct effects, so `eval.in_flight`'s
// cross-effect suppression is provably unreachable in any assert-passing
// construction; the ONLY way to observe it at all is a literal same-effect
// self-reference, which is exactly the class the assert flags).
fn self_referential_flying_board() -> (GameState, ObjectId) {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(
            p1(),
            "PB-DX42b Self-Ref Creature",
            2,
            2,
        ))
        .build()
        .unwrap();

    let creature = find_on_battlefield(&state, "PB-DX42b Self-Ref Creature");
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9_700),
        source: Some(creature),
        timestamp: 1,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::Source,
        modification: LayerModification::AddKeyword(KeywordAbility::Flying),
        is_cda: false,
        affected_set: None,
        // "As long as you control a creature with flying, [this creature] has
        // flying" -- a literal, same-layer self-reference: `has_keywords` needs
        // Ability(6) (`needs_ability` in `TargetFilter::required_characteristic_
        // layer`), the SAME layer this effect's own modification applies at.
        condition: Some(Condition::YouControlNOrMoreWithFilter {
            count: 1,
            filter: TargetFilter {
                has_keywords: [KeywordAbility::Flying].into_iter().collect(),
                ..Default::default()
            },
        }),
    });
    (state, creature)
}

/// **Item 6.** CR 613.1: the layer-bounded walk's termination proof depends on a
/// conditioned effect's condition asking about a STRICTLY EARLIER layer than its
/// own. This synthetic effect violates that (required == own layer), and the
/// `debug_assert!` in `is_effect_condition_satisfied` must fire rather than
/// silently degrade into the labelled deviation below.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "which is at or after its own layer")]
fn debug_assert_fires_on_a_same_layer_self_reference() {
    let (state, creature) = self_referential_flying_board();
    let _ = calculate_characteristics(&state, creature);
}

/// **The wrong-way-round pin for `is_effect_condition_satisfied`'s labelled
/// deviation** (`pb-plan-DX42b.md` §3; adjudication §3.2(ii)/§5.3). Only compiled
/// when `debug_assertions` is off (see this section's module doc for why the two
/// outcomes cannot coexist in one build profile) -- run with e.g. `cargo test
/// --release -p mtg-engine --test primitives self_reference` to exercise it.
///
/// The CR is silent on condition-evaluation cycles. CR 613.8b is evidence about
/// the CR's disposition when it faces an unresolvable circularity -- it picks a
/// TOTAL ORDER (timestamp), not suppression -- but CR 613.8a's own list item (a)
/// confines a "dependency" to effects in the SAME layer AMONG DISTINCT effects,
/// and this is a literal self-reference, not a dependency between two effects, so
/// 613.8a/613.8b do not govern it either way. Treat-as-inactive is therefore an
/// UNDOCUMENTED DEVIATION, and this test pins the deviating outcome on purpose:
/// **a batch that implements a timestamp-ordered tiebreak for this case must
/// INVERT this assertion, not merely delete it.**
#[cfg(not(debug_assertions))]
#[test]
fn same_layer_self_reference_is_suppressed_not_resolved() {
    let (state, creature) = self_referential_flying_board();
    let chars = calculate_characteristics(&state, creature).expect("live on the battlefield");
    assert!(
        !chars.keywords.contains(&KeywordAbility::Flying),
        "KNOWN DEVIATION (labelled, adjudication §3.2(ii)/§5.3, undocumented by the \
         CR): this effect's own condition ('as long as you control a creature with \
         flying') asks about a characteristic its OWN modification grants, at the \
         SAME layer. `eval.in_flight` treats the effect as INACTIVE the instant its \
         own evaluation re-enters itself, so Flying is never granted here -- got \
         keywords {:?}. THIS IS THE DEVIATION BEING PINNED, NOT A CORRECT ANSWER: \
         a future batch that implements CR 613.8b's timestamp-ordered tiebreak for \
         same-layer self-reference must INVERT this assertion.",
        chars.keywords
    );
}

// ── The retired assert's successor: guard state does not leak past an early return ─

/// **The invariant that replaces the retired `process_command` depth-balance
/// assert (`OOS-DX19-4`, `pb-plan-DX42b.md` §4).** `LAYER_WALK_DEPTH` was an
/// ambient thread-local that could leak across a command boundary (a
/// `mem::forget`, an `enter()` outside `calculate_characteristics`) and stay
/// sticky for the rest of the thread; the retired assert existed to catch exactly
/// that leak. `CharacteristicEvalContext` is created FRESH, as an ordinary stack
/// value, for every top-level `calculate_characteristics` call, and is reachable
/// only by `&mut` reference passed DOWN the call stack -- the borrow checker makes
/// it impossible for it to outlive the call that created it, so the class of leak
/// the retired assert guarded against is no longer REPRESENTABLE, not merely
/// unlikely (§4's own claim).
///
/// `calculate_characteristics_through` has an early `?`-return
/// (`state.objects.get(&object_id)?`) that runs AFTER `BoundGuard::enter` has
/// already mutated `eval.bound` -- PB-DP8's "a guard that returns early inherits
/// the obligation of the statements it skipped", one primitive over. This probe
/// exercises that early return with a dead id (the documented `None` contract on
/// `calculate_characteristics` itself) and then, in the SAME test, drives a real
/// nested (bound-decreasing) walk to prove nothing was left corrupted.
///
/// **Disclosed rather than overclaimed**: because a fresh context is constructed
/// per top-level call today, "the dead-id call does not corrupt a later call"
/// already holds BY CONSTRUCTION at the top level, and this file cannot reach
/// `calculate_characteristics_through`/`CharacteristicEvalContext` directly (both
/// `pub(crate)`) to force a dead id to appear MID-walk. The pin exists against a
/// FUTURE change (e.g. caching or sharing a `CharacteristicEvalContext` across
/// calls for performance) that would silently reintroduce the retired hazard one
/// layer up from where it used to live.
#[test]
fn a_dead_id_early_return_does_not_corrupt_a_later_nested_walk() {
    let (state, artifact_a, _land) = nesting_board(true);

    // The dead id: nothing in this fixture ever allocates this ObjectId.
    let dead = ObjectId(999_999);
    assert!(
        state.objects().get(&dead).is_none(),
        "test bug: the 'dead' id must not already exist in this fixture's object map"
    );
    assert_eq!(
        calculate_characteristics(&state, dead),
        None,
        "calculate_characteristics's own documented contract: None iff the object \
         does not exist -- exercises the `?` early return AFTER `BoundGuard::enter` \
         has already mutated `eval.bound`"
    );

    // A real, nested (bound-decreasing) walk immediately afterward must still
    // produce the CR-correct answer -- nothing from the dead-id call's aborted
    // walk survives to affect it.
    let chars = calculate_characteristics(&state, artifact_a).expect("live on the battlefield");
    assert!(
        chars.keywords.contains(&KeywordAbility::Shroud),
        "a prior calculate_characteristics call that hit the dead-id early return \
         must not leave any residual bound/in-flight state behind; got keywords {:?}",
        chars.keywords
    );
}
