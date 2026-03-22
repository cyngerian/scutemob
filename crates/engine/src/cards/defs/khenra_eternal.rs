// Khenra Eternal — {1}{B}, Creature — Zombie Jackal Warrior 2/2; Afflict 1
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("khenra-eternal"),
        name: "Khenra Eternal".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Zombie", "Jackal", "Warrior"]),
        oracle_text: "Afflict 1 (Whenever this creature becomes blocked, defending player loses 1 life.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Afflict(1)),
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
