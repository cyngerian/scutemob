// Vexing Shusher — {R/G}{R/G}, Creature — Goblin Shaman 2/2
// This spell can't be countered.
// {R/G}: Target spell can't be countered.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vexing-shusher"),
        name: "Vexing Shusher".to_string(),
        mana_cost: Some(ManaCost {
            hybrid: vec![
                HybridMana::ColorColor(ManaColor::Red, ManaColor::Green),
                HybridMana::ColorColor(ManaColor::Red, ManaColor::Green),
            ],
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Shaman"]),
        oracle_text: "This spell can't be countered.\n{R/G}: Target spell can't be countered.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "This spell can't be countered" on creature — no cant_be_countered
            // field on creature CardDefinition.
            // TODO: "{R/G}: Target spell can't be countered" — needs an activated ability
            // that applies "can't be countered" to a target spell on the stack. No
            // Effect::MakeSpellUncounterable variant exists.
        ],
        ..Default::default()
    }
}
