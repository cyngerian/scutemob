//! Classified state lookups: *impossible absence* vs *expected fizzle*.
//!
//! # Why this module exists
//!
//! The effect-resolution layer is full of lookups whose failure is silently
//! discarded:
//!
//! ```ignore
//! if let Some(obj) = state.objects.get_mut(&id) { obj.tapped = true; }
//! ```
//!
//! Read on its own, that line cannot tell you which of two very different things
//! is going on:
//!
//! 1. **Expected fizzle.** `id` is a *last known information* reference — a
//!    target that has since changed zones, a sacrificed source, a creature that
//!    died to a state-based action mid-resolution. Under CR 400.7 the old
//!    `ObjectId` names an object that no longer exists, and CR 608.2b says an
//!    effect that needs information about it "fails to determine any such
//!    information. Any part of the effect that requires that information won't
//!    happen." Doing nothing is *correct rules behavior*.
//!
//! 2. **Impossible absence.** `id` was pulled out of `state.objects` three lines
//!    earlier, or it is a `PlayerId`, which this engine never removes from
//!    `state.players`. Absence here means an engine invariant is broken. Doing
//!    nothing silently corrupts the game state and, worse, corrupts the replay
//!    history that rewind depends on (architecture invariant #2).
//!
//! Both compile to the same silent `else`-less `if let`. The bug hides behind
//! the fizzle.
//!
//! # The vocabulary
//!
//! Every lookup in the resolution path should name which case it is:
//!
//! | Absence means | Use | Behavior |
//! |---|---|---|
//! | engine bug | [`GameState::expect_object`] / [`expect_object_mut`](GameState::expect_object_mut) / [`expect_player`](GameState::expect_player) / [`expect_player_mut`](GameState::expect_player_mut) / [`expect_zone`](GameState::expect_zone) | `debug_assert!` fires; releases return `None` |
//! | legal game state | [`GameState::lki_object`] / [`lki_object_mut`](GameState::lki_object_mut) | returns `None`, no assert |
//!
//! Both families return `Option`, so converting an existing site is a one-token
//! change and the classification becomes a fact about the code rather than a fact
//! about a comment. In debug builds — which is every build `cargo test` produces —
//! a violated `expect_*` panics at the *call site* (`#[track_caller]`), which is
//! where the diagnosis is cheap, rather than surfacing many turns later as a
//! replay divergence.
//!
//! # Why `PlayerId` lookups are always `expect_*`
//!
//! `GameState::players` is populated once at game start and never has entries
//! removed — `rg 'players.remove'` finds nothing. A player who loses the game
//! (CR 104.2/104.3) is marked with `PlayerState::has_lost`; under CR 800.4a their
//! *objects* leave the game, but the `PlayerState` remains addressable so that
//! turn order, APNAP ordering and replay all keep working. Therefore a
//! `PlayerId` that misses is an invalid id, i.e. a bug — never a fizzle.
//!
//! # What is *not* a swallow-site
//!
//! `Option::unwrap_or` on a scalar characteristic is not a discarded lookup, it
//! is the model. `power: None` is how a characteristic-defining `*/*` creature is
//! represented (CR 208.2), and `.unwrap_or(0)` is the correct reading of it.
//! Likewise `ctx.damaged_player.unwrap_or(ctx.controller)` selects between two
//! valid sources, and `.map(..).unwrap_or(false)` is a predicate over a possibly
//! absent object. Those sites are deliberately left alone; see
//! `docs/sr-4-silent-failure-audit.md` for the full classification.

use super::{GameObject, GameState, GameStateError, ObjectId, PlayerId, PlayerState, Zone, ZoneId};

/// [`GameState::expect_object`]'s assertion, without the borrow.
///
/// A method call borrows the *whole* `GameState`, whereas `state.objects.get_mut(&id)`
/// borrows only the `objects` field, leaving `state.card_registry` and
/// `state.timestamp_counter` free to be read inside the block. A handful of sites
/// depend on that disjointness. There, assert first and keep the raw field access:
///
/// ```ignore
/// debug_assert_object_live!(state, new_id);
/// if let Some(obj) = state.objects.get_mut(&new_id) {
///     obj.last_transform_timestamp = state.timestamp_counter; // needs the disjoint borrow
/// }
/// ```
macro_rules! debug_assert_object_live {
    ($state:expr, $id:expr) => {
        debug_assert!(
            $state.objects.contains_key(&$id),
            "engine invariant: ObjectId {:?} absent from GameState::objects at a site that \
             requires it to be live. If this id can be last-known-information (CR 400.7), \
             treat the absence as a CR 608.2b fizzle instead of asserting.",
            $id
        );
    };
}
pub(crate) use debug_assert_object_live;

