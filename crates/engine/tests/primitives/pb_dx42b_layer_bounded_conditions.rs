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
//!   `indomitable_archangel.rs` cited "CR 702.45a (Metalcraft)" from the day it was
//!   authored — **wrong**; CR 702.45a is Bushido. **FIXED in this batch**
//!   (`indomitable_archangel.rs`, `OOS-DX42b-2`); the first draft of this paragraph
//!   said "Not fixed here … reported at close-out" and was already false when the
//!   `/review` read it, which is a false comment inside the batch whose subject matter
//!   is false comments. `mox_opal.rs` carries the SAME wrong cite twice and is fixed
//!   too — the seed's site list was a floor (dispatch hygiene 6).
//! - **CR 702.18a** (Shroud): "'Shroud' means 'This permanent or player can't be the
//!   target of spells or abilities.'"
//! - **CR 712.8d/712.8e** (double-faced permanent characteristics): "712.8d: While a
//!   double-faced permanent has its front face up, it has only the characteristics
//!   of its front face." / "712.8e: While a nonmodal double-faced permanent has its
//!   back face up, it has only the characteristics of its back face."
//! - **CR 613.8a / CR 613.8b** (the dependency/timestamp tiebreak that does NOT
//!   govern the labelled deviation this file pins wrong-way-round): CR 613.8a's own
//!   text is a single rule with an internal (a)/(b)/(c) list — **"CR 613.8a(a)" is
//!   not a real citation form**, because a reader who greps the CR for it finds
//!   nothing. `layers.rs`'s two NEW sites say "CR 613.8a clause (a)"; its only
//!   remaining `613.8a(` occurrence is `613.8a(c)` at the CDA-symmetry comment, which
//!   is pre-existing and a different clause. The form survives in the adjudication and
//!   the memo and is filed as `OOS-DX42b-3` to ride PB-DX38's cite sweep. Cited as
//!   plain **CR 613.8a** below.
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

// ── Two rows the coordinator's revert matrix found UNCOVERED, added because a
//    revert that reddens only a source gate is telling you the behaviour has no
//    probe (`OOS-DX52-2`), not that the row is uninteresting ──────────────────

/// **`OOS-DX42b-1`, pinned behaviourally.** A condition that reads NO characteristic
/// at all is not a condition that needs Layer 1, and collapsing the two is a live
/// debug-build panic.
///
/// `is_effect_condition_satisfied` gates its CR 613.1d `debug_assert!` on
/// `Condition::required_characteristic_layer()` returning `Some`. The first draft
/// wrote `.unwrap_or(EffectLayer::Copy)` instead — and `Copy` is the FIRST layer, so
/// `required < effect.layer` is false for every `EffectLayer::Copy` effect. A Layer-1
/// effect carrying `Condition::IsYourTurn` (which reads the turn structure and nothing
/// else) therefore panicked the debug build with a message asserting that its condition
/// "requires characteristics resolved through layer Copy" — characteristics it does not
/// require at any layer.
///
/// **Zero corpus exposure today and that is not the reason this test exists**:
/// `rules/copy.rs` records the measured fact that `crates/card-defs/src/defs` contains
/// zero occurrences of `EffectLayer::Copy`, so no shipped card reaches this. A
/// debug-build panic on a legitimate configuration is a defect whether or not a card
/// reaches it, and the revert matrix found that reverting the fix reddened **only**
/// `core::pb_dx39_source_view_gates::r4` — a VOCABULARY gate that catches it
/// incidentally, because the identifiers `unwrap_or` / `EffectLayer` / `Copy` come
/// back. A vocabulary gate proves a body is spelled a certain way; it cannot prove the
/// body does the right thing, and a later batch that respells the fix while keeping the
/// bug satisfies it completely.
///
/// CR 613.1a (Layer 1, copy effects) / CR 604.2 (conditional static abilities).
#[test]
fn a_layer_one_effect_with_a_characteristic_free_condition_does_not_trip_the_assert() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Bear", 2, 2))
        .build()
        .unwrap();
    let bear = find_on_battlefield(&state, "Bear");

    // Non-vacuity: the condition really must require NO characteristic layer, or this
    // test would be exercising a different arm than the one it names.
    assert_eq!(
        Condition::IsYourTurn.required_characteristic_layer(),
        None,
        "precondition: Condition::IsYourTurn reads the turn structure and no \
         characteristic at any layer, so required_characteristic_layer must be None. \
         If this has changed, this test is no longer about the None case."
    );

    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9_777),
        source: Some(bear),
        timestamp: 1,
        layer: EffectLayer::Copy,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(bear),
        modification: LayerModification::AddKeyword(KeywordAbility::Flying),
        condition: Some(Condition::IsYourTurn),
        is_cda: false,
        affected_set: None,
    });

    // Under the reverted `unwrap_or(EffectLayer::Copy)` this call ABORTS the debug
    // test binary on the CR 613.1d assert. It must simply answer.
    let chars = calculate_characteristics(&state, bear)
        .expect("the Bear is live on the battlefield and the walk must not panic");

    // And it must answer CORRECTLY: p1 is the active player in this fixture, so the
    // condition holds and the Layer-1 grant applies. Asserting the OUTCOME rather than
    // merely "it did not panic" keeps this from passing on an engine that answers
    // nothing at all.
    assert!(
        chars.keywords.contains(&KeywordAbility::Flying),
        "the condition (IsYourTurn, and p1 is active) is satisfied, so the effect is \
         active and its grant applies; got keywords {:?}",
        chars.keywords
    );
}

