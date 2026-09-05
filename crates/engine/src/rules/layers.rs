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
use crate::cards::card_definition::{EffectAmount, EffectTarget, PlayerTarget, ZoneTarget};
use crate::state::{
    continuous_effect::{
        ContinuousEffect, EffectDuration, EffectFilter, EffectLayer, LayerModification,
    },
    game_object::{Characteristics, Designations, GameObject, ManaAbility, ObjectId},
    player::PlayerId,
    types::{CardType, CounterType, KeywordAbility, ManaColor, SubType, SuperType},
    zone::ZoneId,
    GameState,
};
use imbl::OrdSet;
use std::collections::VecDeque;
// ── Layer-walk re-entrancy guard (OOS-SIM2-6 / PB-DX19) ─────────────────────────

thread_local! {
    /// Depth of the currently-executing [`calculate_characteristics`] walk on this
    /// thread. Zero means "not inside the layer system".
    ///
    /// This is a scratch flag describing the *call stack*, not game state. It is not
    /// serialized, not hashed, never read by a rule, and cannot differ between two
    /// runs of the same command — so it does not weaken Architecture Invariant 2
    /// (immutable `GameState`) or Invariant 3 (all mutation through a `Command`).
    /// `GameState` remains untouched; what this records is where in the engine's own
    /// execution we are, which no `&GameState` parameter can express.
    static LAYER_WALK_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// RAII marker for the dynamic extent of a [`calculate_characteristics`] call.
///
/// Decrements on `Drop`, so the depth is restored on an early `return` (this
/// function has several) and on unwind.
pub(crate) struct LayerWalkGuard;

impl LayerWalkGuard {
    pub(crate) fn enter() -> Self {
        LAYER_WALK_DEPTH.with(|d| d.set(d.get().saturating_add(1)));
        LayerWalkGuard
    }
}

impl Drop for LayerWalkGuard {
    fn drop(&mut self) {
        LAYER_WALK_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    }
}

/// Is the current thread inside a [`calculate_characteristics`] walk?
pub fn in_layer_walk() -> bool {
    LAYER_WALK_DEPTH.with(|d| d.get()) > 0
}

/// The characteristics a **condition's filter test** may read for `obj`
/// (CR 604.2 / CR 613.1d) — the single decision point for OOS-SIM2-6.
///
/// # Why this exists
///
/// `check_static_condition` and `check_condition` are **shared evaluators**. They
/// are reached from five different places, and only one of them is dangerous:
///
/// | caller | dangerous? |
/// |---|---|
/// | `is_effect_active`, inside `calculate_characteristics` | **YES** — closes a cycle |
/// | `activation_condition` (`rules/abilities.rs`, `rules/mana.rs`) | no |
/// | `intervening_if` (`rules/abilities.rs`) | no |
/// | `Effect::Conditional` | no |
/// | `unless_condition` (ETB replacement) | no |
///
/// On the first, resolving another object's characteristics calls back into
/// `calculate_characteristics`, which re-enters `is_effect_active` for **every**
/// registered effect — whatever object it was asked about, and whatever zone that
/// object is in. The recursion runs through the *effect*, not the object, so it is
/// unconditional and it overflows the stack (SIGABRT; `indomitable_archangel` made
/// it reachable from a legal deck).
///
/// On the other four there is no cycle, and CR 613.1d demands the **layer-resolved**
/// answer: a 2/2 with two `+1/+1` counters really does have power 4 for
/// `garruks_uprising`'s intervening-if, and a changeling really is a Vampire for
/// `bloodline_keeper`'s activation cost.
///
/// **PB-DX19's first attempt read base characteristics unconditionally and so broke
/// all four of the safe paths to fix the one unsafe one.** That regression is the
/// reason this function exists rather than a bare `obj.characteristics`.
///
/// # The deviation, and its exact scope
///
/// Inside the layer walk this returns **printed** characteristics, which is wrong by
/// CR 613.1d whenever another continuous effect has changed the object's types,
/// subtypes or P/T. The instance that is **pinned by a test** is
/// `blinkmoth_nexus` / `inkmoth_nexus` animating into artifacts, so they do not feed
/// Metalcraft — see `deviation_animated_nexus_does_not_count_toward_metalcraft` in
/// `tests/primitives/pb_dx19_characteristics_recursion.rs`. Others are known and
/// **not** pinned: CR 712.8d/e (DFC), 712.8g (meld), 729.2a (merge), 702.73a
/// (changeling), and — in the opposite direction — CR 708.2a face-down permanents,
/// where the printed types are still the *hidden* card's, so an in-walk count can be
/// too HIGH rather than too low. All are catalogued on `OOS-DX19-2`, whose CR 613.8b
/// dependency-aware fixpoint is the honest repair and a batch of its own.
///
/// The deviation applies **only** inside the layer walk. Everywhere else this is
/// exactly `expect_characteristics`.
pub fn characteristics_for_condition(state: &GameState, obj: &GameObject) -> Characteristics {
    if in_layer_walk() {
        obj.characteristics.clone()
    } else {
        expect_characteristics(state, obj.id)
    }
}

/// Calculate the effective characteristics of an object after applying all active
/// continuous effects through the layer system (CR 613).
///
/// Starts with the object's base (printed) characteristics and applies all active
/// continuous effects in layer order (1 → 7d), with timestamp and dependency ordering
/// within each layer.
///
/// Returns `None` if — and only if — the object does not exist in the game state.
/// That is the function's sole failure mode: every other step is total. A caller
/// holding an id it knows to be live should therefore use
/// [`expect_characteristics`] rather than papering over the `None`.
///
/// # Re-entrancy (OOS-SIM2-6 / PB-DX19)
///
/// This function is **re-entrant by design**: evaluating a conditional continuous
/// effect's `condition` (CR 604.2) can require inspecting other permanents. Left
/// unguarded that is an unbounded recursion, because the cycle runs through the
/// *effect*, not the object — see [`characteristics_for_condition`]. The whole
/// dynamic extent of this call is therefore marked by a [`LayerWalkGuard`], and
/// condition evaluation consults [`in_layer_walk`] to decide which characteristics
/// it is allowed to read.
pub fn calculate_characteristics(
    state: &GameState,
    object_id: ObjectId,
) -> Option<Characteristics> {
    // OOS-SIM2-6 / PB-DX19: mark the layer walk for its whole dynamic extent. See
    // [`characteristics_for_condition`] — a condition evaluated from *inside* here
    // must not resolve another object's characteristics, and a condition evaluated
    // from anywhere else must.
    let _layer_walk = LayerWalkGuard::enter();
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
                        chars.colors = color_indicator.iter().cloned().collect::<imbl::OrdSet<_>>();
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
                                    color_indicator.iter().cloned().collect::<imbl::OrdSet<_>>();
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
        chars.mana_abilities = imbl::Vector::new();
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
            if let Some(obj_ref) = state.expect_object(object_id) {
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
            if let Some(obj_ref) = state.expect_object(object_id) {
                if obj_ref.zone == ZoneId::Battlefield
                    && obj_ref.designations.contains(Designations::RECONFIGURED)
                {
                    chars.card_types.remove(&CardType::Creature);
                    // CR 702.151b + ruling 2022-02-18: "It also loses any creature subtypes
                    // it had." Retain non-creature subtypes (Equipment, Fortification, etc.).
                    // imbl::OrdSet has no retain; rebuild from filtered iterator.
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
            if let Some(obj_ref) = state.expect_object(object_id) {
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
            apply_layer_modification(
                state,
                &mut chars,
                &effect.modification,
                mana_value,
                object_id,
            );
        }
        // CR 305.6 / CR 613.1d / CR 613.1f (PB-DX43): derive each basic land
        // type's intrinsic "{T}: Add [symbol]" mana ability now that every
        // Layer-4 (TypeChange) effect for THIS iteration has been applied.
        //
        // Why here and not inside one of the `apply_layer_modification` arms
        // above (e.g. `SetLandTypes`/`AddSubtypes`): the subtype set must be
        // FULLY resolved before the derivation reads it. Urborg's
        // `AddSubtypes(Swamp)` and Blood Moon's `SetLandTypes(Mountain)` are
        // ordered against each other by the CR 613.8 `depends_on` arm below
        // (`SetLandTypes` depends on a co-resident basic-type `AddSubtypes`),
        // NOT by raw timestamp order. Deriving from `chars.subtypes` inside a
        // single arm would read an INTERMEDIATE subtype set — whichever of the
        // two effects that arm's own modification happens to run as — and
        // re-open exactly the ordering dependency `depends_on` exists to
        // settle. Running once, after the whole ordered Layer-4 list has been
        // applied, guarantees the derivation always sees the final, dependency-
        // resolved subtype set for this object.
        //
        // Why here and not after the ENTIRE 8-layer walk (i.e. outside this
        // `for &layer in &layers_in_order` loop, past Layer 6/Ability): CR
        // 613.1f puts ability-adding AND ability-removing effects in Layer 6,
        // strictly after Layer 4. A Layer-6 ability-removal effect (Humility,
        // Dress Down, `LayerModification::RemoveAllAbilities`) must be able to
        // strip this intrinsic ability exactly as it strips every other one —
        // that only works if the intrinsic has already been appended to
        // `chars.mana_abilities` by the time Layer 6 runs. Deriving it here, at
        // the close of Layer 4, keeps it subject to Layer 6 removal without any
        // extra bookkeeping; deriving it after the whole walk would make it
        // immune to Humility, which is CR-wrong (CR 613.1f applies after CR
        // 305.6's layer-4 grant, not around it).
        if layer == EffectLayer::TypeChange {
            derive_intrinsic_land_mana_abilities(&mut chars);
        }
        // Layer 7c (PtModify): also apply counter P/T contributions (CR 613.4c).
        // Counters are not modeled as ContinuousEffects — they live on the GameObject.
        // We apply them here (at the correct layer position) regardless of whether there
        // are any static Layer 7c effects.
        if layer == EffectLayer::PtModify {
            // Re-borrow: obj is still valid since we haven't mutated state.
            // SR-25: `calculate_characteristics` takes `&GameState` and holds the line-39
            // `obj` borrow live, so `object_id` cannot have been removed here — a `None` is
            // an engine bug (asserts in debug via `expect_object`, `break`s in release, the
            // same fallback the old MR-M5-01 if-let took).
            let Some(obj_ref) = state.expect_object(object_id) else {
                break;
            };
            // OOS-SIM2-5 / PB-DX19: counters are `u32` and P/T is `i32`, so both the
            // widening and the arithmetic are saturating. `try_into().unwrap_or(i32::MAX)`
            // rather than `as i32`, because an `as` cast does NOT panic under
            // `overflow-checks` — a count above `i32::MAX` would wrap to a NEGATIVE
            // modifier and silently invert the counter's sign in every build. See the
            // ceiling deviation note on `apply_modification`.
            let plus_ones: i32 = obj_ref
                .counters
                .get(&CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0)
                .try_into()
                .unwrap_or(i32::MAX);
            let minus_ones: i32 = obj_ref
                .counters
                .get(&CounterType::MinusOneMinusOne)
                .copied()
                .unwrap_or(0)
                .try_into()
                .unwrap_or(i32::MAX);
            let net = plus_ones.saturating_sub(minus_ones);
            if net != 0 {
                if let Some(p) = &mut chars.power {
                    *p = p.saturating_add(net);
                }
                if let Some(t) = &mut chars.toughness {
                    *t = t.saturating_add(net);
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
        if let Some(obj_ref) = state.expect_object(object_id) {
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
    // PB-EF3b (CR 702.86a/702.91a/702.121a, 613.1f): a trigger-bearing keyword GRANTED by a
    // continuous effect (LayerModification::AddKeyword) inserts into `chars.keywords` but carries
    // no derived TriggeredAbilityDef, so its trigger would be a silent no-op. Synthesize it here,
    // AFTER all layers (incl. Layer 6 add/remove) and merge integration, so the derived trigger
    // exists in the RESOLVED characteristics that collect_triggers_for_event reads.
    //
    // Keyword model is a SET (OrdSet), so printed+granted collapse to one entry (CR 702.x.b "each
    // instance triggers separately" is not representable — known limitation). Dedup by exact
    // description equality against the shared helper's output so a PRINTED derived def (already in
    // base chars via builder.rs) is not duplicated. Humility/RemoveAllAbilities empties
    // chars.keywords, so nothing is appended (correct). These are SelfAttacks triggers only — no
    // ETB / Panharmonicon interaction.
    let kws: Vec<KeywordAbility> = chars.keywords.iter().cloned().collect();
    for kw in kws {
        if let Some(def) = crate::state::builder::derived_attack_trigger_for_keyword(&kw) {
            let already = chars
                .triggered_abilities
                .iter()
                .any(|t| t.description == def.description);
            if !already {
                chars.triggered_abilities.push(def);
            }
        }
    }
    Some(chars)
}

/// [`calculate_characteristics`] for a caller that has already established the
/// object is live — for example one iterating `state.objects()` directly.
///
/// The only way [`calculate_characteristics`] returns `None` is a missing
/// `ObjectId`, so at such a site `None` is an engine-invariant violation, not a
/// rules event. Fires a `debug_assert!` naming the id, and in release builds
/// falls back to the object's printed characteristics if it can find them, or
/// `Characteristics::default()` if it cannot.
///
/// Do **not** use this for an id that may be last known information (CR 400.7) —
/// a target captured earlier, a sacrificed source, a creature that died to a
/// state-based action mid-resolution. Call [`calculate_characteristics`] and
/// handle the `None` as the fizzle CR 608.2b requires.
#[track_caller]
pub fn expect_characteristics(state: &GameState, object_id: ObjectId) -> Characteristics {
    if let Some(chars) = calculate_characteristics(state, object_id) {
        return chars;
    }
    debug_assert!(
        false,
        "engine invariant: calculate_characteristics({object_id:?}) returned None at a site \
         that requires the object to be live. Its only failure mode is an absent ObjectId \
         (CR 400.7). If the id may be last known information, handle the None as a CR 608.2b \
         fizzle instead of calling expect_characteristics."
    );
    state
        .objects()
        .get(&object_id)
        .map(|o| o.characteristics.clone())
        .unwrap_or_default()
}

/// Returns true if a continuous effect is currently active.
///
/// An effect is active when its duration condition is met:
/// - `WhileSourceOnBattlefield`: source object exists and is on the battlefield
/// - `UntilEndOfTurn`: always active (removed explicitly by `expire_end_of_turn_effects`)
/// - `Indefinite`: always active
///
/// Deliberately does NOT consult `effect.affected_set` (CR 611.2c/PB-DX5): this
/// function answers "is this effect running at all?" (duration, CR 611.2a/b, and
/// `condition`, CR 604.2) and takes no `object_id`, so a per-object locked set is
/// not expressible here. An effect whose locked set happens to be empty is still
/// active (CR 611.2b's "does nothing" describes an outcome, not non-existence) --
/// see `effect_applies_to` for the per-object question `affected_set` answers.
pub fn is_effect_active(state: &GameState, effect: &ContinuousEffect) -> bool {
    let duration_active = match effect.duration {
        // PB-DX39: this read is LIVE-ONLY ON PURPOSE and must never be routed through
        // `source_view_at_resolution` / `lki_object_snapshot`. The question it answers is
        // "is the source still on the battlefield" (CR 611.3b: "the effect applies at all
        // times that the permanent generating it is on the battlefield" -- NOT CR 611.2b,
        // which is the "for as long as" duration of a RESOLUTION-generated effect), and an
        // LKI
        // fallback would answer "yes, as it last was" forever -- a departed permanent's
        // static ability would run for the rest of the game. Pinned by
        // `core::pb_dx39_source_view_gates::r4`.
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
        // CR 611.2b: Active until the specified player's next turn begins.
        // Removal is handled by expire_until_next_turn_effects at the untap step.
        EffectDuration::UntilYourNextTurn(_) => true,
        // CR 611.2b/c: activity/removal is handled imperatively by
        // expire_while_you_control_source_effects (one-shot, never resumes). Mirror
        // UntilYourNextTurn: always "active" here; the expiry pass owns termination.
        // Do NOT put a live control check here -- that would let the effect resume,
        // violating CR 611.2c. (`SetController` is a layer no-op anyway, so this arm
        // never affects characteristics.)
        EffectDuration::WhileYouControlSource(_) => true,
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
            // PB-DX39: LIVE-ONLY ON PURPOSE, like the duration read above. This supplies
            // the controller for a CR 604.2 *static* ability's condition, whose source is
            // on the battlefield by construction (CR 604.2: "static abilities ... function
            // ... while the permanent is on the battlefield"). CR 608.2h's last-known-
            // information rule is scoped to an ability that "exists on the stack
            // independently of its source" (CR 113.7a), which a static ability never does,
            // so an LKI fallback here would be CR-wrong as well as unnecessary. Pinned by
            // `core::pb_dx39_source_view_gates::r4`.
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
/// Delegates to [`effect_applies_to`], which carries the full contract (CR
/// 611.2c / CR 611.3a two-mode dispatch, the CR 702.26e phased-out guard, and
/// the per-filter predicates) -- see its doc comment.
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
/// Everything an `EffectFilter` arm of [`effect_applies_to_inner`] needs to know about
/// the continuous effect's **source**, answered once instead of once per arm.
///
/// PB-DX39 (`OOS-DX5-3`, `OOS-DX5-7`): before this existed, twenty of the thirty-seven
/// filter arms each re-read `state.objects.get(&source_id)` for themselves, so twenty
/// places had to be right about CR 608.2h and none of them was. There is now exactly one
/// read, in the two constructors below, and the arms consume the answer.
///
/// # The moment this represents
///
/// *The SET is determined at RESOLUTION (CR 611.2c: "the set of objects it affects is
/// determined when that continuous effect begins. After that point, the set won't
/// change"), from the source AS IT MOST RECENTLY EXISTED (CR 608.2h).* Those are two
/// different moments and conflating them is the whole defect. This type answers the
/// second; [`snapshot_affected_set`] owns the first and this batch does not move it.
///
/// # Why a LIVE source always beats a snapshot
///
/// Umezawa's Jitte, ruling 2005-02-01 **#3**: *"If the Jitte is moved after the '+2/+2'
/// mode is announced but before it resolves, the bonus is given to the creature that is
/// equipped when the ability resolves."* So a view captured at ACTIVATION time would be
/// CR-wrong, not merely more expensive: it would name the creature equipped when the
/// ability went on the stack. Both constructors read the live object first for exactly
/// that reason; only when the id is retired (CR 400.7) does the resolution constructor
/// fall back.
///
/// **#3 alone reads as an argument AGAINST the fallback, and #5 is what settles it** — it
/// is quoted here rather than paraphrased, because a reader who has only #3 in front of
/// them will conclude that a departed Jitte gives its bonus to nobody. Ruling 2005-02-01
/// **#5**, verbatim: *"If the Jitte leaves the battlefield after the '+2/+2' mode is
/// announced but before it resolves, the bonus is given to the creature that was most
/// recently equipped once the ability resolves."* #3 governs a source that is still there
/// (live read); #5 governs a source that is not (LKI). They are the two halves of
/// CR 608.2h's own ordering, which is why one constructor implements both in that order.
///
/// Fields are borrowed, never cloned: `chosen_creature_type` wraps a `String` and
/// [`effect_applies_to_inner`] is on the layer walk (`calculate_characteristics`), so an
/// owned view would allocate per arm per effect per object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SourceView<'a> {
    /// CR 109.4 / CR 604.2: the source's controller, for the "you control" filters.
    pub controller: PlayerId,
    /// CR 301.5 / CR 702.6a / CR 702.67a: what the source Equipment / Fortification /
    /// Aura is attached to.
    pub attached_to: Option<ObjectId>,
    /// CR 205.3m: the creature type chosen for the source (Morophon, Patchwork Banner).
    pub chosen_creature_type: Option<&'a SubType>,
    /// CR 105.1 / CR 614.12: the color chosen for the source.
    pub chosen_color: Option<crate::state::types::Color>,
}
impl<'a> SourceView<'a> {
    fn of(obj: &'a GameObject) -> Self {
        SourceView {
            controller: obj.controller,
            attached_to: obj.attached_to,
            chosen_creature_type: obj.chosen_creature_type.as_ref(),
            chosen_color: obj.chosen_color,
        }
    }
}
/// CR 611.3a: the source view for an effect generated by a **static ability** --
/// live read only, and **deliberately no last-known-information fallback**.
///
/// CR 611.3a: such an effect "isn't 'locked in'; it applies at any given moment to
/// whatever its text indicates". The effect exists only while the ability does, and the
/// ability leaves with the object -- so once the source is gone there is nothing to
/// apply, and answering from LKI would make a departed permanent's static ability run
/// forever. CR 608.2h and CR 113.7a are both scoped to an ability that "exists on the
/// stack independently of its source" (CR 113.7a), which a static ability never does.
///
/// `is_effect_active`'s `EffectDuration::WhileSourceOnBattlefield` arm already refuses
/// the common case, but it does not cover every duration a static registration can
/// carry (`rules/replacement.rs` and the `ClassLevelAbility` arm of `rules/resolution.rs`
/// both forward a card def's own duration verbatim, and `Effect::CreateEmblem`'s static
/// loop uses `Indefinite`), so the scoping lives here rather than resting on that arm.
fn source_view_live(state: &GameState, source_id: ObjectId) -> Option<SourceView<'_>> {
    state.objects.get(&source_id).map(SourceView::of)
}
/// CR 608.2h / CR 113.7a: the source view for an effect generated by the **resolution**
/// of a spell or ability -- live first, last known information second.
///
/// CR 608.2h: *"If the effect requires information from a specific object, including the
/// source of the ability itself, the effect uses the current information of that object
/// if it's in the public zone it was expected to be in; if it's no longer in that zone,
/// or if the effect has moved it from a public zone to a hidden zone, the effect uses the
/// object's last known information."* CR 113.7a says the same for the ability itself:
/// *"Once activated or triggered, an ability exists on the stack independently of its
/// source. Destruction or removal of the source after that time won't affect the
/// ability."*
///
/// The order is the rule: **live first, LKI second.** See [`SourceView`] for why.
///
/// The LKI store is only populated for a departing permanent that some pending ability
/// can still read -- see `GameState::capture_lki_snapshot` and
/// `GameState::capture_source_lki_for_pending_ability`.
///
/// This is the **only** LKI-consulting constructor, and `snapshot_affected_set` is its
/// only caller. Pinned by `core::pb_dx39_source_view_gates::r5`.
fn source_view_at_resolution(state: &GameState, source_id: ObjectId) -> Option<SourceView<'_>> {
    state
        .objects
        .get(&source_id)
        .or_else(|| state.lki_object_snapshot(source_id))
        .map(SourceView::of)
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
///
/// CR 611.2c / CR 611.3a: this function serves two contracts, selected by
/// `effect.affected_set`.
/// - `Some(set)` — the effect was generated by the **resolution** of a spell or
///   ability. The set of objects it affects was determined once, when the
///   effect began (`snapshot_affected_set`, called from
///   `Effect::ApplyContinuousEffect`), and never changes afterward. This
///   function answers by **membership in that set alone** and does not
///   re-consult `filter` or `chars` for those effects.
/// - `None` — the effect was generated by a **static ability** (CR 611.3a:
///   "isn't 'locked in'; it applies at any given moment to whatever its text
///   indicates"). `filter` is re-evaluated live against `chars`, below.
fn effect_applies_to(
    state: &GameState,
    effect: &ContinuousEffect,
    object_id: ObjectId,
    obj_zone: ZoneId,
    chars: &Characteristics,
) -> bool {
    // CR 611.3a (PB-DX39): the LIVE path. A static ability's effect is not locked in and
    // its source's information is read live, with NO last-known-information fallback --
    // see `source_view_live` for the CR argument. `snapshot_affected_set` is the one
    // caller that passes a resolution-time view instead.
    //
    // LAZY ON PURPOSE (PB-DX39 `/review`). The first draft resolved the view here
    // unconditionally, above both of `effect_applies_to_inner`'s short-circuits, which
    // added one `OrdMap::get` per (effect, object) on the layer walk for two populations
    // that never consult it: every LOCKED effect (`affected_set.is_some()` returns by
    // membership alone, CR 611.2c -- the common resolution case) and the seventeen
    // non-source-relative filter arms. Pre-PB-DX39 those paths did no source lookup at
    // all, so publishing the batch as "twenty reads became one" while adding an
    // unconditional one here would have been half the story.
    //
    // `filter_is_source_relative` is exhaustive with no `_` arm, so a new `EffectFilter`
    // variant is a compile error until it is classified, and
    // `core::pb_dx39_source_view_gates::r2c` asserts its `true` set is exactly the set of
    // arms that consume `source` -- a mis-classification here would silently hand an arm
    // `None`, which is why it is pinned rather than trusted.
    let needs_source = effect.affected_set.is_none() && filter_is_source_relative(&effect.filter);
    let source = if needs_source {
        effect.source.and_then(|sid| source_view_live(state, sid))
    } else {
        None
    };
    effect_applies_to_inner(state, effect, object_id, obj_zone, chars, source.as_ref())
}
/// Does this `EffectFilter`'s arm in [`effect_applies_to_inner`] consume the caller's
/// [`SourceView`]?
///
/// Exhaustive, **no `_` arm**, mirroring the SR-5 keyword-catchall discipline and
/// [`candidate_ids_for_filter`] directly above: a new variant cannot join silently on the
/// `false` side, which is the side that would make a source-relative arm receive `None` and
/// quietly match nothing.
fn filter_is_source_relative(filter: &EffectFilter) -> bool {
    match filter {
        // CR 301.5 / CR 301.6 / CR 303.4: reads the source's `attached_to`.
        EffectFilter::AttachedCreature
        | EffectFilter::AttachedLand
        | EffectFilter::AttachedPermanent
        // CR 604.2 / CR 109.4: reads the source's `controller`.
        | EffectFilter::CreaturesYouControl
        | EffectFilter::OtherCreaturesYouControl
        | EffectFilter::OtherCreaturesYouControlWithSubtype(_)
        | EffectFilter::CreaturesOpponentsControl
        | EffectFilter::CreaturesYouControlWithSubtype(_)
        | EffectFilter::AttackingCreaturesYouControl
        | EffectFilter::ArtifactsYouControl
        | EffectFilter::CreaturesYouControlWithSupertype(_)
        | EffectFilter::CreaturesYouControlWithColor(_)
        | EffectFilter::OtherCreaturesYouControlExcludingSubtype(_)
        | EffectFilter::CreaturesYouControlExcludingSubtype(_)
        | EffectFilter::AttackingCreaturesYouControlWithSubtype(_)
        | EffectFilter::OtherCreaturesYouControlWithSubtypes(_)
        | EffectFilter::LandsYouControl
        // CR 205.3m / CR 105.1: reads `chosen_creature_type` / `chosen_color` as well.
        | EffectFilter::CreaturesYouControlOfChosenType
        | EffectFilter::CreaturesYouControlOfChosenColor
        | EffectFilter::OtherCreaturesYouControlOfChosenType => true,
        EffectFilter::SingleObject(_)
        | EffectFilter::AllCreatures
        | EffectFilter::AllLands
        | EffectFilter::AllNonbasicLands
        | EffectFilter::AllEnchantments
        | EffectFilter::AllNonAuraEnchantments
        | EffectFilter::AllPermanents
        | EffectFilter::AllCardsInGraveyards
        | EffectFilter::ControlledBy(_)
        | EffectFilter::CreaturesControlledBy(_)
        | EffectFilter::DeclaredTarget { .. }
        | EffectFilter::Source
        | EffectFilter::TriggeringCreature
        | EffectFilter::CreaturesControlledByDefendingPlayer
        | EffectFilter::AllCreaturesWithSubtype(_)
        | EffectFilter::AllCreaturesExcludingSubtype(_)
        | EffectFilter::AllCreaturesExcludingChosenSubtype => false,
    }
}
/// The body of [`effect_applies_to`], with the source's information supplied by the
/// caller rather than re-read per arm.
///
/// `source` is `None` when the effect has no source at all, or when the source could not
/// be resolved on the path the caller chose (live-only for CR 611.3a static abilities;
/// live-then-LKI for CR 608.2h resolution effects). Every source-relative arm answers
/// `false` in that case, which is the pre-PB-DX39 behaviour for a missing source.
fn effect_applies_to_inner(
    state: &GameState,
    effect: &ContinuousEffect,
    object_id: ObjectId,
    obj_zone: ZoneId,
    chars: &Characteristics,
    source: Option<&SourceView<'_>>,
) -> bool {
    // CR 702.26e: Phased-out permanents are excluded from continuous effect sets.
    // Check phased_out status for all battlefield-scope effects (except SingleObject,
    // which is allowed to specifically reference a phased-out permanent if needed).
    //
    // This guard stays ABOVE the CR 611.2c membership check below, and applies to
    // both the locked and the live path: CR 702.26b says a phased-out permanent
    // can't be affected by anything while it's out, and CR 702.26e/f say a
    // continuous effect that references it won't include it in the affected set
    // while phased out but resumes applying once it phases back in. Keeping this
    // guard above the `affected_set` return produces exactly that for locked
    // effects, and it is also what makes `snapshot_affected_set`'s determination-
    // time exclusion (CR 702.26e) fall out for free -- it calls this same
    // function via `effect_applies_to`.
    //
    // The `SingleObject` exemption below reads `effect.filter`, not
    // `effect.affected_set` -- deliberately, to keep the 79 SingleObject-derived
    // resolution effects (DeclaredTarget/Source/TriggeringCreature at execution
    // time) byte-identical to their pre-CR-611.2c-fix behavior. This is a
    // pre-existing deviation from CR 702.26e's second sentence (which does not
    // carve out SingleObject); PB-DX5 neither fixes nor worsens it -- OOS-DX5-4.
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
    // CR 611.2c: a continuous effect generated by the resolution of a spell or
    // ability affects a set of objects determined when the effect began; after
    // that point the set does not change. `Some` therefore answers this
    // question by membership alone -- deliberately ignoring `chars`, `obj_zone`
    // and the live filter, all of which are exactly the things CR 611.2c says
    // must not be re-consulted. `None` means a static ability (CR 611.3a),
    // which is NOT locked in and falls through to the live filter below.
    if let Some(ref affected) = effect.affected_set {
        return affected.contains(&object_id);
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
        // TriggeringCreature should be resolved to SingleObject(ctx.triggering_creature_id)
        // at ApplyContinuousEffect execution time. If it somehow reaches here unresolved,
        // treat it as non-matching (same pattern as Source/DeclaredTarget).
        EffectFilter::TriggeringCreature => false,
        // CreaturesControlledByDefendingPlayer is a DSL placeholder resolved to
        // CreaturesControlledBy(pid) at ApplyContinuousEffect execution. If it reaches
        // here unresolved, treat as non-matching (same pattern as Source/TriggeringCreature).
        EffectFilter::CreaturesControlledByDefendingPlayer => false,
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
            // CR 608.2h / CR 113.7a: `source` carries the attachment as the source last
            // had it -- the Umezawa's Jitte ruling's "most recently equipped" creature
            // (OOS-DX5-3) when the resolution path supplied it.
            let Some(src) = source else {
                return false;
            };
            src.attached_to == Some(object_id)
        }
        // CR 301.6 / CR 702.67a: Fortification static ability applies only to the
        // fortified land. The source object's `attached_to` field identifies that land.
        // The SBA already ensures Fortifications are only attached to lands.
        // If the fortification is not attached to anything, the filter matches nothing.
        EffectFilter::AttachedLand => {
            if obj_zone != ZoneId::Battlefield {
                return false;
            }
            // CR 608.2h / CR 113.7a: `source` carries the attachment as the source last
            // had it -- the Umezawa's Jitte ruling's "most recently equipped" creature
            // (OOS-DX5-3) when the resolution path supplied it.
            let Some(src) = source else {
                return false;
            };
            src.attached_to == Some(object_id)
        }
        // Applies to any permanent the source Aura/Equipment/Fortification is attached to.
        EffectFilter::AttachedPermanent => {
            if obj_zone != ZoneId::Battlefield {
                return false;
            }
            // CR 608.2h / CR 113.7a: `source` carries the attachment as the source last
            // had it -- the Umezawa's Jitte ruling's "most recently equipped" creature
            // (OOS-DX5-3) when the resolution path supplied it.
            let Some(src) = source else {
                return false;
            };
            src.attached_to == Some(object_id)
        }
        // CR 604.2: Static ability "Creatures you control have [keyword]."
        // Resolves the source's controller dynamically at layer-application time.
        EffectFilter::CreaturesYouControl => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 604.2: Static ability "Other creatures you control have [keyword]."
        // Same as CreaturesYouControl but excludes the source object itself.
        EffectFilter::OtherCreaturesYouControl => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if effect.source == Some(object_id) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
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
            if effect.source == Some(object_id) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 604.2: "Creatures your opponents control get -2/-2."
        // Applies to all creatures NOT controlled by the source's controller.
        EffectFilter::CreaturesOpponentsControl => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            let obj_controller = state.objects.get(&object_id).map(|obj| obj.controller);
            obj_controller.is_some() && obj_controller != Some(src.controller)
        }
        // CR 604.2: "[Subtype] creatures you control get +N/+N" (includes source).
        // Used for activated abilities like Ezuri where the source Elf benefits too.
        EffectFilter::CreaturesYouControlWithSubtype(subtype) => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if !chars.subtypes.contains(subtype) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 611.3a: "Attacking creatures you control have [keyword]."
        // Dynamic — checks state.combat.attackers at layer-application time.
        // Outside combat (state.combat is None), matches nothing.
        EffectFilter::AttackingCreaturesYouControl => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if !state
                .combat
                .as_ref()
                .is_some_and(|c| c.attackers.contains_key(&object_id))
            {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 604.2: "Artifacts you control have [keyword]." (Indomitable Archangel).
        EffectFilter::ArtifactsYouControl => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Artifact) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 604.2: "Legendary creatures you control get +1/+0." (Rising of the Day).
        // Checks supertypes — already layer-resolved at this point (Layers 4-5 before 6/7).
        EffectFilter::CreaturesYouControlWithSupertype(supertype) => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if !chars.supertypes.contains(supertype) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 604.2: "Red creatures you control have first strike." (Bloodmark Mentor).
        // Uses layer-resolved colors (colors resolved before Layer 6 ability grants).
        EffectFilter::CreaturesYouControlWithColor(color) => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if !chars.colors.contains(color) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 604.2: "Other non-[Subtype] creatures you control get +1/+1 and have undying."
        // (Mikaeus, the Unhallowed). Excludes source AND any creatures with the subtype.
        EffectFilter::OtherCreaturesYouControlExcludingSubtype(subtype) => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if chars.subtypes.contains(subtype) {
                return false;
            }
            if effect.source == Some(object_id) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 604.2: "Non-[Subtype] creatures you control get +3/+3 until end of turn."
        // (Return of the Wildspeaker). Includes source — used for spell/ability effects.
        EffectFilter::CreaturesYouControlExcludingSubtype(subtype) => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if chars.subtypes.contains(subtype) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 611.3a: "Attacking [Subtype] creatures you control have [keyword]."
        // (Crossway Troublemakers, Elderfang Venom). Dynamic — checks combat state.
        EffectFilter::AttackingCreaturesYouControlWithSubtype(subtype) => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if !chars.subtypes.contains(subtype) {
                return false;
            }
            if !state
                .combat
                .as_ref()
                .is_some_and(|c| c.attackers.contains_key(&object_id))
            {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 604.2: "[Subtype] creatures get +1/+1 until end of turn" (Bladewing the Risen).
        // No controller restriction — affects ALL players' creatures of the given type.
        EffectFilter::AllCreaturesWithSubtype(subtype) => {
            obj_zone == ZoneId::Battlefield
                && chars.card_types.contains(&CardType::Creature)
                && chars.subtypes.contains(subtype)
        }
        // CR 604.2: "Other [Subtype A] and [Subtype B] creatures you control get +1/+1."
        // (Silver-Fur Master). OR semantics: matches if creature has ANY of the subtypes.
        // Excludes source object.
        EffectFilter::OtherCreaturesYouControlWithSubtypes(subtypes) => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if !subtypes.iter().any(|st| chars.subtypes.contains(st)) {
                return false;
            }
            if effect.source == Some(object_id) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 305.7: "Lands you control are every basic land type in addition to their other types."
        // Matches all land permanents controlled by the same player as the effect's source.
        EffectFilter::LandsYouControl => {
            if obj_zone != ZoneId::Battlefield {
                return false;
            }
            if !chars.card_types.contains(&CardType::Land) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            state.objects.get(&object_id).map(|obj| obj.controller) == Some(src.controller)
        }
        // CR 205.3m: Creatures you control of the chosen type (INCLUDING source).
        // Reads chosen_creature_type from source permanent dynamically at layer time.
        EffectFilter::CreaturesYouControlOfChosenType => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            let obj_controller = state.objects.get(&object_id).map(|o| o.controller);
            obj_controller == Some(src.controller)
                && src
                    .chosen_creature_type
                    .map(|ct| chars.subtypes.contains(ct))
                    .unwrap_or(false)
        }
        // CR 205.3m: Other creatures you control of the chosen type (EXCLUDING source).
        // Used for Morophon's "+1/+1 to other creatures of the chosen type".
        // CR 613.1f: "Non-[Subtype] creatures" — all creatures (any controller) that
        // do NOT have the specified subtype. Used for Eyeblight Massacre, Olivia's Wrath.
        EffectFilter::AllCreaturesExcludingSubtype(subtype) => {
            obj_zone == ZoneId::Battlefield
                && chars.card_types.contains(&CardType::Creature)
                && !chars.subtypes.contains(subtype)
        }
        // CR 608.2h: this placeholder must be substituted at Effect::ApplyContinuousEffect
        // execution time (effects/mod.rs). Reaching it during layer application is a bug.
        EffectFilter::AllCreaturesExcludingChosenSubtype => {
            debug_assert!(
                false,
                "AllCreaturesExcludingChosenSubtype must be substituted before storage into ContinuousEffect"
            );
            false
        }
        // CR 614.12 / CR 105.1 / CR 613.1e: Creatures you control of the chosen color.
        // Reads source.chosen_color dynamically at layer-application time.
        // Color comparison uses layer-resolved colors (chars.colors already resolved by layer 5).
        EffectFilter::CreaturesYouControlOfChosenColor => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            let obj_controller = state.objects.get(&object_id).map(|o| o.controller);
            obj_controller == Some(src.controller)
                && src
                    .chosen_color
                    .map(|c| chars.colors.contains(&c))
                    .unwrap_or(false)
        }
        EffectFilter::OtherCreaturesYouControlOfChosenType => {
            if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
                return false;
            }
            if effect.source == Some(object_id) {
                return false;
            }
            let Some(src) = source else {
                return false;
            };
            let obj_controller = state.objects.get(&object_id).map(|o| o.controller);
            obj_controller == Some(src.controller)
                && src
                    .chosen_creature_type
                    .map(|ct| chars.subtypes.contains(ct))
                    .unwrap_or(false)
        }
    }
}
/// CR 611.2c: compute the set of objects a resolution-generated continuous
/// effect affects, at the moment it begins.
///
/// Uses `effect_applies_to` -- the SAME predicate the layer system applies -- so
/// determination and application cannot drift. The caller (`Effect::ApplyContinuousEffect`)
/// MUST call this before `state.continuous_effects.push_back(eff)`: the set is
/// determined by the board with all pre-existing effects applied and this one
/// not yet applying, and that ordering is also what keeps `calculate_characteristics`
/// from seeing the effect being created (it iterates `state.continuous_effects`,
/// which does not yet contain `eff`).
///
/// CR 702.26e falls out for free: `effect_applies_to`'s phased-out guard rejects
/// a phased-out candidate here, so it is excluded from the set permanently (it
/// will not re-enter even if it phases back in, because the set is locked).
///
/// One deliberate, and REAL, consequence (fix-cycle Finding 2 / OOS-DX5-6 --
/// corrected from the implement phase's mis-scoped, and false, "none exist"
/// claim): this reads FULLY layer-resolved characteristics
/// (`calculate_characteristics` runs the whole layer stack), whereas the live
/// path in `effect_applies_to` is called mid-`calculate_characteristics` with
/// `chars` resolved only through the layers *before* the current one -- and,
/// crucially, `calculate_characteristics` gathers ALL effects for a layer
/// before applying ANY of them, so at Layer *L* the live `chars` carries NO
/// Layer-*L* modification at all, earlier- or later-timestamped. The relevant
/// question is therefore not "which *mass-filter* defs write the filtered
/// characteristic at Layer <= 4" (the implement phase's mis-scoped check) but
/// "which effects of ANY filter write it" -- and `inkmoth_nexus` (and the rest
/// of the creature-land family) do exactly that: a Layer-4 `TypeChange` +
/// `AddCardTypes([Creature])` `EffectFilter::Source` effect. Mirror Entity
/// (`Complete`) is the corpus's one Layer<=4 mass-filter def
/// (`AddAllCreatureTypes`, `CreaturesYouControl`, `EffectLayer::TypeChange`);
/// animating Inkmoth Nexus and then activating Mirror Entity is a real,
/// reachable scenario in which this snapshot changes the outcome (Nexus now
/// receives every creature type; pre-PB-DX5 it did not, because the live
/// gather at Layer 4 saw it as a bare Land). Reproduced and pinned by T15 in
/// `pb_dx5_affected_set_snapshot.rs`. Full resolution IS the CR-correct input
/// either way (CR 611.2c determines the set once, from the state as it stands
/// when the effect begins, i.e. with all continuous effects -- including
/// Nexus's own animate, which ran first -- already applied), so this is not a
/// wrongness finding; it is a real, previously-untested behaviour change.
///
/// Only `Effect::ApplyContinuousEffect` populates `affected_set`. Every other
/// creation site is either a static ability (CR 611.3a: not locked in) or a
/// `SingleObject` filter (locking is a no-op): `rules/resolution.rs`'s 13
/// sites -- 12 `SingleObject` keyword-trigger/token grants, PLUS the
/// `StackObjectKind::ClassLevelAbility` arm (CR 716.2a level-up), which is
/// NEITHER a keyword-trigger grant nor `SingleObject` -- it forwards an
/// arbitrary card-def `Static` filter verbatim, and `None` is correct there
/// because that filter is registered by a static ability (CR 611.3a), not
/// because it happens to be `SingleObject` (fix-cycle Finding 4 -- the
/// implement phase's "13 keyword-trigger grants, all verified SingleObject"
/// characterisation was false for this 13th site); `rules/replacement.rs`'s
/// static-ability registrations; `effects/mod.rs`'s token P/T overrides;
/// `Effect::CreateEmblem`'s static-effect loop; `rules/copy.rs`'s Layer-1 copy
/// effect. A future mass filter at any other resolution-time site would be a
/// silent CR 611.2c hole -- filed as OOS-DX5-1 (widened by fix-cycle Finding 5
/// to also name the three *read* sites, `copy_effect_applies_to`,
/// `recompute_object_controller` and `expire_while_you_control_source_effects`,
/// that ignore `affected_set` entirely).
pub(crate) fn snapshot_affected_set(
    state: &GameState,
    effect: &ContinuousEffect,
) -> OrdSet<ObjectId> {
    // The set *is* that one object, by definition; costs nothing, scans
    // nothing, and keeps the 79 single-object-derived defs byte-identical
    // even when the object is not on the battlefield (CR 611.2c's set is
    // still exactly {id} in that case -- SingleObject is unconditional).
    if let EffectFilter::SingleObject(id) = &effect.filter {
        return OrdSet::unit(*id);
    }
    // CR 608.2h / CR 113.7a (PB-DX39, `OOS-DX5-3` / `OOS-DX5-7`): this is a
    // resolution-generated effect, so its source's controller / attachment / chosen
    // type / chosen colour come from the live object if the source is still in its
    // expected zone and from its LAST KNOWN INFORMATION if it is not. Umezawa's Jitte
    // destroyed in response to its own ability, and Mardu Ascendancy sacrificed as that
    // ability's cost, are both this case -- and before this the answer was "no source,
    // so the locked set is empty and the effect does nothing".
    //
    // Resolved ONCE, outside the candidate loop, and the same borrow is handed to every
    // candidate: the answer cannot vary between candidates (it is a property of the
    // source alone), and `effect_applies_to_inner` is on the layer walk.
    let source = effect
        .source
        .and_then(|sid| source_view_at_resolution(state, sid));
    let mut affected = OrdSet::new();
    for object_id in candidate_ids_for_filter(state, &effect.filter) {
        // SR-25: `expect_object`, not a bare `.objects.get(..)` -- `object_id`
        // was enumerated from `state.objects` inside `candidate_ids_for_filter`
        // moments ago in the same synchronous call, with no mutation in
        // between, so a `None` here is an engine bug (CR 400.7 cannot apply),
        // not a legitimate fizzle. `expect_characteristics` shares the same
        // classification and is used below for the same reason.
        let Some(obj) = state.expect_object(object_id) else {
            continue;
        };
        let obj_zone = obj.zone;
        let chars = expect_characteristics(state, object_id);
        if effect_applies_to_inner(state, effect, object_id, obj_zone, &chars, source.as_ref()) {
            affected.insert(object_id);
        }
    }
    affected
}
/// CR 611.2c: classify an `EffectFilter` into the zone scope its candidates can
/// come from, for `snapshot_affected_set`'s battlefield/graveyard scan.
///
/// Exhaustive match, **no `_` arm**: a new `EffectFilter` variant is a compile
/// error here until it is classified, mirroring the SR-5 keyword-catchall
/// discipline. `SingleObject` is included for completeness even though
/// `snapshot_affected_set` short-circuits it before this is ever called.
fn candidate_ids_for_filter(state: &GameState, filter: &EffectFilter) -> Vec<ObjectId> {
    match filter {
        EffectFilter::SingleObject(id) => vec![*id],
        EffectFilter::AllCardsInGraveyards => state
            .objects
            .iter()
            .filter(|(_, obj)| matches!(obj.zone, ZoneId::Graveyard(_)))
            .map(|(id, _)| *id)
            .collect(),
        EffectFilter::AllCreatures
        | EffectFilter::AllLands
        | EffectFilter::AllNonbasicLands
        | EffectFilter::AllEnchantments
        | EffectFilter::AllNonAuraEnchantments
        | EffectFilter::AllPermanents
        | EffectFilter::ControlledBy(_)
        | EffectFilter::CreaturesControlledBy(_)
        | EffectFilter::AttachedCreature
        | EffectFilter::AttachedLand
        | EffectFilter::AttachedPermanent
        | EffectFilter::CreaturesYouControl
        | EffectFilter::OtherCreaturesYouControl
        | EffectFilter::OtherCreaturesYouControlWithSubtype(_)
        | EffectFilter::CreaturesOpponentsControl
        | EffectFilter::CreaturesYouControlWithSubtype(_)
        | EffectFilter::AttackingCreaturesYouControl
        | EffectFilter::ArtifactsYouControl
        | EffectFilter::CreaturesYouControlWithSupertype(_)
        | EffectFilter::CreaturesYouControlWithColor(_)
        | EffectFilter::OtherCreaturesYouControlExcludingSubtype(_)
        | EffectFilter::CreaturesYouControlExcludingSubtype(_)
        | EffectFilter::AttackingCreaturesYouControlWithSubtype(_)
        | EffectFilter::AllCreaturesWithSubtype(_)
        | EffectFilter::OtherCreaturesYouControlWithSubtypes(_)
        | EffectFilter::LandsYouControl
        | EffectFilter::CreaturesYouControlOfChosenType
        | EffectFilter::OtherCreaturesYouControlOfChosenType
        | EffectFilter::AllCreaturesExcludingSubtype(_)
        | EffectFilter::CreaturesYouControlOfChosenColor => state
            .objects
            .iter()
            .filter(|(_, obj)| obj.zone == ZoneId::Battlefield)
            .map(|(id, _)| *id)
            .collect(),
        // These five variants are placeholders substituted into a concrete
        // filter (usually SingleObject) at `Effect::ApplyContinuousEffect`
        // execution time, before the effect is built -- see the `match
        // &effect_def.filter` block at the top of that arm. They must never
        // reach a stored `ContinuousEffect`, so they must never reach here.
        EffectFilter::DeclaredTarget { .. }
        | EffectFilter::Source
        | EffectFilter::TriggeringCreature
        | EffectFilter::CreaturesControlledByDefendingPlayer
        | EffectFilter::AllCreaturesExcludingChosenSubtype => {
            debug_assert!(
                false,
                "{filter:?} must be substituted before storage into ContinuousEffect"
            );
            vec![]
        }
    }
}

/// PB-DX5 T11: `snapshot_affected_set`'s zone-scope shortcut
/// (`candidate_ids_for_filter`) must agree with a brute-force scan over EVERY
/// object in `state.objects`, regardless of zone. This is the only place this
/// property can be tested: `snapshot_affected_set`, `effect_applies_to_object`
/// and `candidate_ids_for_filter` are all `pub(crate)`, unreachable from the
/// `crates/engine/tests/` integration crate -- hence an in-source unit test
/// rather than a member of `pb_dx5_affected_set_snapshot.rs`, mirroring the
/// `expect_characteristics_tests` precedent a few hundred lines above.
///
/// Fix-cycle Finding 9: the board and filter list were widened from the
/// original 6-filter set (two of which returned small or empty sets on that
/// fixture, and none of which exercised the CR 702.26e phased-out guard) to
/// also cover a phased-out battlefield permanent (so the guard is part of the
/// agreement being checked, not just the zone-scope shortcut) and a real
/// `AttachedCreature` match (an Equipment actually attached to a creature,
/// rather than the previous fixture's unconditionally-empty case) and a
/// subtype-filtered variant (`AllCreaturesWithSubtype`), so all three
/// predicate shapes named in the original plan -- zone, controller/attachment,
/// `chars`-reading -- are exercised with a non-trivial result.
#[cfg(test)]
mod pb_dx5_snapshot_tests {
    use super::*;
    use crate::state::continuous_effect::EffectId;
    use crate::state::types::SubType;
    use crate::state::{GameStateBuilder, ObjectSpec, PlayerId, ZoneId};

    /// A board with objects in battlefield, graveyard, hand, library and exile,
    /// so the brute-force comparison actually exercises every zone
    /// `candidate_ids_for_filter` claims NOT to need to scan. Also carries a
    /// phased-out battlefield creature and an Equipment attached to a
    /// battlefield creature (fix-cycle Finding 9), set up post-build since the
    /// builder has no first-class support for either.
    fn multi_zone_board() -> GameState {
        let mut state = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .object(ObjectSpec::creature(PlayerId(1), "BF Creature P1", 2, 2))
            .object(ObjectSpec::creature(PlayerId(2), "BF Creature P2", 2, 2))
            .object(
                ObjectSpec::creature(PlayerId(1), "BF Phased Out Creature", 1, 1)
                    .with_subtypes(vec![SubType("Goblin".to_string())]),
            )
            .object(
                ObjectSpec::creature(PlayerId(1), "BF Goblin", 1, 1)
                    .with_subtypes(vec![SubType("Goblin".to_string())]),
            )
            .object(
                ObjectSpec::card(PlayerId(1), "BF Equipment")
                    .with_types(vec![CardType::Artifact])
                    .with_subtypes(vec![SubType("Equipment".to_string())])
                    .in_zone(ZoneId::Battlefield),
            )
            .object(
                ObjectSpec::card(PlayerId(1), "BF Land")
                    .with_types(vec![CardType::Land])
                    .in_zone(ZoneId::Battlefield),
            )
            .object(
                ObjectSpec::creature(PlayerId(1), "GY Creature", 1, 1)
                    .in_zone(ZoneId::Graveyard(PlayerId(1))),
            )
            .object(
                ObjectSpec::creature(PlayerId(1), "Hand Creature", 1, 1)
                    .in_zone(ZoneId::Hand(PlayerId(1))),
            )
            .object(
                ObjectSpec::creature(PlayerId(1), "Library Creature", 1, 1)
                    .in_zone(ZoneId::Library(PlayerId(1))),
            )
            .object(
                ObjectSpec::creature(PlayerId(1), "Exile Creature", 1, 1).in_zone(ZoneId::Exile),
            )
            .build()
            .expect("multi-zone board builds");

        let phased_id = find(&state, "BF Phased Out Creature");
        if let Some(obj) = state.expect_object_mut(phased_id) {
            obj.status.phased_out = true;
        }
        let equipment_id = find(&state, "BF Equipment");
        let goblin_id = find(&state, "BF Goblin");
        if let Some(obj) = state.expect_object_mut(equipment_id) {
            obj.attached_to = Some(goblin_id);
        }
        state
    }

    /// A brute-force reimplementation of what `snapshot_affected_set` computes,
    /// scanning `state.objects` in ITS ENTIRETY (every zone) rather than using
    /// `candidate_ids_for_filter`'s zone-scope shortcut, but calling the exact
    /// same `effect_applies_to_object` predicate. The two must agree.
    fn brute_force_affected_set(state: &GameState, effect: &ContinuousEffect) -> OrdSet<ObjectId> {
        let mut out = OrdSet::new();
        for (id, obj) in state.objects.iter() {
            let Some(chars) = calculate_characteristics(state, *id) else {
                continue;
            };
            if effect_applies_to_object(state, effect, *id, obj.zone, &chars) {
                out.insert(*id);
            }
        }
        out
    }

    fn find(state: &GameState, name: &str) -> ObjectId {
        state
            .objects
            .iter()
            .find(|(_, o)| o.characteristics.name == name)
            .map(|(id, _)| *id)
            .unwrap_or_else(|| panic!("'{name}' not found"))
    }

    fn effect_with(source: Option<ObjectId>, filter: EffectFilter) -> ContinuousEffect {
        ContinuousEffect {
            id: EffectId(1),
            source,
            timestamp: 1,
            layer: EffectLayer::PtModify,
            duration: EffectDuration::UntilEndOfTurn,
            filter,
            modification: LayerModification::ModifyBoth(1),
            is_cda: false,
            affected_set: None,
            condition: None,
        }
    }

    #[test]
    fn snapshot_matches_brute_force_over_every_zone() {
        let state = multi_zone_board();
        let bf_p1 = find(&state, "BF Creature P1");
        let equipment = find(&state, "BF Equipment");

        // One representative filter per zone-scope bucket in
        // `candidate_ids_for_filter`'s table: battlefield-scope (several
        // shapes, including a subtype-filtered one -- the `chars`-reading
        // predicate shape), graveyard-scope, and TWO `AttachedCreature`
        // cases -- one sourced at an object attached to nothing (empty, the
        // original edge case) and one sourced at the Equipment actually
        // attached to "BF Goblin" (a real, non-trivial match; fix-cycle
        // Finding 9). The board also carries a phased-out battlefield
        // creature, so `AllCreatures`/`AllPermanents`/etc. exercise the CR
        // 702.26e guard as part of the agreement being checked, not just the
        // zone-scope shortcut.
        let filters: Vec<ContinuousEffect> = vec![
            effect_with(None, EffectFilter::AllCreatures),
            effect_with(None, EffectFilter::AllPermanents),
            effect_with(None, EffectFilter::AllCardsInGraveyards),
            effect_with(Some(bf_p1), EffectFilter::CreaturesYouControl),
            effect_with(Some(bf_p1), EffectFilter::CreaturesOpponentsControl),
            effect_with(Some(bf_p1), EffectFilter::AttachedCreature),
            effect_with(Some(equipment), EffectFilter::AttachedCreature),
            effect_with(
                None,
                EffectFilter::AllCreaturesWithSubtype(SubType("Goblin".to_string())),
            ),
        ];

        for eff in &filters {
            let shortcut = snapshot_affected_set(&state, eff);
            let brute = brute_force_affected_set(&state, eff);
            assert_eq!(
                shortcut, brute,
                "candidate_ids_for_filter's zone-scope shortcut disagrees with a \
                 brute-force scan over every zone for filter {:?}",
                eff.filter
            );
        }
    }
}

/// CR 305.6 / CR 613.1d (PB-DX43): derive the intrinsic "{T}: Add [symbol]"
/// mana ability for every basic land type present in `chars.subtypes`, once
/// this object's Layer-4 type-changing effects have all been applied for the
/// current layer-walk iteration. See the call site (end of the Layer-4 pass
/// inside `calculate_characteristics`) for why the derivation runs exactly
/// there rather than post-walk or inside an individual `apply_layer_
/// modification` arm.
///
/// No-op unless `chars.card_types` contains `CardType::Land` — CR 305.6 reads
/// "An object with the land card type AND a basic land type has the intrinsic
/// ability ...", both conjuncts required. Consequently a face-down permanent
/// (`chars.card_types == {Creature}`, `chars.subtypes == {}` per CR 708.2a,
/// set before the layer loop even starts) derives nothing.
///
/// For each of CR 305.6's five basic types, in CR 305.6's own listed order
/// (`BASIC_LAND_TYPES`, not `OrdSet` iteration order), if the object carries
/// that subtype and does not already carry a `ManaAbility` that discharges
/// the intrinsic for that colour (`discharges_intrinsic_mana_ability`, D4),
/// appends `ManaAbility::tap_for(color)`.
///
/// Idempotent by construction (PB-DX43 D3/D4). This is load-bearing, not a
/// nicety, for three reasons: (1) it lets a basic land's own hand-authored
/// `{T}: Add [symbol]` (`swamp.rs` et al.) coexist with the derivation
/// without ever producing a second, duplicate ability — the printed ability
/// keeps its original index (`Command::TapForMana.ability_index` stays a
/// stable, dense index into `mana_abilities`), so no client's existing
/// index-0 tap-for-mana command silently starts referring to a different
/// ability; (2) it is what closes `OOS-DX27-10` (two "nonbasic lands are
/// Mountains" sources on the same board, e.g. Blood Moon + Magus of the
/// Moon) WITHOUT a `push_back` dedup guard at either card def — one
/// derivation pass appends exactly one `{R}`, because the second pass finds
/// the first pass's grant already discharging the intrinsic; (3) it means a
/// land whose type is set to a basic type by TWO independent effects (e.g.
/// Urborg AND the Dryad, both naming Swamp) still ends up with exactly one
/// `{T}: Add {B}`, not two.
fn derive_intrinsic_land_mana_abilities(chars: &mut Characteristics) {
    if !chars.card_types.contains(&CardType::Land) {
        return;
    }
    for (name, color) in crate::state::types::BASIC_LAND_TYPES {
        // Compare the interned strings directly rather than allocating a `SubType` per basic type
        // per call (`/review` Issue 11). This runs for EVERY land on EVERY
        // `calculate_characteristics` call, which is the hottest path in the layer walk; the
        // first draft allocated up to five `String`s each time to build throwaway lookup keys.
        // `OrdSet::contains` would be O(log n) against ~1-2 subtypes, so the linear scan is not a
        // regression at these sizes and it allocates nothing.
        if !chars.subtypes.iter().any(|st| st.0 == name) {
            continue;
        }
        let already_present = chars
            .mana_abilities
            .iter()
            .any(|ma| discharges_intrinsic_mana_ability(ma, color));
        if !already_present {
            chars.mana_abilities.push_back(ManaAbility::tap_for(color));
        }
    }
}
/// PB-DX43 design decision D4: does an existing `ManaAbility` already
/// discharge CR 305.6's intrinsic, unconditional "{T}: Add [color]" for
/// `color`? An ability counts only if it is EXACTLY that ability and
/// nothing more — written as an exhaustive struct destructure with **no**
/// `..` rest pattern (the SR-5 idiom): a future field added to `ManaAbility`
/// is a compile error here until someone decides whether it belongs in the
/// predicate, rather than silently being ignored.
///
/// A conditioned (`activation_condition`) or costed (`life_cost`,
/// `mana_cost`, `sacrifice_self`, `exile_self_from_hand`, `remove_counter`,
/// `damage_to_controller`) ability does NOT discharge the intrinsic: CR
/// 305.6's ability is unconditional and free, so a land with a restricted or
/// costed "{T}: Add {B}" that becomes a Swamp genuinely gains a SECOND,
/// unrestricted one.
fn discharges_intrinsic_mana_ability(ma: &ManaAbility, color: ManaColor) -> bool {
    let ManaAbility {
        produces,
        requires_tap,
        sacrifice_self,
        any_color,
        damage_to_controller,
        mana_cost,
        life_cost,
        scaled_amount,
        activation_condition,
        exile_self_from_hand,
        remove_counter,
    } = ma;
    *requires_tap
        && !*sacrifice_self
        && !*any_color
        && *damage_to_controller == 0
        && mana_cost.is_none()
        && *life_cost == 0
        && scaled_amount.is_none()
        && activation_condition.is_none()
        && !*exile_self_from_hand
        && remove_counter.is_none()
        && produces.len() == 1
        && produces.get(&color).copied() == Some(1)
}

/// Blank every ability-bearing field of `chars` — the operation CR 305.7's *"It loses
/// all abilities generated from its rules text, its old land types, and any copiable
/// effects affecting that land"* and `LayerModification::RemoveAllAbilities` both
/// perform.
///
/// **Written as an exhaustive destructure with no `..` rest pattern on purpose**
/// (PB-DX43 `/review` Issue 6, the SR-5 idiom this file already uses for
/// [`discharges_intrinsic_mana_ability`]). Before this existed the same five
/// assignments were hand-written twice, in the `SetLandTypes` and
/// `RemoveAllAbilities` arms, with nothing keeping them in sync: a sixth
/// ability-bearing field on `Characteristics` would have been added to one copy and
/// silently missed by the other. Now a new field is a compile error here until
/// someone decides whether blanking should clear it.
///
/// The non-ability fields are bound and explicitly dropped rather than elided, so the
/// destructure documents what blanking deliberately does NOT touch (CR 613.1f removes
/// abilities; it does not touch types, colours or P/T).
fn clear_all_abilities(chars: &mut Characteristics) {
    let Characteristics {
        // Ability-bearing — cleared.
        keywords,
        mana_abilities,
        activated_abilities,
        triggered_abilities,
        abilities,
        // Deliberately untouched by an ability blank.
        name: _,
        mana_cost: _,
        colors: _,
        color_indicator: _,
        supertypes: _,
        card_types: _,
        subtypes: _,
        rules_text: _,
        power: _,
        toughness: _,
        loyalty: _,
        defense: _,
    } = chars;
    *keywords = OrdSet::new();
    *mana_abilities = imbl::Vector::new();
    *activated_abilities = Vec::new();
    *triggered_abilities = Vec::new();
    *abilities = imbl::Vector::new();
}

/// Does a `SetLandTypes` payload name at least one of CR 305.6's five basic land
/// types? This is CR 305.7's own stated precondition — *"If an effect sets a land's
/// subtype to one or more of the **basic** land types"* — so a payload naming only
/// nonbasic land types (Gate, Cave, Desert, ...) sets the type without triggering
/// either the ability loss or any intrinsic mana grant.
fn set_land_types_payload_is_basic(new_types: &OrdSet<SubType>) -> bool {
    new_types
        .iter()
        .any(|st| crate::state::types::basic_land_type_mana_color(st).is_some())
}

/// Does `modification` blank an object's abilities? **Exhaustive over every
/// `LayerModification` variant with no wildcard arm** — the enum has **33** variants (2
/// classified `true`, 31 `false`), so adding a 34th is a compile error here until someone
/// classifies it. (The first draft of this line said "31st", counting only the `false`
/// arm's list — `/review` N5. The count is stated because it is checkable, and it was
/// wrong the first time.)
///
/// # Why this function exists (PB-DX43 `/review` Issue 1, a HIGH)
///
/// Before PB-DX43 there was exactly one way to blank a permanent's abilities — a
/// Layer-6 `RemoveAllAbilities` — so every consumer that needed to ask "are this
/// object's abilities gone?" could and did match that one variant literally. PB-DX43
/// added a **second** channel: CR 305.7's loss, performed by the Layer-4
/// `SetLandTypes` arm when its payload is basic. `replacement.rs`'s IG-1 ETB-trigger
/// suppressor was still matching the first channel only, so deleting Blood Moon's and
/// Magus of the Moon's Layer-6 static — correct in itself — silently re-enabled the
/// CardDef ETB triggers of **26** nonbasic land defs entering under either moon (the
/// ten Karoo bounce lands, the six Temples, the five gain-lands, and others), with the
/// whole test suite green.
///
/// That is this batch's own thesis arriving inside its own work: **a gate written for
/// one variant measures that variant.** The fix is not to add a second `matches!` at
/// the one call site but to encode the question once, exhaustively, so the next
/// channel cannot be added without every consumer being forced to notice.
///
/// CR 603.2 is why IG-1 cares: a triggered ability only exists to trigger if the
/// object has it, and an object whose abilities are blanked has none.
///
/// `chars` is the object the modification would apply to. It is a parameter because CR
/// 305.7 is scoped to **lands** — a modification alone cannot answer the question, and an
/// earlier draft that tried (checking only the payload) would have had this function tell
/// IG-1 to suppress a non-land's ETB triggers while the layer walk correctly declined to
/// blank it (`/review` N4). Callers pass whatever characteristics they are reasoning
/// about: the layer walk passes the in-flight `chars`, IG-1 passes the entering object's
/// base characteristics (the same basis it evaluates the effect's filter against).
pub fn modification_blanks_abilities(
    modification: &LayerModification,
    chars: &Characteristics,
) -> bool {
    match modification {
        // Channel 1 (CR 613.1f, Layer 6): the explicit ability wipe.
        LayerModification::RemoveAllAbilities => true,
        // Channel 2 (CR 305.7, Layer 4): setting a land's subtype to one or more BASIC
        // land types makes it lose the abilities from its rules text and old land
        // types. A nonbasic payload does not (CR 305.7's own precondition).
        LayerModification::SetLandTypes(new_types) => {
            // Both conjuncts, because CR 305.7 opens "If an effect sets **a land's** subtype to
            // one or more of the **basic** land types". `chars` is why this function takes the
            // object at all: a modification alone cannot answer a rule that is scoped to lands.
            chars.card_types.contains(&CardType::Land) && set_land_types_payload_is_basic(new_types)
        }
        // Everything else leaves at least some ability intact **as this engine implements
        // it today** — that is a claim about the code, not about the CR, and one arm below
        // is a known gated residual (see `SetTypeLine`).
        //
        // `CopyOf` is deliberately NOT a blanking channel: it REPLACES the copiable values
        // wholesale (CR 707.2), and the copy may well have abilities of its own — "blanked"
        // is a different claim from "changed". Whether the copy source happens to have no
        // abilities is a property of the RESULT, not of the modification.
        //
        // **`SetTypeLine` is a third CR 305.7 channel, half-implemented** (`/review` N3).
        // Its arm sets `supertypes`/`card_types`/`subtypes` and clears nothing, so a
        // payload naming a basic land subtype would gain the CR 305.6 intrinsic mana
        // ability — `derive_intrinsic_land_mana_abilities` reads the FINAL subtype set and
        // cannot tell which arm produced it — while skipping CR 305.7's ability loss. It is
        // classified `false` here because that is what the arm actually does; the honest
        // fix is to make the arm perform the loss, not to lie here. **Zero corpus members
        // today**, and `core::pb_dx43_land_type_roster` R1 walks the `SetTypeLine` axis and
        // pins the conferring population by name, so a new member reddens R1 rather than
        // arriving silently.
        LayerModification::CopyOf(_)
        | LayerModification::SetController(_)
        | LayerModification::SetTypeLine { .. }
        | LayerModification::AddCardTypes(_)
        | LayerModification::RemoveCardTypes(_)
        | LayerModification::AddSubtypes(_)
        | LayerModification::LoseAllSubtypes
        | LayerModification::RemoveSuperType(_)
        | LayerModification::AddAllCreatureTypes
        | LayerModification::SetCreatureTypes(_)
        | LayerModification::SetCardTypes(_)
        | LayerModification::SetColors(_)
        | LayerModification::AddColors(_)
        | LayerModification::BecomeColorless
        | LayerModification::AddKeyword(_)
        | LayerModification::AddKeywords(_)
        | LayerModification::RemoveKeyword(_)
        | LayerModification::AddActivatedAbility(_)
        | LayerModification::AddManaAbility(_)
        | LayerModification::SetPtViaCda { .. }
        | LayerModification::SetPtDynamic { .. }
        | LayerModification::SetPtToManaValue
        | LayerModification::SetPowerToughness { .. }
        | LayerModification::SetBothDynamic { .. }
        | LayerModification::ModifyPower(_)
        | LayerModification::ModifyToughness(_)
        | LayerModification::ModifyBoth(_)
        | LayerModification::ModifyBothDynamic { .. }
        | LayerModification::ModifyPowerDynamic { .. }
        | LayerModification::ModifyToughnessDynamic { .. }
        | LayerModification::SwitchPowerToughness => false,
    }
}

/// CR 613.1f / CR 305.7 / CR 708.2a — **the** ability-blanking predicate: does anything
/// leave this permanent with no abilities at all?
///
/// This is the single question "are `id`'s abilities gone", asked once for the whole
/// engine. Two channels reach it today:
///
/// 1. **CR 708.2a face-down** — *"no text, no name, no subtypes, and no mana cost"*. A
///    face-down permanent has no abilities by definition, and (load-bearing elsewhere) no
///    subtypes either, so it is not a Saga / not an Equipment / not anything.
/// 2. **A continuous effect whose modification blanks abilities** — CR 613.1f's Layer-6
///    `RemoveAllAbilities`, and CR 305.7's Layer-4 `SetLandTypes` with a basic payload.
///
/// **The classification of channel 2 is delegated to [`modification_blanks_abilities`]
/// rather than matched here** (PB-DX43 `/review` Issue 1). That function is exhaustive
/// over `LayerModification` with **no wildcard arm**, so a *fourth* channel is a compile
/// error until someone classifies it. The alternative — a local `matches!` — is exactly
/// what was wrong before: `replacement.rs`'s IG-1 suppressor used to read
/// `e.layer == EffectLayer::Ability && matches!(e.modification, RemoveAllAbilities)`,
/// correct while Layer-6 `RemoveAllAbilities` was the only blanking channel and silently
/// wrong the moment PB-DX43 made CR 305.7's ability loss a **Layer-4** consequence of
/// `SetLandTypes`. Blood Moon and Magus of the Moon dropped their Layer-6 static in that
/// batch, so the scan stopped seeing them and 26 nonbasic land defs (the ten Karoos, the
/// six Temples, the five gain-lands, ...) began firing CardDef ETB triggers off a land
/// whose abilities were gone — with the whole suite green. **Keying on the modification
/// rather than on the layer is what makes a third channel impossible to add silently.**
///
/// The effect filter is evaluated against the object's **stored** `obj.characteristics`,
/// never against `calculate_characteristics`. That is IG-1's deliberate choice and this
/// function does not change it: CardDef abilities are not in `chars`, so the layer-resolved
/// characteristics cannot answer this question anyway, and re-entering the layer walk from
/// here would reopen `OOS-SIM2-6`'s recursion. The same `chars` is handed to
/// [`modification_blanks_abilities`] because CR 305.7 is scoped to **lands** — a
/// modification alone cannot answer a rule with a type precondition.
///
/// A missing object yields `false`: that is a CR 400.7 fizzle (nothing here to blank),
/// not a blank.
pub fn abilities_are_blanked(state: &GameState, id: ObjectId) -> bool {
    // Channel 1 — CR 708.2a. The `face_down_as.is_some()` conjunct is the same one
    // `layers.rs`'s face-down characteristics path and `replacement.rs`'s CR 708.3 return
    // already use, and it is there to distinguish morph/manifest/cloak (a face-down
    // *permanent*) from Foretell/Hideaway's unrelated `face_down` usage on cards in other
    // zones. Do not invent a second spelling of this test.
    let Some(obj) = state.fizzle_object(id) else {
        return false;
    };
    if obj.status.face_down && obj.face_down_as.is_some() {
        return true;
    }
    // Channel 2 — the continuous-effect scan.
    let obj_zone = obj.zone;
    let chars = obj.characteristics.clone();
    state
        .continuous_effects
        .iter()
        .filter(|e| is_effect_active(state, e))
        .any(|e| {
            modification_blanks_abilities(&e.modification, &chars)
                && effect_applies_to_object(state, e, id, obj_zone, &chars)
        })
}

/// Apply a single layer modification to the given characteristics.
///
/// `state` is needed for Layer 1 copy effects to look up the target object's
/// copiable values (CR 707.2).  `mana_value` is the object's printed mana value,
/// used for `SetPtToManaValue`.
///
/// # Documented deviation: P/T saturates at `i32` bounds (OOS-SIM2-5, PB-DX19)
///
/// The Comprehensive Rules put **no ceiling on power or toughness**. This engine
/// stores both as `i32`, so an unbounded doubling chain — `devilish_valet` is the
/// worked example, and it is `Complete`: `effects/mod.rs` substitutes its
/// `ModifyPowerDynamic` to a concrete `ModifyPower(current_power)` per trigger
/// (CR 608.2h), so each trigger *adds the creature's current power* — reaches
/// `i32::MAX` in about 31 triggers, which is a reachable number of combat triggers
/// in a Commander game.
///
/// Every P/T write in this file therefore uses `saturating_add` /
/// `saturating_sub` / `saturating_neg` — the six `Modify*` / `Modify*Dynamic` arms
/// below, and the `+1/+1` / `-1/-1` counter path in `calculate_characteristics`,
/// which additionally widens its `u32` counter counts with
/// `try_into().unwrap_or(i32::MAX)` rather than `as i32`.
/// **The choice matters because the two supported build profiles fail differently**:
/// `Cargo.toml`'s `[profile.fuzz]` sets `overflow-checks = true`, so bare `+=`
/// *panicked* there, while a plain `--release` build *wrapped silently to negative
/// power* — a creature that "gets huge" would quietly become a 0/0 and die to
/// CR 704.5a. An `as` cast, note, does neither: it wraps in **every** profile,
/// including under `overflow-checks`, which is why the counter widening could not
/// stay an `as`.
///
/// Saturating is a deviation, not a rules-correct answer: a creature pinned at
/// `i32::MAX` power is wrong per CR, just far less wrong than one that wrapped
/// negative and died. Making the ceiling unreachable means widening the stored type
/// (or clamping at the effect layer with an explicit CR-blessed rule), and that is
/// out of scope here — filed as OOS-DX19-3.
fn apply_layer_modification(
    state: &GameState,
    chars: &mut Characteristics,
    modification: &LayerModification,
    mana_value: u32,
    object_id: ObjectId,
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
        // CR 707.9b: Remove a single supertype (e.g., "except it isn't legendary").
        LayerModification::RemoveSuperType(st) => {
            chars.supertypes.remove(st);
        }
        // CR 702.73a, CR 205.3m: Adds every creature type (used by Changeling CDA and
        // Maskwood Nexus-style "is every creature type" continuous effects).
        LayerModification::AddAllCreatureTypes => {
            for s in crate::state::types::ALL_CREATURE_TYPES.iter() {
                chars.subtypes.insert(s.clone());
            }
        }
        // CR 205.1a: SETS the creature-type subtypes, replacing only the creature-type
        // subset of `subtypes` while preserving land/artifact/enchantment/planeswalker/
        // spell subtypes (mirrors the Reconfigure idiom above at ~line 308).
        LayerModification::SetCreatureTypes(new_types) => {
            let mut kept: OrdSet<SubType> = chars
                .subtypes
                .iter()
                .filter(|st| !crate::state::types::ALL_CREATURE_TYPES.contains(*st))
                .cloned()
                .collect();
            for s in new_types {
                kept.insert(s.clone());
            }
            chars.subtypes = kept;
        }
        // CR 205.1a: SETS the card types, leaving supertypes untouched. Companion to
        // `SetCreatureTypes` — used together so "becomes a [type] creature" effects
        // preserve supertypes (e.g. Legendary) that `SetTypeLine` would wipe.
        //
        // CR 205.1a correlated-subtype-removal clause: "If an object's card type is
        // removed, the subtypes correlated with that card type will remain if they
        // are also the subtypes of a card type the object currently has; otherwise,
        // they are also removed for the entire time the object's card type is
        // removed." E.g. Darksteel Mutation on a Shrine (enchantment-creature):
        // Enchantment is removed, so the Shrine subtype must drop too. But an
        // Equipment subtype survives if Artifact is retained (Darksteel Mutation
        // keeps Artifact). A subtype not in any recognized CR 205.3 correlated-set
        // (`correlated_card_types` returns empty) is left untouched.
        LayerModification::SetCardTypes(new_types) => {
            chars.card_types = new_types.clone();
            chars.subtypes = chars
                .subtypes
                .iter()
                .filter(|st| {
                    let correlated = crate::state::types::correlated_card_types(st);
                    correlated.is_empty()
                        || correlated.iter().any(|ct| chars.card_types.contains(ct))
                })
                .cloned()
                .collect();
        }
        // CR 205.1a: SETS the LAND-type subtypes, replacing only the land-type subset
        // of `subtypes` while preserving creature/artifact/enchantment/planeswalker/
        // spell subtypes and (unlike `SetTypeLine`) leaving `card_types` and
        // `supertypes` untouched entirely. Mirrors `SetCreatureTypes` above, keyed off
        // `ALL_LAND_TYPES` instead of `ALL_CREATURE_TYPES`. Used by "[nonbasic] lands
        // are Mountains" effects (Blood Moon, Magus of the Moon — OOS-ADJ-7): the
        // printed cards change only land subtypes, never the Artifact/Creature card
        // type an artifact land or creature land (e.g. Ancient Den, Dryad Arbor) has.
        //
        // PB-DX43 / CR 305.7: "If an effect sets a land's subtype to one or more of
        // the basic land types, the land no longer has its old land type. It loses
        // all abilities generated from its rules text, its old land types, and any
        // copiable effects affecting that land, and it gains the appropriate mana
        // ability for each new basic land type. Note that this doesn't remove any
        // abilities that were granted to the land by other effects." So: IFF
        // `new_types` intersects the five BASIC land types (CR 305.7's own stated
        // precondition — a hypothetical "becomes a Gate" `SetLandTypes` payload sets
        // a land type WITHOUT this clause applying), this arm additionally clears
        // `keywords`, `mana_abilities`, `activated_abilities`, `triggered_abilities`
        // and `abilities` — the land's printed/rules-text abilities and whatever an
        // earlier land-type change ("its old land types") had produced. The intrinsic
        // mana ability for each new basic type is then supplied separately by
        // `derive_intrinsic_land_mana_abilities`, run once per Layer-4 pass after
        // every `apply_layer_modification` call in this layer (including this one)
        // has completed — this is NOT the same claim as "leaves everything but
        // `subtypes` untouched" the comment above made before PB-DX43.
        //
        // This removal belongs in LAYER 4, not a separate layer-6 static (the
        // pre-PB-DX43 shape: one `RemoveAllAbilities` static per moon card in
        // `blood_moon.rs`/`magus_of_the_moon.rs`): CR 305.7's loss is a direct
        // consequence of the type-SETTING event itself, the same layer-4 event that
        // replaces the land's old land type — CR 613.1d says layer 4 is where
        // type-changing effects (and their direct consequences) apply. Doing it here
        // rather than in a companion Layer-6 removal is also what makes the rule's
        // OWN final sentence true: any Layer-6 ability GRANTED to the land by
        // another effect (Cryptolith Rite, Chromatic Lantern, The World Tree, ...)
        // is applied to `chars` by Layer 6 running strictly AFTER this Layer-4
        // clearing, so it survives no matter its own timestamp — whereas a Layer-6
        // `RemoveAllAbilities` static (timestamp-ordered against every other Layer-6
        // effect) could instead strip an earlier-timestamped Layer-6 grant right
        // along with the land's own abilities, which CR 305.7 explicitly forbids.
        LayerModification::SetLandTypes(new_types) => {
            let mut kept: OrdSet<SubType> = chars
                .subtypes
                .iter()
                .filter(|st| !crate::state::types::ALL_LAND_TYPES.contains(*st))
                .cloned()
                .collect();
            for s in new_types {
                kept.insert(s.clone());
            }
            chars.subtypes = kept;
            // CR 305.7's ability loss, delegated to the SAME predicate `replacement.rs`'s IG-1
            // asks (`/review` N4). Restating the conjuncts here instead would put the rule in two
            // places and let them drift — which is the defect the shared predicate was created to
            // fix one review round earlier.
            if modification_blanks_abilities(modification, chars) {
                clear_all_abilities(chars);
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
            clear_all_abilities(chars);
        }
        LayerModification::RemoveKeyword(kw) => {
            chars.keywords.remove(kw);
        }
        // CR 613.1f: Grants a single non-mana activated ability; appended to vec.
        // Multiple grant sources produce multiple entries (CR 613.5).
        LayerModification::AddActivatedAbility(ability) => {
            chars.activated_abilities.push((**ability).clone());
        }
        // CR 605.1a, 613.1f: Grants a single mana ability; appended to vector.
        // Append-only — preserves original abilities (Chromatic Lantern 2018-10-05 ruling).
        LayerModification::AddManaAbility(ability) => {
            chars.mana_abilities.push_back(ability.clone());
        }
        // Layer 7a: CDAs
        LayerModification::SetPtViaCda { power, toughness } => {
            chars.power = Some(*power);
            chars.toughness = Some(*toughness);
        }
        // Layer 7a: Dynamic CDAs evaluated at layer-calculation time (CR 613.4a).
        LayerModification::SetPtDynamic { power, toughness } => {
            let controller = state
                .objects
                .get(&object_id)
                .map(|o| o.controller)
                .unwrap_or(crate::state::player::PlayerId(0));
            let p = resolve_cda_amount(state, power, object_id, controller);
            let t = resolve_cda_amount(state, toughness, object_id, controller);
            chars.power = Some(p);
            chars.toughness = Some(t);
        }
        LayerModification::SetPtToManaValue => {
            // OOS-SIM2-5 / PB-DX19: `mana_value` is `u32`; bounded in practice by a
            // printed cost, so completeness rather than a live defect -- but it is a
            // u32->i32 widening that writes P/T, the shape this batch leaves none of.
            let mv = i32::try_from(mana_value).unwrap_or(i32::MAX);
            chars.power = Some(mv);
            chars.toughness = Some(mv);
        }
        // Layer 7b: P/T-setting
        LayerModification::SetPowerToughness { power, toughness } => {
            chars.power = Some(*power);
            chars.toughness = Some(*toughness);
        }
        // Layer 7b: residual live-eval SET (spell path substitutes to SetPowerToughness
        // before reaching here; this arm handles any direct/static registration). CR 613.4b.
        LayerModification::SetBothDynamic { amount } => {
            let controller = state
                .objects
                .get(&object_id)
                .map(|o| o.controller)
                .unwrap_or(crate::state::player::PlayerId(0));
            let v = resolve_cda_amount(state, amount, object_id, controller);
            chars.power = Some(v);
            chars.toughness = Some(v);
        }
        // Layer 7c: P/T-modifying
        LayerModification::ModifyPower(delta) => {
            if let Some(p) = &mut chars.power {
                *p = p.saturating_add(*delta);
            }
        }
        LayerModification::ModifyToughness(delta) => {
            if let Some(t) = &mut chars.toughness {
                *t = t.saturating_add(*delta);
            }
        }
        LayerModification::ModifyBoth(delta) => {
            if let Some(p) = &mut chars.power {
                *p = p.saturating_add(*delta);
            }
            if let Some(t) = &mut chars.toughness {
                *t = t.saturating_add(*delta);
            }
        }
        // CR 611.3a / PB-CC-C-followup: ModifyBothDynamic re-evaluates live at every
        // `calculate_characteristics` call so the modifier is never locked in.
        //
        // Both is_cda paths (is_cda=true for static abilities, is_cda=false for residual
        // spell-effect cases) now route through `resolve_cda_amount`. The spell-effect
        // lock-in semantic (CR 608.2h) relies on the substitution arm in `effects/mod.rs`
        // replacing this with a concrete `ModifyBoth(N)` at execute_effect time. If
        // substitution is bypassed for a spell effect (is_cda=false reaching here),
        // behavior degrades to live-eval rather than locked-in — see PB-CC-C T3/T4
        // which document this residual path as intentional non-panic behavior.
        //
        // Note: `AbilityDefinition::CdaModifyPowerToughness` with both axes Some now
        // registers two separate ModifyPowerDynamic + ModifyToughnessDynamic effects
        // instead of one ModifyBothDynamic, so this arm is only reached from the
        // spell-effect substitution path or future direct registrations.
        LayerModification::ModifyBothDynamic { amount, negate } => {
            let controller = state
                .objects
                .get(&object_id)
                .map(|o| o.controller)
                .unwrap_or(crate::state::player::PlayerId(0));
            let raw = resolve_cda_amount(state, amount, object_id, controller);
            // OOS-SIM2-5: `-raw` panics under `overflow-checks` and wraps otherwise at
            // `i32::MIN`; `saturating_neg` is total.
            let delta = if *negate { raw.saturating_neg() } else { raw };
            if let Some(p) = &mut chars.power {
                *p = p.saturating_add(delta);
            }
            if let Some(t) = &mut chars.toughness {
                *t = t.saturating_add(delta);
            }
        }
        // CR 611.3a / PB-CC-C-followup: ModifyPowerDynamic stored with `is_cda: true` —
        // re-evaluate live so power modifier tracks the dynamic quantity continuously.
        LayerModification::ModifyPowerDynamic { amount, negate } => {
            let controller = state
                .objects
                .get(&object_id)
                .map(|o| o.controller)
                .unwrap_or(crate::state::player::PlayerId(0));
            let raw = resolve_cda_amount(state, amount, object_id, controller);
            // OOS-SIM2-5: see the `ModifyBothDynamic` arm above.
            let delta = if *negate { raw.saturating_neg() } else { raw };
            if let Some(p) = &mut chars.power {
                *p = p.saturating_add(delta);
            }
        }
        // CR 611.3a / PB-CC-C-followup: ModifyToughnessDynamic stored with `is_cda: true` —
        // re-evaluate live so toughness modifier tracks the dynamic quantity continuously.
        LayerModification::ModifyToughnessDynamic { amount, negate } => {
            let controller = state
                .objects
                .get(&object_id)
                .map(|o| o.controller)
                .unwrap_or(crate::state::player::PlayerId(0));
            let raw = resolve_cda_amount(state, amount, object_id, controller);
            // OOS-SIM2-5: see the `ModifyBothDynamic` arm above.
            let delta = if *negate { raw.saturating_neg() } else { raw };
            if let Some(t) = &mut chars.toughness {
                *t = t.saturating_add(delta);
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
    //
    // NOTE (F-VR1, PB-AC7 card review): `sort_by_key` is a STABLE sort — for two
    // effects with an EQUAL timestamp (which happens whenever multiple
    // `ApplyContinuousEffect`s are executed from one `Effect::Sequence` within a
    // single resolution; see the `ts`-not-advanced note in
    // `effects/mod.rs::execute_effect_inner`), this relies on stability to
    // preserve the effects' original push/vec order as the tiebreak. Some card
    // defs depend on this for correctness when there is no explicit `depends_on`
    // dependency edge between the two effects below — e.g. Vraska, Betrayal's
    // Sting's -2 pushes `RemoveAllAbilities` before the granted `AddManaAbility`
    // so the grant survives the removal at the SAME timestamp (regression-guarded
    // by `test_vraska_betrayals_sting_minus2_full_integration` in
    // `crates/engine/tests/pb_ac7_card_integration.rs`). Do not replace this with
    // an unstable sort.
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
    //
    // SR-30: this branch is UNREACHABLE under the engine's current `depends_on`
    // relation. Every dependency arm there is Layer 4 and has the shape
    // "`Set*` depends on `Add*`/`Set*`": only `SetTypeLine`/`SetCardTypes`/
    // `SetCreatureTypes`/`SetLandTypes` (OOS-ADJ-7, PB-DX27 rider) ever appear as
    // the dependent (`a`), and the only `Set` that appears as a dependency (`b`)
    // is `SetCreatureTypes` (in the `SetCardTypes → SetCreatureTypes` arm), which
    // itself depends only on `AddSubtypes`. `SetLandTypes` likewise depends only
    // on `AddSubtypes`, never appears as a `b`. `Add*` effects never depend on
    // anything, so no directed cycle can form. The
    // `no_dependency_cycle_is_constructible_from_current_relation`
    // unit test below guards that premise: if a future arm makes the relation
    // symmetric, it fails and this `debug_assert` fires, forcing a real 613.8b
    // cycle test to be written. The release-build fallback below (emit the
    // remaining effects in timestamp order) keeps the engine correct and
    // loop-free per CR 613.8b even if that ever happens.
    if result.len() < n {
        debug_assert!(
            false,
            "CR 613.8b dependency-loop fallback reached, but no cycle is \
             constructible under the current Layer-4-only `depends_on` relation. \
             A new dependency arm has introduced a cycle: write a real 613.8b \
             cycle test and revisit this assertion (SR-30)."
        );
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
        // PB-AC7 review fix (M1, CR 613.8a): `SetCreatureTypes` depends on a
        // co-resident `AddSubtypes` IFF the added subtypes include at least one
        // CREATURE type. `SetCreatureTypes` unconditionally replaces the ENTIRE
        // creature-type subset of `subtypes` with its own payload, discarding any
        // prior creature-type subtype regardless of identity. So:
        // - `AddSubtypes` applied BEFORE `SetCreatureTypes`: the added creature type
        //   gets wiped along with everything else in the creature-type subset —
        //   `SetCreatureTypes`'s own payload is the final result.
        // - `AddSubtypes` applied AFTER (no dependency, natural timestamp order):
        //   the added creature type survives unconditionally, giving a UNION of
        //   `SetCreatureTypes`'s payload and the added type — order-dependent,
        //   hence a genuine CR 613.8a dependency (applying B changes what A does).
        // A land/artifact/enchantment-only `AddSubtypes` never touches the
        // creature-type subset either way — no dependency needed for that case (the
        // outcome is identical regardless of order), so the check is payload-aware
        // rather than a blanket dependency (avoids a spurious/no-op dependency arm).
        // Locked in by `test_set_creature_types_layer4_dependency_with_add_subtypes`
        // (disjoint land-subtype case, no dependency needed, order-independent) and
        // `test_set_creature_types_layer4_dependency_nondisjoint_creature_subtype`
        // (Zombie counterexample, both orders now converge because of this arm).
        (LayerModification::SetCreatureTypes(_), LayerModification::AddSubtypes(added)) => added
            .iter()
            .any(|st| crate::state::types::ALL_CREATURE_TYPES.contains(st)),
        // OOS-ADJ-7 (PB-DX27 rider): `SetLandTypes` depends on a co-resident
        // `AddSubtypes` IFF the added subtypes include at least one LAND type — the
        // same payload-aware rule as the `SetCreatureTypes`/`AddSubtypes` arm above,
        // now keyed off `ALL_LAND_TYPES`. This is the Blood Moon + Urborg dependency,
        // re-derived for `SetLandTypes` now that Blood Moon/Magus of the Moon no
        // longer use `SetTypeLine` (see the `(SetTypeLine, AddSubtypes)` arm above,
        // which this replaces for those two cards): Urborg's `AddSubtypes(Swamp)`
        // must apply BEFORE Blood Moon's `SetLandTypes(Mountain)` so Blood Moon's
        // "SET" wins (result: Mountain, not Mountain+Swamp). Locked in by
        // `t7_blood_moon_still_overrides_urborg_dependency` in
        // `pb_dx27_blood_moon_type_scope.rs`.
        (LayerModification::SetLandTypes(_), LayerModification::AddSubtypes(added)) => added
            .iter()
            .any(|st| crate::state::types::ALL_LAND_TYPES.contains(st)),
        // PB-AC7 review fix (M1, "additional coupling to H1"): once `SetCardTypes`
        // reads `chars.card_types` to decide which subtypes survive the CR 205.1a
        // correlated-subtype-removal clause, its OWN action (the subtype-filter it
        // performs) becomes order-sensitive against effects that change either
        // `card_types` or `subtypes` on the same object. Three dependency arms,
        // each independently justified against the CR 613.8a test ("applying B
        // changes what A does"):
        //
        // 1. `SetCardTypes` unconditionally OVERWRITES `card_types` (same rationale
        //    as the `SetTypeLine`/`AddCardTypes` precedent above — the "set" must
        //    win over a co-resident "add" regardless of which named types are
        //    involved, so this one mirrors that precedent unconditionally).
        (LayerModification::SetCardTypes(_), LayerModification::AddCardTypes(_)) => true,
        // 2. `SetCardTypes`'s subtype-filter reads whatever `subtypes` currently
        //    holds at its own application time. If `AddSubtypes` applies AFTER
        //    `SetCardTypes`, the added subtype bypasses the correlation filter
        //    entirely and survives even if its correlated card type was just
        //    removed — wrong per CR 205.1a. Dependency exists only when the added
        //    subtype's correlated card type(s) are NOT in `SetCardTypes`'s new card
        //    types (only then would the filter actually drop it) — payload-aware to
        //    avoid a spurious dependency for an added subtype that would survive the
        //    filter regardless (e.g. adding an Elf subtype alongside a SetCardTypes
        //    that keeps Creature).
        (
            LayerModification::SetCardTypes(new_card_types),
            LayerModification::AddSubtypes(added),
        ) => added.iter().any(|st| {
            let correlated = crate::state::types::correlated_card_types(st);
            !correlated.is_empty() && !correlated.iter().any(|ct| new_card_types.contains(ct))
        }),
        // 3. `SetCreatureTypes` always draws its payload from the creature-type set
        //    (correlated card type: `Creature`). If `SetCardTypes` removes Creature
        //    from the type line, a `SetCreatureTypes` applied AFTER it would
        //    unconditionally re-add a creature-type subtype the object should no
        //    longer have — order matters only in that specific case. When
        //    `SetCardTypes`'s new types still include Creature (true for every
        //    roster card this batch — the "becomes an X creature" effects always
        //    retain Creature), there is no dependency: `SetCreatureTypes`'s output
        //    always survives the correlation filter regardless of order, so no
        //    reordering is forced on the current roster's timestamp-natural order
        //    (`SetCardTypes` listed before `SetCreatureTypes` in each card's ability
        //    vec — verified consistent either way, see pb-review-AC7.md fix notes).
        (
            LayerModification::SetCardTypes(new_card_types),
            LayerModification::SetCreatureTypes(_),
        ) => !new_card_types.contains(&CardType::Creature),
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
    // Collect the objects of any expiring Layer-2 SetController effect BEFORE the
    // reassignment below so control can be reverted (CR 613.7) once the effect is
    // actually gone (see `recompute_object_controller`'s doc comment -- calling it
    // before removal would re-observe the effect and no-op the revert).
    let reverted: Vec<ObjectId> = state
        .continuous_effects
        .iter()
        .filter(|e| {
            e.duration == EffectDuration::UntilEndOfTurn
                && e.layer == EffectLayer::Control
                && matches!(e.modification, LayerModification::SetController(_))
        })
        .filter_map(|e| match e.filter {
            EffectFilter::SingleObject(id) => Some(id),
            _ => None,
        })
        .collect();
    let keep: imbl::Vector<ContinuousEffect> = state
        .continuous_effects
        .iter()
        .filter(|e| e.duration != EffectDuration::UntilEndOfTurn)
        .cloned()
        .collect();
    state.continuous_effects = keep;
    // CR 514.2/613.7: revert control of any object whose UntilEndOfTurn SetController
    // effect just expired. Must run after the reassignment above.
    for id in reverted {
        recompute_object_controller(state, id);
    }
    // Expire UntilEndOfTurn replacement effects (CR 514.2).
    // Collect IDs to remove first so we can also clean up prevention_counters.
    let expired_ids: Vec<ReplacementId> = state
        .replacement_effects
        .iter()
        .filter(|e| e.duration == EffectDuration::UntilEndOfTurn)
        .map(|e| e.id)
        .collect();
    if !expired_ids.is_empty() {
        let keep_replacements: imbl::Vector<_> = state
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
    // PB-I: Expire UntilEndOfTurn flash grants (CR 514.2).
    let keep_grants: imbl::Vector<crate::state::stubs::FlashGrant> = state
        .flash_grants
        .iter()
        .filter(|g| g.duration != EffectDuration::UntilEndOfTurn)
        .cloned()
        .collect();
    state.flash_grants = keep_grants;
}

/// Expire continuous effects and temporary player protections with
/// `EffectDuration::UntilYourNextTurn(active_player)` at the start of the
/// specified player's turn (CR 611.2b).
///
/// Called from `turn_actions::untap_active_player_permanents` at the start of the
/// untap step, before untapping and phasing. Also resets `abilities_activated_this_turn`
/// on all objects controlled by the active player (CR 602.5b once-per-turn enforcement).
pub fn expire_until_next_turn_effects(state: &mut GameState, active_player: PlayerId) {
    // Expire UntilYourNextTurn continuous effects for this player.
    // Collect the objects of any expiring Layer-2 SetController effect BEFORE the
    // reassignment below so control can be reverted (CR 611.2b/613.7) once the effect
    // is actually gone (see `recompute_object_controller`'s doc comment).
    let reverted: Vec<ObjectId> = state
        .continuous_effects
        .iter()
        .filter(|e| {
            e.duration == EffectDuration::UntilYourNextTurn(active_player)
                && e.layer == EffectLayer::Control
                && matches!(e.modification, LayerModification::SetController(_))
        })
        .filter_map(|e| match e.filter {
            EffectFilter::SingleObject(id) => Some(id),
            _ => None,
        })
        .collect();
    let keep: imbl::Vector<ContinuousEffect> = state
        .continuous_effects
        .iter()
        .filter(|e| e.duration != EffectDuration::UntilYourNextTurn(active_player))
        .cloned()
        .collect();
    state.continuous_effects = keep;
    // CR 611.2b/613.7: revert control of any object whose UntilYourNextTurn
    // SetController effect just expired. Must run after the reassignment above.
    for id in reverted {
        recompute_object_controller(state, id);
    }
    // Expire UntilYourNextTurn replacement effects for this player (CR 611.2b).
    // This fixes the gap where dynamically-registered replacement effects (e.g.
    // Lightning's Stagger) would persist forever without this cleanup.
    let keep_repl: imbl::Vector<crate::state::replacement_effect::ReplacementEffect> = state
        .replacement_effects
        .iter()
        .filter(|e| e.duration != EffectDuration::UntilYourNextTurn(active_player))
        .cloned()
        .collect();
    state.replacement_effects = keep_repl;
    // PB-I: Expire UntilYourNextTurn flash grants for this player (CR 611.2b).
    let keep_grants: imbl::Vector<crate::state::stubs::FlashGrant> = state
        .flash_grants
        .iter()
        .filter(|g| g.duration != EffectDuration::UntilYourNextTurn(active_player))
        .cloned()
        .collect();
    state.flash_grants = keep_grants;
    // Clear temporary protection for the active player.
    if let Some(ps) = state.expect_player_mut(active_player) {
        if !ps.temporary_protection_qualities.is_empty() {
            ps.temporary_protection_qualities.clear();
        }
    }
    // Reset abilities_activated_this_turn on all battlefield objects.
    // CR 602.5b: "Activate only once each turn" resets at the start of each player's turn.
    let ids: Vec<crate::state::game_object::ObjectId> = state
        .objects
        .iter()
        .filter(|(_, obj)| {
            obj.zone == crate::state::ZoneId::Battlefield && obj.abilities_activated_this_turn > 0
        })
        .map(|(id, _)| *id)
        .collect();
    for id in ids {
        if let Some(obj) = state.expect_object_mut(id) {
            obj.abilities_activated_this_turn = 0;
        }
    }
    // CR 603.2c/603.2h (PB-AC1): Reset triggered_abilities_fired_this_turn on all
    // battlefield objects. Same cadence as abilities_activated_this_turn above -- this
    // runs at every untap step, for all objects (not just the active player's), which
    // matches the "once each turn" semantics of "This ability triggers only once each
    // turn" (Morbid Opportunist, etc.).
    let trigger_reset_ids: Vec<crate::state::game_object::ObjectId> = state
        .objects
        .iter()
        .filter(|(_, obj)| {
            obj.zone == crate::state::ZoneId::Battlefield
                && !obj.triggered_abilities_fired_this_turn.is_empty()
        })
        .map(|(id, _)| *id)
        .collect();
    for id in trigger_reset_ids {
        if let Some(obj) = state.expect_object_mut(id) {
            obj.triggered_abilities_fired_this_turn = imbl::OrdSet::new();
        }
    }
}

// ── WhileYouControlSource expiry (PB-EF9) ────────────────────────────────────

/// Expire `EffectDuration::WhileYouControlSource` continuous effects whose creator no
/// longer controls the source permanent, and revert control of the borrowed object
/// (CR 611.2b/c, 702.26e).
///
/// This is a one-shot imperative pass: once an effect is removed here, it is never
/// re-added, so the borrowed permanent never resumes under its borrower even if control
/// of the source later returns (CR 611.2c — "the set of objects [the effect] affects is
/// determined when that continuous effect begins. After that point, the set won't
/// change."). `is_effect_active`'s arm for this duration always returns `true` for
/// exactly this reason: termination is owned solely by this function, never by a live
/// re-evaluation.
///
/// CR 702.26e: a phased-out source is STILL controlled by its controller — this check
/// deliberately does NOT test `is_phased_in()` (unlike the `WhileSourceOnBattlefield`
/// arm of `is_effect_active`, which treats a phased-out source as inactive). A borrower
/// does not lose a stolen permanent just because the source phases out.
///
/// CR 400.7: if the source object no longer resolves (it left the battlefield and is a
/// new object in its new zone), the effect has ended.
///
/// Call-site assumption: control of a source only changes through effect resolution or
/// the source leaving the battlefield, and both paths are immediately followed by
/// `sba::check_and_apply_sbas`. This pass is called once at the top of
/// `check_and_apply_sbas`, before the SBA fixpoint loop, so it observes every
/// post-resolution state. If a future feature ever changes control of a permanent
/// outside a resolution/SBA boundary, this pass would lag behind it.
///
/// No `GameEvent` variant exists today for "control changed"/"control reverted" (grep
/// confirmed), so this function returns nothing rather than `Vec<GameEvent>`. If such an
/// event is ever added, wire it through the caller the same way SBA events are (extend
/// `all_events` and run `abilities::check_triggers` on it).
pub fn expire_while_you_control_source_effects(state: &mut GameState) {
    // Step 1: find ended effects and the objects they affect.
    let mut ended_ids: Vec<crate::state::continuous_effect::EffectId> = Vec::new();
    let mut affected: Vec<ObjectId> = Vec::new();
    for e in state.continuous_effects.iter() {
        let pid = match e.duration {
            EffectDuration::WhileYouControlSource(pid) => pid,
            _ => continue,
        };
        let ended = match e.source {
            Some(src) => state
                .objects
                .get(&src)
                // CR 702.26e: phased-out source is STILL controlled -- do NOT check
                // is_phased_in() here.
                .map(|o| !(o.zone == ZoneId::Battlefield && o.controller == pid))
                .unwrap_or(true), // source object gone (CR 400.7 new id) -> ended
            None => false,
        };
        if ended {
            ended_ids.push(e.id);
            // NOTE (LOW, PB-EF9 review; updated by PB-DX5 fix-cycle Finding 12): only
            // `EffectFilter::SingleObject` reverts control here. Every card authored
            // today (GainControl) always produces a `SingleObject` filter, so this is
            // not a live gap -- but a future `WhileYouControlSource` effect built via
            // `ApplyContinuousEffect` with a broader filter (e.g. `AllPermanents`,
            // `CreaturesYouControl`) would have its effect correctly removed in Step 2
            // below, yet none of the objects it applied to would have their
            // `controller` reverted (Step 3 never sees them). As of PB-DX5,
            // "resolve the broader filter's matching object ids" is no longer
            // re-derivation work: for any RESOLUTION-generated effect, `e.affected_set`
            // (CR 611.2c) already IS that list, computed once at
            // `Effect::ApplyContinuousEffect` and never re-derived. If you add such a
            // card, extend this match to fall back to `e.affected_set` when present.
            // Residual: for a STATIC `WhileYouControlSource` effect, `affected_set` is
            // `None` and the broader filter would still need to be resolved live here
            // (PB-DX5 fix-cycle Finding 5 widens OOS-DX5-1 to cover this site).
            if let EffectFilter::SingleObject(obj_id) = e.filter {
                affected.push(obj_id);
            }
        }
    }
    if ended_ids.is_empty() {
        return;
    }
    // Step 2: PERMANENTLY remove the ended effects (CR 611.2c -- never resumes).
    let keep: imbl::Vector<ContinuousEffect> = state
        .continuous_effects
        .iter()
        .filter(|e| !ended_ids.contains(&e.id))
        .cloned()
        .collect();
    state.continuous_effects = keep;
    // Step 3: revert control of each affected object.
    for obj_id in affected {
        recompute_object_controller(state, obj_id);
    }
}

/// Recompute an object's controller from its owner plus any still-active Layer-2
/// `SetController` continuous effects, applied in timestamp order (CR 613.7).
///
/// This is a minimal single-object Layer-2 application rather than a full layer
/// recalculation. In the common case (no other control effect on this object) it
/// reverts the object to its owner; if the object is under a *second* still-active
/// control effect, this correctly keeps that one instead of blindly snapping to the
/// owner (stacked-control correctness).
///
/// The just-removed `WhileYouControlSource` effect must already be gone from
/// `state.continuous_effects` before this runs (see
/// `expire_while_you_control_source_effects`), so it is excluded here automatically.
///
/// NOTE (PB-DX5 fix-cycle Finding 5, widens OOS-DX5-1): this is CR 611.2c's
/// **controller** half -- the rule names it explicitly ("modifies the
/// characteristics OR CHANGES THE CONTROLLER of any objects") -- and it is
/// matched by `e.filter == EffectFilter::SingleObject(object_id)`, not by
/// `e.affected_set`. Measured: zero occurrences of
/// `LayerModification::SetController` in `crates/card-defs/src/defs` (the sole
/// mention, `captivating_vampire.rs`, is a TODO comment), and the only engine
/// producers (`Effect::GainControl`, `Effect::ExchangeControl`) always build
/// `SingleObject`, so no exposure today. A future mass-filter control-change
/// effect would need this to consult `affected_set` first.
fn recompute_object_controller(state: &mut GameState, object_id: ObjectId) {
    let owner = match state.objects.get(&object_id) {
        Some(o) => o.owner,
        None => return,
    };
    let mut active: Vec<ContinuousEffect> = state
        .continuous_effects
        .iter()
        .filter(|e| {
            e.layer == EffectLayer::Control
                && e.filter == EffectFilter::SingleObject(object_id)
                && matches!(e.modification, LayerModification::SetController(_))
        })
        .filter(|e| is_effect_active(state, e))
        .cloned()
        .collect();
    active.sort_by_key(|e| e.timestamp);
    let mut controller = owner;
    for e in &active {
        if let LayerModification::SetController(p) = e.modification {
            controller = p;
        }
    }
    if let Some(obj) = state.objects.get_mut(&object_id) {
        obj.controller = controller;
    }
}

// ── CDA evaluation helpers (PB-28) ───────────────────────────────────────────

/// Evaluate an `EffectAmount` in CDA context (no `EffectContext` available).
///
/// CR 604.3: CDAs function in all zones. The evaluation uses the source object's
/// controller as the reference player for "you control" semantics.
///
/// Only a subset of `EffectAmount` variants are valid for CDA evaluation:
/// `Fixed`, `PermanentCount`, `CardCount`, `DevotionTo`, `CounterCount`, `Sum`.
/// Variants requiring `EffectContext` (`XValue`, `LastEffectCount`, `LastDiceRoll`) will
/// return 0 with a `debug_assert`.
pub(crate) fn resolve_cda_amount(
    state: &GameState,
    amount: &EffectAmount,
    object_id: ObjectId,
    controller: PlayerId,
) -> i32 {
    match amount {
        EffectAmount::Fixed(n) => *n,
        EffectAmount::PermanentCount {
            filter,
            controller: player_target,
        } => {
            // Resolve PlayerTarget to concrete player IDs using the source controller.
            let players = resolve_cda_player_target(state, player_target, controller);
            state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && players.contains(&obj.controller)
                        && {
                            // NOTE: We deliberately use base characteristics here (not
                            // calculate_characteristics) to avoid recursive CDA evaluation.
                            // CR 604.3: CDA filters typically check card types (Creature, Land)
                            // or subtypes, which are set in Layers 4-6 (not by other CDAs).
                            // This avoids an infinite recursion when the CDA creature itself
                            // is included in the count (e.g., "*/* = creatures you control"
                            // counts the creature with the CDA).
                            crate::effects::matches_filter(&obj.characteristics, filter)
                                // CR 122.1: counter check against GameObject (not Characteristics).
                                && crate::effects::check_has_counter_type(obj, filter)
                        }
                })
                .count() as i32
        }
        EffectAmount::CardCount {
            zone,
            player: _,
            filter,
        } => {
            let zone_id = resolve_cda_zone_target(zone, state, controller);
            state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == zone_id
                        && filter
                            .as_ref()
                            .map(|f| {
                                crate::effects::matches_filter(&obj.characteristics, f)
                                    // CR 122.1: counter check against GameObject (not Characteristics).
                                    && crate::effects::check_has_counter_type(obj, f)
                            })
                            .unwrap_or(true)
                })
                .count() as i32
        }
        EffectAmount::DevotionTo(color) => {
            // CR 700.5: Count mana symbols of that color in permanents controller controls.
            state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && obj.controller == controller
                })
                .map(|obj| {
                    obj.characteristics
                        .mana_cost
                        .as_ref()
                        .map(|mc| {
                            use crate::state::types::Color;
                            match color {
                                Color::White => mc.white as i32,
                                Color::Blue => mc.blue as i32,
                                Color::Black => mc.black as i32,
                                Color::Red => mc.red as i32,
                                Color::Green => mc.green as i32,
                            }
                        })
                        .unwrap_or(0)
                })
                .sum()
        }
        EffectAmount::CounterCount { target, counter } => {
            // For CDA context, target should be EffectTarget::Source (the object itself).
            if matches!(target, EffectTarget::Source) {
                state
                    .objects
                    .get(&object_id)
                    // OOS-SIM2-5 / PB-DX19 review finding: `try_into`, NOT `as i32`.
                    // Unlike this function's `.count()` arms, this one is NOT bounded:
                    // `counters` is `OrdMap<CounterType, u32>` and the value flows into
                    // the `SetPtDynamic` / `Modify*Dynamic` P/T writes. An `as` cast is
                    // not checked arithmetic even under `overflow-checks`, so a count
                    // above `i32::MAX` would wrap to a NEGATIVE power in every profile.
                    .and_then(|obj| obj.counters.get(counter).copied())
                    .unwrap_or(0)
                    .try_into()
                    .unwrap_or(i32::MAX)
            } else {
                debug_assert!(false, "CDA CounterCount with non-Source target");
                0
            }
        }
        // PB-CC-A: CDA evaluation for counters on a player (poison-counter scaling).
        // Mirrors `resolve_amount` PlayerCounterCount arm. Reads `PlayerState`
        // fields, which are NOT layer-derived characteristics (CR 122 / 613) — no
        // layer recursion concern.
        //
        // Sum semantic: `PlayerTarget::EachOpponent` sums over every non-controller
        // (CR 122.1, Vishgraz ruling 2023-02-04). `Poison` reads
        // `PlayerState::poison_counters`; other kinds return 0 (no panic).
        EffectAmount::PlayerCounterCount { player, counter } => {
            let players = resolve_cda_player_target(state, player, controller);
            match counter {
                crate::state::types::CounterType::Poison => players
                    .iter()
                    .filter_map(|pid| state.expect_player(*pid))
                    // OOS-SIM2-5 / PB-DX19: saturating widen + saturating fold.
                    .map(|ps| i32::try_from(ps.poison_counters).unwrap_or(i32::MAX))
                    .fold(0i32, |acc, n| acc.saturating_add(n)),
                _ => 0,
            }
        }
        // PB-28: Sum of two amounts (e.g. "Elves you control plus Elf cards in graveyard").
        EffectAmount::Sum(a, b) => {
            // OOS-SIM2-5 / PB-DX19: saturating -- this value reaches a P/T write.
            resolve_cda_amount(state, a, object_id, controller)
                .saturating_add(resolve_cda_amount(state, b, object_id, controller))
        }
        // PB-L: Domain count — number of distinct basic land types among lands the
        // controller controls. Uses base characteristics (avoids recursion; land types
        // are set by Layer 4 effects like Dryad, not by CDAs).
        // CR 305.6 / ability word "Domain".
        // Limitation: Layer 4 type-changing effects (Blood Moon, Dryad) are not reflected
        // here because resolve_cda_amount runs inside the layer loop. The resolve_amount
        // path (effects/mod.rs) does use calculate_characteristics().
        // The `player` field is ignored in the CDA context — CDAs always reference the
        // controller (the permanent's controller at the time of evaluation).
        EffectAmount::DomainCount { .. } => {
            let basic_land_subtypes = [
                crate::state::types::SubType("Plains".to_string()),
                crate::state::types::SubType("Island".to_string()),
                crate::state::types::SubType("Swamp".to_string()),
                crate::state::types::SubType("Mountain".to_string()),
                crate::state::types::SubType("Forest".to_string()),
            ];
            let mut count = 0i32;
            for sub in &basic_land_subtypes {
                let has_it = state.objects.values().any(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && obj.controller == controller
                        && obj.characteristics.card_types.contains(&CardType::Land)
                        && obj.characteristics.subtypes.contains(sub)
                });
                if has_it {
                    count += 1;
                }
            }
            count
        }
        // CR 613: CDAs cannot reference LKI — they evaluate continuously while the source
        // is on the battlefield, where counters are live. Returns 0 as defensive default.
        // Card authors should not pair CounterCountAtLastKnownInformation with a CDA;
        // use the live `CounterCount` variant instead.
        EffectAmount::CounterCountAtLastKnownInformation { .. } => 0,
        // CR 613: CDAs cannot reference LKI source power — power is a live characteristic
        // while on the battlefield. Returns 0 as defensive default.
        // Card authors should not pair SourcePowerAtLastKnownInformation with a CDA.
        EffectAmount::SourcePowerAtLastKnownInformation => 0,
        // PB-AC3 (discriminant 19, LOCKSTEP with resolve_amount): CR 508.1/509. Combat/tap
        // state are not layer-derived (CR 122/613), so reading `state.combat.attackers`
        // here introduces no CDA recursion. Filter matching uses BASE characteristics
        // (mirrors the existing `PermanentCount` CDA arm) to avoid layer recursion.
        EffectAmount::AttackingCreatureCount {
            controller: pt,
            filter,
        } => {
            let players = resolve_cda_player_target(state, pt, controller);
            let Some(combat) = state.combat.as_ref() else {
                return 0;
            };
            state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && players.contains(&obj.controller)
                        && combat.is_attacking(obj.id)
                        && filter
                            .as_ref()
                            .map(|f| {
                                crate::effects::matches_filter(&obj.characteristics, f)
                                    && crate::effects::check_has_counter_type(obj, f)
                                    && (!f.exclude_self || obj.id != object_id)
                            })
                            .unwrap_or(true)
                })
                .count() as i32
        }
        // PB-AC3 (discriminant 20, LOCKSTEP with resolve_amount): tapped status lives on
        // `GameObject.status`, not layer-derived — no CDA recursion. Base characteristics
        // for filter matching (mirrors `PermanentCount` CDA arm).
        EffectAmount::TappedCreatureCount {
            controller: pt,
            filter,
        } => {
            let players = resolve_cda_player_target(state, pt, controller);
            state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && players.contains(&obj.controller)
                        && obj.status.tapped
                        && obj.characteristics.card_types.contains(&CardType::Creature)
                        && filter
                            .as_ref()
                            .map(|f| {
                                crate::effects::matches_filter(&obj.characteristics, f)
                                    && crate::effects::check_has_counter_type(obj, f)
                                    && (!f.exclude_self || obj.id != object_id)
                            })
                            .unwrap_or(true)
                })
                .count() as i32
        }
        // PB-AC3 (discriminant 21): convenience alias for
        // `CardCount { zone: Hand{owner: player}, .. }` — delegates to the identical
        // CardCount CDA evaluation (see doc-comment on `EffectAmount::HandSize`).
        EffectAmount::HandSize { player } => resolve_cda_amount(
            state,
            &EffectAmount::CardCount {
                zone: ZoneTarget::Hand {
                    owner: player.clone(),
                },
                player: PlayerTarget::Controller,
                filter: None,
            },
            object_id,
            controller,
        ),
        // PB-OS5 (discriminant 24): CR 613 — resolving `relative_to` (the triggering
        // creature) requires `EffectContext`, which is absent here. This variant is
        // only ever used via the spell-effect ApplyContinuousEffect -> ModifyPowerDynamic
        // substitution (`resolve_amount`, value locked in at resolution per CR 608.2h/
        // 107.3f) and is never stored in a Layer-7a CDA. Returns 0 defensively (mirrors
        // `CounterCountAtLastKnownInformation` / `SourcePowerAtLastKnownInformation` above).
        EffectAmount::OtherAttackersSharingCreatureType { .. } => 0,
        _ => {
            debug_assert!(
                false,
                "EffectAmount variant not valid in CDA context: {:?}",
                amount
            );
            0
        }
    }
}

