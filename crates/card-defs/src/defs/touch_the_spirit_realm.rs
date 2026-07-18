// Touch the Spirit Realm — {2}{W}, Enchantment
// When this enchantment enters, exile up to one target artifact or creature until this
// enchantment leaves the battlefield.
// Channel — {1}{W}, Discard this card: Exile target artifact or creature. Return it to the
// battlefield under its owner's control at the beginning of the next end step.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("touch-the-spirit-realm"),
        name: "Touch the Spirit Realm".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When this enchantment enters, exile up to one target artifact or creature \
                      until this enchantment leaves the battlefield.\nChannel \u{2014} {1}{W}, \
                      Discard this card: Exile target artifact or creature. Return it to the \
                      battlefield under its owner's control at the beginning of the next end step."
            .to_string(),
        abilities: vec![
            // When this enchantment enters, exile up to one target artifact or creature until
            // this enchantment leaves the battlefield (CR 610.3).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ExileWithDelayedReturn {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    return_timing:
                        crate::state::stubs::DelayedTriggerTiming::WhenSourceLeavesBattlefield,
                    return_tapped: false,
                    return_to: crate::cards::card_definition::DelayedReturnDestination::Battlefield,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::UpToN {
                    count: 1,
                    inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        has_card_types: vec![CardType::Artifact, CardType::Creature],
                        ..Default::default()
                    })),
                }],
                modes: None,
                trigger_zone: None,
            },
            // Channel — {1}{W}, Discard this card: Exile target artifact or creature. Return
            // it to the battlefield under its owner's control at the beginning of the next
            // end step.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 1,
                        white: 1,
                        ..Default::default()
                    }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::ExileWithDelayedReturn {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    return_timing: crate::state::stubs::DelayedTriggerTiming::AtNextEndStep,
                    return_tapped: false,
                    return_to: crate::cards::card_definition::DelayedReturnDestination::Battlefield,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Creature],
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}