/// **The activity sweep's layer bound is load-bearing for TERMINATION and invisible to
/// every behavioural test in this workspace — so it is gated on its source, with the
/// executed evidence written into the failure message.**
///
/// `calculate_characteristics_through` filters its activity sweep by `e.layer <= through`,
/// the SAME bound the query uses. The adjudication (§3.2(iii)) calls that the
/// load-bearing precondition and states it *"here because it is stated nowhere else"*: a
/// bounded query over a GLOBAL activity sweep is the original recursion with an extra
/// parameter.
///
/// **Why this is a source gate and not a probe.** Deleting the conjunct on its own
/// reddens NOTHING in the workspace, and that is not a missing test — it is structural.
/// An effect in a later layer cannot change an earlier layer's output, which is the very
/// fact that makes bounding the sweep semantically free; and with the `in_flight`
/// backstop present, the backstop absorbs the extra recursion. So no assertion on
/// characteristics can separate the two designs. What separates them is TERMINATION, and
/// the coordinator measured it with a complementary pair of reverts rather than arguing
/// it:
///
/// | revert | sweep bound | `in_flight` backstop | result |
/// |---|---|---|---|
/// | R3  | removed | present | **green** — the backstop absorbs it, nothing observes the difference |
/// | R3c | present | removed | **green, 23/23** — termination really is by construction |
/// | R3b | removed | removed | **`fatal runtime error: stack overflow, aborting` (SIGABRT)** |
///
/// R3b is `OOS-SIM2-6`'s original crash, reproduced. So the conjunct below is what makes
/// the recursion finite, and the labelled `in_flight` deviation is genuinely unreachable
/// rather than merely unused — which is the claim §3.2(iii) makes and which nobody had
/// executed until now.
#[test]
fn the_activity_sweep_is_bounded_by_the_same_layer_as_the_query() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/rules/layers.rs"))
        .expect("layers.rs must be readable from the test binary");

    let at = src
        .find("pub(crate) fn calculate_characteristics_through(")
        .expect("calculate_characteristics_through not found — did it get renamed?");
    let open = src[at..].find('{').expect("header must have a body") + at;
    let mut depth = 0usize;
    let mut end = open;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = open + i;
                    break;
                }
            }
            _ => {}
        }
    }
    let body = &src[open..end];

    // Non-vacuity: a body that shrank to nothing would make this gate pass forever.
    assert!(
        body.lines().count() > 100,
        "calculate_characteristics_through's body is only {} lines — the brace match \
         found the wrong thing, or the layer walk moved somewhere this gate does not \
         scan. Re-derive before trusting a green here.",
        body.lines().count()
    );

    // Strip comments so the doc paragraph above the filter cannot satisfy this gate.
    let code: String = body
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");

    // Locate the SWEEP itself -- the loop over `state.continuous_effects` -- and scope
    // the bound check to it. **The first draft asserted `code.contains("e.layer <=
    // through")` over the whole body, and the `/review` defeated it with one dead line**:
    //
    //     let _spelling_kept_for_the_gate = |e: &ContinuousEffect| e.layer <= through;
    //
    // planted beside a sweep whose conjunct had been deleted. All gates green, the
    // load-bearing conjunct gone. A PRESENCE check over a whole function body asks
    // "does this string occur", which is not the property; the property is "the sweep
    // is bounded", and a string can occur anywhere. This gate is the ENTIRE coverage
    // for that conjunct -- the batch measured that removing it reddens nothing else in
    // the workspace -- so a presence check here is worth less than nothing, because it
    // reads as coverage.
    let sweep_at = code
        .find("for e in state.continuous_effects.iter()")
        .expect(
            "the activity sweep is no longer a `for e in state.continuous_effects.iter()` \
             loop in `calculate_characteristics_through`. Re-derive where the sweep lives \
             and re-key this gate on it; do NOT relax it to a whole-body search, which is \
             what the /review defeated with a dead closure.",
        );
    // The sweep's own condition: everything from the loop header to the first `{` that
    // opens the `active_effects.push` body. Brace-matched rather than a byte window, so
    // a longer condition cannot slide out of range (PB-DX49's `r5b` over-scan lesson).
    let sweep_tail = &code[sweep_at..];
    let cond_end = sweep_tail
        .find("active_effects.push")
        .expect("the sweep must still push into `active_effects`");
    let sweep_condition = &sweep_tail[..cond_end];

    assert_eq!(
        sweep_condition.matches("e.layer <= through").count(),
        1,
        "the activity sweep's layer bound must appear EXACTLY ONCE inside the sweep's own \
         condition, and it appears {} time(s) there. A COUNT rather than a presence check, \
         and scoped to the sweep rather than the body, because the /review defeated both \
         weaker forms. Sweep condition as scanned:\n{}",
        sweep_condition.matches("e.layer <= through").count(),
        sweep_condition
    );

    assert!(
        code.contains("e.layer <= through"),
        "the activity sweep in `calculate_characteristics_through` no longer bounds \
         itself by `through`. This is NOT a style preference and it will not be caught \
         by any behavioural test in this workspace -- deleting this conjunct alone was \
         measured GREEN across the entire suite, because a later-layer effect cannot \
         change an earlier layer's output and the `in_flight` backstop absorbs the extra \
         recursion. What it destroys is TERMINATION BY CONSTRUCTION: with the conjunct \
         removed AND the backstop removed, `pb_dx19_characteristics_recursion::\
         recursion_metalcraft_on_grants_shroud_and_terminates` aborts with `fatal \
         runtime error: stack overflow` (SIGABRT) -- OOS-SIM2-6's original crash -- \
         while with the conjunct present and the backstop removed it terminates 23/23. \
         See adjudication section 3.2(iii) and this test's doc comment before changing it."
    );
}

