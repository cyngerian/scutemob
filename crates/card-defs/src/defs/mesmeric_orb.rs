// Mesmeric Orb — {2}, Artifact
// Whenever a permanent becomes untapped, that permanent's controller mills a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mesmeric-orb"),
        name: "Mesmeric Orb".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever a permanent becomes untapped, that permanent's controller mills a \
                      card."
            .to_string(),
        abilities: vec![
            // CR 502.3 / 603.2e: "Whenever a permanent becomes untapped, that permanent's
            // controller mills a card." Global trigger (filter: None = any permanent, any
            // controller). The milling player is the CONTROLLER OF the untapped permanent
            // (the trigger's "TriggeringCreature", resolved via entering_object_id --
            // the field name is legacy and applies to any triggering object, not just
            // creatures).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentUntaps { filter: None },
                effect: Effect::MillCards {
                    player: PlayerTarget::ControllerOf(Box::new(EffectTarget::TriggeringCreature)),
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
