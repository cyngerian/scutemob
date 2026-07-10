// Grisly Salvage — {B}{G}, Instant
// Reveal the top five cards of your library. You may put a creature or land card
// from among them into your hand. Put the rest into your graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grisly-salvage"),
        name: "Grisly Salvage".to_string(),
        mana_cost: Some(ManaCost { black: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Reveal the top five cards of your library. You may put a creature or land card from among them into your hand. Put the rest into your graveyard.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::RevealAndRoute {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(5),
                filter: TargetFilter {
                    has_card_types: vec![CardType::Creature, CardType::Land],
                    ..Default::default()
                },
                matched_dest: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                unmatched_dest: ZoneTarget::Graveyard { owner: PlayerTarget::Controller },
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
