// Bontu's Monument — {3}, Legendary Artifact
// Black creature spells you cast cost {1} less to cast.
// Whenever you cast a creature spell, each opponent loses 1 life and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bontus-monument"),
        name: "Bontu's Monument".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "Black creature spells you cast cost {1} less to cast.\nWhenever you cast a creature spell, each opponent loses 1 life and you gain 1 life.".to_string(),
        // CR 601.2f: Black creature spells controller casts cost {1} less.
        // Uses ColorAndCreature(Black) — compound filter (must be both creature AND black).
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::ColorAndCreature(Color::Black),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        abilities: vec![
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
