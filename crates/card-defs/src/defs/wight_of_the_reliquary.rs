// Wight of the Reliquary — {B}{G}, Creature — Zombie Knight 2/2
// Vigilance
// This creature gets +1/+1 for each creature card in your graveyard.
// {T}, Sacrifice another creature: Search your library for a land card,
// put it onto the battlefield tapped, then shuffle.
//
// TODO: "Sacrifice another creature" cost requires exclude-self semantics that don't exist
// on Cost::Sacrifice(TargetFilter) (it has no "another" / exclude-self variant, and this
// card is itself a creature that would match a bare creature filter, allowing illegal
// self-sacrifice). Same gap documented on vampire_gourmand.rs (Cost::SacrificeAnother does
// not exist); that card sets the project precedent of omitting the ability entirely rather
// than risking the self-sacrifice edge case (W5 policy). Omitted here for the same reason.
// The CDA power/toughness modifier below is unaffected and correctly authored.
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
            // TODO: "{T}, Sacrifice another creature: Search your library for a land card,
            // put it onto the battlefield tapped, then shuffle." — omitted; see file-header
            // comment for the exclude-self sacrifice-cost gap and vampire_gourmand.rs
            // precedent.
        ],
        completeness: Completeness::partial(
            "'Sacrifice another creature' cost requires exclude-self semantics that don't exist \
             on Cost::Sacrifice(TargetFilter) (it...",
        ),
        ..Default::default()
    }
}
