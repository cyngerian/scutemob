// Diabolic Intent — {1}{B}, Sorcery; sacrifice a creature as additional cost,
// search your library for a card, put it into your hand, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("diabolic-intent"),
        name: "Diabolic Intent".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text:
            "As an additional cost to cast this spell, sacrifice a creature.\n\
             Search your library for a card, put that card into your hand, then shuffle."
                .to_string(),
        spell_additional_costs: vec![SpellAdditionalCost::SacrificeCreature],
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter::default(),
                    reveal: false,
                    destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle { player: PlayerTarget::Controller },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
