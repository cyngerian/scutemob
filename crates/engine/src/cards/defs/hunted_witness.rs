// Hunted Witness — {W}, Creature — Human 1/1
// When this creature dies, create a 1/1 white Soldier creature token with lifelink.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hunted-witness"),
        name: "Hunted Witness".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: creature_types(&["Human"]),
        oracle_text: "When this creature dies, create a 1/1 white Soldier creature token with lifelink.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Soldier".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Soldier".to_string())].into_iter().collect(),
                        colors: [Color::White].into_iter().collect(),
                        supertypes: im::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        keywords: [KeywordAbility::Lifelink].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
