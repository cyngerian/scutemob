// Teferi, Temporal Archmage — {4}{U}{U}, Legendary Planeswalker — Teferi [5]
// +1: Look at top two cards. Put one in hand, other on bottom.
// -1: Untap up to four target permanents.
// -10: Emblem: "You may activate loyalty abilities at instant speed."
// Teferi, Temporal Archmage can be your commander.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teferi-temporal-archmage"),
        name: "Teferi, Temporal Archmage".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Planeswalker], &["Teferi"]),
        oracle_text: "+1: Look at the top two cards of your library. Put one of them into your hand and the other on the bottom of your library.\n\u{2212}1: Untap up to four target permanents.\n\u{2212}10: You get an emblem with \"You may activate loyalty abilities of planeswalkers you control any time you could cast an instant.\"\nTeferi, Temporal Archmage can be your commander.".to_string(),
        starting_loyalty: Some(5),
        abilities: vec![
            // +1: Look at top 2, one to hand, one to bottom.
            // TODO: RevealAndRoute reveals all; "look" is private. Using RevealAndRoute
            // with count=2 and any-card filter as approximation.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::RevealAndRoute {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                    filter: TargetFilter::default(),
                    matched_dest: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    unmatched_dest: ZoneTarget::Library {
                        owner: PlayerTarget::Controller,
                        position: LibraryPosition::Bottom,
                    },
                },
                targets: vec![],
            },
            // -1: Untap up to 4 permanents.
            // TODO: "up to four" variable targets. Using one target as approximation.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(1),
                effect: Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetPermanent],
            },
            // -10: Emblem (instant-speed loyalty abilities).
            // TODO: Emblem creation for "activate loyalty at instant speed" not in DSL.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(10),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // "Can be your commander" — inherent, no ability definition needed.
        ],
        ..Default::default()
    }
}
