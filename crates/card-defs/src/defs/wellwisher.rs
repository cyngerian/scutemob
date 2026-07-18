// Wellwisher — {1}{G}, Creature — Elf 1/1
// {T}: You gain 1 life for each Elf on the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wellwisher"),
        name: "Wellwisher".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf"]),
        oracle_text: "{T}: You gain 1 life for each Elf on the battlefield.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            // Elves on the whole battlefield (any controller), including Wellwisher
            // itself — PermanentCount with controller: EachPlayer sums across all
            // players, filter carries no controller restriction.
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_subtype: Some(SubType("Elf".to_string())),
                        ..Default::default()
                    },
                    controller: PlayerTarget::EachPlayer,
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    }
}
