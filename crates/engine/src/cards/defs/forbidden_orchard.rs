// Forbidden Orchard — Land, {T}: Add any color; gives opponent a Spirit token (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("forbidden-orchard"),
        name: "Forbidden Orchard".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add one mana of any color.\nWhenever you tap this land for mana, target opponent creates a 1/1 colorless Spirit creature token.".to_string(),
        abilities: vec![
            // {T}: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // TODO: Whenever you tap this land for mana, target opponent creates a 1/1 colorless
            // Spirit creature token.
            // DSL gap: triggered_trigger fired by tapping for mana (mana ability trigger)
            // not expressible; targeted_trigger (target opponent) not in DSL.
        ],
        ..Default::default()
    }
}
