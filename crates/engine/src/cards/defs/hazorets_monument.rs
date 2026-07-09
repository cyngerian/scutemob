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
            colored_mana_reduction: None,
        }],
        abilities: vec![
            // PB-AC2 (CR 118.12): "Whenever you cast a creature spell, you may discard a
            // card. If you do, draw a card." Beneficial optional-pay wrapper — the draw
            // only happens if a card is discarded (was previously modeled as an
            // unconditional draw, which was wrong game state).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Creature]),
                    noncreature_only: false,
                    chosen_subtype_filter: false,
                spell_subtype_filter: None,
                },
                effect: Effect::MayPayThenEffect {
                    cost: Cost::DiscardCard,
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    }),
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
