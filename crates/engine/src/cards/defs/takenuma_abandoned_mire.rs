// Takenuma, Abandoned Mire — Legendary Land, {T}: Add {B}; Channel discard ability (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("takenuma-abandoned-mire"),
        name: "Takenuma, Abandoned Mire".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {B}.\nChannel — {3}{B}, Discard this card: Mill three cards, then return a creature or planeswalker card from your graveyard to your hand. This ability costs {1} less to activate for each legendary creature you control.".to_string(),
        abilities: vec![
            // {T}: Add {B}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
            },
            // TODO: Channel — {3}{B}, Discard this card: Mill three cards, then return a creature
            // or planeswalker card from your graveyard to your hand.
            // DSL gaps: Channel discard cost not expressible; return_from_graveyard with
            // multi-type filter (creature or planeswalker) not in Effect enum;
            // variable cost reduction based on legendary creature count not expressible.
        ],
        ..Default::default()
    }
}
