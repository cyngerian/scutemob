// Awakening Zone — {2}{G}, Enchantment
// At the beginning of your upkeep, you may create a 0/1 colorless Eldrazi Spawn creature
// token. It has "Sacrifice this token: Add {C}."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("awakening-zone"),
        name: "Awakening Zone".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Enchantment], &[]),
        oracle_text: "At the beginning of your upkeep, you may create a 0/1 colorless Eldrazi Spawn creature token. It has \"Sacrifice this token: Add {C}.\"".to_string(),
        abilities: vec![
            // CR 603.2: "At the beginning of your upkeep, you may create a 0/1 colorless
            // Eldrazi Spawn creature token with 'Sacrifice this creature: Add {C}.'"
            // Note: "you may" optional not in DSL — always creates (bot always opts in).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Eldrazi Spawn".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Eldrazi".to_string()), SubType("Spawn".to_string())].into_iter().collect(),
                        power: 0,
                        toughness: 1,
                        count: 1,
                        mana_color: Some(ManaColor::Colorless),
                        mana_abilities: vec![ManaAbility {
                            sacrifice_self: true,
                            any_color: false,
                            requires_tap: false,
                            produces: im::ordmap! { ManaColor::Colorless => 1 },
                            damage_to_controller: 0,
                        }],
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
