//! Seat-scoped rendering of `GameEvent`s — Architecture Invariant 7.
//!
//! # Why this module exists, and its relationship to `GameEvent::private_to()`
//!
//! Architecture Invariant 7 says the engine knows everything and *the server*
//! filters events before broadcasting, private events going only to the relevant
//! player via `GameEvent::private_to() -> Option<PlayerId>`.
//!
//! That function now **exists** — PB-DP9 (`scutemob-157`) added it at
//! `crates/engine/src/rules/events.rs` — but its own doc block says plainly that
//! it is *"a declaration, not an enforcement point"*: nothing in the workspace
//! calls it except its tests, because the M10 centralized server that was meant
//! to consume it does not exist yet. It also currently classifies only two
//! variants as private (`EffectChoiceRequired`, `CleanupDiscardChoiceRequired`),
//! which is correct as far as it goes but is a per-*event* verdict, not a
//! per-*field* one: `GameEvent::CardDrawn` is public (everyone sees that a card
//! was drawn) while the identity of the card drawn is not.
//!
//! So M11-local does not ship raw serialized `GameEvent`s to the browser at all.
//! It ships **rendered, redacted lines** produced here. This module is the
//! M11-local stand-in for the M10 broadcast filter, and M10a should look here
//! first: the per-field entitlement logic (`redact::viewer_may_identify`) is the
//! part `private_to()` cannot express, and either it moves into the engine or
//! the server keeps a rendering layer like this one.
//!
//! # The rules
//!
//! 1. If `ev.private_to()` names a seat other than the viewer, the event is not
//!    rendered at all (`None`). This is the engine's own verdict, honoured
//!    first.
//! 2. An arm that would name a card checks `redact::viewer_may_identify` and
//!    falls back to a name-free line ("bob draws a card") when the viewer is not
//!    entitled to the identity. It never emits the name and then hides it
//!    client-side.
//! 3. The `_ =>` catch-all emits a **kind-only** line. The kind string is
//!    derived from the serde variant discriminant, so it is structurally
//!    incapable of interpolating a card name — there is no formatting of any
//!    payload field anywhere on that path.
//!
//! The `Viewer::Omniscient` path skips rules 1 and 2 entirely and may name
//! anything; it is a developer tool.

use mtg_engine::{GameEvent, GameState, ObjectId, PlayerId};
use serde::Serialize;

use crate::redact::{player_display_name, viewer_may_identify};
use crate::Viewer;
use std::collections::HashMap;

/// One rendered, already-redacted line of game history.
///
/// The client renders `text` and may use `kind` for styling or filtering. There
/// is deliberately no payload field: anything the client could need must have
/// been rendered into `text` by code that consulted the viewer, otherwise it is
/// a path around Invariant 7.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventView {
    /// The serde variant discriminant of the source `GameEvent`, e.g.
    /// `"CardDrawn"`. Never carries payload data.
    pub kind: String,
    /// A rendered, redacted, human-readable line.
    pub text: String,
    /// The player the line is about, if any — display name, always public.
    pub player: Option<String>,
}

