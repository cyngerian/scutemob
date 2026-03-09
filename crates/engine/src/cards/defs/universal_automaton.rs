// 65. Universal Automaton — {1}, Artifact Creature — Shapeshifter 1/1; Changeling.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("universal-automaton"),
        name: "Universal Automaton".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Shapeshifter"]),
        oracle_text: "Changeling (This card is every creature type.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Changeling),
        ],
        back_face: None,
    }
}
