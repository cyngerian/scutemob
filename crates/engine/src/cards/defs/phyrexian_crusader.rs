// Phyrexian Crusader — {1}{B}{B}, Creature — Phyrexian Zombie Knight 2/2
// First strike, protection from red and from white
// Infect
use crate::cards::helpers::*;
use crate::state::types::ProtectionQuality;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phyrexian-crusader"),
        name: "Phyrexian Crusader".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Zombie", "Knight"]),
        oracle_text: "First strike, protection from red and from white\nInfect (This creature deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromColor(Color::Red),
            )),
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromColor(Color::White),
            )),
            AbilityDefinition::Keyword(KeywordAbility::Infect),
        ],
        ..Default::default()
    }
}
