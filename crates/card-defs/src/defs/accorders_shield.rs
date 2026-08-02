// Accorder's Shield — {0}, Artifact — Equipment
// Equipped creature gets +0/+3 and has vigilance. (Attacking doesn't cause it to tap.)
// Equip {3}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("accorders-shield"),
        name: "Accorder's Shield".to_string(),
        mana_cost: Some(ManaCost {
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +0/+3 and has vigilance. (Attacking doesn't cause it \
                      to tap.)\nEquip {3} ({3}: Attach to target creature you control. Equip only \
                      as a sorcery.)"
            .to_string(),
        abilities: vec![
            // Static: equipped creature gets +0/+3
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(3),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Static: equipped creature has vigilance
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [KeywordAbility::Vigilance].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Equip {3}
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 3,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // CARDS-1 (OOS-M11-10) / CR 702.6a: "Equip {3}" means "[Cost]: Attach this
                // permanent to target creature you control." The printed line was MCP-verified
                // as plain "Equip {3}" -- no CR 702.6c quality restriction -- so the requirement is
                // the unmodified 702.6a one. Authoring it is what makes the target announceable:
                // an empty `targets` list left `queries::ability_target_requirements` reporting
                // zero slots, so the browser picker never asked and the attach silently fizzled
                // with the cost paid.
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}