// ── The `/review` finding: a coarse summary defeats the assert that guards it ──

/// **The CR 613.1d defect this batch closes was STILL LIVE one `Condition` variant over,
/// and this batch's own `debug_assert!` was silenced by the very thing that caused it.**
///
/// `Condition::required_characteristic_layer` originally delegated to
/// [`TargetFilter::required_characteristic_layer`] for `YouControlNOrMoreWithFilter`
/// alone and fixed `YouControlPermanent` / `OpponentControlsPermanent` at
/// `EffectLayer::TypeChange`, under a doc sentence asserting that those two *"test only
/// card types, supertypes or subtypes -- never power/toughness, color or keywords"*.
/// **That sentence was false**: both arms pass the WHOLE filter to
/// `effects::matches_filter`, which reads `power`, `toughness`, `colors` and `keywords`
/// alongside the type fields.
///
/// **What the coarse answer actually cost, measured rather than argued.** With a Layer-6
/// `AddKeyword(Flying)` conditioned on
/// `YouControlPermanent(TargetFilter { min_power: Some(4), .. })`, over a 2/2 pumped to
/// 4/2 by a Layer-7c `ModifyPower(2)`:
///
/// * **before**: `required` came back `TypeChange`, so `TypeChange < Ability` held, the
///   `debug_assert!` stayed **silent**, the nested walk was bounded at Layer 4, the
///   condition compared the **printed** power of 2, and the grant silently did not apply.
///   A wrong answer with no signal — CR 613.1d violated exactly as this batch's headline
///   defect violated it.
/// * **after**: `required` comes back `PtSwitch`, and the `debug_assert!` **FIRES**,
///   naming the effect, the layer it needs and its own layer.
///
/// **The fix does not make that configuration WORK, and it is not supposed to.** A
/// Layer-6 effect whose condition depends on Layer 7 is not a case CR 613.1 can answer:
/// Layer 7 runs *after* Layer 6, so the query cannot be bounded below its own effect and
/// the walk has no termination-by-construction argument left. Refusing it loudly is the
/// correct engine behaviour and is the documented deviation
/// (`is_effect_condition_satisfied`). **What the fix changes is a SILENT WRONG ANSWER
/// into a LOUD NAMED FAILURE** — and that is the whole value of the assert, which the
/// coarse map had disabled.
///
/// *A summary coarser than the thing it summarises does not merely lose precision — it
/// defeats the assertion that was supposed to catch the imprecision.*
///
/// Zero corpus exposure: both layer-querying corpus members are
/// `YouControlNOrMoreWithFilter`, which always delegated, so nothing in the tree could
/// have gone red. CR 613.1d, CR 613.4c (Layer 7c), CR 604.2.
#[test]
fn every_filter_carrying_condition_variant_asks_the_filter_for_its_layer() {
    use mtg_engine::CardType;

    let pt_filter = TargetFilter {
        min_power: Some(4),
        has_card_type: Some(CardType::Creature),
        ..Default::default()
    };
    // Non-vacuity: the filter must really need a LATER layer than TypeChange, or this
    // test would pass against the coarse map it exists to reject.
    assert_eq!(
        pt_filter.required_characteristic_layer(),
        Some(EffectLayer::PtSwitch),
        "precondition: a min_power filter needs P/T, which is Layer 7 (CR 613.4a-d)"
    );

    for (label, cond) in [
        (
            "YouControlPermanent",
            Condition::YouControlPermanent(pt_filter.clone()),
        ),
        (
            "OpponentControlsPermanent",
            Condition::OpponentControlsPermanent(pt_filter.clone()),
        ),
        (
            "YouControlNOrMoreWithFilter",
            Condition::YouControlNOrMoreWithFilter {
                count: 1,
                filter: pt_filter.clone(),
            },
        ),
    ] {
        assert_eq!(
            cond.required_characteristic_layer(),
            Some(EffectLayer::PtSwitch),
            "Condition::{label} carries an arbitrary caller-supplied TargetFilter and \
             passes it WHOLE to effects::matches_filter, which reads power / toughness / \
             colors / keywords as well as types. It must ASK THE FILTER rather than claim \
             a fixed layer. A coarse TypeChange answer does two things, and the second is \
             the dangerous one: it bounds the nested walk BELOW the layer the filter \
             actually reads (CR 613.1d), and it DEFEATS `is_effect_condition_satisfied`'s \
             debug_assert, which is handed this value and sees TypeChange < Ability as \
             satisfied. The result is a wrong answer with no signal."
        );
    }

    // And the eight filter-free layer-querying variants must still answer TypeChange --
    // asserted so the fix above cannot be over-applied into "everything asks a filter".
    assert_eq!(
        Condition::ControlLegendaryCreature.required_characteristic_layer(),
        Some(EffectLayer::TypeChange),
        "a variant carrying no TargetFilter tests supertypes/card types only (CR 613.1d)"
    );
}

