// Umbral Mantle — {3}, Artifact — Equipment
// Equipped creature has "{3}, {Q}: This creature gets +2/+2 until end of turn."
// Equip {0}
//
// Partially unblocked by PB-S: the grant uses AddActivatedAbility with
//   EffectFilter::AttachedCreature. Still blocked on:
//   (1) {Q} (untap symbol) — no Cost variant carries an untap-self requirement
//   (2) self-pump effect ("this creature gets +2/+2 until EOT") is expressible
//       but needs the {Q} cost to be complete
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("umbral-mantle"),
        name: "Umbral Mantle".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature has \"{3}, {Q}: This creature gets +2/+2 until end of \
                      turn.\"\nEquip {0}"
            .to_string(),
        abilities: vec![
            // TODO: grant "3, {Q}: gets +2/+2 until EOT" to equipped creature via
            //   LayerModification::AddActivatedAbility + EffectFilter::AttachedCreature.
            //   Blocked on {Q} (untap symbol) — the Cost enum needs an untap-self requirement.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // Equip {0}: attach this Equipment to target creature you control. A {0} cost is
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost::default()),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {0}" means "[Cost]: Attach this
                // permanent to target creature you control." Printed line MCP-verified as
                // plain "Equip {0}" with no CR 702.6c quality restriction, so the requirement
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
            "grant '3, {Q}: gets +2/+2 until EOT' to equipped creature via \
             LayerModification::AddActivatedAbility + EffectFilter::AttachedCreature; blocked on \
             {Q} (untap symbol) — no Cost variant carries an untap-self requirement (re-checked \
             against the current enum 2026-08-11). Equip {0} is now authored as an \
             Activated/AttachEquipment ability.",
        ),
        ..Default::default()
    }
}
