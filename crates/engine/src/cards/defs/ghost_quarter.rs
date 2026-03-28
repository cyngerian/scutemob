// Ghost Quarter — Land
// {T}: Add {C}.
// {T}, Sacrifice: Destroy target land. Its controller may search for basic land,
//   put onto battlefield, shuffle.
// CR 701.23: opponent search uses ControllerOf(DeclaredTarget).
// Note: "may search" modeled as unconditional search (deterministic fallback).
use crate::cards::helpers::*;
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ghost-quarter"),
        name: "Ghost Quarter".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Sacrifice this land: Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield, then shuffle.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // {T}, Sacrifice: Destroy target land, its controller searches for basic land.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::SacrificeSelf]),
                effect: Effect::Sequence(vec![
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                    },
                    Effect::SearchLibrary {
                        player: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            basic: true,
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: false },
                        shuffle_before_placing: false,
                    also_search_graveyard: false,
                    },
                    Effect::Shuffle {
                        player: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetLand],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
