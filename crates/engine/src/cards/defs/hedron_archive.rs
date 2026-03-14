// 8. Hedron Archive — {4}, Artifact, tap: add {C}{C};
// {2}, tap, sacrifice: draw 2 cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hedron-archive"),
        name: "Hedron Archive".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add {C}{C}.\n{2}, {T}, Sacrifice Hedron Archive: Draw two cards.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 2),
                },
                timing_restriction: None,
                targets: vec![],
            },
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
