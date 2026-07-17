// Jaddi Offshoot — {G}, Creature — Plant 0/3
// Defender
// Landfall — Whenever a land you control enters, you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("jaddi-offshoot"),
        name: "Jaddi Offshoot".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Plant"]),
        oracle_text: "Defender\nLandfall — Whenever a land you control enters, you gain 1 life."
            .to_string(),
        power: Some(0),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Defender),
            // Landfall — gain 1 life when a land you control enters
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
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
