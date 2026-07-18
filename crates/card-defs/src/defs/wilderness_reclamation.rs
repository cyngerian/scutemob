// Wilderness Reclamation — {3}{G}, Enchantment
// At the beginning of your end step, untap all lands you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wilderness-reclamation"),
        name: "Wilderness Reclamation".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your end step, untap all lands you control.".to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
            effect: Effect::UntapAll {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Land),
                    controller: TargetController::You,
                    ..Default::default()
                },
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}