impl GameState {
    // ---------------------------------------------------------------------
    // Impossible-absence lookups. A `None` here is an engine bug.
    // ---------------------------------------------------------------------

    /// Look up a player whose existence is guaranteed by an engine invariant.
    ///
    /// `state.players` never loses entries (see [module docs](self)), so `None`
    /// means the `PlayerId` was fabricated or corrupted. Fires a `debug_assert!`
    /// and degrades to `None` in release builds.
    #[track_caller]
    pub(crate) fn expect_player(&self, id: PlayerId) -> Option<&PlayerState> {
        let found = self.players.get(&id);
        debug_assert!(
            found.is_some(),
            "engine invariant: PlayerId {id:?} absent from GameState::players. \
             Players are never removed (CR 800.4a removes their objects, not the player), \
             so this id is invalid."
        );
        found
    }

    /// Mutable [`expect_player`](GameState::expect_player).
    #[track_caller]
    pub(crate) fn expect_player_mut(&mut self, id: PlayerId) -> Option<&mut PlayerState> {
        let found = self.players.get_mut(&id);
        debug_assert!(
            found.is_some(),
            "engine invariant: PlayerId {id:?} absent from GameState::players. \
             Players are never removed (CR 800.4a removes their objects, not the player), \
             so this id is invalid."
        );
        found
    }

    /// Look up an object that the caller has already established is live —
    /// typically one whose id was just read out of `state.objects` or a zone,
    /// with no intervening zone change.
    ///
    /// A `None` means the object was destroyed, exiled or otherwise moved
    /// between the two reads (CR 400.7), which the caller's control flow claims
    /// cannot happen. Fires a `debug_assert!`.
    ///
    /// If the id *can* legitimately be stale, use [`lki_object`](GameState::lki_object).
    #[track_caller]
    pub(crate) fn expect_object(&self, id: ObjectId) -> Option<&GameObject> {
        let found = self.objects.get(&id);
        debug_assert!(
            found.is_some(),
            "engine invariant: ObjectId {id:?} absent from GameState::objects at a site that \
             requires it to be live. If this id can be last-known-information (CR 400.7), \
             use GameState::lki_object instead."
        );
        found
    }

    /// Mutable [`expect_object`](GameState::expect_object).
    #[track_caller]
    pub(crate) fn expect_object_mut(&mut self, id: ObjectId) -> Option<&mut GameObject> {
        let found = self.objects.get_mut(&id);
        debug_assert!(
            found.is_some(),
            "engine invariant: ObjectId {id:?} absent from GameState::objects at a site that \
             requires it to be live. If this id can be last-known-information (CR 400.7), \
             use GameState::lki_object_mut instead."
        );
        found
    }

    /// Look up a zone whose existence is guaranteed.
    ///
    /// Every zone a game can address is created by `GameStateBuilder` before turn
    /// one and none are ever removed, so `None` means a fabricated [`ZoneId`].
    #[track_caller]
    pub(crate) fn expect_zone(&self, id: &ZoneId) -> Option<&Zone> {
        let found = self.zones.get(id);
        debug_assert!(
            found.is_some(),
            "engine invariant: ZoneId {id:?} absent from GameState::zones. \
             All zones are created at game start and never removed."
        );
        found
    }

    /// Mutable [`expect_zone`](GameState::expect_zone).
    #[track_caller]
    pub(crate) fn expect_zone_mut(&mut self, id: &ZoneId) -> Option<&mut Zone> {
        let found = self.zones.get_mut(id);
        debug_assert!(
            found.is_some(),
            "engine invariant: ZoneId {id:?} absent from GameState::zones. \
             All zones are created at game start and never removed."
        );
        found
    }

