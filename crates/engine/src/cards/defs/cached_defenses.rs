// Cached Defenses — {2}{G}, Sorcery; bolster 3.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cached-defenses"),
        name: "Cached Defenses".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Bolster 3. (Choose a creature with the least toughness among creatures you control and put three +1/+1 counters on it.)".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Bolster {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(3),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
