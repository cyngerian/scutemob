// Berserk — {G}, Instant
// "Cast this spell only before the combat damage step.
// Target creature gains trample and gets +X/+0 until end of turn, where X is its power.
// At the beginning of the next end step, destroy that creature if it attacked this turn."
//
// TODO: DSL gap — this card requires:
// 1. Timing restriction "only before the combat damage step" (no DSL support).
// 2. EffectAmount::TargetPower (power of the target creature at resolution) — not in DSL.
// 3. A deferred conditional destroy at the next end step ("if it attacked this turn")
//    — no DelayedTrigger or conditional end-step destroy in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("berserk"),
        name: "Berserk".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Cast this spell only before the combat damage step.\nTarget creature gains trample and gets +X/+0 until end of turn, where X is its power. At the beginning of the next end step, destroy that creature if it attacked this turn.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
