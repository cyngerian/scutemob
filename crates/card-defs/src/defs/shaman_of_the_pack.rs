// Shaman of the Pack — {1}{B}{G}, Creature — Elf Shaman 3/2
// When this creature enters, target opponent loses life equal to the number of Elves
// you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shaman-of-the-pack"),
        name: "Shaman of the Pack".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Shaman"]),
        oracle_text: "When this creature enters, target opponent loses life equal to the number \
                      of Elves you control."
            .to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // CR 603.3d: "When this creature enters, target opponent loses life equal to the
            // number of Elves you control." PB-EF6: TargetRequirement::TargetOpponent supplies
            // the opponent-only target restriction.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::LoseLife {
                    player: PlayerTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_subtype: Some(SubType("Elf".to_string())),
                            controller: TargetController::You,
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetOpponent],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
