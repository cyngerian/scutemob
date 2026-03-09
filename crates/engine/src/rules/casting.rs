//! Casting spells (CR 601).
//!
//! A spell is cast by moving a card from the caster's hand to the Stack zone
//! and placing a `StackObject` onto `GameState::stack_objects`. After casting,
//! the active player receives priority (CR 601.2i).
//!
//! Casting speed (CR 601.3):
//! - **Instant speed**: Instants, and any spell with Flash (CR 702.36), may be
//!   cast whenever the player has priority.
//! - **Sorcery speed**: All other spells may only be cast during the active
//!   player's precombat or postcombat main phase while the stack is empty
//!   (CR 307.1).
//!
//! **Targets (CR 601.2c)**: Targets are announced at cast time and validated
//! for existence. The zone of each object target is recorded for the fizzle
//! rule checked at resolution (CR 608.2b).
//!
//! **Mana cost (CR 601.2f-h)**: If the spell has a mana cost, the caster's
//! mana pool must cover it. The cost is deducted from the pool when the spell
//! is cast. Spells with no mana cost (e.g., `mana_cost: None`) are cast for free.

use im::OrdSet;

use crate::cards::card_definition::{AbilityDefinition, TargetController, TargetRequirement};
use crate::rules::commander::apply_commander_tax;
use crate::rules::layers::calculate_characteristics;
use crate::state::error::GameStateError;
use crate::state::game_object::{Characteristics, Designations, ManaCost, ObjectId};
use crate::state::player::PlayerId;
use crate::state::stack::{StackObject, StackObjectKind, TriggerData};
use crate::state::stubs::PendingTriggerKind;
use crate::state::targeting::{SpellTarget, Target};
use crate::state::turn::Step;
use crate::state::types::{
    AffinityTarget, AltCostKind, CardType, EnchantTarget, KeywordAbility, SubType,
};
use crate::state::zone::ZoneId;
use crate::state::{GameState, PendingTrigger};

use super::events::GameEvent;

