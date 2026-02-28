//! Suspend special action handler (CR 702.62).
//!
//! Suspend is a keyword representing three abilities. The first is a static ability
//! while the card is in hand: any time the player has priority and could begin to
//! cast the card, they may pay the suspend cost and exile it with N time counters
//! (CR 702.62a / CR 116.2f). This is a special action -- it does NOT use the stack.
//!
//! The second and third abilities trigger from exile:
//! - At the beginning of the owner's upkeep, remove a time counter (queued in
//!   `turn_actions::upkeep_actions`).
//! - When the last time counter is removed, cast the card without paying its mana
//!   cost (queued as SuspendCastTrigger in `resolution::resolve_top_of_stack`).
//!
//! Key rules:
//! - Special action -- no stack, no priority interruption (CR 116.2f)
//! - Legal any time the player has priority AND could begin to cast the card normally:
//!   - Instant / Flash cards: any time with priority
//!   - Others (sorcery, creature, etc.): active player, main phase, empty stack
//! - Card enters exile FACE UP (unlike foretell which is face down)
//! - Creature spells cast via the third ability gain haste (CR 702.62a)

use crate::cards::card_definition::AbilityDefinition;
use crate::rules::casting;
use crate::rules::events::GameEvent;
use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::turn::Step;
use crate::state::types::{CardType, CounterType, KeywordAbility};
use crate::state::zone::ZoneId;
use crate::state::GameState;

/// CR 702.62a / CR 116.2f: Handle the SuspendCard special action.
///
/// Validates:
/// 1. The player has priority (CR 116.2f).
/// 2. The card is in the player's hand (CR 702.62a).
/// 3. The card has `KeywordAbility::Suspend` (CR 702.62a).
/// 4. The player could begin to cast the card normally (CR 702.62c):
///    - Instants / Flash cards: any time with priority.
///    - Other card types: active player, main phase, empty stack.
/// 5. The player can pay the suspend cost.
///
/// On success:
/// - Deducts the suspend cost from the player's mana pool.
/// - Moves the card from hand to exile (CR 400.7: new ObjectId).
/// - Sets `is_suspended = true` on the new exile object (face up, not face down).
/// - Adds N time counters to the exiled card.
/// - Emits `ManaCostPaid` and `CardSuspended` events.
///
/// V1 simplifications (see plan):
/// - `SuspendCard` is a player-initiated command; auto-detect is deferred.
/// - No {X} in suspend cost handling beyond the existing cost infrastructure.
pub fn handle_suspend_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    // CR 116.2f: Suspend requires the player to have priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // Validate the card is in the player's hand.
    let (has_suspend, is_instant_speed) = {
        let card_obj = state.object(card)?;
        if card_obj.zone != ZoneId::Hand(player) {
            return Err(GameStateError::InvalidCommand(
                "suspend: card must be in your hand (CR 702.62a)".into(),
            ));
        }

        // Validate the card has the Suspend keyword (CR 702.62a).
        let has_suspend = card_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Suspend);

        // CR 702.62c: The player must be able to "begin to cast" the card.
        // Instants and Flash cards can be cast at any time; others require sorcery timing.
        let is_instant_speed = card_obj
            .characteristics
            .card_types
            .contains(&CardType::Instant)
            || card_obj
                .characteristics
                .keywords
                .contains(&KeywordAbility::Flash);

        (has_suspend, is_instant_speed)
    };

    if !has_suspend {
        return Err(GameStateError::InvalidCommand(
            "suspend: card does not have the Suspend keyword (CR 702.62a)".into(),
        ));
    }

    // CR 702.62c: Check timing restrictions for the suspend special action.
    // The player must be able to "begin to cast" the card normally.
    if !is_instant_speed {
        // Sorcery-speed timing: active player, main phase, empty stack (CR 307.1).
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "suspend: can only be used during your own turn (CR 702.62c)".into(),
            ));
        }
        if !matches!(state.turn.step, Step::PreCombatMain | Step::PostCombatMain) {
            return Err(GameStateError::NotMainPhase);
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::StackNotEmpty);
        }
    }

    // Look up AbilityDefinition::Suspend { cost, time_counters } from the card registry.
    // CR 702.62a: The suspend cost and N are printed on the card and stored in the card def.
    let (suspend_cost, time_counters) = {
        let card_obj = state.object(card)?;
        let card_id = card_obj.card_id.clone();
        let registry = state.card_registry.clone();

        card_id
            .as_ref()
            .and_then(|cid| registry.get(cid.clone()))
            .and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Suspend {
                        cost,
                        time_counters,
                    } = a
                    {
                        Some((cost.clone(), *time_counters))
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                GameStateError::InvalidCommand(
                    "suspend: card has Suspend keyword but no AbilityDefinition::Suspend".into(),
                )
            })?
    };

    // Validate and deduct the suspend cost (CR 702.62a).
    {
        let ps = state.player(player)?;
        if !casting::can_pay_cost(&ps.mana_pool, &suspend_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        let ps_mut = state.player_mut(player)?;
        casting::pay_cost(&mut ps_mut.mana_pool, &suspend_cost);
    }

    // Emit ManaCostPaid event for the suspend cost.
    events.push(GameEvent::ManaCostPaid {
        player,
        cost: suspend_cost,
    });

    // Move the card from hand to exile (CR 400.7: new ObjectId after zone change).
    let (new_exile_id, _old_obj) = state.move_object_to_zone(card, ZoneId::Exile)?;

    // Set suspend attributes on the new exile object.
    // - is_suspended: true (marks this as a suspended card for upkeep trigger scanning)
    // - face_down: false (suspended cards are exiled FACE UP, unlike foretell)
    // - Add N time counters (CR 702.62a: "with N time counters on it")
    if let Some(exile_obj) = state.objects.get_mut(&new_exile_id) {
        exile_obj.is_suspended = true;
        // Suspended cards are face up per rulings (not face down like foretell).
        // exile_obj.status.face_down = false; // already false by default
        let current = exile_obj
            .counters
            .get(&CounterType::Time)
            .copied()
            .unwrap_or(0);
        exile_obj.counters = exile_obj
            .counters
            .update(CounterType::Time, current + time_counters);
    }

    // Emit CardSuspended event.
    events.push(GameEvent::CardSuspended {
        player,
        object_id: card,
        new_exile_id,
        time_counters,
    });

    Ok(events)
}
