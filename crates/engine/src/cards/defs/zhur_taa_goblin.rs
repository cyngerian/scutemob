// 78. Zhur-Taa Goblin — {R}{G}, Creature — Goblin Berserker 2/2; Riot.
// CR 702.136a: As this enters, choose +1/+1 counter OR haste.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zhur-taa-goblin"),
        name: "Zhur-Taa Goblin".to_string(),
        mana_cost: Some(ManaCost { red: 1, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Goblin", "Berserker"]),
        oracle_text: "Riot (As this enters, choose to have it enter with a +1/+1 counter or gain haste.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Riot),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
    }
}
