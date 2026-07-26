//! PB-OS4b / PB-RS4: face-aware ability gathering for transformed permanents
//! (CR 712.8d/e).
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
//! - **Channel B** -- static registrations (`state.continuous_effects` and the nine
//!   sibling collections `register_static_continuous_effects` also writes --
//!   `trigger_doublers`, `etb_suppressors`, `restrictions`,
//!   `additional_land_play_sources`, `flash_grants`,
//!   `play_from_graveyard_permissions`, `play_from_top_permissions`), registered
//!   once at ETB and never automatically re-derived on transform.
//!
//! [`apply_face_change`] is the single choke point that keeps both channels correct:
//! deregister the OLD face's statics, flip `is_transformed`, rebuild the Channel-A
//! vectors from the NEW face, then register the NEW face's statics. Every site in the
//! engine that flips `is_transformed` on a battlefield permanent must route through
//! this function -- no other code should mutate `is_transformed` directly.
//!
//! PB-OS4b shipped Channel B deregistration for `AbilityDefinition::Static` only,
//! deliberately deferring the other nine `register_static_continuous_effects`
//! families (documented as a known gap, OOS-OS4-2 / OOS-RS-3). PB-RS4 closes that
//! gap: [`deregister_face_statics`] now covers all ten families symmetrically via
//! [`remove_one_registration`], with a source-scan drift guard
//! (`tests/core/face_dereg_parity.rs`) keeping the two functions in lockstep.
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
/// CR 604.1 / 613 / 712.8e / 712.18: remove the OLD face's static registrations
/// from state when a permanent transforms away from that face (CR 712.18: this is
/// an in-place flip -- the object never changes zones, so nothing else ever cleans
/// these up while the permanent stays on the battlefield).
///
/// The structural inverse of [`super::replacement::register_static_continuous_effects`]
/// for **all ten** families that function registers, arm for arm, in the same order.
/// None of `ContinuousEffect` / `TriggerDoubler` / `ActiveRestriction` / etc. carry an
/// origin-face tag (adding one would be a HASH-affecting wire change, out of scope),
/// so removal is a **structural match**: for each ability in the old face's list,
/// remove AT MOST the number of entries that ability's registration arm would have
/// created (one, or two for `CdaModifyPowerToughness`) via first-`position()`-match +
/// `remove()`. See [`remove_one_registration`] for the per-family match.
///
/// **Never bulk-purge by source** (e.g. `retain(|e| e.source != obj_id)`). At least
/// three other registrants share a source `ObjectId` with a transforming permanent
/// without being part of this deregistration:
/// - `resolution.rs:7447-7470` (Class level-up) pushes both a `ContinuousEffect` and
///   an `AdditionalLandPlaySource` with `source: <the Class permanent>` -- a bulk
///   purge on that permanent's own transform (were Classes ever DFCs) would delete
///   the level-up grant too.
/// - `effects/mod.rs:5574` (emblem `PlayFromGraveyardPermission`) uses a *different*
///   `ObjectId` (the emblem), so it cannot collide by construction.
/// - `effects/mod.rs:6084-6091` (`Effect::GrantFlash`) registers with `source: None`,
///   so it cannot collide with a `Some(obj_id)` match either.
///
/// Where a same-source duplicate genuinely could exist (Class level-up sharing a
/// source with the transforming permanent's own registration), first-match removal
/// picks a match closest to the removed ability's own shape (see
/// [`remove_one_registration`]'s per-arm comparisons); where the two entries happen
/// to be field-identical, removing either is observationally identical.
///
/// Ordering invariant preserved by [`apply_face_change`] (do not reorder): deregister
/// OLD (this function, reads the old face) -> flip `is_transformed` -> rebuild
/// Channel-A -> register NEW.
///
/// Drift guard: `tests/core/face_dereg_parity.rs` source-scans this function's body
/// against `register_static_continuous_effects`'s and asserts the same
/// `AbilityDefinition::<Name>` set appears in both, so a family added to one and not
/// the other fails the build rather than silently reopening this hole.
pub(crate) fn deregister_face_statics(
    state: &mut GameState,
    obj_id: ObjectId,
    old_face_abilities: &[AbilityDefinition],
) {
    for ability in old_face_abilities {
        remove_one_registration(state, obj_id, ability);
    }
}
/// The exact inverse of one `register_static_continuous_effects` match arm.
/// Removes AT MOST the number of entries that arm would have registered (one, or
/// two for `CdaModifyPowerToughness`), matching structurally on `source == obj_id`
/// plus that family's identifying fields. Arms are in the same order as
/// `register_static_continuous_effects`'s match.
fn remove_one_registration(state: &mut GameState, obj_id: ObjectId, ability: &AbilityDefinition) {
    match ability {
        // CR 604.1 / 613: a plain static continuous effect.
        AbilityDefinition::Static { continuous_effect } => {
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
        // CR 603.2d: a Panharmonicon-style trigger-doubling effect.
        AbilityDefinition::TriggerDoubling {
            filter,
            additional_triggers,
        } => {
            if let Some(pos) = state.trigger_doublers.iter().position(|d| {
                d.source == obj_id
                    && d.filter == *filter
                    && d.additional_triggers == *additional_triggers
            }) {
                state.trigger_doublers.remove(pos);
            }
        }
        // CR 614.16a: a Torpor Orb-style ETB trigger suppressor.
        AbilityDefinition::SuppressCreatureETBTriggers { filter } => {
            if let Some(pos) = state
                .etb_suppressors
                .iter()
                .position(|s| s.source == obj_id && s.filter == *filter)
            {
                state.etb_suppressors.remove(pos);
            }
        }
        // CR 604.1: a stax/action restriction (Rule of Law, Propaganda, etc.).
        AbilityDefinition::StaticRestriction { restriction } => {
            if let Some(pos) = state
                .restrictions
                .iter()
                .position(|r| r.source == obj_id && r.restriction == *restriction)
            {
                state.restrictions.remove(pos);
            }
        }
        // CR 604.3 / 613.4a: CDA Layer 7a continuous effect for dynamic P/T (sets).
        AbilityDefinition::CdaPowerToughness { power, toughness } => {
            let modification = crate::state::continuous_effect::LayerModification::SetPtDynamic {
                power: Box::new(power.clone()),
                toughness: Box::new(toughness.clone()),
            };
            if let Some(pos) = state.continuous_effects.iter().position(|e| {
                e.source == Some(obj_id)
                    && e.is_cda
                    && e.layer == crate::state::continuous_effect::EffectLayer::PtCda
                    && e.duration
                        == crate::state::continuous_effect::EffectDuration::WhileSourceOnBattlefield
                    && e.filter == EffectFilter::SingleObject(obj_id)
                    && e.modification == modification
            }) {
                state.continuous_effects.remove(pos);
            }
        }
        // CR 604.3 / 613.4c: CDA Layer 7c continuous effect(s) for dynamic P/T
        // (modifies) -- up to TWO entries, one per Some(power)/Some(toughness).
        // Build the same `modifications` vector `register_static_continuous_effects`
        // builds and remove one entry per element.
        AbilityDefinition::CdaModifyPowerToughness { power, toughness } => {
            let mut modifications: Vec<crate::state::continuous_effect::LayerModification> =
                Vec::new();
            if let Some(p) = power {
                modifications.push(
                    crate::state::continuous_effect::LayerModification::ModifyPowerDynamic {
                        amount: Box::new(p.clone()),
                        negate: false,
                    },
                );
            }
            if let Some(t) = toughness {
                modifications.push(
                    crate::state::continuous_effect::LayerModification::ModifyToughnessDynamic {
                        amount: Box::new(t.clone()),
                        negate: false,
                    },
                );
            }
            for modification in modifications {
                if let Some(pos) = state.continuous_effects.iter().position(|e| {
                    e.source == Some(obj_id)
                        && e.is_cda
                        && e.layer == crate::state::continuous_effect::EffectLayer::PtModify
                        && e.duration
                            == crate::state::continuous_effect::EffectDuration::WhileSourceOnBattlefield
                        && e.filter == EffectFilter::SingleObject(obj_id)
                        && e.modification == modification
                }) {
                    state.continuous_effects.remove(pos);
                }
            }
        }
        // CR 305.2: an additional-land-play source.
        AbilityDefinition::AdditionalLandPlays { count } => {
            if let Some(pos) = state
                .additional_land_play_sources
                .iter()
                .position(|s| s.source == obj_id && s.count == *count)
            {
                state.additional_land_play_sources.remove(pos);
            }
        }
        // CR 601.3b: a static flash grant (Yeva-style).
        AbilityDefinition::StaticFlashGrant { filter } => {
            if let Some(pos) = state
                .flash_grants
                .iter()
                .position(|f| f.source == Some(obj_id) && f.filter == *filter)
            {
                state.flash_grants.remove(pos);
            }
        }
        // CR 601.3 / 305.1: a static play-from-graveyard permission.
        AbilityDefinition::StaticPlayFromGraveyard { filter, condition } => {
            if let Some(pos) = state.play_from_graveyard_permissions.iter().position(|pm| {
                pm.source == obj_id
                    && pm.filter == *filter
                    && pm.condition == condition.as_ref().map(|c| *c.clone())
            }) {
                state.play_from_graveyard_permissions.remove(pos);
            }
        }
        // CR 601.3: a static play-from-top-of-library permission.
        AbilityDefinition::StaticPlayFromTop {
            filter,
            look_at_top,
            reveal_top,
            pay_life_instead,
            condition,
            on_cast_effect,
        } => {
            if let Some(pos) = state.play_from_top_permissions.iter().position(|pm| {
                pm.source == obj_id
                    && pm.filter == *filter
                    && pm.look_at_top == *look_at_top
                    && pm.reveal_top == *reveal_top
                    && pm.pay_life_instead == *pay_life_instead
                    && pm.condition == condition.as_ref().map(|c| *c.clone())
                    && pm.on_cast_effect == on_cast_effect.clone()
            }) {
                state.play_from_top_permissions.remove(pos);
            }
        }
        _ => {}
    }
}
