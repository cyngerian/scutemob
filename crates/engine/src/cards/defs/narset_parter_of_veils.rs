// Narset, Parter of Veils — {1}{U}{U}, Legendary Planeswalker — Narset [5]
// Each opponent can draw no more than one card each turn.
// -2: Look at the top four cards of your library. You may reveal a noncreature,
// nonland card and put it into your hand. Put the rest on the bottom in random order.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("narset-parter-of-veils"),
        name: "Narset, Parter of Veils".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Planeswalker], &["Narset"]),
        oracle_text: "Each opponent can draw no more than one card each turn.\n\u{2212}2: Look at the top four cards of your library. You may reveal a noncreature, nonland card from among them and put it into your hand. Put the rest on the bottom of your library in a random order.".to_string(),
        starting_loyalty: Some(5),
        abilities: vec![
            // TODO: "opponents can't draw more than 1 card each turn" — needs
            // GameRestriction::MaxCardsDrawnPerTurn. Not in DSL.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::RevealAndRoute {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(4),
                    filter: TargetFilter {
                        non_creature: true,
                        non_land: true,
                        ..Default::default()
                    },
                    matched_dest: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    unmatched_dest: ZoneTarget::Library {
                        owner: PlayerTarget::Controller,
                        position: LibraryPosition::Bottom,
                    },
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
