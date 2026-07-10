// 48. Batterskull — {5}, Artifact — Equipment; Living weapon. Equipped creature gets
// +4/+4 and has vigilance and lifelink. {3}: Return this Equipment to its owner's hand.
// Equip {5}.
// CR 702.92a: Living weapon ETB trigger creates 0/0 black Phyrexian Germ, attaches.
// CR 702.6a: Equipment static ability grants keywords to equipped creature.
// The {3} bounce ability allows resetting the Living Weapon by returning to hand.
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
             {3}: Return this Equipment to its owner's hand.\n\
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
                    condition: None,
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
                    condition: None,
                },
            },
            // {3}: Return this Equipment to its owner's hand.
            // Allows resetting Living Weapon by bouncing back to hand.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
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
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
