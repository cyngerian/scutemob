//! Plot special action handler (CR 702.170).
//!
//! Plot is a keyword that functions while the card is in a player's hand.
//! "Any time you have priority during your main phase while the stack is empty,
//! you may exile this card from your hand and pay [cost]. It becomes a plotted card."
//! (CR 702.170a)
//!
//! This is a special action (CR 116.2k) -- it does NOT use the stack.
//! The card can be cast for free on any LATER turn during the owner's main phase
//! while the stack is empty (CR 702.170d).
//!
//! Key rules:
//! - Special action -- no stack, no priority interruption (CR 702.170b)
//! - Requires main phase + empty stack (sorcery speed, CR 702.170a / CR 116.2k)
//! - Cost: the plot cost from AbilityDefinition::Plot { cost }
//! - Card enters exile FACE UP (public information, CR 702.170a)
//! - Card can only be cast on a LATER turn (CR 702.170d: "any turn after the turn
//!   in which it became plotted")
use crate::cards::card_definition::AbilityDefinition;
use crate::rules::casting;
use crate::rules::events::GameEvent;
use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::turn::Step;
use crate::state::types::KeywordAbility;
use crate::state::zone::ZoneId;
use crate::state::GameState;
/// CR 702.170a / CR 116.2k: Handle the PlotCard special action.
///
/// Validates:
/// 1. It is the player's turn (CR 116.2k: "during their own turn").
/// 2. It is a main phase (CR 702.170a: "during your main phase").
/// 3. The stack is empty (CR 702.170a: "while the stack is empty").
/// 4. The card is in the player's hand (CR 702.170a: "exile this card from your hand").
/// 5. The card has KeywordAbility::Plot (CR 702.170a).
/// 6. Player can pay the plot cost.
///
/// On success:
/// - Deducts the plot cost from the player's mana pool.
/// - Moves the card from hand to exile (CR 400.7: new ObjectId).
/// - Sets face_down = false on the new exile object (face-up, public info).
/// - Sets is_plotted = true and plotted_turn = current turn number.
/// - Emits ManaCostPaid and CardPlotted events.
///
/// NOTE: Unlike Foretell (any time during your turn with priority), Plot requires
/// main phase + empty stack (sorcery speed, CR 116.2k / CR 702.170a).
pub fn handle_plot_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();
    // CR 116.2k: Plot requires the player to have priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // CR 116.2k: Plot can only be done during the player's own turn.
    if state.turn.active_player != player {
        return Err(GameStateError::InvalidCommand(
            "plot: can only plot a card during your own turn (CR 116.2k)".into(),
        ));
    }
    // CR 702.170a: Plot requires main phase.
    if !matches!(state.turn.step, Step::PreCombatMain | Step::PostCombatMain) {
        return Err(GameStateError::NotMainPhase);
    }
    // CR 702.170a: Plot requires empty stack.
    if !state.stack_objects.is_empty() {
        return Err(GameStateError::StackNotEmpty);
    }
    // Look up the plot cost and validate the card in a contained scope.
    let plot_cost = {
        let card_obj = state.object(card)?;
        // Validate the card is in the player's hand (CR 702.170a).
        if card_obj.zone != ZoneId::Hand(player) {
            return Err(GameStateError::InvalidCommand(
                "plot: card must be in your hand (CR 702.170a)".into(),
            ));
        }
        // Validate the card has the Plot keyword (CR 702.170a).
        if !card_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Plot)
        {
            return Err(GameStateError::InvalidCommand(
                "plot: card does not have the Plot keyword (CR 702.170a)".into(),
            ));
        }
        // Look up the plot cost from AbilityDefinition::Plot { cost }.
        // The card_id must exist in the registry for the cost to be resolved.
        let registry = state.card_registry.clone();
        let cost = card_obj.card_id.as_ref().and_then(|cid| {
            registry.get(cid.clone()).and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::AltCastAbility {
                        kind: crate::state::types::AltCostKind::Plot,
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
        });
        cost.unwrap_or_default()
    };
    // Validate and deduct the plot cost (CR 702.170a).
    {
        let ps = state.player(player)?;
        if !casting::can_pay_cost(&ps.mana_pool, &plot_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        let ps_mut = state.player_mut(player)?;
        casting::pay_cost(&mut ps_mut.mana_pool, &plot_cost);
    }
    // Emit ManaCostPaid event for the plot cost.
    events.push(GameEvent::ManaCostPaid {
        player,
        cost: plot_cost,
    });
    // Record the current turn number before moving the card (zone move creates new id).
    let current_turn = state.turn.turn_number;
    // Move the card from hand to exile (CR 400.7: new ObjectId).
    let (new_exile_id, _old_obj) = state.move_object_to_zone(card, ZoneId::Exile)?;
    // Set the plotted attributes on the new exile object.
    // - face_down: false (plotted cards are face-up, public information, CR 702.170a)
    // - is_plotted: true (marks this as a plotted card)
    // - plotted_turn: current turn number (cannot cast until a later turn, CR 702.170d)
    if let Some(exile_obj) = state.objects.get_mut(&new_exile_id) {
        exile_obj.status.face_down = false;
        exile_obj.is_plotted = true;
        exile_obj.plotted_turn = current_turn;
    }
    // Emit CardPlotted event.
    events.push(GameEvent::CardPlotted {
        player,
        object_id: card,
        new_exile_id,
    });
    Ok(events)
}
