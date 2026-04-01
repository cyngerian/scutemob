// Goblin Lackey — {R}, Creature — Goblin 1/1
// Whenever this creature deals damage to a player, you may put a Goblin
// permanent card from your hand onto the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-lackey"),
        name: "Goblin Lackey".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Whenever this creature deals damage to a player, you may put a Goblin permanent card from your hand onto the battlefield.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                // TODO: "put a Goblin from hand onto battlefield" — needs MoveZone from
                // hand with subtype filter. Using Nothing stub.
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
