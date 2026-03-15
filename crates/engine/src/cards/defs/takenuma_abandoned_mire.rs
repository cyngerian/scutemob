// Takenuma, Abandoned Mire — Legendary Land, {T}: Add {B}; Channel — mill + return from GY.
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
                targets: vec![],
            },
            // Channel — {3}{B}, Discard this card: Mill 3, then return creature/planeswalker from GY.
            // Implementing the mill portion; return-from-graveyard is a separate effect.
            // TODO: "return a creature or planeswalker card from your graveyard to your hand" —
            //       requires MoveZone from graveyard with multi-type filter (creature OR planeswalker).
            //       Deterministic fallback would pick first matching card. Using mill-only for now.
            // TODO: Cost reduction — {1} less per legendary creature you control.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, black: 1, ..Default::default() }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::MillCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
