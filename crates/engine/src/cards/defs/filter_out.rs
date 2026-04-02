// Filter Out — {1}{U}{U}, Instant
// Return all noncreature, nonland permanents to their owners' hands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("filter-out"),
        name: "Filter Out".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Return all noncreature, nonland permanents to their owners' hands.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::BounceAll {
                filter: TargetFilter {
                    non_creature: true,
                    non_land: true,
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
