// Indulgent Aristocrat — {B}, Creature — Vampire Noble 1/1
// Lifelink
// {2}, Sacrifice a creature: Put a +1/+1 counter on each Vampire you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("indulgent-aristocrat"),
        name: "Indulgent Aristocrat".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Noble"]),
        oracle_text: "Lifelink\n{2}, Sacrifice a creature: Put a +1/+1 counter on each Vampire you control.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::ForEach {
                    over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        has_subtype: Some(SubType("Vampire".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    })),
                    effect: Box::new(Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    }),
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
