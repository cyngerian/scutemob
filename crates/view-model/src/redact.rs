//! Seat-scoped redaction of the view model — Architecture Invariant 7.
//!
//! The engine knows everything. This module is one of the two chokepoints where
//! that knowledge is narrowed to what a single seat is entitled to see before it
//! leaves the process (the other is the `event_view` module). Everything here is
//! reached only from `StateViewModel::from_game_state_for` with
//! `Viewer::Seat`; the `Omniscient` path never calls into this module at all,
//! so the replay viewer is unaffected by construction.
//!
//! # The rules, and why each one is here
//!
//! | Zone | Rule | CR |
//! |------|------|----|
//! | Hand (another seat's) | every card becomes an anonymous placeholder; the *count* survives | CR 402.1 |
//! | Library (any seat's, including the viewer's own) | never enumerated at all | CR 401.2 |
//! | Battlefield | a face-down permanent the viewer does not own is name-redacted | CR 708.2 |
//! | Exile | a face-down exiled card the viewer does not own is name-redacted | CR 708.2, CR 406.3 |
//! | Graveyard, command zone, stack | public, untouched | CR 404.1, CR 903.6, CR 405.1 |
//!
//! # Ownership, not control
//!
//! Face-down redaction keys on `obj.owner`. Exile is a single shared zone
//! (CR 406.1) with no per-player partition, so control is not even available
//! there for a card that is not a permanent; and for a *stolen* face-down
//! permanent, owner is the conservative key — the thief is denied the name even
//! though CR 708.5a would let them look. Denying too much never leaks; the
//! reverse does. (A future, more precise pass may want control for battlefield
//! and owner for exile; the test pins the current behaviour either way.)

use std::collections::HashMap;

use mtg_engine::{GameState, ObjectId, PlayerId, ZoneId};

use crate::{CardInZoneView, StateViewModel};

/// What another seat's hand card is called in a redacted view.
///
/// CR 402.1: the hand is a hidden zone. One placeholder is emitted per real
/// card, so `hand.len()` and `PlayerView`'s `hand_size` still agree and the client
/// can render the right number of card backs.
pub(crate) const HIDDEN_CARD_NAME: &str = "Hidden card";

/// What a face-down object the viewer may not identify is called.
///
/// Deliberately distinct from `HIDDEN_CARD_NAME`: the viewer *can* see that a
/// face-down permanent exists and where it is (it is a public object, CR 708.2),
/// they simply may not know which card it is.
pub(crate) const FACE_DOWN_NAME: &str = "Face-down card";

/// Apply every Architecture Invariant 7 redaction for `seat`, in place.
///
/// Called only from `StateViewModel::from_game_state_for`.
pub(crate) fn redact_state_for_seat(view: &mut StateViewModel, state: &GameState, seat: PlayerId) {
    redact_hands(view, state, seat);
    redact_face_down_permanents(view, state, seat);
    redact_face_down_exile(view, state, seat);
    // Libraries: nothing to do, and that is the point. `ZonesView` has no
    // library field and `PlayerView::library_size` is a count, so library ORDER
    // and library CONTENTS are unrepresentable in this view model (CR 401.2).
    // `test_seat_view_never_enumerates_any_library` pins that, so a future
    // library field cannot be added here without the gate reddening.
}

/// CR 402.1: a player's hand is hidden from every other player.
///
/// Each of another seat's hand cards is replaced by an anonymous placeholder
/// with `object_id: 0` — the id itself is a handle onto a hidden object and is
/// not the viewer's to hold. The placeholder count matches the real count, so no
/// information beyond "how many cards" (which is public, CR 402.2) is lost.
///
/// The viewer's own hand is left exactly as the omniscient view built it.
fn redact_hands(view: &mut StateViewModel, state: &GameState, seat: PlayerId) {
    let own_hand = ZoneId::Hand(seat);
    for cards in view.zones.hand.values_mut() {
        for card in cards.iter_mut() {
            // Resolve the entry back to its engine object rather than trusting
            // the player-name key: names are display strings and could collide.
            let is_own = state
                .objects()
                .get(&ObjectId(card.object_id))
                .map(|obj| obj.zone == own_hand)
                .unwrap_or(false);
            if !is_own {
                *card = hidden_placeholder();
            }
        }
    }
}

