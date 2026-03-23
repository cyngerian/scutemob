// Rings of Brighthearth — {3} Artifact
// Whenever you activate an ability, if it isn't a mana ability, you may pay {2}.
//   If you do, copy that ability. You may choose new targets for the copy.
//
// DSL gap: "copy that ability" in response to activation is a triggered copy-ability effect.
//   No TriggerCondition::WheneverYouActivateNonManaAbility + Effect::CopyAbilityOnStack in DSL.
// W5 policy: cannot faithfully express this — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rings-of-brighthearth"),
        name: "Rings of Brighthearth".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever you activate an ability, if it isn't a mana ability, you may pay {2}. If you do, copy that ability. You may choose new targets for the copy.".to_string(),
        abilities: vec![
            // TODO: whenever you activate a non-mana ability, may pay {2} to copy it
            //   (no TriggerCondition::WheneverYouActivateNonManaAbility + Effect::CopyAbilityOnStack)
        ],
        ..Default::default()
    }
}
