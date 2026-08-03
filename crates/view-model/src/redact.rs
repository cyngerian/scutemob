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
//! | Stack | the *zone* is public, but a face-down spell's identity is not, and neither is a source or target the viewer may not identify | CR 405.1, CR 702.36b, CR 708.2 |
//! | Combat | an attacker, a blocker or an attacked planeswalker the viewer may not identify is name-redacted | CR 508.1, CR 509.1, CR 708.2 |
//! | Graveyard, command zone | public, untouched | CR 404.1, CR 903.6 |
//!
//! # Every surface that can identify a card must be listed here
//!
//! The first cut of this module redacted `hand`, `battlefield` and `exile` and
//! stopped, because those are the zones CR calls hidden. That was the wrong unit
//! of analysis: **the leak follows the rendering site, not the zone.** A morph
//! creature that attacks is on the battlefield, so the battlefield redaction
//! "covers" it in the zone sense while `combat.attackers[i].name` prints
//! "Exalted Angel" to the whole table.
//!
//! The complete inventory of sites in `lib.rs` that can identify a card, and the
//! disposition of each:
//!
//! | Site | Reads | Redacted? |
//! |------|-------|-----------|
//! | `PermanentView::name` | `calculate_characteristics` | yes (belt-and-braces; layers already blank it) |
//! | `PermanentView::is_commander` | raw `obj.card_id` | **yes — a live leak layers cannot close**, see `redact_face_down_permanents` |
//! | `objects_in_zone_as_card_views` (hand/graveyard/exile/command) | raw `characteristics.name` | hand + exile yes; graveyard and command zone are public |
//! | `StackItemView::source_name` | raw `characteristics.name` | yes |
//! | `format_target` | raw `characteristics.name` | yes (object targets only — a player target is public, CR 102.1 / 115.1 / 400.2) |
//! | `AttackerView::name` and its planeswalker `target` | raw `characteristics.name` | yes |
//! | `BlockerView::name` | raw `characteristics.name` | yes |
//! | `PlayerView::commander_damage_received` (inner keys) | raw `characteristics.name` | **no, and correctly so** — a non-zero entry requires that commander to have dealt combat damage, at which point CR 903.10a makes the association public in paper too. Same information, same timing. |
//!
//! If a field is added to `StateViewModel` that renders *or derives* a card
//! identity, it belongs in this table with a disposition. `is_commander` is the
//! reminder that "renders a name" is too narrow a test: it renders a boolean and
//! leaks a name.
//!
//! # Known conservatism (safe direction, wrong log)
//!
//! `viewer_may_identify` denies any object in another seat's hand
//! unconditionally, so a stack source that is legitimately *revealed from hand* —
//! Miracle (CR 702.94a), Forecast (CR 702.56a) — renders as "Face-down card" to
//! every other seat. Likewise a source or combatant that has already left
//! `state.objects()` (CR 400.7) denies rather than guesses, where the omniscient
//! view shows `"unknown"` / `"object_<id>"`. Correct as a leak posture, wrong as
//! a game log; S6 will want to narrow it, and narrowing it means adding a
//! "publicly revealed" notion the engine does not currently track.
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

use mtg_engine::{AttackTarget, GameState, ObjectId, PlayerId, Target, ZoneId};

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

/// What a spell or ability target the viewer may not identify is called.
///
/// Keeps the `"<kind>:<label>"` shape the omniscient renderer uses
/// (`format_target`), so a client parsing the prefix does not need a special
/// case — it just gets a label it cannot resolve to a card.
pub(crate) const HIDDEN_TARGET: &str = "object:Face-down card";

