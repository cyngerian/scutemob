// Marwyn, the Nurturer — {2}{G}, Legendary Creature — Elf Druid 1/1
// Whenever another Elf you control enters, put a +1/+1 counter on Marwyn.
// {T}: Add an amount of {G} equal to Marwyn's power.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("marwyn-the-nurturer"),
        name: "Marwyn, the Nurturer".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elf", "Druid"]),
        oracle_text: "Whenever another Elf you control enters, put a +1/+1 counter on Marwyn, the Nurturer.\n{T}: Add an amount of {G} equal to Marwyn's power.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // Whenever another Elf you control enters, put a +1/+1 counter on this.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Elf".to_string())),
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
            // {T}: Add an amount of {G} equal to Marwyn's power.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaScaled {
                    player: PlayerTarget::Controller,
                    color: ManaColor::Green,
                    count: EffectAmount::PowerOf(EffectTarget::Source),
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
