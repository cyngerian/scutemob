// Basilisk Collar — {1}, Artifact — Equipment; equipped creature has deathtouch and lifelink.
// Equip {2}.
// CR 702.6a: Equipment static ability grants keywords to equipped creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("basilisk-collar"),
        name: "Basilisk Collar".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature has deathtouch and lifelink. (Any amount of damage it deals to a creature is enough to destroy it. Damage dealt by this creature also causes you to gain that much life.)\nEquip {2} ({2}: Attach to target creature you control. Equip only as a sorcery.)".to_string(),
        abilities: vec![
            // Static ability: equipped creature has Deathtouch and Lifelink (layer 6).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [KeywordAbility::Deathtouch, KeywordAbility::Lifelink]
                            .into_iter()
                            .collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // Equip {2}: sorcery-speed attach (CR 702.6b/d).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
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
