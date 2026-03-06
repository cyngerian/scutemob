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

use crate::effects::{execute_effect, EffectContext};
use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::stack::StackObjectKind;
use crate::state::stubs::PendingTriggerKind;
use crate::state::targeting::{SpellTarget, Target};
use crate::state::types::{
    AltCostKind, CardType, Color, CounterType, EnchantTarget, KeywordAbility, SubType,
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
                            {
                                ZoneId::Exile // CR 702.34a / CR 702.133a
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
                        // CR 702.127a + CR 709.3b: If the aftermath half was cast, use the
                        // aftermath effect instead of the first-half Spell effect.
                        // CR 702.42b: For entwined modal spells, collect the modes so we
                        // can execute all of them (or just mode[0] when not entwined).
                        let (spell_effect, spell_modes) =
                            if stack_obj.cast_with_aftermath {
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

                            // CR 702.42b / CR 702.120a: Mode dispatch.
                            // Entwine takes precedence (all modes). If escalate was paid,
                            // execute modes 0..=escalate_modes_paid. Otherwise, mode[0] only.
                            // If no modes, execute the spell's main effect as before.
                            let effects_to_run: Vec<crate::cards::card_definition::Effect> =
                                if let Some(modes) = spell_modes {
                                    if stack_obj.was_entwined {
                                        // CR 702.42b: "follow the text of each of the modes in
                                        // the order written on the card"
                                        modes.modes.clone()
                                    } else if stack_obj.escalate_modes_paid > 0 {
                                        // CR 702.120a: execute modes 0..=escalate_modes_paid
                                        // (escalate cost was paid once per extra mode beyond
                                        // the first).
                                        let count = (stack_obj.escalate_modes_paid as usize + 1)
                                            .min(modes.modes.len());
                                        modes.modes[..count].to_vec()
                                    } else {
                                        // Auto-select first mode (Batch 11 will add full
                                        // interactive mode selection).
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
                                let splice_events =
                                    execute_effect(state, spliced_effect, &mut splice_ctx);
                                events.extend(splice_events);
                            }
                        }
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
                    obj.kicker_times_paid = stack_obj.kicker_times_paid;
                    // CR 702.166b: Transfer bargained status from stack to permanent so ETB
                    // triggers can check Condition::WasBargained.
                    obj.was_bargained = stack_obj.was_bargained;
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
                    obj.is_bestowed = stack_obj.was_bestowed && !bestow_fallback;
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
                        obj.echo_pending = true;
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

                // CR 604 / CR 613: Register static continuous effects from this
                // permanent's card definition (Equipment, Aura, global ability grants).
                super::replacement::register_static_continuous_effects(
                    state,
                    new_id,
                    card_id.as_ref(),
                    &registry,
                );

                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: controller,
                    object_id: new_id,
                });

                // CR 603.2: Fire mandatory WhenEntersBattlefield triggered effects
                // from card definition inline (Rest in Peace ETB exile, etc.).
                // Interactive/stackable ETB triggers are handled via PendingTrigger.
                let etb_trigger_evts = super::replacement::fire_when_enters_triggered_effects(
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
                let destination = if stack_obj.cast_with_flashback || stack_obj.cast_with_jump_start
                {
                    ZoneId::Exile // CR 702.34a / CR 702.133a — overrides all other destinations
                } else if stack_obj.was_buyback_paid {
                    ZoneId::Hand(owner) // CR 702.27a
                } else {
                    ZoneId::Graveyard(owner)
                };
                let (new_id, _old) = state.move_object_to_zone(source_object, destination)?;

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
        } => {
            // CR 603.4: Check intervening-if condition at resolution time.
            // If the condition is false, the ability has no effect (but still resolves).
            let condition_holds = {
                let source_obj = state.objects.get(&source_object);
                match source_obj {
                    Some(obj) => {
                        let ability_def =
                            obj.characteristics.triggered_abilities.get(ability_index);
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
                let triggered_effect = state
                    .objects
                    .get(&source_object)
                    .and_then(|obj| obj.characteristics.triggered_abilities.get(ability_index))
                    .and_then(|ab| ab.effect.clone());

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
        }

        // CR 702.85a: Cascade trigger resolves — run the cascade procedure.
        StackObjectKind::CascadeTrigger {
            source_object: _,
            spell_mana_value,
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
        StackObjectKind::StormTrigger {
            source_object: _,
            original_stack_id,
            storm_count,
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
        StackObjectKind::CasualtyTrigger {
            source_object: _,
            original_stack_id,
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
        StackObjectKind::ReplicateTrigger {
            source_object: _,
            original_stack_id,
            replicate_count,
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
        StackObjectKind::GravestormTrigger {
            source_object: _,
            original_stack_id,
            gravestorm_count,
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
        StackObjectKind::VanishingCounterTrigger {
            source_object: _,
            vanishing_permanent,
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
                                kind: PendingTriggerKind::VanishingSacrifice,
                                triggering_event: None,
                                entering_object_id: None,
                                targeting_stack_id: None,
                                triggering_player: None,
                                exalted_attacker_id: None,
                                defending_player_id: None,
                                madness_exiled_card: None,
                                madness_cost: None,
                                miracle_revealed_card: None,
                                miracle_cost: None,
                                modular_counter_count: None,
                                evolve_entering_creature: None,
                                suspend_card_id: None,
                                hideaway_count: None,
                                partner_with_name: None,
                                ingest_target_player: None,
                                flanking_blocker_id: None,
                                rampage_n: None,
                                provoke_target_creature: None,
                                renown_n: None,
                                poisonous_n: None,
                                poisonous_target_player: None,
                                enlist_enlisted_creature: None,
                                encore_activator: None,
                                echo_cost: None,
                                cumulative_upkeep_cost: None,
                                recover_cost: None,
                                recover_card: None,
                            });
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
        StackObjectKind::VanishingSacrificeTrigger {
            source_object: _,
            vanishing_permanent,
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
        StackObjectKind::FadingTrigger {
            source_object: _,
            fading_permanent,
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
        StackObjectKind::EchoTrigger {
            source_object: _,
            echo_permanent,
            echo_cost,
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
        StackObjectKind::CumulativeUpkeepTrigger {
            source_object: _,
            cu_permanent,
            per_counter_cost,
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
        StackObjectKind::RecoverTrigger {
            source_object: _,
            recover_card,
            recover_cost,
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
        StackObjectKind::EvokeSacrificeTrigger { source_object } => {
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

                // 6. Fire WhenEntersBattlefield triggered effects from card definition.
                let etb_trigger_evts = super::replacement::fire_when_enters_triggered_effects(
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
        StackObjectKind::UnearthTrigger { source_object } => {
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
        StackObjectKind::DashReturnTrigger { source_object } => {
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
        StackObjectKind::BlitzSacrificeTrigger { source_object } => {
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
        StackObjectKind::ImpendingCounterTrigger {
            source_object: _,
            impending_permanent,
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

        // CR 702.110a: Exploit trigger resolves -- the controller may sacrifice
        // a creature. Default (deterministic, no interactive choice): decline.
        //
        // TODO: Add Command::ExploitCreature for player-interactive sacrifice choice.
        // When interactive choice is added, the trigger would pause and emit an
        // ExploitChoiceRequired event; the player responds with ExploitCreature
        // (naming the creature to sacrifice) or DeclineExploit.
        StackObjectKind::ExploitTrigger { source_object } => {
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
        StackObjectKind::ModularTrigger {
            source_object: _,
            counter_count,
        } => {
            let controller = stack_obj.controller;

            // CR 608.2b: Fizzle check -- verify target is still a legal artifact creature
            // on the battlefield. If it is not, the trigger fizzles with no effect.
            let target_id_opt = stack_obj.targets.first().and_then(|t| match &t.target {
                Target::Object(id) => {
                    let still_legal = state.objects.get(id).is_some_and(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.characteristics.card_types.contains(&CardType::Artifact)
                            && obj.characteristics.card_types.contains(&CardType::Creature)
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
        StackObjectKind::EvolveTrigger {
            source_object,
            entering_creature,
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
        StackObjectKind::MyriadTrigger {
            source_object,
            defending_player,
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
                    is_bestowed: false,
                    is_foretold: false,
                    foretold_turn: 0,
                    was_unearthed: false,
                    // CR 702.116a: "exile the tokens at end of combat"
                    // Tagged here so end_combat() in turn_actions.rs can find them.
                    myriad_exile_at_eoc: true,
                    decayed_sacrifice_at_eoc: false,
                    is_suspended: false,
                    exiled_by_hideaway: None,
                    is_renowned: false,
                    encore_sacrifice_at_end_step: false,
                    encore_must_attack: None,
                    encore_activated_by: None,
                    is_plotted: false,
                    plotted_turn: 0,
                    is_prototyped: false,
                    was_bargained: false,
                    echo_pending: false,
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
                .filter(|obj| obj.zone == ZoneId::Exile && obj.is_suspended)
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
                        state
                            .pending_triggers
                            .push_back(crate::state::stubs::PendingTrigger {
                                source: suspended_card,
                                ability_index: 0,
                                controller: owner,
                                kind: PendingTriggerKind::SuspendCast,
                                triggering_event: None,
                                entering_object_id: None,
                                targeting_stack_id: None,
                                triggering_player: None,
                                exalted_attacker_id: None,
                                defending_player_id: None,
                                madness_exiled_card: None,
                                madness_cost: None,
                                miracle_revealed_card: None,
                                miracle_cost: None,
                                modular_counter_count: None,
                                evolve_entering_creature: None,
                                suspend_card_id: Some(suspended_card),
                                hideaway_count: None,
                                partner_with_name: None,
                                ingest_target_player: None,
                                flanking_blocker_id: None,
                                rampage_n: None,
                                provoke_target_creature: None,
                                renown_n: None,
                                poisonous_n: None,
                                poisonous_target_player: None,
                                enlist_enlisted_creature: None,
                                encore_activator: None,
                                echo_cost: None,
                                cumulative_upkeep_cost: None,
                                recover_cost: None,
                                recover_card: None,
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
                        let suspend_stack_obj = crate::state::stack::StackObject {
                            id: stack_entry_id,
                            controller: owner,
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
                            // CR 702.62a: mark this spell as cast via suspend
                            // so resolution.rs can clear summoning sickness on ETB.
                            was_suspended: true,
                            // CR 702.96a: suspend casts cannot be overloaded.
                            was_overloaded: false,
                            // CR 702.133a: suspend casts are not jump-start casts.
                            cast_with_jump_start: false,
                            // CR 702.127a: suspend casts are not aftermath casts.
                            cast_with_aftermath: false,
                            // CR 702.109a: suspend casts are not dash casts.
                            was_dashed: false,
                            // CR 702.152a: suspend casts are not blitz casts.
                            was_blitzed: false,
                            // CR 702.170d: suspend casts are not plot casts.
                            was_plotted: false,
                            was_prototyped: false,
                            // CR 702.176a: suspend casts are not impending casts.
                            was_impended: false,
                            // CR 702.166b: suspend casts are not bargain casts.
                            was_bargained: false,
                            // CR 702.117a: suspend casts are not surge casts.
                            was_surged: false,
                            // CR 702.153a: suspend casts are not casualty casts.
                            was_casualty_paid: false,
                            // CR 702.148a: suspend casts are not cleave casts.
                            was_cleaved: false,
                            // CR 702.42a: suspend casts are not entwine casts.
                            was_entwined: false,
                            // CR 702.120a: suspend casts have no escalate modes paid.
                            escalate_modes_paid: 0,
                            // CR 702.47a: suspend free-casts have no spliced effects.
                            spliced_effects: vec![],
                            spliced_card_ids: vec![],
                        };
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
        StackObjectKind::HideawayTrigger {
            source_object,
            hideaway_count,
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
        StackObjectKind::PartnerWithTrigger {
            source_object: _,
            partner_name,
            target_player,
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
        StackObjectKind::IngestTrigger {
            source_object: _,
            target_player,
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
        StackObjectKind::FlankingTrigger {
            source_object: _,
            blocker_id,
        } => {
            let controller = stack_obj.controller;

            // Check if the blocker is still on the battlefield.
            let blocker_alive = state
                .objects
                .get(&blocker_id)
                .map(|obj| obj.zone == ZoneId::Battlefield)
                .unwrap_or(false);

            if blocker_alive {
                // Register the -1/-1 continuous effect (Layer 7c, UntilEndOfTurn).
                let eff_id = state.next_object_id().0;
                let ts = state.timestamp_counter;
                state.timestamp_counter += 1;
                let effect = crate::state::continuous_effect::ContinuousEffect {
                    id: crate::state::continuous_effect::EffectId(eff_id),
                    source: None, // spell/trigger-based effect, not from a permanent
                    timestamp: ts,
                    layer: crate::state::continuous_effect::EffectLayer::PtModify,
                    duration: crate::state::continuous_effect::EffectDuration::UntilEndOfTurn,
                    filter: crate::state::continuous_effect::EffectFilter::SingleObject(blocker_id),
                    modification: crate::state::continuous_effect::LayerModification::ModifyBoth(
                        -1,
                    ),
                    is_cda: false,
                };
                state.continuous_effects.push_back(effect);
            }
            // If blocker left the battlefield, do nothing (CR 400.7).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.23a: Rampage N -- "Whenever this creature becomes blocked, it
        // gets +N/+N until end of turn for each creature blocking it beyond
        // the first."
        // CR 702.23b: Bonus calculated once at resolution time (not trigger time).
        StackObjectKind::RampageTrigger {
            source_object,
            rampage_n,
        } => {
            let controller = stack_obj.controller;

            // Count blockers for this attacker from combat state.
            // CR 702.23b: Snapshot count at resolution; changes after don't matter.
            let blocker_count = state
                .combat
                .as_ref()
                .map(|c| c.blockers_for(source_object).len())
                .unwrap_or(0);

            // CR 702.23a: "for each creature blocking it beyond the first"
            let beyond_first = blocker_count.saturating_sub(1);
            let bonus = (beyond_first as i32) * (rampage_n as i32);

            if bonus > 0 {
                // Only apply if the source is still on the battlefield.
                let source_alive = state
                    .objects
                    .get(&source_object)
                    .map(|obj| obj.zone == ZoneId::Battlefield)
                    .unwrap_or(false);

                if source_alive {
                    // Register the +N/+N continuous effect (Layer 7c, UntilEndOfTurn).
                    // Uses ModifyBoth matching the Flanking pattern (CR 702.45a).
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

        // CR 702.39a: Provoke trigger resolves -- untap the provoked creature
        // and create a forced-block requirement in CombatState.
        //
        // "Whenever this creature attacks, you may have target creature defending
        // player controls untap and block this creature this combat if able."
        // 1. If the provoked creature is no longer on the battlefield, fizzle.
        // 2. Untap the provoked creature (CR 702.39a: "untap that creature").
        // 3. Add a forced-block entry to CombatState::forced_blocks (CR 509.1c).
        StackObjectKind::ProvokeTrigger {
            source_object,
            provoked_creature,
        } => {
            let controller = stack_obj.controller;

            // Target legality: provoked creature must still be on the battlefield.
            let target_valid = state
                .objects
                .get(&provoked_creature)
                .map(|obj| obj.zone == ZoneId::Battlefield)
                .unwrap_or(false);

            if target_valid {
                // 1. Untap the provoked creature (CR 702.39a: "untap that creature").
                if let Some(obj) = state.objects.get(&provoked_creature) {
                    if obj.status.tapped {
                        // Need to clone the controller before borrowing state mutably
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

                // 2. Add forced-block requirement to CombatState (CR 509.1c).
                if let Some(combat) = state.combat.as_mut() {
                    combat
                        .forced_blocks
                        .insert(provoked_creature, source_object);
                }
            }
            // If target invalid, trigger fizzles -- do nothing (CR 608.2b).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }
        // CR 702.112a: Renown trigger resolves -- re-check the intervening-if
        // (CR 603.4) and place N +1/+1 counters on the source creature, then
        // set it as renowned (CR 702.112b).
        //
        // Ruling 2015-06-22: "If a renown ability triggers, but the creature
        // leaves the battlefield before that ability resolves, the creature
        // doesn't become renowned."
        StackObjectKind::RenownTrigger {
            source_object,
            renown_n,
        } => {
            let controller = stack_obj.controller;

            // CR 603.4: Re-check intervening-if at resolution time.
            // Source must still be on the battlefield AND not yet renowned.
            let should_resolve = state
                .objects
                .get(&source_object)
                .map(|obj| obj.zone == ZoneId::Battlefield && !obj.is_renowned)
                .unwrap_or(false);

            if should_resolve {
                // CR 702.112a: Place N +1/+1 counters on the source creature.
                if let Some(obj) = state.objects.get_mut(&source_object) {
                    let current = obj
                        .counters
                        .get(&CounterType::PlusOnePlusOne)
                        .copied()
                        .unwrap_or(0);
                    obj.counters = obj
                        .counters
                        .update(CounterType::PlusOnePlusOne, current + renown_n);
                    // CR 702.112b: Set the renowned designation.
                    obj.is_renowned = true;
                }
            }
            // CR 603.4: Whether the intervening-if passed or failed,
            // the ability always emits AbilityResolved (it "resolves" even if it
            // does nothing because the intervening-if failed at resolution).
            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

        // CR 702.121a: Melee trigger resolves -- count distinct opponents attacked
        // with creatures, then apply +count/+count until end of turn if source is
        // still on the battlefield.
        //
        // Ruling 2016-08-23: "You determine the size of the bonus as the melee
        // ability resolves. Count each opponent that you attacked with one or more
        // creatures."
        // Ruling 2016-08-23: Only opponents (players) count, NOT planeswalkers.
        // Only `AttackTarget::Player(pid)` entries in state.combat.attackers count.
        StackObjectKind::MeleeTrigger { source_object } => {
            let controller = stack_obj.controller;

            // Count distinct opponents attacked with creatures (players only).
            // CR 702.121a: "for each opponent you attacked with a creature"
            // Ruling: "It doesn't matter how many creatures you attacked a player
            // with, only that you attacked a player with at least one creature."
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
                // Only apply if the source is still on the battlefield.
                let source_alive = state
                    .objects
                    .get(&source_object)
                    .map(|obj| obj.zone == ZoneId::Battlefield)
                    .unwrap_or(false);

                if source_alive {
                    // Register the +bonus/+bonus continuous effect (Layer 7c, UntilEndOfTurn).
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

        // CR 702.70a: Poisonous trigger resolves -- give the damaged player
        // N poison counters.
        //
        // CR 603.10: The source creature does NOT need to be on the battlefield
        // at resolution time (the trigger is already on the stack).
        // The poison counters are given regardless of the source's current state.
        //
        // Ruling (Virulent Sliver 2021-03-19): "Poisonous 1 causes the player to
        // get just one poison counter when a Sliver deals combat damage to them,
        // no matter how much damage that Sliver dealt." The N value is fixed.
        StackObjectKind::PoisonousTrigger {
            source_object,
            target_player,
            poisonous_n,
        } => {
            let controller = stack_obj.controller;

            // Give target_player exactly poisonous_n poison counters.
            if let Some(player) = state.players.get_mut(&target_player) {
                player.poison_counters += poisonous_n;
            }

            // Reuse the existing PoisonCountersGiven event from Infect infrastructure.
            // The event semantics are identical: a player received poison counters from
            // a source object. The origin (Poisonous trigger vs. Infect damage
            // replacement) is transparent to downstream consumers.
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
        // CR 702.154a: Enlist trigger resolves -- the enlisting creature gets
        // +X/+0 until end of turn, where X is the tapped creature's power.
        //
        // The +X/+0 is a continuous effect in Layer 7c (PtModify) with
        // UntilEndOfTurn duration. If the source (enlisting) creature has
        // left the battlefield by resolution time (CR 400.7), the trigger
        // does nothing.
        //
        // Power of the enlisted creature: use calculate_characteristics if
        // the creature is still on the battlefield or in any zone. If the
        // object no longer exists at all, use 0.
        StackObjectKind::EnlistTrigger {
            source_object,
            enlisted_creature,
        } => {
            let controller = stack_obj.controller;

            // Check if the source (enlisting) creature is still on the battlefield.
            let source_alive = state
                .objects
                .get(&source_object)
                .map(|obj| obj.zone == ZoneId::Battlefield)
                .unwrap_or(false);

            if source_alive {
                // Read the enlisted creature's power (layer-aware).
                // calculate_characteristics works regardless of zone.
                let enlisted_power =
                    crate::rules::layers::calculate_characteristics(state, enlisted_creature)
                        .and_then(|c| c.power)
                        .unwrap_or(0);

                if enlisted_power != 0 {
                    // Register the +X/+0 continuous effect.
                    let eff_id = state.next_object_id().0;
                    let ts = state.timestamp_counter;
                    state.timestamp_counter += 1;
                    let effect = crate::state::continuous_effect::ContinuousEffect {
                        id: crate::state::continuous_effect::EffectId(eff_id),
                        source: None, // trigger-based effect, not from a permanent
                        timestamp: ts,
                        layer: crate::state::continuous_effect::EffectLayer::PtModify,
                        duration: crate::state::continuous_effect::EffectDuration::UntilEndOfTurn,
                        filter: crate::state::continuous_effect::EffectFilter::SingleObject(
                            source_object,
                        ),
                        modification:
                            crate::state::continuous_effect::LayerModification::ModifyPower(
                                enlisted_power,
                            ),
                        is_cda: false,
                    };
                    state.continuous_effects.push_back(effect);
                }
                // If enlisted_power == 0, still resolve successfully (no buff applied).
            }
            // If source left the battlefield, do nothing (CR 400.7).

            events.push(GameEvent::AbilityResolved {
                controller,
                stack_object_id: stack_obj.id,
            });
        }

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

                // 9. Fire WhenEntersBattlefield triggered effects from card definition.
                //    CR 603.2: ETB triggers on the ninja's own definition must fire.
                let etb_trigger_evts = super::replacement::fire_when_enters_triggered_effects(
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
                    is_bestowed: false,
                    is_foretold: false,
                    foretold_turn: 0,
                    was_unearthed: false,
                    myriad_exile_at_eoc: false,
                    decayed_sacrifice_at_eoc: false,
                    is_suspended: false,
                    exiled_by_hideaway: None,
                    is_renowned: false,
                    encore_sacrifice_at_end_step: false,
                    encore_must_attack: None,
                    encore_activated_by: None,
                    is_plotted: false,
                    plotted_turn: 0,
                    is_prototyped: false,
                    was_bargained: false,
                    echo_pending: false,
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

                // Fire WhenEntersBattlefield triggered effects from card definition.
                let etb_trigger_evts = super::replacement::fire_when_enters_triggered_effects(
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
                    is_bestowed: false,
                    is_foretold: false,
                    foretold_turn: 0,
                    was_unearthed: false,
                    myriad_exile_at_eoc: false,
                    decayed_sacrifice_at_eoc: false,
                    is_suspended: false,
                    exiled_by_hideaway: None,
                    is_renowned: false,
                    encore_sacrifice_at_end_step: false,
                    encore_must_attack: None,
                    encore_activated_by: None,
                    is_plotted: false,
                    plotted_turn: 0,
                    is_prototyped: false,
                    was_bargained: false,
                    echo_pending: false,
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

                // Fire WhenEntersBattlefield triggered effects from card definition.
                let etb_trigger_evts = super::replacement::fire_when_enters_triggered_effects(
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
                        is_bestowed: false,
                        is_foretold: false,
                        foretold_turn: 0,
                        was_unearthed: false,
                        myriad_exile_at_eoc: false,
                        decayed_sacrifice_at_eoc: false,
                        is_suspended: false,
                        exiled_by_hideaway: None,
                        is_renowned: false,
                        encore_sacrifice_at_end_step: true, // sacrificed at end step
                        encore_must_attack: Some(opponent_id), // must attack this opponent
                        // Ruling 2020-11-10: track the original activator so the end-step
                        // sacrifice trigger can verify control hasn't changed.
                        encore_activated_by: Some(controller),
                        is_plotted: false,
                        plotted_turn: 0,
                        is_prototyped: false,
                        was_bargained: false,
                        echo_pending: false,
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

                    // Fire WhenEntersBattlefield triggered effects from card definition.
                    let etb_trigger_evts = super::replacement::fire_when_enters_triggered_effects(
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
        StackObjectKind::EncoreSacrificeTrigger {
            source_object,
            activator,
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
        StackObjectKind::Spell { source_object } => {
            let controller = stack_obj.controller;
            let owner = state.object(source_object)?.owner;
            // CR 702.34a: If cast with flashback, exile instead of graveyard when countered.
            // CR 702.133a: Jump-start also exiles instead of graveyard when countered.
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
        | StackObjectKind::CascadeTrigger { .. }
        | StackObjectKind::StormTrigger { .. }
        | StackObjectKind::EvokeSacrificeTrigger { .. }
        | StackObjectKind::MadnessTrigger { .. }
        | StackObjectKind::MiracleTrigger { .. }
        | StackObjectKind::UnearthAbility { .. }
        | StackObjectKind::UnearthTrigger { .. }
        | StackObjectKind::ExploitTrigger { .. }
        | StackObjectKind::ModularTrigger { .. }
        | StackObjectKind::EvolveTrigger { .. }
        | StackObjectKind::MyriadTrigger { .. }
        | StackObjectKind::SuspendCounterTrigger { .. }
        | StackObjectKind::SuspendCastTrigger { .. }
        | StackObjectKind::HideawayTrigger { .. }
        | StackObjectKind::PartnerWithTrigger { .. }
        | StackObjectKind::IngestTrigger { .. }
        | StackObjectKind::FlankingTrigger { .. }
        | StackObjectKind::RampageTrigger { .. }
        | StackObjectKind::ProvokeTrigger { .. }
        | StackObjectKind::RenownTrigger { .. }
        | StackObjectKind::MeleeTrigger { .. }
        | StackObjectKind::PoisonousTrigger { .. }
        | StackObjectKind::EnlistTrigger { .. }
        | StackObjectKind::NinjutsuAbility { .. }
        | StackObjectKind::EmbalmAbility { .. }
        | StackObjectKind::EternalizeAbility { .. }
        | StackObjectKind::EncoreAbility { .. }
        | StackObjectKind::EncoreSacrificeTrigger { .. }
        | StackObjectKind::DashReturnTrigger { .. }
        | StackObjectKind::BlitzSacrificeTrigger { .. }
        | StackObjectKind::ImpendingCounterTrigger { .. }
        | StackObjectKind::CasualtyTrigger { .. }
        | StackObjectKind::ReplicateTrigger { .. }
        | StackObjectKind::GravestormTrigger { .. }
        | StackObjectKind::VanishingCounterTrigger { .. }
        | StackObjectKind::VanishingSacrificeTrigger { .. }
        | StackObjectKind::FadingTrigger { .. }
        | StackObjectKind::EchoTrigger { .. }
        | StackObjectKind::CumulativeUpkeepTrigger { .. }
        | StackObjectKind::RecoverTrigger { .. }
        | StackObjectKind::ForecastAbility { .. } => {
            // Countering abilities is non-standard; just remove from stack.
            // Note: For ForecastAbility, if countered (e.g. by Stifle), the forecast
            // activation is already consumed (once-per-turn tracked) and the card
            // remains in hand (CR 702.57a).
            // Note: For EchoTrigger, if countered (e.g. by Stifle), echo_pending
            // remains set so the trigger fires again on the next upkeep (CR 702.30a).
            // Note: For CumulativeUpkeepTrigger, if countered (e.g. by Stifle), no
            // age counter is added (counter addition happens at resolution, not queueing).
            // The trigger fires again next upkeep with the same counter count (CR 702.24a).
            // Note: For EncoreAbility, the card is already in exile (exiled as cost
            // during activation). Countering does not return the card (CR 702.141a).
            // Note: For DashReturnTrigger, the creature stays on the battlefield
            // with haste (haste is a static ability, not tied to this trigger -- CR 702.109a).
            // Note: For BlitzSacrificeTrigger, the creature stays on the battlefield
            // with haste and the draw-on-death trigger intact (CR 702.152a).
            // Note: For ImpendingCounterTrigger, if countered (e.g. by Stifle), the
            // permanent retains its time counter(s) and remains a non-creature (CR 702.176a).
            // Note: For CasualtyTrigger, if countered (e.g. by Stifle), the original
            // spell stays on the stack but no copy is made (CR 702.153a).
            // Note: For ReplicateTrigger, if countered (e.g. by Stifle), no copies are
            // made but the original spell stays on the stack (CR 702.56a).
            // Note: For GravestormTrigger, if countered (e.g. by Stifle), no copies are
            // made but the original spell stays on the stack (CR 702.69a).
            // Note: For VanishingCounterTrigger, if countered (e.g. by Stifle), the
            // permanent retains its time counter(s) (CR 702.63a).
            // Note: For VanishingSacrificeTrigger, if countered (e.g. by Stifle), the
            // permanent stays on the battlefield with 0 time counters (CR 702.63a ruling).
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