/// Handle a CastSpell command: move a card from hand to the stack.
///
/// Validates the casting window, validates targets, pays the mana cost, moves
/// the card to `ZoneId::Stack`, creates a `StackObject`, resets priority to
/// the active player (CR 601.2i), and returns the events produced.
///
/// `convoke_creatures` is a list of creature ObjectIds to tap for convoke cost
/// reduction (CR 702.51). Pass an empty vec for non-convoke spells.
/// `improvise_artifacts` is a list of artifact ObjectIds to tap for improvise cost
/// reduction (CR 702.126). Pass an empty vec for non-improvise spells.
/// `delve_cards` is a list of card ObjectIds in the caster's graveyard to exile for
/// delve cost reduction (CR 702.66). Pass an empty vec for non-delve spells.
#[allow(clippy::too_many_arguments)]
pub fn handle_cast_spell(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
    convoke_creatures: Vec<ObjectId>,
    improvise_artifacts: Vec<ObjectId>,
    delve_cards: Vec<ObjectId>,
    kicker_times: u32,
    alt_cost: Option<AltCostKind>,
    prototype: bool,
    mut modes_chosen: Vec<usize>,
    x_value: u32,
    _face_down_kind: Option<crate::state::types::FaceDownKind>,
    additional_costs: Vec<crate::state::types::AdditionalCost>,
) -> Result<Vec<GameEvent>, GameStateError> {
    // RC-1 Session 3: Extract individual additional-cost values from the consolidated vec.
    // These local variables replace the old individual parameters.
    use crate::state::types::AdditionalCost;

    let retrace_discard_land: Option<ObjectId> = additional_costs.iter().find_map(|c| match c {
        AdditionalCost::Discard(ids) => ids.first().copied(),
        _ => None,
    });
    // Jump-start discard: same Discard variant. Retrace and Jump-Start are mutually exclusive
    // (retrace requires AltCostKind::Retrace, jump-start requires AltCostKind::JumpStart).
    let jump_start_discard: Option<ObjectId> = retrace_discard_land;

    let escape_exile_cards: Vec<ObjectId> = additional_costs
        .iter()
        .find_map(|c| match c {
            AdditionalCost::EscapeExile { cards } => Some(cards.clone()),
            _ => None,
        })
        .unwrap_or_default();
    let collect_evidence_cards: Vec<ObjectId> = additional_costs
        .iter()
        .find_map(|c| match c {
            AdditionalCost::CollectEvidenceExile { cards } => Some(cards.clone()),
            _ => None,
        })
        .unwrap_or_default();

    let (assist_player, assist_amount): (Option<PlayerId>, u32) = additional_costs
        .iter()
        .find_map(|c| match c {
            AdditionalCost::Assist { player, amount } => Some((Some(*player), *amount)),
            _ => None,
        })
        .unwrap_or((None, 0));

    let replicate_count: u32 = additional_costs
        .iter()
        .find_map(|c| match c {
            AdditionalCost::Replicate { count } => Some(*count),
            _ => None,
        })
        .unwrap_or(0);

    let squad_count: u32 = additional_costs
        .iter()
        .find_map(|c| match c {
            AdditionalCost::Squad { count } => Some(*count),
            _ => None,
        })
        .unwrap_or(0);

    let escalate_modes: u32 = additional_costs
        .iter()
        .find_map(|c| match c {
            AdditionalCost::EscalateModes { count } => Some(*count),
            _ => None,
        })
        .unwrap_or(0);

    let splice_cards: Vec<ObjectId> = additional_costs
        .iter()
        .find_map(|c| match c {
            AdditionalCost::Splice { cards } => Some(cards.clone()),
            _ => None,
        })
        .unwrap_or_default();

    let entwine_paid: bool = additional_costs
        .iter()
        .any(|c| matches!(c, AdditionalCost::Entwine));

    let fuse: bool = additional_costs
        .iter()
        .any(|c| matches!(c, AdditionalCost::Fuse));

    let offspring_paid: bool = additional_costs
        .iter()
        .any(|c| matches!(c, AdditionalCost::Offspring));

    let gift_opponent: Option<crate::state::PlayerId> =
        additional_costs.iter().find_map(|c| match c {
            AdditionalCost::Gift { opponent } => Some(*opponent),
            _ => None,
        });

    let (mutate_target, _mutate_on_top): (Option<ObjectId>, bool) = additional_costs
        .iter()
        .find_map(|c| match c {
            AdditionalCost::Mutate { target, on_top } => Some((Some(*target), *on_top)),
            _ => None,
        })
        .unwrap_or((None, false));

    // Derive individual alternative-cost booleans from alt_cost for internal logic.
    let cast_with_evoke = alt_cost == Some(AltCostKind::Evoke);
    let cast_with_mutate = alt_cost == Some(AltCostKind::Mutate);
    let cast_with_bestow = alt_cost == Some(AltCostKind::Bestow);
    let cast_with_miracle = alt_cost == Some(AltCostKind::Miracle);
    let cast_with_escape = alt_cost == Some(AltCostKind::Escape);
    let cast_with_foretell = alt_cost == Some(AltCostKind::Foretell);
    let cast_with_buyback = alt_cost == Some(AltCostKind::Buyback);
    let cast_with_overload = alt_cost == Some(AltCostKind::Overload);
    let cast_with_jump_start = alt_cost == Some(AltCostKind::JumpStart);
    let cast_with_aftermath = alt_cost == Some(AltCostKind::Aftermath);
    let cast_with_dash = alt_cost == Some(AltCostKind::Dash);
    let cast_with_blitz = alt_cost == Some(AltCostKind::Blitz);
    let cast_with_plot = alt_cost == Some(AltCostKind::Plot);
    let cast_with_impending = alt_cost == Some(AltCostKind::Impending);
    let cast_with_emerge = alt_cost == Some(AltCostKind::Emerge);
    let cast_with_spectacle = alt_cost == Some(AltCostKind::Spectacle);
    let cast_with_surge = alt_cost == Some(AltCostKind::Surge);
    let cast_with_cleave = alt_cost == Some(AltCostKind::Cleave);
    let cast_with_disturb = alt_cost == Some(AltCostKind::Disturb);
    let cast_with_morph = alt_cost == Some(AltCostKind::Morph);
    // CR 702.102a: Fuse is a static ability, not an alternative cost. The `fuse` param
    // indicates the player's intent to cast both halves. Validated below.
    let casting_with_fuse = fuse;

    // RC-1: Extract sacrifice ObjectIds from additional_costs.
    // A single helper extracts the first Sacrifice entry.
    // Disambiguation: emerge uses alt_cost == Emerge; bargain/casualty/devour are
    // determined by the spell's keywords (checked at each usage site below).
    let sacrifice_from_additional_costs: Option<ObjectId> = additional_costs.iter().find_map(|c| {
        if let crate::state::types::AdditionalCost::Sacrifice(ids) = c {
            ids.first().copied()
        } else {
            None
        }
    });

    // For emerge, the sacrifice is only valid when alt_cost is Emerge.
    let emerge_sacrifice: Option<ObjectId> = if cast_with_emerge {
        sacrifice_from_additional_costs
    } else {
        None
    };

    // For bargain/casualty, the sacrifice is the same ID (from additional_costs)
    // but only used when the spell has the respective keyword. The validation code
    // below checks the keyword before consuming the sacrifice. Both bargain and
    // casualty reference the same field, but they're mutually exclusive in practice
    // (no card has both Bargain and Casualty keywords).
    let bargain_sacrifice: Option<ObjectId> = if !cast_with_emerge {
        sacrifice_from_additional_costs
    } else {
        None
    };
    let casualty_sacrifice: Option<ObjectId> = if !cast_with_emerge {
        sacrifice_from_additional_costs
    } else {
        None
    };

    // CR 601.2: Casting a spell requires priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // CR 702.61a: If a spell with split second is on the stack, no spells
    // can be cast (mana abilities and special actions are still allowed).
    if has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; no spells can be cast (CR 702.61a)".into(),
        ));
    }

    // Fetch the card and validate it is in the player's hand, command zone, graveyard
    // (with flashback), or exile (with madness).
    // CR 903.8: A player may cast their commander from the command zone.
    // CR 702.34a: A card with flashback may be cast from its owner's graveyard.
    // CR 702.35a: A card with madness may be cast from exile (exiled via discard).
    // CR 702.94a: A card with miracle may be cast from hand (while MiracleTrigger is on stack).
    let (
        casting_from_command_zone,
        casting_from_graveyard,
        casting_from_hand,
        casting_with_flashback,
        casting_with_madness,
        card_has_escape_keyword,
        card_id,
        base_mana_cost,
        casting_with_retrace,
        casting_with_jump_start,
        casting_with_aftermath,
    ) = {
        let card_obj = state.object(card)?;
        let casting_from_command_zone = card_obj.zone == ZoneId::Command(player);
        let casting_from_graveyard = card_obj.zone == ZoneId::Graveyard(player);
        let casting_from_exile = card_obj.zone == ZoneId::Exile;
        let casting_from_hand = card_obj.zone == ZoneId::Hand(player);

        // CR 702.34a: Flashback — allowed if card has the flashback keyword and is in graveyard.
        // Suppress auto-detection when cast_with_escape is true: the player is explicitly
        // choosing escape over flashback, which is legal per CR 118.9a and the ruling for
        // Glimpse of Freedom / Ox of Agonas (2020-01-24): "If a card has multiple abilities
        // giving you permission to cast it... you choose which one to apply."
        // Also suppress when cast_with_jump_start is true: if a card somehow has both Flashback
        // and JumpStart, the player explicitly chose jump-start, so flashback should not activate.
        let casting_with_flashback = casting_from_graveyard
            && card_obj
                .characteristics
                .keywords
                .contains(&KeywordAbility::Flashback)
            && !cast_with_escape
            && !cast_with_jump_start; // CR 702.133a: suppress flashback if player chose jump-start

        // CR 702.138a: Escape — allowed if card has the escape keyword and is in graveyard.
        let card_has_escape_keyword = card_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Escape);
        let casting_with_escape_auto =
            casting_from_graveyard && card_has_escape_keyword && !casting_with_flashback;

        // CR 702.35a: Madness — allowed if card has the madness keyword and is in exile.
        // The card must have been exiled via the madness discard replacement.
        let casting_with_madness = casting_from_exile
            && card_obj
                .characteristics
                .keywords
                .contains(&KeywordAbility::Madness);

        // CR 702.94a: Miracle — validate if cast_with_miracle is true.
        // Card must be in hand with miracle keyword, and a MiracleTrigger for it must be on stack.
        if cast_with_miracle {
            if !casting_from_hand {
                return Err(GameStateError::InvalidCommand(
                    "miracle: card must be in your hand (CR 702.94a)".into(),
                ));
            }
            if !card_obj
                .characteristics
                .keywords
                .contains(&KeywordAbility::Miracle)
            {
                return Err(GameStateError::InvalidCommand(
                    "miracle: card does not have the Miracle keyword (CR 702.94a)".into(),
                ));
            }
            // Verify a MiracleTrigger for this card is on the stack.
            let has_miracle_trigger = state.stack_objects.iter().any(|so| {
                matches!(&so.kind, StackObjectKind::MiracleTrigger { revealed_card, .. }
                    if *revealed_card == card)
            });
            if !has_miracle_trigger {
                return Err(GameStateError::InvalidCommand(
                    "miracle: no MiracleTrigger for this card on the stack (CR 702.94a)".into(),
                ));
            }
        }

        // CR 702.143a: Foretell -- allowed if cast_with_foretell is true.
        // Card must be in ZoneId::Exile with is_foretold == true and foretold on a prior turn.
        if cast_with_foretell {
            if card_obj.zone != ZoneId::Exile {
                return Err(GameStateError::InvalidCommand(
                    "foretell: card must be in exile (CR 702.143a)".into(),
                ));
            }
            if !card_obj.designations.contains(Designations::FORETOLD) {
                return Err(GameStateError::InvalidCommand(
                    "foretell: card was not foretold (CR 702.143a)".into(),
                ));
            }
            if card_obj.foretold_turn >= state.turn.turn_number {
                return Err(GameStateError::InvalidCommand(
                    "foretell: cannot cast foretold card on the same turn it was foretold (CR 702.143a: 'after the current turn has ended')".into(),
                ));
            }
        }

        // CR 702.170d: Plot -- allowed if cast_with_plot is true.
        // Card must be in ZoneId::Exile with is_plotted == true and plotted on a prior turn.
        // CR 702.170d: "during any turn after the turn in which it became plotted"
        if cast_with_plot {
            if card_obj.zone != ZoneId::Exile {
                return Err(GameStateError::InvalidCommand(
                    "plot: card must be in exile (CR 702.170d)".into(),
                ));
            }
            if !card_obj.is_plotted {
                return Err(GameStateError::InvalidCommand(
                    "plot: card was not plotted (CR 702.170d)".into(),
                ));
            }
            if card_obj.plotted_turn >= state.turn.turn_number {
                return Err(GameStateError::InvalidCommand(
                    "plot: cannot cast plotted card on the same turn it was plotted (CR 702.170d: 'any turn after the turn in which it became plotted')".into(),
                ));
            }
        }

        // CR 702.146a: Disturb — allowed if cast_with_disturb is true.
        // Card must be in the player's graveyard and have AbilityDefinition::Disturb.
        // Card must be a DFC (have a back_face in its CardDefinition).
        if cast_with_disturb {
            if !casting_from_graveyard {
                return Err(GameStateError::InvalidCommand(
                    "disturb: card must be in your graveyard (CR 702.146a)".into(),
                ));
            }
            // Verify the card has a Disturb ability definition and is a DFC.
            let has_disturb = card_obj
                .card_id
                .as_ref()
                .and_then(|cid| state.card_registry.get(cid.clone()))
                .map(|def| {
                    def.back_face.is_some()
                        && def.abilities.iter().any(|a| {
                            matches!(
                                a,
                                crate::cards::card_definition::AbilityDefinition::Disturb { .. }
                            )
                        })
                })
                .unwrap_or(false);
            if !has_disturb {
                return Err(GameStateError::InvalidCommand(
                    "disturb: card does not have the Disturb ability or is not a DFC (CR 702.146a)"
                        .into(),
                ));
            }
        }

        // CR 702.81a: Retrace — allowed if card has the Retrace keyword and is in graveyard,
        // AND the player is providing a land card to discard (retrace_discard_land.is_some()).
        // Retrace is an additional cost (CR 118.8), NOT an alternative cost — it does not
        // conflict with Flashback or Escape. However, if the card is being cast via Flashback
        // (alternative cost), Retrace's additional cost is not required for that cast.
        let casting_with_retrace = casting_from_graveyard
            && card_obj
                .characteristics
                .keywords
                .contains(&KeywordAbility::Retrace)
            && !casting_with_flashback // Flashback takes priority (Flashback is alt-cost)
            && retrace_discard_land.is_some(); // Player must signal retrace with a land

        // CR 702.133a: Jump-Start — allowed if card has JumpStart keyword and is in graveyard,
        // AND the player signals intent with cast_with_jump_start: true.
        // Jump-start is NOT an alternative cost — it pays the card's normal mana cost PLUS
        // discards any card from hand as an additional cost (2018-10-05 ruling on Radical Idea).
        // The player must explicitly set cast_with_jump_start: true to use this ability.
        let casting_with_jump_start = cast_with_jump_start
            && casting_from_graveyard
            && card_obj
                .characteristics
                .keywords
                .contains(&KeywordAbility::JumpStart);

        // CR 702.133a: If cast_with_jump_start: true but card doesn't have the keyword, reject.
        if cast_with_jump_start
            && !card_obj
                .characteristics
                .keywords
                .contains(&KeywordAbility::JumpStart)
        {
            return Err(GameStateError::InvalidCommand(
                "jump-start: card does not have the JumpStart keyword (CR 702.133a)".into(),
            ));
        }

        // CR 702.127a: Aftermath — allowed if cast_with_aftermath is true, card has the Aftermath
        // keyword, and the card is in the caster's graveyard.
        let casting_with_aftermath = cast_with_aftermath
            && casting_from_graveyard
            && card_obj
                .characteristics
                .keywords
                .contains(&KeywordAbility::Aftermath);

        // CR 702.127a: If cast_with_aftermath: true but card doesn't have the keyword, reject.
        if cast_with_aftermath
            && !card_obj
                .characteristics
                .keywords
                .contains(&KeywordAbility::Aftermath)
        {
            return Err(GameStateError::InvalidCommand(
                "aftermath: card does not have the Aftermath keyword (CR 702.127a)".into(),
            ));
        }

        // CR 702.127a: Aftermath second half CANNOT be cast from any zone other than graveyard.
        if cast_with_aftermath && !casting_from_graveyard {
            return Err(GameStateError::InvalidCommand(
                "aftermath: the aftermath half can only be cast from your graveyard (CR 702.127a)"
                    .into(),
            ));
        }

        if card_obj.zone != ZoneId::Hand(player)
            && !casting_from_command_zone
            && !casting_with_flashback
            && !casting_with_madness
            && !casting_with_escape_auto
            && !cast_with_escape
            && !cast_with_foretell
            && !cast_with_plot // CR 702.170d: Plot allows exile cast
            && !casting_with_retrace
            && !casting_with_jump_start
            && !casting_with_aftermath
            && !cast_with_disturb
        // CR 702.146a: Disturb allows graveyard cast
        // CR 702.127a: Aftermath allows graveyard cast
        {
            return Err(GameStateError::InvalidCommand(
                "card is not in your hand".into(),
            ));
        }
        (
            casting_from_command_zone,
            casting_from_graveyard,
            casting_from_hand,
            casting_with_flashback,
            casting_with_madness,
            card_has_escape_keyword,
            card_obj.card_id.clone(),
            card_obj.characteristics.mana_cost.clone(),
            casting_with_retrace,
            casting_with_jump_start,
            casting_with_aftermath,
        )
    };

    // CR 903.8: Only a player's own commander may be cast from the command zone.
    if casting_from_command_zone {
        let player_state = state.player(player)?;
        let is_commander = card_id
            .as_ref()
            .map(|cid| player_state.commander_ids.contains(cid))
            .unwrap_or(false);
        if !is_commander {
            return Err(GameStateError::InvalidCommand(
                "only your own commander may be cast from the command zone".into(),
            ));
        }
    }

    // Use calculate_characteristics for type/keyword checks to respect continuous effects
    // (CR 613). Falls back to raw characteristics if the object is not found (command zone
    // objects may not participate in layer calculations).
    let mut chars = calculate_characteristics(state, card).unwrap_or_else(|| {
        state
            .object(card)
            .map(|o| o.characteristics.clone())
            .unwrap_or_default()
    });

    // Lands are not cast — they are played as a special action (CR 305.1).
    if chars.card_types.contains(&CardType::Land) {
        return Err(GameStateError::InvalidCommand(
            "lands are played with PlayLand, not cast".into(),
        ));
    }

    // Determine casting speed (CR 601.3).
    let is_instant_speed = chars.card_types.contains(&CardType::Instant)
        || chars.keywords.contains(&KeywordAbility::Flash);

    // CR 702.34a: Flashback — type validation: only instants and sorceries can use flashback.
    // CR 702.34a: "You may cast this card from your graveyard if the resulting spell is an
    // instant or sorcery spell."
    if casting_with_flashback {
        let is_instant_or_sorcery = chars.card_types.contains(&CardType::Instant)
            || chars.card_types.contains(&CardType::Sorcery);
        if !is_instant_or_sorcery {
            return Err(GameStateError::InvalidCommand(
                "flashback can only be used on instants and sorceries".into(),
            ));
        }
    }

    // CR 702.133a: Jump-start — type validation: only instants and sorceries.
    // CR 702.133a: "You may cast this card from your graveyard if the resulting spell is an
    // instant or sorcery spell."
    if casting_with_jump_start {
        let is_instant_or_sorcery = chars.card_types.contains(&CardType::Instant)
            || chars.card_types.contains(&CardType::Sorcery);
        if !is_instant_or_sorcery {
            return Err(GameStateError::InvalidCommand(
                "jump-start can only be used on instants and sorceries (CR 702.133a)".into(),
            ));
        }
    }

    // CR 702.127a + CR 709.3a: When casting the aftermath half, use the aftermath half's
    // card_type for timing validation instead of the first half's card types.
    // The first half's type may differ (e.g., Cut is Sorcery, Ribbons might be Instant).
    // Recalculate is_instant_speed for aftermath casts.
    let is_instant_speed = if casting_with_aftermath {
        let aftermath_type = get_aftermath_card_type(&card_id, &state.card_registry);
        aftermath_type == Some(CardType::Instant) || chars.keywords.contains(&KeywordAbility::Flash)
    } else {
        is_instant_speed
    };

    // CR 702.102b + CR 709.4d: A fused spell has the combined characteristics of both halves.
    // If either half is an instant, the combined spell can be cast at instant speed.
    let is_instant_speed = if casting_with_fuse {
        let right_type = get_fuse_card_type(&card_id, &state.card_registry);
        is_instant_speed || right_type == Some(CardType::Instant)
    } else {
        is_instant_speed
    };

    // CR 702.74a / CR 702.34a / CR 903.8: Determine the cost to pay.
    // - Evoke: pay the evoke cost (alternative cost, CR 118.9) instead of mana cost.
    //   Cannot combine with flashback (CR 118.9a: only one alternative cost).
    // - Flashback: pay the flashback cost (alternative cost, CR 118.9) instead of mana cost.
    // - Otherwise: pay the printed mana cost.
    // CR 118.9d: additional costs (commander tax) apply ON TOP of any alternative cost.
    // This means casting a commander with evoke from the command zone costs evoke + tax.

    // Step 1: Validate evoke (CR 702.74a / CR 118.9a).
    let casting_with_evoke = if cast_with_evoke {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine evoke with flashback (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if get_evoke_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "spell does not have evoke".into(),
            ));
        }
        // Escape mutual exclusion is handled in Step 1e below.
        true
    } else {
        false
    };

    // Step 1b: Validate bestow (CR 702.103a / CR 118.9a).
    let casting_with_bestow = if cast_with_bestow {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine bestow with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine bestow with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if get_bestow_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "spell does not have bestow".into(),
            ));
        }
        // CR 702.103b: When cast bestowed, the spell becomes an Aura enchantment
        // with enchant creature. Transform chars for target validation.
        chars.card_types.remove(&CardType::Creature);
        chars.card_types.insert(CardType::Enchantment);
        chars.subtypes.insert(SubType("Aura".to_string()));
        chars
            .keywords
            .insert(KeywordAbility::Enchant(EnchantTarget::Creature));
        true
    } else {
        false
    };

    // Step 1c: Validate madness exclusion (CR 601.2b / CR 118.9a).
    // A player can't apply two alternative costs to a single spell. Since madness is an
    // auto-detected alternative cost (from the card's exile zone + keyword), a buggy or
    // malicious client could submit cast_with_evoke/bestow alongside a madness card.
    if casting_with_madness {
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine madness with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine madness with bestow (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        // Note: madness (exile) and flashback (graveyard) are mutually exclusive by zone,
        // but we validate explicitly for defense-in-depth (CR 118.9a).
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine madness with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
    }

    // Step 1d: Validate miracle exclusion (CR 118.9a).
    // Miracle is an alternative cost — cannot combine with other alternative costs.
    if cast_with_miracle {
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine miracle with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine miracle with bestow (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine miracle with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine miracle with madness (CR 118.9a: only one alternative cost)".into(),
            ));
        }
    }

    // Step 1e: Validate escape (CR 702.138a / CR 118.9a).
    // Escape is an alternative cost -- cannot combine with other alternative costs.
    // Auto-detect: if the card is in the graveyard with Escape keyword but no Flashback,
    // treat it as an escape cast (for convenience / backward compat). The explicit
    // cast_with_escape: true flag is also accepted (for disambiguation with flashback).
    let casting_with_escape = if cast_with_escape {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine escape with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine escape with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine escape with bestow (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine escape with madness (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine escape with miracle (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        // Card must be in graveyard with Escape keyword (CR 702.138a).
        if !casting_from_graveyard {
            return Err(GameStateError::InvalidCommand(
                "escape: card must be in your graveyard (CR 702.138a)".into(),
            ));
        }
        if !card_has_escape_keyword {
            return Err(GameStateError::InvalidCommand(
                "escape: card does not have the Escape keyword (CR 702.138a)".into(),
            ));
        }
        true
    } else {
        // Auto-detect: card in graveyard with Escape but no Flashback keyword.
        // If casting_with_flashback is also true, prefer flashback (Flashback wins by position check).
        // This branch handles the common case where the player doesn't set cast_with_escape: true
        // but the card is in the graveyard with only Escape (no Flashback).
        casting_from_graveyard
            && card_has_escape_keyword
            && !casting_with_flashback
            && !casting_with_madness
            && !casting_with_retrace // Player explicitly chose retrace
    };

    // Also add escape to existing mutual exclusion checks for evoke, bestow, madness, miracle:
    if casting_with_escape {
        // (Already validated mutual exclusion above in the cast_with_escape: true branch.)
        // For auto-detected escape, evoke/bestow/madness/miracle checks:
        if !cast_with_escape {
            if casting_with_evoke {
                return Err(GameStateError::InvalidCommand(
                    "cannot combine escape with evoke (CR 118.9a: only one alternative cost)"
                        .into(),
                ));
            }
            if casting_with_bestow {
                return Err(GameStateError::InvalidCommand(
                    "cannot combine escape with bestow (CR 118.9a: only one alternative cost)"
                        .into(),
                ));
            }
            if cast_with_miracle {
                return Err(GameStateError::InvalidCommand(
                    "cannot combine escape with miracle (CR 118.9a: only one alternative cost)"
                        .into(),
                ));
            }
        }
    }

    // Step 1f: Validate foretell mutual exclusion (CR 118.9a).
    // Foretell is an alternative cost -- cannot combine with other alternative costs.
    let casting_with_foretell = if cast_with_foretell {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine foretell with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine foretell with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine foretell with bestow (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine foretell with madness (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine foretell with miracle (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_escape {
            return Err(GameStateError::InvalidCommand(
                "cannot combine foretell with escape (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        true
    } else {
        false
    };

    // Step 1g: Validate overload (CR 702.96a / CR 118.9a).
    // Overload is an alternative cost -- cannot combine with other alternative costs.
    let casting_with_overload = if cast_with_overload {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine overload with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine overload with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine overload with bestow (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine overload with madness (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine overload with miracle (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_escape {
            return Err(GameStateError::InvalidCommand(
                "cannot combine overload with escape (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_foretell {
            return Err(GameStateError::InvalidCommand(
                "cannot combine overload with foretell (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if get_overload_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "spell does not have overload".into(),
            ));
        }
        true
    } else {
        false
    };

    // Step 1h: Validate aftermath mutual exclusion (CR 702.127a / CR 118.9a).
    // Aftermath is an alternative cost -- cannot combine with other alternative costs.
    // Note: aftermath and jump-start are both compatible with each other per CR 118.8 / 118.9a
    // (jump-start is an additional cost, not alternative), but we reject the combination here
    // since aftermath cards don't have jump-start, and this guards against misuse.
    if casting_with_aftermath {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine aftermath with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine aftermath with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine aftermath with bestow (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine aftermath with madness (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine aftermath with miracle (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_escape {
            return Err(GameStateError::InvalidCommand(
                "cannot combine aftermath with escape (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_foretell {
            return Err(GameStateError::InvalidCommand(
                "cannot combine aftermath with foretell (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_overload {
            return Err(GameStateError::InvalidCommand(
                "cannot combine aftermath with overload (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        // Validate the aftermath ability definition exists.
        if get_aftermath_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "aftermath: card has Aftermath keyword but no AbilityDefinition::Aftermath defined"
                    .into(),
            ));
        }
    }

    // CR 702.102a: Fuse validation.
    // Fuse is a static ability (not an alternative cost) that allows casting both halves
    // of a split card from hand, paying the combined mana cost (CR 702.102c).
    if casting_with_fuse {
        // CR 702.102a: Card must have the Fuse keyword.
        if !chars.keywords.contains(&KeywordAbility::Fuse) {
            return Err(GameStateError::InvalidCommand(
                "fuse: card does not have the Fuse keyword (CR 702.102a)".into(),
            ));
        }
        // CR 702.102a: Fuse only applies when casting from hand.
        if !casting_from_hand {
            return Err(GameStateError::InvalidCommand(
                "fuse: can only fuse when casting from hand (CR 702.102a)".into(),
            ));
        }
        // Validate the fuse ability definition exists.
        if get_fuse_data(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "fuse: card has Fuse keyword but no AbilityDefinition::Fuse defined".into(),
            ));
        }
        // CR 702.102a: Fuse requires hand. Most alt costs that change zone are incompatible.
        // Reject combination with any alternative cost for safety.
        if alt_cost.is_some() {
            return Err(GameStateError::InvalidCommand(
                "cannot combine fuse with an alternative cost (CR 702.102a: from hand only)".into(),
            ));
        }
    }

    // Step 1h-mutate: Validate mutate (CR 702.140a / CR 118.9a).
    // Mutate is an alternative cost. The spell must have the Mutate keyword.
    // The mutate_target must be a non-Human creature on the battlefield that the
    // caster owns (same owner as the mutating spell, per CR 702.140a).
    // Mutate can only be applied to creature spells (CR 702.140a: "non-Human creature spell").
    if cast_with_mutate {
        if mutate_target.is_none() {
            return Err(GameStateError::InvalidCommand(
                "mutate: must specify a mutate_target creature (CR 702.140a)".into(),
            ));
        }
        // Card must have the Mutate keyword.
        if !chars.keywords.contains(&KeywordAbility::Mutate) {
            return Err(GameStateError::InvalidCommand(
                "mutate: card does not have the Mutate keyword (CR 702.140a)".into(),
            ));
        }
        // Card must be a creature spell (CR 702.140a: "non-Human creature spell").
        if !chars.card_types.contains(&CardType::Creature) {
            return Err(GameStateError::InvalidCommand(
                "mutate: only creature spells can be cast using the mutate cost (CR 702.140a)"
                    .into(),
            ));
        }
        // Validate the mutate target.
        if let Some(target_id) = mutate_target {
            let target_obj = state.objects.get(&target_id).ok_or_else(|| {
                GameStateError::InvalidCommand("mutate: target creature not found".into())
            })?;
            // CR 702.140a: target must be on the battlefield.
            if target_obj.zone != ZoneId::Battlefield {
                return Err(GameStateError::InvalidCommand(
                    "mutate: target must be on the battlefield (CR 702.140a)".into(),
                ));
            }
            // CR 702.140a: target must be a creature (by layer-resolved characteristics).
            let target_chars = calculate_characteristics(state, target_id)
                .unwrap_or_else(|| target_obj.characteristics.clone());
            if !target_chars.card_types.contains(&CardType::Creature) {
                return Err(GameStateError::InvalidCommand(
                    "mutate: target must be a creature (CR 702.140a)".into(),
                ));
            }
            // CR 702.140a: target must NOT be a Human.
            if target_chars
                .subtypes
                .contains(&SubType("Human".to_string()))
            {
                return Err(GameStateError::InvalidCommand(
                    "mutate: cannot mutate onto a Human creature (CR 702.140a)".into(),
                ));
            }
            // CR 702.140a: target must be owned by the caster ("same owner as this spell").
            if target_obj.owner != player {
                return Err(GameStateError::InvalidCommand(
                    "mutate: target must be owned by you (CR 702.140a)".into(),
                ));
            }
        }
    }

    // Step 1i: Validate dash mutual exclusion (CR 702.109a / CR 118.9a).
    // Dash is an alternative cost -- cannot combine with other alternative costs.
    let casting_with_dash = if cast_with_dash {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with flashback (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with bestow (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with madness (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with miracle (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_escape {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with escape (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_foretell {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with foretell (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_overload {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with overload (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_retrace {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with retrace (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_jump_start {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with jump-start (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_aftermath {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with aftermath (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_impending {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with impending (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_emerge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with emerge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_spectacle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with spectacle (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_surge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine dash with surge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if get_dash_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "spell does not have dash".into(),
            ));
        }
        true
    } else {
        false
    };

    // Step 1j: Validate blitz mutual exclusion (CR 702.152a / CR 118.9a).
    // Blitz is an alternative cost -- cannot combine with other alternative costs.
    let casting_with_blitz = if cast_with_blitz {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with flashback (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with bestow (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with madness (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with miracle (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_escape {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with escape (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_foretell {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with foretell (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_overload {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with overload (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_retrace {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with retrace (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_jump_start {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with jump-start (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_aftermath {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with aftermath (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_dash {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with dash (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_impending {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with impending (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_emerge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with emerge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_spectacle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with spectacle (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_surge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine blitz with surge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if get_blitz_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "spell does not have blitz".into(),
            ));
        }
        true
    } else {
        false
    };

    // Step 1k: Validate plot mutual exclusion (CR 702.170d / CR 118.9a).
    // Plot is an alternative cost -- cannot combine with other alternative costs.
    // Also enforces sorcery-speed timing for the free-cast (CR 702.170d).
    let casting_with_plot = if cast_with_plot {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with flashback (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with bestow (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with madness (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with miracle (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_escape {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with escape (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_foretell {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with foretell (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_overload {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with overload (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_retrace {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with retrace (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_jump_start {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with jump-start (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_aftermath {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with aftermath (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_dash {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with dash (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_blitz {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with blitz (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_impending {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with impending (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_emerge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with emerge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_spectacle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with spectacle (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_surge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine plot with surge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        // CR 702.170d: Plot free-cast timing = main phase + empty stack (sorcery speed).
        // Even instants can only be plot-cast at sorcery speed.
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "plot: plotted cards can only be cast during your turn (CR 702.170d)".into(),
            ));
        }
        if !matches!(state.turn.step, Step::PreCombatMain | Step::PostCombatMain) {
            return Err(GameStateError::InvalidCommand(
                "plot: plotted cards can only be cast during your main phase (CR 702.170d)".into(),
            ));
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "plot: plotted cards can only be cast while the stack is empty (CR 702.170d)"
                    .into(),
            ));
        }
        true
    } else {
        false
    };

    // Step 1m: Validate impending mutual exclusion (CR 702.176a / CR 118.9a).
    // Impending is an alternative cost -- cannot combine with other alternative costs.
    let casting_with_impending = if cast_with_impending {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with bestow (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with madness (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with miracle (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_escape {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with escape (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_foretell {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with foretell (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_overload {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with overload (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_retrace {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with retrace (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_jump_start {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with jump-start (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_aftermath {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with aftermath (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_dash {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with dash (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_blitz {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with blitz (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_plot {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with plot (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_emerge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with emerge (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if cast_with_spectacle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with spectacle (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if cast_with_surge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine impending with surge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if get_impending_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has no impending cost defined (CR 702.176a)".into(),
            ));
        }
        true
    } else {
        false
    };

    // Step 1n: Validate emerge mutual exclusion (CR 702.119a / CR 118.9a).
    // Emerge is an alternative cost -- cannot combine with other alternative costs.
    let casting_with_emerge = if cast_with_emerge {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with bestow (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with madness (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with miracle (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_escape {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with escape (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_foretell {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with foretell (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_overload {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with overload (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_retrace {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with retrace (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_jump_start {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with jump-start (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_aftermath {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with aftermath (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_dash {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with dash (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_blitz {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with blitz (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_plot {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with plot (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_impending {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with impending (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if cast_with_spectacle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with spectacle (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if cast_with_surge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine emerge with surge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if get_emerge_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has no emerge cost defined (CR 702.119a)".into(),
            ));
        }
        true
    } else {
        false
    };

    // CR 702.119a / CR 601.2b,f: Emerge -- validate the sacrifice target and compute
    // the creature's mana value for cost reduction.
    // Emerge is an alternative cost: `alt_cost` must be `Some(AltCostKind::Emerge)`.
    // The sacrifice is mandatory when using emerge; `emerge_sacrifice` must be `Some`.
    // This validation must occur BEFORE base cost selection (emerge_creature_mv is used there).
    let (emerge_sacrifice_id, emerge_creature_mv): (Option<ObjectId>, Option<u32>) =
        if let Some(sac_id) = emerge_sacrifice {
            // Validate the spell is being cast with emerge alt cost.
            if !casting_with_emerge {
                return Err(GameStateError::InvalidCommand(
                    "emerge_sacrifice provided but alt_cost is not Emerge (CR 702.119a)".into(),
                ));
            }
            // Validate the sacrifice target is on the battlefield.
            let (sac_zone, sac_controller) = {
                let sac_obj = state.object(sac_id)?;
                (sac_obj.zone, sac_obj.controller)
            };
            if sac_zone != ZoneId::Battlefield {
                return Err(GameStateError::InvalidCommand(
                    "emerge: sacrifice target must be on the battlefield (CR 702.119a)".into(),
                ));
            }
            if sac_controller != player {
                return Err(GameStateError::InvalidCommand(
                    "emerge: sacrifice target must be controlled by the caster (CR 702.119a)"
                        .into(),
                ));
            }
            // Must be a creature (by layer-resolved characteristics).
            let sac_chars = calculate_characteristics(state, sac_id)
                .or_else(|| {
                    state
                        .objects
                        .get(&sac_id)
                        .map(|o| o.characteristics.clone())
                })
                .unwrap_or_default();
            if !sac_chars.card_types.contains(&CardType::Creature) {
                return Err(GameStateError::InvalidCommand(
                    "emerge: sacrifice target must be a creature (CR 702.119a)".into(),
                ));
            }
            // Compute the creature's mana value for cost reduction (CR 702.119a).
            // MV is derived from the layer-resolved mana cost; tokens and face-down creatures
            // have no mana cost and thus MV = 0 (providing no reduction).
            let mv = sac_chars
                .mana_cost
                .as_ref()
                .map(|mc| mc.mana_value())
                .unwrap_or(0);
            (Some(sac_id), Some(mv))
        } else if casting_with_emerge {
            // Emerge requires a sacrifice — cannot cast with emerge without sacrificing a creature.
            return Err(GameStateError::InvalidCommand(
                "emerge: alt_cost is Emerge but no creature was provided to sacrifice (CR 702.119a)"
                    .into(),
            ));
        } else {
            (None, None)
        };

    // Step 1o: Validate spectacle mutual exclusion and precondition (CR 702.137a / CR 118.9a).
    // Spectacle is an alternative cost -- cannot combine with other alternative costs.
    let casting_with_spectacle = if cast_with_spectacle {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with bestow (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with madness (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with miracle (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_escape {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with escape (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_foretell {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with foretell (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_overload {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with overload (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_retrace {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with retrace (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_jump_start {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with jump-start (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_aftermath {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with aftermath (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_dash {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with dash (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_blitz {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with blitz (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_plot {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with plot (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_impending {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with impending (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_emerge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with emerge (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if cast_with_surge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine spectacle with surge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        // Validate the card has the Spectacle keyword.
        if !chars.keywords.contains(&KeywordAbility::Spectacle) {
            return Err(GameStateError::InvalidCommand(
                "spell does not have spectacle (CR 702.137a)".into(),
            ));
        }
        // Validate the card has a spectacle cost defined.
        if get_spectacle_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has Spectacle keyword but no spectacle cost defined (CR 702.137a)".into(),
            ));
        }
        // CR 702.137a: Validate that an opponent of the caster lost life this turn.
        // CR 800.4a / CR 102.3: Eliminated players (has_lost or has_conceded) are no
        // longer opponents, so their life loss does not enable spectacle.
        let any_opponent_lost_life = state.players.iter().any(|(pid, ps)| {
            *pid != player && !ps.has_lost && !ps.has_conceded && ps.life_lost_this_turn > 0
        });
        if !any_opponent_lost_life {
            return Err(GameStateError::InvalidCommand(
                "spectacle: no opponent has lost life this turn (CR 702.137a)".into(),
            ));
        }
        true
    } else {
        false
    };

    // Step 1p: Validate surge mutual exclusion and precondition (CR 702.117a / CR 118.9a).
    // Surge is an alternative cost -- cannot combine with other alternative costs.
    let casting_with_surge = if cast_with_surge {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with flashback (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with bestow (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with madness (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with miracle (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_escape {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with escape (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_foretell {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with foretell (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_overload {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with overload (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_retrace {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with retrace (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_jump_start {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with jump-start (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_aftermath {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with aftermath (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_dash {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with dash (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_blitz {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with blitz (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_plot {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with plot (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_impending {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with impending (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_emerge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with emerge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_spectacle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine surge with spectacle (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        // Validate the card has the Surge keyword.
        if !chars.keywords.contains(&KeywordAbility::Surge) {
            return Err(GameStateError::InvalidCommand(
                "spell does not have surge (CR 702.117a)".into(),
            ));
        }
        // Validate the card has a surge cost defined.
        if get_surge_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has Surge keyword but no surge cost defined (CR 702.117a)".into(),
            ));
        }
        // CR 702.117a: Validate that the caster has cast another spell this turn.
        // spells_cast_this_turn is incremented AFTER the spell enters the stack,
        // so at this point it reflects spells cast BEFORE this one.
        // >= 1 means at least one other spell was already cast this turn.
        let caster_cast_count = state
            .players
            .get(&player)
            .map(|ps| ps.spells_cast_this_turn)
            .unwrap_or(0);
        if caster_cast_count < 1 {
            return Err(GameStateError::InvalidCommand(
                "surge: you have not cast another spell this turn (CR 702.117a)".into(),
            ));
        }
        true
    } else {
        false
    };

    // Step 1q: Validate cleave mutual exclusion (CR 702.148a / CR 118.9a).
    // Cleave is an alternative cost -- cannot combine with other alternative costs.
    let casting_with_cleave = if cast_with_cleave {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine cleave with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_overload {
            return Err(GameStateError::InvalidCommand(
                "cannot combine cleave with overload (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_surge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine cleave with surge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_spectacle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine cleave with spectacle (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_emerge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine cleave with emerge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        // Validate the card has a cleave cost defined (implies the Cleave ability).
        // CR 702.148a: "Cleave [cost]" -- if the cost isn't defined, the card doesn't
        // have cleave and cannot be cast for its cleave cost.
        if get_cleave_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "spell does not have cleave (CR 702.148a)".into(),
            ));
        }
        true
    } else {
        false
    };

    // Step 1r: Validate disturb mutual exclusion (CR 702.146a / CR 118.9a).
    // Disturb is an alternative cost -- cannot combine with other alternative costs.
    // Disturb also requires the card to be in the graveyard (already checked in zone guard)
    // and to have AbilityDefinition::Disturb (already checked above).
    let casting_with_disturb = if cast_with_disturb {
        if casting_with_flashback {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with flashback (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_evoke {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with evoke (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_bestow {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with bestow (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_madness {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with madness (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if cast_with_miracle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with miracle (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_escape {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with escape (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_foretell {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with foretell (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_overload {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with overload (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_retrace {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with retrace (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_jump_start {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with jump-start (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_aftermath {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with aftermath (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_dash {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with dash (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_blitz {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with blitz (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_plot {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with plot (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_impending {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with impending (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_emerge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with emerge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_spectacle {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with spectacle (CR 118.9a: only one alternative cost)"
                    .into(),
            ));
        }
        if casting_with_surge {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with surge (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        if casting_with_cleave {
            return Err(GameStateError::InvalidCommand(
                "cannot combine disturb with cleave (CR 118.9a: only one alternative cost)".into(),
            ));
        }
        // Validate the card has a disturb cost defined.
        if get_disturb_cost(&card_id, &state.card_registry).is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has Disturb keyword but no disturb cost defined (CR 702.146a)".into(),
            ));
        }
        true
    } else {
        false
    };

    // Step 1l: Validate prototype (CR 702.160a / CR 718.3).
    // Prototype is NOT an alternative cost (CR 118.9, ruling 2022-10-14) -- it can combine
    // with any alternative cost. We validate that the card has AbilityDefinition::Prototype
    // and extract the prototype data for use in Step 2 cost selection.
    let prototype_data: Option<(ManaCost, i32, i32)> = if prototype {
        let data = get_prototype_data(&card_id, &state.card_registry);
        if data.is_none() {
            return Err(GameStateError::InvalidCommand(
                "prototype: card does not have the Prototype ability (CR 702.160a)".into(),
            ));
        }
        data
    } else {
        None
    };

    // Step 2: Select the base cost (alternative cost takes precedence over mana cost).
    // CR 718.3a: When prototype is true, the prototype mana cost REPLACES the card's
    // normal mana cost as the base. The alt-cost chain then operates on this base.
    // For example, "prototype + without paying mana cost" → pay {0} but still prototyped.
    let base_mana_cost = if let Some((ref proto_cost, _, _)) = prototype_data {
        // CR 718.3a: Use only the prototype mana cost when evaluating what can be paid.
        Some(proto_cost.clone())
    } else {
        base_mana_cost
    };

    let base_cost_before_tax: Option<ManaCost> = if casting_with_evoke {
        // CR 702.74a: Pay evoke cost instead of mana cost.
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        get_evoke_cost(&card_id, &state.card_registry)
    } else if casting_with_bestow {
        // CR 702.103a: Pay bestow cost instead of mana cost.
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        get_bestow_cost(&card_id, &state.card_registry)
    } else if casting_with_flashback {
        // CR 702.34a: Pay flashback cost instead of mana cost.
        get_flashback_cost(&card_id, &state.card_registry)
    } else if casting_with_madness {
        // CR 702.35b: Pay madness cost instead of mana cost (alternative cost, CR 118.9).
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        // A None cost here means the card has the Madness keyword but no AbilityDefinition::Madness
        // cost — that is a malformed card definition, not a valid free-cast scenario.
        let cost = get_madness_cost(&card_id, &state.card_registry);
        if cost.is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has Madness keyword but no madness cost defined".into(),
            ));
        }
        cost
    } else if cast_with_miracle {
        // CR 702.94a: Pay miracle cost instead of mana cost (alternative cost, CR 118.9).
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        let cost = get_miracle_cost(&card_id, &state.card_registry);
        if cost.is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has Miracle keyword but no miracle cost defined".into(),
            ));
        }
        cost
    } else if casting_with_escape {
        // CR 702.138a: Pay escape mana cost instead of mana cost (alternative cost, CR 118.9).
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        let cost_opt = get_escape_cost(&card_id, &state.card_registry);
        if cost_opt.is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has Escape keyword but no escape cost defined".into(),
            ));
        }
        cost_opt.map(|(cost, _)| cost)
    } else if casting_with_foretell {
        // CR 702.143a: Pay foretell cost instead of mana cost (alternative cost, CR 118.9).
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        let cost = get_foretell_cost(&card_id, &state.card_registry);
        if cost.is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has Foretell keyword but no foretell cost defined".into(),
            ));
        }
        cost
    } else if casting_with_overload {
        // CR 702.96a: Pay overload cost instead of mana cost (alternative cost, CR 118.9).
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        let cost = get_overload_cost(&card_id, &state.card_registry);
        if cost.is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has Overload keyword but no overload cost defined".into(),
            ));
        }
        cost
    } else if casting_with_aftermath {
        // CR 702.127a: Pay the aftermath half's mana cost (alternative cost, CR 118.9).
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        // The aftermath half's cost is stored in AbilityDefinition::Aftermath { cost, .. }.
        let cost = get_aftermath_cost(&card_id, &state.card_registry);
        if cost.is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has Aftermath keyword but no aftermath cost defined".into(),
            ));
        }
        cost
    } else if casting_with_dash {
        // CR 702.109a: Pay dash cost instead of mana cost.
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        get_dash_cost(&card_id, &state.card_registry)
    } else if casting_with_blitz {
        // CR 702.152a: Pay blitz cost instead of mana cost.
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        get_blitz_cost(&card_id, &state.card_registry)
    } else if casting_with_impending {
        // CR 702.176a: Pay impending cost instead of mana cost.
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        get_impending_cost(&card_id, &state.card_registry)
    } else if casting_with_emerge {
        // CR 702.119a: Pay emerge cost instead of mana cost, reduced by sacrificed creature's MV.
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        let emerge_cost = get_emerge_cost(&card_id, &state.card_registry);
        if let (Some(cost), Some(sac_mv)) = (emerge_cost, emerge_creature_mv) {
            Some(reduce_cost_by_mv(&cost, sac_mv))
        } else {
            return Err(GameStateError::InvalidCommand(
                "emerge: card has Emerge keyword but no emerge cost defined (CR 702.119a)".into(),
            ));
        }
    } else if casting_with_spectacle {
        // CR 702.137a: Pay spectacle cost instead of mana cost.
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        get_spectacle_cost(&card_id, &state.card_registry)
    } else if casting_with_surge {
        // CR 702.117a: Pay surge cost instead of mana cost.
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        get_surge_cost(&card_id, &state.card_registry)
    } else if casting_with_cleave {
        // CR 702.148a: Pay cleave cost instead of mana cost (alternative cost, CR 118.9).
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        get_cleave_cost(&card_id, &state.card_registry)
    } else if cast_with_mutate {
        // CR 702.140a: Pay mutate cost instead of mana cost (alternative cost, CR 118.9).
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        // The mutate cost is stored in AbilityDefinition::MutateCost { cost }.
        let cost = get_mutate_cost(&card_id, &state.card_registry);
        if cost.is_none() {
            return Err(GameStateError::InvalidCommand(
                "mutate: card has Mutate keyword but no MutateCost ability defined (CR 702.140a)"
                    .into(),
            ));
        }
        cost
    } else if casting_with_plot {
        // CR 702.170d: Cast without paying mana cost (alternative cost, CR 118.9).
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        // Cost is zero -- free cast. Additional costs (kicker) still apply.
        Some(ManaCost::default())
    } else if casting_with_disturb {
        // CR 702.146a: Pay disturb cost instead of mana cost (alternative cost, CR 118.9).
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        let cost = get_disturb_cost(&card_id, &state.card_registry);
        if cost.is_none() {
            return Err(GameStateError::InvalidCommand(
                "card has Disturb keyword but no disturb cost defined (CR 702.146a)".into(),
            ));
        }
        cost
    } else if cast_with_morph {
        // CR 702.37c: Cast face-down for {3} instead of the card's mana cost.
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        // The card must have Morph, Megamorph, or Disguise keyword.
        let has_morph = has_morph_keyword(&card_id, &state.card_registry);
        if !has_morph {
            return Err(GameStateError::InvalidCommand(
                "card has no Morph, Megamorph, or Disguise ability (CR 702.37c)".into(),
            ));
        }
        Some(ManaCost {
            generic: 3,
            ..Default::default()
        })
    } else {
        base_mana_cost
    };

    // CR 702.102c: For fused split spells, add the right half's mana cost to the base cost.
    // Fuse is NOT an alternative cost — it pays BOTH halves' mana costs combined.
    // This addition happens after the base cost is selected and before commander tax.
    let base_cost_before_tax = if casting_with_fuse {
        match get_fuse_data(&card_id, &state.card_registry) {
            Some(right_cost) => base_cost_before_tax.map(|left| ManaCost {
                white: left.white + right_cost.white,
                blue: left.blue + right_cost.blue,
                black: left.black + right_cost.black,
                red: left.red + right_cost.red,
                green: left.green + right_cost.green,
                generic: left.generic + right_cost.generic,
                colorless: left.colorless + right_cost.colorless,
            }),
            None => {
                return Err(GameStateError::InvalidCommand(
                    "fuse: card has Fuse keyword but no fuse cost defined".into(),
                ));
            }
        }
    } else {
        base_cost_before_tax
    };

    // Step 3: Apply commander tax ON TOP of the selected base cost (CR 118.9d / CR 903.8).
    let mana_cost: Option<ManaCost> = if casting_from_command_zone {
        let tax = {
            let player_state = state.player(player)?;
            card_id
                .as_ref()
                .and_then(|cid| player_state.commander_tax.get(cid).copied())
                .unwrap_or(0)
        };
        base_cost_before_tax.map(|cost| apply_commander_tax(&cost, tax))
    } else {
        base_cost_before_tax
    };

    // CR 702.33a / 601.2b: If the player declared intention to pay kicker, validate
    // the spell has kicker and add the kicker cost to the total.
    // CR 118.8d: Additional costs don't change the spell's mana cost, only what is paid.
    let (kicker_times_paid, kicker_cost_opt) = if kicker_times > 0 {
        match get_kicker_cost(&card_id, &state.card_registry) {
            Some((kicker_cost, is_multikicker)) => {
                // CR 702.33d: Standard kicker can only be paid once.
                if !is_multikicker && kicker_times > 1 {
                    return Err(GameStateError::InvalidCommand(
                        "standard kicker can only be paid once (CR 702.33d)".into(),
                    ));
                }
                (kicker_times, Some(kicker_cost))
            }
            None => {
                return Err(GameStateError::InvalidCommand(
                    "spell does not have kicker".into(),
                ));
            }
        }
    } else {
        (0, None)
    };

    // CR 601.2f: Add kicker cost(s) to the total mana cost.
    let mana_cost = if let Some(kicker_cost) = kicker_cost_opt {
        let mut total = mana_cost.unwrap_or_default();
        for _ in 0..kicker_times_paid {
            total.white += kicker_cost.white;
            total.blue += kicker_cost.blue;
            total.black += kicker_cost.black;
            total.red += kicker_cost.red;
            total.green += kicker_cost.green;
            total.generic += kicker_cost.generic;
            total.colorless += kicker_cost.colorless;
        }
        Some(total)
    } else {
        mana_cost
    };

    // CR 702.56a / 601.2b / 601.2f-h: If the player declared intention to pay replicate,
    // validate the spell has the Replicate keyword and add the replicate cost N times.
    // CR 118.8d: Additional costs don't change the spell's mana cost, only what is paid.
    let replicate_cost_opt: Option<ManaCost> = if replicate_count > 0 {
        // Validate the spell has the Replicate keyword.
        if !chars.keywords.contains(&KeywordAbility::Replicate) {
            return Err(GameStateError::InvalidCommand(
                "spell does not have replicate (CR 702.56a)".into(),
            ));
        }
        match get_replicate_cost(&card_id, &state.card_registry) {
            Some(cost) => Some(cost),
            None => {
                return Err(GameStateError::InvalidCommand(
                    "spell has replicate keyword but no replicate cost defined".into(),
                ));
            }
        }
    } else {
        None
    };

    // CR 601.2f: Add replicate cost N times to the total mana cost.
    let mana_cost = if let Some(replicate_cost) = replicate_cost_opt {
        let mut total = mana_cost.unwrap_or_default();
        for _ in 0..replicate_count {
            total.white += replicate_cost.white;
            total.blue += replicate_cost.blue;
            total.black += replicate_cost.black;
            total.red += replicate_cost.red;
            total.green += replicate_cost.green;
            total.generic += replicate_cost.generic;
            total.colorless += replicate_cost.colorless;
        }
        Some(total)
    } else {
        mana_cost
    };

    // CR 702.157a / 601.2b / 601.2f-h: Squad -- if the player declared intent to pay the squad
    // cost N times, validate the spell has KeywordAbility::Squad and add the cost N times.
    // CR 118.8d: Additional costs don't change the spell's mana cost, only what is paid.
    let squad_cost_opt: Option<ManaCost> = if squad_count > 0 {
        // Validate the spell has the Squad keyword.
        if !chars.keywords.contains(&KeywordAbility::Squad) {
            return Err(GameStateError::InvalidCommand(
                "spell does not have squad (CR 702.157a)".into(),
            ));
        }
        match get_squad_cost(&card_id, &state.card_registry) {
            Some(cost) => Some(cost),
            None => {
                return Err(GameStateError::InvalidCommand(
                    "spell has squad keyword but no squad cost defined".into(),
                ));
            }
        }
    } else {
        None
    };

    // CR 601.2f: Add squad cost N times to the total mana cost.
    let mana_cost = if let Some(squad_cost) = squad_cost_opt {
        let mut total = mana_cost.unwrap_or_default();
        for _ in 0..squad_count {
            total.white += squad_cost.white;
            total.blue += squad_cost.blue;
            total.black += squad_cost.black;
            total.red += squad_cost.red;
            total.green += squad_cost.green;
            total.generic += squad_cost.generic;
            total.colorless += squad_cost.colorless;
        }
        Some(total)
    } else {
        mana_cost
    };

    // CR 702.175a / 601.2b / 601.2f-h: Offspring -- if the player declared intent to pay the
    // offspring cost, validate the spell has KeywordAbility::Offspring and add the cost once.
    // Binary: paid once or not at all (unlike Squad which can be paid N times).
    // CR 118.8d: Additional costs don't change the spell's mana cost, only what is paid.
    let offspring_cost_opt: Option<ManaCost> = if offspring_paid {
        // Validate the spell has the Offspring keyword.
        if !chars.keywords.contains(&KeywordAbility::Offspring) {
            return Err(GameStateError::InvalidCommand(
                "spell does not have offspring (CR 702.175a)".into(),
            ));
        }
        match get_offspring_cost(&card_id, &state.card_registry) {
            Some(cost) => Some(cost),
            None => {
                return Err(GameStateError::InvalidCommand(
                    "spell has offspring keyword but no offspring cost defined".into(),
                ));
            }
        }
    } else {
        None
    };

    // CR 601.2f: Add offspring cost once to the total mana cost.
    let mana_cost = if let Some(offspring_cost) = offspring_cost_opt {
        let mut total = mana_cost.unwrap_or_default();
        total.white += offspring_cost.white;
        total.blue += offspring_cost.blue;
        total.black += offspring_cost.black;
        total.red += offspring_cost.red;
        total.green += offspring_cost.green;
        total.generic += offspring_cost.generic;
        total.colorless += offspring_cost.colorless;
        Some(total)
    } else {
        mana_cost
    };

    // CR 702.174a / CR 601.2b: Gift -- validate the chosen opponent.
    // Gift is an optional additional cost: the player MAY choose an opponent as an additional
    // cost to cast this spell. Unlike most additional costs, Gift has no mana component --
    // choosing an opponent IS the cost (CR 702.174a). The gift effect fires at resolution.
    let _gift_chosen_opponent: Option<crate::state::PlayerId> =
        if let Some(opponent) = gift_opponent {
            // Validate the spell has Gift keyword.
            if !chars.keywords.contains(&KeywordAbility::Gift) {
                return Err(GameStateError::InvalidCommand(
                    "spell does not have gift (CR 702.174a)".into(),
                ));
            }
            // Validate the chosen player is not the caster (must be an opponent).
            if opponent == player {
                return Err(GameStateError::InvalidCommand(
                    "gift: must choose an opponent, not yourself (CR 702.174a)".into(),
                ));
            }
            // Validate the chosen player is in the game (not eliminated).
            if !state.active_players().contains(&opponent) {
                return Err(GameStateError::InvalidCommand(
                    "gift: chosen opponent is not in the game (CR 702.174a)".into(),
                ));
            }
            Some(opponent)
        } else {
            None
        };

    // CR 702.42a / 601.2b / 601.2f-h: Entwine -- if the player declared intent to pay the entwine
    // cost, validate the spell has KeywordAbility::Entwine and add the entwine cost to the total.
    // CR 118.8d: Additional costs don't change the spell's mana cost, only what is paid.
    let mana_cost = if entwine_paid {
        if !chars.keywords.contains(&KeywordAbility::Entwine) {
            return Err(GameStateError::InvalidCommand(
                "spell does not have entwine (CR 702.42a)".into(),
            ));
        }
        match get_entwine_cost(&card_id, &state.card_registry) {
            Some(entwine_cost) => {
                // CR 601.2f: Add the entwine cost to the total mana cost.
                let mut total = mana_cost.unwrap_or_default();
                total.white += entwine_cost.white;
                total.blue += entwine_cost.blue;
                total.black += entwine_cost.black;
                total.red += entwine_cost.red;
                total.green += entwine_cost.green;
                total.generic += entwine_cost.generic;
                total.colorless += entwine_cost.colorless;
                Some(total)
            }
            None => {
                return Err(GameStateError::InvalidCommand(
                    "spell has entwine keyword but no entwine cost defined".into(),
                ));
            }
        }
    } else {
        mana_cost
    };

    // CR 702.120a / 601.2f-h: Escalate -- if escalate_modes > 0, validate the spell has
    // KeywordAbility::Escalate and add the escalate cost * escalate_modes to the total.
    // CR 118.8d: Additional costs don't change the spell's mana cost, only what is paid.
    let mana_cost = if escalate_modes > 0 {
        if !chars.keywords.contains(&KeywordAbility::Escalate) {
            return Err(GameStateError::InvalidCommand(
                "spell does not have escalate (CR 702.120a)".into(),
            ));
        }
        // CR 702.120a: Escalate is "a static ability of modal spells." Reject if the card
        // definition has no modal structure (modes: None or modes missing from Spell ability).
        let has_modes = card_id.as_ref().and_then(|cid| {
            state.card_registry.get(cid.clone()).and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Spell { modes: Some(m), .. } = a {
                        Some(m.modes.len())
                    } else {
                        None
                    }
                })
            })
        });
        match has_modes {
            None => {
                return Err(GameStateError::InvalidCommand(
                    "escalate requires a modal spell (modes: Some(...)) (CR 702.120a)".into(),
                ));
            }
            Some(0) => {
                return Err(GameStateError::InvalidCommand(
                    "escalate requires a modal spell with at least one mode (CR 702.120a)".into(),
                ));
            }
            _ => {}
        }
        match get_escalate_cost(&card_id, &state.card_registry) {
            Some(escalate_cost) => {
                // CR 601.2f: Add escalate cost × N to the total mana cost.
                let mut total = mana_cost.unwrap_or_default();
                total.white += escalate_cost.white * escalate_modes;
                total.blue += escalate_cost.blue * escalate_modes;
                total.black += escalate_cost.black * escalate_modes;
                total.red += escalate_cost.red * escalate_modes;
                total.green += escalate_cost.green * escalate_modes;
                total.generic += escalate_cost.generic * escalate_modes;
                total.colorless += escalate_cost.colorless * escalate_modes;
                Some(total)
            }
            None => {
                return Err(GameStateError::InvalidCommand(
                    "spell has escalate keyword but no escalate cost defined".into(),
                ));
            }
        }
    } else {
        mana_cost
    };

    // CR 702.172a / 700.2h: Spree -- for each chosen mode, add that mode's per-mode
    // additional cost to the total mana cost. Spree requires at least one mode to be chosen.
    // CR 118.8d: Additional costs don't change the spell's mana value, only what is paid.
    // Note: `modes_chosen` is the raw (not-yet-validated) list; invalid indices return None
    // from `costs.get(idx)` and are safely skipped. Validation happens below at line ~2874.
    let mana_cost = if chars.keywords.contains(&KeywordAbility::Spree) {
        // CR 702.172a: Spree requires at least one mode to be chosen.
        // When entwine_paid is true all modes are chosen; otherwise modes_chosen must be non-empty.
        if !entwine_paid && modes_chosen.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "spree spell requires at least one mode to be chosen (CR 702.172a)".into(),
            ));
        }
        // Look up per-mode costs from ModeSelection.
        let mode_costs = card_id.as_ref().and_then(|cid| {
            state.card_registry.get(cid.clone()).and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Spell { modes: Some(m), .. } = a {
                        m.mode_costs.clone()
                    } else {
                        None
                    }
                })
            })
        });
        match mode_costs {
            Some(costs) => {
                let mut total = mana_cost.unwrap_or_default();
                // Determine which mode indices to charge for.
                // If entwine_paid, charge all modes; otherwise charge only the chosen modes.
                let indices_to_charge: Vec<usize> = if entwine_paid {
                    (0..costs.len()).collect()
                } else {
                    modes_chosen.clone()
                };
                for idx in indices_to_charge {
                    if let Some(cost) = costs.get(idx) {
                        total.white += cost.white;
                        total.blue += cost.blue;
                        total.black += cost.black;
                        total.red += cost.red;
                        total.green += cost.green;
                        total.generic += cost.generic;
                        total.colorless += cost.colorless;
                    }
                }
                Some(total)
            }
            None => {
                return Err(GameStateError::InvalidCommand(
                    "spree spell has no per-mode costs defined in ModeSelection (CR 700.2h)".into(),
                ));
            }
        }
    } else {
        mana_cost
    };

    // CR 702.47a / 601.2b / 601.2f-h: Splice onto [subtype] -- validate and collect splice info.
    // For each splice card declared: verify it's in hand, has Splice keyword, the target spell has
    // the matching subtype, no duplicates, then add the splice cost as an additional cost and
    // collect the effect for attachment to the StackObject.
    // CR 118.8d: Additional costs don't change the spell's mana cost, only what is paid.
    let (mana_cost, collected_spliced_effects, collected_spliced_ids) = if !splice_cards.is_empty()
    {
        // CR 702.47b: Each card may only be spliced onto the same spell once.
        let mut seen_splice_ids = std::collections::HashSet::new();
        let mut splice_effects_out = Vec::new();
        let mut splice_ids_out = Vec::new();
        let mut running_cost = mana_cost;

        for splice_card_id in &splice_cards {
            // Duplicate check (CR 702.47b).
            if !seen_splice_ids.insert(*splice_card_id) {
                return Err(GameStateError::InvalidCommand(format!(
                        "splice: card {:?} appears more than once (CR 702.47b: can't splice the same card twice)",
                        splice_card_id
                    )));
            }

            // CR 702.47a / Ruling: A card cannot be spliced onto itself.
            // "A card with a splice ability can't be spliced onto itself because the spell
            // is on the stack (and not in your hand) when you reveal the cards you want to
            // splice onto it." At this point in casting the card hasn't moved yet, so we
            // explicitly reject if the splice card is the card being cast.
            if *splice_card_id == card {
                return Err(GameStateError::InvalidCommand(format!(
                        "splice: card {:?} cannot be spliced onto itself (CR 702.47a ruling: card must be in hand when spliced)",
                        splice_card_id
                    )));
            }

            // CR 702.47a: The splice card must be in the caster's hand.
            // It cannot be the card being cast (that card is on the stack, not in hand).
            let splice_zone = {
                let splice_obj = state.object(*splice_card_id)?;
                splice_obj.zone
            };
            if splice_zone != ZoneId::Hand(player) {
                return Err(GameStateError::InvalidCommand(format!(
                        "splice: card {:?} is not in caster's hand (CR 702.47a: splice card must be in hand)",
                        splice_card_id
                    )));
            }

            // CR 702.47a: The splice card must have the Splice keyword.
            let splice_card_chars = calculate_characteristics(state, *splice_card_id)
                .unwrap_or_else(|| {
                    state
                        .objects
                        .get(splice_card_id)
                        .map(|o| o.characteristics.clone())
                        .unwrap_or_default()
                });
            if !splice_card_chars.keywords.contains(&KeywordAbility::Splice) {
                return Err(GameStateError::InvalidCommand(format!(
                    "splice: card {:?} does not have the Splice keyword (CR 702.47a)",
                    splice_card_id
                )));
            }

            // Fetch the Splice ability definition from the registry.
            let splice_card_obj = state.objects.get(splice_card_id);
            let splice_def_card_id = splice_card_obj.and_then(|o| o.card_id.clone());
            let splice_info =
                    get_splice_info(&splice_def_card_id, &state.card_registry).ok_or_else(
                        || {
                            GameStateError::InvalidCommand(format!(
                                "splice: card {:?} has Splice keyword but no AbilityDefinition::Splice (missing cost/subtype/effect)",
                                splice_card_id
                            ))
                        },
                    )?;
            let (splice_cost, splice_onto_subtype, splice_effect) = splice_info;

            // CR 702.47a: The spell being cast must have the matching subtype.
            // e.g., Splice onto Arcane requires the target spell to have the Arcane subtype.
            if !chars.subtypes.contains(&splice_onto_subtype) {
                return Err(GameStateError::InvalidCommand(format!(
                    "splice: spell does not have subtype {:?} required for splice (CR 702.47a)",
                    splice_onto_subtype
                )));
            }

            // CR 601.2f-h: Add the splice cost as an additional cost.
            let mut total = running_cost.unwrap_or_default();
            total.white += splice_cost.white;
            total.blue += splice_cost.blue;
            total.black += splice_cost.black;
            total.red += splice_cost.red;
            total.green += splice_cost.green;
            total.generic += splice_cost.generic;
            total.colorless += splice_cost.colorless;
            running_cost = Some(total);

            splice_effects_out.push(splice_effect);
            splice_ids_out.push(*splice_card_id);
        }
        (running_cost, splice_effects_out, splice_ids_out)
    } else {
        (mana_cost, vec![], vec![])
    };

    // CR 702.27a / 601.2f: If the player declared intention to pay buyback, validate
    // the spell has a buyback ability and bind the cost for use below.
    // CR 118.8d: Additional costs don't change the spell's mana cost, only what is paid.
    let buyback_cost_opt: Option<ManaCost> = if cast_with_buyback {
        match get_buyback_cost(&card_id, &state.card_registry) {
            Some(cost) => Some(cost),
            None => {
                return Err(GameStateError::InvalidCommand(
                    "spell does not have buyback".into(),
                ));
            }
        }
    } else {
        None
    };
    let was_buyback_paid = buyback_cost_opt.is_some();

    // CR 601.2f: Add buyback cost to the total mana cost.
    let mana_cost = if let Some(buyback_cost) = buyback_cost_opt {
        let mut total = mana_cost.unwrap_or_default();
        total.white += buyback_cost.white;
        total.blue += buyback_cost.blue;
        total.black += buyback_cost.black;
        total.red += buyback_cost.red;
        total.green += buyback_cost.green;
        total.generic += buyback_cost.generic;
        total.colorless += buyback_cost.colorless;
        Some(total)
    } else {
        mana_cost
    };

    // CR 702.81a / CR 601.2b,f: Retrace — validate and bind the land discard cost.
    // Retrace is an additional cost (CR 118.8): the player pays the card's normal mana
    // cost PLUS discards a land card from hand. The discard is validated here (before the
    // mana cost is paid) and executed after mana payment alongside other cost events.
    // CR 601.2f: Additional costs are paid as part of the total cost payment.
    //
    // Unlike Flashback (exile on departure), Retrace does NOT change where the card goes
    // on resolution — instants/sorceries go to the graveyard normally (CR 702.81a ruling
    // 2008-08-01: "put back into your graveyard").
    let retrace_land_to_discard: Option<ObjectId> = if casting_with_retrace {
        let land_id = retrace_discard_land.ok_or_else(|| {
            GameStateError::InvalidCommand(
                "retrace: internal error -- retrace_discard_land must be Some".into(),
            )
        })?;

        // Validate the land card:
        // 1. Must be in the player's hand (CR 702.81a: "discarding a land card").
        // 2. Must have CardType::Land.
        let (land_owner, land_is_in_hand, land_is_land_type) = {
            let land_obj = state.object(land_id)?;
            (
                land_obj.owner,
                land_obj.zone == ZoneId::Hand(player),
                land_obj
                    .characteristics
                    .card_types
                    .contains(&CardType::Land),
            )
        };
        let _ = land_owner; // owner used in event below
        if !land_is_in_hand {
            return Err(GameStateError::InvalidCommand(
                "retrace: discarded card must be in your hand (CR 702.81a)".into(),
            ));
        }
        if !land_is_land_type {
            return Err(GameStateError::InvalidCommand(
                "retrace: discarded card must be a land card (CR 702.81a)".into(),
            ));
        }
        Some(land_id)
    } else {
        None
    };

    // CR 702.133a / CR 601.2b,f: Jump-start — validate and bind the discard card.
    // Jump-start is an additional cost (CR 601.2f-h): the player pays the card's normal
    // mana cost PLUS discards any card from hand. The discard is validated here (before
    // mana payment) and executed after mana payment.
    // CR 601.2f: Additional costs are paid as part of the total cost payment.
    //
    // Unlike Retrace, the discarded card may be any card type (not just lands).
    // Like Flashback, the jump-start card is exiled on departure (see resolution.rs).
    let jump_start_card_to_discard: Option<ObjectId> = if casting_with_jump_start {
        let discard_id = jump_start_discard.ok_or_else(|| {
            GameStateError::InvalidCommand(
                "jump-start: must provide a card to discard (jump_start_discard must be Some) (CR 702.133a)".into(),
            )
        })?;

        // Validate the discard card:
        // 1. Must be in the player's hand.
        // 2. Can be any card type (any card, not just lands -- CR 702.133a).
        {
            let discard_obj = state.object(discard_id)?;
            if discard_obj.zone != ZoneId::Hand(player) {
                return Err(GameStateError::InvalidCommand(
                    "jump-start: discard card must be in caster's hand (CR 702.133a)".into(),
                ));
            }
        }
        Some(discard_id)
    } else {
        None
    };

    // CR 702.166a / CR 601.2b,f: Bargain -- validate the sacrifice target.
    // Bargain is an optional additional cost: the player MAY sacrifice an artifact,
    // enchantment, or token as an additional cost to cast this spell. The sacrifice
    // is paid during cost payment (CR 601.2h), after mana payment.
    let bargain_sacrifice_id: Option<ObjectId> = if let Some(sac_id) = bargain_sacrifice {
        // RC-1: After consolidation, a Sacrifice in additional_costs may be for
        // casualty or devour, not bargain. Skip silently when the spell lacks
        // the Bargain keyword instead of returning an error.
        if !chars.keywords.contains(&KeywordAbility::Bargain) {
            None
        } else {
            // Validate the sacrifice target is on the battlefield.
            let (sac_zone, sac_controller, sac_is_token) = {
                let sac_obj = state.object(sac_id)?;
                (sac_obj.zone, sac_obj.controller, sac_obj.is_token)
            };
            if sac_zone != ZoneId::Battlefield {
                return Err(GameStateError::InvalidCommand(
                    "bargain: sacrifice target must be on the battlefield (CR 702.166a)".into(),
                ));
            }
            if sac_controller != player {
                return Err(GameStateError::InvalidCommand(
                    "bargain: sacrifice target must be controlled by the caster (CR 702.166a)"
                        .into(),
                ));
            }
            // Must be an artifact, enchantment, or token.
            let sac_chars = calculate_characteristics(state, sac_id)
                .or_else(|| {
                    state
                        .objects
                        .get(&sac_id)
                        .map(|o| o.characteristics.clone())
                })
                .unwrap_or_default();
            let is_artifact = sac_chars.card_types.contains(&CardType::Artifact);
            let is_enchantment = sac_chars.card_types.contains(&CardType::Enchantment);
            if !is_artifact && !is_enchantment && !sac_is_token {
                return Err(GameStateError::InvalidCommand(
                    "bargain: sacrifice target must be an artifact, enchantment, or token (CR 702.166a)".into(),
                ));
            }
            Some(sac_id)
        }
    } else {
        None
    };

    // CR 702.153a / CR 601.2b,f: Casualty -- validate the sacrifice target.
    // Casualty N is an optional additional cost: the player MAY sacrifice a creature
    // with power N or greater as an additional cost to cast this spell.
    // The sacrifice is paid during cost payment (CR 601.2h), after mana payment.
    let casualty_sacrifice_id: Option<ObjectId> = if let Some(sac_id) = casualty_sacrifice {
        // RC-1: After consolidation, a Sacrifice in additional_costs may be for
        // bargain or devour, not casualty. Skip silently when the spell lacks
        // the Casualty keyword.
        let casualty_n = chars.keywords.iter().find_map(|kw| {
            if let KeywordAbility::Casualty(n) = kw {
                Some(*n)
            } else {
                None
            }
        });
        if let Some(casualty_n) = casualty_n {
            // Validate the sacrifice target is on the battlefield.
            let (sac_zone, sac_controller) = {
                let sac_obj = state.object(sac_id)?;
                (sac_obj.zone, sac_obj.controller)
            };
            if sac_zone != ZoneId::Battlefield {
                return Err(GameStateError::InvalidCommand(
                    "casualty: sacrifice target must be on the battlefield (CR 702.153a)".into(),
                ));
            }
            if sac_controller != player {
                return Err(GameStateError::InvalidCommand(
                    "casualty: sacrifice target must be controlled by the caster (CR 702.153a)"
                        .into(),
                ));
            }
            // Must be a creature (by layer-resolved characteristics).
            let sac_chars = calculate_characteristics(state, sac_id)
                .or_else(|| {
                    state
                        .objects
                        .get(&sac_id)
                        .map(|o| o.characteristics.clone())
                })
                .unwrap_or_default();
            if !sac_chars.card_types.contains(&CardType::Creature) {
                return Err(GameStateError::InvalidCommand(
                    "casualty: sacrifice target must be a creature (CR 702.153a)".into(),
                ));
            }
            // Must have power >= N (use layer-resolved power, not raw).
            let sac_power = sac_chars.power.unwrap_or(0);
            if sac_power < casualty_n as i32 {
                return Err(GameStateError::InvalidCommand(format!(
                    "casualty: sacrificed creature power {} is less than required {} (CR 702.153a)",
                    sac_power, casualty_n
                )));
            }
            Some(sac_id)
        } else {
            None
        }
    } else {
        None
    };

    // CR 701.59a / CR 601.2b,f: Collect Evidence -- validate the graveyard cards.
    // Collect Evidence is an additional cost: the player exiles cards from their graveyard
    // with total mana value >= N. Unlike Delve, the exiled cards do NOT reduce mana cost.
    // This validation block determines `evidence_was_collected: bool` used when building
    // the StackObject.
    let evidence_was_collected: bool = if !collect_evidence_cards.is_empty() {
        // 1. Validate the spell has CollectEvidence ability definition.
        let registry = state.card_registry.clone();
        let evidence_threshold =
            card_id
                .clone()
                .and_then(|cid| registry.get(cid))
                .and_then(|def| {
                    def.abilities.iter().find_map(|a| {
                        if let AbilityDefinition::CollectEvidence { threshold, .. } = a {
                            Some(*threshold)
                        } else {
                            None
                        }
                    })
                });
        let evidence_threshold = match evidence_threshold {
            Some(t) => t,
            None => {
                return Err(GameStateError::InvalidCommand(
                    "spell does not have collect evidence (CR 701.59a)".into(),
                ));
            }
        };
        // 2. Validate uniqueness (no duplicate ObjectIds).
        let mut seen_ids = std::collections::HashSet::new();
        for &id in &collect_evidence_cards {
            if !seen_ids.insert(id) {
                return Err(GameStateError::InvalidCommand(
                    "collect evidence: duplicate card ObjectId in exiled list (CR 701.59a)".into(),
                ));
            }
        }
        // 3. Validate each card is in the caster's own graveyard.
        for &id in &collect_evidence_cards {
            let card_zone = state.object(id)?.zone;
            if card_zone != ZoneId::Graveyard(player) {
                return Err(GameStateError::InvalidCommand(
                    "collect evidence: card must be in caster's graveyard (CR 701.59a)".into(),
                ));
            }
        }
        // 4. Sum the mana values of all exiled cards.
        let total_mv: u32 = collect_evidence_cards
            .iter()
            .map(|&id| {
                state
                    .objects
                    .get(&id)
                    .and_then(|o| o.characteristics.mana_cost.as_ref())
                    .map(|mc| mc.mana_value())
                    .unwrap_or(0)
            })
            .sum();
        // 5. Validate total mana value >= threshold (CR 701.59a: "N or greater").
        if total_mv < evidence_threshold {
            return Err(GameStateError::InvalidCommand(format!(
                "collect evidence: total mana value {} is less than required {} (CR 701.59a)",
                total_mv, evidence_threshold
            )));
        }
        true
    } else {
        // Player chose not to collect evidence (optional) OR spell has no collect evidence.
        // If the spell has mandatory collect evidence, reject.
        let registry = state.card_registry.clone();
        let mandatory_evidence = card_id
            .clone()
            .and_then(|cid| registry.get(cid))
            .and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::CollectEvidence { mandatory, .. } = a {
                        Some(*mandatory)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(false);
        if mandatory_evidence {
            return Err(GameStateError::InvalidCommand(
                "collect evidence: this spell requires collect evidence as a mandatory cost (CR 701.59a)".into(),
            ));
        }
        false
    };

    // Validate casting window.
    // CR 702.35 ruling: Madness ignores timing restrictions — a sorcery cast via madness
    // can be cast any time the player has priority, like an instant.
    // CR 702.94a ruling: Miracle ignores timing restrictions — a sorcery cast via miracle
    // can be cast at instant speed (while the miracle trigger is on the stack).
    // CR 702.138a ruling (2020-01-24): Escape does NOT ignore timing restrictions —
    // sorcery-speed cards with escape can only be cast at sorcery speed.
    // CR 702.81a ruling (2008-08-01): Retrace does NOT ignore timing restrictions —
    // sorceries with retrace can only be cast at sorcery speed from the graveyard.
    if !is_instant_speed && !casting_with_madness && !cast_with_miracle {
        // Sorcery speed: active player only, main phase, empty stack (CR 307.1).
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "sorcery-speed spells can only be cast during your own turn".into(),
            ));
        }
        if !matches!(state.turn.step, Step::PreCombatMain | Step::PostCombatMain) {
            return Err(GameStateError::NotMainPhase);
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::StackNotEmpty);
        }
    }

    // Look up target requirements and cant_be_countered from the card definition (CR 601.2c).
    // CR 702.127a + CR 709.3a: When casting the aftermath half, use the aftermath half's
    // target requirements instead of the first half's Spell targets.
    let (requirements, cant_be_countered): (Vec<TargetRequirement>, bool) = {
        let registry = state.card_registry.clone();
        card_id
            .clone()
            .and_then(|cid| registry.get(cid))
            .and_then(|def| {
                if casting_with_aftermath {
                    // Find the Aftermath ability's targets.
                    def.abilities.iter().find_map(|a| {
                        if let AbilityDefinition::Aftermath { targets, .. } = a {
                            Some((targets.clone(), false))
                        } else {
                            None
                        }
                    })
                } else {
                    def.abilities.iter().find_map(|a| {
                        if let AbilityDefinition::Spell {
                            targets,
                            cant_be_countered,
                            ..
                        } = a
                        {
                            Some((targets.clone(), *cant_be_countered))
                        } else {
                            None
                        }
                    })
                }
            })
            .unwrap_or_default()
    };

    // CR 702.96b: When overloaded, the spell has no targets.
    // Override requirements to empty so validate_targets doesn't require targets.
    let requirements = if casting_with_overload {
        // CR 702.96b: Overloaded spells have no targets.
        if !targets.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "overloaded spells have no targets (CR 702.96b)".into(),
            ));
        }
        vec![]
    } else {
        requirements
    };

    // CR 601.2c: Validate and record targets at cast time.
    // Pass source characteristics for protection-from checks (CR 702.16b).
    let spell_targets = validate_targets(state, &targets, &requirements, player, Some(&chars))?;

    // CR 702.5a / 303.4a: Aura spells require exactly one target matching the Enchant restriction.
    // The Enchant keyword defines the target restriction — it is derived from the card's
    // keywords rather than from an explicit TargetRequirement (which applies to instants/sorceries).
    if chars.subtypes.contains(&SubType("Aura".to_string()))
        && chars.card_types.contains(&CardType::Enchantment)
    {
        if let Some(enchant_target) = super::sba::get_enchant_target(&chars.keywords) {
            // CR 303.4a: Aura spell must target exactly one legal object.
            if spell_targets.is_empty() {
                return Err(GameStateError::InvalidCommand(
                    "Aura spells require exactly one target (CR 303.4a)".into(),
                ));
            }
            for st in &spell_targets {
                if let Target::Object(target_id) = st.target {
                    // CR 303.4a / 115.4: The target must be on the battlefield.
                    // Auras can only enchant permanents (or players for Enchant Player).
                    // A creature in the graveyard or hand is not a legal Aura target.
                    let is_on_battlefield = state
                        .objects
                        .get(&target_id)
                        .map(|o| o.zone == ZoneId::Battlefield)
                        .unwrap_or(false);
                    if !is_on_battlefield {
                        return Err(GameStateError::InvalidTarget(
                            "Aura target must be on the battlefield (CR 303.4a)".into(),
                        ));
                    }
                    let target_chars = calculate_characteristics(state, target_id).or_else(|| {
                        state
                            .objects
                            .get(&target_id)
                            .map(|o| o.characteristics.clone())
                    });
                    if let Some(tc) = target_chars {
                        if !super::sba::matches_enchant_target(&enchant_target, &tc) {
                            return Err(GameStateError::InvalidTarget(format!(
                                "target does not match Enchant restriction ({enchant_target:?})"
                            )));
                        }
                    }
                }
            }
        }
    }

    // CR 702.41a / 601.2f: Apply affinity cost reduction AFTER total cost is determined
    // (including commander tax, kicker) and BEFORE convoke/improvise/delve.
    // Affinity is a static ability — the engine counts qualifying permanents automatically.
    // CR 702.41b: Multiple instances of affinity are cumulative.
    let mana_cost = apply_affinity_reduction(state, player, &chars, mana_cost);

    // CR 702.125a: Apply undaunted cost reduction AFTER total cost is determined
    // (including commander tax, kicker, affinity) and BEFORE convoke/improvise/delve.
    // Undaunted is a static ability — the engine counts opponents automatically.
    // CR 702.125c: Multiple instances of undaunted are cumulative.
    let mana_cost = apply_undaunted_reduction(state, player, &chars, mana_cost);

    // CR 702.51a / 702.51b: Apply convoke cost reduction AFTER total cost is determined.
    // Convoke is not an additional or alternative cost — it applies to the total cost.
    // Order: base_mana_cost → commander_tax → kicker → affinity → CONVOKE → improvise → delve → pay.
    let mut convoke_events: Vec<GameEvent> = Vec::new();
    let mana_cost = if !convoke_creatures.is_empty() {
        if !chars.keywords.contains(&KeywordAbility::Convoke) {
            return Err(GameStateError::InvalidCommand(
                "spell does not have convoke".into(),
            ));
        }
        apply_convoke_reduction(
            state,
            player,
            &convoke_creatures,
            mana_cost,
            &mut convoke_events,
        )?
    } else {
        mana_cost
    };

    // CR 702.126a / 702.126b: Apply improvise cost reduction AFTER total cost is determined.
    // Improvise is not an additional or alternative cost — it applies to the total cost.
    // Order: base_mana_cost → commander_tax → flashback → convoke → IMPROVISE → delve → pay.
    let mut improvise_events: Vec<GameEvent> = Vec::new();
    let mana_cost = if !improvise_artifacts.is_empty() {
        if !chars.keywords.contains(&KeywordAbility::Improvise) {
            return Err(GameStateError::InvalidCommand(
                "spell does not have improvise".into(),
            ));
        }
        apply_improvise_reduction(
            state,
            player,
            &improvise_artifacts,
            mana_cost,
            &mut improvise_events,
        )?
    } else {
        mana_cost
    };

    // CR 702.66a / 702.66b: Apply delve cost reduction AFTER total cost is determined.
    // Delve is not an additional or alternative cost — it applies to the total cost.
    // Order: base_mana_cost → commander_tax → flashback → convoke → improvise → DELVE → pay.
    let mut delve_events: Vec<GameEvent> = Vec::new();
    let mana_cost = if !delve_cards.is_empty() {
        if !chars.keywords.contains(&KeywordAbility::Delve) {
            return Err(GameStateError::InvalidCommand(
                "spell does not have delve".into(),
            ));
        }
        apply_delve_reduction(state, player, &delve_cards, mana_cost, &mut delve_events)?
    } else {
        mana_cost
    };

    // CR 702.132a: Apply assist — another player pays generic mana in the total cost.
    // Assist applies AFTER all other cost reductions (convoke, improvise, delve) and
    // BEFORE the caster pays. Pipeline order:
    //   base_mana_cost → alt_cost → commander_tax → kicker → affinity → undaunted →
    //   convoke → improvise → delve → ASSIST → pay.
    let mut assist_events: Vec<GameEvent> = Vec::new();
    let mana_cost = if let Some(assist_pid) = assist_player {
        // Validate: spell must have Assist keyword.
        if !chars.keywords.contains(&KeywordAbility::Assist) {
            return Err(GameStateError::InvalidCommand(
                "spell does not have assist".into(),
            ));
        }
        // Validate: assisting player is not the caster (CR 702.132a: "another player").
        if assist_pid == player {
            return Err(GameStateError::InvalidCommand(
                "cannot assist yourself — must choose another player".into(),
            ));
        }
        // Validate: assisting player is active (not eliminated, CR 800.4a).
        if !state.active_players().contains(&assist_pid) {
            return Err(GameStateError::InvalidCommand(
                "assisting player is not active".into(),
            ));
        }
        // Validate: assist_amount <= generic mana remaining in total cost.
        let generic_remaining = mana_cost.as_ref().map_or(0, |c| c.generic);
        if assist_amount > generic_remaining {
            return Err(GameStateError::InvalidCommand(format!(
                "assist amount {} exceeds generic mana {} in total cost",
                assist_amount, generic_remaining
            )));
        }
        if assist_amount > 0 {
            // Deduct generic mana from assisting player's pool.
            let assist_pool_total = state.player(assist_pid)?.mana_pool.total();
            if assist_pool_total < assist_amount {
                return Err(GameStateError::InsufficientMana);
            }
            let assist_cost = crate::state::game_object::ManaCost {
                generic: assist_amount,
                ..Default::default()
            };
            // can_pay_cost verifies the assisting player can actually pay.
            let assist_pool = &state.player(assist_pid)?.mana_pool;
            if !can_pay_cost(assist_pool, &assist_cost) {
                return Err(GameStateError::InsufficientMana);
            }
            let assist_player_state = state.player_mut(assist_pid)?;
            pay_cost(&mut assist_player_state.mana_pool, &assist_cost);
            assist_events.push(GameEvent::ManaCostPaid {
                player: assist_pid,
                cost: assist_cost,
            });
            // Reduce the generic component the caster still owes.
            let mut reduced = mana_cost.unwrap_or_default();
            reduced.generic = reduced.generic.saturating_sub(assist_amount);
            Some(reduced)
        } else {
            mana_cost
        }
    } else {
        mana_cost
    };

    // CR 702.138a: Validate and exile cards for escape cost (CR 601.2h).
    // The escape exile cards are validated and exiled BEFORE the card moves to the stack
    // (the card hasn't moved yet at this point -- it will move below via move_object_to_zone).
    // Since the card being cast is still in the graveyard at this point, we must ensure
    // the exile list doesn't contain the escape card itself.
    // The exile happens as part of cost payment (CR 601.2h), similar to delve.
    let mut escape_exile_events: Vec<GameEvent> = Vec::new();
    if casting_with_escape {
        let escape_exile_count = get_escape_cost(&card_id, &state.card_registry)
            .map(|(_, count)| count)
            .unwrap_or(0);
        apply_escape_exile_cost(
            state,
            player,
            card,
            &escape_exile_cards,
            escape_exile_count,
            &mut escape_exile_events,
        )?;
    }

    // CR 107.3m: Add the chosen X value to the generic portion of the mana cost.
    // X spells have an implicit variable generic component. The player declares
    // x_value at cast time, which is added to the total cost here.
    // Non-X spells pass x_value = 0 (no change to mana cost).
    let mana_cost = if x_value > 0 {
        mana_cost.map(|mut c| {
            c.generic += x_value;
            c
        })
    } else {
        mana_cost
    };

    // CR 601.2f-h: Pay the mana cost if the card has one.
    let mut events = Vec::new();

    if let Some(ref cost) = mana_cost {
        if cost.mana_value() > 0 {
            // Check the player has enough mana.
            let player_state = state.player_mut(player)?;
            if !can_pay_cost(&player_state.mana_pool, cost) {
                return Err(GameStateError::InsufficientMana);
            }
            pay_cost(&mut player_state.mana_pool, cost);
        }
        // CR 601.2f: ManaCostPaid is emitted for all costs, including {0}.
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: cost.clone(),
        });
    }

    // CR 702.51a / CR 601.2h: Emit PermanentTapped events for convoke creatures.
    // Tapping happens as part of cost payment (CR 601.2h), after the mana payment.
    events.extend(convoke_events);

    // CR 702.126a / CR 601.2h: Emit PermanentTapped events for improvise artifacts.
    // Tapping happens as part of cost payment (CR 601.2h), after the mana payment.
    events.extend(improvise_events);

    // CR 702.132a / CR 601.2h: Emit ManaCostPaid events for the assisting player.
    // Assist payment happens as part of cost payment (CR 601.2h).
    events.extend(assist_events);

    // CR 702.66a / CR 601.2h: Emit ObjectExiled events for delve cards.
    // Exile happens as part of cost payment (CR 601.2h), after the mana payment.
    events.extend(delve_events);

    // CR 702.138a / CR 601.2h: Emit ObjectExiled events for escape exile cards.
    // Exile happens as part of cost payment (CR 601.2h), after the mana payment.
    events.extend(escape_exile_events);

    // CR 702.81a / CR 601.2f: Pay the retrace additional cost — discard a land from hand.
    // The discard is a real discard (CR 118.8 / 701.8): the land goes from hand to the
    // owner's graveyard and triggers any "whenever a player discards a card" effects.
    // This happens as part of cost payment (CR 601.2h), after the mana payment.
    if let Some(land_id) = retrace_land_to_discard {
        let land_owner = state.object(land_id)?.owner;
        let (new_land_id, _) = state.move_object_to_zone(land_id, ZoneId::Graveyard(land_owner))?;
        events.push(GameEvent::CardDiscarded {
            player,
            object_id: land_id,
            new_id: new_land_id,
        });
    }

    // CR 702.133a / CR 601.2f-h: Pay the jump-start additional cost — discard any card from hand.
    // The discard is a real discard (CR 118.8 / 701.8): the card goes from hand to the
    // owner's graveyard and triggers any "whenever a player discards a card" effects.
    // This happens as part of cost payment (CR 601.2h), after the mana payment.
    // CR 702.35a: If the discarded card has Madness, it goes to exile instead of graveyard,
    // and a MadnessTrigger is queued so the player may cast it for its madness cost.
    if let Some(discard_id) = jump_start_card_to_discard {
        let discard_owner = state.object(discard_id)?.owner;
        let discard_card_id_opt = state.object(discard_id)?.card_id.clone();
        let has_madness = state
            .object(discard_id)?
            .characteristics
            .keywords
            .contains(&KeywordAbility::Madness);
        let discard_destination = if has_madness {
            ZoneId::Exile
        } else {
            ZoneId::Graveyard(discard_owner)
        };
        let (new_discard_id, _) = state.move_object_to_zone(discard_id, discard_destination)?;
        // CR 701.8: CardDiscarded is always emitted, even when the card goes to exile via madness.
        events.push(GameEvent::CardDiscarded {
            player,
            object_id: discard_id,
            new_id: new_discard_id,
        });
        // CR 702.35a: Queue MadnessTrigger so the player gets a window to cast at madness cost.
        if has_madness {
            let madness_cost = discard_card_id_opt.as_ref().and_then(|cid| {
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
                source: new_discard_id,
                ability_index: 0,
                controller: player,
                kind: PendingTriggerKind::Madness,
                triggering_event: None,
                entering_object_id: None,
                targeting_stack_id: None,
                triggering_player: None,
                exalted_attacker_id: None,
                defending_player_id: None,
                madness_exiled_card: Some(new_discard_id),
                madness_cost,
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
                graft_entering_creature: None,
                backup_abilities: None,
                backup_n: None,
                champion_filter: None,
                champion_exiled_card: None,
                soulbond_pair_target: None,
                squad_count: None,
                gift_opponent: None,
                cipher_encoded_card_id: None,
                cipher_encoded_object_id: None,
                haunt_source_object_id: None,
                haunt_source_card_id: None,
            });
        }
    }

    // CR 702.166a / CR 601.2f-h: Pay the bargain additional cost -- sacrifice
    // an artifact, enchantment, or token. The sacrifice is a real sacrifice
    // (CR 701.17): the permanent goes from battlefield to the owner's graveyard.
    // This happens as part of cost payment (CR 601.2h), after mana payment.
    // Bargain is optional -- only execute if bargain_sacrifice_id is Some.
    if let Some(sac_id) = bargain_sacrifice_id {
        let sac_owner = state.object(sac_id)?.owner;
        let (new_sac_id, _) = state.move_object_to_zone(sac_id, ZoneId::Graveyard(sac_owner))?;
        events.push(GameEvent::ObjectPutInGraveyard {
            player,
            object_id: sac_id,
            new_grave_id: new_sac_id,
        });
    }

    // CR 702.119a / CR 601.2f-h: Pay the emerge alternative cost -- sacrifice a creature.
    // The sacrifice is part of cost payment (CR 601.2h): the creature goes from the
    // battlefield to the owner's graveyard before the spell goes on the stack.
    // Die triggers fire; if the spell is later countered, the creature is still gone.
    if let Some(sac_id) = emerge_sacrifice_id {
        let sac_owner = state.object(sac_id)?.owner;
        let (new_sac_id, _) = state.move_object_to_zone(sac_id, ZoneId::Graveyard(sac_owner))?;
        events.push(GameEvent::ObjectPutInGraveyard {
            player,
            object_id: sac_id,
            new_grave_id: new_sac_id,
        });
    }

    // CR 702.153a / CR 601.2f-h: Pay the casualty additional cost -- sacrifice a creature
    // with power >= N. The sacrifice is a real sacrifice (CR 701.17): the permanent goes
    // from battlefield to the owner's graveyard.
    // This happens as part of cost payment (CR 601.2h), after mana payment.
    // Casualty is optional -- only execute if casualty_sacrifice_id is Some.
    if let Some(sac_id) = casualty_sacrifice_id {
        let sac_owner = state.object(sac_id)?.owner;
        let (new_sac_id, _) = state.move_object_to_zone(sac_id, ZoneId::Graveyard(sac_owner))?;
        events.push(GameEvent::ObjectPutInGraveyard {
            player,
            object_id: sac_id,
            new_grave_id: new_sac_id,
        });
    }

    // CR 701.59a / CR 601.2f-h: Pay the collect evidence additional cost -- exile
    // cards from the caster's graveyard with total mana value >= N.
    // This happens as part of cost payment (CR 601.2h), after mana payment.
    // The cards go directly to the exile zone (they are not sacrificed -- CR 701.59a).
    // Collect evidence is optional -- only execute if evidence_was_collected is true.
    if evidence_was_collected {
        for &ev_id in &collect_evidence_cards {
            let (new_exile_id, _) = state.move_object_to_zone(ev_id, ZoneId::Exile)?;
            events.push(GameEvent::ObjectExiled {
                player,
                object_id: ev_id,
                new_exile_id,
            });
        }
    }

    // CR 601.2c: Move the card to the Stack zone (CR 400.7: new ObjectId).
    let (new_card_id, _old_obj) = state.move_object_to_zone(card, ZoneId::Stack)?;

    // CR 601.2: Create the StackObject and push it (LIFO — last in, first out).
    let stack_entry_id = state.next_object_id();

    // CR 702.21a: Collect battlefield object targets before moving spell_targets into
    // the stack object. These are used to emit PermanentTargeted events for Ward.
    let battlefield_targets: Vec<ObjectId> = spell_targets
        .iter()
        .filter_map(|st| {
            if let Target::Object(id) = st.target {
                // Only objects that were on the battlefield at cast time trigger ward.
                if matches!(st.zone_at_cast, Some(ZoneId::Battlefield)) {
                    return Some(id);
                }
            }
            None
        })
        .collect();

    // CR 702.82a: Validate devour sacrifices from additional_costs -- each ObjectId must be:
    // - On the battlefield, controlled by the caster
    // - A creature (by current characteristics)
    // - Not duplicated (no ObjectId appears twice)
    // - Not the card being cast (new_card_id is now on the stack, but card is the original)
    //
    // RC-1: Devour sacrifices are extracted from AdditionalCost::Sacrifice in additional_costs.
    // The spell must have the Devour keyword; otherwise the Sacrifice is for bargain/casualty.
    // The actual sacrifice and counter placement happen at resolution (ETB replacement),
    // not here. We validate at cast time for early error detection.
    let devour_sacrifices: Vec<ObjectId> = additional_costs
        .iter()
        .find_map(|c| {
            if let crate::state::types::AdditionalCost::Sacrifice(ids) = c {
                if chars
                    .keywords
                    .iter()
                    .any(|kw| matches!(kw, KeywordAbility::Devour(_)))
                {
                    Some(ids.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or_default();
    let _validated_devour_sacrifices: Vec<ObjectId> = if !devour_sacrifices.is_empty() {
        // Check for duplicates.
        let mut seen = std::collections::HashSet::new();
        for &id in &devour_sacrifices {
            if !seen.insert(id) {
                return Err(GameStateError::InvalidCommand(format!(
                    "duplicate ObjectId {id:?} in devour_sacrifices (CR 702.82a)"
                )));
            }
        }
        // Validate each sacrifice target.
        for &sac_id in &devour_sacrifices {
            let sac_obj = state.objects.get(&sac_id).ok_or_else(|| {
                GameStateError::InvalidCommand(format!(
                    "devour sacrifice target {sac_id:?} does not exist (CR 702.82a)"
                ))
            })?;
            // Must be on the battlefield.
            if sac_obj.zone != ZoneId::Battlefield {
                return Err(GameStateError::InvalidCommand(format!(
                    "devour sacrifice target {sac_id:?} is not on the battlefield (CR 702.82a)"
                )));
            }
            // Must be controlled by the caster.
            if sac_obj.controller != player {
                return Err(GameStateError::InvalidCommand(format!(
                    "devour sacrifice target {sac_id:?} is not controlled by the caster (CR 702.82a)"
                )));
            }
            // Must be a creature.
            let sac_chars = crate::rules::layers::calculate_characteristics(state, sac_id)
                .ok_or_else(|| {
                    GameStateError::InvalidCommand(format!(
                        "could not calculate characteristics for devour target {sac_id:?}"
                    ))
                })?;
            if !sac_chars.card_types.contains(&CardType::Creature) {
                return Err(GameStateError::InvalidCommand(format!(
                    "devour sacrifice target {sac_id:?} is not a creature (CR 702.82a)"
                )));
            }
        }
        devour_sacrifices
    } else {
        vec![]
    };

    // CR 700.2a / 601.2b: Validate explicit mode choices for modal spells.
    // If modes_chosen is non-empty, the spell must be modal and the indices must be valid.
    // When entwine_paid is true, modes_chosen is ignored (all modes are chosen).
    let validated_modes_chosen: Vec<usize> = if !modes_chosen.is_empty() && !entwine_paid {
        // Look up ModeSelection from the card definition.
        let mode_selection = card_id.as_ref().and_then(|cid| {
            state.card_registry.get(cid.clone()).and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Spell { modes: Some(m), .. } = a {
                        Some(m.clone())
                    } else {
                        None
                    }
                })
            })
        });
        match mode_selection {
            None => {
                return Err(GameStateError::InvalidCommand(
                    "modes_chosen specified but spell has no modal structure (modes: Some(...)) (CR 700.2a)".into(),
                ));
            }
            Some(ms) => {
                // CR 700.2a: Each chosen index must be within range.
                for &idx in &modes_chosen {
                    if idx >= ms.modes.len() {
                        return Err(GameStateError::InvalidCommand(format!(
                            "mode index {} is out of range (spell has {} modes) (CR 700.2a)",
                            idx,
                            ms.modes.len()
                        )));
                    }
                }
                // CR 700.2d: Duplicate modes are only allowed when allow_duplicate_modes is set.
                if !ms.allow_duplicate_modes {
                    let mut seen = std::collections::HashSet::new();
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
    } else {
        // Empty = non-modal spell or auto-select mode[0] (backward compatible).
        // Also used when entwine_paid overrides mode selection.
        modes_chosen
    };
    // TODO(CR 700.2c): per-mode targeting — each mode may have its own targets; currently deferred, all targets treated uniformly

    // CR 702.140a / CR 729.2: If casting with the mutate cost, the spell goes on the stack
    // as a MutatingCreatureSpell rather than a plain Spell. The `target` field is the
    // validated non-Human creature the spell will merge with on resolution.
    // The source_object is still the card in the Stack zone (same as normal casts).
    let spell_kind = if cast_with_mutate {
        if let Some(target_id) = mutate_target {
            StackObjectKind::MutatingCreatureSpell {
                source_object: new_card_id,
                target: target_id,
            }
        } else {
            // This branch cannot be reached: mutate validation above already rejected
            // cast_with_mutate with no mutate_target. Guard for compiler completeness.
            StackObjectKind::Spell {
                source_object: new_card_id,
            }
        }
    } else {
        StackObjectKind::Spell {
            source_object: new_card_id,
        }
    };

    let stack_obj = StackObject {
        id: stack_entry_id,
        controller: player,
        kind: spell_kind,
        targets: spell_targets,
        cant_be_countered,
        is_copy: false,
        // CR 702.34a / CR 702.127a: Set exile-on-departure flag for flashback or aftermath casts.
        // Both flashback and aftermath exile the card when it leaves the stack for any reason.
        cast_with_flashback: casting_with_flashback || casting_with_aftermath,
        kicker_times_paid,
        // CR 702.74a: Record whether this spell was cast by paying its evoke cost.
        was_evoked: casting_with_evoke,
        // CR 702.103b: Record whether this spell was cast by paying its bestow cost.
        was_bestowed: casting_with_bestow,
        // CR 702.35a: Record whether this spell was cast via madness from exile.
        cast_with_madness: casting_with_madness,
        // CR 702.94a: Record whether this spell was cast via miracle from hand.
        cast_with_miracle,
        // CR 702.138b: Record whether this spell was cast via escape from graveyard.
        was_escaped: casting_with_escape,
        // CR 702.143a: Record whether this spell was cast via foretell from exile.
        cast_with_foretell: casting_with_foretell,
        // CR 702.27a: Record whether the buyback additional cost was paid.
        was_buyback_paid,
        // CR 702.62a: Not cast via suspend in the normal casting path.
        was_suspended: false,
        // CR 702.96a: Record whether this spell was cast by paying its overload cost.
        was_overloaded: casting_with_overload,
        // CR 702.133a: Record whether this spell was cast via jump-start from graveyard.
        cast_with_jump_start: casting_with_jump_start,
        // CR 702.127a: Record whether this spell was cast as the aftermath half from graveyard.
        cast_with_aftermath: casting_with_aftermath,
        // CR 702.109a: Record whether this spell was cast by paying its dash cost.
        was_dashed: casting_with_dash,
        // CR 702.152a: Record whether this spell was cast by paying its blitz cost.
        was_blitzed: casting_with_blitz,
        // CR 702.170d: Record whether this spell was cast from exile as a plotted card.
        was_plotted: casting_with_plot,
        // CR 718.3b: Record whether this spell was cast as a prototyped spell.
        // Prototype is NOT an alternative cost (CR 118.9, ruling 2022-10-14).
        was_prototyped: prototype,
        // CR 702.176a: Record whether this spell was cast by paying its impending cost.
        was_impended: casting_with_impending,
        // CR 702.166b: Record whether this spell was cast with its bargain cost paid.
        was_bargained: bargain_sacrifice_id.is_some(),
        // CR 702.117a: Record whether this spell was cast by paying its surge cost.
        was_surged: casting_with_surge,
        // CR 702.153a: Record whether this spell was cast with its casualty cost paid
        // (sacrificed a creature with power >= N as an additional cost).
        was_casualty_paid: casualty_sacrifice_id.is_some(),
        // CR 702.148a: Record whether this spell was cast by paying its cleave cost.
        was_cleaved: casting_with_cleave,
        // was_entwined: REMOVED — read from AdditionalCost::Entwine in additional_costs
        // escalate_modes_paid: REMOVED — read from AdditionalCost::EscalateModes in additional_costs
        // CR 702.47a: Spliced effects collected during splice validation above.
        // These are executed after the main spell effect at resolution (CR 702.47b).
        spliced_effects: collected_spliced_effects,
        // CR 702.47a: ObjectIds of cards spliced onto this spell (for display/validation).
        spliced_card_ids: collected_spliced_ids,
        // devour_sacrifices: REMOVED — devour IDs read from additional_costs at resolution
        // CR 700.2a / 601.2b: Mode indices chosen at cast time. Validated below.
        // Empty = auto-select mode[0] (backward compatible).
        modes_chosen: validated_modes_chosen,
        // was_fused: REMOVED — read from AdditionalCost::Fuse in additional_costs
        // CR 107.3m: The value chosen for X in the spell's mana cost. 0 for non-X spells.
        x_value,
        // CR 701.59c: Record whether this spell was cast with collect evidence cost paid.
        // Used by Condition::EvidenceWasCollected at resolution time (linked ability, CR 607).
        evidence_collected: evidence_was_collected,
        // squad_count, offspring_paid, gift_was_given, gift_opponent, mutate_target,
        // mutate_on_top: ALL REMOVED — read from additional_costs at resolution
        // CR 702.146a / CR 712.11a: When cast with disturb, the spell has its back face up.
        // This flag propagates to the permanent when it enters the battlefield.
        is_cast_transformed: cast_with_disturb,
        additional_costs,
    };
    state.stack_objects.push_back(stack_obj);

    // CR 702.103b: When cast bestowed, apply the type transformation to the source
    // object on the stack. The spell becomes an Aura enchantment with enchant creature
    // and loses the Creature type while on the stack.
    if casting_with_bestow {
        if let Some(stack_source) = state.objects.get_mut(&new_card_id) {
            stack_source
                .characteristics
                .card_types
                .remove(&CardType::Creature);
            stack_source
                .characteristics
                .card_types
                .insert(CardType::Enchantment);
            stack_source
                .characteristics
                .subtypes
                .insert(SubType("Aura".to_string()));
            stack_source
                .characteristics
                .keywords
                .insert(KeywordAbility::Enchant(EnchantTarget::Creature));
        }
    }

    // CR 718.3b: When cast as a prototyped spell, apply prototype characteristics to
    // the source object on the stack. The spell uses the alternative P/T, mana cost,
    // and color (derived from prototype mana cost, CR 718.3b / CR 105.2).
    //
    // IMPORTANT: This is NOT an alternative cost (CR 118.9, ruling 2022-10-14).
    // The prototype flag is orthogonal to alt_cost — both can be set simultaneously.
    // CR 718.2a: These values become part of the copiable values while on the stack.
    if let Some((ref proto_cost, proto_power, proto_toughness)) = prototype_data {
        if let Some(stack_source) = state.objects.get_mut(&new_card_id) {
            // CR 718.3b: Set alternative mana cost (replaces the card's normal cost).
            stack_source.characteristics.mana_cost = Some(proto_cost.clone());
            // CR 718.3b: Set alternative P/T.
            stack_source.characteristics.power = Some(proto_power);
            stack_source.characteristics.toughness = Some(proto_toughness);
            // CR 718.3b / CR 105.2: Set color from prototype mana cost symbols.
            stack_source.characteristics.colors = colors_from_mana_cost(proto_cost);
        }
    }

    // CR 702.37c / 702.168b: When cast with morph/megamorph/disguise, set the source object
    // on the stack to face-down. The layer system will make it appear as a 2/2 colorless
    // creature with no name, text, subtypes, or mana cost (CR 708.2a).
    if cast_with_morph {
        use crate::state::types::FaceDownKind;
        let kind = {
            // Determine which face-down kind based on the card's abilities.
            let registry = state.card_registry.clone();
            let def = card_id.as_ref().and_then(|cid| registry.get(cid.clone()));
            if let Some(d) = def {
                if d.abilities
                    .iter()
                    .any(|a| matches!(a, AbilityDefinition::Disguise { .. }))
                {
                    FaceDownKind::Disguise
                } else if d
                    .abilities
                    .iter()
                    .any(|a| matches!(a, AbilityDefinition::Megamorph { .. }))
                {
                    FaceDownKind::Megamorph
                } else {
                    FaceDownKind::Morph
                }
            } else {
                FaceDownKind::Morph
            }
        };
        if let Some(stack_source) = state.objects.get_mut(&new_card_id) {
            stack_source.status.face_down = true;
            stack_source.face_down_as = Some(kind);
        }
    }

    // CR 903.8: Increment commander tax counter if cast from command zone.
    let commander_tax_paid = if casting_from_command_zone {
        if let Some(ref cid) = card_id {
            let player_state = state.player_mut(player)?;
            let count = player_state.commander_tax.entry(cid.clone()).or_insert(0);
            let tax = *count;
            *count += 1;
            tax
        } else {
            0
        }
    } else {
        0
    };

    // CR 601.2i: "Then the active player receives priority."
    // Reset the priority round — a game action occurred.
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(state.turn.active_player);

    events.push(GameEvent::SpellCast {
        player,
        stack_object_id: stack_entry_id,
        source_object_id: new_card_id,
    });

    // CR 702.21a: Emit PermanentTargeted for each battlefield permanent that this
    // spell targets. These events drive Ward trigger checks in check_triggers.
    // `targeting_stack_id` is the stack entry's own ObjectId so the ward CounterSpell
    // effect can locate it via direct stack ID match (so.id == id).
    for target_id in battlefield_targets {
        events.push(GameEvent::PermanentTargeted {
            target_id,
            targeting_stack_id: stack_entry_id,
            targeting_controller: player,
        });
    }

    // CR 702.40a: Track spells cast this turn for storm count.
    // Increment after the spell enters the stack (it is now a spell cast this turn).
    // NOTE: The increment happens here, before the storm trigger is queued, so that
    // `storm_count()` (which uses `spells_cast_this_turn - 1`) yields the correct
    // count of OTHER spells cast before this one. If this ordering changes, storm
    // count would be wrong.
    if let Some(ps) = state.players.get_mut(&player) {
        ps.spells_cast_this_turn += 1;
    }

    // CR 702.40a: Storm — "When you cast this spell, copy it for each other spell
    // cast before it this turn." Storm is a triggered ability (CR 702.40a). It goes
    // on the stack above the original spell and resolves through normal priority.
    // The storm count is captured now (at trigger time) because spells_cast_this_turn
    // could change before the trigger resolves (e.g., cascade casting another spell).
    if chars.keywords.contains(&KeywordAbility::Storm) {
        let count = crate::rules::copy::storm_count(state, player);
        let trigger_id = state.next_object_id();
        let trigger_obj = StackObject {
            id: trigger_id,
            controller: player,
            kind: StackObjectKind::KeywordTrigger {
                source_object: new_card_id,
                keyword: KeywordAbility::Storm,
                data: TriggerData::SpellCopy {
                    original_stack_id: stack_entry_id,
                    copy_count: count,
                },
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
            was_overloaded: false,
            cast_with_jump_start: false,
            cast_with_aftermath: false,
            was_dashed: false,
            was_blitzed: false,
            was_plotted: false,
            was_prototyped: false,
            was_impended: false,
            was_bargained: false,
            was_surged: false,
            was_casualty_paid: false,
            // CR 702.148a: trigger/copy stack objects are not cleave casts.
            was_cleaved: false,
            // CR 702.42a: trigger/copy stack objects are not entwine casts.
            // CR 702.120a: trigger/copy stack objects have no escalate modes paid.
            // CR 702.47a: trigger/copy stack objects have no spliced effects.
            spliced_effects: vec![],
            spliced_card_ids: vec![],
            // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
            modes_chosen: vec![],
            // CR 702.102a: trigger/copy stack objects are never fused spells.
            x_value: 0,
            // CR 701.59c: trigger/copy stack objects are never collect evidence casts.
            evidence_collected: false,
            // CR 702.157a: trigger/copy stack objects have no squad cost payments.
            // CR 702.174a: trigger/copy stack objects are never gift casts.
            is_cast_transformed: false,
            additional_costs: vec![],
        };
        state.stack_objects.push_back(trigger_obj);
        events.push(GameEvent::AbilityTriggered {
            controller: player,
            source_object_id: new_card_id,
            stack_object_id: trigger_id,
        });
    }

    // CR 702.69a: Gravestorm — "When you cast this spell, copy it for each permanent
    // that was put into a graveyard from the battlefield this turn."
    // Gravestorm is a triggered ability (CR 702.69a). It goes on the stack above the
    // original spell and resolves through normal priority.
    // The gravestorm count is captured now (at trigger creation time) because the count
    // could change between cast and resolution (e.g., creatures dying in response).
    // NOTE: Unlike storm, gravestorm count is NOT decremented by 1 — it counts permanents
    // going to graveyards, which is unrelated to this spell being cast.
    if chars.keywords.contains(&KeywordAbility::Gravestorm) {
        let count = state.permanents_put_into_graveyard_this_turn;
        let trigger_id = state.next_object_id();
        let trigger_obj = StackObject {
            id: trigger_id,
            controller: player,
            kind: StackObjectKind::KeywordTrigger {
                source_object: new_card_id,
                keyword: KeywordAbility::Gravestorm,
                data: TriggerData::SpellCopy {
                    original_stack_id: stack_entry_id,
                    copy_count: count,
                },
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
            was_overloaded: false,
            cast_with_jump_start: false,
            cast_with_aftermath: false,
            was_dashed: false,
            was_blitzed: false,
            was_plotted: false,
            was_prototyped: false,
            was_impended: false,
            was_bargained: false,
            was_surged: false,
            was_casualty_paid: false,
            // CR 702.148a: trigger/copy stack objects are not cleave casts.
            was_cleaved: false,
            // CR 702.42a: trigger/copy stack objects are not entwine casts.
            // CR 702.120a: trigger/copy stack objects have no escalate modes paid.
            // CR 702.47a: trigger/copy stack objects have no spliced effects.
            spliced_effects: vec![],
            spliced_card_ids: vec![],
            // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
            modes_chosen: vec![],
            // CR 702.102a: trigger/copy stack objects are never fused spells.
            x_value: 0,
            // CR 701.59c: trigger/copy stack objects are never collect evidence casts.
            evidence_collected: false,
            // CR 702.157a: trigger/copy stack objects have no squad cost payments.
            // CR 702.174a: trigger/copy stack objects are never gift casts.
            is_cast_transformed: false,
            additional_costs: vec![],
        };
        state.stack_objects.push_back(trigger_obj);
        events.push(GameEvent::AbilityTriggered {
            controller: player,
            source_object_id: new_card_id,
            stack_object_id: trigger_id,
        });
    }

    // CR 702.85a: Cascade — "When you cast this spell, exile cards from the top of
    // your library until you exile a nonland card whose mana value is less than this
    // spell's mana value. You may cast that card without paying its mana cost."
    // Cascade is a triggered ability (CR 702.85a). It goes on the stack above the
    // original spell and resolves through normal priority. The mana value is captured
    // now (at trigger time) per CR 702.85a.
    if chars.keywords.contains(&KeywordAbility::Cascade) {
        let spell_mv = chars
            .mana_cost
            .as_ref()
            .map(|mc| mc.mana_value())
            .unwrap_or(0);
        let trigger_id = state.next_object_id();
        let trigger_obj = StackObject {
            id: trigger_id,
            controller: player,
            kind: StackObjectKind::KeywordTrigger {
                source_object: new_card_id,
                keyword: KeywordAbility::Cascade,
                data: TriggerData::CascadeExile {
                    spell_mana_value: spell_mv,
                },
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
            was_overloaded: false,
            cast_with_jump_start: false,
            cast_with_aftermath: false,
            was_dashed: false,
            was_blitzed: false,
            was_plotted: false,
            was_prototyped: false,
            was_impended: false,
            was_bargained: false,
            was_surged: false,
            was_casualty_paid: false,
            // CR 702.148a: trigger/copy stack objects are not cleave casts.
            was_cleaved: false,
            // CR 702.42a: trigger/copy stack objects are not entwine casts.
            // CR 702.120a: trigger/copy stack objects have no escalate modes paid.
            // CR 702.47a: trigger/copy stack objects have no spliced effects.
            spliced_effects: vec![],
            spliced_card_ids: vec![],
            // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
            modes_chosen: vec![],
            // CR 702.102a: trigger/copy stack objects are never fused spells.
            x_value: 0,
            // CR 701.59c: trigger/copy stack objects are never collect evidence casts.
            evidence_collected: false,
            // CR 702.157a: trigger/copy stack objects have no squad cost payments.
            // CR 702.174a: trigger/copy stack objects are never gift casts.
            is_cast_transformed: false,
            additional_costs: vec![],
        };
        state.stack_objects.push_back(trigger_obj);
        events.push(GameEvent::AbilityTriggered {
            controller: player,
            source_object_id: new_card_id,
            stack_object_id: trigger_id,
        });
    }

    // CR 702.153a: Casualty -- "When you cast this spell, if a casualty cost was paid
    // for it, copy it." This is a triggered ability (CR 702.153a). It goes on the stack
    // above the original spell and resolves through normal priority.
    // The copy is NOT cast (ruling 2022-04-29) — it does not trigger "whenever you
    // cast a spell" abilities and does not increment spells_cast_this_turn.
    if casualty_sacrifice_id.is_some() {
        let trigger_id = state.next_object_id();
        let trigger_obj = StackObject {
            id: trigger_id,
            controller: player,
            kind: StackObjectKind::KeywordTrigger {
                source_object: new_card_id,
                keyword: KeywordAbility::Casualty(0),
                data: TriggerData::CasualtyCopy {
                    original_stack_id: stack_entry_id,
                },
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
            was_overloaded: false,
            cast_with_jump_start: false,
            cast_with_aftermath: false,
            was_dashed: false,
            was_blitzed: false,
            was_plotted: false,
            was_prototyped: false,
            was_impended: false,
            was_bargained: false,
            was_surged: false,
            was_casualty_paid: false,
            // CR 702.148a: trigger/copy stack objects are not cleave casts.
            was_cleaved: false,
            // CR 702.42a: trigger/copy stack objects are not entwine casts.
            // CR 702.120a: trigger/copy stack objects have no escalate modes paid.
            // CR 702.47a: trigger/copy stack objects have no spliced effects.
            spliced_effects: vec![],
            spliced_card_ids: vec![],
            // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
            modes_chosen: vec![],
            // CR 702.102a: trigger/copy stack objects are never fused spells.
            x_value: 0,
            // CR 701.59c: trigger/copy stack objects are never collect evidence casts.
            evidence_collected: false,
            // CR 702.157a: trigger/copy stack objects have no squad cost payments.
            // CR 702.174a: trigger/copy stack objects are never gift casts.
            is_cast_transformed: false,
            additional_costs: vec![],
        };
        state.stack_objects.push_back(trigger_obj);
        events.push(GameEvent::AbilityTriggered {
            controller: player,
            source_object_id: new_card_id,
            stack_object_id: trigger_id,
        });
    }

    // CR 702.56a: Replicate -- "When you cast this spell, if a replicate cost was paid
    // for it, copy it for each time its replicate cost was paid." This is a triggered
    // ability (CR 702.56a). It goes on the stack above the original spell and resolves
    // through normal priority.
    // Copies are NOT cast (ruling 2024-01-12 for Shattering Spree) — they do not trigger
    // "whenever you cast a spell" abilities and do not increment spells_cast_this_turn.
    if replicate_count > 0 {
        let trigger_id = state.next_object_id();
        let trigger_obj = StackObject {
            id: trigger_id,
            controller: player,
            kind: StackObjectKind::KeywordTrigger {
                source_object: new_card_id,
                keyword: KeywordAbility::Replicate,
                data: TriggerData::SpellCopy {
                    original_stack_id: stack_entry_id,
                    copy_count: replicate_count,
                },
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
            was_overloaded: false,
            cast_with_jump_start: false,
            cast_with_aftermath: false,
            was_dashed: false,
            was_blitzed: false,
            was_plotted: false,
            was_prototyped: false,
            was_impended: false,
            was_bargained: false,
            was_surged: false,
            was_casualty_paid: false,
            // CR 702.148a: trigger/copy stack objects are not cleave casts.
            was_cleaved: false,
            // CR 702.42a: trigger/copy stack objects are not entwine casts.
            // CR 702.120a: trigger/copy stack objects have no escalate modes paid.
            // CR 702.47a: trigger/copy stack objects have no spliced effects.
            spliced_effects: vec![],
            spliced_card_ids: vec![],
            // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
            modes_chosen: vec![],
            // CR 702.102a: trigger/copy stack objects are never fused spells.
            x_value: 0,
            // CR 701.59c: trigger/copy stack objects are never collect evidence casts.
            evidence_collected: false,
            // CR 702.157a: trigger/copy stack objects have no squad cost payments.
            // CR 702.174a: trigger/copy stack objects are never gift casts.
            is_cast_transformed: false,
            additional_costs: vec![],
        };
        state.stack_objects.push_back(trigger_obj);
        events.push(GameEvent::AbilityTriggered {
            controller: player,
            source_object_id: new_card_id,
            stack_object_id: trigger_id,
        });
    }

    // CR 903.8: Emit commander-specific event when casting from command zone.
    if casting_from_command_zone {
        if let Some(cid) = card_id {
            events.push(GameEvent::CommanderCastFromCommandZone {
                player,
                card_id: cid,
                tax_paid: commander_tax_paid,
            });
        }
    }

    events.push(GameEvent::PriorityGiven {
        player: state.turn.active_player,
    });

    Ok(events)
}

/// CR 702.34a / CR 118.9: Look up the flashback cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::AltCastAbility { kind: AltCostKind::Flashback, .. }`,
/// or `None` if the card has no definition or no flashback ability defined.
fn get_flashback_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility { kind: AltCostKind::Flashback, cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.33a: Look up the kicker cost from the card's `AbilityDefinition`.
///
/// Returns `Some((ManaCost, is_multikicker))` if the card has a kicker/multikicker
/// ability, or `None` if it has no kicker.
fn get_kicker_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<(ManaCost, bool)> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Kicker {
                    cost,
                    is_multikicker,
                } = a
                {
                    Some((cost.clone(), *is_multikicker))
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.56a: Look up the replicate cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Replicate { cost }`, or `None`
/// if the card has no definition or no replicate ability defined.
fn get_replicate_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Replicate { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.42a: Look up the entwine cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Entwine { cost }`, or `None`
/// if the card has no definition or no entwine ability defined.
fn get_entwine_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Entwine { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.120a: Look up the escalate cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Escalate { cost }`, or `None`
/// if the card has no definition or no escalate ability defined.
fn get_escalate_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Escalate { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.27a: Look up the buyback cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Buyback { cost }`, or `None`
/// if the card has no definition or no buyback ability defined.
fn get_buyback_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Buyback { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.74a: Look up the evoke cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Evoke { cost }`, or `None`
/// if the card has no definition or no evoke ability defined.
fn get_evoke_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Evoke { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.103a / CR 118.9: Look up the bestow cost from the card's `AbilityDefinition`.
fn get_bestow_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Bestow { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.35b / CR 118.9: Look up the madness cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Madness { cost }`, or `None`
/// if the card has no definition or no madness ability defined. When `None` is returned,
/// no mana payment is required (free madness — rare but correct per CR 118.9).
fn get_madness_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Madness { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.94a / CR 118.9: Look up the miracle cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Miracle { cost }`, or `None`
/// if the card has no definition or no miracle ability defined.
fn get_miracle_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Miracle { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.138a / CR 118.9: Look up the escape cost from the card's `AbilityDefinition`.
///
/// Returns `Some((ManaCost, exile_count))` from `AbilityDefinition::AltCastAbility { kind: AltCostKind::Escape, .. }`,
/// or `None` if the card has no escape ability definition.
fn get_escape_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<(ManaCost, u32)> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility {
                    kind: AltCostKind::Escape,
                    cost,
                    details: Some(crate::cards::card_definition::AltCastDetails::Escape { exile_count }),
                } = a
                {
                    Some((cost.clone(), *exile_count))
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.143a / CR 118.9: Look up the foretell cost from the card's `AbilityDefinition`.
///
/// Returns `Some(ManaCost)` from `AbilityDefinition::Foretell { cost }`,
/// or `None` if the card has no foretell ability definition.
fn get_foretell_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Foretell { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.127a / CR 118.9: Look up the aftermath cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Aftermath { cost, .. }`, or `None`
/// if the card has no definition or no aftermath ability defined.
fn get_aftermath_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Aftermath { cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.127a + CR 709.3a: Look up the aftermath half's card type.
///
/// When casting the aftermath half, its card_type is used for timing validation
/// (instant vs sorcery speed) instead of the first half's card types.
///
/// Returns `Some(CardType)` from `AbilityDefinition::Aftermath { card_type, .. }`,
/// or `None` if the card has no aftermath ability definition.
fn get_aftermath_card_type(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<CardType> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Aftermath { card_type, .. } = a {
                    Some(*card_type)
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.152a: Look up the blitz cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::AltCastAbility { kind: AltCostKind::Blitz, .. }`,
/// or `None` if the card has no definition or no blitz ability defined.
fn get_blitz_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility { kind: AltCostKind::Blitz, cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.109a: Look up the dash cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::AltCastAbility { kind: AltCostKind::Dash, .. }`,
/// or `None` if the card has no definition or no dash ability defined.
fn get_dash_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility { kind: AltCostKind::Dash, cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.96a: Look up the overload cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Overload { cost }`, or `None`
/// if the card has no definition or no overload ability defined.
fn get_overload_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Overload { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.138a: Validate and exile cards for escape cost payment.
///
/// Unlike delve (which reduces generic mana), escape's exile is a FIXED count
/// that is part of the alternative cost. The player must exile exactly
/// `required_count` OTHER cards from their graveyard (not the escape card itself).
///
/// Validation:
/// - Count must match exactly `required_count`.
/// - No duplicates in the exile list.
/// - Each card must be in the caster's graveyard.
/// - Each card must NOT be the escape card itself (`escape_card_id`).
///
/// On success, moves each card to exile and emits `ObjectExiled` events.
fn apply_escape_exile_cost(
    state: &mut GameState,
    player: PlayerId,
    escape_card_id: ObjectId,
    escape_cards: &[ObjectId],
    required_count: u32,
    events: &mut Vec<GameEvent>,
) -> Result<(), GameStateError> {
    if escape_cards.len() as u32 != required_count {
        return Err(GameStateError::InvalidCommand(format!(
            "escape requires exactly {} cards to exile, but {} were provided",
            required_count,
            escape_cards.len()
        )));
    }
    // Validate uniqueness.
    let mut seen = std::collections::HashSet::new();
    for &id in escape_cards {
        if !seen.insert(id) {
            return Err(GameStateError::InvalidCommand(format!(
                "duplicate card {:?} in escape_exile_cards",
                id
            )));
        }
    }
    // Validate each card is in caster's graveyard and is not the escape card itself.
    for &id in escape_cards {
        if id == escape_card_id {
            return Err(GameStateError::InvalidCommand(
                "escape: cannot exile the card being cast as part of its own escape cost (CR 702.138a: 'other cards')"
                    .into(),
            ));
        }
        let obj = state.objects.get(&id).ok_or_else(|| {
            GameStateError::InvalidCommand(format!("escape exile card {:?} not found", id))
        })?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "escape exile card {:?} is not in your graveyard (CR 702.138a)",
                id
            )));
        }
    }
    // Exile each card (CR 400.7: new ObjectId after zone change).
    for &id in escape_cards {
        let (new_exile_id, _) = state.move_object_to_zone(id, ZoneId::Exile)?;
        events.push(GameEvent::ObjectExiled {
            player,
            object_id: id,
            new_exile_id,
        });
    }
    Ok(())
}

/// CR 702.51a: Validate convoke creatures and compute the reduced mana cost.
///
/// For each creature in `convoke_creatures`:
/// - Must exist in `state.objects` on the battlefield.
/// - Must be controlled by `player`.
/// - Must be a creature (by current characteristics via `calculate_characteristics`).
/// - Must be untapped (`status.tapped == false`).
/// - Must not appear twice in the list (no duplicates).
///
/// Reduction (CR 702.51a):
/// - For each creature, try to pay one colored pip that matches one of the creature's
///   colors (WUBRG order). If no colored pip matches, reduce one generic pip.
/// - If neither colored nor generic mana remains to reduce, the creature list is too
///   long — return an error.
///
/// Taps each creature in `state.objects` and emits a `PermanentTapped` event.
/// Returns the reduced `Option<ManaCost>`.
fn apply_convoke_reduction(
    state: &mut GameState,
    player: PlayerId,
    convoke_creatures: &[ObjectId],
    cost: Option<ManaCost>,
    events: &mut Vec<GameEvent>,
) -> Result<Option<ManaCost>, GameStateError> {
    use crate::state::types::Color;

    // Validate uniqueness (no duplicates in convoke_creatures).
    let mut seen = std::collections::HashSet::new();
    for &id in convoke_creatures {
        if !seen.insert(id) {
            return Err(GameStateError::InvalidCommand(format!(
                "duplicate creature {:?} in convoke_creatures",
                id
            )));
        }
    }

    // Build per-creature color list with validation, before mutably borrowing state.
    let mut creature_colors: Vec<(ObjectId, im::OrdSet<Color>)> = Vec::new();
    for &id in convoke_creatures {
        let obj = state
            .objects
            .get(&id)
            .ok_or(GameStateError::ObjectNotFound(id))?;

        // Must be on the battlefield.
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::InvalidCommand(format!(
                "convoke creature {:?} is not on the battlefield",
                id
            )));
        }
        // Must be controlled by the caster.
        if obj.controller != player {
            return Err(GameStateError::InvalidCommand(format!(
                "convoke creature {:?} is not controlled by the caster",
                id
            )));
        }
        // Must be untapped.
        if obj.status.tapped {
            return Err(GameStateError::InvalidCommand(format!(
                "convoke creature {:?} is already tapped",
                id
            )));
        }

        // Must be a creature (use calculate_characteristics for layer-correct check).
        let chars = calculate_characteristics(state, id)
            .or_else(|| state.objects.get(&id).map(|o| o.characteristics.clone()))
            .unwrap_or_default();
        if !chars
            .card_types
            .contains(&crate::state::types::CardType::Creature)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "convoke creature {:?} is not a creature",
                id
            )));
        }

        creature_colors.push((id, chars.colors.clone()));
    }

    // Apply cost reduction.
    let had_cost = cost.is_some();
    let mut reduced = cost.unwrap_or_default();

    for (_id, colors) in &creature_colors {
        // Try to pay one colored pip in WUBRG order.
        let paid_colored = if colors.contains(&Color::White) && reduced.white > 0 {
            reduced.white -= 1;
            true
        } else if colors.contains(&Color::Blue) && reduced.blue > 0 {
            reduced.blue -= 1;
            true
        } else if colors.contains(&Color::Black) && reduced.black > 0 {
            reduced.black -= 1;
            true
        } else if colors.contains(&Color::Red) && reduced.red > 0 {
            reduced.red -= 1;
            true
        } else if colors.contains(&Color::Green) && reduced.green > 0 {
            reduced.green -= 1;
            true
        } else {
            false
        };

        if !paid_colored {
            // No matching colored pip — reduce one generic pip.
            if reduced.generic > 0 {
                reduced.generic -= 1;
            } else {
                return Err(GameStateError::InvalidCommand(
                    "too many creatures tapped for convoke — exceeds total cost".into(),
                ));
            }
        }
    }

    // Tap each convoke creature and emit PermanentTapped events.
    for &id in convoke_creatures {
        if let Some(obj) = state.objects.get_mut(&id) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: id,
        });
    }

    // If the original cost was Some, return Some(reduced); if it was None, return None.
    if had_cost {
        Ok(Some(reduced))
    } else {
        Ok(None)
    }
}

/// CR 702.126a: Validate improvise artifacts and compute the reduced mana cost.
///
/// For each artifact in `improvise_artifacts`:
/// - Must exist in `state.objects` on the battlefield.
/// - Must be controlled by `player`.
/// - Must be an artifact (by current characteristics via `calculate_characteristics`).
/// - Must be untapped (`status.tapped == false`).
/// - Must not appear twice in the list (no duplicates).
///
/// Reduction (CR 702.126a):
/// - Each artifact reduces one generic pip. Cannot exceed total generic mana.
///   Improvise can ONLY reduce generic mana — never colored or colorless ({C}).
///
/// Taps each artifact in `state.objects` and emits a `PermanentTapped` event.
/// Returns the reduced `Option<ManaCost>`.
///
/// CR 702.126b: Improvise is not an additional or alternative cost — it applies
/// after the total cost (including commander tax, convoke) is determined.
fn apply_improvise_reduction(
    state: &mut GameState,
    player: PlayerId,
    improvise_artifacts: &[ObjectId],
    cost: Option<ManaCost>,
    events: &mut Vec<GameEvent>,
) -> Result<Option<ManaCost>, GameStateError> {
    // Validate uniqueness (no duplicates in improvise_artifacts).
    let mut seen = std::collections::HashSet::new();
    for &id in improvise_artifacts {
        if !seen.insert(id) {
            return Err(GameStateError::InvalidCommand(format!(
                "duplicate artifact {:?} in improvise_artifacts",
                id
            )));
        }
    }

    // Validate each artifact before mutably borrowing state for tapping.
    for &id in improvise_artifacts {
        let obj = state
            .objects
            .get(&id)
            .ok_or(GameStateError::ObjectNotFound(id))?;

        // Must be on the battlefield.
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::InvalidCommand(format!(
                "improvise artifact {:?} is not on the battlefield",
                id
            )));
        }
        // Must be controlled by the caster.
        if obj.controller != player {
            return Err(GameStateError::InvalidCommand(format!(
                "improvise artifact {:?} is not controlled by the caster",
                id
            )));
        }
        // Must be untapped.
        if obj.status.tapped {
            return Err(GameStateError::InvalidCommand(format!(
                "improvise artifact {:?} is already tapped",
                id
            )));
        }

        // Must be an artifact (use calculate_characteristics for layer-correct check).
        let chars = calculate_characteristics(state, id)
            .or_else(|| state.objects.get(&id).map(|o| o.characteristics.clone()))
            .unwrap_or_default();
        if !chars
            .card_types
            .contains(&crate::state::types::CardType::Artifact)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "improvise artifact {:?} is not an artifact",
                id
            )));
        }
    }

    // Apply cost reduction: each artifact reduces one generic pip (CR 702.126a).
    // Improvise can ONLY reduce generic mana — it cannot pay for colored or colorless ({C}) pips.
    let had_cost = cost.is_some();
    let mut reduced = cost.unwrap_or_default();

    // Validate that we don't tap more artifacts than the generic portion allows.
    if improvise_artifacts.len() as u32 > reduced.generic {
        return Err(GameStateError::InvalidCommand(format!(
            "improvise_artifacts.len() ({}) exceeds generic mana in cost ({})",
            improvise_artifacts.len(),
            reduced.generic
        )));
    }
    reduced.generic -= improvise_artifacts.len() as u32;

    // Tap each improvise artifact and emit PermanentTapped events.
    for &id in improvise_artifacts {
        if let Some(obj) = state.objects.get_mut(&id) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: id,
        });
    }

    // If the original cost was Some, return Some(reduced); if it was None, return None.
    if had_cost {
        Ok(Some(reduced))
    } else {
        Ok(None)
    }
}

/// CR 702.66a: Validate delve cards and compute the reduced mana cost.
///
/// For each card in `delve_cards`:
/// - Must exist in the caster's graveyard (`ZoneId::Graveyard(player)`).
/// - Must not appear twice (no duplicates).
///
/// Reduction (CR 702.66a):
/// - Each exiled card reduces one generic pip. Cannot exceed total generic mana.
///
/// Exiles each card via `state.move_object_to_zone(id, ZoneId::Exile)` and
/// emits `ObjectExiled` events for each.
/// Returns the reduced `Option<ManaCost>`.
///
/// CR 702.66b: Delve is not an additional or alternative cost — it applies
/// after the total cost (including commander tax) is determined.
fn apply_delve_reduction(
    state: &mut GameState,
    player: PlayerId,
    delve_cards: &[ObjectId],
    cost: Option<ManaCost>,
    events: &mut Vec<GameEvent>,
) -> Result<Option<ManaCost>, GameStateError> {
    // Validate uniqueness (no duplicates in delve_cards).
    let mut seen = std::collections::HashSet::new();
    for &id in delve_cards {
        if !seen.insert(id) {
            return Err(GameStateError::InvalidCommand(format!(
                "duplicate card {:?} in delve_cards",
                id
            )));
        }
    }

    // Validate each card is in the caster's graveyard (CR 702.66a: "your graveyard").
    for &id in delve_cards {
        let obj = state
            .objects
            .get(&id)
            .ok_or(GameStateError::ObjectNotFound(id))?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "delve card {:?} is not in the caster's graveyard (zone: {:?})",
                id, obj.zone
            )));
        }
    }

    // Apply cost reduction: each card reduces one generic pip (CR 702.66a).
    let had_cost = cost.is_some();
    let mut reduced = cost.unwrap_or_default();

    // Validate that we don't exile more cards than the generic portion allows
    // (CR 702.66a / Treasure Cruise ruling).
    if delve_cards.len() as u32 > reduced.generic {
        return Err(GameStateError::InvalidCommand(format!(
            "delve_cards.len() ({}) exceeds generic mana in cost ({})",
            delve_cards.len(),
            reduced.generic
        )));
    }
    reduced.generic -= delve_cards.len() as u32;

    // Exile each card and emit ObjectExiled events (CR 400.7: new ObjectId after zone change).
    for &id in delve_cards {
        let (new_exile_id, _old_obj) = state.move_object_to_zone(id, ZoneId::Exile)?;
        events.push(GameEvent::ObjectExiled {
            player,
            object_id: id,
            new_exile_id,
        });
    }

    // If the original cost was Some, return Some(reduced); if it was None, return None.
    if had_cost {
        Ok(Some(reduced))
    } else {
        Ok(None)
    }
}

/// CR 601.2c: Validate targets at cast time and snapshot their current zones.
///
/// For each target:
/// - Player: must be an active (non-eliminated) player matching the requirement
/// - Object: must exist, pass hexproof/shroud/protection checks, and satisfy the TargetRequirement
///
/// `requirements` is indexed in parallel with `targets` (requirements[i] applies to
/// targets[i]). If there are fewer requirements than targets, extra targets are
/// existence-only validated (no type restriction). This handles cards without
/// definitions registered at cast time.
///
/// `source_chars` is the characteristics of the spell being cast, used for protection-from
/// checks (CR 702.16b). Pass `None` when unavailable (protection check is skipped).
fn validate_targets(
    state: &GameState,
    targets: &[Target],
    requirements: &[TargetRequirement],
    caster: PlayerId,
    source_chars: Option<&Characteristics>,
) -> Result<Vec<SpellTarget>, GameStateError> {
    let mut spell_targets = Vec::with_capacity(targets.len());

    for (i, target) in targets.iter().enumerate() {
        let req = requirements.get(i);

        let spell_target = match target {
            Target::Player(id) => {
                let player = state
                    .players
                    .get(id)
                    .ok_or(GameStateError::PlayerNotFound(*id))?;
                if player.has_lost || player.has_conceded {
                    return Err(GameStateError::InvalidTarget(format!(
                        "player {:?} is not an active player",
                        id
                    )));
                }
                // CR 702.16b: a player with protection from a quality cannot be targeted
                // by sources that match that quality.
                if let Some(sc) = source_chars {
                    for quality in &player.protection_qualities {
                        if crate::rules::protection::has_protection_from_source_quality(quality, sc)
                        {
                            return Err(GameStateError::InvalidTarget(format!(
                                "player {:?} has protection from the source and cannot be targeted",
                                id
                            )));
                        }
                    }
                }
                // CR 601.2c: Validate the target satisfies the declared requirement.
                if let Some(req) = req {
                    validate_player_satisfies_requirement(*id, req)?;
                }
                SpellTarget {
                    target: Target::Player(*id),
                    zone_at_cast: None, // Players are not in a zone
                }
            }
            Target::Object(id) => {
                // Object targets are always looked up in state.objects.
                // Spells on the stack are also in state.objects (zone == ZoneId::Stack);
                // StackObject entries in state.stack_objects have separate IDs used
                // internally by the engine, not as targets.
                let obj = state
                    .objects
                    .get(id)
                    .ok_or(GameStateError::ObjectNotFound(*id))?;

                // CR 702.11a / CR 702.18a / CR 702.16b: Hexproof, shroud, and protection.
                super::validate_target_protection(
                    &obj.characteristics.keywords,
                    obj.controller,
                    caster,
                    source_chars,
                )?;

                // CR 601.2c: Validate the target satisfies the declared requirement.
                if let Some(req) = req {
                    validate_object_satisfies_requirement(state, *id, req, caster)?;
                }

                SpellTarget {
                    target: Target::Object(*id),
                    zone_at_cast: Some(obj.zone),
                }
            }
        };
        spell_targets.push(spell_target);
    }

    Ok(spell_targets)
}

/// CR 601.2c: Check that a player target satisfies a requirement.
///
/// Player targets are valid for `TargetPlayer`, `TargetCreatureOrPlayer`,
/// `TargetAny`, and `TargetPlayerOrPlaneswalker`. All other requirements
/// expect an object, so a player target is illegal.
fn validate_player_satisfies_requirement(
    id: PlayerId,
    req: &TargetRequirement,
) -> Result<(), GameStateError> {
    match req {
        TargetRequirement::TargetPlayer
        | TargetRequirement::TargetCreatureOrPlayer
        | TargetRequirement::TargetAny
        | TargetRequirement::TargetPlayerOrPlaneswalker => Ok(()),
        _ => Err(GameStateError::InvalidTarget(format!(
            "player {:?} does not satisfy requirement {:?} (expected an object)",
            id, req
        ))),
    }
}

/// CR 601.2c: Check that an object target satisfies a `TargetRequirement`.
///
/// Uses `calculate_characteristics` for type/keyword checks to respect
/// continuous effects (e.g., type-changing effects from the layer system).
fn validate_object_satisfies_requirement(
    state: &GameState,
    id: ObjectId,
    req: &TargetRequirement,
    caster: PlayerId,
) -> Result<(), GameStateError> {
    // All requirements look up the object in state.objects.
    // Spells on the stack exist in state.objects with zone == ZoneId::Stack.
    let obj = state
        .objects
        .get(&id)
        .ok_or(GameStateError::ObjectNotFound(id))?;

    // TargetSpell / TargetSpellWithFilter: object must be in the stack zone (CR 601.2c).
    if matches!(
        req,
        TargetRequirement::TargetSpell | TargetRequirement::TargetSpellWithFilter(_)
    ) {
        if obj.zone != ZoneId::Stack {
            return Err(GameStateError::InvalidTarget(format!(
                "object {:?} is not on the stack",
                id
            )));
        }
        // For TargetSpellWithFilter, also check the filter against the spell's characteristics.
        if let TargetRequirement::TargetSpellWithFilter(filter) = req {
            let chars: Characteristics =
                calculate_characteristics(state, id).unwrap_or_else(|| obj.characteristics.clone());
            if !crate::effects::matches_filter(&chars, filter) {
                return Err(GameStateError::InvalidTarget(format!(
                    "spell {:?} does not match the filter for {:?}",
                    id, req
                )));
            }
        }
        return Ok(());
    }

    // Use calculate_characteristics to respect continuous effects (CR 613).
    let chars: Characteristics =
        calculate_characteristics(state, id).unwrap_or_else(|| obj.characteristics.clone());

    let on_battlefield = obj.zone == ZoneId::Battlefield;
    let is_creature = chars.card_types.contains(&CardType::Creature);
    let is_artifact = chars.card_types.contains(&CardType::Artifact);
    let is_enchantment = chars.card_types.contains(&CardType::Enchantment);
    let is_land = chars.card_types.contains(&CardType::Land);
    let is_planeswalker = chars.card_types.contains(&CardType::Planeswalker);

    let valid = match req {
        TargetRequirement::TargetCreature => on_battlefield && is_creature,
        TargetRequirement::TargetPermanent => on_battlefield,
        TargetRequirement::TargetArtifact => on_battlefield && is_artifact,
        TargetRequirement::TargetEnchantment => on_battlefield && is_enchantment,
        TargetRequirement::TargetLand => on_battlefield && is_land,
        TargetRequirement::TargetPlaneswalker => on_battlefield && is_planeswalker,
        // "target creature or player" — object side requires creature on battlefield
        TargetRequirement::TargetCreatureOrPlayer => on_battlefield && is_creature,
        // "any target" = creature, planeswalker, or player (CR 115.4) — object side
        TargetRequirement::TargetAny => on_battlefield && (is_creature || is_planeswalker),
        // "target player or planeswalker" — object side requires planeswalker
        TargetRequirement::TargetPlayerOrPlaneswalker => on_battlefield && is_planeswalker,
        TargetRequirement::TargetCreatureWithFilter(filter) => {
            if !on_battlefield || !is_creature {
                false
            } else {
                let passes_filter = crate::effects::matches_filter(&chars, filter);
                let passes_controller = match filter.controller {
                    TargetController::Any => true,
                    TargetController::You => obj.controller == caster,
                    TargetController::Opponent => obj.controller != caster,
                };
                passes_filter && passes_controller
            }
        }
        TargetRequirement::TargetPermanentWithFilter(filter) => {
            if !on_battlefield {
                false
            } else {
                let passes_filter = crate::effects::matches_filter(&chars, filter);
                let passes_controller = match filter.controller {
                    TargetController::Any => true,
                    TargetController::You => obj.controller == caster,
                    TargetController::Opponent => obj.controller != caster,
                };
                passes_filter && passes_controller
            }
        }
        // Player requirement — object target is illegal
        TargetRequirement::TargetPlayer => false,
        // TargetSpell and TargetSpellWithFilter handled above via early return (zone + filter check).
        TargetRequirement::TargetSpell | TargetRequirement::TargetSpellWithFilter(_) => false,
    };

    if valid {
        Ok(())
    } else {
        Err(GameStateError::InvalidTarget(format!(
            "object {:?} does not satisfy requirement {:?}",
            id, req
        )))
    }
}

/// Returns true if the mana pool can cover the mana cost.
///
/// Colored mana (W/U/B/R/G) must be paid with the matching color.
/// Colorless mana (`{C}`) must be paid with colorless mana specifically (CR 106.1).
/// Generic mana (`{N}`) can be paid with any remaining mana in the pool.
pub fn can_pay_cost(
    pool: &crate::state::player::ManaPool,
    cost: &crate::state::game_object::ManaCost,
) -> bool {
    if pool.white < cost.white {
        return false;
    }
    if pool.blue < cost.blue {
        return false;
    }
    if pool.black < cost.black {
        return false;
    }
    if pool.red < cost.red {
        return false;
    }
    if pool.green < cost.green {
        return false;
    }
    if pool.colorless < cost.colorless {
        return false;
    }

    // Remaining mana after paying colored and colorless requirements.
    let remaining = (pool.white - cost.white)
        + (pool.blue - cost.blue)
        + (pool.black - cost.black)
        + (pool.red - cost.red)
        + (pool.green - cost.green)
        + (pool.colorless - cost.colorless);

    remaining >= cost.generic
}

/// CR 702.61a: Check if any spell on the stack has split second.
///
/// While a spell with split second is on the stack, players can't cast other
/// spells or activate abilities that aren't mana abilities (CR 702.61a).
/// Mana abilities and special actions are still allowed (CR 702.61b).
/// Triggered abilities still trigger and resolve normally (CR 702.61b).
///
/// Uses `calculate_characteristics` to respect continuous effects that might
/// grant or remove split second (layer system, CR 613).
pub fn has_split_second_on_stack(state: &GameState) -> bool {
    state.stack_objects.iter().any(|stack_obj| {
        if let StackObjectKind::Spell { source_object } = &stack_obj.kind {
            let chars = calculate_characteristics(state, *source_object).unwrap_or_else(|| {
                state
                    .object(*source_object)
                    .map(|o| o.characteristics.clone())
                    .unwrap_or_default()
            });
            chars.keywords.contains(&KeywordAbility::SplitSecond)
        } else {
            false
        }
    })
}

/// Deduct a mana cost from the mana pool. Caller must verify `can_pay_cost` first.
///
/// For generic mana, mana is taken from remaining colored/colorless in order:
/// colorless, then green, red, black, blue, white. The specific order doesn't
/// affect correctness since generic can use any color.
pub fn pay_cost(
    pool: &mut crate::state::player::ManaPool,
    cost: &crate::state::game_object::ManaCost,
) {
    pool.white -= cost.white;
    pool.blue -= cost.blue;
    pool.black -= cost.black;
    pool.red -= cost.red;
    pool.green -= cost.green;
    pool.colorless -= cost.colorless;

    // Pay generic cost from remaining mana.
    let mut remaining = cost.generic;
    for slot in [
        &mut pool.colorless,
        &mut pool.green,
        &mut pool.red,
        &mut pool.black,
        &mut pool.blue,
        &mut pool.white,
    ] {
        let take = remaining.min(*slot);
        *slot -= take;
        remaining -= take;
        if remaining == 0 {
            break;
        }
    }
}

/// CR 702.41a: Apply affinity cost reduction to the total mana cost.
///
/// For each instance of `KeywordAbility::Affinity(target)` on the spell,
/// count the number of permanents matching `target` that the caster controls,
/// and reduce the generic mana component by that count.
///
/// CR 702.41b: Multiple instances of affinity are cumulative — each one
/// independently counts and reduces. Two instances of "affinity for artifacts"
/// with 3 artifacts = 6 generic mana reduction.
///
/// CR 601.2f: The generic mana component cannot be reduced below 0.
/// Colored and colorless pips are unaffected.
fn apply_affinity_reduction(
    state: &GameState,
    player: PlayerId,
    chars: &Characteristics,
    cost: Option<ManaCost>,
) -> Option<ManaCost> {
    // Collect all affinity instances from the spell's keywords.
    let affinity_targets: Vec<&AffinityTarget> = chars
        .keywords
        .iter()
        .filter_map(|kw| {
            if let KeywordAbility::Affinity(target) = kw {
                Some(target)
            } else {
                None
            }
        })
        .collect();

    if affinity_targets.is_empty() {
        return cost;
    }

    // CR 601.2f: If the spell has no mana cost (None), there is nothing to reduce.
    let mut reduced = cost?;

    // CR 702.41b: Each instance applies independently.
    for target in &affinity_targets {
        let count = count_affinity_permanents(state, player, target);
        // CR 601.2f: Generic cannot go below 0.
        let reduction = count.min(reduced.generic);
        reduced.generic -= reduction;
    }

    Some(reduced)
}

/// Count permanents on the battlefield matching the affinity target
/// that are controlled by the given player.
///
/// CR 702.41a: Counts ALL matching permanents — tapped or untapped.
fn count_affinity_permanents(state: &GameState, player: PlayerId, target: &AffinityTarget) -> u32 {
    state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.controller == player
                && matches_affinity_target(state, obj, target)
        })
        .count() as u32
}

/// Check if a game object matches the given affinity target.
///
/// Uses `calculate_characteristics` for layer-correct type checking.
fn matches_affinity_target(
    state: &GameState,
    obj: &crate::state::game_object::GameObject,
    target: &AffinityTarget,
) -> bool {
    let chars =
        calculate_characteristics(state, obj.id).unwrap_or_else(|| obj.characteristics.clone());
    match target {
        AffinityTarget::Artifacts => chars.card_types.contains(&CardType::Artifact),
        AffinityTarget::BasicLandType(subtype) => {
            chars.card_types.contains(&CardType::Land) && chars.subtypes.contains(subtype)
        }
    }
}

/// CR 702.125a: Apply undaunted cost reduction to the total mana cost.
///
/// For each instance of `KeywordAbility::Undaunted` on the spell,
/// count the number of opponents the caster has (CR 702.125b: only active
/// players who have not left the game), and reduce the generic mana
/// component by that count.
///
/// CR 702.125c: Multiple instances of undaunted are cumulative -- each one
/// independently counts opponents and reduces. Two instances with 3 opponents
/// = 6 generic mana reduction.
///
/// CR 601.2f: The generic mana component cannot be reduced below 0.
/// Colored and colorless pips are unaffected.
fn apply_undaunted_reduction(
    state: &GameState,
    player: PlayerId,
    chars: &Characteristics,
    cost: Option<ManaCost>,
) -> Option<ManaCost> {
    // Count how many instances of Undaunted the spell has.
    let undaunted_count = chars
        .keywords
        .iter()
        .filter(|kw| matches!(kw, KeywordAbility::Undaunted))
        .count() as u32;

    if undaunted_count == 0 {
        return cost;
    }

    // CR 601.2f: If the spell has no mana cost (None), there is nothing to reduce.
    let mut reduced = cost?;

    // CR 702.125b: Count only active players (not lost/conceded) who are NOT the caster.
    let opponent_count = state
        .active_players()
        .iter()
        .filter(|&&pid| pid != player)
        .count() as u32;

    // CR 702.125c: Each instance applies independently.
    let total_reduction = undaunted_count * opponent_count;

    // CR 601.2f: Generic cannot go below 0.
    let reduction = total_reduction.min(reduced.generic);
    reduced.generic -= reduction;

    Some(reduced)
}

/// CR 702.160a: Extract prototype data (cost, power, toughness) from a card definition.
///
/// Returns `Some((cost, power, toughness))` if the card has `AbilityDefinition::AltCastAbility { kind: AltCostKind::Prototype, .. }`,
/// or `None` if the card has no definition or no prototype ability defined.
pub(crate) fn get_prototype_data(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<(ManaCost, i32, i32)> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility {
                    kind: AltCostKind::Prototype,
                    cost,
                    details: Some(crate::cards::card_definition::AltCastDetails::Prototype { power, toughness }),
                } = a
                {
                    Some((cost.clone(), *power, *toughness))
                } else {
                    None
                }
            })
        })
    })
}

/// CR 105.2: Derive the colors of an object from its mana cost.
///
/// An object is the color or colors of the mana symbols in its mana cost.
/// Used to compute the colors of a prototyped permanent (CR 718.3b).
pub(crate) fn colors_from_mana_cost(cost: &ManaCost) -> im::OrdSet<crate::state::types::Color> {
    let mut colors = im::OrdSet::new();
    if cost.white > 0 {
        colors.insert(crate::state::types::Color::White);
    }
    if cost.blue > 0 {
        colors.insert(crate::state::types::Color::Blue);
    }
    if cost.black > 0 {
        colors.insert(crate::state::types::Color::Black);
    }
    if cost.red > 0 {
        colors.insert(crate::state::types::Color::Red);
    }
    if cost.green > 0 {
        colors.insert(crate::state::types::Color::Green);
    }
    colors
}

/// CR 702.176a: Look up the impending cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Impending { cost, .. }`, or `None`
/// if the card has no definition or no impending ability defined.
fn get_impending_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Impending { cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.176a: Look up the impending counter count from the card's `AbilityDefinition`.
///
/// Returns the `count` stored in `AbilityDefinition::Impending { count, .. }`, or `None`
/// if the card has no definition or no impending ability defined.
pub(crate) fn get_impending_count(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<u32> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Impending { count, .. } = a {
                    Some(*count)
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.119a: Look up the emerge cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Emerge { cost }`, or `None`
/// if the card has no definition or no emerge ability defined.
fn get_emerge_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Emerge { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.137a: Look up the spectacle cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Spectacle { cost }`, or `None`
/// if the card has no definition or no spectacle ability defined.
fn get_spectacle_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Spectacle { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.117a: Look up the surge cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Surge { cost }`, or `None`
/// if the card has no definition or no surge ability defined.
fn get_surge_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Surge { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.148a: Look up the cleave cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Cleave { cost }`, or `None`
/// if the card has no definition or no cleave ability defined.
fn get_cleave_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Cleave { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.140a: Look up a card's mutate alternative cost from its CardDefinition.
///
/// Returns the cost stored in `AbilityDefinition::MutateCost { cost }`, or None if the
/// card has no mutate cost defined (malformed card definition or not a mutate card).
fn get_mutate_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::MutateCost { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.119a: Reduce a mana cost by a creature's mana value.
///
/// Reduces generic mana first, then colorless, then colored pips (WUBRG order)
/// if generic and colorless are fully exhausted. The cost cannot go below zero
/// in any component.
fn reduce_cost_by_mv(cost: &ManaCost, mv: u32) -> ManaCost {
    let mut reduced = cost.clone();
    let mut remaining_reduction = mv;

    // Reduce generic first.
    let generic_reduction = remaining_reduction.min(reduced.generic);
    reduced.generic -= generic_reduction;
    remaining_reduction -= generic_reduction;

    if remaining_reduction == 0 {
        return reduced;
    }

    // Then reduce colorless.
    let colorless_reduction = remaining_reduction.min(reduced.colorless);
    reduced.colorless -= colorless_reduction;
    remaining_reduction -= colorless_reduction;

    if remaining_reduction == 0 {
        return reduced;
    }

    // Then reduce colored pips (WUBRG order).
    let fields = [
        &mut reduced.white,
        &mut reduced.blue,
        &mut reduced.black,
        &mut reduced.red,
        &mut reduced.green,
    ];
    for field in fields {
        let reduction = remaining_reduction.min(*field);
        *field -= reduction;
        remaining_reduction -= reduction;
        if remaining_reduction == 0 {
            break;
        }
    }

    reduced
}

/// CR 702.47a: Look up the splice info from the card's `AbilityDefinition`.
///
/// Returns `Some((ManaCost, SubType, Effect))` if the card has a
/// `AbilityDefinition::Splice { cost, onto_subtype, effect }` ability, or `None`
/// if the card has no definition or no splice ability defined.
fn get_splice_info(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<(ManaCost, SubType, crate::cards::card_definition::Effect)> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Splice {
                    cost,
                    onto_subtype,
                    effect,
                } = a
                {
                    Some((cost.clone(), onto_subtype.clone(), *effect.clone()))
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.102c: Look up the fuse (right half) cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Fuse { cost, .. }`, or `None`
/// if the card has no definition or no fuse ability defined.
fn get_fuse_data(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Fuse { cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.102b + CR 709.4d: Look up the fuse (right half) card type.
///
/// Used for timing validation — if the right half is an instant, the fused
/// spell can be cast at instant speed (combined characteristics rule).
///
/// Returns `Some(CardType)` from `AbilityDefinition::Fuse { card_type, .. }`,
/// or `None` if the card has no fuse ability definition.
fn get_fuse_card_type(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<CardType> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Fuse { card_type, .. } = a {
                    Some(*card_type)
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.175a: Look up the offspring cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Offspring { cost }`, or `None`
/// if the card has no definition or no offspring ability defined.
fn get_offspring_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Offspring { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.157a: Look up the squad cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Squad { cost }`, or `None`
/// if the card has no definition or no squad ability defined.
fn get_squad_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Squad { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.146a: Look up the disturb cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Disturb { cost }`, or `None`
/// if the card has no definition or no disturb ability defined.
fn get_disturb_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Disturb { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 702.37c / 702.37b / 702.168a: Returns true if the card has Morph, Megamorph, or Disguise.
/// Used to validate morph cast legality (cast_with_morph path).
fn has_morph_keyword(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> bool {
    use crate::state::types::KeywordAbility;
    card_id.as_ref().is_some_and(|cid| {
        registry
            .get(cid.clone())
            .map(|def| {
                def.abilities.iter().any(|a| {
                    matches!(
                        a,
                        AbilityDefinition::Morph { .. }
                            | AbilityDefinition::Megamorph { .. }
                            | AbilityDefinition::Disguise { .. }
                    ) || matches!(
                        a,
                        AbilityDefinition::Keyword(KeywordAbility::Morph)
                            | AbilityDefinition::Keyword(KeywordAbility::Megamorph)
                            | AbilityDefinition::Keyword(KeywordAbility::Disguise)
                    )
                })
            })
            .unwrap_or(false)
    })
}
