// Skullclamp — {1}, Artifact — Equipment
// Equipped creature gets +1/-1.
// Whenever equipped creature dies, draw two cards.
// Equip {1}
//
// CR 702.6a: Equipment static ability modifies equipped creature.
// CR 613.1d / 613.4c: +1 power (layer 7c) and -1 toughness (layer 7c) applied
//   via two separate ModifyPower / ModifyToughness static abilities (no combined
//   +N/-M variant exists in the DSL).
// CR 702.6b / 702.6d: Equip is a sorcery-speed activated ability.
//
// TODO: DSL gap — "Whenever equipped creature dies, draw two cards" cannot be
//   expressed faithfully. `TriggerCondition::WheneverCreatureDies` fires on ANY
//   creature's death, not specifically the creature this Equipment is attached to.
//   `TriggerCondition::WhenDies` fires when this object (the Equipment) dies, not
//   its host. A new variant such as `WheneverEquippedCreatureDies` is required to
//   implement this trigger correctly. The trigger is omitted to avoid incorrect
//   behavior (firing on unrelated creature deaths).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skullclamp"),
        name: "Skullclamp".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +1/-1.\nWhenever equipped creature dies, draw two cards.\nEquip {1}".to_string(),
        abilities: vec![
            // Static: Equipped creature gets +1 power (layer 7c, CR 613.4c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Static: Equipped creature gets -1 toughness (layer 7c, CR 613.4c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(-1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: Triggered ability omitted — DSL cannot express "whenever the
            // equipped creature (specifically) dies". See comment at top of file.
            // Effect: draw two cards (PlayerTarget::Controller, EffectAmount::Fixed(2)).

            // Equip {1}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; sorcery speed (CR 702.6d).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
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
