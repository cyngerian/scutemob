// Natural Order — {2}{G}{G}, Sorcery
// As an additional cost to cast this spell, sacrifice a green creature.
// Search your library for a green creature card, put it onto the battlefield, then shuffle.
//
// Note: SpellAdditionalCost has no "SacrificeGreenCreature" variant combining both color
// and type. SacrificeColorPermanent(Color::Green) is used as the closest approximation —
// it requires sacrificing a green permanent (any type), not specifically a green creature.
// TODO: Add SpellAdditionalCost::SacrificeColorCreature(Color) for precision.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("natural-order"),
        name: "Natural Order".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a green creature.\nSearch your library for a green creature card, put it onto the battlefield, then shuffle.".to_string(),
        // CR 601.2h: "As an additional cost to cast this spell, sacrifice a green creature."
        // Approximated as SacrificeColorPermanent(Green) — no SacrificeGreenCreature variant.
        spell_additional_costs: vec![SpellAdditionalCost::SacrificeColorPermanent(Color::Green)],
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // CR 701.23: Search your library for a green creature card.
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        colors: Some([Color::Green].iter().copied().collect()),
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: false },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                // CR 701.23: then shuffle.
                Effect::Shuffle { player: PlayerTarget::Controller },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
