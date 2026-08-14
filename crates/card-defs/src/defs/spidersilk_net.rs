// Spidersilk Net — {0}, Artifact — Equipment
// Equipped creature gets +0/+2 and has reach. (It can block creatures with flying.)
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spidersilk-net"),
        name: "Spidersilk Net".to_string(),
        mana_cost: Some(ManaCost {
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +0/+2 and has reach. (It can block creatures with \
                      flying.)\nEquip {2} ({2}: Attach to target creature you control. Equip only \
                      as a sorcery.)"
            .to_string(),
        abilities: vec![
            // Static: equipped creature gets +0/+2
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Static: equipped creature has reach
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [KeywordAbility::Reach].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Equip {2}
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 2,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // CARDS-1 (OOS-M11-10E) / CR 702.6a: "Equip {2}" means "[Cost]: Attach this
                // permanent to target creature you control." The printed line was MCP-verified
                // as plain "Equip {2}" -- no CR 702.6c quality restriction -- so the requirement is
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
