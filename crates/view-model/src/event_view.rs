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
//!    client-side. **Every** identity in this module is obtained through the
//!    local `card_name` closure (or the `card_or` wrapper around it), which is
//!    the single gate; no arm reads `state.objects()` directly.
//! 3. The `_ =>` catch-all emits a **kind-only** line. The kind string is
//!    derived from the serde variant discriminant, so it is structurally
//!    incapable of interpolating a card name — there is no formatting of any
//!    payload field anywhere on that path. Its `text` is therefore the bare
//!    variant name (`"DamageCleared"`), which is a correct redaction floor and a
//!    poor sentence. UI-3 (`scutemob-180`) converted the ~50 variants a player
//!    actually cares about into prose arms; the floor stays for the rest.
//!    Adding another arm is safe as long as it routes any object id through
//!    `card_name` the way every existing named arm does.
//! 4. Every rendered line carries an [`EventTier`], so the client can group or
//!    filter the feed without parsing `text` or re-deriving meaning from `kind`.
//!    Classification lives in [`event_tier`], a function that matches on the
//!    variant and reads **no payload field at all** — it cannot leak by
//!    construction, and keeping it separate from the rendering match means the
//!    "what bucket is this" question has exactly one answer site.
//!
//! The `Viewer::Omniscient` path skips rules 1 and 2 entirely and may name
//! anything; it is a developer tool.
//!
//! # Known conservatism inherited from `redact::viewer_may_identify`
//!
//! `viewer_may_identify` denies any object it cannot find in `state.objects()`
//! (CR 400.7 — a zone change makes a new object and retires the old id). Several
//! events below carry a *retired* id alongside a live one; each arm deliberately
//! reads the **live** one (the graveyard object for a death, the exile object
//! for an exile, and so on) so the line can be named when CR says the identity
//! is public.
//!
//! Two arms are conservative in the safe direction and known to be so:
//!
//! * `ObjectReturnedToHand` names the card only for the seat whose hand it went
//!   to, because the only live id is the new hand object (CR 402.1). In paper,
//!   bouncing a creature off the battlefield is fully public — every player saw
//!   which card it was. Narrowing this needs the "publicly revealed" notion the
//!   engine does not track (see `redact.rs`'s own conservatism section).
//! * `DiscardedToHandSize` carries only the pre-move hand id, which is retired
//!   by the time the event is rendered, so the line is always name-free even
//!   though CR 701.9a puts the card in a public graveyard. (`CardDiscarded`,
//!   which *does* carry the graveyard id, names it.)

use mtg_engine::{
    AttackTarget, CombatDamageTarget, GameEvent, GameState, ManaColor, ObjectId, PlayerId,
};
use serde::Serialize;

use crate::redact::{player_display_name, viewer_may_identify};
use crate::Viewer;
use std::collections::HashMap;

/// Which bucket of the event feed a line belongs to.
///
/// The four tiers come from the first human playtest's own sketch
/// (`test-data/bot testing notes.md`, "there should be 3 versions of events"),
/// plus a fourth for turn structure and game outcome, which fits none of the
/// three the note names.
///
/// Serialized `snake_case` as a bare JSON string (`"player"`, `"card"`,
/// `"stack"`, `"game"`) rather than as a `String` field the renderer fills in:
/// the consumer is a Svelte client that will switch on this value, and a
/// unit-only enum makes the set of legal values a compile-time fact on the Rust
/// side while still arriving as the plain string the client wants. A `String`
/// would have allowed a typo'd tier to ship silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventTier {
    /// Turn and step structure, and how the game ends.
    Game,
    /// Something a *player* did, or had done to them.
    Player,
    /// Something that happened to a *card or permanent*.
    Card,
    /// Something that happened on the *stack*.
    Stack,
}

