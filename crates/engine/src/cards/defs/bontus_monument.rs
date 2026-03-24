// Bontu's Monument — {3}, Legendary Artifact
// Black creature spells you cast cost {1} less to cast.
// Whenever you cast a creature spell, each opponent loses 1 life and you gain 1 life.
//
// TODO: "Black creature spells" — SpellCostFilter needs compound HasColor+HasCardType filter.
//   HasColor(Black) alone reduces all black spells, not just creatures. Wrong game state.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bontus-monument"),
        name: "Bontu's Monument".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "Black creature spells you cast cost {1} less to cast.\nWhenever you cast a creature spell, each opponent loses 1 life and you gain 1 life.".to_string(),
        abilities: vec![
            // TODO: "Black creature spells cost {1} less" — color+type cost reduction not in DSL.
            // Whenever you cast a creature spell, each opponent loses 1 life and you gain 1 life.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Creature]),
                    noncreature_only: false,
                },
                effect: Effect::Sequence(vec![
                    Effect::ForEach {
                        over: ForEachTarget::EachOpponent,
                        effect: Box::new(Effect::LoseLife {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(1),
                        }),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
