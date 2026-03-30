// Consecrated Sphinx — {4}{U}{U}, Creature — Sphinx 4/6
// Flying
// Whenever an opponent draws a card, you may draw two cards.
//
// The "you may" optional is approximated as mandatory (bot always draws).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("consecrated-sphinx"),
        name: "Consecrated Sphinx".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 2, ..Default::default() }),
        types: creature_types(&["Sphinx"]),
        oracle_text: "Flying\nWhenever an opponent draws a card, you may draw two cards.".to_string(),
        power: Some(4),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Whenever an opponent draws a card, you may draw two cards.
            // CR 603.2: "Whenever an opponent draws a card, you may draw two cards."
            // Note: "you may" optional not in DSL — always draws (bot always opts in).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPlayerDrawsCard {
                    player_filter: Some(TargetController::Opponent),
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
