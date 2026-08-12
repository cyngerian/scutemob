// Sword of Light and Shadow — {3}, Artifact — Equipment
// Equipped creature gets +2/+2 and has protection from white and from black.
// Whenever equipped creature deals combat damage to a player, you gain 3 life and you may
// return up to one target creature card from your graveyard to your hand.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-light-and-shadow"),
        name: "Sword of Light and Shadow".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +2/+2 and has protection from white and from \
                      black.\nWhenever equipped creature deals combat damage to a player, you \
                      gain 3 life and you may return up to one target creature card from your \
                      graveyard to your hand.\nEquip {2}"
            .to_string(),
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
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(
                        ProtectionQuality::FromColor(Color::White),
                    )),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(
                        ProtectionQuality::FromColor(Color::Black),
                    )),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 510.3a: "Whenever equipped creature deals combat damage to a player,
            // gain 3 life and return up to one target creature card from your graveyard to hand."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(3),
                    },
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Hand {
                            owner: PlayerTarget::Controller,
                        },
                        controller_override: None,
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // Equip {2}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 2,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {2}" means "[Cost]: Attach this
                // permanent to target creature you control." Printed line MCP-verified as
                // plain "Equip {2}" with no CR 702.6c quality restriction, so the requirement
                // is the unmodified 702.6a one.
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
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