/// Resolve a `PlayerTarget` in CDA context (no `EffectContext`).
///
/// CR 800.4i: when a player leaves a multiplayer game, their permanents are
/// removed from the game (including this CDA's source). However, the layer
/// system processes objects regardless of zone (CR 604.3), so a CDA evaluation
/// can still observe `state.turn.turn_order` entries for already-lost players
/// (the turn order is updated independently of CDA evaluation timing).
///
/// Divergence from `resolve_player_target_list` (effects/mod.rs): that function
/// filters out `PlayerState::has_lost` players because spell-effect target
/// resolution is illegal for lost players (CR 800.4i: "any objects targeted
/// or chosen by a controlled spell or ability ... that became illegal targets
/// are no longer chosen"). CDA evaluation does NOT target — it reads source
/// characteristics — so filtering is unnecessary at this layer. A future
/// primitive that needs lost-player filtering during CDA evaluation should
/// add it explicitly here with a CR citation.
fn resolve_cda_player_target(
    state: &GameState,
    target: &PlayerTarget,
    controller: PlayerId,
) -> Vec<PlayerId> {
    match target {
        PlayerTarget::Controller => vec![controller],
        PlayerTarget::EachPlayer => state.turn.turn_order.iter().copied().collect(),
        PlayerTarget::EachOpponent => state
            .turn
            .turn_order
            .iter()
            .copied()
            .filter(|&p| p != controller)
            .collect(),
        // Fallback: treat other variants as controller for CDA purposes.
        _ => vec![controller],
    }
}

