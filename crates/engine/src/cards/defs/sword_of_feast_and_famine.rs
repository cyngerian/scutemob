// Sword of Feast and Famine — {3}, Artifact — Equipment
// Equipped creature gets +2/+2 and has protection from black and from green.
// Whenever equipped creature deals combat damage to a player, that player discards a card
// and you untap all lands you control.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-feast-and-famine"),
        name: "Sword of Feast and Famine".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +2/+2 and has protection from black and from green.\nWhenever equipped creature deals combat damage to a player, that player discards a card and you untap all lands you control.\nEquip {2}".to_string(),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Black))),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Green))),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 510.3a: "Whenever equipped creature deals combat damage to a player,
            // that player discards a card and you untap all lands you control."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::DiscardCards {
                        player: PlayerTarget::DamagedPlayer,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::ForEach {
                        over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
                            has_card_type: Some(CardType::Land),
                            controller: TargetController::You,
                            ..Default::default()
                        })),
                        effect: Box::new(Effect::UntapPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        }),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}
