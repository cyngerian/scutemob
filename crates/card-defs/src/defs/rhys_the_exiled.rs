// Rhys the Exiled — {2}{G}, Legendary Creature — Elf Warrior 3/2
// Whenever Rhys attacks, you gain 1 life for each Elf you control.
// {B}, Sacrifice an Elf: Regenerate Rhys.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rhys-the-exiled"),
        name: "Rhys the Exiled".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Warrior"],
        ),
        oracle_text: "Whenever Rhys attacks, you gain 1 life for each Elf you control.\n{B}, \
                      Sacrifice an Elf: Regenerate Rhys."
            .to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // CR 508.1 / 603.2: attack trigger — gain 1 life for each Elf you control.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_subtype: Some(SubType("Elf".to_string())),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 701.19a / 602.2: {B}, Sacrifice an Elf: Regenerate Rhys.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        black: 1,
                        ..Default::default()
                    }),
                    Cost::Sacrifice(TargetFilter {
                        has_subtype: Some(SubType("Elf".to_string())),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::Regenerate {
                    target: EffectTarget::Source,
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
