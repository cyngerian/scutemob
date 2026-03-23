// Heartstone — {3} Artifact
// Activated abilities of creatures cost {1} less to activate.
//   This effect can't reduce the mana in that cost to less than one mana.
//
// DSL gap: cost reduction scoped to "activated abilities of creatures" is not in DSL.
//   CostReduction layer only supports SpellsYouCast filter (spell casting reduction),
//   not activated ability cost reduction. No LayerModification::ReduceActivatedAbilityCost.
// W5 policy: cannot faithfully express this — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("heartstone"),
        name: "Heartstone".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Activated abilities of creatures cost {1} less to activate. This effect can't reduce the mana in that cost to less than one mana.".to_string(),
        abilities: vec![
            // TODO: activated abilities of creatures cost {1} less (applies globally, not just controller)
            //   (no EffectFilter::ActivatedAbilitiesOfCreatures + ReduceActivatedAbilityCost in DSL)
        ],
        ..Default::default()
    }
}
