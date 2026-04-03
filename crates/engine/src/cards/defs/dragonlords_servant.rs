// Dragonlord's Servant — {1}{R}, Creature — Goblin Shaman 1/3
// Dragon spells you cast cost {1} less to cast.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragonlords-servant"),
        name: "Dragonlord's Servant".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Shaman"]),
        oracle_text: "Dragon spells you cast cost {1} less to cast.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasSubtype(SubType("Dragon".to_string())),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
            colored_mana_reduction: None,
        }],
        ..Default::default()
    }
}
