// Geier Reach Sanitarium — Legendary Land, {T}: Add {C}. {2},{T}: Each player draws then discards (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("geier-reach-sanitarium"),
        name: "Geier Reach Sanitarium".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Legendary], &[CardType::Land], &[]),
        oracle_text: "{T}: Add {C}.\n{2}, {T}: Each player draws a card, then discards a card.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: {2},{T}: Each player draws a card then discards a card — ForEach EachPlayer with Sequence draw+discard not in DSL
        ],
        ..Default::default()
    }
}
