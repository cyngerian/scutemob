//! Foretell special action handler (CR 702.143).
//!
//! Foretell is a keyword that functions while the card is in a player's hand.
//! Any time a player has priority during their own turn, they may pay {2} and
//! exile a card with foretell from their hand face down (CR 702.143a).
//!
//! This is a special action (CR 116.2h) -- it does NOT use the stack.
//! The card can be cast for its foretell cost on any LATER turn.
//!
//! Key rules:
//! - Special action -- no stack, no priority interruption (CR 702.143b)
//! - Any time the player has priority during their OWN turn (not sorcery speed)
//! - Cost: {2} generic mana, paid immediately
//! - Card enters exile face-down (hidden from opponents)
//! - Card can only be cast AFTER the current turn ends (CR 702.143a)
use crate::rules::casting;
use crate::rules::events::GameEvent;
use crate::state::error::GameStateError;
use crate::state::game_object::{Designations, ManaCost, ObjectId};
use crate::state::player::PlayerId;
use crate::state::types::KeywordAbility;
use crate::state::zone::ZoneId;
use crate::state::GameState;
/// CR 702.143a / CR 116.2h: Handle the ForetellCard special action.
///
/// Validates:
/// 1. It is the player's turn (CR 116.2h: "during their turn")
/// 2. The card is in the player's hand
/// 3. The card has KeywordAbility::Foretell (CR 702.143a)
/// 4. Player has {2} generic mana available
///
/// On success:
/// - Deducts {2} generic mana
/// - Moves the card from hand to exile (CR 400.7: new ObjectId)
/// - Sets face_down = true on the new exile object
/// - Sets is_foretold = true and foretold_turn = current turn number
/// - Emits ManaCostPaid and CardForetold events
///
/// NOTE: Unlike BringCompanion, foretell does NOT require main phase or empty stack.
/// It can be done any time the player has priority during their own turn (CR 116.2h).
pub fn handle_foretell_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();
    // CR 116.2h: Foretell requires the player to have priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // CR 116.2h: Foretell can only be done during the player's own turn.
    if state.turn.active_player != player {
        return Err(GameStateError::InvalidCommand(
            "foretell: can only foretell a card during your own turn (CR 116.2h)".into(),
        ));
    }
    // Validate the card is in the player's hand.
    {
        let card_obj = state.object(card)?;
        if card_obj.zone != ZoneId::Hand(player) {
            return Err(GameStateError::InvalidCommand(
                "foretell: card must be in your hand (CR 702.143a)".into(),
            ));
        }
        // Validate the card has the Foretell keyword (CR 702.143a).
        if !card_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Foretell)
        {
            return Err(GameStateError::InvalidCommand(
                "foretell: card does not have the Foretell keyword (CR 702.143a)".into(),
            ));
        }
    }
    // Validate and deduct {2} generic mana (CR 702.143a).
    {
        let foretell_cost = ManaCost {
            generic: 2,
            ..Default::default()
        };
        let ps = state.player(player)?;
        if !casting::can_pay_cost(&ps.mana_pool, &foretell_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        let ps_mut = state.player_mut(player)?;
        casting::pay_cost(&mut ps_mut.mana_pool, &foretell_cost);
    }
    // Emit ManaCostPaid event for the {2} foretell action cost.
    events.push(GameEvent::ManaCostPaid {
        player,
        cost: ManaCost {
            generic: 2,
            ..Default::default()
        },
    });
    // Record the current turn number before moving the card (zone move creates new id).
    let current_turn = state.turn.turn_number;
    // Move the card from hand to exile (CR 400.7: new ObjectId).
    let (new_exile_id, _old_obj) = state.move_object_to_zone(card, ZoneId::Exile)?;
    // Set the foretold attributes on the new exile object.
    // - face_down: true (opponents cannot see the card identity)
    // - is_foretold: true (marks this as a foretold card)
    // - foretold_turn: current turn number (cannot cast until a later turn)
    if let Some(exile_obj) = state.objects.get_mut(&new_exile_id) {
        exile_obj.status.face_down = true;
        exile_obj.designations.insert(Designations::FORETOLD);
        exile_obj.foretold_turn = current_turn;
    }
    // Emit CardForetold event.
    events.push(GameEvent::CardForetold {
        player,
        object_id: card,
        new_exile_id,
    });
    Ok(events)
}
