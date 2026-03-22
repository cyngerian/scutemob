// Bontu's Monument — {3}, Legendary Artifact
// Black creature spells you cast cost {1} less to cast.
// Whenever you cast a creature spell, each opponent loses 1 life and you gain 1 life.
//
// TODO: "Black creature spells" filter — SpellCostFilter needs a compound HasColor+HasCardType.
//   Using HasColor(Black) only (slightly wrong — reduces all black spells, not just creatures).
// TODO: "Whenever you cast a creature spell" — WheneverYouCastSpell lacks a creature filter.
//   Using unfiltered trigger (fires on all spells — slightly wrong).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bontus-monument"),
        name: "Bontu's Monument".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "Black creature spells you cast cost {1} less to cast.\nWhenever you cast a creature spell, each opponent loses 1 life and you gain 1 life.".to_string(),
        abilities: vec![
            // Whenever you cast a creature spell, drain 1
            // TODO: should be creature spells only
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                },
                effect: Effect::DrainLife { amount: EffectAmount::Fixed(1) },
                intervening_if: None,
                targets: vec![],
            },
        ],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            // TODO: should be HasColor(Black) AND HasCardType(Creature) compound
            filter: SpellCostFilter::HasColor(Color::Black),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        ..Default::default()
    }
}
