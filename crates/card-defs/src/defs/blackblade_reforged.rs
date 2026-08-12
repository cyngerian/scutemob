// Blackblade Reforged — {2}, Legendary Artifact — Equipment
// Equipped creature gets +1/+1 for each land you control.
// Equip legendary creature {3}
// Equip {7}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blackblade-reforged"),
        name: "Blackblade Reforged".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &["Equipment"],
        ),
        oracle_text: "Equipped creature gets +1/+1 for each land you control.\nEquip legendary \
                      creature {3}\nEquip {7}"
            .to_string(),
        abilities: vec![
            // RE-VERIFIED 2026-08-11 (PB-DX26 fix cycle, review Finding 4): the dynamic
            // "+1/+1 for each land you control" IS expressible —
            // `LayerModification::ModifyBothDynamic` + `EffectAmount::PermanentCount` both
            // exist. The old TODO claiming `LayerModification` needs an `EffectAmount` it
            // does not have was stale; the clause is unauthored, not unexpressible.
            // TODO: still genuinely blocked — the CR 702.6c variant "Equip legendary
            //   creature {3}" has no DSL representation (`AbilityDefinition::Activated`
            //   carries one cost, and 702.6c restricts the TARGET too). See `OOS-DX26-2`.
            //   The plain "Equip {7}" IS authored below (PB-DX26).
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // Plain Equip {7}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
            // The "Equip legendary creature {3}" CR 702.6c variant cost is NOT modeled here
            // (see completeness note) — only the plain Equip {7} line is authored.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 7,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {7}" means "[Cost]: Attach this
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
            "The dynamic +1/+1-per-land clause is now expressible \
             (LayerModification::ModifyBothDynamic + EffectAmount::PermanentCount + \
             EffectFilter::AttachedCreature) and should be authored. The plain Equip {7} line is \
             now authored as an Activated/AttachEquipment ability (skullclamp.rs is the \
             reference). Remaining blocker: the second 'Equip legendary creature {3}' variant \
             equip cost has no DSL representation (AbilityDefinition::Activated has no \
             per-quality alternate cost).",
        ),
        ..Default::default()
    }
}
