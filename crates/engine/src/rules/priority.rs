//! Priority system: APNAP ordering, priority passing (CR 116-117).

use im::OrdSet;

use crate::state::player::PlayerId;
use crate::state::GameState;

use super::events::GameEvent;

/// Result of a priority pass action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PriorityResult {
    /// Another player now has priority.
    PlayerHasPriority { player: PlayerId },
    /// All active players have passed in succession.
    AllPassed,
}

/// CR 116.3d: "If all players pass in succession (that is, if all players pass
/// without any player taking an action in between passing), the spell or ability
/// on top of the stack resolves or, if the stack is empty, the phase or step ends."
///
/// Validates that `player` is the current priority holder, adds them to
/// the passed set, and determines what happens next.
pub fn pass_priority(
    state: &GameState,
    player: PlayerId,
) -> Result<(PriorityResult, Vec<GameEvent>), crate::state::error::GameStateError> {
    // Validate the player is the priority holder
    let holder = state.turn.priority_holder;
    if holder != Some(player) {
        return Err(crate::state::error::GameStateError::NotPriorityHolder {
            expected: holder,
            actual: player,
        });
    }

    let mut events = vec![GameEvent::PriorityPassed { player }];

    // Add to passed set
    let mut passed = state.turn.players_passed.clone();
    passed.insert(player);

    // Check if all active players have passed
    let active_players = state.active_players();
    let all_passed = active_players.iter().all(|p| passed.contains(p));

    if all_passed {
        events.push(GameEvent::AllPlayersPassed);
        Ok((PriorityResult::AllPassed, events))
    } else {
        // Find next player in APNAP order
        let next = next_priority_player(state, player)
            .expect("at least one active player should not have passed");
        events.push(GameEvent::PriorityGiven { player: next });
        Ok((PriorityResult::PlayerHasPriority { player: next }, events))
    }
}

/// CR 116.3: "Which player has priority is determined by the following rules:"
/// Priority passes in APNAP order (Active Player, Non-Active Player).
///
/// In multiplayer, APNAP order is: active player, then clockwise from active.
/// Skip eliminated players and players who have already passed.
pub fn next_priority_player(state: &GameState, current: PlayerId) -> Option<PlayerId> {
    let order = &state.turn.turn_order;
    let len = order.len();
    if len == 0 {
        return None;
    }

    let current_pos = order.iter().position(|&p| p == current)?;

    for offset in 1..=len {
        let idx = (current_pos + offset) % len;
        let candidate = order[idx];

        // Skip eliminated players
        if let Some(player) = state.players.get(&candidate) {
            if player.has_lost || player.has_conceded {
                continue;
            }
        } else {
            continue;
        }

        // Skip players who already passed
        if state.turn.players_passed.contains(&candidate) {
            continue;
        }

        return Some(candidate);
    }

    None
}

/// Grant initial priority to the active player at the start of a step.
/// Resets the passed set.
pub fn grant_initial_priority(state: &GameState) -> (OrdSet<PlayerId>, Vec<GameEvent>) {
    let active = state.turn.active_player;
    let events = vec![GameEvent::PriorityGiven { player: active }];
    (OrdSet::new(), events)
}
