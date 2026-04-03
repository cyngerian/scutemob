// Three Tree City — Legendary Land
// As this land enters, choose a creature type.
// {T}: Add {C}.
// {2}, {T}: Choose a color. Add an amount of mana of that color equal to the number of
// creatures you control of the chosen type.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("three-tree-city"),
        name: "Three Tree City".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "As Three Tree City enters, choose a creature type.\n{T}: Add {C}.\n{2}, {T}: Choose a color. Add an amount of mana of that color equal to the number of creatures you control of the chosen type.".to_string(),
        abilities: vec![
            // "As this enters, choose a creature type"
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseCreatureType(SubType("Human".to_string())),
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {C}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // {2}, {T}: Add N mana of any color (deterministic: colorless), where N = count of
            // creatures you control of the chosen type.
            // Interactive color choice deferred to M10.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::AddManaOfAnyColorAmount {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::ChosenTypeCreatureCount {
                        controller: PlayerTarget::Controller,
                    },
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
