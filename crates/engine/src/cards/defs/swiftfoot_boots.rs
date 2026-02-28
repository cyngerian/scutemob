// 46. Swiftfoot Boots — {2}, Artifact — Equipment; Equipped creature has
// haste and hexproof. Equip {1}.
// CR 702.6a: Equipment static ability grants keywords to equipped creature.
// CR 604.2: Static ability functions while on the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("swiftfoot-boots"),
        name: "Swiftfoot Boots".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature has haste and hexproof.\nEquip {1}".to_string(),
        abilities: vec![
            // CR 702.6a: Equipped creature has Haste and Hexproof (layer 6 ability grant).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [KeywordAbility::Haste, KeywordAbility::Hexproof]
                            .into_iter()
                            .collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // Equip {1}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; sorcery speed (CR 702.6d).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }), // Equip {1}
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
            },
        ],
        ..Default::default()
    }
}
