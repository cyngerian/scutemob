//! PB-OS4b: face-aware ability gathering for transformed permanents (CR 712.8d/e).
//!
//! A double-faced permanent showing its back face has only that face's
//! characteristics -- including its abilities (CR 712.8d/e). Two independent
//! "ability channels" exist in the engine and both must be re-derived at the
//! exact instant a battlefield permanent's `is_transformed` flag changes:
//!
//! - **Channel A** -- the runtime `Characteristics.{mana,activated,triggered}_abilities`
//!   vectors, lowered once at object construction by
//!   `testing::replay_harness::build_face_ability_vectors` and otherwise read directly
//!   (bypassing the layer system) by activation/trigger dispatch.
//! - **Channel B** -- static continuous effects (`state.continuous_effects`), which are
//!   registered once at ETB and never automatically re-derived on transform.
//!
//! [`apply_face_change`] is the single choke point that keeps both channels correct:
//! deregister the OLD face's statics, flip `is_transformed`, rebuild the Channel-A
//! vectors from the NEW face, then register the NEW face's statics. Every site in the
//! engine that flips `is_transformed` on a battlefield permanent must route through
//! this function -- no other code should mutate `is_transformed` directly.
use crate::cards::card_definition::AbilityDefinition;
use crate::state::continuous_effect::EffectFilter;
use crate::state::game_object::ObjectId;
use crate::state::zone::ZoneId;
use crate::state::GameState;
/// CR 712.8d/e / 712.18: flip a battlefield permanent's `is_transformed` flag,
/// keeping both ability channels (Channel A runtime vectors, Channel B static
/// continuous effects) synchronized with the newly-visible face.
///
/// No-ops if:
/// - the object isn't live or isn't on the battlefield,
/// - the object isn't a double-faced card (no `back_face`) -- CR 701.27c,
/// - `new_is_transformed` equals the object's current `is_transformed` (nothing changed).
///
/// Order of operations (must not be reordered -- deregister reads the OLD face,
/// register reads the NEW face, and both must see a consistent `is_transformed`
/// relative to the flip):
/// 1. Deregister the OLD face's static continuous effects (see [`deregister_face_statics`]).
/// 2. Flip `is_transformed` and bump `last_transform_timestamp`.
/// 3. Rebuild the Channel-A ability vectors from the NEW face via
///    `build_face_ability_vectors` (mirrors `enrich_spec_from_def`'s front-face lowering).
/// 4. Register the NEW face's static continuous effects
///    (`replacement::register_static_continuous_effects`).
///
/// This does NOT queue ETB triggers, fire "when turned face up" triggers, or check
/// SBAs -- callers retain responsibility for those (unchanged from before this PB).
pub(crate) fn apply_face_change(state: &mut GameState, obj_id: ObjectId, new_is_transformed: bool) {
    let Some(obj) = state.expect_object(obj_id) else {
        return;
    };
    if obj.zone != ZoneId::Battlefield {
        return;
    }
    let old_is_transformed = obj.is_transformed;
    if old_is_transformed == new_is_transformed {
        return;
    }
    let Some(card_id) = obj.card_id.clone() else {
        return;
    };
    // Clone the registry Arc so `def` doesn't hold a borrow of `state` across the
    // mutations below (established pattern -- see e.g. effects/mod.rs
    // ExileSourceAndReturnTransformed).
    let registry = std::sync::Arc::clone(&state.card_registry);
    let Some(def) = registry.get(card_id.clone()) else {
        return;
    };
    if def.back_face.is_none() {
        // CR 701.27c: nothing happens when "transforming" a non-DFC.
        return;
    }
    // Step 1: deregister the OLD face's statics before anything else changes.
    let old_abilities = def.effective_abilities(old_is_transformed).to_vec();
    deregister_face_statics(state, obj_id, &old_abilities);
    // Step 2: flip is_transformed + bump last_transform_timestamp.
    let ts = state.timestamp_counter;
    state.timestamp_counter += 1;
    let Some(obj_mut) = state.expect_object_mut(obj_id) else {
        return;
    };
    obj_mut.is_transformed = new_is_transformed;
    obj_mut.last_transform_timestamp = ts;
    // Step 3: rebuild Channel-A ability vectors from the NEW face. Base == the
    // effective face's abilities after this write, so every downstream reader
    // (direct-base or `calculate_characteristics`-based) is correct with no
    // per-reader auditing (see module doc + PB-OS4b plan "Mechanism Design").
    let (mana_abilities, activated_abilities, triggered_abilities) =
        crate::testing::replay_harness::build_face_ability_vectors(
            def.effective_abilities(new_is_transformed),
        );
    if let Some(obj_mut) = state.expect_object_mut(obj_id) {
        obj_mut.characteristics.mana_abilities = mana_abilities;
        obj_mut.characteristics.activated_abilities = activated_abilities;
        obj_mut.characteristics.triggered_abilities = triggered_abilities;
    }
    // Step 4: register the NEW face's statics (Channel B).
    super::replacement::register_static_continuous_effects(
        state,
        obj_id,
        Some(&card_id),
        &registry,
        new_is_transformed,
    );
}
/// CR 613 / 604: remove the OLD face's static continuous effects from
/// `state.continuous_effects` when a permanent transforms away from that face.
///
/// `ContinuousEffect` carries no origin-face tag (adding one would be a
/// HASH-affecting change, out of PB-OS4b's wire-neutral mandatory scope), so this
/// does a **structural match**: for each `AbilityDefinition::Static` in the old
/// face's ability list, remove exactly one entry from `state.continuous_effects`
/// where `source == Some(obj_id)` and `(layer, duration, modification,
/// resolved_filter)` match -- resolving `EffectFilter::Source ->
/// SingleObject(obj_id)` the same way `register_static_continuous_effects` does,
/// so the comparison is apples-to-apples. Removing at most one entry per static
/// ability avoids disturbing any other continuous effect the object happens to own
/// (e.g. a temporary effect granted by a different source).
///
/// Non-`Static` abilities are intentionally NOT deregistered here in the mandatory
/// scope -- no roster card's back face declares any of them, and each lives in its
/// own `state.*` collection with its own shape (some registering more than one
/// entry per ability, e.g. `CdaModifyPowerToughness`). As of this writing
/// `register_static_continuous_effects` (`replacement.rs`) also registers, from the
/// *effective* face, the following, none of which this function removes:
/// - `AbilityDefinition::TriggerDoubling` -> `state.trigger_doublers`
/// - `AbilityDefinition::SuppressCreatureETBTriggers` -> `state.etb_suppressors`
/// - `AbilityDefinition::StaticRestriction` -> `state.restrictions`
/// - `AbilityDefinition::CdaPowerToughness` -> `state.continuous_effects` (Layer 7a, `is_cda: true`)
/// - `AbilityDefinition::CdaModifyPowerToughness` -> `state.continuous_effects` (Layer 7c,
///   `is_cda: true`; up to TWO entries per ability, one per Some(power)/Some(toughness))
/// - `AbilityDefinition::AdditionalLandPlays` -> `state.additional_land_play_sources`
/// - `AbilityDefinition::StaticFlashGrant` -> `state.flash_grants`
/// - `AbilityDefinition::StaticPlayFromGraveyard` -> `state.play_from_graveyard_permissions`
/// - `AbilityDefinition::StaticPlayFromTop` -> `state.play_from_top_permissions`
///
/// (PB-OS4b review E2 named only the first four of these; re-reading
/// `register_static_continuous_effects` for this fix turned up five more --
/// `CdaModifyPowerToughness`, `AdditionalLandPlays`, `StaticFlashGrant`,
/// `StaticPlayFromGraveyard`, `StaticPlayFromTop` -- that were also missing from
/// that enumeration. The full family is materially larger and more heterogeneous
/// than a `Static`-shaped symmetric extension: several of these collections key on
/// different field shapes (`Option<ObjectId>` vs `ObjectId` source, 1-or-2 entries
/// per ability, no shared `(layer, duration, modification, filter)` tuple to compare
/// against outside `state.continuous_effects`), so a precise structural remove for
/// all nine is a distinctly larger and riskier change than the one this function
/// already does for `Static`. Deferred rather than attempted opportunistically here;
/// if a future DFC back face declares any of the nine, extend this function
/// symmetrically with `register_static_continuous_effects`, collection by
/// collection, at that time.
pub(crate) fn deregister_face_statics(
    state: &mut GameState,
    obj_id: ObjectId,
    old_face_abilities: &[AbilityDefinition],
) {
    for ability in old_face_abilities {
        if let AbilityDefinition::Static { continuous_effect } = ability {
            let resolved_filter = match &continuous_effect.filter {
                EffectFilter::Source => EffectFilter::SingleObject(obj_id),
                other => other.clone(),
            };
            if let Some(pos) = state.continuous_effects.iter().position(|e| {
                e.source == Some(obj_id)
                    && e.layer == continuous_effect.layer
                    && e.duration == continuous_effect.duration
                    && e.modification == continuous_effect.modification
                    && e.filter == resolved_filter
            }) {
                state.continuous_effects.remove(pos);
            }
        }
    }
}
