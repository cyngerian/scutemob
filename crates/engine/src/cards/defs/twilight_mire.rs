// Twilight Mire — filter land, {T}: Add {C}. {B/G}, {T}: Add {B}{B}, {B}{G}, or {G}{G} (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("twilight-mire"),
        name: "Twilight Mire".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{B/G}, {T}: Add {B}{B}, {B}{G}, or {G}{G}.".to_string(),
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
            // {B/G}, {T}: Add {B}{B}, {B}{G}, or {G}{G}.
            // TODO: Triple-choice mana output not expressible with current DSL.
            // Hybrid activation cost is correct; defaulting output to {B}{G}.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        hybrid: vec![HybridMana::ColorColor(ManaColor::Black, ManaColor::Green)],
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
