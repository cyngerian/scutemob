// Vampiric Tutor — {B}, Instant: search library for any card, shuffle, put it on top; lose 2 life
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vampiric-tutor"),
        name: "Vampiric Tutor".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Search your library for a card, then shuffle and put that card on top. You lose 2 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter::default(),
                    reveal: false,
                    destination: ZoneTarget::Library {
                        owner: PlayerTarget::Controller,
                        position: LibraryPosition::Top,
                    },
                },
                Effect::Shuffle { player: PlayerTarget::Controller },
                Effect::LoseLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
