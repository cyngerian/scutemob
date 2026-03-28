// Sunken Ruins — Filter land, {T}: Add {C}. {U/B},{T}: Add {U}{U}, {U}{B}, or {B}{B}.
// CR 605.1a: AddManaFilterChoice produces 1{U}+1{B} (middle option). M10 for full choice.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sunken-ruins"),
        name: "Sunken Ruins".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{U/B}, {T}: Add {U}{U}, {U}{B}, or {B}{B}.".to_string(),
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
            // {U/B}, {T}: Add {U}{U}, {U}{B}, or {B}{B}
            // CR 605.1a: filter land mana ability. Simplified to 1{U}+1{B} (middle option).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        hybrid: vec![HybridMana::ColorColor(ManaColor::Blue, ManaColor::Black)],
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::AddManaFilterChoice {
                    player: PlayerTarget::Controller,
                    color_a: ManaColor::Blue,
                    color_b: ManaColor::Black,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
