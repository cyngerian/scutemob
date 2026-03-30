// Spectral Sailor — {U}, Creature — Spirit Pirate 1/1
// Flash
// Flying
// {3}{U}: Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spectral-sailor"),
        name: "Spectral Sailor".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Spirit", "Pirate"]),
        oracle_text: "Flash\nFlying\n{3}{U}: Draw a card.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 3, blue: 1, ..Default::default() }),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
