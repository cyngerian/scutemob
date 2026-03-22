// Goblin Electromancer — {U}{R}, Creature — Goblin Wizard 2/2
// Instant and sorcery spells you cast cost {1} less to cast.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-electromancer"),
        name: "Goblin Electromancer".to_string(),
        mana_cost: Some(ManaCost { blue: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Wizard"]),
        oracle_text: "Instant and sorcery spells you cast cost {1} less to cast.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::InstantOrSorcery,
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        ..Default::default()
    }
}
