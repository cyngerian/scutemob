// Ornithopter of Paradise — {2}, Artifact Creature — Thopter 0/2
// Flying; {T}: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ornithopter-of-paradise"),
        name: "Ornithopter of Paradise".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Thopter"]),
        oracle_text: "Flying\n{T}: Add one mana of any color.".to_string(),
        power: Some(0),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