/// Apply every Architecture Invariant 7 redaction for `seat`, in place.
///
/// Called only from `StateViewModel::from_game_state_for`.
pub(crate) fn redact_state_for_seat(view: &mut StateViewModel, state: &GameState, seat: PlayerId) {
    redact_hands(view, state, seat);
    redact_face_down_permanents(view, state, seat);
    redact_face_down_exile(view, state, seat);
    redact_stack(view, state, seat);
    redact_combat(view, state, seat);

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
///
/// `is_commander` is cleared for the same object, and it is **not** belt-and-
/// braces: it is a live leak the layer system structurally cannot close.
/// `build_zones_view` derives it from the raw `obj.card_id` (`lib.rs`), and
/// CR 903.3 calls the commander designation "an attribute of the card itself",
/// not a characteristic — which is exactly why CR 708.2a's override does not
/// touch it and why `calculate_characteristics` cannot. So on a face-down
/// permanent every characteristic comes back blanked and `is_commander: true`
/// survives, naming the card outright: an opponent knows which card in the game
/// is your commander (CR 903.6, it started in the command zone), so the flag
/// resolves the identity to exactly one card the instant the permanent enters.
/// Reachable whenever a commander with morph, megamorph or disguise is cast face
/// down. Clearing it is correct, not merely conservative: in paper the opponents
/// see a face-down card and cannot tell it apart from any other.
///
/// NOTE on the split predicate: this function keys on `status.face_down` alone,
/// while the CR 708.2a layer override also requires `face_down_as.is_some()`.
/// If a battlefield permanent ever had the former without the latter, layers
/// would not blank it and this function substitutes only `name` — `subtypes`,
/// `keywords`, `power`/`toughness` and `supertypes` would ship printed values.
/// Unreachable today: the only two `status.face_down = true` sites with no
/// companion `face_down_as` are exile-bound (`foretell.rs`, `resolution.rs`) and
/// are covered fully by `redact_face_down_exile`. Recorded because the
/// belt-and-braces argument above refuses to depend on the layer system, and
/// then does depend on it for five of the six identifying fields.
fn redact_face_down_permanents(view: &mut StateViewModel, state: &GameState, seat: PlayerId) {
    for permanents in view.zones.battlefield.values_mut() {
        for permanent in permanents.iter_mut() {
            if is_face_down_not_owned_by(state, ObjectId(permanent.object_id), seat) {
                permanent.name = FACE_DOWN_NAME.to_string();
                permanent.is_commander = false;
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

/// CR 405.1: the stack is a public zone, so *that* a spell is there is public —
/// but CR 702.36b lets a spell be cast face down, and a source or a target can be
/// any object at all, including a face-down permanent or (through a synthetic or
/// mid-move state) a card in a hidden zone.
///
/// `StackItemView::source_name` and each entry of `targets` are built in
/// `build_zones_view` from a raw `obj.characteristics.name` read, with no layer
/// pass, so both keep the printed name of a face-down object.
///
/// Stack entries are matched back to their `StackObject` **by id, not by index**.
/// The two are 1:1 and in order by construction today, but positional
/// correspondence between two separately-derived lists is the trick that caused
/// the Monastery Mentor filter-bypass bug (see the "Index-namespace fix" note in
/// `testing/replay_harness.rs`); an id match cannot drift. Within a single
/// `StackObject`, `targets` *is* index-correspondent — both the view's vector and
/// the engine's are the same `so.targets`, mapped in place.
fn redact_stack(view: &mut StateViewModel, state: &GameState, seat: PlayerId) {
    for item in view.zones.stack.iter_mut() {
        let Some(stack_object) = state.stack_objects().iter().find(|so| so.id.0 == item.id) else {
            // No matching engine object: we cannot establish entitlement for
            // anything on this entry, so deny both surfaces rather than guess.
            item.source_name = FACE_DOWN_NAME.to_string();
            for target in item.targets.iter_mut() {
                *target = HIDDEN_TARGET.to_string();
            }
            continue;
        };

        let (_, source_id) = crate::stack_kind_info(&stack_object.kind);
        if let Some(source_id) = source_id {
            if !viewer_may_identify(state, source_id, seat) {
                item.source_name = FACE_DOWN_NAME.to_string();
            }
        }

        for (rendered, spell_target) in item.targets.iter_mut().zip(stack_object.targets.iter()) {
            // A player target is always public (CR 102.1 / 115.1 / 400.2) — only
            // object targets can carry an identity the viewer is not entitled to.
            if let Target::Object(object_id) = spell_target.target {
                if !viewer_may_identify(state, object_id, seat) {
                    *rendered = HIDDEN_TARGET.to_string();
                }
            }
        }
    }
}

/// CR 508.1 / CR 509.1: which creatures are attacking and blocking is public, and
/// so is which player or planeswalker is being attacked — but a face-down
/// creature can attack and block (CR 708.2), and its identity stays hidden while
/// it does.
///
/// `build_combat_view` reads `obj.characteristics.name` raw for the attacker, the
/// attacked planeswalker and every blocker, so all three keep the printed name.
///
/// The attacked planeswalker's id is taken from `CombatState::attackers` rather
/// than parsed back out of the rendered `"planeswalker:<name>"` string — parsing
/// a display string to recover an id is how a redaction gets silently skipped by
/// a name containing the separator.
fn redact_combat(view: &mut StateViewModel, state: &GameState, seat: PlayerId) {
    let Some(combat_view) = view.combat.as_mut() else {
        return;
    };
    let engine_combat = state.combat().as_ref();

    for attacker in combat_view.attackers.iter_mut() {
        let attacker_id = ObjectId(attacker.object_id);
        if !viewer_may_identify(state, attacker_id, seat) {
            attacker.name = FACE_DOWN_NAME.to_string();
        }

        // CR 508.1a: an attack target may be a planeswalker, whose identity is
        // hidden if it is face down.
        if let Some(AttackTarget::Planeswalker(pw_id)) =
            engine_combat.and_then(|c| c.attackers.get(&attacker_id))
        {
            if !viewer_may_identify(state, *pw_id, seat) {
                attacker.target = format!("planeswalker:{FACE_DOWN_NAME}");
            }
        }

        for blocker in attacker.blockers.iter_mut() {
            if !viewer_may_identify(state, ObjectId(blocker.object_id), seat) {
                blocker.name = FACE_DOWN_NAME.to_string();
            }
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
