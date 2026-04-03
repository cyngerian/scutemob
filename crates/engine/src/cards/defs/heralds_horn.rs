// Herald's Horn — {3}, Artifact
// As this enters, choose a creature type.
// Creature spells of the chosen type cost {1} less to cast.
// At the beginning of your upkeep, look at the top card of your library. If it's a
// creature card of the chosen type, you may reveal it and put it into your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("heralds-horn"),
        name: "Herald's Horn".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "As Herald's Horn enters, choose a creature type.\nCreature spells you cast of the chosen type cost {1} less to cast.\nAt the beginning of your upkeep, look at the top card of your library. If it's a creature card of the chosen type, you may reveal it and put it into your hand.".to_string(),
        abilities: vec![
            // "As this enters, choose a creature type" — self-replacement (CR 614.1c)
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseCreatureType(SubType("Human".to_string())),
                is_self: true,
                unless_condition: None,
            },
            // Upkeep trigger: if top card is a creature of the chosen type, draw it.
            // Deterministic approximation: if condition is true, draw (bot always takes the card).
            // CR 614.1c note: "look at the top card" is a hidden-info peek; deterministic engine
            // sees all. Per ruling, if you don't put it into your hand it stays on top unrevealed.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::Conditional {
                    condition: Condition::TopCardIsCreatureOfChosenType,
                    if_true: Box::new(Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    }),
                    if_false: Box::new(Effect::Nothing),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        // Creature spells of the chosen type cost {1} less to cast (CR 601.2f).
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasChosenCreatureSubtype,
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
            colored_mana_reduction: None,
        }],
        ..Default::default()
    }
}
