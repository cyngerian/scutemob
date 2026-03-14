// Inventors' Fair — Legendary Land, {T}: Add {C}; upkeep life gain trigger (TODO); sacrifice to tutor (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("inventors-fair"),
        name: "Inventors' Fair".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "At the beginning of your upkeep, if you control three or more artifacts, you gain 1 life.\n{T}: Add {C}.\n{4}, {T}, Sacrifice Inventors' Fair: Search your library for an artifact card, reveal it, put it into your hand, then shuffle. Activate only if you control three or more artifacts.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: At the beginning of your upkeep, if you control three or more artifacts,
            // you gain 1 life.
            // DSL gap: intervening_if condition "control three or more artifacts" (count_threshold)
            // not expressible.

            // TODO: {4}, {T}, Sacrifice: Search for artifact, activate only with 3+ artifacts — PB-17
            // Cost::SacrificeSelf available; blocked on typed search + conditional activation
        ],
        ..Default::default()
    }
}
