// Propaganda — {2}{U}, Enchantment
// Creatures can't attack you unless their controller pays {2} for each creature
// they control that's attacking you.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("propaganda"),
        name: "Propaganda".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures can't attack you unless their controller pays {2} for each creature they control that's attacking you.".to_string(),
        abilities: vec![
            AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::CantAttackYouUnlessPay {
                    cost_per_creature: ManaCost { generic: 2, ..Default::default() },
                },
            },
        ],
        ..Default::default()
    }
}