    // ---------------------------------------------------------------------
    // Fallible mutations.
    //
    // `move_object_to_zone` has four error variants, and exactly one of them can
    // be a legal game state:
    //
    //   ObjectNotFound(id)          -- a CR 400.7 fizzle *or* a bug, per provenance
    //   ZoneNotFound(to)            -- always a bug; zones are never removed
    //   ZoneNotFound(from)          -- always a bug; ditto
    //   ObjectNotInZone(id, from)   -- always a bug; the object and its zone's
    //                                  contents disagree, i.e. corrupted state
    //
    // A stale (last-known-information) id is *fully removed* from `state.objects`,
    // so it surfaces as `ObjectNotFound` and never as `ObjectNotInZone`. That is
    // why `lki_move_object_to_zone` can silence `ObjectNotFound` alone and assert
    // on the rest without risking a panic on a legitimate fizzle.
    //
    // `add_object` fails only with `ZoneNotFound`, so a discarded `Err` from it is
    // unambiguously a bug. A discarded `Err` from a move needs the caller to say
    // which id provenance it has.
    // ---------------------------------------------------------------------

    /// Move an object the caller knows is live, into a zone that must exist.
    ///
    /// Every error variant is an engine bug here, so a `debug_assert!` fires and
    /// release builds get `None`. Returns the object's *new* `ObjectId` and its
    /// pre-move state, exactly as [`move_object_to_zone`](GameState::move_object_to_zone).
    #[track_caller]
    pub(crate) fn expect_move_object_to_zone(
        &mut self,
        object_id: ObjectId,
        to: ZoneId,
    ) -> Option<(ObjectId, GameObject)> {
        match self.move_object_to_zone(object_id, to) {
            Ok(moved) => Some(moved),
            Err(e) => {
                debug_assert!(
                    false,
                    "engine invariant: move of {object_id:?} to {to:?} failed ({e}) at a site \
                     that requires the object to be live and the zone to exist. If the id may be \
                     last-known-information (CR 400.7), use lki_move_object_to_zone."
                );
                None
            }
        }
    }

    /// Move an object whose id may be *last known information*.
    ///
    /// A missing object is the rules-correct fizzle of CR 400.7 / 608.2b and
    /// returns `None` quietly. The other three error variants — a missing source or
    /// destination zone, or an object whose zone disagrees with that zone's contents
    /// — are corrupted state, never a legal fizzle, and still assert.
    #[track_caller]
    pub(crate) fn lki_move_object_to_zone(
        &mut self,
        object_id: ObjectId,
        to: ZoneId,
    ) -> Option<(ObjectId, GameObject)> {
        match self.move_object_to_zone(object_id, to) {
            Ok(moved) => Some(moved),
            // CR 400.7: the object changed zones since this id was captured, so it
            // is a different object now. CR 608.2b: the effect "fails to determine
            // any such information", and the part that needed it doesn't happen.
            Err(GameStateError::ObjectNotFound(_)) => None,
            Err(e) => {
                debug_assert!(
                    false,
                    "engine invariant: move of {object_id:?} to {to:?} failed with {e}. \
                     Only ObjectNotFound is a legal fizzle (CR 400.7); zones are never \
                     removed, and an object whose zone disagrees with that zone's \
                     contents is corrupted state."
                );
                None
            }
        }
    }

    /// [`expect_move_object_to_zone`](GameState::expect_move_object_to_zone) for the
    /// bottom-of-library variant.
    #[track_caller]
    pub(crate) fn expect_move_object_to_bottom_of_zone(
        &mut self,
        object_id: ObjectId,
        to: ZoneId,
    ) -> Option<(ObjectId, GameObject)> {
        match self.move_object_to_bottom_of_zone(object_id, to) {
            Ok(moved) => Some(moved),
            Err(e) => {
                debug_assert!(
                    false,
                    "engine invariant: move of {object_id:?} to bottom of {to:?} failed ({e}) at \
                     a site that requires the object to be live and the zone to exist."
                );
                None
            }
        }
    }

    /// Add a freshly-built object to a zone that must exist.
    ///
    /// `add_object`'s only error is `ZoneNotFound`, which is always a bug, so a
    /// discarded `Err` here can never be a fizzle.
    #[track_caller]
    pub(crate) fn expect_add_object(
        &mut self,
        object: GameObject,
        zone_id: ZoneId,
    ) -> Option<ObjectId> {
        match self.add_object(object, zone_id) {
            Ok(id) => Some(id),
            Err(e) => {
                debug_assert!(
                    false,
                    "engine invariant: add_object into {zone_id:?} failed ({e}). \
                     add_object's only error is a missing zone, and zones are never removed."
                );
                None
            }
        }
    }

