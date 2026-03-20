//! Stack resolution (CR 608).
//!
//! When all players pass priority in succession with a non-empty stack,
//! the top of the stack resolves (CR 608.1 / LIFO).
//!
//! Instants and sorceries: card moves to owner's graveyard (CR 608.2n).
//! Permanents: card enters the battlefield under spell's controller (CR 608.3a).
//! After resolution: priority resets to the active player (CR 116.3b).
//!
//! **Fizzle rule (CR 608.2b)**: If ALL targets are illegal at resolution time,
//! the spell is removed from the stack and its card goes to the graveyard without
//! resolving (`SpellFizzled`). If only SOME targets are illegal (partial fizzle),
//! the spell resolves normally; illegal targets are unaffected (M7+).

use im::OrdSet;

use crate::effects::{check_condition, execute_effect, EffectContext};
use crate::state::error::GameStateError;
use crate::state::game_object::{Designations, MergedComponent, ObjectId};
use crate::state::stack::StackObjectKind;
use crate::state::stubs::PendingTriggerKind;
use crate::state::targeting::{SpellTarget, Target};
use crate::state::types::{
    AdditionalCost, AltCostKind, CardType, ChampionFilter, Color, CounterType, EnchantTarget,
    KeywordAbility, SubType,
};
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::abilities;
use super::events::GameEvent;
use super::sba;

