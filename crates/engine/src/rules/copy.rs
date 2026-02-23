//! Copy effects: copiable values, Layer 1 copy effects, and copy-effect helpers (CR 707).
//!
//! CR 707.2 defines which characteristics are "copiable values":
//! name, mana cost, color indicator, card type, subtype, supertype, rules text,
//! P/T, loyalty, hand modifier, life modifier.  Notably excluded: counters,
//! damage marked, attached objects, controller, zone.
//!
//! CR 707.3 describes the clone chain: a copy of a copy sees the copiable
//! values of the object being copied AFTER all copy effects on that object have
//! been applied in layer 1.  This means a Clone copying a Clone-that-copied-a-Bear
//! becomes a Bear — not a Clone.

use crate::state::{
    continuous_effect::{ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer},
    game_object::{Characteristics, ObjectId},
    player::PlayerId,
    GameState,
};

use crate::state::continuous_effect::LayerModification;

// Maximum recursion depth for clone-chain resolution (CR 707.3).
// In practice more than 5 nested copies is impossible in a real game,
// but the limit prevents potential infinite loops from bad state.
const MAX_COPY_CHAIN_DEPTH: u32 = 16;

/// CR 707.2: Returns the copiable values of `source_id`.
///
/// The copiable values are the printed characteristics of the source object,
/// modified by any Layer 1 copy effects already active on that object.
/// This implements the clone chain (CR 707.3): a copy of a copy resolves
/// through the chain so that a Clone copying a Clone-that-copied-a-Bear
/// yields Bear characteristics — not Clone characteristics.
///
/// Returns `None` if the object does not exist in state.
pub fn get_copiable_values(state: &GameState, source_id: ObjectId) -> Option<Characteristics> {
    get_copiable_values_inner(state, source_id, 0)
}

/// Internal recursive helper for clone-chain resolution.
///
/// `depth` guards against hypothetical infinite loops in malformed state.
fn get_copiable_values_inner(
    state: &GameState,
    source_id: ObjectId,
    depth: u32,
) -> Option<Characteristics> {
    if depth >= MAX_COPY_CHAIN_DEPTH {
        // Cycle or pathological depth: return the raw printed characteristics.
        return state
            .objects
            .get(&source_id)
            .map(|obj| obj.characteristics.clone());
    }

    let obj = state.objects.get(&source_id)?;

    // Start with the object's base (printed) characteristics.
    let mut chars = obj.characteristics.clone();

    // Find all Layer 1 (Copy) effects that apply to this object (CR 707.3).
    // Apply them in timestamp order (no dependency ordering needed in layer 1).
    let mut copy_effects: Vec<&ContinuousEffect> = state
        .continuous_effects
        .iter()
        .filter(|e| e.layer == EffectLayer::Copy && copy_effect_applies_to(state, e, source_id))
        .collect();

    // Sort by timestamp (earliest first = applied first).
    copy_effects.sort_by_key(|e| e.timestamp);

    for ce in copy_effects {
        if let LayerModification::CopyOf(target_id) = ce.modification {
            // Recursively resolve the target's copiable values (CR 707.3).
            if let Some(target_chars) = get_copiable_values_inner(state, target_id, depth + 1) {
                // Replace ALL copiable values with those of the target.
                // CR 707.2: copiable values = name, mana cost, color indicator, card types,
                // supertypes, subtypes, rules text, P/T, loyalty, hand modifier, life modifier.
                // NOT copied: counters, damage, attached objects, controller, zone.
                chars.name = target_chars.name;
                chars.mana_cost = target_chars.mana_cost;
                chars.colors = target_chars.colors;
                chars.color_indicator = target_chars.color_indicator;
                chars.supertypes = target_chars.supertypes;
                chars.card_types = target_chars.card_types;
                chars.subtypes = target_chars.subtypes;
                chars.rules_text = target_chars.rules_text;
                chars.abilities = target_chars.abilities;
                chars.keywords = target_chars.keywords;
                chars.mana_abilities = target_chars.mana_abilities;
                chars.activated_abilities = target_chars.activated_abilities;
                chars.triggered_abilities = target_chars.triggered_abilities;
                chars.power = target_chars.power;
                chars.toughness = target_chars.toughness;
                chars.loyalty = target_chars.loyalty;
                chars.defense = target_chars.defense;
            }
        }
    }

    Some(chars)
}

/// Returns true if a Layer 1 copy effect applies to the given object.
///
/// Uses `EffectFilter::SingleObject` semantics — copy effects target specific objects.
/// Other filter types (AllCreatures, etc.) are not valid for Layer 1 copy effects.
fn copy_effect_applies_to(
    state: &GameState,
    effect: &ContinuousEffect,
    object_id: ObjectId,
) -> bool {
    // Only active effects apply.
    if !super::layers::is_effect_active(state, effect) {
        return false;
    }
    match &effect.filter {
        EffectFilter::SingleObject(id) => *id == object_id,
        // Copy effects should use SingleObject; other filters are not applicable.
        _ => false,
    }
}

/// Create a Layer 1 copy continuous effect: `copier_id` copies `source_id`.
///
/// Returns a `ContinuousEffect` with the following properties:
/// - Layer: Copy (Layer 1)
/// - Filter: SingleObject(copier_id)  — applies only to the copier
/// - Modification: CopyOf(source_id)  — the object being copied
/// - Duration: Indefinite             — copy effects persist until removed
/// - Timestamp: current game timestamp
///
/// Caller must push the returned effect into `state.continuous_effects`.
/// This function does NOT modify state — it only constructs the effect struct.
/// The `controller` parameter is accepted for API symmetry with future
/// control-changing copy effects (e.g., "copy under opponent's control").
pub fn create_copy_effect(
    state: &mut GameState,
    copier_id: ObjectId,
    source_id: ObjectId,
    _controller: PlayerId,
) -> ContinuousEffect {
    let ts = state.timestamp_counter;
    state.timestamp_counter += 1;
    let effect_id = state.timestamp_counter;
    state.timestamp_counter += 1;

    ContinuousEffect {
        id: EffectId(effect_id),
        source: Some(copier_id),
        timestamp: ts,
        layer: EffectLayer::Copy,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::SingleObject(copier_id),
        modification: LayerModification::CopyOf(source_id),
        is_cda: false,
    }
}
