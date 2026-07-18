// Thundering Falls
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thundering-falls"),
        name: "Thundering Falls".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Island", "Mountain"]),
        oracle_text: "({T}: Add {U} or {R}.)\nThis land enters tapped.\nWhen this land enters, \
                      surveil 1. (Look at the top card of your library. You may put it into your \
                      graveyard.)"
            .to_string(),
        abilities: vec![
            // SR-33 (CR 305.6 / 605.1a): the printed mana ability, modelled
            // explicitly — one activated ability per colour, as `forest.rs`
            // does. The engine does not derive mana abilities from basic land
            // subtypes, so a def that leaves this to CR 305.6 produces nothing
            // at all. `TapForMana { ability_index }` picks the colour.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // CR 614.1c: self-replacement — this land enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // CR 701.25: Surveil 1 on ETB.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Surveil {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // SR-33: the Island/Mountain subtypes do NOT grant mana abilities — the engine
            // does not implement CR 305.6, so this comment previously described a
            // mana source that did not exist and this land produced nothing at all.
            // The printed ability is modelled explicitly above (see `forest.rs`).
        ],
        ..Default::default()
    }
}
