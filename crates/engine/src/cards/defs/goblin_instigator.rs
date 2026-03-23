// Goblin Instigator — {1}{R}, Creature — Goblin Rogue 1/1
// When this creature enters, create a 1/1 red Goblin creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-instigator"),
        name: "Goblin Instigator".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Rogue"]),
        oracle_text: "When this creature enters, create a 1/1 red Goblin creature token.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        supertypes: im::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
