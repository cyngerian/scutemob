//! The layer system: calculate effective characteristics of game objects (CR 613).
//!
//! Continuous effects modify object characteristics and must be applied in a strict
//! order across eight layers (CR 613.1). Within each layer, effects apply by:
//! 1. CDAs first (CR 613.3)
//! 2. Then all other effects in dependency order (CR 613.8), falling back to
//!    timestamp order (CR 613.7) for independent effects and circular dependencies.
//!
//! The main entry point is `calculate_characteristics`, which returns the effective
//! characteristics of any game object after applying all active continuous effects.

use im::OrdSet;

use crate::state::{
    continuous_effect::{
        ContinuousEffect, EffectDuration, EffectFilter, EffectLayer, LayerModification,
    },
    game_object::{Characteristics, ObjectId},
    types::{CardType, CounterType, SubType, SuperType},
    zone::ZoneId,
    GameState,
};

/// Calculate the effective characteristics of an object after applying all active
/// continuous effects through the layer system (CR 613).
///
/// Starts with the object's base (printed) characteristics and applies all active
/// continuous effects in layer order (1 → 7d), with timestamp and dependency ordering
/// within each layer.
///
/// Returns `None` if the object does not exist in the game state.
pub fn calculate_characteristics(
    state: &GameState,
    object_id: ObjectId,
) -> Option<Characteristics> {
    let obj = state.objects.get(&object_id)?;
    let obj_zone = obj.zone;
    let mut chars = obj.characteristics.clone();

    // Collect all active continuous effects once (avoids repeated filtering).
    let active_effects: Vec<&ContinuousEffect> = state
        .continuous_effects
        .iter()
        .filter(|e| is_effect_active(state, e))
        .collect();

    // Process layers in order (CR 613.1). Layer 7 is split into sublayers 7a–7d.
    let layers_in_order = [
        EffectLayer::Copy,
        EffectLayer::Control,
        EffectLayer::Text,
        EffectLayer::TypeChange,
        EffectLayer::ColorChange,
        EffectLayer::Ability,
        EffectLayer::PtCda,
        EffectLayer::PtSet,
        EffectLayer::PtModify,
        EffectLayer::PtSwitch,
    ];

    for &layer in &layers_in_order {
        // Gather effects for this layer that apply to this object.
        // The filter is evaluated against `chars` as modified by earlier layers —
        // this is correct because type changes from layer 4 affect whether "AllCreatures"
        // filters match in layers 6 and 7.
        let layer_effects: Vec<&ContinuousEffect> = active_effects
            .iter()
            .copied()
            .filter(|e| {
                e.layer == layer && effect_applies_to(state, e, object_id, obj_zone, &chars)
            })
            .collect();

        // Sort by CDAs first, then dependency/timestamp order (CR 613.3, 613.7, 613.8).
        let ordered = resolve_layer_order(layer_effects);

        // The mana value comes from the base mana cost (printed on the card).
        // Used by SetPtToManaValue modifications (Opalescence-style).
        let mana_value = chars
            .mana_cost
            .as_ref()
            .map(|c| c.mana_value())
            .unwrap_or(0);

        for effect in ordered {
            apply_layer_modification(&mut chars, &effect.modification, mana_value);
        }

        // Layer 7c (PtModify): also apply counter P/T contributions (CR 613.4c).
        // Counters are not modeled as ContinuousEffects — they live on the GameObject.
        // We apply them here (at the correct layer position) regardless of whether there
        // are any static Layer 7c effects.
        if layer == EffectLayer::PtModify {
            // Re-borrow: obj is still valid since we haven't mutated state.
            // MR-M5-01: if-let instead of expect — object may have been removed by an effect.
            let Some(obj_ref) = state.objects.get(&object_id) else {
                break;
            };
            let plus_ones = obj_ref
                .counters
                .get(&CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0) as i32;
            let minus_ones = obj_ref
                .counters
                .get(&CounterType::MinusOneMinusOne)
                .copied()
                .unwrap_or(0) as i32;
            let net = plus_ones - minus_ones;
            if net != 0 {
                if let Some(p) = &mut chars.power {
                    *p += net;
                }
                if let Some(t) = &mut chars.toughness {
                    *t += net;
                }
            }
        }
    }

    Some(chars)
}

