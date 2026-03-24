// Sword of Fire and Ice — {3} Artifact — Equipment
// Equipped creature gets +2/+2 and has protection from red and from blue.
// Whenever equipped creature deals combat damage to a player, this Equipment deals
// 2 damage to any target and you draw a card.
// Equip {2}
use crate::cards::helpers::*;
use crate::state::types::ProtectionQuality;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-fire-and-ice"),
        name: "Sword of Fire and Ice".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +2/+2 and has protection from red and from blue.\nWhenever equipped creature deals combat damage to a player, Sword of Fire and Ice deals 2 damage to any target and you draw a card.\nEquip {2}".to_string(),
        abilities: vec![
            // +2/+2 static buff to equipped creature
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Protection from red and blue
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
                         KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Blue))]
                            .into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: "Whenever equipped creature deals combat damage to a player" trigger —
            // WhenEquippedCreatureDealsCombatDamage not in DSL.
            // Equip {2}
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
