// Dragonspeaker Shaman — {1}{R}{R}, Creature — Human Barbarian Shaman 2/2
// Dragon spells you cast cost {2} less to cast.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragonspeaker-shaman"),
        name: "Dragonspeaker Shaman".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
        types: creature_types(&["Human", "Barbarian", "Shaman"]),
        oracle_text: "Dragon spells you cast cost {2} less to cast.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -2,
            filter: SpellCostFilter::HasSubtype(SubType("Dragon".to_string())),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        ..Default::default()
    }
}