/// CR 608.1: Resolve the top object on the stack.
///
/// Called when all players pass priority in succession with a non-empty stack.
/// The top object (last in `stack_objects`) resolves via LIFO ordering.
/// After resolution, the active player receives priority (CR 116.3b).
pub fn resolve_top_of_stack(state: &mut GameState) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    // Pop the top of the stack (LIFO — last pushed, first resolved).
    let stack_obj = state
        .stack_objects
        .pop_back()
        .ok_or_else(|| GameStateError::InvalidCommand("stack is empty".into()))?;

    match stack_obj.kind.clone() {
        StackObjectKind::Spell { source_object } => {
            let controller = stack_obj.controller;

            // CR 608.2b: Check target legality before resolving.
            // CR 702.103e / 608.3b: Bestowed Aura spells with all-illegal targets revert
            // to creature spells instead of fizzling.
            let targets = &stack_obj.targets;
            let bestow_fallback = if !targets.is_empty() {
                let legal_count = targets.iter().filter(|t| is_target_legal(state, t)).count();

                if legal_count == 0 {
                    if stack_obj.was_bestowed {
                        // CR 702.103e / 608.3b: Bestowed Aura with illegal target ceases
                        // to be bestowed and resolves as a creature spell. Revert the
                        // type transformation on the source object.
                        if let Some(source_obj) = state.objects.get_mut(&source_object) {
                            source_obj
                                .characteristics
                                .subtypes
                                .remove(&SubType("Aura".to_string()));
                            source_obj
                                .characteristics
                                .keywords
                                .remove(&KeywordAbility::Enchant(EnchantTarget::Creature));
                            source_obj
                                .characteristics
                                .card_types
                                .insert(CardType::Creature);
                        }
                        // Fall through to permanent resolution; targets cleared below.
                        true
                    } else {
                        // CR 608.2b: All targets illegal — fizzle.
                        // Card goes to graveyard without effect (same zone move as normal
                        // instant/sorcery resolution, but emits SpellFizzled, not SpellResolved).
                        //
                        // CR 702.34a: If cast with flashback, the card is exiled instead of
                        // going to the graveyard — this applies even on fizzle.
                        // CR 702.133a: If cast with jump-start, the card is also exiled on fizzle.
                        //
                        // CR 707.10: Copies have no physical card to move — skip zone move.
                        let fizzle_source_id = if stack_obj.is_copy {
                            source_object
                        } else {
                            let owner = state.object(source_object)?.owner;
                            let destination = if stack_obj.cast_with_flashback
                                || stack_obj.cast_with_jump_start
                                || stack_obj.is_cast_transformed
                            {
                                // CR 702.34a / CR 702.133a: Flashback and jump-start exile on fizzle.
                                // CR 702.146c: Disturb spells (is_cast_transformed) also exile on fizzle.
                                ZoneId::Exile
                            } else {
                                ZoneId::Graveyard(owner)
                            };
                            let (new_id, _old) =
                                state.move_object_to_zone(source_object, destination)?;
                            new_id
                        };

                        events.push(GameEvent::SpellFizzled {
                            player: controller,
                            stack_object_id: stack_obj.id,
                            source_object_id: fizzle_source_id,
                        });

                        // CR 704.3: Check SBAs before granting priority.
                        let sba_evts = sba::check_and_apply_sbas(state);
                        events.extend(sba_evts);

                        // Priority resets to active player after fizzle.
                        state.turn.players_passed = OrdSet::new();
                        let active = state.turn.active_player;
                        state.turn.priority_holder = Some(active);
                        events.push(GameEvent::PriorityGiven { player: active });

                        return Ok(events);
                    }
                } else {
                    false
                }
            } else {
                false
            };
            // Partial fizzle (some targets illegal): spell resolves normally.
            // Illegal targets will be unaffected when effects are implemented (M7+).

            // Determine destination zone based on card type (CR 608.2n vs 608.3).
            let (card_types, owner, card_id) = {
                let card = state.object(source_object)?;
                (
                    card.characteristics.card_types.clone(),
                    card.owner,
                    card.card_id.clone(),
                )
            };

            let is_permanent = card_types.iter().any(|t| {
                matches!(
                    t,
                    CardType::Creature
                        | CardType::Artifact
                        | CardType::Enchantment
                        | CardType::Planeswalker
                        | CardType::Battle
                )
            });

            // CR 608.2: Execute the card's effect before it moves to its final zone.
            // Look up the CardDefinition from the registry (if available) and run its Spell effect.
            // registry is also used below for self-ETB replacements (CR 614.15).
            let registry = state.card_registry.clone();
            {
                if let Some(cid) = card_id.clone() {
                    if let Some(def) = registry.get(cid) {
                        // CR 702.102d: If this is a fused split spell, execute the left half
                        // first, then the right half, in order. Targets for the left half
                        // come from the full target list; targets for the right half also
                        // use the same list (since we don't split targets in this impl).
                        if stack_obj
                            .additional_costs
                            .iter()
                            .any(|c| matches!(c, AdditionalCost::Fuse))
                        {
                            let left_effect = def.abilities.iter().find_map(|a| {
                                if let crate::cards::card_definition::AbilityDefinition::Spell {
                                    effect,
                                    ..
                                } = a
                                {
                                    Some(effect.clone())
                                } else {
                                    None
                                }
                            });
                            let right_effect = def.abilities.iter().find_map(|a| {
                                if let crate::cards::card_definition::AbilityDefinition::Fuse {
                                    effect,
                                    ..
                                } = a
                                {
                                    Some(effect.clone())
                                } else {
                                    None
                                }
                            });
                            // Target index contract for fused split spells (CR 702.102):
                            // The combined target list is shared by both halves using a
                            // single EffectContext. Left-half targets occupy indices
                            // 0..left_target_count; right-half targets follow at indices
                            // left_target_count.. . Card definition authors must ensure
                            // `DeclaredTarget { index: N }` uses globally-offset indices —
                            // e.g., if the left half declares one target at index 0, the
                            // right half's first target must use index 1, not index 0.
                            let legal_targets: Vec<SpellTarget> = stack_obj
                                .targets
                                .iter()
                                .filter(|t| is_target_legal(state, t))
                                .cloned()
                                .collect();
                            let mut ctx = EffectContext::new_with_kicker(
                                controller,
                                source_object,
                                legal_targets,
                                stack_obj.kicker_times_paid,
                            );
                            ctx.was_overloaded = stack_obj.was_overloaded;
                            ctx.was_bargained = stack_obj.was_bargained;
                            ctx.was_cleaved = stack_obj.was_cleaved;
                            // CR 701.59c: Propagate collect evidence status to effect context.
                            ctx.evidence_collected = stack_obj.evidence_collected;
                            // CR 107.3m: Propagate X value to effect context.
                            ctx.x_value = stack_obj.x_value;
                            // CR 702.102d: Execute left half first.
                            if let Some(eff) = left_effect {
                                let eff_events = execute_effect(state, &eff, &mut ctx);
                                events.extend(eff_events);
                            }
                            // CR 702.102d: Execute right half second.
                            if let Some(eff) = right_effect {
                                let eff_events = execute_effect(state, &eff, &mut ctx);
                                events.extend(eff_events);
                            }
                        } else {
                            // CR 702.127a + CR 709.3b: If the aftermath half was cast, use the
                            // aftermath effect instead of the first-half Spell effect.
                            // CR 702.42b: For entwined modal spells, collect the modes so we
                            // can execute all of them (or just mode[0] when not entwined).
                            let (spell_effect, spell_modes) = if stack_obj.cast_with_aftermath {
                                let eff = def.abilities.iter().find_map(|a| {
                                if let crate::cards::card_definition::AbilityDefinition::Aftermath {
                                    effect,
                                    ..
                                } = a
                                {
                                    Some(effect.clone())
                                } else {
                                    None
                                }
                            });
                                (eff, None)
                            } else {
                                def.abilities.iter().find_map(|a| {
                                if let crate::cards::card_definition::AbilityDefinition::Spell {
                                    effect,
                                    modes,
                                    ..
                                } = a
                                {
                                    Some((effect.clone(), modes.clone()))
                                } else {
                                    None
                                }
                            }).map(|(e, m)| (Some(e), m)).unwrap_or((None, None))
                            };
                            if spell_effect.is_some() || spell_modes.is_some() {
                                // CR 608.2b: Partial fizzle — filter out illegal targets before
                                // executing effects. Illegal targets are simply skipped; they are
                                // not affected by the spell's effect. Full fizzle (all illegal)
                                // is handled above before we reach this point.
                                let legal_targets: Vec<SpellTarget> = stack_obj
                                    .targets
                                    .iter()
                                    .filter(|t| is_target_legal(state, t))
                                    .cloned()
                                    .collect();
                                // CR 702.33d: Pass kicker status to the effect context so
                                // Condition::WasKicked can be checked during resolution.
                                // CR 702.96a: Pass overload status so Condition::WasOverloaded works.
                                // CR 702.47b: Clone legal_targets before the move so splice effects
                                // can use the same target list as the main spell.
                                let legal_targets_for_splice = legal_targets.clone();

                                // CR 700.2 / CR 702.42b / CR 702.120a: Mode dispatch.
                                // Priority order:
                                // 1. Entwine takes precedence — all modes in printed order (CR 702.42b).
                                // 2. Explicit modes_chosen — execute chosen modes in index order (CR 700.2a).
                                // 3. Escalate backward compat — execute modes 0..=escalate_modes_paid.
                                // 4. Auto-select mode[0] (default for bots/backward compat).
                                // If no modes, execute the spell's main effect as before.
                                let effects_to_run: Vec<crate::cards::card_definition::Effect> =
                                    if let Some(modes) = spell_modes {
                                        if stack_obj
                                            .additional_costs
                                            .iter()
                                            .any(|c| matches!(c, AdditionalCost::Entwine))
                                        {
                                            // CR 702.42b: "follow the text of each of the modes in
                                            // the order written on the card"
                                            modes.modes.clone()
                                        } else if !stack_obj.modes_chosen.is_empty() {
                                            // CR 700.2a: execute the explicitly chosen modes in index
                                            // order. Invalid indices are silently skipped (validated
                                            // at cast time in casting.rs).
                                            stack_obj
                                                .modes_chosen
                                                .iter()
                                                .filter_map(|&idx| modes.modes.get(idx).cloned())
                                                .collect()
                                        } else if stack_obj
                                            .additional_costs
                                            .iter()
                                            .find_map(|c| match c {
                                                AdditionalCost::EscalateModes { count } => {
                                                    Some(*count)
                                                }
                                                _ => None,
                                            })
                                            .unwrap_or(0)
                                            > 0
                                        {
                                            // CR 702.120a: backward compat — escalate without explicit
                                            // modes_chosen executes modes 0..=escalate_modes_paid.
                                            let count = (stack_obj
                                                .additional_costs
                                                .iter()
                                                .find_map(|c| match c {
                                                    AdditionalCost::EscalateModes { count } => {
                                                        Some(*count)
                                                    }
                                                    _ => None,
                                                })
                                                .unwrap_or(0)
                                                as usize
                                                + 1)
                                            .min(modes.modes.len());
                                            modes.modes[..count].to_vec()
                                        } else {
                                            // Auto-select first mode (default for bots and backward
                                            // compat with existing scripts/tests).
                                            modes.modes.into_iter().take(1).collect()
                                        }
                                    } else if let Some(effect) = spell_effect {
                                        vec![effect]
                                    } else {
                                        vec![]
                                    };

                                let mut ctx = EffectContext::new_with_kicker(
                                    controller,
                                    source_object,
                                    legal_targets,
                                    stack_obj.kicker_times_paid,
                                );
                                ctx.was_overloaded = stack_obj.was_overloaded;
                                // CR 702.166b: Pass bargained status so Condition::WasBargained works.
                                ctx.was_bargained = stack_obj.was_bargained;
                                // CR 702.148a: Pass cleave status so Condition::WasCleaved works.
                                ctx.was_cleaved = stack_obj.was_cleaved;
                                // CR 701.59c: Pass collect evidence status so Condition::EvidenceWasCollected works.
                                ctx.evidence_collected = stack_obj.evidence_collected;
                                // CR 107.3m: Propagate X value to effect context.
                                ctx.x_value = stack_obj.x_value;
                                // RC-1: Extract gift status from additional_costs.
                                let ac_gift_opponent: Option<crate::state::PlayerId> =
                                    stack_obj.additional_costs.iter().find_map(|c| match c {
                                        AdditionalCost::Gift { opponent } => Some(*opponent),
                                        _ => None,
                                    });
                                let ac_gift_was_given = ac_gift_opponent.is_some();
                                // CR 702.174b: Pass gift status so Condition::GiftWasGiven works.
                                ctx.gift_was_given = ac_gift_was_given;
                                ctx.gift_opponent = ac_gift_opponent;

                                // CR 702.174j: For instant/sorcery spells, the gift effect always
                                // happens BEFORE any other spell abilities of the card.
                                // Only execute if gift cost was paid AND it's an instant/sorcery.
                                if ac_gift_was_given && !is_permanent {
                                    if let Some(opponent) = ac_gift_opponent {
                                        // Look up the gift type from the card definition.
                                        // `def` is already in scope from the enclosing
                                        // `if let Some(def) = registry.get(cid)` block.
                                        let gift_type_opt = def.abilities.iter().find_map(|a| {
                                            if let crate::cards::card_definition::AbilityDefinition::Gift {
                                                gift_type,
                                            } = a
                                            {
                                                Some(gift_type.clone())
                                            } else {
                                                None
                                            }
                                        });
                                        if let Some(gift_type) = gift_type_opt {
                                            let gift_events = execute_gift_effect(
                                                state, opponent, controller, &gift_type,
                                            );
                                            events.extend(gift_events);
                                        }
                                    }
                                }

                                // CR 702.42b: Execute each effect in order. For entwined spells,
                                // state changes from earlier modes are visible to later modes.
                                for effect in &effects_to_run {
                                    let effect_events = execute_effect(state, effect, &mut ctx);
                                    events.extend(effect_events);
                                }

                                // CR 702.47b: Execute spliced effects after the main spell effect.
                                // "The effects of the main spell must happen first." (CR 702.47b)
                                // Each spliced effect uses the same resolution context (controller,
                                // source_object) per CR 702.47c: text gained refers to the spell,
                                // not the spliced card.
                                for spliced_effect in &stack_obj.spliced_effects {
                                    let mut splice_ctx = EffectContext::new_with_kicker(
                                        controller,
                                        source_object,
                                        legal_targets_for_splice.clone(),
                                        stack_obj.kicker_times_paid,
                                    );
                                    splice_ctx.was_overloaded = stack_obj.was_overloaded;
                                    splice_ctx.was_bargained = stack_obj.was_bargained;
                                    splice_ctx.was_cleaved = stack_obj.was_cleaved;
                                    // CR 701.59c: Propagate collect evidence status to splice context.
                                    splice_ctx.evidence_collected = stack_obj.evidence_collected;
                                    // CR 107.3m: Propagate X value to splice context.
                                    splice_ctx.x_value = stack_obj.x_value;
                                    let splice_events =
                                        execute_effect(state, spliced_effect, &mut splice_ctx);
                                    events.extend(splice_events);
                                }
                            }
                        } // close else { (non-fuse path)
                    }
                }
            }

            // CR 702.131a: Ascend on an instant or sorcery is a spell ability.
            // "If you control ten or more permanents and you don't have the city's
            // blessing, you get the city's blessing for the rest of the game."
            // Checked at resolution time (after effects execute), not at cast time.
            // Note: uses raw characteristics.keywords for the stack spell (not layer-
            // computed) because the spell is on the stack, not the battlefield.
            {
                let has_ascend = state
                    .objects
                    .get(&source_object)
                    .map(|obj| {
                        obj.characteristics
                            .keywords
                            .contains(&KeywordAbility::Ascend)
                    })
                    .unwrap_or(false);
                if has_ascend {
                    let already_has = state
                        .players
                        .get(&controller)
                        .map(|p| p.has_citys_blessing)
                        .unwrap_or(true);
                    if !already_has {
                        let permanent_count = state
                            .objects
                            .values()
                            .filter(|o| o.zone == ZoneId::Battlefield && o.controller == controller)
                            .count();
                        if permanent_count >= 10 {
                            if let Some(p) = state.players.get_mut(&controller) {
                                p.has_citys_blessing = true;
                            }
                            events.push(GameEvent::CitysBlessingGained { player: controller });
                        }
                    }
                }
            }

            // CR 707.10: Copies of spells are not real cards — they don't move to
            // a destination zone when they resolve.  The source_object belongs to
            // the original spell and must not be moved by a copy's resolution.
            if stack_obj.is_copy {
                // Copy resolves: execute the effect, then emit SpellResolved.
                // source_object is the original card (still in Stack zone for now).
                events.push(GameEvent::SpellResolved {
                    player: controller,
                    stack_object_id: stack_obj.id,
                    source_object_id: source_object,
                });
            } else if is_permanent {
                // CR 608.3a: Permanent spell — card enters the battlefield under
                // the spell's controller's control.
                //
                // CR 708.2 / CR 702.37a: If the spell was cast face-down via morph, megamorph,
                // or disguise, the permanent enters the battlefield face-down. Capture the
                // face_down_as value from the source object BEFORE move_object_to_zone clears it
                // (CR 400.7: zone change creates a new object; face_down_as resets to None).
                let morph_face_down_kind: Option<crate::state::types::FaceDownKind> = state
                    .objects
                    .get(&source_object)
                    .and_then(|o| o.face_down_as.clone());
                let (new_id, _old) =
                    state.move_object_to_zone(source_object, ZoneId::Battlefield)?;

                // CR 608.3a: "under the control of the spell's controller"
                // (move_object_to_zone resets controller to owner; restore it here).
                // CR 702.33d: Transfer kicked status from stack to permanent so ETB
                // triggers can check Condition::WasKicked.
                // CR 702.74a: Transfer evoked status from stack to permanent so the
                // ETB sacrifice trigger can check was_evoked.
                if let Some(obj) = state.objects.get_mut(&new_id) {
                    obj.controller = controller;
                    // CR 702.37a / CR 708.2a: Restore face-down status for morph/megamorph/disguise
                    // permanents. The permanent enters the battlefield face-down as a 2/2 with no
                    // name, subtypes, text, colors, or other abilities (CR 708.2a). move_object_to_zone
                    // clears face_down_as (CR 400.7), so we restore it here before any ETB processing.
                    if let Some(ref kind) = morph_face_down_kind {
                        obj.status.face_down = true;
                        obj.face_down_as = Some(kind.clone());
                    }
                    obj.kicker_times_paid = stack_obj.kicker_times_paid;
                    // CR 702.166b: Transfer bargained status from stack to permanent so ETB
                    // triggers can check Condition::WasBargained.
                    obj.was_bargained = stack_obj.was_bargained;
                    // CR 701.59c: Transfer collect evidence status from stack to permanent so
                    // ETB triggers can check Condition::EvidenceWasCollected.
                    obj.evidence_collected = stack_obj.evidence_collected;
                    // CR 107.3m: Transfer X value from stack to permanent for ETB replacement
                    // effects and triggers that reference X (e.g., Ravenous CR 702.156a).
                    obj.x_value = stack_obj.x_value;
                    // CR 702.157a: Transfer squad count from stack to permanent.
                    obj.squad_count = stack_obj
                        .additional_costs
                        .iter()
                        .find_map(|c| match c {
                            AdditionalCost::Squad { count } => Some(*count),
                            _ => None,
                        })
                        .unwrap_or(0);
                    // CR 702.175a: Transfer offspring_paid from stack to permanent.
                    obj.offspring_paid = stack_obj
                        .additional_costs
                        .iter()
                        .any(|c| matches!(c, AdditionalCost::Offspring));
                    // CR 702.174a: Transfer gift status from stack to permanent.
                    let gift_opp_for_obj: Option<crate::state::PlayerId> =
                        stack_obj.additional_costs.iter().find_map(|c| match c {
                            AdditionalCost::Gift { opponent } => Some(*opponent),
                            _ => None,
                        });
                    obj.gift_was_given = gift_opp_for_obj.is_some();
                    obj.gift_opponent = gift_opp_for_obj;
                    // CR 702.74a: Transfer evoked status from stack to permanent so the
                    // ETB sacrifice trigger can check cast_alt_cost.
                    // CR 702.138b: Transfer escaped status. A permanent "escaped" if cast
                    // using an escape ability (CR 702.138c/d).
                    // CR 702.109a: Transfer dashed status. "it has haste" and return at end step.
                    obj.cast_alt_cost = if stack_obj.was_evoked {
                        Some(AltCostKind::Evoke)
                    } else if stack_obj.was_escaped {
                        Some(AltCostKind::Escape)
                    } else if stack_obj.was_dashed {
                        Some(AltCostKind::Dash)
                    } else if stack_obj.was_blitzed {
                        Some(AltCostKind::Blitz)
                    } else if stack_obj.was_impended {
                        Some(AltCostKind::Impending)
                    } else if stack_obj.was_surged {
                        // CR 702.117a: Transfer surge status to permanent for "if surge cost was paid" effects.
                        Some(AltCostKind::Surge)
                    } else {
                        None
                    };
                    // CR 702.103b: Transfer bestowed status from stack to permanent.
                    // If bestow_fallback is true, the spell reverted to creature mode;
                    // the permanent enters as a creature (not as a bestowed Aura).
                    if stack_obj.was_bestowed && !bestow_fallback {
                        obj.designations.insert(Designations::BESTOWED);
                    }
                    // CR 702.146a / CR 712.11a: Transfer disturb/transform status from stack to permanent.
                    // When a card was cast with disturb, it enters transformed (back face up).
                    // The was_cast_disturbed flag enables the graveyard→exile replacement effect
                    // (CR 702.146b) when the permanent would later be put into a graveyard.
                    if stack_obj.is_cast_transformed {
                        obj.is_transformed = true;
                        // Disturb is the only current source of is_cast_transformed.
                        // Verify by checking the card has AbilityDefinition::Disturb.
                        let has_disturb_abil = obj
                            .card_id
                            .as_ref()
                            .and_then(|cid| registry.get(cid.clone()))
                            .map(|def| {
                                def.abilities.iter().any(|a| {
                                    matches!(
                                        a,
                                        crate::cards::card_definition::AbilityDefinition::Disturb { .. }
                                    )
                                })
                            })
                            .unwrap_or(false);
                        obj.was_cast_disturbed = has_disturb_abil;
                    }
                    if stack_obj.was_dashed {
                        // CR 702.109a: "it has haste" -- grant haste keyword.
                        obj.characteristics.keywords.insert(KeywordAbility::Haste);
                    }
                    if stack_obj.was_blitzed {
                        // CR 702.152a: "it has haste" -- grant haste keyword.
                        obj.characteristics.keywords.insert(KeywordAbility::Haste);
                        // CR 702.152a: "'When this permanent is put into a graveyard
                        // from the battlefield, draw a card.'" -- add SelfDies trigger.
                        // Uses standard TriggeredAbilityDef with inline Effect::DrawCards.
                        // Resolves through the standard TriggeredAbility resolution path
                        // (resolution.rs:620-679) when the creature dies via any path.
                        obj.characteristics.triggered_abilities.push(
                            crate::state::game_object::TriggeredAbilityDef {
                                etb_filter: None,
                                targets: vec![],
                                trigger_on: crate::state::game_object::TriggerEvent::SelfDies,
                                intervening_if: None,
                                description: "Blitz (CR 702.152a): When this permanent is \
                                              put into a graveyard from the battlefield, \
                                              draw a card."
                                    .to_string(),
                                effect: Some(crate::cards::card_definition::Effect::DrawCards {
                                    player: crate::cards::card_definition::PlayerTarget::Controller,
                                    count: crate::cards::card_definition::EffectAmount::Fixed(1),
                                }),
                            },
                        );
                    }
                    // CR 702.62a: If the spell was cast via suspend and the permanent
                    // is a creature, it gains haste (modelled as clearing summoning
                    // sickness; V1 simplification per plan). The "until you lose control"
                    // clause is deferred (V1: permanent effect for the casting player).
                    if stack_obj.was_suspended
                        && obj.characteristics.card_types.contains(&CardType::Creature)
                    {
                        obj.has_summoning_sickness = false;
                    }
                    // CR 718.3b: Transfer prototyped status from stack to permanent.
                    // The permanent uses the alternative P/T, mana cost, and color
                    // while on the battlefield (CR 718.4). These values were already
                    // set on the stack object's source card in casting.rs, and they
                    // carry over via move_object_to_zone (the characteristics are
                    // preserved). We also write them again here for correctness and to
                    // handle any edge cases where characteristics might be reset.
                    if stack_obj.was_prototyped {
                        obj.is_prototyped = true;
                        if let Some(proto_data) = crate::rules::casting::get_prototype_data(
                            &obj.card_id,
                            &state.card_registry,
                        ) {
                            let (proto_cost, proto_power, proto_toughness) = proto_data;
                            obj.characteristics.power = Some(proto_power);
                            obj.characteristics.toughness = Some(proto_toughness);
                            obj.characteristics.colors =
                                crate::rules::casting::colors_from_mana_cost(&proto_cost);
                            obj.characteristics.mana_cost = Some(proto_cost);
                        }
                    }
                    // CR 702.176a: "If you chose to pay this permanent's impending cost,
                    // it enters with N time counters on it."
                    // This is a replacement effect on ETB -- the permanent enters WITH
                    // the counters, not as a triggered ability after entering.
                    if stack_obj.was_impended {
                        let impending_count = crate::rules::casting::get_impending_count(
                            &obj.card_id,
                            &state.card_registry,
                        )
                        .unwrap_or(0);
                        if impending_count > 0 {
                            let current =
                                obj.counters.get(&CounterType::Time).copied().unwrap_or(0);
                            obj.counters = obj
                                .counters
                                .update(CounterType::Time, current + impending_count);
                        }
                    }
                    // CR 702.63a: "This permanent enters with N time counters on it."
                    // Vanishing N places N time counters on the permanent as it enters.
                    // Fires for ALL permanents with Vanishing, regardless of how they
                    // were cast (no alt-cost condition like Impending).
                    // CR 702.63b: Vanishing without a number (N=0) does NOT place counters.
                    // CR 702.63c: Multiple instances of Vanishing each work separately --
                    // sum all N values so Vanishing 3 + Vanishing 2 places 5 counters.
                    {
                        let total_vanishing: u32 = obj
                            .characteristics
                            .keywords
                            .iter()
                            .filter_map(|kw| {
                                if let crate::state::types::KeywordAbility::Vanishing(n) = kw {
                                    Some(*n)
                                } else {
                                    None
                                }
                            })
                            .sum();
                        if total_vanishing > 0 {
                            let current =
                                obj.counters.get(&CounterType::Time).copied().unwrap_or(0);
                            obj.counters = obj
                                .counters
                                .update(CounterType::Time, current + total_vanishing);
                        }
                    }
                    // CR 702.32a: "This permanent enters with N fade counters on it."
                    // Fading N places N fade counters on the permanent as it enters.
                    // Unlike Vanishing, Fading always has N >= 1 (no "Fading without a number").
                    {
                        let total_fading: u32 = obj
                            .characteristics
                            .keywords
                            .iter()
                            .filter_map(|kw| {
                                if let crate::state::types::KeywordAbility::Fading(n) = kw {
                                    Some(*n)
                                } else {
                                    None
                                }
                            })
                            .sum();
                        if total_fading > 0 {
                            let current =
                                obj.counters.get(&CounterType::Fade).copied().unwrap_or(0);
                            obj.counters = obj
                                .counters
                                .update(CounterType::Fade, current + total_fading);
                        }
                    }
                }

                // CR 702.30a: Mark permanents with Echo as pending their echo trigger.
                // "At the beginning of your upkeep, if this permanent came under your
                // control since the beginning of your last upkeep, sacrifice it unless
                // you pay [cost]." Setting echo_pending models the condition.
                if let Some(obj) = state.objects.get_mut(&new_id) {
                    if obj
                        .characteristics
                        .keywords
                        .iter()
                        .any(|kw| matches!(kw, crate::state::types::KeywordAbility::Echo(_)))
                    {
                        obj.designations.insert(Designations::ECHO_PENDING);
                    }
                }

                // CR 702.138c: "Escapes with [counter]" -- if this permanent escaped,
                // it enters the battlefield with the specified counters. This is a
                // replacement effect on ETB (not a triggered ability). Applied here
                // immediately after the permanent enters the battlefield.
                if stack_obj.was_escaped {
                    if let Some(cid) = card_id.clone() {
                        if let Some(def) = registry.get(cid) {
                            for ability in &def.abilities {
                                if let crate::cards::card_definition::AbilityDefinition::EscapeWithCounter {
                                    counter_type,
                                    count,
                                } = ability
                                {
                                    if let Some(obj) = state.objects.get_mut(&new_id) {
                                        let current =
                                            obj.counters.get(counter_type).copied().unwrap_or(0);
                                        obj.counters = obj
                                            .counters
                                            .update(counter_type.clone(), current + count);
                                    }
                                }
                            }
                        }
                    }
                }

                // CR 702.136a: Riot -- "You may have this permanent enter with an
                // additional +1/+1 counter on it. If you don't, it gains haste."
                // CR 702.136b: Multiple instances each work separately.
                // CR 614.1c: This is a replacement effect -- applied inline before
                // PermanentEnteredBattlefield is emitted, not a triggered ability.
                //
                // Implementation: For each instance of Riot on the permanent,
                // default to choosing +1/+1 counter (deterministic testing).
                // TODO: Add Command::ChooseRiot for player-interactive choice.
                //
                // OrdSet deduplicates KeywordAbility::Riot, so we count Riot
                // instances from the card definition, not from the keywords set
                // (same approach as Afterlife/Annihilator count parameters).
                {
                    let riot_count = card_id
                        .as_ref()
                        .and_then(|cid| registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .filter(|a| {
                                    matches!(
                                        a,
                                        crate::cards::card_definition::AbilityDefinition::Keyword(
                                            KeywordAbility::Riot
                                        )
                                    )
                                })
                                .count()
                        })
                        .unwrap_or(0);

                    for _ in 0..riot_count {
                        // Default choice: +1/+1 counter (CR 702.136a).
                        // Each Riot instance adds one +1/+1 counter.
                        if let Some(obj) = state.objects.get_mut(&new_id) {
                            let current = obj
                                .counters
                                .get(&CounterType::PlusOnePlusOne)
                                .copied()
                                .unwrap_or(0);
                            obj.counters = obj
                                .counters
                                .update(CounterType::PlusOnePlusOne, current + 1);
                        }
                        events.push(GameEvent::CounterAdded {
                            object_id: new_id,
                            counter: CounterType::PlusOnePlusOne,
                            count: 1,
                        });
                    }
                }

                // CR 702.43a: Modular N -- "This permanent enters with N +1/+1 counters
                // on it." (static ability / ETB replacement effect, CR 614.1c)
                // CR 702.43b: Multiple instances each work separately; their N values sum.
                // Count from the card definition (same approach as Riot / Afterlife) since
                // OrdSet deduplication would collapse Modular(1) + Modular(2) if they had
                // the same discriminant -- but they don't (different N), so they ARE distinct
                // variants and would NOT be deduplicated. We still count from the card def
                // for consistency and correctness.
                {
                    let modular_total: u32 = card_id
                        .as_ref()
                        .and_then(|cid| registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .filter_map(|a| match a {
                                    crate::cards::card_definition::AbilityDefinition::Keyword(
                                        KeywordAbility::Modular(n),
                                    ) => Some(*n),
                                    _ => None,
                                })
                                .sum()
                        })
                        .unwrap_or(0);

                    if modular_total > 0 {
                        if let Some(obj) = state.objects.get_mut(&new_id) {
                            let current = obj
                                .counters
                                .get(&CounterType::PlusOnePlusOne)
                                .copied()
                                .unwrap_or(0);
                            obj.counters = obj
                                .counters
                                .update(CounterType::PlusOnePlusOne, current + modular_total);
                        }
                        events.push(GameEvent::CounterAdded {
                            object_id: new_id,
                            counter: CounterType::PlusOnePlusOne,
                            count: modular_total,
                        });
                    }
                }

                // CR 702.58a: Graft N -- "This permanent enters with N +1/+1 counters on it."
                // CR 702.58b: Multiple instances each work separately; their N values sum.
                // Count from the card definition (same approach as Modular).
                {
                    let graft_total: u32 = card_id
                        .as_ref()
                        .and_then(|cid| registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .filter_map(|a| match a {
                                    crate::cards::card_definition::AbilityDefinition::Keyword(
                                        KeywordAbility::Graft(n),
                                    ) => Some(*n),
                                    _ => None,
                                })
                                .sum()
                        })
                        .unwrap_or(0);

                    if graft_total > 0 {
                        if let Some(obj) = state.objects.get_mut(&new_id) {
                            let current = obj
                                .counters
                                .get(&CounterType::PlusOnePlusOne)
                                .copied()
                                .unwrap_or(0);
                            obj.counters = obj
                                .counters
                                .update(CounterType::PlusOnePlusOne, current + graft_total);
                        }
                        events.push(GameEvent::CounterAdded {
                            object_id: new_id,
                            counter: CounterType::PlusOnePlusOne,
                            count: graft_total,
                        });
                    }
                }

                // CR 702.38a: Amplify N -- "As this object enters, reveal any number of
                // cards from your hand that share a creature type with it. This permanent
                // enters with N +1/+1 counters on it for each card revealed this way."
                // CR 702.38b: Multiple instances work separately; each is processed over
                // the same eligible hand cards (the CR does not restrict reveals per instance).
                // CR 614.1c: This is a static/replacement ability, not a triggered ability.
                //
                // Implementation: count Amplify instances from the card definition (same
                // approach as Modular/Graft). For each instance, auto-reveal all eligible
                // hand cards (deterministic bot play -- maximises counters).
                {
                    // Collect all Amplify(n) instances from the card definition.
                    let amplify_instances: Vec<u32> = card_id
                        .as_ref()
                        .and_then(|cid| registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .filter_map(|a| match a {
                                    crate::cards::card_definition::AbilityDefinition::Keyword(
                                        KeywordAbility::Amplify(n),
                                    ) => Some(*n),
                                    _ => None,
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    if !amplify_instances.is_empty() {
                        // Resolve the entering creature's subtypes via the layer system
                        // (respects Changeling / CDAs in all zones -- CR 604.3).
                        let entering_subtypes: im::OrdSet<SubType> =
                            crate::rules::layers::calculate_characteristics(state, new_id)
                                .map(|c| c.subtypes)
                                .unwrap_or_default();

                        // Collect hand object IDs for the controller (excluding the entering
                        // creature itself, which is now on the battlefield, not in hand).
                        let controller = {
                            state
                                .objects
                                .get(&new_id)
                                .map(|o| o.controller)
                                .unwrap_or(stack_obj.controller)
                        };
                        let hand_obj_ids: Vec<ObjectId> = state
                            .objects
                            .iter()
                            .filter(|(_, obj)| {
                                obj.zone == ZoneId::Hand(controller) && obj.is_phased_in()
                            })
                            .map(|(id, _)| *id)
                            .collect();

                        // Count hand cards that share at least one creature subtype with the
                        // entering creature. Calculate characteristics for each hand card to
                        // honour CDAs like Changeling (CR 604.3).
                        let eligible_count = hand_obj_ids
                            .iter()
                            .filter(|&&hand_id| {
                                let hand_subtypes =
                                    crate::rules::layers::calculate_characteristics(state, hand_id)
                                        .map(|c| c.subtypes)
                                        .unwrap_or_default();
                                // Cards with no creature subtypes cannot share a type.
                                !entering_subtypes.is_empty()
                                    && !entering_subtypes
                                        .clone()
                                        .intersection(hand_subtypes)
                                        .is_empty()
                            })
                            .count() as u32;

                        // Apply each Amplify instance: N * eligible_count counters.
                        let mut total_amplify_counters: u32 = 0;
                        for n in &amplify_instances {
                            total_amplify_counters += n * eligible_count;
                        }

                        if total_amplify_counters > 0 {
                            if let Some(obj) = state.objects.get_mut(&new_id) {
                                let current = obj
                                    .counters
                                    .get(&CounterType::PlusOnePlusOne)
                                    .copied()
                                    .unwrap_or(0);
                                obj.counters = obj.counters.update(
                                    CounterType::PlusOnePlusOne,
                                    current + total_amplify_counters,
                                );
                            }
                            events.push(GameEvent::CounterAdded {
                                object_id: new_id,
                                counter: CounterType::PlusOnePlusOne,
                                count: total_amplify_counters,
                            });
                        }
                    }
                }

                // CR 702.54a: Bloodthirst N -- "If an opponent was dealt damage this turn,
                // this permanent enters with N +1/+1 counters on it."
                // CR 702.54c: Multiple instances work separately; each N is added independently.
                // CR 800.4a: Eliminated/conceded players are not opponents.
                {
                    let bloodthirst_instances: Vec<u32> = card_id
                        .as_ref()
                        .and_then(|cid| registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .filter_map(|a| match a {
                                    crate::cards::card_definition::AbilityDefinition::Keyword(
                                        KeywordAbility::Bloodthirst(n),
                                    ) => Some(*n),
                                    _ => None,
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    if !bloodthirst_instances.is_empty() {
                        let controller = {
                            state
                                .objects
                                .get(&new_id)
                                .map(|o| o.controller)
                                .unwrap_or(stack_obj.controller)
                        };
                        // Check if any opponent was dealt damage this turn.
                        // CR 800.4a: eliminated/conceded players are not opponents.
                        let any_opponent_damaged = state.players.iter().any(|(pid, ps)| {
                            *pid != controller
                                && !ps.has_lost
                                && !ps.has_conceded
                                && ps.damage_received_this_turn > 0
                        });

                        if any_opponent_damaged {
                            let total_counters: u32 = bloodthirst_instances.iter().sum();
                            if total_counters > 0 {
                                if let Some(obj) = state.objects.get_mut(&new_id) {
                                    let current = obj
                                        .counters
                                        .get(&CounterType::PlusOnePlusOne)
                                        .copied()
                                        .unwrap_or(0);
                                    obj.counters = obj.counters.update(
                                        CounterType::PlusOnePlusOne,
                                        current + total_counters,
                                    );
                                }
                                events.push(GameEvent::CounterAdded {
                                    object_id: new_id,
                                    counter: CounterType::PlusOnePlusOne,
                                    count: total_counters,
                                });
                            }
                        }
                    }
                }

                // CR 702.82a: Devour N -- "As this object enters, you may sacrifice any number
                // of creatures. This permanent enters with N +1/+1 counters on it for each
                // creature sacrificed this way."
                // CR 702.82c: Multiple instances work separately; each N is processed over the
                // same devour_sacrifices list.
                // CR 614.1c: This is a static/replacement ability, not a triggered ability.
                //
                // Implementation: consume devour_sacrifices from the StackObject. For each
                // Devour(N) instance, multiply N by the number of sacrificed creatures.
                // The sacrifice happens HERE (during ETB), not at cast time.
                {
                    // Collect all Devour(n) instances from the card definition.
                    let devour_instances: Vec<u32> = card_id
                        .as_ref()
                        .and_then(|cid| registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .filter_map(|a| match a {
                                    crate::cards::card_definition::AbilityDefinition::Keyword(
                                        KeywordAbility::Devour(n),
                                    ) => Some(*n),
                                    _ => None,
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    // RC-1: Extract devour sacrifices from additional_costs.
                    let devour_sacrifice_ids: Vec<ObjectId> = stack_obj
                        .additional_costs
                        .iter()
                        .find_map(|c| {
                            if let crate::state::types::AdditionalCost::Sacrifice(ids) = c {
                                Some(ids.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    if !devour_instances.is_empty() && !devour_sacrifice_ids.is_empty() {
                        let entering_controller = state
                            .objects
                            .get(&new_id)
                            .map(|o| o.controller)
                            .unwrap_or(stack_obj.controller);

                        let mut sacrifice_count: u32 = 0;

                        // Sacrifice each creature. Re-validate at resolution time since
                        // state may have changed since cast time (CR 608.3b).
                        for sac_id in &devour_sacrifice_ids {
                            let sac_id = *sac_id;
                            // Validate: still on battlefield, still controlled by caster, still creature.
                            let still_valid = state
                                .objects
                                .get(&sac_id)
                                .map(|obj| {
                                    obj.zone == ZoneId::Battlefield
                                        && obj.controller == entering_controller
                                        && obj
                                            .characteristics
                                            .card_types
                                            .contains(&CardType::Creature)
                                })
                                .unwrap_or(false);

                            if !still_valid {
                                continue;
                            }

                            // Capture last-known information before zone move (CR 400.7).
                            let (sac_owner, pre_death_controller, pre_death_counters) = {
                                let obj = match state.objects.get(&sac_id) {
                                    Some(o) => o,
                                    None => continue,
                                };
                                (obj.owner, obj.controller, obj.counters.clone())
                            };

                            // CR 614: Check replacement effects (e.g., Rest in Peace).
                            let action = crate::rules::replacement::check_zone_change_replacement(
                                state,
                                sac_id,
                                crate::state::zone::ZoneType::Battlefield,
                                crate::state::zone::ZoneType::Graveyard,
                                sac_owner,
                                &std::collections::HashSet::new(),
                            );

                            match action {
                                crate::rules::replacement::ZoneChangeAction::Redirect {
                                    to: dest,
                                    events: repl_events,
                                    ..
                                } => {
                                    events.extend(repl_events);
                                    if let Ok((new_grave_id, _old)) =
                                        state.move_object_to_zone(sac_id, dest)
                                    {
                                        match dest {
                                            ZoneId::Exile => {
                                                // Replacement (e.g., Rest in Peace) exiled instead of graveyard.
                                                events.push(GameEvent::ObjectExiled {
                                                    player: sac_owner,
                                                    object_id: sac_id,
                                                    new_exile_id: new_grave_id,
                                                });
                                            }
                                            ZoneId::Command(_) => {
                                                // Commander redirected to command zone -- no CreatureDied.
                                            }
                                            _ => {
                                                events.push(GameEvent::CreatureDied {
                                                    object_id: sac_id,
                                                    new_grave_id,
                                                    controller: pre_death_controller,
                                                    pre_death_counters,
                                                });
                                            }
                                        }
                                        sacrifice_count += 1;
                                    }
                                }
                                crate::rules::replacement::ZoneChangeAction::Proceed => {
                                    if let Ok((new_grave_id, _old)) = state
                                        .move_object_to_zone(sac_id, ZoneId::Graveyard(sac_owner))
                                    {
                                        events.push(GameEvent::CreatureDied {
                                            object_id: sac_id,
                                            new_grave_id,
                                            controller: pre_death_controller,
                                            pre_death_counters,
                                        });
                                        sacrifice_count += 1;
                                    }
                                }
                                crate::rules::replacement::ZoneChangeAction::ChoiceRequired {
                                    ..
                                } => {
                                    // For simplicity, treat ChoiceRequired as Proceed (go to graveyard).
                                    // Full interactive choice support is deferred.
                                    if let Ok((new_grave_id, _old)) = state
                                        .move_object_to_zone(sac_id, ZoneId::Graveyard(sac_owner))
                                    {
                                        events.push(GameEvent::CreatureDied {
                                            object_id: sac_id,
                                            new_grave_id,
                                            controller: pre_death_controller,
                                            pre_death_counters,
                                        });
                                        sacrifice_count += 1;
                                    }
                                }
                            }
                        }

                        if sacrifice_count > 0 {
                            // CR 702.82b: Record the number of creatures devoured for
                            // abilities that reference "it devoured".
                            if let Some(obj) = state.objects.get_mut(&new_id) {
                                obj.creatures_devoured = sacrifice_count;
                            }

                            // Add +1/+1 counters: for each Devour(N) instance, add N * sacrifice_count.
                            let mut total_devour_counters: u32 = 0;
                            for n in &devour_instances {
                                total_devour_counters += n * sacrifice_count;
                            }

                            if total_devour_counters > 0 {
                                if let Some(obj) = state.objects.get_mut(&new_id) {
                                    let current = obj
                                        .counters
                                        .get(&CounterType::PlusOnePlusOne)
                                        .copied()
                                        .unwrap_or(0);
                                    obj.counters = obj.counters.update(
                                        CounterType::PlusOnePlusOne,
                                        current + total_devour_counters,
                                    );
                                }
                                events.push(GameEvent::CounterAdded {
                                    object_id: new_id,
                                    counter: CounterType::PlusOnePlusOne,
                                    count: total_devour_counters,
                                });
                            }
                        }
                    }
                }

                // CR 702.104a: Tribute N -- "As this creature enters, choose an opponent.
                // That player may put an additional N +1/+1 counters on it as it enters."
                // CR 702.104b: Objects with tribute have triggered abilities that check
                // "if tribute wasn't paid."
                //
                // Implementation: Deterministic bot play -- opponent always declines tribute.
                // The creature enters without extra counters, and the "tribute wasn't paid"
                // triggered ability fires. `tribute_was_paid` stays false (default).
                //
                // Future: when interactive opponent choices are implemented, this block
                // should prompt the chosen opponent and conditionally add counters +
                // set tribute_was_paid = true.
                {
                    let tribute_instances: Vec<u32> = card_id
                        .as_ref()
                        .and_then(|cid| registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .filter_map(|a| match a {
                                    crate::cards::card_definition::AbilityDefinition::Keyword(
                                        KeywordAbility::Tribute(n),
                                    ) => Some(*n),
                                    _ => None,
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    if !tribute_instances.is_empty() {
                        // Bot play: opponent does not pay tribute.
                        // tribute_was_paid remains false (default).
                        // The "if tribute wasn't paid" triggered ability will fire
                        // via queue_carddef_etb_triggers with TributeNotPaid condition.
                        //
                        // (No counters are placed; no state mutation needed here.)
                        let _ = tribute_instances; // explicitly consumed; no-op in bot play
                    }
                }

                // CR 702.156a: Ravenous -- "This permanent enters with X +1/+1 counters on it."
                // CR 107.3m: X is the value chosen at cast time (stack_obj.x_value), NOT the
                // permanent's X (which is 0 per CR 107.3i). The counter placement is a
                // replacement effect that modifies how the permanent enters the battlefield.
                {
                    let has_ravenous = card_id
                        .as_ref()
                        .and_then(|cid| registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities.iter().any(|a| {
                                matches!(
                                    a,
                                    crate::cards::card_definition::AbilityDefinition::Keyword(
                                        KeywordAbility::Ravenous
                                    )
                                )
                            })
                        })
                        .unwrap_or(false);

                    if has_ravenous && stack_obj.x_value > 0 {
                        if let Some(obj) = state.objects.get_mut(&new_id) {
                            let current = obj
                                .counters
                                .get(&CounterType::PlusOnePlusOne)
                                .copied()
                                .unwrap_or(0);
                            obj.counters = obj
                                .counters
                                .update(CounterType::PlusOnePlusOne, current + stack_obj.x_value);
                        }
                        events.push(GameEvent::CounterAdded {
                            object_id: new_id,
                            counter: CounterType::PlusOnePlusOne,
                            count: stack_obj.x_value,
                        });
                    }

                    // CR 702.156a: "When this permanent enters, if X is 5 or more, draw a card."
                    // CR 603.4: Intervening-if -- checked at trigger time AND resolution.
                    // X is the cast-time value (stack_obj.x_value), regardless of counters placed.
                    if has_ravenous && stack_obj.x_value >= 5 {
                        state
                            .pending_triggers
                            .push_back(crate::state::stubs::PendingTrigger {
                                source: new_id,
                                ability_index: 0,
                                controller: stack_obj.controller,
                                kind: crate::state::stubs::PendingTriggerKind::RavenousDraw,
                                triggering_event: None,
                                entering_object_id: None,
                                targeting_stack_id: None,
                                triggering_player: None,
                                exalted_attacker_id: None,
                                defending_player_id: None,
                                ingest_target_player: None,
                                flanking_blocker_id: None,
                                rampage_n: None,
                                renown_n: None,
                                poisonous_n: None,
                                poisonous_target_player: None,
                                enlist_enlisted_creature: None,
                                recover_cost: None,
                                recover_card: None,
                                cipher_encoded_card_id: None,
                                cipher_encoded_object_id: None,
                                haunt_source_object_id: None,
                                haunt_source_card_id: None,
                                data: None,                            });
                    }

                    // CR 702.157a: Squad ETB trigger -- "When this creature enters, if its
                    // squad cost was paid, create a token that's a copy of it for each time
                    // its squad cost was paid."
                    //
                    // CR 603.4: Intervening-if -- only queue if squad_count > 0.
                    // Ruling 2022-10-07: also require the permanent has Squad in layer-resolved
                    // characteristics at trigger time (not just at cast time).
                    let has_squad = {
                        let chars = crate::rules::layers::calculate_characteristics(state, new_id);
                        chars
                            .map(|c| c.keywords.contains(&KeywordAbility::Squad))
                            .unwrap_or(false)
                    };
                    let permanent_squad_count = state
                        .objects
                        .get(&new_id)
                        .map(|o| o.squad_count)
                        .unwrap_or(0);
                    if has_squad && permanent_squad_count > 0 {
                        state.pending_triggers.push_back(crate::state::stubs::PendingTrigger {
                            data: Some(crate::state::stack::TriggerData::ETBSquad {
                                count: permanent_squad_count,
                            }),
                            ..crate::state::stubs::PendingTrigger::blank(
                                new_id,
                                stack_obj.controller,
                                crate::state::stubs::PendingTriggerKind::SquadETB,
                            )
                        });
                    }

                    // CR 702.175a: Offspring ETB trigger -- "When this permanent enters, if its
                    // offspring cost was paid, create a token that's a copy of it, except it's 1/1."
                    //
                    // CR 603.4: Intervening-if -- only queue if offspring_paid == true AND
                    // the permanent has KeywordAbility::Offspring in layer-resolved characteristics.
                    // Ruling 2024-07-26: if offspring is lost before the trigger fires, no token.
                    let has_offspring = {
                        let chars = crate::rules::layers::calculate_characteristics(state, new_id);
                        chars
                            .map(|c| c.keywords.contains(&KeywordAbility::Offspring))
                            .unwrap_or(false)
                    };
                    let permanent_offspring_paid = state
                        .objects
                        .get(&new_id)
                        .map(|o| o.offspring_paid)
                        .unwrap_or(false);
                    if has_offspring && permanent_offspring_paid {
                        state
                            .pending_triggers
                            .push_back(crate::state::stubs::PendingTrigger {
                                source: new_id,
                                ability_index: 0,
                                controller: stack_obj.controller,
                                kind: crate::state::stubs::PendingTriggerKind::OffspringETB,
                                triggering_event: None,
                                entering_object_id: None,
                                targeting_stack_id: None,
                                triggering_player: None,
                                exalted_attacker_id: None,
                                defending_player_id: None,
                                ingest_target_player: None,
                                flanking_blocker_id: None,
                                rampage_n: None,
                                renown_n: None,
                                poisonous_n: None,
                                poisonous_target_player: None,
                                enlist_enlisted_creature: None,
                                recover_cost: None,
                                recover_card: None,
                                cipher_encoded_card_id: None,
                                cipher_encoded_object_id: None,
                                haunt_source_object_id: None,
                                haunt_source_card_id: None,
                                data: None,                            });
                    }

                    // CR 702.174b: Gift ETB trigger -- "When this permanent enters, if its
                    // gift cost was paid, [give the gift to the chosen opponent]."
                    //
                    // CR 603.4: Intervening-if -- only queue if gift_was_given == true AND the
                    // permanent has KeywordAbility::Gift in layer-resolved characteristics.
                    let has_gift = {
                        let chars = crate::rules::layers::calculate_characteristics(state, new_id);
                        chars
                            .map(|c| c.keywords.contains(&KeywordAbility::Gift))
                            .unwrap_or(false)
                    };
                    let permanent_gift_was_given = state
                        .objects
                        .get(&new_id)
                        .map(|o| o.gift_was_given)
                        .unwrap_or(false);
                    let permanent_gift_opponent =
                        state.objects.get(&new_id).and_then(|o| o.gift_opponent);
                    if has_gift && permanent_gift_was_given {
                        if let Some(gift_opp) = permanent_gift_opponent {
                            state.pending_triggers.push_back(crate::state::stubs::PendingTrigger {
                                data: Some(crate::state::stack::TriggerData::ETBGift {
                                    source_card_id: state.objects.get(&new_id).and_then(|o| o.card_id.clone()),
                                    gift_opponent: gift_opp,
                                }),
                                ..crate::state::stubs::PendingTrigger::blank(
                                    new_id,
                                    stack_obj.controller,
                                    crate::state::stubs::PendingTriggerKind::GiftETB,
                                )
                            });
                        }
                    }
                }

                // CR 702.103b: If the permanent is bestowed, re-apply the type
                // transformation after move_object_to_zone (which resets to printed types).
                // The permanent enters as an Aura enchantment with enchant creature,
                // not as a creature.
                if stack_obj.was_bestowed && !bestow_fallback {
                    if let Some(obj) = state.objects.get_mut(&new_id) {
                        obj.characteristics.card_types.remove(&CardType::Creature);
                        obj.characteristics.card_types.insert(CardType::Enchantment);
                        obj.characteristics
                            .subtypes
                            .insert(SubType("Aura".to_string()));
                        obj.characteristics
                            .keywords
                            .insert(KeywordAbility::Enchant(EnchantTarget::Creature));
                    }
                }

                // CR 303.4a / 303.4b: If the resolved permanent is an Aura, attach it
                // to its target BEFORE registering static continuous effects. The
                // EffectFilter::AttachedCreature filter reads `attached_to`, so the
                // attachment must be set before register_static_continuous_effects runs.
                {
                    let is_aura = {
                        let obj = state.objects.get(&new_id);
                        obj.map(|o| {
                            o.characteristics
                                .card_types
                                .contains(&CardType::Enchantment)
                                && o.characteristics
                                    .subtypes
                                    .contains(&SubType("Aura".to_string()))
                        })
                        .unwrap_or(false)
                    };
                    if is_aura {
                        // Find the first legal Object target from the original stack entry.
                        let aura_target = stack_obj
                            .targets
                            .iter()
                            .filter(|t| is_target_legal(state, t))
                            .find_map(|t| {
                                if let Target::Object(target_id) = t.target {
                                    Some(target_id)
                                } else {
                                    None
                                }
                            });
                        if let Some(target_id) = aura_target {
                            // Set attached_to on the Aura.
                            if let Some(aura_obj) = state.objects.get_mut(&new_id) {
                                aura_obj.attached_to = Some(target_id);
                            }
                            // Add to target's attachments list.
                            if let Some(target_obj) = state.objects.get_mut(&target_id) {
                                if !target_obj.attachments.contains(&new_id) {
                                    target_obj.attachments.push_back(new_id);
                                }
                            }
                            events.push(GameEvent::AuraAttached {
                                aura_id: new_id,
                                target_id,
                                controller,
                            });
                        }
                        // If no legal target exists, the Aura is left unattached.
                        // SBA 704.5m will move it to the graveyard on the next SBA check.
                    }
                }

                // CR 614.12 / 614.15: Apply ETB replacement effects before emitting
                // PermanentEnteredBattlefield. Self-ETB replacements from the card's
                // own definition apply first (CR 614.15: self-replacement first), then
                // global replacement effects from state.replacement_effects.
                let self_evts = super::replacement::apply_self_etb_from_definition(
                    state,
                    new_id,
                    controller,
                    card_id.as_ref(),
                    &registry,
                );
                events.extend(self_evts);
                let etb_evts =
                    super::replacement::apply_etb_replacements(state, new_id, controller);
                events.extend(etb_evts);

                // CR 614: Register global replacement abilities from this permanent's
                // card definition. Must happen after ETB replacements are applied so
                // the permanent is fully settled. The new effects activate immediately
                // (in time to intercept events from the same resolution batch if any).
                super::replacement::register_permanent_replacement_abilities(
                    state,
                    new_id,
                    controller,
                    card_id.as_ref(),
                    &registry,
                );

                // CR 708.3: A face-down permanent has no abilities while it's face-down.
                // Its ETB abilities do NOT fire when it enters the battlefield face-down
                // (cast with morph/megamorph/disguise). Only global triggers on OTHER
                // permanents that watch for any creature entering CAN still trigger
                // (they see a 2/2 creature, but not the card's name/abilities).
                let is_face_down_entering = morph_face_down_kind.is_some();

                // CR 604 / CR 613: Register static continuous effects from this
                // permanent's card definition (Equipment, Aura, global ability grants).
                // CR 708.3: Face-down permanents have no abilities, so their static
                // continuous effects do not activate while they are face-down.
                // Effects will be registered when the permanent is turned face-up.
                if !is_face_down_entering {
                    super::replacement::register_static_continuous_effects(
                        state,
                        new_id,
                        card_id.as_ref(),
                        &registry,
                    );
                }

                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: controller,
                    object_id: new_id,
                });

                // CR 702.145c / CR 702.145f: When a Daybound or Nightbound permanent enters
                // the battlefield, enforce the current day/night state (or establish it).
                // "This ability triggers immediately" (CR 702.145d/g rulings).
                let db_evts = crate::rules::turn_actions::enforce_daybound_nightbound(state);
                events.extend(db_evts);

                // CR 603.3, 603.6a: Queue WhenEntersBattlefield triggered abilities from
                // card definition as PendingTrigger (goes on stack at next priority window).
                // CR 708.3: face-down guard is handled inside queue_carddef_etb_triggers.
                // Returns events only for Fabricate inline bot-play approximation.
                let etb_trigger_evts = super::replacement::queue_carddef_etb_triggers(
                    state,
                    new_id,
                    controller,
                    card_id.as_ref(),
                    &registry,
                );
                events.extend(etb_trigger_evts);

                events.push(GameEvent::SpellResolved {
                    player: controller,
                    stack_object_id: stack_obj.id,
                    source_object_id: new_id,
                });
            } else {
                // CR 608.2n: Instant/sorcery — card moves to owner's graveyard.
                // CR 702.34a: If cast with flashback, exile instead of graveyard.
                // Flashback overrides buyback: "exile instead of putting it anywhere
                // else any time it would leave the stack" (CR 702.34a).
                // CR 702.133a: Jump-start also exiles instead of graveyard on resolution.
                // Jump-start overrides buyback: "exile this card instead of putting it
                // anywhere else any time it would leave the stack" (CR 702.133a).
                // CR 702.27a: If buyback was paid (and not flashbacked or jump-started), return to hand.
                //
                // CR 702.99a: Cipher -- "If this spell is represented by a card, you may
                // exile this card encoded on a creature you control."
                // Cipher is checked BEFORE flashback/jump-start (those override cipher):
                // per CR 702.34a / 702.133a "exile instead of putting anywhere else".
                // The controller may choose to encode only if:
                //   1. The spell is NOT a copy (cipher: "represented by a card")
                //   2. The spell was NOT cast with flashback or jump-start (those override)
                //   3. The controller has at least one creature on the battlefield
                // MVP: auto-encode on the first available creature (deterministic).
                let has_cipher = !stack_obj.is_copy
                    && !stack_obj.cast_with_flashback
                    && !stack_obj.cast_with_jump_start
                    && !stack_obj.cast_with_aftermath
                    && {
                        // Check cipher in the card's characteristics or registry.
                        // Use raw characteristics (cipher is a printed keyword).
                        state
                            .objects
                            .get(&source_object)
                            .map(|obj| {
                                obj.characteristics
                                    .keywords
                                    .contains(&KeywordAbility::Cipher)
                            })
                            .unwrap_or(false)
                    };

                // Find the first creature controlled by this player (for MVP auto-encode).
                let cipher_creature = if has_cipher {
                    state
                        .objects
                        .values()
                        .find(|obj| {
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller == controller
                                // CR 613.1d: Use layer-resolved types (animated permanents).
                                && crate::rules::layers::calculate_characteristics(state, obj.id)
                                    .unwrap_or_else(|| obj.characteristics.clone())
                                    .card_types
                                    .contains(&CardType::Creature)
                        })
                        .map(|obj| obj.id)
                } else {
                    None
                };

                let destination = if stack_obj.cast_with_flashback
                    || stack_obj.cast_with_jump_start
                    || (has_cipher && cipher_creature.is_some())
                {
                    // CR 702.34a / CR 702.133a: flashback/jump-start exile on resolution.
                    // CR 702.99a: cipher exile on resolution (card encoded on a creature).
                    ZoneId::Exile
                } else if stack_obj.was_buyback_paid {
                    ZoneId::Hand(owner) // CR 702.27a
                } else {
                    ZoneId::Graveyard(owner)
                };
                let (new_id, _old) = state.move_object_to_zone(source_object, destination)?;

                // CR 702.99a: If cipher resolved and we have a target creature, encode.
                // The card is now in exile (new_id). Set encoded_cards on the creature.
                // Ruling 2013-04-15: encoding happens after the spell's effects resolve.
                if let Some(creature_id) = cipher_creature {
                    if let Some(card_id_val) = &card_id {
                        // Add the encoded card to the creature's encoded_cards list.
                        if let Some(creature_obj) = state.objects.get_mut(&creature_id) {
                            creature_obj
                                .encoded_cards
                                .push_back((new_id, card_id_val.clone()));
                        }
                        events.push(GameEvent::CipherEncoded {
                            player: controller,
                            exiled_card: new_id,
                            creature: creature_id,
                        });
                    }
                }

                events.push(GameEvent::SpellResolved {
                    player: controller,
                    stack_object_id: stack_obj.id,
                    source_object_id: new_id,
                });
            }
        }
        StackObjectKind::ActivatedAbility {
            source_object,
            ability_index,
            embedded_effect,
        } => {
            // CR 608.3b: Activated ability resolves — execute its effect.
            // Use the embedded_effect captured at activation time (required when the source
            // was sacrificed as a cost and is no longer in the objects map).
            // Fall back to live object lookup for non-sacrificed sources.
            let ability_effect = embedded_effect.as_deref().cloned().or_else(|| {
                state
                    .objects
                    .get(&source_object)
                    .and_then(|obj| obj.characteristics.activated_abilities.get(ability_index))
                    .and_then(|ab| ab.effect.clone())
            });

            if let Some(effect) = ability_effect {
                let mut ctx = EffectContext::new(
                    stack_obj.controller,
                    source_object,
                    stack_obj.targets.clone(),
                );
                let effect_events = execute_effect(state, &effect, &mut ctx);
                events.extend(effect_events);
            }

            events.push(GameEvent::AbilityResolved {
                controller: stack_obj.controller,
                stack_object_id: stack_obj.id,
            });
        }

        StackObjectKind::LoyaltyAbility {
            source_object,
            ability_index: _,
            effect,
        } => {
            // CR 606: Loyalty ability resolves — execute the captured effect.
            // The loyalty cost was already paid at activation time.
            let mut ctx = EffectContext::new(
                stack_obj.controller,
                source_object,
                stack_obj.targets.clone(),
            );
            let effect_events = execute_effect(state, &effect, &mut ctx);
            events.extend(effect_events);

            events.push(GameEvent::AbilityResolved {
                controller: stack_obj.controller,
                stack_object_id: stack_obj.id,
            });
        }

        StackObjectKind::ForecastAbility {
            source_object,
            embedded_effect,
        } => {
            // CR 702.57a: Forecast ability resolves — execute the embedded effect.
            // The source card remains in the player's hand (not moved by resolution).
            // Targets were recorded at activation time; validate at resolution (CR 608.2b).
            let mut ctx = EffectContext::new(
                stack_obj.controller,
                source_object,
                stack_obj.targets.clone(),
            );
            let effect_events = execute_effect(state, &embedded_effect, &mut ctx);
            events.extend(effect_events);

            events.push(GameEvent::AbilityResolved {
                controller: stack_obj.controller,
                stack_object_id: stack_obj.id,
            });
        }

        StackObjectKind::TriggeredAbility {
            source_object,
            ability_index,
            is_carddef_etb,
        } => {
            // CR 603.4: Check intervening-if condition at resolution time.
            // If the condition is false, the ability has no effect (but still resolves).
            // Look up triggered ability effect and intervening-if condition.
            // characteristics.triggered_abilities is populated for keyword-derived triggers.
            // Plain AbilityDefinition::Triggered entries (e.g. upkeep/end-step CardDef triggers
            // pushed via PendingTriggerKind::Normal) are looked up from the card registry
            // using ability_index into the CardDef's abilities Vec.
            //
            // When is_carddef_etb is true, ability_index is into CardDef::abilities (not
            // runtime triggered_abilities). Always use the card registry path to avoid
            // index-namespace collisions when the card also has runtime triggers at the
            // same index position (e.g. Acererak: ETB at def[0], WhenAttacks at runtime[0]).
            let (triggered_effect_opt, triggered_carddef_iif) = {
                let obj = state.objects.get(&source_object);
                if let Some(obj) = obj {
                    if !is_carddef_etb {
                        // CR 613.1f: Use layer-resolved triggered_abilities to match
                        // the namespace used by collect_triggers_for_event (S2 fix).
                        let resolved =
                            crate::rules::layers::calculate_characteristics(state, source_object)
                                .unwrap_or_else(|| obj.characteristics.clone());
                        if let Some(_ab) = resolved.triggered_abilities.get(ability_index) {
                            // Characteristics path — intervening_if handled below via original code.
                            (
                                None::<crate::cards::card_definition::Effect>,
                                None::<crate::cards::card_definition::Condition>,
                            )
                        } else {
                            // Card registry fallback for plain AbilityDefinition::Triggered
                            // and AbilityDefinition::SagaChapter (CR 714.2b).
                            let result = obj
                                .card_id
                                .as_ref()
                                .and_then(|cid| state.card_registry.get(cid.clone()))
                                .and_then(|def| def.abilities.get(ability_index))
                                .and_then(|abil| {
                                    match abil {
                                        crate::cards::card_definition::AbilityDefinition::Triggered {
                                            effect,
                                            intervening_if,
                                            ..
                                        } => Some((effect.clone(), intervening_if.clone())),
                                        crate::cards::card_definition::AbilityDefinition::SagaChapter {
                                            effect, ..
                                        } => Some((effect.clone(), None)),
                                        _ => None,
                                    }
                                });
                            if let Some((eff, iif)) = result {
                                (Some(eff), iif)
                            } else {
                                (None, None)
                            }
                        }
                    } else {
                        // CardDefETB path: ability_index is into CardDef::abilities.
                        // Always use the card registry — never runtime triggered_abilities.
                        let result = obj
                            .card_id
                            .as_ref()
                            .and_then(|cid| state.card_registry.get(cid.clone()))
                            .and_then(|def| def.abilities.get(ability_index))
                            .and_then(|abil| match abil {
                                crate::cards::card_definition::AbilityDefinition::Triggered {
                                    effect,
                                    intervening_if,
                                    ..
                                } => Some((effect.clone(), intervening_if.clone())),
                                crate::cards::card_definition::AbilityDefinition::SagaChapter {
                                    effect,
                                    ..
                                } => Some((effect.clone(), None)),
                                _ => None,
                            });
                        if let Some((eff, iif)) = result {
                            (Some(eff), iif)
                        } else {
                            (None, None)
                        }
                    }
                } else {
                    (None, None)
                }
            };

            // If we got a CardDef-registry effect, execute it directly.
            // CR 603.4: intervening-if conditions must be checked both when the trigger fires
            // AND when it resolves. If the condition no longer holds at resolution, the
            // triggered ability is removed from the stack without effect.
            if triggered_effect_opt.is_some() {
                // CR 608.2b: If the triggered ability has declared targets and all of them
                // are now illegal, the ability fizzles — it is removed from the stack without
                // effect. This is the "all targets illegal" fizzle rule for triggered abilities.
                let fizzled = !stack_obj.targets.is_empty()
                    && stack_obj.targets.iter().all(|t| !is_target_legal(state, t));
                if fizzled {
                    events.push(GameEvent::AbilityResolved {
                        controller: stack_obj.controller,
                        stack_object_id: stack_obj.id,
                    });
                    // Fizzled — skip effect execution entirely.
                } else {
                    // CR 603.4: Re-evaluate the intervening-if condition at resolution.
                    // If the condition no longer holds, the triggered ability is removed
                    // from the stack without effect.
                    // Use check_condition() from effects/mod.rs which correctly handles
                    // all Condition variants including Condition::Not (e.g. Acererak's
                    // "if you haven't completed Tomb of Annihilation").
                    let condition_holds = triggered_carddef_iif
                        .as_ref()
                        .map(|cond| {
                            let ctx = EffectContext::new(
                                stack_obj.controller,
                                source_object,
                                stack_obj.targets.clone(),
                            );
                            check_condition(state, cond, &ctx)
                        })
                        .unwrap_or(true);
                    if condition_holds {
                        if let Some(effect) = triggered_effect_opt {
                            // CR 702.33d: Propagate kicker_times_paid from the source permanent so
                            // kicker-conditional ETB effects (e.g., Torch Slinger) work correctly.
                            let kicker_times_paid = state
                                .objects
                                .get(&source_object)
                                .map(|o| o.kicker_times_paid)
                                .unwrap_or(0);
                            let mut ctx = EffectContext::new_with_kicker(
                                stack_obj.controller,
                                source_object,
                                stack_obj.targets.clone(),
                                kicker_times_paid,
                            );
                            let effect_events = execute_effect(state, &effect, &mut ctx);
                            events.extend(effect_events);
                        }
                    }
                    events.push(GameEvent::AbilityResolved {
                        controller: stack_obj.controller,
                        stack_object_id: stack_obj.id,
                    });
                } // end else (non-fizzled path)
            } else {
                let condition_holds = {
                    let source_obj = state.objects.get(&source_object);
                    match source_obj {
                        Some(obj) => {
                            // CR 613.1f: Use layer-resolved triggered_abilities
                            // to match collect_triggers_for_event namespace.
                            let resolved = crate::rules::layers::calculate_characteristics(
                                state,
                                source_object,
                            )
                            .unwrap_or_else(|| obj.characteristics.clone());
                            let ability_def = resolved.triggered_abilities.get(ability_index);
                            match ability_def {
                                Some(def) => def
                                    .intervening_if
                                    .as_ref()
                                    .map(|cond| {
                                        // CR 603.4: At resolution, pass None for pre_death_counters.
                                        // For persist/undying, the source is now in the graveyard
                                        // with no counters; the MoveZone effect will no-op if the
                                        // source has since left the graveyard.
                                        abilities::check_intervening_if(
                                            state,
                                            cond,
                                            stack_obj.controller,
                                            None,
                                        )
                                    })
                                    .unwrap_or(true),
                                None => true, // No definition found — resolve without effect
                            }
                        }
                        None => true, // Source gone — ability still resolves (no effect)
                    }
                };

                // CR 608.3b: Execute effect if condition holds.
                if condition_holds {
                    // CR 613.1f: Use layer-resolved triggered_abilities to match
                    // collect_triggers_for_event namespace.
                    let triggered_effect = state.objects.get(&source_object).and_then(|obj| {
                        let resolved =
                            crate::rules::layers::calculate_characteristics(state, source_object)
                                .unwrap_or_else(|| obj.characteristics.clone());
                        resolved
                            .triggered_abilities
                            .get(ability_index)
                            .and_then(|ab| ab.effect.clone())
                    });

                    if let Some(effect) = triggered_effect {
                        let mut ctx = EffectContext::new(
                            stack_obj.controller,
                            source_object,
                            stack_obj.targets.clone(),
                        );
                        let effect_events = execute_effect(state, &effect, &mut ctx);
                        events.extend(effect_events);
                    }
                }

                events.push(GameEvent::AbilityResolved {
                    controller: stack_obj.controller,
                    stack_object_id: stack_obj.id,
                });
            } // end else (characteristics-based path)
        }

        // CR 702.85a: Cascade trigger resolves — run the cascade procedure.
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Cascade,
            data: crate::state::stack::TriggerData::CascadeExile { spell_mana_value },
        } => {
            let controller = stack_obj.controller;
            let (cascade_events, _cast_id) =
                crate::rules::copy::resolve_cascade(state, controller, spell_mana_value);
            events.extend(cascade_events);

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.40a: Storm trigger resolves — create copies of the original spell.
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Storm,
            data:
                crate::state::stack::TriggerData::SpellCopy {
                    original_stack_id,
                    copy_count: storm_count,
                },
        } => {
            let controller = stack_obj.controller;
            let copy_events = crate::rules::copy::create_storm_copies(
                state,
                original_stack_id,
                controller,
                storm_count,
            );
            events.extend(copy_events);

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.153a: Casualty trigger resolves — create one copy of the original spell.
        //
        // "When you cast this spell, if a casualty cost was paid for it, copy it."
        // The copy is NOT cast (ruling 2022-04-29 / CR 707.10) — it does not trigger
        // "whenever you cast a spell" abilities.
        //
        // The copy is pushed onto the stack above the original and resolves first (LIFO).
        // If the original spell is no longer on the stack at resolution time (was countered,
        // etc.), `copy_spell_on_stack` returns Err and no copy is created (graceful no-op).
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Casualty(_),
            data: crate::state::stack::TriggerData::CasualtyCopy { original_stack_id },
        } => {
            let controller = stack_obj.controller;
            match crate::rules::copy::copy_spell_on_stack(
                state,
                original_stack_id,
                controller,
                false,
            ) {
                Ok((_, copy_evt)) => {
                    events.push(copy_evt);
                }
                Err(_) => {
                    // Original spell is no longer on the stack (e.g., was countered).
                    // The trigger does nothing (CR 400.7 — the original is a dead object).
                }
            }

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.56a: Replicate trigger resolves — create copies of the original spell.
        //
        // "When you cast this spell, if a replicate cost was paid for it, copy it for
        // each time its replicate cost was paid."
        // Copies are NOT cast (ruling 2024-01-12 for Shattering Spree / CR 707.10) —
        // they do not trigger "whenever you cast a spell" abilities.
        //
        // Reuses `create_storm_copies` which calls `copy_spell_on_stack` N times.
        // If the original spell is no longer on the stack (was countered), `create_storm_copies`
        // returns no events (graceful no-op). This is a known LOW gap — the 2024-01-12
        // ruling states copies should still be created even if the original is gone, but
        // the current copy infrastructure does not support this edge case.
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Replicate,
            data:
                crate::state::stack::TriggerData::SpellCopy {
                    original_stack_id,
                    copy_count: replicate_count,
                },
        } => {
            let controller = stack_obj.controller;
            let copy_events = crate::rules::copy::create_storm_copies(
                state,
                original_stack_id,
                controller,
                replicate_count,
            );
            events.extend(copy_events);

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.69a: Gravestorm trigger resolves — create copies of the original spell.
        //
        // "When you cast this spell, copy it for each permanent that was put into a
        // graveyard from the battlefield this turn."
        // Copies are NOT cast (CR 702.69a / CR 707.10) — they do not trigger "whenever
        // you cast a spell" abilities and do not increment `spells_cast_this_turn`.
        //
        // Reuses `create_storm_copies` which calls `copy_spell_on_stack` N times.
        // If the original spell is no longer on the stack (was countered), `create_storm_copies`
        // returns no events (graceful no-op).
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Gravestorm,
            data:
                crate::state::stack::TriggerData::SpellCopy {
                    original_stack_id,
                    copy_count: gravestorm_count,
                },
        } => {
            let controller = stack_obj.controller;
            let copy_events = crate::rules::copy::create_storm_copies(
                state,
                original_stack_id,
                controller,
                gravestorm_count,
            );
            events.extend(copy_events);

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.63a: Vanishing upkeep counter-removal trigger resolves.
        //
        // "At the beginning of your upkeep, if this permanent has a time counter on it,
        // remove a time counter from it."
        //
        // Intervening-if re-check (CR 603.4): permanent must still be on the battlefield
        // AND have at least one time counter. If either fails, the trigger does nothing.
        StackObjectKind::KeywordTrigger {
            keyword: KeywordAbility::Vanishing(_),
            data:
                crate::state::stack::TriggerData::CounterRemoval {
                    permanent: vanishing_permanent,
                },
            ..
        } => {
            let controller = stack_obj.controller;

            // CR 603.4: Re-check intervening-if condition at resolution.
            let current_counters = state
                .objects
                .get(&vanishing_permanent)
                .filter(|obj| obj.zone == ZoneId::Battlefield)
                .and_then(|obj| obj.counters.get(&CounterType::Time).copied());

            if let Some(count) = current_counters {
                if count > 0 {
                    // Remove one time counter (CR 702.63a).
                    if let Some(obj) = state.objects.get_mut(&vanishing_permanent) {
                        let new_count = count - 1;
                        if new_count == 0 {
                            obj.counters.remove(&CounterType::Time);
                        } else {
                            obj.counters.insert(CounterType::Time, new_count);
                        }
                    }
                    events.push(GameEvent::CounterRemoved {
                        object_id: vanishing_permanent,
                        counter: CounterType::Time,
                        count: 1,
                    });

                    // CR 702.63a (third triggered ability): "When the last time counter
                    // is removed from this permanent, sacrifice it."
                    // Queue a VanishingSacrifice trigger when the last counter was removed.
                    if count == 1 {
                        let owner = state
                            .objects
                            .get(&vanishing_permanent)
                            .map(|obj| obj.controller)
                            .unwrap_or(controller);
                        state
                            .pending_triggers
                            .push_back(crate::state::stubs::PendingTrigger {
                                source: vanishing_permanent,
                                ability_index: 0,
                                controller: owner,
                                kind: PendingTriggerKind::KeywordTrigger {
                                    keyword: crate::state::types::KeywordAbility::Vanishing(0),
                                    data: crate::state::stack::TriggerData::CounterSacrifice {
                                        permanent: vanishing_permanent,
                                    },
                                },
                                triggering_event: None,
                                entering_object_id: None,
                                targeting_stack_id: None,
                                triggering_player: None,
                                exalted_attacker_id: None,
                                defending_player_id: None,
                                ingest_target_player: None,
                                flanking_blocker_id: None,
                                rampage_n: None,
                                renown_n: None,
                                poisonous_n: None,
                                poisonous_target_player: None,
                                enlist_enlisted_creature: None,
                                recover_cost: None,
                                recover_card: None,
                                cipher_encoded_card_id: None,
                                cipher_encoded_object_id: None,
                                haunt_source_object_id: None,
                                haunt_source_card_id: None,
                                data: None,                            });
                    }
                }
            }
            // If not on battlefield or no counters, trigger does nothing (CR 603.4).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.63a: Vanishing sacrifice trigger resolves.
        //
        // "When the last time counter is removed from this permanent, sacrifice it."
        // If the source has left the battlefield by resolution time (CR 400.7),
        // the trigger does nothing — the permanent is a new object elsewhere.
        // CR 702.63a ruling (Dreamtide Whale): If the sacrifice trigger is countered
        // (e.g., Stifle), the permanent stays on the battlefield with 0 time counters
        // and neither trigger can fire again (both have intervening-if for time counters).
        StackObjectKind::KeywordTrigger {
            keyword: KeywordAbility::Vanishing(_),
            data:
                crate::state::stack::TriggerData::CounterSacrifice {
                    permanent: vanishing_permanent,
                },
            ..
        } => {
            let controller = stack_obj.controller;

            // Check if the source is still on the battlefield (CR 400.7).
            let source_info = state.objects.get(&vanishing_permanent).and_then(|obj| {
                if obj.zone == ZoneId::Battlefield {
                    Some((obj.owner, obj.controller, obj.counters.clone()))
                } else {
                    None
                }
            });

            if let Some((owner, pre_death_controller, pre_death_counters)) = source_info {
                // CR 701.17a: Sacrifice bypasses indestructible.
                // CR 614: Replacement effects (e.g., Rest in Peace) still apply.
                let action = crate::rules::replacement::check_zone_change_replacement(
                    state,
                    vanishing_permanent,
                    crate::state::zone::ZoneType::Battlefield,
                    crate::state::zone::ZoneType::Graveyard,
                    owner,
                    &std::collections::HashSet::new(),
                );

                match action {
                    crate::rules::replacement::ZoneChangeAction::Redirect {
                        to: dest,
                        events: repl_events,
                        ..
                    } => {
                        events.extend(repl_events);
                        if let Ok((new_id, _old)) =
                            state.move_object_to_zone(vanishing_permanent, dest)
                        {
                            match dest {
                                ZoneId::Exile => {
                                    events.push(GameEvent::ObjectExiled {
                                        player: owner,
                                        object_id: vanishing_permanent,
                                        new_exile_id: new_id,
                                    });
                                }
                                ZoneId::Command(_) => {
                                    // Commander redirected -- no sacrifice event.
                                }
                                _ => {
                                    events.push(GameEvent::CreatureDied {
                                        object_id: vanishing_permanent,
                                        new_grave_id: new_id,
                                        controller: pre_death_controller,
                                        pre_death_counters,
                                    });
                                }
                            }
                        }
                    }
                    crate::rules::replacement::ZoneChangeAction::Proceed => {
                        if let Ok((new_grave_id, _old)) =
                            state.move_object_to_zone(vanishing_permanent, ZoneId::Graveyard(owner))
                        {
                            events.push(GameEvent::CreatureDied {
                                object_id: vanishing_permanent,
                                new_grave_id,
                                controller: pre_death_controller,
                                pre_death_counters,
                            });
                        }
                    }
                    crate::rules::replacement::ZoneChangeAction::ChoiceRequired {
                        player,
                        choices,
                        event_description,
                    } => {
                        // CR 616.1: Multiple replacement effects -- defer to player choice.
                        state.pending_zone_changes.push_back(
                            crate::state::replacement_effect::PendingZoneChange {
                                object_id: vanishing_permanent,
                                original_from: crate::state::zone::ZoneType::Battlefield,
                                original_destination: crate::state::zone::ZoneType::Graveyard,
                                affected_player: player,
                                already_applied: Vec::new(),
                            },
                        );
                        events.push(GameEvent::ReplacementChoiceRequired {
                            player,
                            event_description,
                            choices,
                        });
                    }
                }
            }
            // If not on battlefield, do nothing (CR 400.7 -- permanent is a new object).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.32a: Fading upkeep trigger resolves.
        //
        // "At the beginning of your upkeep, remove a fade counter from this permanent.
        // If you can't, sacrifice the permanent."
        //
        // Single trigger handles both counter removal and sacrifice (unlike Vanishing's
        // two-trigger approach). If fade counters > 0, remove one. If 0, sacrifice.
        // No intervening-if condition -- the trigger fires unconditionally.
        // If the permanent has left the battlefield (CR 400.7), trigger does nothing.
        StackObjectKind::KeywordTrigger {
            keyword: KeywordAbility::Fading(_),
            data:
                crate::state::stack::TriggerData::CounterRemoval {
                    permanent: fading_permanent,
                },
            ..
        } => {
            let controller = stack_obj.controller;

            // Check if permanent is still on the battlefield (CR 400.7).
            let source_info = state.objects.get(&fading_permanent).and_then(|obj| {
                if obj.zone == ZoneId::Battlefield {
                    Some((
                        obj.owner,
                        obj.controller,
                        obj.counters.clone(),
                        obj.counters.get(&CounterType::Fade).copied().unwrap_or(0),
                    ))
                } else {
                    None
                }
            });

            if let Some((owner, pre_sacrifice_controller, pre_sacrifice_counters, fade_count)) =
                source_info
            {
                if fade_count > 0 {
                    // CR 702.32a: Remove one fade counter.
                    if let Some(obj) = state.objects.get_mut(&fading_permanent) {
                        let new_count = fade_count - 1;
                        if new_count == 0 {
                            obj.counters.remove(&CounterType::Fade);
                        } else {
                            obj.counters.insert(CounterType::Fade, new_count);
                        }
                    }
                    events.push(GameEvent::CounterRemoved {
                        object_id: fading_permanent,
                        counter: CounterType::Fade,
                        count: 1,
                    });
                } else {
                    // CR 702.32a: Can't remove a fade counter -- sacrifice the permanent.
                    // CR 701.17a: Sacrifice bypasses indestructible.
                    let action = crate::rules::replacement::check_zone_change_replacement(
                        state,
                        fading_permanent,
                        crate::state::zone::ZoneType::Battlefield,
                        crate::state::zone::ZoneType::Graveyard,
                        owner,
                        &std::collections::HashSet::new(),
                    );

                    match action {
                        crate::rules::replacement::ZoneChangeAction::Redirect {
                            to: dest,
                            events: repl_events,
                            ..
                        } => {
                            events.extend(repl_events);
                            if let Ok((new_id, _old)) =
                                state.move_object_to_zone(fading_permanent, dest)
                            {
                                match dest {
                                    ZoneId::Exile => {
                                        events.push(GameEvent::ObjectExiled {
                                            player: owner,
                                            object_id: fading_permanent,
                                            new_exile_id: new_id,
                                        });
                                    }
                                    ZoneId::Command(_) => {
                                        // Commander redirected -- no sacrifice event.
                                    }
                                    _ => {
                                        events.push(GameEvent::CreatureDied {
                                            object_id: fading_permanent,
                                            new_grave_id: new_id,
                                            controller: pre_sacrifice_controller,
                                            pre_death_counters: pre_sacrifice_counters,
                                        });
                                    }
                                }
                            }
                        }
                        crate::rules::replacement::ZoneChangeAction::Proceed => {
                            if let Ok((new_grave_id, _old)) = state
                                .move_object_to_zone(fading_permanent, ZoneId::Graveyard(owner))
                            {
                                events.push(GameEvent::CreatureDied {
                                    object_id: fading_permanent,
                                    new_grave_id,
                                    controller: pre_sacrifice_controller,
                                    pre_death_counters: pre_sacrifice_counters,
                                });
                            }
                        }
                        crate::rules::replacement::ZoneChangeAction::ChoiceRequired {
                            player,
                            choices,
                            event_description,
                        } => {
                            // CR 616.1: Multiple replacement effects -- defer to player choice.
                            state.pending_zone_changes.push_back(
                                crate::state::replacement_effect::PendingZoneChange {
                                    object_id: fading_permanent,
                                    original_from: crate::state::zone::ZoneType::Battlefield,
                                    original_destination: crate::state::zone::ZoneType::Graveyard,
                                    affected_player: player,
                                    already_applied: Vec::new(),
                                },
                            );
                            events.push(GameEvent::ReplacementChoiceRequired {
                                player,
                                event_description,
                                choices,
                            });
                        }
                    }
                }
            }
            // If not on battlefield, do nothing (CR 400.7 -- permanent is a new object).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.30a: Echo upkeep trigger resolves.
        //
        // "At the beginning of your upkeep, if this permanent came under your control
        // since the beginning of your last upkeep, sacrifice it unless you pay [cost]."
        //
        // On resolution: emit EchoPaymentRequired and add to pending_echo_payments.
        // The game pauses until a Command::PayEcho is received.
        // If the permanent has left the battlefield (CR 400.7), trigger does nothing.
        // echo_pending is cleared only in the PayEcho handler (not here), so that if
        // the trigger is countered (Stifle), it fires again on the next upkeep.
        StackObjectKind::KeywordTrigger {
            keyword: KeywordAbility::Echo(_),
            data:
                crate::state::stack::TriggerData::UpkeepCost {
                    permanent: echo_permanent,
                    cost: crate::state::stack::UpkeepCostKind::Echo(echo_cost),
                },
            ..
        } => {
            let controller = stack_obj.controller;

            // Check if permanent is still on the battlefield (CR 400.7).
            let still_on_battlefield = state
                .objects
                .get(&echo_permanent)
                .map(|obj| obj.zone == ZoneId::Battlefield)
                .unwrap_or(false);

            if still_on_battlefield {
                // Emit the payment required event and pause for player choice.
                events.push(GameEvent::EchoPaymentRequired {
                    player: controller,
                    permanent: echo_permanent,
                    cost: echo_cost.clone(),
                });
                // Track the pending payment so Command::PayEcho can find it.
                state
                    .pending_echo_payments
                    .push_back((controller, echo_permanent, echo_cost));
            }
            // If not on battlefield, do nothing (CR 400.7 -- permanent is a new object).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.24a: Cumulative upkeep trigger resolves.
        //
        // "At the beginning of your upkeep, if this permanent is on the battlefield,
        // put an age counter on this permanent. Then you may pay [cost] for each age
        // counter on it. If you don't, sacrifice it."
        //
        // On resolution:
        // 1. Check if permanent is still on the battlefield (CR 400.7).
        // 2. Add one age counter to the permanent.
        // 3. Emit CumulativeUpkeepPaymentRequired with the age count.
        // 4. Add to pending_cumulative_upkeep_payments.
        // If countered (Stifle), no age counter is added -- the trigger fires again
        // next upkeep with the same counter count.
        StackObjectKind::KeywordTrigger {
            keyword: KeywordAbility::CumulativeUpkeep(_),
            data:
                crate::state::stack::TriggerData::UpkeepCost {
                    permanent: cu_permanent,
                    cost: crate::state::stack::UpkeepCostKind::CumulativeUpkeep(per_counter_cost),
                },
            ..
        } => {
            let controller = stack_obj.controller;

            // Check if permanent is still on the battlefield (CR 400.7).
            let still_on_battlefield = state
                .objects
                .get(&cu_permanent)
                .map(|obj| obj.zone == ZoneId::Battlefield)
                .unwrap_or(false);

            if still_on_battlefield {
                // CR 702.24a: "put an age counter on this permanent"
                if let Some(obj) = state.objects.get_mut(&cu_permanent) {
                    let current = obj.counters.get(&CounterType::Age).copied().unwrap_or(0);
                    obj.counters.insert(CounterType::Age, current + 1);
                }

                // Count total age counters after adding.
                let age_count = state
                    .objects
                    .get(&cu_permanent)
                    .and_then(|obj| obj.counters.get(&CounterType::Age).copied())
                    .unwrap_or(0);

                // Emit payment required event.
                events.push(GameEvent::CumulativeUpkeepPaymentRequired {
                    player: controller,
                    permanent: cu_permanent,
                    per_counter_cost: per_counter_cost.clone(),
                    age_counter_count: age_count,
                });

                // Track pending payment.
                state.pending_cumulative_upkeep_payments.push_back((
                    controller,
                    cu_permanent,
                    per_counter_cost,
                ));
            }
            // If not on battlefield, do nothing (CR 400.7 -- permanent is a new object).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.59a: Recover trigger resolves.
        //
        // "When a creature is put into your graveyard from the battlefield, you may
        // pay [cost]. If you do, return this card from your graveyard to your hand.
        // Otherwise, exile this card."
        //
        // On resolution: check if recover_card is still in the graveyard (CR 400.7).
        // If yes, emit RecoverPaymentRequired and add to pending_recover_payments.
        // If not, do nothing (card is a new object elsewhere).
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Recover,
            data:
                crate::state::stack::TriggerData::DeathRecover {
                    recover_card,
                    recover_cost,
                },
        } => {
            let controller = stack_obj.controller;

            // Check if the Recover card is still in the graveyard (CR 400.7).
            let still_in_graveyard = state
                .objects
                .get(&recover_card)
                .map(|obj| matches!(obj.zone, ZoneId::Graveyard(_)))
                .unwrap_or(false);

            if still_in_graveyard {
                // Emit the payment required event and pause for player choice.
                events.push(GameEvent::RecoverPaymentRequired {
                    player: controller,
                    recover_card,
                    cost: recover_cost.clone(),
                });
                // Track pending payment so Command::PayRecover can find it.
                state
                    .pending_recover_payments
                    .push_back((controller, recover_card, recover_cost));
            }
            // If not in graveyard, do nothing (CR 400.7 -- card is a new object).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.74a: Evoke sacrifice trigger resolves — sacrifice the source permanent.
        //
        // "When this permanent enters, if its evoke cost was paid, its controller
        // sacrifices it." If the source has left the battlefield by resolution time
        // (blinked, bounced, etc.), the sacrifice does nothing — CR 400.7 ensures
        // the source is now a new object and is no longer the evoked permanent.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Evoke,
            data: crate::state::stack::TriggerData::DelayedZoneChange,
        } => {
            let controller = stack_obj.controller;

            // Check if the source is still on the battlefield (CR 400.7).
            let source_info = state.objects.get(&source_object).and_then(|obj| {
                if obj.zone == ZoneId::Battlefield {
                    Some((obj.owner, obj.controller, obj.counters.clone()))
                } else {
                    None
                }
            });

            if let Some((owner, pre_sacrifice_controller, pre_death_counters)) = source_info {
                // CR 701.17a: Sacrifice is NOT destruction — no indestructible check.
                // CR 614: Check replacement effects before moving to graveyard.
                let action = crate::rules::replacement::check_zone_change_replacement(
                    state,
                    source_object,
                    crate::state::zone::ZoneType::Battlefield,
                    crate::state::zone::ZoneType::Graveyard,
                    owner,
                    &std::collections::HashSet::new(),
                );

                match action {
                    crate::rules::replacement::ZoneChangeAction::Redirect {
                        to: dest,
                        events: repl_events,
                        ..
                    } => {
                        events.extend(repl_events);
                        if let Ok((new_id, _old)) = state.move_object_to_zone(source_object, dest) {
                            match dest {
                                ZoneId::Exile => {
                                    events.push(GameEvent::ObjectExiled {
                                        player: owner,
                                        object_id: source_object,
                                        new_exile_id: new_id,
                                    });
                                }
                                ZoneId::Command(_) => {
                                    // Commander redirected — no sacrifice event.
                                }
                                _ => {
                                    events.push(GameEvent::CreatureDied {
                                        object_id: source_object,
                                        new_grave_id: new_id,
                                        controller: pre_sacrifice_controller,
                                        pre_death_counters,
                                    });
                                }
                            }
                        }
                    }
                    crate::rules::replacement::ZoneChangeAction::Proceed => {
                        if let Ok((new_grave_id, _old)) =
                            state.move_object_to_zone(source_object, ZoneId::Graveyard(owner))
                        {
                            events.push(GameEvent::CreatureDied {
                                object_id: source_object,
                                new_grave_id,
                                controller: pre_sacrifice_controller,
                                pre_death_counters,
                            });
                        }
                    }
                    crate::rules::replacement::ZoneChangeAction::ChoiceRequired {
                        player,
                        choices,
                        event_description,
                    } => {
                        // CR 616.1: Multiple replacement effects — defer to player choice.
                        use crate::state::replacement_effect::PendingZoneChange;
                        state.pending_zone_changes.push_back(PendingZoneChange {
                            object_id: source_object,
                            original_from: crate::state::zone::ZoneType::Battlefield,
                            original_destination: crate::state::zone::ZoneType::Graveyard,
                            affected_player: player,
                            already_applied: Vec::new(),
                        });
                        events.push(GameEvent::ReplacementChoiceRequired {
                            player,
                            event_description,
                            choices,
                        });
                    }
                }
            }
            // If source is not on the battlefield, the trigger does nothing (CR 400.7).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.35a: Madness triggered ability resolves.
        //
        // "When this card is exiled this way, its owner may cast it by paying [cost]
        // rather than paying its mana cost. If that player doesn't, they put this
        // card into their graveyard."
        //
        // MVP: Auto-decline — card goes to graveyard. The player can also cast the
        // card from exile (before the trigger resolves) using CastSpell, which
        // handle_cast_spell allows when the card has KeywordAbility::Madness and is
        // in ZoneId::Exile.
        StackObjectKind::MadnessTrigger {
            source_object: _,
            exiled_card,
            madness_cost: _,
            owner,
        } => {
            let controller = stack_obj.controller;

            // CR 702.35a: Check if the card is still in exile (CR 400.7).
            // If the owner already cast it (or it moved via another effect), do nothing.
            let still_in_exile = state
                .objects
                .get(&exiled_card)
                .map(|obj| obj.zone == ZoneId::Exile)
                .unwrap_or(false);

            if still_in_exile {
                // Auto-decline: move the card from exile to its owner's graveyard.
                if let Ok((new_grave_id, _)) =
                    state.move_object_to_zone(exiled_card, ZoneId::Graveyard(owner))
                {
                    events.push(GameEvent::ObjectPutInGraveyard {
                        player: owner,
                        object_id: exiled_card,
                        new_grave_id,
                    });
                }
            }

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.94a: Miracle triggered ability resolves.
        //
        // "When you reveal this card this way, you may cast it by paying [cost]
        // rather than its mana cost."
        //
        // The player's window to cast for miracle cost is while this trigger was
        // on the stack (they had priority). They use `CastSpell` with
        // `cast_with_miracle: true`. On resolution, the trigger just expires.
        // If the card is still in hand (player did not cast it), it stays there.
        // If the card was already cast (left hand), nothing to do (CR 400.7).
        StackObjectKind::MiracleTrigger {
            source_object: _,
            revealed_card: _,
            miracle_cost: _,
            owner: _,
        } => {
            let controller = stack_obj.controller;
            // CR 702.94a: The trigger resolves — nothing happens to the card here.
            // If the player cast it (CastSpell with cast_with_miracle: true), it was
            // already moved to the stack when cast. If they declined, the card remains
            // in hand normally. No auto-movement is needed.
            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.84a: Unearth activated ability resolves.
        //
        // "Return this card from your graveyard to the battlefield. It gains haste."
        // The card stays in the graveyard until this ability resolves; it is NOT
        // moved to the stack as a cost. Per ruling: "If that card is removed from
        // your graveyard before the ability resolves, that unearth ability will
        // resolve and do nothing." (CR 400.7)
        StackObjectKind::UnearthAbility { source_object } => {
            let controller = stack_obj.controller;

            // Check if the source card is still in the graveyard (CR 400.7).
            let still_in_graveyard = state
                .objects
                .get(&source_object)
                .map(|obj| matches!(obj.zone, ZoneId::Graveyard(_)))
                .unwrap_or(false);

            if still_in_graveyard {
                // 1. Move card from graveyard to battlefield (CR 702.84a).
                let (new_id, _old) =
                    state.move_object_to_zone(source_object, ZoneId::Battlefield)?;

                // 2. Set controller, was_unearthed flag, and grant haste.
                //    CR 702.84a: "It gains haste."
                //    CR 702.84a: The exile effects are NOT granted to the creature
                //    (per ruling) -- they are tracked via was_unearthed flag on the
                //    object, which persists even if the creature loses all abilities.
                let card_id = state.objects.get(&new_id).and_then(|o| o.card_id.clone());
                if let Some(obj) = state.objects.get_mut(&new_id) {
                    obj.controller = controller;
                    obj.was_unearthed = true;
                    // CR 702.10a: Haste — can attack and use tap abilities immediately.
                    obj.characteristics.keywords.insert(KeywordAbility::Haste);
                }

                // 3. Apply self ETB replacements (e.g., "enters tapped") and global
                //    ETB replacements (Rest in Peace, etc.) before emitting PEB event.
                let registry = state.card_registry.clone();
                let self_evts = super::replacement::apply_self_etb_from_definition(
                    state,
                    new_id,
                    controller,
                    card_id.as_ref(),
                    &registry,
                );
                events.extend(self_evts);
                let etb_evts =
                    super::replacement::apply_etb_replacements(state, new_id, controller);
                events.extend(etb_evts);

                // 4. Register replacement abilities and static continuous effects.
                super::replacement::register_permanent_replacement_abilities(
                    state,
                    new_id,
                    controller,
                    card_id.as_ref(),
                    &registry,
                );
                super::replacement::register_static_continuous_effects(
                    state,
                    new_id,
                    card_id.as_ref(),
                    &registry,
                );

                // 5. Emit PermanentEnteredBattlefield.
                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: controller,
                    object_id: new_id,
                });

                // 6. Queue WhenEntersBattlefield triggered abilities from card definition.
                // CR 603.3: goes on stack at next priority window.
                let etb_trigger_evts = super::replacement::queue_carddef_etb_triggers(
                    state,
                    new_id,
                    controller,
                    card_id.as_ref(),
                    &registry,
                );
                events.extend(etb_trigger_evts);
            }

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.84a: Unearth delayed triggered ability resolves.
        //
        // "Exile it at the beginning of the next end step."
        // This is a delayed triggered ability created when the unearthed permanent
        // entered the battlefield. If countered (e.g., by Stifle), the permanent
        // stays on the battlefield, but the replacement effect still applies (per ruling).
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Unearth,
            data: crate::state::stack::TriggerData::DelayedZoneChange,
        } => {
            let controller = stack_obj.controller;

            // Check if the source is still on the battlefield (CR 400.7).
            let owner_opt = state
                .objects
                .get(&source_object)
                .filter(|obj| obj.zone == ZoneId::Battlefield)
                .map(|obj| obj.owner);

            if let Some(owner) = owner_opt {
                // Exile the permanent directly. No zone-change replacement needed:
                // the replacement effect only fires when the permanent would go to a
                // NON-exile zone. Here we are already exiling it, so no replacement applies.
                let (new_exile_id, _old) =
                    state.move_object_to_zone(source_object, ZoneId::Exile)?;
                events.push(GameEvent::ObjectExiled {
                    player: owner,
                    object_id: source_object,
                    new_exile_id,
                });
            }
            // If not on battlefield, do nothing (already exiled by replacement or removed).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.109a: Dash delayed triggered ability resolves.
        //
        // "Return the permanent this spell becomes to its owner's hand at the
        // beginning of the next end step."
        // If the source has left the battlefield by resolution time (CR 400.7),
        // the trigger does nothing -- the creature is a new object elsewhere.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Dash,
            data: crate::state::stack::TriggerData::DelayedZoneChange,
        } => {
            let controller = stack_obj.controller;

            // Check if the source is still on the battlefield (CR 400.7).
            let owner_opt = state
                .objects
                .get(&source_object)
                .filter(|obj| obj.zone == ZoneId::Battlefield)
                .map(|obj| obj.owner);

            if let Some(owner) = owner_opt {
                // Return to owner's hand (not controller's -- CR 702.109a says "owner's hand").
                let (new_hand_id, _old) =
                    state.move_object_to_zone(source_object, ZoneId::Hand(owner))?;

                events.push(GameEvent::ObjectReturnedToHand {
                    player: owner,
                    object_id: source_object,
                    new_hand_id,
                });
            }
            // If not on battlefield, do nothing (CR 400.7 -- creature is a new object).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.152a: Blitz delayed triggered ability resolves.
        //
        // "Sacrifice the permanent this spell becomes at the beginning of the
        // next end step."
        // If the source has left the battlefield by resolution time (CR 400.7),
        // the trigger does nothing -- the creature is a new object elsewhere.
        // Ruling 2022-04-29: "if it's still on the battlefield when that triggered
        // ability resolves. If it dies or goes to another zone before then, it will
        // stay where it is."
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Blitz,
            data: crate::state::stack::TriggerData::DelayedZoneChange,
        } => {
            let controller = stack_obj.controller;

            // Check if the source is still on the battlefield (CR 400.7).
            let source_info = state.objects.get(&source_object).and_then(|obj| {
                if obj.zone == ZoneId::Battlefield {
                    Some((obj.owner, obj.controller, obj.counters.clone()))
                } else {
                    None
                }
            });

            if let Some((owner, pre_death_controller, pre_death_counters)) = source_info {
                // Sacrifice: move to owner's graveyard.
                // Sacrifice bypasses indestructible (CR 701.17a).
                // Replacement effects (e.g., Rest in Peace) still apply.
                let action = crate::rules::replacement::check_zone_change_replacement(
                    state,
                    source_object,
                    crate::state::zone::ZoneType::Battlefield,
                    crate::state::zone::ZoneType::Graveyard,
                    owner,
                    &std::collections::HashSet::new(),
                );

                match action {
                    crate::rules::replacement::ZoneChangeAction::Redirect {
                        to: dest,
                        events: repl_events,
                        ..
                    } => {
                        events.extend(repl_events);
                        if let Ok((new_id, _old)) = state.move_object_to_zone(source_object, dest) {
                            match dest {
                                ZoneId::Exile => {
                                    events.push(GameEvent::ObjectExiled {
                                        player: owner,
                                        object_id: source_object,
                                        new_exile_id: new_id,
                                    });
                                }
                                ZoneId::Command(_) => {
                                    // Commander redirected -- no sacrifice event.
                                }
                                _ => {
                                    events.push(GameEvent::CreatureDied {
                                        object_id: source_object,
                                        new_grave_id: new_id,
                                        controller: pre_death_controller,
                                        pre_death_counters,
                                    });
                                }
                            }
                        }
                    }
                    crate::rules::replacement::ZoneChangeAction::Proceed => {
                        if let Ok((new_grave_id, _old)) =
                            state.move_object_to_zone(source_object, ZoneId::Graveyard(owner))
                        {
                            events.push(GameEvent::CreatureDied {
                                object_id: source_object,
                                new_grave_id,
                                controller: pre_death_controller,
                                pre_death_counters,
                            });
                        }
                    }
                    crate::rules::replacement::ZoneChangeAction::ChoiceRequired {
                        player,
                        choices,
                        event_description,
                    } => {
                        // CR 616.1: Multiple replacement effects -- defer to player choice.
                        use crate::state::replacement_effect::PendingZoneChange;
                        state.pending_zone_changes.push_back(PendingZoneChange {
                            object_id: source_object,
                            original_from: crate::state::zone::ZoneType::Battlefield,
                            original_destination: crate::state::zone::ZoneType::Graveyard,
                            affected_player: player,
                            already_applied: Vec::new(),
                        });
                        events.push(GameEvent::ReplacementChoiceRequired {
                            player,
                            event_description,
                            choices,
                        });
                    }
                }
            }
            // If not on battlefield, do nothing (CR 400.7 -- creature is a new object).
            // Ruling 2022-04-29: "it will stay where it is"

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.176a: Impending counter-removal trigger resolves.
        //
        // "At the beginning of your end step, if this permanent's impending cost
        // was paid and it has a time counter on it, remove a time counter from it."
        //
        // Intervening-if re-check (CR 603.4): permanent must still be on the
        // battlefield, must have cast_alt_cost == Impending, and must have at
        // least one time counter.
        StackObjectKind::KeywordTrigger {
            keyword: KeywordAbility::Impending,
            data:
                crate::state::stack::TriggerData::CounterRemoval {
                    permanent: impending_permanent,
                },
            ..
        } => {
            let controller = stack_obj.controller;

            // CR 603.4: Re-check intervening-if condition at resolution.
            let current_counters = state
                .objects
                .get(&impending_permanent)
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.cast_alt_cost == Some(AltCostKind::Impending)
                })
                .and_then(|obj| obj.counters.get(&CounterType::Time).copied());

            if let Some(count) = current_counters {
                if count > 0 {
                    // Remove one time counter (CR 702.176a).
                    if let Some(obj) = state.objects.get_mut(&impending_permanent) {
                        let new_count = count - 1;
                        if new_count == 0 {
                            obj.counters.remove(&CounterType::Time);
                        } else {
                            obj.counters.insert(CounterType::Time, new_count);
                        }
                    }
                    events.push(GameEvent::CounterRemoved {
                        object_id: impending_permanent,
                        counter: CounterType::Time,
                        count: 1,
                    });
                    // No follow-up trigger when last counter removed -- the
                    // permanent simply becomes a creature because the Layer 4
                    // type-removal effect in calculate_characteristics stops
                    // applying (no time counters => condition is false).
                }
            }
            // If not on battlefield, or no impending status, or no counters,
            // the trigger does nothing (CR 603.4).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // --- Group 1: Combat triggers (migrated to KeywordTrigger) ---

        // CR 702.25a: Flanking -- blocker gets -1/-1 until end of turn.
        StackObjectKind::KeywordTrigger {
            keyword: KeywordAbility::Flanking,
            data: crate::state::stack::TriggerData::CombatFlanking { blocker },
            ..
        } => {
            let controller = stack_obj.controller;
            let blocker_alive = state
                .objects
                .get(&blocker)
                .map(|obj| obj.zone == ZoneId::Battlefield)
                .unwrap_or(false);
            if blocker_alive {
                let eff_id = state.next_object_id().0;
                let ts = state.timestamp_counter;
                state.timestamp_counter += 1;
                state.continuous_effects.push_back(
                    crate::state::continuous_effect::ContinuousEffect {
                        id: crate::state::continuous_effect::EffectId(eff_id),
                        source: None,
                        timestamp: ts,
                        layer: crate::state::continuous_effect::EffectLayer::PtModify,
                        duration: crate::state::continuous_effect::EffectDuration::UntilEndOfTurn,
                        filter: crate::state::continuous_effect::EffectFilter::SingleObject(
                            blocker,
                        ),
                        modification:
                            crate::state::continuous_effect::LayerModification::ModifyBoth(-1),
                        is_cda: false,
                    },
                );
            }
            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.23a: Rampage N -- +N/+N for each blocker beyond the first.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Rampage(_),
            data: crate::state::stack::TriggerData::CombatRampage { n: rampage_n },
        } => {
            let controller = stack_obj.controller;
            let blocker_count = state
                .combat
                .as_ref()
                .map(|c| c.blockers_for(source_object).len())
                .unwrap_or(0);
            let beyond_first = blocker_count.saturating_sub(1);
            let bonus = (beyond_first as i32) * (rampage_n as i32);
            if bonus > 0 {
                let source_alive = state
                    .objects
                    .get(&source_object)
                    .map(|obj| obj.zone == ZoneId::Battlefield)
                    .unwrap_or(false);
                if source_alive {
                    let eff_id = state.next_object_id().0;
                    let ts = state.timestamp_counter;
                    state.timestamp_counter += 1;
                    state.continuous_effects.push_back(
                        crate::state::continuous_effect::ContinuousEffect {
                            id: crate::state::continuous_effect::EffectId(eff_id),
                            source: None,
                            timestamp: ts,
                            layer: crate::state::continuous_effect::EffectLayer::PtModify,
                            duration:
                                crate::state::continuous_effect::EffectDuration::UntilEndOfTurn,
                            filter: crate::state::continuous_effect::EffectFilter::SingleObject(
                                source_object,
                            ),
                            modification:
                                crate::state::continuous_effect::LayerModification::ModifyBoth(
                                    bonus,
                                ),
                            is_cda: false,
                        },
                    );
                }
            }
            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.39a: Provoke -- untap provoked creature, add forced-block requirement.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Provoke,
            data:
                crate::state::stack::TriggerData::CombatProvoke {
                    target: provoked_creature,
                },
        } => {
            let controller = stack_obj.controller;
            let target_valid = state
                .objects
                .get(&provoked_creature)
                .map(|obj| obj.zone == ZoneId::Battlefield)
                .unwrap_or(false);
            if target_valid {
                if let Some(obj) = state.objects.get(&provoked_creature) {
                    if obj.status.tapped {
                        let provoked_controller = obj.controller;
                        if let Some(obj_mut) = state.objects.get_mut(&provoked_creature) {
                            obj_mut.status.tapped = false;
                        }
                        events.push(GameEvent::PermanentUntapped {
                            player: provoked_controller,
                            object_id: provoked_creature,
                        });
                    }
                }
                if let Some(combat) = state.combat.as_mut() {
                    combat
                        .forced_blocks
                        .insert(provoked_creature, source_object);
                }
            }
            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.112a: Renown N -- place N +1/+1 counters and set renowned.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Renown(_),
            data: crate::state::stack::TriggerData::RenownDamage { n: renown_n },
        } => {
            let controller = stack_obj.controller;
            let should_resolve = state
                .objects
                .get(&source_object)
                .map(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && !obj.designations.contains(Designations::RENOWNED)
                })
                .unwrap_or(false);
            if should_resolve {
                if let Some(obj) = state.objects.get_mut(&source_object) {
                    let current = obj
                        .counters
                        .get(&CounterType::PlusOnePlusOne)
                        .copied()
                        .unwrap_or(0);
                    obj.counters = obj
                        .counters
                        .update(CounterType::PlusOnePlusOne, current + renown_n);
                    obj.designations.insert(Designations::RENOWNED);
                }
            }
            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.121a: Melee -- +count/+count for each opponent attacked.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Melee,
            data: crate::state::stack::TriggerData::Simple,
        } => {
            let controller = stack_obj.controller;
            let opponents_attacked = state
                .combat
                .as_ref()
                .map(|c| {
                    c.attackers
                        .values()
                        .filter_map(|target| {
                            if let crate::state::combat::AttackTarget::Player(pid) = target {
                                Some(*pid)
                            } else {
                                None
                            }
                        })
                        .collect::<OrdSet<crate::state::player::PlayerId>>()
                        .len()
                })
                .unwrap_or(0);
            let bonus = opponents_attacked as i32;
            if bonus > 0 {
                let source_alive = state
                    .objects
                    .get(&source_object)
                    .map(|obj| obj.zone == ZoneId::Battlefield)
                    .unwrap_or(false);
                if source_alive {
                    let eff_id = state.next_object_id().0;
                    let ts = state.timestamp_counter;
                    state.timestamp_counter += 1;
                    state.continuous_effects.push_back(
                        crate::state::continuous_effect::ContinuousEffect {
                            id: crate::state::continuous_effect::EffectId(eff_id),
                            source: None,
                            timestamp: ts,
                            layer: crate::state::continuous_effect::EffectLayer::PtModify,
                            duration:
                                crate::state::continuous_effect::EffectDuration::UntilEndOfTurn,
                            filter: crate::state::continuous_effect::EffectFilter::SingleObject(
                                source_object,
                            ),
                            modification:
                                crate::state::continuous_effect::LayerModification::ModifyBoth(
                                    bonus,
                                ),
                            is_cda: false,
                        },
                    );
                }
            }
            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.70a: Poisonous N -- give target player N poison counters.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Poisonous(_),
            data:
                crate::state::stack::TriggerData::CombatPoisonous {
                    target_player,
                    n: poisonous_n,
                },
        } => {
            let controller = stack_obj.controller;
            if let Some(player) = state.players.get_mut(&target_player) {
                player.poison_counters += poisonous_n;
            }
            events.push(GameEvent::PoisonCountersGiven {
                player: target_player,
                amount: poisonous_n,
                source: source_object,
            });
            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.154a: Enlist -- source gets +X/+0 where X is enlisted creature's power.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Enlist,
            data: crate::state::stack::TriggerData::CombatEnlist { enlisted },
        } => {
            let controller = stack_obj.controller;
            let source_alive = state
                .objects
                .get(&source_object)
                .map(|obj| obj.zone == ZoneId::Battlefield)
                .unwrap_or(false);
            if source_alive {
                let enlisted_power =
                    crate::rules::layers::calculate_characteristics(state, enlisted)
                        .and_then(|c| c.power)
                        .unwrap_or(0);
                if enlisted_power != 0 {
                    let eff_id = state.next_object_id().0;
                    let ts = state.timestamp_counter;
                    state.timestamp_counter += 1;
                    state.continuous_effects.push_back(
                        crate::state::continuous_effect::ContinuousEffect {
                            id: crate::state::continuous_effect::EffectId(eff_id),
                            source: None,
                            timestamp: ts,
                            layer: crate::state::continuous_effect::EffectLayer::PtModify,
                            duration:
                                crate::state::continuous_effect::EffectDuration::UntilEndOfTurn,
                            filter: crate::state::continuous_effect::EffectFilter::SingleObject(
                                source_object,
                            ),
                            modification:
                                crate::state::continuous_effect::LayerModification::ModifyPower(
                                    enlisted_power,
                                ),
                            is_cda: false,
                        },
                    );
                }
            }
            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.110a: Exploit trigger resolves -- the controller may sacrifice
        // a creature. Default (deterministic, no interactive choice): decline.
        //
        // TODO: Add Command::ExploitCreature for player-interactive sacrifice choice.
        // When interactive choice is added, the trigger would pause and emit an
        // ExploitChoiceRequired event; the player responds with ExploitCreature
        // (naming the creature to sacrifice) or DeclineExploit.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Exploit,
            data: crate::state::stack::TriggerData::Simple,
        } => {
            let controller = stack_obj.controller;

            // CR 400.7: Check if the source is still on the battlefield.
            // If it left (blinked, bounced, destroyed), the trigger still resolves
            // but there's nothing to "exploit with." (Check is informational only;
            // the default "decline sacrifice" path is identical in both cases.)
            let _source_on_bf = state
                .objects
                .get(&source_object)
                .is_some_and(|obj| obj.zone == ZoneId::Battlefield);

            // Default: decline sacrifice. No creature is sacrificed.
            // The trigger resolves with no effect.
            // NOTE: Even though the sacrifice is declined, the ability DID resolve.
            // "When this creature exploits a creature" secondary triggers do NOT fire
            // because no creature was sacrificed (CR 702.110b).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.43a: Modular trigger resolves -- put +1/+1 counters on target
        // artifact creature equal to counter_count (last-known information from
        // pre_death_counters, Arcbound Worker ruling 2006-09-25).
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Modular(_),
            data: crate::state::stack::TriggerData::DeathModular { counter_count },
        } => {
            let controller = stack_obj.controller;

            // CR 608.2b: Fizzle check -- verify target is still a legal artifact creature
            // on the battlefield. If it is not, the trigger fizzles with no effect.
            let target_id_opt = stack_obj.targets.first().and_then(|t| match &t.target {
                Target::Object(id) => {
                    // CR 613.1d: Use layer-resolved types for artifact creature check.
                    let still_legal = state.objects.get(id).is_some_and(|obj| {
                        obj.zone == ZoneId::Battlefield && {
                            let chars = crate::rules::layers::calculate_characteristics(state, *id)
                                .unwrap_or_else(|| obj.characteristics.clone());
                            chars.card_types.contains(&CardType::Artifact)
                                && chars.card_types.contains(&CardType::Creature)
                        }
                    });
                    if still_legal {
                        Some(*id)
                    } else {
                        None
                    }
                }
                _ => None,
            });

            if let Some(target_id) = target_id_opt {
                if counter_count > 0 {
                    if let Some(obj) = state.objects.get_mut(&target_id) {
                        let current = obj
                            .counters
                            .get(&CounterType::PlusOnePlusOne)
                            .copied()
                            .unwrap_or(0);
                        obj.counters = obj
                            .counters
                            .update(CounterType::PlusOnePlusOne, current + counter_count);
                    }
                    events.push(GameEvent::CounterAdded {
                        object_id: target_id,
                        counter: CounterType::PlusOnePlusOne,
                        count: counter_count,
                    });
                }
            }
            // If target illegal (fizzled) or counter_count == 0, do nothing.

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.100a: Evolve trigger resolves -- re-check the intervening-if
        // condition (CR 603.4) and place a +1/+1 counter on the source creature
        // if the entering creature still has greater P and/or T.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Evolve,
            data: crate::state::stack::TriggerData::ETBEvolve { entering_creature },
        } => {
            let controller = stack_obj.controller;

            // CR 603.4: Resolution-time intervening-if re-check.
            // Compare entering creature's P/T vs evolve creature's P/T.
            //
            // Ruling 2013-04-15: "If the creature that entered the battlefield
            // leaves the battlefield before evolve tries to resolve, use its
            // last known power and toughness to compare the stats."
            //
            // Use calculate_characteristics for layer-aware P/T; fall back to
            // raw characteristics for objects that left the battlefield.
            let entering_chars =
                crate::rules::layers::calculate_characteristics(state, entering_creature).or_else(
                    || {
                        state
                            .objects
                            .get(&entering_creature)
                            .map(|o| o.characteristics.clone())
                    },
                );

            let evolve_chars =
                crate::rules::layers::calculate_characteristics(state, source_object).or_else(
                    || {
                        state
                            .objects
                            .get(&source_object)
                            .map(|o| o.characteristics.clone())
                    },
                );

            let condition_holds = match (entering_chars, evolve_chars) {
                (Some(entering), Some(evolve)) => {
                    let ep = entering.power.unwrap_or(0);
                    let et = entering.toughness.unwrap_or(0);
                    let sp = evolve.power.unwrap_or(0);
                    let st = evolve.toughness.unwrap_or(0);
                    // CR 702.100a: "greater than this creature's power and/or
                    // that creature's toughness is greater than this creature's
                    // toughness" — inclusive OR.
                    ep > sp || et > st
                }
                // One or both objects no longer exist — condition fails (conservative).
                _ => false,
            };

            if condition_holds {
                // CR 702.100a: Put a +1/+1 counter on the evolve creature.
                // The source must still be on the battlefield.
                if let Some(obj) = state.objects.get_mut(&source_object) {
                    if obj.zone == ZoneId::Battlefield {
                        let current = obj
                            .counters
                            .get(&CounterType::PlusOnePlusOne)
                            .copied()
                            .unwrap_or(0);
                        obj.counters = obj
                            .counters
                            .update(CounterType::PlusOnePlusOne, current + 1);

                        events.push(GameEvent::CounterAdded {
                            object_id: source_object,
                            counter: CounterType::PlusOnePlusOne,
                            count: 1,
                        });
                    }
                }
            }

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.58a: Graft trigger resolves -- re-check the intervening-if condition
        // (CR 603.4) and move one +1/+1 counter from the graft source to the entering
        // creature if both conditions still hold.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Graft(_),
            data: crate::state::stack::TriggerData::ETBGraft { entering_creature },
        } => {
            let controller = stack_obj.controller;

            // CR 603.4: Re-check intervening-if at resolution time.
            // Source must still be on the battlefield AND have at least one +1/+1 counter.
            let source_has_counter = state
                .objects
                .get(&source_object)
                .map(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj
                            .counters
                            .get(&CounterType::PlusOnePlusOne)
                            .copied()
                            .unwrap_or(0)
                            > 0
                })
                .unwrap_or(false);

            // The entering creature must still be on the battlefield.
            let target_on_battlefield = state
                .objects
                .get(&entering_creature)
                .map(|obj| obj.zone == ZoneId::Battlefield)
                .unwrap_or(false);

            if source_has_counter && target_on_battlefield {
                // CR 702.58a: "you may move a +1/+1 counter" -- auto-accept (always move).
                // Remove one +1/+1 counter from source.
                if let Some(obj) = state.objects.get_mut(&source_object) {
                    let current = obj
                        .counters
                        .get(&CounterType::PlusOnePlusOne)
                        .copied()
                        .unwrap_or(0);
                    if current > 1 {
                        obj.counters = obj
                            .counters
                            .update(CounterType::PlusOnePlusOne, current - 1);
                    } else {
                        obj.counters = obj.counters.without(&CounterType::PlusOnePlusOne);
                    }
                }
                events.push(GameEvent::CounterRemoved {
                    object_id: source_object,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                });

                // Add one +1/+1 counter to entering creature.
                if let Some(obj) = state.objects.get_mut(&entering_creature) {
                    let current = obj
                        .counters
                        .get(&CounterType::PlusOnePlusOne)
                        .copied()
                        .unwrap_or(0);
                    obj.counters = obj
                        .counters
                        .update(CounterType::PlusOnePlusOne, current + 1);
                }
                events.push(GameEvent::CounterAdded {
                    object_id: entering_creature,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                });
            }
            // If either condition fails, the trigger fizzles silently.

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.97a: Scavenge activated ability resolves.
        // "Put a number of +1/+1 counters equal to the power of the card you exiled
        // on target creature."
        //
        // The card was already exiled as cost at activation time. `power_snapshot`
        // holds the card's power as it last existed in the graveyard (Varolz ruling
        // 2013-04-15).
        //
        // Fizzle check (CR 608.2b): if the target creature is no longer on the
        // battlefield or is no longer a creature, the ability does nothing.
        StackObjectKind::ScavengeAbility {
            source_card_id: _,
            power_snapshot,
        } => {
            let controller = stack_obj.controller;

            // Extract the target creature from the stack object's targets.
            let target_creature_id = stack_obj.targets.first().and_then(|t| {
                if let crate::state::targeting::Target::Object(id) = t.target {
                    Some(id)
                } else {
                    None
                }
            });

            let target_id = match target_creature_id {
                Some(id) => id,
                None => {
                    // No target recorded -- ability fizzles.
                    events.push(GameEvent::AbilityResolved {
                        controller,
                        stack_object_id: stack_obj.id,
                    });
                    return Ok(events);
                }
            };

            // CR 608.2b: Fizzle check -- target must still be on the battlefield and
            // must still be a creature at resolution time.
            let target_valid = state
                .objects
                .get(&target_id)
                .map(|obj| {
                    if obj.zone != ZoneId::Battlefield {
                        return false;
                    }
                    // Re-check creature type via layer-resolved characteristics.
                    crate::rules::layers::calculate_characteristics(state, target_id)
                        .map(|c| {
                            c.card_types
                                .contains(&crate::state::types::CardType::Creature)
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if target_valid && power_snapshot > 0 {
                // CR 702.97a: Add power_snapshot +1/+1 counters to the target creature.
                if let Some(obj) = state.objects.get_mut(&target_id) {
                    let current = obj
                        .counters
                        .get(&CounterType::PlusOnePlusOne)
                        .copied()
                        .unwrap_or(0);
                    obj.counters = obj
                        .counters
                        .update(CounterType::PlusOnePlusOne, current + power_snapshot);
                }
                events.push(GameEvent::CounterAdded {
                    object_id: target_id,
                    counter: CounterType::PlusOnePlusOne,
                    count: power_snapshot,
                });
            }
            // If target is invalid or power_snapshot == 0, ability fizzles / adds 0
            // counters -- either way emit AbilityResolved.

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.165a: Backup trigger resolves.
        // 1. Put N +1/+1 counters on target creature.
        // 2. If target is another creature (not the source), grant keyword abilities
        //    until EOT via Layer 6 continuous effect (CR 702.165a, 702.165d).
        // CR 702.72a: Champion ETB trigger resolves -- "sacrifice it unless you exile
        // another [object] you control." Auto-selects first qualifying permanent to exile.
        // If no qualifying permanent exists, sacrifice the champion instead.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Champion,
            data:
                crate::state::stack::TriggerData::ETBChampion {
                    filter: champion_filter,
                },
        } => {
            let controller = stack_obj.controller;

            // CR 400.7: Check if the champion is still on the battlefield.
            let source_on_bf = state
                .objects
                .get(&source_object)
                .is_some_and(|obj| obj.zone == ZoneId::Battlefield);

            if source_on_bf {
                // Find first qualifying permanent controlled by the champion's controller
                // that is not the champion itself.
                let target_opt: Option<crate::state::game_object::ObjectId> = {
                    let candidates: Vec<_> = state
                        .objects
                        .iter()
                        .filter_map(|(&id, obj)| {
                            if id == source_object {
                                return None;
                            }
                            if obj.zone != ZoneId::Battlefield {
                                return None;
                            }
                            if obj.controller != controller {
                                return None;
                            }
                            if !obj.is_phased_in() {
                                return None;
                            }
                            // Check the champion filter using layer-resolved characteristics.
                            let matches = if let Some(chars) =
                                crate::rules::layers::calculate_characteristics(state, id)
                            {
                                match &champion_filter {
                                    ChampionFilter::AnyCreature => {
                                        chars.card_types.contains(&CardType::Creature)
                                    }
                                    ChampionFilter::Subtype(st) => chars.subtypes.contains(st),
                                }
                            } else {
                                // Fall back to base characteristics.
                                let obj = state.objects.get(&id)?;
                                match &champion_filter {
                                    ChampionFilter::AnyCreature => {
                                        obj.characteristics.card_types.contains(&CardType::Creature)
                                    }
                                    ChampionFilter::Subtype(st) => {
                                        obj.characteristics.subtypes.contains(st)
                                    }
                                }
                            };
                            if matches {
                                Some(id)
                            } else {
                                None
                            }
                        })
                        .collect();
                    candidates.into_iter().next()
                };

                if let Some(target_id) = target_opt {
                    // Exile the qualifying permanent.
                    let target_owner = state
                        .objects
                        .get(&target_id)
                        .map(|o| o.owner)
                        .unwrap_or(controller);
                    if let Ok((new_exile_id, _old)) =
                        state.move_object_to_zone(target_id, ZoneId::Exile)
                    {
                        // CR 702.72a / CR 607.2a: Record the exiled card ID on the champion.
                        if let Some(champion_obj) = state.objects.get_mut(&source_object) {
                            champion_obj.champion_exiled_card = Some(new_exile_id);
                        }
                        events.push(GameEvent::ObjectExiled {
                            player: controller,
                            object_id: target_id,
                            new_exile_id,
                        });
                        let _ = target_owner; // suppress unused warning
                    }
                } else {
                    // No qualifying target: sacrifice the champion (CR 702.72a).
                    let source_info = state.objects.get(&source_object).and_then(|obj| {
                        if obj.zone == ZoneId::Battlefield {
                            Some((obj.owner, obj.controller, obj.counters.clone()))
                        } else {
                            None
                        }
                    });
                    if let Some((owner, pre_sacrifice_controller, pre_death_counters)) = source_info
                    {
                        // CR 701.17a: Sacrifice is NOT destruction — no indestructible check.
                        let action = crate::rules::replacement::check_zone_change_replacement(
                            state,
                            source_object,
                            crate::state::zone::ZoneType::Battlefield,
                            crate::state::zone::ZoneType::Graveyard,
                            owner,
                            &std::collections::HashSet::new(),
                        );
                        match action {
                            crate::rules::replacement::ZoneChangeAction::Redirect {
                                to: dest,
                                events: repl_events,
                                ..
                            } => {
                                events.extend(repl_events);
                                if let Ok((new_id, _old)) =
                                    state.move_object_to_zone(source_object, dest)
                                {
                                    match dest {
                                        ZoneId::Exile => {
                                            events.push(GameEvent::ObjectExiled {
                                                player: owner,
                                                object_id: source_object,
                                                new_exile_id: new_id,
                                            });
                                        }
                                        _ => {
                                            events.push(GameEvent::CreatureDied {
                                                object_id: source_object,
                                                new_grave_id: new_id,
                                                controller: pre_sacrifice_controller,
                                                pre_death_counters,
                                            });
                                        }
                                    }
                                }
                            }
                            crate::rules::replacement::ZoneChangeAction::Proceed => {
                                if let Ok((new_grave_id, _old)) = state
                                    .move_object_to_zone(source_object, ZoneId::Graveyard(owner))
                                {
                                    events.push(GameEvent::CreatureDied {
                                        object_id: source_object,
                                        new_grave_id,
                                        controller: pre_sacrifice_controller,
                                        pre_death_counters,
                                    });
                                }
                            }
                            crate::rules::replacement::ZoneChangeAction::ChoiceRequired {
                                player,
                                choices,
                                event_description,
                            } => {
                                use crate::state::replacement_effect::PendingZoneChange;
                                state.pending_zone_changes.push_back(PendingZoneChange {
                                    object_id: source_object,
                                    original_from: crate::state::zone::ZoneType::Battlefield,
                                    original_destination: crate::state::zone::ZoneType::Graveyard,
                                    affected_player: player,
                                    already_applied: Vec::new(),
                                });
                                events.push(GameEvent::ReplacementChoiceRequired {
                                    player,
                                    event_description,
                                    choices,
                                });
                            }
                        }
                    }
                }
            }
            // If champion is not on the battlefield, the trigger does nothing (CR 400.7).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.72a: Champion LTB trigger resolves -- "return the exiled card to the
        // battlefield under its owner's control." Checks if the exiled card is still in exile.
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Champion,
            data: crate::state::stack::TriggerData::LTBChampion { exiled_card },
        } => {
            let controller = stack_obj.controller;

            // CR 607.2a: Check if the exiled card is still in exile.
            let exiled_info = state.objects.get(&exiled_card).and_then(|obj| {
                if obj.zone == ZoneId::Exile {
                    Some((obj.owner, obj.card_id.clone()))
                } else {
                    None
                }
            });

            if let Some((owner, card_id)) = exiled_info {
                // CR 702.72a: Return the card to the battlefield under its OWNER's control.
                if let Ok((new_id, _old)) =
                    state.move_object_to_zone(exiled_card, ZoneId::Battlefield)
                {
                    // Set controller to owner (CR 702.72a: "under its owner's control").
                    if let Some(obj) = state.objects.get_mut(&new_id) {
                        obj.controller = owner;
                    }

                    // Run the full ETB pipeline.
                    let registry = state.card_registry.clone();
                    let self_evts = super::replacement::apply_self_etb_from_definition(
                        state,
                        new_id,
                        owner,
                        card_id.as_ref(),
                        &registry,
                    );
                    events.extend(self_evts);
                    let etb_evts = super::replacement::apply_etb_replacements(state, new_id, owner);
                    events.extend(etb_evts);

                    super::replacement::register_permanent_replacement_abilities(
                        state,
                        new_id,
                        owner,
                        card_id.as_ref(),
                        &registry,
                    );
                    super::replacement::register_static_continuous_effects(
                        state,
                        new_id,
                        card_id.as_ref(),
                        &registry,
                    );

                    events.push(GameEvent::PermanentEnteredBattlefield {
                        player: owner,
                        object_id: new_id,
                    });

                    let etb_trigger_evts = super::replacement::queue_carddef_etb_triggers(
                        state,
                        new_id,
                        owner,
                        card_id.as_ref(),
                        &registry,
                    );
                    events.extend(etb_trigger_evts);
                }
            }
            // If exiled card is not in exile (already moved), do nothing (CR 607.2a).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.95a/702.95c: Soulbond ETB trigger resolution.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Soulbond,
            data: crate::state::stack::TriggerData::ETBSoulbond { pair_target },
        } => {
            use crate::state::continuous_effect::{
                ContinuousEffect, EffectDuration, EffectFilter, EffectId,
            };
            let controller = stack_obj.controller;

            // CR 702.95c: Both source and target must still be on the battlefield as creatures
            // controlled by the same player, and both must be unpaired.
            // Use calculate_characteristics (layer-resolved) for the creature check, consistent
            // with check_soulbond_unpairing in sba.rs (CR 702.95c: "no longer a creature").
            let source_ok = state
                .objects
                .get(&source_object)
                .map(|o| {
                    o.zone == ZoneId::Battlefield
                        && o.is_phased_in()
                        && o.controller == controller
                        && o.paired_with.is_none()
                })
                .unwrap_or(false)
                && crate::rules::layers::calculate_characteristics(state, source_object)
                    .map(|c| c.card_types.contains(&CardType::Creature))
                    .unwrap_or(false);
            let target_ok = pair_target != source_object
                && state
                    .objects
                    .get(&pair_target)
                    .map(|o| {
                        o.zone == ZoneId::Battlefield
                            && o.is_phased_in()
                            && o.controller == controller
                            && o.paired_with.is_none()
                    })
                    .unwrap_or(false)
                && crate::rules::layers::calculate_characteristics(state, pair_target)
                    .map(|c| c.card_types.contains(&CardType::Creature))
                    .unwrap_or(false);

            if source_ok && target_ok {
                // CR 702.95b: Set paired_with symmetrically on both creatures.
                if let Some(src) = state.objects.get_mut(&source_object) {
                    src.paired_with = Some(pair_target);
                }
                if let Some(tgt) = state.objects.get_mut(&pair_target) {
                    tgt.paired_with = Some(source_object);
                }

                // Register WhilePaired CEs for any soulbond grants from the card definition.
                // Look up the soulbond creature's card definition for grants.
                let registry = state.card_registry.clone();
                let card_id = state
                    .objects
                    .get(&source_object)
                    .and_then(|o| o.card_id.clone());
                if let Some(cid) = card_id {
                    if let Some(def) = registry.get(cid) {
                        for ability in &def.abilities {
                            if let crate::cards::card_definition::AbilityDefinition::Soulbond {
                                grants,
                            } = ability
                            {
                                for grant in grants {
                                    // Grant applies to the soulbond creature itself.
                                    let ts = state.timestamp_counter;
                                    state.timestamp_counter += 1;
                                    let effect_id = EffectId(state.next_object_id().0);
                                    state.continuous_effects.push_back(ContinuousEffect {
                                        id: effect_id,
                                        source: Some(source_object),
                                        layer: grant.layer,
                                        filter: EffectFilter::SingleObject(source_object),
                                        modification: grant.modification.clone(),
                                        duration: EffectDuration::WhilePaired(
                                            source_object,
                                            pair_target,
                                        ),
                                        timestamp: ts,
                                        is_cda: false,
                                    });
                                    // Grant applies to the paired partner too.
                                    let ts2 = state.timestamp_counter;
                                    state.timestamp_counter += 1;
                                    let effect_id2 = EffectId(state.next_object_id().0);
                                    state.continuous_effects.push_back(ContinuousEffect {
                                        id: effect_id2,
                                        source: Some(source_object),
                                        layer: grant.layer,
                                        filter: EffectFilter::SingleObject(pair_target),
                                        modification: grant.modification.clone(),
                                        duration: EffectDuration::WhilePaired(
                                            source_object,
                                            pair_target,
                                        ),
                                        timestamp: ts2,
                                        is_cda: false,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            // CR 702.95c: If either check fails, neither creature becomes paired (fizzle).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Ravenous,
            data:
                crate::state::stack::TriggerData::ETBRavenousDraw {
                    permanent: _,
                    x_value,
                },
        } => {
            let controller = stack_obj.controller;

            // CR 603.4: Intervening-if re-check. x_value is fixed at cast time and
            // immutable, so this always passes if the trigger fired. But we re-check
            // for correctness (e.g., if a future replacement effect could change it).
            // CR 702.156a: the only intervening-if condition is "if X is 5 or more."
            // The draw trigger has no targets and is not conditioned on the Ravenous
            // permanent remaining on the battlefield. Triggered abilities only fizzle
            // for lack of legal targets (CR 608.2b); this ability has none. If the
            // creature is removed in response to this trigger, the draw still happens.
            if x_value >= 5 {
                // CR 702.156a: "draw a card" — controller of the Ravenous permanent.
                // draw_card returns Ok(events) on success; if the library is empty,
                // it returns Ok(vec![GameEvent::PlayerLost { ... }]).
                // MR-B12-09: The Err case is intentionally discarded here. Attempting
                // to draw from an empty library is NOT an immediate game loss inline —
                // it is handled as a State-Based Action (CR 121.3, CR 704.5b): the
                // next SBA check after this trigger resolves will detect the empty
                // library and emit the PlayerLost event. Dropping the Err is correct.
                if let Ok(drawn_events) = crate::rules::turn_actions::draw_card(state, controller) {
                    events.extend(drawn_events);
                }
            }

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.157a: Squad ETB trigger resolution.
        //
        // "Create a token that's a copy of it for each time its squad cost was paid."
        //
        // CR 608.2b / CR 400.7: If the source creature has left the battlefield before
        // this trigger resolves, skip token creation entirely (LKI not yet available).
        //
        // CR 707.2: Token copies use copiable values of the source at resolution time.
        // This is the same pattern as Myriad (tokens copy the original, not the permanent's
        // current layer-modified state -- copiable values are what matter per CR 707.2).
        //
        // Tokens are NOT cast (ruling 2022-10-07) and do NOT have summoning sickness
        // prevented (they enter normally with summoning sickness, unlike Myriad which
        // enters tapped and attacking).
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Squad,
            data: crate::state::stack::TriggerData::ETBSquad { count: squad_count },
        } => {
            let controller = stack_obj.controller;

            // CR 603.4: Intervening-if re-check. squad_count is fixed at cast time and
            // immutable, so this always passes if the trigger fired. Verified for correctness.
            if squad_count > 0 {
                for _ in 0..squad_count {
                    // CR 608.2b / CR 400.7: If source left the battlefield, skip.
                    if state
                        .objects
                        .get(&source_object)
                        .is_none_or(|o| o.zone != ZoneId::Battlefield)
                    {
                        break;
                    }

                    // Build a blank token that will become a copy of the source.
                    // CR 111.10: Tokens enter the battlefield as the stated kind of object.
                    // CR 707.2: Copy uses copiable values of the source creature.
                    let token_obj = crate::state::game_object::GameObject {
                        id: crate::state::game_object::ObjectId(0), // replaced by add_object
                        card_id: None,
                        characteristics: state
                            .objects
                            .get(&source_object)
                            .map(|o| o.characteristics.clone())
                            .unwrap_or_default(),
                        controller,
                        owner: controller,
                        zone: ZoneId::Battlefield,
                        status: crate::state::game_object::ObjectStatus {
                            // CR 302.6: Tokens have summoning sickness (enter normally).
                            ..crate::state::game_object::ObjectStatus::default()
                        },
                        counters: im::OrdMap::new(),
                        attachments: im::Vector::new(),
                        attached_to: None,
                        damage_marked: 0,
                        deathtouch_damage: false,
                        is_token: true,
                        timestamp: 0, // replaced by add_object
                        has_summoning_sickness: true,
                        goaded_by: im::Vector::new(),
                        kicker_times_paid: 0,
                        cast_alt_cost: None,
                        foretold_turn: 0,
                        was_unearthed: false,
                        myriad_exile_at_eoc: false,
                        decayed_sacrifice_at_eoc: false,
                        ring_block_sacrifice_at_eoc: false,
                        exiled_by_hideaway: None,
                        encore_sacrifice_at_end_step: false,
                        encore_must_attack: None,
                        encore_activated_by: None,
                        is_plotted: false,
                        plotted_turn: 0,
                        is_prototyped: false,
                        was_bargained: false,
                        evidence_collected: false,
                        phased_out_indirectly: false,
                        phased_out_controller: None,
                        creatures_devoured: 0,
                        paired_with: None,
                        tribute_was_paid: false,
                        // CR 107.3m: Squad tokens are never cast, so x_value is always 0.
                        x_value: 0,
                        // CR 702.157a: Tokens created by Squad have squad_count: 0
                        // (not cast, so cannot trigger their own squad ETB trigger).
                        squad_count: 0,
                        offspring_paid: false,
                        // CR 702.174a: tokens/copies are never gift casts.
                        gift_was_given: false,
                        champion_exiled_card: None,
                        gift_opponent: None,
            // CR 702.171b: tokens are not saddled by default.
                        encoded_cards: im::Vector::new(),
                        haunting_target: None,
                        // CR 702.151b: tokens are not reconfigured by default.
                        // CR 729.2: tokens are not part of a merged permanent by default.
                        merged_components: im::Vector::new(),
                        // CR 712.8a: DFC state is reset for all new permanents.
                        is_transformed: false,
                        last_transform_timestamp: 0,
                        was_cast_disturbed: false,
                        craft_exiled_cards: im::Vector::new(),
                        chosen_creature_type: None,
                        face_down_as: None,
                        loyalty_ability_activated_this_turn: false,
                        class_level: 0,
                        designations: Designations::default(),
                        meld_component: None,
                    };

                    // Add the token to the battlefield.
                    let token_id = match state.add_object(token_obj, ZoneId::Battlefield) {
                        Ok(id) => id,
                        Err(_) => continue,
                    };

                    // CR 707.2: Apply a Layer 1 CopyOf continuous effect so the token
                    // has the copiable characteristics of the source creature.
                    let copy_effect = crate::rules::copy::create_copy_effect(
                        state,
                        token_id,
                        source_object,
                        controller,
                    );
                    state.continuous_effects.push_back(copy_effect);

                    events.push(GameEvent::TokenCreated {
                        player: controller,
                        object_id: token_id,
                    });
                    events.push(GameEvent::PermanentEnteredBattlefield {
                        player: controller,
                        object_id: token_id,
                    });
                }
            }

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.175a: Offspring triggered ability resolution.
        //
        // "When this permanent enters, if its offspring cost was paid, create a token
        // that's a copy of it, except it's 1/1."
        //
        // KEY DIFFERENCE FROM SQUAD: Ruling 2024-07-26 states "If the spell resolves but
        // the creature with offspring leaves the battlefield before the offspring ability
        // resolves, you'll still create a token copy of it." This means we NEVER skip when
        // the source is gone -- instead, use last-known information (LKI) from the source
        // GameObject's characteristics (before zone change) or fall back to the card registry.
        //
        // CR 707.9d: "except it's 1/1" -- the copy instruction modifies base P/T to 1/1.
        // Implemented as: CopyOf effect (Layer 1) + SetPowerToughness {1, 1} (Layer 7b).
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Offspring,
            data: crate::state::stack::TriggerData::ETBOffspring { source_card_id },
        } => {
            use crate::state::continuous_effect::{
                ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer,
                LayerModification,
            };
            let controller = stack_obj.controller;

            // CR 702.175a: Intervening-if re-check.
            // Offspring is binary (paid or not), so the only re-check needed is whether
            // the permanent (if still on the battlefield) still has KeywordAbility::Offspring.
            // Ruling 2024-07-26: if source left the battlefield, skip the intervening-if
            // re-check for the keyword (we still create the token per the ruling).
            let source_on_battlefield = state
                .objects
                .get(&source_object)
                .is_some_and(|o| o.zone == ZoneId::Battlefield);
            if source_on_battlefield {
                let has_offspring = {
                    let chars =
                        crate::rules::layers::calculate_characteristics(state, source_object);
                    chars
                        .map(|c| c.keywords.contains(&KeywordAbility::Offspring))
                        .unwrap_or(false)
                };
                if !has_offspring {
                    // CR 603.4: Intervening-if failed; source no longer has Offspring.
                    events.push(GameEvent::AbilityResolved {
                        controller,
                        stack_object_id: stack_obj.id,
                    });
                    return Ok(events);
                }
            }

            // CR 707.2 / 707.9d: Build the token using copiable values of the source.
            // If the source is still on the battlefield, clone its characteristics.
            // If it left, clone from the card registry (LKI per ruling 2024-07-26).
            // source_card_id was captured at trigger-queue time for this exact purpose.
            let source_characteristics = state
                .objects
                .get(&source_object)
                .filter(|o| o.zone == ZoneId::Battlefield)
                .map(|o| o.characteristics.clone())
                .or_else(|| {
                    // LKI fallback: use card registry for base characteristics.
                    // source_card_id was captured when the trigger was queued (while the
                    // source was still on the battlefield), so it's available even after
                    // the source has left (ruling 2024-07-26).
                    source_card_id
                        .clone()
                        .and_then(|cid| state.card_registry.get(cid))
                        .map(|def| {
                            // Build minimal characteristics from the card definition.
                            // Keywords from def.abilities (keyword entries only)
                            let mut keywords =
                                im::OrdSet::<crate::state::types::KeywordAbility>::new();
                            for ability in &def.abilities {
                                if let crate::cards::card_definition::AbilityDefinition::Keyword(
                                    kw,
                                ) = ability
                                {
                                    keywords.insert(kw.clone());
                                }
                            }
                            crate::state::game_object::Characteristics {
                                name: def.name.clone(),
                                mana_cost: def.mana_cost.clone(),
                                card_types: def.types.card_types.clone(),
                                subtypes: def.types.subtypes.clone(),
                                supertypes: def.types.supertypes.clone(),
                                power: def.power,
                                toughness: def.toughness,
                                keywords,
                                ..crate::state::game_object::Characteristics::default()
                            }
                        })
                })
                .unwrap_or_default();

            let token_obj = crate::state::game_object::GameObject {
                id: crate::state::game_object::ObjectId(0), // replaced by add_object
                card_id: None,
                characteristics: source_characteristics,
                controller,
                owner: controller,
                zone: ZoneId::Battlefield,
                status: crate::state::game_object::ObjectStatus {
                    // CR 302.6: Tokens have summoning sickness (enter normally).
                    ..crate::state::game_object::ObjectStatus::default()
                },
                counters: im::OrdMap::new(),
                attachments: im::Vector::new(),
                attached_to: None,
                damage_marked: 0,
                deathtouch_damage: false,
                is_token: true,
                timestamp: 0, // replaced by add_object
                has_summoning_sickness: true,
                goaded_by: im::Vector::new(),
                kicker_times_paid: 0,
                cast_alt_cost: None,
                foretold_turn: 0,
                was_unearthed: false,
                myriad_exile_at_eoc: false,
                decayed_sacrifice_at_eoc: false,
                ring_block_sacrifice_at_eoc: false,
                exiled_by_hideaway: None,
                encore_sacrifice_at_end_step: false,
                encore_must_attack: None,
                encore_activated_by: None,
                is_plotted: false,
                plotted_turn: 0,
                is_prototyped: false,
                was_bargained: false,
                evidence_collected: false,
                phased_out_indirectly: false,
                phased_out_controller: None,
                creatures_devoured: 0,
                paired_with: None,
                tribute_was_paid: false,
                // CR 107.3m: Offspring tokens are never cast, so x_value is always 0.
                x_value: 0,
                // CR 702.157a: Offspring tokens are never cast, so squad_count is always 0.
                squad_count: 0,
                // CR 702.175a: Offspring tokens are never cast, so offspring_paid is always false.
                offspring_paid: false,
                // CR 702.174a: tokens/copies are never gift casts.
                gift_was_given: false,
                champion_exiled_card: None,
                gift_opponent: None,
            // CR 702.171b: tokens are not saddled by default.
                encoded_cards: im::Vector::new(),
                haunting_target: None,
                // CR 702.151b: tokens are not reconfigured by default.
                // CR 729.2: tokens are not part of a merged permanent by default.
                merged_components: im::Vector::new(),
                // CR 712.8a: DFC state is reset for all new permanents.
                is_transformed: false,
                last_transform_timestamp: 0,
                was_cast_disturbed: false,
                craft_exiled_cards: im::Vector::new(),
                chosen_creature_type: None,
                face_down_as: None,
                loyalty_ability_activated_this_turn: false,
                class_level: 0,
                designations: Designations::default(),
                meld_component: None,
            };

            // Add the token to the battlefield.
            let token_id = match state.add_object(token_obj, ZoneId::Battlefield) {
                Ok(id) => id,
                Err(_) => {
                    events.push(GameEvent::AbilityResolved {
                        controller,
                        stack_object_id: stack_obj.id,
                    });
                    return Ok(events);
                }
            };

            // CR 707.2: Apply a Layer 1 CopyOf continuous effect so the token
            // has the copiable characteristics of the source creature.
            // NOTE: source_object may have left the battlefield; CopyOf still records
            // the source_id for layer resolution. If source is gone, Layer 1 applies
            // the last-known characteristics we already copied into token_obj.characteristics.
            if source_on_battlefield {
                let copy_effect = crate::rules::copy::create_copy_effect(
                    state,
                    token_id,
                    source_object,
                    controller,
                );
                state.continuous_effects.push_back(copy_effect);
            }

            // CR 702.175a: "except it's 1/1" -- apply a Layer 7b effect that sets base P/T to 1/1.
            // This effect is indefinite (not until-end-of-turn) and applies on top of the CopyOf.
            //
            // TODO: CR 707.9b deviation -- the "except it's 1/1" clause is a copy-with-exception,
            // meaning the modified P/T (1/1) should become part of the token's *copiable values*
            // (Layer 1), not a separate Layer 7b effect. The current Layer 7b approach is incorrect
            // when another copy effect subsequently targets this token: get_copiable_values() only
            // resolves Layer 1 (CopyOf) effects, not Layer 7b. So a Clone copying the Offspring
            // token would inherit the *source creature's* original P/T instead of 1/1.
            //
            // CR 707.9d also states CDAs defining the overridden characteristic (P/T) should not
            // be copied. The Layer 7b approach happens to mask any P/T CDA since 7b overrides 7a,
            // but the CDA is still present in the token's copiable values -- wrong per 707.9d.
            //
            // A proper fix requires a new LayerModification variant (e.g., SetCopiablePT) applied
            // at Layer 1 with a later timestamp than the CopyOf effect, and modifications to
            // apply_layer_modification and get_copiable_values in copy.rs. This is a separate
            // infrastructure task; the behavior is correct in the common case (no subsequent copy).
            let ts = state.timestamp_counter;
            state.timestamp_counter += 1;
            let effect_id_val = state.timestamp_counter;
            state.timestamp_counter += 1;
            let pt_override = ContinuousEffect {
                id: EffectId(effect_id_val),
                source: Some(token_id),
                timestamp: ts,
                layer: EffectLayer::PtSet,
                duration: EffectDuration::Indefinite,
                filter: EffectFilter::SingleObject(token_id),
                modification: LayerModification::SetPowerToughness {
                    power: 1,
                    toughness: 1,
                },
                is_cda: false,
            };
            state.continuous_effects.push_back(pt_override);

            events.push(GameEvent::TokenCreated {
                player: controller,
                object_id: token_id,
            });
            events.push(GameEvent::PermanentEnteredBattlefield {
                player: controller,
                object_id: token_id,
            });

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.174b: Gift ETB trigger resolution.
        //
        // "When this permanent enters, if its gift cost was paid, [gift effect]."
        //
        // CR 603.4: Intervening-if re-check — the permanent must still have
        // KeywordAbility::Gift in layer-resolved characteristics.
        //
        // The gift effect is determined by the `AbilityDefinition::Gift { gift_type }` on the
        // card definition. The gift_opponent receives the gift (token, draw, or extra turn).
        //
        // CR 702.174d-i: Gift types and their effects:
        //   Food     (702.174d): chosen player creates a Food token
        //   Card     (702.174e): chosen player draws a card
        //   Treasure (702.174h): chosen player creates a Treasure token
        //   TappedFish/Octopus/ExtraTurn: other effects (partially deferred)
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Gift,
            data:
                crate::state::stack::TriggerData::ETBGift {
                    source_card_id,
                    gift_opponent,
                },
        } => {
            let controller = stack_obj.controller;

            // CR 603.4: Intervening-if re-check — source must still have Gift keyword.
            let source_on_battlefield = state
                .objects
                .get(&source_object)
                .is_some_and(|o| o.zone == ZoneId::Battlefield);
            if source_on_battlefield {
                let has_gift = {
                    let chars =
                        crate::rules::layers::calculate_characteristics(state, source_object);
                    chars
                        .map(|c| c.keywords.contains(&KeywordAbility::Gift))
                        .unwrap_or(false)
                };
                if !has_gift {
                    // CR 603.4: Intervening-if failed; source no longer has Gift.
                    events.push(GameEvent::AbilityResolved {
                        controller,
                        stack_object_id: stack_obj.id,
                    });
                    return Ok(events);
                }
            }

            // Look up the gift type from the card definition.
            // Use source_card_id for LKI fallback if source has left the battlefield.
            let card_id_for_lookup = state
                .objects
                .get(&source_object)
                .filter(|o| o.zone == ZoneId::Battlefield)
                .and_then(|o| o.card_id.clone())
                .or_else(|| source_card_id.clone());

            let gift_type = card_id_for_lookup
                .and_then(|cid| state.card_registry.get(cid))
                .and_then(|def| {
                    def.abilities.iter().find_map(|a| {
                        if let crate::cards::card_definition::AbilityDefinition::Gift {
                            gift_type,
                        } = a
                        {
                            Some(gift_type.clone())
                        } else {
                            None
                        }
                    })
                });

            if let Some(gift_type) = gift_type {
                let gift_events = execute_gift_effect(state, gift_opponent, controller, &gift_type);
                events.extend(gift_events);
            }

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.171a: Saddle ability resolution.
        //
        // The Mount becomes saddled until end of turn (CR 702.171b). If the Mount
        // has left the battlefield between activation and resolution, the ability
        // fizzles-like: no effect (CR 608.2b analog for non-targeted abilities).
        StackObjectKind::SaddleAbility { source_object } => {
            let controller = stack_obj.controller;

            // CR 702.171b: "stays saddled until the end of the turn or it leaves
            // the battlefield." Only set if Mount is still on the battlefield.
            if let Some(obj) = state.objects.get_mut(&source_object) {
                if obj.zone == ZoneId::Battlefield {
                    obj.designations.insert(Designations::SADDLED);
                }
            }

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.99a: Cipher trigger resolution.
        //
        // "Whenever [encoded creature] deals combat damage to a player, you may copy
        // the encoded card and you may cast the copy without paying its mana cost."
        //
        // Ruling 2013-04-15: The copy is cast (so "whenever you cast" triggers fire).
        // Ruling 2013-04-15: Cast during trigger resolution ignoring timing restrictions.
        //
        // MVP: Auto-cast the copy (deterministic). Interactive choice deferred.
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Cipher,
            data:
                crate::state::stack::TriggerData::CipherDamage {
                    source_creature: _,
                    encoded_card_id: _,
                    encoded_object_id,
                },
        } => {
            let controller = stack_obj.controller;

            // CR 702.99c: Verify the encoded card still exists in exile.
            // If not, the trigger fizzles (no copy created).
            let still_in_exile = state
                .objects
                .get(&encoded_object_id)
                .map(|obj| matches!(obj.zone, ZoneId::Exile))
                .unwrap_or(false);

            if still_in_exile {
                // Create a copy of the spell from the encoded card definition.
                // The copy is placed on the stack as a new StackObject.
                // Ruling 2013-04-15: Copies created by cipher ARE cast (unlike Storm copies).
                // The copy is_copy: true so no physical card moves when it resolves --
                // the original stays encoded in exile.
                //
                // MVP: Cast the copy without selecting targets (no target selection
                // for targeted copies -- deferred). Non-targeted cipher spells work correctly.
                let copy_stack_id = state.next_object_id();
                // MR-TC-25: use trigger_default; override is_copy = true (encoded card stays in exile).
                let mut copy_stack_obj = crate::state::stack::StackObject::trigger_default(
                    copy_stack_id,
                    controller,
                    StackObjectKind::Spell {
                        source_object: encoded_object_id,
                    },
                );
                // is_copy: true -- the encoded card stays in exile; this copy has no
                // physical card to move when it resolves (CR 702.99a / ruling 2013-04-15).
                copy_stack_obj.is_copy = true;

                state.stack_objects.push_back(copy_stack_obj);

                // Ruling 2013-04-15: cipher casts trigger "whenever you cast" abilities.
                // Increment spells_cast_this_turn for the controller.
                if let Some(ps) = state.players.get_mut(&controller) {
                    ps.spells_cast_this_turn = ps.spells_cast_this_turn.saturating_add(1);
                }

                // CR 116.3b: Casting a spell resets priority (all players must pass again).
                state.turn.players_passed = im::OrdSet::new();
                let active = state.turn.active_player;
                state.turn.priority_holder = Some(active);

                events.push(GameEvent::SpellCast {
                    player: controller,
                    stack_object_id: copy_stack_id,
                    source_object_id: encoded_object_id,
                });
            }

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.55a: HauntExileTrigger resolution.
        //
        // "When this creature dies / this spell is put into a graveyard during its
        // resolution, exile it haunting target creature."
        //
        // At resolution:
        // 1. Verify the haunt card is still in the graveyard (fizzle if not — e.g., if
        //    a commander was moved to the command zone, or a token ceased to exist).
        // 2. Find a legal creature target on the battlefield (MVP: auto-select first).
        //    If no legal creature exists, fizzle and leave the card in the graveyard.
        // 3. Move the haunt card from graveyard to exile (new ObjectId, CR 400.7).
        // 4. Set haunting_target on the new exiled object to the target creature's ObjectId.
        // 5. Emit HauntExiled event.
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Haunt,
            data:
                crate::state::stack::TriggerData::DeathHauntExile {
                    haunt_card,
                    haunt_card_id: _,
                },
        } => {
            let controller = stack_obj.controller;

            // CR 702.55a: Verify the haunt card is still in the graveyard.
            let card_in_graveyard = state
                .objects
                .get(&haunt_card)
                .map(|obj| matches!(obj.zone, crate::state::zone::ZoneId::Graveyard(_)))
                .unwrap_or(false);

            if card_in_graveyard {
                // Find the first legal creature on the battlefield for MVP auto-targeting.
                // CR 702.55a: "exile it haunting target creature" — any creature is legal.
                // Interactive target selection is deferred.
                let target_creature_id = state
                    .objects
                    .iter()
                    .filter(|(_, obj)| {
                        obj.zone == crate::state::zone::ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj
                                .characteristics
                                .card_types
                                .contains(&crate::state::types::CardType::Creature)
                    })
                    .map(|(&id, _)| id)
                    .next();

                if let Some(target_creature) = target_creature_id {
                    // Move the haunt card from graveyard to exile.
                    // CR 400.7: creates a new ObjectId for the exiled card.
                    let (new_exile_id, _) =
                        state.move_object_to_zone(haunt_card, crate::state::zone::ZoneId::Exile)?;

                    // CR 702.55b: Set haunting_target on the newly exiled card.
                    // This links the exiled card to the target creature's current ObjectId.
                    if let Some(exiled_obj) = state.objects.get_mut(&new_exile_id) {
                        exiled_obj.haunting_target = Some(target_creature);
                    }

                    events.push(GameEvent::HauntExiled {
                        controller,
                        exiled_card: new_exile_id,
                        haunted_creature: target_creature,
                    });
                }
                // Else: no legal creature target exists — fizzle (card stays in graveyard).
            }
            // Else: haunt card is no longer in graveyard — fizzle.

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.55c: HauntedCreatureDiesTrigger resolution.
        //
        // "When the creature [this card] haunts dies, [effect]."
        //
        // At resolution:
        // 1. Verify the haunt card is still in exile with a haunting_target set.
        //    If not, fizzle (card was removed from exile, or haunting_target is gone).
        // 2. Look up the card's haunt effect from the card registry.
        // 3. Execute the effect on behalf of the haunt card's controller.
        // 4. The haunt card stays in exile (does NOT leave exile or lose haunting status).
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Haunt,
            data:
                crate::state::stack::TriggerData::DeathHauntedCreatureDies {
                    haunt_source,
                    haunt_card_id,
                },
        } => {
            let controller = stack_obj.controller;

            // CR 702.55c: Verify the haunt card is still in exile with haunting_target set.
            let still_in_exile = state
                .objects
                .get(&haunt_source)
                .map(|obj| {
                    matches!(obj.zone, crate::state::zone::ZoneId::Exile)
                        && obj.haunting_target.is_some()
                })
                .unwrap_or(false);

            if still_in_exile {
                // Look up the card definition to find the haunt effect.
                // The effect is the "When the creature it haunts dies" triggered ability effect.
                // We look for AbilityDefinition::Triggered with trigger_condition == HauntedCreatureDies.
                use crate::cards::card_definition::{AbilityDefinition, TriggerCondition};
                let haunt_effect: Option<crate::cards::card_definition::Effect> = {
                    let card_id_opt = haunt_card_id.clone().or_else(|| {
                        state
                            .objects
                            .get(&haunt_source)
                            .and_then(|o| o.card_id.clone())
                    });

                    card_id_opt.and_then(|cid| {
                        state.card_registry.get(cid).and_then(|def| {
                            // Find the triggered ability with HauntedCreatureDies condition.
                            // If the card has no such ability, there's nothing to execute.
                            def.abilities.iter().find_map(|ab| {
                                if let AbilityDefinition::Triggered {
                                    trigger_condition,
                                    effect,
                                    ..
                                } = ab
                                {
                                    if *trigger_condition == TriggerCondition::HauntedCreatureDies {
                                        return Some(effect.clone());
                                    }
                                }
                                None
                            })
                        })
                    })
                };

                if let Some(effect) = haunt_effect {
                    let mut ctx =
                        crate::effects::EffectContext::new(controller, haunt_source, vec![]);
                    let effect_events = crate::effects::execute_effect(state, &effect, &mut ctx);
                    events.extend(effect_events);
                    // CR 702.55c: Haunt fires exactly once — clear the haunting relationship
                    // after the trigger resolves so that a recycled ObjectId cannot cause a
                    // spurious re-trigger against an unrelated creature's death.
                    if let Some(haunt_obj) = state.objects.get_mut(&haunt_source) {
                        haunt_obj.haunting_target = None;
                    }
                }
                // Else: no HauntedCreatureDies ability found — fizzle (card has no such trigger).
            }
            // Else: haunt card no longer in exile or haunting_target cleared — fizzle.

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 207.2c: Bloodrush activated ability resolution.
        //
        // At resolution, re-validate the target (CR 608.2b):
        // - Still on the battlefield as a creature.
        // - Still registered as an attacker in CombatState.
        //   "Target attacking creature" is the targeting restriction; if the
        //   creature is no longer attacking, the target is no longer legal and
        //   the ability fizzles.
        // If legal: register +power_boost/+toughness_boost (Layer 7c, UntilEndOfTurn)
        // and optionally grant a keyword (Layer 6, UntilEndOfTurn).
        StackObjectKind::BloodrushAbility {
            source_object,
            target_creature,
            power_boost,
            toughness_boost,
            grants_keyword,
        } => {
            use crate::state::continuous_effect::{
                ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer,
                LayerModification,
            };
            use crate::state::types::CardType;
            let controller = stack_obj.controller;

            // CR 608.2b: Check target legality at resolution.
            // The target must still be on the battlefield as a creature AND still attacking.
            // CR 613.1d: Use layer-resolved types for creature check (animated permanents).
            let is_on_battlefield = state
                .objects
                .get(&target_creature)
                .map(|o| {
                    matches!(o.zone, crate::state::zone::ZoneId::Battlefield)
                        && crate::rules::layers::calculate_characteristics(state, target_creature)
                            .unwrap_or_else(|| o.characteristics.clone())
                            .card_types
                            .contains(&CardType::Creature)
                })
                .unwrap_or(false);
            let is_attacking = state
                .combat
                .as_ref()
                .map(|c| c.attackers.contains_key(&target_creature))
                .unwrap_or(false);

            if is_on_battlefield && is_attacking {
                // Register +power_boost/+toughness_boost (Layer 7c, UntilEndOfTurn).
                // Use separate Power and Toughness modifications if they differ;
                // for bloodrush they are always equal so ModifyBoth is correct.
                let ts = state.timestamp_counter;
                state.timestamp_counter += 1;
                let eff_id = state.next_object_id().0;
                if power_boost == toughness_boost {
                    state.continuous_effects.push_back(ContinuousEffect {
                        id: EffectId(eff_id),
                        source: Some(source_object),
                        timestamp: ts,
                        layer: EffectLayer::PtModify,
                        duration: EffectDuration::UntilEndOfTurn,
                        filter: EffectFilter::SingleObject(target_creature),
                        modification: LayerModification::ModifyBoth(power_boost),
                        is_cda: false,
                    });
                } else {
                    // Asymmetric boost: register Power and Toughness separately.
                    state.continuous_effects.push_back(ContinuousEffect {
                        id: EffectId(eff_id),
                        source: Some(source_object),
                        timestamp: ts,
                        layer: EffectLayer::PtModify,
                        duration: EffectDuration::UntilEndOfTurn,
                        filter: EffectFilter::SingleObject(target_creature),
                        modification: LayerModification::ModifyPower(power_boost),
                        is_cda: false,
                    });
                    let ts2 = state.timestamp_counter;
                    state.timestamp_counter += 1;
                    let eff_id2 = state.next_object_id().0;
                    state.continuous_effects.push_back(ContinuousEffect {
                        id: EffectId(eff_id2),
                        source: Some(source_object),
                        timestamp: ts2,
                        layer: EffectLayer::PtModify,
                        duration: EffectDuration::UntilEndOfTurn,
                        filter: EffectFilter::SingleObject(target_creature),
                        modification: LayerModification::ModifyToughness(toughness_boost),
                        is_cda: false,
                    });
                }

                // If a keyword is granted, register Layer 6 effect (UntilEndOfTurn).
                if let Some(keyword) = grants_keyword {
                    let kw_set: im::OrdSet<crate::state::types::KeywordAbility> =
                        std::iter::once(keyword).collect();
                    let ts_kw = state.timestamp_counter;
                    state.timestamp_counter += 1;
                    let eff_id_kw = state.next_object_id().0;
                    state.continuous_effects.push_back(ContinuousEffect {
                        id: EffectId(eff_id_kw),
                        source: Some(source_object),
                        timestamp: ts_kw,
                        layer: EffectLayer::Ability,
                        duration: EffectDuration::UntilEndOfTurn,
                        filter: EffectFilter::SingleObject(target_creature),
                        modification: LayerModification::AddKeywords(kw_set),
                        is_cda: false,
                    });
                }
            }
            // If not legal (fizzled): card is already in graveyard (discarded as cost).
            // No pump or keyword grant applied.

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Backup(_),
            data:
                crate::state::stack::TriggerData::ETBBackup {
                    target: target_creature,
                    count: counter_count,
                    abilities: abilities_to_grant,
                },
        } => {
            let controller = stack_obj.controller;

            // Fizzle check: target must still be on the battlefield.
            let target_exists = state
                .objects
                .get(&target_creature)
                .map(|o| o.zone == ZoneId::Battlefield)
                .unwrap_or(false);

            if !target_exists {
                // Target is gone; trigger fizzles silently (CR 608.2b).
                // Emit AbilityResolved to complete resolution without effect.
                events.push(GameEvent::AbilityResolved {
                    controller,
                    stack_object_id: stack_obj.id,
                });
            } else {
                // 1. Place N +1/+1 counters on target.
                if let Some(obj) = state.objects.get_mut(&target_creature) {
                    let current = obj
                        .counters
                        .get(&CounterType::PlusOnePlusOne)
                        .copied()
                        .unwrap_or(0);
                    obj.counters = obj
                        .counters
                        .update(CounterType::PlusOnePlusOne, current + counter_count);
                }
                events.push(GameEvent::CounterAdded {
                    object_id: target_creature,
                    counter: CounterType::PlusOnePlusOne,
                    count: counter_count,
                });

                // 2. If target is another creature and there are abilities to grant,
                //    register Layer 6 UntilEndOfTurn continuous effect (CR 702.165a, 702.165d).
                if target_creature != source_object && !abilities_to_grant.is_empty() {
                    use crate::state::continuous_effect::{
                        ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer,
                        LayerModification,
                    };
                    let kw_set: im::OrdSet<KeywordAbility> =
                        abilities_to_grant.into_iter().collect();
                    let ts = state.timestamp_counter;
                    state.timestamp_counter += 1;
                    let id_inner = state.next_object_id().0;
                    let eff = ContinuousEffect {
                        id: EffectId(id_inner),
                        source: Some(source_object),
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeywords(kw_set),
                        filter: EffectFilter::SingleObject(target_creature),
                        duration: EffectDuration::UntilEndOfTurn,
                        is_cda: false,
                        timestamp: ts,
                    };
                    state.continuous_effects.push_back(eff);
                }

                events.push(GameEvent::AbilityResolved {
                    controller,
                    stack_object_id: stack_obj.id,
                });
            }
        }

        // CR 702.116a: Myriad trigger resolves -- create token copies of the source
        // creature for each opponent other than the defending player, each tapped and
        // attacking that opponent.
        //
        // CR 702.116a: "for each opponent other than defending player, you may create
        // a token that's a copy of this creature that's tapped and attacking that player."
        // V1 simplification: auto-accept "you may" (always create tokens).
        //
        // CR 702.116b: Multiple instances trigger separately — this arm handles one
        // trigger at a time.
        //
        // CR 707.2: Tokens that are copies of a permanent use Layer 1 (CopyOf) to
        // reflect copiable values of the source at resolution time.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Myriad,
            data: crate::state::stack::TriggerData::MyriadAttack { defending_player },
        } => {
            let controller = stack_obj.controller;

            // Find all active opponents of the source's controller excluding the defending player.
            // CR 702.116a: "for each opponent other than defending player."
            let opponents: Vec<crate::state::player::PlayerId> = state
                .players
                .values()
                .filter(|p| {
                    !p.has_lost && !p.has_conceded && p.id != controller && p.id != defending_player
                })
                .map(|p| p.id)
                .collect();

            // CR 702.116a: If no eligible opponents, no tokens are created.
            // (e.g. 2-player game where defending player is the only opponent).
            for opponent_id in opponents {
                // CR 608.2b / CR 400.7: If the source creature has left the battlefield
                // before this trigger resolves, skip token creation entirely.
                // "This creature" no longer exists; LKI infrastructure is not yet
                // available to reconstruct its characteristics, so no tokens are created.
                if state
                    .objects
                    .get(&source_object)
                    .is_none_or(|o| o.zone != ZoneId::Battlefield)
                {
                    break;
                }

                // Build a blank token object that will become a copy of the source.
                // CR 111.10: Tokens enter the battlefield as the stated kind of object.
                // CR 707.2: Copy uses copiable values of the source creature.
                let token_obj = crate::state::game_object::GameObject {
                    id: crate::state::game_object::ObjectId(0), // replaced by add_object
                    card_id: None,
                    characteristics: state
                        .objects
                        .get(&source_object)
                        .map(|o| o.characteristics.clone())
                        .unwrap_or_default(),
                    controller,
                    owner: controller,
                    zone: ZoneId::Battlefield,
                    status: crate::state::game_object::ObjectStatus {
                        // CR 702.116a: "tapped and attacking" — enters tapped.
                        tapped: true,
                        ..crate::state::game_object::ObjectStatus::default()
                    },
                    counters: im::OrdMap::new(),
                    attachments: im::Vector::new(),
                    attached_to: None,
                    damage_marked: 0,
                    deathtouch_damage: false,
                    is_token: true,
                    timestamp: 0, // replaced by add_object
                    // CR 302.6: Tokens have summoning sickness; they are already attacking
                    // so sickness does not prevent combat participation this turn.
                    has_summoning_sickness: true,
                    goaded_by: im::Vector::new(),
                    kicker_times_paid: 0,
                    cast_alt_cost: None,
                    foretold_turn: 0,
                    was_unearthed: false,
                    // CR 702.116a: "exile the tokens at end of combat"
                    // Tagged here so end_combat() in turn_actions.rs can find them.
                    myriad_exile_at_eoc: true,
                    decayed_sacrifice_at_eoc: false,
                    ring_block_sacrifice_at_eoc: false,
                    exiled_by_hideaway: None,
                    // CR 701.60b: tokens are not suspected by default.
                    encore_sacrifice_at_end_step: false,
                    encore_must_attack: None,
                    encore_activated_by: None,
                    is_plotted: false,
                    plotted_turn: 0,
                    is_prototyped: false,
                    was_bargained: false,
                    evidence_collected: false,
                    phased_out_indirectly: false,
                    phased_out_controller: None,
                    creatures_devoured: 0,
                    paired_with: None,
                    tribute_was_paid: false,
                    // CR 107.3m: Tokens/copies are never cast, so x_value is always 0.
                    x_value: 0,
                    // CR 702.157a: Tokens/copies are never cast, so squad_count is always 0.
                    squad_count: 0,
                    // CR 702.175a: Tokens/copies are never cast, so offspring_paid is always false.
                    offspring_paid: false,
                    // CR 702.174a: tokens/copies are never gift casts.
                    gift_was_given: false,
                    champion_exiled_card: None,
                    gift_opponent: None,
            // CR 702.171b: tokens are not saddled by default.
                    encoded_cards: im::Vector::new(),
                    haunting_target: None,
                    // CR 702.151b: tokens are not reconfigured by default.
                    // CR 729.2: tokens are not part of a merged permanent by default.
                    merged_components: im::Vector::new(),
                    // CR 712.8a: DFC state is reset for all new permanents.
                    is_transformed: false,
                    last_transform_timestamp: 0,
                    was_cast_disturbed: false,
                    craft_exiled_cards: im::Vector::new(),
                    chosen_creature_type: None,
                    face_down_as: None,
                    loyalty_ability_activated_this_turn: false,
                    class_level: 0,
                    designations: Designations::default(),
                    meld_component: None,
                };

                // Add the token to the battlefield.
                let token_id = match state.add_object(token_obj, ZoneId::Battlefield) {
                    Ok(id) => id,
                    Err(_) => continue,
                };

                // CR 707.2: Apply a Layer 1 CopyOf continuous effect so the token
                // has the copiable characteristics of the source creature.
                // This ensures correct P/T, name, subtypes, etc. via the layer system.
                let copy_effect = crate::rules::copy::create_copy_effect(
                    state,
                    token_id,
                    source_object,
                    controller,
                );
                state.continuous_effects.push_back(copy_effect);

                // CR 702.116a: Token is "tapped and attacking" -- register it in combat state
                // as attacking the opponent. Tokens enter attacking but were NOT declared
                // as attackers, so "whenever a creature attacks" triggers do NOT fire
                // on them (including the token's own myriad ability).
                if let Some(combat) = state.combat.as_mut() {
                    combat.attackers.insert(
                        token_id,
                        crate::state::combat::AttackTarget::Player(opponent_id),
                    );
                }

                events.push(GameEvent::TokenCreated {
                    player: controller,
                    object_id: token_id,
                });
                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: controller,
                    object_id: token_id,
                });
            }

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.62a: Suspend counter-removal trigger resolves.
        //
        // "At the beginning of your upkeep, if this card is suspended, remove a
        // time counter from it." (CR 702.62a second triggered ability)
        //
        // Intervening-if check (CR 603.4): verify card is still in exile and
        // still has time counters (is still "suspended" per CR 702.62b).
        StackObjectKind::SuspendCounterTrigger {
            source_object: _,
            suspended_card,
        } => {
            let controller = stack_obj.controller;

            // CR 603.4: Re-check intervening-if condition at resolution.
            // Card must still be in exile and have at least one time counter.
            let current_counters = state
                .objects
                .get(&suspended_card)
                .filter(|obj| {
                    obj.zone == ZoneId::Exile && obj.designations.contains(Designations::SUSPENDED)
                })
                .and_then(|obj| obj.counters.get(&CounterType::Time).copied());

            if let Some(count) = current_counters {
                if count > 0 {
                    // Remove one time counter (CR 702.62a).
                    if let Some(obj) = state.objects.get_mut(&suspended_card) {
                        let new_count = count - 1;
                        if new_count == 0 {
                            obj.counters.remove(&CounterType::Time);
                        } else {
                            obj.counters.insert(CounterType::Time, new_count);
                        }
                    }
                    events.push(GameEvent::CounterRemoved {
                        object_id: suspended_card,
                        counter: CounterType::Time,
                        count: 1,
                    });

                    // If this was the last time counter, queue the suspend cast trigger.
                    // CR 702.62a (third triggered ability): "When the last time counter
                    // is removed from this card, if it's exiled, you may play it without
                    // paying its mana cost if able."
                    if count == 1 {
                        let owner = state
                            .objects
                            .get(&suspended_card)
                            .map(|obj| obj.owner)
                            .unwrap_or(controller);
                        state.pending_triggers.push_back(crate::state::stubs::PendingTrigger {
                            data: Some(crate::state::stack::TriggerData::Suspend {
                                card: suspended_card,
                            }),
                            ..crate::state::stubs::PendingTrigger::blank(
                                suspended_card,
                                owner,
                                PendingTriggerKind::SuspendCast,
                            )
                        });
                    }
                }
            }
            // If not in exile or no counters, the trigger does nothing (CR 603.4).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.62a: Suspend cast trigger resolves.
        //
        // "When the last time counter is removed from this card, if it's exiled,
        // you may play it without paying its mana cost if able." (CR 702.62a third ability)
        //
        // V1: Always cast (no interactive "may" choice). Cards are cast without
        // paying their mana cost. Timing restrictions are ignored (CR 702.62d).
        // If the spell is a creature, clear summoning sickness on ETB to grant haste.
        StackObjectKind::SuspendCastTrigger {
            source_object: _,
            suspended_card,
            owner,
        } => {
            let controller = stack_obj.controller;

            // CR 603.4: Re-check intervening-if condition — card must still be in exile.
            let still_in_exile = state
                .objects
                .get(&suspended_card)
                .map(|obj| obj.zone == ZoneId::Exile)
                .unwrap_or(false);

            if still_in_exile {
                // Cast the card without paying its mana cost (CR 702.62a / CR 702.62d).
                // This follows the same pattern as cascade's free-cast (copy.rs:resolve_cascade).
                let stack_entry_id = state.next_object_id();

                // Move card from exile to stack zone (new ObjectId via CR 400.7).
                match state.move_object_to_zone(suspended_card, ZoneId::Stack) {
                    Ok((stack_source_id, _old)) => {
                        // Check if the spell is a creature (for haste grant).
                        let is_creature = state
                            .objects
                            .get(&stack_source_id)
                            .map(|obj| obj.characteristics.card_types.contains(&CardType::Creature))
                            .unwrap_or(false);

                        // Create a StackObject for the suspended spell.
                        // CR 702.62a: suspend IS a cast — it triggers "whenever you cast a spell".
                        // MR-TC-25: use trigger_default; override was_suspended = true.
                        let mut suspend_stack_obj =
                            crate::state::stack::StackObject::trigger_default(
                                stack_entry_id,
                                owner,
                                StackObjectKind::Spell {
                                    source_object: stack_source_id,
                                },
                            );
                        // CR 702.62a: mark this spell as cast via suspend
                        // so resolution.rs can clear summoning sickness on ETB.
                        suspend_stack_obj.was_suspended = true;
                        state.stack_objects.push_back(suspend_stack_obj);

                        // CR 116.3b: Casting a spell resets priority. All players must
                        // pass again before the newly-cast suspend spell resolves.
                        state.turn.players_passed = im::OrdSet::new();

                        // CR 702.62a: suspend triggers "whenever you cast a spell".
                        if let Some(ps) = state.players.get_mut(&owner) {
                            ps.spells_cast_this_turn = ps.spells_cast_this_turn.saturating_add(1);
                        }

                        events.push(GameEvent::SpellCast {
                            player: owner,
                            stack_object_id: stack_entry_id,
                            source_object_id: stack_source_id,
                        });

                        // For creature spells cast via suspend: the permanent will gain
                        // haste. We mark this by noting is_creature here. The actual
                        // haste grant (clearing summoning sickness) is done in the
                        // Spell resolution arm when was_suspended is true.
                        let _ = is_creature; // used at permanent ETB time via was_suspended flag
                    }
                    Err(_) => {
                        // Card disappeared — nothing to cast.
                    }
                }
            }
            // If not in exile (already moved, countered, etc.), do nothing per CR 603.4.

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.75a: Hideaway ETB trigger resolution.
        //
        // "When this permanent enters, look at the top N cards of your library.
        // Exile one of them face down and put the rest on the bottom of your
        // library in a random order."
        //
        // Deterministic fallback: exile the top card; put the rest on the
        // bottom in seeded-shuffle order (using `timestamp_counter` as seed).
        // CR 603.3: Trigger resolves even if source left the battlefield (CR 400.7).
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Hideaway(_),
            data:
                crate::state::stack::TriggerData::ETBHideaway {
                    count: hideaway_count,
                },
        } => {
            let controller = stack_obj.controller;
            let lib_zone = ZoneId::Library(controller);

            // Collect the top N cards from the controller's library.
            // Library is an ordered zone: last element = top (CR 400.7, zone.rs).
            let top_ids: Vec<ObjectId> = {
                let lib = state.zones.get(&lib_zone);
                lib.map(|z| {
                    let all = z.object_ids(); // bottom-to-top order
                    let n = hideaway_count as usize;
                    let start = if all.len() > n { all.len() - n } else { 0 };
                    all[start..].iter().rev().copied().collect() // top-first order
                })
                .unwrap_or_default()
            };

            if top_ids.is_empty() {
                // Library has no cards; trigger resolves with no effect (CR 702.75a).
                events.push(GameEvent::AbilityResolved {
                    controller,
                    stack_object_id: stack_obj.id,
                });
            } else {
                // Deterministic fallback: exile the first (top) card.
                let exile_card_id = top_ids[0];
                let remaining: Vec<ObjectId> = top_ids[1..].to_vec();

                // Move chosen card to exile face-down (CR 702.75a, CR 406.3).
                match state.move_object_to_zone(exile_card_id, ZoneId::Exile) {
                    Ok((new_exile_id, _)) => {
                        // Set face_down and exiled_by_hideaway on the exiled object.
                        if let Some(exile_obj) = state.objects.get_mut(&new_exile_id) {
                            exile_obj.status.face_down = true;
                            exile_obj.exiled_by_hideaway = Some(source_object);
                        }

                        // Put remaining cards on the bottom of the library in a random
                        // (seeded) order (CR 702.75a: "random order").
                        // Seeded Fisher-Yates using timestamp_counter as seed.
                        let seed = state.timestamp_counter;
                        let mut shuffled = remaining.clone();
                        let mut rng_state = seed;
                        for i in (1..shuffled.len()).rev() {
                            rng_state = rng_state
                                .wrapping_mul(6_364_136_223_846_793_005)
                                .wrapping_add(1_442_695_040_888_963_407);
                            let j = (rng_state as usize) % (i + 1);
                            shuffled.swap(i, j);
                        }
                        // Each remaining card is already in the library (they were just
                        // looked at, not moved). They stay in the library; we reorder
                        // the bottom N-1 by moving them out and back in at the bottom.
                        for &card_id in &shuffled {
                            let _ = state.move_object_to_bottom_of_zone(card_id, lib_zone);
                        }

                        events.push(GameEvent::HideawayExiled {
                            player: controller,
                            source: source_object,
                            exiled_card: new_exile_id,
                            remaining_count: shuffled.len() as u32,
                        });
                        events.push(GameEvent::AbilityResolved {
                            controller,
                            stack_object_id: stack_obj.id,
                        });
                    }
                    Err(_) => {
                        // Could not exile the card (already gone); do nothing.
                        events.push(GameEvent::AbilityResolved {
                            controller,
                            stack_object_id: stack_obj.id,
                        });
                    }
                }
            }
        }

        // CR 702.124j: Partner With ETB trigger resolution.
        //
        // "When this permanent enters, target player may search their library
        // for a card named [name], reveal it, put it into their hand, then
        // shuffle."
        //
        // Deterministic: always search (the 'may' is treated as 'do'), targeting
        // the controller (the player most likely to have the partner in their
        // library). If the card is not in the library, the search finds nothing
        // and the library is shuffled anyway.
        //
        // CR 603.3: Trigger resolves even if source left the battlefield (CR 400.7).
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::PartnerWith(_),
            data:
                crate::state::stack::TriggerData::ETBPartnerWith {
                    partner_name,
                    target_player,
                },
        } => {
            let controller = stack_obj.controller;
            let lib_zone = ZoneId::Library(target_player);

            // Find the first card in the target player's library with the exact name.
            // Use lowest ObjectId for determinism (im::OrdMap iteration order is
            // by key, so iteration is already in ascending ObjectId order).
            let matching_card: Option<crate::state::game_object::ObjectId> = state
                .objects
                .iter()
                .filter(|(_, obj)| obj.zone == lib_zone && obj.characteristics.name == partner_name)
                .map(|(id, _)| *id)
                .next();

            if let Some(card_id) = matching_card {
                // Found -- move to target player's hand (reveal is implicit since
                // the card is being put into hand from a search).
                let hand_zone = ZoneId::Hand(target_player);
                let _ = state.move_object_to_zone(card_id, hand_zone);
            }

            // Whether found or not, shuffle the target player's library (CR 701.20).
            // Use seeded LCG (same pattern as Hideaway) for determinism.
            let seed = state.timestamp_counter;
            state.timestamp_counter += 1;
            if let Some(zone) = state.zones.get_mut(&lib_zone) {
                let ids: Vec<crate::state::game_object::ObjectId> = zone.object_ids();
                let mut shuffled = ids;
                let mut rng_state = seed;
                for i in (1..shuffled.len()).rev() {
                    rng_state = rng_state
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1_442_695_040_888_963_407);
                    let j = (rng_state as usize) % (i + 1);
                    shuffled.swap(i, j);
                }
                // Reorder: move each card to the bottom in the new order.
                for &card_id in &shuffled {
                    let _ = state.move_object_to_bottom_of_zone(card_id, lib_zone);
                }
            }

            events.push(GameEvent::LibraryShuffled {
                player: target_player,
            });
            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.115a: Ingest trigger resolves -- exile the top card of the
        // damaged player's library.
        //
        // Ruling 2015-08-25: "If the player has no cards in their library when
        // the ingest ability resolves, nothing happens."
        //
        // The exile is face-up (ruling 2015-08-25: "The card exiled by the
        // ingest ability is exiled face up."). The engine's default exile
        // behavior is face-up, so no special handling needed.
        StackObjectKind::KeywordTrigger {
            source_object: _,
            keyword: KeywordAbility::Ingest,
            data: crate::state::stack::TriggerData::IngestExile { target_player },
        } => {
            let controller = stack_obj.controller;
            let lib_id = ZoneId::Library(target_player);

            // Check if the target player has cards in their library.
            let top_card = state.zones.get(&lib_id).and_then(|z| z.top());

            if let Some(card_id) = top_card {
                // Exile the top card (CR 702.115a).
                if let Ok((new_exile_id, _old_obj)) =
                    state.move_object_to_zone(card_id, ZoneId::Exile)
                {
                    events.push(GameEvent::ObjectExiled {
                        player: controller,
                        object_id: card_id,
                        new_exile_id,
                    });
                }
            }
            // If library is empty, do nothing (ruling 2015-08-25).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.25a: Flanking trigger resolves -- the blocking creature gets
        // -1/-1 until end of turn.
        //
        // The -1/-1 is a continuous effect in Layer 7c (PtModify) with
        // UntilEndOfTurn duration. If the blocker has left the battlefield
        // by resolution time (CR 400.7), the trigger does nothing.
        // Combat triggers (Flanking, Rampage, Provoke, Renown, Melee, Poisonous, Enlist)
        // are now dispatched via KeywordTrigger -- see the KeywordTrigger match arm below.

        // CR 702.49a: Ninjutsu activated ability resolves -- put the ninja card from
        // hand (or command zone for commander ninjutsu, CR 702.49d) onto the battlefield
        // tapped and attacking the inherited attack target.
        //
        // CR 702.49c: "The creature with ninjutsu is put onto the battlefield unblocked.
        // It will be attacking the same player, planeswalker, or battle as the creature
        // that was returned to its owner's hand."
        //
        // Ruling 2021-03-19: "The Ninja isn't put onto the battlefield until the
        // ability resolves. If it leaves your hand before then, it won't enter
        // the battlefield at all." (CR 400.7)
        StackObjectKind::NinjutsuAbility {
            source_object: _,
            ninja_card,
            attack_target,
            from_command_zone,
        } => {
            let controller = stack_obj.controller;

            // 1. Check if ninja card is still in the expected zone (CR 400.7).
            //    Hand for regular ninjutsu; command zone for commander ninjutsu.
            //    CRITICAL: ZoneId::Command(player), NOT ZoneId::CommandZone.
            let expected_zone = if from_command_zone {
                ZoneId::Command(controller)
            } else {
                ZoneId::Hand(controller)
            };
            let still_in_zone = state
                .objects
                .get(&ninja_card)
                .map(|obj| obj.zone == expected_zone)
                .unwrap_or(false);

            if still_in_zone {
                // 2. Check attack target is still valid (CR 508.4a).
                //    If invalid, creature enters battlefield but is not attacking.
                let target_valid = match &attack_target {
                    crate::state::combat::AttackTarget::Player(pid) => state
                        .players
                        .get(pid)
                        .map(|p| !p.has_lost && !p.has_conceded)
                        .unwrap_or(false),
                    crate::state::combat::AttackTarget::Planeswalker(oid) => state
                        .objects
                        .get(oid)
                        .map(|o| o.zone == ZoneId::Battlefield)
                        .unwrap_or(false),
                };

                let combat_active = state.combat.is_some();

                // 3. Move ninja from hand/command zone to battlefield (CR 702.49a).
                let (new_id, _old) = state.move_object_to_zone(ninja_card, ZoneId::Battlefield)?;

                // 4. Set controller and tapped status.
                let card_id = state.objects.get(&new_id).and_then(|o| o.card_id.clone());
                if let Some(obj) = state.objects.get_mut(&new_id) {
                    obj.controller = controller;
                    // CR 702.49a: "Put this card onto the battlefield from your hand
                    // tapped and attacking."
                    obj.status.tapped = true;
                }

                // 5. Register in combat state as attacking the same target.
                //    CR 702.49c: "attacking the same player, planeswalker, or battle"
                //    CR 702.49c: "put onto the battlefield unblocked"
                //    Only if target is valid AND combat is still active.
                //
                //    CR 508.4: "Such creatures are 'attacking' but, for the purposes
                //    of trigger events and effects, they never 'attacked.'"
                //    (AttackersDeclared is NOT emitted -- SelfAttacks triggers don't fire.)
                if target_valid && combat_active {
                    if let Some(combat) = state.combat.as_mut() {
                        combat.attackers.insert(new_id, attack_target.clone());
                    }
                }
                // If target_valid is false (CR 508.4a), ninja enters tapped but is NOT
                // registered as an attacking creature.

                // 6. Apply self ETB replacements + global ETB replacements.
                //    Follow the full ETB site pattern (gotchas-infra.md).
                let registry = state.card_registry.clone();
                let self_evts = super::replacement::apply_self_etb_from_definition(
                    state,
                    new_id,
                    controller,
                    card_id.as_ref(),
                    &registry,
                );
                events.extend(self_evts);
                let etb_evts =
                    super::replacement::apply_etb_replacements(state, new_id, controller);
                events.extend(etb_evts);

                // 7. Register replacement abilities and static continuous effects.
                super::replacement::register_permanent_replacement_abilities(
                    state,
                    new_id,
                    controller,
                    card_id.as_ref(),
                    &registry,
                );
                super::replacement::register_static_continuous_effects(
                    state,
                    new_id,
                    card_id.as_ref(),
                    &registry,
                );

                // 8. Emit PermanentEnteredBattlefield.
                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: controller,
                    object_id: new_id,
                });

                // 9. Queue WhenEntersBattlefield triggered abilities from card definition.
                //    CR 603.3: ETB triggers on the ninja's own definition go on the stack.
                let etb_trigger_evts = super::replacement::queue_carddef_etb_triggers(
                    state,
                    new_id,
                    controller,
                    card_id.as_ref(),
                    &registry,
                );
                events.extend(etb_trigger_evts);
            }
            // If ninja left the expected zone, ability does nothing (CR 400.7).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.128a: Embalm activated ability resolves.
        //
        // "Create a token that's a copy of this card, except it's white, it has no
        // mana cost, and it's a Zombie in addition to its other types."
        //
        // The card was already exiled as part of the activation cost (CR 702.128a:
        // "[Cost], Exile this card from your graveyard"). The token's characteristics
        // are derived from the CardDefinition looked up via source_card_id.
        //
        // CR 707.9b: Color override to White (replaces all original colors).
        // CR 707.9d: No mana cost; mana value is 0.
        // CR 702.128a: Zombie subtype added to existing subtypes.
        StackObjectKind::EmbalmAbility { source_card_id } => {
            let controller = stack_obj.controller;
            let registry = state.card_registry.clone();

            // Look up the card definition for token characteristics.
            let def_opt = source_card_id
                .as_ref()
                .and_then(|cid| registry.get(cid.clone()));

            if let Some(def) = def_opt {
                // Build token subtypes: copy from card definition, add Zombie.
                // CR 702.128a: "Zombie in addition to its other types"
                let mut subtypes: im::OrdSet<SubType> = im::OrdSet::new();
                for st in &def.types.subtypes {
                    subtypes.insert(st.clone() as SubType);
                }
                subtypes.insert(SubType("Zombie".to_string()));

                // Build token card types from card definition.
                let mut card_types: im::OrdSet<CardType> = im::OrdSet::new();
                for ct in &def.types.card_types {
                    card_types.insert(*ct);
                }

                // Build token keywords from card definition's printed abilities.
                let mut keywords: im::OrdSet<KeywordAbility> = im::OrdSet::new();
                for ability in &def.abilities {
                    if let crate::cards::card_definition::AbilityDefinition::Keyword(kw) = ability {
                        keywords.insert(kw.clone());
                    }
                }

                // CR 702.128a: "except it's white" -- replace all colors with White.
                // CR 707.9b: This color override becomes the copiable value.
                let mut colors: im::OrdSet<Color> = im::OrdSet::new();
                colors.insert(Color::White);

                // CR 702.128a: "it has no mana cost" -- mana cost is None (mana value 0).
                // CR 707.9d: The CDA that might define color from mana cost is not copied.
                let characteristics = crate::state::game_object::Characteristics {
                    name: def.name.clone(),
                    mana_cost: None, // CR 702.128a: no mana cost
                    colors,
                    color_indicator: None,
                    // CR 707.2: supertypes are copiable values; copy from card definition.
                    // CR 702.128a does not list supertypes among the exceptions, so they
                    // must be preserved (e.g., a Legendary embalm token stays Legendary).
                    supertypes: def.types.supertypes.clone(),
                    card_types,
                    subtypes,
                    rules_text: def.oracle_text.clone(),
                    abilities: im::Vector::new(),
                    keywords,
                    mana_abilities: im::Vector::new(),
                    // TODO(embalm-review-finding-2): Non-keyword triggered/activated abilities
                    // from the card definition are not populated on runtime-created tokens.
                    // This is a pre-existing systemic gap: the builder converts AbilityDefinition
                    // entries into TriggeredAbilityDef/ActivatedAbility structs at state-build
                    // time, but that conversion is not available here at resolution time.
                    // For Sacred Cat (Lifelink = static keyword), impact is zero. For cards
                    // like Angel of Sanctions (ETB exile ability), post-ETB triggers would be
                    // missing. Fix: extract builder conversion into a shared fn callable at
                    // both build time and token-creation time. See ability-review-embalm.md #2.
                    activated_abilities: Vec::new(),
                    triggered_abilities: Vec::new(),
                    power: def.power,
                    toughness: def.toughness,
                    loyalty: None,
                    defense: None,
                };

                let token_obj = crate::state::game_object::GameObject {
                    id: crate::state::game_object::ObjectId(0), // replaced by add_object
                    card_id: source_card_id.clone(),
                    characteristics,
                    controller,
                    owner: controller,
                    zone: ZoneId::Battlefield,
                    status: crate::state::game_object::ObjectStatus::default(),
                    counters: im::OrdMap::new(),
                    attachments: im::Vector::new(),
                    attached_to: None,
                    damage_marked: 0,
                    deathtouch_damage: false,
                    is_token: true,
                    timestamp: 0, // replaced by add_object
                    // CR 302.6: Tokens have summoning sickness when they enter the battlefield.
                    has_summoning_sickness: true,
                    goaded_by: im::Vector::new(),
                    kicker_times_paid: 0,
                    cast_alt_cost: None,
                    foretold_turn: 0,
                    was_unearthed: false,
                    myriad_exile_at_eoc: false,
                    decayed_sacrifice_at_eoc: false,
                    ring_block_sacrifice_at_eoc: false,
                    exiled_by_hideaway: None,
                    // CR 701.60b: tokens are not suspected by default.
                    encore_sacrifice_at_end_step: false,
                    encore_must_attack: None,
                    encore_activated_by: None,
                    is_plotted: false,
                    plotted_turn: 0,
                    is_prototyped: false,
                    was_bargained: false,
                    evidence_collected: false,
                    phased_out_indirectly: false,
                    phased_out_controller: None,
                    creatures_devoured: 0,
                    paired_with: None,
                    tribute_was_paid: false,
                    // CR 107.3m: Tokens/copies are never cast, so x_value is always 0.
                    x_value: 0,
                    // CR 702.157a: Tokens/copies are never cast, so squad_count is always 0.
                    squad_count: 0,
                    // CR 702.175a: Tokens/copies are never cast, so offspring_paid is always false.
                    offspring_paid: false,
                    // CR 702.174a: tokens/copies are never gift casts.
                    gift_was_given: false,
                    champion_exiled_card: None,
                    gift_opponent: None,
            // CR 702.171b: tokens are not saddled by default.
                    encoded_cards: im::Vector::new(),
                    haunting_target: None,
                    // CR 702.151b: tokens are not reconfigured by default.
                    // CR 729.2: tokens are not part of a merged permanent by default.
                    merged_components: im::Vector::new(),
                    // CR 712.8a: DFC state is reset for all new permanents.
                    is_transformed: false,
                    last_transform_timestamp: 0,
                    was_cast_disturbed: false,
                    craft_exiled_cards: im::Vector::new(),
                    chosen_creature_type: None,
                    face_down_as: None,
                    loyalty_ability_activated_this_turn: false,
                    class_level: 0,
                    designations: Designations::default(),
                    meld_component: None,
                };

                // Add the token to the battlefield.
                let token_id = state.add_object(token_obj, ZoneId::Battlefield)?;

                // Set controller (add_object uses a default; enforce it here).
                if let Some(obj) = state.objects.get_mut(&token_id) {
                    obj.controller = controller;
                }

                // Run the full ETB pipeline for the token.
                // (ETB replacements, static continuous effects, ETB triggers.)
                let self_evts = super::replacement::apply_self_etb_from_definition(
                    state,
                    token_id,
                    controller,
                    source_card_id.as_ref(),
                    &registry,
                );
                events.extend(self_evts);
                let etb_evts =
                    super::replacement::apply_etb_replacements(state, token_id, controller);
                events.extend(etb_evts);

                super::replacement::register_permanent_replacement_abilities(
                    state,
                    token_id,
                    controller,
                    source_card_id.as_ref(),
                    &registry,
                );
                super::replacement::register_static_continuous_effects(
                    state,
                    token_id,
                    source_card_id.as_ref(),
                    &registry,
                );

                events.push(GameEvent::TokenCreated {
                    player: controller,
                    object_id: token_id,
                });
                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: controller,
                    object_id: token_id,
                });

                // Queue WhenEntersBattlefield triggered abilities from card definition.
                // CR 603.3: goes on stack at next priority window.
                let etb_trigger_evts = super::replacement::queue_carddef_etb_triggers(
                    state,
                    token_id,
                    controller,
                    source_card_id.as_ref(),
                    &registry,
                );
                events.extend(etb_trigger_evts);
            }
            // If no card definition found, ability does nothing (shouldn't happen in practice).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.129a: Eternalize activated ability resolves.
        //
        // "Create a token that's a copy of this card, except it's black, it's 4/4,
        // it has no mana cost, and it's a Zombie in addition to its other types."
        //
        // The card was already exiled as part of the activation cost (CR 702.129a:
        // "[Cost], Exile this card from your graveyard"). The token's characteristics
        // are derived from the CardDefinition looked up via source_card_id.
        //
        // CR 707.9b: Color override to Black (replaces all original colors).
        // CR 707.9b: P/T override to 4/4 (replaces original power/toughness).
        // CR 707.9d: No mana cost; mana value is 0.
        // CR 702.129a: Zombie subtype added to existing subtypes.
        StackObjectKind::EternalizeAbility {
            source_card_id,
            source_name: _,
        } => {
            let controller = stack_obj.controller;
            let registry = state.card_registry.clone();

            // Look up the card definition for token characteristics.
            let def_opt = source_card_id
                .as_ref()
                .and_then(|cid| registry.get(cid.clone()));

            if let Some(def) = def_opt {
                // Build token subtypes: copy from card definition, add Zombie.
                // CR 702.129a: "Zombie in addition to its other types"
                let mut subtypes: im::OrdSet<SubType> = im::OrdSet::new();
                for st in &def.types.subtypes {
                    subtypes.insert(st.clone() as SubType);
                }
                subtypes.insert(SubType("Zombie".to_string()));

                // Build token card types from card definition.
                let mut card_types: im::OrdSet<CardType> = im::OrdSet::new();
                for ct in &def.types.card_types {
                    card_types.insert(*ct);
                }

                // Build token keywords from card definition's printed abilities.
                let mut keywords: im::OrdSet<KeywordAbility> = im::OrdSet::new();
                for ability in &def.abilities {
                    if let crate::cards::card_definition::AbilityDefinition::Keyword(kw) = ability {
                        keywords.insert(kw.clone());
                    }
                }

                // CR 702.129a: "except it's black" -- replace all colors with Black.
                // CR 707.9b: This color override becomes the copiable value.
                let mut colors: im::OrdSet<Color> = im::OrdSet::new();
                colors.insert(Color::Black);

                // CR 702.129a: "it has no mana cost" -- mana cost is None (mana value 0).
                // CR 702.129a: "it's 4/4" -- P/T overridden to 4/4.
                // CR 707.9d: The CDA that might define color from mana cost is not copied.
                let characteristics = crate::state::game_object::Characteristics {
                    name: def.name.clone(),
                    mana_cost: None, // CR 702.129a: no mana cost
                    colors,
                    color_indicator: None,
                    // CR 707.2: supertypes are copiable values; copy from card definition.
                    // CR 702.129a does not list supertypes among the exceptions, so they
                    // must be preserved (e.g., a Legendary eternalize token stays Legendary).
                    supertypes: def.types.supertypes.clone(),
                    card_types,
                    subtypes,
                    rules_text: def.oracle_text.clone(),
                    abilities: im::Vector::new(),
                    keywords,
                    mana_abilities: im::Vector::new(),
                    // TODO(eternalize-review-finding-1): Non-keyword triggered/activated
                    // abilities from the card definition are not populated on runtime-created
                    // tokens. This is the same gap as Embalm (ability-review-embalm.md #2).
                    // Fix: extract builder conversion into a shared fn callable at both build
                    // time and token-creation time.
                    activated_abilities: Vec::new(),
                    triggered_abilities: Vec::new(),
                    power: Some(4),     // CR 702.129a: 4/4
                    toughness: Some(4), // CR 702.129a: 4/4
                    loyalty: None,
                    defense: None,
                };

                let token_obj = crate::state::game_object::GameObject {
                    id: crate::state::game_object::ObjectId(0), // replaced by add_object
                    card_id: source_card_id.clone(),
                    characteristics,
                    controller,
                    owner: controller,
                    zone: ZoneId::Battlefield,
                    status: crate::state::game_object::ObjectStatus::default(),
                    counters: im::OrdMap::new(),
                    attachments: im::Vector::new(),
                    attached_to: None,
                    damage_marked: 0,
                    deathtouch_damage: false,
                    is_token: true,
                    timestamp: 0, // replaced by add_object
                    // CR 302.6: Tokens have summoning sickness when they enter the battlefield.
                    has_summoning_sickness: true,
                    goaded_by: im::Vector::new(),
                    kicker_times_paid: 0,
                    cast_alt_cost: None,
                    foretold_turn: 0,
                    was_unearthed: false,
                    myriad_exile_at_eoc: false,
                    decayed_sacrifice_at_eoc: false,
                    ring_block_sacrifice_at_eoc: false,
                    exiled_by_hideaway: None,
                    // CR 701.60b: tokens are not suspected by default.
                    encore_sacrifice_at_end_step: false,
                    encore_must_attack: None,
                    encore_activated_by: None,
                    is_plotted: false,
                    plotted_turn: 0,
                    is_prototyped: false,
                    was_bargained: false,
                    evidence_collected: false,
                    phased_out_indirectly: false,
                    phased_out_controller: None,
                    creatures_devoured: 0,
                    paired_with: None,
                    tribute_was_paid: false,
                    // CR 107.3m: Tokens/copies are never cast, so x_value is always 0.
                    x_value: 0,
                    // CR 702.157a: Tokens/copies are never cast, so squad_count is always 0.
                    squad_count: 0,
                    // CR 702.175a: Tokens/copies are never cast, so offspring_paid is always false.
                    offspring_paid: false,
                    // CR 702.174a: tokens/copies are never gift casts.
                    gift_was_given: false,
                    champion_exiled_card: None,
                    gift_opponent: None,
            // CR 702.171b: tokens are not saddled by default.
                    encoded_cards: im::Vector::new(),
                    haunting_target: None,
                    // CR 702.151b: tokens are not reconfigured by default.
                    // CR 729.2: tokens are not part of a merged permanent by default.
                    merged_components: im::Vector::new(),
                    // CR 712.8a: DFC state is reset for all new permanents.
                    is_transformed: false,
                    last_transform_timestamp: 0,
                    was_cast_disturbed: false,
                    craft_exiled_cards: im::Vector::new(),
                    chosen_creature_type: None,
                    face_down_as: None,
                    loyalty_ability_activated_this_turn: false,
                    class_level: 0,
                    designations: Designations::default(),
                    meld_component: None,
                };

                // Add the token to the battlefield.
                let token_id = state.add_object(token_obj, ZoneId::Battlefield)?;

                // Set controller (add_object uses a default; enforce it here).
                if let Some(obj) = state.objects.get_mut(&token_id) {
                    obj.controller = controller;
                }

                // Run the full ETB pipeline for the token.
                // (ETB replacements, static continuous effects, ETB triggers.)
                let self_evts = super::replacement::apply_self_etb_from_definition(
                    state,
                    token_id,
                    controller,
                    source_card_id.as_ref(),
                    &registry,
                );
                events.extend(self_evts);
                let etb_evts =
                    super::replacement::apply_etb_replacements(state, token_id, controller);
                events.extend(etb_evts);

                super::replacement::register_permanent_replacement_abilities(
                    state,
                    token_id,
                    controller,
                    source_card_id.as_ref(),
                    &registry,
                );
                super::replacement::register_static_continuous_effects(
                    state,
                    token_id,
                    source_card_id.as_ref(),
                    &registry,
                );

                events.push(GameEvent::TokenCreated {
                    player: controller,
                    object_id: token_id,
                });
                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: controller,
                    object_id: token_id,
                });

                // Queue WhenEntersBattlefield triggered abilities from card definition.
                // CR 603.3: goes on stack at next priority window.
                let etb_trigger_evts = super::replacement::queue_carddef_etb_triggers(
                    state,
                    token_id,
                    controller,
                    source_card_id.as_ref(),
                    &registry,
                );
                events.extend(etb_trigger_evts);
            }
            // If no card definition found, ability does nothing (shouldn't happen in practice).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.141a: Encore activated ability resolves.
        //
        // "For each opponent, create a token that's a copy of this card that
        // attacks that opponent this turn if able. The tokens gain haste.
        // Sacrifice them at the beginning of the next end step."
        //
        // The card was already exiled as part of the activation cost (CR 702.141a:
        // "[Cost], Exile this card from your graveyard"). The token's characteristics
        // are derived from the CardDefinition looked up via source_card_id.
        //
        // CR 702.141a ruling: "opponents who have left the game aren't counted."
        // Only active opponents (not eliminated/conceded) get a token.
        StackObjectKind::EncoreAbility {
            source_card_id,
            activator,
        } => {
            let controller = stack_obj.controller;
            let registry = state.card_registry.clone();

            // Look up the card definition for token characteristics.
            let def_opt = source_card_id
                .as_ref()
                .and_then(|cid| registry.get(cid.clone()));

            if let Some(def) = def_opt {
                // Collect active opponents (players who haven't lost or conceded).
                // CR 702.141a: "for each opponent" -- only active opponents.
                let opponent_ids: Vec<crate::state::player::PlayerId> = state
                    .players
                    .values()
                    .filter(|p| !p.has_lost && !p.has_conceded && p.id != activator)
                    .map(|p| p.id)
                    .collect();

                // Build token keywords from card definition's printed abilities.
                // CR 702.141a: "tokens gain haste" -- add Haste to whatever the card has.
                let mut base_keywords: im::OrdSet<KeywordAbility> = im::OrdSet::new();
                for ability in &def.abilities {
                    if let crate::cards::card_definition::AbilityDefinition::Keyword(kw) = ability {
                        base_keywords.insert(kw.clone());
                    }
                }
                base_keywords.insert(KeywordAbility::Haste);

                // Build token card types from card definition.
                let mut card_types: im::OrdSet<CardType> = im::OrdSet::new();
                for ct in &def.types.card_types {
                    card_types.insert(*ct);
                }

                // Build token subtypes from card definition.
                let mut subtypes: im::OrdSet<SubType> = im::OrdSet::new();
                for st in &def.types.subtypes {
                    subtypes.insert(st.clone() as SubType);
                }

                // Build token colors from card definition (copies original colors).
                // CR 707.2: copiable values include color.
                let mut colors: im::OrdSet<Color> = im::OrdSet::new();
                if let Some(ref mc) = def.mana_cost {
                    if mc.white > 0 {
                        colors.insert(Color::White);
                    }
                    if mc.blue > 0 {
                        colors.insert(Color::Blue);
                    }
                    if mc.black > 0 {
                        colors.insert(Color::Black);
                    }
                    if mc.red > 0 {
                        colors.insert(Color::Red);
                    }
                    if mc.green > 0 {
                        colors.insert(Color::Green);
                    }
                }

                for opponent_id in opponent_ids {
                    let keywords = base_keywords.clone();

                    let characteristics = crate::state::game_object::Characteristics {
                        name: def.name.clone(),
                        mana_cost: def.mana_cost.clone(), // copies original mana cost
                        colors: colors.clone(),
                        color_indicator: None,
                        supertypes: def.types.supertypes.clone(),
                        card_types: card_types.clone(),
                        subtypes: subtypes.clone(),
                        rules_text: def.oracle_text.clone(),
                        abilities: im::Vector::new(),
                        keywords,
                        mana_abilities: im::Vector::new(),
                        activated_abilities: Vec::new(),
                        triggered_abilities: Vec::new(),
                        power: def.power,
                        toughness: def.toughness,
                        loyalty: None,
                        defense: None,
                    };

                    let token_obj = crate::state::game_object::GameObject {
                        id: crate::state::game_object::ObjectId(0), // replaced by add_object
                        card_id: source_card_id.clone(),
                        characteristics,
                        controller,
                        owner: controller,
                        zone: ZoneId::Battlefield,
                        status: crate::state::game_object::ObjectStatus::default(),
                        counters: im::OrdMap::new(),
                        attachments: im::Vector::new(),
                        attached_to: None,
                        damage_marked: 0,
                        deathtouch_damage: false,
                        is_token: true,
                        timestamp: 0, // replaced by add_object
                        // CR 302.6: Tokens have summoning sickness when they enter.
                        // Has Haste so can attack despite summoning sickness.
                        has_summoning_sickness: true,
                        goaded_by: im::Vector::new(),
                        kicker_times_paid: 0,
                        cast_alt_cost: None,
                        foretold_turn: 0,
                        was_unearthed: false,
                        myriad_exile_at_eoc: false,
                        decayed_sacrifice_at_eoc: false,
                        ring_block_sacrifice_at_eoc: false,
                        exiled_by_hideaway: None,
                        // CR 701.60b: tokens are not suspected by default.
                        encore_sacrifice_at_end_step: true, // sacrificed at end step
                        encore_must_attack: Some(opponent_id), // must attack this opponent
                        // Ruling 2020-11-10: track the original activator so the end-step
                        // sacrifice trigger can verify control hasn't changed.
                        encore_activated_by: Some(controller),
                        is_plotted: false,
                        plotted_turn: 0,
                        is_prototyped: false,
                        was_bargained: false,
                        evidence_collected: false,
                        phased_out_indirectly: false,
                        phased_out_controller: None,
                        creatures_devoured: 0,
                        paired_with: None,
                        tribute_was_paid: false,
                        // CR 107.3m: Tokens are never cast, so x_value is always 0.
                        x_value: 0,
                        // CR 702.157a: Tokens are never cast, so squad_count is always 0.
                        squad_count: 0,
                        offspring_paid: false,
                        // CR 702.174a: tokens/copies are never gift casts.
                        gift_was_given: false,
                        champion_exiled_card: None,
                        gift_opponent: None,
            // CR 702.171b: tokens are not saddled by default.
                        encoded_cards: im::Vector::new(),
                        haunting_target: None,
                        // CR 702.151b: tokens are not reconfigured by default.
                        // CR 729.2: tokens are not part of a merged permanent by default.
                        merged_components: im::Vector::new(),
                        // CR 712.8a: DFC state is reset for all new permanents.
                        is_transformed: false,
                        last_transform_timestamp: 0,
                        was_cast_disturbed: false,
                        craft_exiled_cards: im::Vector::new(),
                        chosen_creature_type: None,
                        face_down_as: None,
                        loyalty_ability_activated_this_turn: false,
                        class_level: 0,
                        designations: Designations::default(),
                        meld_component: None,
                    };

                    // Add the token to the battlefield.
                    let token_id = state.add_object(token_obj, ZoneId::Battlefield)?;

                    // Set controller (add_object uses a default; enforce it here).
                    if let Some(obj) = state.objects.get_mut(&token_id) {
                        obj.controller = controller;
                    }

                    // Run the full ETB pipeline for the token.
                    let self_evts = super::replacement::apply_self_etb_from_definition(
                        state,
                        token_id,
                        controller,
                        source_card_id.as_ref(),
                        &registry,
                    );
                    events.extend(self_evts);
                    let etb_evts =
                        super::replacement::apply_etb_replacements(state, token_id, controller);
                    events.extend(etb_evts);

                    super::replacement::register_permanent_replacement_abilities(
                        state,
                        token_id,
                        controller,
                        source_card_id.as_ref(),
                        &registry,
                    );
                    super::replacement::register_static_continuous_effects(
                        state,
                        token_id,
                        source_card_id.as_ref(),
                        &registry,
                    );

                    events.push(GameEvent::TokenCreated {
                        player: controller,
                        object_id: token_id,
                    });
                    events.push(GameEvent::PermanentEnteredBattlefield {
                        player: controller,
                        object_id: token_id,
                    });

                    // Queue WhenEntersBattlefield triggered abilities from card definition.
                    // CR 603.3: goes on stack at next priority window.
                    let etb_trigger_evts = super::replacement::queue_carddef_etb_triggers(
                        state,
                        token_id,
                        controller,
                        source_card_id.as_ref(),
                        &registry,
                    );
                    events.extend(etb_trigger_evts);
                }
            }
            // If no card definition found, ability does nothing.

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.141a: Encore delayed sacrifice trigger resolves.
        //
        // "Sacrifice them at the beginning of the next end step."
        // This delayed triggered ability was created when the encore tokens entered
        // the battlefield. At resolution:
        // 1. Check the token is still on the battlefield (CR 400.7).
        // 2. Check the token is still controlled by the encore activator
        //    (ruling 2020-11-10: can't sacrifice if under another player's control).
        // 3. If both checks pass, sacrifice the token.
        StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::Encore,
            data: crate::state::stack::TriggerData::EncoreSacrifice { activator },
        } => {
            let controller = stack_obj.controller;

            // Check if the token is still on the battlefield (CR 400.7).
            let token_info = state
                .objects
                .get(&source_object)
                .filter(|obj| obj.zone == ZoneId::Battlefield)
                .map(|obj| (obj.owner, obj.controller));

            if let Some((owner, current_controller)) = token_info {
                // Ruling 2020-11-10: "If one of the tokens is under another player's
                // control as the delayed triggered ability resolves, you can't sacrifice
                // that token." Only sacrifice if still controlled by the activator.
                if current_controller == activator {
                    let pre_death_counters = state
                        .objects
                        .get(&source_object)
                        .map(|o| o.counters.clone())
                        .unwrap_or_default();

                    // Check replacement effects before moving to graveyard.
                    let action = crate::rules::replacement::check_zone_change_replacement(
                        state,
                        source_object,
                        crate::state::zone::ZoneType::Battlefield,
                        crate::state::zone::ZoneType::Graveyard,
                        owner,
                        &std::collections::HashSet::new(),
                    );

                    match action {
                        crate::rules::replacement::ZoneChangeAction::Redirect {
                            to,
                            events: repl_events,
                            ..
                        } => {
                            events.extend(repl_events);
                            if let Ok((new_id, _old)) = state.move_object_to_zone(source_object, to)
                            {
                                match to {
                                    ZoneId::Exile => {
                                        events.push(GameEvent::ObjectExiled {
                                            player: current_controller,
                                            object_id: source_object,
                                            new_exile_id: new_id,
                                        });
                                    }
                                    ZoneId::Command(_) => {
                                        // Commander redirected -- no CreatureDied.
                                    }
                                    _ => {
                                        events.push(GameEvent::CreatureDied {
                                            object_id: source_object,
                                            new_grave_id: new_id,
                                            controller: current_controller,
                                            pre_death_counters,
                                        });
                                    }
                                }
                            }
                        }
                        crate::rules::replacement::ZoneChangeAction::Proceed => {
                            if let Ok((new_id, _old)) =
                                state.move_object_to_zone(source_object, ZoneId::Graveyard(owner))
                            {
                                events.push(GameEvent::CreatureDied {
                                    object_id: source_object,
                                    new_grave_id: new_id,
                                    controller: current_controller,
                                    pre_death_counters,
                                });
                            }
                        }
                        crate::rules::replacement::ZoneChangeAction::ChoiceRequired { .. } => {
                            // Multiple replacements -- fall back to Proceed (graveyard).
                            if let Ok((new_id, _old)) =
                                state.move_object_to_zone(source_object, ZoneId::Graveyard(owner))
                            {
                                events.push(GameEvent::CreatureDied {
                                    object_id: source_object,
                                    new_grave_id: new_id,
                                    controller: current_controller,
                                    pre_death_counters,
                                });
                            }
                        }
                    }
                }
                // else: token under another player's control -- do nothing (stays).
            }
            // If not on battlefield, do nothing (already gone).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.140b / CR 729.2: Mutating creature spell resolution.
        //
        // CR 702.140b: If the target becomes illegal before this spell resolves
        // (it left the battlefield, stopped being a creature, became a Human, or
        // gained protection from the mutating spell), the spell ceases to be a
        // mutating creature spell and instead resolves as a normal creature spell —
        // the creature enters the battlefield as if the mutate cost had not been paid.
        //
        // CR 729.2: When the target is still legal, the resolving card is placed onto
        // the target permanent (on top if mutate_on_top=true, underneath if false).
        // The spell does NOT enter the battlefield as a separate permanent.
        // The target permanent's characteristics are updated via the layer system.
        StackObjectKind::MutatingCreatureSpell {
            source_object,
            target,
        } => {
            let controller = stack_obj.controller;
            let mutate_on_top = stack_obj
                .additional_costs
                .iter()
                .find_map(|c| match c {
                    AdditionalCost::Mutate { on_top, .. } => Some(*on_top),
                    _ => None,
                })
                .unwrap_or(false);

            // CR 702.140b: Check if the target is still legal at resolution time.
            let target_still_legal = {
                if let Some(target_obj) = state.objects.get(&target) {
                    let target_on_battlefield = target_obj.zone == ZoneId::Battlefield;
                    let target_chars =
                        crate::rules::layers::calculate_characteristics(state, target)
                            .unwrap_or_else(|| target_obj.characteristics.clone());
                    let target_is_creature = target_chars.card_types.contains(&CardType::Creature);
                    let target_is_human = target_chars
                        .subtypes
                        .contains(&SubType("Human".to_string()));
                    let target_owned_by_controller = target_obj.owner == controller;
                    target_on_battlefield
                        && target_is_creature
                        && !target_is_human
                        && target_owned_by_controller
                } else {
                    false
                }
            };

            if !target_still_legal {
                // CR 702.140b: Illegal target fallback — resolve as a normal creature spell.
                // The spell enters the battlefield as a regular creature (no merge).
                let (new_id, _old) =
                    state.move_object_to_zone(source_object, ZoneId::Battlefield)?;
                if let Some(obj) = state.objects.get_mut(&new_id) {
                    obj.controller = controller;
                    obj.cast_alt_cost = Some(AltCostKind::Mutate);
                    obj.has_summoning_sickness = true;
                }
                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: controller,
                    object_id: new_id,
                });
                events.push(GameEvent::SpellResolved {
                    player: controller,
                    stack_object_id: stack_obj.id,
                    source_object_id: new_id,
                });
            } else {
                // CR 729.2: Legal target — merge the spell with the target permanent.
                // The spell does NOT enter the battlefield separately.

                // Step 1: Capture the spell's data from the source object BEFORE removing it.
                let spell_card_id = state
                    .objects
                    .get(&source_object)
                    .and_then(|o| o.card_id.clone());
                let spell_characteristics = state
                    .objects
                    .get(&source_object)
                    .map(|o| o.characteristics.clone())
                    .unwrap_or_default();
                let spell_is_token = state
                    .objects
                    .get(&source_object)
                    .map(|o| o.is_token)
                    .unwrap_or(false);

                // Step 2: Build a MergedComponent from the spell's data.
                let spell_component = MergedComponent {
                    card_id: spell_card_id,
                    characteristics: spell_characteristics,
                    is_token: spell_is_token,
                };

                // Step 3: Build a MergedComponent from the target permanent's current data.
                // This is needed if the target has no components yet (first merge).
                let target_existing_components = {
                    if let Some(target_obj) = state.objects.get(&target) {
                        target_obj.merged_components.clone()
                    } else {
                        im::Vector::new()
                    }
                };
                let target_card_id = state.objects.get(&target).and_then(|o| o.card_id.clone());
                let target_characteristics = state
                    .objects
                    .get(&target)
                    .map(|o| o.characteristics.clone())
                    .unwrap_or_default();
                let target_is_token = state
                    .objects
                    .get(&target)
                    .map(|o| o.is_token)
                    .unwrap_or(false);

                // Step 4: Build the new merged_components vector.
                // If the target had no components, first record the target itself as component[0].
                // Then insert the spell component at top (index 0) or bottom (end).
                let new_components: im::Vector<MergedComponent> =
                    if target_existing_components.is_empty() {
                        // First merge: target becomes a component, then spell is added.
                        let target_component = MergedComponent {
                            card_id: target_card_id,
                            characteristics: target_characteristics,
                            is_token: target_is_token,
                        };
                        if mutate_on_top {
                            // Spell on top: [spell, target]
                            let mut v = im::Vector::new();
                            v.push_back(spell_component);
                            v.push_back(target_component);
                            v
                        } else {
                            // Spell on bottom: [target, spell]
                            let mut v = im::Vector::new();
                            v.push_back(target_component);
                            v.push_back(spell_component);
                            v
                        }
                    } else {
                        // Subsequent merge: target already has components.
                        let mut v = target_existing_components;
                        if mutate_on_top {
                            // Spell on top: insert at front (index 0).
                            v.push_front(spell_component);
                        } else {
                            // Spell on bottom: append at end.
                            v.push_back(spell_component);
                        }
                        v
                    };

                // Step 5: Update the target permanent's merged_components.
                // CR 729.2c: The merged permanent is the SAME object — its ObjectId is preserved.
                // No ETB triggers fire. Continuous effects (Auras, Equipment) remain valid.
                //
                // CR 729.2a: Also sync base characteristics from the new topmost component
                // (merged_components[0]). This ensures that trigger scanning and other
                // raw-characteristics lookups (which bypass the layer system) see the correct
                // abilities. The layer system's Layer 1 override is consistent with this.
                if let Some(target_obj) = state.objects.get_mut(&target) {
                    if let Some(top) = new_components.front() {
                        target_obj.characteristics = top.characteristics.clone();
                        target_obj.card_id = top.card_id.clone();
                    }
                    target_obj.merged_components = new_components;
                }

                // Step 6: Remove the spell's source_object from state.
                // CR 729.2b: "The spell leaves its previous zone and becomes part of an object."
                // The card is absorbed into the target permanent's merged_components.
                // It is NOT moved to any zone — it simply ceases to exist as a separate entity.
                let spell_zone = state.objects.get(&source_object).map(|o| o.zone);
                if let Some(zone) = spell_zone {
                    if let Some(zone_set) = state.zones.get_mut(&zone) {
                        zone_set.remove(&source_object);
                    }
                }
                state.objects.remove(&source_object);

                // Step 7: Emit CreatureMutated event (CR 702.140d).
                // This event fires BEFORE "whenever this creature mutates" triggers are checked,
                // so check_triggers below will catch SelfMutates triggers on the merged permanent.
                events.push(GameEvent::CreatureMutated {
                    object_id: target,
                    player: controller,
                });

                // Step 8: Emit SpellResolved for the mutating spell.
                // source_object_id is the target (merged permanent) since the spell
                // became part of it (CR 729.2b). No PermanentEnteredBattlefield (CR 729.2c).
                events.push(GameEvent::SpellResolved {
                    player: controller,
                    stack_object_id: stack_obj.id,
                    source_object_id: target,
                });
            }
        }
        // CR 701.28 / CR 712.18a: Transform trigger resolves — flip the permanent.
        // CR 712.18a: "It doesn't become a new object." Counters/damage/Auras persist.
        // CR 701.27c: If the permanent is no longer on the battlefield, or its timestamp
        // changed (meaning it already transformed due to a later effect), do nothing.
        StackObjectKind::TransformTrigger {
            permanent,
            ability_timestamp,
        } => {
            if let Some(obj) = state.objects.get(&permanent) {
                let still_on_battlefield = obj.zone == ZoneId::Battlefield;
                // CR 701.27c/d: Only transform if the permanent hasn't already transformed
                // since this trigger was put on the stack (timestamp guard).
                let not_already_transformed = obj.last_transform_timestamp <= ability_timestamp;
                // Check the permanent has a back face to transform to/from.
                let has_back_face = obj
                    .card_id
                    .as_ref()
                    .and_then(|cid| state.card_registry.get(cid.clone()))
                    .map(|def| def.back_face.is_some())
                    .unwrap_or(false);
                if still_on_battlefield && not_already_transformed && has_back_face {
                    let to_back_face = !obj.is_transformed;
                    if let Some(obj_mut) = state.objects.get_mut(&permanent) {
                        obj_mut.is_transformed = to_back_face;
                        obj_mut.last_transform_timestamp = state.timestamp_counter;
                        state.timestamp_counter += 1;
                    }
                    events.push(GameEvent::PermanentTransformed {
                        object_id: permanent,
                        to_back_face,
                    });
                }
            }
        }
        // CR 702.167a: Craft ability resolves — return the exiled source to the battlefield transformed.
        // The source and materials were already exiled as cost during activation.
        // CR 702.167b: If the source is no longer in exile (moved by another effect), do nothing.
        StackObjectKind::CraftAbility {
            source_card_id: _,
            exiled_source,
            material_ids: _,
            activator,
        } => {
            // Check the source is still in exile (CR 702.167b / CR 400.7).
            let still_in_exile = state
                .objects
                .get(&exiled_source)
                .map(|obj| obj.zone == ZoneId::Exile)
                .unwrap_or(false);
            let has_back_face = state
                .objects
                .get(&exiled_source)
                .and_then(|obj| obj.card_id.as_ref())
                .and_then(|cid| state.card_registry.get(cid.clone()))
                .map(|def| def.back_face.is_some())
                .unwrap_or(false);
            if still_in_exile && has_back_face {
                let (new_id, _old) =
                    state.move_object_to_zone(exiled_source, ZoneId::Battlefield)?;
                if let Some(obj) = state.objects.get_mut(&new_id) {
                    obj.controller = activator;
                    obj.is_transformed = true;
                    obj.last_transform_timestamp = state.timestamp_counter;
                    state.timestamp_counter += 1;
                }
                let card_id_for_etb = state.objects.get(&new_id).and_then(|o| o.card_id.clone());
                let registry = state.card_registry.clone();
                let self_evts = super::replacement::apply_self_etb_from_definition(
                    state,
                    new_id,
                    activator,
                    card_id_for_etb.as_ref(),
                    &registry,
                );
                events.extend(self_evts);
                let etb_evts = super::replacement::apply_etb_replacements(state, new_id, activator);
                events.extend(etb_evts);
                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: activator,
                    object_id: new_id,
                });
                let db_evts = crate::rules::turn_actions::enforce_daybound_nightbound(state);
                events.extend(db_evts);
            }
        }
        // CR 708.8: "When this permanent is turned face up" trigger resolves.
        // The permanent has already been turned face up (face_down = false). Look up any
        // WhenTurnedFaceUp triggered abilities in the CardDefinition and execute their effects.
        StackObjectKind::TurnFaceUpTrigger {
            permanent,
            source_card_id,
            ability_index,
        } => {
            use crate::cards::card_definition::AbilityDefinition;
            // CR 708.8: "When this permanent is turned face up" trigger resolves.
            // The permanent is already face-up. Execute the specific WhenTurnedFaceUp
            // ability at `ability_index` in the CardDefinition — each TurnFaceUpTrigger
            // SOK carries the exact index to support cards with multiple such abilities.
            let controller = stack_obj.controller;
            if let Some(obj) = state.objects.get(&permanent) {
                if obj.zone == ZoneId::Battlefield {
                    let card_id = source_card_id.or_else(|| obj.card_id.clone());
                    if let Some(cid) = card_id {
                        let registry = state.card_registry.clone();
                        if let Some(def) = registry.get(cid) {
                            if let Some(AbilityDefinition::Triggered { effect, .. }) =
                                def.abilities.get(ability_index)
                            {
                                let mut ctx = crate::effects::EffectContext::new(
                                    controller,
                                    permanent,
                                    vec![],
                                );
                                let effect_events =
                                    crate::effects::execute_effect(state, effect, &mut ctx);
                                events.extend(effect_events);
                            }
                        }
                    }
                }
            }
            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }
        // CR 702.145c / CR 702.145f: Daybound/Nightbound transform trigger resolves.
        // Check the permanent is still on the battlefield and the day/night state still
        // requires this transform before applying it.
        StackObjectKind::DayboundTransformTrigger { permanent } => {
            if let Some(obj) = state.objects.get(&permanent) {
                if obj.zone == ZoneId::Battlefield {
                    let chars = super::layers::calculate_characteristics(state, permanent)
                        .or_else(|| {
                            state
                                .objects
                                .get(&permanent)
                                .map(|o| o.characteristics.clone())
                        })
                        .unwrap_or_default();
                    let still_needs_transform = {
                        let obj = state.objects.get(&permanent).unwrap();
                        // Daybound on front face at night → transform to back
                        (state.day_night == Some(crate::state::DayNight::Night)
                            && chars
                                .keywords
                                .contains(&crate::state::types::KeywordAbility::Daybound)
                            && !obj.is_transformed)
                        // Nightbound on back face at day → transform to front
                        || (state.day_night == Some(crate::state::DayNight::Day)
                            && chars
                                .keywords
                                .contains(&crate::state::types::KeywordAbility::Nightbound)
                            && obj.is_transformed)
                    };
                    if still_needs_transform {
                        let to_back_face = {
                            let obj = state.objects.get(&permanent).unwrap();
                            !obj.is_transformed
                        };
                        if let Some(obj_mut) = state.objects.get_mut(&permanent) {
                            obj_mut.is_transformed = to_back_face;
                            obj_mut.last_transform_timestamp = state.timestamp_counter;
                            state.timestamp_counter += 1;
                        }
                        events.push(GameEvent::PermanentTransformed {
                            object_id: permanent,
                            to_back_face,
                        });
                    }
                }
            }
        }
        // CR 701.54c: Ring-bearer triggered ability resolution.
        // Execute the embedded effect with the ring controller as the controller.
        StackObjectKind::RingAbility {
            source_object,
            effect,
            controller,
        } => {
            let mut ctx = crate::effects::EffectContext::new(controller, source_object, vec![]);
            let effect_events = execute_effect(state, &effect, &mut ctx);
            events.extend(effect_events);
        }

        // CR 716.2a: Class level-up activated ability resolution.
        //
        // Set the Class's level to `target_level` and register any continuous
        // effects declared at that level. If the Class has left the battlefield
        // since the ability was activated, do nothing (CR 608.2b analog).
        StackObjectKind::ClassLevelAbility {
            source_object,
            target_level,
        } => {
            use crate::cards::card_definition::AbilityDefinition;

            // CR 608.2b analog: if the Class is no longer on the battlefield, fizzle.
            let still_on_bf = state
                .objects
                .get(&source_object)
                .map(|o| o.zone == crate::state::zone::ZoneId::Battlefield)
                .unwrap_or(false);

            if still_on_bf {
                // Set the class level.
                if let Some(obj) = state.objects.get_mut(&source_object) {
                    obj.class_level = target_level;
                }

                // Register static continuous effects from the new level's abilities.
                let card_id = state
                    .objects
                    .get(&source_object)
                    .and_then(|o| o.card_id.clone());
                if let Some(cid) = card_id {
                    let registry = state.card_registry.clone();
                    if let Some(def) = registry.get(cid) {
                        let level_abilities: Vec<AbilityDefinition> = def
                            .abilities
                            .iter()
                            .filter_map(|a| match a {
                                AbilityDefinition::ClassLevel {
                                    level, abilities, ..
                                } if *level == target_level => Some(abilities.clone()),
                                _ => None,
                            })
                            .flatten()
                            .collect();

                        for sub_ability in &level_abilities {
                            if let AbilityDefinition::Static { continuous_effect } = sub_ability {
                                let eff_id = state.next_object_id().0;
                                let ts = state.timestamp_counter;
                                state.timestamp_counter += 1;
                                state.continuous_effects.push_back(
                                    crate::state::continuous_effect::ContinuousEffect {
                                        id: crate::state::continuous_effect::EffectId(eff_id),
                                        source: Some(source_object),
                                        timestamp: ts,
                                        layer: continuous_effect.layer,
                                        duration: continuous_effect.duration,
                                        filter: continuous_effect.filter.clone(),
                                        modification: continuous_effect.modification.clone(),
                                        is_cda: false,
                                    },
                                );
                            }
                        }
                    }
                }

                events.push(GameEvent::AbilityResolved {
                    controller: stack_obj.controller,
                    stack_object_id: stack_obj.id,
                });
            }
        }

        // CR 309.4c: Room ability resolution — execute the room's effect.
        // The dungeon is in the command zone; the owner is the venture controller.
        StackObjectKind::RoomAbility {
            owner,
            dungeon,
            room,
        } => {
            use crate::state::dungeon::get_dungeon;
            let dungeon_def = get_dungeon(dungeon);
            if let Some(room_def) = dungeon_def.rooms.get(room) {
                let room_effect = (room_def.effect)();
                // Use a sentinel ObjectId (0) since dungeons have no permanent source.
                let mut ctx = crate::effects::EffectContext::new(
                    owner,
                    crate::state::game_object::ObjectId(0),
                    vec![],
                );
                let effect_events = execute_effect(state, &room_effect, &mut ctx);
                events.extend(effect_events);
            }
        }

        // Consolidated keyword trigger resolution dispatch.
        // Each keyword+data combination delegates to the same logic as the
        // original one-off SOK variant it replaced.
        StackObjectKind::KeywordTrigger {
            ref keyword,
            ref data,
            ..
        } => {
            unreachable!(
                "Unhandled KeywordTrigger: keyword={:?}, data={:?}",
                keyword, data
            );
        }
    }

    // Check for triggered abilities arising from this resolution.
    let new_triggers = abilities::check_triggers(state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }

    // CR 704.3: Check SBAs before granting priority (happens after each resolution).
    // Trigger checking is done inside check_and_apply_sbas (per-pass).
    let sba_events = sba::check_and_apply_sbas(state);
    events.extend(sba_events);

    // Flush any pending triggers onto the stack before granting priority (CR 603.3).
    let trigger_events = abilities::flush_pending_triggers(state);
    events.extend(trigger_events);

    // CR 116.3b: After resolution (and trigger flushing), the active player receives priority.
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);
    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
}

/// CR 702.174d-i: Execute the gift effect for a specific opponent.
///
/// Called at resolution time for both instant/sorcery spells (before main effect, CR 702.174j)
/// and for GiftETBTrigger resolution. The `recipient` is the chosen opponent who receives the
/// gift. The `controller` is the source permanent/spell's controller.
///
/// Gift types and effects (CR 702.174d-i):
///   Food     (702.174d): recipient creates a Food token
///   Card     (702.174e): recipient draws a card
///   Treasure (702.174h): recipient creates a Treasure token
///   TappedFish (702.174f): recipient creates a tapped 1/1 blue Fish creature token (deferred)
///   Octopus  (702.174i): recipient creates an 8/8 blue Octopus creature token (deferred)
///   ExtraTurn (702.174g): recipient takes an extra turn (deferred)
fn execute_gift_effect(
    state: &mut GameState,
    recipient: crate::state::PlayerId,
    _controller: crate::state::PlayerId,
    gift_type: &crate::cards::card_definition::GiftType,
) -> Vec<GameEvent> {
    use crate::cards::card_definition::{food_token_spec, treasure_token_spec, GiftType};

    let mut events = vec![];

    match gift_type {
        GiftType::Food => {
            // CR 702.174d: "The chosen player creates a Food token."
            let spec = food_token_spec(1);
            let obj = crate::effects::make_token(&spec, recipient);
            if let Ok(id) = state.add_object(obj, ZoneId::Battlefield) {
                events.push(GameEvent::TokenCreated {
                    player: recipient,
                    object_id: id,
                });
                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: recipient,
                    object_id: id,
                });
            }
        }
        GiftType::Card => {
            // CR 702.174e: "The chosen player draws a card."
            if let Ok(draw_events) = crate::rules::turn_actions::draw_card(state, recipient) {
                events.extend(draw_events);
            }
        }
        GiftType::Treasure => {
            // CR 702.174h: "The chosen player creates a Treasure token."
            let spec = treasure_token_spec(1);
            let obj = crate::effects::make_token(&spec, recipient);
            if let Ok(id) = state.add_object(obj, ZoneId::Battlefield) {
                events.push(GameEvent::TokenCreated {
                    player: recipient,
                    object_id: id,
                });
                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: recipient,
                    object_id: id,
                });
            }
        }
        GiftType::TappedFish | GiftType::Octopus | GiftType::ExtraTurn => {
            // CR 702.174f/i/g: TappedFish, Octopus, ExtraTurn gifts -- deferred.
            // No cards with these gift types are currently in scope.
            // The ability fires but no effect is applied.
            let _ = recipient;
        }
    }

    events
}

/// CR 608.2b: Check whether a spell target is still legal at resolution time.
///
/// A target is illegal if:
/// - It was an object target and the object is no longer in the same zone it was
///   in when targeted ("A target that's no longer in the zone it was in when it
///   was targeted is illegal." — CR 608.2b).
/// - It was a player target and that player is no longer active (eliminated/conceded).
fn is_target_legal(state: &GameState, spell_target: &SpellTarget) -> bool {
    match &spell_target.target {
        Target::Player(id) => state
            .players
            .get(id)
            .map(|p| !p.has_lost && !p.has_conceded)
            .unwrap_or(false),
        Target::Object(id) => {
            // The object must still be in the same zone it was in at cast time.
            state
                .objects
                .get(id)
                .map(|obj| Some(obj.zone) == spell_target.zone_at_cast)
                .unwrap_or(false)
        }
    }
}

/// Counter a specific stack object without it resolving (CR 608.2b, 701.5).
///
/// Finds the stack object by ID, removes it from the stack, and moves the
/// associated card to its owner's graveyard. After countering, the active
/// player receives priority.
///
/// Used by: the fizzle rule (M3-D), counterspell effects (M3-D/E).
pub fn counter_stack_object(
    state: &mut GameState,
    stack_object_id: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    // Find and remove the specified stack object (may not be the top).
    let pos = state
        .stack_objects
        .iter()
        .position(|s| s.id == stack_object_id)
        .ok_or(GameStateError::ObjectNotFound(stack_object_id))?;
    let stack_obj = state.stack_objects.remove(pos);

    match stack_obj.kind.clone() {
        StackObjectKind::Spell { source_object }
        | StackObjectKind::MutatingCreatureSpell { source_object, .. } => {
            let controller = stack_obj.controller;
            let owner = state.object(source_object)?.owner;
            // CR 702.34a: If cast with flashback, exile instead of graveyard when countered.
            // CR 702.133a: Jump-start also exiles instead of graveyard when countered.
            // CR 702.140: Countered mutating spells move to graveyard like normal spells.
            let destination = if stack_obj.cast_with_flashback || stack_obj.cast_with_jump_start {
                ZoneId::Exile // CR 702.34a / CR 702.133a
            } else {
                ZoneId::Graveyard(owner)
            };
            let (new_id, _old) = state.move_object_to_zone(source_object, destination)?;

            events.push(GameEvent::SpellCountered {
                player: controller,
                stack_object_id: stack_obj.id,
                source_object_id: new_id,
            });
        }
        StackObjectKind::ActivatedAbility { .. }
        | StackObjectKind::TriggeredAbility { .. }
        | StackObjectKind::MadnessTrigger { .. }
        | StackObjectKind::MiracleTrigger { .. }
        | StackObjectKind::UnearthAbility { .. }
        | StackObjectKind::SuspendCounterTrigger { .. }
        | StackObjectKind::SuspendCastTrigger { .. }
        | StackObjectKind::NinjutsuAbility { .. }
        | StackObjectKind::EmbalmAbility { .. }
        | StackObjectKind::EternalizeAbility { .. }
        | StackObjectKind::EncoreAbility { .. }
        | StackObjectKind::ForecastAbility { .. }
        | StackObjectKind::ScavengeAbility { .. }
        | StackObjectKind::BloodrushAbility { .. }
        | StackObjectKind::SaddleAbility { .. }
        // CR 701.28 / CR 712: Transform triggers and Craft abilities countered — no effect.
        | StackObjectKind::TransformTrigger { .. }
        | StackObjectKind::CraftAbility { .. }
        | StackObjectKind::DayboundTransformTrigger { .. }
        // CR 708.8: TurnFaceUpTrigger countered — no effect; permanent remains face-up.
        | StackObjectKind::TurnFaceUpTrigger { .. }
        // All migrated triggers now consolidated under KeywordTrigger.
        | StackObjectKind::KeywordTrigger { .. }
        // CR 309.4c: RoomAbility countered (e.g. by Stifle) — no room effect fires.
        // The venture marker has already been advanced; only the room trigger is countered.
        // CR 701.54c: RingAbility countered — no ring effect fires.
        | StackObjectKind::RoomAbility { .. }
        | StackObjectKind::RingAbility { .. }
        // CR 606: Loyalty ability countered — cost already paid, no effect.
        | StackObjectKind::LoyaltyAbility { .. }
        // CR 716.2a: ClassLevelAbility countered — mana cost already paid, level stays unchanged.
        | StackObjectKind::ClassLevelAbility { .. } => {
            // Countering abilities is non-standard; just remove from stack.
            // Note: For HauntExileTrigger, if countered (e.g. by Stifle), the haunt
            // card stays in the graveyard and no haunting relationship is established.
            // Note: For HauntedCreatureDiesTrigger, if countered (e.g. by Stifle),
            // no haunt effect fires, but the haunt card stays in exile (CR 702.55c).
            // Note: For BloodrushAbility, if countered (e.g. by Stifle), the source
            // card is already in the graveyard (discarded as cost — CR 602.2b). No
            // pump or keyword is applied, but the card stays in the graveyard.
            // Note: For BackupTrigger, if countered (e.g. by Stifle), no counters
            // are placed and no abilities are granted (CR 702.165a).
            // Note: For ScavengeAbility, if countered (e.g. by Stifle), the card is
            // already in exile (exiled as cost during activation). No counters are
            // placed on the target, but the source card stays in exile (CR 702.97a).
            // Note: For ForecastAbility, if countered (e.g. by Stifle), the forecast
            // activation is already consumed (once-per-turn tracked) and the card
            // remains in hand (CR 702.57a).
            // Note: For Echo KeywordTrigger, if countered (e.g. by Stifle), echo_pending
            // remains set so the trigger fires again on the next upkeep (CR 702.30a).
            // Note: For CumulativeUpkeep KeywordTrigger, if countered (e.g. by Stifle), no
            // age counter is added (counter addition happens at resolution, not queueing).
            // The trigger fires again next upkeep with the same counter count (CR 702.24a).
            // Note: For EncoreAbility, the card is already in exile (exiled as cost
            // during activation). Countering does not return the card (CR 702.141a).
            // Note: For DashReturnTrigger, the creature stays on the battlefield
            // with haste (haste is a static ability, not tied to this trigger -- CR 702.109a).
            // Note: For BlitzSacrificeTrigger, the creature stays on the battlefield
            // with haste and the draw-on-death trigger intact (CR 702.152a).
            // Note: For Impending KeywordTrigger (CounterRemoval), if countered (e.g. by Stifle),
            // the permanent retains its time counter(s) and remains a non-creature (CR 702.176a).
            // Note: For CasualtyTrigger, if countered (e.g. by Stifle), the original
            // spell stays on the stack but no copy is made (CR 702.153a).
            // Note: For ReplicateTrigger, if countered (e.g. by Stifle), no copies are
            // made but the original spell stays on the stack (CR 702.56a).
            // Note: For GravestormTrigger, if countered (e.g. by Stifle), no copies are
            // made but the original spell stays on the stack (CR 702.69a).
            // Note: For Vanishing KeywordTrigger (CounterRemoval), if countered (e.g. by Stifle),
            // the permanent retains its time counter(s) (CR 702.63a).
            // Note: For Vanishing KeywordTrigger (CounterSacrifice), if countered (e.g. by Stifle),
            // the permanent stays on the battlefield with 0 time counters (CR 702.63a ruling).
        }
    }

    // CR 704.3: Check SBAs before granting priority.
    let sba_evts = sba::check_and_apply_sbas(state);
    events.extend(sba_evts);

    // After countering, the active player receives priority (same as resolution).
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);
    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
}
