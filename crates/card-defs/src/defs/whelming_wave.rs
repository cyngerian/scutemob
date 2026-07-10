// Whelming Wave — {2}{U}{U}, Sorcery
// Return all creatures to their owners' hands except for Krakens, Leviathans, Octopuses,
// and Serpents.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("whelming-wave"),
        name: "Whelming Wave".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return all creatures to their owners' hands except for Krakens, Leviathans, Octopuses, and Serpents.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::BounceAll {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    exclude_subtypes: vec![
                        SubType("Kraken".to_string()),
                        SubType("Leviathan".to_string()),
                        SubType("Octopus".to_string()),
                        SubType("Serpent".to_string()),
                    ],
                    ..Default::default()
                },
                max_toughness_amount: None,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
