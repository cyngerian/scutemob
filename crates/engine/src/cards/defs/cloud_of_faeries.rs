// Cloud of Faeries — {1}{U}, Creature — Faerie 1/1
// Flying
// When this creature enters, untap up to two lands.
// Cycling {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cloud-of-faeries"),
        name: "Cloud of Faeries".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Faerie"]),
        oracle_text: "Flying\nWhen this creature enters, untap up to two lands.\nCycling {2}".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Untap up to two lands" — multi-target untap with land filter not expressible.
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { generic: 2, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
