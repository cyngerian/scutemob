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
            // TODO: DSL gap — dynamic +1/+1 per land you control. LayerModification
            // needs EffectAmount, not fixed i32.
            // TODO: DSL gap — "Equip legendary creature {3}" variant equip cost.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        completeness: Completeness::partial(
            "The dynamic +1/+1-per-land clause is now expressible \
             (LayerModification::ModifyBothDynamic + EffectAmount::PermanentCount + \
             EffectFilter::AttachedCreature) and should be authored, along with Equip {7} as an \
             Activated/AttachEquipment ability (skullclamp.rs is the reference). Remaining \
             blocker: the second 'Equip legendary creature {3}' variant equip cost has no DSL \
             representation.",
        ),
        ..Default::default()
    }
}
