//! Commander format rules: deck validation, commander tax, color identity, mulligan,
//! companion, and partner mechanics.
//!
//! See architecture doc Section 3.x and CR 903 for design rationale.

use crate::cards::{CardDefinition, CardRegistry};
use crate::rules::events::GameEvent;
use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::turn::Step;
use crate::state::zone::{ZoneId, ZoneType};
use crate::state::GameState;
use crate::state::{CardId, CardType, Color, ManaCost, SuperType};

// ── Deck Validation ───────────────────────────────────────────────────────────

/// Result of validating a Commander deck (CR 903.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeckValidationResult {
    pub valid: bool,
    pub violations: Vec<DeckViolation>,
}

/// A single violation found during deck validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeckViolation {
    /// CR 903.5a: deck must be exactly 100 cards (including commander(s)).
    WrongDeckSize { actual: usize, expected: usize },
    /// CR 903.5b: singleton rule — no duplicate non-basic-land card names.
    DuplicateCard { name: String, count: usize },
    /// CR 903.5c: card color identity outside commander's combined color identity.
    ColorIdentityViolation {
        card: String,
        card_colors: Vec<Color>,
        commander_colors: Vec<Color>,
    },
    /// Card appears on the Commander banned list.
    BannedCard { name: String },
    /// CR 903.3: Commander must be a legendary creature (or have "can be your commander").
    InvalidCommander { name: String, reason: String },
}

/// Validate a Commander deck against the format rules.
///
/// CR 903.5a: deck size must be exactly 100, including the commander(s).
/// CR 903.5b: singleton rule — no duplicate non-basic-land names.
/// CR 903.5c: each card's color identity must be a subset of the commander's combined identity.
/// CR 903.3: commander must be a legendary creature.
///
/// `commander_card_ids`: the CardId(s) of the designated commander(s) (1 or 2 for partners).
/// `deck_card_ids`: ALL 100 card IDs including the commander(s).
pub fn validate_deck(
    commander_card_ids: &[CardId],
    deck_card_ids: &[CardId],
    registry: &CardRegistry,
    banned_list: &[String],
) -> DeckValidationResult {
    let mut violations = Vec::new();

    // CR 902.124h / CR 903.5: if 2 commanders, both must have partner keyword.
    // A single commander needs no partner check.
    if commander_card_ids.len() == 2 {
        let def1 = registry.get(commander_card_ids[0].clone());
        let def2 = registry.get(commander_card_ids[1].clone());
        if let (Some(d1), Some(d2)) = (def1, def2) {
            if let Err(reason) = validate_partner_commanders(d1, d2) {
                violations.push(DeckViolation::InvalidCommander {
                    name: format!("{} + {}", d1.name, d2.name),
                    reason,
                });
            }
        }
    } else if commander_card_ids.len() > 2 {
        violations.push(DeckViolation::InvalidCommander {
            name: "multiple commanders".to_string(),
            reason: "more than 2 commanders is not allowed".to_string(),
        });
    }

    // CR 903.5a: total deck size must be exactly 100
    if deck_card_ids.len() != 100 {
        violations.push(DeckViolation::WrongDeckSize {
            actual: deck_card_ids.len(),
            expected: 100,
        });
    }

    // Validate each commander
    for cid in commander_card_ids {
        if let Some(def) = registry.get(cid.clone()) {
            let is_legendary = def.types.supertypes.contains(&SuperType::Legendary);
            let is_creature = def.types.card_types.contains(&CardType::Creature);
            if !is_legendary || !is_creature {
                violations.push(DeckViolation::InvalidCommander {
                    name: def.name.clone(),
                    reason: if !is_legendary && !is_creature {
                        "not a legendary creature".to_string()
                    } else if !is_legendary {
                        "not legendary".to_string()
                    } else {
                        "not a creature".to_string()
                    },
                });
            }
        }
    }

    // Compute combined commander color identity
    let commander_colors: Vec<Color> = {
        let mut colors = Vec::new();
        for cid in commander_card_ids {
            if let Some(def) = registry.get(cid.clone()) {
                for c in compute_color_identity(def) {
                    if !colors.contains(&c) {
                        colors.push(c);
                    }
                }
            }
        }
        colors.sort();
        colors
    };

    // Check each card in the deck
    let mut name_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for card_id in deck_card_ids {
        let def = match registry.get(card_id.clone()) {
            Some(d) => d,
            None => continue, // Unknown card; skip (would be caught at deck-build time)
        };

        // CR 903.5b: singleton rule — basic lands are exempt
        let is_basic_land = def.types.supertypes.contains(&SuperType::Basic)
            && def.types.card_types.contains(&CardType::Land);
        if !is_basic_land {
            *name_counts.entry(def.name.clone()).or_insert(0) += 1;
        }

        // Banned list check
        if banned_list.iter().any(|b| b == &def.name) {
            violations.push(DeckViolation::BannedCard {
                name: def.name.clone(),
            });
        }

        // CR 903.5c: color identity must be subset of commander's identity
        let card_identity = compute_color_identity(def);
        for color in &card_identity {
            if !commander_colors.contains(color) {
                violations.push(DeckViolation::ColorIdentityViolation {
                    card: def.name.clone(),
                    card_colors: card_identity.clone(),
                    commander_colors: commander_colors.clone(),
                });
                break; // One violation per card is enough
            }
        }
    }

    // CR 903.5b: report duplicates
    for (name, count) in name_counts {
        if count > 1 {
            violations.push(DeckViolation::DuplicateCard { name, count });
        }
    }

    DeckValidationResult {
        valid: violations.is_empty(),
        violations,
    }
}

