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
use crate::state::game_object::{Characteristics, ManaCost, ObjectId};
use crate::state::player::PlayerId;
use crate::state::stack::{StackObject, StackObjectKind};
use crate::state::targeting::{SpellTarget, Target};
use crate::state::turn::Step;
use crate::state::types::{AffinityTarget, CardType, EnchantTarget, KeywordAbility, SubType};
use crate::state::zone::ZoneId;
use crate::state::GameState;

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
    cast_with_evoke: bool,
    cast_with_bestow: bool,
    cast_with_miracle: bool,
    cast_with_escape: bool,
    escape_exile_cards: Vec<ObjectId>,
    cast_with_foretell: bool,
    cast_with_buyback: bool,
    cast_with_overload: bool,
) -> Result<Vec<GameEvent>, GameStateError> {
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
        casting_with_flashback,
        casting_with_madness,
        card_has_escape_keyword,
        card_id,
        base_mana_cost,
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
        let casting_with_flashback = casting_from_graveyard
            && card_obj
                .characteristics
                .keywords
                .contains(&KeywordAbility::Flashback)
            && !cast_with_escape;

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
            if !card_obj.is_foretold {
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

        if card_obj.zone != ZoneId::Hand(player)
            && !casting_from_command_zone
            && !casting_with_flashback
            && !casting_with_madness
            && !casting_with_escape_auto
            && !cast_with_escape
            && !cast_with_foretell
        {
            return Err(GameStateError::InvalidCommand(
                "card is not in your hand".into(),
            ));
        }
        (
            casting_from_command_zone,
            casting_from_graveyard,
            casting_with_flashback,
            casting_with_madness,
            card_has_escape_keyword,
            card_obj.card_id.clone(),
            card_obj.characteristics.mana_cost.clone(),
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

    // Step 2: Select the base cost (alternative cost takes precedence over mana cost).
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
    } else {
        base_mana_cost
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

    // Validate casting window.
    // CR 702.35 ruling: Madness ignores timing restrictions — a sorcery cast via madness
    // can be cast any time the player has priority, like an instant.
    // CR 702.94a ruling: Miracle ignores timing restrictions — a sorcery cast via miracle
    // can be cast at instant speed (while the miracle trigger is on the stack).
    // CR 702.138a ruling (2020-01-24): Escape does NOT ignore timing restrictions —
    // sorcery-speed cards with escape can only be cast at sorcery speed.
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
    let (requirements, cant_be_countered): (Vec<TargetRequirement>, bool) = {
        let registry = state.card_registry.clone();
        card_id
            .clone()
            .and_then(|cid| registry.get(cid))
            .and_then(|def| {
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

    // CR 702.66a / CR 601.2h: Emit ObjectExiled events for delve cards.
    // Exile happens as part of cost payment (CR 601.2h), after the mana payment.
    events.extend(delve_events);

    // CR 702.138a / CR 601.2h: Emit ObjectExiled events for escape exile cards.
    // Exile happens as part of cost payment (CR 601.2h), after the mana payment.
    events.extend(escape_exile_events);

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

    let stack_obj = StackObject {
        id: stack_entry_id,
        controller: player,
        kind: StackObjectKind::Spell {
            source_object: new_card_id,
        },
        targets: spell_targets,
        cant_be_countered,
        is_copy: false,
        cast_with_flashback: casting_with_flashback,
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
            kind: StackObjectKind::StormTrigger {
                source_object: new_card_id,
                original_stack_id: stack_entry_id,
                storm_count: count,
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
            kind: StackObjectKind::CascadeTrigger {
                source_object: new_card_id,
                spell_mana_value: spell_mv,
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
/// Returns the `ManaCost` stored in `AbilityDefinition::Flashback { cost }`, or `None`
/// if the card has no definition or no flashback ability defined. When `None` is returned,
/// no mana payment is required (free flashback — rare, but correct per CR 118.9).
fn get_flashback_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Flashback { cost } = a {
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
/// Returns `Some((ManaCost, exile_count))` from `AbilityDefinition::Escape { cost, exile_count }`,
/// or `None` if the card has no escape ability definition.
fn get_escape_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<(ManaCost, u32)> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Escape { cost, exile_count } = a {
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
