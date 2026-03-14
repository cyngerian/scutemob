// Hope of Ghirapur — {1}, Legendary Artifact Creature — Thopter 1/1
// Flying
// Sacrifice Hope of Ghirapur: Until your next turn, target player who was dealt combat damage
// by Hope of Ghirapur this turn can't cast noncreature spells.
//
// Flying is implemented. The sacrifice ability is a TODO.
//
// TODO: Sacrifice: target player dealt combat damage can't cast noncreature — PB-5+PB-13
// Cost::SacrificeSelf available; blocked on combat-damage tracking + cast restriction effect
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hope-of-ghirapur"),
        name: "Hope of Ghirapur".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact, CardType::Creature],
            &["Thopter"],
        ),
        oracle_text: "Flying\nSacrifice Hope of Ghirapur: Until your next turn, target player who was dealt combat damage by Hope of Ghirapur this turn can't cast noncreature spells.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        ..Default::default()
    }
}
