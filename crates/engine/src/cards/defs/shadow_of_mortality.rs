// Shadow of Mortality — {13}{B}{B}, Creature — Avatar 7/7
// If your life total is less than your starting life total, this spell costs {X} less
// to cast, where X is the difference.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shadow-of-mortality"),
        name: "Shadow of Mortality".to_string(),
        mana_cost: Some(ManaCost { generic: 13, black: 2, ..Default::default() }),
        types: creature_types(&["Avatar"]),
        oracle_text: "If your life total is less than your starting life total, this spell costs {X} less to cast, where X is the difference.".to_string(),
        power: Some(7),
        toughness: Some(7),
        abilities: vec![],
        self_cost_reduction: Some(SelfCostReduction::LifeLostFromStarting),
        ..Default::default()
    }
}
