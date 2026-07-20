// Rugged Prairie — filter land, {T}: Add {C}. {R/W}, {T}: Add {R}{R}, {R}{W}, or {W}{W}.
// CR 605.1a: AddManaFilterChoice produces 1{R}+1{W} (middle option). M10 for full choice.
// PB-RS2: the input hybrid pip in the filter ability's activation cost is now correctly charged (CR 107.4e) -- was free (OOS-RS-2). The output-side fixed-mode simplification noted above is unrelated and remains open.
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
                modes: None,
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
                modes: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "filter mana ability fixed to the {R}{W} mode; the other CR 605.1a modes are \
             unavailable (unrelated to activation-cost payment). PB-RS2: the input hybrid pip is \
             now correctly charged at activation time (was free -- OOS-RS-2).",
        ),
        ..Default::default()
    }
}
