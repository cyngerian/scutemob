// Turn // Burn — Until end of turn, target creature loses all abilities and becomes a r
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("turn"),
        name: "Turn // Burn".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Until end of turn, target creature loses all abilities and becomes a red Weird with base power and toughness 0/1.\nFuse (You may cast one or both halves of this card from your hand.)".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
