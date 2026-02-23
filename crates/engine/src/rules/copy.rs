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
    error::GameStateError,
    game_object::{Characteristics, ObjectId},
    player::PlayerId,
    stack::StackObject,
    GameState,
};

use crate::rules::events::GameEvent;
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

/// CR 707.10: Create a copy of a spell on the stack.
///
/// A copy of a spell on the stack has the same characteristics as the original:
/// same kind, same controller, same targets. The copy is NOT cast — no "whenever
/// you cast a spell" triggers fire for the copy (CR 707.10c).
///
/// The copy is pushed onto the stack above the original (above = later index in
/// `state.stack_objects`, since LIFO means the last entry resolves first).
///
/// Returns the new `StackObject` ID (as `ObjectId`) or an error if the source
/// stack object is not found.
///
/// `choose_new_targets` is reserved for future interactive use. In M9.4,
/// the deterministic fallback keeps the same targets as the original.
pub fn copy_spell_on_stack(
    state: &mut GameState,
    stack_object_id: ObjectId,
    controller: PlayerId,
    _choose_new_targets: bool,
) -> Result<(ObjectId, GameEvent), GameStateError> {
    // Find the stack object to copy.
    let original = state
        .stack_objects
        .iter()
        .find(|s| s.id == stack_object_id)
        .ok_or(GameStateError::ObjectNotFound(stack_object_id))?
        .clone();

    // CR 707.10: The copy has the same characteristics as the original.
    // Assign a new unique ID for the copy stack object.
    let copy_id = state.next_object_id();

    // CR 707.10: Copies have no physical card — is_copy = true signals resolution
    // to skip the zone-move of the source card. The copy still executes the same
    // effect as the original.
    let copy = StackObject {
        id: copy_id,
        controller,
        kind: original.kind.clone(),
        targets: original.targets.clone(),
        cant_be_countered: original.cant_be_countered,
        is_copy: true,
    };

    // Push the copy onto the stack (above the original).
    state.stack_objects.push_back(copy);

    let event = GameEvent::SpellCopied {
        original_stack_id: stack_object_id,
        copy_stack_id: copy_id,
        controller,
    };

    Ok((copy_id, event))
}

/// CR 702.40a: Create N copies of a storm spell on the stack.
///
/// Called when a spell with the Storm keyword resolves its storm trigger.
/// N = the caster's `spells_cast_this_turn - 1` (copies for each OTHER spell
/// cast before the storm spell this turn). If N ≤ 0, no copies are created.
///
/// Each copy is pushed onto the stack above the original in sequence. All copies
/// resolve before the original (LIFO).
///
/// Returns the events generated (one `SpellCopied` per copy).
pub fn create_storm_copies(
    state: &mut GameState,
    stack_object_id: ObjectId,
    controller: PlayerId,
    storm_count: u32,
) -> Vec<GameEvent> {
    let mut events = Vec::new();
    for _ in 0..storm_count {
        match copy_spell_on_stack(state, stack_object_id, controller, false) {
            Ok((_, evt)) => events.push(evt),
            Err(_) => break, // stack object disappeared; abort
        }
    }
    events
}

/// CR 702.40a: Compute the storm count for the current caster.
///
/// Storm count = number of OTHER spells cast before the storm spell this turn.
/// This is `spells_cast_this_turn - 1` (the storm spell itself is already counted).
/// Returns 0 if no other spells were cast or if player state is unavailable.
pub fn storm_count(state: &GameState, caster: PlayerId) -> u32 {
    state
        .players
        .get(&caster)
        .map(|p| p.spells_cast_this_turn.saturating_sub(1))
        .unwrap_or(0)
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
