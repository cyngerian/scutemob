// Exsanguinate — {X}{B}{B}, Sorcery
// Each opponent loses X life. You gain life equal to the life lost this way.
//
// CR 702.101a: DrainLife is exactly this pattern — each opponent loses `amount`, and the
// controller gains life equal to the total actually lost (not amount * opponent count).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("exsanguinate"),
        name: "Exsanguinate".to_string(),
        mana_cost: Some(ManaCost {
            black: 2,
            x_count: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each opponent loses X life. You gain life equal to the life lost this way."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrainLife {
                amount: EffectAmount::XValue,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