/// Returns true if a continuous effect is currently active.
///
/// An effect is active when its duration condition is met:
/// - `WhileSourceOnBattlefield`: source object exists and is on the battlefield
/// - `UntilEndOfTurn`: always active (removed explicitly by `expire_end_of_turn_effects`)
/// - `Indefinite`: always active
pub fn is_effect_active(state: &GameState, effect: &ContinuousEffect) -> bool {
    match effect.duration {
        EffectDuration::WhileSourceOnBattlefield => match effect.source {
            Some(source_id) => state
                .objects
                .get(&source_id)
                .map(|obj| obj.zone == ZoneId::Battlefield)
                .unwrap_or(false),
            // No source means the effect is inherently active (e.g., from a spell).
            None => true,
        },
        // Active until explicitly removed during cleanup (CR 514.2).
        EffectDuration::UntilEndOfTurn => true,
        EffectDuration::Indefinite => true,
    }
}

/// Returns true if a continuous effect applies to the given object.
///
/// The filter is evaluated against `chars`, which reflects all modifications applied
/// by earlier layers in the current `calculate_characteristics` call. This correctly
/// handles cases like Opalescence making enchantments into creatures (layer 4) before
/// Humility's "AllCreatures" filter is evaluated (layers 6 and 7).
fn effect_applies_to(
    state: &GameState,
    effect: &ContinuousEffect,
    object_id: ObjectId,
    obj_zone: ZoneId,
    chars: &Characteristics,
) -> bool {
    match &effect.filter {
        EffectFilter::SingleObject(id) => *id == object_id,

        EffectFilter::AllCreatures => {
            obj_zone == ZoneId::Battlefield && chars.card_types.contains(&CardType::Creature)
        }

        EffectFilter::AllLands => {
            obj_zone == ZoneId::Battlefield && chars.card_types.contains(&CardType::Land)
        }

        EffectFilter::AllNonbasicLands => {
            obj_zone == ZoneId::Battlefield
                && chars.card_types.contains(&CardType::Land)
                && !chars.supertypes.contains(&SuperType::Basic)
        }

        EffectFilter::AllEnchantments => {
            obj_zone == ZoneId::Battlefield && chars.card_types.contains(&CardType::Enchantment)
        }

        EffectFilter::AllNonAuraEnchantments => {
            obj_zone == ZoneId::Battlefield
                && chars.card_types.contains(&CardType::Enchantment)
                && !chars.subtypes.contains(&SubType("Aura".to_string()))
        }

        EffectFilter::AllPermanents => {
            obj_zone == ZoneId::Battlefield
                && (chars.card_types.contains(&CardType::Creature)
                    || chars.card_types.contains(&CardType::Artifact)
                    || chars.card_types.contains(&CardType::Enchantment)
                    || chars.card_types.contains(&CardType::Land)
                    || chars.card_types.contains(&CardType::Planeswalker)
                    || chars.card_types.contains(&CardType::Battle))
        }

        EffectFilter::AllCardsInGraveyards => matches!(obj_zone, ZoneId::Graveyard(_)),

        EffectFilter::ControlledBy(player_id) => {
            obj_zone == ZoneId::Battlefield
                && state
                    .objects
                    .get(&object_id)
                    .map(|o| o.controller == *player_id)
                    .unwrap_or(false)
        }

        EffectFilter::CreaturesControlledBy(player_id) => {
            obj_zone == ZoneId::Battlefield
                && chars.card_types.contains(&CardType::Creature)
                && state
                    .objects
                    .get(&object_id)
                    .map(|o| o.controller == *player_id)
                    .unwrap_or(false)
        }
    }
}

