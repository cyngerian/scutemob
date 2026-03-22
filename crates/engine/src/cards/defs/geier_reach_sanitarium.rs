// Geier Reach Sanitarium — Legendary Land, {T}: Add {C}. {2}, {T}: Each player draws a card, then discards a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("geier-reach-sanitarium"),
        name: "Geier Reach Sanitarium".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Legendary], &[CardType::Land], &[]),
        oracle_text: "{T}: Add {C}.\n{2}, {T}: Each player draws a card, then discards a card.".to_string(),
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
            // {2}, {T}: Each player draws a card, then discards a card.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::ForEach {
                    over: ForEachTarget::EachPlayer,
                    effect: Box::new(Effect::Sequence(vec![
                        Effect::DrawCards {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            count: EffectAmount::Fixed(1),
                        },
                        Effect::DiscardCards {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            count: EffectAmount::Fixed(1),
                        },
                    ])),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
