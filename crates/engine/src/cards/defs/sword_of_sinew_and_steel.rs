// Sword of Sinew and Steel — {3}, Artifact — Equipment
// Equipped creature gets +2/+2 and has protection from black and from red.
// Whenever equipped creature deals combat damage to a player, destroy up to one target
// planeswalker and up to one target artifact.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-sinew-and-steel"),
        name: "Sword of Sinew and Steel".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +2/+2 and has protection from black and from red.\nWhenever equipped creature deals combat damage to a player, destroy up to one target planeswalker and up to one target artifact.\nEquip {2}".to_string(),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Black))),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red))),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 510.3a / CR 601.2c / 115.1b: "Whenever equipped creature deals combat
            // damage to a player, destroy up to one target planeswalker and up to one
            // target artifact." Two parallel UpToN slots (different inner types).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                        cant_be_regenerated: false,
                    },
                ]),
                intervening_if: None,
                targets: vec![
                    TargetRequirement::UpToN {
                        count: 1,
                        inner: Box::new(TargetRequirement::TargetPlaneswalker),
                    },
                    TargetRequirement::UpToN {
                        count: 1,
                        inner: Box::new(TargetRequirement::TargetArtifact),
                    },
                ],
                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}