/// Apply a single layer modification to the given characteristics.
///
/// `mana_value` is the object's printed mana value, used for `SetPtToManaValue`.
fn apply_layer_modification(
    chars: &mut Characteristics,
    modification: &LayerModification,
    mana_value: u32,
) {
    match modification {
        // Layer 1: Copy effects — full implementation in M7; placeholder here.
        LayerModification::CopyOf(_target) => {
            // TODO M7: Replace copiable values with those of the target object.
        }

        // Layer 2: Control-changing — controller lives on GameObject, not Characteristics.
        // Control-change effects are applied to obj.controller separately.
        LayerModification::SetController(_) => {
            // Handled outside calculate_characteristics (controller is on GameObject).
        }

        // Layer 4: Type-changing
        LayerModification::SetTypeLine {
            supertypes,
            card_types,
            subtypes,
        } => {
            chars.supertypes = supertypes.clone();
            chars.card_types = card_types.clone();
            chars.subtypes = subtypes.clone();
        }

        LayerModification::AddCardTypes(types) => {
            for t in types {
                chars.card_types.insert(*t);
            }
        }

        LayerModification::AddSubtypes(subtypes) => {
            for s in subtypes {
                chars.subtypes.insert(s.clone());
            }
        }

        LayerModification::LoseAllSubtypes => {
            chars.subtypes = OrdSet::new();
        }

        // Layer 5: Color-changing
        LayerModification::SetColors(colors) => {
            chars.colors = colors.clone();
        }

        LayerModification::AddColors(colors) => {
            for c in colors {
                chars.colors.insert(*c);
            }
        }

        LayerModification::BecomeColorless => {
            chars.colors = OrdSet::new();
        }

        // Layer 6: Ability-adding/removing
        LayerModification::AddKeyword(kw) => {
            chars.keywords.insert(*kw);
        }

        LayerModification::AddKeywords(kws) => {
            for kw in kws {
                chars.keywords.insert(*kw);
            }
        }

        LayerModification::RemoveAllAbilities => {
            // Removes all static, activated, triggered, and keyword abilities.
            // The continuous effect itself persists (CR 611.2d — effects from removed
            // abilities continue if they were already in effect).
            chars.keywords = OrdSet::new();
            chars.mana_abilities = im::Vector::new();
            chars.activated_abilities = Vec::new();
            chars.triggered_abilities = Vec::new();
            chars.abilities = im::Vector::new();
        }

        LayerModification::RemoveKeyword(kw) => {
            chars.keywords.remove(kw);
        }

        // Layer 7a: CDAs
        LayerModification::SetPtViaCda { power, toughness } => {
            chars.power = Some(*power);
            chars.toughness = Some(*toughness);
        }

        LayerModification::SetPtToManaValue => {
            let mv = mana_value as i32;
            chars.power = Some(mv);
            chars.toughness = Some(mv);
        }

        // Layer 7b: P/T-setting
        LayerModification::SetPowerToughness { power, toughness } => {
            chars.power = Some(*power);
            chars.toughness = Some(*toughness);
        }

        // Layer 7c: P/T-modifying
        LayerModification::ModifyPower(delta) => {
            if let Some(p) = &mut chars.power {
                *p += delta;
            }
        }

        LayerModification::ModifyToughness(delta) => {
            if let Some(t) = &mut chars.toughness {
                *t += delta;
            }
        }

        LayerModification::ModifyBoth(delta) => {
            if let Some(p) = &mut chars.power {
                *p += delta;
            }
            if let Some(t) = &mut chars.toughness {
                *t += delta;
            }
        }

        // Layer 7d: P/T-switching
        LayerModification::SwitchPowerToughness => {
            let old_p = chars.power;
            let old_t = chars.toughness;
            chars.power = old_t;
            chars.toughness = old_p;
        }
    }
}

/// Sort effects for a single layer in the order they must be applied.
///
/// Ordering rules (CR 613.3, 613.7, 613.8):
/// 1. CDAs apply first, in timestamp order (CR 613.3).
/// 2. Non-CDAs apply after CDAs, in dependency-aware order (CR 613.8), falling back
///    to timestamp order for independent effects and circular dependencies (CR 613.7).
fn resolve_layer_order(effects: Vec<&ContinuousEffect>) -> Vec<&ContinuousEffect> {
    if effects.is_empty() {
        return effects;
    }

    // Partition into CDAs and non-CDAs.
    let (mut cdas, non_cdas): (Vec<_>, Vec<_>) = effects.into_iter().partition(|e| e.is_cda);

    // CDAs apply in timestamp order (CR 613.3).
    cdas.sort_by_key(|e| e.timestamp);

    // Non-CDAs: dependency-aware topological sort, timestamp as tiebreaker.
    let sorted_non_cdas = toposort_with_timestamp_fallback(non_cdas);

    cdas.into_iter().chain(sorted_non_cdas).collect()
}

