// Gudul Lurker — {U}, Creature — Salamander 1/1
// This creature can't be blocked.
// Megamorph {U}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gudul-lurker"),
        name: "Gudul Lurker".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Salamander"]),
        oracle_text: "Gudul Lurker can't be blocked.\nMegamorph {U}".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CantBeBlocked),
            AbilityDefinition::Keyword(KeywordAbility::Megamorph),
            AbilityDefinition::Megamorph { cost: ManaCost { blue: 1, ..Default::default() } },
        ],
        ..Default::default()
    }
}
