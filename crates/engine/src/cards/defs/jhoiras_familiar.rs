// Jhoira's Familiar — {4}, Artifact Creature — Bird 2/2
// "Flying
// Historic spells you cast cost {1} less to cast. (Artifacts, legendaries, and Sagas are historic.)"
//
// Flying is implemented.
//
// TODO: DSL gap — "Historic spells you cast cost {1} less to cast" requires a cost-reduction
// continuous effect filtered by spell type (Artifact, Legendary, or Saga). No such
// cost-reduction layer effect is expressible in the current DSL.
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
        ..Default::default()
    }
}
