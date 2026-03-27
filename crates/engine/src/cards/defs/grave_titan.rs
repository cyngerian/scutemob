// Grave Titan — {4}{B}{B}, Creature — Giant 6/6
// Deathtouch
// Whenever this creature enters or attacks, create two 2/2 black Zombie creature tokens.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grave-titan"),
        name: "Grave Titan".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 2, ..Default::default() }),
        types: creature_types(&["Giant"]),
        oracle_text: "Deathtouch\nWhenever this creature enters or attacks, create two 2/2 black Zombie creature tokens.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            // ETB trigger: "Whenever this creature enters ... create two 2/2 black Zombie tokens"
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Zombie".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Zombie".to_string())].into_iter().collect(),
                        colors: [Color::Black].into_iter().collect(),
                        power: 2,
                        toughness: 2,
                        count: 2,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
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
            },
            // Attack trigger: same token creation
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Zombie".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Zombie".to_string())].into_iter().collect(),
                        colors: [Color::Black].into_iter().collect(),
                        power: 2,
                        toughness: 2,
                        count: 2,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
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
            },
        ],
        ..Default::default()
    }
}
