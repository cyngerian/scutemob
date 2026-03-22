// Sapphire Medallion — {2}, Artifact
// Blue spells you cast cost {1} less to cast.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sapphire-medallion"),
        name: "Sapphire Medallion".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Blue spells you cast cost {1} less to cast.".to_string(),
        abilities: vec![],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasColor(Color::Blue),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        ..Default::default()
    }
}
