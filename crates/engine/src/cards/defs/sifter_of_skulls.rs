// Sifter of Skulls — {3}{B}, Creature — Eldrazi 4/3
// Devoid. Whenever another nontoken creature you control dies, create a 1/1 colorless
// Eldrazi Scion creature token with "Sacrifice this token: Add {C}."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sifter-of-skulls"),
        name: "Sifter of Skulls".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Eldrazi"]),
        oracle_text: "Devoid (This card has no color.)\nWhenever another nontoken creature you control dies, create a 1/1 colorless Eldrazi Scion creature token. It has \"Sacrifice this token: Add {C}.\" ({C} represents colorless mana.)".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Devoid),
            // CR 603.10a: "Whenever another nontoken creature you control dies, create
            // a 1/1 colorless Eldrazi Scion token."
            // PB-23: controller_you + exclude_self + nontoken_only filters via DeathTriggerFilter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: true,
                    nontoken_only: true,
                                filter: None,
            },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Eldrazi Scion".to_string(),
                        power: 1,
                        toughness: 1,
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Eldrazi".to_string()), SubType("Scion".to_string())]
                            .into_iter()
                            .collect(),
                        count: 1,
                        mana_abilities: vec![ManaAbility {
                            sacrifice_self: true,
                            any_color: true,
                            requires_tap: false,
                            produces: im::OrdMap::new(),
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
