// Blighted Agent — {1}{U}, Creature — Phyrexian Human Rogue 1/1
// Infect
// Blighted Agent can't be blocked.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blighted-agent"),
        name: "Blighted Agent".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Human", "Rogue"]),
        oracle_text: "Infect (This creature deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)\nBlighted Agent can't be blocked.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Infect),
            AbilityDefinition::Keyword(KeywordAbility::CantBeBlocked),
        ],
        ..Default::default()
    }
}