/// Resolve a `ZoneTarget` to a `ZoneId` in CDA context (no `EffectContext`).
fn resolve_cda_zone_target(zone: &ZoneTarget, state: &GameState, controller: PlayerId) -> ZoneId {
    let resolve_owner = |owner: &PlayerTarget| -> PlayerId {
        resolve_cda_player_target(state, owner, controller)
            .into_iter()
            .next()
            .unwrap_or(controller)
    };
    match zone {
        ZoneTarget::Hand { owner } => ZoneId::Hand(resolve_owner(owner)),
        ZoneTarget::Graveyard { owner } => ZoneId::Graveyard(resolve_owner(owner)),
        ZoneTarget::Library { owner, .. } => ZoneId::Library(resolve_owner(owner)),
        ZoneTarget::Battlefield { .. } => ZoneId::Battlefield,
        ZoneTarget::Exile => ZoneId::Exile,
        ZoneTarget::CommandZone => ZoneId::Command(controller),
    }
}

#[cfg(test)]
mod expect_characteristics_tests {
    use super::*;
    use crate::state::{GameStateBuilder, ObjectSpec, PlayerId, ZoneId};

    fn state_with_a_creature() -> (GameState, ObjectId) {
        let state = GameStateBuilder::new()
            .add_player(PlayerId(0))
            .add_player(PlayerId(1))
            .object(
                ObjectSpec::creature(PlayerId(0), "Grizzly Bears", 2, 2)
                    .in_zone(ZoneId::Battlefield),
            )
            .build()
            .expect("builder is valid");
        let id = state
            .objects
            .iter()
            .find(|(_, o)| o.zone == ZoneId::Battlefield)
            .map(|(id, _)| *id)
            .expect("the creature was placed");
        (state, id)
    }

