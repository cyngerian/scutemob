// Paradise Mantle — {0}, Artifact — Equipment
// Equipped creature has "{T}: Add one mana of any color."
// Equip {1}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("paradise-mantle"),
        name: "Paradise Mantle".to_string(),
        mana_cost: Some(ManaCost {
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature has \"{T}: Add one mana of any color.\"\nEquip {1}"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // CR 613.1f: Layer 6 static ability — grants mana ability to the equipped
            // creature only (EffectFilter::AttachedCreature resolves via source.attached_to
            // at characteristic-calculation time). Unequipped = filter matches nothing.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddManaAbility(ManaAbility {
                        produces: Default::default(),
                        requires_tap: true,
                        sacrifice_self: false,
                        any_color: true,
                        damage_to_controller: 0,
                        ..Default::default()
                    }),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Equip {1}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {1}" means "[Cost]: Attach this
                // permanent to target creature you control." Printed line MCP-verified as
                // plain "Equip {1}" with no CR 702.6c quality restriction, so the requirement
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
        ..Default::default()
    }
}
