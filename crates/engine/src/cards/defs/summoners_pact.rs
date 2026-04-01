// Summoner's Pact — {0}, Instant
// Search your library for a green creature card, reveal it, put it into your hand, then shuffle.
// At the beginning of your next upkeep, pay {2}{G}{G}. If you don't, you lose the game.
//
// TODO: "At the beginning of your next upkeep, pay {2}{G}{G} or lose the game" — delayed trigger
//   with a pact payment requirement. No DelayedTrigger DSL primitive exists, and no
//   Effect::LoseGameUnlessPay variant exists. The search effect is implemented; the pact
//   trigger is omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("summoners-pact"),
        name: "Summoner's Pact".to_string(),
        mana_cost: Some(ManaCost::default()),
        types: types(&[CardType::Instant]),
        oracle_text: "Search your library for a green creature card, reveal it, put it into your hand, then shuffle.\nAt the beginning of your next upkeep, pay {2}{G}{G}. If you don't, you lose the game.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::SearchLibrary {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        colors: Some([Color::Green].into_iter().collect()),
                        ..Default::default()
                    },
                    destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    reveal: true,
                    player: PlayerTarget::Controller,
                    also_search_graveyard: false,
                    shuffle_before_placing: false,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
            // TODO: Pact upkeep trigger — "At the beginning of your next upkeep, pay {2}{G}{G}
            //   or you lose the game." DSL gap: no DelayedTrigger variant, no Effect::LoseGameUnlessPay.
        ],
        ..Default::default()
    }
}
