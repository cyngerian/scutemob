// Goblin Cratermaker — {1}{R} Creature — Goblin Warrior 2/2
// {1}, Sacrifice this creature: Choose one —
// • This creature deals 2 damage to target creature.
// • Destroy target colorless nonland permanent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-cratermaker"),
        name: "Goblin Cratermaker".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "{1}, Sacrifice this creature: Choose one —\n• Goblin Cratermaker deals 2 damage to target creature.\n• Destroy target colorless nonland permanent.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: Modal activated abilities not expressible in current DSL.
            // AbilityDefinition::Activated doesn't support modes field.
            // The sacrifice cost is Cost::Sequence(vec![Cost::Mana(...), Cost::SacrificeSelf])
            // but choosing between two effects requires a modes selection on the Activated ability.
            // DSL gap: Activated abilities lack a modes field.
        ],
        ..Default::default()
    }
}
