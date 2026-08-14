// Sword of Truth and Justice — {3}, Artifact — Equipment
// Equipped creature gets +2/+2 and has protection from white and from blue.
// Whenever equipped creature deals combat damage to a player, put a +1/+1 counter on a
// creature you control, then proliferate.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-truth-and-justice"),
        name: "Sword of Truth and Justice".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +2/+2 and has protection from white and from \
                      blue.\nWhenever equipped creature deals combat damage to a player, put a \
                      +1/+1 counter on a creature you control, then proliferate.\nEquip {2}"
            .to_string(),
        abilities: vec![
            // Layer 7c: equipped creature gets +2/+2.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Layer 6: equipped creature has protection from white.
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
            // Layer 6: equipped creature has protection from blue.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(
                        ProtectionQuality::FromColor(Color::Blue),
                    )),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 510.3a / CR 115.10 (PB-DX28): "Whenever equipped creature deals combat
            // damage to a player, put a +1/+1 counter on a creature you control, then
            // proliferate." Printed with no "target" — this is a resolution-time
            // UNTARGETED choice (CR 115.10), not a declared target. Previously authored
            // as a real `TargetCreatureWithFilter`, which was wrong in BOTH directions
            // (OOS-DX4-6, filed by PB-DX4's fix cycle): hexproof/shroud/protection
            // wrongly restricted the choice, and CR 608.2b fizzled the WHOLE trigger —
            // counter AND proliferate — when the chosen creature left in response, where
            // the printed card would simply choose another creature on resolution.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::ChosenObject {
                            zone: ChoiceZone::Battlefield,
                            filter: Box::new(TargetFilter {
                                has_card_type: Some(CardType::Creature),
                                controller: TargetController::You,
                                ..Default::default()
                            }),
                            count: 1,
                            up_to: false,
                        },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::Proliferate,
                ]),
                intervening_if: None,
                targets: vec![],
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
