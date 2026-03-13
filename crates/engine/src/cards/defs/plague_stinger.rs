// Plague Stinger — {1}{B}, Creature — Phyrexian Insect Horror 1/1
// "Flying
// Infect (This creature deals damage to creatures in the form of -1/-1 counters and to players
// in the form of poison counters.)"
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("plague-stinger"),
        name: "Plague Stinger".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Insect", "Horror"]),
        oracle_text: "Flying\nInfect (This creature deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Infect),
        ],
        ..Default::default()
    }
}