    #[test]
    fn expect_characteristics_returns_the_layer_result_for_a_live_object() {
        let (state, id) = state_with_a_creature();
        let chars = expect_characteristics(&state, id);
        assert_eq!(chars.power, Some(2));
        assert_eq!(chars.toughness, Some(2));
    }

    /// `calculate_characteristics` returns `None` for exactly one reason: the id is
    /// absent. So `expect_characteristics` at a site that guarantees liveness must be
    /// loud rather than quietly handing back a blank `Characteristics` — which is how
    /// `combat.rs`'s landwalk check would have silently decided a Forest is not a land.
    #[test]
    #[should_panic(expected = "requires the object to be live")]
    fn expect_characteristics_panics_in_debug_on_a_dead_id() {
        let (mut state, old_id) = state_with_a_creature();
        state
            .move_object_to_zone(old_id, ZoneId::Graveyard(PlayerId(0)))
            .expect("legal move");
        // CR 400.7: `old_id` names nothing now.
        let _ = expect_characteristics(&state, old_id);
    }

    /// The fizzle path stays available and stays silent.
    #[test]
    fn calculate_characteristics_returns_none_for_a_dead_id_without_panicking() {
        let (mut state, old_id) = state_with_a_creature();
        state
            .move_object_to_zone(old_id, ZoneId::Graveyard(PlayerId(0)))
            .expect("legal move");
        assert!(calculate_characteristics(&state, old_id).is_none());
    }
}

