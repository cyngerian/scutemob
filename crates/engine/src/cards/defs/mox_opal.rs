// Mox Opal — {0}, Legendary Artifact
// Metalcraft — {T}: Add one mana of any color. Activate only if you control
// three or more artifacts.
//
// TODO: Metalcraft activation condition — "control three or more artifacts" requires
//   a count-based Condition (e.g. YouControlNOrMorePermanents { count: 3, filter })
//   which doesn't exist in the DSL yet. Implementing the tap ability without the condition.
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
                // TODO: activation_condition: Some(Condition::YouControlNOrMorePermanents { count: 3,
                //   filter: TargetFilter { has_card_type: Some(CardType::Artifact), ..Default::default() } })
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
