// Ornithopter — {0}, Artifact Creature — Thopter 0/2; Flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ornithopter"),
        name: "Ornithopter".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Thopter"]),
        oracle_text: "Flying".to_string(),
        power: Some(0),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        ..Default::default()
    }
}
