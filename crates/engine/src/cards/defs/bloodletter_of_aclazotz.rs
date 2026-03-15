// Bloodletter of Aclazotz — {1}{B}{B}{B}, Creature — Vampire Demon 2/4
// Flying
// If an opponent would lose life during your turn, they lose twice that much life instead.
//
// Note: The "during your turn" condition requires runtime checking of the active player.
// The replacement is registered unconditionally; the apply_life_loss_doubling helper
// will fire whenever life loss occurs. A proper implementation would add a turn-check
// condition to the replacement. For now, the replacement fires on all opponent life loss
// regardless of whose turn it is (conservative — doubles more than it should).
// TODO: Add turn-condition checking to WouldLoseLife replacement (requires Condition support
// on ReplacementEffect, similar to unless_condition on ETB replacements).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodletter-of-aclazotz"),
        name: "Bloodletter of Aclazotz".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 3, ..Default::default() }),
        types: creature_types(&["Vampire", "Demon"]),
        oracle_text: "Flying\nIf an opponent would lose life during your turn, they lose twice that much life instead. (Damage causes loss of life.)".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 614.1: Life-loss doubling for opponents.
            // PlayerId(0) placeholder — bound to controller at registration.
            // NOTE: Missing "during your turn" condition — see file header.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldLoseLife {
                    player_filter: PlayerFilter::OpponentsOf(PlayerId(0)),
                },
                modification: ReplacementModification::DoubleLifeLoss,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}
