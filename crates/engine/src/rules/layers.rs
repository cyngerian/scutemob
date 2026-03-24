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
use crate::state::{
    continuous_effect::{
        ContinuousEffect, EffectDuration, EffectFilter, EffectLayer, LayerModification,
    },
    game_object::{Characteristics, Designations, ObjectId},
    types::{CardType, CounterType, KeywordAbility, SubType, SuperType},
    zone::ZoneId,
    GameState,
};
use im::OrdSet;
use std::collections::VecDeque;
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
    // CR 701.60c: A suspected permanent has menace and "This creature can't block"
    // for as long as it's suspected. Menace is inserted into base keywords BEFORE the
    // layer loop so that Layer 6 ability-removal effects (e.g., Humility) can correctly
    // strip it. This matches the ruling (2024-02-02): if a suspected creature loses all
    // abilities, it loses menace, but the suspected designation itself persists.
    //
    // "Can't block" is enforced separately in combat.rs (like Decayed) by checking
    // `obj.designations.contains(Designations::SUSPECTED)` directly. The designation persists through ability-removal;
    // only the GRANTS (menace, can't-block) are affected by ability removal.
    if obj.designations.contains(Designations::SUSPECTED) && obj.zone == ZoneId::Battlefield {
        chars.keywords.insert(KeywordAbility::Menace);
    }
    // CR 701.54c (ring level >= 1): Ring-bearer is legendary.
    //
    // The Legendary supertype is applied pre-layer-loop (Layer 4 analogue) so that
    // Layer 6 ability-removal effects (e.g., Humility) do not strip it — supertypes
    // are set in Layer 4, not Layer 6.
    //
    // Any creature with the RING_BEARER designation always has ring_level >= 1, since
    // ring_level is advanced before the ring-bearer is chosen (CR 701.54c).
    // We do not verify ring_level here — the designation itself implies level >= 1.
    if obj.designations.contains(Designations::RING_BEARER) && obj.zone == ZoneId::Battlefield {
        chars.supertypes.insert(SuperType::Legendary);
    }
    // CR 712.8d/712.8e: Double-Faced Card face resolution.
    //
    // When a DFC permanent has its back face up (is_transformed == true), its effective
    // characteristics are derived from the back face (CR 712.8d). However, its mana value
    // is calculated from the FRONT face's mana cost (CR 712.8e).
    //
    // This runs BEFORE the merged_components check so that a mutated DFC permanent's
    // topmost component can itself be transformed.
    //
    // CR 712.8a: DFCs in non-battlefield zones always use front face characteristics.
    // Since is_transformed is reset on zone changes (CR 400.7), this is automatic:
    // is_transformed is always false outside the battlefield.
    if obj.is_transformed {
        if let Some(ref card_id) = obj.card_id {
            if let Some(def) = state.card_registry.get(card_id.clone()) {
                if let Some(ref back_face) = def.back_face {
                    // CR 712.8d: Use back face characteristics as the base.
                    // CR 712.8e: mana_value is computed from the front face's mana cost
                    // (stored in def.mana_cost). We keep back face's mana_cost in chars
                    // for color derivation (CR 105.2), but the engine's mana_value()
                    // lookups must use def.mana_cost when obj.is_transformed is true.
                    // See mana_value() helper in state/mod.rs for the override.
                    chars.name = back_face.name.clone();
                    chars.mana_cost = back_face.mana_cost.clone();
                    chars.card_types = back_face.types.card_types.clone();
                    chars.subtypes = back_face.types.subtypes.clone();
                    chars.supertypes = back_face.types.supertypes.clone();
                    // Note: oracle_text is not part of Characteristics (it's on CardDefinition).
                    // The UI/display layer reads oracle text from CardDefinition, not Characteristics.
                    chars.keywords = OrdSet::new();
                    // Apply back face abilities to chars.keywords
                    for ability in &back_face.abilities {
                        if let crate::cards::card_definition::AbilityDefinition::Keyword(kw) =
                            ability
                        {
                            chars.keywords.insert(kw.clone());
                        }
                    }
                    chars.power = back_face.power;
                    chars.toughness = back_face.toughness;
                    // CR 204: color indicator overrides mana-cost-derived colors for back faces
                    // that have no mana cost (e.g., Insectile Aberration is blue via indicator).
                    if let Some(ref color_indicator) = back_face.color_indicator {
                        chars.colors = color_indicator.iter().cloned().collect::<im::OrdSet<_>>();
                    } else if let Some(ref mc) = back_face.mana_cost {
                        chars.colors = crate::rules::casting::colors_from_mana_cost(mc);
                    }
                    // CR 712.20: "As [this permanent] transforms..." abilities are applied
                    // during transformation, not here. No action needed at characteristics time.
                }
                // CR 701.27c: If back_face is None, transform is a no-op — is_transformed
                // should never be true for non-DFCs, but guard defensively.
            }
        }
    }
    // CR 712.8g: Melded permanent face resolution.
    //
    // When a permanent is melded (meld_component is Some), its effective characteristics
    // are derived from the combined back face of the meld pair. The meld pair's back_face
    // is stored on the melded CardDefinition (referenced by meld_pair.melded_card_id).
    //
    // CR 712.8g: Mana value of a melded permanent = sum of both front face mana values.
    // CR 712.4c: Meld cards cannot be transformed — ignored by this code (is_transformed
    // is never true for melded permanents since meld doesn't set it).
    if obj.meld_component.is_some() {
        if let Some(ref card_id) = obj.card_id {
            if let Some(def) = state.card_registry.get(card_id.clone()) {
                if let Some(ref meld_pair) = def.meld_pair {
                    if let Some(melded_def) =
                        state.card_registry.get(meld_pair.melded_card_id.clone())
                    {
                        if let Some(ref melded_face) = melded_def.back_face {
                            chars.name = melded_face.name.clone();
                            // CR 712.8g: mana value = sum of both front face mana values.
                            // The melded back face has no mana cost (None → 0), so we
                            // compute the sum explicitly from both front faces and store it
                            // as a synthetic ManaCost with generic = sum.
                            let source_mv =
                                def.mana_cost.as_ref().map(|c| c.mana_value()).unwrap_or(0);
                            let partner_mv = obj
                                .meld_component
                                .as_ref()
                                .and_then(|pid| state.card_registry.get(pid.clone()))
                                .and_then(|pd| pd.mana_cost.as_ref().map(|c| c.mana_value()))
                                .unwrap_or(0);
                            let combined_mv = source_mv + partner_mv;
                            chars.mana_cost = if combined_mv > 0 {
                                Some(crate::state::game_object::ManaCost {
                                    generic: combined_mv,
                                    ..Default::default()
                                })
                            } else {
                                None
                            };
                            chars.card_types = melded_face.types.card_types.clone();
                            chars.subtypes = melded_face.types.subtypes.clone();
                            chars.supertypes = melded_face.types.supertypes.clone();
                            chars.keywords = OrdSet::new();
                            for ability in &melded_face.abilities {
                                if let crate::cards::card_definition::AbilityDefinition::Keyword(
                                    kw,
                                ) = ability
                                {
                                    chars.keywords.insert(kw.clone());
                                }
                            }
                            chars.power = melded_face.power;
                            chars.toughness = melded_face.toughness;
                            if let Some(ref color_indicator) = melded_face.color_indicator {
                                chars.colors =
                                    color_indicator.iter().cloned().collect::<im::OrdSet<_>>();
                            } else if let Some(ref mc) = melded_face.mana_cost {
                                chars.colors = crate::rules::casting::colors_from_mana_cost(mc);
                            }
                        }
                    }
                }
            }
        }
    }
    // CR 708.2 / 708.2a: Face-down permanent characteristic override.
    //
    // When a permanent is face-down AND has a face_down_as value (distinguishing
    // morph/manifest/cloak from Foretell/Hideaway's unrelated face_down usage),
    // its characteristics are completely replaced by the face-down defaults BEFORE
    // the merged_components check and BEFORE the layer loop.
    //
    // CR 708.2a: Face-down characteristics: 2/2 colorless creature, no name,
    // no text, no subtypes, no mana cost. These ARE the copiable values (CR 707.2).
    // Continuous effects from the layer loop (e.g., Aura granting +1/+1) apply
    // on TOP of these base values.
    //
    // This must come BEFORE the merged_components block: a face-down merged
    // permanent should present as a 2/2 with no characteristics to opponents.
    if obj.status.face_down && obj.face_down_as.is_some() {
        use crate::state::types::FaceDownKind;
        chars.name = String::new();
        chars.mana_cost = None;
        chars.card_types = OrdSet::unit(CardType::Creature);
        chars.subtypes = OrdSet::new();
        chars.supertypes = OrdSet::new();
        chars.colors = OrdSet::new();
        chars.keywords = OrdSet::new();
        chars.power = Some(2);
        chars.toughness = Some(2);
        chars.triggered_abilities = vec![];
        chars.activated_abilities = vec![];
        chars.mana_abilities = im::Vector::new();
        // CR 702.168a / 701.58a: Disguise and Cloak grant ward {2} while face-down.
        if matches!(
            obj.face_down_as,
            Some(FaceDownKind::Disguise) | Some(FaceDownKind::Cloak)
        ) {
            chars.keywords.insert(KeywordAbility::Ward(2));
        }
    }
    // CR 729.2a: Merged permanent — Layer 1 (Copy) integration.
    // If this permanent has non-empty merged_components, the topmost component's
    // characteristics become the base characteristics before applying any continuous effects.
    // This is a "copiable effect" whose timestamp is the time the objects merged.
    // Applied BEFORE the layer loop so that all 7 layers apply on top of it.
    if obj.zone == ZoneId::Battlefield && !obj.merged_components.is_empty() {
        chars = obj.merged_components[0].characteristics.clone();
    }
    for &layer in &layers_in_order {
        // CR 702.73a + CR 613.3: Changeling is a characteristic-defining ability that adds
        // all creature subtypes in Layer 4 (TypeChange), before any non-CDA Layer 4 effects.
        // CDAs apply first within each layer (CR 613.3), so this runs before gathering
        // layer_effects. A subsequent SetTypeLine effect (e.g., Blood Moon) will correctly
        // override the Changeling subtypes because it runs after the CDA within Layer 4.
        if layer == EffectLayer::TypeChange && chars.keywords.contains(&KeywordAbility::Changeling)
        {
            for s in crate::state::types::ALL_CREATURE_TYPES.iter() {
                chars.subtypes.insert(s.clone());
            }
        }
        // CR 702.114a + CR 613.3: Devoid is a characteristic-defining ability that makes
        // the object colorless in Layer 5 (ColorChange), before any non-CDA Layer 5 effects.
        // CDAs apply first within each layer (CR 613.3), so this runs before gathering
        // layer_effects. A subsequent SetColors/AddColors effect (e.g., Painter's Servant)
        // will correctly override the Devoid colorlessness because it runs after the CDA
        // within Layer 5.
        // CR 604.3: Functions in all zones, not just the battlefield.
        if layer == EffectLayer::ColorChange && chars.keywords.contains(&KeywordAbility::Devoid) {
            chars.colors = OrdSet::new();
        }
        // CR 702.176a: Impending -- "As long as this permanent's impending cost was paid
        // and it has a time counter on it, it's not a creature."
        // Applied at Layer 4 (TypeChange) inline, after CDAs, before non-CDA Layer 4 effects.
        // This is a static ability of the permanent (not a CDA), but it functions only on
        // the battlefield and is conditional on both impending cost paid AND time counters
        // present. Uses `cast_alt_cost` (a game-state marker, not an ability) so it persists
        // even if the Impending keyword is removed by Layer 6 effects (e.g., Humility).
        if layer == EffectLayer::TypeChange {
            if let Some(obj_ref) = state.objects.get(&object_id) {
                if obj_ref.zone == ZoneId::Battlefield
                    && obj_ref.cast_alt_cost == Some(crate::state::types::AltCostKind::Impending)
                    && obj_ref
                        .counters
                        .get(&CounterType::Time)
                        .copied()
                        .unwrap_or(0)
                        > 0
                {
                    chars.card_types.remove(&CardType::Creature);
                    // CR 702.176a: "it's not a creature" -- removes the Creature card type.
                    // Creature subtypes are NOT removed (they're simply non-functional while
                    // the permanent isn't a creature; they return when counters are gone).
                }
            }
        }
        // CR 702.151b: Reconfigure -- "While attached, the Equipment stops being a creature
        // (and loses creature subtypes)."
        // Applied at Layer 4 (TypeChange) using the is_reconfigured flag -- NOT the keyword.
        // Ruling 2022-02-18: the "not a creature" effect persists even if the Reconfigure
        // keyword is removed by Humility/Dress Down while the Equipment is attached.
        // The flag is cleared only when the Equipment becomes unattached.
        if layer == EffectLayer::TypeChange {
            if let Some(obj_ref) = state.objects.get(&object_id) {
                if obj_ref.zone == ZoneId::Battlefield
                    && obj_ref.designations.contains(Designations::RECONFIGURED)
                {
                    chars.card_types.remove(&CardType::Creature);
                    // CR 702.151b + ruling 2022-02-18: "It also loses any creature subtypes
                    // it had." Retain non-creature subtypes (Equipment, Fortification, etc.).
                    // im::OrdSet has no retain; rebuild from filtered iterator.
                    chars.subtypes = chars
                        .subtypes
                        .iter()
                        .filter(|st| !crate::state::types::ALL_CREATURE_TYPES.contains(*st))
                        .cloned()
                        .collect();
                }
            }
        }
        // CR 702.161a: Living Metal -- "During your turn, this permanent is an
        // artifact creature in addition to its other types."
        // Applied at Layer 4 (TypeChange) inline, after CDAs, before non-CDA Layer 4
        // effects. The condition is: (1) object is on the battlefield, AND (2) the
        // active player is the permanent's controller.
        // Uses chars.keywords (pre-Layer-6) so the check runs at Layer 4 time before
        // Humility could strip the keyword in Layer 6. This is intentionally correct:
        // Layer 4 runs before Layer 6, so Living Metal adds Creature before Humility
        // removes abilities. Same behavior as Changeling CDA surviving Humility.
        if layer == EffectLayer::TypeChange && chars.keywords.contains(&KeywordAbility::LivingMetal)
        {
            if let Some(obj_ref) = state.objects.get(&object_id) {
                if obj_ref.zone == ZoneId::Battlefield
                    && state.turn.active_player == obj_ref.controller
                {
                    chars.card_types.insert(CardType::Creature);
                }
            }
        }
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
            apply_layer_modification(state, &mut chars, &effect.modification, mana_value);
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
    // CR 702.140e / CR 729.3: Merged permanent — Layer 6 (Ability) integration.
    // ALL components of a merged permanent contribute their abilities. The topmost
    // component's abilities were already included in the base characteristics (via the
    // Layer 1 merge above). Here we add abilities from non-topmost components (indices 1..N).
    //
    // This runs AFTER the layer loop so that Layer 6 ability-removal effects (Humility,
    // Dress Down) can remove abilities that were granted by the layer loop first, before
    // we add the merge-contributed abilities. This is correct per CR 702.140e which says
    // the merged permanent "has all abilities of all objects that are represented by it" —
    // these are characteristic-defining aspects of the merge, not separate continuous effects.
    // They are applied in Layer 6 at the merge timestamp (the permanent's existing timestamp).
    if obj.zone == ZoneId::Battlefield && obj.merged_components.len() > 1 {
        // Re-borrow to get the current merged_components (obj may have changed during layer loop).
        if let Some(obj_ref) = state.objects.get(&object_id) {
            // Collect abilities from non-topmost components (indices 1..N).
            // Index 0 = topmost, already in base chars from Layer 1.
            let components_slice: Vec<_> = obj_ref.merged_components.iter().skip(1).collect();
            for component in components_slice {
                // Add keyword abilities from this component.
                for kw in component.characteristics.keywords.iter() {
                    chars.keywords.insert(kw.clone());
                }
                // Add triggered abilities from this component.
                for triggered in component.characteristics.triggered_abilities.iter() {
                    chars.triggered_abilities.push(triggered.clone());
                }
                // Add activated abilities from this component.
                for activated in component.characteristics.activated_abilities.iter() {
                    chars.activated_abilities.push(activated.clone());
                }
                // Note: mana_abilities are part of activated_abilities already; no separate field.
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
    let duration_active = match effect.duration {
        EffectDuration::WhileSourceOnBattlefield => match effect.source {
            Some(source_id) => state
                .objects
                .get(&source_id)
                // CR 702.26e: A phased-out permanent's static effects don't apply.
                .map(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in())
                .unwrap_or(false),
            // No source means the effect is inherently active (e.g., from a spell).
            None => true,
        },
        // Active until explicitly removed during cleanup (CR 514.2).
        EffectDuration::UntilEndOfTurn => true,
        EffectDuration::Indefinite => true,
        // CR 702.95a: Active as long as both creatures are on the battlefield,
        // phased in, and still have their paired_with pointing at each other.
        EffectDuration::WhilePaired(a, b) => {
            let a_ok = state
                .objects
                .get(&a)
                .map(|o| {
                    o.zone == ZoneId::Battlefield && o.is_phased_in() && o.paired_with == Some(b)
                })
                .unwrap_or(false);
            let b_ok = state
                .objects
                .get(&b)
                .map(|o| {
                    o.zone == ZoneId::Battlefield && o.is_phased_in() && o.paired_with == Some(a)
                })
                .unwrap_or(false);
            a_ok && b_ok
        }
    };
    if !duration_active {
        return false;
    }
    // CR 604.2: Conditional static abilities — check the condition if present.
    // Conditions are evaluated against the current game state at layer-application time.
    if let Some(ref condition) = effect.condition {
        if let Some(source_id) = effect.source {
            let controller = state
                .objects
                .get(&source_id)
                .map(|obj| obj.controller)
                .unwrap_or_else(|| crate::state::player::PlayerId(0));
            if !crate::effects::check_static_condition(state, condition, source_id, controller) {
                return false;
            }
        } else {
            // A conditional effect without a source object has no controller to evaluate
            // the condition against — treat it as inactive.
            return false;
        }
    }
    true
}
/// Returns true if a continuous effect applies to the given object.
///
/// The filter is evaluated against `chars`, which reflects all modifications applied
/// by earlier layers in the current `calculate_characteristics` call. This correctly
/// handles cases like Opalescence making enchantments into creatures (layer 4) before
/// Humility's "AllCreatures" filter is evaluated (layers 6 and 7).
///
/// CR 702.26e: Phased-out permanents are NOT included in the set of objects affected
/// by continuous effects (except for effects that specifically reference phased-out
/// permanents). This is enforced here for all battlefield-scope filters.
/// Returns true if a continuous effect applies to the given object.
///
/// Public within the crate for use in `replacement.rs` (IG-1 Layer 6 check).
pub(crate) fn effect_applies_to_object(
    state: &GameState,
    effect: &ContinuousEffect,
    object_id: ObjectId,
    obj_zone: ZoneId,
    chars: &Characteristics,
) -> bool {
    effect_applies_to(state, effect, object_id, obj_zone, chars)
}
fn effect_applies_to(
    state: &GameState,
    effect: &ContinuousEffect,
    object_id: ObjectId,
    obj_zone: ZoneId,
    chars: &Characteristics,
) -> bool {
    // CR 702.26e: Phased-out permanents are excluded from continuous effect sets.
    // Check phased_out status for all battlefield-scope effects (except SingleObject,
    // which is allowed to specifically reference a phased-out permanent if needed).
    if obj_zone == ZoneId::Battlefield {
        if let Some(obj) = state.objects.get(&object_id) {
            if obj.status.phased_out {
                // SingleObject may target a phased-out permanent explicitly.
                if !matches!(&effect.filter, EffectFilter::SingleObject(_)) {
                    return false;
                }
            }
        }
    }
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
        // MR-M5-05: CR 110.4 defines permanents as anything on the battlefield.
        // The old 6-type check incorrectly missed objects whose card type was
        // set by a layer effect (e.g., an enchantment made into a Battle) and
        // would also fail for future card types. Zone membership is the correct test.
        EffectFilter::AllPermanents => obj_zone == ZoneId::Battlefield,
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
        // DeclaredTarget should be resolved to SingleObject before being stored in state.
        // If it somehow reaches here unresolved, treat it as non-matching.
        EffectFilter::DeclaredTarget { .. } => false,
        // Source should be resolved to SingleObject(ctx.source) at ApplyContinuousEffect
        // execution time. If it somehow reaches here unresolved, treat it as non-matching.
        EffectFilter::Source => false,
        // CR 301.5 / CR 702.6a: Equipment static ability applies only to the equipped
        // creature. The source object's `attached_to` field identifies that creature.
        // If the equipment is not attached to anything, the filter matches nothing.
        EffectFilter::AttachedCreature => {
            if obj_zone != ZoneId::Battlefield {
                return false;
            }
            // Find the source of this effect and check if it is attached to object_id.
            // `effect.source` must be `Some(source_id)` for AttachedCreature to work
            // (true for WhileSourceOnBattlefield static abilities on Equipment).
            if let Some(source_id) = effect.source {
                state
                    .objects
                    .get(&source_id)
                    .and_then(|src| src.attached_to)
                    .map(|attached| attached == object_id)
                    .unwrap_or(false)
            } else {
                false
            }
        }
        // CR 301.6 / CR 702.67a: Fortification static ability applies only to the
        // fortified land. The source object's `attached_to` field identifies that land.
        // The SBA already ensures Fortifications are only attached to lands.
        // If the fortification is not attached to anything, the filter matches nothing.
        EffectFilter::AttachedLand => {
            if obj_zone != ZoneId::Battlefield {
                return false;
            }
            if let Some(source_id) = effect.source {
                state
                    .objects
                    .get(&source_id)
                    .and_then(|src| src.attached_to)
                    .map(|attached| attached == object_id)
                    .unwrap_or(false)
            } else {
                false
            }
        }
        // CR 604.2: Static ability "Creatures you control have [keyword]."
        // Resolves the source's controller dynamically at layer-application time.
        EffectFilter::CreaturesYouControl => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if let Some(source_id) = effect.source {
                let source_controller = state.objects.get(&source_id).map(|src| src.controller);
                let obj_controller = state.objects.get(&object_id).map(|obj| obj.controller);
                source_controller.is_some() && source_controller == obj_controller
            } else {
                false
            }
        }
        // CR 604.2: Static ability "Other creatures you control have [keyword]."
        // Same as CreaturesYouControl but excludes the source object itself.
        EffectFilter::OtherCreaturesYouControl => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if let Some(source_id) = effect.source {
                if source_id == object_id {
                    return false;
                }
                let source_controller = state.objects.get(&source_id).map(|src| src.controller);
                let obj_controller = state.objects.get(&object_id).map(|obj| obj.controller);
                source_controller.is_some() && source_controller == obj_controller
            } else {
                false
            }
        }
        // CR 604.2: Static ability "Other [Subtype] creatures you control get [bonus]."
        // Filters by subtype and excludes the source object.
        EffectFilter::OtherCreaturesYouControlWithSubtype(subtype) => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if !chars.subtypes.contains(subtype) {
                return false;
            }
            if let Some(source_id) = effect.source {
                if source_id == object_id {
                    return false;
                }
                let source_controller = state.objects.get(&source_id).map(|src| src.controller);
                let obj_controller = state.objects.get(&object_id).map(|obj| obj.controller);
                source_controller.is_some() && source_controller == obj_controller
            } else {
                false
            }
        }
    }
}
/// Apply a single layer modification to the given characteristics.
///
/// `state` is needed for Layer 1 copy effects to look up the target object's
/// copiable values (CR 707.2).  `mana_value` is the object's printed mana value,
/// used for `SetPtToManaValue`.
fn apply_layer_modification(
    state: &GameState,
    chars: &mut Characteristics,
    modification: &LayerModification,
    mana_value: u32,
) {
    match modification {
        // Layer 1: Copy effects (CR 707.2).
        // Replace all copiable values of `chars` with those of the target object,
        // including any copy effects already applied to the target (CR 707.3 clone chain).
        LayerModification::CopyOf(target) => {
            if let Some(target_chars) = super::copy::get_copiable_values(state, *target) {
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
        // CR 604.2: "As long as your devotion to [color] is less than N, [this] isn't a creature."
        // Removes the specified card types without affecting other types on the type line.
        // Applied conditionally via ContinuousEffect::condition in is_effect_active.
        LayerModification::RemoveCardTypes(types_to_remove) => {
            for ct in types_to_remove {
                chars.card_types.remove(ct);
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
        // CR 702.73a, CR 205.3m: Adds every creature type (used by Changeling CDA and
        // Maskwood Nexus-style "is every creature type" continuous effects).
        LayerModification::AddAllCreatureTypes => {
            for s in crate::state::types::ALL_CREATURE_TYPES.iter() {
                chars.subtypes.insert(s.clone());
            }
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
            chars.keywords.insert(kw.clone());
        }
        LayerModification::AddKeywords(kws) => {
            for kw in kws {
                chars.keywords.insert(kw.clone());
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
    // MR-M5-06: use VecDeque so pop_front() is O(1) instead of Vec::remove(0) O(n).
    let mut ready: VecDeque<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut result: Vec<&ContinuousEffect> = Vec::with_capacity(n);
    while !ready.is_empty() {
        // Take the first ready node (already in timestamp/index order).
        let i = ready.pop_front().unwrap();
        result.push(effects[i]);
        for &j in &adj[i] {
            in_degree[j] -= 1;
            if in_degree[j] == 0 {
                // Insert maintaining sorted order (by index = by timestamp).
                let pos = ready.make_contiguous().partition_point(|&k| k < j);
                ready.insert(pos, j);
            }
        }
    }
    // Cycle handling (CR 613.8b): any remaining effects form a dependency loop.
    // Apply them in timestamp order (index order = timestamp order since effects is sorted).
    if result.len() < n {
        // Find effects not yet emitted (O(n²), but n is tiny in practice — ≤ 20 effects).
        // MR-M5-03: use EffectId comparison instead of ptr::eq — ptr::eq is fragile across
        // clones and stack allocations; EffectId is the correct logical identity for effects.
        for effect in &effects {
            let was_emitted = result.iter().any(|e| e.id == effect.id);
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
/// Remove all "until end of turn" continuous effects and replacement effects
/// during the Cleanup step (CR 514.2).
///
/// Called by `turn_actions::cleanup_actions` immediately after clearing damage.
/// Also removes corresponding `prevention_counters` entries so that depleted
/// `PreventDamage` shields don't persist across turns.
pub fn expire_end_of_turn_effects(state: &mut GameState) {
    use crate::state::replacement_effect::ReplacementId;
    // Expire UntilEndOfTurn continuous effects (CR 514.2).
    let keep: im::Vector<ContinuousEffect> = state
        .continuous_effects
        .iter()
        .filter(|e| e.duration != EffectDuration::UntilEndOfTurn)
        .cloned()
        .collect();
    state.continuous_effects = keep;
    // Expire UntilEndOfTurn replacement effects (CR 514.2).
    // Collect IDs to remove first so we can also clean up prevention_counters.
    let expired_ids: Vec<ReplacementId> = state
        .replacement_effects
        .iter()
        .filter(|e| e.duration == EffectDuration::UntilEndOfTurn)
        .map(|e| e.id)
        .collect();
    if !expired_ids.is_empty() {
        let keep_replacements: im::Vector<_> = state
            .replacement_effects
            .iter()
            .filter(|e| e.duration != EffectDuration::UntilEndOfTurn)
            .cloned()
            .collect();
        state.replacement_effects = keep_replacements;
        // Also remove any prevention shield counters for the expired effects.
        for id in &expired_ids {
            state.prevention_counters.remove(id);
        }
    }
}
