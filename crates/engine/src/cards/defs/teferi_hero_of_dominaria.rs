// Teferi, Hero of Dominaria — {3}{W}{U} Legendary Planeswalker — Teferi
// +1: Draw a card. At the beginning of the next end step, untap up to two lands.
// −3: Put target nonland permanent into its owner's library third from the top.
// −8: You get an emblem with "Whenever you draw a card, exile target permanent
//     an opponent controls."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teferi-hero-of-dominaria"),
        name: "Teferi, Hero of Dominaria".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Teferi"],
        ),
        oracle_text: "+1: Draw a card. At the beginning of the next end step, untap up to two lands.\n\u{2212}3: Put target nonland permanent into its owner's library third from the top.\n\u{2212}8: You get an emblem with \"Whenever you draw a card, exile target permanent an opponent controls.\"".to_string(),
        starting_loyalty: Some(4),
        abilities: vec![
            // +1: Draw a card. At the beginning of the next end step, untap up to two lands.
            // TODO: Delayed trigger "at the beginning of the next end step, untap up to two
            // lands" is not expressible in DSL. Implementing the draw only.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
            },
            // −3: Put target nonland permanent into its owner's library third from the top.
            // NOTE: LibraryPosition lacks ThirdFromTop — using Top as approximation.
            // TODO: Add LibraryPosition::NthFromTop(u32) for precise placement.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(3),
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Library {
                        owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })),
                        position: LibraryPosition::Top,
                    },
                    controller_override: None,
                },
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    non_land: true,
                    ..Default::default()
                })],
            },
            // −8: You get an emblem with "Whenever you draw a card, exile target permanent
            //     an opponent controls."
            // TODO: TriggerEvent::WheneverYouDrawCard not in DSL — emblem trigger gap.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(8),
                effect: Effect::CreateEmblem {
                    triggered_abilities: vec![],
                    static_effects: vec![],
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
