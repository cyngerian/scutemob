// 60. Siege Wurm — {5GG}, Creature — Wurm 5/5; Convoke. Trample.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("siege-wurm"),
        name: "Siege Wurm".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 2, ..Default::default() }),
        types: creature_types(&["Wurm"]),
        oracle_text: "Convoke (Your creatures can help cast this spell. Each creature you tap while casting this spell pays for {1} or one mana of that creature's color.)\nTrample".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Convoke),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
        ],
        back_face: None,
    }
}
