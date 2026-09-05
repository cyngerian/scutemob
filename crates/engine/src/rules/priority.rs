//! Priority system: APNAP ordering, priority passing (CR 116-117).
use super::events::GameEvent;
use crate::state::player::PlayerId;
use crate::state::GameState;
use imbl::OrdSet;
/// Result of a priority pass action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PriorityResult {
    /// Another player now has priority.
    PlayerHasPriority { player: PlayerId },
    /// All active players have passed in succession.
    AllPassed,
}
/// CR 117.4: "If all players pass in succession (that is, if all players pass
/// without taking any actions in between passing), the spell or ability on top of the
/// stack resolves or, if the stack is empty, the phase or step ends."
/// CR 117.3d: the passing player announces floating mana, then the next player in turn
/// order receives priority.
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
        // Find next player in APNAP order — MR-M2-01: typed error instead of expect.
        let next = next_priority_player(state, player)
            .ok_or(crate::state::error::GameStateError::NoActivePlayers)?;
        events.push(GameEvent::PriorityGiven { player: next });
        Ok((PriorityResult::PlayerHasPriority { player: next }, events))
    }
}
/// CR 117.3: "Which player has priority is determined by the following rules:"
/// CR 117.3d: the next player in turn order receives priority.
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
        if let Some(player) = state.expect_player(candidate) {
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
/// CR 117.3a / CR 117.3b, with CR 800.4j: give priority to the active player --
/// **unless the active player has left the game**, in which case "the next player
/// in turn order receives priority" (CR 800.4j).
///
/// This is the shared answer for every site that starts a fresh priority round on
/// the active player *in the middle of a turn* (after a resolution, after a
/// turn-based action). It resets `players_passed`, because a new round starts here
/// whoever receives it.
///
/// # Why this exists (closing-review HIGH-1, second-closing-review HIGH-1)
///
/// Several sites wrote `priority_holder = Some(state.turn.active_player)` with no
/// liveness test, while `enter_step`'s two mid-turn grants and
/// `handle_all_passed`'s forced-payment grant had carried one for a long time
/// ("Active player lost (e.g., drew from empty library)"). The unconditional
/// sites hand priority to a seat that has left the game, and the resulting state
/// is an **unrecoverable deadlock**: `blocking_decision` is `None`, so
/// `PassPriority` is admitted, and it answers `PlayerEliminated` from the
/// departed seat and `NotPriorityHolder` from every other seat. Every driving
/// loop (`LocalGame::advance`, `GameDriver`, the TUI auto-pass, the fuzzer) dies
/// there.
///
/// The two reachable ones are fixed by routing through here, each with its own
/// fail-before probe:
///
/// * `resolution::resolve_top_of_stack_inner`'s CR 117.3b tail and CR 608.2b
///   fizzle path. `sba::check_and_apply_sbas` runs a few lines above the tail, so
///   *any* resolution that eliminates the active player -- lethal damage, a
///   mill-out, 21 commander damage -- arrives with `active.has_lost` already
///   true. Probes:
///   `test_dp9_resolution_grant_skips_an_active_player_killed_by_an_sba`,
///   `test_dp9_active_player_concedes_under_a_foreign_block`.
/// * `combat::handle_declare_blockers`'s CR 509.1 / 117.3a tail, reachable
///   whenever the active player is eliminated during its own combat phase.
///   Probe: `test_509_declare_blockers_grant_skips_a_departed_active_player`.
///
/// # The full inventory of active-player grants (mechanically enumerated)
///
/// ```text
/// grep -rn 'priority_holder = ' crates/*/src tools/*/src
/// ```
///
/// Every production write of `priority_holder` in the workspace, classified.
/// **Note what this corrects:** the closing review's fix comment claimed its two
/// `resolve_top_of_stack_inner` sites were "the only place in the engine" that
/// granted unconditionally, and that `enter_step`'s grants "all carried the
/// liveness test". Both are false, which is why the list is written out here.
///
/// * **Through this helper** (CR 800.4j honoured): `resolution.rs`'s two
///   `resolve_top_of_stack_inner` grants (the CR 117.3b tail and the CR 608.2b
///   fizzle path), `resolution.rs`'s `counter_stack_object` tail,
///   `combat.rs`'s `handle_declare_blockers` CR 509.1 / 117.3a tail, and
///   (PB-DX56 F3, closing `OOS-DP9-19`) `engine.rs`'s **cleanup-SBA-round
///   grant inside `enter_step`** -- the third grant in that function, and the
///   one the closing review's "all carried the liveness test" claim missed.
/// * **A second, independent implementation of the same test**:
///   `abilities::grant_priority_after_batch` (PB-DP8's CR 603.3b batch grant,
///   also used by `repair_departed_priority_holder`). Same liveness rule, kept
///   separate because it resets `players_passed` only on the granting branches.
/// * **Already conditional, inline**: `engine.rs`'s `handle_all_passed`
///   forced-payment re-grant and `enter_step`'s ordinary step grant -- each has
///   its own `has_lost` / `has_conceded` test plus a `next_priority_player`
///   fallback.
/// * **Grants to the ACTOR, not the active player** (`= Some(player)`, PB-DP1's
///   CR 117.3c priority-to-actor rule): `engine.rs` ×4, `combat.rs`'s
///   `handle_declare_attackers`, `abilities.rs` ×12, `casting.rs` ×1. The actor
///   just issued a legal command, so it holds by construction; not this helper's
///   business.
/// * **Clears** (`= None`): `turn_structure.rs` ×2, plus the no-live-seat branch
///   of every conditional grant including this one.
/// * **Still unconditional, deliberately not routed here**:
///   * `resolution.rs`'s cipher-copy grant (`StackObjectKind::KeywordTrigger` /
///     `KeywordAbility::Cipher`). Benign: the arm falls through to the CR 117.3b
///     tail, which overwrites the field before the command returns, and the write
///     pushes no `PriorityGiven`, so routing it would add an event to the stream
///     for no behavioural gain.
///
/// The classification above is a snapshot of that `grep`, not a machine-checked
/// invariant: a new unconditional grant added tomorrow will not fail any gate.
pub(crate) fn grant_priority_to_active_player(state: &mut GameState, events: &mut Vec<GameEvent>) {
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    // SR-25: a departed player legitimately answers `false` here -- that is the
    // question being asked, not a swallowed miss.
    let active_is_alive = state
        .expect_player(active)
        .map(|p| !p.has_lost && !p.has_conceded)
        .unwrap_or(false);
    if active_is_alive {
        state.turn.priority_holder = Some(active);
        events.push(GameEvent::PriorityGiven { player: active });
    } else if let Some(next) = next_priority_player(state, active) {
        // CR 800.4j: "If the active player would receive priority, instead the
        // next player in turn order receives priority."
        state.turn.priority_holder = Some(next);
        events.push(GameEvent::PriorityGiven { player: next });
    } else {
        // No live seat left to receive it. `check_game_over` (the caller's own
        // follow-up, or the next SBA sweep) ends the game.
        state.turn.priority_holder = None;
    }
}
