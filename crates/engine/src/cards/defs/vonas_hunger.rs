// Vona's Hunger — {2}{B}, Instant
// Ascend (If you control ten or more permanents, you get the city's blessing for the
// rest of the game.)
// Each opponent sacrifices a creature of their choice. If you have the city's blessing,
// instead each opponent sacrifices half the creatures they control of their choice, rounded up.
//
// Note: The basic "each opponent sacrifices a creature" is expressible via ForEach +
// SacrificePermanents. The Ascend conditional (city's blessing → sacrifice half rounded up)
// requires Condition::HasCitysBlessing and EffectAmount::HalfCreatureCount, neither of which
// is in the DSL. W5: implementing only the non-Ascend portion would produce wrong game state
// when the city's blessing is active. Abilities empty per W5.
// TODO: Needs Condition::HasCitysBlessing + EffectAmount::HalfCreatureCount(rounded_up).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vonas-hunger"),
        name: "Vona's Hunger".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Ascend (If you control ten or more permanents, you get the city's blessing for the rest of the game.)\nEach opponent sacrifices a creature of their choice. If you have the city's blessing, instead each opponent sacrifices half the creatures they control of their choice, rounded up.".to_string(),
        abilities: vec![
            // TODO: Ascend conditional — Condition::HasCitysBlessing + half-creature-count
            // EffectAmount variant not in DSL. Basic sacrifice also deferred to avoid
            // producing incorrect game state when city's blessing is active. W5.
        ],
        ..Default::default()
    }
}
