// Follow the Bodies — {2}{U}, Sorcery; Gravestorm, Investigate.
// CR 702.69a: Gravestorm — copy for each permanent put into a graveyard this turn.
// CR 701.16a: Investigate — create a Clue token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("follow-the-bodies"),
        name: "Follow the Bodies".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Gravestorm (When you cast this spell, copy it for each permanent put into a graveyard from the battlefield this turn.)\nInvestigate. (Create a Clue token. It's an artifact with \"{2}, Sacrifice this token: Draw a card.\")".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Gravestorm),
            AbilityDefinition::Spell {
                effect: Effect::Investigate { count: EffectAmount::Fixed(1) },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
