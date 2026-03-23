// Omnath, Locus of Creation — {R}{G}{W}{U}, Legendary Creature — Elemental 4/4
// When Omnath enters, draw a card.
// Landfall — Whenever a land you control enters, you gain 4 life if this is the first
// time this ability has resolved this turn. If it's the second time, add {R}{G}{W}{U}.
// If it's the third time, Omnath deals 4 damage to each opponent and each planeswalker
// you don't control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("omnath-locus-of-creation"),
        name: "Omnath, Locus of Creation".to_string(),
        mana_cost: Some(ManaCost { red: 1, green: 1, white: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elemental"],
        ),
        oracle_text: "When Omnath enters, draw a card.\nLandfall — Whenever a land you control enters, you gain 4 life if this is the first time this ability has resolved this turn. If it's the second time, add {R}{G}{W}{U}. If it's the third time, Omnath deals 4 damage to each opponent and each planeswalker you don't control.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            // ETB: draw a card.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: Landfall with resolution-count tracking (1st/2nd/3rd time this turn).
            // Requires per-object ability resolution counter per turn — not in DSL.
            // 1st: gain 4 life. 2nd: add {R}{G}{W}{U}. 3rd: deal 4 to each opponent + each
            // planeswalker you don't control.
        ],
        ..Default::default()
    }
}