/// One rendered, already-redacted line of game history.
///
/// The client renders `text`, may use `kind` for styling, and uses `tier` for
/// grouping and filtering. There is deliberately no payload field: anything the
/// client could need must have been rendered into `text` by code that consulted
/// the viewer, otherwise it is a path around Invariant 7.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventView {
    /// The serde variant discriminant of the source `GameEvent`, e.g.
    /// `"CardDrawn"`. Never carries payload data.
    pub kind: String,
    /// A rendered, redacted, human-readable line.
    pub text: String,
    /// The player the line is about, if any — display name, always public.
    pub player: Option<String>,
    /// Which bucket of the feed this line belongs to. See [`event_tier`].
    pub tier: EventTier,
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
    // The name-or-fallback shorthand every prose arm uses. `fallback` is written
    // at the call site with the capitalisation the sentence needs, because a
    // name-free line must still be a sentence — never a placeholder token.
    let card_or = |id: ObjectId, fallback: &str| -> String {
        card_name(id).unwrap_or_else(|| fallback.to_string())
    };
    // CR 508.1a: an attack may be aimed at a player (public, CR 108.1) or at a
    // planeswalker, whose identity goes through the same gate as any other
    // battlefield permanent.
    let attack_target = |t: &AttackTarget| -> String {
        match t {
            AttackTarget::Player(pid) => name(*pid),
            AttackTarget::Planeswalker(id) => card_or(*id, "a planeswalker"),
        }
    };
    // CR 120.1: damage is dealt to players, creatures, planeswalkers, battles.
    let damage_target = |t: &CombatDamageTarget| -> String {
        match t {
            CombatDamageTarget::Creature(id) => card_or(*id, "a creature"),
            CombatDamageTarget::Player(pid) => name(*pid),
            CombatDamageTarget::Planeswalker(id) => card_or(*id, "a planeswalker"),
        }
    };

    let (text, player) = match ev {
        // ── Game tier: turn structure and outcome ──────────────────────────
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
        // CR 103.5: mulligans are taken in public; the NUMBER of cards put on
        // the bottom is public, their identities are not — and this arm reads no
        // id at all, only `.len()`, so there is nothing to redact.
        GameEvent::MulliganTaken {
            player,
            mulligan_number,
            is_free,
        } => {
            let suffix = if *is_free { " (free)" } else { "" };
            (
                format!(
                    "{} takes mulligan #{mulligan_number}{suffix}",
                    name(*player)
                ),
                Some(*player),
            )
        }
        GameEvent::MulliganKept {
            player,
            cards_to_bottom,
        } => {
            let text = if cards_to_bottom.is_empty() {
                format!("{} keeps their hand", name(*player))
            } else {
                format!(
                    "{} keeps their hand, putting {} on the bottom",
                    name(*player),
                    count_noun(cards_to_bottom.len(), "card", "cards")
                )
            };
            (text, Some(*player))
        }
        // CR 701.24: shuffling is public; what it shuffles is not, and this arm
        // names nothing.
        GameEvent::LibraryShuffled { player } => (
            format!("{} shuffles their library", name(*player)),
            Some(*player),
        ),
        GameEvent::CleanupPerformed => ("Cleanup step".to_string(), None),
        // CR 104.4b: a mandatory loop makes the game a draw. `description` is
        // engine-authored prose about board-state repetition, not a card name.
        GameEvent::LoopDetected { description } => (format!("Loop detected — {description}"), None),

        // ── Player tier: what a player did, or had done to them ────────────
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
        // CR 701.9a: discarding puts the card in the graveyard, a public zone —
        // its identity becomes public. `new_id` is the graveyard object.
        //
        // NOTE: `events.rs`'s own doc for this variant says "CR 701.8", which is
        // Destroy in the current CR — the keyword-action block was renumbered and
        // a run of those doc comments did not follow. See UI-3's handoff.
        GameEvent::CardDiscarded { player, new_id, .. } => {
            let text = match card_name(*new_id) {
                Some(n) => format!("{} discards {n}", name(*player)),
                None => format!("{} discards a card", name(*player)),
            };
            (text, Some(*player))
        }
        // CR 514.1: the cleanup discard. Unlike `CardDiscarded` this event
        // carries ONLY the pre-move hand id, which CR 400.7 has already retired
        // by the time the event is rendered — so `card_name` denies it and the
        // line is always name-free. Safe direction, and noted in the module doc
        // as known conservatism rather than left to be rediscovered.
        GameEvent::DiscardedToHandSize {
            player, object_id, ..
        } => {
            let text = match card_name(*object_id) {
                Some(n) => format!("{} discards {n} to hand size", name(*player)),
                None => format!("{} discards a card to hand size", name(*player)),
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
        // CR 701.26: tapping and untapping. The battlefield is a public zone
        // (CR 400.2), so the permanent is nameable — except when it is face down
        // (CR 708.2), which is exactly the case `card_name` denies.
        GameEvent::PermanentTapped { player, object_id } => (
            format!(
                "{} taps {}",
                name(*player),
                card_or(*object_id, "a permanent")
            ),
            Some(*player),
        ),
        GameEvent::PermanentUntapped { player, object_id } => (
            format!(
                "{} untaps {}",
                name(*player),
                card_or(*object_id, "a permanent")
            ),
            Some(*player),
        ),
        // CR 502.2: the untap step untaps the active player's permanents as a
        // turn-based action. Only the COUNT is read — the identities are already
        // public, but a line naming a dozen permanents is not a sentence.
        GameEvent::PermanentsUntapped { player, objects } => {
            let text = if objects.is_empty() {
                format!("{} untaps nothing", name(*player))
            } else {
                format!(
                    "{} untaps {}",
                    name(*player),
                    count_noun(objects.len(), "permanent", "permanents")
                )
            };
            (text, Some(*player))
        }
        // CR 508.1: which creatures attack, and what each attacks, is public.
        GameEvent::AttackersDeclared {
            attacking_player,
            attackers,
        } => {
            let text = if attackers.is_empty() {
                format!("{} declares no attackers", name(*attacking_player))
            } else {
                let parts: Vec<String> = attackers
                    .iter()
                    .map(|(id, target)| {
                        format!("{} → {}", card_or(*id, "a creature"), attack_target(target))
                    })
                    .collect();
                format!(
                    "{} attacks with {}",
                    name(*attacking_player),
                    parts.join(", ")
                )
            };
            (text, Some(*attacking_player))
        }
        // CR 509.1: which creatures block, and what each blocks, is public.
        GameEvent::BlockersDeclared {
            defending_player,
            blockers,
        } => {
            let text = if blockers.is_empty() {
                format!("{} declares no blockers", name(*defending_player))
            } else {
                let parts: Vec<String> = blockers
                    .iter()
                    .map(|(blocker, attacker)| {
                        format!(
                            "{} blocks {}",
                            card_or(*blocker, "a creature"),
                            card_or(*attacker, "an attacker")
                        )
                    })
                    .collect();
                format!("{} blocks: {}", name(*defending_player), parts.join(", "))
            };
            (text, Some(*defending_player))
        }
        // CR 119.3: life totals are public and no object identity is involved.
        GameEvent::LifeGained { player, amount } => (
            format!("{} gains {amount} life", name(*player)),
            Some(*player),
        ),
        GameEvent::LifeLost { player, amount } => (
            format!("{} loses {amount} life", name(*player)),
            Some(*player),
        ),
        // CR 106.4: mana is added to a pool. The producing permanent is on the
        // battlefield (public, CR 400.2) but may be face down, so it goes
        // through the gate like anything else.
        GameEvent::ManaAdded {
            player,
            color,
            amount,
            source,
        } => {
            let from = source
                .and_then(card_name)
                .map(|n| format!(" from {n}"))
                .unwrap_or_default();
            (
                format!(
                    "{} adds {amount} {} mana{from}",
                    name(*player),
                    mana_color_label(color)
                ),
                Some(*player),
            )
        }
        // CR 500.4: unused mana empties at every step and phase boundary.
        GameEvent::ManaPoolsEmptied => ("Mana pools empty".to_string(), None),
        // CR 122.1 / CR 104.3c: poison counters on a player are public, and the
        // infect source is a public battlefield object.
        GameEvent::PoisonCountersGiven {
            player,
            amount,
            source,
        } => {
            let from = match card_name(*source) {
                Some(n) => format!(" from {n}"),
                None => String::new(),
            };
            (
                format!(
                    "{} gets {}{from}",
                    name(*player),
                    count_noun(*amount as usize, "poison counter", "poison counters")
                ),
                Some(*player),
            )
        }

        // ── Card tier: what happened to a card or permanent ────────────────
        // CR 608.3a: the permanent is now on the battlefield, a public zone
        // (CR 400.2). A permanent that entered face down is denied by the gate.
        GameEvent::PermanentEnteredBattlefield { player, object_id } => (
            format!(
                "{} enters the battlefield under {}'s control",
                card_or(*object_id, "A permanent"),
                name(*player)
            ),
            Some(*player),
        ),
        // CR 704.5f-h: death is an SBA move to the graveyard, a public zone, so
        // the GRAVEYARD id is the one to name — `object_id` is retired
        // (CR 400.7) and would always deny.
        GameEvent::CreatureDied {
            new_grave_id,
            controller,
            ..
        } => (
            format!("{} dies", card_or(*new_grave_id, "A creature")),
            Some(*controller),
        ),
        GameEvent::PermanentDestroyed { new_grave_id, .. } => (
            format!("{} is destroyed", card_or(*new_grave_id, "A permanent")),
            None,
        ),
        // CR 701.21: sacrifice. `new_id` is the graveyard (or exile) object.
        GameEvent::PermanentSacrificed { player, new_id, .. } => (
            format!(
                "{} sacrifices {}",
                name(*player),
                card_or(*new_id, "a permanent")
            ),
            Some(*player),
        ),
        // CR 701.13: exile is a public zone (CR 400.2) unless the card was
        // exiled face down (CR 406.3), which the gate already denies.
        GameEvent::ObjectExiled {
            player,
            new_exile_id,
            ..
        } => (
            format!(
                "{} exiles {}",
                name(*player),
                card_or(*new_exile_id, "a card")
            ),
            Some(*player),
        ),
        // CR 402.1: the hand is hidden, so only the seat the card went to may be
        // told which card it is. Conservative for a battlefield bounce, which is
        // public in paper — see the module doc.
        GameEvent::ObjectReturnedToHand {
            player,
            new_hand_id,
            ..
        } => (
            format!(
                "{} returns {} to their hand",
                name(*player),
                card_or(*new_hand_id, "a card")
            ),
            Some(*player),
        ),
        // CR 404.1 / CR 400.2: the graveyard is public.
        GameEvent::ObjectPutInGraveyard {
            player,
            new_grave_id,
            ..
        } => (
            format!(
                "{} puts {} into a graveyard",
                name(*player),
                card_or(*new_grave_id, "a card")
            ),
            Some(*player),
        ),
        // CR 401.2: a library is hidden. `viewer_may_identify` allows the
        // library's OWNER and nobody else, which is right — a player who puts a
        // card on top of their own library knows what it is.
        GameEvent::ObjectPutOnLibrary {
            player, new_lib_id, ..
        } => (
            format!(
                "{} puts {} into their library",
                name(*player),
                card_or(*new_lib_id, "a card")
            ),
            Some(*player),
        ),
        // CR 701.17: milling puts the card in the graveyard, a public zone —
        // which is precisely why a mill may be named while a draw may not.
        GameEvent::CardMilled { player, new_id } => (
            format!("{} mills {}", name(*player), card_or(*new_id, "a card")),
            Some(*player),
        ),
        // CR 702.29a: cycling discards, so `new_id` is the graveyard object.
        GameEvent::CardCycled { player, new_id, .. } => (
            format!("{} cycles {}", name(*player), card_or(*new_id, "a card")),
            Some(*player),
        ),
        // CR 701.7: a token on the battlefield is a public object.
        GameEvent::TokenCreated { player, object_id } => (
            format!(
                "{} creates {}",
                name(*player),
                match card_name(*object_id) {
                    Some(n) => format!("a {n} token"),
                    None => "a token".to_string(),
                }
            ),
            Some(*player),
        ),
        // CR 704.5d: a token in a non-battlefield zone ceases to exist. Its id
        // is usually already retired, so this line is usually name-free.
        GameEvent::TokenCeasedToExist { object_id } => (
            match card_name(*object_id) {
                Some(n) => format!("{n} (token) ceases to exist"),
                None => "A token ceases to exist".to_string(),
            },
            None,
        ),
        // CR 122.1: counters are markers on a public object. `lib.rs::format_counter_type`
        // renders the KIND of counter, never a card name.
        GameEvent::CounterAdded {
            object_id,
            counter,
            count,
        } => (
            format!(
                "{} put on {}",
                count_noun(
                    *count as usize,
                    &format!("{} counter", crate::format_counter_type(counter)),
                    &format!("{} counters", crate::format_counter_type(counter))
                ),
                card_or(*object_id, "a permanent")
            ),
            None,
        ),
        GameEvent::CounterRemoved {
            object_id,
            counter,
            count,
        } => (
            format!(
                "{} removed from {}",
                count_noun(
                    *count as usize,
                    &format!("{} counter", crate::format_counter_type(counter)),
                    &format!("{} counters", crate::format_counter_type(counter))
                ),
                card_or(*object_id, "a permanent")
            ),
            None,
        ),
        // CR 303.4a/b: an Aura enters attached to what it enchants. Both objects
        // are on the battlefield and go through the gate independently.
        GameEvent::AuraAttached {
            aura_id,
            target_id,
            controller,
        } => (
            format!(
                "{} is attached to {}",
                card_or(*aura_id, "An Aura"),
                card_or(*target_id, "a permanent")
            ),
            Some(*controller),
        ),
        // CR 704.5m: an Aura attached to nothing legal goes to the graveyard.
        GameEvent::AuraFellOff { new_grave_id, .. } => (
            format!(
                "{} falls off and is put into its owner's graveyard",
                card_or(*new_grave_id, "An Aura")
            ),
            None,
        ),
        // CR 702.6a: equip attaches the Equipment to a creature its controller
        // controls. Both are public battlefield objects.
        GameEvent::EquipmentAttached {
            equipment_id,
            target_id,
            controller,
        } => (
            format!(
                "{} is attached to {}",
                card_or(*equipment_id, "An Equipment"),
                card_or(*target_id, "a creature")
            ),
            Some(*controller),
        ),
        // CR 704.5n: the Equipment becomes unattached as an SBA. It stays on the
        // battlefield, so its id is still live and usually nameable.
        GameEvent::EquipmentUnattached { object_id } => (
            format!("{} becomes unattached", card_or(*object_id, "An Equipment")),
            None,
        ),
        // CR 702.21a / CR 115.1: what a spell or ability targets is public. The
        // TARGETING object is identified only by its `StackObject` id, which is
        // never in `state.objects()` (see the `SpellCast` note), so this line
        // names the target and attributes it to the controller by name only.
        GameEvent::PermanentTargeted {
            target_id,
            targeting_controller,
            ..
        } => (
            format!(
                "{} targets {}",
                name(*targeting_controller),
                card_or(*target_id, "a permanent")
            ),
            Some(*targeting_controller),
        ),
        // CR 510.2: combat damage is assigned and dealt in one batch. Rendered
        // as one line per assignment so a multi-blocker combat is readable.
        GameEvent::CombatDamageDealt { assignments } => {
            let text = if assignments.is_empty() {
                "No combat damage is dealt".to_string()
            } else {
                assignments
                    .iter()
                    .map(|a| {
                        format!(
                            "{} deals {} combat damage to {}",
                            card_or(a.source, "A creature"),
                            a.amount,
                            damage_target(&a.target)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            (text, None)
        }
        // CR 120: non-combat damage from a spell or ability.
        GameEvent::DamageDealt {
            source,
            target,
            amount,
        } => (
            format!(
                "{} deals {amount} damage to {}",
                card_or(*source, "A source"),
                damage_target(target)
            ),
            None,
        ),
        // CR 701.27: transforming a double-faced permanent. It stays on the
        // battlefield, so the id is live.
        GameEvent::PermanentTransformed {
            object_id,
            to_back_face,
        } => {
            let direction = if *to_back_face {
                "into its back face"
            } else {
                "back to its front face"
            };
            (
                format!(
                    "{} transforms {direction}",
                    card_or(*object_id, "A permanent")
                ),
                None,
            )
        }
        // CR 707.2 / CR 702.37b: turning a permanent face up reveals it to every
        // player, and it is face up in `state` by the time this renders — so the
        // gate now allows the name it denied a moment earlier.
        GameEvent::PermanentTurnedFaceUp { player, permanent } => (
            format!(
                "{} turns {} face up",
                name(*player),
                card_or(*permanent, "a permanent")
            ),
            Some(*player),
        ),

        // ── Stack tier: what happened on the stack ─────────────────────────
        // CR 405.1 / CR 601.2: casting a spell puts it on the stack, a public
        // zone, so every seat may name it.
        //
        // `source_object_id`, NOT `stack_object_id`. The two ids are different
        // objects and only one of them is reachable here: `handle_cast_spell`
        // mints `stack_entry_id = state.next_object_id()` and uses it *solely*
        // to build the `StackObject` it pushes onto `state.stack_objects()`
        // (`rules/casting.rs:4401`, `:4529`) — that id is never inserted into
        // `state.objects()`, so looking it up always misses and every cast
        // would render as the name-free fallback. `source_object_id` is the
        // card's new object in `ZoneId::Stack` (`casting.rs:4732`), which is in
        // `state.objects()`. The moved `stack_kind_info` resolves the stack's
        // `source_object` against `state.objects()` for exactly this reason.
        //
        // Every stack arm below inherits this rule: name off the *source*
        // object id, never off a `stack_object_id`.
        GameEvent::SpellCast {
            player,
            source_object_id,
            ..
        } => {
            let text = match card_name(*source_object_id) {
                Some(n) => format!("{} casts {n}", name(*player)),
                None => format!("{} casts a spell", name(*player)),
            };
            (text, Some(*player))
        }
        // CR 608.2n/608.3: on resolution the card has moved to the graveyard or
        // the battlefield, both public zones, and `source_object_id` is its new
        // id there.
        GameEvent::SpellResolved {
            player,
            source_object_id,
            ..
        } => {
            let text = match card_name(*source_object_id) {
                Some(n) => format!("{n} resolves"),
                None => format!("{}'s spell resolves", name(*player)),
            };
            (text, Some(*player))
        }
        // CR 701.6: a countered spell goes to its owner's graveyard.
        GameEvent::SpellCountered {
            player,
            source_object_id,
            ..
        } => {
            let text = match card_name(*source_object_id) {
                Some(n) => format!("{n} is countered"),
                None => format!("{}'s spell is countered", name(*player)),
            };
            (text, Some(*player))
        }
        // CR 608.2b: every target illegal — the spell is removed from the stack
        // without resolving. Distinct from being countered by an effect.
        GameEvent::SpellFizzled {
            player,
            source_object_id,
            ..
        } => {
            let text = match card_name(*source_object_id) {
                Some(n) => format!("{n} fizzles — all its targets are illegal"),
                None => format!(
                    "{}'s spell fizzles — all its targets are illegal",
                    name(*player)
                ),
            };
            (text, Some(*player))
        }
        // CR 602.2: activating an ability puts it on the stack. The SOURCE stays
        // in its zone and keeps a live id, so it can be named.
        GameEvent::AbilityActivated {
            player,
            source_object_id,
            ..
        } => {
            let text = match card_name(*source_object_id) {
                Some(n) => format!("{} activates an ability of {n}", name(*player)),
                None => format!("{} activates an ability", name(*player)),
            };
            (text, Some(*player))
        }
        // CR 603.3: triggered abilities are put on the stack the next time a
        // player would receive priority.
        GameEvent::AbilityTriggered {
            controller,
            source_object_id,
            ..
        } => {
            let text = match card_name(*source_object_id) {
                Some(n) => format!("A triggered ability of {n} goes on the stack"),
                None => "A triggered ability goes on the stack".to_string(),
            };
            (text, Some(*controller))
        }
        // CR 608.3b: the ability resolved. This event carries only the
        // `StackObject` id, which names nothing in `state.objects()`, so there
        // is no identity to gate here at all.
        GameEvent::AbilityResolved { controller, .. } => (
            format!("{}'s ability resolves", name(*controller)),
            Some(*controller),
        ),
        // CR 707.10: both ids are `StackObject` ids, not `state.objects()`
        // entries — nothing to name and nothing to gate.
        GameEvent::SpellCopied { controller, .. } => (
            format!("{} copies a spell on the stack", name(*controller)),
            Some(*controller),
        ),
        // CR 702.85b / CR 701.57a: the cascaded/discovered card is in the Stack
        // zone, which is public.
        GameEvent::CascadeCast { player, card_id } => (
            format!(
                "{} cascades into {}",
                name(*player),
                card_or(*card_id, "a spell")
            ),
            Some(*player),
        ),
        GameEvent::DiscoverCast { player, card_id } => (
            format!(
                "{} discovers and casts {}",
                name(*player),
                card_or(*card_id, "a spell")
            ),
            Some(*player),
        ),
        // CR 903.8: casting a commander from the command zone. `card_id` here is
        // a `CardId` — a card-DEFINITION identifier, not an `ObjectId` — and
        // resolving it to a printed name would be a path around the entitlement
        // gate, which takes `ObjectId`s only. So this line names nobody's card;
        // the matching `SpellCast` event, which does carry a gated `ObjectId`,
        // renders the name.
        GameEvent::CommanderCastFromCommandZone {
            player, tax_paid, ..
        } => {
            let text = if *tax_paid == 0 {
                format!(
                    "{} casts their commander from the command zone",
                    name(*player)
                )
            } else {
                format!(
                    "{} casts their commander from the command zone (commander tax {{{}}})",
                    name(*player),
                    tax_paid * 2
                )
            };
            (text, Some(*player))
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
        tier: event_tier(ev),
    })
}

/// Which feed bucket `ev` belongs to.
///
/// Deliberately a separate function from the rendering match: classification and
/// prose answer different questions, and keeping them apart means this one can
/// be read as a single table. It also matches on the **variant only** and reads
/// no payload field whatsoever, so unlike the renderer it is leak-proof by
/// construction and needs no entitlement gate.
///
/// `GameEvent` has ~141 variants and is not `#[non_exhaustive]`. This is
/// deliberately NOT an exhaustive match: the vast majority of those variants are
/// niche mechanics that no client is going to bucket, and an exhaustive match
/// would turn every new engine event into a compile error here for no benefit.
///
/// **The default is [`EventTier::Game`]**, and the choice is deliberate. `Game`
/// is by far the least-populated tier and the one a client is least likely to
/// filter out — the turn-structure spine of the feed. An unclassified new
/// variant therefore lands somewhere a player will actually see it (it shows up
/// as a bare `kind` line among the turn markers, which reads as "something
/// happened that this build does not narrate yet"), rather than being buried in
/// the busiest tier or dropped. It is pinned by
/// `test_event_tier_defaults_to_game_for_an_unclassified_variant`.
fn event_tier(ev: &GameEvent) -> EventTier {
    match ev {
        // ── Turn/step structure and game outcome ───────────────────────────
        GameEvent::TurnStarted { .. }
        | GameEvent::StepChanged { .. }
        | GameEvent::ExtraTurnAdded { .. }
        | GameEvent::CleanupPerformed
        | GameEvent::LibraryShuffled { .. }
        | GameEvent::LoopDetected { .. }
        | GameEvent::MulliganTaken { .. }
        | GameEvent::MulliganKept { .. }
        | GameEvent::PlayerLost { .. }
        | GameEvent::PlayerConceded { .. }
        | GameEvent::GameOver { .. } => EventTier::Game,

        // ── Things a player does, or has done to them ──────────────────────
        GameEvent::PriorityGiven { .. }
        | GameEvent::PriorityPassed { .. }
        | GameEvent::AllPlayersPassed
        | GameEvent::CardDrawn { .. }
        | GameEvent::CardDiscarded { .. }
        | GameEvent::DiscardedToHandSize { .. }
        | GameEvent::LandPlayed { .. }
        | GameEvent::PermanentTapped { .. }
        | GameEvent::PermanentUntapped { .. }
        | GameEvent::PermanentsUntapped { .. }
        | GameEvent::AttackersDeclared { .. }
        | GameEvent::BlockersDeclared { .. }
        | GameEvent::LifeGained { .. }
        | GameEvent::LifeLost { .. }
        | GameEvent::ManaAdded { .. }
        | GameEvent::ManaPoolsEmptied
        | GameEvent::PoisonCountersGiven { .. } => EventTier::Player,

        // ── Things that happen to a card or permanent ──────────────────────
        GameEvent::PermanentEnteredBattlefield { .. }
        | GameEvent::CreatureDied { .. }
        | GameEvent::PlaneswalkerDied { .. }
        | GameEvent::PermanentDestroyed { .. }
        | GameEvent::PermanentSacrificed { .. }
        | GameEvent::ObjectExiled { .. }
        | GameEvent::ObjectReturnedToHand { .. }
        | GameEvent::ObjectPutInGraveyard { .. }
        | GameEvent::ObjectPutOnLibrary { .. }
        | GameEvent::CardMilled { .. }
        | GameEvent::CardCycled { .. }
        | GameEvent::TokenCreated { .. }
        | GameEvent::TokenCeasedToExist { .. }
        | GameEvent::CounterAdded { .. }
        | GameEvent::CounterRemoved { .. }
        | GameEvent::CountersAnnihilated { .. }
        | GameEvent::AuraAttached { .. }
        | GameEvent::AuraFellOff { .. }
        | GameEvent::EquipmentAttached { .. }
        | GameEvent::EquipmentUnattached { .. }
        | GameEvent::FortificationAttached { .. }
        | GameEvent::PermanentTargeted { .. }
        | GameEvent::PermanentTransformed { .. }
        | GameEvent::PermanentTurnedFaceUp { .. }
        | GameEvent::CombatDamageDealt { .. }
        | GameEvent::DamageDealt { .. } => EventTier::Card,

        // ── Things that happen on the stack ────────────────────────────────
        GameEvent::SpellCast { .. }
        | GameEvent::SpellResolved { .. }
        | GameEvent::SpellCountered { .. }
        | GameEvent::SpellFizzled { .. }
        | GameEvent::AbilityActivated { .. }
        | GameEvent::AbilityTriggered { .. }
        | GameEvent::AbilityResolved { .. }
        | GameEvent::SpellCopied { .. }
        | GameEvent::CascadeCast { .. }
        | GameEvent::DiscoverCast { .. }
        | GameEvent::CommanderCastFromCommandZone { .. } => EventTier::Stack,

        // Documented default — see this function's doc block.
        _ => EventTier::Game,
    }
}

/// `"1 card"` / `"3 cards"` — the pluralisation every counted line needs.
fn count_noun(n: usize, singular: &str, plural: &str) -> String {
    if n == 1 {
        format!("1 {singular}")
    } else {
        format!("{n} {plural}")
    }
}

/// A lowercase colour word for a mana symbol (CR 106.1b).
fn mana_color_label(color: &ManaColor) -> &'static str {
    match color {
        ManaColor::White => "white",
        ManaColor::Blue => "blue",
        ManaColor::Black => "black",
        ManaColor::Red => "red",
        ManaColor::Green => "green",
        ManaColor::Colorless => "colorless",
    }
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
///
/// COST, for S5/S6 to weigh: this serializes the whole event payload to build a
/// `Value` and then reads one key from it. Correct and leak-proof, but pure
/// waste per call, and the play-server will call it for every event on every
/// poll. If it shows up, replace it with a `match` returning a `&'static str` —
/// but keep the property that the arm reads no payload field, or the leak-proof
/// argument above stops holding.
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
