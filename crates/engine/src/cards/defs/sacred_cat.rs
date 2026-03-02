// Sacred Cat — {W}, Creature — Cat 1/1; Lifelink; Embalm {W}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sacred-cat"),
        name: "Sacred Cat".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: creature_types(&["Cat"]),
        oracle_text: "Lifelink\nEmbalm {W} ({W}, Exile this card from your graveyard: Create a token that's a copy of it, except it's a white Zombie Cat with no mana cost. Embalm only as a sorcery.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            AbilityDefinition::Keyword(KeywordAbility::Embalm),
            AbilityDefinition::Embalm { cost: ManaCost { white: 1, ..Default::default() } },
        ],
    }
}
