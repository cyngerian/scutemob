// Vanquish the Horde — {6}{W}{W} Sorcery
// This spell costs {1} less to cast for each creature on the battlefield.
// Destroy all creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vanquish-the-horde"),
        name: "Vanquish the Horde".to_string(),
        mana_cost: Some(ManaCost { generic: 6, white: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text:
            "This spell costs {1} less to cast for each creature on the battlefield.\nDestroy all creatures."
                .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.8: Destroy all creatures.
            effect: Effect::DestroyAll {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                },
                cant_be_regenerated: false,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        // This spell costs {1} less for each creature on the battlefield (all controllers).
        self_cost_reduction: Some(SelfCostReduction::PerPermanent {
            per: 1,
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
            controller: PlayerTarget::EachPlayer,
        }),
        ..Default::default()
    }
}
