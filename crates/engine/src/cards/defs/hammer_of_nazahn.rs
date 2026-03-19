// Hammer of Nazahn — {4}, Legendary Artifact — Equipment
// ETB trigger: attach this or another Equipment to target creature you control
// Equipped creature gets +2/+0 and has indestructible; Equip {4}
//
// TODO: "Whenever Hammer of Nazahn or another Equipment you control enters, you may attach
// that Equipment to target creature you control" — requires:
// 1. TriggerCondition::WheneverPermanentEntersBattlefield with a filter for Equipment subtype
//    and controller_you constraint. WheneverPermanentEntersBattlefield EXISTS but
//    TargetFilter.has_subtype filter would need SubType("Equipment") AND controller check.
// 2. Effect::AttachEquipment targeting the ENTERING equipment (not necessarily Source) —
//    requires EffectTarget::TriggeringObject which does not exist. DSL gap.
// 3. The "you may" optional trigger choice is not yet expressible in triggered ability effects.
// Deferred until EffectTarget::TriggeringObject and optional trigger choice are added.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hammer-of-nazahn"),
        name: "Hammer of Nazahn".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &["Equipment"],
        ),
        oracle_text: "Whenever Hammer of Nazahn or another Equipment you control enters, you may attach that Equipment to target creature you control.\nEquipped creature gets +2/+0 and has indestructible.\nEquip {4}".to_string(),
        abilities: vec![
            // TODO: ETB trigger watching for any Equipment entering (self or other you control)
            // and attaching it to target creature — see top-of-file comment for full DSL gap analysis.
            // Blocked on: EffectTarget::TriggeringObject, TargetFilter with Equipment subtype +
            // controller_you, and optional triggered effect choice.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Indestructible),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 4, ..Default::default() }),
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
