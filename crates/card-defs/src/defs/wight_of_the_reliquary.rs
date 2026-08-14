// Wight of the Reliquary — {B}{G}, Creature — Zombie Knight 2/2
// Vigilance
// This creature gets +1/+1 for each creature card in your graveyard.
// {T}, Sacrifice another creature: Search your library for a land card,
// put it onto the battlefield tapped, then shuffle.
//
// PB-DX27 (stale blocker note, closed): "Sacrifice another creature" IS expressible.
// Cost::SacrificeAnother does not exist, but it is not needed — TargetFilter.exclude_self
// (CR 109.1) is lowered onto the activation cost by flatten_cost_into and enforced in
// handle_activate_ability (rules/abilities.rs), exactly as on yahenni_undying_partisan.rs
// and razaketh_the_foulblooded.rs. {T} + Sacrifice compose via Cost::Sequence.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wight-of-the-reliquary"),
        name: "Wight of the Reliquary".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Zombie", "Knight"]),
        oracle_text: "Vigilance\nThis creature gets +1/+1 for each creature card in your \
                      graveyard.\n{T}, Sacrifice another creature: Search your library for a land \
                      card, put it onto the battlefield tapped, then shuffle."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // CR 611.3a, 613.4c: +1/+1 for each creature card in your graveyard —
            // static Layer 7c modify on top of the base 2/2 (PB-AC3 CdaModifyPowerToughness).
            AbilityDefinition::CdaModifyPowerToughness {
                power: Some(EffectAmount::CardCount {
                    zone: ZoneTarget::Graveyard {
                        owner: PlayerTarget::Controller,
                    },
                    player: PlayerTarget::Controller,
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                }),
                toughness: Some(EffectAmount::CardCount {
                    zone: ZoneTarget::Graveyard {
                        owner: PlayerTarget::Controller,
                    },
                    player: PlayerTarget::Controller,
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                }),
            },
            // CR 109.1/602.2 (PB-DX27): "{T}, Sacrifice another creature: Search your
            // library for a land card, put it onto the battlefield tapped, then shuffle."
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        exclude_self: true,
                        ..Default::default()
                    }),
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle {
                        player: PlayerTarget::Controller,
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