/// Render `ev` for `viewer`, or `None` if `viewer` may not see it at all.
///
/// Architecture Invariant 7 chokepoint #2 (the other is
/// `StateViewModel::from_game_state_for`).
///
/// DEVIATION from the session plan's sketch, which writes
/// `event_view_for(ev, state, viewer)`: `player_names` is a fourth parameter.
/// A rendered line has to say "bob draws a card", and `GameState` carries
/// `PlayerId`s only — the display names live beside it, exactly as they do for
/// `StateViewModel::from_game_state_for`. Without it every line would read
/// `player_2`, and the caller would have to re-render, which would put string
/// formatting back outside the redaction chokepoint. Player display names are
/// public information, so the extra parameter carries no hidden data.
pub fn event_view_for(
    ev: &GameEvent,
    state: &GameState,
    player_names: &HashMap<PlayerId, String>,
    viewer: Viewer,
) -> Option<EventView> {
    let seat = match viewer {
        Viewer::Omniscient => None,
        Viewer::Seat(p) => {
            // Rule 1: honour the engine's own privacy verdict first.
            if let Some(private_to) = ev.private_to() {
                if private_to != p {
                    return None;
                }
            }
            Some(p)
        }
    };

    let kind = event_kind(ev);
    let name = |pid: PlayerId| player_display_name(pid, player_names);

    // `may_name(id)` is the single entitlement gate for card identities.
    // Omniscient may name anything; a seat must be entitled.
    let may_name = |id: ObjectId| match seat {
        None => true,
        Some(p) => viewer_may_identify(state, id, p),
    };
    let card_name = |id: ObjectId| -> Option<String> {
        if !may_name(id) {
            return None;
        }
        state
            .objects()
            .get(&id)
            .map(|o| o.characteristics.name.clone())
    };

    let (text, player) = match ev {
        GameEvent::TurnStarted {
            player,
            turn_number,
        } => (
            format!("Turn {turn_number} — {}", name(*player)),
            Some(*player),
        ),
        GameEvent::StepChanged { step, phase } => (format!("{phase:?} — {step:?}"), None),
        GameEvent::PriorityGiven { player } => {
            (format!("{} has priority", name(*player)), Some(*player))
        }
        GameEvent::PriorityPassed { player } => {
            (format!("{} passes", name(*player)), Some(*player))
        }
        GameEvent::AllPlayersPassed => ("All players passed".to_string(), None),
        // CR 402.1: that a card was drawn is public; WHICH card it was is known
        // only to the drawing player until it leaves their hand.
        GameEvent::CardDrawn {
            player,
            new_object_id,
        } => {
            let text = match card_name(*new_object_id) {
                Some(n) => format!("{} draws {n}", name(*player)),
                None => format!("{} draws a card", name(*player)),
            };
            (text, Some(*player))
        }
        // CR 701.8a: discarding puts the card in the graveyard, a public zone —
        // its identity becomes public. `new_id` is the graveyard object.
        GameEvent::CardDiscarded { player, new_id, .. } => {
            let text = match card_name(*new_id) {
                Some(n) => format!("{} discards {n}", name(*player)),
                None => format!("{} discards a card", name(*player)),
            };
            (text, Some(*player))
        }
        // CR 305.1: playing a land is a public action.
        GameEvent::LandPlayed {
            player,
            new_land_id,
        } => {
            let text = match card_name(*new_land_id) {
                Some(n) => format!("{} plays {n}", name(*player)),
                None => format!("{} plays a land", name(*player)),
            };
            (text, Some(*player))
        }
        // CR 601.2: casting a spell puts it on the stack, a public zone.
        GameEvent::SpellCast {
            player,
            stack_object_id,
            ..
        } => {
            let text = match card_name(*stack_object_id) {
                Some(n) => format!("{} casts {n}", name(*player)),
                None => format!("{} casts a spell", name(*player)),
            };
            (text, Some(*player))
        }
        GameEvent::PlayerConceded { player } => {
            (format!("{} concedes", name(*player)), Some(*player))
        }
        GameEvent::PlayerLost { player, reason } => (
            format!("{} loses the game ({reason:?})", name(*player)),
            Some(*player),
        ),
        GameEvent::GameOver { winner } => {
            let text = match winner {
                Some(w) => format!("Game over — {} wins", name(*w)),
                None => "Game over — draw".to_string(),
            };
            (text, *winner)
        }
        // Rule 3: kind only. No payload field is read on this path, so no card
        // name can be interpolated. (The kind itself comes from the serde
        // discriminant — see `event_kind`.)
        _ => (kind.clone(), None),
    };

    Some(EventView {
        kind,
        text,
        player: player.map(name),
    })
}

/// The serde variant discriminant of `ev`, e.g. `"CardDrawn"`.
///
/// Derived from the serialized form rather than a hand-maintained match, so a
/// new `GameEvent` variant needs no edit here and cannot be mislabelled.
///
/// PB-DP10's gate-integrity finding applies directly: `GameEvent` is externally
/// tagged, so a variant WITH fields serializes as a one-key object
/// (`{"CardDrawn": {…}}`) but a UNIT variant serializes as a bare JSON **string**
/// (`"AllPlayersPassed"`). A walk that matches object keys only would silently
/// return nothing for every unit variant. Both shapes are handled.
///
/// Only the key (or the bare string) is ever read — never a payload value — so
/// this function cannot leak a card name by construction.
fn event_kind(ev: &GameEvent) -> String {
    match serde_json::to_value(ev) {
        // Unit variant: `"AllPlayersPassed"`.
        Ok(serde_json::Value::String(s)) => s,
        // Struct/tuple variant: `{"CardDrawn": {…}}` — take the single key.
        Ok(serde_json::Value::Object(map)) => map
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "Event".to_string()),
        _ => "Event".to_string(),
    }
}
