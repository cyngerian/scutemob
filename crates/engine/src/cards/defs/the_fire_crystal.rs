// The Fire Crystal — {2}{R}{R}, Legendary Artifact
// Red spells you cast cost {1} less to cast.
// Creatures you control have haste.
// {4}{R}{R}, {T}: Create a token that's a copy of target creature you control. Sacrifice
// it at the beginning of the next end step.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-fire-crystal"),
        name: "The Fire Crystal".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &[],
        ),
        oracle_text: "Red spells you cast cost {1} less to cast.\nCreatures you control have haste.\n{4}{R}{R}, {T}: Create a token that's a copy of target creature you control. Sacrifice it at the beginning of the next end step.".to_string(),
        abilities: vec![
            // Creatures you control have haste
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // {4}{R}{R}, {T}: Create a token that's a copy of target creature you control.
            // Sacrifice it at the beginning of the next end step.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 4, red: 2, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::CreateTokenCopy {
                    source: EffectTarget::DeclaredTarget { index: 0 },
                    enters_tapped_and_attacking: false,
                    except_not_legendary: false,
                    gains_haste: false,
                    delayed_action: Some((
                        crate::state::stubs::DelayedTriggerTiming::AtNextEndStep,
                        crate::state::stubs::DelayedTriggerAction::SacrificeObject,
                    )),
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    controller: TargetController::You,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasColor(Color::Red),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        ..Default::default()
    }
}
