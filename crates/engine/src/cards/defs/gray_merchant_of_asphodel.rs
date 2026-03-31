// Gray Merchant of Asphodel — {3}{B}{B}, Creature — Zombie
// When this creature enters, each opponent loses X life where X is your devotion to black.
// You gain life equal to the life lost this way.
//
// Note: DrainLife with EffectAmount::DevotionTo(Color::Black) captures the drain pattern.
// The devotion reminder text is part of the oracle but does not change behavior.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gray-merchant-of-asphodel"),
        name: "Gray Merchant of Asphodel".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: creature_types(&["Zombie"]),
        oracle_text: "When this creature enters, each opponent loses X life, where X is your devotion to black. You gain life equal to the life lost this way. (Each {B} in the mana costs of permanents you control counts toward your devotion to black.)".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // CR 603.3: ETB trigger — each opponent loses X life, you gain that much.
            // DrainLife with DevotionTo(Black) handles the lose+gain pattern.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrainLife {
                    amount: EffectAmount::DevotionTo(Color::Black),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