/// SR-30: guards the premise of the `debug_assert` in the CR 613.8b cycle-fallback
/// branch of [`toposort_with_timestamp_fallback`] — that no dependency cycle is
/// constructible under the engine's current `depends_on` relation.
///
/// The engine only approximates CR 613.8 statically: every `depends_on` arm is in
/// Layer 4 and has the shape "`Set*` depends on `Add*`/`Set*`". Only `Set*` effects
/// are ever the dependent; `Add*` effects never depend on anything. If a future
/// arm ever makes the relation symmetric (a real dependency loop), these tests
/// fail — pointing the author at the fallback branch that must then be exercised
/// for real rather than left as dead code.
#[cfg(test)]
mod dependency_cycle_guard_tests {
    use super::*;
    use crate::state::continuous_effect::EffectId;

    fn card_types(ts: &[CardType]) -> OrdSet<CardType> {
        ts.iter().cloned().collect()
    }
    fn subtypes(ts: &[&str]) -> OrdSet<SubType> {
        ts.iter().map(|s| SubType(s.to_string())).collect()
    }

    /// A representative modification for every variant that participates in a
    /// `depends_on` arm (both as dependent and as dependency), chosen so each arm
    /// can actually fire (e.g. `AddSubtypes` includes a creature type; `SetCardTypes`
    /// omits Creature so the `SetCardTypes → SetCreatureTypes` arm fires).
    fn representative_modifications() -> Vec<LayerModification> {
        vec![
            LayerModification::SetTypeLine {
                supertypes: OrdSet::new(),
                card_types: card_types(&[CardType::Land]),
                subtypes: subtypes(&["Mountain"]),
            },
            LayerModification::AddCardTypes(card_types(&[CardType::Creature])),
            LayerModification::AddSubtypes(subtypes(&["Zombie"])), // Zombie is a creature type
            LayerModification::SetCreatureTypes(subtypes(&["Zombie"])),
            // Omits Creature → `SetCardTypes → SetCreatureTypes` dependency fires.
            LayerModification::SetCardTypes(card_types(&[CardType::Artifact])),
            // OOS-ADJ-7 (PB-DX27 rider): Swamp is a land type, so this fires the
            // `SetLandTypes → AddSubtypes` arm (and, unconditionally, the
            // `SetTypeLine → AddSubtypes` arm above already covers it too).
            LayerModification::AddSubtypes(subtypes(&["Swamp"])),
            LayerModification::SetLandTypes(subtypes(&["Mountain"])),
        ]
    }

