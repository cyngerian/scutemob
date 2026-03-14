// Exotic Orchard — Land; {T}: Add one mana of any color that a land an opponent
// controls could produce.
// TODO: DSL gap — AddManaAnyColor does not restrict to colors opponents' lands
// could produce. Simplified to tap for any color (always available). Full
// opponent-land check requires a new Effect variant or runtime query.
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
        }],
        ..Default::default()
    }
}
