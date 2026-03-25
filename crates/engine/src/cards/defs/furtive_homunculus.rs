// Furtive Homunculus — {1}{U}, Creature — Homunculus 2/1; Skulk
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("furtive-homunculus"),
        name: "Furtive Homunculus".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Homunculus"]),
        oracle_text: "Skulk (This creature can't be blocked by creatures with greater power.)"
            .to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Skulk)],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        activated_ability_cost_reductions: vec![],
    }
}
