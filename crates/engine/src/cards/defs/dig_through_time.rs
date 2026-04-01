// Dig Through Time — {6}{U}{U}, Instant
// Delve
// Look at the top seven cards of your library. Put two of them into your hand and
// the rest on the bottom of your library in any order.
//
// TODO: Interactive "choose 2 of 7" — M10 player choice. Approximated as DrawCards(2).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dig-through-time"),
        name: "Dig Through Time".to_string(),
        mana_cost: Some(ManaCost { generic: 6, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Delve (Each card you exile from your graveyard while casting this spell pays for {1}.)\nLook at the top seven cards of your library. Put two of them into your hand and the rest on the bottom of your library in any order.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Delve),
            // TODO: "look at top 7, choose 2" — approximated as DrawCards(2).
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
