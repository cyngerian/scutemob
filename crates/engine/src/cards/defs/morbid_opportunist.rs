// Morbid Opportunist — {2}{B}, Creature — Human Rogue 1/3
// Whenever one or more other creatures die, draw a card. This ability triggers
// only once each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("morbid-opportunist"),
        name: "Morbid Opportunist".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Rogue"]),
        oracle_text: "Whenever one or more other creatures die, draw a card. This ability triggers only once each turn.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            // TODO: WheneverCreatureDies overbroad + once-per-turn not in DSL.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: None, exclude_self: true, nontoken_only: false },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
