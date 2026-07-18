// Devilish Valet — {2}{R}, Creature — Devil Warrior 1/3; Trample, Haste.
// Alliance — Whenever another creature you control enters, double this creature's power
// until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("devilish-valet"),
        name: "Devilish Valet".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Devil", "Warrior"]),
        oracle_text: "Trample, haste\nAlliance \u{2014} Whenever another creature you control \
                      enters, double this creature's power until end of turn."
            .to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // Alliance — Whenever another creature you control enters, double this
            // creature's power until end of turn. "Double" = add its current power
            // (PowerOf(Source), locked in at resolution per CR 608.2h).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: true,
                },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPowerDynamic {
                            amount: Box::new(EffectAmount::PowerOf(EffectTarget::Source)),
                            negate: false,
                        },
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
