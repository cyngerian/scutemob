// Dockside Extortionist — {1}{R}, Creature — Goblin Pirate 1/2
// When this creature enters, create X Treasure tokens, where X is the number of artifacts
// and enchantments your opponents control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dockside-extortionist"),
        name: "Dockside Extortionist".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Pirate"]),
        oracle_text: "When this creature enters, create X Treasure tokens, where X is the number of artifacts and enchantments your opponents control.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // TODO: "X = artifacts + enchantments opponents control" — count-based
            //   EffectAmount not in DSL. Using fixed 3 as approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken { spec: treasure_token_spec(3) },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
