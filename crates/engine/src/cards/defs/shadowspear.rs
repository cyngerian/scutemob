// Shadowspear — {1}, Legendary Artifact — Equipment
// Equipped creature gets +1/+1 and has trample and lifelink.
// {1}: Permanents your opponents control lose hexproof and indestructible until end of turn.
// Equip {2}
//
// Static ability granting +1/+1, Trample, and Lifelink to equipped creature is implemented.
// The Equip {2} activated ability is implemented.
//
// TODO: DSL gap — the {1} activated ability removes hexproof and indestructible from all
// permanents opponents control. There is no EffectFilter for "all permanents controlled by
// opponents" combined with an UntilEndOfTurn removal of keywords. This ability is omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shadowspear"),
        name: "Shadowspear".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +1/+1 and has trample and lifelink.\n{1}: Permanents your opponents control lose hexproof and indestructible until end of turn.\nEquip {2}".to_string(),
        abilities: vec![
            // Static: Equipped creature gets +1/+1 (layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // Static: Equipped creature has Trample and Lifelink (layer 6).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [KeywordAbility::Trample, KeywordAbility::Lifelink]
                            .into_iter()
                            .collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // Equip {2}: attach this Equipment to target creature you control (CR 702.6b/d).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
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
