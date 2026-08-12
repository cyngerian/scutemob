// Sword of the Paruns — {4}, Artifact — Equipment
// As long as equipped creature is tapped, tapped creatures you control get +2/+0.
// As long as equipped creature is untapped, untapped creatures you control get +0/+2.
// {3}: You may tap or untap equipped creature.
// Equip {3}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-the-paruns"),
        name: "Sword of the Paruns".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "As long as equipped creature is tapped, tapped creatures you control get \
                      +2/+0.\nAs long as equipped creature is untapped, untapped creatures you \
                      control get +0/+2.\n{3}: You may tap or untap equipped creature.\nEquip {3}"
            .to_string(),
        abilities: vec![
            // TODO: DSL gap — conditional statics based on tapped state of equipped creature,
            // affecting tapped/untapped subsets of your creatures. Needs:
            // Condition::EquippedCreatureIsTapped + EffectFilter::TappedCreaturesYouControl.
            // TODO: "{3}: You may tap or untap equipped creature." is separately unauthored.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // Equip {3}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
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
                // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {3}" means "[Cost]: Attach this
                // permanent to target creature you control." Printed line MCP-verified as
                // plain "Equip {3}" with no CR 702.6c quality restriction, so the requirement
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
            "DSL gap — conditional statics based on tapped state of equipped creature, affecting \
             tapped/untapped subsets of your creatures (Condition::EquippedCreatureIsTapped + \
             EffectFilter::TappedCreaturesYouControl, neither of which exist yet), and the \
             separate '{3}: You may tap or untap equipped creature.' ability is unauthored. Equip \
             {3} is now authored as an Activated/AttachEquipment ability.",
        ),
        ..Default::default()
    }
}
