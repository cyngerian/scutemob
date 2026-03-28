// Two-Headed Hellkite — {1}{W}{U}{B}{R}{G}, Creature — Dragon 5/5
// Flying, menace, haste
// Whenever this creature attacks, draw two cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("two-headed-hellkite"),
        name: "Two-Headed Hellkite".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, blue: 1, black: 1, red: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying, menace, haste\nWhenever this creature attacks, draw two cards.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
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
