//! Escape hatches for constructing arbitrary [`GameState`] positions in tests.
//!
//! Compiled **only** under `cfg(test)` or the `test-util` cargo feature (SR-3).
//!
//! # Why this module exists
//!
//! [`GameState`]'s fields are `pub(crate)`, and its mutating methods
//! (`add_object`, `move_object_to_zone`, …) are `pub(crate)` too. Outside the
//! engine crate the only sanctioned way to change a `GameState` is to submit a
//! [`Command`](crate::rules::commands::Command) through `process_command`. That
//! is what turns architecture invariant #3 — *"there is no way to change game
//! state except through the Command enum"* — from a convention that reviewers
//! must police into a rule the compiler enforces.
//!
//! Tests and benchmarks legitimately need to build mid-game positions that no
//! cheap sequence of `Command`s can reach: a creature with damage already
//! marked, a specific `CombatState`, a stack captured mid-resolution. Rather
//! than re-open the fields for everyone, those needs are served here, by name,
//! in a module that cannot exist in a production build.
//!
//! # The guarantee
//!
//! `test-util` is off in any production build. The engine's own integration
//! tests and benches enable it through a self dev-dependency:
//!
//! ```toml
//! [dev-dependencies]
//! mtg-engine = { path = ".", features = ["test-util"] }
//! ```
//!
//! `cargo build --workspace` is the gate that proves the seal — it does not
//! build dev-dependencies, so `test-util` is off and this module does not
//! exist. If a production consumer (tui, replay-viewer, simulator, network)
//! ever reaches for one of these, that build fails.
//!
//! **Caveat:** under `--all-targets` (`cargo test --all`, `cargo clippy
//! --all-targets`) cargo unifies features across the workspace, so `test-util`
//! *is* enabled for every crate in that profile. Those commands cannot detect a
//! production consumer using an escape hatch; only `cargo build --workspace`
//! can. Keep it in the gate list.
//!
//! # Prefer the builder
//!
//! [`GameStateBuilder`](crate::state::GameStateBuilder) is the documented,
//! always-public constructor for setting up a position, and it is the right
//! tool most of the time. Reach for this module only for what it cannot say.
//!
//! Free functions rather than methods, deliberately: at the call site
//! `test_util::move_object_to_zone(&mut state, id, zone)` reads as an escape
//! hatch, and `rg 'test_util::'` enumerates every use of one.

use super::{
    GameObject, GameState, GameStateError, ObjectId, PlayerId, PlayerState, ReplacementId, ZoneId,
};

/// Escape hatch: allocate a fresh [`ObjectId`], advancing the timestamp counter.
///
/// Note that `timestamp_counter` **is** the object-id counter — rewinding it
/// aliases `ObjectId`s. See `memory/gotchas-infra.md`.
pub fn next_object_id(state: &mut GameState) -> ObjectId {
    state.next_object_id()
}

/// Escape hatch: allocate a fresh [`ReplacementId`].
pub fn next_replacement_id(state: &mut GameState) -> ReplacementId {
    state.next_replacement_id()
}

/// Escape hatch: mutable access to a single player.
pub fn player_mut(state: &mut GameState, id: PlayerId) -> Result<&mut PlayerState, GameStateError> {
    state.player_mut(id)
}

/// Escape hatch: mutable access to a single object.
pub fn object_mut(state: &mut GameState, id: ObjectId) -> Result<&mut GameObject, GameStateError> {
    state.object_mut(id)
}

/// Escape hatch: add an object to a zone, assigning it a fresh id and timestamp.
pub fn add_object(
    state: &mut GameState,
    object: GameObject,
    zone_id: ZoneId,
) -> Result<ObjectId, GameStateError> {
    state.add_object(object, zone_id)
}

/// Escape hatch: move an object between zones (CR 400.7 — the object becomes a
/// new object with a new [`ObjectId`]).
pub fn move_object_to_zone(
    state: &mut GameState,
    object_id: ObjectId,
    to: ZoneId,
) -> Result<(ObjectId, GameObject), GameStateError> {
    state.move_object_to_zone(object_id, to)
}

/// Escape hatch: move an object to the bottom of a zone (library ordering).
pub fn move_object_to_bottom_of_zone(
    state: &mut GameState,
    object_id: ObjectId,
    to: ZoneId,
) -> Result<(ObjectId, GameObject), GameStateError> {
    state.move_object_to_bottom_of_zone(object_id, to)
}
