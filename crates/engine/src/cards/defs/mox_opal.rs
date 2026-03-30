// Mox Opal — {0}, Legendary Artifact
// Metalcraft — {T}: Add one mana of any color. Activate only if you control
// three or more artifacts.
//
// CR 702.45a (Metalcraft ability word): The activation condition checks that you control
// 3+ artifacts. Using Condition::YouControlNOrMoreWithFilter with count: 3 and
// has_card_type: Some(CardType::Artifact).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mox-opal"),
        name: "Mox Opal".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "Metalcraft — {T}: Add one mana of any color. Activate only if you control three or more artifacts.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                // CR 702.45a: Metalcraft — only active when you control 3+ artifacts.
                activation_condition: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 3,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Artifact),
                        ..Default::default()
                    },
                }),

                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
