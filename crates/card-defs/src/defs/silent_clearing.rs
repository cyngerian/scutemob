// Silent Clearing — Land
// {T}, Pay 1 life: Add {W} or {B}.
// {1}, {T}, Sacrifice this land: Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("silent-clearing"),
        name: "Silent Clearing".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Pay 1 life: Add {W} or {B}.\n{1}, {T}, Sacrifice this land: Draw a \
                      card."
            .to_string(),
        abilities: vec![
            // {T}, Pay 1 life: Add {W} or {B}.
            // SR-34: the "or" is modeled as two separate activated abilities, one per
            // color (tainted_field.rs pattern) — see fiery_islet.rs for the full
            // rationale. First option: {W}.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // Second color option: {B}.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 1,
                        ..Default::default()
                    }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}
