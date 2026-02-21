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
//! Cost payment and target selection are deferred to later milestones (M3-D).

use im::OrdSet;

use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::stack::{StackObject, StackObjectKind};
use crate::state::turn::Step;
use crate::state::types::{CardType, KeywordAbility};
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::events::GameEvent;

/// Handle a CastSpell command: move a card from hand to the stack.
///
/// Validates the casting window, moves the card to `ZoneId::Stack`, creates
/// a `StackObject`, resets priority to the active player (CR 601.2i), and
/// returns the events produced.
///
/// M3-B: no cost payment, no targets. Those are validated in later milestones.
pub fn handle_cast_spell(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // CR 601.2: Casting a spell requires priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // Fetch the card and validate it is in the player's hand.
    let card_obj = state.object(card)?;
    if card_obj.zone != ZoneId::Hand(player) {
        return Err(GameStateError::InvalidCommand(
            "card is not in your hand".into(),
        ));
    }

    // Lands are not cast — they are played as a special action (CR 305.1).
    if card_obj
        .characteristics
        .card_types
        .contains(&CardType::Land)
    {
        return Err(GameStateError::InvalidCommand(
            "lands are played with PlayLand, not cast".into(),
        ));
    }

    // Determine casting speed (CR 601.3).
    let is_instant_speed = card_obj
        .characteristics
        .card_types
        .contains(&CardType::Instant)
        || card_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Flash);

    // Validate casting window.
    if !is_instant_speed {
        // Sorcery speed: active player only, main phase, empty stack (CR 307.1).
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "sorcery-speed spells can only be cast during your own turn".into(),
            ));
        }
        if !matches!(
            state.turn.step,
            Step::PreCombatMain | Step::PostCombatMain
        ) {
            return Err(GameStateError::NotMainPhase);
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::StackNotEmpty);
        }
    }

    // CR 601.2c: Move the card to the Stack zone (CR 400.7: new ObjectId).
    let (new_card_id, _old_obj) = state.move_object_to_zone(card, ZoneId::Stack)?;

    // CR 601.2: Create the StackObject and push it (LIFO — last in, first out).
    let stack_entry_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_entry_id,
        controller: player,
        kind: StackObjectKind::Spell {
            source_object: new_card_id,
        },
    };
    state.stack_objects.push_back(stack_obj);

    // CR 601.2i: "Then the active player receives priority."
    // Reset the priority round — a game action occurred.
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(state.turn.active_player);

    Ok(vec![
        GameEvent::SpellCast {
            player,
            stack_object_id: stack_entry_id,
            source_object_id: new_card_id,
        },
        GameEvent::PriorityGiven {
            player: state.turn.active_player,
        },
    ])
}
