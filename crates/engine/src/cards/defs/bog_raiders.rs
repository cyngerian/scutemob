// 58. Bog Raiders — {2B}, Creature — Zombie 2/2; Swampwalk.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bog-raiders"),
        name: "Bog Raiders".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Zombie"]),
        oracle_text: "Swampwalk (This creature can't be blocked as long as defending player controls a Swamp.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Landwalk(
                LandwalkType::BasicType(SubType("Swamp".to_string())),
            )),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}
