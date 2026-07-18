// Ogre Battledriver — {2}{R}{R}, Creature — Ogre Warrior 3/3
// Whenever another creature you control enters, that creature gets +2/+0 and gains
// haste until end of turn.
//
// PB-EF4: "that creature" = the entering creature, aimed via EffectFilter::TriggeringCreature
// on two ApplyContinuousEffect grants (P/T pump + Haste, both until end of turn).
// exclude_self: true = "another" (Ogre's own ETB does not fire this trigger).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ogre-battledriver"),
        name: "Ogre Battledriver".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Ogre", "Warrior"]),
        oracle_text: "Whenever another creature you control enters, that creature gets +2/+0 and \
                      gains haste until end of turn. (It can attack and {T} this turn.)"
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // CR 603.6a / CR 611.2a: "Whenever another creature you control enters, that
            // creature gets +2/+0 and gains haste until end of turn." Both continuous
            // effects target EffectFilter::TriggeringCreature (the entering creature).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: true,
                },
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyPower(2),
                            filter: EffectFilter::TriggeringCreature,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                            filter: EffectFilter::TriggeringCreature,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