    fn effect(id: u64, m: LayerModification) -> ContinuousEffect {
        ContinuousEffect {
            id: EffectId(id),
            source: None,
            timestamp: id,
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::WhileSourceOnBattlefield,
            filter: EffectFilter::AllPermanents,
            modification: m,
            is_cda: false,
            affected_set: None,
            condition: None,
        }
    }

    /// No ordered pair of the representative modifications depends on each other in
    /// both directions — i.e. the relation contains no 2-cycle.
    #[test]
    fn no_dependency_cycle_is_constructible_from_current_relation() {
        let effects: Vec<ContinuousEffect> = representative_modifications()
            .into_iter()
            .enumerate()
            .map(|(i, m)| effect(i as u64 + 1, m))
            .collect();
        for a in &effects {
            for b in &effects {
                if a.id == b.id {
                    continue;
                }
                assert!(
                    !(depends_on(a, b) && depends_on(b, a)),
                    "symmetric dependency (2-cycle) found between {:?} and {:?} — \
                     the CR 613.8b cycle-fallback branch is now reachable and its \
                     debug_assert must be replaced with a real cycle test",
                    a.modification,
                    b.modification,
                );
            }
        }
    }

    /// Feeding every dependency-participating modification through the real
    /// toposort at once produces a complete ordering (no effect dropped), which is
    /// exactly the condition under which the 613.8b fallback branch is NOT taken.
    /// This exercises the whole `toposort_with_timestamp_fallback` path (including
    /// its Kahn's-algorithm dependency edges) without ever entering — and thus
    /// tripping the `debug_assert` in — the cycle branch.
    #[test]
    fn toposort_over_all_dependency_participants_emits_every_effect() {
        let effects: Vec<ContinuousEffect> = representative_modifications()
            .into_iter()
            .enumerate()
            .map(|(i, m)| effect(i as u64 + 1, m))
            .collect();
        let refs: Vec<&ContinuousEffect> = effects.iter().collect();
        let n = refs.len();
        let ordered = toposort_with_timestamp_fallback(refs);
        assert_eq!(
            ordered.len(),
            n,
            "toposort must emit every effect (acyclic); a shortfall means a cycle \
             was hit and the 613.8b fallback branch was taken"
        );
    }
}

