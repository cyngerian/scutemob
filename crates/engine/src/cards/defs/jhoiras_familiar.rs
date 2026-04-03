// Jhoira's Familiar — {4}, Artifact Creature — Bird 2/2
// Flying
// Historic spells you cast cost {1} less to cast. (Artifacts, legendaries, and Sagas are historic.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("jhoiras-familiar"),
        name: "Jhoira's Familiar".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Bird"]),
        oracle_text: "Flying\nHistoric spells you cast cost {1} less to cast. (Artifacts, legendaries, and Sagas are historic.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::Historic,
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
            colored_mana_reduction: None,
        }],
        ..Default::default()
    }
}
