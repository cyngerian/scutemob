// Chaos Warp — {2}{R} Instant
// The owner of target permanent shuffles it into their library, then reveals the
// top card of their library. If it's a permanent card, they put it onto the
// battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chaos-warp"),
        name: "Chaos Warp".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "The owner of target permanent shuffles it into their library, then reveals the top card of their library. If it's a permanent card, they put it onto the battlefield.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // Step 1: Move the target permanent into its owner's library.
                Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Library {
                        owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget {
                            index: 0,
                        })),
                        position: LibraryPosition::Top,
                    },
                    controller_override: None,
                },
                // Step 1b: Explicitly shuffle (ShuffledIn position hint is not
                // implemented in the MoveZone handler).
                Effect::Shuffle {
                    player: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget {
                        index: 0,
                    })),
                },
                // Step 2: Reveal top 1 card of the owner's library. If it's a permanent
                // card, put it onto the battlefield; otherwise leave it on top.
                // Uses has_card_types for the permanent-card check (CR 110.4a:
                // artifact, creature, enchantment, land, planeswalker, battle).
                Effect::RevealAndRoute {
                    player: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget {
                        index: 0,
                    })),
                    count: EffectAmount::Fixed(1),
                    filter: TargetFilter {
                        has_card_types: vec![
                            CardType::Artifact,
                            CardType::Creature,
                            CardType::Enchantment,
                            CardType::Land,
                            CardType::Planeswalker,
                        ],
                        ..Default::default()
                    },
                    matched_dest: ZoneTarget::Battlefield { tapped: false },
                    unmatched_dest: ZoneTarget::Library {
                        owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget {
                            index: 0,
                        })),
                        position: LibraryPosition::Top,
                    },
                },
            ]),
            targets: vec![TargetRequirement::TargetPermanent],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