/// Topologically sort effects by dependency order (CR 613.8).
///
/// If A depends on B, B is applied first (B → A in the output order).
/// Circular dependencies fall back to timestamp order (CR 613.8b).
/// Independent effects are also ordered by timestamp (CR 613.7).
fn toposort_with_timestamp_fallback(mut effects: Vec<&ContinuousEffect>) -> Vec<&ContinuousEffect> {
    let n = effects.len();
    if n <= 1 {
        return effects;
    }

    // Sort by timestamp as the baseline ordering (CR 613.7).
    // The topological sort will preserve timestamp order for independent effects.
    effects.sort_by_key(|e| e.timestamp);

    // Build the dependency graph.
    // in_degree[i]: number of effects that must be applied before effects[i].
    // adj[j]: list of i where effects[i] depends on effects[j] (j must come before i).
    let mut in_degree = vec![0u32; n];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

    for i in 0..n {
        for j in 0..n {
            if i != j && depends_on(effects[i], effects[j]) {
                // effects[i] depends on effects[j]: j must be applied before i.
                if !adj[j].contains(&i) {
                    adj[j].push(i);
                    in_degree[i] += 1;
                }
            }
        }
    }

    // Kahn's algorithm: process nodes with in-degree 0, in index order (= timestamp order).
    let mut ready: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut result: Vec<&ContinuousEffect> = Vec::with_capacity(n);

    while !ready.is_empty() {
        // Take the first ready node (already in timestamp/index order).
        let i = ready.remove(0);
        result.push(effects[i]);

        for &j in &adj[i] {
            in_degree[j] -= 1;
            if in_degree[j] == 0 {
                // Insert maintaining sorted order (by index = by timestamp).
                let pos = ready.partition_point(|&k| k < j);
                ready.insert(pos, j);
            }
        }
    }

    // Cycle handling (CR 613.8b): any remaining effects form a dependency loop.
    // Apply them in timestamp order (index order = timestamp order since effects is sorted).
    if result.len() < n {
        // Find effects not yet emitted (O(n²), but n is tiny in practice — ≤ 20 effects).
        for effect in &effects {
            let was_emitted = result.iter().any(|e| std::ptr::eq(*e, *effect));
            if !was_emitted {
                result.push(effect);
            }
        }
    }

    result
}

/// Returns true if effect `a` depends on effect `b` within the same layer (CR 613.8a).
///
/// A depends on B if:
/// (a) They are in the same layer (caller ensures this).
/// (b) Applying B would change the text, existence, what A applies to, or what A does.
/// (c) Neither is a CDA, OR both are CDAs.
///
/// If A depends on B, B must be applied before A (regardless of timestamp).
fn depends_on(a: &ContinuousEffect, b: &ContinuousEffect) -> bool {
    // CR 613.8a(c): CDAs and non-CDAs cannot depend on each other.
    if a.is_cda != b.is_cda {
        return false;
    }

    match (&a.modification, &b.modification) {
        // --- Layer 4 dependencies ---
        //
        // A "set type line" effect depends on "add card types" or "add subtypes" effects.
        //
        // Rationale: If we apply "add subtypes" first, then "set type line" correctly
        // overrides/replaces the added subtypes. If we apply "set type line" first, then
        // "add subtypes" would still add back subtypes, giving a wrong result.
        //
        // This implements the Blood Moon + Urborg dependency: Blood Moon's SetTypeLine
        // depends on Urborg's AddSubtypes, so Urborg applies first (adding Swamp) and
        // then Blood Moon applies (setting to Mountain, overriding Swamp). Result: Mountain.
        //
        // Without this dependency, if Blood Moon is older (lower timestamp), Urborg would
        // apply second and add Swamp after Blood Moon set the type, giving "Mountain, Swamp."
        (LayerModification::SetTypeLine { .. }, LayerModification::AddSubtypes(_))
        | (LayerModification::SetTypeLine { .. }, LayerModification::AddCardTypes(_)) => {
            // SetTypeLine (a) depends on AddSubtypes/AddCardTypes (b):
            // b must be applied before a.
            true
        }

        // All other combinations are independent (apply in timestamp order).
        _ => false,
    }
}

/// Remove all "until end of turn" continuous effects during the Cleanup step (CR 514.2).
///
/// Called by `turn_actions::cleanup_actions` immediately after clearing damage.
pub fn expire_end_of_turn_effects(state: &mut GameState) {
    let keep: im::Vector<ContinuousEffect> = state
        .continuous_effects
        .iter()
        .filter(|e| e.duration != EffectDuration::UntilEndOfTurn)
        .cloned()
        .collect();
    state.continuous_effects = keep;
}
