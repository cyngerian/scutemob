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
            // PB-DX27 (stale blocker closed): the CR 702.6c "Equip legendary creature {3}"
            // variant IS representable — CR 702.6c makes "Equip [quality] [cost]" a SEPARATE
            // activated ability from the plain Equip line (the card prints two Equip
            // abilities, not one ability with two costs), so `AbilityDefinition::Activated`
            // carrying only one cost is not a blocker; `TargetFilter.legendary` supplies the
            // 702.6c target restriction (patriars_seal.rs / eiganjo_seat_of_the_empire.rs
            // precedent). Authored below, AFTER the existing plain Equip {7} ability so its
            // `ability_index` does not move (CR 702.6c ordering hazard: OOS-DX26-3).
            //
            // TODO: still genuinely unauthored — the dynamic "+1/+1 for each land you
            //   control" static. The DSL SHAPE exists (Static + ModifyBothDynamic +
            //   PermanentCount + AttachedCreature), but `resolve_cda_amount`'s `controller`
            //   parameter (layers.rs:1861-1867 -> :2482-2487) is derived from the MODIFIED
            //   object (the equipped creature via `EffectFilter::AttachedCreature`), not
            //   from the ability's own source (this Equipment). Per CR 108.5/611.2c "you" in
            //   a static ability's text means the controller of the object the ability is
            //   printed on (the Equipment), which can diverge from the equipped creature's
            //   controller after an independent control-change effect (Equip's CR 702.6a
            //   "target creature you control" restriction only holds at attach time, not
            //   continuously). Authoring this now would silently compute "lands the
            //   CREATURE's controller controls" instead of "lands the EQUIPMENT's controller
            //   controls" in that edge case — a KnownWrong, not a Complete. Two sibling defs
            //   hit the identical open question and stayed non-Complete rather than ship it:
            //   crown_of_skemfar.rs and empyrial_plate.rs. This is an engine-level
            //   attribution gap (layers.rs), out of scope for a card-defs-only batch — left
            //   unauthored here for the same reason, matching precedent.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // Plain Equip {7}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
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
            // CR 702.6c (PB-DX27): "Equip legendary creature {3}" — a SEPARATE activated
            // ability from the plain Equip {7} above, with its own cost and an additional
            // target restriction (only a legendary creature you control is a legal target).
            // Authored AFTER the plain Equip {7} ability so ability_index 0/1 above are
            // unchanged (Command::ActivateAbility indexes in declaration order — OOS-DX26-3).
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
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    legendary: true,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::partial(
            "PB-DX27: the CR 702.6c 'Equip legendary creature {3}' variant is now authored as a \
             second Activated/AttachEquipment ability (TargetFilter.legendary), AFTER the plain \
             Equip {7} so no existing ability_index moved. Remaining, genuine blocker: the \
             dynamic '+1/+1 for each land you control' static is still unauthored — not because \
             the DSL shape is missing (Static + ModifyBothDynamic + PermanentCount + \
             AttachedCreature all exist), but because resolve_cda_amount's controller resolves \
             via the MODIFIED object (the equipped creature), not this Equipment's own controller \
             (layers.rs:1861-1867/2482-2487) -- CR 108.5/611.2c wrong whenever the two \
             controllers diverge post-attach. Same open question as crown_of_skemfar.rs / \
             empyrial_plate.rs; an engine fix, out of scope here.",
        ),
        ..Default::default()
    }
}
