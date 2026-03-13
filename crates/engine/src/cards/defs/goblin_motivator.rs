// Goblin Motivator — {R}, Creature — Goblin Warrior 1/1
// {T}: Target creature gains haste until end of turn.
// TODO: Activated ability has a target (target creature). DSL gap: AbilityDefinition::Activated
// has no TargetRequirement field. Deferred (activated_ability_targets gap).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-motivator"),
        name: "Goblin Motivator".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "{T}: Target creature gains haste until end of turn. (It can attack and {T} this turn.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // TODO: {T}: Target creature gains haste until end of turn.
            // DSL gap: AbilityDefinition::Activated has no TargetRequirement field.
            // Deferred — same gap as Forerunner of Slaughter.
        ],
        ..Default::default()
    }
}
