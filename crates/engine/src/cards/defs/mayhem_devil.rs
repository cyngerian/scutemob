// Mayhem Devil — {1}{B}{R} Creature — Devil 3/3
// Whenever a player sacrifices a permanent, this creature deals 1 damage to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mayhem-devil"),
        name: "Mayhem Devil".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Devil"]),
        oracle_text: "Whenever a player sacrifices a permanent, Mayhem Devil deals 1 damage to any target.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: TriggerCondition::WheneverAPlayerSacrifices not in DSL.
            // This requires a sacrifice-event trigger that fires for any player's sacrifice.
        ],
        ..Default::default()
    }
}
