// The Reaver Cleaver — {2}{R}, Legendary Artifact — Equipment
// Equipped creature gets +1/+1 and has trample and "Whenever this creature deals combat
// damage to a player or planeswalker, create that many Treasure tokens."
// Equip {3}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-reaver-cleaver"),
        name: "The Reaver Cleaver".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &["Equipment"],
        ),
        oracle_text: "Equipped creature gets +1/+1 and has trample and \"Whenever this creature \
                      deals combat damage to a player or planeswalker, create that many Treasure \
                      tokens.\"\nEquip {3}"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Trample),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 510.3a: "Whenever equipped creature deals combat damage to a player,
            // create that many Treasure tokens." — equipment trigger with Repeat for amount.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Repeat {
                    effect: Box::new(Effect::CreateToken {
                        spec: treasure_token_spec(1),
                    }),
                    count: EffectAmount::CombatDamageDealt,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // Equip {3}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 3,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {3}" means "[Cost]: Attach this
                // permanent to target creature you control." Printed line MCP-verified as
                // plain "Equip {3}" with no CR 702.6c quality restriction, so the requirement
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
