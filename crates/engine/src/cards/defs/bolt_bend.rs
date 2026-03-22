// Bolt Bend — {3}{R}, Instant
// This spell costs {3} less to cast if you control a creature with power 4 or greater.
// Change the target of target spell or ability with a single target.
//
// TODO: Effect::ChangeTarget — "change the target of target spell or ability with a single
//   target" is not expressible in the DSL. Cost reduction is fully implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bolt-bend"),
        name: "Bolt Bend".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "This spell costs {3} less to cast if you control a creature with power 4 or greater.\nChange the target of target spell or ability with a single target.".to_string(),
        abilities: vec![],
        self_cost_reduction: Some(SelfCostReduction::ConditionalPowerThreshold {
            threshold: 4,
            reduction: 3,
        }),
        ..Default::default()
    }
}
