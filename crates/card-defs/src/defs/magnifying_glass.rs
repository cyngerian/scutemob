// 106. Magnifying Glass — {3}, Artifact; {T}: Add {C}. {4},{T}: Investigate.
// CR 701.16a: Investigate — create a Clue token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("magnifying-glass"),
        name: "Magnifying Glass".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add {C}.\n{4}, {T}: Investigate. (Create a Clue token. It's an \
                      artifact with \"{2}, Sacrifice this token: Draw a card.\")"
            .to_string(),
        abilities: vec![
            // {T}: Add {C}.
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
            // {4}, {T}: Investigate.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 4,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::Investigate {
                    count: EffectAmount::Fixed(1),
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
