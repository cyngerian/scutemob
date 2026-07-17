// Land Tax — {W} Enchantment.
// "At the beginning of your upkeep, if an opponent controls more lands than you,
// you may search your library for up to three basic land cards, reveal them,
// put them into your hand, then shuffle."
//
// CR 508... (upkeep trigger) with PB-AC6's Condition::OpponentControlsMoreLandsThanYou
// as the intervening-if. "You may search ... up to three" follows the established
// engine convention (see farhaven_elf.rs, dark_petition.rs) of a deterministic
// auto-search: each SearchLibrary call independently finds at most one matching
// basic land (or none, if fewer than three remain), naturally implementing "up to
// three" without a real interactive choice model.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("land-tax"),
        name: "Land Tax".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your upkeep, if an opponent controls more lands than \
                      you, you may search your library for up to three basic land cards, reveal \
                      them, put them into your hand, then shuffle."
            .to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: basic_land_filter(),
                    reveal: true,
                    destination: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: basic_land_filter(),
                    reveal: true,
                    destination: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: basic_land_filter(),
                    reveal: true,
                    destination: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle {
                    player: PlayerTarget::Controller,
                },
            ]),
            intervening_if: Some(Condition::OpponentControlsMoreLandsThanYou),
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}
