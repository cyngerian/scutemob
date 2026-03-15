// 69. Ministrant of Obligation — {2}{W}, Creature — Human Cleric 2/1; Afterlife 2.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ministrant-of-obligation"),
        name: "Ministrant of Obligation".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Cleric"]),
        oracle_text: "Afterlife 2 (When this creature dies, create two 1/1 white and black Spirit creature tokens with flying.)".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Afterlife(2)),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}
