// Winged Words — {2}{U}, Sorcery
// This spell costs {1} less to cast if you control a creature with flying.
// Draw two cards.
//
// TODO: Conditional cost reduction "if you control a creature with flying" not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("winged-words"),
        name: "Winged Words".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "This spell costs {1} less to cast if you control a creature with flying.\nDraw two cards.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
