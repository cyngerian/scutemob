// Fiery Islet — Land
// {T}, Pay 1 life: Add {U} or {R}.
// {1}, {T}, Sacrifice this land: Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fiery-islet"),
        name: "Fiery Islet".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Pay 1 life: Add {U} or {R}.\n{1}, {T}, Sacrifice this land: Draw a \
                      card."
            .to_string(),
        abilities: vec![
            // {T}, Pay 1 life: Add {U} or {R}.
            // SR-34: the "or" is modeled as two separate activated abilities, one per
            // color (tainted_field.rs pattern) — the player chooses by activating one,
            // via TapForMana{ability_index} (CR 605.3b; per memory/decisions.md
            // 2026-07-17, TapForMana{ability_index} IS the choice channel for stackless
            // mana abilities). SR-34 widened enrich_spec_from_def's mana-ability
            // lowering to any cost payable through Command::TapForMana (mana + tap +
            // pay-life + sacrifice-self), so Cost::Sequence([Tap, PayLife(1)]) now
            // lowers into a real ManaAbility (CR 605.1a) instead of a stack-using
            // activated ability. First option: {U}.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // Second color option: {R}.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // {1}, {T}, Sacrifice: Draw a card.
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
            },
        ],
        ..Default::default()
    }
}
