// 35. Treasure Cruise — {7U}, Sorcery; delve, draw 3 cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("treasure-cruise"),
        name: "Treasure Cruise".to_string(),
        mana_cost: Some(ManaCost { blue: 1, generic: 7, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Delve (Each card you exile from your graveyard while casting this spell pays for {1}.)\nDraw three cards.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Delve),
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
