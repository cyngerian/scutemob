// Flooded Grove — Land; {T}: Add {C}; {G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U}.
// TODO: the filter ability ({G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U}) requires
// hybrid mana costs and a 3-way choice — not expressible in current DSL.
// Implementing colorless tap only.
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
            },
            // {G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U}
            // TODO: Triple-choice mana output not expressible; defaulting to {G}{U}.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::Blue)],
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