/// PB-DX39 (`OOS-DX5-3`, `OOS-DX5-7`): the CR 608.2h / CR 611.3a split, exercised where
/// it can be exercised.
///
/// `snapshot_affected_set`, `effect_applies_to` and both `source_view_*` constructors are
/// `pub(crate)` or private, so none of this is reachable from the `crates/engine/tests/`
/// integration crate -- hence an in-source unit module, mirroring the
/// `pb_dx5_snapshot_tests` precedent a few hundred lines above. End-to-end drives of the
/// two subject cards live in the integration suite; what is pinned here is the predicate
/// itself, in both directions.
#[cfg(test)]
mod pb_dx39_source_view_tests {
    use super::*;
    use crate::state::continuous_effect::EffectId;
    use crate::state::{GameStateBuilder, ObjectSpec, PlayerId, ZoneId};

    fn find(state: &GameState, name: &str) -> ObjectId {
        state
            .objects
            .iter()
            .find(|(_, o)| o.characteristics.name == name)
            .map(|(id, _)| *id)
            .unwrap_or_else(|| panic!("'{name}' not found"))
    }
    /// A resolution-generated effect (`affected_set` is populated by
    /// `snapshot_affected_set`, CR 611.2c) with the given source and filter.
    fn eff(source: Option<ObjectId>, filter: EffectFilter) -> ContinuousEffect {
        ContinuousEffect {
            id: EffectId(1),
            source,
            timestamp: 1,
            layer: EffectLayer::PtModify,
            duration: EffectDuration::UntilEndOfTurn,
            filter,
            modification: LayerModification::ModifyBoth(1),
            is_cda: false,
            affected_set: None,
            condition: None,
        }
    }
    /// P1 controls "Bearer" and "Other"; P2 controls "Enemy"; the Equipment "Jitte" is
    /// attached to Bearer.
    fn board() -> GameState {
        let mut state = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .object(ObjectSpec::creature(PlayerId(1), "Bearer", 2, 2))
            .object(ObjectSpec::creature(PlayerId(1), "Other", 3, 3))
            .object(ObjectSpec::creature(PlayerId(2), "Enemy", 1, 1))
            .object(ObjectSpec::artifact(PlayerId(1), "Jitte"))
            .build()
            .expect("fixture builds");
        let jitte = find(&state, "Jitte");
        let bearer = find(&state, "Bearer");
        if let Some(o) = state.expect_object_mut(jitte) {
            o.attached_to = Some(bearer);
        }
        state
    }

