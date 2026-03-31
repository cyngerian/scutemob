// Stitcher's Supplier — {B}, Creature — Zombie 1/1
// When this creature enters or dies, mill three cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stitchers-supplier"),
        name: "Stitcher's Supplier".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Zombie"]),
        oracle_text: "When this creature enters or dies, mill three cards. (Put the top three cards of your library into your graveyard.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // ETB: mill three cards (CR 603.1).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MillCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // Death trigger: mill three cards (CR 603.10a).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::MillCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
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
