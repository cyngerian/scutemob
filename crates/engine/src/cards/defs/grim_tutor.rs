// Grim Tutor — {1}{B}{B}, Sorcery; search your library for a card,
// put that card into your hand, then shuffle. You lose 3 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grim-tutor"),
        name: "Grim Tutor".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text:
            "Search your library for a card, put that card into your hand, then shuffle. \
             You lose 3 life."
                .to_string(),
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
                Effect::LoseLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(3),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
