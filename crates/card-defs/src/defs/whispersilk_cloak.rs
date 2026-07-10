// 47. Whispersilk Cloak — {3}, Artifact — Equipment; Equipped creature has
// shroud and can't be blocked. Equip {2}.
// CR 702.6a: Equipment static ability grants keyword to equipped creature.
// CR 604.2: Static ability functions while on the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("whispersilk-cloak"),
        name: "Whispersilk Cloak".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature can't be blocked and has shroud. (It can't be the target of spells or abilities.)\nEquip {2}".to_string(),
        abilities: vec![
            // CR 702.6a: Equipped creature has Shroud (layer 6 ability grant).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Shroud),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 509.1b: Equipped creature can't be blocked.
            // Grants CantBeBlocked in layer 6 (ability) while this Equipment is attached.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::CantBeBlocked),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Equip {2}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; sorcery speed (CR 702.6d).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 2, ..Default::default() }), // Equip {2}
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