/// Compute a card's color identity (CR 903.4).
///
/// CR 903.4: A card's color identity is determined by its mana cost, color indicator,
/// and any mana symbols in its rules text. Reminder text and flavor text are excluded.
///
/// For the purposes of this implementation, we extract colors from the mana cost.
/// Rules-text mana symbols would require oracle text parsing (deferred to a future milestone).
pub fn compute_color_identity(def: &CardDefinition) -> Vec<Color> {
    let mut colors = Vec::new();

    // Extract colors from mana cost symbols
    if let Some(ref cost) = def.mana_cost {
        add_colors_from_mana_cost(cost, &mut colors);
    }

    // Deduplicate and sort for determinism
    colors.sort();
    colors.dedup();
    colors
}

// ── Commander Tax ─────────────────────────────────────────────────────────────

/// Apply commander tax to a base mana cost (CR 903.8).
///
/// CR 903.8: Each time you cast a commander from the command zone, it costs an
/// additional {2} for each previous time you cast it from the command zone. The
/// tax is added to the generic mana component of the cost.
///
/// `tax` is the number of times previously cast (from `commander_tax` in
/// `PlayerState`). `tax * 2` generic mana is added to the generic component.
pub fn apply_commander_tax(base_cost: &ManaCost, tax: u32) -> ManaCost {
    ManaCost {
        generic: base_cost.generic + tax * 2,
        white: base_cost.white,
        blue: base_cost.blue,
        black: base_cost.black,
        red: base_cost.red,
        green: base_cost.green,
        colorless: base_cost.colorless,
    }
}

// ── Commander Zone Return SBA (CR 903.9a / CR 704.6d) ────────────────────────

/// CR 903.9a / CR 704.6d: Check if any commander is in graveyard or exile and,
/// if so, automatically return it to its owner's command zone.
///
/// This is a state-based action (not a replacement effect). It fires each time
/// SBAs are checked. In M9 the return is auto-applied (the player's option to
/// leave the commander in graveyard/exile is deferred to M10+).
///
/// Called from `sba::apply_sbas_once` after counter annihilation.
pub fn check_commander_zone_return_sba(state: &mut GameState) -> Vec<GameEvent> {
    let mut events = Vec::new();

    // Collect (owner, card_id, object_id, from_zone_type) for all commanders
    // found in graveyard or exile.
    let mut to_return: Vec<(PlayerId, CardId, ObjectId, ZoneType)> = Vec::new();

    for (&owner, player_state) in state.players.iter() {
        for card_id in player_state.commander_ids.iter() {
            // Check graveyard
            let graveyard_zone_id = ZoneId::Graveyard(owner);
            if let Some(zone) = state.zones.get(&graveyard_zone_id) {
                for obj_id in zone.object_ids() {
                    if let Some(obj) = state.objects.get(&obj_id) {
                        if obj.card_id.as_ref() == Some(card_id) {
                            to_return.push((owner, card_id.clone(), obj_id, ZoneType::Graveyard));
                        }
                    }
                }
            }

            // Check exile
            let exile_zone_id = ZoneId::Exile;
            if let Some(zone) = state.zones.get(&exile_zone_id) {
                for obj_id in zone.object_ids() {
                    if let Some(obj) = state.objects.get(&obj_id) {
                        if obj.card_id.as_ref() == Some(card_id) && obj.owner == owner {
                            to_return.push((owner, card_id.clone(), obj_id, ZoneType::Exile));
                        }
                    }
                }
            }
        }
    }

    // Return each found commander to its owner's command zone.
    for (owner, card_id, object_id, from_zone) in to_return {
        let command_zone_id = ZoneId::Command(owner);
        if let Ok((_, _)) = state.move_object_to_zone(object_id, command_zone_id) {
            events.push(GameEvent::CommanderReturnedToCommandZone {
                card_id,
                owner,
                from_zone,
            });
        }
    }

    events
}

