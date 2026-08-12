// Empyrial Plate — {2}, Artifact — Equipment
// Equipped creature gets +1/+1 for each card in your hand.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("empyrial-plate"),
        name: "Empyrial Plate".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +1/+1 for each card in your hand.\nEquip {2}"
            .to_string(),
        abilities: vec![
            // TODO: DSL gap — dynamic +1/+1 per card in hand. LayerModification::ModifyBoth
            // takes fixed i32, not EffectAmount. Needs dynamic LayerModification.
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
            "Rewire: AbilityDefinition::Static { ContinuousEffectDef { layer: PtModify, \
             modification: LayerModification::ModifyBothDynamic { amount: \
             Box::new(EffectAmount::HandSize { player: PlayerTarget::Controller }), negate: false \
             }, filter: EffectFilter::AttachedCreature, duration: WhileSourceOnBattlefield } }. \
             Verify first: layers.rs:1270-1275 resolves via the modified object's controller, not \
             the Equipment's — confirm 'your hand' resolves to the Equipment controller under \
             gain-control before marking Complete. Equip {2} is now authored as an \
             Activated/AttachEquipment ability.",
        ),
        ..Default::default()
    }
}
