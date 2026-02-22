//! Casting spells (CR 601).
//!
//! A spell is cast by moving a card from the caster's hand to the Stack zone
//! and placing a `StackObject` onto `GameState::stack_objects`. After casting,
//! the active player receives priority (CR 601.2i).
//!
//! Casting speed (CR 601.3):
//! - **Instant speed**: Instants, and any spell with Flash (CR 702.36), may be
//!   cast whenever the player has priority.
//! - **Sorcery speed**: All other spells may only be cast during the active
//!   player's precombat or postcombat main phase while the stack is empty
//!   (CR 307.1).
//!
//! **Targets (CR 601.2c)**: Targets are announced at cast time and validated
//! for existence. The zone of each object target is recorded for the fizzle
//! rule checked at resolution (CR 608.2b).
//!
//! **Mana cost (CR 601.2f-h)**: If the spell has a mana cost, the caster's
//! mana pool must cover it. The cost is deducted from the pool when the spell
//! is cast. Spells with no mana cost (e.g., `mana_cost: None`) are cast for free.

use im::OrdSet;

use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::stack::{StackObject, StackObjectKind};
use crate::state::targeting::{SpellTarget, Target};
use crate::state::turn::Step;
use crate::state::types::{CardType, KeywordAbility};
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::events::GameEvent;

/// Handle a CastSpell command: move a card from hand to the stack.
///
/// Validates the casting window, validates targets, pays the mana cost, moves
/// the card to `ZoneId::Stack`, creates a `StackObject`, resets priority to
/// the active player (CR 601.2i), and returns the events produced.
pub fn handle_cast_spell(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<Vec<GameEvent>, GameStateError> {
    // CR 601.2: Casting a spell requires priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // Fetch the card and validate it is in the player's hand.
    let card_obj = state.object(card)?;
    if card_obj.zone != ZoneId::Hand(player) {
        return Err(GameStateError::InvalidCommand(
            "card is not in your hand".into(),
        ));
    }

    // Lands are not cast — they are played as a special action (CR 305.1).
    if card_obj
        .characteristics
        .card_types
        .contains(&CardType::Land)
    {
        return Err(GameStateError::InvalidCommand(
            "lands are played with PlayLand, not cast".into(),
        ));
    }

    // Determine casting speed (CR 601.3).
    let is_instant_speed = card_obj
        .characteristics
        .card_types
        .contains(&CardType::Instant)
        || card_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Flash);

    // Validate casting window.
    if !is_instant_speed {
        // Sorcery speed: active player only, main phase, empty stack (CR 307.1).
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "sorcery-speed spells can only be cast during your own turn".into(),
            ));
        }
        if !matches!(state.turn.step, Step::PreCombatMain | Step::PostCombatMain) {
            return Err(GameStateError::NotMainPhase);
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::StackNotEmpty);
        }
    }

    // CR 601.2c: Validate and record targets at cast time.
    let spell_targets = validate_targets(state, &targets)?;

    // CR 601.2f-h: Pay the mana cost if the card has one.
    let mana_cost = card_obj.characteristics.mana_cost.clone();
    let mut events = Vec::new();

    if let Some(ref cost) = mana_cost {
        if cost.mana_value() > 0 {
            // Check the player has enough mana.
            let player_state = state.player_mut(player)?;
            if !can_pay_cost(&player_state.mana_pool, cost) {
                return Err(GameStateError::InsufficientMana);
            }
            pay_cost(&mut player_state.mana_pool, cost);
            events.push(GameEvent::ManaCostPaid {
                player,
                cost: cost.clone(),
            });
        }
    }

    // CR 601.2c: Move the card to the Stack zone (CR 400.7: new ObjectId).
    let (new_card_id, _old_obj) = state.move_object_to_zone(card, ZoneId::Stack)?;

    // CR 601.2: Create the StackObject and push it (LIFO — last in, first out).
    let stack_entry_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_entry_id,
        controller: player,
        kind: StackObjectKind::Spell {
            source_object: new_card_id,
        },
        targets: spell_targets,
    };
    state.stack_objects.push_back(stack_obj);

    // CR 601.2i: "Then the active player receives priority."
    // Reset the priority round — a game action occurred.
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(state.turn.active_player);

    events.push(GameEvent::SpellCast {
        player,
        stack_object_id: stack_entry_id,
        source_object_id: new_card_id,
    });
    events.push(GameEvent::PriorityGiven {
        player: state.turn.active_player,
    });

    Ok(events)
}