/// Handle a `ReturnCommanderToCommandZone` command (CR 903.9a / CR 704.6d).
///
/// Moves the specified object from its current zone (graveyard or exile) to the
/// owner's command zone. Validates that the object is the player's commander and
/// is currently in a zone that permits this action.
pub fn handle_return_commander_to_command_zone(
    state: &mut GameState,
    player: PlayerId,
    object_id: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    // Look up the object and validate it belongs to the player
    let obj = state
        .objects
        .get(&object_id)
        .ok_or(GameStateError::ObjectNotFound(object_id))?;

    let obj_owner = obj.owner;
    let obj_card_id = obj.card_id.clone();
    let obj_zone = obj.zone;

    if obj_owner != player {
        return Err(GameStateError::InvalidCommand(
            "can only return your own commander to command zone".to_string(),
        ));
    }

    // Validate the card_id is registered as this player's commander
    let is_commander = {
        let ps = state.player(player)?;
        obj_card_id
            .as_ref()
            .map(|cid| ps.commander_ids.contains(cid))
            .unwrap_or(false)
    };

    if !is_commander {
        return Err(GameStateError::InvalidCommand(
            "object is not registered as this player's commander".to_string(),
        ));
    }

    // Validate zone: must be in graveyard or exile
    let from_zone_type = match obj_zone {
        ZoneId::Graveyard(_) => ZoneType::Graveyard,
        ZoneId::Exile => ZoneType::Exile,
        _ => {
            return Err(GameStateError::InvalidCommand(
                "commander can only be returned from graveyard or exile".to_string(),
            ));
        }
    };

    let command_zone_id = ZoneId::Command(player);
    if let Ok((_, _)) = state.move_object_to_zone(object_id, command_zone_id) {
        if let Some(cid) = obj_card_id {
            events.push(GameEvent::CommanderReturnedToCommandZone {
                card_id: cid,
                owner: player,
                from_zone: from_zone_type,
            });
        }
    }

    Ok(events)
}

// ── Partner Commanders (CR 702.124) ───────────────────────────────────────────

/// Validate that two commanders can serve as partner commanders (CR 702.124h).
///
/// CR 702.124: Partner is a keyword ability. A legendary creature card with
/// partner can be paired with another legendary creature card that also has
/// partner. Both commanders must have the partner keyword.
///
/// CR 702.124c: When two commanders have partner, their combined color identity
/// is the union of both commanders' individual color identities.
///
/// CR 702.124d: Each partner commander's tax and commander damage are tracked
/// independently — casting commander A does not affect commander B's tax or
/// damage totals.
pub fn validate_partner_commanders(
    cmd1: &CardDefinition,
    cmd2: &CardDefinition,
) -> Result<(), String> {
    use crate::state::KeywordAbility;

    let cmd1_has_partner = cmd1.abilities.iter().any(|a| {
        matches!(
            a,
            crate::cards::AbilityDefinition::Keyword(KeywordAbility::Partner)
        )
    });

    let cmd2_has_partner = cmd2.abilities.iter().any(|a| {
        matches!(
            a,
            crate::cards::AbilityDefinition::Keyword(KeywordAbility::Partner)
        )
    });

    if !cmd1_has_partner && !cmd2_has_partner {
        return Err(format!(
            "neither '{}' nor '{}' has partner",
            cmd1.name, cmd2.name
        ));
    }
    if !cmd1_has_partner {
        return Err(format!(
            "'{}' does not have partner (CR 702.124h)",
            cmd1.name
        ));
    }
    if !cmd2_has_partner {
        return Err(format!(
            "'{}' does not have partner (CR 702.124h)",
            cmd2.name
        ));
    }

    Ok(())
}

// ── Mulligan (CR 103.5 / CR 103.5c) ───────────────────────────────────────────

