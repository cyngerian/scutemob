// Blade of the Bloodchief — {1}, Artifact — Equipment
// Whenever a creature dies, put a +1/+1 counter on equipped creature. If equipped creature
// is a Vampire, put two +1/+1 counters on it instead.
// Equip {1}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blade-of-the-bloodchief"),
        name: "Blade of the Bloodchief".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Whenever a creature dies, put a +1/+1 counter on equipped creature. If \
                      equipped creature is a Vampire, put two +1/+1 counters on it \
                      instead.\nEquip {1}"
            .to_string(),
        abilities: vec![
            // RE-VERIFIED 2026-08-11 (PB-DX26 fix cycle, review Finding 4):
            // `EffectTarget::EquippedCreature` DOES exist (`card_definition.rs`) — the old
            // TODO said it did not, and this def's own `completeness` note has contradicted
            // it since before this batch. `TriggerCondition::WheneverCreatureDies` exists too.
            // TODO: still genuinely blocked — the conditional count ("two +1/+1 counters
            //   INSTEAD if equipped creature is a Vampire") needs a `Condition` testing the
            //   equipped creature's subtype, which does not exist (re-checked 2026-08-11).
            AbilityDefinition::Keyword(KeywordAbility::Equip),
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
        completeness: Completeness::partial(
            "Blocked on a Condition testing the equipped creature's subtype ('two +1/+1 counters \
             instead if equipped creature is a Vampire'). EffectTarget::EquippedCreature and \
             WheneverCreatureDies both exist; Equip {1} is now authored as an \
             Activated/AttachEquipment ability (see skullclamp.rs).",
        ),
        ..Default::default()
    }
}
