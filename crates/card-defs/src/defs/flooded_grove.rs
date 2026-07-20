// Flooded Grove — Land; {T}: Add {C}; {G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U}.
// CR 605.1a: AddManaFilterChoice produces 1{G}+1{U} (middle option). M10 for full choice.
// PB-RS2: the input hybrid pip in the filter ability's activation cost is now
// correctly charged (CR 107.4e) -- was free (OOS-RS-2). The output-side fixed-mode
// simplification noted above is unrelated and remains open.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flooded-grove"),
        name: "Flooded Grove".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U}.".to_string(),
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
            // {G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U}
            // CR 605.1a: filter land mana ability. Simplified to 1{G}+1{U} (middle option).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::Blue)],
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::AddManaFilterChoice {
                    player: PlayerTarget::Controller,
                    color_a: ManaColor::Green,
                    color_b: ManaColor::Blue,
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
            "filter mana ability fixed to the {G}{U} mode; the other CR 605.1a modes are \
             unavailable (unrelated to activation-cost payment). PB-RS2: the input hybrid pip is \
             now correctly charged at activation time (was free -- OOS-RS-2).",
        ),
        ..Default::default()
    }
}
