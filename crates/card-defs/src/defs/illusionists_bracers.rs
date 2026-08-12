// Illusionist's Bracers — {2}, Artifact — Equipment
// Whenever an ability of equipped creature is activated, if it isn't a mana ability,
// copy that ability. You may choose new targets for the copy.
// Equip {3}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("illusionists-bracers"),
        name: "Illusionist's Bracers".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Whenever an ability of equipped creature is activated, if it isn't a mana \
                      ability, copy that ability. You may choose new targets for the copy.\nEquip \
                      {3}"
        .to_string(),
        abilities: vec![
            // TODO: DSL gap — triggered ability that copies activated abilities of
            // equipped creature. Ability copying not in DSL.
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
            "DSL gap — triggered ability that copies activated abilities of equipped creature. \
             Ability copying not in DSL. Equip {3} is now authored as an \
             Activated/AttachEquipment ability.",
        ),
        ..Default::default()
    }
}
