// Commander's Plate — {1}, Artifact — Equipment
// Equipped creature gets +3/+3 and has protection from each color that's not in your
// commander's color identity.
// Equip commander {3}
// Equip {5}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("commanders-plate"),
        name: "Commander's Plate".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +3/+3 and has protection from each color that's not \
                      in your commander's color identity.\nEquip commander {3}\nEquip {5}"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(3),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: DSL gap — dynamic protection from colors not in commander's color identity.
            // TODO: DSL gap — "Equip commander {3}" variant equip cost.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // Plain Equip {5}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
            // The "Equip commander {3}" CR 702.6c variant cost is NOT modeled here (see
            // completeness note) — only the plain Equip {5} line is authored.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 5,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {5}" means "[Cost]: Attach this
                // permanent to target creature you control." Printed line MCP-verified — no
                // color/subtype restriction on the base target, so the requirement is the
                // unmodified 702.6a one.
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
        completeness: Completeness::partial(
            "DSL gap — dynamic protection from colors not in commander's color identity. The \
             plain Equip {5} line is now authored as an Activated/AttachEquipment ability. \
             Remaining blocker: the second 'Equip commander {3}' variant equip cost has no DSL \
             representation (AbilityDefinition::Activated has no per-quality alternate cost).",
        ),
        ..Default::default()
    }
}
