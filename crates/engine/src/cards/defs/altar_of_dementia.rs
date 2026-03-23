// Altar of Dementia — {2}, Artifact
// "Sacrifice a creature: Target player mills cards equal to the sacrificed creature's power."
// TODO: DSL gap — EffectAmount::PowerOfSacrificedCreature does not exist. The mill amount
// depends on the power of the sacrificed creature at the time of sacrifice (LKI). Cannot
// faithfully express this without a new EffectAmount variant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("altar-of-dementia"),
        name: "Altar of Dementia".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Sacrifice a creature: Target player mills cards equal to the sacrificed creature's power.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
