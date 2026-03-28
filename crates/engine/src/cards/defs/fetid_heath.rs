// Fetid Heath — Land (filterland)
// {T}: Add {C}. {W/B},{T}: Add {W}{W}, {W}{B}, or {B}{B}.
// CR 605.1a: mana ability. AddManaFilterChoice produces 1{W}+1{B} (middle option).
// Interactive full-choice (2{W} or 2{B}) deferred to M10.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fetid-heath"),
        name: "Fetid Heath".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}.".to_string(),
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
            // {W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}
            // CR 605.1a: filter land mana ability. Simplified to 1{W}+1{B} (middle option).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        hybrid: vec![HybridMana::ColorColor(ManaColor::White, ManaColor::Black)],
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::AddManaFilterChoice {
                    player: PlayerTarget::Controller,
                    color_a: ManaColor::White,
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
