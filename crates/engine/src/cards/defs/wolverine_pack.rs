// Wolverine Pack — {2}{G}{G}, Creature — Wolverine 2/4; Rampage 2
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wolverine-pack"),
        name: "Wolverine Pack".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: creature_types(&["Wolverine"]),
        oracle_text: "Rampage 2 (Whenever this creature becomes blocked, it gets +2/+2 until end of turn for each creature blocking it beyond the first.)".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Rampage(2)),
        ],
    }
}
