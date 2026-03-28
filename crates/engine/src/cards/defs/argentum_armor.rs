// Argentum Armor — {6} Artifact — Equipment
// Equipped creature gets +6/+6.
// Whenever equipped creature attacks, destroy target permanent.
// Equip {6}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("argentum-armor"),
        name: "Argentum Armor".to_string(),
        mana_cost: Some(ManaCost { generic: 6, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text:
            "Equipped creature gets +6/+6.\nWhenever equipped creature attacks, destroy target permanent.\nEquip {6}"
                .to_string(),
        abilities: vec![
            // CR 613.1f: Static ability — equipped creature gets +6/+6 (layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(6),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: "Whenever equipped creature attacks, destroy target permanent."
            // DSL gap: TriggerCondition::WhenAttacks is self-referential to the source
            // creature/permanent. Equipment is not a creature and cannot attack, so there
            // is no WhenEquippedCreatureAttacks trigger condition. The attack trigger on an
            // attached equipment requires a non-self-referential attack trigger that is not
            // yet in the DSL.

            // Equip {6}: attach this Equipment to target creature you control (sorcery speed).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 6, ..Default::default() }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
