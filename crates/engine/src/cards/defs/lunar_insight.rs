// Lunar Insight — {2}{U}, Sorcery
// Draw a card for each different mana value among nonland permanents you control.
//
// TODO: "Different mana values" count not expressible. Using Fixed(3) placeholder.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lunar-insight"),
        name: "Lunar Insight".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw a card for each different mana value among nonland permanents you control.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Unique mana value count not expressible.
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
