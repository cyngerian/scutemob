// Jet Medallion — {2}, Artifact
// Black spells you cast cost {1} less to cast.
// The card's only rules text is a cost reduction. That is not an `AbilityDefinition` --
// it lives in the `spell_cost_modifiers` field, which `apply_spell_cost_modifiers`
// applies on the real cast path (battlefield + command-zone-with-eminence only).
// `abilities: vec![]` is correct here and is NOT an unimplemented card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("jet-medallion"),
        name: "Jet Medallion".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Black spells you cast cost {1} less to cast.".to_string(),
        abilities: vec![],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasColor(Color::Black),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
            colored_mana_reduction: None,
        }],
        ..Default::default()
    }
}
