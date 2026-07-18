// Scourge of Valkas — {2}{R}{R}{R}, Creature — Dragon 4/4
// Flying
// Whenever this creature or another Dragon you control enters, it deals X damage to any
// target, where X is the number of Dragons you control.
// {R}: This creature gets +1/+0 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scourge-of-valkas"),
        name: "Scourge of Valkas".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 3,
            ..Default::default()
        }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever this creature or another Dragon you control enters, it \
                      deals X damage to any target, where X is the number of Dragons you \
                      control.\n{R}: This creature gets +1/+0 until end of turn."
            .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // "Whenever this creature ... enters, it deals X damage to any target, where X is
            // the number of Dragons you control." Self-ETB half only (CR 603.3): here "it" =
            // Scourge = the ability's source, so Effect::DealDamage's implicit ctx.source
            // sourcing is faithful. See the completeness note for why the "another Dragon you
            // control enters" half is NOT authored here.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Dragon".to_string())),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetAny],

                modes: None,
                trigger_zone: None,
            },
            // CR 613.4c: "{R}: This creature gets +1/+0 until end of turn."
            // EffectFilter::Source resolves to SingleObject(ctx.source) at execution time.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    red: 1,
                    ..Default::default()
                }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPower(1),
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        completeness: Completeness::partial(
            "The self-ETB half ('this creature ... enters') is now authored and Complete-faithful \
             (WhenEntersBattlefield + DealDamage{amount: PermanentCount{Dragon,You}}, \
             targets:[TargetAny] — ctx.source IS Scourge here, so the implicit DealDamage source \
             matches oracle 'it'). The residual: 'another Dragon you control enters' — \
             WheneverCreatureEntersBattlefield{filter:{Dragon,You},exclude_self:true} is \
             wireable, but 'it deals X damage' there means the ENTERING Dragon deals the damage, \
             and Effect::DealDamage has no source-override field (always sources from ctx.source, \
             i.e. Scourge, not the entering Dragon — confirmed no def in the corpus uses a \
             `source:` field on DealDamage). Implementing that half would silently misattribute \
             the damage source (wrong for protection/redirection/'a source you control' \
             interactions) — left unauthored per W5 rather than shipped wrong.",
        ),
        ..Default::default()
    }
}
