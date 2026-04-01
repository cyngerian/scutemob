// Stock Up — {3}{U}, Instant
// Look at the top five cards of your library. Put two of them into your hand
// and the rest on the bottom in a random order.
//
// TODO: DSL gap — "look at top N, choose M to put in hand, rest on bottom" requires
// interactive player choice (select from top-5) which is deferred to M10
// (Command::SelectLibraryCard). Approximated as DrawCards(2) + TODO note.
// The "bottom in random order" is also not expressible in current ZoneTarget variants.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stock-up"),
        name: "Stock Up".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Look at the top five cards of your library. Put two of them into your hand and the rest on the bottom in a random order.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "look at top 5, choose 2 to hand, rest on bottom" requires interactive
            // library-top selection (M10). Approximated as DrawCards(2) which draws from top
            // without the selection step. Upgrade when Command::SelectLibraryCard is available.
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
