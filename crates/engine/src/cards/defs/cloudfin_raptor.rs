// 72. Cloudfin Raptor — {U}, Creature — Bird Mutant 0/1;
// Flying. Evolve (CR 702.100a).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cloudfin-raptor"),
        name: "Cloudfin Raptor".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Bird", "Mutant"]),
        oracle_text: "Flying\nEvolve (Whenever a creature with greater power and/or toughness enters the battlefield under your control, put a +1/+1 counter on this creature.)".to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Evolve),
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
    }
}
