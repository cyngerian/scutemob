// Grim Backwoods — Land, {T}: Add {C}. {2}{B}{G}, {T}, Sacrifice a creature: Draw a card (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grim-backwoods"),
        name: "Grim Backwoods".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{2}{B}{G}, {T}, Sacrifice a creature: Draw a card.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {2}{B}{G}, {T}, Sacrifice a creature: Draw a card.
            // DSL gap: Cost::Sacrifice currently takes TargetFilter but does not support targeting
            // a specific creature as part of the cost (activated_ability_targets gap). Would need
            // Cost::Sequence([Cost::Mana, Cost::Tap, Cost::Sacrifice(creature_filter)]) plus
            // a draw effect.
        ],
        ..Default::default()
    }
}
