// Cathar's Shield — {0}, Artifact — Equipment
// Equipped creature gets +0/+3 and has vigilance.
// Equip {3}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cathars-shield"),
        name: "Cathar's Shield".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +0/+3 and has vigilance.\nEquip {3} ({3}: Attach to target creature you control. Equip only as a sorcery.)".to_string(),
        abilities: vec![
            // Static: equipped creature gets +0/+3
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(3),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Static: equipped creature has vigilance
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [KeywordAbility::Vigilance].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Equip {3}
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
