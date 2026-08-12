// Kite Shield — {0}, Artifact — Equipment
// Equipped creature gets +0/+3.
// Equip {3}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kite-shield"),
        name: "Kite Shield".to_string(),
        mana_cost: Some(ManaCost::default()),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +0/+3.\nEquip {3}".to_string(),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(3),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
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
