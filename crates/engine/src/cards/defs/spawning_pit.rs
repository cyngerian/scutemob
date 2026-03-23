// Spawning Pit — {2}, Artifact
// "Sacrifice a creature: Put a charge counter on this artifact."
// "{1}, Remove two charge counters from this artifact: Create a 2/2 colorless Spawn artifact creature token."
// TODO: DSL gap — two issues:
// 1. "Put a charge counter on this artifact" requires Effect::AddCounters targeting Source,
//    but CounterType::Charge may not exist and the trigger is an activated ability not a trigger.
// 2. "Remove two charge counters" as a cost (Cost::RemoveCounters) does not exist in the DSL.
// 3. Creating a typed creature token (2/2 colorless Spawn artifact creature) requires a
//    token spec not currently available. Cannot faithfully express this card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spawning-pit"),
        name: "Spawning Pit".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Sacrifice a creature: Put a charge counter on this artifact.\n{1}, Remove two charge counters from this artifact: Create a 2/2 colorless Spawn artifact creature token.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
