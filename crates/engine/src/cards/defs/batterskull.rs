// 48. Batterskull — {5}, Artifact — Equipment; Living weapon. Equipped creature gets
// +4/+4 and has vigilance and lifelink. Equip {5}.
// CR 702.92a: Living weapon ETB trigger creates 0/0 black Phyrexian Germ, attaches.
// CR 702.6a: Equipment static ability grants keywords to equipped creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("batterskull"),
        name: "Batterskull".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text:
            "Living weapon (When this Equipment enters, create a 0/0 black Phyrexian Germ \
             creature token, then attach this Equipment to it.)\n\
             Equipped creature gets +4/+4 and has vigilance and lifelink.\n\
             Equip {5}"
                .to_string(),
        abilities: vec![
            // CR 702.92a: Living weapon — ETB trigger handled by builder.rs keyword wiring.
            AbilityDefinition::Keyword(KeywordAbility::LivingWeapon),
            // CR 702.6a: Equipped creature gets +4/+4 (layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(4),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // CR 702.6a: Equipped creature has vigilance and lifelink (layer 6).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [KeywordAbility::Vigilance, KeywordAbility::Lifelink]
                            .into_iter()
                            .collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // Equip {5}: attach this Equipment to target creature you control.
            // CR 702.6b/d: Equip is a sorcery-speed activated ability.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 5, ..Default::default() }),
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
