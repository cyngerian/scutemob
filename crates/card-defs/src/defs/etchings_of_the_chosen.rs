// Etchings of the Chosen — {1}{W}{B}, Enchantment
// As this enchantment enters, choose a creature type.
// Creatures you control of the chosen type get +1/+1.
// {1}, Sacrifice a creature of the chosen type: Target creature you control gains
// indestructible until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("etchings-of-the-chosen"),
        name: "Etchings of the Chosen".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "As this enchantment enters, choose a creature type.\nCreatures you control of the chosen type get +1/+1.\n{1}, Sacrifice a creature of the chosen type: Target creature you control gains indestructible until end of turn.".to_string(),
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
            // "Creatures you control of the chosen type get +1/+1" (CR 205.3m)
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::CreaturesYouControlOfChosenType,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // {1}, Sacrifice a creature of the chosen type: Target creature you control
            // gains indestructible until end of turn.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        has_chosen_subtype: true,
                        ..Default::default()
                    }),
                ]),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(
                            KeywordAbility::Indestructible,
                        ),
                        filter: EffectFilter::DeclaredTarget { index: 0 },
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
