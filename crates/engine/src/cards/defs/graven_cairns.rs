// Graven Cairns — Filter land, {T}: Add {C}. {B/R},{T}: Add {B}{B}, {B}{R}, or {R}{R}.
// CR 605.1a: AddManaFilterChoice produces 1{B}+1{R} (middle option). M10 for full choice.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("graven-cairns"),
        name: "Graven Cairns".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{B/R}, {T}: Add {B}{B}, {B}{R}, or {R}{R}.".to_string(),
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
            },
            // {B/R}, {T}: Add {B}{B}, {B}{R}, or {R}{R}
            // CR 605.1a: filter land mana ability. Simplified to 1{B}+1{R} (middle option).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        hybrid: vec![HybridMana::ColorColor(ManaColor::Black, ManaColor::Red)],
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::AddManaFilterChoice {
                    player: PlayerTarget::Controller,
                    color_a: ManaColor::Black,
                    color_b: ManaColor::Red,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
