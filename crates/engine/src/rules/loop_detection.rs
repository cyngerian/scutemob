//! Infinite loop detection for mandatory game loops (CR 726, CR 104.4b).
//!
//! CR 726: If a game situation arises where the game cannot proceed and all
//! remaining choices are mandatory (no player can choose to break the loop),
//! the game is a draw.
//!
//! CR 104.4b: A game is a draw if a player would be required to take an action
//! but is unable to do so, or if the game state reaches a situation where the
//! rules of the game prevent any further changes from occurring.
//!
//! ## Approach
//!
//! After each SBA + trigger cycle completes (a "mandatory round" — state changes
//! that occurred without any player making a choice), compute a hash of the
//! relevant game state (board, triggers, stack). If the same hash appears N times
//! (configurable threshold = 3), declare a mandatory infinite loop.
//!
//! The hash specifically covers:
//! - All game objects and their current zones
//! - Stack objects (spells and abilities waiting to resolve)
//! - Pending triggers (abilities waiting to go on the stack)
//! - Pending zone changes and replacements
//!
//! Library and hand contents are excluded (they are hidden and rarely change
//! during SBA-only cycles; including them would cause false negatives for
//! Reveillark + Karmic Guide style loops).
//!
//! ## When to Call
//!
//! Call `check_for_mandatory_loop` after each SBA + trigger flush cycle in
//! `engine.rs:enter_step`. Reset `loop_detection_hashes` whenever a player
//! makes a meaningful choice (any Command other than PassPriority during
//! an all-mandatory sequence).
//!
//! ## Threshold
//!
//! Threshold of 3 recurrences is used because some game patterns legitimately
//! repeat once (e.g., triggers resolving symmetrically) but true infinite loops
//! will always recur. 3 occurrences of the exact same state indicates a mandatory loop.

use crate::rules::events::GameEvent;
use crate::state::hash::HashInto;
use crate::state::GameState;
use blake3::Hasher;

/// Threshold: if the same game state hash is seen this many times during a
/// mandatory-action sequence, it is declared an infinite loop (CR 104.4b).
pub const LOOP_DETECTION_THRESHOLD: u32 = 3;

/// CR 104.4b: Check whether the current game state has been seen enough times
/// during a mandatory-action sequence to constitute a mandatory infinite loop.
///
/// Returns `Some(GameEvent::LoopDetected { ... })` if a loop is detected,
/// `None` otherwise.
///
/// The caller must also set all players as having lost (game draws) when
/// `Some` is returned.
///
/// The `loop_detection_hashes` field in `GameState` tracks occurrence counts.
/// This function both checks and updates that field.
///
/// ## MR-M9.4-12: Why &mut GameState
///
/// Conceptually this is a read-only check, but it mutates `state.loop_detection_hashes`
/// as a side effect to record the observation. Separating check from update would require
/// returning both the result and the new hash table and having the caller apply it — a
/// more complex interface for little gain. The mutation is intentional: `loop_detection_hashes`
/// is explicitly excluded from the public state hash (see gotchas-infra.md) and from the
/// HashInto implementation, so this mutation does not affect distributed consistency.
pub fn check_for_mandatory_loop(state: &mut GameState) -> Option<GameEvent> {
    let hash = compute_mandatory_state_hash(state);

    // Increment or insert the occurrence count for this hash
    let count = state.loop_detection_hashes.get(&hash).copied().unwrap_or(0) + 1;
    state.loop_detection_hashes.insert(hash, count);

    if count >= LOOP_DETECTION_THRESHOLD {
        Some(GameEvent::LoopDetected {
            description: format!(
                "Mandatory infinite loop detected: game state hash {:#018x} \
                 has recurred {} times (CR 104.4b, CR 726). Game is a draw.",
                hash, count
            ),
        })
    } else {
        None
    }
}

/// Reset the loop detection hash table.
///
/// Call this whenever a player makes a meaningful choice (any Command that
/// represents a real game decision, not just PassPriority during a
/// mandatory-action sequence). This ensures optional loops (where a player
/// has a choice to break the cycle) are not falsely flagged.
pub fn reset_loop_detection(state: &mut GameState) {
    state.loop_detection_hashes = im::OrdMap::new();
}

/// Compute a hash of the game state for loop detection purposes.
///
/// This is intentionally a restricted view of the state — it hashes only
/// the parts that change during mandatory SBA + trigger cycles:
/// - All game objects and their zones (the "board" including battlefield, graveyard, etc.)
/// - Stack objects (spells and abilities pending resolution)
/// - Pending triggers (abilities waiting to go on stack)
/// - Active continuous effects
/// - Pending zone changes
/// - Turn state (phase/step/priority — passive to track where we are)
///
/// Library and hand contents are EXCLUDED: they are hidden information and
/// changing hand/library is not a mandatory action. Including them would cause
/// false negatives.
///
/// The loop_detection_hashes field itself is EXCLUDED (it's metadata, not game state).
fn compute_mandatory_state_hash(state: &GameState) -> u64 {
    let mut hasher = Hasher::new();

    // 1. Turn state (phase/step only, not priority pass state)
    state.turn.phase.hash_into(&mut hasher);
    state.turn.step.hash_into(&mut hasher);
    state.turn.active_player.hash_into(&mut hasher);

    // 2. All game objects (board state) — sorted by ObjectId for determinism.
    // MR-M9.4-09: state.objects is an im::OrdMap which iterates in key order,
    // so no manual collect+sort is needed.
    for obj in state.objects.values() {
        // Only hash objects in public zones (battlefield, graveyard, exile, command)
        // Skip library/hand (hidden info)
        use crate::state::zone::ZoneId;
        match obj.zone {
            ZoneId::Hand(_) | ZoneId::Library(_) => continue,
            _ => {
                obj.hash_into(&mut hasher);
            }
        }
    }

    // 3. Stack objects (spells and abilities waiting to resolve)
    for so in &state.stack_objects {
        so.hash_into(&mut hasher);
    }

    // 4. Pending triggers (abilities waiting to be put on the stack)
    for pt in &state.pending_triggers {
        pt.hash_into(&mut hasher);
    }

    // 5. Active continuous effects (affect board state)
    for ce in &state.continuous_effects {
        ce.hash_into(&mut hasher);
    }

    // 6. Pending zone changes
    for pzc in &state.pending_zone_changes {
        pzc.hash_into(&mut hasher);
    }

    // Extract the first 8 bytes as a u64 (truncated hash for compact storage)
    let full_hash = hasher.finalize();
    let bytes = full_hash.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}
