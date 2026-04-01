// Mesmeric Orb — {2}, Artifact
// Whenever a permanent becomes untapped, that permanent's controller mills a card.
//
// TODO: "Whenever a permanent becomes untapped" — no TriggerCondition::WheneverPermanentUntaps
//   exists in DSL. This fires for every permanent (opponent's and your own) on untap steps and
//   from other effects. No WhenUntaps trigger condition exists. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mesmeric-orb"),
        name: "Mesmeric Orb".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever a permanent becomes untapped, that permanent's controller mills a card.".to_string(),
        abilities: vec![
            // TODO: TriggerCondition::WheneverPermanentUntaps not in DSL.
            //   This would be a global trigger firing on every untap event.
            //   DSL gap: need WheneverAnyPermanentUntaps + Effect::Mill targeting the
            //   untapped permanent's controller. Omitted per W5 policy.
        ],
        ..Default::default()
    }
}