    // ---------------------------------------------------------------------
    // Expected-fizzle lookups. A `None` here is legal game state.
    // ---------------------------------------------------------------------

    /// Look up an object by an id that may be *last known information*.
    ///
    /// Returns `None` — silently, and correctly — when the object has changed
    /// zones since the id was captured. Per CR 400.7 it became a new object with
    /// no relation to its previous existence, and per CR 608.2b an effect that
    /// "requires information about an illegal target ... fails to determine any
    /// such information. Any part of the effect that requires that information
    /// won't happen." CR 113.7a extends the same treatment to an ability whose
    /// source has left its zone.
    ///
    /// This is a plain lookup — it exists to *document* that the `None` branch is
    /// a rules-correct fizzle rather than a swallowed bug, and to keep the
    /// asserting [`expect_object`](GameState::expect_object) honest by giving
    /// legitimate stale reads somewhere else to go.
    pub(crate) fn lki_object(&self, id: ObjectId) -> Option<&GameObject> {
        self.objects.get(&id)
    }

    /// Mutable [`lki_object`](GameState::lki_object).
    pub(crate) fn lki_object_mut(&mut self, id: ObjectId) -> Option<&mut GameObject> {
        self.objects.get_mut(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{GameStateBuilder, ObjectSpec};

    fn two_player_state() -> GameState {
        GameStateBuilder::new()
            .add_player(PlayerId(0))
            .add_player(PlayerId(1))
            .build()
            .expect("two-player builder is valid")
    }

    /// A state with one creature on the battlefield; returns it and its id.
    fn state_with_a_creature() -> (GameState, ObjectId) {
        let state = GameStateBuilder::new()
            .add_player(PlayerId(0))
            .add_player(PlayerId(1))
            .object(
                ObjectSpec::creature(PlayerId(0), "Grizzly Bears", 2, 2)
                    .in_zone(ZoneId::Battlefield),
            )
            .build()
            .expect("builder is valid");
        let id = state
            .objects
            .iter()
            .find(|(_, o)| o.zone == ZoneId::Battlefield)
            .map(|(id, _)| *id)
            .expect("the creature was placed");
        (state, id)
    }

    #[test]
    fn expect_player_finds_a_real_player() {
        let state = two_player_state();
        assert!(state.expect_player(PlayerId(0)).is_some());
    }

    #[test]
    #[should_panic(expected = "PlayerId")]
    fn expect_player_panics_in_debug_on_a_fabricated_id() {
        let state = two_player_state();
        let _ = state.expect_player(PlayerId(99));
    }

    #[test]
    #[should_panic(expected = "ObjectId")]
    fn expect_object_panics_in_debug_on_a_dead_id() {
        let state = two_player_state();
        let _ = state.expect_object(ObjectId(9_999));
    }

    /// CR 400.7: a stale id is a legal thing to hold. `lki_object` must not assert.
    #[test]
    fn lki_object_returns_none_without_panicking() {
        let state = two_player_state();
        assert!(state.lki_object(ObjectId(9_999)).is_none());
    }

    /// The whole premise of the split, exercised end to end.
    ///
    /// CR 400.7: "An object that moves from one zone to another becomes a new object
    /// with no memory of, or relation to, its previous existence." So after a zone
    /// change the *old* id names nothing — that is the fizzle these helpers exist to
    /// distinguish from a bug. This test proves the situation is reachable rather than
    /// hypothetical: if `move_object_to_zone` ever stopped minting a new id, the
    /// `lki_*` family would be dead code and this assertion would catch it.
    #[test]
    fn a_zone_change_really_does_kill_the_old_object_id() {
        let (mut state, old_id) = state_with_a_creature();
        let (new_id, _) = state
            .move_object_to_zone(old_id, ZoneId::Graveyard(PlayerId(0)))
            .expect("battlefield -> graveyard is a legal move");

        assert_ne!(
            new_id, old_id,
            "CR 400.7: the graveyard card is a new object"
        );
        assert!(
            state.lki_object(old_id).is_none(),
            "CR 400.7: the old id must name nothing"
        );
        assert!(state.lki_object(new_id).is_some());
    }

    /// A move through a dead id is the CR 400.7 / 608.2b fizzle: silent `None`.
    #[test]
    fn lki_move_of_a_dead_id_fizzles_quietly() {
        let (mut state, old_id) = state_with_a_creature();
        state
            .move_object_to_zone(old_id, ZoneId::Graveyard(PlayerId(0)))
            .expect("legal move");

        assert!(state
            .lki_move_object_to_zone(old_id, ZoneId::Exile)
            .is_none());
    }

    /// The same move through `expect_*` is an engine-bug claim, so it must fire.
    #[test]
    #[should_panic(expected = "requires the object to be live")]
    fn expect_move_of_a_dead_id_panics_in_debug() {
        let (mut state, old_id) = state_with_a_creature();
        state
            .move_object_to_zone(old_id, ZoneId::Graveyard(PlayerId(0)))
            .expect("legal move");

        let _ = state.expect_move_object_to_zone(old_id, ZoneId::Exile);
    }

    /// `lki_move_object_to_zone` tolerates a dead object but *not* a missing zone —
    /// that part of the error space is never a legal fizzle.
    #[test]
    #[should_panic(expected = "zones are never removed")]
    fn lki_move_still_asserts_on_a_fabricated_destination_zone() {
        let (mut state, id) = state_with_a_creature();
        let _ = state.lki_move_object_to_zone(id, ZoneId::Graveyard(PlayerId(99)));
    }

    /// The assumption that makes `lki_move_object_to_zone` safe.
    ///
    /// It silences `ObjectNotFound` and asserts on `ZoneNotFound` and
    /// `ObjectNotInZone`. That is only sound if a last-known-information id can never
    /// produce `ObjectNotInZone` — i.e. a zone change removes the old id from
    /// `state.objects` *entirely*, rather than leaving a stale entry whose `zone`
    /// field disagrees with that zone's contents. If `move_object_to_zone` ever
    /// stopped doing that, `lki_move_object_to_zone` would start panicking on a
    /// legitimate CR 400.7 fizzle. This test is the tripwire.
    #[test]
    fn a_stale_id_yields_object_not_found_not_object_not_in_zone() {
        let (mut state, old_id) = state_with_a_creature();
        state
            .move_object_to_zone(old_id, ZoneId::Graveyard(PlayerId(0)))
            .expect("legal move");

        match state.move_object_to_zone(old_id, ZoneId::Exile) {
            Err(GameStateError::ObjectNotFound(id)) => assert_eq!(id, old_id),
            other => panic!(
                "a stale id must surface as ObjectNotFound (CR 400.7), not {other:?} — \
                 lki_move_object_to_zone's assert would fire on a legal fizzle"
            ),
        }
    }

    #[test]
    fn expect_move_of_a_live_id_succeeds() {
        let (mut state, id) = state_with_a_creature();
        assert!(state
            .expect_move_object_to_zone(id, ZoneId::Graveyard(PlayerId(0)))
            .is_some());
    }

    #[test]
    #[should_panic(expected = "add_object")]
    fn expect_add_object_panics_on_a_fabricated_zone() {
        let (mut state, id) = state_with_a_creature();
        let obj = state.objects.get(&id).expect("live").clone();
        let _ = state.expect_add_object(obj, ZoneId::Graveyard(PlayerId(99)));
    }

    #[test]
    fn debug_assert_object_live_accepts_a_live_id() {
        let (state, id) = state_with_a_creature();
        debug_assert_object_live!(state, id);
    }

    #[test]
    #[should_panic(expected = "requires it to be live")]
    fn debug_assert_object_live_fires_on_a_dead_id() {
        let (mut state, old_id) = state_with_a_creature();
        state
            .move_object_to_zone(old_id, ZoneId::Graveyard(PlayerId(0)))
            .expect("legal move");
        debug_assert_object_live!(state, old_id);
    }

    #[test]
    #[should_panic(expected = "ZoneId")]
    fn expect_zone_panics_in_debug_on_a_fabricated_zone() {
        let state = two_player_state();
        let _ = state.expect_zone(&ZoneId::Graveyard(PlayerId(99)));
    }
}