/// The behavioural complement of the pin above: the configuration the coarse map made
/// SILENT now fails LOUDLY, naming the effect and both layers.
///
/// Under the pre-fix map this fixture answered `keywords = {}` — the printed power of 2
/// compared against `min_power: 4` — with no assertion anywhere. See the sibling test's
/// doc for the full before/after.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "condition requires characteristics resolved through")]
fn a_layer_six_effect_whose_condition_needs_layer_seven_is_refused_loudly() {
    use mtg_engine::CardType;

    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Bear", 2, 2))
        .build()
        .unwrap();
    let bear = find_on_battlefield(&state, "Bear");

    // Layer 7c (CR 613.4c): the Bear becomes a 4/2.
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9_810),
        source: Some(bear),
        timestamp: 1,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(bear),
        modification: LayerModification::ModifyPower(2),
        condition: None,
        is_cda: false,
        affected_set: None,
    });
    // Layer 6 (CR 613.1f) conditioned on a Layer-7 fact — unanswerable by construction,
    // because Layer 7 runs after Layer 6.
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9_811),
        source: Some(bear),
        timestamp: 2,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(bear),
        modification: LayerModification::AddKeyword(KeywordAbility::Flying),
        condition: Some(Condition::YouControlPermanent(TargetFilter {
            min_power: Some(4),
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        })),
        is_cda: false,
        affected_set: None,
    });

    let _ = calculate_characteristics(&state, bear);
}
