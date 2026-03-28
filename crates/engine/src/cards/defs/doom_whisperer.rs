// Doom Whisperer — {3}{B}{B}, Creature — Nightmare Demon 6/6
// Flying, trample
// Pay 2 life: Surveil 2.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("doom-whisperer"),
        name: "Doom Whisperer".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: creature_types(&["Nightmare", "Demon"]),
        oracle_text: "Flying, trample\nPay 2 life: Surveil 2.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Activated {
                cost: Cost::PayLife(2),
                effect: Effect::Surveil {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
