// Rugged Prairie — filter land, {T}: Add {C}. {R/W}, {T}: Add {R}{R}, {R}{W}, or {W}{W} (TODO).
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
            },
            // {R/W}, {T}: Add {R}{R}, {R}{W}, or {W}{W}
            // TODO: Triple-choice mana output not expressible; defaulting to {R}{W}.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        hybrid: vec![HybridMana::ColorColor(ManaColor::Red, ManaColor::White)],
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
