// Leonin Warleader — {2}{W}{W}, Creature — Cat Soldier 4/4
// Whenever this creature attacks, create two 1/1 white Cat creature tokens with lifelink
// that are tapped and attacking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("leonin-warleader"),
        name: "Leonin Warleader".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 2, ..Default::default() }),
        types: creature_types(&["Cat", "Soldier"]),
        oracle_text: "Whenever this creature attacks, create two 1/1 white Cat creature tokens with lifelink that are tapped and attacking.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Cat".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Cat".to_string())].into_iter().collect(),
                        colors: [Color::White].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 2,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Lifelink].into_iter().collect(),
                        tapped: true,
                        enters_attacking: true,
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
