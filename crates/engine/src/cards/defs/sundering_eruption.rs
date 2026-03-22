// Sundering Eruption // Volcanic Fissure — {2}{R} Sorcery // Land (MDFC)
// Oracle: "Destroy target land. Its controller may search their library for a basic land
// card, put it onto the battlefield tapped, then shuffle. Creatures without flying can't
// block this turn."
// Note: Mass blocking restriction not expressible as spell effect.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sundering-eruption"),
        name: "Sundering Eruption // Volcanic Fissure".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield tapped, then shuffle. Creatures without flying can't block this turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // CR 701.7a: Destroy target land.
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                // Its controller searches for a basic land (enters tapped).
                // "may search" modeled as unconditional (deterministic fallback).
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
                    destination: ZoneTarget::Battlefield { tapped: true },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle {
                    player: PlayerTarget::ControllerOf(Box::new(
                        EffectTarget::DeclaredTarget { index: 0 },
                    )),
                },
                // TODO: "Creatures without flying can't block this turn" — mass blocking
                // restriction not expressible as spell effect.
            ]),
            targets: vec![TargetRequirement::TargetLand],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
