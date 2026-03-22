// 64. Young Wolf — {G}, Creature — Wolf 1/1; Undying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("young-wolf"),
        name: "Young Wolf".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Wolf"]),
        oracle_text: "Undying (When this creature dies, if it had no +1/+1 counters on it, return it to the battlefield under its owner's control with a +1/+1 counter on it.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Undying),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}
