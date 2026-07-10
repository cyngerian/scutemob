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
            // Passive: "Each opponent can cast spells only any time they could cast a sorcery."
            // CR 307.5 / CR 101.2: restriction overrides permission (beats flash grants).
            AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::OpponentsCanOnlyCastAtSorcerySpeed,
            },
            // +1: "Until your next turn, you may cast sorcery spells as though they had flash."
            // CR 601.3b: PlayerId(0) is resolved to the controller at execution time.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::GrantFlash {
                    filter: FlashGrantFilter::Sorceries,
                    duration: EffectDuration::UntilYourNextTurn(PlayerId(0)),
                },
                targets: vec![],
            },
            // −3: Return up to one target artifact, creature, or enchantment to its owner's
            // hand. Draw a card. (CR 601.2c / 115.1b — draw happens regardless of target count)
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(3),
                effect: Effect::Sequence(vec![
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Hand {
                            owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget {
                                index: 0,
                            })),
                        },
                        controller_override: None,
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                targets: vec![TargetRequirement::UpToN {
                    count: 1,
                    inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        has_card_types: vec![
                            CardType::Artifact,
                            CardType::Creature,
                            CardType::Enchantment,
                        ],
                        ..Default::default()
                    })),
                }],
            },
        ],
        ..Default::default()
    }
}
