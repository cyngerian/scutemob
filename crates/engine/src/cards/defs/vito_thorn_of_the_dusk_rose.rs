// Vito, Thorn of the Dusk Rose — {2}{B}, Legendary Creature — Vampire Cleric 1/3
// Whenever you gain life, target opponent loses that much life.
// {3}{B}{B}: Creatures you control gain lifelink until end of turn.
//
// TODO: DSL gaps — two abilities omitted:
// 1. "Whenever you gain life, target opponent loses that much life." — targeted trigger
//    requiring tracking the amount gained and applying it as life loss to a chosen opponent.
//    No TriggerCondition for WhenYouGainLife; no way to track "that much life" as EffectAmount.
// 2. "{3}{B}{B}: Creatures you control gain lifelink until end of turn." — activated ability
//    that applies a continuous effect to all creatures you control. The Activated ability DSL
//    has no targets field, and ApplyContinuousEffect to EffectTarget::AllCreaturesYouControl
//    is not currently supported.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vito-thorn-of-the-dusk-rose"),
        name: "Vito, Thorn of the Dusk Rose".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Cleric"],
        ),
        oracle_text: "Whenever you gain life, target opponent loses that much life.\n{3}{B}{B}: Creatures you control gain lifelink until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![],
        ..Default::default()
    }
}
