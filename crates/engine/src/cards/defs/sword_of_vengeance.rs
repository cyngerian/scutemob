// Sword of Vengeance — {3}, Artifact — Equipment
// Equipped creature gets +2/+0 and has first strike, vigilance, trample, and haste.
// Equip {3}
//
// CR 702.6a: Equipment static ability grants keywords to equipped creature.
// CR 613.4c: +2 power via layer 7c ModifyPower.
// CR 702.6b/d: Equip is a sorcery-speed activated ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-vengeance"),
        name: "Sword of Vengeance".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +2/+0 and has first strike, vigilance, trample, and haste.\nEquip {3}".to_string(),
        abilities: vec![
            // Static: Equipped creature gets +2/+0 (layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // Static: Equipped creature has First Strike, Vigilance, Trample, Haste (layer 6).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [
                            KeywordAbility::FirstStrike,
                            KeywordAbility::Vigilance,
                            KeywordAbility::Trample,
                            KeywordAbility::Haste,
                        ]
                        .into_iter()
                        .collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // Equip {3}: attach this Equipment to target creature you control (CR 702.6b/d).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
