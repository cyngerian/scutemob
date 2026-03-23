// Corpsejack Menace — {2}{B}{G}, Creature — Fungus 4/4
// If one or more +1/+1 counters would be put on a creature you control, twice that many
// +1/+1 counters are put on it instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("corpsejack-menace"),
        name: "Corpsejack Menace".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Fungus"]),
        oracle_text: "If one or more +1/+1 counters would be put on a creature you control, twice that many +1/+1 counters are put on it instead.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            // TODO: DSL gap — replacement effect for counter doubling.
            // ReplacementTrigger::CounterPlacement with doubling modifier does not exist.
        ],
        ..Default::default()
    }
}
