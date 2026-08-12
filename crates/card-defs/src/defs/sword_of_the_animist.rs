// Sword of the Animist — {2}, Legendary Artifact — Equipment
// Equipped creature gets +1/+1.
// Whenever equipped creature attacks, you may search your library for a basic land card,
// put it onto the battlefield tapped, then shuffle.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-the-animist"),
        name: "Sword of the Animist".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &["Equipment"],
        ),
        oracle_text: "Equipped creature gets +1/+1.\nWhenever equipped creature attacks, you may \
                      search your library for a basic land card, put it onto the battlefield \
                      tapped, then shuffle.\nEquip {2}"
            .to_string(),
        abilities: vec![
            // Equipped creature gets +1/+1 (layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: DSL gap — "Whenever equipped creature attacks" trigger condition
            // (WhenEquippedCreatureAttacks) does not exist. WhenAttacks is self-only.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // Equip {2}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
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
                // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {2}" means "[Cost]: Attach this
                // permanent to target creature you control." Printed line MCP-verified as
                // plain "Equip {2}" with no CR 702.6c quality restriction, so the requirement
                // is the unmodified 702.6a one.
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
            "DSL gap — 'Whenever equipped creature attacks' trigger condition \
             (WhenEquippedCreatureAttacks) does not exist. Equip {2} is now authored as an \
             Activated/AttachEquipment ability.",
        ),
        ..Default::default()
    }
}
