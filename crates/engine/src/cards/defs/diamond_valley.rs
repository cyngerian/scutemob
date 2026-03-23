// Diamond Valley — Land
// {T}, Sacrifice a creature: You gain life equal to the sacrificed creature's toughness.
//
// TODO: "gain life equal to the sacrificed creature's toughness" requires
// EffectAmount::SacrificedCreatureToughness (dynamic amount based on sacrificed creature stats).
// This DSL primitive does not exist. Omitted per W5 policy.
// Note: Diamond Valley has no {T}: Add {C} ability — it only produces life via the sacrifice
// outlet. It cannot tap for mana at all.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("diamond-valley"),
        name: "Diamond Valley".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Sacrifice a creature: You gain life equal to the sacrificed creature's toughness.".to_string(),
        abilities: vec![
            // TODO: {T}, Sacrifice a creature: Gain life = sacrificed creature's toughness.
            // Requires EffectAmount::SacrificedCreatureToughness which does not exist in the DSL.
        ],
        ..Default::default()
    }
}
