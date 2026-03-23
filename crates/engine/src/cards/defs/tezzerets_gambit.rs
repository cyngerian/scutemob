// Tezzeret's Gambit — {3}{U/P}, Sorcery
// ({U/P} can be paid with either {U} or 2 life.)
// Draw two cards, then proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tezzerets-gambit"),
        name: "Tezzeret's Gambit".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Blue)],
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "({U/P} can be paid with either {U} or 2 life.)\nDraw two cards, then proliferate.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                Effect::Proliferate,
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
