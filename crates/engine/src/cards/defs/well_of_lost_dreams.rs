// Well of Lost Dreams — {4}, Artifact
// Whenever you gain life, you may pay {X}, where X is less than or equal to the amount of
//   life you gained. If you do, draw X cards.
//
// TODO: "Whenever you gain life, pay {X} up to the amount gained, draw X cards" —
//   no TriggerCondition::WhenYouGainLife exists, and the pay-variable-X-draw-X pattern
//   requires tracking the life-gained amount as a cap for X. No EffectAmount::LifeGainedThisEvent
//   or Cost::PayUpToX(amount) variant exists in DSL. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("well-of-lost-dreams"),
        name: "Well of Lost Dreams".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever you gain life, you may pay {X}, where X is less than or equal to the amount of life you gained. If you do, draw X cards.".to_string(),
        abilities: vec![
            // TODO: TriggerCondition::WhenYouGainLife not in DSL.
            //   Also requires EffectAmount::LifeGainedThisEvent to cap the X cost,
            //   and a Cost::PayUpToX variant. Multiple DSL gaps. Omitted per W5 policy.
        ],
        ..Default::default()
    }
}
