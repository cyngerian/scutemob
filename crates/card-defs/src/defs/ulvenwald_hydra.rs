// Ulvenwald Hydra — {4}{G}{G}, Creature — Hydra */*
// Reach
// Ulvenwald Hydra's power and toughness are each equal to the number of lands you control.
// When this creature enters, you may search your library for a land card, put it onto the
// battlefield tapped, then shuffle.
//
// CDA (*/*): power: None, toughness: None per KI-4.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ulvenwald-hydra"),
        name: "Ulvenwald Hydra".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 2,
            ..Default::default()
        }),
        types: creature_types(&["Hydra"]),
        oracle_text: "Reach\nUlvenwald Hydra's power and toughness are each equal to the number \
                      of lands you control.\nWhen this creature enters, you may search your \
                      library for a land card, put it onto the battlefield tapped, then shuffle."
            .to_string(),
        // */* CDA — engine handles via characteristic-defining ability
        power: None,
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Reach),
            // CR 604.3, 613.4a: CDA — P/T each equal to the number of lands you control.
            AbilityDefinition::CdaPowerToughness {
                power: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                },
                toughness: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                },
            },
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    // "may search" — modeled as unconditional (deterministic fallback).
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
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "'may search' is modeled as an unconditional search — the controller cannot decline",
        ),
        ..Default::default()
    }
}
