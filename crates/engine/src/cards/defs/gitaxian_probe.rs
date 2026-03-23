// Gitaxian Probe — {U/P}, Sorcery
// ({U/P} can be paid with either {U} or 2 life.)
// Look at target player's hand. Draw a card.
//
// TODO: "Look at hand" — hidden info reveal not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gitaxian-probe"),
        name: "Gitaxian Probe".to_string(),
        mana_cost: Some(ManaCost {
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Blue)],
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "({U/P} can be paid with either {U} or 2 life.)\nLook at target player's hand.\nDraw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "Look at hand" not expressible. Draw only.
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
