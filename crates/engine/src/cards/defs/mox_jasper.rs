// Mox Jasper — {0}, Legendary Artifact
// {T}: Add one mana of any color. Activate only if you control a Dragon.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mox-jasper"),
        name: "Mox Jasper".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "{T}: Add one mana of any color. Activate only if you control a Dragon.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::YouControlPermanent(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    has_subtype: Some(SubType("Dragon".to_string())),
                    ..Default::default()
                })),

                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
