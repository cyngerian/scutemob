// Bloodletter of Aclazotz — {1}{B}{B}{B}, Creature — Vampire Demon 2/4
// Flying
// If an opponent would lose life during your turn, they lose twice that much life instead.
//
// "During your turn" is enforced in apply_life_loss_doubling (replacement.rs):
// DoubleLifeLoss only applies when effect.controller == state.turn.active_player.
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
            // CR 614.1: Life-loss doubling for opponents during controller's turn.
            // PlayerId(0) placeholder — bound to controller at registration.
            // "During your turn" check: apply_life_loss_doubling checks active_player == controller.
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