/// CR 601.2c: Validate targets at cast time and snapshot their current zones.
///
/// For each target:
/// - Player: must be an active (non-eliminated) player
/// - Object: must exist in the game; records current zone for fizzle detection
///
/// Full type-restriction validation (e.g., "target creature") is deferred to M7
/// when card definitions supply targeting criteria.
fn validate_targets(
    state: &GameState,
    targets: &[Target],
) -> Result<Vec<SpellTarget>, GameStateError> {
    let mut spell_targets = Vec::with_capacity(targets.len());

    for target in targets {
        let spell_target = match target {
            Target::Player(id) => {
                let player = state
                    .players
                    .get(id)
                    .ok_or(GameStateError::PlayerNotFound(*id))?;
                if player.has_lost || player.has_conceded {
                    return Err(GameStateError::InvalidTarget(format!(
                        "player {:?} is not an active player",
                        id
                    )));
                }
                SpellTarget {
                    target: Target::Player(*id),
                    zone_at_cast: None, // Players are not in a zone
                }
            }
            Target::Object(id) => {
                let obj = state
                    .objects
                    .get(id)
                    .ok_or(GameStateError::ObjectNotFound(*id))?;
                SpellTarget {
                    target: Target::Object(*id),
                    zone_at_cast: Some(obj.zone),
                }
            }
        };
        spell_targets.push(spell_target);
    }

    Ok(spell_targets)
}

/// Returns true if the mana pool can cover the mana cost.
///
/// Colored mana (W/U/B/R/G) must be paid with the matching color.
/// Colorless mana (`{C}`) must be paid with colorless mana specifically (CR 106.1).
/// Generic mana (`{N}`) can be paid with any remaining mana in the pool.
pub fn can_pay_cost(
    pool: &crate::state::player::ManaPool,
    cost: &crate::state::game_object::ManaCost,
) -> bool {
    if pool.white < cost.white {
        return false;
    }
    if pool.blue < cost.blue {
        return false;
    }
    if pool.black < cost.black {
        return false;
    }
    if pool.red < cost.red {
        return false;
    }
    if pool.green < cost.green {
        return false;
    }
    if pool.colorless < cost.colorless {
        return false;
    }

    // Remaining mana after paying colored and colorless requirements.
    let remaining = (pool.white - cost.white)
        + (pool.blue - cost.blue)
        + (pool.black - cost.black)
        + (pool.red - cost.red)
        + (pool.green - cost.green)
        + (pool.colorless - cost.colorless);

    remaining >= cost.generic
}

/// Deduct a mana cost from the mana pool. Caller must verify `can_pay_cost` first.
///
/// For generic mana, mana is taken from remaining colored/colorless in order:
/// colorless, then green, red, black, blue, white. The specific order doesn't
/// affect correctness since generic can use any color.
pub fn pay_cost(
    pool: &mut crate::state::player::ManaPool,
    cost: &crate::state::game_object::ManaCost,
) {
    pool.white -= cost.white;
    pool.blue -= cost.blue;
    pool.black -= cost.black;
    pool.red -= cost.red;
    pool.green -= cost.green;
    pool.colorless -= cost.colorless;

    // Pay generic cost from remaining mana.
    let mut remaining = cost.generic;
    for slot in [
        &mut pool.colorless,
        &mut pool.green,
        &mut pool.red,
        &mut pool.black,
        &mut pool.blue,
        &mut pool.white,
    ] {
        let take = remaining.min(*slot);
        *slot -= take;
        remaining -= take;
        if remaining == 0 {
            break;
        }
    }
}
