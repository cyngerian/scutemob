// 74. Wayward Swordtooth — {2}{G}, Creature — Dinosaur 5/5;
// Ascend. You may play an additional land on each of your turns.
// Wayward Swordtooth can't attack or block unless you have the city's blessing.
// (Additional land play and attack/block restriction noted in oracle_text;
// Ascend keyword fully modeled.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wayward-swordtooth"),
        name: "Wayward Swordtooth".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Dinosaur"]),
        oracle_text: "Ascend (If you control ten or more permanents, you get the city's blessing for the rest of the game.)\nYou may play an additional land on each of your turns.\nWayward Swordtooth can't attack or block unless you have the city's blessing.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ascend),
        ],
        color_indicator: None,
        back_face: None,
    }
}