/// CR 708.2: a face-down permanent has no name; only players entitled to look at
/// it know which card it is.
///
/// NOTE (verified 2026-08-01, and the reason this is belt-and-braces rather than
/// dead code): today the omniscient view ALREADY renders a face-down permanent's
/// name as the empty string, because `build_zones_view` runs each permanent
/// through `calculate_characteristics` and the layer system applies the CR 708.2a
/// face-down override for everyone. This function therefore changes `""` into a
/// displayable `"Face-down card"` rather than removing a live leak. It exists so
/// that Invariant 7 does not silently depend on the layer system continuing to
/// blank that name — if a future layer change ever stopped doing so, the leak
/// would appear here, in the seat view, with nothing to catch it.
fn redact_face_down_permanents(view: &mut StateViewModel, state: &GameState, seat: PlayerId) {
    for permanents in view.zones.battlefield.values_mut() {
        for permanent in permanents.iter_mut() {
            if is_face_down_not_owned_by(state, ObjectId(permanent.object_id), seat) {
                permanent.name = FACE_DOWN_NAME.to_string();
            }
        }
    }
}

/// CR 708.2 / CR 406.3: exile is a shared, mostly public zone, but a card exiled
/// FACE DOWN is not identifiable by players who are not entitled to look at it
/// (foretell CR 702.143a, cleave/plot-style face-down exile, morph exile costs).
///
/// This one is a real leak in the omniscient view:
/// `objects_in_zone_as_card_views` reads `obj.characteristics.name` raw, with no
/// layer pass, so a face-down exiled card ships its printed name today.
fn redact_face_down_exile(view: &mut StateViewModel, state: &GameState, seat: PlayerId) {
    for card in view.zones.exile.iter_mut() {
        if is_face_down_not_owned_by(state, ObjectId(card.object_id), seat) {
            card.name = FACE_DOWN_NAME.to_string();
            card.card_types = vec![];
            card.hidden = true;
        }
    }
}

/// `true` when `id` names an object that is currently face down and owned by
/// someone other than `seat`.
///
/// `status.face_down` is the truth for "is this face down right now";
/// `face_down_as: Option<FaceDownKind>` is only metadata about *how* it got that
/// way (morph, manifest, cloak, foretell…) and is not consulted here.
fn is_face_down_not_owned_by(state: &GameState, id: ObjectId, seat: PlayerId) -> bool {
    state
        .objects()
        .get(&id)
        .map(|obj| obj.status.face_down && obj.owner != seat)
        .unwrap_or(false)
}

/// The anonymous stand-in for a card in another seat's hidden zone (CR 402.1).
fn hidden_placeholder() -> CardInZoneView {
    CardInZoneView {
        object_id: 0,
        name: HIDDEN_CARD_NAME.to_string(),
        card_types: vec![],
        hidden: true,
    }
}

/// Whether the viewer is entitled to identify an object, used by the
/// `event_view` module to decide whether an event may name a card.
///
/// Returns `true` when `id` names an object the viewer may identify: anything
/// not in a hidden zone belonging to someone else, and not a face-down object
/// they do not own.
pub(crate) fn viewer_may_identify(state: &GameState, id: ObjectId, seat: PlayerId) -> bool {
    let Some(obj) = state.objects().get(&id) else {
        // CR 400.7: the object is gone (it changed zones and became a new
        // object). We cannot establish entitlement, so we deny it.
        return false;
    };
    if obj.status.face_down && obj.owner != seat {
        return false;
    }
    match obj.zone {
        // CR 402.1 / CR 401.2: hidden zones. Only the owning seat may identify.
        ZoneId::Hand(p) | ZoneId::Library(p) => p == seat,
        // Everything else is public: battlefield, graveyard, exile, command,
        // stack. (Face-down-ness was already handled above.)
        _ => true,
    }
}

/// Resolve a `PlayerId` to its display name, falling back to the same
/// `player_<n>` shape the rest of the view model uses.
pub(crate) fn player_display_name(
    pid: PlayerId,
    player_names: &HashMap<PlayerId, String>,
) -> String {
    player_names
        .get(&pid)
        .cloned()
        .unwrap_or_else(|| format!("player_{}", pid.0))
}
