// 45. Lightning Greaves — {2}, Artifact — Equipment; Equipped creature has
// haste and shroud. Equip {0}.
// CR 702.6a: Equipment static ability grants keywords to equipped creature.
// CR 604.2: Static ability functions while on the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lightning-greaves"),
        name: "Lightning Greaves".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature has haste and shroud. (It can't be the target of spells or abilities your opponents control.)\nEquip {0}".to_string(),
        abilities: vec![
            // CR 702.6a: Equipped creature has Haste and Shroud (layer 6 ability grant).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [KeywordAbility::Haste, KeywordAbility::Shroud]
                            .into_iter()
                            .collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // Equip {0}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; sorcery speed (CR 702.6d).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { ..Default::default() }), // Equip {0}
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
