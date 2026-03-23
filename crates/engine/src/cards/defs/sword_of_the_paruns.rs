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
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "As long as equipped creature is tapped, tapped creatures you control get +2/+0.\nAs long as equipped creature is untapped, untapped creatures you control get +0/+2.\n{3}: You may tap or untap equipped creature.\nEquip {3}".to_string(),
        abilities: vec![
            // TODO: DSL gap — conditional statics based on tapped state of equipped creature,
            // affecting tapped/untapped subsets of your creatures. Needs:
            // Condition::EquippedCreatureIsTapped + EffectFilter::TappedCreaturesYouControl.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}
