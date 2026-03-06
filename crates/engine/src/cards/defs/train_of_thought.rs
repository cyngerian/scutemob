// Train of Thought — {1}{U}, Sorcery; Replicate {1}{U}, draw a card.
// CR 702.56: Replicate — optional additional cost paid any number of times; each payment copies the spell.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("train-of-thought"),
        name: "Train of Thought".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Replicate {1}{U} (When you cast this spell, copy it for each time you paid its replicate cost. You may choose new targets for the copies.)\nDraw a card.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Replicate),
            AbilityDefinition::Replicate {
                cost: ManaCost { generic: 1, blue: 1, ..Default::default() },
            },
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
