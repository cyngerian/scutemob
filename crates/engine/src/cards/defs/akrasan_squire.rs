// 61. Akrasan Squire — {W}, Creature — Human Soldier 1/1; Exalted (CR 702.83).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("akrasan-squire"),
        name: "Akrasan Squire".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Soldier"]),
        oracle_text: "Exalted (Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Exalted)],
        back_face: None,
    }
}
