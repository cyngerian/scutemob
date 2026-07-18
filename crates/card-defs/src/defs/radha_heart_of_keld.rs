// Radha, Heart of Keld — {1}{R}{G}, Legendary Creature — Elf Warrior 3/3
// During your turn, Radha has first strike.
// You may look at the top card of your library any time, and you may play lands from
// the top of your library.
// {4}{R}{G}: Radha gets +X/+X until end of turn, where X is the number of lands you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("radha-heart-of-keld"),
        name: "Radha, Heart of Keld".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            green: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Warrior"],
        ),
        oracle_text: "During your turn, Radha has first strike.\nYou may look at the top card of \
                      your library any time, and you may play lands from the top of your \
                      library.\n{4}{R}{G}: Radha gets +X/+X until end of turn, where X is the \
                      number of lands you control."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // CR 604.2 / CR 613.1f (Layer 6): "During your turn, Radha has first strike."
            // Active only when it is the controller's turn (Condition::IsYourTurn).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::FirstStrike),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::IsYourTurn),
                },
            },
            // CR 601.3 / CR 305.1 (PB-A): "You may look at the top card of your library any time,
            // and you may play lands from the top of your library."
            // look_at_top: true (controller sees top card), LandsOnly filter.
            AbilityDefinition::StaticPlayFromTop {
                filter: PlayFromTopFilter::LandsOnly,
                look_at_top: true,
                reveal_top: false,
                pay_life_instead: false,
                condition: None,
                on_cast_effect: None,
            },
            // {4}{R}{G}: Radha gets +X/+X until end of turn, where X is the number of
            // lands you control. X is locked in at resolution (CR 608.2h).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 4,
                    red: 1,
                    green: 1,
                    ..Default::default()
                }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBothDynamic {
                            amount: Box::new(EffectAmount::PermanentCount {
                                filter: TargetFilter {
                                    has_card_type: Some(CardType::Land),
                                    ..Default::default()
                                },
                                controller: PlayerTarget::Controller,
                            }),
                            negate: false,
                        },
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
                modes: None,
            },
        ],
        ..Default::default()
    }
}