/// Handle a `TakeMulligan` command (CR 103.5 / CR 103.5c).
///
/// CR 103.5: A player who mulligans shuffles all cards from their hand into their
/// library, then draws 7 cards.
///
/// CR 103.5c: In a multiplayer game, the first mulligan is free: the player
/// draws back to 7 without putting any cards on the bottom. Second and subsequent
/// mulligans require the player to put N-1 cards on the bottom when keeping
/// (where N is the total number of mulligans taken).
///
/// This function handles drawing 7 cards after shuffling. The "put N-1 on bottom"
/// step happens when the player sends `KeepHand`.
pub fn handle_take_mulligan(
    state: &mut GameState,
    player: PlayerId,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::rules::turn_actions;

    let mut events = Vec::new();

    // Increment mulligan count for this player
    let mulligan_number = {
        let ps = state.player_mut(player)?;
        ps.mulligan_count += 1;
        ps.mulligan_count
    };

    // First mulligan is free in multiplayer (CR 103.5c)
    let is_free = mulligan_number == 1;

    // Shuffle hand back into library: move all hand cards to library
    let hand_zone_id = ZoneId::Hand(player);
    let hand_objects: Vec<ObjectId> = state
        .zones
        .get(&hand_zone_id)
        .map(|z| z.object_ids())
        .unwrap_or_default();

    // Move each card from hand to library
    let lib_zone_id = ZoneId::Library(player);
    for obj_id in hand_objects {
        // Move card back to library (ignore errors for individual cards)
        let _ = state.move_object_to_zone(obj_id, lib_zone_id);
    }

    // Shuffle library (represented by event; order is not tracked in state)
    events.push(GameEvent::LibraryShuffled { player });

    // Draw 7 cards
    for _ in 0..7 {
        let draw_events = turn_actions::draw_card(state, player)?;
        events.extend(draw_events);
    }

    events.push(GameEvent::MulliganTaken {
        player,
        mulligan_number,
        is_free,
    });

    Ok(events)
}

/// Handle a `KeepHand` command (CR 103.5).
///
/// CR 103.5: After deciding to keep, the player with N mulligans taken must
/// put N-1 cards from their hand on the bottom of their library in any order.
/// For the free mulligan (first in multiplayer, N=1 → 0 cards to bottom),
/// `cards_to_bottom` must be empty. For the second mulligan, 1 card must go
/// to the bottom, and so on.
///
/// `cards_to_bottom` lists the ObjectIds to put on the bottom of the library
/// in order (index 0 = placed first, ends up above later entries).
pub fn handle_keep_hand(
    state: &mut GameState,
    player: PlayerId,
    cards_to_bottom: Vec<ObjectId>,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    // Determine how many cards must go to the bottom
    let mulligans_taken = state.player(player)?.mulligan_count;
    // First mulligan is free (CR 103.5c): 1 mulligan taken → 0 cards to bottom.
    // saturating_sub handles 0 mulligans (keep without mulligan) and 1 mulligan (free).
    let required_bottom = mulligans_taken.saturating_sub(1);

    if cards_to_bottom.len() as u32 != required_bottom {
        return Err(GameStateError::InvalidCommand(format!(
            "player {} must put {} cards on bottom (took {} mulligans), got {}",
            player.0,
            required_bottom,
            mulligans_taken,
            cards_to_bottom.len()
        )));
    }

    // Move each card from hand to bottom of library
    let lib_zone_id = ZoneId::Library(player);
    for obj_id in cards_to_bottom.iter() {
        state.move_object_to_zone(*obj_id, lib_zone_id)?;
    }

    events.push(GameEvent::MulliganKept {
        player,
        cards_to_bottom,
    });

    Ok(events)
}

// ── Companion (CR 702.139a) ────────────────────────────────────────────────────

