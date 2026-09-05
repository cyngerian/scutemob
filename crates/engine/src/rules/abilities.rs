//! Activated and triggered ability handling (CR 602-603).
//!
//! ## Activated abilities (CR 602)
//!
//! Activated abilities are written as "Cost: Effect." They are NOT mana abilities
//! (those are handled in `rules/mana.rs`). Activating puts a `StackObject` on
//! the stack. The active player receives priority afterward.
//!
//! ## Triggered abilities (CR 603)
//!
//! Triggered abilities begin with "when", "whenever", or "at". When a trigger
//! condition is met:
//! 1. The ability goes into `GameState::pending_triggers`.
//! 2. The next time a player would receive priority, pending triggers are flushed
//!    to the stack in APNAP order (CR 603.3).
//!
//! **Intervening-if (CR 603.4)**: If the ability reads "... if [condition] ...",
//! the condition is checked at trigger time (ability only queues if true) AND at
//! resolution time (ability has no effect if condition became false).
use super::casting;
use super::events::{CombatDamageTarget, GameEvent};
use crate::cards::card_definition::{AbilityDefinition, TargetController, TriggerCondition};
use crate::state::error::GameStateError;
use crate::state::game_object::{InterveningIf, ManaCost, ObjectId, TriggerEvent};
use crate::state::player::{CardId, PlayerId};
use crate::state::stack::{StackObject, StackObjectKind, TriggerData};
use crate::state::stubs::{
    FlushResumeSite, PendingTrigger, PendingTriggerKind, PendingTriggerTargets, TriggerDoubler,
    TriggerDoublerFilter, TriggerTargetOption,
};
use crate::state::targeting::{SpellTarget, Target};
use crate::state::types::AltCostKind;
use crate::state::types::{CardType, ChampionFilter, CounterType, KeywordAbility};
use crate::state::zone::ZoneId;
use crate::state::GameState;
use imbl::OrdSet;
// ---------------------------------------------------------------------------
// Restriction checks (PB-18)
// ---------------------------------------------------------------------------
/// PB-18: Check active game restrictions that would prevent ability activation.
///
/// Checks:
/// - `ArtifactAbilitiesCantBeActivated` (Collector Ouphe, Stony Silence)
/// - `OpponentsCantCastOrActivateDuringYourTurn` (Grand Abolisher, Myrel)
fn check_activate_restrictions(
    state: &GameState,
    player: PlayerId,
    source: ObjectId,
) -> Result<(), GameStateError> {
    use crate::state::stubs::GameRestriction;
    let active_player = state.turn.active_player;
    // PB-18 review Finding 3: Restrict zone scope — only battlefield objects are affected.
    //
    // Per Stony Silence ruling: "affects only artifacts on the battlefield. Activated
    // abilities that work in other zones (such as cycling) can still be activated."
    // Per Grand Abolisher ruling: "doesn't stop your opponents from activating abilities
    // of artifact, creature, or enchantment cards in zones other than the battlefield."
    let source_on_battlefield = state
        .objects
        .get(&source)
        .map(|o| o.zone == ZoneId::Battlefield)
        .unwrap_or(false);
    // Determine if the source is an artifact on the battlefield (for Collector Ouphe / Stony Silence).
    let source_is_artifact = source_on_battlefield
        && crate::rules::layers::calculate_characteristics(state, source)
            .map(|chars| chars.card_types.contains(&CardType::Artifact))
            .unwrap_or(false);
    for restriction in state.restrictions.iter() {
        // Skip restrictions whose source is no longer on the battlefield.
        let source_on_bf = state
            .objects
            .get(&restriction.source)
            .map(|o| matches!(o.zone, ZoneId::Battlefield))
            .unwrap_or(false);
        if !source_on_bf {
            continue;
        }
        let controller = restriction.controller;
        #[allow(clippy::collapsible_match)]
        match &restriction.restriction {
            // Collector Ouphe / Stony Silence:
            // "Activated abilities of artifacts can't be activated."
            // Only applies to artifacts on the battlefield (Finding 3 fix).
            GameRestriction::ArtifactAbilitiesCantBeActivated => {
                if source_is_artifact {
                    return Err(GameStateError::InvalidCommand(
                        "restriction: activated abilities of artifacts can't be activated (CR 101.2)"
                            .into(),
                    ));
                }
            }
            // Grand Abolisher / Myrel (ability activation component):
            // "During your turn, opponents can't activate abilities of artifacts,
            // creatures, or enchantments."
            // Only applies to permanents on the battlefield (Finding 3 fix).
            GameRestriction::OpponentsCantCastOrActivateDuringYourTurn => {
                if active_player == controller && player != controller && source_on_battlefield {
                    // Check if source is an artifact, creature, or enchantment.
                    let is_restricted_type =
                        crate::rules::layers::calculate_characteristics(state, source)
                            .map(|chars| {
                                chars.card_types.contains(&CardType::Artifact)
                                    || chars.card_types.contains(&CardType::Creature)
                                    || chars.card_types.contains(&CardType::Enchantment)
                            })
                            .unwrap_or(false);
                    if is_restricted_type {
                        return Err(GameStateError::InvalidCommand(
                            "restriction: opponents can't activate abilities of artifacts, creatures, or enchantments during your turn (CR 101.2)".into(),
                        ));
                    }
                }
            }
            // Other restrictions don't affect ability activation.
            _ => {}
        }
    }
    Ok(())
}
// ---------------------------------------------------------------------------
// Activated ability handler
// ---------------------------------------------------------------------------
/// Handle an ActivateAbility command: validate, pay cost, push onto the stack.
///
/// CR 602.2: To activate an ability, the controller announces it, pays all costs
/// in full, and the ability is placed on the stack. Unlike mana abilities, activated
/// abilities DO use the stack and must be responded to before resolving.
///
/// CR 602.2b -> 601.2i: after activation, the player who activated the ability receives
/// priority (CR 117.3c). "CR 116.3b" does not exist; the priority rules live in CR 117.3.
#[allow(clippy::too_many_arguments)]
pub fn handle_activate_ability(
    state: &mut GameState,
    player: PlayerId,
    source: ObjectId,
    ability_index: usize,
    targets: Vec<Target>,
    discard_card: Option<ObjectId>,
    sacrifice_target: Option<ObjectId>,
    x_value: Option<u32>,
    mut modes_chosen: Vec<usize>,
    hybrid_choices: Vec<crate::state::game_object::HybridManaPayment>,
    phyrexian_life_payments: Vec<bool>,
) -> Result<Vec<GameEvent>, GameStateError> {
    // CR 602.2: Activating requires priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // CR 702.61a: If a spell with split second is on the stack, no non-mana
    // abilities can be activated. (Mana abilities are handled in mana.rs and
    // are exempt from this check per CR 702.61b.)
    if crate::rules::casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; non-mana abilities cannot be activated (CR 702.61a)".into(),
        ));
    }
    // PB-18: Check active restrictions that prevent ability activation.
    check_activate_restrictions(state, player, source)?;
    // Source must be on the battlefield (or in hand for Channel/DiscardSelf abilities,
    // or in graveyard for graveyard-activated abilities like Reassembling Skeleton).
    {
        let obj = state.object(source)?;
        // CR 207.2c + CR 613.1f: Use layer-resolved activated abilities to determine whether
        // the ability at `ability_index` is a Channel/graveyard-zone ability. Base
        // characteristics only expose natively printed abilities; Layer 6 grants
        // (LayerModification::AddActivatedAbility) append past the native range.
        // For current grants none are Channel/graveyard-zone, so this is correct-by-accident
        // with base reads today — but reading from calculate_characteristics ensures the
        // dispatch is correct for future "grant a Channel ability" or "grant a
        // graveyard-activated ability" patterns. unwrap_or_else falls back to base
        // characteristics for objects not on the battlefield (LKI path).
        let resolved_ab_chars = crate::rules::layers::expect_characteristics(state, source);
        let (is_channel, activation_zone) = resolved_ab_chars
            .activated_abilities
            .get(ability_index)
            .map(|ab| (ab.cost.discard_self, ab.activation_zone.clone()))
            .unwrap_or((false, None));
        if is_channel {
            // Channel abilities are activated from hand. (Channel is an ability word --
            // CR 207.2c -- with no CR entry of its own; the `CR 702.34` once cited here is
            // FLASHBACK's rule. The behaviour comes from each card's printed text.)
            if obj.zone != ZoneId::Hand(player) {
                return Err(GameStateError::InvalidCommand(
                    "channel ability can only be activated from hand (printed text; \
                     Channel is an ability word, CR 207.2c, with no rule entry)"
                        .into(),
                ));
            }
            if obj.owner != player {
                return Err(GameStateError::InvalidCommand(
                    "you can only activate channel abilities on cards you own".into(),
                ));
            }
        } else if let Some(crate::cards::card_definition::ActivationZone::Graveyard) =
            activation_zone
        {
            // CR 602.2: Graveyard-activated ability — source must be in owner's graveyard.
            if obj.zone != ZoneId::Graveyard(player) {
                return Err(GameStateError::InvalidCommand(
                    "graveyard-activated ability can only be activated from the graveyard".into(),
                ));
            }
            if obj.owner != player {
                return Err(GameStateError::InvalidCommand(
                    "you can only activate graveyard abilities on cards you own".into(),
                ));
            }
        } else {
            if obj.zone != ZoneId::Battlefield {
                return Err(GameStateError::ObjectNotOnBattlefield(source));
            }
            if obj.controller != player {
                return Err(GameStateError::NotController {
                    player,
                    object_id: source,
                });
            }
        }
        // Validate the ability index exists.
        // CR 613.1f: Use layer-resolved activated abilities so Layer 6 grants
        // (e.g. PB-S LayerModification::AddActivatedAbility) are reachable; base
        // characteristics would only see native printed abilities, making granted
        // abilities unreachable at runtime.
        let resolved_ab = crate::rules::layers::expect_characteristics(state, source);
        if resolved_ab.activated_abilities.get(ability_index).is_none() {
            return Err(GameStateError::InvalidAbilityIndex {
                object_id: source,
                index: ability_index,
            });
        }
    }
    // CR 602.5d: Check sorcery-speed restriction before paying any costs.
    // CR 613.1f: Use layer-resolved activated abilities (Humility removes them).
    {
        state.object(source)?;
        let resolved = crate::rules::layers::expect_characteristics(state, source);
        let ab = resolved.activated_abilities.get(ability_index).ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "activated ability index {} not found (may have been removed by a continuous effect)",
                ability_index
            ))
        })?;
        if ab.sorcery_speed {
            // Must be active player's main phase with empty stack.
            if state.turn.active_player != player {
                return Err(GameStateError::InvalidCommand(
                    "sorcery-speed ability can only be activated during your own turn".into(),
                ));
            }
            if !matches!(
                state.turn.step,
                crate::state::turn::Step::PreCombatMain | crate::state::turn::Step::PostCombatMain
            ) {
                return Err(GameStateError::NotMainPhase);
            }
            if !state.stack_objects.is_empty() {
                return Err(GameStateError::StackNotEmpty);
            }
        }
        // CR 602.5b: "Activate only if [condition]" — check activation condition.
        if let Some(condition) = &ab.activation_condition {
            let ctx = crate::effects::EffectContext {
                source,
                controller: player,
                targets: vec![],
                target_remaps: Default::default(),
                kicker_times_paid: 0,
                was_overloaded: false,
                was_bargained: false,
                was_cleaved: false,
                evidence_collected: false,
                x_value: x_value.unwrap_or(0),
                gift_was_given: false,
                gift_opponent: None,
                last_effect_count: 0,
                last_dice_roll: 0,
                last_created_permanent: None,
                triggering_player: None,
                combat_damage_amount: 0,
                damage_dealt_amount: 0,
                damaged_player: None,
                triggering_creature_id: None,
                chosen_creature_type: None,
                mana_produced: None,
                sacrificed_creature_lki: vec![],
                sacrifice_fired: false,
                lki_counters: None,
                lki_power: None,
                countered_spell_controller: None,
                defending_player: None,
                source_transformed_this_resolution: false,
                effect_choice_gate_closed: false,
                chosen_objects: Vec::new(),
            };
            if !crate::effects::check_condition(state, condition, &ctx) {
                return Err(GameStateError::InvalidCommand(
                    "activation condition not met".into(),
                ));
            }
        }
        // CR 602.5b: "Activate only once each turn" — check once-per-turn restriction.
        if ab.once_per_turn {
            let activations = state
                .objects
                .get(&source)
                .map(|o| o.abilities_activated_this_turn)
                .unwrap_or(0);
            if activations > 0 {
                return Err(GameStateError::InvalidCommand(
                    "ability can only be activated once per turn".into(),
                ));
            }
        }
    }
    // Clone the cost, effect, and target requirements before mutating state.
    // Effect must be captured now in case sacrifice-as-cost removes the source object.
    // CR 613.1f: Use layer-resolved activated abilities (Humility removes them).
    let (ability_cost, mut embedded_effect, target_requirements, is_once_per_turn, ability_modes) = {
        state.object(source)?;
        let resolved = crate::rules::layers::expect_characteristics(state, source);
        let ab = resolved
            .activated_abilities
            .get(ability_index)
            .ok_or_else(|| {
                GameStateError::InvalidCommand(format!(
                    "activated ability index {} not found (removed by continuous effect)",
                    ability_index
                ))
            })?;
        (
            ab.cost.clone(),
            ab.effect.clone(),
            ab.targets.clone(),
            ab.once_per_turn,
            ab.modes.clone(),
        )
    };
    // CR 700.2a/700.2d (PB-EF7): Validate explicit mode choices for a modal activated
    // ability. Mirrors the Spell modal validation in casting.rs (`validated_modes_chosen`),
    // performed BEFORE any cost payment so an illegal mode/target choice never spends mana
    // or a sacrifice (CR 602.2 -- an illegal activation rewinds to before it started).
    let validated_modes_chosen: Vec<usize> = if !modes_chosen.is_empty() {
        match &ability_modes {
            None => {
                return Err(GameStateError::InvalidCommand(
                    "modes_chosen specified but this ability has no modal structure (CR 700.2a)"
                        .into(),
                ));
            }
            Some(ms) => {
                // CR 700.2a: Each chosen index must be within range.
                for &idx in &modes_chosen {
                    if idx >= ms.modes.len() {
                        return Err(GameStateError::InvalidCommand(format!(
                            "mode index {} is out of range (ability has {} modes) (CR 700.2a)",
                            idx,
                            ms.modes.len()
                        )));
                    }
                }
                // CR 700.2d: Duplicate modes are only allowed when allow_duplicate_modes is set.
                if !ms.allow_duplicate_modes {
                    let mut seen = std::collections::BTreeSet::new();
                    for &idx in &modes_chosen {
                        if !seen.insert(idx) {
                            return Err(GameStateError::InvalidCommand(format!(
                                "mode index {} chosen more than once; use allow_duplicate_modes: true to allow (CR 700.2d)",
                                idx
                            )));
                        }
                    }
                }
                // CR 700.2a: Count must be between min_modes and max_modes.
                let chosen_count = modes_chosen.len();
                if chosen_count < ms.min_modes {
                    return Err(GameStateError::InvalidCommand(format!(
                        "must choose at least {} mode(s); only {} chosen (CR 700.2a)",
                        ms.min_modes, chosen_count
                    )));
                }
                if chosen_count > ms.max_modes {
                    return Err(GameStateError::InvalidCommand(format!(
                        "may choose at most {} mode(s); {} chosen (CR 700.2a)",
                        ms.max_modes, chosen_count
                    )));
                }
                // CR 700.2a: modes always execute in ascending printed order.
                modes_chosen.sort_unstable();
                modes_chosen
            }
        }
    } else if let Some(ms) = &ability_modes {
        // CR 700.2a / 602.2b (PB-DP3 / DP-4): the controller chooses the mode(s) "as part of …
        // activating that ability". The engine may not pick for them.
        if ms.min_modes == 0 {
            // "Choose up to N" -- announcing zero modes is legal (CR 700.2a). Unlike the Spell
            // path in casting.rs, this IS representable here: with validated_modes_chosen
            // empty, the `if !validated_modes_chosen.is_empty()` guard below leaves
            // `embedded_effect` as the ability's own base effect -- which is the correct
            // "no mode chosen" behaviour ONLY because a modal activated ability's base
            // `effect` is `Effect::Nothing` (or unset) by authoring convention. A modal
            // ability authored with a non-trivial base `effect` would execute it
            // unconditionally on a zero-mode activation. No shipped card has this shape
            // today; the debug_assert below catches a future one at test/CI time
            // (review Finding 6).
            debug_assert!(
                matches!(
                    embedded_effect,
                    None | Some(crate::cards::card_definition::Effect::Nothing)
                ),
                "a modal activated ability with min_modes: 0 must have Effect::Nothing (or no \
                 effect) as its base `effect`, so a zero-mode activation is a true no-op \
                 rather than silently firing this ability's own base effect -- got {:?}",
                embedded_effect
            );
            vec![]
        } else {
            return Err(GameStateError::InvalidCommand(format!(
                "modal ability requires an explicit mode choice: at least {} mode(s) must be \
                 announced as part of activating it (CR 602.2b/700.2a); none were",
                ms.min_modes
            )));
        }
    } else {
        // Non-modal ability.
        vec![]
    };
    // CR 700.2c/700.2f (PB-EF7, mirrors PB-AC4): If the ability has per-mode target
    // requirements (`ModeSelection.mode_targets` is `Some`), targets are announced/
    // validated ONLY for the chosen mode's requirements -- not the flat union of every
    // mode's targets. Post-PB-DP3, `validated_modes_chosen` is either the explicit
    // fully-validated choice or (for a `min_modes: 0` ability) legitimately empty -- there is
    // no more auto-select-mode-0 fallback to account for here (unlike the Spell path in
    // casting.rs, which retains a fail-safe `vec![0]` arm for a different reason -- see
    // Change 2 there).
    let mode_targets_active: Option<Vec<crate::cards::card_definition::TargetRequirement>> =
        ability_modes.as_ref().and_then(|ms| {
            ms.mode_targets.as_ref().map(|mt| {
                debug_assert_eq!(
                    mt.len(),
                    ms.modes.len(),
                    "ModeSelection.mode_targets.len() ({}) must equal modes.len() ({}) \
                     (CR 700.2c author invariant)",
                    mt.len(),
                    ms.modes.len()
                );
                validated_modes_chosen
                    .iter()
                    .flat_map(|&idx| mt.get(idx).cloned().unwrap_or_default())
                    .collect::<Vec<crate::cards::card_definition::TargetRequirement>>()
            })
        });
    // CR 700.2c: Multiple modes chosen combined with per-mode targets is not a supported
    // combination (mirrors the Escalate+mode_targets hard-reject in casting.rs) -- a flat
    // Sequence of chosen-mode effects would break each mode's LOCAL target-slice indexing.
    // No PB-EF7-scoped card hits this branch (both flipped cards are choose-exactly-one).
    if mode_targets_active.is_some() && validated_modes_chosen.len() > 1 {
        return Err(GameStateError::InvalidCommand(
            "multiple modes chosen combined with ModeSelection.mode_targets is not supported (CR 700.2c/700.2a)".into(),
        ));
    }
    // CR 601.2c: General target validation for activated abilities.
    // If the ability declares TargetRequirements, validate each target against them
    // BEFORE spending any costs (so mana is not wasted on illegal activations).
    let target_source_chars = crate::rules::layers::calculate_characteristics(state, source)
        .or_else(|| {
            state
                .objects
                .get(&source)
                .map(|o| o.characteristics.clone())
        });
    if let Some(active_reqs) = &mode_targets_active {
        // CR 700.2c/700.2f: per-mode target validation is POSITIONAL (declaration order
        // must match `active_reqs` order), mirroring the Spell modal path in casting.rs.
        if !target_requirements.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "modal activated ability has both a flat targets list and ModeSelection.mode_targets set; only one may be used (CR 700.2c author invariant)".into(),
            ));
        }
        if active_reqs.iter().any(|r| {
            matches!(
                r,
                crate::cards::card_definition::TargetRequirement::UpToN { .. }
            )
        }) {
            return Err(GameStateError::InvalidCommand(
                "ModeSelection.mode_targets may not contain UpToN (variable-count per-mode targets are unsupported) (CR 700.2c)".into(),
            ));
        }
        crate::rules::casting::validate_targets_positional(
            state,
            &targets,
            active_reqs,
            player,
            target_source_chars.as_ref(),
            Some(source),
        )?;
    } else if !target_requirements.is_empty() {
        // PB-XS: thread the activating object's id through validation so
        // `TargetFilter.exclude_self` (CR 109.1 / 601.2c — "another target X")
        // can reject self-targeting on activated abilities (Samut, Ezuri, etc.).
        // The `with_source` variant also supplies the existing
        // `TargetSpellOrAbilityWithSingleTarget` self-targeting prevention path.
        crate::rules::casting::validate_targets_with_source(
            state,
            &targets,
            &target_requirements,
            player,
            target_source_chars.as_ref(),
            source,
        )?;
    }
    // PB-DX25c §3.1: the requirement list that actually governed the validation above —
    // whichever of `mode_targets_active` (per-mode, CR 700.2c/700.2f) or
    // `target_requirements` (the flat list) applied. Recorded onto the stack object
    // below so a later retarget (`rules::retarget`) validates against the SAME list a
    // real cast was checked against, never a re-derivation.
    let announced_requirements: Vec<crate::cards::card_definition::TargetRequirement> =
        mode_targets_active
            .clone()
            .unwrap_or_else(|| target_requirements.clone());
    // CR 700.2a (PB-EF7): Bake the chosen mode(s) into a concrete effect NOW, at
    // activation time -- not at resolution. Both eligible cards (Goblin Cratermaker,
    // Cankerbloom) cost `Cost::SacrificeSelf`, so at resolution `state.objects.get(source)`
    // is `None` (CR 400.7) and only a captured `embedded_effect` survives. This mirrors how
    // sacrifice-cost activated abilities already capture their effect at activation
    // (see the module doc / gotchas). The chosen mode's `DeclaredTarget` indices are LOCAL
    // to its target slice; because only one mode is chosen for both eligible cards, local
    // == global and `stack_obj.targets` (set below) IS that single slice.
    if let Some(ms) = &ability_modes {
        if !validated_modes_chosen.is_empty() {
            embedded_effect = if validated_modes_chosen.len() == 1 {
                ms.modes.get(validated_modes_chosen[0]).cloned()
            } else {
                // mode_targets_active.is_some() + len() > 1 was already hard-rejected above.
                Some(crate::cards::card_definition::Effect::Sequence(
                    validated_modes_chosen
                        .iter()
                        .filter_map(|&idx| ms.modes.get(idx).cloned())
                        .collect(),
                ))
            };
        }
    }
    // CR 702.6a / CR 601.2c: Equip abilities can only target "a creature you control."
    // Validate target type and controller BEFORE spending any costs, so that mana is
    // not wasted when the activation is illegal.
    //
    // Legacy special-case check for AttachEquipment effects. Cards with proper
    // TargetRequirement declarations will be validated by the general check above.
    //
    // OOS-DX20-7: this guard is now redundant with the declarative check above for
    // every ability that carries a `TargetRequirement` (that check runs first and
    // rejects), and silently PERMISSIVE for any ability that does not, because
    // `targets.first()` on an empty `Vec` is `None` -- the `if let` below simply
    // does not fire, and a zero-target activation proceeds to pay its cost and
    // fizzle at resolution with no error. Kept (not removed) because it still covers
    // card-def-authored equip abilities with no declared `TargetRequirement`; the
    // durable closure would be a roster gate over `all_cards()` pinning
    // "Activated + AttachEquipment ⇒ non-empty targets" (out of scope here).
    if matches!(
        &embedded_effect,
        Some(crate::cards::card_definition::Effect::AttachEquipment { .. })
    ) {
        // PB-DX52: a non-`Object` first target skips this block, exactly as before. CR
        // 702.6a's equip target is "target creature you control", which neither a player
        // nor a stack entry can be -- and the general target validation loop above has
        // already rejected such a declaration against the ability's own
        // `TargetRequirement`, so this block is a second, narrower check rather than the
        // only one.
        if let Some(Target::Object(target_id)) = targets.first() {
            let target_id = *target_id;
            // Check: target must be a creature on the battlefield controlled by the
            // activating player. Use layer-computed characteristics for correctness
            // under continuous effects (e.g. animated artifacts).
            let on_battlefield_and_controlled = state
                .objects
                .get(&target_id)
                .map(|obj| {
                    obj.zone == crate::state::zone::ZoneId::Battlefield && obj.controller == player
                })
                .unwrap_or(false);
            let is_creature = {
                let layer_chars = crate::rules::layers::calculate_characteristics(state, target_id)
                    .or_else(|| {
                        state
                            .objects
                            .get(&target_id)
                            .map(|o| o.characteristics.clone())
                    });
                layer_chars
                    .map(|chars| {
                        chars
                            .card_types
                            .contains(&crate::state::types::CardType::Creature)
                    })
                    .unwrap_or(false)
            };
            if !on_battlefield_and_controlled {
                return Err(GameStateError::InvalidTarget(
                    "equip target must be a creature you control on the battlefield".into(),
                ));
            }
            if !is_creature {
                return Err(GameStateError::InvalidTarget(
                    "equip target must be a creature".into(),
                ));
            }
        }
    }
    // CR 702.67a / CR 601.2c: Fortify abilities can only target "a land you control."
    // Validate target type and controller BEFORE spending any costs, so that mana is
    // not wasted when the activation is illegal.
    if matches!(
        &embedded_effect,
        Some(crate::cards::card_definition::Effect::AttachFortification { .. })
    ) {
        // CR 301.6: A Fortification that's also a creature can't fortify a land.
        // Check source (the Fortification itself) using layer-resolved characteristics.
        let source_is_creature = {
            let layer_chars = crate::rules::layers::calculate_characteristics(state, source)
                .or_else(|| {
                    state
                        .objects
                        .get(&source)
                        .map(|o| o.characteristics.clone())
                });
            layer_chars
                .map(|chars| {
                    chars
                        .card_types
                        .contains(&crate::state::types::CardType::Creature)
                })
                .unwrap_or(false)
        };
        if source_is_creature {
            return Err(GameStateError::InvalidTarget(
                "a Fortification that's also a creature can't fortify a land (CR 301.6)".into(),
            ));
        }
        // PB-DX52: see the equip block above -- CR 301.6's fortify target is "target
        // land you control", which neither a player nor a stack entry can be.
        if let Some(Target::Object(target_id)) = targets.first() {
            let target_id = *target_id;
            // Check: target must be a land on the battlefield controlled by the
            // activating player. Use layer-computed characteristics for correctness
            // under continuous effects (e.g. non-land permanents that became lands).
            let on_battlefield_and_controlled = state
                .objects
                .get(&target_id)
                .map(|obj| {
                    obj.zone == crate::state::zone::ZoneId::Battlefield && obj.controller == player
                })
                .unwrap_or(false);
            let is_land = {
                let layer_chars = crate::rules::layers::calculate_characteristics(state, target_id)
                    .or_else(|| {
                        state
                            .objects
                            .get(&target_id)
                            .map(|o| o.characteristics.clone())
                    });
                layer_chars
                    .map(|chars| {
                        chars
                            .card_types
                            .contains(&crate::state::types::CardType::Land)
                    })
                    .unwrap_or(false)
            };
            if !on_battlefield_and_controlled {
                return Err(GameStateError::InvalidTarget(
                    "fortify target must be a land you control on the battlefield".into(),
                ));
            }
            if !is_land {
                return Err(GameStateError::InvalidTarget(
                    "fortify target must be a land".into(),
                ));
            }
        }
    }
    // CR 702.151a: Reconfigure unattach ability -- "Activate only if this permanent is
    // attached to a creature." Validate BEFORE spending any costs.
    if matches!(
        &embedded_effect,
        Some(crate::cards::card_definition::Effect::DetachEquipment { .. })
    ) {
        let is_attached = state
            .objects
            .get(&source)
            .and_then(|obj| obj.attached_to)
            .is_some();
        if !is_attached {
            return Err(GameStateError::InvalidCommand(
                "reconfigure unattach: permanent must be attached to a creature".into(),
            ));
        }
    }
    let mut events = Vec::new();
    // Pay tap cost if required (CR 602.2b).
    if ability_cost.requires_tap {
        let obj = state.object(source)?;
        if obj.status.tapped {
            return Err(GameStateError::PermanentAlreadyTapped(source));
        }
        // CR 302.6 / CR 702.10: Summoning sickness prevents using {T} abilities
        // on creatures unless they have haste.
        // CR 613.1f: Use layer-resolved characteristics so that animated lands (Layer 4
        // type-change) are recognized as creatures, and granted haste (Layer 6) is seen.
        // Sibling of the mana.rs fix in PB-S: handle_tap_for_mana was already updated to
        // use calculate_characteristics for the same reason. Without this, a granted
        // tap-cost activated ability on an animated non-creature would skip sickness
        // checks, and a granted haste on a summoning-sick creature would still be rejected.
        let resolved_tap = crate::rules::layers::expect_characteristics(state, source);
        let is_creature = resolved_tap
            .card_types
            .contains(&crate::state::types::CardType::Creature);
        if is_creature && obj.has_summoning_sickness {
            let has_haste = resolved_tap
                .keywords
                .contains(&crate::state::types::KeywordAbility::Haste);
            if !has_haste {
                return Err(GameStateError::InvalidCommand(format!(
                    "object {:?} has summoning sickness and cannot use abilities with {{T}}",
                    source
                )));
            }
        }
        if let Some(obj) = state.expect_object_mut(source) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: source,
        });
    }
    // SR-36 / SF-9 (CR 118.3 / CR 119.4b): life-cost legality check, before the mana cost
    // is paid (the {T} cost above is already paid by this point).
    // Mirrors `handle_tap_for_mana`'s step 5b (`rules/mana.rs`) — the mana-ability and
    // non-mana-ability paths are disjoint by construction (`mana_ability_lowering` in
    // `testing/replay_harness.rs`: an ability that lowers into a `ManaAbility` is excluded
    // from `activated_abilities`, so it never reaches this function), so this cannot
    // double-charge a card SR-34 already fixed. CR 119.4b: a cost of 0 is always legal,
    // even at negative life — the check must short-circuit on `life_cost > 0` rather than
    // comparing unconditionally.
    //
    // PB-RS2 (CR 119.4, CR 601.2h/602.2b): when `ability_cost.mana_cost` carries a
    // Phyrexian pip paid with life, the check below must be against the COMBINED total
    // of `life_cost` and that Phyrexian life — not this ability_cost.life_cost alone —
    // because the components of a cost may be paid in any order and CR 119.4 gates "the
    // amount of the payment" for the whole cost. That combined check happens inside the
    // `mana_cost` branch below (after the flatten computes the Phyrexian life amount);
    // this standalone check covers only the (much more common) case where there is no
    // mana cost at all, where it is already the full, correct check.
    if ability_cost.mana_cost.is_none() && ability_cost.life_cost > 0 {
        let player_state = state.player(player)?;
        if player_state.life_total < ability_cost.life_cost as i32 {
            return Err(GameStateError::InsufficientLife {
                player,
                required: ability_cost.life_cost,
                actual: player_state.life_total,
            });
        }
    }
    // Pay mana cost if required (CR 602.2a).
    if let Some(ref mana_cost) = ability_cost.mana_cost {
        // CR 107.3k: For activated abilities with {X} in the activation cost, add x_count * x_value
        // to generic before payment. Mirrors the casting.rs handling for spell X costs.
        let mut resolved_cost = mana_cost.clone();
        let xv = x_value.unwrap_or(0);
        if resolved_cost.x_count > 0 {
            resolved_cost.generic += resolved_cost.x_count * xv;
            resolved_cost.x_count = 0;
        } else if xv > 0 {
            resolved_cost.generic += xv;
        }
        // CR 602.2b + 601.2f: Apply self-activated-cost-reduction from CardDefinition.
        // Uses index-keyed `activated_ability_cost_reductions` field (alternative design to
        // avoid adding a field to AbilityDefinition::Activated which has 400+ match sites).
        //
        // CR 601.2f + CR 613.1f — PB-S-L05 invariant (option b, documented-only):
        // Granted activated abilities (Layer 6 LayerModification::AddActivatedAbility) are
        // appended to the ability list PAST the native printed range. Card defs with
        // `activated_ability_cost_reductions` only reference native ability indices, so
        // `get_self_activated_reduction` for a granted-ability index (beyond the native range)
        // returns None — correct by definition (granted abilities have no card-def-specific
        // cost reductions). A debug_assert is not feasible here because the native range is
        // determined by both AbilityDefinition::Activated entries AND ObjectSpec-level
        // with_activated_ability() entries (the latter is used by some token specs and tests),
        // which cannot be distinguished from the card def alone.
        // Refactoring to a stable ability identifier is deferred until a card def collides
        // (see get_self_activated_reduction doc comment for details).
        // PB-OS4b (CR 712.8d/e): `activated_ability_cost_reductions` is keyed by
        // index into the FRONT `CardDefinition`-level list. After the Channel-A
        // fix, a transformed permanent's `activated_abilities` are back-face
        // -derived, so a back-face activated ability at some index could collide
        // with an unrelated front-face cost reduction keyed at the same index.
        // The schema has no back-face cost reductions (a back face cannot declare
        // one), so skip this lookup entirely when the source is transformed.
        let source_is_transformed = state
            .expect_object(source)
            .map(|o| o.is_transformed)
            .unwrap_or(false);
        if !source_is_transformed {
            if let Some(card_id) = state.expect_object(source).and_then(|o| o.card_id.clone()) {
                if let Some(card_def) = state.card_registry.get(card_id) {
                    let amount = get_self_activated_reduction(card_def, ability_index)
                        .map(|r| evaluate_self_activated_reduction(state, player, &r))
                        .unwrap_or(0);
                    if amount > 0 {
                        resolved_cost.generic = resolved_cost.generic.saturating_sub(amount);
                    }
                }
            }
        }
        // CR 107.4e/107.4f (via CR 602.2b): flatten hybrid/Phyrexian choices before
        // payment. An activated ability's activation cost is its analog to a
        // spell's mana cost (CR 602.2b), so it must go through the same flatten
        // step `casting.rs` uses for spells — pre-PB-RS2 this called `can_spend`/
        // `spend` on the RAW cost, so a pure `{B/R}` (mana_value()==1, passing the
        // gate below) was charged as an all-zero cost (OOS-RS-2). The flatten must
        // happen BEFORE the `mana_value() > 0` gate: a pure Phyrexian pip paid
        // entirely with life has raw mana_value()==1 but a flattened mana_value()
        // of 0, and the gate must correctly skip the mana check in that case while
        // the (sibling, not nested) life deduction below still fires.
        // Review finding #8: call the inherent `ManaCost::flatten_hybrid_phyrexian`
        // directly rather than routing through `super::casting::flatten_hybrid_phyrexian`
        // — the plan's §4 explicitly flagged reaching into `casting` from a non-cast
        // payment path as a layering smell (AC 5119 requires one implementation, not
        // one call path). `legal_actions.rs:1044` already calls the inherent method
        // this way; this call site now matches.
        let (flat_cost, phyrexian_life) =
            if !resolved_cost.hybrid.is_empty() || !resolved_cost.phyrexian.is_empty() {
                resolved_cost
                    .flatten_hybrid_phyrexian(&hybrid_choices, &phyrexian_life_payments)
                    .map_err(GameStateError::InvalidCommand)?
            } else {
                (resolved_cost.clone(), 0)
            };
        // CR 119.4, CR 601.2h/602.2b (PB-RS2): check the COMBINED total of
        // `ability_cost.life_cost` and a Phyrexian pip paid with life against
        // life_total ONCE, before ANY deduction (mana or life) below — not each
        // independently. The cost's components (tap/mana/life/Phyrexian-life) may
        // be paid in any order, and CR 119.4 gates "the amount of the payment" for
        // the whole cost. A player at 3 life activating a "Pay 2 life" ability with
        // a `{G/P}` paid with life may not pay a combined 4.
        let combined_life_cost = ability_cost.life_cost + phyrexian_life;
        if combined_life_cost > 0 {
            let player_state = state.player(player)?;
            if player_state.life_total < combined_life_cost as i32 {
                return Err(GameStateError::InsufficientLife {
                    player,
                    required: combined_life_cost,
                    actual: player_state.life_total,
                });
            }
        }
        if flat_cost.mana_value() > 0 {
            let player_state = state.player_mut(player)?;
            if !player_state.mana_pool.can_spend(&flat_cost, None) {
                return Err(GameStateError::InsufficientMana);
            }
            player_state.mana_pool.spend(&flat_cost, None);
        }
        // CR 107.4f: pay life for a Phyrexian pip paid with life. A sibling of the
        // mana-payment block above, not nested inside it — see the
        // pure-Phyrexian-paid-with-life case in the comment above. Legality
        // (including the combined check with `ability_cost.life_cost`) was already
        // validated above, before any mutation.
        if phyrexian_life > 0 {
            let player_state = state.player_mut(player)?;
            player_state.life_total -= phyrexian_life as i32;
            events.push(GameEvent::LifeLost {
                player,
                amount: phyrexian_life,
            });
        }
        if flat_cost.mana_value() > 0 || phyrexian_life > 0 {
            // Emit the ORIGINAL (unflattened) cost for event consumers — mirrors
            // casting.rs's ManaCostPaid emission, which carries hybrid/Phyrexian
            // pip info rather than the flattened shape.
            events.push(GameEvent::ManaCostPaid {
                player,
                cost: resolved_cost,
            });
        }
    }
    // SR-36 / SF-9 (CR 118.3 / CR 119.4b): pay the life cost. CR 601.2h: tap/mana/life/
    // sacrifice costs in this group may be paid in any order — none of their legality
    // depends on another's result — so placing payment here (after the mana-cost block,
    // before discard/sacrifice) is about legibility, not a transactional guarantee: an
    // `Err` anywhere below discards the whole `GameState` regardless, since
    // `process_command` takes `GameState` by value and only returns it on `Ok`.
    if ability_cost.life_cost > 0 {
        let player_state = state.player_mut(player)?;
        player_state.life_total -= ability_cost.life_cost as i32;
        events.push(GameEvent::LifeLost {
            player,
            amount: ability_cost.life_cost,
        });
    }
    // CR 602.2 / CR 111.10g: Pay discard-a-card cost (e.g., Blood token activation).
    // The discard is a cost, not an effect — it happens at activation time, before the
    // ability goes on the stack. The caller must supply discard_card: Some(ObjectId)
    // if the ability cost requires a discard.
    if ability_cost.discard_card {
        let card_to_discard = discard_card.ok_or_else(|| {
            GameStateError::InvalidCommand(
                "ability requires discarding a card as cost: discard_card must be Some (CR 602.2)"
                    .into(),
            )
        })?;
        // Validate the card is in the player's hand.
        {
            let card_obj = state.object(card_to_discard)?;
            if card_obj.zone != ZoneId::Hand(player) {
                return Err(GameStateError::InvalidCommand(
                    "discard cost: card must be in your hand (CR 602.2)".into(),
                ));
            }
        }
        // Move card from hand to graveyard.
        let (new_grave_id, _) =
            state.move_object_to_zone(card_to_discard, ZoneId::Graveyard(player))?;
        events.push(GameEvent::CardDiscarded {
            player,
            object_id: card_to_discard,
            new_id: new_grave_id,
        });
    }
    // Pay discard-self cost (Channel abilities). The source card is in the player's hand;
    // discarding it is part of the activation cost. (Channel is an ability word, CR 207.2c,
    // with no CR entry -- `CR 702.34` is Flashback.)
    if ability_cost.discard_self {
        // CR 608.2h / CR 113.7a (PB-DX39): costs are paid during activation (CR 601.2h,
        // reached for an activated ability by CR 602.2b), i.e. BEFORE this ability is
        // pushed onto the stack a few statements
        // below -- so `GameState::capture_lki_snapshot`'s pending-ability clause cannot
        // see it and declines. Capture explicitly. A measured no-op for THIS cost (the
        // source is in hand by construction, and the helper is battlefield-only
        // so it does not put hidden information into a public store); present so the three
        // self-move cost blocks stay uniform and a fourth cannot be written without it.
        state.capture_source_lki_for_pending_ability(source);
        let (new_grave_id, _) = state.move_object_to_zone(source, ZoneId::Graveyard(player))?;
        events.push(GameEvent::CardDiscarded {
            player,
            object_id: source,
            new_id: new_grave_id,
        });
    }
    // Pay sacrifice cost (CR 602.2c). Move source to graveyard before pushing to stack.
    if ability_cost.sacrifice_self {
        // PB-AC8 / CR 701.21a: a "can't be sacrificed" source can't pay a
        // sacrifice-self cost -- the ability simply cannot be activated this way.
        if crate::effects::object_cant_be_sacrificed(state, source) {
            return Err(GameStateError::InvalidCommand(
                "sacrifice cost: this permanent can't be sacrificed (CR 701.21a)".into(),
            ));
        }
        let (
            is_creature,
            owner,
            pre_death_controller,
            pre_death_counters,
            sac_self_lki_power,
            sac_self_pre_chars,
        ) = {
            let obj = state.object(source)?;
            // CR 613.1f + CR 603.10a + CR 613.1e: Use layer-resolved card types to determine
            // whether the sacrificed permanent is a creature at the time of sacrifice. A
            // permanent can become a creature via Layer 4 type-change effects (e.g. "animate"
            // enchantments). Reading base obj.characteristics.card_types would return the
            // printed type, causing an animated artifact dying via sacrifice-self to emit
            // PermanentDestroyed instead of CreatureDied — "whenever a creature dies" triggers
            // would fail to fire. unwrap_or_else fallback handles graveyard/exile objects
            // (LKI path) where calculate_characteristics may return None.
            let pre_chars_opt = crate::rules::layers::calculate_characteristics(state, source);
            let resolved = crate::rules::layers::expect_characteristics(state, source);
            // CR 603.10a: capture layer-resolved power for SourcePowerAtLastKnownInformation.
            let lki_power = resolved.power.or(obj.characteristics.power);
            (
                resolved
                    .card_types
                    .contains(&crate::state::types::CardType::Creature),
                obj.owner,
                // CR 603.3a: capture controller before move_object_to_zone resets it to owner.
                obj.controller,
                // CR 702.79a: capture counters before move_object_to_zone resets them.
                obj.counters.clone(),
                lki_power,
                // CR 603.10a / CR 613.1d: full LKI characteristics snapshot for filtered death triggers.
                pre_chars_opt,
            )
        };
        // CR 608.2h / CR 113.7a (PB-DX39, `OOS-DX5-7`): the ORDER is the reason this call
        // must exist. This block's own comment says "Move source to graveyard before
        // pushing to stack", and it means it -- at this instant `state.stack_objects`
        // does not yet contain the ability and `state.pending_triggers` does not contain a
        // trigger for it, so `capture_lki_snapshot`'s `is_source_of_a_pending_ability`
        // clause answers `false` and declines. The ability reaches the stack immediately
        // afterwards and then needs exactly this information. Mardu Ascendancy's
        // `EffectFilter::CreaturesYouControl` applied to NOBODY in every game before this.
        // If costs are ever paid after the push, this becomes redundant, not wrong.
        state.capture_source_lki_for_pending_ability(source);
        let (new_id, _) = state.move_object_to_zone(source, ZoneId::Graveyard(owner))?;
        if is_creature {
            events.push(GameEvent::CreatureDied {
                object_id: source,
                new_grave_id: new_id,
                controller: pre_death_controller,
                pre_death_counters,
                // CR 603.10a: LKI power snapshot for SourcePowerAtLastKnownInformation.
                pre_death_power: sac_self_lki_power,
                pre_death_characteristics: sac_self_pre_chars,
            });
        } else {
            events.push(GameEvent::PermanentDestroyed {
                object_id: source,
                new_grave_id: new_id,
                // CR 603.10a: pass LKI counters for WhenLeavesBattlefield triggers.
                pre_lba_counters: pre_death_counters.clone(),
                // CR 603.10a: pass LKI power snapshot.
                pre_lba_power: sac_self_lki_power,
            });
        }
        // CR 701.21a: PermanentSacrificed alongside death/destroy for sacrifice cost.
        events.push(GameEvent::PermanentSacrificed {
            player: pre_death_controller,
            object_id: source,
            new_id,
        });
    }
    // CR 118.12 + CR 406 + CR 602.2c: Pay exile-self cost. Move source to its owner's exile
    // zone before pushing the ability to the stack. `embedded_effect` was already captured at
    // line ~309 (before cost payment), so resolution works after the source ID is dead.
    // Note: exile is NOT death (CR 700.4) — no CreatureDied event is emitted.
    if ability_cost.exile_self {
        let (pre_exile_controller, pre_exile_counters, exile_self_lki_power) = state
            .expect_object(source)
            .map(|o| {
                let lki_power = crate::rules::layers::calculate_characteristics(state, source)
                    .and_then(|c| c.power)
                    .or(o.characteristics.power);
                (o.controller, o.counters.clone(), lki_power)
            })
            .unwrap_or((state.turn.active_player, imbl::OrdMap::new(), None));
        // CR 608.2h / CR 113.7a (PB-DX39): same ordering as the sacrifice-self block --
        // the exile happens before the push, so the departure-driven clause cannot see the
        // ability yet. CR 700.4: exile is not death, but CR 400.7 retires the id either
        // way, which is what a source-relative filter would otherwise fail to resolve.
        state.capture_source_lki_for_pending_ability(source);
        let (new_exile_id, _) = state.move_object_to_zone(source, ZoneId::Exile)?;
        events.push(crate::rules::events::GameEvent::ObjectExiled {
            player: pre_exile_controller,
            object_id: source,
            new_exile_id,
            // CR 603.10a: pass LKI counters for WhenLeavesBattlefield triggers.
            pre_lba_counters: pre_exile_counters,
            // CR 603.10a: pass LKI power snapshot for SourcePowerAtLastKnownInformation.
            pre_lba_power: exile_self_lki_power,
        });
    }
    // CR 701.43a/c: Pay exert cost (`Cost::Exert`). The source must be on the battlefield
    // (701.43c); set `Designations::EXERTED` so the untap loop (turn_actions.rs) skips
    // this permanent's next untap step and clears the designation at that point
    // (701.43a/b: expires during that untap step, even if exerted more than once first).
    if ability_cost.exert {
        let src_obj = state.object(source)?;
        if src_obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::InvalidCommand(
                "exert cost: source must be on the battlefield (CR 701.43c)".into(),
            ));
        }
        if let Some(obj) = state.expect_object_mut(source) {
            obj.designations
                .insert(crate::state::game_object::Designations::EXERTED);
        }
        events.push(GameEvent::PermanentExerted { object_id: source });
    }
    // PB-P/PB-EF10: CR 608.2b/608.2h/608.2i — LKI of creatures sacrificed as
    // activated-ability cost. Populated inside the sacrifice block (BEFORE
    // move_object_to_zone); read at StackObject construction and propagated to
    // EffectContext at resolution.
    let mut sacrificed_lki: Vec<crate::state::types::SacrificedCreatureLki> = vec![];
    // CR 602.2: Pay sacrifice-another-permanent cost (e.g., "Sacrifice a creature: ...").
    // The caller supplies the ObjectId of the permanent to sacrifice via `sacrifice_target`.
    if let Some(ref filter) = ability_cost.sacrifice_filter {
        let sac_id = sacrifice_target.ok_or_else(|| {
            GameStateError::InvalidCommand(
                "ability requires sacrificing a permanent as cost: sacrifice_target must be Some (CR 602.2)".into(),
            )
        })?;
        // Validate the sacrifice target is on the battlefield and controlled by the player.
        {
            let sac_obj = state.object(sac_id)?;
            if sac_obj.zone != ZoneId::Battlefield {
                return Err(GameStateError::InvalidCommand(
                    "sacrifice cost: permanent must be on the battlefield (CR 602.2)".into(),
                ));
            }
            if sac_obj.controller != player {
                return Err(GameStateError::InvalidCommand(
                    "sacrifice cost: you must control the permanent to sacrifice (CR 602.2)".into(),
                ));
            }
            // PB-EF1 (CR 109.1): "Sacrifice ANOTHER [permanent]" — the source cannot pay
            // its own cost by sacrificing itself. `SacrificeFilter` carries no ObjectId, so
            // the "another" restriction rides on `ActivationCost.sacrifice_exclude_self`.
            if ability_cost.sacrifice_exclude_self && sac_id == source {
                return Err(GameStateError::InvalidCommand(
                    "sacrifice cost: must sacrifice another permanent, not the source (CR 109.1)"
                        .into(),
                ));
            }
            // PB-AC8 / CR 701.21a: a "can't be sacrificed" permanent is not a legal
            // choice to pay a sacrifice-another cost.
            if crate::effects::object_cant_be_sacrificed(state, sac_id) {
                return Err(GameStateError::InvalidCommand(
                    "sacrifice cost: this permanent can't be sacrificed (CR 701.21a)".into(),
                ));
            }
            // Validate the permanent matches the sacrifice filter using layer-resolved characteristics.
            let chars = crate::rules::layers::expect_characteristics(state, sac_id);
            let matches_filter = match filter {
                crate::state::game_object::SacrificeFilter::Creature => chars
                    .card_types
                    .contains(&crate::state::types::CardType::Creature),
                crate::state::game_object::SacrificeFilter::Land => chars
                    .card_types
                    .contains(&crate::state::types::CardType::Land),
                crate::state::game_object::SacrificeFilter::Artifact => chars
                    .card_types
                    .contains(&crate::state::types::CardType::Artifact),
                crate::state::game_object::SacrificeFilter::ArtifactOrCreature => {
                    chars
                        .card_types
                        .contains(&crate::state::types::CardType::Artifact)
                        || chars
                            .card_types
                            .contains(&crate::state::types::CardType::Creature)
                }
                crate::state::game_object::SacrificeFilter::Subtype(sub) => {
                    chars.subtypes.contains(sub)
                }
                crate::state::game_object::SacrificeFilter::CreatureOfChosenType => {
                    // Must be a creature AND have the activating source's chosen_creature_type.
                    if !chars
                        .card_types
                        .contains(&crate::state::types::CardType::Creature)
                    {
                        false
                    } else {
                        let chosen = state
                            .objects
                            .get(&source)
                            .and_then(|o| o.chosen_creature_type.as_ref());
                        chosen
                            .map(|ct| chars.subtypes.contains(ct))
                            .unwrap_or(false)
                    }
                }
            };
            if !matches_filter {
                return Err(GameStateError::InvalidCommand(format!(
                    "sacrifice cost: permanent does not match required filter {:?} (CR 602.2)",
                    filter
                )));
            }
        }
        // Sacrifice the permanent (move to graveyard).
        // CR 603.10a / CR 613.1d: Capture full layer-resolved characteristics BEFORE zone move.
        // Consolidate into one calculate_characteristics call for type-check, power, and LKI snapshot.
        let (
            is_creature,
            owner,
            pre_death_controller,
            pre_death_counters,
            sac_filter_lki_power,
            sac_filter_pre_chars,
        ) = {
            let obj = state.object(sac_id)?;
            // CR 613.1f + CR 603.10a + CR 613.1e: Use layer-resolved card types for the
            // sacrificed permanent. Same reasoning as the sacrifice_self path above: a
            // permanent animated into a creature via Layer 4 must emit CreatureDied when
            // it is sacrificed as a cost, so "whenever a creature dies" triggers fire.
            // unwrap_or_else fallback handles LKI path (object not on battlefield).
            let pre_chars_opt = crate::rules::layers::calculate_characteristics(state, sac_id);
            let resolved = crate::rules::layers::expect_characteristics(state, sac_id);
            // CR 608.2b/608.2h/608.2i: Capture full LKI (power/toughness/mana value)
            // BEFORE the zone move (CR 400.7 kills old id after).
            let lki_power = resolved.power.or(obj.characteristics.power);
            let lki_toughness = resolved.toughness.or(obj.characteristics.toughness);
            let lki_mana_value = resolved
                .mana_cost
                .as_ref()
                .or(obj.characteristics.mana_cost.as_ref())
                .map(|c| c.mana_value())
                .unwrap_or(0);
            sacrificed_lki.push(crate::state::types::SacrificedCreatureLki {
                power: lki_power.unwrap_or(0),
                toughness: lki_toughness.unwrap_or(0),
                mana_value: lki_mana_value,
            });
            (
                resolved
                    .card_types
                    .contains(&crate::state::types::CardType::Creature),
                obj.owner,
                obj.controller,
                obj.counters.clone(),
                lki_power,
                // CR 603.10a / CR 613.1d: full LKI characteristics snapshot for filtered death triggers.
                pre_chars_opt,
            )
        };
        let (new_id, _) = state.move_object_to_zone(sac_id, ZoneId::Graveyard(owner))?;
        if is_creature {
            events.push(GameEvent::CreatureDied {
                object_id: sac_id,
                new_grave_id: new_id,
                controller: pre_death_controller,
                pre_death_counters,
                // CR 603.10a: LKI power snapshot for SourcePowerAtLastKnownInformation.
                pre_death_power: sac_filter_lki_power,
                pre_death_characteristics: sac_filter_pre_chars,
            });
        } else {
            events.push(GameEvent::PermanentDestroyed {
                object_id: sac_id,
                new_grave_id: new_id,
                // CR 603.10a: pass LKI counters for WhenLeavesBattlefield triggers.
                pre_lba_counters: pre_death_counters.clone(),
                // CR 603.10a: pass LKI power snapshot.
                pre_lba_power: sac_filter_lki_power,
            });
        }
        // CR 701.21a: PermanentSacrificed alongside death/destroy for sacrifice cost.
        events.push(GameEvent::PermanentSacrificed {
            player: pre_death_controller,
            object_id: sac_id,
            new_id,
        });
    }
    // CR 701.61a: Pay forage cost — "Exile three cards from your graveyard or sacrifice a Food."
    // Deterministic fallback (M9.5): prefer Food sacrifice when both options are available.
    if ability_cost.forage {
        // Collect Food artifacts controlled by this player on the battlefield (phased in).
        let food_subtype = crate::state::types::SubType("Food".to_string());
        let mut food_ids: Vec<ObjectId> = state
            .objects
            .iter()
            .filter_map(|(&id, obj)| {
                if obj.zone == ZoneId::Battlefield
                    && obj.controller == player
                    && obj.is_phased_in()
                    // PB-AC8 / CR 701.21a: a "can't be sacrificed" Food is not an
                    // eligible forage target.
                    && !crate::effects::object_cant_be_sacrificed(state, id)
                {
                    // Use layer-resolved characteristics to respect continuous effects.
                    let chars = crate::rules::layers::expect_characteristics(state, id);
                    if chars.subtypes.contains(&food_subtype) {
                        Some(id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        food_ids.sort(); // deterministic: smallest ObjectId first
                         // Collect graveyard cards for the exile-3 option.
        let mut grave_ids: Vec<ObjectId> = state
            .objects
            .iter()
            .filter_map(|(&id, obj)| {
                if obj.zone == ZoneId::Graveyard(player) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();
        grave_ids.sort(); // deterministic: smallest ObjectId first
        let has_food = !food_ids.is_empty();
        let has_three_grave = grave_ids.len() >= 3;
        if !has_food && !has_three_grave {
            return Err(GameStateError::InvalidCommand(
                "cannot forage: need a Food you control or 3+ cards in your graveyard (CR 701.61a)"
                    .into(),
            ));
        }
        if has_food {
            // Sacrifice a Food (deterministic: lowest ObjectId).
            let food_id = food_ids[0];
            let (owner, food_pre_lba) = state
                .expect_object(food_id)
                .map(|o| (o.owner, o.counters.clone()))
                .unwrap_or_else(|| (player, imbl::OrdMap::new()));
            // Food tokens are not creatures; no power LKI needed.
            let (new_grave_id, _) = state.move_object_to_zone(food_id, ZoneId::Graveyard(owner))?;
            events.push(GameEvent::PermanentDestroyed {
                object_id: food_id,
                new_grave_id,
                // CR 603.10a: pass LKI counters for WhenLeavesBattlefield triggers.
                pre_lba_counters: food_pre_lba,
                // Food tokens have no power.
                pre_lba_power: None,
            });
        } else {
            // Exile 3 cards from graveyard (deterministic: lowest ObjectId order).
            let to_exile: Vec<ObjectId> = grave_ids.into_iter().take(3).collect();
            for id in to_exile {
                let (new_exile_id, _) = state.move_object_to_zone(id, ZoneId::Exile)?;
                events.push(GameEvent::ObjectExiled {
                    player,
                    object_id: id,
                    new_exile_id,
                    // From graveyard — no LBA trigger LKI needed.
                    pre_lba_counters: imbl::OrdMap::new(),
                    // From graveyard — no battlefield power to snapshot.
                    pre_lba_power: None,
                });
            }
        }
    }
    // CR 602.2 / CR 118.3: Pay remove-counter cost.
    // The permanent must have at least `count` counters of the required type.
    // Counters are removed BEFORE the ability goes on the stack.
    if let Some((ref counter_type, count)) = ability_cost.remove_counter_cost {
        let current = state
            .objects
            .get(&source)
            .and_then(|obj| obj.counters.get(counter_type).copied())
            .unwrap_or(0);
        if current < count {
            return Err(GameStateError::InvalidCommand(format!(
                "remove-counter cost: need {} {:?} counter(s), have {} (CR 118.3)",
                count, counter_type, current
            )));
        }
        if let Some(obj) = state.expect_object_mut(source) {
            let new_count = current - count;
            if new_count == 0 {
                obj.counters.remove(counter_type);
            } else {
                obj.counters.insert(counter_type.clone(), new_count);
            }
        }
        events.push(crate::rules::events::GameEvent::CounterRemoved {
            object_id: source,
            counter: counter_type.clone(),
            count,
        });
    }
    // CR 602.2c: Validate targets for existence, hexproof, shroud, and protection.
    // Fetch source characteristics once for protection-from checks (CR 702.16b).
    let source_chars =
        crate::rules::layers::calculate_characteristics(state, source).or_else(|| {
            state
                .objects
                .get(&source)
                .map(|o| o.characteristics.clone())
        });
    for t in &targets {
        match t {
            Target::Object(id) => {
                // MR-M3-04: Non-existent object must be rejected, not silently skipped.
                let obj = state
                    .objects
                    .get(id)
                    .ok_or(GameStateError::ObjectNotFound(*id))?;
                // CR 702.11a / CR 702.18a / CR 702.16b: Hexproof, shroud, and protection.
                // CR 613.1f: Use layer-resolved keywords (Humility removes hexproof/shroud).
                let target_chars = crate::rules::layers::expect_characteristics(state, *id);
                super::validate_target_protection(
                    &target_chars.keywords,
                    obj.controller,
                    player,
                    source_chars.as_ref(),
                )?;
            }
            Target::Player(pid) => {
                // CR 702.11d: Player hexproof — can't be targeted by opponents' abilities.
                if player != *pid {
                    let player_has_hexproof = state.objects.values().any(|o| {
                        o.zone == ZoneId::Battlefield
                            && o.controller == *pid
                            && crate::rules::layers::calculate_characteristics(state, o.id)
                                .is_some_and(|chars| {
                                    chars.keywords.contains(
                                        &crate::state::types::KeywordAbility::HexproofPlayer,
                                    )
                                })
                    });
                    if player_has_hexproof {
                        return Err(GameStateError::InvalidTarget(format!(
                            "player {:?} has hexproof and cannot be targeted by opponents",
                            pid
                        )));
                    }
                }
            }
            // PB-DX52 (`OOS-DX25b-1`): an ability's stack entry. Existence is checked
            // here for the same reason MR-M3-04 made the object arm reject a
            // non-existent id rather than skip it -- a stale id must be an error, not a
            // silent no-op.
            //
            // NO protection/hexproof/shroud check is owed, and that is a CR reading
            // rather than an omission: CR 702.11b scopes hexproof to "this PERMANENT",
            // CR 702.18a scopes shroud the same way, and CR 702.16b's protection is a
            // property of a permanent, player or (for shroud) a spell. An ability on the
            // stack is none of those -- it has no controller-independent characteristics
            // of its own to carry a protection quality -- CR 113.7a is explicit that an
            // ability on the stack "exists on the stack independently of its source",
            // so it inherits none of the source's protection.
            Target::StackObject(id) => {
                if !state.stack_objects.iter().any(|so| so.id == *id) {
                    return Err(GameStateError::ObjectNotFound(*id));
                }
            }
        }
    }
    // Snapshot targets (zone recorded at activation time for fizzle check at resolution).
    let spell_targets: Vec<SpellTarget> = targets
        .iter()
        .map(|t| match t {
            Target::Player(id) => SpellTarget {
                target: Target::Player(*id),
                zone_at_cast: None,
            },
            Target::Object(id) => {
                let zone = state.expect_object(*id).map(|o| o.zone);
                SpellTarget {
                    target: Target::Object(*id),
                    zone_at_cast: zone,
                }
            }
            // PB-DX52: `zone_at_cast: None`, like a player target. A stack entry is not
            // in a zone the way a card is, so CR 608.2b legality for it is "still in
            // `state.stack_objects`" rather than "still in the zone it was in".
            Target::StackObject(id) => SpellTarget {
                target: Target::StackObject(*id),
                zone_at_cast: None,
            },
        })
        .collect();
    // Push the activated ability onto the stack.
    let stack_id = state.next_object_id();
    // CR 702.21a (PB-DX48): the Ward dispatch is derived by
    // `rules::events::push_target_announcement` from the stack object's own
    // `targets`, with the identical predicate this block used to spell out.
    // MR-TC-25: use trigger_default; override targets with the declared targets.
    let mut stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::ActivatedAbility {
            source_object: source,
            ability_index,
            embedded_effect: embedded_effect.map(Box::new),
        },
    );
    stack_obj.targets = spell_targets;
    // PB-DX25c §3.1: the same hoisted `announced_requirements` the validation above used.
    stack_obj.target_requirements = announced_requirements;
    // CR 700.2a (PB-EF7): Record the chosen mode(s) on the stack object for LKI/replay/hash
    // observability, even though approach (a) already baked them into `embedded_effect`.
    stack_obj.modes_chosen = validated_modes_chosen;
    // CR 107.3k: Propagate x_value so effects using EffectAmount::XValue resolve correctly.
    stack_obj.x_value = x_value.unwrap_or(0);
    // PB-P/PB-EF10: Carry captured LKI of cost-sacrificed creatures forward to
    // resolution, where EffectAmount::{PowerOf,ToughnessOf,ManaValueOf}SacrificedCreature
    // read them from EffectContext.
    stack_obj.sacrificed_creature_lki = sacrificed_lki;
    state.stack_objects.push_back(stack_obj);
    // CR 602.5b: Track once-per-turn activation for abilities with the restriction.
    if is_once_per_turn {
        if let Some(obj) = state.expect_object_mut(source) {
            obj.abilities_activated_this_turn = obj.abilities_activated_this_turn.saturating_add(1);
        }
    }
    // CR 602.2b -> 601.2i / CR 117.3c: the activating player receives priority afterward.
    // CR 117.4: reset the pass-round; an action was taken between passes.
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: source,
        stack_object_id: stack_id,
    });
    // ENG-2 (A1, CR 602.2b): announce the ability's targets, if any.
    super::events::push_target_announcement(state, &mut events, player, source, stack_id);
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
// ---------------------------------------------------------------------------
// Cycling handler
// ---------------------------------------------------------------------------
/// Handle a CycleCard command: validate, pay mana cost, discard self, push draw onto stack.
///
/// CR 702.29a: Cycling is an activated ability from hand. "[Cost], Discard this card: Draw a card."
/// The discard is part of the cost (happens immediately before ability goes on stack).
/// The draw uses the stack and can be responded to (e.g., Stifle).
///
/// CR 702.29b: The keyword exists in all zones, but activation is only legal from hand.
pub fn handle_cycle_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // CR 702.61a: Cycling is an activated ability, not a mana ability.
    // It cannot be activated while a spell with split second is on the stack.
    if crate::rules::casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; cycling cannot be activated (CR 702.61a)"
                .into(),
        ));
    }
    // 2. Zone check (CR 702.29a): card must be in Hand(player).
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Hand(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "CycleCard: card {:?} is not in Hand({:?}); cycling can only be activated from hand (CR 702.29a)",
                card, player
            )));
        }
    }
    // 3. Keyword check (CR 702.29a): card must have KeywordAbility::Cycling.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Cycling)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "CycleCard: card {:?} does not have the Cycling keyword (CR 702.29a)",
                card
            )));
        }
    }
    // 4. Look up cycling cost from CardRegistry (CR 702.29a).
    let card_id_opt = state.object(card)?.card_id.clone();
    let cycling_cost = get_cycling_cost(&card_id_opt, &state.card_registry.clone());
    // 5. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if let Some(ref cost) = cycling_cost {
        if cost.mana_value() > 0 {
            let player_state = state.player_mut(player)?;
            if !casting::can_pay_cost(&player_state.mana_pool, cost) {
                return Err(GameStateError::InsufficientMana);
            }
            casting::pay_cost(&mut player_state.mana_pool, cost);
            events.push(GameEvent::ManaCostPaid {
                player,
                cost: cost.clone(),
            });
        }
    }
    // 6. Discard self as cost (CR 702.29a): move card from hand to graveyard (or exile if madness).
    // This happens BEFORE the ability goes on the stack.
    // Capture owner before zone move (move_object_to_zone resets controller to owner).
    let owner = state.object(card)?.owner;
    // CR 702.35a: If the card has madness, exile instead of graveyard.
    let cycle_card_id_opt = state.object(card)?.card_id.clone();
    let has_madness = state
        .object(card)?
        .characteristics
        .keywords
        .contains(&KeywordAbility::Madness);
    let discard_destination = if has_madness {
        ZoneId::Exile
    } else {
        ZoneId::Graveyard(owner)
    };
    let (new_grave_id, _) = state.move_object_to_zone(card, discard_destination)?;
    // Emit CardDiscarded (CR 701.8 — discard is always announced, even when going to exile).
    events.push(GameEvent::CardDiscarded {
        player,
        object_id: card,
        new_id: new_grave_id,
    });
    // Emit CardCycled (CR 702.29a — distinct event for "when you cycle" trigger matching).
    events.push(GameEvent::CardCycled {
        player,
        object_id: card,
        new_id: new_grave_id,
    });
    // CR 702.35a: If madness applied, queue the madness trigger via pending_triggers
    // so it goes through flush_pending_triggers and properly signals priority granting.
    if has_madness {
        let madness_cost = cycle_card_id_opt.as_ref().and_then(|cid| {
            state.card_registry.get(cid.clone()).and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Madness { cost } = a {
                        Some(cost.clone())
                    } else {
                        None
                    }
                })
            })
        });
        state.pending_triggers.push_back(PendingTrigger {
            data: Some(TriggerData::Madness {
                exiled_card: new_grave_id,
                cost: madness_cost.unwrap_or_default(),
            }),
            ..PendingTrigger::blank(new_grave_id, player, PendingTriggerKind::Madness)
        });
    }
    // 7. Push cycling ability onto stack as ActivatedAbility with embedded DrawCards effect.
    // CR 602.2c: The source object (card) is now in the graveyard; source_object records
    // the retired ObjectId for reference. ability_index 0 is a placeholder.
    let stack_id = state.next_object_id();
    let draw_effect = crate::cards::card_definition::Effect::DrawCards {
        player: crate::cards::card_definition::PlayerTarget::Controller,
        count: crate::cards::card_definition::EffectAmount::Fixed(1),
    };
    // MR-TC-25: use trigger_default for boilerplate cast-specific fields.
    let stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::ActivatedAbility {
            source_object: card,
            ability_index: 0,
            embedded_effect: Some(Box::new(draw_effect)),
        },
    );
    state.stack_objects.push_back(stack_obj);
    // 8. CR 602.2b -> 601.2i / CR 117.3c: the activating player gets priority (CR 117.4:
    //    reset the pass-round).
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: card,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
/// CR 702.29a: Look up the cycling cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Cycling { cost }`, or `None`
/// if the card has no definition or no cycling ability defined. When `None` is returned,
/// no mana payment is required (free cycling, e.g., Street Wraith).
fn get_cycling_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Cycling { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
/// CR 702.59a: Look up the recover cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Recover { cost }`, or `None`
/// if the card has no definition or no recover ability defined.
fn find_recover_cost(
    card_id: &Option<crate::state::player::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Recover { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
// ---------------------------------------------------------------------------
// Forecast (CR 702.57)
// ---------------------------------------------------------------------------
/// Handle an ActivateForecast command: validate timing/zone/once-per-turn,
/// pay mana cost, push forecast ability onto stack.
///
/// CR 702.57a: Forecast is an activated ability from hand.
/// CR 702.57b: May only be activated during the upkeep step of the card's owner,
/// and only once each turn. The card is revealed but stays in hand.
pub fn handle_activate_forecast(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::state::turn::Step;
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // 2. Split second check (CR 702.61a): Forecast is an activated ability, not a mana
    //    ability. It cannot be activated while a spell with split second is on the stack.
    if crate::rules::casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; forecast cannot be activated (CR 702.61a)"
                .into(),
        ));
    }
    // 3. Upkeep check (CR 702.57b): only during the upkeep step.
    if state.turn.step != Step::Upkeep {
        return Err(GameStateError::InvalidCommand(format!(
            "ActivateForecast: forecast may only be activated during the upkeep step (CR 702.57b); \
             current step is {:?}",
            state.turn.step
        )));
    }
    // 4. Owner's upkeep check (CR 702.57b): the card's owner must be the active player.
    //    In multiplayer, only during the turn of the card's owner.
    if state.turn.active_player != player {
        return Err(GameStateError::InvalidCommand(format!(
            "ActivateForecast: forecast may only be activated during the owner's upkeep (CR 702.57b); \
             active player is {:?}, activating player is {:?}",
            state.turn.active_player, player
        )));
    }
    // 5. Zone check (CR 702.57a): card must be in Hand(player).
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Hand(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateForecast: card {:?} is not in Hand({:?}); \
                 forecast can only be activated from hand (CR 702.57a)",
                card, player
            )));
        }
    }
    // 6. Keyword check (CR 702.57a): card must have KeywordAbility::Forecast.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Forecast)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateForecast: card {:?} does not have the Forecast keyword (CR 702.57a)",
                card
            )));
        }
    }
    // 7. Once-per-turn check (CR 702.57b): card must not have already used forecast this turn.
    let card_id_opt = state.object(card)?.card_id.clone();
    if let Some(ref cid) = card_id_opt {
        if state.forecast_used_this_turn.contains(cid) {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateForecast: card {:?} has already activated its forecast this turn (CR 702.57b)",
                card
            )));
        }
    }
    // 8. Look up cost and effect from AbilityDefinition::Forecast in card registry.
    let registry = state.card_registry.clone();
    let (forecast_cost, forecast_effect) = card_id_opt
        .as_ref()
        .and_then(|cid| registry.get(cid.clone()))
        .and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Forecast { cost, effect } = a {
                    Some((cost.clone(), effect.clone()))
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "ActivateForecast: card {:?} has no AbilityDefinition::Forecast entry",
                card
            ))
        })?;
    // 9. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if forecast_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &forecast_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &forecast_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: forecast_cost,
        });
    }
    // 10. Mark forecast as used for this turn (CR 702.57b — once per card per turn).
    if let Some(cid) = card_id_opt {
        state.forecast_used_this_turn = state.forecast_used_this_turn.update(cid);
    }
    // 11. Push forecast ability onto stack.
    // The card stays in hand — no zone move.
    // Convert Vec<Target> → Vec<SpellTarget> capturing zone at activation time (CR 601.2c).
    let spell_targets: Vec<SpellTarget> = targets
        .into_iter()
        .map(|t| {
            let zone_at_cast = match &t {
                Target::Object(id) => state.objects.get(id).map(|obj| obj.zone),
                Target::Player(_) => None,
                // PB-DX52: a stack entry is not in a zone (see `Target::StackObject`).
                Target::StackObject(_) => None,
            };
            SpellTarget {
                target: t,
                zone_at_cast,
            }
        })
        .collect();
    let stack_id = state.next_object_id();
    // MR-TC-25: use trigger_default; override targets with forecast targets.
    let mut stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::ForecastAbility {
            source_object: card,
            embedded_effect: Box::new(forecast_effect),
        },
    );
    stack_obj.targets = spell_targets;
    // PB-DX25c §3.1: `AbilityDefinition::Forecast` carries no `TargetRequirement` list
    // at all (no `targets` field) — there is nothing to record, and none of the
    // corpus's Forecast abilities declares `targets` in practice.
    state.stack_objects.push_back(stack_obj);
    // 12. CR 602.2b -> 601.2i / CR 117.3c: the activating player gets priority (CR 117.4:
    //     reset the pass-round). (This handler is AP-gated above — owner's upkeep, CR
    //     702.57b — so this is an identity write today; it is written as `player` so the
    //     site stays correct if the gate is ever relaxed.)
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: card,
        stack_object_id: stack_id,
    });
    // ENG-2 (A3, CR 602.2b): announce the forecast ability's targets, if any.
    super::events::push_target_announcement(state, &mut events, player, card, stack_id);
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
// ---------------------------------------------------------------------------
// Bloodrush (CR 207.2c — ability word; underlying mechanics: CR 602)
// ---------------------------------------------------------------------------
/// Handle an ActivateBloodrush command: validate zone/target/mana, discard self
/// as cost, and push BloodrushAbility onto the stack.
///
/// CR 207.2c: Bloodrush is an ability word. The underlying ability is an activated
/// ability (CR 602) of the form:
/// "{cost}, Discard this card: Target attacking creature gets +N/+N
/// [and gains {keyword}] until end of turn."
///
/// Key rules:
/// - CR 602.2a: The card is in a hidden zone (hand); it is revealed during activation.
/// - CR 602.2b: The discard is the additional cost; paid before ability goes on stack.
/// - CR 115: "Target attacking creature" — target must be in `state.combat.attackers`.
/// - CR 702.61a: Cannot activate while split second is on the stack.
pub fn handle_activate_bloodrush(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    target: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // 2. Split second check (CR 702.61a): Bloodrush is an activated ability, not a mana
    //    ability. It cannot be activated while a spell with split second is on the stack.
    if crate::rules::casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; bloodrush cannot be activated (CR 702.61a)"
                .into(),
        ));
    }
    // 3. Zone check (CR 602.2a): card must be in Hand(player).
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Hand(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateBloodrush: card {:?} is not in Hand({:?}); \
                 bloodrush can only be activated from hand (CR 602.2a)",
                card, player
            )));
        }
    }
    // 4. AbilityDefinition check: card must have AbilityDefinition::Bloodrush.
    //    We look up from the card registry, not the characteristics keywords,
    //    because bloodrush is an ability word (not a KeywordAbility variant).
    let card_id_opt = state.object(card)?.card_id.clone();
    let registry = state.card_registry.clone();
    let bloodrush_def = card_id_opt
        .as_ref()
        .and_then(|cid| registry.get(cid.clone()))
        .and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Bloodrush {
                    cost,
                    power_boost,
                    toughness_boost,
                    grants_keyword,
                } = a
                {
                    Some((
                        cost.clone(),
                        *power_boost,
                        *toughness_boost,
                        grants_keyword.clone(),
                    ))
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "ActivateBloodrush: card {:?} has no AbilityDefinition::Bloodrush entry",
                card
            ))
        })?;
    let (bloodrush_cost, power_boost, toughness_boost, grants_keyword) = bloodrush_def;
    // 5. Target validation (CR 115): target must be on the battlefield as a creature
    //    AND currently registered as an attacker in CombatState.
    {
        let target_obj = state.objects.get(&target).ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "ActivateBloodrush: target {:?} does not exist",
                target
            ))
        })?;
        if !matches!(target_obj.zone, ZoneId::Battlefield) {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateBloodrush: target {:?} is not on the battlefield (CR 115)",
                target
            )));
        }
        if !target_obj
            .characteristics
            .card_types
            .contains(&crate::state::types::CardType::Creature)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateBloodrush: target {:?} is not a creature (CR 115)",
                target
            )));
        }
    }
    let is_attacking = state
        .combat
        .as_ref()
        .map(|c| c.attackers.contains_key(&target))
        .unwrap_or(false);
    if !is_attacking {
        return Err(GameStateError::InvalidCommand(format!(
            "ActivateBloodrush: target {:?} is not an attacking creature (CR 115). \
             Bloodrush requires 'target attacking creature'.",
            target
        )));
    }
    // 6. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if bloodrush_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &bloodrush_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &bloodrush_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: bloodrush_cost,
        });
    }
    // 7. Discard self as cost (CR 602.2b): the card goes to graveyard before
    //    the ability goes on the stack. Check for Madness first (CR 702.35a).
    let owner = state.object(card)?.owner;
    let has_madness = state
        .object(card)?
        .characteristics
        .keywords
        .contains(&KeywordAbility::Madness);
    let discard_destination = if has_madness {
        ZoneId::Exile
    } else {
        ZoneId::Graveyard(owner)
    };
    let (new_grave_id, _) = state.move_object_to_zone(card, discard_destination)?;
    // Emit CardDiscarded (CR 701.8).
    events.push(GameEvent::CardDiscarded {
        player,
        object_id: card,
        new_id: new_grave_id,
    });
    // Handle Madness if present (CR 702.35a): queue Madness trigger.
    if has_madness {
        let madness_cost = card_id_opt.as_ref().and_then(|cid| {
            state.card_registry.get(cid.clone()).and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Madness { cost } = a {
                        Some(cost.clone())
                    } else {
                        None
                    }
                })
            })
        });
        state
            .pending_triggers
            .push_back(crate::state::stubs::PendingTrigger {
                data: Some(TriggerData::Madness {
                    exiled_card: new_grave_id,
                    cost: madness_cost.unwrap_or_default(),
                }),
                ..PendingTrigger::blank(
                    new_grave_id,
                    player,
                    crate::state::stubs::PendingTriggerKind::Madness,
                )
            });
    }
    // 8. Push BloodrushAbility onto stack (CR 602.2c).
    //    The source card is now in the graveyard; source_object records the
    //    pre-discard ObjectId for attribution only.
    let stack_id = state.next_object_id();
    // MR-TC-25: use trigger_default; override targets with bloodrush target.
    let mut stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::BloodrushAbility {
            source_object: card,
            target_creature: target,
            power_boost,
            toughness_boost,
            grants_keyword,
        },
    );
    stack_obj.targets = vec![SpellTarget {
        target: Target::Object(target),
        zone_at_cast: state.expect_object(target).map(|o| o.zone),
    }];
    // PB-DX25c §3.1: Bloodrush's "target attacking creature" is validated ad-hoc
    // above (step 5), NOT through a `TargetRequirement` — no variant expresses
    // "attacking creature" — so there is nothing accurate to record here.
    state.stack_objects.push_back(stack_obj);
    // 9. CR 602.2b -> 601.2i / CR 117.3c: the activating player gets priority (CR 117.4:
    //    reset the pass-round). Bloodrush (CR 702.94a) has no active-player gate, so this
    //    flips: a non-active player who activates it retains priority afterward.
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: card,
        stack_object_id: stack_id,
    });
    // ENG-2 (A4, CR 602.2b): announce the bloodrush ability's (single, unfiltered)
    // target.
    super::events::push_target_announcement(state, &mut events, player, card, stack_id);
    // CR 702.21a (PB-DX48): emitted by `push_target_announcement` above, from the
    // stack object's own single `SpellTarget`. **Measured behavioural delta: none.**
    // The deleted push was unconditional, where the shared predicate requires
    // `zone_at_cast == Some(Battlefield)`; bloodrush's `zone_at_cast` is
    // `state.expect_object(target).map(|o| o.zone)` and its own step-5 validation
    // already refuses a target that is not an attacking creature, i.e. not on the
    // battlefield. And an event emitted for a non-battlefield object was inert
    // anyway: `check_triggers`'s `PermanentTargeted` arm re-reads the object and
    // requires `zone == Battlefield`.
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
// ---------------------------------------------------------------------------
// Unearth (CR 702.84)
// ---------------------------------------------------------------------------
/// Handle an UnearthCard command: validate, pay cost, push unearth ability onto stack.
///
/// CR 702.84a: Unearth is an activated ability from the graveyard.
/// "[Cost]: Return this card from your graveyard to the battlefield. It gains haste.
/// Exile it at the beginning of the next end step. If it would leave the battlefield,
/// exile it instead. Activate only as a sorcery."
pub fn handle_unearth_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // 2. Split second check (CR 702.61a): activated abilities cannot be used when
    //    a spell with split second is on the stack.
    if casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; unearth cannot be activated (CR 702.61a)"
                .into(),
        ));
    }
    // 3. Zone check (CR 702.84a): card must be in player's own graveyard.
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "UnearthCard: card {:?} is not in Graveyard({:?}); unearth can only be activated from your graveyard (CR 702.84a)",
                card, player
            )));
        }
    }
    // 4. Keyword check (CR 702.84a): card must have KeywordAbility::Unearth.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Unearth)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "UnearthCard: card {:?} does not have the Unearth keyword (CR 702.84a)",
                card
            )));
        }
    }
    // 5. Sorcery speed check (CR 702.84a: "activate only as a sorcery").
    //    Active player only, main phase only (PreCombatMain or PostCombatMain), empty stack.
    {
        use crate::state::turn::Step;
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "UnearthCard: unearth can only be activated during your own turn (CR 702.84a)"
                    .into(),
            ));
        }
        let step = state.turn.step;
        if step != Step::PreCombatMain && step != Step::PostCombatMain {
            return Err(GameStateError::InvalidCommand(
                "UnearthCard: unearth can only be activated during a main phase (CR 702.84a)"
                    .into(),
            ));
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "UnearthCard: unearth can only be activated with an empty stack (CR 702.84a)"
                    .into(),
            ));
        }
    }
    // 6. Look up unearth cost from CardRegistry.
    let card_id_opt = state.object(card)?.card_id.clone();
    let unearth_cost_opt = get_unearth_cost(&card_id_opt, &state.card_registry.clone());
    let unearth_cost = match unearth_cost_opt {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "UnearthCard: no unearth cost found in card definition (CR 702.84a)".into(),
            ));
        }
    };
    // 7. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if unearth_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &unearth_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &unearth_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: unearth_cost.clone(),
        });
    }
    // 8. Push the unearth ability onto the stack as UnearthAbility.
    //    The card stays in the graveyard until the ability resolves (unlike cycling
    //    where the card is discarded as a cost).
    let stack_id = state.next_object_id();
    // MR-TC-25: use trigger_default for boilerplate cast-specific fields.
    let stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::UnearthAbility {
            source_object: card,
        },
    );
    state.stack_objects.push_back(stack_obj);
    // 9. CR 602.2b -> 601.2i / CR 117.3c: the activating player gets priority (CR 117.4:
    //    reset the pass-round). (This handler is AP-gated above -- "activate only as a
    //    sorcery", CR 702.84a -- so this is an identity write today; it is written as
    //    `player` so the site stays correct if the gate is ever relaxed.)
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: card,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
/// CR 702.84a: Look up the unearth cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::AltCastAbility { kind: AltCostKind::Unearth, .. }`,
/// or `None` if the card has no definition or no unearth ability defined.
fn get_unearth_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility {
                    kind: AltCostKind::Unearth,
                    cost,
                    ..
                } = a
                {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
// ---------------------------------------------------------------------------
// Ninjutsu (CR 702.49)
// ---------------------------------------------------------------------------
/// Handle an ActivateNinjutsu command: validate, pay cost, return attacker to
/// hand as a cost, then push ninjutsu ability onto the stack.
///
/// CR 702.49a: Ninjutsu is an activated ability from hand.
/// CR 702.49c: May only be activated when an unblocked attacker exists.
/// CR 702.49d: Commander ninjutsu also functions from the command zone.
pub fn handle_ninjutsu(
    state: &mut GameState,
    player: PlayerId,
    ninja_card: ObjectId,
    attacker_to_return: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // 2. Split second check (CR 702.61a): activated abilities cannot be used when
    //    a spell with split second is on the stack.
    if casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; ninjutsu cannot be activated (CR 702.61a)"
                .into(),
        ));
    }
    // 3. Combat phase + step check (CR 702.49c): must be in combat phase,
    //    at DeclareBlockers or later (not DeclareAttackers or BeginningOfCombat --
    //    before blockers are declared, creatures are neither blocked nor unblocked).
    {
        use crate::state::turn::Step;
        let step = state.turn.step;
        let valid_step = matches!(
            step,
            Step::DeclareBlockers
                | Step::FirstStrikeDamage
                | Step::CombatDamage
                | Step::EndOfCombat
        );
        if !valid_step {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateNinjutsu: ninjutsu can only be activated during DeclareBlockers, \
                 FirstStrikeDamage, CombatDamage, or EndOfCombat steps (CR 702.49c); \
                 current step is {:?}",
                step
            )));
        }
    }
    // 4. Combat state must exist (safety check).
    if state.combat.is_none() {
        return Err(GameStateError::InvalidCommand(
            "ActivateNinjutsu: no active combat state (CR 702.49c)".into(),
        ));
    }
    // 5. Zone check (CR 702.49a/d): ninja card must be in player's hand, OR,
    //    if it has CommanderNinjutsu, in the command zone ZoneId::Command(player).
    //    CRITICAL: ZoneId::Command(player), NOT ZoneId::CommandZone.
    let ninja_zone = {
        let obj = state.object(ninja_card)?;
        obj.zone
    };
    let has_commander_ninjutsu = state
        .object(ninja_card)?
        .characteristics
        .keywords
        .contains(&KeywordAbility::CommanderNinjutsu);
    let in_hand = ninja_zone == ZoneId::Hand(player);
    let in_command_zone = has_commander_ninjutsu && ninja_zone == ZoneId::Command(player);
    if !in_hand && !in_command_zone {
        return Err(GameStateError::InvalidCommand(format!(
            "ActivateNinjutsu: ninja card {:?} is not in hand or command zone (CR 702.49a/d)",
            ninja_card
        )));
    }
    let from_command_zone = in_command_zone;
    // 6. Keyword check (CR 702.49a/d): card must have Ninjutsu or CommanderNinjutsu.
    {
        let obj = state.object(ninja_card)?;
        let has_ninjutsu = obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Ninjutsu);
        if !has_ninjutsu && !has_commander_ninjutsu {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateNinjutsu: card {:?} does not have Ninjutsu or CommanderNinjutsu keyword \
                 (CR 702.49a)",
                ninja_card
            )));
        }
    }
    // 7. Attacker validation (CR 702.49c): attacker must be on battlefield,
    //    controlled by player, in combat.attackers, and unblocked.
    {
        let obj = state.object(attacker_to_return)?;
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::InvalidCommand(
                "ActivateNinjutsu: attacker is not on the battlefield".into(),
            ));
        }
        if obj.controller != player {
            return Err(GameStateError::InvalidCommand(
                "ActivateNinjutsu: attacker is not controlled by the activating player".into(),
            ));
        }
    }
    let combat = state.combat.as_ref().ok_or_else(|| {
        GameStateError::InvalidCommand("ActivateNinjutsu: no active combat state".into())
    })?;
    if !combat.attackers.contains_key(&attacker_to_return) {
        return Err(GameStateError::InvalidCommand(
            "ActivateNinjutsu: attacker is not an attacking creature (CR 702.49c)".into(),
        ));
    }
    if combat.is_blocked(attacker_to_return) {
        return Err(GameStateError::InvalidCommand(
            "ActivateNinjutsu: attacker is blocked; ninjutsu requires an unblocked attacker \
             (CR 702.49c)"
                .into(),
        ));
    }
    // 8. Capture attack target BEFORE returning the attacker (CR 702.49c):
    //    the ninja inherits the attack target of the returned creature.
    let attack_target = state
        .combat
        .as_ref()
        .and_then(|c| c.attackers.get(&attacker_to_return).cloned())
        .ok_or_else(|| {
            GameStateError::InvalidCommand(
                "ActivateNinjutsu: could not retrieve attack target from combat state".into(),
            )
        })?;
    // 9. Cost lookup: find AbilityDefinition::Ninjutsu or ::CommanderNinjutsu.
    let card_id_opt = state.object(ninja_card)?.card_id.clone();
    let ninjutsu_cost_opt = get_ninjutsu_cost(&card_id_opt, &state.card_registry.clone());
    let ninjutsu_cost = match ninjutsu_cost_opt {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "ActivateNinjutsu: no ninjutsu cost found in card definition (CR 702.49a)".into(),
            ));
        }
    };
    // 10. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if ninjutsu_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &ninjutsu_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &ninjutsu_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: ninjutsu_cost.clone(),
        });
    }
    // 11. Return attacker to its OWNER's hand (cost, CR 702.49a).
    //     "Return an unblocked attacking creature you control to its owner's hand."
    //     NOT the controller's hand -- in multiplayer theft, the attacker goes
    //     to the original owner's hand.
    let (attacker_owner, ninja_pre_lba, ninja_lki_power) = state
        .expect_object(attacker_to_return)
        .map(|o| {
            let lki_power =
                crate::rules::layers::calculate_characteristics(state, attacker_to_return)
                    .and_then(|c| c.power)
                    .or(o.characteristics.power);
            (o.owner, o.counters.clone(), lki_power)
        })
        .unwrap_or((state.turn.active_player, imbl::OrdMap::new(), None));
    let (new_hand_id, _old) =
        state.move_object_to_zone(attacker_to_return, ZoneId::Hand(attacker_owner))?;
    // Remove attacker from combat.attackers: move_object_to_zone doesn't touch
    // CombatState, so the old ObjectId is now stale (CR 400.7) and must be removed.
    if let Some(combat) = state.combat.as_mut() {
        combat.attackers.remove(&attacker_to_return);
    }
    events.push(GameEvent::ObjectReturnedToHand {
        player: attacker_owner,
        object_id: attacker_to_return,
        new_hand_id,
        pre_lba_counters: ninja_pre_lba,
        // CR 603.10a: LKI power snapshot for SourcePowerAtLastKnownInformation on bounce triggers.
        pre_lba_power: ninja_lki_power,
    });
    // 12. Push ninjutsu ability onto stack as NinjutsuAbility.
    let stack_id = state.next_object_id();
    // MR-TC-25: use trigger_default for boilerplate cast-specific fields.
    let stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::NinjutsuAbility {
            source_object: ninja_card,
            ninja_card,
            attack_target: attack_target.clone(),
            from_command_zone,
        },
    );
    state.stack_objects.push_back(stack_obj);
    // 13. CR 602.2b -> 601.2i / CR 117.3c: the activating player gets priority (CR 117.4:
    //     reset the pass-round). Ninjutsu (CR 702.49a) has no active-player gate, so this
    //     flips in principle (in practice it needs an unblocked attacker you control, so
    //     it is effectively AP already, but the write follows the actor, not the gate).
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: ninja_card,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
/// CR 702.49a: Look up the ninjutsu cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Ninjutsu { cost }` or
/// `AbilityDefinition::CommanderNinjutsu { cost }`, or `None` if the card has
/// no definition or no ninjutsu ability defined.
fn get_ninjutsu_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| match a {
                AbilityDefinition::Ninjutsu { cost }
                | AbilityDefinition::CommanderNinjutsu { cost } => Some(cost.clone()),
                _ => None,
            })
        })
    })
}
// ---------------------------------------------------------------------------
// Embalm (CR 702.128)
// ---------------------------------------------------------------------------
/// Handle an EmbalmCard command: validate, pay cost, exile card, push embalm ability
/// onto the stack.
///
/// CR 702.128a: Embalm is an activated ability from the graveyard.
/// "[Cost], Exile this card from your graveyard: Create a token that's a copy of
/// this card, except it's white, it has no mana cost, and it's a Zombie in addition
/// to its other types. Activate only as a sorcery."
///
/// KEY DIFFERENCE FROM UNEARTH: the card is exiled as part of the activation cost
/// (before the ability goes on the stack), not when the ability resolves.
pub fn handle_embalm_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // 2. Split second check (CR 702.61a): activated abilities cannot be used when
    //    a spell with split second is on the stack.
    if casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; embalm cannot be activated (CR 702.61a)"
                .into(),
        ));
    }
    // 3. Zone check (CR 702.128a): card must be in player's own graveyard.
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "EmbalmCard: card {:?} is not in Graveyard({:?}); embalm can only be activated from your graveyard (CR 702.128a)",
                card, player
            )));
        }
    }
    // 4. Keyword check (CR 702.128a): card must have KeywordAbility::Embalm.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Embalm)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "EmbalmCard: card {:?} does not have the Embalm keyword (CR 702.128a)",
                card
            )));
        }
    }
    // 5. Sorcery speed check (CR 702.128a: "activate only as a sorcery").
    //    Active player only, main phase only (PreCombatMain or PostCombatMain), empty stack.
    {
        use crate::state::turn::Step;
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "EmbalmCard: embalm can only be activated during your own turn (CR 702.128a)"
                    .into(),
            ));
        }
        let step = state.turn.step;
        if step != Step::PreCombatMain && step != Step::PostCombatMain {
            return Err(GameStateError::InvalidCommand(
                "EmbalmCard: embalm can only be activated during a main phase (CR 702.128a)".into(),
            ));
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "EmbalmCard: embalm can only be activated with an empty stack (CR 702.128a)".into(),
            ));
        }
    }
    // 6. Look up embalm cost from CardRegistry.
    let card_id_opt = state.object(card)?.card_id.clone();
    let embalm_cost_opt = get_embalm_cost(&card_id_opt, &state.card_registry.clone());
    let embalm_cost = match embalm_cost_opt {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "EmbalmCard: no embalm cost found in card definition (CR 702.128a)".into(),
            ));
        }
    };
    // 7. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if embalm_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &embalm_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &embalm_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: embalm_cost.clone(),
        });
    }
    // 8. Capture the card_id BEFORE exiling (object identity is reset on zone change,
    //    CR 400.7 -- but card_id is the registry key and survives the move).
    //    We need it for EmbalmAbility so resolution can find the CardDefinition.
    let source_card_id = state.object(card)?.card_id.clone();
    // 9. Exile the card from graveyard as cost payment (CR 702.128a: "[Cost], Exile
    //    this card from your graveyard"). CRITICAL DIFFERENCE FROM UNEARTH:
    //    the card is exiled immediately as part of cost payment, not at resolution.
    //    Ruling 2017-07-14: "Once you've activated an embalm ability, the card is
    //    immediately exiled. Opponents can't try to stop the ability by exiling the
    //    card with an effect."
    let (exile_id, _old) = state.move_object_to_zone(card, ZoneId::Exile)?;
    events.push(GameEvent::ObjectExiled {
        player,
        object_id: card,
        new_exile_id: exile_id,
        pre_lba_counters: imbl::OrdMap::new(), // graveyard→exile: no battlefield counters
        pre_lba_power: None,                   // graveyard→exile: no battlefield power to snapshot
    });
    // 10. Push the embalm ability onto the stack as EmbalmAbility.
    //     We store source_card_id (the registry key) instead of the ObjectId
    //     because the card's ObjectId is now dead (zone change, CR 400.7).
    let stack_id = state.next_object_id();
    // MR-TC-25: use trigger_default for boilerplate cast-specific fields.
    let stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::EmbalmAbility { source_card_id },
    );
    state.stack_objects.push_back(stack_obj);
    // 11. CR 602.2b -> 601.2i / CR 117.3c: the activating player gets priority (CR 117.4:
    //     reset the pass-round). (This handler is AP-gated above -- "activate only as a
    //     sorcery", CR 702.128a -- so this is an identity write today; it is written as
    //     `player` so the site stays correct if the gate is ever relaxed.)
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: card,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
/// CR 702.128a: Look up the embalm cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::AltCastAbility { kind: AltCostKind::Embalm, .. }`,
/// or `None` if the card has no definition or no embalm ability defined.
fn get_embalm_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility {
                    kind: AltCostKind::Embalm,
                    cost,
                    ..
                } = a
                {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
// ---------------------------------------------------------------------------
// Eternalize (CR 702.129)
// ---------------------------------------------------------------------------
/// Handle an EternalizeCard command: validate, pay cost, exile card, push eternalize ability
/// onto the stack.
///
/// CR 702.129a: Eternalize is an activated ability from the graveyard.
/// "[Cost], Exile this card from your graveyard: Create a token that's a copy of
/// this card, except it's black, it's 4/4, it has no mana cost, and it's a Zombie
/// in addition to its other types. Activate only as a sorcery."
///
/// KEY DIFFERENCE FROM UNEARTH: the card is exiled as part of the activation cost
/// (before the ability goes on the stack), not when the ability resolves.
pub fn handle_eternalize_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // 2. Split second check (CR 702.61a): activated abilities cannot be used when
    //    a spell with split second is on the stack.
    if casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; eternalize cannot be activated (CR 702.61a)"
                .into(),
        ));
    }
    // 3. Zone check (CR 702.129a): card must be in player's own graveyard.
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "EternalizeCard: card {:?} is not in Graveyard({:?}); eternalize can only be activated from your graveyard (CR 702.129a)",
                card, player
            )));
        }
    }
    // 4. Keyword check (CR 702.129a): card must have KeywordAbility::Eternalize.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Eternalize)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "EternalizeCard: card {:?} does not have the Eternalize keyword (CR 702.129a)",
                card
            )));
        }
    }
    // 5. Sorcery speed check (CR 702.129a: "activate only as a sorcery").
    //    Active player only, main phase only (PreCombatMain or PostCombatMain), empty stack.
    {
        use crate::state::turn::Step;
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "EternalizeCard: eternalize can only be activated during your own turn (CR 702.129a)"
                    .into(),
            ));
        }
        let step = state.turn.step;
        if step != Step::PreCombatMain && step != Step::PostCombatMain {
            return Err(GameStateError::InvalidCommand(
                "EternalizeCard: eternalize can only be activated during a main phase (CR 702.129a)"
                    .into(),
            ));
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "EternalizeCard: eternalize can only be activated with an empty stack (CR 702.129a)"
                    .into(),
            ));
        }
    }
    // 6. Look up eternalize cost from CardRegistry.
    let card_id_opt = state.object(card)?.card_id.clone();
    let eternalize_cost_opt = get_eternalize_cost(&card_id_opt, &state.card_registry.clone());
    let eternalize_cost = match eternalize_cost_opt {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "EternalizeCard: no eternalize cost found in card definition (CR 702.129a)".into(),
            ));
        }
    };
    // 7. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if eternalize_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &eternalize_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &eternalize_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: eternalize_cost.clone(),
        });
    }
    // 8. Capture the card_id and name BEFORE exiling (object identity is reset on zone
    //    change, CR 400.7 -- but card_id is the registry key and survives the move).
    //    We need both for EternalizeAbility so resolution can find the CardDefinition
    //    and the TUI can display the card name.
    let source_card_id = state.object(card)?.card_id.clone();
    let source_name = state.object(card)?.characteristics.name.clone();
    // 9. Exile the card from graveyard as cost payment (CR 702.129a: "[Cost], Exile
    //    this card from your graveyard"). CRITICAL DIFFERENCE FROM UNEARTH:
    //    the card is exiled immediately as part of cost payment, not at resolution.
    //    Ruling 2017-07-14: "Once you've activated an eternalize ability, the card is
    //    immediately exiled. Opponents can't try to stop the ability by exiling the
    //    card with an effect."
    let (exile_id, _old) = state.move_object_to_zone(card, ZoneId::Exile)?;
    events.push(GameEvent::ObjectExiled {
        player,
        object_id: card,
        new_exile_id: exile_id,
        pre_lba_counters: imbl::OrdMap::new(), // graveyard→exile: no battlefield counters
        pre_lba_power: None,                   // graveyard→exile: no battlefield power to snapshot
    });
    // 10. Push the eternalize ability onto the stack as EternalizeAbility.
    //     We store source_card_id (the registry key) instead of the ObjectId
    //     because the card's ObjectId is now dead (zone change, CR 400.7).
    //     We also store source_name for TUI display purposes.
    let stack_id = state.next_object_id();
    // MR-TC-25: use trigger_default for boilerplate cast-specific fields.
    let stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::EternalizeAbility {
            source_card_id,
            source_name,
        },
    );
    state.stack_objects.push_back(stack_obj);
    // 11. CR 602.2b -> 601.2i / CR 117.3c: the activating player gets priority (CR 117.4:
    //     reset the pass-round). (This handler is AP-gated above -- "activate only as a
    //     sorcery", CR 702.129a -- so this is an identity write today; it is written as
    //     `player` so the site stays correct if the gate is ever relaxed.)
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: card,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
/// CR 702.129a: Look up the eternalize cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::AltCastAbility { kind: AltCostKind::Eternalize, .. }`,
/// or `None` if the card has no definition or no eternalize ability defined.
fn get_eternalize_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility {
                    kind: AltCostKind::Eternalize,
                    cost,
                    ..
                } = a
                {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
// ---------------------------------------------------------------------------
// Encore (CR 702.141)
// ---------------------------------------------------------------------------
/// Handle an EncoreCard command: validate, pay cost, exile card, push encore ability
/// onto the stack.
///
/// CR 702.141a: Encore is an activated ability from the graveyard.
/// "[Cost], Exile this card from your graveyard: For each opponent, create a token
/// that's a copy of this card that attacks that opponent this turn if able. The tokens
/// gain haste. Sacrifice them at the beginning of the next end step. Activate only
/// as a sorcery."
///
/// KEY DIFFERENCE FROM UNEARTH: the card is exiled as part of the activation cost
/// (before the ability goes on the stack), not when the ability resolves.
/// KEY DIFFERENCE FROM EMBALM/ETERNALIZE: tokens copy original characteristics without
/// modification (no color change, no P/T change, no type addition).
pub fn handle_encore_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // 2. Split second check (CR 702.61a): activated abilities cannot be used when
    //    a spell with split second is on the stack.
    if casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; encore cannot be activated (CR 702.61a)"
                .into(),
        ));
    }
    // 3. Zone check (CR 702.141a): card must be in player's own graveyard.
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "EncoreCard: card {:?} is not in Graveyard({:?}); encore can only be activated from your graveyard (CR 702.141a)",
                card, player
            )));
        }
    }
    // 4. Keyword check (CR 702.141a): card must have KeywordAbility::Encore.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Encore)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "EncoreCard: card {:?} does not have the Encore keyword (CR 702.141a)",
                card
            )));
        }
    }
    // 5. Sorcery speed check (CR 702.141a: "activate only as a sorcery").
    //    Active player only, main phase only (PreCombatMain or PostCombatMain), empty stack.
    {
        use crate::state::turn::Step;
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "EncoreCard: encore can only be activated during your own turn (CR 702.141a)"
                    .into(),
            ));
        }
        let step = state.turn.step;
        if step != Step::PreCombatMain && step != Step::PostCombatMain {
            return Err(GameStateError::InvalidCommand(
                "EncoreCard: encore can only be activated during a main phase (CR 702.141a)".into(),
            ));
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "EncoreCard: encore can only be activated with an empty stack (CR 702.141a)".into(),
            ));
        }
    }
    // 6. Look up encore cost from CardRegistry.
    let card_id_opt = state.object(card)?.card_id.clone();
    let encore_cost_opt = get_encore_cost(&card_id_opt, &state.card_registry.clone());
    let encore_cost = match encore_cost_opt {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "EncoreCard: no encore cost found in card definition (CR 702.141a)".into(),
            ));
        }
    };
    // 7. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if encore_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &encore_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &encore_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: encore_cost.clone(),
        });
    }
    // 8. Capture the card_id BEFORE exiling (object identity is reset on zone change,
    //    CR 400.7 -- but card_id is the registry key and survives the move).
    //    We need it for EncoreAbility so resolution can find the CardDefinition.
    let source_card_id = state.object(card)?.card_id.clone();
    // 9. Exile the card from graveyard as cost payment (CR 702.141a: "[Cost], Exile
    //    this card from your graveyard"). CRITICAL DIFFERENCE FROM UNEARTH:
    //    the card is exiled immediately as part of cost payment, not at resolution.
    //    Ruling: "Once you've activated an encore ability, the card is
    //    immediately exiled. Opponents can't try to stop the ability by exiling the
    //    card with an effect."
    let (exile_id, _old) = state.move_object_to_zone(card, ZoneId::Exile)?;
    events.push(GameEvent::ObjectExiled {
        player,
        object_id: card,
        new_exile_id: exile_id,
        pre_lba_counters: imbl::OrdMap::new(), // graveyard→exile: no battlefield counters
        pre_lba_power: None,                   // graveyard→exile: no battlefield power to snapshot
    });
    // 10. Push the encore ability onto the stack as EncoreAbility.
    //     We store source_card_id (the registry key) instead of the ObjectId
    //     because the card's ObjectId is now dead (zone change, CR 400.7).
    //     We also store the activator to determine token targets at resolution.
    let stack_id = state.next_object_id();
    // MR-TC-25: use trigger_default for boilerplate cast-specific fields.
    let stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::EncoreAbility {
            source_card_id,
            activator: player,
        },
    );
    state.stack_objects.push_back(stack_obj);
    // 11. CR 602.2b -> 601.2i / CR 117.3c: the activating player gets priority (CR 117.4:
    //     reset the pass-round). (This handler is AP-gated above -- "activate only as a
    //     sorcery", CR 702.141a -- so this is an identity write today; it is written as
    //     `player` so the site stays correct if the gate is ever relaxed.)
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: card,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
/// CR 702.141a: Look up the encore cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::AltCastAbility { kind: AltCostKind::Encore, .. }`,
/// or `None` if the card has no definition or no encore ability defined.
fn get_encore_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility {
                    kind: AltCostKind::Encore,
                    cost,
                    ..
                } = a
                {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
// ---------------------------------------------------------------------------
// Trigger checking
// ---------------------------------------------------------------------------
/// Scan all permanents for triggered abilities that fire in response to `events`.
///
/// Called after any batch of events. Returns `PendingTrigger` entries for each
/// ability that triggered. Does NOT modify state — caller pushes results into
/// `state.pending_triggers`.
///
/// CR 603.2: A triggered ability triggers whenever the trigger event occurs
/// and the trigger condition is met.
/// CR 603.4: If an intervening-if clause is present, the condition is checked
/// at trigger time; the ability only queues if the condition is true.
/// Whether a `check_triggers` caller's `events` slice is ONE simultaneous batch or a
/// SEQUENCE of events that happened one after another (PB-DX15a, rider `OOS-DX24-7`).
///
/// This distinction is load-bearing for exactly one thing: the CR 603.10a look-back set
/// (`arrived_in_graveyard_this_batch`) that suppresses a `trigger_zone: Graveyard`
/// ability whose source arrived in the graveyard as part of the same event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventBatchTiming {
    /// Every event in the slice happened at the SAME time, so CR 603.10a's "immediately
    /// prior" means prior to ALL of them. A CR 704.3 state-based-action fixpoint pass is
    /// the canonical example, and the Gatherer ruling this rule is derived from is
    /// explicitly about it: *"If Nether Traitor and another creature are put into your
    /// graveyard **at the same time**, Nether Traitor's ability won't trigger."*
    Simultaneous,
    /// The events happened one after another, so CR 603.10a's "immediately prior" means
    /// prior to THIS event — i.e. each event looks back only at deaths strictly earlier
    /// in the slice. A spell resolution whose sub-effects run in sequence is the case.
    Sequential,
}
/// [`check_triggers_with_timing`] with [`EventBatchTiming::Simultaneous`] — the
/// behaviour every caller had before PB-DX15a.
///
/// Kept as the short form because ~40 primitive-level tests call it to exercise trigger
/// collection directly, and for those the whole-slice look-back set is both the historical
/// and the correct reading (they hand in one simultaneous event batch). **Production code
/// should call [`check_triggers_with_timing`] and name its timing**, so that the choice is
/// visible at the call site rather than inherited; all six production sites do.
pub fn check_triggers(state: &GameState, events: &[GameEvent]) -> Vec<PendingTrigger> {
    check_triggers_with_timing(state, events, EventBatchTiming::Simultaneous)
}
pub fn check_triggers_with_timing(
    state: &GameState,
    events: &[GameEvent],
    timing: EventBatchTiming,
) -> Vec<PendingTrigger> {
    let mut triggers = Vec::new();
    // CR 603.10a (PB-DX24): a leaves-the-battlefield ability looks back in time --
    // the game asks whether the ability EXISTED immediately prior to the event. A
    // card that arrived in a graveyard as part of THIS event batch was NOT yet a
    // functioning graveyard-zone ability immediately prior to the event (CR 113.6m).
    // Gatherer, Nether Traitor: "If Nether Traitor and another creature are put
    // into your graveyard at the same time, Nether Traitor's ability won't
    // trigger."
    //
    // Scope (runner note, plan §3.2): built from every event in this batch that
    // shares `collect_graveyard_carddef_triggers`'s `new_grave_id` field name --
    // CreatureDied, PlaneswalkerDied, PermanentDestroyed, AuraFellOff,
    // ObjectPutInGraveyard. `PermanentSacrificed` is deliberately EXCLUDED: its
    // field is `new_id`, not `new_grave_id` (a real name mismatch against the
    // plan's literal list), and every creature-sacrifice call site that emits it
    // ALSO emits `CreatureDied` with the IDENTICAL id in the same push
    // (e.g. `abilities.rs:966-992`, `casting.rs:4340-4344` alongside `:4321`) --
    // so including it would only ever re-insert an id already in the set.
    // Mill/discard/cycle events (`CardMilled`, `CardDiscarded`, `CardCycled`) are
    // deliberately EXCLUDED too: those move a card into a graveyard from hand or
    // library, never from the battlefield, and this arm's `WheneverCreatureDies`
    // dispatch only ever fires on `CreatureDied` -- so a card whose OWN
    // `trigger_zone` ability needed that coverage does not exist in the corpus
    // today (measured: the `trigger_zone: Some(_)` population is exactly 3 defs,
    // stage 1). Filed as a seed rather than widened further.
    //
    // Per-caller GRANULARITY (fix cycle, review Finding 3 -- plan §10 risk #2,
    // not discharged at ship time): `arrived_in_graveyard_this_batch` is built
    // fresh from whichever `events` slice THIS caller passes, so its accuracy as
    // "one CR 603.10a simultaneous batch" depends entirely on what that caller
    // considers one batch. Measured by enumerating every `check_triggers` call
    // site:
    //   - `sba.rs:97` -- EXACT. `events` is `apply_sbas_once`'s own return value
    //     for ONE fixpoint pass, so this is precisely one CR 704.3 simultaneous
    //     SBA batch.
    //   - `resolution.rs` (the post-resolution call, `abilities::check_triggers`
    //     inside stack-object resolution) -- COARSER. `events` there is the
    //     WHOLE resolution's accumulated events vec, spanning every sequential
    //     sub-effect of one spell/ability resolution. A resolution whose effects
    //     read "sacrifice a creature, THEN destroy target creature" pushes both
    //     deaths into ONE `events` vec; if the sacrificed creature is itself a
    //     `trigger_zone: Graveyard` source (Nether Traitor's shape), its
    //     graveyard id lands in the look-back set from the FIRST sub-effect,
    //     wrongly suppressing what should be a live trigger off the SECOND
    //     sub-effect's death (CR 603.10a asks whether the ability existed
    //     immediately prior to THAT event -- and by then it already did,
    //     having arrived earlier in the SAME resolution). Direction:
    //     over-suppression. Filed as `OOS-DX24-7`.
    //   - `combat.rs:846`/`:1743`, `engine.rs:34`/`:2499` -- NOT audited by this
    //     batch (out of the plan's scope; `engine.rs:34` in particular is
    //     `check_and_flush_triggers`, shared by nearly every `Command` arm, so
    //     its own granularity is a per-command-handler question that would need
    //     its own investigation).
    //
    // PB-DX15a (rider `OOS-DX24-7`) — the per-caller granularity above is now a
    // PARAMETER rather than a property of whichever slice a caller happened to hand in.
    //
    // **The row's own prescribed fix was "rebuild the set per event PREFIX rather than
    // per whole slice". Taken literally that makes the `sba.rs` caller WRONG**, and it
    // is the caller the guard was written for. In one CR 704.3 fixpoint pass the deaths
    // are genuinely simultaneous, so CR 603.10a's "immediately prior" means prior to all
    // of them — which is exactly the Gatherer ruling quoted above ("at the same time").
    // A prefix set there would let a `trigger_zone: Graveyard` source fire off a death
    // whose event merely happens to sort AFTER its own in the slice: a correct answer or
    // a wrong one depending on event order within a batch that has no order.
    //
    // So: `Simultaneous` keeps the whole-slice set (byte-identical to pre-PB-DX15a
    // behaviour) and `Sequential` rebuilds per prefix. Of the six call sites, only
    // `resolution.rs` passes `Sequential`; the four sites PB-DX24 recorded as NOT
    // audited (`combat.rs` ×2, `engine.rs` ×2) pass `Simultaneous`, which is precisely
    // what they did before, so this change moves no behaviour anywhere it was not
    // measured. Naming the parameter is what makes their unaudited status visible at the
    // call site instead of buried in a comment here.
    fn graveyard_arrival_id(event: &GameEvent) -> Option<ObjectId> {
        match event {
            GameEvent::CreatureDied { new_grave_id, .. }
            | GameEvent::PlaneswalkerDied { new_grave_id, .. }
            | GameEvent::PermanentDestroyed { new_grave_id, .. }
            | GameEvent::AuraFellOff { new_grave_id, .. }
            | GameEvent::ObjectPutInGraveyard { new_grave_id, .. } => Some(*new_grave_id),
            _ => None,
        }
    }
    // `BTreeSet`, not `HashSet` (PB-DX7's `unordered_iteration_ratchet` fired on the
    // first draft of this and was right to). These three sets are `contains`-only
    // suppression sets, so `HashSet` would have been category (a) and legal — but a
    // `BTreeSet` costs nothing at this size, keeps the ratchet moving DOWN rather than
    // needing its ceiling raised, and removes the question from a function PB-DP9
    // re-executes wholesale after every suspended choice (`OOS-DP9-10`).
    let whole_batch_arrivals: std::collections::BTreeSet<ObjectId> =
        events.iter().filter_map(graveyard_arrival_id).collect();
    // Arrivals from events STRICTLY EARLIER than the one being handled. Grown at the
    // bottom of the loop; unused when `timing` is `Simultaneous`.
    let mut earlier_arrivals: std::collections::BTreeSet<ObjectId> =
        std::collections::BTreeSet::new();
    for event in events {
        // `OOS-DX24-7`'s own fix sketch says *"rebuild the set per event PREFIX"*.
        // **That is inverted, and this batch is where it was caught.** The set is a
        // SUPPRESSION set: a source in it did NOT yet have a functioning graveyard
        // ability immediately prior to the event. A source that arrived at an EARLIER
        // event was already there, so it must be REMOVED from the set — the prefix is
        // exactly what to subtract, not what to pass. Passing the prefix itself inverts
        // the guard: it would suppress on the arrivals that are settled and permit on
        // the ones that are not.
        //
        // Concretely, on the row's own example (a resolution that sequentially puts a
        // `trigger_zone: Graveyard` source into a graveyard, then kills another
        // creature): the prefix at the second event is `{source}`, which suppresses —
        // i.e. the row's sketch reproduces the very defect it describes. The complement
        // gives `{other}`, and the source's trigger fires, which is the CR 603.10a
        // answer.
        //
        // The subtraction is also what keeps the other order correct. `check_triggers`
        // runs AFTER every event in the slice has been applied, so a source that arrives
        // LATER in the slice is already sitting in the graveyard when
        // `collect_graveyard_carddef_triggers` enumerates `state.objects`. Keeping
        // later-and-current arrivals in the set is what stops it firing off a death that
        // happened before it got there.
        let sequential_arrivals: std::collections::BTreeSet<ObjectId>;
        let arrived_in_graveyard_this_batch: &std::collections::BTreeSet<ObjectId> = match timing {
            EventBatchTiming::Simultaneous => &whole_batch_arrivals,
            EventBatchTiming::Sequential => {
                sequential_arrivals = whole_batch_arrivals
                    .difference(&earlier_arrivals)
                    .copied()
                    .collect();
                &sequential_arrivals
            }
        };
        match event {
            GameEvent::PermanentEnteredBattlefield { object_id, .. } => {
                // SelfEntersBattlefield: fires on the entering permanent itself.
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::SelfEntersBattlefield,
                    Some(*object_id), // Only check this specific object
                    Some(*object_id), // entering_object_id: the permanent itself
                );
                // AnyPermanentEntersBattlefield: fires on ALL permanents (including the entering one).
                // Pass the entering object so TriggerDoublerFilter::ArtifactOrCreatureETB can
                // verify the entering object's card types (CR 603.2d).
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::AnyPermanentEntersBattlefield,
                    None,             // Check all battlefield permanents
                    Some(*object_id), // entering_object_id: the permanent that entered
                );
                // PB-35 / CR 603.3 / TriggerZone::Graveyard: Also scan graveyard objects
                // for CardDef triggered abilities that monitor AnyPermanentEntersBattlefield
                // while in the graveyard (e.g. Bloodghast's Landfall trigger).
                collect_graveyard_carddef_triggers(
                    state,
                    &mut triggers,
                    event,
                    Some(*object_id),
                    arrived_in_graveyard_this_batch,
                );
                // CR 702.74a: If the permanent was evoked, generate the evoke sacrifice trigger.
                // "When this permanent enters, if its evoke cost was paid, its controller
                // sacrifices it." This goes on the stack as a separate triggered ability,
                // allowing the controller to order it relative to other ETB triggers
                // (e.g., Mulldrifter can resolve draw before sacrifice).
                // CR 113.7a: the entering object may have left this event batch; use LKI.
                if let Some(obj) = state.fizzle_object(*object_id) {
                    if obj.cast_alt_cost == Some(crate::state::types::AltCostKind::Evoke) {
                        let evoke_trigger = PendingTrigger {
                            triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                            entering_object_id: Some(*object_id),
                            ..PendingTrigger::blank(
                                *object_id,
                                obj.controller,
                                PendingTriggerKind::Evoke,
                            )
                        };
                        triggers.push(evoke_trigger);
                    }
                }
                // CR 702.110a: If the permanent has Exploit, generate the exploit trigger.
                // "When this creature enters, you may sacrifice a creature."
                // Each instance of Exploit in the card definition triggers separately.
                // CR 113.7a: the entering object may have left this event batch; use LKI.
                if let Some(obj) = state.fizzle_object(*object_id) {
                    if obj
                        .characteristics
                        .keywords
                        .contains(&KeywordAbility::Exploit)
                    {
                        // Count exploit instances from card definition for multiple instances.
                        // OrdSet deduplicates, so check the card definition for exact count.
                        let exploit_count = obj
                            .card_id
                            .as_ref()
                            .and_then(|cid| state.card_registry.get(cid.clone()))
                            .map(|def| {
                                def.abilities
                                    .iter()
                                    .filter(|a| {
                                        matches!(
                                            a,
                                            crate::cards::card_definition::AbilityDefinition::Keyword(
                                                KeywordAbility::Exploit
                                            )
                                        )
                                    })
                                    .count()
                            })
                            .unwrap_or(1)
                            .max(1);
                        let controller = obj.controller;
                        for _ in 0..exploit_count {
                            triggers.push(PendingTrigger {
                                triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                                entering_object_id: Some(*object_id),
                                ..PendingTrigger::blank(
                                    *object_id,
                                    controller,
                                    PendingTriggerKind::Exploit,
                                )
                            });
                        }
                    }
                }
                // CR 702.75a: Hideaway(N) — "When this permanent enters, look at
                // the top N cards of your library. Exile one of them face down
                // and put the rest on the bottom of your library in a random order."
                //
                // Each Hideaway(N) keyword on the permanent generates one trigger.
                // Multiple instances trigger separately (CR 603.2: each keyword instance
                // is a separate triggered ability).
                // CR 113.7a: the entering object may have left this event batch; use LKI.
                if let Some(obj) = state.fizzle_object(*object_id) {
                    let controller = obj.controller;
                    let hideaway_keywords: Vec<u32> = obj
                        .characteristics
                        .keywords
                        .iter()
                        .filter_map(|kw| {
                            if let KeywordAbility::Hideaway(n) = kw {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .collect();
                    for n in hideaway_keywords {
                        triggers.push(PendingTrigger {
                            triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                            entering_object_id: Some(*object_id),
                            data: Some(TriggerData::ETBHideaway { count: n }),
                            ..PendingTrigger::blank(
                                *object_id,
                                controller,
                                PendingTriggerKind::Hideaway,
                            )
                        });
                    }
                }
                // CR 702.124j: Partner With ETB trigger —
                // "When this permanent enters, target player may search their
                // library for a card named [name], reveal it, put it into their
                // hand, then shuffle."
                //
                // CR 603.3: The trigger goes on the stack (can be countered).
                // Target player: deterministic fallback = the entering permanent's
                // controller (the player most likely to have the partner in their
                // library in a Commander game).
                {
                    // CR 113.7a: the entering object may have left this event batch; use LKI.
                    if let Some(obj) = state.fizzle_object(*object_id) {
                        let controller = obj.controller;
                        let partner_with_names: Vec<String> = obj
                            .characteristics
                            .keywords
                            .iter()
                            .filter_map(|kw| {
                                if let KeywordAbility::PartnerWith(name) = kw {
                                    Some(name.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        for name in partner_with_names {
                            triggers.push(PendingTrigger {
                                triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                                entering_object_id: Some(*object_id),
                                data: Some(TriggerData::ETBPartnerWith {
                                    partner_name: name,
                                    target_player: controller,
                                }),
                                ..PendingTrigger::blank(
                                    *object_id,
                                    controller,
                                    PendingTriggerKind::PartnerWith,
                                )
                            });
                        }
                    }
                }
                // CR 702.165a: Backup -- "When this creature enters, put N +1/+1 counters
                // on target creature. If that's another creature, it also gains the non-backup
                // abilities of this creature printed below this one until end of turn."
                //
                // CR 702.165d: Abilities are determined at trigger time (snapshot when trigger
                // fires, not at resolution). Stored in backup_abilities on PendingTrigger.
                // CR 702.165c: Only printed abilities (from card definition), not gained ones.
                // CR 702.165a: Only abilities printed BELOW the Backup entry in the definition.
                {
                    // CR 113.7a: the entering object may have left this event batch; use LKI.
                    if let Some(obj) = state.fizzle_object(*object_id) {
                        let controller = obj.controller;
                        let card_id = obj.card_id.clone();
                        if let Some(cid) = card_id {
                            if let Some(def) = state.card_registry.get(cid) {
                                // CR 702.165a / OOS-DX1-4 Q1 (PB-DX24): one binding serves BOTH
                                // the enumerate() below and the "printed below this one" slice,
                                // so index and slice can never diverge across a DFC's two faces.
                                // "Printed below" is a property of the VISIBLE face.
                                let eff = def.effective_abilities(obj.is_transformed);
                                // Find all Backup(N) instances and their positions.
                                for (idx, ability) in eff.iter().enumerate() {
                                    if let crate::cards::card_definition::AbilityDefinition::Keyword(
                                        KeywordAbility::Backup(n),
                                    ) = ability
                                    {
                                        // CR 702.165d: Snapshot abilities below this Backup entry.
                                        // CR 702.165a: "non-backup abilities printed below this one"
                                        // CR 702.165c: Only printed abilities.
                                        let abilities_below: Vec<KeywordAbility> = eff[idx + 1..]
                                            .iter()
                                            .filter_map(|a| match a {
                                                crate::cards::card_definition::AbilityDefinition::Keyword(kw)
                                                    if !matches!(kw, KeywordAbility::Backup(_)) =>
                                                {
                                                    Some(kw.clone())
                                                }
                                                _ => None,
                                            })
                                            .collect();
                                        triggers.push(PendingTrigger {
                                            ability_index: idx,
                                            triggering_event: Some(
                                                TriggerEvent::SelfEntersBattlefield,
                                            ),
                                            entering_object_id: Some(*object_id),
                                            data: Some(TriggerData::ETBBackup {
                                                target: *object_id,
                                                count: *n,
                                                abilities: abilities_below,
                                            }),
                                            ..PendingTrigger::blank(*object_id, controller, PendingTriggerKind::Backup)
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                // CR 702.72a: Champion ETB trigger.
                // "When this permanent enters, sacrifice it unless you exile
                // another [object] you control."
                {
                    // CR 113.7a: the entering object may have left this event batch; use LKI.
                    if let Some(obj) = state.fizzle_object(*object_id) {
                        if obj
                            .characteristics
                            .keywords
                            .contains(&KeywordAbility::Champion)
                        {
                            let controller = obj.controller;
                            // Look up champion filter from card registry.
                            let filter = obj
                                .card_id
                                .as_ref()
                                .and_then(|cid| state.card_registry.get(cid.clone()))
                                .and_then(|def| {
                                    def.abilities.iter().find_map(|a| {
                                        if let crate::cards::card_definition::AbilityDefinition::Champion {
                                            filter,
                                        } = a
                                        {
                                            Some(filter.clone())
                                        } else {
                                            None
                                        }
                                    })
                                })
                                .unwrap_or(ChampionFilter::AnyCreature);
                            triggers.push(PendingTrigger {
                                triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                                entering_object_id: Some(*object_id),
                                data: Some(TriggerData::ETBChampion { filter }),
                                ..PendingTrigger::blank(
                                    *object_id,
                                    controller,
                                    PendingTriggerKind::ChampionETB,
                                )
                            });
                        }
                    }
                }
                // CR 702.95a: Soulbond — two ETB triggered abilities:
                //   Trigger 1 (SelfETB): When a creature with soulbond enters, if its
                //   controller controls another unpaired creature, pair them.
                //   Trigger 2 (OtherETB): When any creature enters, for each unpaired
                //   soulbond creature controlled by the same player, pair them.
                //
                // CR 603.4: Intervening-if — "you control another unpaired creature" is
                // checked at trigger time AND at resolution.
                {
                    // CR 113.7a: the entering object may have left this event batch; use LKI.
                    let entering_controller = state.fizzle_object(*object_id).map(|o| o.controller);
                    let entering_is_creature =
                        crate::rules::layers::calculate_characteristics(state, *object_id)
                            .or_else(|| {
                                state
                                    .objects
                                    .get(object_id)
                                    .map(|o| o.characteristics.clone())
                            })
                            .map(|chars| chars.card_types.contains(&CardType::Creature))
                            .unwrap_or(false);
                    if entering_is_creature {
                        if let Some(controller) = entering_controller {
                            // Trigger 1 (SoulbondSelfETB): entering creature itself has Soulbond.
                            // CR 613.1f: Use layer-resolved keywords only (Humility
                            // removes Soulbond; base OR layer was over-permissive).
                            let entering_has_soulbond =
                                crate::rules::layers::calculate_characteristics(state, *object_id)
                                    .or_else(|| {
                                        state
                                            .objects
                                            .get(object_id)
                                            .map(|o| o.characteristics.clone())
                                    })
                                    .map(|c| c.keywords.contains(&KeywordAbility::Soulbond))
                                    .unwrap_or(false);
                            if entering_has_soulbond {
                                // Intervening-if: controller has another unpaired creature.
                                // CR 613.1d: Use layer-resolved types for creature check.
                                let pair_target: Option<ObjectId> = state
                                    .objects
                                    .values()
                                    .find(|obj| {
                                        obj.zone == ZoneId::Battlefield
                                            && obj.is_phased_in()
                                            && obj.controller == controller
                                            && obj.id != *object_id
                                            && obj.paired_with.is_none()
                                            && crate::rules::layers::expect_characteristics(
                                                state, obj.id,
                                            )
                                            .card_types
                                            .contains(&CardType::Creature)
                                    })
                                    .map(|obj| obj.id);
                                if let Some(partner_id) = pair_target {
                                    triggers.push(PendingTrigger {
                                        triggering_event: Some(
                                            TriggerEvent::AnyPermanentEntersBattlefield,
                                        ),
                                        entering_object_id: Some(*object_id),
                                        data: Some(TriggerData::ETBSoulbond {
                                            pair_target: partner_id,
                                        }),
                                        ..PendingTrigger::blank(
                                            *object_id,
                                            controller,
                                            PendingTriggerKind::SoulbondSelfETB,
                                        )
                                    });
                                }
                            }
                            // Trigger 2 (SoulbondOtherETB): other unpaired soulbond creatures
                            // controlled by same player pair with the entering creature.
                            // The entering creature must also be unpaired (checked at trigger time).
                            let entering_is_unpaired = state
                                .objects
                                .get(object_id)
                                .map(|o| o.paired_with.is_none())
                                .unwrap_or(false);
                            if entering_is_unpaired {
                                let soulbond_sources: Vec<(ObjectId, PlayerId)> =
                                    state
                                        .objects
                                        .values()
                                        .filter(|obj| {
                                            obj.zone == ZoneId::Battlefield
                                            && obj.is_phased_in()
                                            && obj.controller == controller
                                            && obj.id != *object_id
                                            && obj.paired_with.is_none()
                                            // CR 613.1d/613.1f: Use layer-resolved types and
                                            // keywords for Soulbond pairing candidates.
                                            && {
                                                let chars = crate::rules::layers::expect_characteristics(
                                                    state, obj.id,
                                                );
                                                chars.card_types.contains(&CardType::Creature)
                                                    && chars.keywords.contains(&KeywordAbility::Soulbond)
                                            }
                                        })
                                        .map(|obj| (obj.id, obj.controller))
                                        .collect();
                                for (sb_id, sb_controller) in soulbond_sources {
                                    // Skip if sb_id has Soulbond and already fired SelfETB for this
                                    // same pair (sb_id == object_id handled by filter above).
                                    // This arm fires for OTHER soulbond creatures pairing INTO
                                    // the entering creature — only skip if entering creature itself
                                    // has soulbond (handled by Trigger 1 above).
                                    if entering_has_soulbond && sb_id == *object_id {
                                        continue;
                                    }
                                    triggers.push(PendingTrigger {
                                        triggering_event: Some(
                                            TriggerEvent::AnyPermanentEntersBattlefield,
                                        ),
                                        entering_object_id: Some(*object_id),
                                        data: Some(TriggerData::ETBSoulbond {
                                            pair_target: *object_id,
                                        }),
                                        ..PendingTrigger::blank(
                                            sb_id,
                                            sb_controller,
                                            PendingTriggerKind::SoulbondOtherETB,
                                        )
                                    });
                                }
                            }
                        }
                    }
                }
                // CR 702.100a: Evolve — "Whenever a creature you control enters,
                // if that creature's power is greater than this creature's power
                // and/or that creature's toughness is greater than this creature's
                // toughness, put a +1/+1 counter on this creature."
                //
                // CR 702.100c: Noncreature permanents cannot trigger evolve.
                // CR 702.100d: Multiple instances of evolve each trigger separately.
                // CR 603.4: Intervening-if — P/T comparison is checked at trigger time.
                {
                    // First verify the entering permanent is a creature (CR 702.100c).
                    let entering_is_creature =
                        crate::rules::layers::calculate_characteristics(state, *object_id)
                            .or_else(|| {
                                state
                                    .objects
                                    .get(object_id)
                                    .map(|o| o.characteristics.clone())
                            })
                            .map(|chars| chars.card_types.contains(&CardType::Creature))
                            .unwrap_or(false);
                    if entering_is_creature {
                        let entering_controller =
                            // CR 113.7a: the entering object may have left this event batch; use LKI.
                            state.fizzle_object(*object_id).map(|o| o.controller);
                        if let Some(controller) = entering_controller {
                            // Get the entering creature's P/T (layer-aware).
                            let entering_chars =
                                crate::rules::layers::calculate_characteristics(state, *object_id)
                                    .or_else(|| {
                                        state
                                            .objects
                                            .get(object_id)
                                            .map(|o| o.characteristics.clone())
                                    });
                            let (entering_power, entering_toughness) = entering_chars
                                .as_ref()
                                .map(|c| (c.power.unwrap_or(0), c.toughness.unwrap_or(0)))
                                .unwrap_or((0, 0));
                            // Collect all creatures with evolve controlled by the same player.
                            // Exclude the entering creature itself (cannot evolve from itself).
                            let evolve_sources: Vec<ObjectId> = state
                                .objects
                                .values()
                                .filter(|obj| {
                                    obj.zone == ZoneId::Battlefield
                                        && obj.is_phased_in()
                                        && obj.controller == controller
                                        && obj.id != *object_id
                                        && obj
                                            .characteristics
                                            .keywords
                                            .contains(&KeywordAbility::Evolve)
                                })
                                .map(|obj| obj.id)
                                .collect();
                            for evolve_id in evolve_sources {
                                // CR 603.4: Intervening-if check at trigger time.
                                // Get the evolve creature's current P/T (layer-aware).
                                let evolve_chars = crate::rules::layers::calculate_characteristics(
                                    state, evolve_id,
                                )
                                .or_else(|| {
                                    state
                                        .objects
                                        .get(&evolve_id)
                                        .map(|o| o.characteristics.clone())
                                });
                                let (evolve_power, evolve_toughness) = evolve_chars
                                    .as_ref()
                                    .map(|c| (c.power.unwrap_or(0), c.toughness.unwrap_or(0)))
                                    .unwrap_or((0, 0));
                                // CR 702.100a: trigger fires if entering P > evolve P
                                // OR entering T > evolve T (inclusive or).
                                if entering_power > evolve_power
                                    || entering_toughness > evolve_toughness
                                {
                                    let evolve_controller = state
                                        .expect_object(evolve_id)
                                        .map(|o| o.controller)
                                        .unwrap_or(controller);
                                    // CR 702.100d: Count evolve instances from card
                                    // definition — OrdSet deduplicates, so check the
                                    // card definition for the exact count.
                                    let evolve_count = state
                                        .expect_object(evolve_id)
                                        .and_then(|obj| obj.card_id.as_ref())
                                        .and_then(|cid| state.card_registry.get(cid.clone()))
                                        .map(|def| {
                                            def.abilities
                                                .iter()
                                                .filter(|a| {
                                                    matches!(
                                                        a,
                                                        AbilityDefinition::Keyword(
                                                            KeywordAbility::Evolve
                                                        )
                                                    )
                                                })
                                                .count()
                                        })
                                        .unwrap_or(1)
                                        .max(1);
                                    for _ in 0..evolve_count {
                                        triggers.push(PendingTrigger {
                                            triggering_event: Some(
                                                TriggerEvent::AnyPermanentEntersBattlefield,
                                            ),
                                            entering_object_id: Some(*object_id),
                                            data: Some(TriggerData::ETBEvolve {
                                                entering_creature: *object_id,
                                            }),
                                            ..PendingTrigger::blank(
                                                evolve_id,
                                                evolve_controller,
                                                PendingTriggerKind::Evolve,
                                            )
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                // CR 702.58a: Graft -- "Whenever another creature enters, if this
                // permanent has a +1/+1 counter on it, you may move a +1/+1 counter
                // from this permanent onto that creature."
                //
                // CR 702.58b: Multiple instances each trigger separately.
                // Differences from Evolve:
                // - Fires for ANY player's creature entering (not just controller's)
                // - Has intervening-if: source must have a +1/+1 counter
                // - "Another creature" -- source entering does NOT trigger itself
                {
                    // Only creatures entering trigger Graft (CR 702.58a).
                    let entering_is_creature =
                        crate::rules::layers::calculate_characteristics(state, *object_id)
                            .or_else(|| {
                                state
                                    .objects
                                    .get(object_id)
                                    .map(|o| o.characteristics.clone())
                            })
                            .map(|chars| chars.card_types.contains(&CardType::Creature))
                            .unwrap_or(false);
                    if entering_is_creature {
                        // Collect all battlefield permanents with Graft that:
                        // 1. Are not the entering creature itself ("another creature")
                        // 2. Have at least one +1/+1 counter (intervening-if check at trigger time, CR 603.4)
                        let graft_sources: Vec<(ObjectId, PlayerId, usize)> = state
                            .objects
                            .iter()
                            .filter(|(id, obj)| {
                                obj.zone == ZoneId::Battlefield
                                    && **id != *object_id
                                    && obj.is_phased_in()
                                    && obj
                                        .counters
                                        .get(&CounterType::PlusOnePlusOne)
                                        .copied()
                                        .unwrap_or(0)
                                        > 0
                            })
                            .filter_map(|(id, obj)| {
                                let chars =
                                    crate::rules::layers::expect_characteristics(state, *id);
                                let graft_count = chars
                                    .keywords
                                    .iter()
                                    .filter(|kw| matches!(kw, KeywordAbility::Graft(_)))
                                    .count();
                                if graft_count > 0 {
                                    Some((*id, obj.controller, graft_count))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        for (graft_id, graft_controller, graft_count) in graft_sources {
                            for _ in 0..graft_count {
                                triggers.push(PendingTrigger {
                                    triggering_event: Some(
                                        TriggerEvent::AnyPermanentEntersBattlefield,
                                    ),
                                    entering_object_id: Some(*object_id),
                                    data: Some(TriggerData::ETBGraft {
                                        entering_creature: *object_id,
                                    }),
                                    ..PendingTrigger::blank(
                                        graft_id,
                                        graft_controller,
                                        PendingTriggerKind::Graft,
                                    )
                                });
                            }
                        }
                    }
                }
            }
            GameEvent::SpellCast {
                player,
                source_object_id,
                ..
            } => {
                // AnySpellCast: fires on all permanents that watch for spell casts.
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::AnySpellCast,
                    None,
                    None,
                );
                // CR 702.108a: Prowess — "Whenever you cast a noncreature spell."
                // Check if the cast spell is noncreature by inspecting the source object's
                // card types. Only fire if the spell lacks CardType::Creature.
                let is_noncreature = state
                    .objects
                    .get(source_object_id)
                    .map(|obj| {
                        !obj.characteristics
                            .card_types
                            .contains(&crate::state::types::CardType::Creature)
                    })
                    .unwrap_or(false);
                if is_noncreature {
                    // Collect triggers only for permanents controlled by the caster.
                    // Prowess says "whenever YOU cast" -- only the controller's creatures trigger.
                    let prowess_sources: Vec<ObjectId> = state
                        .objects
                        .values()
                        .filter(|obj| {
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller == *player
                        })
                        .map(|obj| obj.id)
                        .collect();
                    for obj_id in prowess_sources {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::ControllerCastsNoncreatureSpell,
                            Some(obj_id),
                            None,
                        );
                    }
                }
                // CR 702.101a: Extort — "Whenever you cast a spell."
                // Collect triggers only for permanents controlled by the caster.
                // No type restriction (unlike Prowess which requires noncreature).
                // Each extort instance triggers separately (CR 702.101b).
                {
                    let controller_sources: Vec<ObjectId> = state
                        .objects
                        .values()
                        .filter(|obj| {
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller == *player
                        })
                        .map(|obj| obj.id)
                        .collect();
                    for obj_id in controller_sources {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::ControllerCastsSpell,
                            Some(obj_id),
                            None,
                        );
                    }
                }
                // CR 603.2 / CR 102.2: "Whenever an opponent casts a spell."
                // Collect triggers on all permanents whose controller is NOT the caster.
                // In Commander FFA (CR 903.2, CR 102.2), all other players are opponents.
                {
                    let opponent_sources: Vec<ObjectId> = state
                        .objects
                        .values()
                        .filter(|obj| {
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller != *player
                        })
                        .map(|obj| obj.id)
                        .collect();
                    let pre_len = triggers.len();
                    for obj_id in opponent_sources {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::OpponentCastsSpell,
                            Some(obj_id),
                            None,
                        );
                    }
                    // Tag opponent-casts triggers with the casting player so
                    // flush_pending_triggers can set Target::Player at index 0.
                    for t in &mut triggers[pre_len..] {
                        t.triggering_player = Some(*player);
                    }
                }
                // CR 113.6p / CR 114.4: Emblem triggers from command zone emblems.
                // Emblems fire "whenever you cast" triggers for their controlling player.
                // Only scan the caster's emblems (emblem abilities say "whenever YOU cast").
                collect_emblem_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::AnySpellCast,
                    Some(*player),
                );
                collect_emblem_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::ControllerCastsSpell,
                    Some(*player),
                );
                // G-4 / index-namespace fix (2026-07-09): post-processing for
                // ControllerCastsSpell and OpponentCastsSpell triggers that carry a
                // spell_type_filter / noncreature_only / spell_subtype_filter (from a
                // WheneverYouCastSpell or WheneverOpponentCastsSpell CardDef condition).
                //
                // BUG HISTORY: this used to look the ability up via
                // `def.abilities.get(t.ability_index)` -- an index into the RAW
                // `CardDefinition::abilities` Vec (which also contains Keyword/Static/
                // Activated abilities). `t.ability_index` is actually a dense index into
                // the object's runtime `characteristics.triggered_abilities` list (set by
                // `collect_triggers_for_event` from `resolved_chars.triggered_abilities
                // .iter().enumerate()`). These two index spaces only coincide by accident
                // when the card's Triggered ability happens to sit at the same position in
                // both lists. On any multi-ability card where they don't (e.g. Monastery
                // Mentor: Keyword(Prowess) at abilities[0], Triggered at abilities[1] but
                // dense index 0; Leaf-Crowned Visionary: Static at abilities[0], Triggered
                // at abilities[1] but dense index 0), the lookup landed on a non-Triggered
                // ability and fell through the `_ => true` catch-all, silently skipping the
                // filter and firing on every spell cast.
                //
                // FIX: re-resolve the SAME dense runtime list the trigger was built from
                // (`t.source`'s `triggered_abilities[t.ability_index]`, layer-resolved per
                // CR 613.1f) and read the filter off `triggering_creature_filter`
                // (`TargetFilter`), which `enrich_spec_from_def` now populates for
                // WheneverYouCastSpell / WheneverOpponentCastsSpell from
                // spell_type_filter/noncreature_only/spell_subtype_filter --
                // `has_card_types`/`non_creature`/`has_subtypes` are exact semantic
                // equivalents, so this reuses existing hashed struct fields (no new field,
                // no HASH_SCHEMA_VERSION bump). Because `t.ability_index` is guaranteed (by
                // construction in `collect_triggers_for_event`/
                // `collect_emblem_triggers_for_event`) to index the entry whose `trigger_on`
                // matches `t.triggering_event`, this lookup cannot collide with an unrelated
                // ability the way the CardDef lookup did.
                //
                // NOTE: `chosen_subtype_filter` (CR 603.1 "of the chosen type", used only by
                // Vanquisher's Banner) is NOT carried by `triggering_creature_filter` -- it's
                // a dynamic per-source condition (checked against the source's
                // `chosen_creature_type`), not a static TargetFilter predicate. It remains
                // unenforced here, same as before this fix. Vanquisher's Banner is out of
                // scope for this fix (see task scope discipline); it now correctly narrows to
                // creature spells via spell_type_filter but does not further narrow to the
                // chosen type.
                {
                    let spell_chars = state
                        .objects
                        .get(source_object_id)
                        .map(|o| &o.characteristics);
                    triggers.retain(|t| {
                        // Only post-filter ControllerCastsSpell and OpponentCastsSpell triggers.
                        let te = t.triggering_event.as_ref();
                        if te != Some(&TriggerEvent::ControllerCastsSpell)
                            && te != Some(&TriggerEvent::OpponentCastsSpell)
                        {
                            return true;
                        }
                        // Resolve the exact TriggeredAbilityDef this trigger was built from.
                        let resolved_chars =
                            crate::rules::layers::calculate_characteristics(state, t.source)
                                .or_else(|| {
                                    state
                                        .objects
                                        .get(&t.source)
                                        .map(|o| o.characteristics.clone())
                                });
                        let Some(chars) = resolved_chars else {
                            return true;
                        };
                        let Some(trigger_def) = chars.triggered_abilities.get(t.ability_index)
                        else {
                            return true;
                        };
                        let Some(ref filter) = trigger_def.triggering_creature_filter else {
                            return true;
                        };
                        let Some(spell_chars) = spell_chars else {
                            return true;
                        };
                        crate::effects::matches_filter(spell_chars, filter)
                    });
                }
                // G-15: WhenYouCastThisSpell — fires when the spell itself is put on the stack.
                // The trigger source is the stack object (source_object_id).
                // Look up the spell's CardDef for WhenYouCastThisSpell triggered abilities.
                // CR 113.7a: the cast spell may have left the stack this batch; use LKI.
                if let Some(stack_obj) = state.fizzle_object(*source_object_id) {
                    let caster = stack_obj.controller;
                    if let Some(card_id) = stack_obj.card_id.clone() {
                        if let Some(def) = state.card_registry.get(card_id) {
                            // OOS-DX1-4 Q2 (PB-DX24): `is_transformed` is never true on a
                            // stack object (it is set only at ETB and reset on every zone
                            // change, `state/mod.rs`), so this is defensive rather than a
                            // live repair -- it makes the queue side the SAME expression the
                            // read side uses (`resolution.rs`), not accidentally equal to it.
                            let eff = def.effective_abilities(stack_obj.is_transformed);
                            for (idx, ability) in eff.iter().enumerate() {
                                if let AbilityDefinition::Triggered {
                                    trigger_condition: TriggerCondition::WhenYouCastThisSpell,
                                    intervening_if,
                                    ..
                                } = ability
                                {
                                    // CR 603.4 (PB-DP6): queue-time gate. NOTE — the
                                    // source here is the spell's STACK object, not a
                                    // permanent (`source` is `*source_object_id`, the
                                    // stack object read via `fizzle_object` above).
                                    // `SourceOnBattlefield`/`SourceHasCounters`-style
                                    // conditions would (correctly) answer false while
                                    // the spell is on the stack — CR 603.4 asks the
                                    // question against the game state as it actually
                                    // is, and the spell genuinely is not a permanent
                                    // yet, so this is not a bug.
                                    //
                                    // `WasKicked`/`XValueAtLeast` are a DIFFERENT,
                                    // wrong-in-the-suppression-direction case (PB-DP6
                                    // fix-cycle finding, LOW 1): `kicker_times_paid`/
                                    // `x_value` are `GameObject` fields written once,
                                    // at `resolution.rs:619`/`:628`, when the spell
                                    // *resolves into a permanent* — they are still 0
                                    // on the stack object at this site, so a
                                    // hypothetical "when you cast this spell, if it
                                    // was kicked" would read false and the trigger
                                    // would be wrongly suppressed even though the
                                    // spell genuinely was kicked. Not defensible the
                                    // way `SourceOnBattlefield` is: the engine just
                                    // stores the fact on the wrong object at this
                                    // moment. Zero corpus exposure today (no def
                                    // pairs `WhenYouCastThisSpell` with any
                                    // `intervening_if`). The site IS gated below like
                                    // every other Category-A site; what was left alone
                                    // is the `WasKicked`/`XValueAtLeast` carve-out —
                                    // they stay classified queue-time-evaluable rather
                                    // than being special-cased here, because a real
                                    // fix needs either
                                    // `StackObject.kicker_times_paid`/`x_value` or to
                                    // write those fields onto the spell's
                                    // `GameObject` at cast time — both bigger than a
                                    // gate tweak. Seeded as an OOS-DP6 finding for the
                                    // coordinator to file.
                                    if !carddef_intervening_if_holds_at_queue_time(
                                        state,
                                        intervening_if.as_ref(),
                                        caster,
                                        *source_object_id,
                                    ) {
                                        continue;
                                    }
                                    // Push the cast-trigger using the stack object as source.
                                    // PB-EF3 A2 (CR 601.2c/603.3d): `idx` here is a raw index
                                    // into `def.abilities` (this trigger is never lowered into
                                    // runtime `characteristics.triggered_abilities` — it fires
                                    // directly from the spell's CardDef, see comment above).
                                    // CardDefETB kind makes the raw-index/card-registry lookup
                                    // authoritative for both effect AND target selection (Elder
                                    // Deep Fiend's "tap up to four target permanents" needs its
                                    // declared `targets` to survive auto-target selection).
                                    triggers.push(PendingTrigger {
                                        ability_index: idx,
                                        ..PendingTrigger::blank(
                                            *source_object_id,
                                            caster,
                                            PendingTriggerKind::CardDefETB,
                                        )
                                    });
                                }
                            }
                        }
                    }
                }
            }
            GameEvent::PermanentTapped { object_id, .. } => {
                // SelfBecomesTapped: fires on the tapped permanent itself.
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::SelfBecomesTapped,
                    Some(*object_id),
                    None,
                );
            }
            // CR 502.3 / 603.2e (PB-AC1): a single permanent became untapped (effect-driven,
            // e.g. Effect::UntapPermanent / Effect::UntapAll). Dispatch globally; the
            // untapped permanent is carried via the entering_object parameter.
            GameEvent::PermanentUntapped { object_id, .. } => {
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::AnyPermanentUntaps,
                    None,
                    Some(*object_id),
                );
            }
            // CR 502.3 / 502.4: untap-step batch untap. One dispatch per untapped permanent;
            // per CR 502.4 these triggers are held (pending_triggers) and put on the stack
            // at the next priority window (usually upkeep) — the existing pending-trigger
            // queue already provides this hold.
            GameEvent::PermanentsUntapped { objects, .. } => {
                for id in objects {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::AnyPermanentUntaps,
                        None,
                        Some(*id),
                    );
                }
            }
            // CR 122.6 / 122.7 (PB-AC1): one or more counters were put on a permanent.
            // Dispatch globally; the receiving permanent is carried via entering_object.
            // Post-filter by counter kind: collect_triggers_for_event cannot see the placed
            // counter's kind (it only sees the trigger_def), so we retain only the triggers
            // whose `counter_filter` is None (any kind) or matches the placed kind.
            GameEvent::CounterAdded {
                object_id, counter, ..
            } => {
                let pre_len = triggers.len();
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::CounterPlaced,
                    None,
                    Some(*object_id),
                );
                // Drop newly-pushed CounterPlaced triggers whose ability's counter_filter
                // doesn't match the counter kind actually placed (collect_triggers_for_event
                // cannot see the placed counter's kind, only the trigger_def).
                let mut kept: Vec<PendingTrigger> = Vec::with_capacity(triggers.len());
                for (i, t) in triggers.into_iter().enumerate() {
                    if i >= pre_len && t.triggering_event == Some(TriggerEvent::CounterPlaced) {
                        let source_filter =
                            crate::rules::layers::calculate_characteristics(state, t.source)
                                .and_then(|c| c.triggered_abilities.get(t.ability_index).cloned())
                                .and_then(|def| def.counter_filter);
                        if let Some(required) = source_filter {
                            if required != *counter {
                                continue;
                            }
                        }
                    }
                    kept.push(t);
                }
                triggers = kept;
            }
            GameEvent::AttackersDeclared {
                attacking_player,
                attackers,
            } => {
                // SelfAttacks: fires on each creature that is declared as an attacker (CR 508.1m, CR 508.3a).
                // CR 702.86a / CR 508.5: tag each SelfAttacks trigger with the defending player
                // so annihilator (and any future "defending player" attack triggers) can resolve
                // the correct player in multiplayer games (CR 508.5a).
                for (attacker_id, attack_target) in attackers {
                    let pre_len = triggers.len();
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::SelfAttacks,
                        Some(*attacker_id),
                        None,
                    );
                    // Resolve defending player from AttackTarget (CR 508.5).
                    let defending_player = match attack_target {
                        crate::state::combat::AttackTarget::Player(pid) => Some(*pid),
                        crate::state::combat::AttackTarget::Planeswalker(pw_id) => {
                            state.objects.get(pw_id).map(|obj| obj.controller)
                        }
                    };
                    for t in &mut triggers[pre_len..] {
                        t.defending_player_id = defending_player;
                    }
                    // CR 702.116a/b: Tag myriad triggers for special stack handling.
                    // A SelfAttacks trigger is a myriad trigger if its source object has
                    // the Myriad keyword. We check the triggered ability's description
                    // (set by builder.rs) to identify myriad triggers -- they carry
                    // `effect: None` and start with "Myriad". The `kind` field is set to
                    // `PendingTriggerKind::Myriad` so flush_pending_triggers creates a
                    // KeywordTrigger (Myriad) stack object (not a plain TriggeredAbility).
                    for t in &mut triggers[pre_len..] {
                        // PB-EF3b: read layer-RESOLVED characteristics, not raw base, since
                        // `t.ability_index` indexes resolved `triggered_abilities` (a granted
                        // trigger-keyword's derived def only exists there). Still None-tolerant
                        // (CR 113.7a: the trigger source may have left this batch).
                        if let Some(chars) =
                            crate::rules::layers::calculate_characteristics(state, t.source)
                        {
                            if let Some(ta) = chars.triggered_abilities.get(t.ability_index) {
                                if ta.effect.is_none() && ta.description.starts_with("Myriad") {
                                    t.kind = PendingTriggerKind::Myriad;
                                }
                            }
                        }
                    }
                    // CR 702.39a/b: Tag provoke triggers for special stack handling.
                    // A SelfAttacks trigger is a provoke trigger if the triggered ability
                    // description starts with "Provoke" (set by builder.rs). At collection
                    // time, select a target creature the defending player controls
                    // (deterministic: first by ObjectId order in OrdMap).
                    // CR 603.3d: If no valid target exists, provoke_target_creature is None
                    // and the trigger will not be placed on the stack in flush_pending_triggers.
                    // CR 702.39b: When a creature has multiple Provoke instances, each trigger
                    // independently selects a target. Track already-assigned targets so that
                    // successive triggers from the same attacker pick different creatures.
                    let mut provoke_targets_used: Vec<ObjectId> = Vec::new();
                    for t in &mut triggers[pre_len..] {
                        // PB-EF3b: read layer-RESOLVED characteristics, not raw base, since
                        // `t.ability_index` indexes resolved `triggered_abilities` (a granted
                        // trigger-keyword's derived def only exists there). Still None-tolerant
                        // (CR 113.7a: the trigger source may have left this batch).
                        if let Some(chars) =
                            crate::rules::layers::calculate_characteristics(state, t.source)
                        {
                            if let Some(ta) = chars.triggered_abilities.get(t.ability_index) {
                                if ta.description.starts_with("Provoke") {
                                    t.kind = PendingTriggerKind::Provoke;
                                    // Select target: first creature controlled by defending player
                                    // that has not already been claimed by a prior provoke trigger
                                    // from this attacker this combat.
                                    if let Some(dp) = defending_player {
                                        let target = state
                                            .objects
                                            .values()
                                            .filter(|o| {
                                                o.zone == ZoneId::Battlefield
                                                    && o.controller == dp
                                                    && !provoke_targets_used.contains(&o.id)
                                                    && crate::rules::layers::calculate_characteristics(
                                                        state, o.id,
                                                    )
                                                    .map(|c| {
                                                        c.card_types.contains(&CardType::Creature)
                                                    })
                                                    .unwrap_or(false)
                                            })
                                            .map(|o| o.id)
                                            .next(); // OrdMap iteration is by ObjectId order
                                        t.data = target
                                            .map(|tgt| TriggerData::CombatProvoke { target: tgt });
                                        if let Some(tid) = target {
                                            provoke_targets_used.push(tid);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // CR 702.121a/b: Tag melee triggers for special stack handling.
                    // A SelfAttacks trigger is a melee trigger if its triggered ability
                    // description starts with "Melee" (set by builder.rs). Unlike
                    // Rampage which needs an N value, Melee always gives +1/+1 per
                    // opponent attacked -- no parameter to carry.
                    for t in &mut triggers[pre_len..] {
                        // PB-EF3b (load-bearing fix for granted Melee, e.g. Adriana's anthem):
                        // read layer-RESOLVED characteristics, not raw base, since
                        // `t.ability_index` indexes resolved `triggered_abilities` (a granted
                        // Melee's derived def only exists there — see
                        // layers::calculate_characteristics reconciliation). Still None-tolerant
                        // (CR 113.7a: the trigger source may have left this batch).
                        if let Some(chars) =
                            crate::rules::layers::calculate_characteristics(state, t.source)
                        {
                            if let Some(ta) = chars.triggered_abilities.get(t.ability_index) {
                                if ta.effect.is_none() && ta.description.starts_with("Melee") {
                                    t.kind = PendingTriggerKind::Melee;
                                }
                            }
                        }
                    }
                    // CR 702.154a: Enlist trigger post-processing.
                    // Each enlist pairing from combat.enlist_pairings for this attacker
                    // should match one "Enlist"-prefixed placeholder TriggeredAbilityDef.
                    // - If a pairing exists, tag the trigger with is_enlist_trigger=true
                    //   and the enlisted creature's ObjectId.
                    // - If no pairing exists for a given Enlist placeholder trigger,
                    //   REMOVE it (the player chose not to use that Enlist instance).
                    {
                        let enlist_pairings_for_attacker: Vec<ObjectId> = state
                            .combat
                            .as_ref()
                            .map(|c| {
                                c.enlist_pairings
                                    .iter()
                                    .filter(|(aid, _)| aid == attacker_id)
                                    .map(|(_, eid)| *eid)
                                    .collect()
                            })
                            .unwrap_or_default();
                        // Collect indices of Enlist placeholder triggers from this batch.
                        let mut enlist_trigger_indices: Vec<usize> = Vec::new();
                        for (i, t) in triggers[pre_len..].iter().enumerate() {
                            // CR 113.7a: the trigger source may have left this batch; use LKI.
                            if let Some(obj) = state.fizzle_object(t.source) {
                                if let Some(ta) =
                                    obj.characteristics.triggered_abilities.get(t.ability_index)
                                {
                                    if ta.description.starts_with("Enlist") {
                                        enlist_trigger_indices.push(pre_len + i);
                                    }
                                }
                            }
                        }
                        // Match pairings to placeholder triggers.
                        // Tag matched triggers; mark unmatched for removal.
                        let mut indices_to_remove: Vec<usize> = Vec::new();
                        let mut pairing_iter = enlist_pairings_for_attacker.iter();
                        for &idx in &enlist_trigger_indices {
                            if let Some(&enlisted_id) = pairing_iter.next() {
                                triggers[idx].kind = PendingTriggerKind::Enlist;
                                triggers[idx].data = Some(TriggerData::CombatEnlist {
                                    enlisted: enlisted_id,
                                });
                            } else {
                                // No pairing for this Enlist instance -- mark for removal.
                                indices_to_remove.push(idx);
                            }
                        }
                        // Remove unmatched Enlist placeholder triggers (reverse order to
                        // preserve indices).
                        for &idx in indices_to_remove.iter().rev() {
                            triggers.remove(idx);
                        }
                    }
                    // CR 701.43d / CR 607.2h: "When you do" linked exert trigger. Fires ONLY
                    // for attackers the player chose to exert this combat (stored in
                    // `combat.exerted_attackers` by `handle_declare_attackers`) -- NOT on
                    // every attack (contrast with a plain WhenAttacks trigger).
                    //
                    // `TriggerCondition::WhenExertedAsAttacks` has NO conversion loop
                    // in `build_face_ability_vectors`, so this registry scan is the
                    // ONLY dispatch path for it and there is nothing here to
                    // duplicate. `pb_dx47_dispatch_path_roster::r3` proves that
                    // mechanically rather than by assertion: it intersects the
                    // lowered set with the registry-scanned set and fails on any
                    // member.
                    //
                    // PB-DX47 (`OOS-DX24-4`): this comment used to read "CardDef-level
                    // `AbilityDefinition::Triggered` abilities are not converted to
                    // runtime `TriggeredAbilityDef` (that only happens in
                    // `enrich_spec_from_def` for tests), so -- mirroring the
                    // WhenDealsCombatDamageToPlayer CardDef scan above -- we collect
                    // them here". BOTH clauses were false and the mirror was the
                    // damage. (1) The lowering converts 34 distinct
                    // `TriggerCondition`s, and `WhenDealsCombatDamageToPlayer` was one
                    // of them, which is why that "mirror" was a DOUBLE dispatch.
                    // (2) `enrich_spec_from_def` is the PRODUCTION pregame path
                    // (`setup.rs:419/433/440`, `fuzz_setup.rs:119/130`), not a
                    // test-only helper. The claim is true of THIS arm's trigger and
                    // false as the general rule it was phrased as -- so it read as
                    // precedent, and got cited as one.
                    {
                        let was_exerted = state
                            .combat
                            .as_ref()
                            .map(|c| c.exerted_attackers.contains(attacker_id))
                            .unwrap_or(false);
                        if was_exerted {
                            // CR 113.7a: the attacking source may have left this batch; use LKI.
                            if let Some(src_obj) = state.fizzle_object(*attacker_id) {
                                if src_obj.zone == ZoneId::Battlefield && src_obj.is_phased_in() {
                                    let controller = src_obj.controller;
                                    let source_id = src_obj.id;
                                    if let Some(def) = src_obj
                                        .card_id
                                        .as_ref()
                                        .and_then(|cid| state.card_registry.get(cid.clone()))
                                    {
                                        // CR 603.4 (PB-DP6): gate at queue time. The
                                        // guard above already requires
                                        // `zone == Battlefield && is_phased_in()`.
                                        // OOS-DX1-4 Q3 (PB-DX24): an attacking transformed
                                        // DFC is ordinary, and the read side
                                        // (`resolution.rs`) is already face-aware -- read
                                        // the same face here.
                                        //
                                        // Residual, stated not glossed (fix cycle, review
                                        // Finding 7): this reads `is_transformed` at QUEUE
                                        // time; `resolution.rs:2177`/`:2209` documents its
                                        // own read as a CONSUME-time contract. They are the
                                        // SAME EXPRESSION, not the same EVALUATION -- a
                                        // permanent that transforms between this queue point
                                        // and the trigger's later resolution would desync.
                                        // Zero corpus exposure today (stage 1 measured 0
                                        // back-face Q3/Q4 shapes in the whole corpus), and
                                        // this fix is still strictly better than the
                                        // pre-PB-DX24 code on every state reachable today.
                                        // The durable fix is snapshotting the face onto
                                        // `PendingTrigger` itself, which is a HASH bump and
                                        // out of scope here; filed as OOS-DX24-8.
                                        let carddef_indices: Vec<usize> = def
                                            .effective_abilities(src_obj.is_transformed)
                                            .iter()
                                            .enumerate()
                                            .filter_map(|(idx, a)| match a {
                                                AbilityDefinition::Triggered {
                                                    trigger_condition:
                                                        TriggerCondition::WhenExertedAsAttacks,
                                                    intervening_if,
                                                    ..
                                                } => carddef_intervening_if_holds_at_queue_time(
                                                    state,
                                                    intervening_if.as_ref(),
                                                    controller,
                                                    source_id,
                                                )
                                                .then_some(idx),
                                                _ => None,
                                            })
                                            .collect();
                                        for ability_idx in carddef_indices {
                                            // PB-EF3 A2 (CR 601.2c/603.3d): `ability_idx` is a
                                            // raw index into `def.abilities` (not converted to
                                            // runtime `characteristics.triggered_abilities` --
                                            // see comment above). CardDefETB kind keeps the
                                            // raw-index/card-registry lookup authoritative for
                                            // both effect and target selection.
                                            triggers.push(PendingTrigger {
                                                ability_index: ability_idx,
                                                controller,
                                                kind: PendingTriggerKind::CardDefETB,
                                                triggering_event: Some(TriggerEvent::SelfAttacks),
                                                entering_object_id: Some(source_id),
                                                ..PendingTrigger::blank(
                                                    source_id,
                                                    controller,
                                                    PendingTriggerKind::CardDefETB,
                                                )
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // CR 702.105a: Dethrone -- "Whenever this creature attacks the player
                    // with the most life or tied for most life, put a +1/+1 counter on
                    // this creature."
                    // Only triggers when attacking a Player (not planeswalker/battle).
                    // CR 508.2a: condition checked at declaration time only.
                    if let crate::state::combat::AttackTarget::Player(def_pid) = attack_target {
                        // Find the maximum life total among all active (non-eliminated) players.
                        let defending_life = state
                            .expect_player(*def_pid)
                            .map(|p| p.life_total)
                            .unwrap_or(i32::MIN);
                        let max_life = state
                            .players
                            .values()
                            .filter(|p| !p.has_lost && !p.has_conceded)
                            .map(|p| p.life_total)
                            .max()
                            .unwrap_or(i32::MIN);
                        if defending_life >= max_life {
                            let pre_len_dethrone = triggers.len();
                            collect_triggers_for_event(
                                state,
                                &mut triggers,
                                TriggerEvent::SelfAttacksPlayerWithMostLife,
                                Some(*attacker_id),
                                None,
                            );
                            // Tag dethrone triggers with defending player for consistency
                            // with other attack triggers (e.g., annihilator).
                            for t in &mut triggers[pre_len_dethrone..] {
                                t.defending_player_id = defending_player;
                            }
                        }
                    }
                    // CR 702.149a: Training -- "Whenever this creature and at least one
                    // other creature with power greater than this creature's power attack,
                    // put a +1/+1 counter on this creature."
                    // The condition is: among ALL attackers declared in this batch, at
                    // least one other creature has strictly greater power than this creature.
                    // CR 508.2a: condition checked at declaration time only.
                    // Ruling 2021-11-19: "triggers only when both that creature and a
                    // creature with greater power are declared as attackers."
                    {
                        // Get the power of the current attacker (layer-aware).
                        let attacker_power =
                            crate::rules::layers::calculate_characteristics(state, *attacker_id)
                                .and_then(|c| c.power)
                                .unwrap_or(0);
                        // Check if any OTHER attacker in this batch has strictly greater power.
                        let has_greater_power_ally = attackers.iter().any(|(other_id, _)| {
                            *other_id != *attacker_id && {
                                let other_power = crate::rules::layers::calculate_characteristics(
                                    state, *other_id,
                                )
                                .and_then(|c| c.power)
                                .unwrap_or(0);
                                other_power > attacker_power
                            }
                        });
                        if has_greater_power_ally {
                            let pre_len_training = triggers.len();
                            collect_triggers_for_event(
                                state,
                                &mut triggers,
                                TriggerEvent::SelfAttacksWithGreaterPowerAlly,
                                Some(*attacker_id),
                                None,
                            );
                            // Tag training triggers with defending player for consistency
                            // with other attack triggers.
                            for t in &mut triggers[pre_len_training..] {
                                t.defending_player_id = defending_player;
                            }
                        }
                    }
                }
                // CR 701.54c (ring level >= 2): "Whenever your Ring-bearer attacks, draw a
                // card, then discard a card." Queue a RingLoot PendingTrigger for each
                // attacking creature that is this player's ring-bearer.
                for (attacker_id, _) in attackers {
                    let is_ring_bearer = state
                        .objects
                        .get(attacker_id)
                        .map(|o| {
                            o.designations
                                .contains(crate::state::game_object::Designations::RING_BEARER)
                        })
                        .unwrap_or(false);
                    if is_ring_bearer {
                        let ring_level = state
                            .expect_player(*attacking_player)
                            .map(|ps| ps.ring_level)
                            .unwrap_or(0);
                        if ring_level >= 2 {
                            triggers.push(PendingTrigger::blank(
                                *attacker_id,
                                *attacking_player,
                                PendingTriggerKind::RingLoot,
                            ));
                        }
                    }
                }
                // CR 508.1m / CR 603.2: AnyCreatureYouControlAttacks — fires on ALL battlefield
                // permanents for each creature that attacks, controller-filtered so only permanents
                // controlled by the same player as the attacking creature receive the trigger.
                //
                // Fires once per attacking creature (CR 603.2c — one trigger per attacker).
                // The attacking creature's ObjectId is passed as `entering_object` so that
                // collect_triggers_for_event can check controller match for controller_you filtering
                // (using entering_obj.controller == trigger_source.controller).
                //
                // PB-EF3 B1 (CR 508.4/113.7a): capture the defending player at dispatch time
                // into `defending_player_id` -- the same field/pattern SelfAttacks uses above
                // (line ~3573). This is threaded through PendingTrigger -> StackObject ->
                // EffectContext so `PlayerTarget::DefendingPlayer` and the Player-target case
                // of `EffectTarget::AttackTarget` resolve to the correct defender even if the
                // attacker later leaves combat (CR 506.4) before the trigger resolves.
                for (attacker_id, attack_target) in attackers {
                    let pre_len = triggers.len();
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::AnyCreatureYouControlAttacks,
                        None,
                        Some(*attacker_id),
                    );
                    let defending_player = match attack_target {
                        crate::state::combat::AttackTarget::Player(pid) => Some(*pid),
                        crate::state::combat::AttackTarget::Planeswalker(pw_id) => {
                            state.objects.get(pw_id).map(|obj| obj.controller)
                        }
                    };
                    for t in &mut triggers[pre_len..] {
                        t.defending_player_id = defending_player;
                    }
                }
                // CR 702.83a/b: Exalted — "Whenever a creature you control attacks alone."
                // If exactly one creature is declared as an attacker, fire exalted triggers
                // on ALL permanents controlled by the attacking player (not just the attacker).
                // CR 702.83b: "attacks alone" = exactly one creature declared as attacker.
                if attackers.len() == 1 {
                    let (lone_attacker_id, _) = &attackers[0];
                    let exalted_sources: Vec<ObjectId> = state
                        .objects
                        .values()
                        .filter(|obj| {
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller == *attacking_player
                        })
                        .map(|obj| obj.id)
                        .collect();
                    let pre_len = triggers.len();
                    for obj_id in exalted_sources {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::ControllerCreatureAttacksAlone,
                            Some(obj_id),
                            None,
                        );
                    }
                    // Tag exalted triggers with the lone attacker's ObjectId so
                    // flush_pending_triggers can set Target::Object(attacker_id) at index 0.
                    for t in &mut triggers[pre_len..] {
                        t.exalted_attacker_id = Some(*lone_attacker_id);
                    }
                }
                // CR 508.1: WheneverYouAttack — fires once when controller declares one or
                // more attackers. Fires per player (not per creature), so runs once outside
                // the per-attacker loop above.
                if !attackers.is_empty() {
                    let controller_sources: Vec<ObjectId> = state
                        .objects
                        .values()
                        .filter(|obj| {
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller == *attacking_player
                        })
                        .map(|obj| obj.id)
                        .collect();
                    for obj_id in controller_sources {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::ControllerAttacks,
                            Some(obj_id),
                            None,
                        );
                    }
                }
            }
            GameEvent::BlockersDeclared {
                blockers,
                defending_player,
            } => {
                // SelfBlocks: fires on each creature that is blocking (CR 603.5).
                for (blocker_id, _) in blockers {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::SelfBlocks,
                        Some(*blocker_id),
                        None,
                    );
                }
                // CR 702.25a: Flanking -- "Whenever this creature becomes blocked by
                // a creature without flanking, the blocking creature gets -1/-1 until
                // end of turn."
                // CR 702.25b: Multiple instances trigger separately.
                // CR 509.3f: The "without flanking" check is at declaration time.
                for (blocker_id, attacker_id) in blockers {
                    let attacker_obj = match state.objects.get(attacker_id) {
                        Some(obj) if obj.zone == ZoneId::Battlefield && obj.is_phased_in() => {
                            obj.clone()
                        }
                        _ => continue,
                    };
                    // CR 613.1f: Use layer-resolved keywords for Flanking checks
                    // (Humility removes Flanking; equipment/Auras can grant it).
                    let attacker_chars =
                        crate::rules::layers::expect_characteristics(state, *attacker_id);
                    if !attacker_chars.keywords.contains(&KeywordAbility::Flanking) {
                        continue;
                    }
                    // Check that the blocker does NOT have flanking (CR 702.25a).
                    let blocker_has_flanking = state
                        .objects
                        .get(blocker_id)
                        .map(|_b| {
                            crate::rules::layers::expect_characteristics(state, *blocker_id)
                                .keywords
                                .contains(&KeywordAbility::Flanking)
                        })
                        .unwrap_or(false);
                    if blocker_has_flanking {
                        continue;
                    }
                    // Count flanking instances from card definition (CR 702.25b).
                    let flanking_count = attacker_obj
                        .card_id
                        .as_ref()
                        .and_then(|cid| state.card_registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .filter(|a| {
                                    matches!(
                                        a,
                                        AbilityDefinition::Keyword(KeywordAbility::Flanking)
                                    )
                                })
                                .count()
                        })
                        .unwrap_or(1)
                        .max(1);
                    let controller = attacker_obj.controller;
                    let source_id = attacker_obj.id;
                    for _ in 0..flanking_count {
                        triggers.push(PendingTrigger {
                            triggering_event: Some(TriggerEvent::SelfBlocks),
                            data: Some(TriggerData::CombatFlanking {
                                blocker: *blocker_id,
                            }),
                            ..PendingTrigger::blank(
                                source_id,
                                controller,
                                PendingTriggerKind::Flanking,
                            )
                        });
                    }
                }
                // CR 509.1h / CR 702.45a / CR 702.23a: SelfBecomesBlocked -- fires
                // on each ATTACKER that has at least one blocker declared against it.
                // Collect unique attacker IDs to ensure each triggers only once
                // (CR 509.3c: "generally triggers only once each combat").
                let mut blocked_attackers: Vec<ObjectId> = blockers
                    .iter()
                    .map(|(_, attacker_id)| *attacker_id)
                    .collect();
                blocked_attackers.sort();
                blocked_attackers.dedup();
                for attacker_id in blocked_attackers {
                    let pre_len = triggers.len();
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::SelfBecomesBlocked,
                        Some(attacker_id),
                        None,
                    );
                    // CR 702.23a: Tag Rampage triggers with kind=Rampage and
                    // data=TriggerData::CombatRampage { n }.
                    // Each Rampage(n) keyword on the attacker generates a TriggeredAbilityDef
                    // with description starting "Rampage N (CR 702.23a):". We detect these
                    // and set the custom StackObjectKind by tagging the PendingTrigger.
                    // CR 113.7a: the blocked attacker may have left this batch; use LKI.
                    if let Some(obj) = state.fizzle_object(attacker_id) {
                        for t in &mut triggers[pre_len..] {
                            if let Some(ability_def) =
                                obj.characteristics.triggered_abilities.get(t.ability_index)
                            {
                                if ability_def.description.starts_with("Rampage") {
                                    // Find the matching Rampage(n) keyword for this trigger.
                                    // Each Rampage instance generates its own TriggeredAbilityDef
                                    // with a unique description containing "Rampage {n}".
                                    for kw in &obj.characteristics.keywords {
                                        if let KeywordAbility::Rampage(n) = kw {
                                            if ability_def
                                                .description
                                                .contains(&format!("Rampage {n}"))
                                            {
                                                t.kind = PendingTriggerKind::Rampage;
                                                t.data = Some(TriggerData::CombatRampage { n: *n });
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // CR 509.3c / CR 702.130a: Tag all SelfBecomesBlocked triggers with
                    // the defending player so flush_pending_triggers sets Target::Player at
                    // index 0. This enables PlayerTarget::DeclaredTarget { index: 0 } in
                    // Afflict's LoseLife effect to resolve to the correct defending player
                    // in multiplayer games (CR 508.5). Bushido and Rampage target the
                    // source object rather than a player, so tagging defending_player_id
                    // has no effect on them (flush_pending_triggers only uses it for the
                    // LoseLife path via DeclaredTarget; Bushido/Rampage use Source/custom).
                    for t in &mut triggers[pre_len..] {
                        t.defending_player_id = Some(*defending_player);
                    }
                }
                // CR 701.54c (ring level >= 3): "Whenever your Ring-bearer becomes blocked
                // by a creature, that creature's controller sacrifices it at end of combat."
                //
                // The blocker is tagged with `ring_block_sacrifice_at_eoc = true` directly
                // in `handle_declare_blockers` in combat.rs (which has mutable state access).
                // That tag is checked in `end_combat()` in turn_actions.rs. No PendingTrigger
                // is pushed here — the EOC tag pattern (used by Decayed/Myriad) avoids the
                // bugs of an immediate-resolution trigger: wrong timing and wrong sacrifice target.
            }
            GameEvent::PermanentTargeted {
                target_id,
                targeting_stack_id,
                targeting_controller,
            } => {
                // CR 702.21a: Ward triggers when this permanent becomes the target
                // of a spell or ability an opponent controls. Only triggers if the
                // targeting player is an opponent (not the permanent's controller).
                // CR 113.7a: the targeted permanent may have left this batch; use LKI.
                if let Some(obj) = state.fizzle_object(*target_id) {
                    if obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && obj.controller != *targeting_controller
                    {
                        let pre_len = triggers.len();
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::SelfBecomesTargetByOpponent,
                            Some(*target_id),
                            None,
                        );
                        // Tag ward triggers with the targeting stack object ID so
                        // flush_pending_triggers can set the correct target on the
                        // ward triggered ability's stack entry (for CounterSpell resolution).
                        for t in &mut triggers[pre_len..] {
                            t.targeting_stack_id = Some(*targeting_stack_id);
                        }
                    }
                }
                // PB-AC6 / CR 601.2c / 602.2b / 603.2: global "becomes the target of a
                // spell/ability" dispatch. Distinct from the Ward block above (which is
                // self + spell-or-ability + opponent-only, hardcoded). This dispatch
                // reads per-card scope/by_opponent/include_abilities params directly off
                // each candidate source's `TriggerEvent::PermanentBecomesTarget` variant
                // -- it cannot use the generic `collect_triggers_for_event` equality scan
                // because those params differ per card.
                collect_permanent_becomes_target_triggers(
                    state,
                    &mut triggers,
                    *target_id,
                    *targeting_stack_id,
                    *targeting_controller,
                );
            }
            GameEvent::CreatureDied {
                object_id: pre_death_object_id,
                new_grave_id,
                controller: death_controller,
                pre_death_counters,
                pre_death_power,
                pre_death_characteristics,
            } => {
                // CR 603.6c / CR 603.10a / CR 700.4: "When ~ dies" triggers look back in time.
                // The creature is now in the graveyard, but its characteristics (including
                // triggered_abilities) are preserved by move_object_to_zone. Check the graveyard
                // object for SelfDies triggers rather than trying to find the battlefield object
                // (which no longer exists at trigger-check time).
                // CR 603.10a: the dies trigger reads the graveyard object (LKI); it may already be gone.
                if let Some(obj) = state.fizzle_object(*new_grave_id) {
                    for (idx, trigger_def) in
                        obj.characteristics.triggered_abilities.iter().enumerate()
                    {
                        if trigger_def.trigger_on != TriggerEvent::SelfDies {
                            continue;
                        }
                        // CR 603.4: Check intervening-if clause at trigger time.
                        // Pass pre_death_counters for persist/undying counter checks (CR 702.79a).
                        if let Some(ref cond) = trigger_def.intervening_if {
                            // CR 603.10a: SelfDies is a look-back trigger — source is
                            // now in the graveyard (*new_grave_id).
                            if !check_intervening_if(
                                state,
                                cond,
                                *death_controller,
                                *new_grave_id,
                                Some(pre_death_counters),
                                InterveningIfMoment::TriggerTimeLookBack,
                                &[],
                            ) {
                                continue;
                            }
                        }
                        // CR 702.43a: Detect if this is a Modular trigger. Tag it with
                        // the +1/+1 counter count from last-known information so that
                        // flush_pending_triggers can create a KeywordTrigger (Modular) stack entry.
                        let is_modular = trigger_def.description.contains("Modular (CR 702.43a)");
                        let modular_counter_count = if is_modular {
                            Some(
                                pre_death_counters
                                    .get(&CounterType::PlusOnePlusOne)
                                    .copied()
                                    .unwrap_or(0),
                            )
                        } else {
                            None
                        };
                        let kind = if is_modular {
                            PendingTriggerKind::Modular
                        } else {
                            PendingTriggerKind::Normal
                        };
                        let data = modular_counter_count
                            .map(|n| TriggerData::DeathModular { counter_count: n });
                        triggers.push(PendingTrigger {
                            ability_index: idx,
                            // CR 603.3a: use the controller captured at death time (before
                            // move_object_to_zone reset it to owner). This correctly handles
                            // stolen creatures — if Player A controls Player B's creature and
                            // it dies, the trigger is controlled by Player A.
                            triggering_event: Some(TriggerEvent::SelfDies),
                            data,
                            // CR 603.10a: capture the dying creature's pre-death counters into LKI
                            // so EffectAmount::CounterCountAtLastKnownInformation can resolve.
                            lki_counters: pre_death_counters.clone(),
                            // CR 603.10a: capture the dying creature's layer-resolved power into LKI
                            // so EffectAmount::SourcePowerAtLastKnownInformation can resolve.
                            lki_power: *pre_death_power,
                            ..PendingTrigger::blank(*new_grave_id, *death_controller, kind)
                        });
                    }
                }
                // CR 603.10a: SelfLeavesBattlefield — fires on the dead creature (LKI).
                // Check graveyard object for WhenLeavesBattlefield triggers.
                // CR 603.10a: leaves-battlefield trigger reads the graveyard object (LKI); it may already be gone.
                if let Some(dead_obj) = state.fizzle_object(*new_grave_id) {
                    let controller = *death_controller;
                    for (idx, trigger_def) in dead_obj
                        .characteristics
                        .triggered_abilities
                        .iter()
                        .enumerate()
                    {
                        if trigger_def.trigger_on != TriggerEvent::SelfLeavesBattlefield {
                            continue;
                        }
                        if let Some(ref cond) = trigger_def.intervening_if {
                            // CR 603.10a: look-back trigger — source is the graveyard object.
                            if !check_intervening_if(
                                state,
                                cond,
                                controller,
                                *new_grave_id,
                                None,
                                InterveningIfMoment::TriggerTimeLookBack,
                                &[],
                            ) {
                                continue;
                            }
                        }
                        triggers.push(PendingTrigger {
                            ability_index: idx,
                            triggering_event: Some(TriggerEvent::SelfLeavesBattlefield),
                            // CR 603.10a: same LKI snapshot for leaves-battlefield triggers.
                            lki_counters: pre_death_counters.clone(),
                            // CR 603.10a: propagate LKI source-power snapshot.
                            lki_power: *pre_death_power,
                            ..PendingTrigger::blank(
                                *new_grave_id,
                                controller,
                                PendingTriggerKind::Normal,
                            )
                        });
                    }
                }
                // CR 702.59a: Recover triggers. When a creature enters a player's
                // graveyard from the battlefield, each Recover card in that same
                // player's graveyard triggers independently.
                //
                // The dying creature itself CAN trigger its own Recover (if it has
                // Recover) because it is now in the graveyard when the event is
                // processed (CR 702.59a: "while the card with recover is in a player's
                // graveyard").
                //
                // Identify the owner's graveyard by looking at the new_grave_id object.
                // CR 603.10a: Recover reads the graveyard object (LKI); it may already be gone.
                if let Some(dead_obj) = state.fizzle_object(*new_grave_id) {
                    let owner_gy = crate::state::zone::ZoneId::Graveyard(dead_obj.owner);
                    // Collect Recover cards in the owner's graveyard.
                    // Use a snapshot to avoid borrow conflicts during iteration.
                    let recover_cards: Vec<(ObjectId, ManaCost, PlayerId)> = state
                        .objects
                        .iter()
                        .filter_map(|(&obj_id, obj)| {
                            if obj.zone != owner_gy {
                                return None;
                            }
                            // Quick check: does this object have the Recover keyword marker?
                            if !obj
                                .characteristics
                                .keywords
                                .iter()
                                .any(|kw| *kw == KeywordAbility::Recover)
                            {
                                return None;
                            }
                            // Look up the recover cost from the card registry.
                            let cost = find_recover_cost(&obj.card_id, &state.card_registry)?;
                            Some((obj_id, cost, obj.owner))
                        })
                        .collect();
                    for (recover_id, cost, card_owner) in recover_cards {
                        triggers.push(PendingTrigger {
                            triggering_event: Some(TriggerEvent::SelfDies),
                            data: Some(TriggerData::DeathRecover {
                                recover_card: recover_id,
                                recover_cost: cost,
                            }),
                            ..PendingTrigger::blank(
                                recover_id,
                                card_owner,
                                PendingTriggerKind::Recover,
                            )
                        });
                    }
                }
                // CR 702.72a: Champion LTB trigger. When a Champion permanent leaves the
                // battlefield (here: dies), check if it had a champion_exiled_card and
                // fire the LTB trigger to return that card to the battlefield.
                //
                // CR 603.10a: LTB triggers look back in time -- champion_exiled_card is
                // preserved in move_object_to_zone so we can read it from the graveyard object.
                // CR 603.10a: LKI read; the graveyard object may already be gone.
                if let Some(dead_obj) = state.fizzle_object(*new_grave_id) {
                    if let Some(exiled_id) = dead_obj.champion_exiled_card {
                        let champion_controller = *death_controller;
                        triggers.push(PendingTrigger {
                            triggering_event: Some(TriggerEvent::SelfDies),
                            data: Some(TriggerData::LTBChampion {
                                exiled_card: exiled_id,
                            }),
                            ..PendingTrigger::blank(
                                *new_grave_id,
                                champion_controller,
                                PendingTriggerKind::ChampionLTB,
                            )
                        });
                    }
                }
                // CR 702.55b: When a creature with Haunt dies, exile the dying creature
                // haunting another target creature.
                // Look back in time via new_grave_id to check if the dead creature had Haunt.
                // CR 603.10a: Haunt reads the graveyard object (LKI); it may already be gone.
                if let Some(dead_obj) = state.fizzle_object(*new_grave_id) {
                    if dead_obj
                        .characteristics
                        .keywords
                        .iter()
                        .any(|kw| *kw == KeywordAbility::Haunt)
                    {
                        let haunt_controller = *death_controller;
                        let haunt_card_id = dead_obj.card_id.clone();
                        triggers.push(PendingTrigger {
                            triggering_event: Some(TriggerEvent::SelfDies),
                            data: Some(TriggerData::DeathHauntExile {
                                haunt_card: *new_grave_id,
                                haunt_card_id,
                            }),
                            ..PendingTrigger::blank(
                                *new_grave_id,
                                haunt_controller,
                                PendingTriggerKind::HauntExile,
                            )
                        });
                    }
                }
                // CR 702.55c: When the creature a haunt card haunts dies, fire the haunted
                // creature dies trigger for each haunt card in exile that targets this creature.
                // Scan exile for objects whose haunting_target matches the pre-death battlefield ID.
                {
                    let dying_id = *pre_death_object_id;
                    let haunt_exiled: Vec<(
                        ObjectId,
                        Option<crate::state::player::CardId>,
                        PlayerId,
                    )> = state
                        .objects
                        .iter()
                        .filter_map(|(&exiled_obj_id, obj)| {
                            // Must be in the exile zone.
                            if obj.zone != crate::state::zone::ZoneId::Exile {
                                return None;
                            }
                            // Must haunt the dying creature (pre-death battlefield ObjectId).
                            if obj.haunting_target != Some(dying_id) {
                                return None;
                            }
                            Some((exiled_obj_id, obj.card_id.clone(), obj.controller))
                        })
                        .collect();
                    for (haunt_obj_id, haunt_card_id, haunt_controller) in haunt_exiled {
                        // CR 603.4 (PB-DX1, OOS-DP6-9): the queue-time intervening-if
                        // gate for this trigger lives in `flush_pending_triggers`'s
                        // `PendingTriggerKind::HauntedCreatureDies` arm, NOT here.
                        // `check_triggers` only has `&GameState` and CR 702.55c
                        // requires clearing `haunting_target` on suppression (a
                        // suppressed trigger still spends the one-shot haunting
                        // relationship, exactly like a resolved one — mirroring
                        // `resolution.rs`'s "regardless of whether the
                        // intervening-if held" clear) — a mutation `check_triggers`
                        // cannot perform. Gating here and clearing at flush time
                        // would leave a suppressed trigger's exiled card still
                        // haunting a dead creature's `ObjectId` (review Finding 7).
                        // This is the SAME "gate at flush time" shape
                        // `once_per_turn` already uses in this function, applied to
                        // the one trigger family that also needs a mutation on
                        // suppression.
                        triggers.push(PendingTrigger {
                            triggering_event: Some(TriggerEvent::HauntedCreatureDies),
                            data: Some(TriggerData::DeathHauntedCreatureDies {
                                haunt_source: haunt_obj_id,
                                haunt_card_id,
                            }),
                            ..PendingTrigger::blank(
                                haunt_obj_id,
                                haunt_controller,
                                PendingTriggerKind::HauntedCreatureDies,
                            )
                        });
                    }
                }
                // CR 603.10a / CR 603.2: AnyCreatureDies — fires on ALL battlefield permanents
                // when any creature dies. death_filter is applied inside collect_triggers_for_event
                // to check controller_you/controller_opponent/exclude_self/nontoken_only against
                // the dying creature's PRE-DEATH state.
                //
                // We pass new_grave_id as the entering_object parameter (reused for death filter
                // checks — the dying creature is now in the graveyard, but its pre-death controller
                // is stored in the death_controller parameter from the event).
                //
                // Important: We also store the pre-death controller so collect_triggers_for_event
                // can compare it against trigger sources' controllers for the controller_you filter.
                // Since collect_triggers_for_event reads entering_object from state.objects, and
                // the dying creature is now in the graveyard with controller reset to owner by
                // move_object_to_zone, we must filter using death_controller directly here.
                {
                    let dying_obj_id = *new_grave_id;
                    let dying_controller = *death_controller;
                    // PB-DX28: one bare lookup feeds both `dying_is_token` and
                    // `dying_owner` (ownership is invariant across the zone move, see
                    // the `owner_you`/`owner_opponent` comment below) rather than a
                    // second `.objects.get(&dying_obj_id)` site (SR-25 ratchet).
                    let dying_obj_snapshot = state.objects.get(&dying_obj_id);
                    let dying_is_token = dying_obj_snapshot.is_some_and(|o| o.is_token);
                    let dying_owner = dying_obj_snapshot.map(|o| o.owner);
                    // Collect all battlefield permanents that have AnyCreatureDies triggers.
                    let candidate_ids: Vec<ObjectId> = state
                        .objects
                        .values()
                        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in())
                        .map(|obj| obj.id)
                        .collect();
                    for obj_id in candidate_ids {
                        let Some(obj) = state.expect_object(obj_id) else {
                            continue;
                        };
                        let resolved_chars =
                            crate::rules::layers::expect_characteristics(state, obj_id);
                        for (idx, trigger_def) in
                            resolved_chars.triggered_abilities.iter().enumerate()
                        {
                            if trigger_def.trigger_on != TriggerEvent::AnyCreatureDies {
                                continue;
                            }
                            // Apply death_filter using the pre-death controller (not graveyard object's
                            // controller, which was reset to owner by move_object_to_zone).
                            if let Some(ref df) = trigger_def.death_filter {
                                // controller_you: dying creature must share controller with trigger source
                                if df.controller_you && dying_controller != obj.controller {
                                    continue;
                                }
                                // controller_opponent: dying creature must be controlled by an opponent
                                if df.controller_opponent && dying_controller == obj.controller {
                                    continue;
                                }
                                // PB-DX28 (CR 108.3 / CR 404.3): owner_you / owner_opponent —
                                // ownership never changes across a zone move
                                // (`move_object_to_zone` always carries
                                // `owner: old_object.owner` forward), so the dying
                                // creature's owner can be read directly off the
                                // now-in-graveyard object (`dying_owner`, computed
                                // above alongside `dying_is_token`); no pre-death
                                // capture needed, unlike `controller` above.
                                if df.owner_you || df.owner_opponent {
                                    if df.owner_you && dying_owner != Some(obj.owner) {
                                        continue;
                                    }
                                    if df.owner_opponent && dying_owner == Some(obj.owner) {
                                        continue;
                                    }
                                }
                                // exclude_self: dying creature must not be the trigger source
                                if df.exclude_self && dying_obj_id == obj_id {
                                    continue;
                                }
                                // nontoken_only: dying creature must not be a token
                                if df.nontoken_only && dying_is_token {
                                    continue;
                                }
                            }
                            // PB-N: triggering_creature_filter — subtype/color/type filter on
                            // the dying creature. Evaluated against PRE-DEATH characteristics
                            // preserved on the graveyard object by move_object_to_zone
                            // (CR 603.10a LKI). Placed after death_filter checks (cheap first).
                            if let Some(ref creature_filter) =
                                trigger_def.triggering_creature_filter
                            {
                                let dying_obj = match state.objects.get(&dying_obj_id) {
                                    Some(o) => o,
                                    None => continue,
                                };
                                // is_token check: runtime field on GameObject, not in Characteristics.
                                if creature_filter.is_token && !dying_is_token {
                                    continue;
                                }
                                // CR 603.10a / CR 613.1d: Use PRE-DEATH characteristics snapshot
                                // captured before move_object_to_zone (threaded via GameEvent).
                                // This preserves battlefield-gated layer effects (SingleObject,
                                // AttachedCreature, etc.) that drop out after zone change per
                                // CR 400.7. Fixes BASELINE-LKI-01: a creature granted a subtype
                                // (e.g. Zombie) by a continuous effect while on the battlefield
                                // must match "whenever a Zombie you control dies" triggers.
                                // Fallback to graveyard object's preserved characteristics if the
                                // snapshot is absent (e.g. events deserialized from old recordings).
                                let dying_chars = pre_death_characteristics
                                    .clone()
                                    .unwrap_or_else(|| dying_obj.characteristics.clone());
                                if !crate::effects::matches_filter(&dying_chars, creature_filter) {
                                    continue;
                                }
                            }
                            // CR 603.4: Check intervening-if at trigger time. Not a
                            // look-back trigger — the SOURCE (observer) is `obj_id`,
                            // still on the battlefield; only the *dying* creature is LKI.
                            if let Some(ref cond) = trigger_def.intervening_if {
                                if !check_intervening_if(
                                    state,
                                    cond,
                                    obj.controller,
                                    obj_id,
                                    None,
                                    InterveningIfMoment::TriggerTime,
                                    &[],
                                ) {
                                    continue;
                                }
                            }
                            triggers.push(PendingTrigger {
                                ability_index: idx,
                                triggering_event: Some(TriggerEvent::AnyCreatureDies),
                                // Reuse entering_object_id to carry the dying creature's graveyard
                                // ObjectId for post-trigger use if needed.
                                entering_object_id: Some(dying_obj_id),
                                ..PendingTrigger::blank(
                                    obj_id,
                                    obj.controller,
                                    PendingTriggerKind::Normal,
                                )
                            });
                        }
                    }
                }
                // CR 113.6b/113.6m (PB-DX24): a `trigger_zone: Some(Graveyard)` death
                // trigger fires from the graveyard, not from the battlefield. Mirrors
                // the ETB call above (search `AnyPermanentEntersBattlefield`).
                collect_graveyard_carddef_triggers(
                    state,
                    &mut triggers,
                    event,
                    Some(*new_grave_id),
                    arrived_in_graveyard_this_batch,
                );
            }
            GameEvent::AuraFellOff {
                new_grave_id,
                pre_lba_counters: aura_lki_counters,
                pre_lba_power: aura_lki_power,
                ..
            } => {
                // CR 603.6c / CR 603.10a: "When ~ is put into a graveyard from the battlefield"
                // triggers on Auras fire when the Aura moves to the graveyard via SBA 704.5m.
                // The Aura's characteristics (including triggered_abilities) are preserved in
                // the graveyard object by move_object_to_zone — same look-back pattern as
                // CreatureDied. Controller defaults to owner (as reset by move_object_to_zone).
                // CR 603.10a: Aura LTB reads the graveyard object (LKI); it may already be gone.
                if let Some(obj) = state.fizzle_object(*new_grave_id) {
                    let controller = obj.controller;
                    for (idx, trigger_def) in
                        obj.characteristics.triggered_abilities.iter().enumerate()
                    {
                        // Fire both SelfDies and SelfLeavesBattlefield triggers.
                        if trigger_def.trigger_on != TriggerEvent::SelfDies
                            && trigger_def.trigger_on != TriggerEvent::SelfLeavesBattlefield
                        {
                            continue;
                        }
                        // CR 603.4: Check intervening-if clause at trigger time.
                        // CR 603.10a: look-back trigger — source is the graveyard object.
                        if let Some(ref cond) = trigger_def.intervening_if {
                            if !check_intervening_if(
                                state,
                                cond,
                                controller,
                                *new_grave_id,
                                None,
                                InterveningIfMoment::TriggerTimeLookBack,
                                &[],
                            ) {
                                continue;
                            }
                        }
                        let event = trigger_def.trigger_on.clone();
                        triggers.push(PendingTrigger {
                            ability_index: idx,
                            triggering_event: Some(event),
                            // CR 603.10a: LKI snapshot from event — counters before zone change.
                            lki_counters: aura_lki_counters.clone(),
                            // CR 603.10a: LKI source-power snapshot.
                            lki_power: *aura_lki_power,
                            ..PendingTrigger::blank(
                                *new_grave_id,
                                controller,
                                PendingTriggerKind::Normal,
                            )
                        });
                    }
                }
            }
            GameEvent::Surveilled { player, .. } => {
                // CR 701.25d: "Whenever you surveil" triggers on all permanents
                // controlled by the surveilling player.
                let controller_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller == *player
                    })
                    .map(|obj| obj.id)
                    .collect();
                for obj_id in controller_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::ControllerSurveils,
                        Some(obj_id),
                        None,
                    );
                }
            }
            GameEvent::Investigated { player, .. } => {
                // CR 701.16a: "Whenever you investigate" triggers on all permanents
                // controlled by the investigating player.
                let controller_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller == *player
                    })
                    .map(|obj| obj.id)
                    .collect();
                for obj_id in controller_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::ControllerInvestigates,
                        Some(obj_id),
                        None,
                    );
                }
            }
            GameEvent::Amassed { player, .. } => {
                // CR 701.47a: "Whenever you amass" triggers on all permanents
                // controlled by the amassing player. No TriggerEvent::ControllerAmasses
                // exists yet (no card currently uses this trigger condition), so this
                // arm is a no-op placeholder for forward compatibility. When a card
                // with "whenever you amass" is implemented, add a TriggerEvent variant
                // and update collect_triggers_for_event here.
                let _ = player;
            }
            GameEvent::Connived { object_id, .. } => {
                // CR 701.50b: "Whenever [this creature] connives" triggers fire even if
                // the creature left the battlefield before the Connived event is processed.
                // Scryfall ruling (Psychic Pickpocket, 2022-04-29): "If ... that creature
                // has left the battlefield, the creature still connives. Abilities that
                // trigger 'when [that creature] connives' will trigger."
                //
                // `collect_triggers_for_event` enforces a zone == Battlefield check at
                // line 1518 and would skip off-battlefield objects. To comply with CR
                // 701.50b, we bypass the helper and generate the trigger inline,
                // accepting the object in ANY zone.
                // CR 701.50b / CR 113.7a: the connive source may have left any zone; use LKI.
                if let Some(obj) = state.fizzle_object(*object_id) {
                    for (idx, trigger_def) in
                        obj.characteristics.triggered_abilities.iter().enumerate()
                    {
                        if trigger_def.trigger_on != TriggerEvent::SourceConnives {
                            continue;
                        }
                        // CR 603.4: intervening-if check at trigger time.
                        // CR 701.50b / 603.10a: the source may already have left the
                        // battlefield (connive fires even then) — look-back.
                        if let Some(ref cond) = trigger_def.intervening_if {
                            if !check_intervening_if(
                                state,
                                cond,
                                obj.controller,
                                *object_id,
                                None,
                                InterveningIfMoment::TriggerTimeLookBack,
                                &[],
                            ) {
                                continue;
                            }
                        }
                        triggers.push(PendingTrigger {
                            ability_index: idx,
                            triggering_event: Some(TriggerEvent::SourceConnives),
                            ..PendingTrigger::blank(
                                *object_id,
                                obj.controller,
                                PendingTriggerKind::Normal,
                            )
                        });
                    }
                }
            }
            GameEvent::CombatDamageDealt { assignments } => {
                // CR 510.3a / CR 603.2: "Whenever ~ deals combat damage to a player"
                // triggers fire for each creature that dealt > 0 combat damage to a player.
                // CR 603.2g: damage with amount == 0 (fully prevented) does not trigger.
                // CR 603.10: NOT a look-back trigger — creature must be on battlefield;
                // collect_triggers_for_event checks obj.zone == Battlefield internally.
                for assignment in assignments {
                    if assignment.amount == 0 {
                        continue; // CR 603.2g: damage was fully prevented
                    }
                    if matches!(assignment.target, CombatDamageTarget::Player(_)) {
                        let pre_len = triggers.len();
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::SelfDealsCombatDamageToPlayer,
                            Some(assignment.source),
                            None,
                        );
                        // CR 510.3a: Populate combat data on SelfDealsCombatDamageToPlayer triggers
                        // so EffectAmount::CombatDamageDealt and PlayerTarget::DamagedPlayer resolve
                        // correctly (e.g., Lathril creating tokens equal to damage dealt).
                        if let CombatDamageTarget::Player(damaged_pid) = &assignment.target {
                            for t in &mut triggers[pre_len..] {
                                t.damaged_player = Some(*damaged_pid);
                                t.combat_damage_amount = assignment.amount;
                                t.entering_object_id = Some(assignment.source);
                            }
                        }
                        // CR 510.3a / CR 603.2 (PB-DX47, `OOS-DX24-4`): the
                        // runtime lowering above is the SINGLE authoritative
                        // dispatch for this trigger. A card-registry scan used to
                        // stand here and push a SECOND `PendingTrigger`
                        // (`PendingTriggerKind::CardDefETB`) for the same ability,
                        // justified by a comment claiming the CardDef ability was
                        // "not converted to runtime `TriggeredAbilityDef` (that
                        // only happens in `enrich_spec_from_def` for tests)".
                        //
                        // Both halves of that claim were false at HEAD, and PB-DX47
                        // measured the consequence on a game built through the
                        // PRODUCTION pregame path (`setup::build_initial_state`):
                        // ONE `CombatDamageDealt` pushed TWO triggers -- `Normal`
                        // from `collect_triggers_for_event` just above,
                        // `CardDefETB` from the scan -- and
                        // `drana_liberator_of_malakir`, a `Complete` deck-legal def
                        // printing ONE `+1/+1` counter, put TWO on its lone
                        // attacker.
                        //
                        // 1. `build_face_ability_vectors`
                        //    (`testing/replay_harness.rs`) has a dedicated loop
                        //    converting exactly this `TriggerCondition` into a
                        //    `TriggeredAbilityDef { trigger_on:
                        //    TriggerEvent::SelfDealsCombatDamageToPlayer, .. }`.
                        //    PB-DX1 even extended that loop (`intervening_if`
                        //    propagation) without reconciling it with the comment.
                        // 2. `enrich_spec_from_def` is the PRODUCTION pregame path
                        //    -- `setup.rs:419/433/440` (commander, opening hand,
                        //    library) and `fuzz_setup.rs:119/130` -- not a
                        //    test-only helper. Every object in every real game is
                        //    built through it.
                        //
                        // The lowering is also the CR-correct one of the two, which
                        // is why it is the survivor rather than merely the
                        // incumbent: `collect_triggers_for_event` reads
                        // LAYER-RESOLVED characteristics (CR 613.1f), so Humility /
                        // Dress Down / any `RemoveAllAbilities` effect suppresses
                        // the trigger, while a raw registry scan bypasses layers
                        // entirely; it sees granted and copied abilities; and it
                        // sees tokens, which carry no `card_id` for a registry scan
                        // to find.
                        //
                        // The scan's own historical justification (PB-EF3 A2 /
                        // EF-W-MISS-10: Throat Slitter's declared `targets` must
                        // survive auto-target selection) is DISCHARGED, not
                        // ignored: the lowering copies `targets` verbatim, and
                        // `flush_sorted` reads `ab.targets` for a `Normal` trigger
                        // through the same code path it reads the registry for a
                        // `CardDefETB` one. `pb_dx47_dispatch_path_roster.rs`
                        // pins both facts.
                        //
                        // The lowering does not carry `modes`: it pre-selects
                        // mode 0 (CR 700.2b bot fallback). This comment's first two
                        // drafts each got that wrong, in opposite directions, and
                        // both were corrected by EXECUTING something:
                        //
                        // (1) "ZERO corpus defs pair `modes` with this
                        //     `TriggerCondition`" -- refuted by
                        //     `pb_dx47_dispatch_path_roster::r5b` on its first run.
                        //     The population is ONE, `glissa_sunslayer`, three
                        //     modes, `Completeness::partial` (so `validate_deck`
                        //     refuses it and deck-legal exposure is zero).
                        // (2) "a real capability the fix gives up" -- refuted by
                        //     `primitives::pb_dx47_modal_trigger_mode_zero::t1`.
                        //     NOTHING modal is lost, because nothing modal was ever
                        //     offered: at PB-DX47 time `flush_sorted` hard-coded
                        //     `modes_chosen = vec![0]` in BOTH arms of its modal
                        //     branch for any `StackObjectKind::TriggeredAbility`,
                        //     `Normal` and `CardDefETB` alike, and
                        //     `resolution.rs`'s modal replacement sits OUTSIDE the
                        //     `is_carddef_etb` branch, so both kinds resolve
                        //     `modes.modes[0]`. `modal_trigger` (CR 603.3c) is a
                        //     standing `AutoChosen` row in
                        //     `core::decision_site_walk`. Measured: restoring the
                        //     deleted scan takes that probe from +1 life to +2 --
                        //     mode 0 TWICE, not a mode the player picked.
                        //
                        //     PB-DX35 (`OOS-DX4-2`) replaced the hard-code with
                        //     `trigger_modal_plan`, a CR 700.2b-legal first-mode
                        //     pick. For `glissa_sunslayer` this changes nothing
                        //     observable: its modal ability's registry index (2)
                        //     still does not match the `Normal`-kind trigger's
                        //     RUNTIME ability_index (0) -- the census in
                        //     execution-notes §0.5 and `core::
                        //     pb_dx35_modal_trigger_roster::r2` -- so the registry
                        //     lookup at index 0 still finds a non-`Triggered`
                        //     ability and `modes` is still `None`, which is the SAME
                        //     "not modal" fallback (`modes_chosen: vec![]`) as
                        //     before. Filed as `OOS-DX35-1`, not fixed by this batch
                        //     (execution-notes §0.5 "why it is not fixed").
                        //
                        // `OOS-DX47-3` therefore stays open as the STRUCTURAL gap
                        // (`TriggeredAbilityDef` has no `modes` field, so the day
                        // CR 603.3c is actually served the lowering must carry it,
                        // which is a HASH bump) with its behavioural delta measured
                        // at ZERO -- not as a regression this batch shipped.
                        // CR 702.115a: Ingest -- "Whenever this creature deals combat
                        // damage to a player, that player exiles the top card of
                        // their library."
                        // CR 702.115b: Multiple instances trigger separately.
                        // CR 113.7a: the damage source may have left the battlefield; use LKI.
                        if let Some(obj) = state.fizzle_object(assignment.source) {
                            if obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj
                                    .characteristics
                                    .keywords
                                    .contains(&KeywordAbility::Ingest)
                            {
                                // Already guaranteed by the `if matches!(..., Player(_))`
                                // guard above — use `let...else` instead of unreachable!().
                                let CombatDamageTarget::Player(damaged_player) = &assignment.target
                                else {
                                    continue;
                                };
                                let damaged_player = *damaged_player;
                                // Count ingest instances from card definition for
                                // CR 702.115b: multiple instances trigger separately.
                                let ingest_count = obj
                                    .card_id
                                    .as_ref()
                                    .and_then(|cid| state.card_registry.get(cid.clone()))
                                    .map(|def| {
                                        def.abilities
                                            .iter()
                                            .filter(|a| {
                                                matches!(
                                                    a,
                                                    crate::cards::card_definition::AbilityDefinition::Keyword(
                                                        KeywordAbility::Ingest
                                                    )
                                                )
                                            })
                                            .count()
                                    })
                                    .unwrap_or(1)
                                    .max(1);
                                let controller = obj.controller;
                                let source_id = obj.id;
                                for _ in 0..ingest_count {
                                    triggers.push(PendingTrigger {
                                        triggering_event: Some(
                                            TriggerEvent::SelfDealsCombatDamageToPlayer,
                                        ),
                                        data: Some(TriggerData::IngestExile {
                                            target_player: damaged_player,
                                        }),
                                        ..PendingTrigger::blank(
                                            source_id,
                                            controller,
                                            PendingTriggerKind::Ingest,
                                        )
                                    });
                                }
                            }
                        }
                        // CR 702.112a: Renown N -- "When this creature deals combat
                        // damage to a player, if it isn't renowned, put N +1/+1
                        // counters on it and it becomes renowned."
                        // CR 702.112c: Multiple instances trigger separately.
                        // CR 603.4: Intervening-if -- checked here at trigger time
                        // (is_renowned must be false) and again at resolution time.
                        // CR 113.7a: the damage source may have left the battlefield; use LKI.
                        if let Some(obj) = state.fizzle_object(assignment.source) {
                            if obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && !obj
                                    .designations
                                    .contains(crate::state::game_object::Designations::RENOWNED)
                            // CR 603.4: intervening-if at trigger time
                            {
                                // Collect Renown N values from card definition.
                                // CR 702.112c: Each keyword instance triggers separately.
                                let renown_values: Vec<u32> = obj
                                    .card_id
                                    .as_ref()
                                    .and_then(|cid| state.card_registry.get(cid.clone()))
                                    .map(|def| {
                                        def.abilities
                                            .iter()
                                            .filter_map(|a| match a {
                                                AbilityDefinition::Keyword(
                                                    KeywordAbility::Renown(n),
                                                ) => Some(*n),
                                                _ => None,
                                            })
                                            .collect()
                                    })
                                    .unwrap_or_else(|| {
                                        // Fallback: check keywords on the object itself
                                        obj.characteristics
                                            .keywords
                                            .iter()
                                            .filter_map(|kw| match kw {
                                                KeywordAbility::Renown(n) => Some(*n),
                                                _ => None,
                                            })
                                            .collect()
                                    });
                                let controller = obj.controller;
                                let source_id = obj.id;
                                for n in renown_values {
                                    triggers.push(PendingTrigger {
                                        triggering_event: Some(
                                            TriggerEvent::SelfDealsCombatDamageToPlayer,
                                        ),
                                        data: Some(TriggerData::RenownDamage { n }),
                                        ..PendingTrigger::blank(
                                            source_id,
                                            controller,
                                            PendingTriggerKind::Renown,
                                        )
                                    });
                                }
                            }
                        }
                        // CR 702.70a: Poisonous N -- "Whenever this creature deals combat
                        // damage to a player, that player gets N poison counters."
                        // CR 702.70b: Multiple instances trigger separately.
                        // CR 113.7a: the damage source may have left the battlefield; use LKI.
                        if let Some(obj) = state.fizzle_object(assignment.source) {
                            if obj.zone == ZoneId::Battlefield && obj.is_phased_in() {
                                // Already guaranteed by the outer `if matches!(..., Player(_))`
                                // guard -- use `let...else` for safety.
                                let CombatDamageTarget::Player(damaged_player) = &assignment.target
                                else {
                                    continue;
                                };
                                let damaged_player = *damaged_player;
                                // Collect Poisonous N values from card definition.
                                // CR 702.70b: Each keyword instance triggers separately.
                                let poisonous_values: Vec<u32> = obj
                                    .card_id
                                    .as_ref()
                                    .and_then(|cid| state.card_registry.get(cid.clone()))
                                    .map(|def| {
                                        def.abilities
                                            .iter()
                                            .filter_map(|a| match a {
                                                AbilityDefinition::Keyword(
                                                    KeywordAbility::Poisonous(n),
                                                ) => Some(*n),
                                                _ => None,
                                            })
                                            .collect()
                                    })
                                    .unwrap_or_else(|| {
                                        // Fallback: check keywords on the object itself
                                        obj.characteristics
                                            .keywords
                                            .iter()
                                            .filter_map(|kw| match kw {
                                                KeywordAbility::Poisonous(n) => Some(*n),
                                                _ => None,
                                            })
                                            .collect()
                                    });
                                let controller = obj.controller;
                                let source_id = obj.id;
                                for n in poisonous_values {
                                    triggers.push(PendingTrigger {
                                        triggering_event: Some(
                                            TriggerEvent::SelfDealsCombatDamageToPlayer,
                                        ),
                                        data: Some(TriggerData::CombatPoisonous {
                                            target_player: damaged_player,
                                            n,
                                        }),
                                        ..PendingTrigger::blank(
                                            source_id,
                                            controller,
                                            PendingTriggerKind::Poisonous,
                                        )
                                    });
                                }
                            }
                        }
                        // CR 702.99b: Cipher -- "Whenever this creature deals combat damage to a
                        // player, you may copy the encoded card and cast the copy without paying
                        // its mana cost."  One trigger per encoded card per damaged player.
                        // CR 702.99c: If the encoded card left exile, the trigger still goes on
                        // the stack but does nothing at resolution (checked in resolution.rs).
                        if assignment.amount > 0 {
                            // CR 113.7a: the damage source may have left the battlefield; use LKI.
                            if let Some(obj) = state.fizzle_object(assignment.source) {
                                if obj.zone == ZoneId::Battlefield && obj.is_phased_in() {
                                    let CombatDamageTarget::Player(_damaged_player) =
                                        &assignment.target
                                    else {
                                        // already guarded by outer matches! check
                                        continue;
                                    };
                                    if !obj.encoded_cards.is_empty() {
                                        let controller = obj.controller;
                                        let source_id = obj.id;
                                        let encoded = obj.encoded_cards.clone();
                                        for (exiled_obj_id, card_id) in encoded {
                                            triggers.push(PendingTrigger {
                                                triggering_event: Some(
                                                    TriggerEvent::SelfDealsCombatDamageToPlayer,
                                                ),
                                                data: Some(TriggerData::CipherDamage {
                                                    source_creature: source_id,
                                                    encoded_card_id: card_id,
                                                    encoded_object_id: exiled_obj_id,
                                                }),
                                                ..PendingTrigger::blank(
                                                    source_id,
                                                    controller,
                                                    PendingTriggerKind::CipherCombatDamage,
                                                )
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // CR 207.2c / CR 120.3: Enrage -- "Whenever this creature is dealt damage."
                // Collect unique creature ObjectIds that received > 0 combat damage in this
                // simultaneous damage step. Per ruling 2018-01-19, multiple simultaneous
                // sources trigger Enrage only once per creature per damage event.
                // CR 603.2g: amount == 0 (fully prevented) does not trigger.
                let mut damaged_creatures: Vec<ObjectId> = Vec::new();
                for assignment in assignments {
                    if assignment.amount == 0 {
                        continue;
                    }
                    if let CombatDamageTarget::Creature(creature_id) = &assignment.target {
                        if !damaged_creatures.contains(creature_id) {
                            damaged_creatures.push(*creature_id);
                        }
                    }
                }
                for creature_id in damaged_creatures {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::SelfIsDealtDamage,
                        Some(creature_id),
                        None,
                    );
                }
                // CR 510.3a / CR 603.2: AnyCreatureYouControlDealsCombatDamageToPlayer —
                // fires on ALL battlefield permanents when any creature controlled by their
                // controller deals combat damage to a player. Controller filtering is applied
                // inside collect_triggers_for_event via the entering_object check.
                //
                // Fires once per creature per damage event (CR 603.2c).
                // Also populates damaged_player and combat_damage_amount on the new triggers.
                for assignment in assignments {
                    if assignment.amount == 0 {
                        continue; // CR 603.2g: fully prevented damage does not trigger
                    }
                    let CombatDamageTarget::Player(damaged_pid) = &assignment.target else {
                        continue; // only triggers on damage to players (not creatures)
                    };
                    // Only trigger if the source creature is still on the battlefield
                    // (CR 603.10: NOT a look-back trigger for combat damage triggers).
                    if state
                        .objects
                        .get(&assignment.source)
                        .is_none_or(|o| o.zone != ZoneId::Battlefield)
                    {
                        continue;
                    }
                    let pre_len = triggers.len();
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer,
                        None,
                        Some(assignment.source),
                    );
                    // Populate damaged_player and combat_damage_amount on newly-added triggers.
                    for t in &mut triggers[pre_len..] {
                        t.damaged_player = Some(*damaged_pid);
                        t.combat_damage_amount = assignment.amount;
                    }
                }
                // CR 510.3a / CR 603.2c: AnyCreatureYouControlBatchCombatDamage —
                // fires ONCE per (controller, damaged_player) pair per combat damage step.
                // "Whenever one or more creatures you control deal combat damage to a player."
                {
                    // `BTreeMap`, not `HashMap` (PB-DP9 fix-cycle Finding 4's
                    // widened audit): the loop below pushes into `triggers` in
                    // MAP ITERATION ORDER, so with a `HashMap` the relative
                    // order of two batch triggers from different
                    // (controller, damaged player) pairs varied run to run --
                    // and that is CR 603.3b stack order, not cosmetics.
                    use std::collections::BTreeMap;
                    let mut damaged_by_ctrl: BTreeMap<
                        (crate::state::PlayerId, crate::state::PlayerId),
                        u32,
                    > = BTreeMap::new();
                    for assignment in assignments {
                        if assignment.amount == 0 {
                            continue;
                        }
                        let CombatDamageTarget::Player(damaged_pid) = &assignment.target else {
                            continue;
                        };
                        // CR 113.7a: the damage source may have left the battlefield; use LKI.
                        if let Some(obj) = state.fizzle_object(assignment.source) {
                            if obj.zone == ZoneId::Battlefield && obj.is_phased_in() {
                                *damaged_by_ctrl
                                    .entry((obj.controller, *damaged_pid))
                                    .or_default() += assignment.amount;
                            }
                        }
                    }
                    for ((controller, damaged_pid), total_amount) in &damaged_by_ctrl {
                        let pre_len = triggers.len();
                        // Collect batch triggers for all battlefield permanents.
                        // We then retain only those controlled by the triggering controller.
                        // Use a dummy entering_object=None — batch triggers don't carry a single creature.
                        let all_bf: Vec<ObjectId> = state
                            .objects
                            .values()
                            .filter(|o| o.zone == ZoneId::Battlefield && o.is_phased_in())
                            .map(|o| o.id)
                            .collect();
                        for obj_id in all_bf {
                            let obj = match state.expect_object(obj_id) {
                                Some(o) if o.controller == *controller => o,
                                _ => continue,
                            };
                            let resolved_chars =
                                crate::rules::layers::expect_characteristics(state, obj_id);
                            for (idx, trigger_def) in
                                resolved_chars.triggered_abilities.iter().enumerate()
                            {
                                if trigger_def.trigger_on
                                    != TriggerEvent::AnyCreatureYouControlBatchCombatDamage
                                {
                                    continue;
                                }
                                // Apply intervening-if condition. Not a look-back trigger
                                // — the source (`obj_id`) is filtered to the battlefield above.
                                if let Some(ref cond) = trigger_def.intervening_if {
                                    if !check_intervening_if(
                                        state,
                                        cond,
                                        obj.controller,
                                        obj_id,
                                        None,
                                        InterveningIfMoment::TriggerTime,
                                        &[],
                                    ) {
                                        continue;
                                    }
                                }
                                // CR 603.2: combat_damage_filter — check if at least one
                                // creature controlled by `controller` that dealt damage to
                                // `damaged_pid` matches the filter (e.g., Ninja/Rogue for
                                // Prosperous Thief, Faerie for Alela).
                                if let Some(ref filter) = trigger_def.combat_damage_filter {
                                    let any_matches = assignments.iter().any(|a| {
                                        if a.amount == 0 {
                                            return false;
                                        }
                                        let CombatDamageTarget::Player(pid) = &a.target else {
                                            return false;
                                        };
                                        if pid != damaged_pid {
                                            return false;
                                        }
                                        let Some(dealing_obj) = state.objects.get(&a.source) else {
                                            return false;
                                        };
                                        if dealing_obj.controller != *controller {
                                            return false;
                                        }
                                        if dealing_obj.zone != ZoneId::Battlefield
                                            || !dealing_obj.is_phased_in()
                                        {
                                            return false;
                                        }
                                        if filter.is_token && !dealing_obj.is_token {
                                            return false;
                                        }
                                        let dealing_chars =
                                            crate::rules::layers::expect_characteristics(
                                                state, a.source,
                                            );
                                        crate::effects::matches_filter(&dealing_chars, filter)
                                    });
                                    if !any_matches {
                                        continue;
                                    }
                                }
                                triggers.push(PendingTrigger {
                                    ability_index: idx,
                                    triggering_event: Some(
                                        TriggerEvent::AnyCreatureYouControlBatchCombatDamage,
                                    ),
                                    damaged_player: Some(*damaged_pid),
                                    combat_damage_amount: *total_amount,
                                    ..PendingTrigger::blank(
                                        obj_id,
                                        obj.controller,
                                        PendingTriggerKind::Normal,
                                    )
                                });
                            }
                        }
                        let _ = pre_len; // used for debugging if needed
                    }
                }
                // CR 510.3a / CR 603.2 / CR 603.2c (PB-DX36 `/review` HIGH 1,
                // `OOS-CARDS2-6`): SelfDealsDamage family +
                // EquippedCreatureDealsCombatDamageToPlayer + the enchanted-creature
                // combat/any family — fires on the damage source itself and on its
                // Equipment/Aura attachments. CR 510.2 makes every assignment in one
                // `CombatDamageDealt` event SIMULTANEOUS, and CR 603.2c makes an
                // ability trigger only ONCE per event — so a source with more than one
                // assignment (multi-block, trample) must be GROUPED before dispatch,
                // never dispatched once per assignment (that shape fired the SELF
                // family multiple times for one event, once per assignment, each
                // carrying only that assignment's own amount rather than the event
                // total — verified live on `exalted_angel`, `pb_dx36_damage_trigger_
                // dispatch.rs` t8/t9). Grouped by SOURCE, preserving the ORDER of
                // first appearance in `assignments` (never sorted by `ObjectId`,
                // which would reorder triggers relative to a real game). Extracted
                // into `queue_damage_source_triggers` (`is_combat: true`) so the
                // identical arithmetic serves the noncombat `GameEvent::DamageDealt`
                // arm below without duplication.
                let mut grouped_by_source: Vec<(ObjectId, Vec<(CombatDamageTarget, u32)>)> =
                    Vec::new();
                for assignment in assignments {
                    match grouped_by_source
                        .iter_mut()
                        .find(|(src, _)| *src == assignment.source)
                    {
                        Some((_, targets)) => {
                            targets.push((assignment.target.clone(), assignment.amount));
                        }
                        None => {
                            grouped_by_source.push((
                                assignment.source,
                                vec![(assignment.target.clone(), assignment.amount)],
                            ));
                        }
                    }
                }
                for (source, targets) in &grouped_by_source {
                    queue_damage_source_triggers(state, &mut triggers, *source, targets, true);
                }
                // CR 510.3a / 603.2c: EquippedCreatureDealsCombatDamage (any recipient).
                // Fires once per equipped SOURCE creature per combat-damage step, regardless of
                // how many recipients it damaged (trample/multi-block = one dealing event; double
                // strike = two steps = two invocations of this collector).
                let mut damaged_sources: Vec<ObjectId> = Vec::new();
                for assignment in assignments {
                    if assignment.amount == 0 {
                        continue; // CR 603.2g
                    }
                    if !damaged_sources.contains(&assignment.source) {
                        damaged_sources.push(assignment.source);
                    }
                }
                for source_creature in damaged_sources {
                    // CR 603.10: combat-damage triggers do NOT look back — source must still be
                    // on the battlefield. A quiet `None`/wrong-zone here is a legitimate CR
                    // 608.2b-style fizzle (the source could conceivably have left between
                    // damage assignment and this collector running), not an engine bug.
                    let Some(creature_obj) = state.fizzle_object(source_creature) else {
                        continue;
                    };
                    if creature_obj.zone != ZoneId::Battlefield {
                        continue;
                    }
                    let attachments: Vec<ObjectId> =
                        creature_obj.attachments.iter().copied().collect();
                    // total damage this source dealt this step (for cards that read the amount;
                    // Jitte ignores it but populate for parity with the ...ToPlayer path).
                    let total: u32 = assignments
                        .iter()
                        .filter(|a| a.source == source_creature)
                        .map(|a| a.amount)
                        .sum();
                    for attachment_id in attachments {
                        let pre_len = triggers.len();
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::EquippedCreatureDealsCombatDamage,
                            Some(attachment_id),
                            None,
                        );
                        for t in &mut triggers[pre_len..] {
                            t.entering_object_id = Some(source_creature);
                            t.combat_damage_amount = total;
                            // damaged_player intentionally left None — recipient may be a creature/pw.
                        }
                    }
                }
                // CR 510.3a / CR 603.2: AnyCreatureDealsCombatDamageToOpponent —
                // "Whenever a creature deals combat damage to one of your opponents."
                // Fires globally for any creature dealing damage to an opponent of
                // the trigger source's controller (Edric, Spymaster of Trest).
                for assignment in assignments {
                    if assignment.amount == 0 {
                        continue; // CR 603.2g
                    }
                    let CombatDamageTarget::Player(damaged_pid) = &assignment.target else {
                        continue;
                    };
                    if state
                        .objects
                        .get(&assignment.source)
                        .is_none_or(|o| o.zone != ZoneId::Battlefield)
                    {
                        continue;
                    }
                    let pre_len = triggers.len();
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::AnyCreatureDealsCombatDamageToOpponent,
                        None,
                        Some(assignment.source),
                    );
                    // Filter: damaged player must be an OPPONENT of the trigger source's controller.
                    // Then populate combat data.
                    // Use drain/retain equivalent: collect new triggers, filter, set data.
                    let new_triggers: Vec<PendingTrigger> = triggers
                        .drain(pre_len..)
                        .filter(|t| t.controller != *damaged_pid)
                        .map(|mut t| {
                            t.damaged_player = Some(*damaged_pid);
                            t.combat_damage_amount = assignment.amount;
                            t.entering_object_id = Some(assignment.source);
                            t
                        })
                        .collect();
                    triggers.extend(new_triggers);
                }
                // CR 701.54c (ring level >= 4): "Whenever your Ring-bearer deals combat
                // damage to a player, each opponent loses 3 life."
                // Queue a RingCombatDamage PendingTrigger for each assignment where the
                // source is a ring-bearer with ring_level >= 4.
                for assignment in assignments {
                    if assignment.amount == 0 {
                        continue; // no trigger on fully-prevented damage (CR 603.2g)
                    }
                    if !matches!(assignment.target, CombatDamageTarget::Player(_)) {
                        continue; // only triggers on damage to players
                    }
                    let (is_ring_bearer, ring_level, ring_controller) = {
                        let obj = state.objects.get(&assignment.source);
                        match obj {
                            Some(o) if o.zone == ZoneId::Battlefield => {
                                let bearer = o
                                    .designations
                                    .contains(crate::state::game_object::Designations::RING_BEARER);
                                let ctrl = o.controller;
                                let lvl = state
                                    .expect_player(ctrl)
                                    .map(|ps| ps.ring_level)
                                    .unwrap_or(0);
                                (bearer, lvl, ctrl)
                            }
                            _ => (false, 0, crate::state::player::PlayerId(u64::MAX)),
                        }
                    };
                    if is_ring_bearer && ring_level >= 4 {
                        triggers.push(PendingTrigger::blank(
                            assignment.source,
                            ring_controller,
                            PendingTriggerKind::RingCombatDamage,
                        ));
                    }
                }
            }
            GameEvent::Proliferated { controller, .. } => {
                // CR 701.34: "Whenever you proliferate" triggers on all permanents
                // controlled by the proliferating player.
                let controller_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller == *controller
                    })
                    .map(|obj| obj.id)
                    .collect();
                for obj_id in controller_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::ControllerProliferates,
                        Some(obj_id),
                        None,
                    );
                }
            }
            // CR 702.72a: Champion LTB trigger -- when the champion permanent is destroyed
            // (non-creature), check champion_exiled_card on the graveyard object.
            GameEvent::PermanentDestroyed {
                new_grave_id,
                pre_lba_counters: destroyed_lki_counters,
                pre_lba_power: destroyed_lki_power,
                ..
            } => {
                // CR 603.10a: LKI read of the graveyard object; it may already be gone.
                if let Some(dead_obj) = state.fizzle_object(*new_grave_id) {
                    if let Some(exiled_id) = dead_obj.champion_exiled_card {
                        let champion_controller = dead_obj.controller;
                        triggers.push(PendingTrigger {
                            data: Some(TriggerData::LTBChampion {
                                exiled_card: exiled_id,
                            }),
                            ..PendingTrigger::blank(
                                *new_grave_id,
                                champion_controller,
                                PendingTriggerKind::ChampionLTB,
                            )
                        });
                    }
                }
                // CR 603.10a: SelfLeavesBattlefield LTB trigger (look-back via graveyard object).
                // CR 603.10a: LKI read of the graveyard object; it may already be gone.
                if let Some(dead_obj) = state.fizzle_object(*new_grave_id) {
                    let controller = dead_obj.controller;
                    for (idx, trigger_def) in dead_obj
                        .characteristics
                        .triggered_abilities
                        .iter()
                        .enumerate()
                    {
                        if trigger_def.trigger_on != TriggerEvent::SelfLeavesBattlefield {
                            continue;
                        }
                        if let Some(ref cond) = trigger_def.intervening_if {
                            if !check_intervening_if(
                                state,
                                cond,
                                controller,
                                *new_grave_id,
                                None,
                                InterveningIfMoment::TriggerTimeLookBack,
                                &[],
                            ) {
                                continue;
                            }
                        }
                        triggers.push(PendingTrigger {
                            ability_index: idx,
                            triggering_event: Some(TriggerEvent::SelfLeavesBattlefield),
                            // CR 603.10a: LKI snapshot from event — counters before zone change.
                            lki_counters: destroyed_lki_counters.clone(),
                            // CR 603.10a: LKI source-power snapshot.
                            lki_power: *destroyed_lki_power,
                            ..PendingTrigger::blank(
                                *new_grave_id,
                                controller,
                                PendingTriggerKind::Normal,
                            )
                        });
                    }
                }
            }
            // CR 702.72a: Champion LTB trigger -- when the champion permanent is exiled,
            // check champion_exiled_card on the exile-zone object.
            GameEvent::ObjectExiled {
                new_exile_id,
                pre_lba_counters: exiled_lki_counters,
                pre_lba_power: exiled_lki_power,
                ..
            } => {
                // CR 603.10a: LKI read of the exiled object; it may already be gone.
                if let Some(exiled_obj) = state.fizzle_object(*new_exile_id) {
                    if let Some(exiled_card_id) = exiled_obj.champion_exiled_card {
                        let champion_controller = exiled_obj.controller;
                        triggers.push(PendingTrigger {
                            data: Some(TriggerData::LTBChampion {
                                exiled_card: exiled_card_id,
                            }),
                            ..PendingTrigger::blank(
                                *new_exile_id,
                                champion_controller,
                                PendingTriggerKind::ChampionLTB,
                            )
                        });
                    }
                }
                // CR 603.10a: SelfLeavesBattlefield LTB trigger on exile (look-back via exile object).
                // CR 603.10a: LKI read of the exiled object; it may already be gone.
                if let Some(exiled_obj) = state.fizzle_object(*new_exile_id) {
                    let controller = exiled_obj.controller;
                    for (idx, trigger_def) in exiled_obj
                        .characteristics
                        .triggered_abilities
                        .iter()
                        .enumerate()
                    {
                        if trigger_def.trigger_on != TriggerEvent::SelfLeavesBattlefield {
                            continue;
                        }
                        if let Some(ref cond) = trigger_def.intervening_if {
                            if !check_intervening_if(
                                state,
                                cond,
                                controller,
                                *new_exile_id,
                                None,
                                InterveningIfMoment::TriggerTimeLookBack,
                                &[],
                            ) {
                                continue;
                            }
                        }
                        triggers.push(PendingTrigger {
                            ability_index: idx,
                            triggering_event: Some(TriggerEvent::SelfLeavesBattlefield),
                            // CR 603.10a: LKI snapshot from event — counters before zone change.
                            lki_counters: exiled_lki_counters.clone(),
                            // CR 603.10a: LKI source-power snapshot.
                            lki_power: *exiled_lki_power,
                            ..PendingTrigger::blank(
                                *new_exile_id,
                                controller,
                                PendingTriggerKind::Normal,
                            )
                        });
                    }
                }
            }
            // CR 702.72a: Champion LTB trigger -- when the champion permanent bounces to hand,
            // check champion_exiled_card on the hand object.
            GameEvent::ObjectReturnedToHand {
                new_hand_id,
                pre_lba_counters: bounced_lki_counters,
                pre_lba_power: bounced_lki_power,
                ..
            } => {
                // CR 603.10a: LKI read of the hand object; it may already be gone.
                if let Some(hand_obj) = state.fizzle_object(*new_hand_id) {
                    if let Some(exiled_id) = hand_obj.champion_exiled_card {
                        let champion_controller = hand_obj.controller;
                        triggers.push(PendingTrigger {
                            data: Some(TriggerData::LTBChampion {
                                exiled_card: exiled_id,
                            }),
                            ..PendingTrigger::blank(
                                *new_hand_id,
                                champion_controller,
                                PendingTriggerKind::ChampionLTB,
                            )
                        });
                    }
                }
                // CR 603.10a: SelfLeavesBattlefield LTB trigger on bounce (look-back via hand object).
                // CR 603.10a: LKI read of the hand object; it may already be gone.
                if let Some(hand_obj) = state.fizzle_object(*new_hand_id) {
                    let controller = hand_obj.controller;
                    for (idx, trigger_def) in hand_obj
                        .characteristics
                        .triggered_abilities
                        .iter()
                        .enumerate()
                    {
                        if trigger_def.trigger_on != TriggerEvent::SelfLeavesBattlefield {
                            continue;
                        }
                        if let Some(ref cond) = trigger_def.intervening_if {
                            if !check_intervening_if(
                                state,
                                cond,
                                controller,
                                *new_hand_id,
                                None,
                                InterveningIfMoment::TriggerTimeLookBack,
                                &[],
                            ) {
                                continue;
                            }
                        }
                        triggers.push(PendingTrigger {
                            ability_index: idx,
                            triggering_event: Some(TriggerEvent::SelfLeavesBattlefield),
                            // CR 603.10a: LKI snapshot from event — counters before zone change.
                            lki_counters: bounced_lki_counters.clone(),
                            // CR 603.10a: LKI source-power snapshot.
                            lki_power: *bounced_lki_power,
                            ..PendingTrigger::blank(
                                *new_hand_id,
                                controller,
                                PendingTriggerKind::Normal,
                            )
                        });
                    }
                }
            }
            // CR 207.2c / CR 120.3: Enrage -- "Whenever this creature is dealt damage."
            // Non-combat damage to a creature fires SelfIsDealtDamage on that creature.
            // CR 603.2g: amount == 0 (fully prevented) does not trigger.
            #[allow(clippy::collapsible_match)]
            GameEvent::DamageDealt {
                source,
                target,
                amount,
            } => {
                if *amount > 0 {
                    if let CombatDamageTarget::Creature(creature_id) = target {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::SelfIsDealtDamage,
                            Some(*creature_id),
                            None,
                        );
                    }
                }
                // CR 510.3a / CR 603.2 (PB-DX36, `OOS-CARDS2-6`): the same
                // SelfDealsDamage/attachment arithmetic the combat-damage arm runs,
                // called here with `is_combat: false` so a NONcombat damage event
                // fires only the "any damage" family (never the "combat damage"
                // family) — see `queue_damage_source_triggers`'s doc for why the
                // two arms are disjoint by construction. A single `(target,
                // amount)` pair, since one `DamageDealt` event is one assignment
                // (never grouped) — the multi-assignment grouping only applies to
                // `CombatDamageDealt`.
                queue_damage_source_triggers(
                    state,
                    &mut triggers,
                    *source,
                    &[(target.clone(), *amount)],
                    false,
                );
            }
            // CR 702.140d: "Whenever this creature mutates" — fires on the merged permanent.
            // The merged permanent is the same object (same ObjectId) as the target permanent
            // before merging. After the merge, it has ALL abilities from ALL components
            // (via the layer system). We fire SelfMutates on the merged permanent itself.
            //
            // CR 729.2c: The merged permanent is NOT new — it did not enter the battlefield.
            // No ETB triggers fire. Only SelfMutates triggers fire.
            GameEvent::CreatureMutated { object_id, .. } => {
                // collect_triggers_for_event checks zone == Battlefield, which is correct:
                // the merged permanent must still be on the battlefield to fire this trigger.
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::SelfMutates,
                    Some(*object_id),
                    None,
                );
            }
            // CR 708.8 / CR 702.37e: "When this permanent is turned face up" triggers.
            // Fire the TurnFaceUp pending trigger for any WhenTurnedFaceUp ability in the
            // permanent's CardDefinition. The permanent is now face-up; look up its card_id
            // to find the definition. ETB abilities do NOT fire (CR 708.8).
            GameEvent::PermanentTurnedFaceUp {
                player: _,
                permanent,
            } => {
                use crate::cards::card_definition::{AbilityDefinition, TriggerCondition};
                // The permanent is now face-up — its card_id is accessible.
                let card_id = state.objects.get(permanent).and_then(|o| o.card_id.clone());
                let controller_opt = state.objects.get(permanent).map(|o| o.controller);
                if let (Some(cid), Some(ctrl)) = (card_id, controller_opt) {
                    let def_opt = state.card_registry.get(cid);
                    if let Some(def) = def_opt {
                        for (idx, ability) in def.abilities.iter().enumerate() {
                            if let AbilityDefinition::Triggered {
                                trigger_condition: TriggerCondition::WhenTurnedFaceUp,
                                intervening_if,
                                ..
                            } = ability
                            {
                                // CR 603.4 (PB-DP6): queue-time gate. The permanent
                                // was just turned face up (CR 708.8) — full
                                // characteristics are already available.
                                if !carddef_intervening_if_holds_at_queue_time(
                                    state,
                                    intervening_if.as_ref(),
                                    ctrl,
                                    *permanent,
                                ) {
                                    continue;
                                }
                                triggers.push(PendingTrigger {
                                    ability_index: idx,
                                    ..PendingTrigger::blank(
                                        *permanent,
                                        ctrl,
                                        crate::state::stubs::PendingTriggerKind::TurnFaceUp,
                                    )
                                });
                            }
                        }
                    }
                }
            }
            // CR 701.54d: "Whenever the Ring tempts you" — fire triggers on permanents
            // controlled by the tempted player that have WheneverRingTemptsYou trigger condition.
            GameEvent::RingTempted {
                player: tempted_player,
                ..
            } => {
                use crate::cards::card_definition::{AbilityDefinition, TriggerCondition};
                // Collect all permanents controlled by the tempted player.
                let obj_ids: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.controller == *tempted_player
                            && obj.is_phased_in()
                    })
                    .map(|obj| obj.id)
                    .collect();
                for obj_id in obj_ids {
                    let Some(obj) = state.expect_object(obj_id) else {
                        continue;
                    };
                    let card_id = obj.card_id.clone();
                    let is_transformed = obj.is_transformed;
                    let Some(cid) = card_id else { continue };
                    let Some(def) = state.card_registry.get(cid) else {
                        continue;
                    };
                    // OOS-DX1-4 Q6 (PB-DX24): `obj` is already in hand; read the same
                    // face the resolution side (`resolution.rs`) reads.
                    for (idx, ability) in def.effective_abilities(is_transformed).iter().enumerate()
                    {
                        if let AbilityDefinition::Triggered {
                            trigger_condition: TriggerCondition::WheneverRingTemptsYou,
                            intervening_if,
                            ..
                        } = ability
                        {
                            // CR 603.4 (PB-DP6): queue-time gate. `*tempted_player`
                            // is the controller passed here, which the guard above
                            // has already equated to `obj.controller`.
                            if !carddef_intervening_if_holds_at_queue_time(
                                state,
                                intervening_if.as_ref(),
                                *tempted_player,
                                obj_id,
                            ) {
                                continue;
                            }
                            // PB-EF3 A2 (CR 601.2c/603.3d): `idx` is a raw index into
                            // `def.abilities` (not converted to runtime
                            // `characteristics.triggered_abilities`). CardDefETB kind keeps
                            // the raw-index/card-registry lookup authoritative for both
                            // effect and target selection.
                            triggers.push(PendingTrigger {
                                ability_index: idx,
                                ..PendingTrigger::blank(
                                    obj_id,
                                    *tempted_player,
                                    crate::state::stubs::PendingTriggerKind::CardDefETB,
                                )
                            });
                        }
                    }
                }
            }
            // CR 603.2: "Whenever you draw a card" / "Whenever a player draws a card"
            // dispatch. Fires ControllerDrawsCard, OpponentDrawsCard, AnyPlayerDrawsCard.
            GameEvent::CardDrawn { player, .. } => {
                let pre_len = triggers.len();
                // ControllerDrawsCard: fire on permanents controlled by the drawing player.
                let controller_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller == *player
                    })
                    .map(|obj| obj.id)
                    .collect();
                for obj_id in controller_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::ControllerDrawsCard,
                        Some(obj_id),
                        None,
                    );
                }
                // OpponentDrawsCard: fire on permanents controlled by opponents.
                let opponent_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller != *player
                    })
                    .map(|obj| obj.id)
                    .collect();
                for obj_id in opponent_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::OpponentDrawsCard,
                        Some(obj_id),
                        None,
                    );
                }
                // AnyPlayerDrawsCard: fire on all permanents.
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::AnyPlayerDrawsCard,
                    None,
                    None,
                );
                // Tag draw triggers with the drawing player so PlayerTarget::TriggeringPlayer
                // resolves correctly (e.g. Scrawling Crawler, Razorkin Needlehead).
                for t in &mut triggers[pre_len..] {
                    t.triggering_player = Some(*player);
                }
            }
            // CR 603.2 / CR 118.4: "Whenever you gain life" dispatch.
            // Fires ControllerGainsLife on permanents controlled by the gaining player.
            GameEvent::LifeGained { player, .. } => {
                let controller_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller == *player
                    })
                    .map(|obj| obj.id)
                    .collect();
                for obj_id in controller_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::ControllerGainsLife,
                        Some(obj_id),
                        None,
                    );
                }
            }
            // CR 701.9a: Discard trigger dispatch.
            // Fires ControllerDiscards on controller's permanents and OpponentDiscards on opponents'.
            GameEvent::CardDiscarded { player, .. } => {
                // ControllerDiscards: fire on permanents controlled by the discarding player.
                let controller_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller == *player
                    })
                    .map(|obj| obj.id)
                    .collect();
                for obj_id in controller_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::ControllerDiscards,
                        Some(obj_id),
                        None,
                    );
                }
                // OpponentDiscards: fire on permanents controlled by opponents.
                let opponent_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller != *player
                    })
                    .map(|obj| obj.id)
                    .collect();
                let pre_len = triggers.len();
                for obj_id in opponent_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::OpponentDiscards,
                        Some(obj_id),
                        None,
                    );
                }
                // Tag with triggering player so effects can reference "that player".
                for t in &mut triggers[pre_len..] {
                    t.triggering_player = Some(*player);
                }
            }
            // CR 701.21a: Sacrifice trigger dispatch.
            // Fires ControllerSacrifices on permanents controlled by the sacrificing player.
            GameEvent::PermanentSacrificed { player, new_id, .. } => {
                // ControllerSacrifices: fire on permanents controlled by the sacrificing player.
                let controller_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller == *player
                    })
                    .map(|obj| obj.id)
                    .collect();
                let pre_len = triggers.len();
                for obj_id in controller_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::ControllerSacrifices,
                        Some(obj_id),
                        None,
                    );
                }
                // Also fire on ALL battlefield permanents for "any player sacrifices" pattern.
                // This handles WheneverYouSacrifice { player_filter: Some(TargetController::Any) }.
                let all_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller != *player
                    })
                    .map(|obj| obj.id)
                    .collect();
                for obj_id in all_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::ControllerSacrifices,
                        Some(obj_id),
                        None,
                    );
                }
                // Tag all sacrifice triggers with triggering player.
                for t in &mut triggers[pre_len..] {
                    t.triggering_player = Some(*player);
                }
                // Post-filter: check WheneverYouSacrifice.filter and player_filter against
                // the sacrificed object (looked up from its new zone via new_id).
                {
                    let sacrificed_card_id =
                        state.objects.get(new_id).and_then(|o| o.card_id.clone());
                    let sacrificed_types: Vec<crate::state::types::CardType> = state
                        .objects
                        .get(new_id)
                        .map(|o| o.characteristics.card_types.iter().cloned().collect())
                        .unwrap_or_default();
                    let sacrificed_subtypes: imbl::OrdSet<crate::state::types::SubType> = state
                        .objects
                        .get(new_id)
                        .map(|o| o.characteristics.subtypes.clone())
                        .unwrap_or_default();
                    let sacrificed_is_token = state.objects.get(new_id).is_some_and(|o| o.is_token);
                    let _ = sacrificed_card_id;
                    let _ = sacrificed_is_token;
                    triggers.retain(|t| {
                        // Only post-filter ControllerSacrifices triggers.
                        if t.triggering_event.as_ref() != Some(&TriggerEvent::ControllerSacrifices)
                            && t.triggering_player != Some(*player)
                        {
                            return true;
                        }
                        // Look up the trigger source's CardDef triggered ability.
                        // PB-OS4b (CR 712.8d/e): index into the currently-visible
                        // face's effective list -- "is_transformed at consume time"
                        // contract (see plan Index-Stability discussion). Single
                        // lookup captures both card_id and is_transformed so this
                        // doesn't add a second bare `.objects.get` (SR-25 ratchet).
                        let source_info = state
                            .objects
                            .get(&t.source)
                            .map(|o| (o.is_transformed, o.card_id.clone()));
                        let Some((source_is_transformed, source_card_id)) = source_info else {
                            return true;
                        };
                        let def = source_card_id.and_then(|cid| state.card_registry.get(cid));
                        let Some(def) = def else { return true };
                        let ability = def
                            .effective_abilities(source_is_transformed)
                            .get(t.ability_index);
                        let Some(ability) = ability else { return true };
                        match ability {
                            AbilityDefinition::Triggered {
                                trigger_condition:
                                    TriggerCondition::WheneverYouSacrifice {
                                        filter,
                                        player_filter,
                                    },
                                ..
                            } => {
                                // player_filter check: if Some(You), only fire for controller.
                                // If Some(Any), fire for any player (no filter).
                                let trigger_source_controller = state
                                    .objects
                                    .get(&t.source)
                                    .map(|o| o.controller)
                                    .unwrap_or(*player);
                                if let Some(pf) = player_filter {
                                    match pf {
                                        TargetController::You => {
                                            // Must be the trigger source's controller who sacrificed.
                                            if t.triggering_player
                                                != Some(trigger_source_controller)
                                            {
                                                return false;
                                            }
                                        }
                                        TargetController::Opponent => {
                                            // Must be an opponent of the trigger source's controller.
                                            if t.triggering_player
                                                == Some(trigger_source_controller)
                                            {
                                                return false;
                                            }
                                        }
                                        TargetController::Any => {} // No filter
                                        // PB-D: DamagedPlayer makes no sense on sacrifice triggers —
                                        // there is no combat-damage context here. Reject defensively
                                        // so a card author can't accidentally write this.
                                        TargetController::DamagedPlayer => {
                                            return false;
                                        }
                                    }
                                } else {
                                    // Default: only fire when the controller sacrificed (you only).
                                    if t.triggering_player != Some(trigger_source_controller) {
                                        return false;
                                    }
                                }
                                // filter check: sacrificed object must match type filter.
                                if let Some(ref tf) = filter {
                                    if let Some(required_type) = &tf.has_card_type {
                                        if !sacrificed_types.contains(required_type) {
                                            return false;
                                        }
                                    }
                                    if let Some(required_subtype) = &tf.has_subtype {
                                        if !sacrificed_subtypes.contains(required_subtype) {
                                            return false;
                                        }
                                    }
                                }
                                true
                            }
                            _ => true,
                        }
                    });
                }
                // SelfLeavesBattlefield: fire on the sacrificed object (LKI in graveyard/exile).
                // CR 603.10a: look-back trigger — check graveyard/exile object.
                // CR 603.10a: LKI read of the sacrificed object in its new zone; it may already be gone.
                if let Some(gone_obj) = state.fizzle_object(*new_id) {
                    let controller = gone_obj.controller;
                    for (idx, trigger_def) in gone_obj
                        .characteristics
                        .triggered_abilities
                        .iter()
                        .enumerate()
                    {
                        if trigger_def.trigger_on != TriggerEvent::SelfLeavesBattlefield {
                            continue;
                        }
                        if let Some(ref cond) = trigger_def.intervening_if {
                            if !check_intervening_if(
                                state,
                                cond,
                                controller,
                                *new_id,
                                None,
                                InterveningIfMoment::TriggerTimeLookBack,
                                &[],
                            ) {
                                continue;
                            }
                        }
                        triggers.push(PendingTrigger {
                            ability_index: idx,
                            triggering_event: Some(TriggerEvent::SelfLeavesBattlefield),
                            ..PendingTrigger::blank(*new_id, controller, PendingTriggerKind::Normal)
                        });
                    }
                }
            }
            // CR 305.1: "Whenever an opponent plays a land" trigger dispatch.
            // Fires OpponentPlaysLand on all battlefield permanents controlled by opponents
            // of the player who played the land.
            GameEvent::LandPlayed { player, .. } => {
                let pre_len = triggers.len();
                let opponent_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller != *player
                    })
                    .map(|obj| obj.id)
                    .collect();
                for source_id in opponent_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::OpponentPlaysLand,
                        Some(source_id),
                        None,
                    );
                }
                // Tag with triggering player for PlayerTarget resolution.
                for t in &mut triggers[pre_len..] {
                    t.triggering_player = Some(*player);
                }
            }
            _ => {}
        }
        // PB-DX15a (`OOS-DX24-7`): record this event's own arrival AFTER it is handled,
        // so the next event sees it as "strictly earlier" and subtracts it (CR 603.10a).
        // No-op under `Simultaneous`, which reads the whole-batch set instead.
        if timing == EventBatchTiming::Sequential {
            if let Some(id) = graveyard_arrival_id(event) {
                earlier_arrivals.insert(id);
            }
        }
    }
    // CR 610.3: For delayed triggers with WhenSourceLeavesBattlefield timing,
    // check if the source left the battlefield in this event batch. If so,
    // queue a DelayedAction trigger to return/release the exiled object.
    //
    // We scan all events for any permanent leaving the battlefield (CreatureDied,
    // PermanentSacrificed, or ObjectExiled). If the source of a WhenSourceLeavesBattlefield
    // delayed trigger matches, queue the delayed action.
    {
        use crate::state::stubs::DelayedTriggerTiming;
        // Collect source IDs of permanents that left the battlefield in this event batch.
        let mut left_battlefield: std::collections::BTreeSet<ObjectId> =
            std::collections::BTreeSet::new();
        for event in events {
            match event {
                GameEvent::CreatureDied {
                    object_id: pre_death_id,
                    ..
                }
                | GameEvent::PermanentSacrificed {
                    object_id: pre_death_id,
                    ..
                } => {
                    left_battlefield.insert(*pre_death_id);
                }
                GameEvent::ObjectExiled { object_id, .. } => {
                    left_battlefield.insert(*object_id);
                }
                _ => {}
            }
        }
        if !left_battlefield.is_empty() {
            for dt in state.delayed_triggers.iter() {
                if dt.fired {
                    continue;
                }
                if dt.timing != DelayedTriggerTiming::WhenSourceLeavesBattlefield {
                    continue;
                }
                if !left_battlefield.contains(&dt.source) {
                    continue;
                }
                // The source left the battlefield — queue the return trigger.
                triggers.push(PendingTrigger {
                    data: Some(TriggerData::DelayedAction {
                        action: dt.action.clone(),
                        target: dt.target_object,
                    }),
                    ..PendingTrigger::blank(
                        dt.source,
                        dt.controller,
                        PendingTriggerKind::DelayedAction,
                    )
                });
            }
        }
    }
    triggers
}
/// PB-AC6 / CR 601.2c / 602.2b / 603.2: Collect `WhenBecomesTarget`-derived
/// (`TriggerEvent::PermanentBecomesTarget`) triggers for a single targeting event.
///
/// Scans all battlefield permanents for a `PermanentBecomesTarget { scope,
/// by_opponent, include_abilities }` runtime trigger def and applies the per-card
/// params:
/// - `include_abilities`: `false` restricts to spells only (CR 601.2c); `true` also
///   fires for abilities (CR 602.2b). Determined by looking up the stack object for
///   `targeting_stack_id` and checking `StackObjectKind::Spell` (CR 702.140a: a mutating
///   creature spell counts too -- see `targeting_is_spell` below; PB-DX25 review Finding 1).
///   **LIVE as of PB-DX50 (`scutemob-221`), and this comment used to say the opposite.**
///   It read *"Latent for the mutate case today: the mutate target is never entered into
///   `spell_targets` (`OOS-DX25-1`), so no `PermanentBecomesTarget` event is ever raised
///   for a mutate cast's own target -- this fix only takes effect once that gap closes."*
///   PB-DX50 half 1 **is** that gap closing: `casting::handle_cast_spell` now appends the
///   host to the `StackObject`'s `targets`, `rules::events::permanent_targeted_events`
///   reads that list, and the `GameEvent::PermanentTargeted` arm above dispatches Ward and
///   this function from the same place. The comment outlived the commit that falsified it
///   -- the shape this queue keeps filing (`OOS-DX47-6`, `OOS-DX49-6`), committed by the
///   batch whose own headline is a false comment, and caught by neither the batch nor its
///   `/review` until an unrelated finding was being checked. Pinned behaviourally by
///   `primitives::pb_dx50_mutate_target_legality::test_dx50_t12_whenbecomestarget_fires_for_a_mutate_host`,
///   so the correction is a red test rather than a second sentence.
/// - `by_opponent`: `true` restricts to targeting sources controlled by an opponent
///   of the trigger source's controller (CR 702.21a-style gate).
/// - `scope`: `None` = the trigger source itself must be the target ("Whenever this
///   creature becomes the target..."); `Some(filter)` = the target must be a permanent
///   controlled by the trigger source's controller matching `filter` ("a creature/Dragon
///   you control"). The source itself is NOT excluded: a Dragon that targets itself
///   satisfies "a Dragon you control".
///
/// `targeting_stack_id` is recorded on the pushed `PendingTrigger` so
/// `flush_pending_triggers` can resolve `EffectTarget::DeclaredTarget { index: 0 }`
/// to the targeting spell/ability's controller (Bonecrusher Giant-style effects) --
/// same tagging convention as the Ward block above.
fn collect_permanent_becomes_target_triggers(
    state: &GameState,
    triggers: &mut Vec<PendingTrigger>,
    target_id: ObjectId,
    targeting_stack_id: ObjectId,
    targeting_controller: PlayerId,
) {
    // CR 601.2c vs 602.2b, and CR 702.140a: a mutating creature spell IS a
    // creature spell, so it must count as a spell for this "becomes the
    // target of a spell" gate -- mirrors casting.rs:6504-6509's identical
    // `is_spell` question verbatim (review Finding 1, PB-DX25 fix cycle). This
    // is deliberately NOT routed through `state::stack_registry::
    // card_in_stack_zone` -- "is it a spell" is a different question from
    // "does it own a card" (CR 707.10: a copy is a spell with no card), which
    // is the whole point of that registry's own doc comment (plan §3.4).
    let targeting_is_spell = state
        .stack_objects
        .iter()
        .find(|so| so.id == targeting_stack_id)
        .map(|so| {
            matches!(
                so.kind,
                StackObjectKind::Spell { .. } | StackObjectKind::MutatingCreatureSpell { .. }
            )
        })
        .unwrap_or(false);
    let target_controller = state.objects.get(&target_id).map(|o| o.controller);
    for src in state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.is_phased_in())
    {
        let resolved_chars = crate::rules::layers::expect_characteristics(state, src.id);
        for (idx, trigger_def) in resolved_chars.triggered_abilities.iter().enumerate() {
            let TriggerEvent::PermanentBecomesTarget {
                scope,
                by_opponent,
                include_abilities,
            } = &trigger_def.trigger_on
            else {
                continue;
            };
            // CR 601.2c vs 602.2b: spell-only unless include_abilities is set.
            if !*include_abilities && !targeting_is_spell {
                continue;
            }
            // CR 702.21a-style opponent gate.
            if *by_opponent && targeting_controller == src.controller {
                continue;
            }
            // Scope gate.
            match scope {
                None => {
                    if src.id != target_id {
                        continue;
                    }
                }
                Some(filter) => {
                    if target_controller != Some(src.controller) {
                        continue;
                    }
                    // CR 608.2b: target may have left before the trigger check; fizzle the scope gate.
                    let Some(_target_obj) = state.objects.get(&target_id) else {
                        continue;
                    };
                    let target_chars =
                        crate::rules::layers::expect_characteristics(state, target_id);
                    if !crate::effects::matches_filter(&target_chars, filter) {
                        continue;
                    }
                }
            }
            // CR 603.4: Check intervening-if at trigger time. Not a look-back
            // trigger — `src` is filtered to the battlefield above.
            if let Some(ref cond) = trigger_def.intervening_if {
                if !check_intervening_if(
                    state,
                    cond,
                    src.controller,
                    src.id,
                    None,
                    InterveningIfMoment::TriggerTime,
                    &[],
                ) {
                    continue;
                }
            }
            triggers.push(PendingTrigger {
                embedded_effect: trigger_def.effect.clone(),
                ability_index: idx,
                triggering_event: Some(trigger_def.trigger_on.clone()),
                targeting_stack_id: Some(targeting_stack_id),
                ..PendingTrigger::blank(src.id, src.controller, PendingTriggerKind::Normal)
            });
        }
    }
}
/// CR 510.3a / CR 603.2 (PB-DX36, `OOS-CARDS2-6`): queue every "deals damage"
/// trigger caused by one damage event — the SelfDealsDamage family on `source`
/// itself, and the Equipment/Aura families over `source`'s attachments.
///
/// `targets` is every `(recipient, amount)` pair `source` dealt damage to in
/// THIS event — one pair for a `GameEvent::DamageDealt` call, and every
/// assignment a source made in one `GameEvent::CombatDamageDealt` event
/// (multi-block, trample) for the combat arm's grouped-by-source call. CR
/// 510.2 makes every entry in that list SIMULTANEOUS and CR 603.2c makes an
/// ability trigger only ONCE per event, so the SELF family
/// (`SelfDealsDamage`/`…ToPlayer`/`…ToOpponent`) is dispatched exactly ONCE per
/// call, with `amount` = the SUM of every entry — CR 608.2h/113.7a's "that
/// much" for the whole event, not one assignment (`/review` HIGH 1; proven live
/// on `exalted_angel` — a multi-block or trample event used to dispatch this
/// family once PER ASSIGNMENT, each carrying only that assignment's own amount,
/// undercounting BOTH the trigger count and the life gained).
///
/// `damaged_player` is the Player recipient if `source` has one in this event,
/// else `None` — CR 510.2/CR 509.2 admit at most one Player-target assignment
/// per source per combat-damage step (one attacker attacks one player/
/// planeswalker), so this is never ambiguous. The SELF family fires regardless
/// of recipient (CR 603.2's "any damage") and does not gate on it.
///
/// The Equipment/Aura ATTACHMENT family stays keyed on that single
/// Player-target entry's OWN amount (`player_amount` below), not the summed
/// total — it must NOT change amount semantics: a trampler dealing 2 to a
/// blocker and 4 to the defending player reports 4 to its Equipment/Aura, not
/// 6. This is why the attachment family was already correct at count == 1
/// before this fix (only the Player-target assignment ever populated
/// `damaged_player`, so only it ever reached the attachment loop) and only the
/// SELF family's count/amount were wrong.
///
/// `is_combat` is a property of the EVENT, not of any ability:
/// `GameEvent::CombatDamageDealt` passes `true`, `GameEvent::DamageDealt` passes
/// `false`. Combat damage is emitted only as `CombatDamageDealt` (verified:
/// `rules/combat.rs`'s combat-damage-dealing site is the sole combat emit site
/// and it emits no `DamageDealt`), so the two call sites are **disjoint by
/// construction** and a given ability — which lowers to exactly one
/// `trigger_on` (see `build_face_triggered_abilities` in `testing/replay_harness.rs`)
/// — fires exactly once per damage event, once per source (this function's
/// per-source-grouped call), which is the two-part property this function now
/// guarantees together. This is the property PB-DX47's double-push defect
/// violated (`OOS-DX24-4`), and it is why every behavioural probe for this
/// primitive asserts a trigger COUNT rather than `>= 1`.
///
/// On a combat-damage event (`is_combat: true`) this fires BOTH the
/// `…CombatDamage…` events AND the `…AnyDamage…` events (combat damage is also
/// "any damage" — CR 603.2); on a noncombat event it fires only the
/// `…AnyDamage…` events.
fn queue_damage_source_triggers(
    state: &GameState,
    triggers: &mut Vec<PendingTrigger>,
    source: ObjectId,
    targets: &[(CombatDamageTarget, u32)],
    is_combat: bool,
) {
    // CR 603.2g: damage with amount == 0 (fully prevented) does not trigger —
    // per assignment (filtered out of `nonzero` below, so a 0-amount entry
    // contributes nothing to the sum and cannot supply `damaged_player`) AND in
    // aggregate (a source whose every entry was fully prevented must not fire
    // at all, checked on `total_amount` before any dispatch).
    let nonzero: Vec<&(CombatDamageTarget, u32)> =
        targets.iter().filter(|(_, amt)| *amt > 0).collect();
    let total_amount: u32 = nonzero.iter().map(|(_, amt)| *amt).sum();
    if total_amount == 0 {
        return;
    }
    // CR 603.10: combat/noncombat "deals damage" triggers are NOT look-back —
    // the source must still be on the battlefield when the event fires.
    // CR 113.7a: the damage source may have left the battlefield between the
    // damage event and this collector running; a quiet `None` is a rules-correct
    // fizzle (SR-4), not an engine bug — `fizzle_object`, not a bare lookup.
    let Some(source_obj) = state.fizzle_object(source) else {
        return;
    };
    if source_obj.zone != ZoneId::Battlefield {
        return;
    }
    let source_controller = source_obj.controller;
    // CR 510.2/CR 509.2: at most one Player-target entry per source per event —
    // see this function's doc.
    let damaged_player = nonzero.iter().find_map(|(t, _)| match t {
        CombatDamageTarget::Player(pid) => Some(*pid),
        _ => None,
    });
    let player_amount = nonzero
        .iter()
        .find_map(|(t, amt)| match t {
            CombatDamageTarget::Player(_) => Some(*amt),
            _ => None,
        })
        .unwrap_or(0);
    // Set the common per-call fields on every trigger pushed since `pre_len`.
    fn populate(
        triggers: &mut [PendingTrigger],
        pre_len: usize,
        source: ObjectId,
        amount: u32,
        is_combat: bool,
        damaged_player: Option<PlayerId>,
    ) {
        for t in &mut triggers[pre_len..] {
            if let Some(pid) = damaged_player {
                t.damaged_player = Some(pid);
            }
            t.damage_dealt_amount = amount;
            t.entering_object_id = Some(source);
            if is_combat {
                t.combat_damage_amount = amount;
            }
        }
    }
    // ── Self family: TriggerCondition::WhenDealsDamage lowering ──────────────
    // CR 603.2c / CR 510.2: dispatched ONCE per call (i.e. once per source per
    // event), amount = `total_amount` — the sum of every entry `source` dealt
    // in this event.
    {
        let pre_len = triggers.len();
        collect_triggers_for_event(
            state,
            triggers,
            TriggerEvent::SelfDealsDamage,
            Some(source),
            None,
        );
        populate(
            triggers,
            pre_len,
            source,
            total_amount,
            is_combat,
            damaged_player,
        );
    }
    if let Some(pid) = damaged_player {
        let pre_len = triggers.len();
        collect_triggers_for_event(
            state,
            triggers,
            TriggerEvent::SelfDealsDamageToPlayer,
            Some(source),
            None,
        );
        populate(
            triggers,
            pre_len,
            source,
            total_amount,
            is_combat,
            damaged_player,
        );
        if pid != source_controller {
            let pre_len = triggers.len();
            collect_triggers_for_event(
                state,
                triggers,
                TriggerEvent::SelfDealsDamageToOpponent,
                Some(source),
                None,
            );
            populate(
                triggers,
                pre_len,
                source,
                total_amount,
                is_combat,
                damaged_player,
            );
        }
    }
    // ── Attachment family: Equipment + Aura ───────────────────────────────────
    // Keyed on `player_amount` (the single Player-target entry's own amount),
    // NOT `total_amount` — see this function's doc. Amount semantics unchanged
    // from before this fix.
    let attachments: Vec<ObjectId> = source_obj.attachments.iter().copied().collect();
    for attachment_id in attachments {
        // CR 510.3a: "Whenever equipped creature deals combat damage to a
        // player" — the printed text is "deals COMBAT damage", so only fires
        // on the combat-damage arm, and only to a player.
        if is_combat {
            if let Some(_pid) = damaged_player {
                let pre_len = triggers.len();
                collect_triggers_for_event(
                    state,
                    triggers,
                    TriggerEvent::EquippedCreatureDealsCombatDamageToPlayer,
                    Some(attachment_id),
                    None,
                );
                populate(
                    triggers,
                    pre_len,
                    source,
                    player_amount,
                    is_combat,
                    damaged_player,
                );
            }
        }
        let Some(pid) = damaged_player else {
            continue;
        };
        // CR 510.3a: "Whenever enchanted creature deals damage to a player" —
        // combat damage is also "any damage", so this fires on BOTH arms.
        let pre_len = triggers.len();
        collect_triggers_for_event(
            state,
            triggers,
            TriggerEvent::EnchantedCreatureDealsAnyDamageToPlayer,
            Some(attachment_id),
            None,
        );
        populate(
            triggers,
            pre_len,
            source,
            player_amount,
            is_combat,
            damaged_player,
        );
        if is_combat {
            let pre_len = triggers.len();
            collect_triggers_for_event(
                state,
                triggers,
                TriggerEvent::EnchantedCreatureDealsCombatDamageToPlayer,
                Some(attachment_id),
                None,
            );
            populate(
                triggers,
                pre_len,
                source,
                player_amount,
                is_combat,
                damaged_player,
            );
        }
        // The "…ToOpponent" siblings are scoped to an opponent of THAT
        // ATTACHMENT'S controller (not the damage source's controller) — a
        // per-attachment check, which is why it lives inside this loop.
        // CR 113.7a: the attachment may itself have left the battlefield (e.g.
        // an SBA-destroyed Aura whose enchanted permanent left the zone it was
        // attached to) — fizzle_object, not a bare lookup.
        if let Some(att_obj) = state.fizzle_object(attachment_id) {
            if pid != att_obj.controller {
                let pre_len = triggers.len();
                collect_triggers_for_event(
                    state,
                    triggers,
                    TriggerEvent::EnchantedCreatureDealsAnyDamageToOpponent,
                    Some(attachment_id),
                    None,
                );
                populate(
                    triggers,
                    pre_len,
                    source,
                    player_amount,
                    is_combat,
                    damaged_player,
                );
                if is_combat {
                    let pre_len = triggers.len();
                    collect_triggers_for_event(
                        state,
                        triggers,
                        TriggerEvent::EnchantedCreatureDealsCombatDamageToOpponent,
                        Some(attachment_id),
                        None,
                    );
                    populate(
                        triggers,
                        pre_len,
                        source,
                        player_amount,
                        is_combat,
                        damaged_player,
                    );
                }
            }
        }
    }
}
/// Collect triggered abilities of type `event_type` from battlefield permanents.
///
/// If `only_object` is `Some(id)`, only checks that specific object.
/// If `only_object` is `None`, checks all permanents on the battlefield.
///
/// `entering_object` is the object that entered the battlefield to cause this event,
/// if applicable (used by `TriggerDoublerFilter::ArtifactOrCreatureETB` to verify
/// the entering object's card types — CR 603.2d).
fn collect_triggers_for_event(
    state: &GameState,
    triggers: &mut Vec<PendingTrigger>,
    event_type: TriggerEvent,
    only_object: Option<ObjectId>,
    entering_object: Option<ObjectId>,
) {
    let object_ids: Vec<ObjectId> = if let Some(id) = only_object {
        vec![id]
    } else {
        state
            .objects
            .values()
            .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in())
            .map(|obj| obj.id)
            .collect()
    };
    for obj_id in object_ids {
        let Some(obj) = state.objects.get(&obj_id) else {
            continue;
        };
        if obj.zone != ZoneId::Battlefield {
            continue;
        }
        // CR 708.3: Face-down permanents have no triggered abilities.
        // A permanent entering the battlefield face-down (via Manifest, Cloak, or Morph cast)
        // must not fire its ETB triggered abilities. The morph cast path suppresses this at
        // resolution; here we suppress it for any face-down permanent receiving
        // SelfEntersBattlefield — covering Manifest and Cloak effect paths.
        if obj.status.face_down
            && obj.face_down_as.is_some()
            && event_type == TriggerEvent::SelfEntersBattlefield
        {
            continue;
        }
        // CR 613.1f (Layer 6): Use layer-resolved triggered abilities so that
        // ability-removing effects (Humility, Dress Down) suppress triggers.
        let resolved_chars = crate::rules::layers::expect_characteristics(state, obj_id);
        for (idx, trigger_def) in resolved_chars.triggered_abilities.iter().enumerate() {
            if trigger_def.trigger_on != event_type {
                continue;
            }
            // CR 508.1m / CR 603.2: AnyCreatureYouControlAttacks and
            // AnyCreatureYouControlDealsCombatDamageToPlayer controller filtering.
            // These events use `entering_object` to carry the attacking/damage-dealing
            // creature. We only fire on trigger sources controlled by the same player
            // as the attacking/dealing creature.
            if matches!(
                event_type,
                TriggerEvent::AnyCreatureYouControlAttacks
                    | TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer
            ) {
                if let Some(attacking_id) = entering_object {
                    if let Some(attacking_obj) = state.objects.get(&attacking_id) {
                        // Only trigger if the attacking/dealing creature is controlled by
                        // the same player as the trigger source ("you control" filter).
                        if attacking_obj.controller != obj.controller {
                            continue;
                        }
                        // PB-N: tighten combat_damage_filter to DAMAGE events only.
                        // Previously this block ran for both AnyCreatureYouControlAttacks
                        // and AnyCreatureYouControlDealsCombatDamageToPlayer events, which
                        // was a latent semantic bug (the field name says "combat damage"
                        // but fired on attacks too). Now gated on the damage event only.
                        // CR 510.3a: Apply combat_damage_filter — subtype, token, keyword checks.
                        if event_type
                            == TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer
                        {
                            if let Some(ref filter) = trigger_def.combat_damage_filter {
                                let dealing_chars = crate::rules::layers::expect_characteristics(
                                    state,
                                    attacking_id,
                                );
                                // is_token check: uses the object's is_token field directly.
                                if filter.is_token && !attacking_obj.is_token {
                                    continue;
                                }
                                // Other filter fields (subtype, card type, etc.) checked via matches_filter.
                                if !crate::effects::matches_filter(&dealing_chars, filter) {
                                    continue;
                                }
                            }
                        }
                        // PB-N: triggering_creature_filter — subtype/color/type filter on the
                        // attacking creature. Applies to BOTH attack and damage events (author's
                        // choice per trigger def). CR 508.1m / CR 603.2.
                        if let Some(ref creature_filter) = trigger_def.triggering_creature_filter {
                            let attacking_chars =
                                crate::rules::layers::expect_characteristics(state, attacking_id);
                            // is_token check: runtime field on GameObject.
                            if creature_filter.is_token && !attacking_obj.is_token {
                                continue;
                            }
                            if !crate::effects::matches_filter(&attacking_chars, creature_filter) {
                                continue;
                            }
                        }
                    } else {
                        // Attacking creature not found — skip conservatively.
                        continue;
                    }
                } else {
                    // No attacker context — skip.
                    continue;
                }
            }
            // CR 502.3 / 603.2e (PB-AC1): AnyPermanentUntaps — a GLOBAL trigger. The
            // untapped permanent's id is carried via `entering_object`. If
            // `triggering_creature_filter` is set, the untapped permanent must match it
            // (layer-resolved) AND, if the filter sets a controller scope, the untapped
            // permanent's controller must satisfy it relative to the trigger source's
            // controller ("you control" / "an opponent controls").
            if event_type == TriggerEvent::AnyPermanentUntaps {
                let Some(untapped_id) = entering_object else {
                    continue;
                };
                let Some(untapped_obj) = state.objects.get(&untapped_id) else {
                    continue;
                };
                if let Some(ref filter) = trigger_def.triggering_creature_filter {
                    match filter.controller {
                        TargetController::Any => {}
                        TargetController::You => {
                            if untapped_obj.controller != obj.controller {
                                continue;
                            }
                        }
                        TargetController::Opponent => {
                            if untapped_obj.controller == obj.controller {
                                continue;
                            }
                        }
                        TargetController::DamagedPlayer => {}
                    }
                    let untapped_chars =
                        crate::rules::layers::expect_characteristics(state, untapped_id);
                    if !crate::effects::matches_filter(&untapped_chars, filter) {
                        continue;
                    }
                }
            }
            // CR 122.6 / 122.7 (PB-AC1): CounterPlaced — a GLOBAL trigger. The receiving
            // permanent's id is carried via `entering_object`. `counter_filter` restricts
            // which counter kind fires; `counter_on_self` restricts the trigger to firing
            // only when the trigger source itself received the counter(s); otherwise
            // `triggering_creature_filter` (if set) restricts which OTHER permanent's
            // counter-placement fires the trigger.
            if event_type == TriggerEvent::CounterPlaced {
                let Some(receiving_id) = entering_object else {
                    continue;
                };
                if trigger_def.counter_on_self {
                    if receiving_id != obj_id {
                        continue;
                    }
                } else if let Some(ref filter) = trigger_def.triggering_creature_filter {
                    let Some(receiving_obj) = state.objects.get(&receiving_id) else {
                        continue;
                    };
                    match filter.controller {
                        TargetController::Any => {}
                        TargetController::You => {
                            if receiving_obj.controller != obj.controller {
                                continue;
                            }
                        }
                        TargetController::Opponent => {
                            if receiving_obj.controller == obj.controller {
                                continue;
                            }
                        }
                        TargetController::DamagedPlayer => {}
                    }
                    let receiving_chars =
                        crate::rules::layers::expect_characteristics(state, receiving_id);
                    if !crate::effects::matches_filter(&receiving_chars, filter) {
                        continue;
                    }
                }
            }
            // PB-OS11 (CR 508.1 / CR 508.1m / CR 603.2c): WheneverYouAttack's optional
            // attacker-set filter. This is a BATCH trigger — it fires ONCE per combat
            // (the dispatch loop at L4147-4170 already calls collect_triggers_for_event
            // once per controller-source, not once per attacker) iff at least one
            // declared attacker controlled by this trigger's controller matches the
            // filter. Distinct from AnyCreatureYouControlAttacks (WheneverCreatureYou
            // ControlAttacks), which fires once PER matching attacker.
            if event_type == TriggerEvent::ControllerAttacks {
                if let Some(ref filter) = trigger_def.triggering_creature_filter {
                    let any_match = state
                        .combat
                        .as_ref()
                        .map(|combat| {
                            combat.attackers.keys().any(|aid| {
                                let Some(ao) = state.objects.get(aid) else {
                                    return false;
                                };
                                if ao.controller != obj.controller {
                                    return false;
                                }
                                // is_token / is_nontoken: GameObject runtime fields,
                                // not visible to matches_filter — checked explicitly
                                // (mirrors the ETB/death/combat-damage filter blocks).
                                if filter.is_token && !ao.is_token {
                                    return false;
                                }
                                if filter.is_nontoken && ao.is_token {
                                    return false;
                                }
                                let ac = crate::rules::layers::expect_characteristics(state, *aid);
                                crate::effects::matches_filter(&ac, filter)
                            })
                        })
                        .unwrap_or(false);
                    if !any_match {
                        continue;
                    }
                }
            }
            // CR 603.2 / CR 207.2c: Apply ETB filter for Alliance and similar
            // "whenever [another] [creature] [you control] enters" triggers.
            // All filter conditions must pass (AND logic).
            if let Some(ref etb_filter) = trigger_def.etb_filter {
                if let Some(entering_id) = entering_object {
                    // exclude_self: "another" qualifier -- skip if the entering
                    // permanent IS the trigger source.
                    if etb_filter.exclude_self && obj_id == entering_id {
                        continue;
                    }
                    if let Some(entering_obj) = state.objects.get(&entering_id) {
                        // CR 613.1d (Layer 4): Use layer-resolved card types so
                        // animated permanents are recognized as creatures.
                        let entering_chars =
                            crate::rules::layers::expect_characteristics(state, entering_id);
                        // creature_only: entering permanent must be a creature.
                        if etb_filter.creature_only
                            && !entering_chars.card_types.contains(&CardType::Creature)
                        {
                            continue;
                        }
                        // controller_you: entering permanent must share controller
                        // with the trigger source's controller.
                        if etb_filter.controller_you && entering_obj.controller != obj.controller {
                            continue;
                        }
                        // color_filter: entering permanent must have this color
                        // (layer-resolved via entering_chars from calculate_characteristics).
                        if let Some(ref color) = etb_filter.color_filter {
                            if !entering_chars.colors.contains(color) {
                                continue;
                            }
                        }
                        // PB-L (CR 207.2c / CR 603.2): card_type_filter — entering
                        // permanent must have this card type (layer-resolved).
                        // Used by Landfall ("land"), Horn of Greed ("land"), and other
                        // non-creature type-filtered "whenever a [type] enters" triggers.
                        if let Some(ref ct) = etb_filter.card_type_filter {
                            if !entering_chars.card_types.contains(ct) {
                                continue;
                            }
                        }
                        // PB-AC0 (CR 603.2 / CR 205.3 / CR 111.1): honor
                        // triggering_creature_filter on the creature-ETB path — subtype /
                        // nontoken / exclude_subtypes / and any other matches_filter-checked
                        // constraint on the entering creature. Mirrors the AnyCreatureDies
                        // block and the combat-damage block.
                        //
                        // CR 603.10: ETB is NOT a look-back-in-time trigger — the entering
                        // permanent's characteristics are evaluated as they exist immediately
                        // after entry, so we use calculate_characteristics on the live object
                        // (no LKI snapshot needed, unlike the death path which uses CR 603.10a).
                        //
                        // Scoped INSIDE the etb_filter block to exclude death/attack defs.
                        // Death defs are handled by their own arm in check_triggers
                        // (~L4287); attack/combat-damage defs have etb_filter:None
                        // so they never enter this block.
                        if let Some(ref creature_filter) = trigger_def.triggering_creature_filter {
                            // is_token / is_nontoken: GameObject runtime fields, not in
                            // Characteristics — checked explicitly (matches_filter cannot see
                            // them). Mirrors the death-path explicit guards.
                            if creature_filter.is_token && !entering_obj.is_token {
                                continue;
                            }
                            if creature_filter.is_nontoken && entering_obj.is_token {
                                continue;
                            }
                            // CR 613.1d (Layer 4): entering_chars is already layer-resolved
                            // (computed above via calculate_characteristics) — subtypes and
                            // types for animated / type-granted permanents are correct.
                            if !crate::effects::matches_filter(&entering_chars, creature_filter) {
                                continue;
                            }
                        }
                    } else {
                        // Entering object not found -- skip conservatively.
                        continue;
                    }
                }
                // If no entering_object provided but filter is set, skip --
                // ETB filters require knowing the entering object.
                else {
                    continue;
                }
            }
            // CR 603.4: Check intervening-if at trigger time.
            // If the condition is false, the ability does not trigger.
            // PB-DX1: this is the headline site — ALL 34 lowered trigger events
            // dispatch through here. Not a look-back trigger: `obj.zone ==
            // Battlefield` is enforced above (the `only_object`/full-scan filter).
            if let Some(ref cond) = trigger_def.intervening_if {
                if !check_intervening_if(
                    state,
                    cond,
                    obj.controller,
                    obj_id,
                    None,
                    InterveningIfMoment::TriggerTime,
                    &[],
                ) {
                    continue;
                }
            }
            triggers.push(PendingTrigger {
                // MR-B12-04: capture the triggered ability's effect now, while the
                // source object still exists. If the source changes zones before
                // resolution (CR 400.7), this is the only surviving copy of the effect.
                embedded_effect: trigger_def.effect.clone(),
                ability_index: idx,
                triggering_event: Some(event_type.clone()),
                entering_object_id: entering_object,
                ..PendingTrigger::blank(obj_id, obj.controller, PendingTriggerKind::Normal)
            });
        }
    }
}
// ---------------------------------------------------------------------------
// Emblem trigger scanning (CR 113.6p, CR 114.4)
// ---------------------------------------------------------------------------
/// Scan all emblem objects in the command zone for triggered abilities matching `event_type`.
///
/// CR 113.6p / CR 114.4: Abilities of emblems function in the command zone.
/// This function mirrors `collect_triggers_for_event` but targets emblems instead of
/// battlefield permanents.
///
/// `caster_player`: if `Some(p)`, only fires emblem triggers owned by player `p`
/// (for "whenever YOU cast" semantics). If `None`, fires all matching emblem triggers.
pub(crate) fn collect_emblem_triggers_for_event(
    state: &GameState,
    triggers: &mut Vec<PendingTrigger>,
    event_type: TriggerEvent,
    caster_player: Option<PlayerId>,
) {
    let emblem_ids: Vec<ObjectId> = state
        .objects
        .values()
        .filter(|obj| obj.is_emblem && matches!(obj.zone, ZoneId::Command(_)))
        .map(|obj| obj.id)
        .collect();
    for obj_id in emblem_ids {
        let Some(obj) = state.expect_object(obj_id) else {
            continue;
        };
        // If a caster filter is given, only fire triggers for that player's emblems.
        if let Some(caster) = caster_player {
            if obj.controller != caster {
                continue;
            }
        }
        for (idx, trigger_def) in obj.characteristics.triggered_abilities.iter().enumerate() {
            if trigger_def.trigger_on != event_type {
                continue;
            }
            // CR 603.4: Check intervening-if at trigger time. Not a look-back
            // trigger — emblems function in the command zone (CR 113.6p) and
            // `obj_id` is that persistent command-zone object; a source-scoped
            // condition like SourceOnBattlefield correctly reads false here.
            if let Some(ref cond) = trigger_def.intervening_if {
                if !check_intervening_if(
                    state,
                    cond,
                    obj.controller,
                    obj_id,
                    None,
                    InterveningIfMoment::TriggerTime,
                    &[],
                ) {
                    continue;
                }
            }
            triggers.push(PendingTrigger {
                // MR-B12-04: capture the triggered ability's effect now, while the
                // source object still exists. If the source changes zones before
                // resolution (CR 400.7), this is the only surviving copy of the effect.
                embedded_effect: trigger_def.effect.clone(),
                ability_index: idx,
                triggering_event: Some(event_type.clone()),
                ..PendingTrigger::blank(obj_id, obj.controller, PendingTriggerKind::Normal)
            });
        }
    }
}
// ---------------------------------------------------------------------------
// Graveyard trigger dispatch (PB-35, CR 603.3 / TriggerZone::Graveyard)
// ---------------------------------------------------------------------------
/// Scan all objects in graveyard zones for CardDef triggered abilities with
/// `trigger_zone: Some(TriggerZone::Graveyard)` that match the given event.
///
/// CR 603.3: Triggers fire whenever the trigger event occurs, regardless of zone.
/// `trigger_zone: Some(TriggerZone::Graveyard)` marks abilities that monitor events
/// while their source is in the graveyard (e.g. Bloodghast's Landfall).
///
/// The returned triggers use `PendingTriggerKind::CardDefETB` so that
/// `flush_pending_triggers` and `resolution.rs` look up the effect from the
/// card registry by ability_index (which is an index into CardDef::abilities).
fn collect_graveyard_carddef_triggers(
    state: &GameState,
    triggers: &mut Vec<PendingTrigger>,
    event: &GameEvent,
    entering_object: Option<ObjectId>,
    arrived_in_graveyard_this_batch: &std::collections::BTreeSet<ObjectId>,
) {
    use crate::cards::card_definition::{
        AbilityDefinition, TargetController, TargetOwner, TriggerCondition, TriggerZone,
    };
    use crate::state::game_object::TriggerEvent;
    // Collect all graveyard object IDs first to avoid borrow issues.
    let gy_objects: Vec<(
        ObjectId,
        PlayerId,
        Option<crate::state::player::CardId>,
        bool,
    )> = state
        .objects
        .values()
        .filter_map(|obj| match obj.zone {
            ZoneId::Graveyard(owner) => {
                Some((obj.id, owner, obj.card_id.clone(), obj.is_transformed))
            }
            _ => None,
        })
        .collect();
    for (obj_id, owner, card_id_opt, is_transformed) in gy_objects {
        let Some(card_id) = card_id_opt else {
            continue;
        };
        let Some(def) = state.card_registry.get(card_id) else {
            continue;
        };
        // OOS-DX1-4 Q7 (PB-DX24): `is_transformed` is always false for a graveyard
        // object (reset on every zone change, `state/mod.rs`), so this is defensive
        // rather than a live repair -- it makes this loop's expression the SAME as
        // the read side's (`resolution.rs`), rather than resting on a distant
        // reset-on-zone-change invariant. This batch also adds a second `fires` arm
        // to this same loop (Change 3), so making it uniform now matters more than
        // usual.
        for (idx, ability) in def.effective_abilities(is_transformed).iter().enumerate() {
            let AbilityDefinition::Triggered {
                trigger_condition,
                intervening_if,
                trigger_zone: Some(TriggerZone::Graveyard),
                ..
            } = ability
            else {
                continue;
            };
            // Check whether this event matches the trigger condition, and if so,
            // which TriggerEvent it dispatches as (varies by arm below, so this is
            // Option<TriggerEvent> rather than a bare bool -- see the push at the
            // bottom of the loop, which is now shared across BOTH arms).
            let fired_as: Option<TriggerEvent> = match event {
                GameEvent::PermanentEnteredBattlefield {
                    object_id: entering_id,
                    ..
                } => match trigger_condition {
                    TriggerCondition::WheneverPermanentEntersBattlefield {
                        filter,
                        exclude_self,
                    } => {
                        // Landfall: check if the entering permanent matches the filter
                        // (typically land type).
                        // PB-XS-E (CR 109.1 / 603.2): if `exclude_self` is set, the
                        // entering permanent must not be the trigger source itself.
                        // The trigger source (`obj_id`) here lives in the graveyard;
                        // by zone-identity (CR 400.7), an entering battlefield object
                        // is always a different object, so this gate is moot for
                        // graveyard triggers but kept for symmetry with the battlefield
                        // path.
                        let matched = if *exclude_self && *entering_id == obj_id {
                            false
                        } else if let Some(entering_obj) = state.objects.get(entering_id) {
                            let entering_chars =
                                crate::rules::layers::expect_characteristics(state, *entering_id);
                            if let Some(f) = filter {
                                crate::effects::matches_filter(&entering_chars, f)
                                    // "you control" filter: the entering land's controller
                                    // must be the graveyard card's owner.
                                    && match f.controller {
                                        TargetController::You => entering_obj.controller == owner,
                                        _ => true,
                                    }
                            } else {
                                true
                            }
                        } else {
                            false
                        };
                        matched.then_some(TriggerEvent::AnyPermanentEntersBattlefield)
                    }
                    _ => None,
                },
                // CR 603.6c / CR 113.6b (PB-DX24): "Whenever [another/a] [nontoken]
                // creature [you control / an opponent controls] dies" from the
                // GRAVEYARD. Mirrors the battlefield AnyCreatureDies arm
                // (search `df.controller_you` in this file) clause for clause --
                // see the table in the PB-DX24 plan §3.3.
                GameEvent::CreatureDied {
                    object_id: pre_death_id,
                    new_grave_id,
                    controller: death_controller,
                    pre_death_characteristics,
                    ..
                } => match trigger_condition {
                    TriggerCondition::WheneverCreatureDies {
                        controller: death_scope,
                        exclude_self,
                        nontoken_only,
                        filter,
                        owner: owner_scope,
                    } => {
                        // CR 108.4a: a graveyard card has no controller; `owner` (its
                        // OWNER) stands in, mirroring the battlefield arm's
                        // `obj.controller` read. Deliberately controller-scoped, not
                        // owner-scoped, matching the printed-text deviation this DSL
                        // field already carries everywhere else (OOS-DX4-1,
                        // nether_traitor.rs:30-34) -- do NOT read the dying object's
                        // owner here instead, that would give the SAME DSL field two
                        // different meanings at two dispatch sites.
                        let controller_you = matches!(death_scope, Some(TargetController::You));
                        let controller_opponent =
                            matches!(death_scope, Some(TargetController::Opponent));
                        // controller_you / controller_opponent: CR 108.4a, see above.
                        let controller_blocks = (controller_you && *death_controller != owner)
                            || (controller_opponent && *death_controller == owner);
                        // PB-DX28 (CR 108.3 / CR 404.3): the `owner` DSL field, distinct
                        // from `controller` above -- "whenever a creature you OWN dies"
                        // (nether_traitor: "put into YOUR graveyard"). Ownership never
                        // changes across a zone move (`move_object_to_zone` always
                        // carries `owner: old_object.owner` forward), so the dying
                        // creature's owner can be read directly off the now-in-graveyard
                        // object at `new_grave_id` -- no pre-death capture is needed,
                        // unlike `controller` which DOES reset on the move. A missing
                        // `new_grave_id` is a rules-correct fizzle (the card left the
                        // graveyard between the death event and this dispatch), so this
                        // reuses `fizzle_object`, the same helper the `nontoken_only` /
                        // `filter` checks a few lines below already use for the SAME id.
                        let dying_owner = state.fizzle_object(*new_grave_id).map(|o| o.owner);
                        let owner_you = matches!(owner_scope, Some(TargetOwner::You));
                        let owner_opponent = matches!(owner_scope, Some(TargetOwner::Opponent));
                        let owner_blocks = (owner_you && dying_owner != Some(owner))
                            || (owner_opponent && dying_owner == Some(owner));
                        // exclude_self: CR 400.7 -- `obj_id` lives in the GRAVEYARD id
                        // space, so the comparison that can match is `new_grave_id`.
                        // `pre_death_id` is the battlefield id and can never equal a
                        // graveyard id; compared anyway for symmetry (see plan §1.4 --
                        // a battlefield-only comparison here fails OPEN, silently).
                        let exclude_self_blocks =
                            *exclude_self && (*new_grave_id == obj_id || *pre_death_id == obj_id);
                        // nontoken_only: CR 111.7.
                        let nontoken_blocks = *nontoken_only
                            && state
                                .fizzle_object(*new_grave_id)
                                .is_some_and(|o| o.is_token);
                        // CR 603.10a + the Gatherer simultaneity ruling: a
                        // leaves-the-battlefield ability looks back in time. Applied
                        // on THIS arm only -- the ETB arm above dispatches
                        // AnyPermanentEntersBattlefield, which is CR 603.6a, not in
                        // CR 603.10a's list, and must NOT gain this guard (Bloodghast
                        // arriving in the graveyard the same batch as a land entering
                        // DOES trigger).
                        let lookback_blocks = arrived_in_graveyard_this_batch.contains(&obj_id);
                        let matched = if controller_blocks
                            || owner_blocks
                            || exclude_self_blocks
                            || nontoken_blocks
                            || lookback_blocks
                        {
                            false
                        } else {
                            match filter {
                                Some(f) => {
                                    if let Some(dying_obj) = state.fizzle_object(*new_grave_id) {
                                        if f.is_token && !dying_obj.is_token {
                                            false
                                        } else {
                                            // CR 603.10a / 613.1d: pre-death snapshot,
                                            // falling back to the graveyard object's
                                            // base characteristics (mirrors the
                                            // battlefield arm).
                                            let dying_chars =
                                                pre_death_characteristics.clone().unwrap_or_else(
                                                    || dying_obj.characteristics.clone(),
                                                );
                                            crate::effects::matches_filter(&dying_chars, f)
                                        }
                                    } else {
                                        false
                                    }
                                }
                                None => true,
                            }
                        };
                        matched.then_some(TriggerEvent::AnyCreatureDies)
                    }
                    _ => None,
                },
                _ => None,
            };
            let Some(triggering_event) = fired_as else {
                continue;
            };
            // CR 603.4 (PB-DP6): queue-time gate via the shared helper. Behaviour-
            // neutral refactor of the pre-existing inline check (same site the
            // audit named as one of the two already-correct gates); `owner` stays
            // the controller argument and `obj_id` the source, unchanged from
            // before. `SourceOnBattlefield` correctly answers false here since
            // this trigger's source lives in the Graveyard.
            if !carddef_intervening_if_holds_at_queue_time(
                state,
                intervening_if.as_ref(),
                owner,
                obj_id,
            ) {
                continue;
            }
            triggers.push(PendingTrigger {
                ability_index: idx,
                triggering_event: Some(triggering_event),
                entering_object_id: entering_object,
                ..PendingTrigger::blank(
                    obj_id,
                    owner,
                    crate::state::stubs::PendingTriggerKind::CardDefETB,
                )
            });
        }
    }
}
// ---------------------------------------------------------------------------
// Trigger flushing
// ---------------------------------------------------------------------------
/// Place all pending triggered abilities onto the stack in APNAP order (CR 603.3).
///
// ---------------------------------------------------------------------------
// PB-DP8 (DP-6): CR 603.3d triggered-ability target announcement
// ---------------------------------------------------------------------------
/// CR 603.3d / CR 601.2c: does the battlefield object `obj` satisfy `req` as a
/// target for `trigger`?
///
/// This predicate is the pre-PB-DP8 first-match auto-selector's battlefield scan,
/// extracted VERBATIM (`.find(pred)` became `.filter(pred)`). It is deliberately
/// not re-implemented: PB-DP8's answer validation is membership in the candidate
/// set this predicate builds, so a fork here would let the engine reject a target
/// it had just offered (SR-38).
///
/// `req` is the requirement with any `UpToN` wrapper already unwrapped, so the
/// nested `UpToN` arm below is reached only for a (currently unauthored) nested
/// `UpToN { inner: UpToN { .. } }`.
fn trigger_battlefield_target_matches(
    state: &GameState,
    trigger: &PendingTrigger,
    req: &crate::cards::card_definition::TargetRequirement,
    obj: &crate::state::game_object::GameObject,
    src_chars_ref: Option<&crate::state::game_object::Characteristics>,
) -> bool {
    use crate::cards::card_definition::TargetRequirement;
    use crate::state::types::CardType as CT;
    if obj.zone != ZoneId::Battlefield || !obj.is_phased_in() {
        return false;
    }
    // CR 613.1f: Use layer-resolved keywords for
    // hexproof/shroud/protection (Humility removes them).
    let chars = crate::rules::layers::expect_characteristics(state, obj.id);
    // Check protection/hexproof/shroud (CR 603.3d).
    if super::validate_target_protection(
        &chars.keywords,
        obj.controller,
        trigger.controller,
        src_chars_ref,
    )
    .is_err()
    {
        return false;
    }
    let is_creature = chars.card_types.contains(&CT::Creature);
    let is_artifact = chars.card_types.contains(&CT::Artifact);
    let is_enchantment = chars.card_types.contains(&CT::Enchantment);
    let is_land = chars.card_types.contains(&CT::Land);
    let is_planeswalker = chars.card_types.contains(&CT::Planeswalker);
    match req {
        TargetRequirement::TargetCreature => is_creature,
        TargetRequirement::TargetPermanent => true,
        // CR 601.2c ("another target"): type-legality identical
        // to TargetPermanent; distinctness enforced at declaration
        // validation (casting.rs), not here.
        TargetRequirement::TargetPermanentDistinctFrom(_) => true,
        TargetRequirement::TargetArtifact => is_artifact,
        TargetRequirement::TargetEnchantment => is_enchantment,
        TargetRequirement::TargetLand => is_land,
        TargetRequirement::TargetPlaneswalker => is_planeswalker,
        TargetRequirement::TargetCreatureOrPlayer => is_creature,
        TargetRequirement::TargetCreatureWithFilter(f) => {
            if !is_creature {
                return false;
            }
            let passes = crate::effects::matches_filter(&chars, f);
            let ctrl_ok = match f.controller {
                crate::cards::card_definition::TargetController::Any => true,
                crate::cards::card_definition::TargetController::You => {
                    obj.controller == trigger.controller
                }
                crate::cards::card_definition::TargetController::Opponent => {
                    obj.controller != trigger.controller
                }
                // PB-D: CR 510.3a, 601.2c — target must be
                // controlled by the player dealt combat damage in
                // the triggering event. Falls through to false if
                // no damaged_player is set (non-combat trigger).
                crate::cards::card_definition::TargetController::DamagedPlayer => trigger
                    .damaged_player
                    .is_some_and(|dp| obj.controller == dp),
            };
            // PB-XS: CR 109.1 / 601.2c — "another target X" exclusion.
            let passes_self = !f.exclude_self || obj.id != trigger.source;
            // PB-XA2: CR 508.1k / 509.1c / 601.2c — combat-role check.
            let passes_combat_role = match (f.is_attacking, f.is_blocking) {
                (false, false) => true,
                (true, false) => state
                    .combat
                    .as_ref()
                    .is_some_and(|c| c.attackers.contains_key(&obj.id)),
                (false, true) => state.combat.as_ref().is_some_and(|c| c.is_blocking(obj.id)),
                (true, true) => state
                    .combat
                    .as_ref()
                    .is_some_and(|c| c.attackers.contains_key(&obj.id) || c.is_blocking(obj.id)),
            };
            // PB-XA2: CR 701.20a / 701.21a — tapped/untapped.
            let passes_tapped = !f.is_tapped || obj.status.tapped;
            let passes_untapped = !f.is_untapped || !obj.status.tapped;
            // PB-DX28: CR 108.3 — ownership scope, mirrors `casting.rs`'s
            // `passes_owner` ("you" = the ability's controller — `trigger.controller`).
            let passes_owner = match f.owner {
                crate::cards::card_definition::TargetOwner::Any => true,
                crate::cards::card_definition::TargetOwner::You => obj.owner == trigger.controller,
                crate::cards::card_definition::TargetOwner::Opponent => {
                    obj.owner != trigger.controller
                }
            };
            passes
                && ctrl_ok
                && passes_self
                && passes_combat_role
                && passes_tapped
                && passes_untapped
                && passes_owner
        }
        TargetRequirement::TargetPermanentWithFilter(f) => {
            let passes = crate::effects::matches_filter(&chars, f);
            let ctrl_ok = match f.controller {
                crate::cards::card_definition::TargetController::Any => true,
                crate::cards::card_definition::TargetController::You => {
                    obj.controller == trigger.controller
                }
                crate::cards::card_definition::TargetController::Opponent => {
                    obj.controller != trigger.controller
                }
                // PB-D: CR 510.3a, 601.2c — target must be
                // controlled by the player dealt combat damage in
                // the triggering event. Falls through to false if
                // no damaged_player is set (non-combat trigger).
                crate::cards::card_definition::TargetController::DamagedPlayer => trigger
                    .damaged_player
                    .is_some_and(|dp| obj.controller == dp),
            };
            // PB-XS: CR 109.1 / 601.2c — "another target X" exclusion.
            let passes_self = !f.exclude_self || obj.id != trigger.source;
            // PB-XA2: CR 508.1k / 509.1c / 601.2c — combat-role check.
            let passes_combat_role = match (f.is_attacking, f.is_blocking) {
                (false, false) => true,
                (true, false) => state
                    .combat
                    .as_ref()
                    .is_some_and(|c| c.attackers.contains_key(&obj.id)),
                (false, true) => state.combat.as_ref().is_some_and(|c| c.is_blocking(obj.id)),
                (true, true) => state
                    .combat
                    .as_ref()
                    .is_some_and(|c| c.attackers.contains_key(&obj.id) || c.is_blocking(obj.id)),
            };
            // PB-XA2: CR 701.20a / 701.21a — tapped/untapped.
            let passes_tapped = !f.is_tapped || obj.status.tapped;
            let passes_untapped = !f.is_untapped || !obj.status.tapped;
            // PB-DX28: CR 108.3 — ownership scope (same rationale as
            // TargetCreatureWithFilter above).
            let passes_owner = match f.owner {
                crate::cards::card_definition::TargetOwner::Any => true,
                crate::cards::card_definition::TargetOwner::You => obj.owner == trigger.controller,
                crate::cards::card_definition::TargetOwner::Opponent => {
                    obj.owner != trigger.controller
                }
            };
            passes
                && ctrl_ok
                && passes_self
                && passes_combat_role
                && passes_tapped
                && passes_untapped
                && passes_owner
        }
        // Player-only reqs are handled above — no objects.
        TargetRequirement::TargetPlayer | TargetRequirement::TargetOpponent => false,
        // Spell targets not applicable for triggered abilities.
        TargetRequirement::TargetSpell | TargetRequirement::TargetSpellWithFilter(_) => false,
        // Graveyard reqs handled above.
        TargetRequirement::TargetCardInYourGraveyard(_)
        | TargetRequirement::TargetCardInGraveyard(_) => false,
        TargetRequirement::TargetAny => is_creature || is_planeswalker,
        TargetRequirement::TargetPlayerOrPlaneswalker => is_planeswalker,
        // TargetSpellOrAbilityWithSingleTarget targets
        // stack objects, not battlefield permanents.
        TargetRequirement::TargetSpellOrAbilityWithSingleTarget => false,
        // TargetSpellWithSingleTarget targets stack
        // objects (spells only), not battlefield permanents.
        TargetRequirement::TargetSpellWithSingleTarget => false,
        // PB-DX52: TargetSpellOrAbility targets the stack, never a battlefield permanent
        // (this function answers "does this BATTLEFIELD object match", `abilities.rs`'s
        // trigger auto-target picker).
        TargetRequirement::TargetSpellOrAbility => false,
        // CR 601.2c / 115.1b: UpToN delegates to inner.
        TargetRequirement::UpToN { inner, .. } => {
            let is_creature = chars.card_types.contains(&CT::Creature);
            let is_artifact = chars.card_types.contains(&CT::Artifact);
            let is_enchantment = chars.card_types.contains(&CT::Enchantment);
            let is_land = chars.card_types.contains(&CT::Land);
            let is_planeswalker = chars.card_types.contains(&CT::Planeswalker);
            match inner.as_ref() {
                TargetRequirement::TargetCreature => is_creature,
                TargetRequirement::TargetPermanent => true,
                TargetRequirement::TargetArtifact => is_artifact,
                TargetRequirement::TargetEnchantment => is_enchantment,
                TargetRequirement::TargetLand => is_land,
                TargetRequirement::TargetPlaneswalker => is_planeswalker,
                TargetRequirement::TargetCreatureOrPlayer => is_creature,
                TargetRequirement::TargetAny => is_creature || is_planeswalker,
                TargetRequirement::TargetPlayerOrPlaneswalker => is_planeswalker,
                TargetRequirement::TargetCreatureWithFilter(f) => {
                    if !is_creature {
                        false
                    } else {
                        let passes = crate::effects::matches_filter(&chars, f);
                        let ctrl_ok = match f.controller {
                            crate::cards::card_definition::TargetController::Any => true,
                            crate::cards::card_definition::TargetController::You => {
                                obj.controller == trigger.controller
                            }
                            crate::cards::card_definition::TargetController::Opponent => {
                                obj.controller != trigger.controller
                            }
                            crate::cards::card_definition::TargetController::DamagedPlayer => {
                                trigger
                                    .damaged_player
                                    .is_some_and(|dp| obj.controller == dp)
                            }
                        };
                        // PB-XS: CR 109.1 / 601.2c — "another target X" exclusion.
                        let passes_self = !f.exclude_self || obj.id != trigger.source;
                        // PB-XA2: CR 508.1k / 509.1c / 601.2c — combat-role check.
                        let passes_combat_role = match (f.is_attacking, f.is_blocking) {
                            (false, false) => true,
                            (true, false) => state
                                .combat
                                .as_ref()
                                .is_some_and(|c| c.attackers.contains_key(&obj.id)),
                            (false, true) => {
                                state.combat.as_ref().is_some_and(|c| c.is_blocking(obj.id))
                            }
                            (true, true) => state.combat.as_ref().is_some_and(|c| {
                                c.attackers.contains_key(&obj.id) || c.is_blocking(obj.id)
                            }),
                        };
                        // PB-XA2: CR 701.20a / 701.21a — tapped/untapped.
                        let passes_tapped = !f.is_tapped || obj.status.tapped;
                        let passes_untapped = !f.is_untapped || !obj.status.tapped;
                        passes
                            && ctrl_ok
                            && passes_self
                            && passes_combat_role
                            && passes_tapped
                            && passes_untapped
                    }
                }
                TargetRequirement::TargetPermanentWithFilter(f) => {
                    let passes = crate::effects::matches_filter(&chars, f);
                    let ctrl_ok = match f.controller {
                        crate::cards::card_definition::TargetController::Any => true,
                        crate::cards::card_definition::TargetController::You => {
                            obj.controller == trigger.controller
                        }
                        crate::cards::card_definition::TargetController::Opponent => {
                            obj.controller != trigger.controller
                        }
                        crate::cards::card_definition::TargetController::DamagedPlayer => trigger
                            .damaged_player
                            .is_some_and(|dp| obj.controller == dp),
                    };
                    // PB-XS: CR 109.1 / 601.2c — "another target X" exclusion.
                    let passes_self = !f.exclude_self || obj.id != trigger.source;
                    // PB-XA2: CR 508.1k / 509.1c / 601.2c — combat-role check.
                    let passes_combat_role = match (f.is_attacking, f.is_blocking) {
                        (false, false) => true,
                        (true, false) => state
                            .combat
                            .as_ref()
                            .is_some_and(|c| c.attackers.contains_key(&obj.id)),
                        (false, true) => {
                            state.combat.as_ref().is_some_and(|c| c.is_blocking(obj.id))
                        }
                        (true, true) => state.combat.as_ref().is_some_and(|c| {
                            c.attackers.contains_key(&obj.id) || c.is_blocking(obj.id)
                        }),
                    };
                    // PB-XA2: CR 701.20a / 701.21a — tapped/untapped.
                    let passes_tapped = !f.is_tapped || obj.status.tapped;
                    let passes_untapped = !f.is_untapped || !obj.status.tapped;
                    passes
                        && ctrl_ok
                        && passes_self
                        && passes_combat_role
                        && passes_tapped
                        && passes_untapped
                }
                // Nested UpToN, graveyard targets, spell targets: not applicable for auto-target on triggers.
                _ => false,
            }
        }
    }
}
/// CR 603.3d / CR 601.2c: every legal choice for one target slot of a triggered
/// ability, plus the pre-PB-DP8 auto-pick.
///
/// This is the SAME code the first-match fallback used, refactored from
/// `.find(<pred>)` into `.filter(<pred>).collect()` + an explicitly-computed
/// `default`. Validation of a submitted answer is membership in `candidates` --
/// the predicate is never re-implemented and never re-run against a
/// possibly-different state.
///
/// **The player-arm / object-arm union is a deliberate, CR-correct widening.**
/// Before PB-DP8 the player arm returned first for `TargetCreatureOrPlayer` /
/// `TargetAny` / `TargetPlayerOrPlaneswalker`, so the object arm's matching
/// branches were unreachable. CR 601.2c makes both kinds legal, so a controller
/// choosing must be offered both. `default` still comes from the player arm, so
/// no bot behaviour changes.
pub(crate) fn trigger_target_candidates(
    state: &GameState,
    trigger: &PendingTrigger,
    req: &crate::cards::card_definition::TargetRequirement,
) -> TriggerTargetOption {
    use crate::cards::card_definition::TargetRequirement;
    // CR 601.2c "up to": an `UpToN` slot may legally be answered with zero
    // targets. Unwrap to the inner requirement for candidate derivation --
    // before PB-DP8 the `UpToN` arm hand-routed player-inner requirements to the
    // player picker and returned `None` for everything else.
    let optional = matches!(req, TargetRequirement::UpToN { .. });
    // CR 601.2c: "If the spell has a variable number of targets, the player
    // announces how many targets they will choose before they announce those
    // targets." `UpToN` declares that number; every other requirement is a
    // one-target slot. (Fix-cycle Finding 2: this used to be dropped, capping
    // Elder Deep-Fiend's "up to four" and Cloud of Faeries' "up to two" at one.)
    let max: u32 = match req {
        TargetRequirement::UpToN { count, .. } => (*count).max(1),
        _ => 1,
    };
    let req: &TargetRequirement = match req {
        TargetRequirement::UpToN { inner, .. } => inner.as_ref(),
        other => other,
    };
    let source_chars = state
        .objects
        .get(&trigger.source)
        .map(|o| o.characteristics.clone());
    let src_chars_ref = source_chars.as_ref();
    let alive = |p: crate::state::player::PlayerId| {
        state
            .expect_player(p)
            .map(|pl| !pl.has_lost && !pl.has_conceded)
            .unwrap_or(false)
    };
    let player_family = matches!(
        req,
        TargetRequirement::TargetPlayer
            | TargetRequirement::TargetCreatureOrPlayer
            | TargetRequirement::TargetAny
            | TargetRequirement::TargetPlayerOrPlaneswalker
    );
    let mut candidates: Vec<SpellTarget> = Vec::new();
    match req {
        // CR 601.2c: every live player is a legal choice for a player-targeting
        // requirement. (The pre-PB-DP8 auto-pick preferred an opponent; that
        // preference survives as `default` below.)
        TargetRequirement::TargetPlayer
        | TargetRequirement::TargetCreatureOrPlayer
        | TargetRequirement::TargetAny
        | TargetRequirement::TargetPlayerOrPlaneswalker => {
            for &p in state.turn.turn_order.iter() {
                if alive(p) {
                    candidates.push(SpellTarget {
                        target: Target::Player(p),
                        zone_at_cast: None,
                    });
                }
            }
        }
        // PB-EF6: CR 102.3/601.2c -- opponents only, NEVER the controller.
        TargetRequirement::TargetOpponent => {
            for &p in state.turn.turn_order.iter() {
                if p != trigger.controller && alive(p) {
                    candidates.push(SpellTarget {
                        target: Target::Player(p),
                        zone_at_cast: None,
                    });
                }
            }
        }
        // Graveyard card targets: scan objects in the appropriate graveyard.
        TargetRequirement::TargetCardInYourGraveyard(filter) => {
            let controller_gy = ZoneId::Graveyard(trigger.controller);
            let combat_ref = state.combat.as_ref();
            candidates.extend(
                state
                    .objects
                    .iter()
                    .filter(|(_, obj)| {
                        // PB-XA2: CR 508.1k / 509.1c — graveyard objects are never in
                        // combat roles. passes_combat_role rejects correctly for all branches.
                        let role_ok = match (filter.is_attacking, filter.is_blocking) {
                            (false, false) => true,
                            (true, false) => {
                                combat_ref.is_some_and(|c| c.attackers.contains_key(&obj.id))
                            }
                            (false, true) => combat_ref.is_some_and(|c| c.is_blocking(obj.id)),
                            (true, true) => combat_ref.is_some_and(|c| {
                                c.attackers.contains_key(&obj.id) || c.is_blocking(obj.id)
                            }),
                        };
                        // PB-XA2: CR 701.20a / 701.21a — tapped/untapped state.
                        let tapped_ok = !filter.is_tapped || obj.status.tapped;
                        let untapped_ok = !filter.is_untapped || !obj.status.tapped;
                        // PB-DX28: CR 108.3 — ownership scope ("you" = trigger.controller).
                        let owner_ok = match filter.owner {
                            crate::cards::card_definition::TargetOwner::Any => true,
                            crate::cards::card_definition::TargetOwner::You => {
                                obj.owner == trigger.controller
                            }
                            crate::cards::card_definition::TargetOwner::Opponent => {
                                obj.owner != trigger.controller
                            }
                        };
                        obj.zone == controller_gy
                        && crate::effects::matches_filter(
                            &obj.characteristics,
                            filter,
                        )
                        // PB-XS: CR 109.1 / 601.2c — "another target X" exclusion.
                        // Death triggers like Elderfang Ritualist scan the graveyard
                        // where the trigger source's post-death object lives.
                        && (!filter.exclude_self || obj.id != trigger.source)
                        && role_ok && tapped_ok && untapped_ok && owner_ok
                    })
                    .map(|(id, obj)| SpellTarget {
                        target: Target::Object(*id),
                        zone_at_cast: Some(obj.zone),
                    }),
            );
        }
        TargetRequirement::TargetCardInGraveyard(filter) => {
            let combat_ref2 = state.combat.as_ref();
            candidates.extend(
                state
                    .objects
                    .iter()
                    .filter(|(_, obj)| {
                        // PB-XA2: CR 508.1k / 509.1c — graveyard objects are never in
                        // combat roles (same rationale as T1 above).
                        let role_ok = match (filter.is_attacking, filter.is_blocking) {
                            (false, false) => true,
                            (true, false) => {
                                combat_ref2.is_some_and(|c| c.attackers.contains_key(&obj.id))
                            }
                            (false, true) => combat_ref2.is_some_and(|c| c.is_blocking(obj.id)),
                            (true, true) => combat_ref2.is_some_and(|c| {
                                c.attackers.contains_key(&obj.id) || c.is_blocking(obj.id)
                            }),
                        };
                        // PB-XA2: CR 701.20a / 701.21a — tapped/untapped state.
                        let tapped_ok = !filter.is_tapped || obj.status.tapped;
                        let untapped_ok = !filter.is_untapped || !obj.status.tapped;
                        // PB-DX28: CR 108.3 — ownership scope (same rationale as T1 above).
                        let owner_ok = match filter.owner {
                            crate::cards::card_definition::TargetOwner::Any => true,
                            crate::cards::card_definition::TargetOwner::You => {
                                obj.owner == trigger.controller
                            }
                            crate::cards::card_definition::TargetOwner::Opponent => {
                                obj.owner != trigger.controller
                            }
                        };
                        matches!(obj.zone, ZoneId::Graveyard(_))
                        && crate::effects::matches_filter(&obj.characteristics, filter)
                        // PB-XS: CR 109.1 / 601.2c — "another target X" exclusion.
                        && (!filter.exclude_self || obj.id != trigger.source)
                        && role_ok && tapped_ok && untapped_ok && owner_ok
                    })
                    .map(|(id, obj)| SpellTarget {
                        target: Target::Object(*id),
                        zone_at_cast: Some(obj.zone),
                    }),
            );
        }
        _ => {}
    }
    // The battlefield scan runs for EVERY family: its own match returns `false`
    // for the player-only, graveyard-only and spell-only requirements, so this
    // adds nothing to those and supplies the whole candidate set for the rest.
    // For the three player/permanent hybrid requirements it is the half that was
    // dead code before PB-DP8.
    candidates.extend(
        state
            .objects
            .iter()
            .filter(|(_, obj)| {
                trigger_battlefield_target_matches(state, trigger, req, obj, src_chars_ref)
            })
            .map(|(id, obj)| SpellTarget {
                target: Target::Object(*id),
                zone_at_cast: Some(obj.zone),
            }),
    );
    // CR 603.3d: the pre-PB-DP8 auto-pick, reproduced exactly.
    //  * player families: first live OPPONENT, else the controller if alive;
    //  * `TargetOpponent`: first live opponent, no self-fallback (PB-EF6);
    //  * an `optional` (`UpToN`) slot with a non-player inner: `None` -- the old
    //    code contributed no target for it;
    //  * everything else: the first match of the same scan, i.e. `candidates[0]`.
    let first_opponent = || {
        state
            .turn
            .turn_order
            .iter()
            .copied()
            .find(|&p| p != trigger.controller && alive(p))
            .map(|p| SpellTarget {
                target: Target::Player(p),
                zone_at_cast: None,
            })
    };
    let default: Option<SpellTarget> = if player_family {
        first_opponent().or_else(|| {
            if alive(trigger.controller) {
                Some(SpellTarget {
                    target: Target::Player(trigger.controller),
                    zone_at_cast: None,
                })
            } else {
                None
            }
        })
    } else if matches!(req, TargetRequirement::TargetOpponent) {
        first_opponent()
    } else if optional {
        None
    } else {
        candidates.first().cloned()
    };
    debug_assert!(
        default
            .as_ref()
            .map(|d| candidates.contains(d))
            .unwrap_or(true),
        "PB-DP8: a slot default must be a member of its own candidate set"
    );
    TriggerTargetOption {
        optional,
        candidates,
        default,
        max,
    }
}
/// PB-DX35 (`OOS-DX4-2`): the mode(s) a modal triggered ability is put on the
/// stack with, and the target requirements those modes announce.
///
/// `None` means CR 700.2b's "If no mode is chosen, the ability is removed from
/// the stack."
///
/// This is the ONE shared arithmetic behind what were three hand-rolled copies
/// (execution-notes §0.5): `trigger_target_requirements` (has_ability_targets +
/// `StackObject.target_requirements`), `ability_targets` (the CR 603.3d slot
/// derivation feeding the offer/auto-pick), and the modes_chosen assignment at
/// the trigger-push site. A fourth reader, `trigger_ability_target_requirements`
/// (the CR 601.2c cross-slot distinctness check on the answer path), also calls
/// this. `rules::mana.rs`'s `WhenTappedForMana` targeted-vs-untargeted decision
/// (site 4) asks a different question — whether targeted at all, not which
/// targets — and is deliberately NOT unified here.
///
/// **The controller still does not choose a mode** (`decision_site_walk`'s
/// `modal_trigger` row stays `AutoChosen`, execution-notes §0.3): what changes is
/// that the engine's automatic choice becomes CR 700.2b-legal, i.e. it may not
/// pick a mode whose target requirement has no legal candidate.
///
/// **No per-mode offset loop is needed on the trigger path** (unlike the
/// spell-side `resolution.rs`, which rebases a chosen mode's `DeclaredTarget`
/// indices because a spell may choose SEVERAL modes at once and each one's
/// slice must land at its own offset in the concatenated flat list). A
/// triggered ability's `modes` is unsupported above `max_modes: 1`
/// (roster gate `r4` pins the corpus at exactly 1), so `requirements` here is
/// always exactly ONE mode's slice, never a concatenation — it already sits at
/// offset 0, which is where `EffectTarget::DeclaredTarget { index: 0 }` reads.
/// Verified by execution: `pb_dx35_modal_trigger_targets::t2`/`t4` drive the two
/// repaired defs' mode-0 `DeclaredTarget { index: 0 }` clauses through this path.
pub(crate) struct TriggerModalPlan {
    pub modes_chosen: Vec<usize>,
    pub requirements: Vec<crate::cards::card_definition::TargetRequirement>,
}
/// Does every `TargetRequirement` in one mode's slice have a legal candidate (or
/// is itself optional, CR 601.2c "up to")? CR 700.2b: "If one of the modes would
/// be illegal (due to an inability to choose legal targets, for example), that
/// mode can't be chosen."
///
/// # Stated residual: this is a PER-SLOT test, and CR 700.2b is not (`OOS-DX35-10`)
///
/// Legality here means "every slot in this mode's slice has a candidate, or is optional". It
/// does **not** consult `forced_answer_breaks_distinctness`, which `flush_sorted` applies
/// downstream: a mode with two mutually-distinct `TargetPermanentDistinctFrom` slots sharing a
/// single candidate has a candidate per slot and NO legal combination. Such a mode is judged
/// legal here, chosen, and then removed by the CR 601.2c cross-slot check — where CR 700.2b says
/// the NEXT mode should have been chosen instead.
///
/// **Zero corpus exposure, measured rather than assumed**: every `mode_targets` slice in the
/// corpus today holds 0 or 1 requirement, pinned by
/// `core::pb_dx35_modal_trigger_roster`'s census, so no mode can have two slots at all. Closing
/// it means threading the cross-slot check into this predicate, which needs the whole slice's
/// candidate sets rather than one slot's. Found by this batch's own `/review`.
fn trigger_modal_mode_is_legal(
    state: &GameState,
    trigger: &PendingTrigger,
    reqs: &[crate::cards::card_definition::TargetRequirement],
) -> bool {
    // CR 700.2c author invariant, mirroring `casting.rs:3856` (spell) and the activated path's
    // `abilities.rs:481-486`: `mode_targets` may not contain `UpToN`. Both peers hard-REJECT the
    // combination with an `InvalidCommand`; a trigger is not a command and has nothing to reject
    // to, so the trigger path fails CLOSED instead — an `UpToN` slice makes the mode ILLEGAL, so
    // CR 700.2b falls through to a mode that really is legal rather than choosing this one.
    //
    // **↻ Added after this batch's own `/review`, which proved the omission by execution.** The
    // first draft mirrored only the OTHER of the two author invariants (they sit five lines apart
    // in `casting.rs`), and an `UpToN` slot is `optional`, so `opt.optional ||` below judged such
    // a mode unconditionally legal — CR 700.2b's fall-through died and the mode was chosen with
    // no target. Zero corpus exposure (roster `r5` pins the population at zero); the point is
    // that the day a def carries one, the behaviour is defined rather than silently wrong.
    if reqs.iter().any(|r| {
        matches!(
            r,
            crate::cards::card_definition::TargetRequirement::UpToN { .. }
        )
    }) {
        debug_assert!(
            false,
            "CR 700.2c: `ModeSelection.mode_targets` may not contain `UpToN` on a triggered \
             ability (variable-count per-mode targets are unsupported, as on the cast and \
             activated paths). This mode is treated as ILLEGAL so CR 700.2b falls through."
        );
        return false;
    }
    reqs.iter().all(|req| {
        let opt = trigger_target_candidates(state, trigger, req);
        opt.optional || !opt.candidates.is_empty()
    })
}
/// **The object lookup is the LKI one, and that is a deliberate widening (`OOS-DX35-9`).**
/// Before PB-DX35, sites 1 and 2 resolved the trigger source with `state.objects.get(..)` while
/// site 3 used `state.fizzle_object(..)`, the CR 113.7a last-known-information lookup. Unifying
/// them on the LKI one means sites 1 and 2 now see a source that has LEFT the battlefield where
/// they previously saw nothing and fell through to `vec![]`. CR 113.7a says that is the correct
/// reading — an ability on the stack exists independently of its source — and the full suite is
/// green either way, **which is the point: no fixture in this tree distinguishes the two**, so
/// "it changes nothing" is an absence of evidence rather than evidence of absence. A probe that
/// kills the source between queue and flush and asserts the requirement list would settle it.
pub(crate) fn trigger_modal_plan(
    state: &GameState,
    trigger: &PendingTrigger,
) -> Option<TriggerModalPlan> {
    use crate::cards::card_definition::AbilityDefinition;
    if !matches!(
        trigger.kind,
        PendingTriggerKind::Normal | PendingTriggerKind::CardDefETB
    ) {
        // Every other PendingTriggerKind (Evoke, Madness, Miracle, Provoke,
        // KeywordTrigger, ...) is never modal — today's `vec![]`.
        return Some(TriggerModalPlan {
            modes_chosen: vec![],
            requirements: vec![],
        });
    }
    // CR 113.7a: `fizzle_object` is `self.objects.get(&id)` verbatim — the same
    // lookup the two pre-existing target-requirement sites performed directly on
    // `state.objects`. Routing through the LKI-documenting name here is a
    // deliberate widening (the third reader, `trigger_ability_target_requirements`,
    // already used it) that changes nothing observable: it is the identical map
    // lookup under a name that also documents the CR 113.7a fizzle.
    let obj = match state.fizzle_object(trigger.source) {
        Some(o) => o,
        None => {
            return Some(TriggerModalPlan {
                modes_chosen: vec![],
                requirements: vec![],
            })
        }
    };
    // The flat (non-modal) target list, kind-dispatched exactly as the three
    // pre-existing copies read it. Also the requirements for every arm below
    // that is not genuinely modal (CR 700.2b/700.2c: a flat list applies to every
    // mode identically, so legality cannot differ by mode).
    let flat_targets: Vec<crate::cards::card_definition::TargetRequirement> =
        if trigger.kind == PendingTriggerKind::Normal {
            obj.characteristics
                .triggered_abilities
                .get(trigger.ability_index)
                .map(|ab| ab.targets.clone())
                .unwrap_or_default()
        } else {
            obj.card_id
                .as_ref()
                .and_then(|cid| state.card_registry.get(cid.clone()))
                .and_then(|def| {
                    def.effective_abilities(obj.is_transformed)
                        .get(trigger.ability_index)
                })
                .and_then(|abil| match abil {
                    AbilityDefinition::Triggered { targets, .. } => Some(targets.clone()),
                    _ => None,
                })
                .unwrap_or_default()
        };
    // Step 2: `ModeSelection` is read from the REGISTRY regardless of trigger
    // kind — the incumbent source (execution-notes §0.5): the runtime
    // `TriggeredAbilityDef` carries no `modes` field at all. For a `Normal`-kind
    // trigger this reuses `trigger.ability_index` as a registry-ability-list
    // index, which is the SAME pre-existing index-space mismatch the old
    // modes-assignment site already had for three misaligned defs
    // (`hullbreaker_horror`, `glissa_sunslayer`, `junji_the_midnight_sky`) — filed
    // as `OOS-DX35-1`, not fixed here (execution-notes §0.5 "why it is not fixed
    // in this batch"; see also A3/r2/r3).
    let modes: Option<&crate::cards::card_definition::ModeSelection> = obj
        .card_id
        .as_ref()
        .and_then(|cid| state.card_registry.get(cid.clone()))
        .and_then(|def| {
            def.effective_abilities(obj.is_transformed)
                .get(trigger.ability_index)
        })
        .and_then(|abil| match abil {
            AbilityDefinition::Triggered { modes, .. } => modes.as_ref(),
            _ => None,
        });
    let modes = match modes {
        Some(m) if !m.modes.is_empty() => m,
        // Non-modal (no `AbilityDefinition::Triggered.modes`, or an empty mode
        // list — the latter never occurs in the corpus but is handled the same
        // as "no modes"): today's behaviour exactly, `modes_chosen: vec![]`.
        _ => {
            return Some(TriggerModalPlan {
                modes_chosen: vec![],
                requirements: flat_targets,
            })
        }
    };
    if modes.mode_targets.is_none() {
        // A1 step 3: a FLAT list applies to every mode identically, so CR 700.2b
        // legality cannot differ by mode and the existing CR 603.3d slot check
        // already removes the trigger when it is unsatisfiable. `modes_chosen` is
        // today's value, byte-identical: mode 0 whenever a mode exists. This is
        // the arm that keeps every non-repaired corpus modal-triggered-ability
        // def unchanged by this batch (`felidar_retreat`, and every def this
        // batch does not touch).
        return Some(TriggerModalPlan {
            modes_chosen: vec![0],
            requirements: flat_targets,
        });
    }
    // A1 step 5 (CR 700.2c/700.2a): a modal ability combining `max_modes > 1`
    // with `mode_targets: Some(_)` is unsupported, exactly as the activated-ability
    // path already hard-rejects it. Zero corpus members (roster gate `r4`
    // pins `max_modes == 1` for all seven modal triggered abilities) — fail-safe
    // to "choose one mode" (the loop below already does that unconditionally)
    // rather than panicking in release.
    debug_assert!(
        modes.max_modes <= 1,
        "PB-DX35: max_modes > 1 combined with ModeSelection.mode_targets is \
         unsupported on a triggered ability (CR 700.2c/700.2a), mirroring \
         abilities.rs's activated-ability gate; zero corpus members (roster gate r4)"
    );
    // CR 700.2b: choose the FIRST mode (by declared order) whose per-mode target
    // requirements all have a legal candidate. `per_mode_target_requirements` is
    // the SAME helper `handle_cast_spell` and `rules::queries::spell_target_requirements`
    // call for the spell-side modal cast — the shared arithmetic the criterion
    // demands, reused rather than re-derived for the trigger side.
    let chosen_idx = (0..modes.modes.len()).find(|&idx| {
        let reqs = casting::per_mode_target_requirements(modes, std::slice::from_ref(&idx))
            .unwrap_or_default();
        trigger_modal_mode_is_legal(state, trigger, &reqs)
    });
    match chosen_idx {
        Some(idx) => {
            let reqs = casting::per_mode_target_requirements(modes, std::slice::from_ref(&idx))
                .unwrap_or_default();
            Some(TriggerModalPlan {
                modes_chosen: vec![idx],
                requirements: reqs,
            })
        }
        // CR 700.2b: "choose up to one" (min_modes: 0) legally chooses zero modes
        // when none is legal — the ability resolves with no effect.
        None if modes.min_modes == 0 => Some(TriggerModalPlan {
            modes_chosen: vec![],
            requirements: vec![],
        }),
        // CR 700.2b: "If no mode is chosen, the ability is removed from the
        // stack." min_modes >= 1 and no legal mode exists.
        None => None,
    }
}
/// CR 601.2c (closing-review Finding 3, LOW / SR-38): reconcile the per-slot
/// defaults with the cross-slot distinctness the answer handler enforces.
///
/// `trigger_target_candidates` computes each slot's `default` in isolation
/// (`candidates.first()`), so two `TargetPermanentDistinctFrom` slots -- "another
/// target permanent", Hidden Strings' shape -- both defaulted to the SAME
/// permanent, and `handle_choose_trigger_targets`' check (8) then rejected the
/// engine's own answer. Everything that submits the offered default verbatim
/// (`StubProvider`, both bots, the replay-harness pump, the TUI announce key) took
/// that refusal, and `LocalGame` converts a refused fallback into a `Halted`.
///
/// Only `TargetPermanentDistinctFrom` slots are touched, and only when they
/// collide: CR 601.2c forbids repeating a target within ONE instance of the word
/// "target", not across two independent instances, so two ordinary `TargetCreature`
/// slots may legitimately name the same creature and keep the pre-PB-DP8
/// first-match value.
///
/// **Residual** (OOS-DP8-4): if no distinct candidate exists the default is left
/// as-is and the engine still refuses it. That position is a genuine CR 603.3d
/// question ("no legal choices can be made") that this batch does not answer.
fn make_distinct_slot_defaults(
    reqs: &[crate::cards::card_definition::TargetRequirement],
    slots: &mut [TriggerTargetOption],
) {
    use crate::cards::card_definition::TargetRequirement as TR;
    let is_distinct = |i: usize| matches!(reqs.get(i), Some(TR::TargetPermanentDistinctFrom(_)));
    for i in 0..slots.len() {
        if !is_distinct(i) {
            continue;
        }
        // Exactly the pairs the handler's cross-slot check examines.
        let taken: Vec<SpellTarget> = (0..i)
            .filter(|&j| is_distinct(j))
            .filter_map(|j| slots[j].default.clone())
            .collect();
        let collides = slots[i]
            .default
            .as_ref()
            .map(|d| taken.contains(d))
            .unwrap_or(false);
        if !collides {
            continue;
        }
        if let Some(alt) = slots[i]
            .candidates
            .iter()
            .find(|c| !taken.contains(c))
            .cloned()
        {
            slots[i].default = Some(alt);
        }
    }
}
/// CR 601.2c (PB-DP8 fix cycle, Finding 6): flatten a per-slot answer into the one
/// `Vec<SpellTarget>` a stack object carries, preserving each slot's declared
/// width.
///
/// `EffectTarget::DeclaredTarget { index }` reads that vector by absolute index, so
/// slot *i* must start at `sum(slots[..i].max)` no matter how many targets slot *i*
/// was actually answered with. Interior holes are filled with
/// [`SpellTarget::unchosen_slot`]; trailing holes are omitted so an all-empty
/// announcement still produces an EMPTY list (which is what keeps CR 608.2b's "all
/// targets are illegal" fizzle from firing on a legally-empty "up to" answer).
fn flatten_slot_answers(
    slots: &[TriggerTargetOption],
    per_slot: &[Vec<SpellTarget>],
) -> Vec<SpellTarget> {
    debug_assert_eq!(slots.len(), per_slot.len());
    let mut flat: Vec<SpellTarget> = Vec::new();
    let mut offset = 0usize;
    for (slot, chosen) in slots.iter().zip(per_slot.iter()) {
        if !chosen.is_empty() {
            // Pad the earlier slots' holes, now that something real follows them.
            while flat.len() < offset {
                flat.push(SpellTarget::unchosen_slot());
            }
            flat.extend(chosen.iter().cloned());
        }
        offset += slot.max as usize;
    }
    flat
}
/// CR 603.3d (PB-DP8): the deterministic default answer -- byte-identical to the
/// pre-PB-DP8 first-match auto-pick, because each slot's `default` IS the value
/// the old `candidate` expression produced.
///
/// **The engine never calls this on a decision path.** It exists so the
/// simulator's `StubProvider`, the replay harness and the TUI cannot drift from
/// one another (SR-38): each of them submits this as a real `Command`, which is
/// what keeps the replay log a complete record of every choice
/// (Architecture Invariant 1 -- the engine must not know which seats are human).
///
/// # Acceptance guarantee, and its one exception
///
/// `handle_choose_trigger_targets` accepts this answer for every slot list the
/// engine offers, with exactly one exception, which is stated here rather than
/// left implicit (closing-review Finding 3; OOS-DP7-2's failure mode was a doc
/// comment asserting a property the code did not have): if two
/// `TargetPermanentDistinctFrom` slots collide and slot *i*'s candidate set holds
/// **no** member the earlier slot has not already taken, `make_distinct_slot_defaults`
/// has nothing to swap in, both defaults name the same permanent, and the handler's
/// cross-slot distinctness check (CR 601.2c) rejects it. Colliding defaults are
/// otherwise resolved at offer time. No def in the corpus reaches the exception
/// (OOS-DP8-4).
///
/// The **sub-case where every slot has exactly one candidate is no longer part of
/// that exception** (second closing review, Finding 2 -- LOW): those slot lists are
/// never offered at all. Per-slot determinacy used to short-circuit straight past
/// the cross-slot check, so the trigger was placed naming one permanent twice; now
/// `forced_trigger_target_answer` + `forced_answer_breaks_distinctness` recognise
/// that the constraint has no solution and CR 603.3d removes the ability instead.
/// What survives is only the *default-quality* half: a slot list where some slot
/// has two or more candidates always HAS a legal answer, and the first-match
/// default is simply not always it.
pub fn default_trigger_targets(slots: &[TriggerTargetOption]) -> Vec<Vec<Target>> {
    slots
        .iter()
        .map(|s| match &s.default {
            Some(t) => vec![t.target.clone()],
            None => Vec::new(),
        })
        .collect()
}
/// CR 601.2c: an announcement with exactly one legal answer is determined.
///
/// When every slot is required and has exactly one candidate there is nothing for
/// the controller to decide, so the engine places the trigger directly rather than
/// spending a wire round trip on a question with one answer. An `optional` slot is
/// excluded because "choose zero" is a genuine second answer -- **unless** it has
/// no candidate at all, in which case "choose zero" is its only legal answer too
/// (fix-cycle Finding 8).
///
/// Returns the answer itself rather than a `bool` (second closing review,
/// Finding 2 -- LOW) so the caller can check it against the CROSS-slot constraints,
/// which are not a property of any one slot. Per-slot determinacy does not imply
/// the combination is legal.
fn forced_trigger_target_answer(slots: &[TriggerTargetOption]) -> Option<Vec<Vec<SpellTarget>>> {
    slots
        .iter()
        .map(trigger_target_slot_forced_answer)
        .collect()
}
/// CR 601.2c: `true` if a per-slot answer names the same permanent for two
/// `TargetPermanentDistinctFrom` slots -- i.e. the announcement is illegal.
///
/// The exact predicate `handle_choose_trigger_targets`' check (8) applies to a
/// submitted answer, applied here to the engine's own forced answer. Second
/// closing review, Finding 2 (LOW): the forced path bypassed (8) entirely, so two
/// mutually-distinct slots with one shared candidate were placed on the stack
/// naming that permanent twice -- a silent CR 601.2c violation rather than a
/// refusal.
fn forced_answer_breaks_distinctness(
    reqs: &[crate::cards::card_definition::TargetRequirement],
    per_slot: &[Vec<SpellTarget>],
) -> bool {
    use crate::cards::card_definition::TargetRequirement as TR;
    let is_distinct = |i: usize| matches!(reqs.get(i), Some(TR::TargetPermanentDistinctFrom(_)));
    for a in 0..per_slot.len() {
        for b in (a + 1)..per_slot.len() {
            if is_distinct(a)
                && is_distinct(b)
                && !per_slot[a].is_empty()
                && per_slot[a] == per_slot[b]
            {
                return true;
            }
        }
    }
    false
}
/// CR 601.2c: the sole legal answer for a slot, if it has exactly one.
///
/// A required slot with exactly one candidate is determined; an `optional` slot
/// with NO candidates can only be answered with zero targets, so it is determined
/// too (fix-cycle Finding 8 -- asking it was a question with one possible answer).
/// Everything else is a real choice.
fn trigger_target_slot_forced_answer(slot: &TriggerTargetOption) -> Option<Vec<SpellTarget>> {
    if slot.optional {
        if slot.candidates.is_empty() {
            Some(Vec::new())
        } else {
            None
        }
    } else if slot.candidates.len() == 1 {
        Some(vec![slot.candidates[0].clone()])
    } else {
        None
    }
}
/// Called immediately before a player would receive priority. If no pending
/// triggers exist, this is a no-op.
///
/// CR 603.3: "Each time a player would receive priority, the game checks for any
/// triggered abilities that have triggered since the last time a player received
/// priority. If any have triggered, those abilities are put on the stack."
///
/// APNAP ordering (CR 101.4): Active player's triggers go on the stack first
/// (ending up at the bottom), then each non-active player in turn order. The last
/// player's triggers are on top and resolve first.
///
/// Returns events for each ability placed on the stack. Does NOT emit
/// `PriorityGiven` — the caller is responsible for granting priority after.
/// **PB-DP8 (CR 603.3d)**: the batch may now SUSPEND. If this returns with
/// `state.pending_trigger_targets` `Some`, the CR 603.3b batch is INCOMPLETE and
/// the caller must not grant priority or advance -- see the **six** guarded call
/// sites (`enter_step`'s two branches, `handle_declare_attackers`,
/// `handle_declare_blockers`, the resolution tail, and -- added by the fix cycle,
/// review Finding 3 -- `handle_all_passed`'s forced-overdue-payment branch).
/// What each of them still owes on resume is carried as
/// [`crate::state::stubs::FlushResumeSite`].
/// CR 702.21a / CR 603.3b (PB-DX48): the bound on how many *consecutive*
/// becomes-target waves one flush may place.
///
/// A wave exists because putting an ability on the stack can itself make a permanent
/// "become the target" (CR 702.21a), and the trigger that causes is then "triggered
/// but not yet on the stack" in the SAME CR 603.3b window — it must be placed before
/// any player receives priority, not left in the queue until the next command.
///
/// In the corpus this terminates at wave 2 by construction: the only wave-2 trigger
/// buildable today is Ward's, whose own single target is the TARGETING STACK OBJECT
/// (`SpellTarget { target: Object(tsid), zone_at_cast: None }`, built by the
/// `trigger_targets_opt` chain below), and `zone_at_cast: None` never satisfies the
/// battlefield predicate in `rules::events::permanent_targeted_events`. The bound
/// therefore truncates nothing reachable today; it exists so a future
/// `PermanentBecomesTarget` card whose trigger targets a permanent cannot spin the
/// engine, and it stops with a `debug_assert!` rather than silently (SR-4: an engine
/// bug, not a rules-correct fizzle).
const MAX_BECOMES_TARGET_WAVES: u32 = 16;

/// CR 603.3b / CR 702.21a (PB-DX48, `OOS-ENG2-1` ≡ `OOS-ENG2-2`) — flush the queue,
/// then flush again for whatever the flush itself caused to become a target.
///
/// **This wrapper is the batch's second half, and neither seed states that it was
/// needed.** Both rows describe the fix as emitting `GameEvent::PermanentTargeted` at
/// five more announcement sites. Emitting it is necessary and not sufficient: every
/// caller of this function scanned its events for triggers and only THEN called it,
/// so the events a flush ITSELF produced were fed back to nothing. A Ward trigger
/// caused by a *triggered* ability going on the stack would have sat in
/// `state.pending_triggers` until the next command — after priority had already been
/// granted, which CR 603.3b forbids. A batch that took the two rows at their word
/// would have shipped a diff that looks like a fix and moves nothing at the headline
/// site.
///
/// **Why here and not in the callers.** There are six, and they do not agree: two
/// (`rules/engine.rs`'s `check_and_flush_triggers`, `rules/resolution.rs`'s
/// post-resolution sweep) scan their events first, four do not, and
/// `resume_trigger_flush` / `drop_departed_trigger_flush` bypass this function
/// entirely by calling `flush_sorted` directly. Putting the wave loop in the one
/// function all six flush through makes the dispatch a property of flushing rather
/// than of remembering to sweep afterwards.
///
/// **Exactly-once scanning is what makes Ward fire ONCE per targeting event**
/// (CR 702.21a). The cursor is the mechanism, and it is not a theoretical concern: a
/// first implementation hooked `flush_sorted`'s tail instead, and because
/// `Command::ChooseTriggerTargets` then re-scanned the very events that hook had
/// already dispatched, **Ward fired twice** — two `AbilityTriggered`, two ward stack
/// objects, observed before the design changed. Waves here read only the events THIS
/// function appended, and each event is read by exactly one wave.
pub fn flush_pending_triggers(state: &mut GameState) -> Vec<GameEvent> {
    let mut events = flush_pending_triggers_once(state);
    dispatch_becomes_target_waves(state, &mut events);
    events
}

/// CR 702.21a / CR 603.3b — place whatever became a target while `events` was being
/// produced, and keep going until nothing new does.
///
/// `events` must be the events of ONE flush, and every `PermanentTargeted` in it must
/// not already have been dispatched by someone else — the cursor makes each event
/// readable by exactly one wave *within* this call, but it cannot see a second
/// dispatcher upstream. The two callers are chosen for that reason:
///
/// * `flush_pending_triggers` (above) — the function five of the six flush sites go
///   through, and the only one their callers do not sweep after in any consistent way.
/// * `rules::engine::handle_concede` — CR 800.4d resumes a suspended batch through
///   `drop_departed_trigger_flush`, and `Command::Concede`'s arm calls no trigger sweep
///   at all, so without this its resumed triggers would announce targets that nothing
///   dispatches.
///
/// **Deliberately NOT called from `resume_trigger_flush`**, whose events ARE swept:
/// `Command::ChooseTriggerTargets`'s arm runs `check_and_flush_triggers` over them.
/// Calling it there too would dispatch the same event twice and fire Ward twice — the
/// failure an earlier design of this batch actually produced and that was caught by
/// execution. The residual that leaves is `OOS-DX48-3`: that arm's sweep is guarded on
/// `pending_trigger_targets.is_none()`, so a batch that suspends a SECOND time hands
/// its middle section's `PermanentTargeted` events to a caller that never scans them.
pub(crate) fn dispatch_becomes_target_waves(state: &mut GameState, events: &mut Vec<GameEvent>) {
    let mut scanned = 0usize;
    let mut wave = 0u32;
    loop {
        let slice: Vec<GameEvent> = events[scanned..]
            .iter()
            .filter(|e| matches!(e, GameEvent::PermanentTargeted { .. }))
            .cloned()
            .collect();
        scanned = events.len();
        if slice.is_empty() {
            return;
        }
        // The bound is tested HERE, after establishing that there is genuinely more
        // work to do, and not at the bottom of the previous iteration: a batch that
        // needs exactly `MAX_BECOMES_TARGET_WAVES` waves and then finishes is not a
        // runaway, and asserting on it would be a false positive at the boundary.
        if wave >= MAX_BECOMES_TARGET_WAVES {
            debug_assert!(
                false,
                "engine invariant: more than {MAX_BECOMES_TARGET_WAVES} consecutive \
                 CR 702.21a becomes-target waves in one flush. Ward's own trigger targets \
                 the TARGETING STACK OBJECT with `zone_at_cast: None`, which the \
                 battlefield predicate in `rules::events::permanent_targeted_events` never \
                 matches, so a real cascade this deep means a new \
                 `PermanentBecomesTarget` card whose trigger targets a permanent is looping."
            );
            return;
        }
        // CR 603.3b: every ability of one batch goes on the stack in one window, so
        // the targetings they cause are simultaneous with one another for
        // CR 603.10a's look-back purposes — the same timing every caller passes.
        let new_triggers =
            check_triggers_with_timing(state, &slice, EventBatchTiming::Simultaneous);
        if new_triggers.is_empty() {
            return;
        }
        for t in new_triggers {
            state.pending_triggers.push_back(t);
        }
        // CR 603.3d (PB-DP8) — QUEUE, then stop. A suspended flush owns its own
        // continuation and must not be re-entered here.
        //
        // **The queue-then-stop ORDER is the whole of this fix, and the first draft had
        // it backwards.** That draft tested suspension at the TOP of the loop and
        // returned having collected nothing, which silently dropped every
        // `PermanentTargeted` emitted by the members `flush_sorted` placed BEFORE it
        // suspended — and nothing else ever scans them, because every
        // `flush_pending_triggers` caller scans its events before the flush and
        // `Command::ChooseTriggerTargets` sweeps only the RESUMED events. The draft's
        // comment asserted the resumed call would cover it, and that was false in both
        // halves: `resume_trigger_flush` never runs this loop, and its caller's sweep
        // cannot see the prefix. Found by the `/review`, reproduced with TWO triggers
        // (one asking) rather than the three `OOS-DX48-3` originally claimed, and
        // pinned by `primitives::pb_dx48_ward_dispatch::test_dx48_t9_...`.
        //
        // Queueing is both sufficient and safe. Sufficient: `state.pending_triggers`
        // survives the suspension, and the resume's own `check_and_flush_triggers` ->
        // `flush_pending_triggers_once` drains it once the rest of the CR 603.3b batch
        // is on the stack — which is the CR-correct order anyway. Safe against double
        // dispatch: the events that produced these triggers have already been consumed
        // by this loop's cursor and are never scanned again, because the resume starts
        // from a fresh `events` vec.
        if state.pending_trigger_targets.is_some() {
            return;
        }
        events.extend(flush_pending_triggers_once(state));
        wave += 1;
    }
}

/// One CR 603.3b flush: the pre-PB-DX48 body of `flush_pending_triggers`, unchanged.
fn flush_pending_triggers_once(state: &mut GameState) -> Vec<GameEvent> {
    // CR 800.4d (fix-cycle Finding 9, LOW): reconcile the liveness filter with the
    // raw field. `rules::engine::blocking_decision` reports `None` for an entry
    // whose player is no longer alive, but this function and the six in-crate
    // guards read `state.pending_trigger_targets` directly. `handle_concede` clears
    // its OWN player's entry; every other route to elimination (the CR 704.5a/b
    // player-loss SBAs, a resolving effect, a replacement effect) does not -- so
    // such an entry was invisible to the gate while permanently blocking every
    // flush from here on. Reap it the CR 800.4d way at the one place the block
    // actually bites.
    let departed = state
        .pending_trigger_targets
        .as_ref()
        .map(|e| e.player)
        .filter(|p| {
            // SR-25: `expect_player` (a NONSWALLOW predicate read) -- a departed
            // player legitimately answers `alive == false` here, which is exactly
            // the question being asked.
            !state
                .expect_player(*p)
                .map(|pl| !pl.has_lost && !pl.has_conceded)
                .unwrap_or(false)
        });
    let did_reap = departed.is_some();
    let mut reaped = Vec::new();
    if let Some(player) = departed {
        // CLOSING-REVIEW Finding 2 (MEDIUM): drop the reaped entry's priority debt
        // before reaping it. `drop_departed_trigger_flush` ends in
        // `finish_resumed_flush`, which for any `resume_site` other than `None`
        // GRANTS PRIORITY -- here, inside the current caller's own flush. That
        // caller is either one of the six guards (which grants again the moment
        // this function returns with no entry: two `PriorityGiven` for one step
        // entry, two `players_passed` resets) or one of the 30
        // `check_and_flush_triggers` sites (where PB-DP1 already left priority
        // correctly with the actor, so a grant to the ACTIVE player would be an
        // overwrite). The debt belongs to a call site whose moment has passed; the
        // current caller's own obligation is the one that is owed now, and it is
        // recorded by that caller's own `mark_flush_resume_site` if the
        // continuation suspends again.
        //
        // SECOND CLOSING-REVIEW Finding 3 (LOW / OOS-DP8-13): only the PRIORITY
        // half of the debt is dropped. Zeroing the whole `FlushResumeSite` also
        // threw away the `cleanup_sba_rounds` ratchet and the CR 726 mandatory-loop
        // check, and those are not the same severity class as a duplicate event --
        // they are the bound on a genuinely repeating position. The site is still
        // zeroed (so nothing downstream can grant), and its obligations are run
        // here explicitly by `run_flush_resume_obligations`.
        let reaped_site = state
            .pending_trigger_targets
            .as_ref()
            .map(|e| e.resume_site)
            .unwrap_or(FlushResumeSite::None);
        if let Some(e) = state.pending_trigger_targets.as_mut() {
            e.resume_site = FlushResumeSite::None;
        }
        if let Some(evs) = drop_departed_trigger_flush(state, player) {
            reaped = evs;
        }
        // Only once the reaped batch's continuation is COMPLETE: CR 726 cannot be
        // evaluated against a half-placed CR 603.3b batch. Residual, now the whole
        // of OOS-DP8-13: a continuation that immediately re-suspends loses the
        // reaped site's ratchet bump for that round -- the current caller's own
        // site (recorded by its `mark_flush_resume_site`) carries its own copy of
        // both obligations, so the bound is restored on the next completing round.
        if reaped_site != FlushResumeSite::None && state.pending_trigger_targets.is_none() {
            let game_ended = run_flush_resume_obligations(state, reaped_site, &mut reaped);
            if game_ended {
                return reaped;
            }
        }
    }
    // A suspended flush must never be re-entered: the caller's guard should have
    // prevented it, so this is an engine bug, not a rules-correct fizzle (SR-4).
    // An entry the reap above just created is not a re-entrance -- CR 603.3b's
    // continuation may legitimately suspend again on a live player's trigger.
    debug_assert!(
        did_reap || state.pending_trigger_targets.is_none(),
        "flush_pending_triggers re-entered while a CR 603.3d target choice is outstanding"
    );
    if state.pending_trigger_targets.is_some() {
        return reaped;
    }
    if state.pending_triggers.is_empty() {
        return reaped;
    }
    // CR 603.2d: Remove stale TriggerDoubler entries whose source left the battlefield.
    // This prevents accumulation of dead entries from permanents that left the battlefield.
    state.trigger_doublers.retain(|d| {
        state
            .objects
            .get(&d.source)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false)
    });
    // Drain all pending triggers.
    let pending: Vec<PendingTrigger> = state.pending_triggers.iter().cloned().collect();
    state.pending_triggers = imbl::Vector::new();
    // Build APNAP order starting from the active player.
    let apnap = apnap_order(state);
    // Stable-sort by controller position in APNAP order.
    let mut sorted = pending;
    sorted.sort_by_key(|t| {
        apnap
            .iter()
            .position(|&p| p == t.controller)
            .unwrap_or(usize::MAX)
    });
    let mut events = reaped;
    events.extend(flush_sorted(state, sorted, None));
    events
}
/// CR 603.3b / CR 603.3d: place an already-APNAP-sorted batch on the stack, one
/// ability at a time.
///
/// `head_targets`, when `Some`, are the answered targets for `sorted[0]` (the
/// resume path). Every later trigger derives its own normally, which is what
/// CR 603.3d requires -- targets are chosen "as [the ability] goes on the stack",
/// so the earlier abilities of the batch being on the stack already is correct
/// information for the later chooser, not a hazard.
///
/// This function NEVER touches `state.pending_triggers` and NEVER re-sorts: both
/// belong to the public entry point, so a pause preserves the batch's CR 603.3b
/// order byte-for-byte.
fn flush_sorted(
    state: &mut GameState,
    sorted: Vec<PendingTrigger>,
    head_targets: Option<Vec<SpellTarget>>,
) -> Vec<GameEvent> {
    let mut head_targets = head_targets;
    let mut events = Vec::new();
    let mut next_index = 0usize;
    // CR 117.3d (fix-cycle Finding 10): did this call actually put anything on the
    // stack? `events` cannot answer that on the suspend path, because it also
    // carries the `TriggerTargetChoiceRequired` question.
    let mut placed_any = false;
    while next_index < sorted.len() {
        let trigger = sorted[next_index].clone();
        next_index += 1;
        // CR 603.3d / CR 601.2c: bind the resume answer POSITIONALLY to the head of
        // the batch, here, before any `continue` or any branch that can exit the
        // target-derivation chain early.
        //
        // Fix-cycle Finding 1 (HIGH): `head_targets` used to be consumed lazily,
        // inside the `else if let Some(pre) = head_targets.take()` arm of the
        // target chain -- which sits BEHIND both the CR 603.3d "a required slot has
        // no candidates" removal and the `ability_targets.is_empty()` escape, and
        // behind the CR 603.2c once-per-turn `continue`. If the head exited by any
        // of those routes at resume time the answer survived to the NEXT trigger of
        // the batch: a different ability, possibly a different controller, with
        // entirely different `TargetRequirement`s, whose stack object then carried
        // targets that were never validated against its own requirements.
        //
        // An answer belongs to exactly one trigger -- `sorted[0]`, the trigger the
        // entry named -- so bind it by position and let it be dropped with the head
        // if the head is removed.
        let this_head = if next_index == 1 {
            head_targets.take()
        } else {
            None
        };
        // PB-DX35 (CR 700.2b / OOS-DX4-2): compute the modal plan ONCE per trigger
        // and thread it through every consumer below (`trigger_target_requirements`,
        // `ability_targets`, and the `modes_chosen` assignment at the push site) so
        // "one arithmetic" is structural rather than three separately-maintained
        // lookups that can re-diverge. `None` = CR 700.2b removal (no legal mode,
        // `min_modes >= 1`); handled identically to the pre-existing CR 603.3d "no
        // legal target" removal a few lines below.
        let modal_plan = trigger_modal_plan(state, &trigger);
        if modal_plan.is_none() {
            continue;
        }
        let modal_plan = modal_plan.expect("checked is_none above");
        // CR 603.2c/603.2h (PB-AC1): once-per-turn gate. Determine whether this
        // trigger's ability is marked `once_per_turn` (card text "This ability
        // triggers only once each turn"). Look up the layer-resolved runtime
        // TriggeredAbilityDef first (mirrors the target-requirement fallback below);
        // fall back to the card registry definition when the runtime characteristics
        // lookup misses (e.g. PendingTriggerKind other than Normal never populates
        // `characteristics.triggered_abilities`).
        let once_per_turn_flag: bool = {
            let obj = state.objects.get(&trigger.source);
            if let Some(obj) = obj {
                // PB-DX1 review Finding 10: this now genuinely reads layer-resolved
                // characteristics (CR 613.1f), matching `collect_triggers_for_event`'s
                // own namespace, instead of merely claiming to. Was inert either way
                // (an ability-removal effect already suppresses the trigger upstream,
                // at `collect_triggers_for_event`, before a `PendingTrigger` for it is
                // ever created), but phase 7 made a genuine mismatch here load-bearing
                // for three `Complete` defs, so the comment's claim is honored for real.
                let resolved =
                    crate::rules::layers::calculate_characteristics(state, trigger.source)
                        .unwrap_or_else(|| obj.characteristics.clone());
                let from_runtime = if trigger.kind == PendingTriggerKind::Normal {
                    resolved
                        .triggered_abilities
                        .get(trigger.ability_index)
                        .map(|ab| ab.once_per_turn)
                } else {
                    None
                };
                from_runtime.unwrap_or_else(|| {
                    // PB-OS4b (CR 712.8d/e): index into the currently-visible
                    // face's effective list ("is_transformed at consume time"
                    // contract).
                    obj.card_id
                        .as_ref()
                        .and_then(|cid| state.card_registry.get(cid.clone()))
                        .and_then(|def| {
                            def.effective_abilities(obj.is_transformed)
                                .get(trigger.ability_index)
                        })
                        .map(|abil| {
                            matches!(
                                abil,
                                crate::cards::card_definition::AbilityDefinition::Triggered {
                                    once_per_turn: true,
                                    ..
                                }
                            )
                        })
                        .unwrap_or(false)
                })
            } else {
                false
            }
        };
        if once_per_turn_flag {
            // CR 603.2c: skip entirely if the ability already fired this turn — do not
            // put a second instance on the stack, and do not let trigger doublers
            // multiply it.
            let already_fired = state
                .objects
                .get(&trigger.source)
                .map(|o| {
                    o.triggered_abilities_fired_this_turn
                        .contains(&trigger.ability_index)
                })
                .unwrap_or(false);
            if already_fired {
                continue;
            }
        }
        // CR 603.2d: Check for Panharmonicon-style trigger doublers.
        // Compute how many times this trigger fires (1 base + additional from doublers).
        // CR 603.2h (PB-AC1): a once-per-turn ability is never multiplied by doublers —
        // it goes on the stack exactly once.
        let additional_count = if once_per_turn_flag {
            0
        } else {
            compute_trigger_doubling(state, &trigger)
        };
        // CR 702.21a: For Ward triggers, the targeting stack object ID is carried
        // through PendingTrigger.targeting_stack_id. Set it as the triggered
        // ability's target so CounterSpell resolution can find the right stack entry.
        // CR 603.2 / CR 102.2: For OpponentCastsSpell triggers, the casting player
        // is set as Target::Player at index 0 so DeclaredTarget { index: 0 } resolves
        // to the specific opponent who cast the spell (e.g. Rhystic Study resolution).
        //
        // PB-EF3 (CR 601.2c/603.3d): a real CardDef-declared TargetRequirement takes
        // priority over the `defending_player_id` / `exalted_attacker_id` single-slot
        // shortcuts below. Those shortcuts exist for triggers whose effect resolves via
        // an implicit DeclaredTarget{0} and which never declare a real target
        // requirement (annihilator, dethrone, training all carry `targets: vec![]`).
        // Since PB-EF3 B1 now tags EVERY `AnyCreatureYouControlAttacks` trigger with
        // `defending_player_id` (not just annihilator-style ones), a card with a real
        // declared target (e.g. Ojutai, Soul of Winter's "tap target nonland permanent")
        // must not have that target silently replaced by the defending player. This is a
        // cheap presence check (not the full auto-select below); does not change behavior
        // for any existing trigger, since every current defending_player_id/exalted_attacker_id
        // user declares `targets: vec![]`.
        //
        // PB-DX25c §3.1: this is also the CardDef-declared `TargetRequirement` list
        // recorded onto the stack object (line ~9451) — `has_ability_targets` just
        // below is its emptiness check, not a second, independent lookup.
        //
        // PB-DX35 (CR 700.2b/700.2c / OOS-DX4-2): this used to read the FLAT
        // `targets` list unconditionally, ignoring `ModeSelection.mode_targets`
        // entirely — a modal triggered ability's requirement is now
        // `modal_plan.requirements`, the chosen mode's slice (or the flat list
        // when the ability isn't modal / has no per-mode targets — see
        // `trigger_modal_plan`'s arms, which reproduce this exact lookup byte for
        // byte for every non-repaired corpus def).
        let trigger_target_requirements: Vec<crate::cards::card_definition::TargetRequirement> =
            modal_plan.requirements.clone();
        let has_ability_targets = !trigger_target_requirements.is_empty();
        // Returns None if a required target cannot be satisfied (trigger skipped per CR 603.3d).
        let trigger_targets_opt: Option<Vec<SpellTarget>> =
            if let Some(tsid) = trigger.targeting_stack_id {
                Some(vec![SpellTarget {
                    target: Target::Object(tsid),
                    zone_at_cast: None,
                }])
            } else if let Some(pid) = trigger.triggering_player {
                Some(vec![SpellTarget {
                    target: Target::Player(pid),
                    zone_at_cast: None,
                }])
            } else if let Some(dp) = trigger
                .defending_player_id
                .filter(|_| !has_ability_targets)
                .filter(|_| {
                    // PB-EF3 fix (review Finding 2): this shortcut exists ONLY for the
                    // annihilator/dethrone/training/afflict keyword-derived triggers,
                    // whose CardDef-generated effects read the defending player via
                    // `PlayerTarget::DeclaredTarget { index: 0 }` (annihilator's
                    // SacrificePermanents, afflict's LoseLife) or simply had it tagged
                    // for consistency (dethrone/training put a counter on the source
                    // and never read index 0). B1 (PB-EF3) now tags EVERY
                    // `AnyCreatureYouControlAttacks` trigger with `defending_player_id`
                    // too, but those triggers' effects (token creation, life gain,
                    // `EffectTarget::AttackTarget` damage — Utvara Hellkite, Dromoka,
                    // Hellrider, Raid Bombardment) never consume `DeclaredTarget{0}`:
                    // they read `ctx.defending_player` directly via
                    // `PlayerTarget::DefendingPlayer` / `EffectTarget::AttackTarget`,
                    // which do not depend on `stack_obj.targets`. Setting a spurious
                    // `Target::Player(dp)` on their stack object wrongly fizzles the
                    // WHOLE (non-targeted) ability if `dp` leaves the game before it
                    // resolves — CR 608.2b's "all targets illegal" fizzle applies only
                    // to a targeted ability. Restrict the shortcut to the four
                    // keyword-family trigger events so the new AttackTarget/
                    // DefendingPlayer-based cards are unaffected.
                    matches!(
                        trigger.triggering_event,
                        Some(TriggerEvent::SelfAttacks)
                            | Some(TriggerEvent::SelfAttacksPlayerWithMostLife)
                            | Some(TriggerEvent::SelfAttacksWithGreaterPowerAlly)
                            | Some(TriggerEvent::SelfBecomesBlocked)
                    )
                })
            {
                // CR 702.86a / CR 508.5: Annihilator triggers carry the defending player ID.
                // Set as Target::Player at index 0 so PlayerTarget::DeclaredTarget { index: 0 }
                // resolves to the correct defending player for the SacrificePermanents effect.
                Some(vec![SpellTarget {
                    target: Target::Player(dp),
                    zone_at_cast: None,
                }])
            } else if let Some(attacker_id) =
                trigger.exalted_attacker_id.filter(|_| !has_ability_targets)
            {
                // CR 702.83a: Exalted triggers carry the lone attacker's ObjectId.
                // Set it as Target::Object at index 0 so CEFilter::DeclaredTarget { index: 0 }
                // resolves to the attacking creature (not the exalted source permanent).
                Some(vec![SpellTarget {
                    target: Target::Object(attacker_id),
                    zone_at_cast: None,
                }])
            } else if trigger.kind == PendingTriggerKind::Provoke {
                // CR 702.39a: Provoke triggers target the provoked creature.
                // Set it as Target::Object so target legality can be checked at resolution.
                let provoked = match &trigger.data {
                    Some(TriggerData::CombatProvoke { target }) => Some(*target),
                    _ => None,
                };
                if let Some(provoked) = provoked {
                    Some(vec![SpellTarget {
                        target: Target::Object(provoked),
                        zone_at_cast: Some(ZoneId::Battlefield),
                    }])
                } else {
                    Some(vec![])
                }
            } else if matches!(
                trigger.kind,
                PendingTriggerKind::Normal | PendingTriggerKind::CardDefETB
            ) {
                // CR 603.3d: For CardDef-based triggered abilities (Normal / CardDefETB),
                // look up the target requirements from the ability definition and
                // auto-select legal targets using deterministic first-match fallback.
                // If any required target has no legal candidate, skip this trigger.
                //
                // PB-DX35 (CR 700.2b/700.2c / OOS-DX4-2): identical value to
                // `trigger_target_requirements` above — `modal_plan` is computed once
                // per trigger and threaded through every consumer so the two cannot
                // re-diverge into separate hand-rolled copies.
                let ability_targets: Vec<crate::cards::card_definition::TargetRequirement> =
                    modal_plan.requirements.clone();
                if ability_targets.is_empty() {
                    // No targets required — proceed normally with empty targets.
                    Some(vec![])
                } else {
                    // CR 603.3d / CR 601.2c (PB-DP8 / DP-6): the controller ANNOUNCES
                    // the targets. Derive every legal choice per slot with the same
                    // predicates the pre-PB-DP8 first-match auto-pick used, then
                    // decide whether a question is owed.
                    let mut slots: Vec<TriggerTargetOption> = ability_targets
                        .iter()
                        .map(|req| trigger_target_candidates(state, &trigger, req))
                        .collect();
                    // CR 601.2c (closing-review Finding 3, LOW): a per-slot default is
                    // computed in isolation, so two mutually-distinct slots both got
                    // `candidates.first()` -- an answer the engine's own cross-slot
                    // check rejects. Reconcile them before anything can submit it.
                    make_distinct_slot_defaults(&ability_targets, &mut slots);
                    // CR 603.3d: "if a choice is required when the triggered ability
                    // goes on the stack but no legal choices can be made for it ...
                    // the ability is simply removed from the stack." An `optional`
                    // (CR 601.2c "up to") slot always has a legal choice -- zero
                    // targets -- so only a REQUIRED slot with an empty candidate set
                    // removes the trigger.
                    if slots.iter().any(|s| !s.optional && s.candidates.is_empty()) {
                        None
                    } else if let Some(pre) = this_head {
                        // CR 603.3d resume: this is the head of a suspended batch and
                        // its controller has already answered. Every LATER trigger in
                        // the batch derives its own targets at its own turn, which is
                        // what CR 603.3d requires ("as it goes on the stack").
                        Some(pre)
                    } else if let Some(per_slot) = forced_trigger_target_answer(&slots) {
                        // CR 601.2c: one legal answer is not a choice.
                        if forced_answer_breaks_distinctness(&ability_targets, &per_slot) {
                            // CR 603.3d: "if a choice is required when the triggered
                            // ability goes on the stack but no legal choices can be
                            // made for it ... the ability is simply removed from the
                            // stack." Every slot is determined AND the combination is
                            // illegal, so there is no legal announcement -- the
                            // constraint has no solution, not the candidate sets.
                            // Asking would be a question with no acceptable answer;
                            // placing it anyway (what this path used to do) is a
                            // silent CR 601.2c violation. Second closing review,
                            // Finding 2 (LOW); zero corpus exposure (OOS-DP8-4).
                            None
                        } else {
                            Some(flatten_slot_answers(&slots, &per_slot))
                        }
                    } else if !state
                        .expect_player(trigger.controller)
                        .map(|pl| !pl.has_lost && !pl.has_conceded)
                        .unwrap_or(false)
                    {
                        // CR 800.4d neighbourhood: never ask a player who has left the
                        // game -- nobody could answer and the game would hang. Use the
                        // engine's own default, i.e. today's behaviour unchanged.
                        // (Actually DROPPING the trigger per CR 800.4d is a behaviour
                        // flip this batch is not chartered to make: seed OOS-DP8-5.)
                        Some(default_spell_targets(&slots))
                    } else {
                        // Suspend the CR 603.3b batch. The entry owns this trigger AND
                        // the un-flushed tail; `handle_choose_trigger_targets` resumes.
                        let choice_id = state.next_choice_id();
                        let ability_index = trigger.ability_index;
                        let source = trigger.source;
                        let player = trigger.controller;
                        let remaining: imbl::Vector<PendingTrigger> =
                            sorted[next_index..].iter().cloned().collect();
                        events.push(GameEvent::TriggerTargetChoiceRequired {
                            player,
                            choice_id,
                            source_object_id: source,
                            ability_index,
                            slots: slots.clone(),
                        });
                        state.pending_trigger_targets = Some(PendingTriggerTargets {
                            choice_id,
                            player,
                            source,
                            trigger: trigger.clone(),
                            remaining,
                            slots: slots.into_iter().collect(),
                            // Set by the caller's guard if this call site owed
                            // anything (see `mark_flush_resume_site`).
                            resume_site: FlushResumeSite::None,
                        });
                        // CR 117.3d (fix-cycle Finding 10): putting a triggered ability
                        // on the stack is a game action, and the function's tail resets
                        // the pass count for exactly that reason. The suspend return
                        // skips that tail, so do it here for whatever this partial
                        // batch already placed. `events` also holds the question, which
                        // is why the flag rather than `!events.is_empty()` is the test.
                        if placed_any {
                            state.turn.players_passed = OrdSet::new();
                        }
                        return events;
                    }
                }
            } else {
                Some(vec![])
            };
        // CR 603.3d: If trigger_targets_opt is None, no legal target exists — skip trigger.
        let trigger_targets = match trigger_targets_opt {
            Some(t) => t,
            None => continue,
        };
        // Push the triggered ability onto the stack (1 + additional_count) times.
        for _ in 0..=(additional_count) {
            let stack_id = state.next_object_id();
            // CR 702.74a: Evoke sacrifice triggers use EvokeSacrificeTrigger kind
            // instead of TriggeredAbility to distinguish them at resolution time.
            // CR 702.35a: Madness triggers use MadnessTrigger kind to carry
            // the exiled card ObjectId and madness cost for resolution.
            let kind = match trigger.kind {
                PendingTriggerKind::Evoke => StackObjectKind::KeywordTrigger {
                    source_object: trigger.source,
                    keyword: KeywordAbility::Evoke,
                    data: TriggerData::DelayedZoneChange,
                },
                PendingTriggerKind::Madness => {
                    let (exiled_card, madness_cost) = match &trigger.data {
                        Some(TriggerData::Madness { exiled_card, cost }) => {
                            (*exiled_card, cost.clone())
                        }
                        _ => (trigger.source, Default::default()),
                    };
                    StackObjectKind::MadnessTrigger {
                        source_object: trigger.source,
                        exiled_card,
                        madness_cost,
                        owner: trigger.controller,
                    }
                }
                PendingTriggerKind::Miracle => {
                    // CR 702.94a: Miracle trigger carries the revealed card and cost.
                    let (revealed_card, miracle_cost) = match &trigger.data {
                        Some(TriggerData::Miracle {
                            revealed_card,
                            cost,
                        }) => (*revealed_card, cost.clone()),
                        _ => (trigger.source, Default::default()),
                    };
                    StackObjectKind::MiracleTrigger {
                        source_object: trigger.source,
                        revealed_card,
                        miracle_cost,
                        owner: trigger.controller,
                    }
                }
                PendingTriggerKind::Unearth => {
                    // CR 702.84a: Unearth delayed exile trigger -- "Exile [this permanent]
                    // at the beginning of the next end step."
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Unearth,
                        data: TriggerData::DelayedZoneChange,
                    }
                }
                PendingTriggerKind::Exploit => {
                    // CR 702.110a: Exploit ETB trigger -- "When this creature enters,
                    // you may sacrifice a creature."
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Exploit,
                        data: TriggerData::Simple,
                    }
                }
                PendingTriggerKind::Modular => {
                    // CR 702.43a: Modular dies trigger -- "you may put a +1/+1 counter
                    // on target artifact creature for each +1/+1 counter on this permanent."
                    // Deterministic target selection: first artifact creature on the
                    // battlefield by ObjectId ascending (OrdMap is sorted by key).
                    // CR 603.3d: If no legal artifact creature target exists, the trigger
                    // is not placed on the stack. Use `continue` to skip this trigger.
                    // CR 613.1d: Use layer-resolved types for artifact creature check
                    // (animated artifacts are creatures; type-changing effects apply).
                    let target_id = state
                        .objects
                        .iter()
                        .find(|(id, obj)| {
                            obj.zone == ZoneId::Battlefield && obj.is_phased_in() && {
                                let chars =
                                    crate::rules::layers::expect_characteristics(state, **id);
                                chars.card_types.contains(&CardType::Artifact)
                                    && chars.card_types.contains(&CardType::Creature)
                            }
                        })
                        .map(|(id, _)| *id);
                    let Some(tid) = target_id else {
                        // No legal artifact creature target -- skip this trigger (CR 603.3d).
                        continue;
                    };
                    // Override trigger_targets with the selected artifact creature target.
                    // (trigger_targets computed above does not apply to modular triggers.)
                    let modular_targets = vec![SpellTarget {
                        target: Target::Object(tid),
                        zone_at_cast: Some(ZoneId::Battlefield),
                    }];
                    let counter_count = match trigger.data {
                        Some(TriggerData::DeathModular { counter_count }) => counter_count,
                        _ => 0,
                    };
                    let stack_id = state.next_object_id();
                    // MR-TC-25: use trigger_default; override targets with modular target.
                    let mut stack_obj = StackObject::trigger_default(
                        stack_id,
                        trigger.controller,
                        StackObjectKind::KeywordTrigger {
                            source_object: trigger.source,
                            keyword: KeywordAbility::Modular(counter_count),
                            data: TriggerData::DeathModular { counter_count },
                        },
                    );
                    stack_obj.targets = modular_targets;
                    // PB-DX25c §3.1: Modular's "target artifact creature" is chosen by the
                    // deterministic auto-target scan above, not validated through a
                    // `TargetRequirement` — there is no formal requirement to record.
                    state.stack_objects.push_back(stack_obj);
                    events.push(GameEvent::AbilityTriggered {
                        controller: trigger.controller,
                        source_object_id: trigger.source,
                        stack_object_id: stack_id,
                    });
                    // ENG-2 (T6, CR 603.3d): announce the modular trigger's target.
                    super::events::push_target_announcement(
                        state,
                        &mut events,
                        trigger.controller,
                        trigger.source,
                        stack_id,
                    );
                    // For trigger doubling: already handled via additional_count loop below,
                    // but modular uses an early-exit path above. We run additional_count
                    // copies too. However, for simplicity and correctness, break out of the
                    // per-duplication loop by skipping the rest. The doubler case is handled
                    // after the if-else chain below -- but since we already pushed the stack
                    // object and emitted the event, we must NOT fall through to the bottom
                    // of the loop. Use a labeled continue to advance to the next trigger.
                    // NOTE: trigger doubling (Panharmonicon) is not applicable to non-ETB
                    // triggers, so additional_count will always be 0 here.
                    continue;
                }
                PendingTriggerKind::Evolve => {
                    // CR 702.100a: Evolve ETB trigger — "Whenever a creature you control
                    // enters, if that creature's P > this creature's P and/or that creature's
                    // T > this creature's T, put a +1/+1 counter on this creature."
                    // The resolution handler re-checks the intervening-if (CR 603.4).
                    let entering_creature = match trigger.data {
                        Some(TriggerData::ETBEvolve { entering_creature }) => entering_creature,
                        _ => trigger.source,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Evolve,
                        data: TriggerData::ETBEvolve { entering_creature },
                    }
                }
                PendingTriggerKind::Myriad => {
                    // CR 702.116a: Myriad SelfAttacks trigger -- "Whenever this creature
                    // attacks, for each opponent other than defending player, create a token
                    // copy tapped and attacking that player."
                    // The `defending_player_id` was tagged by the AttackersDeclared handler
                    // in check_triggers. Fallback to active player if somehow None.
                    let defending = trigger
                        .defending_player_id
                        .unwrap_or(state.turn.active_player);
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Myriad,
                        data: TriggerData::MyriadAttack {
                            defending_player: defending,
                        },
                    }
                }
                PendingTriggerKind::SuspendCounter => {
                    // CR 702.62a: Suspend upkeep counter-removal trigger.
                    let suspended_card = match trigger.data {
                        Some(TriggerData::Suspend { card }) => card,
                        _ => trigger.source,
                    };
                    StackObjectKind::SuspendCounterTrigger {
                        source_object: trigger.source,
                        suspended_card,
                    }
                }
                PendingTriggerKind::SuspendCast => {
                    // CR 702.62a: Suspend cast trigger (last time counter removed).
                    let suspended_card = match trigger.data {
                        Some(TriggerData::Suspend { card }) => card,
                        _ => trigger.source,
                    };
                    StackObjectKind::SuspendCastTrigger {
                        source_object: trigger.source,
                        suspended_card,
                        owner: trigger.controller,
                    }
                }
                PendingTriggerKind::Hideaway => {
                    // CR 702.75a: Hideaway ETB trigger — "When this permanent enters,
                    // look at the top N cards of your library. Exile one of them face
                    // down and put the rest on the bottom of your library in a random order."
                    let hide_count = match trigger.data {
                        Some(TriggerData::ETBHideaway { count }) => count,
                        _ => 4,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Hideaway(hide_count),
                        data: TriggerData::ETBHideaway { count: hide_count },
                    }
                }
                PendingTriggerKind::PartnerWith => {
                    // CR 702.124j: Partner With ETB trigger — "When this permanent enters,
                    // target player may search their library for a card named [name], reveal
                    // it, put it into their hand, then shuffle."
                    // Target player: deterministic fallback = the trigger controller (owner).
                    let (partner_name, target_player) = match &trigger.data {
                        Some(TriggerData::ETBPartnerWith {
                            partner_name,
                            target_player,
                        }) => (partner_name.clone(), *target_player),
                        _ => (String::new(), trigger.controller),
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::PartnerWith(partner_name.clone()),
                        data: TriggerData::ETBPartnerWith {
                            partner_name,
                            target_player,
                        },
                    }
                }
                PendingTriggerKind::Ingest => {
                    // CR 702.115a: Ingest combat damage trigger — "Whenever this creature
                    // deals combat damage to a player, that player exiles the top card of
                    // their library."
                    let target_player = match &trigger.data {
                        Some(TriggerData::IngestExile { target_player }) => *target_player,
                        _ => trigger.controller,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Ingest,
                        data: TriggerData::IngestExile { target_player },
                    }
                }
                PendingTriggerKind::Flanking => {
                    let blocker = match &trigger.data {
                        Some(TriggerData::CombatFlanking { blocker }) => *blocker,
                        _ => trigger.source,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Flanking,
                        data: TriggerData::CombatFlanking { blocker },
                    }
                }
                PendingTriggerKind::Rampage => {
                    let n = match &trigger.data {
                        Some(TriggerData::CombatRampage { n }) => *n,
                        _ => 1,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Rampage(n),
                        data: TriggerData::CombatRampage { n },
                    }
                }
                PendingTriggerKind::Provoke => {
                    if let Some(TriggerData::CombatProvoke { target: provoked }) = trigger.data {
                        StackObjectKind::KeywordTrigger {
                            source_object: trigger.source,
                            keyword: KeywordAbility::Provoke,
                            data: TriggerData::CombatProvoke { target: provoked },
                        }
                    } else {
                        continue;
                    }
                }
                PendingTriggerKind::Renown => {
                    let n = match &trigger.data {
                        Some(TriggerData::RenownDamage { n }) => *n,
                        _ => 1,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Renown(n),
                        data: TriggerData::RenownDamage { n },
                    }
                }
                PendingTriggerKind::Melee => StackObjectKind::KeywordTrigger {
                    source_object: trigger.source,
                    keyword: KeywordAbility::Melee,
                    data: TriggerData::Simple,
                },
                PendingTriggerKind::Poisonous => {
                    let (target_player, n) = match &trigger.data {
                        Some(TriggerData::CombatPoisonous { target_player, n }) => {
                            (*target_player, *n)
                        }
                        _ => (trigger.controller, 1),
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Poisonous(n),
                        data: TriggerData::CombatPoisonous { target_player, n },
                    }
                }
                PendingTriggerKind::Enlist => {
                    let enlisted = match &trigger.data {
                        Some(TriggerData::CombatEnlist { enlisted }) => *enlisted,
                        _ => trigger.source,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Enlist,
                        data: TriggerData::CombatEnlist { enlisted },
                    }
                }
                PendingTriggerKind::EncoreSacrifice => {
                    // CR 702.141a: Encore delayed sacrifice trigger -- "Sacrifice them
                    // at the beginning of the next end step."
                    let activator = match trigger.data {
                        Some(TriggerData::EncoreSacrifice { activator }) => activator,
                        _ => trigger.controller,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Encore,
                        data: TriggerData::EncoreSacrifice { activator },
                    }
                }
                PendingTriggerKind::DashReturn => {
                    // CR 702.109a: Dash delayed return trigger -- "return the permanent to
                    // its owner's hand at the beginning of the next end step."
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Dash,
                        data: TriggerData::DelayedZoneChange,
                    }
                }
                PendingTriggerKind::BlitzSacrifice => {
                    // CR 702.152a: Blitz delayed sacrifice trigger -- "sacrifice the
                    // permanent at the beginning of the next end step."
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Blitz,
                        data: TriggerData::DelayedZoneChange,
                    }
                }
                // ImpendingCounter: migrated to KeywordTrigger
                // VanishingCounter and VanishingSacrifice: migrated to KeywordTrigger
                // FadingUpkeep: migrated to KeywordTrigger
                // EchoUpkeep: migrated to KeywordTrigger
                // CumulativeUpkeep: migrated to KeywordTrigger
                PendingTriggerKind::Recover => {
                    // CR 702.59a: Recover trigger — data carries DeathRecover.
                    let (recover_card, recover_cost) = match trigger.data.clone() {
                        Some(TriggerData::DeathRecover {
                            recover_card,
                            recover_cost,
                        }) => (recover_card, recover_cost),
                        _ => (trigger.source, Default::default()),
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Recover,
                        data: TriggerData::DeathRecover {
                            recover_card,
                            recover_cost,
                        },
                    }
                }
                PendingTriggerKind::Graft => {
                    // CR 702.58a: Graft trigger.
                    // "Whenever another creature enters, if this permanent has a +1/+1
                    // counter on it, you may move a +1/+1 counter from this permanent
                    // onto that creature."
                    let entering_creature = match trigger.data {
                        Some(TriggerData::ETBGraft { entering_creature }) => entering_creature,
                        _ => trigger.source,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Graft(0),
                        data: TriggerData::ETBGraft { entering_creature },
                    }
                }
                PendingTriggerKind::Backup => {
                    // CR 702.165a: Backup ETB trigger.
                    // Default target = self (gets counters but no abilities per CR 702.165a).
                    // In real play the controller chooses; deterministic default = source.
                    let source = trigger.source;
                    let (target, count, abilities) = match &trigger.data {
                        Some(TriggerData::ETBBackup {
                            target,
                            count,
                            abilities,
                        }) => (*target, *count, abilities.clone()),
                        _ => (source, 1, vec![]),
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: source,
                        keyword: KeywordAbility::Backup(count),
                        data: TriggerData::ETBBackup {
                            target,
                            count,
                            // Self-targeting: no abilities granted (CR 702.165a "if that's another creature").
                            abilities: if target == source { vec![] } else { abilities },
                        },
                    }
                }
                PendingTriggerKind::ChampionETB => {
                    // CR 702.72a: Champion ETB trigger.
                    let filter = match &trigger.data {
                        Some(TriggerData::ETBChampion { filter }) => filter.clone(),
                        _ => ChampionFilter::AnyCreature,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Champion,
                        data: TriggerData::ETBChampion { filter },
                    }
                }
                PendingTriggerKind::ChampionLTB => {
                    // CR 702.72a: Champion LTB trigger.
                    let exiled_card = match trigger.data {
                        Some(TriggerData::LTBChampion { exiled_card }) => exiled_card,
                        _ => trigger.source,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Champion,
                        data: TriggerData::LTBChampion { exiled_card },
                    }
                }
                PendingTriggerKind::SoulbondSelfETB | PendingTriggerKind::SoulbondOtherETB => {
                    // CR 702.95a: Soulbond ETB triggers (self-ETB and other-ETB).
                    // source = soulbond creature; pair_target = the creature to pair with.
                    let pair_target = match trigger.data {
                        Some(TriggerData::ETBSoulbond { pair_target }) => pair_target,
                        _ => trigger.source,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Soulbond,
                        data: TriggerData::ETBSoulbond { pair_target },
                    }
                }
                PendingTriggerKind::RavenousDraw => {
                    // CR 702.156a: Ravenous draw trigger. Read x_value from the GameObject
                    // (stored at ETB time per CR 107.3m). Intervening-if re-check happens
                    // at resolution.
                    let x_value = state
                        .objects
                        .get(&trigger.source)
                        .map(|o| o.x_value)
                        .unwrap_or(0);
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Ravenous,
                        data: TriggerData::ETBRavenousDraw {
                            permanent: trigger.source,
                            x_value,
                        },
                    }
                }
                PendingTriggerKind::SquadETB => {
                    // CR 702.157a: Squad ETB trigger. Read squad_count from trigger.data.
                    let count = match trigger.data {
                        Some(TriggerData::ETBSquad { count }) => count,
                        _ => 0,
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Squad,
                        data: TriggerData::ETBSquad { count },
                    }
                }
                PendingTriggerKind::OffspringETB => {
                    // CR 702.175a: Offspring ETB trigger. The source_object is the creature
                    // that entered with offspring cost paid. At resolution, creates 1 token
                    // copy except it's 1/1. Uses LKI if source has left the battlefield.
                    // Capture source_card_id now (while source is on battlefield) for LKI
                    // fallback at resolution time (ruling 2024-07-26).
                    let source_card_id = state
                        .objects
                        .get(&trigger.source)
                        .and_then(|o| o.card_id.clone());
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Offspring,
                        data: TriggerData::ETBOffspring { source_card_id },
                    }
                }
                PendingTriggerKind::GiftETB => {
                    // CR 702.174b: Gift ETB trigger. Data is ETBGift captured at queue time.
                    let (source_card_id, gift_opponent) = match &trigger.data {
                        Some(TriggerData::ETBGift {
                            source_card_id,
                            gift_opponent,
                        }) => (source_card_id.clone(), *gift_opponent),
                        _ => {
                            // No gift opponent — skip this trigger (should not happen).
                            continue;
                        }
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Gift,
                        data: TriggerData::ETBGift {
                            source_card_id,
                            gift_opponent,
                        },
                    }
                }
                PendingTriggerKind::CipherCombatDamage => {
                    // CR 702.99a: Cipher combat damage trigger — the encoded card info is
                    // carried in trigger.data as TriggerData::CipherDamage.
                    match trigger.data.clone() {
                        Some(TriggerData::CipherDamage {
                            source_creature,
                            encoded_card_id,
                            encoded_object_id,
                        }) => StackObjectKind::KeywordTrigger {
                            source_object: trigger.source,
                            keyword: KeywordAbility::Cipher,
                            data: TriggerData::CipherDamage {
                                source_creature,
                                encoded_card_id,
                                encoded_object_id,
                            },
                        },
                        _ => continue, // Missing data — skip (should not happen).
                    }
                }
                PendingTriggerKind::HauntExile => {
                    // CR 702.55a: Haunt exile trigger — data carries DeathHauntExile.
                    match trigger.data.clone() {
                        Some(TriggerData::DeathHauntExile {
                            haunt_card,
                            haunt_card_id,
                        }) => StackObjectKind::KeywordTrigger {
                            source_object: trigger.source,
                            keyword: KeywordAbility::Haunt,
                            data: TriggerData::DeathHauntExile {
                                haunt_card,
                                haunt_card_id,
                            },
                        },
                        _ => StackObjectKind::KeywordTrigger {
                            source_object: trigger.source,
                            keyword: KeywordAbility::Haunt,
                            data: TriggerData::DeathHauntExile {
                                haunt_card: trigger.source,
                                haunt_card_id: None,
                            },
                        },
                    }
                }
                PendingTriggerKind::HauntedCreatureDies => {
                    // CR 702.55c: Haunted creature dies trigger — data carries DeathHauntedCreatureDies.
                    match trigger.data.clone() {
                        Some(TriggerData::DeathHauntedCreatureDies {
                            haunt_source,
                            haunt_card_id,
                        }) => {
                            // CR 603.4 (PB-DX1 review Finding 7 / OOS-DP6-9): the
                            // queue-time intervening-if gate lives HERE, not in
                            // `check_triggers` — this arm mirrors that function's own
                            // find_map (locate the card-def's `HauntedCreatureDies`
                            // ability, read its `intervening_if`) and gates with the
                            // same `carddef_intervening_if_holds_at_queue_time` every
                            // other queue site uses. It lives at flush time (like
                            // `once_per_turn`, checked earlier in this same function)
                            // specifically because only here is `state` `&mut`, and
                            // CR 702.55c requires clearing `haunting_target` on
                            // suppression: a suppressed trigger still spends the
                            // one-shot haunting relationship (mirroring
                            // `resolution.rs`'s clear, which runs "regardless of
                            // whether the intervening-if held") — otherwise the
                            // exiled card would keep haunting a dead creature's
                            // `ObjectId`, the exact recycled-`ObjectId` hazard that
                            // clear exists to prevent.
                            let intervening_if = haunt_card_id.clone().and_then(|cid| {
                                state.card_registry.get(cid).and_then(|def| {
                                    def.abilities.iter().find_map(|ab| {
                                        if let AbilityDefinition::Triggered {
                                            trigger_condition:
                                                TriggerCondition::HauntedCreatureDies,
                                            intervening_if,
                                            ..
                                        } = ab
                                        {
                                            Some(intervening_if.clone())
                                        } else {
                                            None
                                        }
                                    })
                                })
                            });
                            let gate_holds = intervening_if
                                .map(|cond| {
                                    carddef_intervening_if_holds_at_queue_time(
                                        state,
                                        cond.as_ref(),
                                        trigger.controller,
                                        haunt_source,
                                    )
                                })
                                // No card def / no matching ability found: nothing to gate on.
                                .unwrap_or(true);
                            if !gate_holds {
                                if let Some(haunt_obj) = state.fizzle_object_mut(haunt_source) {
                                    haunt_obj.haunting_target = None;
                                }
                                continue;
                            }
                            StackObjectKind::KeywordTrigger {
                                source_object: trigger.source,
                                keyword: KeywordAbility::Haunt,
                                data: TriggerData::DeathHauntedCreatureDies {
                                    haunt_source,
                                    haunt_card_id,
                                },
                            }
                        }
                        _ => StackObjectKind::KeywordTrigger {
                            source_object: trigger.source,
                            keyword: KeywordAbility::Haunt,
                            data: TriggerData::DeathHauntedCreatureDies {
                                haunt_source: trigger.source,
                                haunt_card_id: None,
                            },
                        },
                    }
                }
                // CR 708.8 / CR 702.37e: "When this permanent is turned face up" trigger.
                // The source is the permanent itself; card_id is looked up from the object.
                PendingTriggerKind::TurnFaceUp => {
                    let source_card_id = state
                        .objects
                        .get(&trigger.source)
                        .and_then(|o| o.card_id.clone());
                    StackObjectKind::TurnFaceUpTrigger {
                        permanent: trigger.source,
                        source_card_id,
                        ability_index: trigger.ability_index,
                    }
                }
                // CR 701.54c: Ring level 2 — "Whenever your Ring-bearer attacks,
                // draw a card, then discard a card." (Loot effect for the controller.)
                PendingTriggerKind::RingLoot => {
                    use crate::cards::card_definition::{Effect, EffectAmount, PlayerTarget};
                    StackObjectKind::RingAbility {
                        source_object: trigger.source,
                        effect: Box::new(Effect::Sequence(vec![
                            Effect::DrawCards {
                                player: PlayerTarget::Controller,
                                count: EffectAmount::Fixed(1),
                            },
                            Effect::DiscardCards {
                                player: PlayerTarget::Controller,
                                count: EffectAmount::Fixed(1),
                            },
                        ])),
                        controller: trigger.controller,
                    }
                }
                // PendingTriggerKind::RingBlockSacrifice is retired: the ring level 3
                // EOC sacrifice is now handled via the `ring_block_sacrifice_at_eoc` flag
                // on GameObject, checked in `end_combat()` in turn_actions.rs.
                // This arm is unreachable but kept for exhaustiveness (Rust requires it).
                PendingTriggerKind::RingBlockSacrifice => {
                    // Should never be reached — ring level 3 uses EOC flag pattern now.
                    // Fallback: no-op (empty sequence).
                    use crate::cards::card_definition::Effect;
                    StackObjectKind::RingAbility {
                        source_object: trigger.source,
                        effect: Box::new(Effect::Sequence(vec![])),
                        controller: trigger.controller,
                    }
                }
                // CR 701.54c: Ring level 4 — "Whenever your Ring-bearer deals combat damage
                // to a player, each opponent loses 3 life."
                PendingTriggerKind::RingCombatDamage => {
                    use crate::cards::card_definition::{Effect, EffectAmount, PlayerTarget};
                    StackObjectKind::RingAbility {
                        source_object: trigger.source,
                        effect: Box::new(Effect::LoseLife {
                            player: PlayerTarget::EachOpponent,
                            amount: EffectAmount::Fixed(3),
                        }),
                        controller: trigger.controller,
                    }
                }
                // CR 603.7: Delayed trigger fires — execute the stored action on the target.
                PendingTriggerKind::DelayedAction => {
                    let (action, target) = match trigger.data.clone() {
                        Some(TriggerData::DelayedAction { action, target }) => (action, target),
                        _ => continue, // malformed trigger, skip
                    };
                    StackObjectKind::DelayedActionTrigger {
                        source_object: trigger.source,
                        target,
                        action,
                    }
                }
                PendingTriggerKind::Normal => StackObjectKind::TriggeredAbility {
                    source_object: trigger.source,
                    ability_index: trigger.ability_index,
                    is_carddef_etb: false,
                    // MR-B12-04: carry the effect captured at trigger-queue time so
                    // resolution can run it even if the source has since left its zone.
                    embedded_effect: trigger.embedded_effect.clone().map(Box::new),
                },
                // CR 603.3: Card-definition ETB triggers use CardDefETB kind.
                // ability_index is into CardDef::abilities, NOT runtime triggered_abilities.
                // At resolution, always use the card registry path.
                PendingTriggerKind::CardDefETB => StackObjectKind::TriggeredAbility {
                    source_object: trigger.source,
                    ability_index: trigger.ability_index,
                    is_carddef_etb: true,
                    // CardDefETB resolves via the card registry (ability_index into
                    // CardDef::abilities) — no embedded effect needed.
                    embedded_effect: None,
                },
                PendingTriggerKind::KeywordTrigger {
                    ref keyword,
                    ref data,
                } => StackObjectKind::KeywordTrigger {
                    source_object: trigger.source,
                    keyword: keyword.clone(),
                    data: data.clone(),
                },
            };
            // MR-TC-25: use trigger_default; override targets if non-empty.
            let mut stack_obj = StackObject::trigger_default(stack_id, trigger.controller, kind);
            stack_obj.targets = trigger_targets.clone();
            // PB-DX25c §3.1: `trigger_target_requirements` (computed above, the same
            // lookup `has_ability_targets` used) whenever this trigger's kind is
            // Normal/CardDefETB. Residual, stated rather than glossed: the
            // engine-internal Ward (`targeting_stack_id`) and OpponentCastsSpell
            // (`triggering_player`) shortcuts take precedence UNCONDITIONALLY above
            // and could in principle fire alongside a CardDef-declared requirement,
            // in which case this list would describe a requirement that did not
            // actually govern `trigger_targets` — no corpus trigger combines both
            // today, and it is moot regardless: `rules::retarget` can never reach an
            // ability-kind stack object at all (§8 R2 of the plan,
            // `stack_index_for_announced_target` only resolves card-owning kinds).
            stack_obj.target_requirements = trigger_target_requirements.clone();
            // CR 510.3a / CR 608.2h: Propagate combat AND combat-or-noncombat
            // damage data from PendingTrigger to StackObject so resolution.rs
            // can populate EffectContext correctly.
            stack_obj.damaged_player = trigger.damaged_player;
            stack_obj.combat_damage_amount = trigger.combat_damage_amount;
            stack_obj.damage_dealt_amount = trigger.damage_dealt_amount;
            // The entering_object_id carries the dealing creature for per-creature triggers.
            stack_obj.triggering_creature_id = trigger.entering_object_id;
            // CR 508.4: Propagate the defending player captured at attack-trigger dispatch
            // (PB-EF3 B1) from PendingTrigger to StackObject so resolution.rs can build
            // EffectContext.defending_player.
            stack_obj.defending_player = trigger.defending_player_id;
            // CR 603.10a / CR 113.7a: Propagate LKI counter snapshot from PendingTrigger
            // to StackObject so resolution.rs can build EffectContext.lki_counters.
            stack_obj.lki_counters = trigger.lki_counters.clone();
            // CR 603.10a / CR 113.7a: Propagate LKI source-power snapshot from PendingTrigger
            // to StackObject so resolution.rs can build EffectContext.lki_power.
            stack_obj.lki_power = trigger.lki_power; // Option<i32> is Copy
                                                     // CR 700.2b (PB-DX35, `OOS-DX4-2`): choose modes when the trigger is
                                                     // put on the stack, from the SAME `modal_plan` computed once at the
                                                     // top of this trigger's loop iteration and already threaded into
                                                     // `trigger_target_requirements`/`ability_targets` above -- "one
                                                     // arithmetic" is structural, not three independently-computed copies
                                                     // that can re-diverge. The controller still does not choose
                                                     // (`decision_site_walk`'s `modal_trigger` row stays `AutoChosen`,
                                                     // execution-notes §0.3): the automatic choice is now CR 700.2b-legal
                                                     // (it will not pick a mode with no legal target) instead of always
                                                     // picking mode 0.
            if matches!(stack_obj.kind, StackObjectKind::TriggeredAbility { .. }) {
                stack_obj.modes_chosen = modal_plan.modes_chosen.clone();
            }
            state.stack_objects.push_back(stack_obj);
            placed_any = true;
            events.push(GameEvent::AbilityTriggered {
                controller: trigger.controller,
                source_object_id: trigger.source,
                stack_object_id: stack_id,
            });
            // ENG-2 (T7, CR 603.3d): announce the triggered ability's targets, if
            // any -- this is the reported defect's site (the Fell Specter class).
            // Covers both the auto-default and the PB-DP8 human-answered path,
            // since both flush_pending_triggers and resume_trigger_flush call
            // flush_sorted.
            super::events::push_target_announcement(
                state,
                &mut events,
                trigger.controller,
                trigger.source,
                stack_id,
            );
        }
        // CR 603.2c/603.2h (PB-AC1): mark this once-per-turn ability as fired now that
        // it has been put on the stack (exactly once, per the additional_count == 0
        // override above).
        if once_per_turn_flag {
            // CR 113.7a: the trigger source may have left before its trigger flushed; use LKI.
            if let Some(obj) = state.fizzle_object_mut(trigger.source) {
                obj.triggered_abilities_fired_this_turn
                    .insert(trigger.ability_index);
            }
        }
    }
    if placed_any {
        // Triggers going on the stack is a game action — reset priority pass count.
        state.turn.players_passed = OrdSet::new();
    }
    events
}
/// CR 603.3d (PB-DP8): continue a suspended batch once its head trigger's
/// controller has announced targets.
///
/// Called only by `handle_choose_trigger_targets`, after every validation.
pub(crate) fn resume_trigger_flush(
    state: &mut GameState,
    chosen: Vec<SpellTarget>,
) -> Vec<GameEvent> {
    let entry = match state.pending_trigger_targets.take() {
        Some(e) => e,
        // The caller validates the entry exists; a `None` here is an engine bug,
        // not a rules-correct fizzle (SR-4).
        None => {
            debug_assert!(false, "resume_trigger_flush with no outstanding entry");
            return Vec::new();
        }
    };
    let owed = entry.resume_site;
    let mut sorted = vec![entry.trigger];
    sorted.extend(entry.remaining.iter().cloned());
    let mut events = flush_sorted(state, sorted, Some(chosen));
    finish_resumed_flush(state, owed, &mut events);
    // CR 800.4 (closing-review Finding 1, HIGH): a concede that happened WHILE this
    // batch was suspended left its priority-advance to us -- see
    // `rules::engine::handle_concede`'s gate and the doc comment below.
    repair_departed_priority_holder(state, &mut events);
    events
}
/// CR 603.3 / CR 117.3a (PB-DP8): after a suspended batch resumes, either carry
/// the priority obligation forward (the batch suspended again on a later trigger)
/// or discharge it.
///
/// The guarded call sites were each about to do something when the flush
/// suspended; all of the priority grants converge on "the active player receives
/// priority" (`combat.rs`'s declare-attackers site writes `Some(player)`, but its
/// own entry check proves `player` is the active player), so one shape reproduces
/// all of them. `enter_step`'s dead-active-player fallback is folded in.
///
/// **Fix-cycle Finding 4**: the priority grant was not the only thing those sites
/// owed. `enter_step`'s two guards both return *before*
/// `loop_detection::check_for_mandatory_loop`, and the Cleanup one additionally
/// before `state.turn.cleanup_sba_rounds += 1`. Skipping them turns two bounded
/// pathological states into unbounded ones: CR 726's mandatory-loop draw is never
/// declared for any batch that suspends, and the 100-round cleanup ratchet stops
/// advancing so the cleanup step can never fall through to auto-advance. Both are
/// reproduced here, selected by [`FlushResumeSite`].
fn finish_resumed_flush(state: &mut GameState, owed: FlushResumeSite, events: &mut Vec<GameEvent>) {
    if let Some(entry) = state.pending_trigger_targets.as_mut() {
        // Suspended again on a later trigger of the SAME CR 603.3b batch --
        // the obligation belongs to the batch, not to any one question.
        entry.resume_site = owed;
        return;
    }
    if owed == FlushResumeSite::None {
        return;
    }
    if run_flush_resume_obligations(state, owed, events) {
        return;
    }
    grant_priority_after_batch(state, events);
}
/// CR 514.3a / CR 726: the NON-priority half of what a suspended call site owed.
///
/// Split out of [`finish_resumed_flush`] by the second closing review's Finding 3
/// (LOW / OOS-DP8-13). The two halves of a [`FlushResumeSite`] are not
/// interchangeable: a duplicate `PriorityGiven` is a wire anomaly, while a dropped
/// ratchet or mandatory-loop check removes a *bound* on a pathological position.
/// `flush_pending_triggers`' reap has to discard the first half and must not
/// discard the second, so it calls this directly.
///
/// Returns `true` if it ended the game (a CR 726 draw), in which case no priority
/// is granted by anybody.
fn run_flush_resume_obligations(
    state: &mut GameState,
    owed: FlushResumeSite,
    events: &mut Vec<GameEvent>,
) -> bool {
    // CR 514.3a: the cleanup ratchet the Cleanup guard returned before reaching.
    // Bumped unconditionally rather than under `cleanup_sba_rounds <
    // MAX_CLEANUP_SBA_ROUNDS`: the fall-through-at-max the original branch does is
    // `enter_step`'s to make, and the next non-suspending cleanup round makes it.
    // The CR 726 check below is the real bound on a genuinely repeating state.
    if owed == FlushResumeSite::EnterStepCleanup {
        state.turn.cleanup_sba_rounds = state.turn.cleanup_sba_rounds.saturating_add(1);
    }
    // CR 104.4b / CR 726: the mandatory-loop check both `enter_step` guards
    // returned before reaching. The has-priority branch runs it only when the
    // batch actually placed something, which `!events.is_empty()` reproduces.
    if matches!(
        owed,
        FlushResumeSite::EnterStepPriority | FlushResumeSite::EnterStepCleanup
    ) && !events.is_empty()
    {
        if let Some(loop_event) = crate::rules::loop_detection::check_for_mandatory_loop(state) {
            events.push(loop_event);
            // All active players lose — the game is a draw.
            let active_players: Vec<_> = state.active_players();
            for p in active_players {
                if let Some(player) = state.expect_player_mut(p) {
                    player.has_lost = true;
                }
            }
            events.extend(crate::rules::engine::check_game_over(state));
            return true;
        }
    }
    false
}
/// CR 603.3b / CR 117.3a: "then the appropriate player gets priority" -- the active
/// player, routed past a dead one.
///
/// The shape every guarded call site was about to execute, factored out so
/// [`finish_resumed_flush`] and [`repair_departed_priority_holder`] cannot drift.
fn grant_priority_after_batch(state: &mut GameState, events: &mut Vec<GameEvent>) {
    let active = state.turn.active_player;
    if player_is_alive(state, active) {
        state.turn.players_passed = OrdSet::new();
        state.turn.priority_holder = Some(active);
        events.push(GameEvent::PriorityGiven { player: active });
    } else if let Some(next) = crate::rules::priority::next_priority_player(state, active) {
        state.turn.players_passed = OrdSet::new();
        state.turn.priority_holder = Some(next);
        events.push(GameEvent::PriorityGiven { player: next });
    } else {
        state.turn.priority_holder = None;
    }
}
/// `true` if `p` is still in the game (SR-25: a departed player legitimately
/// answers `false` here -- that is the question being asked, not a swallowed miss).
fn player_is_alive(state: &GameState, p: PlayerId) -> bool {
    state
        .players()
        .get(&p)
        .map(|pl| !pl.has_lost && !pl.has_conceded)
        .unwrap_or(false)
}
/// CR 800.4 / CR 603.3b (closing-review Finding 1, HIGH): a completed batch must
/// never hand the game back with priority pinned on a player who has left it.
///
/// `handle_concede` deliberately SKIPS its priority-advance block while another
/// player's announcement is outstanding (fix-cycle Finding 5: running it can
/// resolve the top of the stack, or advance a whole turn, under a half-placed
/// CR 603.3b batch). That gate's original comment claimed the resume would grant
/// priority anyway -- true for every [`FlushResumeSite`] except
/// [`FlushResumeSite::None`], which is the resume site of all 30 in-match
/// `check_and_flush_triggers` calls and the most common suspension class by far.
/// There, `finish_resumed_flush` returns without touching `priority_holder`, so a
/// conceding priority holder left the field naming a player who can never act
/// again: `PassPriority` from them is `PlayerEliminated`, from anyone else
/// `NotPriorityHolder`, and nothing else reassigns it. Every driving loop
/// (`LocalGame::advance`, `GameDriver`, the TUI auto-pass) dies there.
///
/// So the concede keeps its gate and the debt is discharged HERE, at the one
/// moment CR 603.3b permits it -- after the batch is complete. The successor is
/// the next player in APNAP order who has not passed (the actor's priority simply
/// moves on, exactly as `handle_concede` would have moved it); if every remaining
/// player has already passed, the batch just put objects on the stack, so
/// CR 603.3b's "the appropriate player gets priority" gives it to the active
/// player with the pass count reset.
///
/// The resume is not the only way out of a suspended batch, so it is not the only
/// caller. `handle_concede` calls this too, as the LAST thing it does (second
/// closing review, Finding 1 -- MEDIUM): a departure completes the batch through
/// [`drop_departed_trigger_flush`] without ever reaching `resume_trigger_flush`,
/// and `handle_concede`'s own advance is guarded on `priority_holder ==
/// Some(player)` -- so it can only repair holdership belonging to the CONCEDER,
/// never a holder stranded by an earlier departure. The claim that used to close
/// this doc block ("`handle_concede` runs its own (ungated, because the field is
/// now clear) advance straight afterwards") was false for exactly that reason.
///
/// Still deliberately NOT called from `flush_pending_triggers`' reap: that runs
/// inside a caller which either holds correct priority already (the 30
/// `check_and_flush_triggers` sites) or grants it itself (the six guards).
///
/// That argument covers the CURRENT caller's priority, not a PRE-EXISTING stranded
/// holder, so it is not sufficient on its own -- the combination is unreachable
/// instead, and the second closing review asked for the reason to be written down
/// rather than left implicit. A stranded holder requires a prior concede under a
/// suspended batch; the reap requires the entry's owner to be `has_lost` /
/// `has_conceded` by some route OTHER than `handle_concede`. While an entry is
/// outstanding the admission gate admits only the answer and `Concede`, and
/// `handle_concede` runs no SBA sweep -- so **no player can be marked `has_lost`
/// while an entry exists**, and the reap is reachable only by direct state
/// manipulation (which is exactly what its own test does).
///
/// **That last step is a scheduling accident, not a stated invariant.** If anything
/// ever runs an SBA sweep while a blocking decision is outstanding, the reap becomes
/// a real exit and this function must be called from it too. See OOS-DP8-13.
pub(crate) fn repair_departed_priority_holder(state: &mut GameState, events: &mut Vec<GameEvent>) {
    if state.pending_trigger_targets.is_some() {
        // Suspended again: the batch is still incomplete, so CR 603.3b still
        // forbids a grant. The next resume repairs it.
        return;
    }
    // CR 608.2d (PB-DP9): same reasoning for a rolled-back resolution. Granting
    // priority while a resolution-time choice is outstanding would step over the
    // engine's own admission gate (CR 608.1: the spell is still resolving).
    //
    // CLOSING-REVIEW HIGH-1 corrects what this comment used to claim. It named
    // "`handle_answer_effect_choice`'s tail -- `resolve_top_of_stack`'s own
    // grant -- and `handle_concede`'s discharge" as the two sites that pick the
    // skipped repair up, and `resolve_top_of_stack_inner`'s tail granted
    // priority to `turn.active_player` UNCONDITIONALLY, so it could not repair a
    // stranded holder when the departed seat WAS the active player -- it created
    // one. The accurate statement is in two parts:
    //
    //  * There is nothing to repair while the entry stands. `priority_holder` is
    //    `None` by construction: the roll-back restores the state
    //    `resolve_top_of_stack` was entered with, and both callers of
    //    `handle_all_passed` set `priority_holder = None` immediately before it
    //    (`rules/engine.rs`, the `AllPassed` arm and `handle_concede`'s
    //    all-others-passed branch). The `debug_assert!` below would fire if that
    //    ever stopped holding, because `gone` would then be a real value.
    //  * When the entry clears, the holder is assigned by
    //    `priority::grant_priority_to_active_player`, which IS liveness-aware
    //    (CR 800.4j). That is the repair, and it covers all three callers of
    //    `resolve_top_of_stack`, not just the answer path.
    if state.pending_effect_choice.is_some() {
        debug_assert!(
            state.turn.priority_holder.is_none(),
            "CR 608.2d: nobody may hold priority while a resolution-time choice \
             is outstanding, but {:?} does",
            state.turn.priority_holder
        );
        return;
    }
    let gone = match state.turn.priority_holder {
        Some(p) if !player_is_alive(state, p) => p,
        _ => return,
    };
    if let Some(next) = crate::rules::priority::next_priority_player(state, gone) {
        state.turn.priority_holder = Some(next);
        events.push(GameEvent::PriorityGiven { player: next });
        return;
    }
    grant_priority_after_batch(state, events);
}
/// CR 603.3 / CR 117.3a / CR 726 (PB-DP8): record what the call site whose
/// `flush_pending_triggers` just suspended still owes once the batch completes.
///
/// Called by exactly the guards named in the `BlockingDecision` doc block. The 30
/// `check_and_flush_triggers` sites inside `process_command`'s `match` must NOT
/// call this: PB-DP1 moved priority assignment into the command handlers, ahead of
/// the flush, so priority is already correctly held by the actor there. The 31st
/// (`handle_all_passed`'s forced-overdue-payment branch, fix-cycle Finding 3) DOES
/// grant priority afterwards, and passes [`FlushResumeSite::GrantPriority`].
pub(crate) fn mark_flush_resume_site(state: &mut GameState, site: FlushResumeSite) {
    if let Some(entry) = state.pending_trigger_targets.as_mut() {
        entry.resume_site = site;
    }
}
/// CR 800.4d / CR 603.3b / CR 800.4j (PB-DP8): a player LEAVES THE GAME while
/// their trigger-target announcement is outstanding.
///
/// CR 800.4d -- "If a triggered ability that would be controlled by a player who
/// has left the game would be put onto the stack, it isn't put on the stack." --
/// so the departed player's un-placed trigger is DROPPED, along with every other
/// trigger of the suspended batch they controlled. CR 800.4j requires the turn to
/// continue, so the REST of the batch is still placed; that continuation may
/// legitimately suspend again on a different player's trigger.
///
/// Returns `None` if no entry belonged to `player` (the caller then leaves the
/// field alone -- another player's outstanding question still blocks).
pub(crate) fn drop_departed_trigger_flush(
    state: &mut GameState,
    player: PlayerId,
) -> Option<Vec<GameEvent>> {
    let belongs = state
        .pending_trigger_targets
        .as_ref()
        .map(|e| e.player == player)
        .unwrap_or(false);
    if !belongs {
        return None;
    }
    let entry = state.pending_trigger_targets.take()?;
    let owed = entry.resume_site;
    // CR 800.4d: `entry.trigger` is dropped outright -- it was never put on the
    // stack. Same for every remaining trigger this player controls.
    let sorted: Vec<PendingTrigger> = entry
        .remaining
        .iter()
        .filter(|t| t.controller != player)
        .cloned()
        .collect();
    let mut events = flush_sorted(state, sorted, None);
    finish_resumed_flush(state, owed, &mut events);
    Some(events)
}
/// CR 603.3d / CR 601.2c (PB-DP8 / DP-6): the trigger's controller answers the
/// outstanding target announcement, and the CR 603.3b batch resumes.
///
/// Every check runs BEFORE any mutation. `process_command` takes `GameState` by
/// value and discards the locally-mutated copy on `Err`, so an `Err` here leaves
/// the caller's state byte-identical -- but only because nothing below mutates
/// before the last validation passes.
pub fn handle_choose_trigger_targets(
    state: &mut GameState,
    player: PlayerId,
    choice_id: u64,
    targets: Vec<Vec<Target>>,
) -> Result<Vec<GameEvent>, GameStateError> {
    // (2) An entry must exist.
    let entry = match state.pending_trigger_targets.as_ref() {
        Some(e) => e,
        None => {
            return Err(GameStateError::InvalidCommand(
                "no trigger-target choice is pending (CR 603.3d)".to_string(),
            ))
        }
    };
    // (3) CR 603.3a: only the trigger's controller may answer. SR-29 trust
    // boundary. `process_command`'s admission gate rejects a foreign sender with
    // `BlockedByPendingDecision` before reaching here, so this check is only
    // reachable by a direct handler call -- which is exactly the hole PB-DP7's
    // review Finding 12 found.
    if entry.player != player {
        return Err(GameStateError::InvalidCommand(format!(
            "trigger-target choice belongs to {:?}, not {:?} (CR 603.3a)",
            entry.player, player
        )));
    }
    // (4) The MOMENT guard: an answer to question k of a CR 603.3b batch must not
    // be applied to question k+1.
    if entry.choice_id != choice_id {
        return Err(GameStateError::InvalidCommand(format!(
            "stale trigger-target choice: expected {}, got {}",
            entry.choice_id, choice_id
        )));
    }
    // (5) One inner Vec per offered slot, in order.
    if targets.len() != entry.slots.len() {
        return Err(GameStateError::InvalidCommand(format!(
            "trigger-target choice has {} slot(s), expected {} (CR 601.2c)",
            targets.len(),
            entry.slots.len()
        )));
    }
    let mut per_slot: Vec<Vec<SpellTarget>> = Vec::with_capacity(entry.slots.len());
    for (i, slot) in entry.slots.iter().enumerate() {
        let submitted = &targets[i];
        // (6) CR 601.2c: exactly one target for a required slot; zero to `max` for
        // an "up to" slot ("If the spell has a variable number of targets, the
        // player announces how many targets they will choose"). Fix-cycle
        // Finding 2: this bound used to be a hard `1`.
        let limit_ok = if slot.optional {
            submitted.len() <= slot.max as usize
        } else {
            submitted.len() == 1
        };
        if !limit_ok {
            return Err(GameStateError::InvalidCommand(format!(
                "trigger-target slot {} got {} target(s); expected {} (CR 601.2c)",
                i,
                submitted.len(),
                if slot.optional {
                    format!("0 to {}", slot.max)
                } else {
                    "exactly 1".to_string()
                }
            )));
        }
        // (6b) CR 601.2c: "The same target can't be chosen multiple times for any
        // one instance of the word 'target'." A slot IS one instance of the word,
        // so its submitted targets must be pairwise distinct. Latent until
        // Finding 2 raised the cap above one.
        for a in 0..submitted.len() {
            if submitted[a + 1..].contains(&submitted[a]) {
                return Err(GameStateError::InvalidCommand(format!(
                    "trigger-target slot {i} names the same target twice (CR 601.2c)"
                )));
            }
        }
        // (7) CR 603.3d legality: membership in the candidate set the engine
        // itself offered. The engine takes `zone_at_cast` from the candidate,
        // never from the wire.
        let mut resolved: Vec<SpellTarget> = Vec::with_capacity(submitted.len());
        for t in submitted {
            match slot.candidates.iter().find(|c| &c.target == t) {
                Some(c) => resolved.push(c.clone()),
                None => {
                    return Err(GameStateError::InvalidCommand(format!(
                        "target is not a legal choice for slot {i} (CR 603.3d)"
                    )))
                }
            }
        }
        per_slot.push(resolved);
    }
    // (8) Cross-slot distinctness, narrow: CR 601.2c's "the same target can't be
    // chosen multiple times for any one instance of the word 'target'" is
    // per-slot and covered by (6). This covers the one cross-slot case the DSL
    // can express, `TargetPermanentDistinctFrom`. Zero corpus exposure today; the
    // auto-fallback keeps the defect (OOS-DP8-4).
    {
        let reqs = trigger_ability_target_requirements(state, &entry.trigger);
        for a in 0..entry.slots.len() {
            for b in (a + 1)..entry.slots.len() {
                let distinct = matches!(
                    (reqs.get(a), reqs.get(b)),
                    (
                        Some(crate::cards::card_definition::TargetRequirement::TargetPermanentDistinctFrom(_)),
                        Some(crate::cards::card_definition::TargetRequirement::TargetPermanentDistinctFrom(_)),
                    )
                );
                if distinct && !targets[a].is_empty() && targets[a] == targets[b] {
                    return Err(GameStateError::InvalidCommand(format!(
                        "slots {a} and {b} both require distinct permanents but name the same one (CR 601.2c)"
                    )));
                }
            }
        }
    }
    // (1) The player must exist. Deliberately last of the cheap checks and still
    // before any mutation: the entry's own `player` field is the authority on who
    // may answer, and (3) already compared against it.
    state.player(player)?;
    // (9) CR 601.2c (fix-cycle Finding 6): flatten to the stack object's flat
    // target list, keeping each slot at its declared width so an under-filled
    // "up to" slot does not shift the later clauses' `DeclaredTarget` indices.
    let slots: Vec<TriggerTargetOption> = entry.slots.iter().cloned().collect();
    let chosen = flatten_slot_answers(&slots, &per_slot);
    // (10) Only now: resume the CR 603.3b batch.
    Ok(resume_trigger_flush(state, chosen))
}
/// The `TargetRequirement` list of the ability behind `trigger`, re-derived the
/// same way `flush_sorted` derives it. Used only by the cross-slot distinctness
/// check (CR 601.2c).
///
/// PB-DX35 (`OOS-DX4-2`): this is `trigger_modal_plan(state, trigger).requirements`
/// -- the fourth reader of the one shared arithmetic, alongside `flush_sorted`'s
/// `trigger_target_requirements` / `ability_targets` / modes_chosen sites.
///
/// **Re-derivation stability**: this runs inside `handle_choose_trigger_targets`,
/// which is reached only while a `BlockingDecision::TriggerTargets` is
/// outstanding. `process_command`'s admission gate (`rules::engine`, the
/// PB-DP7/DP-3 gate) admits only `Command::Concede` and the SAME player's
/// `Command::ChooseTriggerTargets` while such a decision is outstanding --
/// verified in source, not assumed. So between the suspend (where `entry.slots`
/// was built from this SAME `trigger_modal_plan`) and this call, the only
/// mutation a command boundary could have inserted is a `Concede`. This function
/// and the `resume_trigger_flush` call immediately after it (which re-enters
/// `flush_sorted` for the same trigger) both read the state at the SAME instant
/// -- there is no command boundary between them -- so they cannot disagree with
/// each other even if a `Concede` earlier changed which mode is CR 700.2b-legal.
/// A residual is stated rather than hidden: if a `Concede` between the ORIGINAL
/// suspend and this resume changes which mode is legal, the previously-offered
/// `entry.slots` (built from the stale plan) could describe a different mode's
/// target shape than the one this resume now computes. The corpus population
/// that could ever reach a suspending choice inside a modal trigger's chosen
/// mode is zero today (`core::pb_dx35_modal_trigger_roster::r4`/`r5`: all seven
/// modal triggered abilities are `max_modes: 1` with at most one non-empty mode
/// slice), so this is filed rather than engineered around (`OOS-DX35-2`).
fn trigger_ability_target_requirements(
    state: &GameState,
    trigger: &PendingTrigger,
) -> Vec<crate::cards::card_definition::TargetRequirement> {
    trigger_modal_plan(state, trigger)
        .map(|p| p.requirements)
        .unwrap_or_default()
}
/// CR 603.3d (PB-DP8): the deterministic default as `SpellTarget`s, i.e. exactly
/// the value the pre-PB-DP8 first-match chain produced for the whole slot list.
///
/// The `SpellTarget` twin of the exported [`default_trigger_targets`], used on the
/// engine's own dead-controller path where there is nobody to ask.
///
/// Fix-cycle Finding 6: goes through [`flatten_slot_answers`] rather than a bare
/// `filter_map`, so a slot with no default does not shift the later slots'
/// `EffectTarget::DeclaredTarget { index }` down.
fn default_spell_targets(slots: &[TriggerTargetOption]) -> Vec<SpellTarget> {
    let per_slot: Vec<Vec<SpellTarget>> = slots
        .iter()
        .map(|s| s.default.clone().into_iter().collect())
        .collect();
    flatten_slot_answers(slots, &per_slot)
}
// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
/// Returns player IDs in APNAP order starting from the active player.
///
/// CR 101.4 (APNAP): Active Player, Non-Active Players in turn order.
pub fn apnap_order(state: &GameState) -> Vec<PlayerId> {
    let active = state.turn.active_player;
    let order = &state.turn.turn_order;
    let n = order.len();
    // MR-M3-11: active player must always be in turn_order; assert in debug builds.
    let start_pos = order.iter().position(|&p| p == active);
    debug_assert!(
        start_pos.is_some(),
        "apnap_order: active player {:?} not found in turn_order {:?}",
        active,
        order
    );
    let start = start_pos.unwrap_or(0);
    (0..n).map(|i| order[(start + i) % n]).collect()
}
/// Every player in the game, in APNAP order — the ordering CR 608.2e requires for a
/// multi-player resolution-time choice (PB-DX15a, closes `OOS-DP9-8`).
///
/// CR 608.2e: *"the choices for the first action are made in APNAP order, and then the
/// first action is processed simultaneously"*. CR 101.4 defines that order as the active
/// player, then the remaining players in turn order.
///
/// # Why this is not just `apnap_order`
///
/// [`apnap_order`] enumerates `state.turn.turn_order` and nothing else. Every production
/// state is built by `GameStateBuilder`, which sets `turn_order` from the same player
/// list it seeds `state.players` from (`state/builder.rs`), so the two agree — but
/// `turn_order` is never re-derived afterwards and `retarget.rs`'s own doc block already
/// records that an alive-but-absent player is *believed* impossible rather than enforced.
/// `resolve_player_target_list`'s universe is `state.players`, so swapping it for
/// `turn_order` alone would silently **drop** any such player from an "each player"
/// effect — trading a wrong order for a missing player, which is strictly worse.
///
/// So: order by `turn_order`, then append anything in `state.players` that `turn_order`
/// does not name, ascending — i.e. the pre-PB-DX15a order, for exactly the residue that
/// has no APNAP position to be given. The `debug_assert` states that this residue is
/// expected to be empty (SR-4: an engine-bug assertion, not a runtime rejection).
pub fn apnap_order_all_players(state: &GameState) -> Vec<PlayerId> {
    let mut out = apnap_order(state);
    let mut residue: Vec<PlayerId> = state
        .players
        .keys()
        .copied()
        .filter(|p| !out.contains(p))
        .collect();
    debug_assert!(
        residue.is_empty(),
        "apnap_order_all_players: {residue:?} are in state.players but absent from \
         turn_order {:?} — they keep their pre-APNAP ascending position rather than \
         being dropped, but the divergence is an engine bug",
        state.turn.turn_order
    );
    residue.sort();
    out.extend(residue);
    out
}
/// CR 603.2d: Compute how many additional times a trigger should fire due to
/// Panharmonicon-style trigger-doubling effects.
///
/// Returns the number of ADDITIONAL triggers beyond the base 1. So a return
/// value of 0 means fire exactly once; 1 means fire twice; etc.
///
/// Each active `TriggerDoubler` whose filter matches the trigger contributes
/// `additional_triggers` extra instances. With two Panharmonicons, an ETB
/// trigger that would fire once instead fires three times (2 extra each).
///
/// Panharmonicon-style rulings (2024): the ability "triggers an additional time"
/// — each Panharmonicon adds another copy; they stack independently.
fn compute_trigger_doubling(state: &GameState, trigger: &PendingTrigger) -> u32 {
    let mut additional = 0u32;
    for doubler in state.trigger_doublers.iter() {
        if doubler_applies_to_trigger(state, doubler, trigger) {
            additional += doubler.additional_triggers;
        }
    }
    additional
}
/// CR 603.2d: Determine whether a specific `TriggerDoubler` applies to the given trigger.
///
/// For `ArtifactOrCreatureETB`: the trigger must be from a permanent entering the
/// battlefield, AND the trigger's source (the permanent with the ability) must be
/// controlled by the doubler's controller, AND the triggering event must be an ETB
/// (`AnyPermanentEntersBattlefield` or `SelfEntersBattlefield`) caused by an artifact
/// or creature entering.
///
/// Panharmonicon ruling 2021-03-19: "Panharmonicon affects a permanent's own
/// enters-the-battlefield triggered abilities as well as other triggered abilities that
/// trigger when that permanent enters the battlefield." This means both
/// `AnyPermanentEntersBattlefield` and `SelfEntersBattlefield` must be matched.
fn doubler_applies_to_trigger(
    state: &GameState,
    doubler: &TriggerDoubler,
    trigger: &PendingTrigger,
) -> bool {
    // Doubler source must still be on the battlefield.
    let source_active = state
        .objects
        .get(&doubler.source)
        .map(|o| o.zone == ZoneId::Battlefield)
        .unwrap_or(false);
    if !source_active {
        return false;
    }
    // The trigger must be controlled by the same player as the doubler.
    if trigger.controller != doubler.controller {
        return false;
    }
    // CR 110.1 / CR 603.2d (PB-DX15a `/review` Issue 5, closes rider `OOS-DX24-1`):
    // every printed doubler reads "a triggered ability of a **permanent** you control",
    // and a card in a graveyard is not a permanent — so a `trigger_zone: Graveyard`
    // ability (Nether Traitor's shape, CR 113.6m) is out of scope for all four filters.
    //
    // # Why this is NOT the source-zone conjunct `OOS-DX24-1` prescribed
    //
    // That row asked for a bare "the trigger's source is on the battlefield" test. It is
    // CR-wrong, and PB-DX15a proved it by executing it: a CR 603.6c/603.10a look-back
    // "when this dies" trigger is built as `PendingTrigger::blank(*new_grave_id, ..)`
    // (`:4839`), so **its source is a graveyard object too**. The bare conjunct stops
    // Teysa Karlov doubling a dying creature's own dies trigger — the commonest real use
    // of the `CreatureDeath` arm — and does so with the whole workspace green, because
    // that arm had zero behavioural coverage until this batch added
    // `test_dx15a_creature_death_doubler_doubles_a_look_back_dies_trigger`.
    //
    // # The discriminator is the EVENT, not the zone
    //
    // Both cases present a graveyard source; they differ in *why*. Measured over every
    // construction site, the four `triggering_event` values that can reach the `match`
    // below with a non-battlefield source split exactly two ways:
    //
    // | event | built by | source | verdict |
    // |---|---|---|---|
    // | `SelfDies` | `:4831`/`:4924`, look-back | the dying object's graveyard id | **was** a permanent — double |
    // | `SelfEntersBattlefield` | the ETB paths, look-back if it has since left | LKI | **was** a permanent — double |
    // | `AnyCreatureDies` | `collect_graveyard_carddef_triggers:7590` | a graveyard-resident card | never a permanent — do NOT double |
    // | `AnyPermanentEntersBattlefield` | the same graveyard collector | a graveyard-resident card | never a permanent — do NOT double |
    //
    // The split is total because the battlefield-sourced `AnyCreatureDies` collector
    // filters on `obj.zone == ZoneId::Battlefield` (`:5070`), so a graveyard source
    // carrying that event can ONLY have come from the graveyard collector. Verified by
    // enumeration, not assumed — and it is what makes a `Self*` allowlist safe rather
    // than merely plausible.
    //
    // Keyed on "exists and is not on the battlefield" rather than "is in a graveyard":
    // the graveyard is the only non-battlefield trigger-source channel today, so the two
    // are equivalent now, and the broader form does not need revisiting if a second one
    // appears. A source ABSENT from `state.objects` keeps the pre-existing permissive
    // behaviour — that is LKI, and narrowing it is a separate question this row does not
    // reach.
    // SR-25/SR-4: `fizzle_object`, not a bare `.objects.get(..)`. A `None` here is
    // RULES-CORRECT, not an engine bug: the trigger's source may legitimately have left
    // (CR 113.7a LKI), which is the `unwrap_or(false)`-equivalent branch below — absent
    // means "not known to be off the battlefield", i.e. keep the permissive
    // pre-existing behaviour. (The `bare_lookup_ratchet` fired on the first draft of
    // this line and was right to.)
    let source_is_off_battlefield = state
        .fizzle_object(trigger.source)
        .is_some_and(|o| o.zone != ZoneId::Battlefield);
    let is_look_back_self_trigger = matches!(
        trigger.triggering_event,
        Some(TriggerEvent::SelfDies) | Some(TriggerEvent::SelfEntersBattlefield)
    );
    if source_is_off_battlefield && !is_look_back_self_trigger {
        return false;
    }
    match &doubler.filter {
        TriggerDoublerFilter::ArtifactOrCreatureETB => {
            // The triggering event must be an ETB event (CR 603.2d + Panharmonicon ruling
            // 2021-03-19): both AnyPermanentEntersBattlefield (other permanents watching)
            // and SelfEntersBattlefield (the entering artifact/creature's own ETB ability)
            // are matched. This mirrors the CreatureDeath arm's dual-event pattern.
            let is_etb = matches!(
                trigger.triggering_event,
                Some(TriggerEvent::AnyPermanentEntersBattlefield)
                    | Some(TriggerEvent::SelfEntersBattlefield)
            );
            if !is_etb {
                return false;
            }
            // The entering object must be an artifact or creature (CR 603.2d).
            // Use entering_object_id (set by check_triggers from PermanentEnteredBattlefield event).
            // If entering_object_id is absent, we cannot confirm the type — conservatively skip.
            let entering_id = match trigger.entering_object_id {
                Some(id) => id,
                None => return false,
            };
            // Use calculate_characteristics for type checks under continuous effects,
            // falling back to raw characteristics if the object is no longer in the
            // objects map (e.g., it moved zones since entering).
            let entering_chars =
                crate::rules::layers::calculate_characteristics(state, entering_id).or_else(|| {
                    state
                        .objects
                        .get(&entering_id)
                        .map(|o| o.characteristics.clone())
                });
            entering_chars
                .map(|chars| {
                    use crate::state::types::CardType;
                    chars.card_types.contains(&CardType::Artifact)
                        || chars.card_types.contains(&CardType::Creature)
                })
                .unwrap_or(false)
        }
        TriggerDoublerFilter::CreatureDeath => {
            // CR 603.2d: The triggering event must be a creature dying.
            // Matches both SelfDies (the dying creature's own "when ~ dies" abilities)
            // and AnyCreatureDies (other permanents with "whenever a creature dies" abilities
            // like Blood Artist, Zulaport Cutthroat, Grave Pact, etc.). PB-23 wired both.
            matches!(
                trigger.triggering_event,
                Some(TriggerEvent::SelfDies) | Some(TriggerEvent::AnyCreatureDies)
            )
        }
        TriggerDoublerFilter::AnyPermanentETB => {
            // CR 603.2d: Yarok / Elesh Norn pattern — doubles ETB triggers from ANY
            // permanent entering, not just artifacts and creatures. No type check needed.
            // Matches both the "watching" trigger variant and the self-ETB variant.
            matches!(
                trigger.triggering_event,
                Some(TriggerEvent::AnyPermanentEntersBattlefield)
                    | Some(TriggerEvent::SelfEntersBattlefield)
            )
        }
        TriggerDoublerFilter::LandETB => {
            // CR 603.2d: Ancient Greenwarden pattern — doubles ETB triggers only when a
            // land enters. Checks the entering permanent's card types (under continuous
            // effects) to confirm it is a land.
            let is_etb = matches!(
                trigger.triggering_event,
                Some(TriggerEvent::AnyPermanentEntersBattlefield)
                    | Some(TriggerEvent::SelfEntersBattlefield)
            );
            if !is_etb {
                return false;
            }
            let entering_id = match trigger.entering_object_id {
                Some(id) => id,
                None => return false,
            };
            let entering_chars =
                crate::rules::layers::calculate_characteristics(state, entering_id).or_else(|| {
                    state
                        .objects
                        .get(&entering_id)
                        .map(|o| o.characteristics.clone())
                });
            entering_chars
                .map(|chars| {
                    use crate::state::types::CardType;
                    chars.card_types.contains(&CardType::Land)
                })
                .unwrap_or(false)
        }
    }
}
// ---------------------------------------------------------------------------
// Crew handler (CR 702.122)
// ---------------------------------------------------------------------------
/// Handle a CrewVehicle command: validate, tap crew creatures, push crew ability onto the stack.
///
/// CR 702.122a: "Tap any number of other untapped creatures you control with total power N
/// or greater: This permanent becomes an artifact creature until end of turn."
///
/// When the crew ability resolves, an `AddCardTypes({Creature})` continuous effect is
/// registered in Layer 4 (TypeChange) with `UntilEndOfTurn` duration.
///
/// Notable rulings:
/// - Summoning sickness does NOT prevent crewing (ruling): tapping for crew cost is not
///   a {T} activated ability — summoning sickness only prevents those.
/// - Crewing an already-crewed Vehicle is legal but has no effect (ruling).
/// - Becoming a creature via crew does NOT trigger ETB effects (ruling).
pub fn handle_crew_vehicle(
    state: &mut GameState,
    player: PlayerId,
    vehicle: ObjectId,
    crew_creatures: Vec<ObjectId>,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::cards::card_definition::ContinuousEffectDef;
    use crate::rules::layers::calculate_characteristics;
    use crate::state::continuous_effect::{
        EffectDuration, EffectFilter, EffectLayer, LayerModification,
    };
    use crate::state::types::CardType;
    use std::collections::HashSet;
    // CR 602.2: Crewing requires priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // CR 702.61a: If a spell with split second is on the stack, no non-mana
    // abilities can be activated.
    if crate::rules::casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; crew ability cannot be activated (CR 702.61a)".into(),
        ));
    }
    // Validate the Vehicle: must be on the battlefield, controlled by the player,
    // and must have KeywordAbility::Crew(n). Use calculate_characteristics for
    // layer correctness (e.g., Humility may have removed the keyword).
    let crew_cost_n: u32 = {
        let obj = state.object(vehicle)?;
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(vehicle));
        }
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: vehicle,
            });
        }
        // Use layer-computed characteristics to account for continuous effects.
        let chars = calculate_characteristics(state, vehicle).or_else(|| {
            state
                .objects
                .get(&vehicle)
                .map(|o| o.characteristics.clone())
        });
        let crew_n = chars.as_ref().and_then(|c| {
            c.keywords.iter().find_map(|kw| {
                if let KeywordAbility::Crew(n) = kw {
                    Some(*n)
                } else {
                    None
                }
            })
        });
        crew_n.ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "object {:?} does not have the Crew keyword (CR 702.122a)",
                vehicle
            ))
        })?
    };
    // Validate crew_creatures is non-empty (you must tap at least one creature).
    if crew_creatures.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "must provide at least one creature to crew the vehicle (CR 702.122a)".into(),
        ));
    }
    // CR 702.122a: Validate uniqueness — no duplicates in crew_creatures.
    let mut seen: HashSet<ObjectId> = HashSet::new();
    for &id in &crew_creatures {
        if !seen.insert(id) {
            return Err(GameStateError::InvalidCommand(format!(
                "duplicate creature {:?} in crew_creatures (CR 702.122a)",
                id
            )));
        }
    }
    // CR 702.122a: Validate each crew creature — must be an untapped creature
    // you control on the battlefield, and must not be the vehicle itself.
    // Also sum total power for the crew cost threshold check.
    // Note: summoning sickness does NOT prevent crewing (ruling under CR 702.122a);
    // tapping for crew cost is not a {T} activated ability.
    let mut total_power: i32 = 0;
    for &id in &crew_creatures {
        // CR 702.122a: "other" — vehicle cannot crew itself.
        if id == vehicle {
            return Err(GameStateError::InvalidCommand(
                "a vehicle cannot be used to crew itself (CR 702.122a: 'other untapped creatures')"
                    .into(),
            ));
        }
        let obj = state
            .objects
            .get(&id)
            .ok_or(GameStateError::ObjectNotFound(id))?;
        // Must be on the battlefield.
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(id));
        }
        // Must be controlled by the player.
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: id,
            });
        }
        // Must be untapped (CR 702.122a: "untapped creatures").
        if obj.status.tapped {
            return Err(GameStateError::InvalidCommand(format!(
                "creature {:?} is already tapped and cannot be used to crew (CR 702.122a)",
                id
            )));
        }
        // Must be a creature (use layer-computed characteristics).
        let chars = calculate_characteristics(state, id)
            .or_else(|| state.objects.get(&id).map(|o| o.characteristics.clone()));
        let is_creature = chars
            .as_ref()
            .map(|c| c.card_types.contains(&CardType::Creature))
            .unwrap_or(false);
        if !is_creature {
            return Err(GameStateError::InvalidCommand(format!(
                "object {:?} is not a creature and cannot be used to crew (CR 702.122a)",
                id
            )));
        }
        // Accumulate power for the total power check.
        let power = chars.and_then(|c| c.power).unwrap_or(0);
        total_power = total_power.saturating_add(power);
    }
    // CR 702.122a: Total power of tapped creatures must be >= N.
    if total_power < crew_cost_n as i32 {
        return Err(GameStateError::InvalidCommand(format!(
            "total power of crew creatures ({}) is less than Crew {} cost (CR 702.122a)",
            total_power, crew_cost_n
        )));
    }
    // Pay the cost: tap all crew creatures (CR 602.2b analog for crew cost).
    let mut events = Vec::new();
    for &id in &crew_creatures {
        if let Some(obj) = state.expect_object_mut(id) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: id,
        });
    }
    // Push the crew ability onto the stack as an activated ability.
    // The embedded effect is `ApplyContinuousEffect` that adds `Creature` type
    // in Layer 4 with `UntilEndOfTurn` duration, targeting the vehicle (source).
    let stack_id = state.next_object_id();
    // Build the embedded effect: AddCardTypes({Creature}) in Layer 4, on the source.
    let effect_def = ContinuousEffectDef {
        layer: EffectLayer::TypeChange,
        modification: LayerModification::AddCardTypes(imbl::OrdSet::from(vec![CardType::Creature])),
        filter: EffectFilter::Source, // resolved to SingleObject(vehicle) at execution
        duration: EffectDuration::UntilEndOfTurn,
        condition: None,
    };
    let embedded_effect = crate::cards::card_definition::Effect::ApplyContinuousEffect {
        effect_def: Box::new(effect_def),
    };
    // MR-TC-25: use trigger_default for the boilerplate cast-specific fields.
    let stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::ActivatedAbility {
            source_object: vehicle,
            ability_index: 0, // synthetic — crew ability has no index in activated_abilities
            embedded_effect: Some(Box::new(embedded_effect)),
        },
    );
    state.stack_objects.push_back(stack_obj);
    // CR 602.2b -> 601.2i / CR 117.3c: the activating player receives priority afterward.
    // (Neither "CR 602.2e" nor "CR 116.3b" exists.) Crew (CR 702.122a) has no
    // active-player gate, so this flips: a non-active player who crews retains
    // priority afterward. CR 117.4: reset the pass-round.
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: vehicle,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
/// CR 702.171a: Handle the `SaddleMount` command.
///
/// Validates that:
/// - The player holds priority (CR 602.2).
/// - No split-second spell is on the stack (CR 702.61a).
/// - Sorcery-speed restriction (CR 702.171a): active player's turn, main phase, empty stack.
/// - The Mount is on the battlefield and controlled by the player.
/// - The Mount has `KeywordAbility::Saddle(n)` in layer-resolved characteristics.
/// - Each saddling creature is an untapped creature controlled by the player (not the Mount).
/// - Total power of saddling creatures >= N.
/// - No duplicate creature IDs.
///
/// On success: taps all saddling creatures, pushes `StackObjectKind::SaddleAbility` onto
/// the stack, and grants priority to the active player.
///
/// Key differences from `handle_crew_vehicle`:
/// - Sorcery-speed only (CR 702.171a): active player, main phase, empty stack.
/// - No layer-4 type change: Mount is already a creature. Sets `is_saddled` flag instead.
/// - Ruling 2024-04-12: activating saddle on an already-saddled Mount is legal.
pub fn handle_saddle_mount(
    state: &mut GameState,
    player: PlayerId,
    mount: ObjectId,
    saddle_creatures: Vec<ObjectId>,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::rules::layers::calculate_characteristics;
    use crate::state::types::CardType;
    use std::collections::HashSet;
    // CR 602.2: Saddling requires priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // CR 702.61a: If a spell with split second is on the stack, no non-mana
    // abilities can be activated.
    if crate::rules::casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; saddle ability cannot be activated (CR 702.61a)".into(),
        ));
    }
    // CR 702.171a: "Activate only as a sorcery." Enforce sorcery-speed:
    // - Must be the active player's turn.
    // - Must be a main phase (PreCombatMain or PostCombatMain).
    // - Stack must be empty.
    if state.turn.active_player != player {
        return Err(GameStateError::InvalidCommand(
            "saddle ability can only be activated during your own turn (CR 702.171a: 'activate only as a sorcery')".into(),
        ));
    }
    let in_main_phase = matches!(
        state.turn.step,
        crate::state::turn::Step::PreCombatMain | crate::state::turn::Step::PostCombatMain
    );
    if !in_main_phase {
        return Err(GameStateError::InvalidCommand(
            "saddle ability can only be activated during a main phase (CR 702.171a: 'activate only as a sorcery')".into(),
        ));
    }
    if !state.stack_objects.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "saddle ability can only be activated when the stack is empty (CR 702.171a: 'activate only as a sorcery')".into(),
        ));
    }
    // Validate the Mount: must be on the battlefield, controlled by the player,
    // and must have KeywordAbility::Saddle(n). Use calculate_characteristics for
    // layer correctness (e.g., Humility may have removed the keyword).
    let saddle_cost_n: u32 = {
        let obj = state.object(mount)?;
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(mount));
        }
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: mount,
            });
        }
        // Use layer-computed characteristics to account for continuous effects.
        let chars = calculate_characteristics(state, mount)
            .or_else(|| state.objects.get(&mount).map(|o| o.characteristics.clone()));
        let saddle_n = chars.as_ref().and_then(|c| {
            c.keywords.iter().find_map(|kw| {
                if let KeywordAbility::Saddle(n) = kw {
                    Some(*n)
                } else {
                    None
                }
            })
        });
        saddle_n.ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "object {:?} does not have the Saddle keyword (CR 702.171a)",
                mount
            ))
        })?
    };
    // Validate saddle_creatures is non-empty (you must tap at least one creature).
    if saddle_creatures.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "must provide at least one creature to saddle the mount (CR 702.171a)".into(),
        ));
    }
    // CR 702.171a: Validate uniqueness — no duplicates in saddle_creatures.
    let mut seen: HashSet<ObjectId> = HashSet::new();
    for &id in &saddle_creatures {
        if !seen.insert(id) {
            return Err(GameStateError::InvalidCommand(format!(
                "duplicate creature {:?} in saddle_creatures (CR 702.171a)",
                id
            )));
        }
    }
    // CR 702.171a: Validate each saddling creature — must be an untapped creature
    // you control on the battlefield, and must not be the mount itself.
    // Also sum total power for the saddle cost threshold check.
    // Note: summoning sickness does NOT prevent saddling (same ruling as Crew);
    // tapping for saddle cost is not a {T} activated ability.
    let mut total_power: i32 = 0;
    for &id in &saddle_creatures {
        // CR 702.171a: "other" — mount cannot saddle itself.
        if id == mount {
            return Err(GameStateError::InvalidCommand(
                "a mount cannot be used to saddle itself (CR 702.171a: 'other untapped creatures')"
                    .into(),
            ));
        }
        let obj = state
            .objects
            .get(&id)
            .ok_or(GameStateError::ObjectNotFound(id))?;
        // Must be on the battlefield.
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(id));
        }
        // Must be controlled by the player.
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: id,
            });
        }
        // Must be untapped (CR 702.171a: "untapped creatures").
        if obj.status.tapped {
            return Err(GameStateError::InvalidCommand(format!(
                "creature {:?} is already tapped and cannot be used to saddle (CR 702.171a)",
                id
            )));
        }
        // Must be a creature (use layer-computed characteristics).
        let chars = calculate_characteristics(state, id)
            .or_else(|| state.objects.get(&id).map(|o| o.characteristics.clone()));
        let is_creature = chars
            .as_ref()
            .map(|c| c.card_types.contains(&CardType::Creature))
            .unwrap_or(false);
        if !is_creature {
            return Err(GameStateError::InvalidCommand(format!(
                "object {:?} is not a creature and cannot be used to saddle (CR 702.171a)",
                id
            )));
        }
        // Accumulate power for the total power check.
        let power = chars.and_then(|c| c.power).unwrap_or(0);
        total_power = total_power.saturating_add(power);
    }
    // CR 702.171a: Total power of tapped creatures must be >= N.
    if total_power < saddle_cost_n as i32 {
        return Err(GameStateError::InvalidCommand(format!(
            "total power of saddle creatures ({}) is less than Saddle {} cost (CR 702.171a)",
            total_power, saddle_cost_n
        )));
    }
    // Pay the cost: tap all saddling creatures (CR 602.2b analog for saddle cost).
    let mut events = Vec::new();
    for &id in &saddle_creatures {
        if let Some(obj) = state.expect_object_mut(id) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: id,
        });
    }
    // Push the saddle ability onto the stack.
    // When resolved, `SaddleAbility` sets `is_saddled = true` on the Mount (resolution.rs).
    let stack_id = state.next_object_id();
    // MR-TC-25: use trigger_default for the boilerplate cast-specific fields.
    let stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::SaddleAbility {
            source_object: mount,
        },
    );
    state.stack_objects.push_back(stack_obj);
    // CR 602.2b -> 601.2i / CR 117.3c: the activating player receives priority afterward.
    // (Neither "CR 602.2e" nor "CR 116.3b" exists.) (This handler is AP-gated above --
    // "activate only as a sorcery", CR 702.171a -- so this is an identity write today; it
    // is written as `player` so the site stays correct if the gate is ever relaxed.)
    // CR 117.4: reset the pass-round.
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: mount,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
/// CR 603.4: evaluate a **card-definition** intervening-if at the moment the trigger
/// event occurs. `true` = queue the trigger; `false` = the ability does not trigger
/// at all.
///
/// `intervening_if` MUST be handed in by the caller, taken from the same
/// `AbilityDefinition::Triggered` the caller matched on. Do NOT re-derive it by
/// index inside this helper: the callers iterate three different index spaces
/// (`def.abilities`, `def.effective_abilities(is_transformed)`, and the runtime
/// vec), and face-awareness (CR 712.8d/e, PB-OS4b/PB-RS4) is inherited from
/// whichever list the caller walked.
///
/// The context mirrors `rules/resolution.rs:2160-2177` for the fields that exist
/// before the ability reaches the stack; `targets` is necessarily empty, which is
/// why `condition_is_queue_time_evaluable` exists.
pub(crate) fn carddef_intervening_if_holds_at_queue_time(
    state: &GameState,
    intervening_if: Option<&crate::cards::card_definition::Condition>,
    controller: PlayerId,
    source: ObjectId,
) -> bool {
    let Some(cond) = intervening_if else {
        return true;
    };
    if !crate::effects::condition_is_queue_time_evaluable(cond) {
        return true; // hard constraint 3: never suppress on an unanswerable condition
    }
    // CR 113.7a: several callers legitimately hold an LKI source (combat damage,
    // cast triggers, graveyard triggers) — `fizzle_object`, not a bare lookup
    // (SR-25 ratchet), and a vanished source simply contributes 0/0.
    let (kicker_times_paid, x_value) = state
        .fizzle_object(source)
        .map(|o| (o.kicker_times_paid, o.x_value))
        .unwrap_or((0, 0));
    let mut ctx = crate::effects::EffectContext::new_with_kicker(
        controller,
        source,
        vec![],
        kicker_times_paid,
    );
    ctx.x_value = x_value;
    crate::effects::check_condition(state, cond, &ctx)
}
/// When a CR 603.4 intervening-if is being evaluated. Not serialized, not hashed,
/// not on the wire — a pure call-site classification (PB-DX1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterveningIfMoment {
    /// CR 603.4 sentence 1, source still in the zone its ability functions in.
    TriggerTime,
    /// CR 603.4 sentence 1 for a **leave-the-battlefield** trigger (CR 603.10a).
    /// The source has already moved; the game must "look back in time" and the
    /// engine has no LKI-aware `check_condition`. A card-def condition is treated
    /// as HOLDING here (hard constraint 3: never suppress a trigger on a state we
    /// cannot query faithfully). Seeded as OOS-DX1-1.
    TriggerTimeLookBack,
    /// CR 603.4 sentence 2 — re-check as the ability resolves.
    Resolution,
    /// CR 603.4 sentence 2 for a **leave-the-battlefield** trigger (CR 603.10a).
    /// PB-DX1 review Finding 2: CR 603.4 explicitly says the intervening-if
    /// mechanism "mirrors the check for legal targets" (CR 608.2b), and 608.2b is
    /// unambiguous that a departed source's last-known-information is used, not
    /// current state. Evaluating a source-scoped condition (`SourceOnBattlefield`,
    /// ...) against the CURRENT state at resolution would read false for a source
    /// that has legitimately left the zone its ability functions in — the same
    /// false-negative failure `TriggerTimeLookBack` exists to prevent, just one
    /// step later ("queue-then-fizzle": PB-DP6's review named this exact shape).
    /// Treated as HOLDING, matching `TriggerTimeLookBack` and the pre-existing
    /// `InterveningIf::SourceHadNoCounterOfType` precedent (which also answers
    /// `true` at resolution rather than re-deriving from an LKI snapshot the
    /// caller cannot supply). Threaded from `resolution.rs` when the resolving
    /// ability's `trigger_on` is `SelfDies` / `SelfLeavesBattlefield` /
    /// `SourceConnives` — the same three `TriggerEvent`s the 8 `TriggerTimeLookBack`
    /// queue sites cover.
    ResolutionLookBack,
}

/// Evaluate an intervening-if condition against the current game state (CR 603.4).
///
/// `pre_death_counters` — counters captured from the creature just before it left
/// the battlefield. Required for `SourceHadNoCounterOfType` checks (persist/undying).
/// Pass `None` for all non-death trigger contexts.
///
/// `source` and `moment` are PB-DX1 additions for the `InterveningIf::CardDef` arm:
/// `source` feeds `carddef_intervening_if_holds_at_queue_time`'s `fizzle_object` /
/// kicker/x-value lookup; `moment` selects which of CR 603.4's two sentences (and
/// CR 603.10a's look-back carve-out) applies at this call site.
///
/// `resolution_targets` is a PB-DX1 review (Finding 6) addition: the resolving
/// stack object's declared targets, read by `Condition::TargetIsLegal` — the one
/// `Condition` variant that reads `ctx.targets`. At `TriggerTime`/
/// `TriggerTimeLookBack` no targets have been declared yet (targets are chosen
/// when a trigger is placed on the stack, not when it is collected), so every
/// queue-time caller correctly passes `&[]`; only the `Resolution` arm reads this
/// parameter for real. Mirrors `resolution.rs`'s registry-path re-check, which
/// already threads `stack_obj.targets.clone()`.
pub fn check_intervening_if(
    state: &GameState,
    cond: &InterveningIf,
    controller: PlayerId,
    source: ObjectId,
    pre_death_counters: Option<&imbl::OrdMap<crate::state::types::CounterType, u32>>,
    moment: InterveningIfMoment,
    resolution_targets: &[SpellTarget],
) -> bool {
    match cond {
        InterveningIf::ControllerLifeAtLeast(n) => state
            .expect_player(controller)
            .map(|p| p.life_total >= *n as i32)
            .unwrap_or(false),
        // CR 702.79a / CR 702.93a: "if it had no [counter type] counters on it"
        // Checked against last-known-information (pre-death counters) at trigger time.
        // At resolution time, caller passes None; the condition is treated as true
        // (the MoveZone effect will silently no-op if the source left the graveyard).
        InterveningIf::SourceHadNoCounterOfType(ct) => pre_death_counters
            .map(|counters| !counters.contains_key(ct))
            .unwrap_or(true),
        // PB-DX1 (CR 603.4, OOS-DP6-1): the card-def condition, carried through the
        // lowering by `build_face_ability_vectors`.
        InterveningIf::CardDef(c) => match moment {
            // CR 603.10a: the source has already left the battlefield and
            // `check_condition` has no LKI-aware evaluation path -- evaluating a
            // source-scoped condition (SourceOnBattlefield, SourceHasCounters, ...)
            // against the CURRENT state would read false and wrongly suppress a
            // trigger CR 603.4 requires to fire. Queue unconditionally. The
            // `ResolutionLookBack` arm below is the SAME carve-out applied at the
            // resolution end (review Finding 2) -- treating this arm's "true" as
            // sufficient and letting a real re-check run at resolution would have
            // been queue-then-fizzle, not a functioning carve-out.
            InterveningIfMoment::TriggerTimeLookBack => true,
            InterveningIfMoment::TriggerTime => {
                carddef_intervening_if_holds_at_queue_time(state, Some(c), controller, source)
            }
            // CR 603.4 s2 / CR 603.10a (review Finding 2): the resolution-time
            // counterpart of `TriggerTimeLookBack`. See the `InterveningIfMoment`
            // doc comment for the CR 608.2b mirroring argument. Not `Resolution`'s
            // evaluability-guard-then-`check_condition` shape -- unconditionally
            // true, matching `TriggerTimeLookBack` and the `SourceHadNoCounterOfType`
            // precedent above.
            InterveningIfMoment::ResolutionLookBack => true,
            InterveningIfMoment::Resolution => {
                // CR 603.4 sentence 2. The SAME evaluability guard as the queue end:
                // of `condition_is_queue_time_evaluable`'s seven `false` variants, six
                // (WasOverloaded/WasBargained/WasCleaved/EvidenceWasCollected/
                // GiftWasGiven/SacrificeFired) are ALSO unpropagated into a trigger's
                // resolution context (OOS-DP6-6), so gating on them here would be the
                // same false negative one step later. The seventh, TargetIsLegal, IS
                // answerable at resolution and is therefore over-conservative here —
                // deliberately, because CR 608.2b's all-targets-illegal fizzle at
                // `resolution.rs:2274` already removes exactly that ability, so nothing
                // is lost. Split seeded as OOS-DX1-2.
                //
                // PB-DX1 review Finding 6: `resolution_targets` is threaded from the
                // resolving `StackObject.targets` (see the doc comment above) so that
                // if OOS-DX1-2 is ever closed and `TargetIsLegal` becomes evaluable
                // here, `ctx.targets` is the REAL declared target list, not an empty
                // one that would turn every such trigger into a guaranteed false
                // negative.
                if !crate::effects::condition_is_queue_time_evaluable(c) {
                    return true;
                }
                // CR 113.7a: the source may be LKI (a leave-the-battlefield trigger
                // resolving) — `fizzle_object`, not a bare lookup (SR-25 ratchet).
                let (kicker_times_paid, x_value) = state
                    .fizzle_object(source)
                    .map(|o| (o.kicker_times_paid, o.x_value))
                    .unwrap_or((0, 0));
                let mut ctx = crate::effects::EffectContext::new_with_kicker(
                    controller,
                    source,
                    resolution_targets.to_vec(),
                    kicker_times_paid,
                );
                ctx.x_value = x_value;
                crate::effects::check_condition(state, c, &ctx)
            }
        },
    }
}
// ---------------------------------------------------------------------------
// Scavenge (CR 702.97)
// ---------------------------------------------------------------------------
/// Handle a ScavengeCard command: validate, pay cost, snapshot power, exile card,
/// push scavenge ability onto the stack targeting the specified creature.
///
/// CR 702.97a: Scavenge is an activated ability from the graveyard.
/// "[Cost], Exile this card from your graveyard: Put a number of +1/+1 counters
/// equal to the power of the card you exiled on target creature. Activate only
/// as a sorcery."
///
/// KEY RULE: Power is snapshotted BEFORE exile (Varolz ruling 2013-04-15 -- "the
/// number of counters that a card's scavenge ability puts on a creature is based on
/// the card's power as it last existed in the graveyard").
pub fn handle_scavenge_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    target_creature: ObjectId,
) -> Result<Vec<crate::rules::events::GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // 2. Split second check (CR 702.61a): activated abilities cannot be used when
    //    a spell with split second is on the stack.
    if casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; scavenge cannot be activated (CR 702.61a)"
                .into(),
        ));
    }
    // 3. Zone check (CR 702.97a): card must be in player's own graveyard.
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "ScavengeCard: card {:?} is not in Graveyard({:?}); scavenge can only be activated from your graveyard (CR 702.97a)",
                card, player
            )));
        }
    }
    // 4. Keyword check (CR 702.97a): card must have KeywordAbility::Scavenge.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Scavenge)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "ScavengeCard: card {:?} does not have the Scavenge keyword (CR 702.97a)",
                card
            )));
        }
    }
    // 5. Sorcery speed check (CR 702.97a: "activate only as a sorcery").
    //    Active player only, main phase only (PreCombatMain or PostCombatMain), empty stack.
    {
        use crate::state::turn::Step;
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: scavenge can only be activated during your own turn (CR 702.97a)"
                    .into(),
            ));
        }
        let step = state.turn.step;
        if step != Step::PreCombatMain && step != Step::PostCombatMain {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: scavenge can only be activated during a main phase (CR 702.97a)"
                    .into(),
            ));
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: scavenge can only be activated with an empty stack (CR 702.97a)"
                    .into(),
            ));
        }
    }
    // 6. Target validation: target_creature must be a creature on the battlefield.
    {
        let target_on_battlefield = state
            .objects
            .get(&target_creature)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false);
        if !target_on_battlefield {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: target_creature is not on the battlefield (CR 702.97a)".into(),
            ));
        }
        let target_is_creature =
            crate::rules::layers::calculate_characteristics(state, target_creature)
                .map(|c| c.card_types.contains(&CardType::Creature))
                .unwrap_or(false);
        if !target_is_creature {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: target_creature is not a creature (CR 702.97a)".into(),
            ));
        }
    }
    // 7. Look up scavenge cost from CardRegistry.
    let card_id_opt = state.object(card)?.card_id.clone();
    let scavenge_cost = match get_scavenge_cost(&card_id_opt, &state.card_registry.clone()) {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: no scavenge cost found in card definition (CR 702.97a)".into(),
            ));
        }
    };
    // 8. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if scavenge_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &scavenge_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &scavenge_cost);
        events.push(crate::rules::events::GameEvent::ManaCostPaid {
            player,
            cost: scavenge_cost.clone(),
        });
    }
    // 9. Snapshot power BEFORE exile (Varolz ruling 2013-04-15: "the number of counters
    //    is based on the card's power as it last existed in the graveyard").
    //    Use layer-resolved characteristics to capture any in-graveyard modifiers.
    let power_snapshot: u32 = crate::rules::layers::calculate_characteristics(state, card)
        .and_then(|c| c.power)
        .map(|p| p.max(0) as u32)
        .unwrap_or(0);
    // Capture source_card_id BEFORE exiling (registry key survives zone change, CR 400.7).
    let source_card_id = state.object(card)?.card_id.clone();
    // 10. Exile the card from graveyard as cost payment (CR 702.97a: "[Cost], Exile this
    //     card from your graveyard"). The card is exiled immediately at activation time.
    //     Ruling 2013-04-15: "Once the ability is activated and the cost is paid, it's too
    //     late to stop the ability by trying to remove the card from the graveyard."
    let (exile_id, _old) = state.move_object_to_zone(card, ZoneId::Exile)?;
    events.push(crate::rules::events::GameEvent::ObjectExiled {
        player,
        object_id: card,
        new_exile_id: exile_id,
        pre_lba_counters: imbl::OrdMap::new(), // graveyard→exile: no battlefield counters
        pre_lba_power: None,                   // graveyard→exile: no battlefield power to snapshot
    });
    // 11. Push the ScavengeAbility onto the stack with the target creature.
    // MR-TC-25: use trigger_default; override targets with the scavenge target.
    let stack_id = state.next_object_id();
    let mut stack_obj = StackObject::trigger_default(
        stack_id,
        player,
        StackObjectKind::ScavengeAbility {
            source_card_id,
            power_snapshot,
        },
    );
    stack_obj.targets = vec![SpellTarget {
        target: Target::Object(target_creature),
        zone_at_cast: Some(ZoneId::Battlefield),
    }];
    // PB-DX25c §3.1: Scavenge's "target creature" is validated ad-hoc above (step 6),
    // NOT through a `TargetRequirement` — there is nothing accurate to record here.
    state.stack_objects.push_back(stack_obj);
    // 12. CR 602.2b -> 601.2i / CR 117.3c: the activating player gets priority (CR 117.4:
    //     reset the pass-round). (This handler is AP-gated above -- "activate only as a
    //     sorcery", CR 702.97a -- so this is an identity write today; it is written as
    //     `player` so the site stays correct if the gate is ever relaxed.)
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(crate::rules::events::GameEvent::AbilityActivated {
        player,
        source_object_id: card,
        stack_object_id: stack_id,
    });
    // ENG-2 (A12, CR 602.2b): announce the scavenge ability's target creature.
    crate::rules::events::push_target_announcement(state, &mut events, player, card, stack_id);
    events.push(crate::rules::events::GameEvent::PriorityGiven { player });
    Ok(events)
}
/// CR 702.97a: Look up the scavenge cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Scavenge { cost }`, or `None`
/// if the card has no definition or no scavenge ability defined.
fn get_scavenge_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| match a {
                AbilityDefinition::Scavenge { cost } => Some(cost.clone()),
                _ => None,
            })
        })
    })
}
// ── Self-activated-cost-reduction helpers ─────────────────────────────────────
/// CR 602.2b + 601.2f: Look up the `SelfActivatedCostReduction` for an activated ability.
///
/// `ability_index` is the index into `characteristics.activated_abilities`, which corresponds
/// to the same index in `CardDefinition.activated_ability_cost_reductions` (keyed by ability index).
/// Channel lands: mana tap abilities go into `mana_abilities`, not `activated_abilities`,
/// so the channel ability at activated_ability index 0 maps to the first (and only) entry
/// with key 0 in `activated_ability_cost_reductions`.
///
/// # PB-S-L05 invariant (CR 601.2f + CR 613.1f)
///
/// This function is keyed by the NATIVE printed ability index. Layer 6 grants
/// (`LayerModification::AddActivatedAbility`) append abilities past the native range, so
/// any `ability_index >= <native count>` corresponds to a granted ability for which no
/// native cost reduction applies — this function correctly returns `None` for those indices.
///
/// A runtime debug_assert is not feasible to verify this invariant: the native ability count
/// includes both `AbilityDefinition::Activated` entries in `card_def.abilities` AND
/// `ObjectSpec::with_activated_ability()` entries installed at object-creation time (used by
/// some token specs and tests), so `card_def.abilities` alone does not reflect the full
/// native count.
///
/// Deferred: if a future card def adds an `activated_ability_cost_reductions` entry at an
/// index that collides with a Layer 6 grant's index, refactor to use a stable ability
/// identifier instead of a numeric index (see `docs/mtg-engine-low-issues-remediation.md`
/// PB-S-L05).
fn get_self_activated_reduction(
    card_def: &crate::cards::card_definition::CardDefinition,
    ability_index: usize,
) -> Option<crate::cards::card_definition::SelfActivatedCostReduction> {
    card_def
        .activated_ability_cost_reductions
        .iter()
        .find(|(idx, _)| *idx == ability_index)
        .map(|(_, r)| r.clone())
}
/// CR 602.2b + 601.2f: Evaluate a `SelfActivatedCostReduction` against the current game state.
///
/// Returns the number of generic mana to subtract. The caller uses `.saturating_sub()` to
/// ensure the generic component cannot go below 0 (CR 601.2f: "can't be reduced to less than {0}").
fn evaluate_self_activated_reduction(
    state: &crate::state::GameState,
    controller: crate::state::player::PlayerId,
    reduction: &crate::cards::card_definition::SelfActivatedCostReduction,
) -> u32 {
    use crate::cards::card_definition::{PlayerTarget, SelfActivatedCostReduction};
    match reduction {
        SelfActivatedCostReduction::PerPermanent {
            per,
            filter,
            controller: player_target,
        } => {
            // CR 602.2b: The relevant player for self-activated-cost-reduction is always
            // the activating player (controller). Other PlayerTarget values fall back to
            // controller since activated ability cost reduction is always self-referential.
            let target_player = match player_target {
                PlayerTarget::Controller => controller,
                _ => controller,
            };
            let count = state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == crate::state::zone::ZoneId::Battlefield
                        && obj.controller == target_player
                        && crate::effects::matches_filter(&obj.characteristics, filter)
                })
                .count();
            ((count as i32) * per).max(0) as u32
        }
    }
}
#[cfg(test)]
mod pb_dx35_trigger_modal_plan_tests {
    //! PB-DX35 (`OOS-DX4-2`) — t9: sites 1/2/D and site 3 (the CR 601.2c
    //! cross-slot distinctness check on the answer path) now share ONE
    //! arithmetic (`trigger_modal_plan`) rather than three-then-a-fourth
    //! hand-rolled copies. Sites 1/2/D are unifed *by construction* inside
    //! `flush_sorted` (they read the SAME `modal_plan` local, computed once
    //! per trigger) -- that cannot be probed externally to this module, since
    //! `flush_sorted` is a private function. Site 3
    //! (`trigger_ability_target_requirements`) is the fourth, independent
    //! reader (called from `handle_choose_trigger_targets`, on the answer
    //! path) and is where re-divergence is actually possible: this is an
    //! `#[cfg(test)]` unit test, not an integration test under
    //! `crates/engine/tests/`, because `trigger_ability_target_requirements`
    //! is a bare private `fn` -- not even `pub(crate)` -- so no external
    //! test in this crate can name it. `crates/engine/tests/primitives/
    //! pb_dx35_modal_trigger_targets.rs` t1-t8 cover the PUBLICLY observable
    //! behaviour (life totals, tokens, `modes_chosen`, CR 700.2b removal).
    use super::*;
    use crate::cards::card_definition::{
        CardDefinition, Effect, ModeSelection, TargetFilter, TargetRequirement,
    };
    use crate::cards::CardRegistry;
    use crate::state::builder::{GameStateBuilder, ObjectSpec};
    use crate::testing::replay_harness::enrich_spec_from_def;
    use std::collections::HashMap;

    fn p(n: u64) -> PlayerId {
        PlayerId(n)
    }

    /// A modal `WhenDies` trigger whose mode 0 targets a creature and whose
    /// mode 1 needs none -- the `shambling_ghast`/`retreat_to_kazandu` shape,
    /// registry-aligned (its ONLY ability, so registry index 0 == runtime
    /// index 0 -- this test is NOT about the index-space mismatch, which
    /// `core::pb_dx35_modal_trigger_roster::r2` covers separately).
    fn modal_subject() -> CardDefinition {
        CardDefinition {
            card_id: CardId("dx35-t9-modal-subject".to_string()),
            name: "DX35 T9 Modal Subject".to_string(),
            types: crate::cards::card_definition::TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "When this creature dies, choose one -- target creature you control \
                          gains indestructible; or another target creature you control gains \
                          lifelink."
                .to_string(),
            power: Some(1),
            toughness: Some(1),
            abilities: vec![AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    // BOTH modes target a creature you control (excluding this
                    // ability's own source, per the filter below), so with no
                    // OTHER creature on the battlefield neither mode has a
                    // legal candidate -- CR 700.2b removal (case B).
                    modes: vec![Effect::Nothing, Effect::Nothing],
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    mode_targets: Some(vec![
                        vec![creature_you_control_excluding_self()],
                        vec![creature_you_control_excluding_self()],
                    ]),
                }),
                trigger_zone: None,
            }],
            ..Default::default()
        }
    }
    /// The shared target requirement both `modal_subject` modes use --
    /// factored out so the two mode slices are visibly the SAME requirement
    /// rather than two independently-typed ones that happen to look alike.
    fn creature_you_control_excluding_self() -> TargetRequirement {
        TargetRequirement::TargetCreatureWithFilter(TargetFilter {
            controller: TargetController::You,
            // Excludes the trigger's own source: this is a WhenDies trigger
            // and the synthetic fixture never actually kills the subject (it
            // just constructs a PendingTrigger directly while the subject is
            // still on the battlefield), so without this the subject itself
            // would satisfy "target creature you control" and case B's "no
            // legal candidate" premise would be false.
            exclude_self: true,
            ..Default::default()
        })
    }

    /// t9: `trigger_ability_target_requirements` (site 3, the CR 601.2c
    /// cross-slot distinctness check's own re-derivation on the answer path)
    /// returns the SAME value as `trigger_modal_plan(..).requirements` (the
    /// value sites 1/2/D thread through `flush_sorted`) -- for both the
    /// legal-mode-0 case and the no-legal-mode (min_modes: 1) CR 700.2b
    /// removal case. A hand-rolled fourth copy that quietly re-diverged from
    /// the other three would redden this the moment its answer differed.
    #[test]
    fn t9_site3_agrees_with_the_shared_plan_by_value() {
        let def = modal_subject();
        let defs = HashMap::from([(def.name.clone(), def.clone())]);

        // Case A: a legal creature exists (mode 0's target is satisfiable).
        let spec_a = enrich_spec_from_def(
            ObjectSpec::creature(p(1), &def.name, 1, 1).with_card_id(def.card_id.clone()),
            &defs,
        );
        let ally = ObjectSpec::creature(p(1), "DX35 T9 Ally", 1, 1);
        let state_a = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(CardRegistry::new(vec![def.clone()]))
            .active_player(p(1))
            .object(spec_a)
            .object(ally)
            .build()
            .expect("PB-DX35 t9 case A fixture must build");
        let subject_a = state_a
            .objects()
            .values()
            .find(|o| o.characteristics.name == def.name)
            .expect("subject must be on the battlefield");
        let trigger_a = PendingTrigger {
            ability_index: 0,
            ..PendingTrigger::blank(subject_a.id, p(1), PendingTriggerKind::Normal)
        };
        let plan_a = trigger_modal_plan(&state_a, &trigger_a)
            .expect("a legal mode exists -- CR 700.2b must not remove the trigger");
        let site3_a = trigger_ability_target_requirements(&state_a, &trigger_a);
        assert_eq!(
            site3_a, plan_a.requirements,
            "site 3 must agree with the shared plan by value (legal-mode case)"
        );
        assert_eq!(
            plan_a.modes_chosen,
            vec![0],
            "non-vacuity: mode 0 must actually have been the one chosen"
        );
        assert!(
            !plan_a.requirements.is_empty(),
            "non-vacuity: mode 0's requirement (a target creature) must be present"
        );

        // Case B: NO legal creature (mode 0 illegal) and min_modes: 1 -- CR
        // 700.2b removal. `trigger_ability_target_requirements` must fail
        // open to an empty list (it has no `None` return of its own), and
        // `trigger_modal_plan` must return `None`.
        let spec_b = enrich_spec_from_def(
            ObjectSpec::creature(p(1), &def.name, 1, 1).with_card_id(def.card_id.clone()),
            &defs,
        );
        let state_b = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(CardRegistry::new(vec![def.clone()]))
            .active_player(p(1))
            .object(spec_b)
            .build()
            .expect("PB-DX35 t9 case B fixture must build");
        let subject_b = state_b
            .objects()
            .values()
            .find(|o| o.characteristics.name == def.name)
            .expect("subject must be on the battlefield");
        let trigger_b = PendingTrigger {
            ability_index: 0,
            ..PendingTrigger::blank(subject_b.id, p(1), PendingTriggerKind::Normal)
        };
        assert!(
            trigger_modal_plan(&state_b, &trigger_b).is_none(),
            "CR 700.2b: min_modes 1 with no legal mode must remove the trigger"
        );
        assert_eq!(
            trigger_ability_target_requirements(&state_b, &trigger_b),
            Vec::<TargetRequirement>::new(),
            "site 3 fails open to empty on the CR 700.2b-removed case"
        );

        // Case C: the SAME agreement on a `CardDefETB`-kind trigger.
        //
        // **↻ Added after this batch's own `/review` DEFEATED cases A and B by execution.**
        // Both drive `PendingTriggerKind::Normal` only, and `trigger_modal_plan` /
        // `trigger_ability_target_requirements` both branch on `trigger.kind`. The reviewer
        // re-planted the original `OOS-DX4-2` defect in site 3 behind
        // `if trigger.kind == PendingTriggerKind::CardDefETB` -- a hand-rolled fifth copy
        // reading the flat registry `targets` and ignoring `mode_targets` -- and the ENTIRE
        // `mtg-engine` crate stayed green, t9 included. **A differential probe proves agreement
        // on the branches it drives and nothing about the branches it does not**, which is the
        // same shape as PB-DX45's "a gate written for one variant measures that variant".
        //
        // The fixture is reused deliberately: `modal_subject`'s Triggered ability is its ONLY
        // ability, so registry index 0 == runtime index 0 and the two kinds address the same
        // ability. That is what makes the two arms comparable rather than merely both green.
        let trigger_c = PendingTrigger {
            ability_index: 0,
            ..PendingTrigger::blank(subject_a.id, p(1), PendingTriggerKind::CardDefETB)
        };
        let plan_c = trigger_modal_plan(&state_a, &trigger_c)
            .expect("a legal mode exists on the CardDefETB branch too");
        assert_eq!(
            trigger_ability_target_requirements(&state_a, &trigger_c),
            plan_c.requirements,
            "site 3 must agree with the shared plan by value on the CardDefETB branch as well \
             -- a branch-selective re-divergence is exactly what the `/review` planted"
        );
        assert_eq!(
            plan_c.requirements, plan_a.requirements,
            "non-vacuity: both kinds address the same ability on this fixture (its Triggered \
             ability is its only one, so registry index 0 == runtime index 0), so the two \
             branches must produce the SAME requirement list -- if they ever differ here, the \
             comparison above is passing for the wrong reason"
        );
    }
}
