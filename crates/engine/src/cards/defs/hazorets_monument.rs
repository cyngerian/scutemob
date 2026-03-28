// Hazoret's Monument — {3}, Legendary Artifact
// Red creature spells you cast cost {1} less to cast.
// Whenever you cast a creature spell, you may discard a card. If you do, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hazorets-monument"),
        name: "Hazoret's Monument".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "Red creature spells you cast cost {1} less to cast.\nWhenever you cast a creature spell, you may discard a card. If you do, draw a card.".to_string(),
        // CR 601.2f: Red creature spells controller casts cost {1} less.
        // Uses ColorAndCreature(Red) — compound filter (must be both creature AND red).
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::ColorAndCreature(Color::Red),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        abilities: vec![
            // TODO: "may discard, if you do draw" — optional loot on cast trigger (DSL gap).
            // Creature spell filter applied.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Creature]),
                    noncreature_only: false,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
