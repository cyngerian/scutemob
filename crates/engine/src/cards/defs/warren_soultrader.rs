// Warren Soultrader — {2}{B}, Creature — Zombie Goblin Wizard 3/3
// Pay 1 life, Sacrifice another creature: Create a Treasure token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("warren-soultrader"),
        name: "Warren Soultrader".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Zombie", "Goblin", "Wizard"]),
        oracle_text: "Pay 1 life, Sacrifice another creature: Create a Treasure token.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::PayLife(1),
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::CreateToken { spec: treasure_token_spec(1) },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
