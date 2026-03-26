// Prosperous Thief — {2}{U}, Creature — Human Ninja 3/2
// Ninjutsu {1}{U}
// Whenever one or more Ninja or Rogue creatures you control deal combat damage to a
// player, create a Treasure token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("prosperous-thief"),
        name: "Prosperous Thief".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Ninja"]),
        oracle_text: "Ninjutsu {1}{U}\nWhenever one or more Ninja or Rogue creatures you control deal combat damage to a player, create a Treasure token.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { generic: 1, blue: 1, ..Default::default() },
            },
            // CR 510.3a / CR 603.2c: "Whenever one or more Ninja or Rogue creatures you control
            // deal combat damage to a player, create a Treasure token." — batch trigger with
            // Ninja OR Rogue subtype filter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer {
                    filter: Some(TargetFilter {
                        has_subtypes: vec![SubType("Ninja".to_string()), SubType("Rogue".to_string())],
                        ..Default::default()
                    }),
                },
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
