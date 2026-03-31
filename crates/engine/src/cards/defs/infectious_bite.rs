// Infectious Bite — {1}{G}, Instant
// Target creature you control deals damage equal to its power to target creature you
// don't control. Each opponent gets a poison counter.
//
// Bite handles the fight-adjacent "deals damage equal to its power" to one target.
// Poison counters via ForEach over EachOpponent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("infectious-bite"),
        name: "Infectious Bite".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Target creature you control deals damage equal to its power to target creature you don't control. Each opponent gets a poison counter.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // CR 701.12: source creature (index 0) deals damage equal to its power
                // to target creature (index 1).
                Effect::Bite {
                    source: EffectTarget::DeclaredTarget { index: 0 },
                    target: EffectTarget::DeclaredTarget { index: 1 },
                },
                // "Each opponent gets a poison counter."
                Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::Poison,
                        count: 1,
                    }),
                },
            ]),
            targets: vec![
                TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                }),
                TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                }),
            ],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
