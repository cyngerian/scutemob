// Rugged Prairie — filter land, {T}: Add {C}. {R/W}, {T}: Add {R}{R}, {R}{W}, or {W}{W}.
// CR 605.1a: AddManaFilterChoice produces 1{R}+1{W} (middle option). M10 for full choice.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rugged-prairie"),
        name: "Rugged Prairie".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{R/W}, {T}: Add {R}{R}, {R}{W}, or {W}{W}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {R/W}, {T}: Add {R}{R}, {R}{W}, or {W}{W}
            // CR 605.1a: filter land mana ability. Simplified to 1{R}+1{W} (middle option).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        hybrid: vec![HybridMana::ColorColor(ManaColor::Red, ManaColor::White)],
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::AddManaFilterChoice {
                    player: PlayerTarget::Controller,
                    color_a: ManaColor::Red,
                    color_b: ManaColor::White,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
