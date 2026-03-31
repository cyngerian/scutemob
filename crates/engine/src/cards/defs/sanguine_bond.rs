// Sanguine Bond — {3}{B}{B}, Enchantment
// Whenever you gain life, target opponent loses that much life.
//
// WheneverYouGainLife trigger exists. The amount is "that much life" (the amount gained)
// which needs EffectAmount::TriggeringAmount — not in the DSL.
// Fixed(1) would produce wrong game state for larger life gains.
// W5: abilities empty.
// TODO: EffectAmount::TriggeringAmount (life gained this trigger) not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sanguine-bond"),
        name: "Sanguine Bond".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever you gain life, target opponent loses that much life.".to_string(),
        abilities: vec![
            // TODO: EffectAmount::TriggeringAmount not in DSL.
            // "Whenever you gain life, target opponent loses that much life" — the amount
            // depends on how much life was gained (the triggering event's amount).
            // WheneverYouGainLife trigger exists but there's no way to forward the
            // triggering amount to LoseLife. W5: omitted.
        ],
        ..Default::default()
    }
}
