// Dragonspeaker Shaman — {1}{R}{R}, Creature — Human Barbarian Shaman 2/2
// Dragon spells you cast cost {2} less to cast.
// The card's only rules text is a cost reduction. That is not an `AbilityDefinition` --
// it lives in the `spell_cost_modifiers` field, which `apply_spell_cost_modifiers`
// applies on the real cast path (battlefield + command-zone-with-eminence only).
// `abilities: vec![]` is correct here and is NOT an unimplemented card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragonspeaker-shaman"),
        name: "Dragonspeaker Shaman".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 2,
            ..Default::default()
        }),
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
            colored_mana_reduction: None,
        }],
        ..Default::default()
    }
}
