// Throne of the God-Pharaoh — {2}, Legendary Artifact
// At the beginning of your end step, each opponent loses life equal to the number
// of tapped creatures you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("throne-of-the-god-pharaoh"),
        name: "Throne of the God-Pharaoh".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "At the beginning of your end step, each opponent loses life equal to the number of tapped creatures you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                // CR 613 / status.tapped: PB-AC3 TappedCreatureCount.
                effect: Effect::LoseLife {
                    player: PlayerTarget::EachOpponent,
                    amount: EffectAmount::TappedCreatureCount {
                        controller: PlayerTarget::Controller,
                        filter: None,
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
