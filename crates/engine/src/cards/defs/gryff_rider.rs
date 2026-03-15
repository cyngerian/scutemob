// Gryff Rider — {2}{W}, Creature — Human Knight 2/1; Flying, Training
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gryff-rider"),
        name: "Gryff Rider".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Knight"]),
        oracle_text: "Flying\nTraining (Whenever this creature attacks with another creature with greater power, put a +1/+1 counter on this creature.)".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Training),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    }
}
