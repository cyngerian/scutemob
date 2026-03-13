// Blade Historian — {R/W}{R/W}{R/W}{R/W}, Creature — Human Cleric 2/3
// "Attacking creatures you control have double strike."
//
// Hybrid cost {R/W}{R/W}{R/W}{R/W} approximated as {R}{R}{W}{W}; ManaCost lacks hybrid support.
//
// TODO: DSL gap — "Attacking creatures you control have double strike" is a conditional
// continuous effect (Layer 6) that applies only during the attack. No EffectFilter or
// TriggerCondition supports granting keywords only to currently-attacking creatures you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blade-historian"),
        name: "Blade Historian".to_string(),
        mana_cost: Some(ManaCost { red: 2, white: 2, ..Default::default() }),
        types: creature_types(&["Human", "Cleric"]),
        oracle_text: "Attacking creatures you control have double strike.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![],
        ..Default::default()
    }
}
