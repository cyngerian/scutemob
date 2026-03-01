//! Commander format rules: deck validation, commander tax, color identity, mulligan,
//! companion, and partner mechanics.
//!
//! See architecture doc Section 3.x and CR 903 for design rationale.

use crate::cards::{CardDefinition, CardRegistry};
use crate::rules::casting;
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
    /// Architecture Invariant 9: every card must have a CardDefinition in the registry.
    ///
    /// A card_id present in the deck has no corresponding definition. This is
    /// always an error — unknown cards silently pass all other validation checks.
    UnknownCard { card_id: String },
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
            None => {
                // Architecture Invariant 9: every card must have a definition.
                // Silently skipping unknown cards permits illegal decks — return a violation.
                violations.push(DeckViolation::UnknownCard {
                    card_id: card_id.0.clone(),
                });
                continue;
            }
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
/// This implementation extracts colors from both the mana cost and from mana symbols
/// appearing in the oracle text (e.g., `{W}`, `{U}`, `{B}`, `{R}`, `{G}`, and hybrid
/// symbols such as `{W/B}` or `{2/W}`). This correctly handles commanders like
/// Alesha, Who Smiles at Death ({W/B} in ability text → Mardu identity).
pub fn compute_color_identity(def: &CardDefinition) -> Vec<Color> {
    let mut colors = Vec::new();

    // Extract colors from mana cost symbols
    if let Some(ref cost) = def.mana_cost {
        add_colors_from_mana_cost(cost, &mut colors);
    }

    // CR 903.4: also extract colors from mana symbols in oracle text.
    // Scan for `{...}` symbols and add any colored mana they contain.
    add_colors_from_oracle_text(&def.oracle_text, &mut colors);

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
///
/// Uses saturating arithmetic to prevent overflow (MR-M9-03). In the impossible
/// case that `tax * 2` or the resulting sum exceeds `u32::MAX`, the value
/// saturates at `u32::MAX` rather than panicking or wrapping.
pub fn apply_commander_tax(base_cost: &ManaCost, tax: u32) -> ManaCost {
    let tax_mana = tax.saturating_mul(2);
    ManaCost {
        generic: base_cost.generic.saturating_add(tax_mana),
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
/// if so, emit a `CommanderZoneReturnChoiceRequired` event for the owner.
///
/// CR 903.9a: The owner **may** put the commander into the command zone — it is
/// optional. This SBA emits a choice event and records the pending choice;
/// the actual move happens only when the owner responds with
/// `ReturnCommanderToCommandZone`. If the owner prefers to leave the commander
/// in graveyard or exile (e.g., for reanimation), they respond with
/// `LeaveCommanderInZone` instead.
///
/// Commanders already in `pending_commander_zone_choices` are skipped so the
/// choice event is not re-emitted on each SBA pass.
///
/// Called from `sba::apply_sbas_once` after counter annihilation.
pub fn check_commander_zone_return_sba(state: &mut GameState) -> Vec<GameEvent> {
    let mut events = Vec::new();

    // Collect (owner, card_id, object_id, from_zone_type) for all commanders
    // found in graveyard or exile that don't already have a pending choice.
    let mut needs_choice: Vec<(PlayerId, CardId, ObjectId, ZoneType)> = Vec::new();

    for (&owner, player_state) in state.players.iter() {
        for card_id in player_state.commander_ids.iter() {
            // Check graveyard
            let graveyard_zone_id = ZoneId::Graveyard(owner);
            if let Some(zone) = state.zones.get(&graveyard_zone_id) {
                for obj_id in zone.object_ids() {
                    // Skip if already awaiting a choice for this object.
                    if state
                        .pending_commander_zone_choices
                        .iter()
                        .any(|(_, oid)| *oid == obj_id)
                    {
                        continue;
                    }
                    if let Some(obj) = state.objects.get(&obj_id) {
                        if obj.card_id.as_ref() == Some(card_id) {
                            needs_choice.push((
                                owner,
                                card_id.clone(),
                                obj_id,
                                ZoneType::Graveyard,
                            ));
                        }
                    }
                }
            }

            // Check exile
            let exile_zone_id = ZoneId::Exile;
            if let Some(zone) = state.zones.get(&exile_zone_id) {
                for obj_id in zone.object_ids() {
                    // Skip if already awaiting a choice for this object.
                    if state
                        .pending_commander_zone_choices
                        .iter()
                        .any(|(_, oid)| *oid == obj_id)
                    {
                        continue;
                    }
                    if let Some(obj) = state.objects.get(&obj_id) {
                        if obj.card_id.as_ref() == Some(card_id) && obj.owner == owner {
                            needs_choice.push((owner, card_id.clone(), obj_id, ZoneType::Exile));
                        }
                    }
                }
            }
        }
    }

    // For each commander needing a choice, record the pending entry and emit the event.
    for (owner, card_id, object_id, from_zone) in needs_choice {
        state
            .pending_commander_zone_choices
            .push_back((owner, object_id));
        events.push(GameEvent::CommanderZoneReturnChoiceRequired {
            owner,
            card_id,
            object_id,
            from_zone,
        });
    }

    events
}

/// Handle a `LeaveCommanderInZone` command (CR 903.9a / CR 704.6d).
///
/// The owner has chosen to leave their commander in its current zone (graveyard
/// or exile) rather than returning it to the command zone. Clears the pending
/// choice recorded by `check_commander_zone_return_sba`.
pub fn handle_leave_commander_in_zone(
    state: &mut GameState,
    player: PlayerId,
    object_id: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // Validate the object exists and belongs to this player.
    let obj = state
        .objects
        .get(&object_id)
        .ok_or(GameStateError::ObjectNotFound(object_id))?;

    if obj.owner != player {
        return Err(GameStateError::InvalidCommand(
            "can only decide zone for your own commander".to_string(),
        ));
    }

    // Remove the pending choice entry.
    let before = state.pending_commander_zone_choices.len();
    state
        .pending_commander_zone_choices
        .retain(|(_, oid)| *oid != object_id);
    let removed = before - state.pending_commander_zone_choices.len();

    if removed == 0 {
        return Err(GameStateError::InvalidCommand(
            "no pending commander zone-return choice for this object".to_string(),
        ));
    }

    // No zone move — commander stays where it is. No event emitted beyond
    // confirming the pending choice is cleared.
    Ok(vec![])
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

    // Clear any pending choice for this object (may not exist if command issued
    // independently without a prior SBA choice event).
    state
        .pending_commander_zone_choices
        .retain(|(_, oid)| *oid != object_id);

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
///
/// CR 702.124f: Different partner abilities are distinct and cannot be combined.
/// A card with "partner with [name]" can ONLY partner with the specifically named
/// card. It cannot combine with plain "partner" or other partner variants.
///
/// CR 702.124j: "Partner with [name]" allows the named pair as co-commanders,
/// provided each has a 'partner with' ability naming the other (cross-reference).
pub fn validate_partner_commanders(
    cmd1: &CardDefinition,
    cmd2: &CardDefinition,
) -> Result<(), String> {
    use crate::state::KeywordAbility;

    // Check for plain Partner keyword.
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

    // Check for "Partner with [name]" keyword.
    let cmd1_partner_with: Option<&str> = cmd1.abilities.iter().find_map(|a| {
        if let crate::cards::AbilityDefinition::Keyword(KeywordAbility::PartnerWith(name)) = a {
            Some(name.as_str())
        } else {
            None
        }
    });
    let cmd2_partner_with: Option<&str> = cmd2.abilities.iter().find_map(|a| {
        if let crate::cards::AbilityDefinition::Keyword(KeywordAbility::PartnerWith(name)) = a {
            Some(name.as_str())
        } else {
            None
        }
    });

    // Case 1: Both have plain Partner — valid pair (CR 702.124h).
    if cmd1_has_partner && cmd2_has_partner {
        return Ok(());
    }

    // Case 2: Both have PartnerWith — verify names cross-reference each other
    // (CR 702.124j: each must have 'partner with [name]' naming the other).
    if let (Some(pw1), Some(pw2)) = (cmd1_partner_with, cmd2_partner_with) {
        if pw1 == cmd2.name && pw2 == cmd1.name {
            return Ok(());
        } else {
            return Err(format!(
                "'{}' has partner with '{}' but '{}' has partner with '{}' \
                 -- names don't match (CR 702.124j)",
                cmd1.name, pw1, cmd2.name, pw2
            ));
        }
    }

    // Case 3: Mixed Partner + PartnerWith — not allowed (CR 702.124f).
    if (cmd1_has_partner && cmd2_partner_with.is_some())
        || (cmd2_has_partner && cmd1_partner_with.is_some())
    {
        return Err(format!(
            "'Partner' and 'Partner with [name]' cannot be combined (CR 702.124f): \
             '{}' and '{}'",
            cmd1.name, cmd2.name
        ));
    }

    // Case 4: One has PartnerWith but the other has nothing — incomplete pair.
    if cmd1_partner_with.is_some() || cmd2_partner_with.is_some() {
        return Err(format!(
            "partner with pairing incomplete: '{}' and '{}' (CR 702.124j)",
            cmd1.name, cmd2.name
        ));
    }

    // Case 5: Neither has plain Partner and neither has PartnerWith.
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

    Err(format!(
        "'{}' does not have partner (CR 702.124h)",
        cmd2.name
    ))
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

    // Draw 7 cards for the new hand.
    // MR-M9-05: Do not use draw_card here — it triggers PlayerLost on empty library.
    // During the pregame mulligan procedure (CR 103.5) the game has not started yet;
    // drawing from an empty library must not cause a game loss. Instead we move cards
    // directly without the loss check.
    for _ in 0..7 {
        let lib_zone = ZoneId::Library(player);
        let top = state.zones.get(&lib_zone).and_then(|z| z.top());
        match top {
            Some(top_id) => {
                let hand_zone = ZoneId::Hand(player);
                let (new_id, _) = state.move_object_to_zone(top_id, hand_zone)?;
                events.push(GameEvent::CardDrawn {
                    player,
                    new_object_id: new_id,
                });
            }
            None => {
                // Library exhausted during pregame mulligan draw — stop drawing.
                // No game loss is triggered (CR 103.5: mulligan is a pregame procedure).
                break;
            }
        }
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

    if cards_to_bottom.len() != required_bottom as usize {
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

    // Deduct 3 generic mana from pool via shared pay_cost logic.
    {
        let companion_cost = ManaCost {
            generic: 3,
            ..Default::default()
        };
        let ps = state.player_mut(player)?;
        casting::pay_cost(&mut ps.mana_pool, &companion_cost);
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

/// CR 903.4: Scan oracle text for mana symbols and add any colored mana to the accumulator.
///
/// Handles simple colored symbols `{W}`, `{U}`, `{B}`, `{R}`, `{G}` and hybrid
/// symbols such as `{W/B}`, `{W/U}`, `{2/W}`, `{R/G}` etc. Each character within
/// a `{...}` symbol is checked for a color initial (W/U/B/R/G).
fn add_colors_from_oracle_text(text: &str, colors: &mut Vec<Color>) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            // Find matching '}'
            if let Some(close) = bytes[i..].iter().position(|&b| b == b'}') {
                let symbol = &text[i + 1..i + close];
                for ch in symbol.chars() {
                    let color = match ch {
                        'W' => Some(Color::White),
                        'U' => Some(Color::Blue),
                        'B' => Some(Color::Black),
                        'R' => Some(Color::Red),
                        'G' => Some(Color::Green),
                        _ => None,
                    };
                    if let Some(c) = color {
                        if !colors.contains(&c) {
                            colors.push(c);
                        }
                    }
                }
                i += close + 1;
                continue;
            }
        }
        i += 1;
    }
}
