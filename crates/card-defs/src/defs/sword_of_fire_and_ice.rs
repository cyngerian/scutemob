// Sword of Fire and Ice — {3} Artifact — Equipment
// Equipped creature gets +2/+2 and has protection from red and from blue.
// Whenever equipped creature deals combat damage to a player, this Equipment deals
// 2 damage to any target and you draw a card.
// Equip {2}
use crate::cards::helpers::*;
use crate::state::types::ProtectionQuality;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-fire-and-ice"),
        name: "Sword of Fire and Ice".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +2/+2 and has protection from red and from \
                      blue.\nWhenever equipped creature deals combat damage to a player, Sword of \
                      Fire and Ice deals 2 damage to any target and you draw a card.\nEquip {2}"
            .to_string(),
        abilities: vec![
            // +2/+2 static buff to equipped creature
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Protection from red and blue
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [
                            KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(
                                Color::Red,
                            )),
                            KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(
                                Color::Blue,
                            )),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 510.3a: "Whenever equipped creature deals combat damage to a player,
            // deal 2 damage to any target and draw a card."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::DealDamage {
                        source: None,
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(2),
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::TargetAny],

                modes: None,
                trigger_zone: None,
            },
            // Equip {2}
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
                // CARDS-1 (OOS-M11-10) / CR 702.6a: "Equip {2}" means "[Cost]: Attach this
                // permanent to target creature you control." The printed line was MCP-verified
                // as plain "Equip {2}" -- no CR 702.6c quality restriction -- so the requirement is
                // the unmodified 702.6a one. Authoring it is what makes the target announceable:
                // an empty `targets` list left `queries::ability_target_requirements` reporting
                // zero slots, so the browser picker never asked and the attach silently fizzled
                // with the cost paid.
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
