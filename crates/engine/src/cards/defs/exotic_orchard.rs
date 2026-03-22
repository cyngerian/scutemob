// Exotic Orchard — Land; {T}: Add one mana of any color that a land an opponent
// controls could produce.
// Note: Simplified — AddManaAnyColor produces any color unconditionally.
// Full opponent-land color query would require a new Effect variant or runtime check.
// TODO: Restrict to colors opponents' lands could produce (opponent-land mana query DSL gap).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("exotic-orchard"),
        name: "Exotic Orchard".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add one mana of any color that a land an opponent controls could produce.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
            timing_restriction: None,
            targets: vec![],
                activation_condition: None,
        }],
        ..Default::default()
    }
}