    /// CR 608.2h: *"the effect uses the current information of that object if it's in the
    /// public zone it was expected to be in; if it's no longer in that zone ... the effect
    /// uses the object's last known information."* Umezawa's Jitte ruling 2005-02-01: the
    /// bonus goes to *"the creature that was most recently equipped"*.
    ///
    /// RED before PB-DX39, and red again under a revert that drops the
    /// `lki_object_snapshot` fallback from `source_view_at_resolution` (executed).
    #[test]
    fn attached_creature_survives_the_sources_departure() {
        let mut state = board();
        let jitte = find(&state, "Jitte");
        let bearer = find(&state, "Bearer");
        let e = eff(Some(jitte), EffectFilter::AttachedCreature);

        // Non-vacuity floor: the live source really does lock the bearer, so a later
        // failure cannot be "the fixture never matched anything".
        assert_eq!(
            snapshot_affected_set(&state, &e),
            OrdSet::unit(bearer),
            "precondition: a live attached source locks its bearer"
        );

        state.capture_source_lki_for_pending_ability(jitte);
        state
            .move_object_to_zone(jitte, ZoneId::Graveyard(PlayerId(1)))
            .expect("battlefield -> graveyard is legal");
        assert!(
            state.lki_object_snapshot(jitte).is_some(),
            "PB-DX39 clause B stored the source's last known information"
        );
        assert_eq!(
            snapshot_affected_set(&state, &e),
            OrdSet::unit(bearer),
            "CR 608.2h: the locked set is the most recently equipped creature"
        );
    }

    /// CR 611.3a: a static ability's effect *"isn't 'locked in'"* and exists only while
    /// the ability does. The LIVE path must therefore NOT consult last known information,
    /// even when a snapshot exists -- otherwise a departed permanent's static ability
    /// would run for the rest of the game.
    #[test]
    fn the_live_static_path_never_consults_last_known_information() {
        let mut state = board();
        let jitte = find(&state, "Jitte");
        let bearer = find(&state, "Bearer");
        let e = eff(Some(jitte), EffectFilter::AttachedCreature);
        state.capture_source_lki_for_pending_ability(jitte);
        state
            .move_object_to_zone(jitte, ZoneId::Graveyard(PlayerId(1)))
            .expect("battlefield -> graveyard is legal");
        // Non-vacuity floor: a snapshot really is available, so "false" below is a
        // decision and not an absence.
        assert!(state.lki_object_snapshot(jitte).is_some());
        let chars = expect_characteristics(&state, bearer);
        assert!(
            !effect_applies_to(&state, &e, bearer, ZoneId::Battlefield, &chars),
            "CR 611.3a: the live path is live-only"
        );
    }

    /// Umezawa's Jitte, same ruling block: *"Choosing the '+2/+2' mode does nothing if the
    /// Jitte isn't equipped to a creature when the ability resolves."* Losing the bonus is
    /// sometimes LEGAL, and the fix must not degenerate into "match something if the set
    /// came out empty".
    #[test]
    fn a_live_but_unattached_source_still_matches_nothing() {
        let mut state = board();
        let jitte = find(&state, "Jitte");
        if let Some(o) = state.expect_object_mut(jitte) {
            o.attached_to = None;
        }
        let e = eff(Some(jitte), EffectFilter::AttachedCreature);
        assert!(
            snapshot_affected_set(&state, &e).is_empty(),
            "unattached at resolution legally does nothing"
        );
    }

    /// `OOS-DX5-7`: Mardu Ascendancy's `Cost::SacrificeSelf` means the source is ALWAYS
    /// gone at resolution, so `EffectFilter::CreaturesYouControl` applied to nobody in
    /// every game. Also pins the controller axis wrong-way-round: the opponent's creature
    /// must NOT join the set.
    #[test]
    fn creatures_you_control_survives_a_sacrifice_self_cost() {
        let mut state = board();
        let source = find(&state, "Jitte");
        let bearer = find(&state, "Bearer");
        let other = find(&state, "Other");
        let enemy = find(&state, "Enemy");
        let e = eff(Some(source), EffectFilter::CreaturesYouControl);
        state.capture_source_lki_for_pending_ability(source);
        state
            .move_object_to_zone(source, ZoneId::Graveyard(PlayerId(1)))
            .expect("battlefield -> graveyard is legal");
        let set = snapshot_affected_set(&state, &e);
        assert!(
            set.contains(&bearer) && set.contains(&other),
            "both of P1's creatures"
        );
        assert!(!set.contains(&enemy), "P2's creature is not 'you control'");

        // The inequality axis, from the same departed source.
        let opp = eff(Some(source), EffectFilter::CreaturesOpponentsControl);
        assert_eq!(snapshot_affected_set(&state, &opp), OrdSet::unit(enemy));
    }

    /// SR-24 is intact: a departing permanent with none of the four damage keywords AND
    /// nothing of its own pending stores no snapshot, so the board-wipe optimisation still
    /// holds and the locked set is still empty. This is the control that proves PB-DX39's
    /// capture widening is a disjunct and not a blanket.
    #[test]
    fn no_pending_ability_and_no_keyword_means_no_snapshot_and_no_set() {
        let mut state = board();
        let jitte = find(&state, "Jitte");
        let e = eff(Some(jitte), EffectFilter::AttachedCreature);
        state
            .move_object_to_zone(jitte, ZoneId::Graveyard(PlayerId(1)))
            .expect("battlefield -> graveyard is legal");
        assert!(
            state.lki_object_snapshot(jitte).is_none(),
            "SR-24: keyword-less, nothing pending -> not captured"
        );
        assert!(snapshot_affected_set(&state, &e).is_empty());
    }
}
