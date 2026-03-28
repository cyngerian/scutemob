// Reflecting Pool — Land; {T}: Add one mana of any type that a land you
// control could produce.
// TODO: DSL gap — mana type query on own lands not expressible.
// Simplified to any color (same as Exotic Orchard/Fellwar Stone approach).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reflecting-pool"),
        name: "Reflecting Pool".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add one mana of any type that a land you control could produce.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
            timing_restriction: None,
            targets: vec![],
                activation_condition: None,
                activation_zone: None,
        }],
        ..Default::default()
    }
}
