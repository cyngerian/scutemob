// Regal Force — {4}{G}{G}{G}, Creature — Elemental 5/5
// When this creature enters, draw a card for each green creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("regal-force"),
        name: "Regal Force".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 3, ..Default::default() }),
        types: creature_types(&["Elemental"]),
        oracle_text: "When this creature enters, draw a card for each green creature you control.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // TODO: "green creature" filter — PermanentCount with color filter.
            //   Using all creatures as approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            controller: TargetController::You,
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
        ],
        ..Default::default()
    }
}
