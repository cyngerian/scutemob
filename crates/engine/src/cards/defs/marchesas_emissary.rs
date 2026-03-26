// 77. Marchesa's Emissary — {3}{U}, Creature — Human Rogue 2/2; Hexproof, Dethrone.
// Dethrone: CR 702.105 — whenever this attacks the player with most life (or tied),
// put a +1/+1 counter on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("marchesas-emissary"),
        name: "Marchesa's Emissary".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Human", "Rogue"]),
        oracle_text: "Hexproof\nDethrone".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Hexproof),
            AbilityDefinition::Keyword(KeywordAbility::Dethrone),
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
