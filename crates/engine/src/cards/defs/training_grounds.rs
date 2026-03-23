// Training Grounds — {U} Enchantment
// Activated abilities of creatures you control cost {2} less to activate.
//   This effect can't reduce the mana in that cost to less than one mana.
//
// DSL gap: cost reduction scoped to "activated abilities of creatures you control" is not in DSL.
//   CostReduction layer only supports SpellsYouCast (spell casting reduction),
//   not activated ability cost reduction.
// W5 policy: cannot faithfully express this — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("training-grounds"),
        name: "Training Grounds".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Activated abilities of creatures you control cost {2} less to activate. This effect can't reduce the mana in that cost to less than one mana.".to_string(),
        abilities: vec![
            // TODO: activated abilities of creatures you control cost {2} less
            //   (no EffectFilter::ActivatedAbilitiesOfCreaturesYouControl + ReduceActivatedAbilityCost)
        ],
        ..Default::default()
    }
}
