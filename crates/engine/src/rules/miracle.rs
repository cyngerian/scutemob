//! Miracle keyword ability handler (CR 702.94).
//!
//! Miracle is a static ability linked to a triggered ability (CR 702.94a):
//! "You may reveal this card from your hand as you draw it if it's the first
//! card you've drawn this turn. When you reveal this card this way, you may
//! cast it by paying [cost] rather than its mana cost."
//!
//! Implementation:
//! 1. When a miracle card is drawn as the first draw of the turn, the engine
//!    emits `GameEvent::MiracleRevealChoiceRequired`.
//! 2. The player responds with `Command::ChooseMiracle { reveal: true/false }`.
//! 3. If `reveal: true`, a `PendingTrigger` with `is_miracle_trigger: true` is
//!    queued and flushed to the stack as a `StackObjectKind::MiracleTrigger`.
//! 4. While the MiracleTrigger is on the stack, the player may cast the card
//!    from hand using `CastSpell` with `cast_with_miracle: true` (handled in
//!    `casting.rs`). The miracle cost is used as the alternative cost.
//! 5. When the MiracleTrigger resolves, it expires. The card stays in hand if
//!    the player did not cast it.

use crate::cards::card_definition::AbilityDefinition;
use crate::state::error::GameStateError;
use crate::state::game_object::{ManaCost, ObjectId};
use crate::state::player::PlayerId;
use crate::state::stubs::{PendingTrigger, PendingTriggerKind};
use crate::state::types::KeywordAbility;
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::events::GameEvent;

/// CR 702.94a: Handle a player's miracle reveal choice.
///
/// Called from `engine.rs::process_command` when a `Command::ChooseMiracle` is received.
///
/// If `reveal` is `true`:
///   1. Validate the card is in the player's hand with `KeywordAbility::Miracle`.
///   2. Validate `cards_drawn_this_turn == 1` (it was the first draw this turn).
///   3. Look up the miracle cost from the card registry.
///   4. Queue a `PendingTrigger` with `is_miracle_trigger: true`.
///      (Caller calls `flush_pending_triggers` to put it on the stack.)
///
/// If `reveal` is `false`:
///   - Do nothing. Card stays in hand as a normal draw.
///
/// Returns the events produced (empty for `reveal: false`; priority events for `reveal: true`).
pub fn handle_choose_miracle(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    reveal: bool,
) -> Result<Vec<GameEvent>, GameStateError> {
    if !reveal {
        // CR 702.94a: Player declined reveal. Card stays in hand. No trigger.
        return Ok(vec![]);
    }

    // Validate the card is in the player's hand.
    let (card_id_opt, miracle_cost) = {
        let obj = state
            .objects
            .get(&card)
            .ok_or_else(|| GameStateError::InvalidCommand("miracle card not found".into()))?;

        if obj.zone != ZoneId::Hand(player) {
            return Err(GameStateError::InvalidCommand(
                "miracle: card must be in your hand (CR 702.94a)".into(),
            ));
        }

        // Validate the card has the Miracle keyword.
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Miracle)
        {
            return Err(GameStateError::InvalidCommand(
                "miracle: card does not have the Miracle keyword (CR 702.94a)".into(),
            ));
        }

        (obj.card_id.clone(), lookup_miracle_cost(state, card))
    };

    let miracle_cost = miracle_cost.ok_or_else(|| {
        GameStateError::InvalidCommand(
            "miracle: card has Miracle keyword but no miracle cost defined".into(),
        )
    })?;

    // Validate this is the first draw of the turn (CR 702.94a).
    let cards_drawn = state
        .players
        .get(&player)
        .map(|p| p.cards_drawn_this_turn)
        .unwrap_or(0);
    if cards_drawn != 1 {
        return Err(GameStateError::InvalidCommand(
            "miracle: can only reveal miracle card on the first draw of the turn (CR 702.94a)"
                .into(),
        ));
    }

    // Queue the miracle trigger.
    let source = card_id_opt
        .map(|_| card) // use the current ObjectId as the source
        .unwrap_or(card);

    state.pending_triggers.push_back(PendingTrigger {
        source,
        ability_index: 0, // unused for miracle triggers
        controller: player,
        kind: PendingTriggerKind::Miracle,
        triggering_event: None,
        entering_object_id: None,
        targeting_stack_id: None,
        triggering_player: None,
        exalted_attacker_id: None,
        defending_player_id: None,
        madness_exiled_card: None,
        madness_cost: None,
        miracle_revealed_card: Some(card),
        miracle_cost: Some(miracle_cost),
        modular_counter_count: None,
        evolve_entering_creature: None,
        suspend_card_id: None,
        hideaway_count: None,
        partner_with_name: None,
        ingest_target_player: None,
        flanking_blocker_id: None,
        rampage_n: None,
        provoke_target_creature: None,
        renown_n: None,
        poisonous_n: None,
        poisonous_target_player: None,
        enlist_enlisted_creature: None,
        encore_activator: None,
        echo_cost: None,
        cumulative_upkeep_cost: None,
        recover_cost: None,
        recover_card: None,
        graft_entering_creature: None,
        backup_abilities: None,
        backup_n: None,
        champion_filter: None,
        champion_exiled_card: None,
    });

    Ok(vec![])
}

/// CR 702.94a: Check if a just-drawn card has miracle and is eligible for reveal.
///
/// Returns `Some(MiracleRevealChoiceRequired)` if:
///   1. `player.cards_drawn_this_turn == 1` (it was the first draw this turn).
///   2. The card has `KeywordAbility::Miracle`.
///   3. The card has an `AbilityDefinition::Miracle { cost }` in the card registry.
///
/// Returns `None` if the card is not miracle-eligible.
pub fn check_miracle_eligible(
    state: &GameState,
    player: PlayerId,
    drawn_card_id: ObjectId,
) -> Option<GameEvent> {
    // Step 1: Was this the first draw of the turn?
    let cards_drawn = state.players.get(&player)?.cards_drawn_this_turn;
    if cards_drawn != 1 {
        return None;
    }

    // Step 2: Does the drawn card have the Miracle keyword?
    let obj = state.objects.get(&drawn_card_id)?;
    if !obj
        .characteristics
        .keywords
        .contains(&KeywordAbility::Miracle)
    {
        return None;
    }

    // Step 3: Look up the miracle cost from the card definition.
    let miracle_cost = lookup_miracle_cost(state, drawn_card_id)?;

    Some(GameEvent::MiracleRevealChoiceRequired {
        player,
        card_object_id: drawn_card_id,
        miracle_cost,
    })
}

/// Look up the miracle cost for the given card object from the registry.
fn lookup_miracle_cost(state: &GameState, card: ObjectId) -> Option<ManaCost> {
    let card_id = state.objects.get(&card)?.card_id.clone();
    card_id.and_then(|cid| {
        state.card_registry.get(cid).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Miracle { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
