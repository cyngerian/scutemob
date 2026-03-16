// Assassin's Trophy — {B}{G} Instant
// Destroy target nonland permanent. Its controller may search their library for a
// basic land card, put it onto the battlefield, then shuffle.
// CR 701.19: opponent search portion uses ControllerOf(DeclaredTarget) for player target.
// Note: "may search" is modeled as unconditional search (deterministic fallback).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("assassins-trophy"),
        name: "Assassin's Trophy".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target nonland permanent. Its controller may search their library for a basic land card, put it onto the battlefield, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                Effect::SearchLibrary {
                    player: PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget {
                        index: 0,
                    })),
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        basic: true,
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: false },
                },
                Effect::Shuffle {
                    player: PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget {
                        index: 0,
                    })),
                },
            ]),
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                non_land: true,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
