// Champion of Lambholt — {1}{G}{G}, Creature — Human Warrior 1/1
// Creatures with power less than this creature's power can't block creatures you control.
// Whenever another creature you control enters, put a +1/+1 counter on this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("champion-of-lambholt"),
        name: "Champion of Lambholt".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: creature_types(&["Human", "Warrior"]),
        oracle_text: "Creatures with power less than Champion of Lambholt's power can't block creatures you control.\nWhenever another creature you control enters, put a +1/+1 counter on Champion of Lambholt.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // TODO: DSL gap — "Creatures with power less than ~'s power can't block creatures
            // you control." Dynamic blocking restriction based on source power not in DSL.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
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
