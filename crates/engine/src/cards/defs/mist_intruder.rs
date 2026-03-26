// Mist Intruder — {1}{U}, Creature — Eldrazi Drone 1/2; Devoid, Flying, Ingest
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mist-intruder"),
        name: "Mist Intruder".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Eldrazi", "Drone"]),
        oracle_text: "Devoid (This card has no color.)\nFlying\nIngest (Whenever this creature deals combat damage to a player, that player exiles the top card of their library.)"
            .to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Devoid),
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Ingest),
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
