// Conclave Mentor — {G}{W}, Creature — Centaur Cleric 2/2
// If one or more +1/+1 counters would be put on a creature you control, that many plus
// one +1/+1 counters are put on that creature instead.
// When this creature dies, you gain life equal to its power.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("conclave-mentor"),
        name: "Conclave Mentor".to_string(),
        mana_cost: Some(ManaCost { green: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Centaur", "Cleric"]),
        oracle_text: "If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on that creature instead.\nWhen Conclave Mentor dies, you gain life equal to its power.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: DSL gap — replacement effect for +1 additional +1/+1 counter.
            // TODO: DSL gap — "When this creature dies, gain life equal to its power."
            // WhenThisDies trigger + EffectAmount::SourcePower.
        ],
        ..Default::default()
    }
}
