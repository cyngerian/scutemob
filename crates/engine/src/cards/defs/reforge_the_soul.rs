// Reforge the Soul — {3}{R}{R}, Sorcery
// Each player discards their hand, then draws seven cards.
// Miracle {1}{R}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reforge-the-soul"),
        name: "Reforge the Soul".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each player discards their hand, then draws seven cards.\nMiracle {1}{R} (You may cast this card for its miracle cost when you draw it if it's the first card you drew this turn.)".to_string(),
        abilities: vec![
            // TODO: Miracle {1}{R} — KeywordAbility::Miracle not yet implemented.
            // When Miracle is added, include AltCastAbility with Miracle cost.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // Each player discards their hand.
                    // TODO: EffectAmount::HandSize not in DSL — using Fixed(7) approximation.
                    Effect::DiscardCards {
                        player: PlayerTarget::EachPlayer,
                        count: EffectAmount::Fixed(7),
                    },
                    // Then draws seven cards.
                    Effect::DrawCards {
                        player: PlayerTarget::EachPlayer,
                        count: EffectAmount::Fixed(7),
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
