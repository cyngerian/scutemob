// Reverse Engineer — {3UU}, Sorcery; improvise, draw 3 cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reverse-engineer"),
        name: "Reverse Engineer".to_string(),
        mana_cost: Some(ManaCost { blue: 2, generic: 3, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Improvise (Your artifacts can help cast this spell. Each artifact you tap after you're done activating mana abilities pays for {1}.)\nDraw three cards.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Improvise),
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
