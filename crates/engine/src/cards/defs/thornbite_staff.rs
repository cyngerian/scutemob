// Thornbite Staff — {2}, Kindred Artifact — Shaman Equipment
// Equipped creature has "{2}, {T}: This creature deals 1 damage to any target" and
//   "Whenever a creature dies, untap this creature."
// Whenever a Shaman creature enters, you may attach this Equipment to it.
// Equip {4}
//
// ENGINE-BLOCKED (granted abilities): "Equipped creature has [activated ability] and
// [triggered ability]" requires granting arbitrary activated/triggered abilities to the
// equipped creature via a continuous effect. EffectFilter::AttachedCreature exists, but
// LayerModification::AddActivatedAbility and AddTriggeredAbility do not exist in DSL.
//
// ENGINE-BLOCKED (auto-attach trigger): "Whenever a Shaman creature enters, you may attach
// this Equipment to it" requires an ETB trigger that attaches an Equipment. No
// Effect::AttachEquipment-on-trigger pattern with subtype filtering exists; AttachEquipment
// is only an effect of an activated ability (the equip ability), not a triggered effect.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thornbite-staff"),
        name: "Thornbite Staff".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Shaman", "Equipment"]),
        oracle_text: "Equipped creature has \"{2}, {T}: This creature deals 1 damage to any target\" and \"Whenever a creature dies, untap this creature.\"\nWhenever a Shaman creature enters, you may attach Thornbite Staff to it.\nEquip {4}".to_string(),
        abilities: vec![
            // Equip {4}
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 4, ..Default::default() }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
