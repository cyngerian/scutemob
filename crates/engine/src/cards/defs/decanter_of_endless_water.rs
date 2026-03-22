// Decanter of Endless Water — {3}, Artifact
// You have no maximum hand size.
// {T}: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("decanter-of-endless-water"),
        name: "Decanter of Endless Water".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "You have no maximum hand size.\n{T}: Add one mana of any color.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::NoMaxHandSize),
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