/// Handle a `BringCompanion` command (CR 702.139a).
///
/// CR 702.139a: Once per game, any time you could cast a sorcery (during your main
/// phase when the stack is empty and you have priority), you may pay {3} to put
/// your companion from outside the game into your hand. This is a special action.
///
/// Validates:
/// - Player has a companion registered (`PlayerState.companion`)
/// - Player has not yet used this action (`companion_used == false`)
/// - It is the player's main phase (CR 702.139a: sorcery speed)
/// - The stack is empty (CR 702.139a)
/// - Player is the active player (sorcery speed)
/// - Player has {3} mana available
///
/// On success: deducts {3} generic mana, moves companion card to hand, marks
/// `companion_used = true`, emits `CompanionBroughtToHand`.
pub fn handle_bring_companion(
    state: &mut GameState,
    player: PlayerId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    // Validate player exists and has a companion
    let companion_card_id = {
        let ps = state.player(player)?;
        match ps.companion.clone() {
            Some(cid) => cid,
            None => {
                return Err(GameStateError::InvalidCommand(
                    "player has no companion registered".to_string(),
                ))
            }
        }
    };

    // Validate companion not yet used
    {
        let ps = state.player(player)?;
        if ps.companion_used {
            return Err(GameStateError::InvalidCommand(
                "companion special action already used this game (CR 702.139a)".to_string(),
            ));
        }
    }

    // Validate sorcery speed: active player, main phase, empty stack
    if state.turn.active_player != player {
        return Err(GameStateError::InvalidCommand(
            "companion special action requires active player (sorcery speed, CR 702.139a)"
                .to_string(),
        ));
    }

    let is_main_phase =
        state.turn.step == Step::PreCombatMain || state.turn.step == Step::PostCombatMain;

    if !is_main_phase {
        return Err(GameStateError::InvalidCommand(
            "companion special action requires main phase (CR 702.139a)".to_string(),
        ));
    }

    if !state.stack_objects.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "companion special action requires empty stack (CR 702.139a)".to_string(),
        ));
    }

    // Validate {3} mana available and deduct it
    {
        let ps = state.player(player)?;
        let total_mana = ps.mana_pool.total();
        if total_mana < 3 {
            return Err(GameStateError::InsufficientMana);
        }
    }

    // Deduct 3 generic mana from pool (colorless first, then any color)
    {
        let ps = state.player_mut(player)?;
        let mut remaining = 3u32;
        // Deduct colorless first
        let colorless_used = remaining.min(ps.mana_pool.colorless);
        ps.mana_pool.colorless -= colorless_used;
        remaining -= colorless_used;
        if remaining > 0 {
            let green_used = remaining.min(ps.mana_pool.green);
            ps.mana_pool.green -= green_used;
            remaining -= green_used;
        }
        if remaining > 0 {
            let red_used = remaining.min(ps.mana_pool.red);
            ps.mana_pool.red -= red_used;
            remaining -= red_used;
        }
        if remaining > 0 {
            let black_used = remaining.min(ps.mana_pool.black);
            ps.mana_pool.black -= black_used;
            remaining -= black_used;
        }
        if remaining > 0 {
            let blue_used = remaining.min(ps.mana_pool.blue);
            ps.mana_pool.blue -= blue_used;
            remaining -= blue_used;
        }
        if remaining > 0 {
            let white_used = remaining.min(ps.mana_pool.white);
            ps.mana_pool.white -= white_used;
            remaining -= white_used;
        }
        // remaining should be 0 now (we checked total_mana >= 3 above)
        debug_assert_eq!(remaining, 0);
    }

    // Emit mana cost paid event
    events.push(GameEvent::ManaCostPaid {
        player,
        cost: ManaCost {
            generic: 3,
            ..Default::default()
        },
    });

    // Find the companion card object in the command zone (or treat as not yet in game)
    // The companion's card is in the player's command zone (stored there at setup)
    let companion_zone_id = ZoneId::Command(player);
    let companion_obj_id = state.zones.get(&companion_zone_id).and_then(|z| {
        z.object_ids().into_iter().find(|oid| {
            state
                .objects
                .get(oid)
                .and_then(|o| o.card_id.as_ref())
                .map(|cid| cid == &companion_card_id)
                .unwrap_or(false)
        })
    });

    // If companion is in command zone, move it to hand
    if let Some(obj_id) = companion_obj_id {
        let hand_zone_id = ZoneId::Hand(player);
        state.move_object_to_zone(obj_id, hand_zone_id)?;
    }
    // If not in command zone, it may have been moved there; either way emit the event

    // Mark companion as used
    if let Some(ps) = state.players.get_mut(&player) {
        ps.companion_used = true;
    }

    events.push(GameEvent::CompanionBroughtToHand {
        player,
        card_id: companion_card_id,
    });

    Ok(events)
}

/// Add colors present in a mana cost to the accumulator.
fn add_colors_from_mana_cost(cost: &ManaCost, colors: &mut Vec<Color>) {
    if cost.white > 0 && !colors.contains(&Color::White) {
        colors.push(Color::White);
    }
    if cost.blue > 0 && !colors.contains(&Color::Blue) {
        colors.push(Color::Blue);
    }
    if cost.black > 0 && !colors.contains(&Color::Black) {
        colors.push(Color::Black);
    }
    if cost.red > 0 && !colors.contains(&Color::Red) {
        colors.push(Color::Red);
    }
    if cost.green > 0 && !colors.contains(&Color::Green) {
        colors.push(Color::Green);
    }
}
