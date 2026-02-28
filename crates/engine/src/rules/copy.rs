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
    stack::{StackObject, StackObjectKind},
    types::CardType,
    zone::ZoneId,
    GameState,
};

use super::layers::calculate_characteristics;

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
        // CR 707.10: Copies are never cast, so cast_with_flashback is always false.
        cast_with_flashback: false,
        // CR 702.33d ruling: copies of kicked spells on the stack are also kicked.
        kicker_times_paid: original.kicker_times_paid,
        // CR 702.74a: Copies are never cast — they cannot be evoked.
        was_evoked: false,
        // CR 702.103c: Copies of bestowed spells are also bestowed.
        was_bestowed: original.was_bestowed,
        // CR 702.35a: Copies are never cast, so cast_with_madness is always false.
        cast_with_madness: false,
        // CR 702.94a: Copies are never cast, so cast_with_miracle is always false.
        cast_with_miracle: false,
        // CR 702.138b: Copies are never cast, so was_escaped is always false.
        was_escaped: false,
        // CR 702.143a: Copies are never cast, so cast_with_foretell is always false.
        cast_with_foretell: false,
        was_buyback_paid: false,
        // CR 702.62a: Copies are never cast via suspend.
        was_suspended: false,
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

/// CR 702.85: Resolve cascade for the given spell.
///
/// Cascade triggers when the cascade spell is cast. Resolution:
/// 1. Exile cards from the top of the caster's library one at a time.
/// 2. Stop when a nonland card with mana value STRICTLY LESS THAN the
///    cascade spell's mana value is found.
/// 3. The caster may cast that card without paying its mana cost
///    (deterministic M9.4: always cast if the card is castable — nonland).
/// 4. Put all remaining exiled cards (those NOT cast) on the bottom of
///    the library in a random order (deterministic: ObjectId ascending order).
///
/// CR 708.4: For split card mana value when not on the stack, use the combined
/// mana value of both halves. Split cards are not yet implemented; this rule
/// is documented here for future reference — `spell_mana_value` is always the
/// full printed mana value of the cascade spell passed by the caller.
///
/// Returns (events, cascaded_card_id) where `cascaded_card_id` is the ObjectId
/// of the card placed on the stack (if any was found and cast), or `None` if the
/// library was empty or no qualifying card was found.
///
/// NOTE: The cascade card goes onto the stack WITHOUT going through the normal
/// CastSpell flow (no priority check, no mana payment). It is cast "free" and
/// the trigger fires separately from normal casting. However, it IS cast — cascade
/// does trigger "whenever you cast a spell" abilities for the cascaded-into spell
/// (CR 702.85c).
pub fn resolve_cascade(
    state: &mut GameState,
    caster: PlayerId,
    spell_mana_value: u32,
) -> (Vec<GameEvent>, Option<ObjectId>) {
    let mut events = Vec::new();
    let mut exiled_ids: Vec<ObjectId> = Vec::new();
    let mut cast_id: Option<ObjectId> = None;

    // Step 1–2: Exile cards one at a time from the top of the caster's library.
    loop {
        // Get top card of library (ordered zone: first element = top).
        let library_zone_id = ZoneId::Library(caster);
        let top_card_id = {
            let library = state.zones.get(&library_zone_id);
            // CR 702.85: exile from TOP of library — top = last element in ordered zone
            // (push_back appends; draw_card uses top() = last()).
            library.and_then(|z| z.top())
        };

        let Some(top_id) = top_card_id else {
            // Library empty: stop searching.
            break;
        };

        // Exile the top card.
        let (exile_id, _old) = match state.move_object_to_zone(top_id, ZoneId::Exile) {
            Ok(r) => r,
            Err(_) => break,
        };
        exiled_ids.push(exile_id);

        // Check if this is a qualifying card: nonland with mana value < spell_mana_value.
        // CR 702.85a: use the card's actual characteristics (applying continuous effects such
        // as Mycosynth Lattice or Trinisphere), falling back to raw characteristics if the
        // layer system has no information for the object (MR-M9.4-04).
        let calc_chars = calculate_characteristics(state, exile_id);
        let is_land = calc_chars
            .as_ref()
            .map(|c| c.card_types.contains(&CardType::Land))
            .unwrap_or_else(|| {
                state
                    .objects
                    .get(&exile_id)
                    .map(|obj| obj.characteristics.card_types.contains(&CardType::Land))
                    .unwrap_or(false)
            });

        let card_mv = calc_chars
            .as_ref()
            .and_then(|c| c.mana_cost.as_ref())
            .map(|mc| mc.mana_value())
            .unwrap_or_else(|| {
                state
                    .objects
                    .get(&exile_id)
                    .and_then(|obj| obj.characteristics.mana_cost.as_ref())
                    .map(|mc| mc.mana_value())
                    .unwrap_or(0)
            });

        if !is_land && card_mv < spell_mana_value {
            // Found the qualifying card. Cast it for free (deterministic: always cast).
            // Step 3: Put it on the stack without paying mana cost.
            let stack_entry_id = state.next_object_id();
            // Move card from exile to stack zone (new ObjectId via CR 400.7).
            let (stack_source_id, _old) = match state.move_object_to_zone(exile_id, ZoneId::Stack) {
                Ok(r) => r,
                Err(_) => {
                    // Failed to move card — leave in exile and stop.
                    break;
                }
            };
            // Remove it from the exiled list (it was cast, not put on bottom).
            exiled_ids.pop();

            // Create a StackObject for the cascaded spell.
            // CR 702.85c: cascade IS a cast — it triggers "whenever you cast a spell".
            let stack_obj = StackObject {
                id: stack_entry_id,
                controller: caster,
                kind: StackObjectKind::Spell {
                    source_object: stack_source_id,
                },
                targets: vec![],
                cant_be_countered: false,
                is_copy: false,
                cast_with_flashback: false,
                kicker_times_paid: 0,
                was_evoked: false,
                was_bestowed: false,
                cast_with_madness: false,
                cast_with_miracle: false,
                was_escaped: false,
                cast_with_foretell: false,
                was_buyback_paid: false,
                was_suspended: false,
            };
            state.stack_objects.push_back(stack_obj);

            // CR 702.85c: cascade triggers "whenever you cast" — increment spells_cast_this_turn.
            // Use saturating_add to match defensive overflow handling used elsewhere (CR 702.85).
            if let Some(ps) = state.players.get_mut(&caster) {
                ps.spells_cast_this_turn = ps.spells_cast_this_turn.saturating_add(1);
            }

            events.push(GameEvent::SpellCast {
                player: caster,
                stack_object_id: stack_entry_id,
                source_object_id: stack_source_id,
            });
            events.push(GameEvent::CascadeCast {
                player: caster,
                card_id: stack_source_id,
            });

            cast_id = Some(stack_source_id);
            break;
        }
        // Non-qualifying card: continue to next card.
    }

    // Emit CascadeExiled for all cards that were exiled (not including the cast card).
    if !exiled_ids.is_empty() {
        events.insert(
            0,
            GameEvent::CascadeExiled {
                player: caster,
                cards_exiled: exiled_ids.clone(),
            },
        );
    }

    // Step 4: Put remaining exiled cards on the bottom of the library (CR 702.85a).
    // Deterministic order: sort by ObjectId (ascending) so placement is reproducible.
    // Each card is moved to position 0 (the front/bottom) of the library zone via
    // move_object_to_bottom_of_zone, which uses Zone::push_front (MR-M9.4-08).
    // Because we iterate in ascending ObjectId order and each card is inserted at
    // position 0, the last card in the sort order ends up at the very bottom.
    exiled_ids.sort();
    let library_zone_id = ZoneId::Library(caster);
    for exile_id in exiled_ids {
        // MR-M9.4-01: If the zone move fails (e.g., the object was already removed
        // from exile), the card is left in its current zone rather than silently
        // disappearing. The card NOT reaching the library is detectable via the
        // CascadeExiled event listing cards that did not end up in the library.
        if let Err(_e) = state.move_object_to_bottom_of_zone(exile_id, library_zone_id) {
            // Card stays in its current zone — no silent data loss.
            // This branch is unreachable in well-formed game state.
        }
    }

    (events, cast_id)
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
