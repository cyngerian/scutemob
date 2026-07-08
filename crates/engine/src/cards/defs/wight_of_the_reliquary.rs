// Wight of the Reliquary — {B}{G}, Creature — Zombie Knight 2/2
// Vigilance
// This creature gets +1/+1 for each creature card in your graveyard.
// {T}, Sacrifice another creature: Search your library for a land card,
// put it onto the battlefield tapped, then shuffle.
//
// TODO: "Sacrifice another creature" cost requires Cost::SacrificeAnother with creature filter;
// only Cost::SacrificeSelf and Cost::Sacrifice(TargetFilter) exist. The current
// Cost::Sacrifice is ambiguous (could sacrifice self). Implemented below using
// Cost::Sacrifice(TargetFilter { creature: true, ..Default::default() }) as best approximation,
// but this may allow sacrificing self. When Cost::SacrificeAnother is added, update this.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wight-of-the-reliquary"),
        name: "Wight of the Reliquary".to_string(),
        mana_cost: Some(ManaCost { black: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Zombie", "Knight"]),
        oracle_text: "Vigilance\nThis creature gets +1/+1 for each creature card in your graveyard.\n{T}, Sacrifice another creature: Search your library for a land card, put it onto the battlefield tapped, then shuffle.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // CR 611.3a, 613.4c: +1/+1 for each creature card in your graveyard —
            // static Layer 7c modify on top of the base 2/2 (PB-AC3 CdaModifyPowerToughness).
            AbilityDefinition::CdaModifyPowerToughness {
                power: Some(EffectAmount::CardCount {
                    zone: ZoneTarget::Graveyard { owner: PlayerTarget::Controller },
                    player: PlayerTarget::Controller,
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                }),
                toughness: Some(EffectAmount::CardCount {
                    zone: ZoneTarget::Graveyard { owner: PlayerTarget::Controller },
                    player: PlayerTarget::Controller,
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                }),
            },
            // {T}, Sacrifice another creature: Search your library for a land card,
            // put it onto the battlefield tapped, then shuffle.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    // TODO: Should be Cost::SacrificeAnother (creature) not Cost::Sacrifice
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
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
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
