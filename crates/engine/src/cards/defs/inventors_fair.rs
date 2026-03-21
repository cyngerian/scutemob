// Inventors' Fair — Legendary Land
// {T}: Add {C}.
// At the beginning of your upkeep, if you control three or more artifacts, you gain 1 life.
// {4}, {T}, Sacrifice: Search for artifact card, reveal, put into hand, shuffle.
//   Activate only if you control three or more artifacts.
// TODO: Upkeep life gain trigger with intervening-if "control 3+ artifacts" (count_threshold gap)
// TODO: Activation condition "only if you control three or more artifacts" (PB-18 stax/restrictions)
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
                activation_condition: None,
            },
            // TODO: At the beginning of your upkeep, if you control 3+ artifacts, gain 1 life.
            // {4}, {T}, Sacrifice: Search for artifact, reveal, put into hand, shuffle.
            // TODO: Missing activation condition "only if you control 3+ artifacts"
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 4,
                        ..Default::default()
                    }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Artifact),
                            ..Default::default()
                        },
                        reveal: true,
                        destination: ZoneTarget::Hand {
                            owner: PlayerTarget::Controller,
                        },
                        shuffle_before_placing: false,
                    },
                    Effect::Shuffle {
                        player: PlayerTarget::Controller,
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
