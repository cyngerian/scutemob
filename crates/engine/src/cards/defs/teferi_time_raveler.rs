// Teferi, Time Raveler — {1}{W}{U}, Legendary Planeswalker — Teferi
// Each opponent can cast spells only any time they could cast a sorcery.
// +1: Until your next turn, you may cast sorcery spells as though they had flash.
// −3: Return up to one target artifact, creature, or enchantment to its owner's hand. Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teferi-time-raveler"),
        name: "Teferi, Time Raveler".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Teferi"],
        ),
        oracle_text: "Each opponent can cast spells only any time they could cast a sorcery.\n+1: Until your next turn, you may cast sorcery spells as though they had flash.\n\u{2212}3: Return up to one target artifact, creature, or enchantment to its owner's hand. Draw a card.".to_string(),
        starting_loyalty: Some(4),
        abilities: vec![
            // TODO: "Each opponent can cast spells only any time they could cast a sorcery"
            //   — stax restriction not expressible in DSL.
            // TODO: +1 flash-for-sorceries not expressible.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // −3: Bounce + draw
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(3),
                effect: Effect::Sequence(vec![
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Hand {
                            owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })),
                        },
                        controller_override: None,
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                targets: vec![TargetRequirement::TargetPermanent],
            },
        ],
        ..Default::default()
    }
}
