// Emerald Medallion — {2}, Artifact
// Green spells you cast cost {1} less to cast.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("emerald-medallion"),
        name: "Emerald Medallion".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Green spells you cast cost {1} less to cast.".to_string(),
        abilities: vec![],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasColor(Color::Green),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
            colored_mana_reduction: None,
        }],
        ..Default::default()
    }
}
