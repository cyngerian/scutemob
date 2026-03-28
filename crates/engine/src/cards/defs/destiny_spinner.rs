// Destiny Spinner — {1}{G}, Enchantment Creature — Human 2/3.
// Creature and enchantment spells you control can't be countered.
// {3}{G}: Target land becomes X/X Elemental with trample and haste until EOT, where
// X = number of enchantments you control.
// TODO: DSL gap — "can't be countered" static for specific spell types not expressible
// (AbilityDefinition::Spell has cant_be_countered but only for the card itself, not a
// blanket static grant); land animation with X based on enchantment count not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("destiny-spinner"),
        name: "Destiny Spinner".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Human"]),
        oracle_text: "Creature and enchantment spells you control can't be countered.\n{3}{G}: Target land you control becomes an X/X Elemental creature with trample and haste until end of turn, where X is the number of enchantments you control. It's still a land.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // TODO: static "can't counter creature/enchantment spells you control"
            // CR 613.1d/613.4b: {3}{G}: Target land becomes an X/X Elemental with trample
            // and haste until end of turn, where X is the number of enchantments you control.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 3, green: 1, ..Default::default() }),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::AddCardTypes(
                                [CardType::Creature].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::AddSubtypes(
                                [SubType("Elemental".to_string())].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtSet,
                            modification: LayerModification::SetPtDynamic {
                                power: Box::new(EffectAmount::PermanentCount {
                                    filter: TargetFilter {
                                        has_card_type: Some(CardType::Enchantment),
                                        controller: TargetController::You,
                                        ..Default::default()
                                    },
                                    controller: PlayerTarget::Controller,
                                }),
                                toughness: Box::new(EffectAmount::PermanentCount {
                                    filter: TargetFilter {
                                        has_card_type: Some(CardType::Enchantment),
                                        controller: TargetController::You,
                                        ..Default::default()
                                    },
                                    controller: PlayerTarget::Controller,
                                }),
                            },
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeywords(
                                [KeywordAbility::Trample, KeywordAbility::Haste]
                                    .into_iter()
                                    .collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetLand],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        // TODO: static "can't counter creature/enchantment spells you control"
        ..Default::default()
    }
}
