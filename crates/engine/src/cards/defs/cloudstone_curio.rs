// Cloudstone Curio — {3}, Artifact
// Whenever a nonartifact permanent you control enters, you may return another permanent you
//   control that shares a permanent type with it to its owner's hand.
//
// TODO: "Whenever a nonartifact permanent you control enters" — no TriggerCondition that fires
//   on any nonartifact permanent entering under your control. WheneverCreatureEntersBattlefield
//   exists but is creature-only. A general WheneverPermanentEntersUnderYourControl with a
//   non_artifact filter does not exist.
// TODO: "return another permanent that shares a permanent type with it" — shared-type check
//   between the entered permanent and the chosen return target is a DSL gap. No filter for
//   "shares a card type with [source]" exists.
//   Both gaps are blocking. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cloudstone-curio"),
        name: "Cloudstone Curio".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever a nonartifact permanent you control enters, you may return another permanent you control that shares a permanent type with it to its owner's hand.".to_string(),
        abilities: vec![
            // TODO: No WheneverNonartifactPermanentEntersUnderYourControl trigger condition.
            //   Also requires "shares a permanent type with" filter for the return target.
            //   DSL gap: both trigger condition and shared-type filter missing. Omitted per W5 policy.
        ],
        ..Default::default()
    }
}
