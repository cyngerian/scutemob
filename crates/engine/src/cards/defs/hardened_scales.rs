// Hardened Scales — {G}, Enchantment
// If one or more +1/+1 counters would be put on a creature you control, that many plus
// one +1/+1 counters are put on it instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hardened-scales"),
        name: "Hardened Scales".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on it instead.".to_string(),
        abilities: vec![
            // TODO: DSL gap — replacement effect for counter placement (+1 additional
            // +1/+1 counter). ReplacementTrigger::CounterPlacement with modifier
            // does not exist.
        ],
        ..Default::default()
    }
}
